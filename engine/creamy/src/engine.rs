use alloc::{boxed::Box, format, vec::Vec};
use core::num::NonZeroU8;

use creamy_engine_core::{
    GroupTable, Metadata, Module, Runtime,
    bus::{
        MessageBus, SubscriberLookupData,
        core::{
            Subscriber, SubscriberId,
            buffer::{IncBuf, OutBuf},
        },
    },
    package::{
        BinaryPlugin,
        manifest::{Package, RequestedProtocol},
        model::{
            definition::{BinaryReader, ProtocolModel},
            strpool::{StringPool, StringPoolResolver},
        },
        pool::StringId,
    },
};
use smol_str::{SmolStr, ToSmolStr};

use crate::{
    driver::EngineBusDriver,
    error::{Error, PluginError},
    package::{Models, PluginPackage},
    registry::{Owner, ProtocolRegistry},
    system::SystemPlugin,
};

const SYSTEM_PLUGIN: SubscriberId = SubscriberId::new(NonZeroU8::new(1).unwrap());
const SYSTEM_GROUP: NonZeroU8 = NonZeroU8::new(1).unwrap();

pub trait HostSubscriber: Subscriber {
    fn get_group_table(&self) -> GroupTable<'_>;
}

impl HostSubscriber for () {
    fn get_group_table(&self) -> GroupTable<'static> {
        GroupTable::new(&[], 0, &[], 0)
    }
}

pub enum SubscriberType<R, H, const MM: usize, const MS: usize>
where
    R: Runtime<MM>,
    H: HostSubscriber,
{
    System(SystemPlugin<MM, MS>),
    Custom(H),
    Module(R::Module),
}

impl<R, H, const MM: usize, const MS: usize> SubscriberType<R, H, MM, MS>
where
    R: Runtime<MM>,
    H: HostSubscriber,
{
    pub fn as_custom_mut(&mut self) -> &mut H {
        match self {
            SubscriberType::Custom(c) => c,
            SubscriberType::System(_) => {
                panic!("invalid subscriber type: System, expected: Custom")
            }
            SubscriberType::Module(_) => {
                panic!("invalid subscriber type: Module, expected: Custom")
            }
        }
    }
}

pub struct Plugin<R, H, const MM: usize, const MS: usize>
where
    R: Runtime<MM>,
    H: HostSubscriber,
{
    manifest: Package,
    kind: SubscriberType<R, H, MM, MS>,
}

impl<R, H, const MM: usize, const MS: usize> Plugin<R, H, MM, MS>
where
    R: Runtime<MM>,
    H: HostSubscriber,
{
    pub const fn manifest(&self) -> &Package {
        &self.manifest
    }

    pub const fn inner(&self) -> &SubscriberType<R, H, MM, MS> {
        &self.kind
    }

    pub const fn inner_mut(&mut self) -> &mut SubscriberType<R, H, MM, MS> {
        &mut self.kind
    }
}

impl<R, H, const MM: usize, const MS: usize> Subscriber for Plugin<R, H, MM, MS>
where
    R: Runtime<MM>,
    H: HostSubscriber,
{
    fn notify(&mut self) {
        match &mut self.kind {
            SubscriberType::System(s) => s.notify(),
            SubscriberType::Custom(s) => s.notify(),
            SubscriberType::Module(s) => s.notify(),
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

/*
 * Unload a plugin
 * Remove from the bus
 * Send system messages
 */

struct ValidationContext {
    id: SubscriberId,
    name: SmolStr,
    pool: StringPool,
    models: Models,
}

struct Groups {
    inner: Vec<StringId>,
    provide: bool,
}

#[derive(Default)]
struct PluginData {
    models: Vec<(StringId, Groups)>,
}

pub struct PluginEngine<
    R: Runtime<MM>,
    H: HostSubscriber,
    const MM: usize,
    const MG: usize,
    const MS: usize,
> {
    runtime: R,
    bus: MessageBus<EngineBusDriver<MS>, MM, MG, MS, Plugin<R, H, MM, MS>>,
    registry: ProtocolRegistry,
    errors: Vec<PluginError>,
    data: Box<[PluginData; MS]>,
}

impl<R: Runtime<MM>, H: HostSubscriber, const MM: usize, const MG: usize, const MS: usize>
    PluginEngine<R, H, MM, MG, MS>
{
    pub fn new(runtime: R) -> Self {
        let bus = MessageBus::new(EngineBusDriver::new());

        let mut instance = Self {
            bus,
            runtime,
            registry: ProtocolRegistry::default(),
            errors: Vec::with_capacity(32),
            data: Box::new(core::array::from_fn(|_| PluginData::default())),
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

    fn get_system(&self) -> &SystemPlugin<MM, MS> {
        const UNREACHABLE: &str =
            "System plugin is registered when the engine instance is created.";

        let plugin = self
            .bus
            .get_subscriber(SYSTEM_PLUGIN)
            .unwrap_or_else(|| unreachable!("{UNREACHABLE}"));

        match &plugin.kind {
            SubscriberType::System(s) => s,
            SubscriberType::Custom(_) | SubscriberType::Module(_) => unreachable!("{UNREACHABLE}"),
        }
    }

    fn get_system_mut(&mut self) -> &mut SystemPlugin<MM, MS> {
        const UNREACHABLE: &str =
            "System plugin is registered when the engine instance is created.";

        let plugin = self
            .bus
            .get_subscriber_mut(SYSTEM_PLUGIN)
            .unwrap_or_else(|| unreachable!("{UNREACHABLE}"));

        match &mut plugin.kind {
            SubscriberType::System(s) => s,
            SubscriberType::Custom(_) | SubscriberType::Module(_) => unreachable!("{UNREACHABLE}"),
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
        if let Some(context) = self.registry.try_get_protocol_context(model.name()) {
            if context.model() != model {
                return Err(PluginError::DifferentProtocolModels {
                    target_model: model.name().resolve(pool).into(),
                    version_a: model.version().clone(),
                    version_b: context.model().version().clone(),
                });
            }

            for group_name in request.groups() {
                if let Some(owner) =
                    context.try_get_owner_of_group(pool.get_id(group_name.as_str()))
                {
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
        metadata: Metadata,
        custom: impl Fn(IncBuf<MM>, OutBuf<MM>) -> H,
    ) -> Result<Option<ValidationContext>, Error> {
        let package = match PluginPackage::from_metadata(metadata) {
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

    fn create_validation_context(
        &mut self,
        metadata: Metadata,
        module: R::Module,
    ) -> Result<Option<ValidationContext>, Error> {
        let package = match PluginPackage::from_metadata(metadata) {
            Ok(v) => v,
            Err(e) => {
                self.errors.push(e);
                return Ok(None);
            }
        };

        let inc_ptr = module.incoming_ptr();
        let out_ptr = module.outgoing_ptr();

        let inc = unsafe { IncBuf::<MM>::from_ptr(inc_ptr, false) };
        let out = unsafe { OutBuf::<MM>::from_ptr(out_ptr, false) };

        let name = package.manifest.name().to_smolstr();
        let id = self.bus.add_subscriber_with(
            inc,
            out,
            Plugin {
                manifest: package.manifest,
                kind: SubscriberType::Module(module),
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
            SubscriberType::Module(s) => s.get_group_table(),
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

            log::debug!(
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
                SubscriberType::Module(s) => s.get_group_table(),
            };

            let path = format!("{model_name}.{}", group_name.as_str());
            let group_id = group_table.get_group_id(&path);
            let group_name_id = pool.get_id(group_name.as_str());

            if context.try_add_consumer(group_name_id, plugin_id, group_id)
                && let Some(owner) = context.try_get_owner_of_group(group_name_id)
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

            log::debug!("Provide group ({group_id}) for subscriber ({plugin_id})");
        }

        for (owner, consumer) in to_send {
            self.get_system_mut().send_group_declared(
                owner.provider_id(),
                owner.group_id(),
                consumer,
            );

            self.get_system_mut().send_consumer_appeared(
                owner.provider_id(),
                owner.group_id(),
                consumer,
            );
        }
    }

    fn resolve_dependencies(&mut self, models: Models, id: SubscriberId) {
        let mut data = PluginData::default();

        for (_, (def, request)) in models {
            let name = def.name();
            if request.provide() {
                self.provide_model(id, def, &request);
            } else {
                self.consume_model(id, def, &request);
            }

            let inner = request
                .groups()
                .iter()
                .map(|s| self.registry.pool_mut().get_id(s))
                .collect::<Vec<_>>();
            data.models.push((
                name,
                Groups {
                    inner,
                    provide: request.provide(),
                },
            ));
        }

        self.provide_system_group(id);
        self.bus.update_lookup_table(id);
        self.bus.update_lookup_table(SYSTEM_PLUGIN);
        self.data[id.as_usize()] = data;
    }

    fn register_plugin(&mut self, context: ValidationContext) {
        self.resolve_dependencies(context.models, context.id);
        self.get_system_mut()
            .add_plugin_name(context.id, context.name);
        self.get_system_mut()
            .send_group_declared(SYSTEM_PLUGIN, SYSTEM_GROUP, context.id);
    }

    pub fn unregister_plugin(&mut self, id: SubscriberId) -> Result<(), Error> {
        let mut to_send_disappeared = Vec::new();
        let mut to_send_undeclared = Vec::new();

        self.get_system_mut().remove_plugin_name(id);
        self.bus.remove_subscriber(id)?;
        let PluginData { models, .. } = core::mem::take(&mut self.data[id.as_usize()]);
        for (name, requested) in models {
            let protocol = self.registry.get_mut_protocol_context_by_name(name);
            if requested.provide {
                for group in requested.inner {
                    let context = protocol.undeclare_group(group);
                    to_send_undeclared.push(context);
                }
            } else {
                for group in requested.inner {
                    protocol.remove_consumer(group, id);
                    let owner = protocol.get_owner_of_group(group);
                    to_send_disappeared.push(owner);
                }
            }
        }

        for owner in to_send_disappeared {
            self.get_system_mut().send_consumer_disappeared(
                owner.provider_id(),
                owner.group_id(),
                id,
            );
        }

        for context in to_send_undeclared {
            for (consumer, group) in context.consumer_list() {
                self.get_system_mut()
                    .send_group_undeclared(id, group, consumer);
            }
        }

        //TODO: обнулить слайсы в lookup table

        Ok(())
    }

    fn handle_ready_plugin(
        &mut self,
        context: Result<Option<ValidationContext>, Error>,
    ) -> Option<SubscriberId> {
        match context {
            Ok(result) => {
                let mut context = result?;

                if let Err(e) = self.validate_plugin(&mut context) {
                    self.remove_validation_context(context);
                    log::error!("{e}");
                    None
                } else {
                    let id = context.id;
                    self.register_plugin(context);
                    Some(id)
                }
            }
            Err(e) => {
                log::error!("{e}");
                None
            }
        }
    }

    pub fn register_custom_plugin(
        &mut self,
        mut package: &[u8],
        custom: impl Fn(IncBuf<MM>, OutBuf<MM>) -> H,
    ) -> Option<SubscriberId> {
        let package = match BinaryPlugin::read_ne(&mut package) {
            Ok(v) => v,
            Err(e) => {
                self.errors.push(PluginError::Binary(e));
                return None;
            }
        };

        let metadata = Metadata {
            version: package.version,
            manifest: package.manifest,
            pool: package.pool,
            models: package.models,
        };

        let context = self.create_custom_validation_context(metadata, custom);
        self.handle_ready_plugin(context)
    }

    fn load_plugins(&mut self) {
        match self.runtime.pop_ready_plugin() {
            Ok(Some((metadata, module))) => {
                let context = self.create_validation_context(metadata, module);
                self.handle_ready_plugin(context);
            }
            Ok(None) => {}
            Err(e) => log::error!("{e}"),
        }
    }

    pub fn tick(&mut self, roundtrip: NonZeroU8) {
        self.runtime.tick();
        self.load_plugins();
        for _ in 0..roundtrip.get() {
            self.bus.tick();
        }
    }

    pub fn get_plugin(&self, id: SubscriberId) -> Option<&Plugin<R, H, MM, MS>> {
        self.bus.get_subscriber(id)
    }

    pub fn get_plugin_mut(&mut self, id: SubscriberId) -> Option<&mut Plugin<R, H, MM, MS>> {
        self.bus.get_subscriber_mut(id)
    }

    pub fn plugins(&self) -> impl Iterator<Item = &Plugin<R, H, MM, MS>> {
        self.bus.subscriber_list()
    }

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

    pub const fn runtime(&self) -> &R {
        &self.runtime
    }

    pub const fn runtime_mut(&mut self) -> &mut R {
        &mut self.runtime
    }
}
