use alloc::boxed::Box;

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Protocol model `{target}` not found")]
    ProtocolModelNotFound { target: Box<str> },
    #[error("Protocol `{target}` has already been declared")]
    ProtocolDeclaredAlready { target: Box<str> },
    #[error("Provided `{target}` protocol differs ")]
    DifferentProtocol { target: Box<str> },
    #[error("Protocol '{protocol}' is already provided by '{provider}'")]
    ProtocolAlreadyProvided {
        protocol: Box<str>,
        provider: Box<str>,
    },
    #[error("Protocol '{protocol}' is busy")]
    ProtocolIsBusy { protocol: Box<str> },

    #[error("{0}")]
    Binary(#[from] creamy_binary::ReadError<core::convert::Infallible>),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Bus(#[from] creamy_engine_core::bus::BusError),

    #[error("{0}")]
    Other(Box<dyn core::error::Error>),
}
