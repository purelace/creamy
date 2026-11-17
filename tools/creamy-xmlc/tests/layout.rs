use creamy_xmlc::{
    symbols::{
        ArraySymbol, FieldSymbol, FieldType, T_I16_ID, T_I128_ID, T_U8_ID, T_U32_ID, T_U64_ID,
    },
    utils::{
        Size,
        layout::{LayoutCalculator, LayoutItem, LayoutStep},
        strpool::StringId,
    },
};
use pretty_assertions::assert_eq;

#[test]
fn single_field() {
    let vec = [FieldSymbol {
        ident: StringId::new(1),
        kind: FieldType::Type(T_U64_ID),
        is_padding: false,
    }];

    let steps = LayoutCalculator::new(None, vec.iter().copied()).collect::<Vec<_>>();

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
        /* HEADER */
        FieldSymbol {
            ident: StringId::new(10),
            kind: FieldType::StaticArray(ArraySymbol {
                kind: T_U8_ID,
                len: Size::try_new(4)?,
            }),
            is_padding: false,
        },
        FieldSymbol {
            ident: StringId::new(1),
            kind: FieldType::StaticArray(ArraySymbol {
                kind: T_I16_ID,
                len: Size::try_new(2)?,
            }),
            is_padding: false,
        },
        FieldSymbol {
            ident: StringId::new(2),
            kind: FieldType::Type(T_U8_ID),
            is_padding: false,
        },
        FieldSymbol {
            ident: StringId::new(3),
            kind: FieldType::Type(T_U32_ID),
            is_padding: false,
        },
        FieldSymbol {
            ident: StringId::new(4),
            kind: FieldType::Type(T_I128_ID),
            is_padding: false,
        },
        FieldSymbol {
            ident: StringId::new(5),
            kind: FieldType::Type(T_U8_ID),
            is_padding: false,
        },
    ];

    let steps = LayoutCalculator::new(None, vec.iter().copied())
        .skip(1)
        .collect::<Vec<_>>();

    assert_eq!(
        steps,
        [
            (
                // a: [i16; 2],
                LayoutItem::Field(vec[1]),
                LayoutStep {
                    size: 4,
                    align: 2,
                    padding: 0,
                    offset: 4
                }
            ),
            (
                // b: u8,
                LayoutItem::Field(vec[2]),
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
                LayoutItem::Field(vec[3]),
                LayoutStep {
                    size: 4,
                    align: 4,
                    padding: 3,
                    offset: 12
                }
            ),
            (
                // d: i128,
                LayoutItem::Field(vec[4]),
                LayoutStep {
                    size: 16,
                    align: 16,
                    padding: 0,
                    offset: 16
                }
            ),
            (
                // e: u8,
                LayoutItem::Field(vec[5]),
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
