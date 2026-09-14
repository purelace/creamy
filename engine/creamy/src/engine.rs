use alloc::{boxed::Box, format, vec::Vec};
use core::{marker::PhantomData, num::NonZeroU8};

use creamy_engine_core::{
    Constants, GroupTable, PluginLoader, WasmModule, WasmRuntime,
    bus::{
        MessageBus, SubscriberLookupData,
        config::BusConfig,
        core::{
            Subscriber, SubscriberId,
            buffer::{IncBuf, OutBuf, SharedBuf},
        },
    },
    devkit::{
        BinaryPlugin,
        binrw::io::Cursor,
        compiler::model::{
            definition::ProtocolModel,
            strpool::{StringPool, StringPoolResolver},
        },
        manifest::{Package, RequestedProtocol},
    },
};
use hashbrown::HashMap;
use rustc_hash::FxBuildHasher;
use smol_str::{SmolStr, ToSmolStr};

use crate::{
    driver::EngineBusDriver,
    error::{Error, PluginError},
    registry::{Owner, ProtocolRegistry},
    system::SystemPlugin,
};

const SYSTEM_PLUGIN: SubscriberId = SubscriberId::new(NonZeroU8::new(1).unwrap());
const SYSTEM_GROUP: NonZeroU8 = NonZeroU8::new(1).unwrap();

type Models = HashMap<Box<str>, (ProtocolModel, RequestedProtocol), FxBuildHasher>;

struct PluginPackage {
    manifest: Package,
    pool: StringPool,
    models: Models,
}

impl PluginPackage {
    fn from_package(
        BinaryPlugin {
            manifest,
            pool,
            mut models,
            ..
        }: BinaryPlugin,
    ) -> Result<Self, PluginError> {
        let mut map = HashMap::default();
        for (name, request) in manifest.requested_groups() {
            if let Some(index) = models.iter().enumerate().find_map(|(idx, def)| {
                if def.name().resolve(&pool) == name.as_str() {
                    Some(idx)
                } else {
                    None
                }
            }) {
                let definition = models.swap_remove(index);
                map.insert(name.as_str().into(), (definition, request.clone()));
            } else {
                return Err(PluginError::ProtocolModelNotFound {
                    target_model: manifest.name().into(),
                    version: request.version().clone(),
                });
            }
        }

        Ok(Self {
            manifest: manifest.into_package_manifest(),
            pool,
            models: map,
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

struct Plugin<C, R, H, const S: usize, const M: usize>
where
    C: BusConfig,
    R: WasmRuntime<C>,
    H: HostSubscriber,
{
    manifest: Package,
    kind: SubscriberType<C, R, H, S, M>,
}

pub enum SubscriberType<C, R, H, const S: usize, const M: usize>
where
    C: BusConfig,
    R: WasmRuntime<C>,
    H: HostSubscriber,
{
    System(SystemPlugin<S, M>),
    Custom(H),
    Wasm(R::Module),
}

impl<C, R, H, const S: usize, const M: usize> Subscriber for Plugin<C, R, H, S, M>
where
    C: BusConfig,
    R: WasmRuntime<C>,
    H: HostSubscriber,
{
    fn notify(&mut self) {
        match &mut self.kind {
            SubscriberType::System(s) => s.notify(),
            SubscriberType::Custom(s) => s.notify(),
            SubscriberType::Wasm(s) => s.notify(),
        }
    }
}
/*
 * Load a package
 * Init a plugin
 * Add to the bus
 * Resolve dependencies
 * Send system messages
 * Finish the plugin initialization
 */

struct ValidationContext {
    id: SubscriberId,
    name: SmolStr,
    pool: StringPool,
    models: Models,
}

pub struct PluginEngine<
    C: BusConfig,
    R: WasmRuntime<C>,
    L: PluginLoader,
    H: HostSubscriber,
    const S: usize,
    const M: usize,
> {
    bus: MessageBus<C, EngineBusDriver<S>, M, Plugin<C, R, H, S, M>>,
    constants: Constants, //WTF?
    runtime: R,
    loader: L,
    registry: ProtocolRegistry,
    errors: Vec<PluginError>,
    _phantom: PhantomData<C>,
}

impl<
    C: BusConfig,
    R: WasmRuntime<C>,
    L: PluginLoader,
    H: HostSubscriber,
    const S: usize,
    const M: usize,
> PluginEngine<C, R, L, H, S, M>
{
    pub fn new(constants: Constants, runtime: R, loader: L) -> Self {
        let bus = MessageBus::new(EngineBusDriver::new());

        let mut instance = Self {
            bus,
            constants,
            runtime,
            loader,
            registry: ProtocolRegistry::default(),
            errors: Vec::with_capacity(32),
            _phantom: PhantomData,
        };

        instance.setup_system_plugin();
        instance
    }

    fn setup_system_plugin(&mut self) {
        let incoming = IncBuf::default();
        let outgoing = OutBuf::default();

        self.bus
            .add_subscriber_with(
                incoming.clone(),
                outgoing.clone(),
                Plugin {
                    manifest: Package::default(),
                    kind: SubscriberType::System(SystemPlugin::new(incoming, outgoing)),
                },
            )
            .unwrap_or_else(|_| unreachable!("Maximum subscribers cannot be zero."));

        self.bus.get_driver_mut().provide_api(
            SYSTEM_PLUGIN,
            SubscriberLookupData {
                consumer_group_id: SYSTEM_GROUP.get(),
                provider_group_id: SYSTEM_GROUP.get(),
                provider_id: SYSTEM_PLUGIN,
            },
        );
    }

    fn get_system(&self) -> &SystemPlugin<S, M> {
        const UNREACHABLE: &str =
            "System plugin is registered when the engine instance is created.";

        let plugin = self
            .bus
            .get_subscriber(SYSTEM_PLUGIN)
            .unwrap_or_else(|| unreachable!("{UNREACHABLE}"));

        match &plugin.kind {
            SubscriberType::System(s) => s,
            SubscriberType::Custom(_) | SubscriberType::Wasm(_) => unreachable!("{UNREACHABLE}"),
        }
    }

    fn get_system_mut(&mut self) -> &mut SystemPlugin<S, M> {
        const UNREACHABLE: &str =
            "System plugin is registered when the engine instance is created.";

        let plugin = self
            .bus
            .get_subscriber_mut(SYSTEM_PLUGIN)
            .unwrap_or_else(|| unreachable!("{UNREACHABLE}"));

        match &mut plugin.kind {
            SubscriberType::System(s) => s,
            SubscriberType::Custom(_) | SubscriberType::Wasm(_) => unreachable!("{UNREACHABLE}"),
        }
    }

    fn provide_system_group(&mut self, id: SubscriberId) {
        self.bus.get_driver_mut().provide_api(
            id,
            SubscriberLookupData {
                consumer_group_id: SYSTEM_GROUP.get(),
                provider_group_id: SYSTEM_GROUP.get(),
                provider_id: SYSTEM_PLUGIN,
            },
        );
    }

    fn is_possible_to_resolve(
        &self,
        model: &ProtocolModel,
        request: &RequestedProtocol,
    ) -> Result<(), PluginError> {
        let pool = self.registry.pool();
        if let Some(context) = self.registry.get_protocol_context(model.name()) {
            if context.model() != model {
                return Err(PluginError::DifferentProtocolModels {
                    target_model: model.name().resolve(pool).into(),
                    version_a: model.version().clone(),
                    version_b: context.model().version().clone(),
                });
            }

            for group_name in request.groups() {
                if let Some(owner) = context.get_owner_of_group(pool.get_id(group_name.as_str())) {
                    return Err(PluginError::GroupAlreadyProvided {
                        group: group_name.as_str().into(),
                        protocol: model.name().resolve(pool).into(),
                        provider: self
                            .get_system()
                            .get_plugin_name(owner.provider_id())
                            .unwrap_or("Unreachable!")
                            .into(),
                    });
                }
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    fn create_custom_validation_context(
        &mut self,
        package: BinaryPlugin,
        custom: impl Fn(IncBuf<M>, OutBuf<M>) -> H,
    ) -> Result<Option<ValidationContext>, Error> {
        let package = match PluginPackage::from_package(package) {
            Ok(v) => v,
            Err(e) => {
                self.errors.push(e);
                return Ok(None);
            }
        };

        let name = package.manifest.name().to_smolstr();

        let id = self.bus.add_subscriber(|inc, out| Plugin {
            manifest: package.manifest,
            kind: SubscriberType::Custom(custom(inc, out)),
        })?;

        Ok(Some(ValidationContext {
            id,
            name,
            pool: package.pool,
            models: package.models,
        }))
    }

    fn create_wasm_validation_context(
        &mut self,
        package: BinaryPlugin,
    ) -> Result<Option<ValidationContext>, Error> {
        let module = self
            .runtime
            .init_module(&self.constants, package.core())
            .map_err(|e| Error::Other(Box::new(e)))?;

        let package = match PluginPackage::from_package(package) {
            Ok(v) => v,
            Err(e) => {
                self.errors.push(e);
                return Ok(None);
            }
        };

        let inc_ptr = module.incoming_ptr();
        let out_ptr = module.outgoing_ptr();

        let inc = IncBuf::<M>::from_buf(unsafe { SharedBuf::from_ptr(inc_ptr, false) });
        let out = OutBuf::<M>::from_buf(unsafe { SharedBuf::from_ptr(out_ptr, false) });

        let name = package.manifest.name().to_smolstr();
        let id = self.bus.add_subscriber_with(
            inc,
            out,
            Plugin {
                manifest: package.manifest,
                kind: SubscriberType::Wasm(module),
            },
        )?;

        Ok(Some(ValidationContext {
            id,
            name,
            pool: package.pool,
            models: package.models,
        }))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn remove_validation_context(&mut self, context: ValidationContext) {
        let _ = self.bus.remove_subscriber(context.id);
    }

    fn validate_plugin(&mut self, context: &mut ValidationContext) -> Result<(), PluginError> {
        for (model, request) in &mut context.models.values_mut() {
            self.registry.replace_strings(&context.pool, model);

            if !request.provide() {
                continue;
            }

            self.is_possible_to_resolve(model, request)?;
        }

        Ok(())
    }

    fn provide_model(
        &mut self,
        provider: SubscriberId,
        model: ProtocolModel,
        request: &RequestedProtocol,
    ) {
        let model_name_id = model.name();
        let model_name = model_name_id.resolve(self.registry.pool()).to_smolstr();
        let (pool, context) = self.registry.get_or_declare_protocol(model);

        let group_table = match &mut self.bus.get_subscriber_mut(provider).unwrap().kind {
            SubscriberType::System(_) => unreachable!(),
            SubscriberType::Custom(s) => s.get_group_table(),
            SubscriberType::Wasm(s) => s.get_group_table(),
        };

        let mut to_send = alloc::vec![];

        for group_name in request.groups() {
            let path = format!("{model_name}.{}", group_name.as_str());
            let group_id = group_table.get_group_id(&path);
            let group_name_id = pool.get_id(group_name.as_str());

            context.set_group_owner(group_name_id, Owner::new(provider, group_id));

            for (consumer, group_id) in context.get_consumers_of(group_name_id) {
                to_send.push((provider, group_id, consumer));
            }
            to_send.push((provider, group_id, provider));

            tracing::debug!(
                "Set id {group_id} for `{model_name}.{}` group",
                group_name.as_str()
            );
        }

        for (provider, group_id, consumer) in to_send {
            self.get_system_mut()
                .send_group_declared(provider, group_id, consumer);
        }
    }

    fn consume_model(
        &mut self,
        plugin_id: SubscriberId,
        model: ProtocolModel,
        request: &RequestedProtocol,
    ) {
        let (pool, context) = self.registry.get_or_declare_protocol(model);

        let model_name = context.model().name().resolve(pool);
        let mut to_send = alloc::vec![];

        for group_name in request.groups() {
            let group_table = match &mut self.bus.get_subscriber_mut(plugin_id).unwrap().kind {
                SubscriberType::System(_) => todo!(),
                SubscriberType::Custom(s) => s.get_group_table(),
                SubscriberType::Wasm(s) => s.get_group_table(),
            };

            let path = format!("{model_name}.{}", group_name.as_str());
            let group_id = group_table.get_group_id(&path);
            let group_name_id = pool.get_id(group_name.as_str());

            if context.try_add_consumer(group_name_id, plugin_id, group_id)
                && let Some(owner) = context.get_owner_of_group(group_name_id)
            {
                self.bus.get_driver_mut().provide_api(
                    plugin_id,
                    SubscriberLookupData {
                        consumer_group_id: group_id.get(),
                        provider_group_id: owner.group_id().get(),
                        provider_id: owner.provider_id(),
                    },
                );
                to_send.push((owner, plugin_id));
            }

            tracing::debug!("Provide group ({group_id}) for subscriber ({plugin_id})",);
        }

        for (owner, consumer) in to_send {
            self.get_system_mut().send_group_declared(
                owner.provider_id(),
                owner.group_id(),
                consumer,
            );
        }
    }

    fn resolve_dependencies(&mut self, models: Models, id: SubscriberId) {
        for (_, (def, request)) in models {
            if request.provide() {
                self.provide_model(id, def, &request);
            } else {
                self.consume_model(id, def, &request);
            }
        }

        self.provide_system_group(id);
        self.bus.update_lookup_table(id);
        self.bus.update_lookup_table(SYSTEM_PLUGIN);
    }

    fn register_plugin(&mut self, context: ValidationContext) {
        self.resolve_dependencies(context.models, context.id);
        self.get_system_mut()
            .add_plugin_name(context.id, context.name);
    }

    fn unregister_plugin(&mut self, id: SubscriberId) -> Result<(), Error> {
        //TODO: resolve dependencies
        self.get_system_mut().remove_plugin_name(id);
        self.bus.remove_subscriber(id)?;
        Ok(())
    }

    fn handle_loaded_plugin(&mut self, context: Result<Option<ValidationContext>, Error>) {
        match context {
            Ok(result) => {
                let Some(mut context) = result else {
                    return;
                };

                if let Err(e) = self.validate_plugin(&mut context) {
                    self.remove_validation_context(context);
                    tracing::error!("{e}");
                } else {
                    self.register_plugin(context);
                }
            }
            Err(e) => tracing::error!("{e}"),
        }
    }

    pub fn register_custom_plugin(
        &mut self,
        package: &[u8],
        custom: impl Fn(IncBuf<M>, OutBuf<M>) -> H,
    ) {
        let mut reader = Cursor::new(package);
        let package = match BinaryPlugin::read_from(&mut reader) {
            Ok(v) => v,
            Err(e) => {
                self.errors.push(PluginError::Binary(e));
                return;
            }
        };

        let context = self.create_custom_validation_context(package, custom);
        self.handle_loaded_plugin(context);
    }

    fn load_wasm_plugins(&mut self) {
        self.loader.load();

        while self.loader.loaded() != 0
            && let Some(package) = self.loader.take_loaded_package()
        {
            let context = self.create_wasm_validation_context(package);
            self.handle_loaded_plugin(context);
        }
    }

    pub fn tick(&mut self, roundtrip: NonZeroU8) {
        self.load_wasm_plugins();
        for _ in 0..roundtrip.get() {
            self.bus.tick();
        }
    }

    pub fn unload(&mut self, id: SubscriberId) {}

    #[must_use]
    pub const fn loaded_plugins(&self) -> u8 {
        self.bus.subscribers()
    }

    pub const fn protocol_registry(&self) -> &ProtocolRegistry {
        &self.registry
    }

    pub const fn errors(&mut self) -> &mut Vec<PluginError> {
        &mut self.errors
    }
}
