use cbus_core::{
    Subscriber, SubscriberId,
    buffer::{IncBuf, OutBuf},
};
use creamy_sdk::{
    api::handle_incoming,
    dispatcher::dispatch_message,
    get_incoming, get_outgoing, initialize_buffers,
    logging::{LogReader, LogWriter},
    stream::{StreamId, StreamReader, StreamWriter},
    system::builtin::LogType,
};

#[test]
fn writer() -> Result<(), Box<dyn core::error::Error>> {
    //initialize_te(sub)?;

    Ok(())
}
