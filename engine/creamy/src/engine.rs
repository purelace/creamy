use alloc::{boxed::Box, format};
use core::{marker::PhantomData, num::NonZeroU8};

use creamy_engine_core::{
    Constants, GroupTable, PluginLoader, WasmModule, WasmRuntime,
    bus::{MessageBus, SubscriberLookupData, config::BusConfig},
    devkit::{
        BinaryPlugin,
        compiler::{
            ProtocolDefinition,
            utils::strpool::{StringPool, StringPoolResolver},
        },
        manifest::{Manifest, RequestedProtocol},
        semver::Version,
    },
};
use creamy_sdk::bus::{
    Subscriber, SubscriberId,
    buffer::{IncBuf, OutBuf, SharedBuf},
};
use creamy_system_plugin::SystemPlugin;
use rustc_hash::FxHashMap;

use crate::{driver::EngineBusDriver, registry::ProtocolRegistry};

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

pub trait HostSubscriber: Subscriber {
    fn get_group_table(&self) -> GroupTable<'_>;
}

impl HostSubscriber for () {
    fn get_group_table(&self) -> GroupTable<'static> {
        GroupTable::new(&[], 0, &[], 0)
    }
}

pub enum SubscriberType<C: BusConfig, R: WasmRuntime<C>, S: HostSubscriber, const M: usize> {
    System(SystemPlugin<M>),
    Custom(S),
    Wasm(R::Module),
}

impl<C: BusConfig, R: WasmRuntime<C>, S: HostSubscriber, const M: usize> Subscriber
    for SubscriberType<C, R, S, M>
{
    fn notify(&mut self) {
        match self {
            Self::System(s) => s.notify(),
            Self::Custom(s) => s.notify(),
            Self::Wasm(s) => s.notify(),
        }
    }
}

impl<C: BusConfig, R: WasmRuntime<C>, S: HostSubscriber, const M: usize>
    SubscriberType<C, R, S, M>
{
    fn as_system_mut(&mut self) -> &mut SystemPlugin<M> {
        match self {
            Self::System(p) => p,
            Self::Custom(_) | Self::Wasm(_) => unreachable!(),
        }
    }
}

pub struct PluginEngine<
    C: BusConfig,
    R: WasmRuntime<C>,
    L: PluginLoader,
    S: HostSubscriber,
    const M: usize,
> {
    bus: MessageBus<C, EngineBusDriver, M, SubscriberType<C, R, S, M>>,
    constants: Constants,
    runtime: R,
    loader: L,
    registry: ProtocolRegistry,
    _phantom: PhantomData<C>,
}

impl<C: BusConfig, R: WasmRuntime<C>, L: PluginLoader, S: HostSubscriber, const M: usize>
    PluginEngine<C, R, L, S, M>
{
    pub fn new(constants: Constants, runtime: R, loader: L) -> Self {
        let mut bus = MessageBus::new(EngineBusDriver::new(
            C::MAX_SUBSCRIBERS.get(),
            C::MAX_GROUPS.get(),
        ));

        let incoming = IncBuf::default();
        let outgoing = OutBuf::default();

        bus.add_subscriber_with(
            incoming.clone(),
            outgoing.clone(),
            SubscriberType::System(SystemPlugin::new(incoming, outgoing, ())),
        )
        .unwrap();

        Self {
            bus,
            constants,
            runtime,
            loader,
            registry: ProtocolRegistry::default(),
            _phantom: PhantomData,
        }
    }

    pub fn add_custom_subscriber(
        &mut self,
        package: &[u8],
        custom: impl Fn(IncBuf<M>, OutBuf<M>) -> S,
    ) {
        let package = BinaryPlugin::load_from_bytes(package).unwrap();
        let package = TempPluginPackage::from_package(package).unwrap();
        let id = self
            .bus
            .add_subscriber(|inc, out| SubscriberType::Custom(custom(inc, out)))
            .unwrap();

        self.resolve_dependencies(package, id);
    }

    fn provide_model(
        &mut self,
        id: SubscriberId,
        model: Box<str>,
        def: ProtocolDefinition,
        request: RequestedProtocol,
    ) -> Result<(), Error> {
        if let Some(ctx) = self.registry.get_protocol_context(&model) {
            return Err(Error::ProtocolDeclaredAlready {
                target_model: model,
                version: ctx.definition().version().clone(),
            });
        }

        self.registry.declare_protocol(model.clone(), def, id);

        let group_table = match self.bus.get_subscriber_mut(id).unwrap() {
            SubscriberType::System(_) => unreachable!(),
            SubscriberType::Custom(s) => s.get_group_table(),
            SubscriberType::Wasm(s) => s.get_group_table(),
        };

        for group_name in request.groups() {
            let path = format!("{model}.{}", group_name.as_str());
            let group_id = group_table.get_group_id(&path);

            self.registry
                .set_group_id(&model, group_name.as_str(), group_id);

            tracing::debug!(
                "Set id ({group_id}) for `{model}.{}` group",
                group_name.as_str()
            );
        }

        Ok(())
    }

    fn consume_model(
        &mut self,
        id: SubscriberId,
        model: Box<str>,
        def: ProtocolDefinition,
        request: RequestedProtocol,
    ) -> Result<(), Error> {
        let Some(ctx) = self.registry.get_protocol_context(&model) else {
            return Err(Error::ProtocolModelNotFound {
                target_model: model,
                version: def.version().clone(),
            });
        };

        if ctx.definition() != &def {
            return Err(Error::DifferentProtocolModels {
                target_model: model,
                version_a: def.version().clone(),
                version_b: ctx.definition().version().clone(),
            });
        }

        for group_name in request.groups() {
            let group_table = match self.bus.get_subscriber_mut(id).unwrap() {
                SubscriberType::System(_) => todo!(),
                SubscriberType::Custom(_) => todo!(),
                SubscriberType::Wasm(s) => s.get_group_table(),
            };

            let path = format!("{model}.{}", group_name.as_str());
            let group_id = group_table.get_group_id(&path);

            let penis = self.registry.pool().get_id(group_name.as_str());

            self.bus.get_driver_mut().provide_api(
                id,
                SubscriberLookupData {
                    consumer_group_id: group_id.get(),
                    provider_group_id: ctx.get_provider_group_id(penis).unwrap().get(),
                    provider_id: ctx.owner(),
                },
            );

            tracing::debug!("Provide group ({group_id}) for subscriber ({id})",);
        }

        Ok(())
    }

    fn resolve_dependencies(
        &mut self,
        package: TempPluginPackage,
        id: SubscriberId,
    ) -> Result<(), Error> {
        // Provide system capabilities
        self.bus.get_driver_mut().provide_api(
            id,
            SubscriberLookupData {
                consumer_group_id: 1,
                provider_group_id: 1,
                provider_id: SubscriberId::new_u8(1).unwrap(),
            },
        );

        for (model, (mut def, request)) in package.definitions {
            self.registry.replace_strings(&package.pool, &mut def);

            if request.provide() {
                self.provide_model(id, model, def, request)?;
            } else {
                self.consume_model(id, model, def, request)?;
            }
        }

        self.bus.update_lookup_table(id);
        self.bus
            .update_lookup_table(SubscriberId::new_u8(1).unwrap());

        Ok(())
    }

    #[allow(clippy::too_many_lines)]
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

        //TODO: это надо убрать, это только для системного плагина
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

        self.resolve_dependencies(package, id)?;

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
