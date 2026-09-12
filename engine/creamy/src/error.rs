use alloc::boxed::Box;

use creamy_engine_core::devkit::semver::Version;

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Protocol model `{target_model}@{version}` not found")]
    ProtocolModelNotFound {
        target_model: Box<str>,
        version: Version,
    },
    #[error("Protocol `{target}@{version}` has already been declared")]
    ProtocolDeclaredAlready { target: Box<str>, version: Version },
    #[error(
        "Protocols `{target_model}@{version_a}` and `{target_model}@{version_b}` have different models"
    )]
    DifferentProtocolModels {
        target_model: Box<str>,
        version_a: Version,
        version_b: Version,
    },
    #[error("Group '{group}' from protocol '{protocol}' is already provided by '{provider}'")]
    GroupAlreadyProvided {
        group: Box<str>,
        protocol: Box<str>,
        provider: Box<str>,
    },
    #[error("{0}")]
    Binary(#[from] creamy_engine_core::devkit::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Bus(#[from] creamy_engine_core::bus::BusError),

    #[error("{0}")]
    Other(Box<dyn core::error::Error>),
}
