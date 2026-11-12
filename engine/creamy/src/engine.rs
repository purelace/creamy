use alloc::{boxed::Box, string::ToString, vec::Vec};
use core::num::NonZeroU8;

use creamy_engine_core::{
    Metadata, Module, ProtocolTable, Runtime,
    bus::{
        MessageBus, SubscriberLookupData,
        core::{
            SubscriberId,
            buffer::{IncBuf, OutBuf},
        },
    },
    package::{
        BinaryPlugin,
        manifest::Package,
        model::{
            definition::{BinaryReader, ProtocolModel},
            strpool::{StringPool, StringPoolResolver},
        },
        pool::StringId,
    },
};
use creamy_sdk::state::PluginState;
use smol_str::{SmolStr, ToSmolStr};

use crate::{
    driver::EngineBusDriver,
    error::{Error, PluginError},
    package::{Models, PluginPackage},
    plugin::{Plugin, SubscriberType},
    registry::{Owner, ProtocolRegistry},
    system::System,
};

const SYSTEM_PLUGIN: SubscriberId = SubscriberId::new(NonZeroU8::new(1).unwrap());
const SYSTEM_PROTOCOL: NonZeroU8 = NonZeroU8::new(1).unwrap();

pub trait HostSubscriber: creamy_sdk::api::Plugin {
    fn get_protocol_table(&self) -> ProtocolTable<'_>;
}

impl HostSubscriber for () {
    fn get_protocol_table(&self) -> ProtocolTable<'static> {
        ProtocolTable::new(&[], 0, &[], 0)
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

struct Protocol {
    name: StringId,
    provide: bool,
}

#[derive(Default)]
struct PluginData {
    protocols: Vec<Protocol>,
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
        let inc = IncBuf::default();
        let out = OutBuf::default();

        self.bus
            .add_subscriber_with(
                inc.clone(),
                out.clone(),
                Plugin {
                    manifest: Package::default(),
                    kind: SubscriberType::System(
                        PluginState::new(inc.as_dyn_buf(), out.as_dyn_buf(), out.as_dyn_buf())
                            .unwrap(),
                    ),
                },
            )
            .unwrap_or_else(|_| unreachable!("Maximum subscribers cannot be zero."));

        self.bus.get_driver_mut().provide_api(
            SYSTEM_PLUGIN,
            SubscriberLookupData {
                consumer_protocol_id: SYSTEM_PROTOCOL.get(),
                provider_protocol_id: SYSTEM_PROTOCOL.get(),
                provider_id: SYSTEM_PLUGIN,
            },
        );
    }

    fn get_system(&self) -> &System<MS> {
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

    fn get_system_mut(&mut self) -> &mut System<MS> {
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

    fn provide_system_protocol(&mut self, id: SubscriberId) {
        self.bus.get_driver_mut().provide_api(
            id,
            SubscriberLookupData {
                consumer_protocol_id: SYSTEM_PROTOCOL.get(),
                provider_protocol_id: SYSTEM_PROTOCOL.get(),
                provider_id: SYSTEM_PLUGIN,
            },
        );
    }

    fn is_possible_to_consume(&self, model: &ProtocolModel) -> Result<(), PluginError> {
        let pool = self.registry.pool();
        if let Some(context) = self.registry.try_get_protocol_context(model.name()) {
            if context.model() != model {
                return Err(PluginError::DifferentProtocolModels {
                    target_model: model.name().resolve(pool).into(),
                    version_a: model.version().clone(),
                    version_b: context.model().version().clone(),
                });
            }

            if !context.can_add_consumer() {
                return Err(PluginError::ProtocolIsBusy {
                    protocol: context
                        .model()
                        .name()
                        .resolve(pool)
                        .to_string()
                        .into_boxed_str(),
                });
            }

            Ok(())
        } else {
            Err(PluginError::ProtocolModelNotFound {
                target_model: model.name().resolve(pool).to_string().into_boxed_str(),
                version: model.version().clone(),
            })
        }
    }

    fn is_possible_to_provide(&self, model: &ProtocolModel) -> Result<(), PluginError> {
        let pool = self.registry.pool();
        if let Some(context) = self.registry.try_get_protocol_context(model.name()) {
            if context.model() != model {
                return Err(PluginError::DifferentProtocolModels {
                    target_model: model.name().resolve(pool).into(),
                    version_a: model.version().clone(),
                    version_b: context.model().version().clone(),
                });
            }

            if let Some(owner) = context.try_get_owner() {
                return Err(PluginError::ProtocolAlreadyProvided {
                    protocol: model.name().resolve(pool).into(),
                    provider: self
                        .get_system()
                        .get_plugin_name(owner.provider_id())
                        .unwrap_or("Unreachable!")
                        .into(),
                });
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    fn create_custom_validation_context(
        &mut self,
        metadata: Metadata,
        custom: impl Fn(IncBuf<MM>, OutBuf<MM>) -> PluginState<H>,
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

            if request.provide() {
                self.is_possible_to_provide(model)?;
            } else {
                self.is_possible_to_consume(model)?;
            }
        }

        Ok(())
    }

    fn provide_model(&mut self, provider: SubscriberId, name: &str, model: ProtocolModel) {
        let context = self.registry.get_or_declare_protocol(model).1;

        let protocol_table = match &mut self.bus.get_subscriber_mut(provider).unwrap().kind {
            SubscriberType::System(_) => unreachable!(),
            SubscriberType::Custom(s) => s.get_protocol_table(),
            SubscriberType::Module(s) => s.get_protocol_table(),
        };

        let mut to_send = alloc::vec![];

        let protocol_id = protocol_table.get_protocol_id(name);
        context.set_protocol_owner(Owner::new(provider, protocol_id));

        for (consumer, protocol_id) in context.consumer_list() {
            to_send.push((provider, protocol_id, consumer));
        }
        to_send.push((provider, protocol_id, provider));

        log::debug!("Set id {protocol_id} for `{name}` protocol");

        for (provider, protocol_id, consumer) in to_send {
            self.get_system_mut()
                .send_protocol_declared(provider, protocol_id, consumer);
        }
    }

    fn consume_model(&mut self, plugin_id: SubscriberId, name: &str, model: ProtocolModel) -> bool {
        let context = self.registry.get_or_declare_protocol(model).1;

        let mut to_send = alloc::vec![];

        let protocol_table = match &mut self.bus.get_subscriber_mut(plugin_id).unwrap().kind {
            SubscriberType::System(_) => todo!(),
            SubscriberType::Custom(s) => s.get_protocol_table(),
            SubscriberType::Module(s) => s.get_protocol_table(),
        };

        let protocol_id = protocol_table.get_protocol_id(name);

        if context.try_add_consumer(plugin_id, protocol_id)
            && let Some(owner) = context.try_get_owner()
        {
            self.bus.get_driver_mut().provide_api(
                plugin_id,
                SubscriberLookupData {
                    consumer_protocol_id: protocol_id.get(),
                    provider_protocol_id: owner.protocol_id().get(),
                    provider_id: owner.provider_id(),
                },
            );
            to_send.push((owner, plugin_id));
            log::debug!("Provide protocol ({protocol_id}) for subscriber ({plugin_id})");
        } else {
            self.errors.push(PluginError::ProtocolIsBusy {
                protocol: name.to_string().into_boxed_str(),
            });

            return false;
        }

        for (owner, consumer) in to_send {
            self.get_system_mut().send_protocol_declared(
                owner.provider_id(),
                owner.protocol_id(),
                consumer,
            );

            self.get_system_mut().send_consumer_appeared(
                owner.provider_id(),
                owner.protocol_id(),
                consumer,
            );
        }

        true
    }

    fn resolve_dependencies(&mut self, models: Models, id: SubscriberId) {
        let mut data = PluginData::default();

        for (name, (model, request)) in models {
            let name_id = model.name();
            if request.provide() {
                self.provide_model(id, &name, model);
            } else {
                self.consume_model(id, &name, model);
            }

            data.protocols.push(Protocol {
                name: name_id,
                provide: request.provide(),
            });
        }

        self.provide_system_protocol(id);
        self.bus.update_lookup_table(id);
        self.bus.update_lookup_table(SYSTEM_PLUGIN);
        self.data[id.as_usize()] = data;
    }

    fn register_plugin(&mut self, context: ValidationContext) {
        self.resolve_dependencies(context.models, context.id);
        self.get_system_mut()
            .add_plugin_name(context.id, context.name);
        self.get_system_mut()
            .send_protocol_declared(SYSTEM_PLUGIN, SYSTEM_PROTOCOL, context.id);
    }

    pub fn unregister_plugin(&mut self, id: SubscriberId) -> Result<(), Error> {
        let mut to_send_disappeared = Vec::new();
        let mut to_send_undeclared = Vec::new();

        self.get_system_mut().remove_plugin_name(id);
        self.bus.remove_subscriber(id)?;
        let PluginData {
            protocols: models, ..
        } = core::mem::take(&mut self.data[id.as_usize()]);
        for Protocol { name, provide } in models {
            if provide {
                let context = self.registry.undeclare_protocol(name);
                to_send_undeclared.push(context);
            } else {
                let protocol = self.registry.get_mut_protocol_context_by_name(name);
                protocol.remove_consumer(id);
                to_send_disappeared.push(protocol.get_owner());
            }
        }

        for owner in to_send_disappeared {
            self.get_system_mut().send_consumer_disappeared(
                owner.provider_id(),
                owner.protocol_id(),
                id,
            );
        }

        for context in to_send_undeclared {
            for (consumer, protocol) in context.consumer_list() {
                self.get_system_mut()
                    .send_protocol_undeclared(id, protocol, consumer);
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
                    self.errors.push(e);
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
        custom: impl Fn(IncBuf<MM>, OutBuf<MM>) -> PluginState<H>,
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
