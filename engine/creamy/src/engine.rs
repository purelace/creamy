use alloc::{boxed::Box, vec::Vec};
use core::num::NonZeroU8;

use creamy_engine_core::{
    Module, Runtime,
    bus::{
        MessageBus, SubscriberLookupData,
        core::{
            SubscriberId,
            buffer::{IncBuf, OutBuf},
        },
    },
    scanner::ApiScanner,
};
use creamy_sdk::{ProtocolId, state::PluginState};

use crate::{
    GlobalProtocolId,
    driver::EngineBusDriver,
    error::{Error, PluginError},
    plugin::{Plugin, SubscriberType},
    registry::{Owner, ProtocolRegistry},
    scanner::{Api, CheckedProtocolApi, PluginApi},
    system::System,
};

const SYSTEM_PLUGIN: SubscriberId = SubscriberId::new(NonZeroU8::new(1).unwrap());
pub const SYSTEM_PROTOCOL: ProtocolId = ProtocolId::new_u8(1).unwrap();

pub trait HostSubscriber: creamy_sdk::api::Plugin + ApiScanner {}
impl<T: creamy_sdk::api::Plugin + ApiScanner> HostSubscriber for T {}

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
    api: Api,
}

struct Protocol {
    id: GlobalProtocolId,
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
                    manifest: None,
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
                consumer_protocol_id: SYSTEM_PROTOCOL.as_u8(),
                provider_protocol_id: SYSTEM_PROTOCOL.as_u8(),
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
                consumer_protocol_id: SYSTEM_PROTOCOL.as_u8(),
                provider_protocol_id: SYSTEM_PROTOCOL.as_u8(),
                provider_id: SYSTEM_PLUGIN,
            },
        );
    }

    fn is_possible_to_consume(&self, protocol: &CheckedProtocolApi) -> Result<(), PluginError> {
        if let Some(context) = self.registry.find_protocol(protocol) {
            if context.protocol() != protocol {
                return Err(PluginError::DifferentProtocol {
                    target: protocol.name.clone().into_boxed_str(),
                });
            }

            if !context.can_add_consumer() {
                return Err(PluginError::ProtocolIsBusy {
                    protocol: protocol.name.clone().into_boxed_str(),
                });
            }

            Ok(())
        } else {
            Err(PluginError::ProtocolModelNotFound {
                target: protocol.name.clone().into_boxed_str(),
            })
        }
    }

    fn is_possible_to_provide(&self, protocol: &CheckedProtocolApi) -> Result<(), PluginError> {
        if let Some(context) = self.registry.find_protocol(protocol) {
            if context.protocol() != protocol {
                return Err(PluginError::DifferentProtocol {
                    target: protocol.name.clone().into_boxed_str(),
                });
            }

            if let Some(owner) = context.try_get_owner() {
                return Err(PluginError::ProtocolAlreadyProvided {
                    protocol: protocol.name.clone().into_boxed_str(),
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

    /*
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

            Ok(Some(ValidationContext { id, name }))
        }
    */
    fn create_validation_context(
        &mut self,
        mut module: R::Module,
    ) -> Result<Option<ValidationContext>, Error> {
        let inc_ptr = module.incoming_ptr();
        let out_ptr = module.outgoing_ptr();

        let inc = unsafe { IncBuf::<MM>::from_ptr(inc_ptr, false) };
        let out = unsafe { OutBuf::<MM>::from_ptr(out_ptr, false) };

        let api = Api::scan(&mut module);
        let id = self.bus.add_subscriber_with(
            inc,
            out,
            Plugin {
                manifest: None, //TODO manifest
                kind: SubscriberType::Module(module),
            },
        )?;

        Ok(Some(ValidationContext { id, api }))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn remove_validation_context(&mut self, context: ValidationContext) {
        let _ = self.bus.remove_subscriber(context.id);
    }

    fn validate_plugin(&mut self, context: &mut ValidationContext) -> Result<(), PluginError> {
        for api in &mut context.api.list {
            if api.provide {
                self.is_possible_to_provide(&api.protocol)?;
            } else {
                self.is_possible_to_consume(&api.protocol)?;
            }
        }

        Ok(())
    }

    fn provide_protocol(
        &mut self,
        provider: SubscriberId,
        protocol_id: ProtocolId,
        api: CheckedProtocolApi,
    ) {
        let context = self.registry.get_or_declare_protocol(api);
        context.set_protocol_owner(Owner::new(provider, protocol_id));

        let mut to_send = alloc::vec![];
        for (consumer, protocol_id) in context.consumer_list() {
            to_send.push((provider, protocol_id, consumer));
        }
        to_send.push((provider, protocol_id, provider));

        log::debug!(
            "Set id {protocol_id} for `{}` protocol",
            context.protocol().name
        );

        for (provider, protocol_id, consumer) in to_send {
            self.get_system_mut()
                .send_protocol_declared(provider, protocol_id, consumer);
        }
    }

    fn consume_model(
        &mut self,
        plugin_id: SubscriberId,
        protocol_id: ProtocolId,
        api: CheckedProtocolApi,
    ) -> bool {
        let context = self.registry.get_or_declare_protocol(api);

        let mut to_send = alloc::vec![];

        if context.try_add_consumer(plugin_id, protocol_id)
            && let Some(owner) = context.try_get_owner()
        {
            self.bus.get_driver_mut().provide_api(
                plugin_id,
                SubscriberLookupData {
                    consumer_protocol_id: protocol_id.as_u8(),
                    provider_protocol_id: owner.protocol_id().as_u8(),
                    provider_id: owner.provider_id(),
                },
            );
            to_send.push((owner, plugin_id));
            log::debug!("Provide protocol ({protocol_id}) for subscriber ({plugin_id})");
        } else {
            self.errors.push(PluginError::ProtocolIsBusy {
                protocol: context.protocol().name.clone().into_boxed_str(),
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

    fn resolve_dependencies(&mut self, list: Vec<PluginApi>, id: SubscriberId) {
        let mut data = PluginData::default();

        for api in list {
            if api.provide {
                self.provide_protocol(id, api.id, api.protocol);
            } else {
                self.consume_model(id, api.id, api.protocol);
            }

            //data.protocols.push(Protocol {
            //    name: name_id,
            //    provide: *provide,
            //});
        }

        self.provide_system_protocol(id);
        self.bus.update_lookup_table(id);
        self.bus.update_lookup_table(SYSTEM_PLUGIN);
        self.data[id.as_usize()] = data;
    }

    fn register_plugin(&mut self, context: ValidationContext) {
        self.resolve_dependencies(context.api.list, context.id);
        self.get_system_mut().add_plugin_name(
            context.id,
            context
                .api
                .manifest
                .as_ref()
                .map_or("unnamed", |m| m.name()),
        );
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
        for Protocol {
            id: global_id,
            provide,
        } in models
        {
            if provide {
                let context = self.registry.undeclare_protocol(global_id);
                to_send_undeclared.push(context);
            } else {
                let protocol = self.registry.get_mut_protocol_context(global_id);
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

    /*
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
        };

        let context = self.create_custom_validation_context(metadata, custom);
        self.handle_ready_plugin(context)
    }*/

    fn load_plugins(&mut self) {
        match self.runtime.pop_ready_plugin() {
            Ok(Some(module)) => {
                let context = self.create_validation_context(module);
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
