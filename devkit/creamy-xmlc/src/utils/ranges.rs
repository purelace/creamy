#[macro_export]
macro_rules! define_readonly_range {
    (
        struct $name:ident {
            start: $start_type: ty,
            len: $len_type: ty $(,)?
        }
    ) => {
        $crate::define_readonly_struct! {
            struct $name {
                start: $start_type,
                len: $len_type,
            }
        }

        impl $crate::utils::Range for $name {
            fn as_range(&self) -> core::ops::Range<usize> {
                let start = self.start as usize;
                let len = self.len as usize;
                start..start + len
            }
        }
    };
}

pub trait Range {
    fn as_range(&self) -> core::ops::Range<usize>;
}

define_readonly_range! {
    struct GroupsRange {
        start: u8,
        len: u8,
    }
}

define_readonly_range! {
    struct EnumsRange {
        start: u16,
        len: u16,
    }
}

define_readonly_range! {
    struct StructsRange {
        start: u16,
        len: u16,
    }
}

define_readonly_range! {
    struct MessagesRange {
        start: u16,
        len: u16,
    }
}

define_readonly_range! {
    struct StreamsRange {
        start: u16,
        len: u16,
    }
}

define_readonly_range! {
    struct StreamsFieldsRange {
        start: u16,
        len: u16,
    }
}

//TODO: pack range
define_readonly_range! {
    struct FieldsRange {
        start: u16,
        len: u8,
    }
}

//TODO: pack range
define_readonly_range! {
    struct VariantsRange {
        start: u16,
        len: u16,
    }
}

//TODO: pack range
define_readonly_range! {
    struct OptionsRange {
        start: u16,
        len: u16,
    }
}

//TODO: pack range
define_readonly_range! {
    struct BitsetValuesRange {
        start: u16,
        len: u16,
    }
}

define_readonly_range! {
    struct FlagsRange {
        start: u16,
        len: u16,
    }
}

define_readonly_range! {
    struct BitsetsRange {
        start: u16,
        len: u16,
    }
}

define_readonly_range! {
    struct TypesRange {
        start: u16,
        len: u16,
    }
}
