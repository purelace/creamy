use creamy_protocol_model::{
    Size,
    constraints::HEADER_BYTES,
    layout::{LayoutCalculator, compute_layout},
    symbols::{ArraySymbol, FieldSymbol, FieldType, T_I16_ID, T_I128_ID, T_U8_ID, T_U32_ID},
};
use creamy_utils::strpool::StringId;

#[test]
fn validate() -> Result<(), Box<dyn core::error::Error>> {
    let vec = [
        FieldSymbol::new(
            StringId::new(1),
            FieldType::Array(ArraySymbol::new(T_I16_ID, Size::try_new(2)?)),
        ),
        FieldSymbol::new(StringId::new(2), FieldType::Type(T_U8_ID)),
        FieldSymbol::new(StringId::new(3), FieldType::Type(T_U32_ID)),
        FieldSymbol::new(StringId::new(4), FieldType::Type(T_I128_ID)),
    ];

    let steps = LayoutCalculator::new(HEADER_BYTES, vec.iter().copied()).collect::<Vec<_>>();

    assert_eq!(steps.len(), vec.len());

    let step = steps[0].1;
    assert_eq!(step.size, 4);
    assert_eq!(step.align, 2);
    assert_eq!(step.padding, 0);
    assert_eq!(step.offset, 4);

    let step = steps[1].1;
    assert_eq!(step.size, 1);
    assert_eq!(step.align, 1);
    assert_eq!(step.padding, 0);
    assert_eq!(step.offset, 8);

    let step = steps[2].1;
    assert_eq!(step.size, 4);
    assert_eq!(step.align, 4);
    assert_eq!(step.padding, 3);
    assert_eq!(step.offset, 12);

    let step = steps[3].1;
    assert_eq!(step.size, 16);
    assert_eq!(step.align, 8);
    assert_eq!(step.padding, 0);
    assert_eq!(step.offset, 16);

    Ok(())
}

/*
#[test]
fn panic() {
    let collection = core::iter::repeat_n(
        FieldSymbol::new(StringId::new(4), FieldType::Type(T_I128_ID)),
        2048,
    )
    .collect::<Box<[_]>>();

    compute_layout(0, &collection, |_, _| {});
}
*/
