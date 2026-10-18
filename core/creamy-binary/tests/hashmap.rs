use creamy_binary::{BinRead, BinWrite, BinaryReader, BinaryWriter};

macro_rules! map {
    (
        $path: path,
        $(
            [$key:expr, $value:expr]
        ),*
        $(,)?

    ) => {{
        let mut map = <$path>::new();
        $(
            map.insert($key, $value);
        )*
        map
    }};
}

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Default)]
struct Maps {
    std_map: std::collections::HashMap<String, i8>,
    hashbrown_map: hashbrown::HashMap<String, i64>,
}

#[test]
fn serialization() -> Result<(), Box<dyn core::error::Error>> {
    let maps = Maps {
        std_map: map! {
            std::collections::HashMap::<String, i8>,
            [String::new(), 0],
            ["positive_max".into(), i8::MAX],
            ["negative_min".into(), i8::MIN],
            ["standard_value".into(), 42],
            ["small_negative".into(), -7],
            ["special_char_#$@".into(), 100],
            ["12345".into(), -1],
        },

        hashbrown_map: map! {
            hashbrown::HashMap::<String, i64>,
            ["String".into(), 1],
            ["zero".into(), 0],
            ["huge_positive".into(), i64::MAX],
            ["huge_negative".into(), i64::MIN],
            ["timestamp_example".into(), 1_718_000_000],
            ["id_example".into(), 9_876_543_210],
            ["-".into(), -999_999],
        },
    };

    let mut buffer = Vec::with_capacity(1024);
    maps.write_ne(&mut buffer)?;
    let result = Maps::read_ne(&mut buffer.as_slice())?;

    assert_eq!(maps, result);

    Ok(())
}
