use creamy_binary::{BinRead, BinWrite, BinaryReader, DynBinaryWrite, DynWrite, DynWriter};

trait Storage: DynBinaryWrite {}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq, Eq)]
struct UnsignedIntegerStorage(Vec<u64>);

impl Storage for UnsignedIntegerStorage {}

macro_rules! test_with {
    ($ident: ident, $write_method: ident, $read_method: ident) => {
        fn $ident(
            buffer: &mut Vec<u8>,
            storage: &UnsignedIntegerStorage,
        ) -> Result<UnsignedIntegerStorage, Box<dyn core::error::Error>> {
            let mut writer_instance = buffer.as_dyn_writer();
            let mut dyn_writer = DynWriter::new(&mut writer_instance);

            let dyn_instance: Box<dyn Storage> = Box::new(storage.clone());
            dyn_instance.$write_method(&mut dyn_writer)?;
            Ok(UnsignedIntegerStorage::$read_method(
                &mut buffer.as_slice(),
            )?)
        }
    };
}

test_with!(native, dyn_write_ne, read_ne);
test_with!(little, dyn_write_le, read_le);
test_with!(big, dyn_write_be, read_be);

#[test]
fn serialization() -> Result<(), Box<dyn core::error::Error>> {
    let instance = UnsignedIntegerStorage(vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144]);
    let mut buffer = Vec::with_capacity(1024);

    assert_eq!(native(&mut buffer, &instance)?, instance);
    buffer.clear();
    assert_eq!(little(&mut buffer, &instance)?, instance);
    buffer.clear();
    assert_eq!(big(&mut buffer, &instance)?, instance);

    Ok(())
}
