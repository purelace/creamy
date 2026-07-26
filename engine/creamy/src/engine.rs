use alloc::boxed::Box;
use core::num::NonZeroU8;

use creamy_engine_core::{
    Constants, PluginLoader, WasmModule, WasmRuntime,
    bus::{MessageBus, SubscriberLookupData, config::BusConfig, define_bus_config},
    devkit::{
        BinaryPlugin,
        manifest::{Manifest, RequestedProtocol},
        semver::Version,
        utils::strpool::StringPool,
        xmlc::{ProtocolDefinition, StringPoolResolver},
    },
};
use creamy_sdk::bus::{
    Subscriber, SubscriberId,
    buffer::{IncBuf, OutBuf, SharedBuf},
};
use rustc_hash::FxHashMap;

use crate::{driver::EngineBusDriver, registry::ProtocolRegistry, system_plugin::SystemPlugin};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Protocol model `{target_model}@{version}` not found")]
    ProtocolModelNotFound {
        target_model: Box<str>,
        version: Version,
    },
    #[error("Protocol `{target_model}@{version}` has already been declared")]
    ProtocolDeclaredAlready {
        target_model: Box<str>,
        version: Version,
    },
    #[error(
        "Protocols `{target_model}@{version_a}` and `{target_model}@{version_b}` have different models"
    )]
    DifferentProtocolModels {
        target_model: Box<str>,
        version_a: Version,
        version_b: Version,
    },

    #[error("{0}")]
    Bus(#[from] creamy_engine_core::bus::BusError),
}

struct TempPluginPackage {
    manifest: Manifest,
    pool: StringPool,
    definitions: FxHashMap<Box<str>, (ProtocolDefinition, RequestedProtocol)>,
}

impl TempPluginPackage {
    fn from_package(
        BinaryPlugin {
            manifest,
            pool,
            mut definitions,
            ..
        }: BinaryPlugin,
    ) -> Result<Self, Error> {
        let mut map = FxHashMap::default();
        for (name, request) in manifest.requested_groups() {
            if let Some(index) = definitions.iter().enumerate().find_map(|(idx, def)| {
                if def.name().resolve(&pool) == name.as_str() {
                    Some(idx)
                } else {
                    None
                }
            }) {
                let definition = definitions.swap_remove(index);
                map.insert(name.as_str().into(), (definition, request.clone()));
            } else {
                return Err(Error::ProtocolModelNotFound {
                    target_model: manifest.name().into(),
                    version: request.version().clone(),
                });
            }
        }

        Ok(Self {
            manifest,
            pool,
            definitions: map,
        })
    }
}

pub(crate) const M: usize = 1024;
define_bus_config! {
    Legacy,
    max_subscribers: 32,
    max_messages: 1024,
    max_groups: 32,
}

pub enum SubscriberType<R: WasmRuntime<Legacy> + 'static> {
    System(SystemPlugin),
    Wasm(R::Module),
}

impl<R: WasmRuntime<Legacy> + 'static> Subscriber for SubscriberType<R> {
    fn notify(&mut self) {
        match self {
            SubscriberType::System(s) => s.notify(),
            SubscriberType::Wasm(s) => s.notify(),
        }
    }
}

impl<R: WasmRuntime<Legacy> + 'static> SubscriberType<R> {
    fn as_system_mut(&mut self) -> &mut SystemPlugin {
        match self {
            SubscriberType::System(p) => p,
            SubscriberType::Wasm(_) => unreachable!(),
        }
    }
}

pub struct PluginEngine<R: WasmRuntime<Legacy> + 'static, L: PluginLoader> {
    bus: MessageBus<Legacy, EngineBusDriver, M, SubscriberType<R>>,
    constants: Constants,
    runtime: R,
    loader: L,
    registry: ProtocolRegistry,
}

impl<R: WasmRuntime<Legacy>, L: PluginLoader> PluginEngine<R, L> {
    pub fn new(constants: Constants, runtime: R, loader: L) -> Self {
        let mut bus = MessageBus::new(EngineBusDriver::new(
            Legacy::MAX_SUBSCRIBERS.get(),
            Legacy::MAX_GROUPS.get(),
        ));

        let incoming = IncBuf::default();
        let outgoing = OutBuf::default();

        bus.add_subscriber_with(
            incoming.clone(),
            outgoing.clone(),
            SubscriberType::System(SystemPlugin::new(incoming, outgoing)),
        )
        .unwrap();

        Self {
            bus,
            constants,
            runtime,
            loader,
            registry: ProtocolRegistry::default(),
        }
    }

    fn init_package(&mut self, package: BinaryPlugin) -> Result<(), Error> {
        let module = self
            .runtime
            .init_module(&self.constants, package.core())
            .unwrap();
        let inc_ptr = module.incoming_ptr();
        let out_ptr = module.outgoing_ptr();

        let inc = IncBuf::<M>::from_buf(unsafe { SharedBuf::from_ptr_only(inc_ptr) });
        let out = OutBuf::<M>::from_buf(unsafe { SharedBuf::from_ptr_only(out_ptr) });

        let id = self
            .bus
            .add_subscriber_with(inc, out, SubscriberType::Wasm(module))?;

        let package = TempPluginPackage::from_package(package)?;
        let protocol_name = package.manifest.name();

        self.bus.get_driver_mut().provide_api(
            SubscriberId::new_u8(1).unwrap(),
            SubscriberLookupData {
                consumer_group_id: 1,
                provider_group_id: 1,
                provider_id: SubscriberId::new_u8(1).unwrap(),
            },
        );

        let sys = self
            .bus
            .get_subscriber_mut(SubscriberId::new_u8(1).unwrap())
            .unwrap();
        sys.as_system_mut()
            .add_plugin_name(id, package.manifest.id());

        for (name, (mut def, request)) in package.definitions {
            self.registry.replace_strings(&package.pool, &mut def);
            if request.provide() {
                if let Some(def) = self.registry.get_model(&name) {
                    return Err(Error::ProtocolDeclaredAlready {
                        target_model: name,
                        version: def.version().clone(),
                    });
                }

                self.registry.declare_protocol(protocol_name, def, id);
                self.bus.get_driver_mut().provide_api(
                    id,
                    SubscriberLookupData {
                        consumer_group_id: 1,
                        provider_group_id: 1,
                        provider_id: id,
                    },
                );
            } else {
                match self.registry.get_model(&name) {
                    Some(decl) => {
                        if decl != &def {
                            return Err(Error::DifferentProtocolModels {
                                target_model: name,
                                version_a: def.version().clone(),
                                version_b: decl.version().clone(),
                            });
                        }
                        //TODO: provide
                    }
                    None => {
                        return Err(Error::ProtocolModelNotFound {
                            target_model: name,
                            version: def.version().clone(),
                        });
                    }
                }
            }
        }

        self.bus.update_lookup_table(id);
        self.bus
            .update_lookup_table(SubscriberId::new_u8(1).unwrap());

        Ok(())
    }

    pub fn tick(&mut self, roundtrip: NonZeroU8) {
        self.loader.load();

        while self.loader.loaded() != 0
            && let Some(package) = self.loader.take_loaded_package()
        {
            if let Err(e) = self.init_package(package) {
                tracing::error!("{e}");
            }
        }

        for _ in 0..roundtrip.get() {
            self.bus.tick();
        }
    }

    #[must_use]
    pub const fn loaded_plugins(&self) -> u8 {
        self.bus.subscribers()
    }

    pub const fn protocol_registry(&self) -> &ProtocolRegistry {
        &self.registry
    }
}
