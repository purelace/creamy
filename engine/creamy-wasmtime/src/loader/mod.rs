#[cfg(feature = "async-loader")]
mod r#async;
#[cfg(not(feature = "async-loader"))]
mod sync;

pub mod config;

use creamy_engine_core::package::BinaryPlugin;

#[cfg(feature = "async-loader")]
pub use crate::loader::r#async::Loader;
#[cfg(not(feature = "async-loader"))]
pub use crate::loader::sync::Loader;

pub trait PluginLoader {
    fn tick(&mut self);
    fn take_loaded_package(&mut self) -> Option<BinaryPlugin>;
}
