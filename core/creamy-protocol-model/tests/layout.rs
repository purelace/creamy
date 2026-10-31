use creamy_protocol_model::{
    Size,
    constraints::HEADER_BYTES,
    layout::{LayoutCalculator, LayoutItem, LayoutStep},
    symbols::{
        ArraySymbol, FieldSymbol, FieldType, T_I16_ID, T_I128_ID, T_U8_ID, T_U32_ID, T_U64_ID,
    },
};
use creamy_utils::strpool::StringId;
use pretty_assertions::assert_eq;

#[test]
fn single_field() {
    let vec = [FieldSymbol::new(
        StringId::new(1),
        FieldType::Type(T_U64_ID),
        false,
    )];

    let steps = LayoutCalculator::new(0, None, vec.iter().copied()).collect::<Vec<_>>();

    assert_eq!(
        steps,
        [(
            LayoutItem::Field(vec[0]),
            LayoutStep {
                size: 8,
                align: 8,
                padding: 0,
                offset: 0
            }
        ),]
    );
}

/*
#[repr(C)]
struct Test {
    header: [u8; 4],
    a: [i16; 2],
    b: u8,
    _pad: [u8; 3],
    c: u32,
    d: i128,
    e: u8,
    _pad1: [u8; 15],
}
*/
#[test]
#[allow(clippy::too_many_lines)]
fn validate() -> Result<(), Box<dyn core::error::Error>> {
    let vec = [
        FieldSymbol::new(
            StringId::new(1),
            FieldType::Array(ArraySymbol::new(T_I16_ID, Size::try_new(2)?)),
            false,
        ),
        FieldSymbol::new(StringId::new(2), FieldType::Type(T_U8_ID), false),
        FieldSymbol::new(StringId::new(3), FieldType::Type(T_U32_ID), false),
        FieldSymbol::new(StringId::new(4), FieldType::Type(T_I128_ID), false),
        FieldSymbol::new(StringId::new(5), FieldType::Type(T_U8_ID), false),
    ];

    let steps = LayoutCalculator::new(HEADER_BYTES, None, vec.iter().copied()).collect::<Vec<_>>();

    assert_eq!(
        steps,
        [
            (
                // a: [i16; 2],
                LayoutItem::Field(vec[0]),
                LayoutStep {
                    size: 4,
                    align: 2,
                    padding: 0,
                    offset: 4
                }
            ),
            (
                // b: u8,
                LayoutItem::Field(vec[1]),
                LayoutStep {
                    size: 1,
                    align: 1,
                    padding: 0,
                    offset: 8
                }
            ),
            (
                // _pad: [u8; 3],
                LayoutItem::Padding {
                    index: 0,
                    size: Size::try_new(3)?
                },
                LayoutStep {
                    size: 3,
                    align: 1,
                    padding: 0,
                    offset: 9
                }
            ),
            (
                // c: u32,
                LayoutItem::Field(vec[2]),
                LayoutStep {
                    size: 4,
                    align: 4,
                    padding: 3,
                    offset: 12
                }
            ),
            (
                // d: i128,
                LayoutItem::Field(vec[3]),
                LayoutStep {
                    size: 16,
                    align: 16,
                    padding: 0,
                    offset: 16
                }
            ),
            (
                // e: u8,
                LayoutItem::Field(vec[4]),
                LayoutStep {
                    size: 1,
                    align: 1,
                    padding: 0,
                    offset: 32
                }
            ),
            (
                // _pad1: [u8; 15],
                LayoutItem::Padding {
                    index: 1,
                    size: Size::try_new(15)?
                },
                LayoutStep {
                    size: 15,
                    align: 1,
                    padding: 0,
                    offset: 33
                }
            ),
        ]
    );

    Ok(())
}
