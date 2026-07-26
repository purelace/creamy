use std::num::NonZeroUsize;

use creamy_sdk::{
    get_outgoing, initialize_buffers,
    logging::{LogReader, LogWriter},
    stream::{StreamId, StreamReader, StreamWriter},
    system::builtin::LogType,
};

#[test]
fn writer() -> Result<(), Box<dyn core::error::Error>> {
    initialize_buffers(NonZeroUsize::new(1024).unwrap())?;
    let mut writer = StreamWriter::new(LogWriter::new(LogType::Info), StreamId::new(1));
    writer.write("Hello, World!");

    let mut reader = StreamReader::new(StreamId::new(1), LogReader::default());

    Ok(())
}
