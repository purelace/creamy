use creamy_binary::{BinaryReader, BinaryWriter};
use creamy_binary_derive::{BinRead, BinWrite};

#[derive(BinWrite, BinRead, Debug, PartialEq, Eq)]
#[repr(u8)]
enum CustomOption {
    None = 0,
    Some { value: i16 } = 1,
    Other(i64) = 100,
}

#[derive(BinWrite, BinRead, Debug, PartialEq, Eq)]
#[repr(u8)]
enum LogType {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq)]
struct Sample {
    kind: LogType,
    option0: CustomOption,
    option1: CustomOption,
}

#[test]
fn serialization() -> Result<(), Box<dyn core::error::Error>> {
    const SAMPLE: Sample = Sample {
        kind: LogType::Warning,
        option0: CustomOption::Some { value: 1337 },
        option1: CustomOption::Other(0xdead_beef),
    };

    let mut buffer = Vec::with_capacity(1024);
    SAMPLE.write_ne(&mut buffer)?;

    assert_eq!(buffer.len(), 13);

    let result = Sample::read_ne(&mut buffer.as_slice())?;
    assert_eq!(SAMPLE, result);
    Ok(())
}
