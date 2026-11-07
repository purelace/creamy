use creamy_engine_core::{Runtime, bus::core::Subscriber, package::manifest::Package};
use creamy_sdk::state::PluginState;

use crate::{engine::HostSubscriber, system::SystemPlugin};

pub enum SubscriberType<R, H, const MM: usize, const MS: usize>
where
    R: Runtime<MM>,
    H: HostSubscriber,
{
    System(SystemPlugin<MM, MS>),
    Custom(PluginState<H>),
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
    pub(crate) manifest: Package,
    pub(crate) kind: SubscriberType<R, H, MM, MS>,
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
