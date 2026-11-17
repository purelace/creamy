use super::Context;
use crate::{
    MAX_PAYLOAD,
    symbols::{ArraySymbol, FieldSymbol, FieldType, SpecialType, T_U8_ID},
    utils::{Size, strpool::StringPoolIntern},
};

pub fn dst(context: &mut Context) -> FieldSymbol {
    FieldSymbol {
        ident: "dst".intern(&mut context.pool),
        kind: FieldType::Special(SpecialType::SubscriberId),
        is_padding: false,
    }
}

pub fn protocol(context: &mut Context) -> FieldSymbol {
    FieldSymbol {
        ident: "protocol".intern(&mut context.pool),
        kind: FieldType::Special(SpecialType::ProtocolId),
        is_padding: false,
    }
}

pub fn src(context: &mut Context) -> FieldSymbol {
    FieldSymbol {
        ident: "src".intern(&mut context.pool),
        kind: FieldType::Special(SpecialType::SubscriberId),
        is_padding: false,
    }
}

pub fn kind(context: &mut Context) -> FieldSymbol {
    FieldSymbol {
        ident: "kind".intern(&mut context.pool),
        kind: FieldType::Type(T_U8_ID),
        is_padding: false,
    }
}

pub fn message_header(context: &mut Context) -> [FieldSymbol; 4] {
    [dst(context), protocol(context), src(context), kind(context)]
}

pub fn stream_message_fields(context: &mut Context) -> Box<[FieldSymbol]> {
    Box::new([
        dst(context),
        protocol(context),
        src(context),
        kind(context),
        FieldSymbol {
            ident: "meta".intern(&mut context.pool),
            kind: FieldType::Type(T_U8_ID),
            is_padding: false,
        },
        FieldSymbol {
            ident: "data".intern(&mut context.pool),
            kind: FieldType::StaticArray(ArraySymbol {
                kind: T_U8_ID,
                len: Size::try_new(MAX_PAYLOAD - 1).expect("Unreachable!"),
            }),
            is_padding: false,
        },
    ])
}
