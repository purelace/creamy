#[macro_export]
macro_rules! define_readonly_struct {
    {
        [no_brw]
        [element($size:expr, $type:ty)]
        struct $name:ident {
            $(
                $([Documentation($doc_str:expr)])?
                $field:ident: $field_type:ty,
            )* $(,)?
        }
    } => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name {
            $(
                $(
                    #[doc = $doc_str]
                )?
                $field:$field_type,
            )*
        }

        $crate::define_readonly_struct!(@impl_vector_element $name $size $type);
        $crate::define_readonly_struct!(@impl_methods $name { $($field: $field_type,)* });
    };
    {
        [no_brw]
        struct $name:ident {
            $(
                $([Documentation($doc_str:expr)])?
                $field:ident: $field_type:ty,
            )* $(,)?
        }
    } => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name {
            $(
                $(
                    #[doc = $doc_str]
                )?
                $field:$field_type,
            )*
        }

        $crate::define_readonly_struct!(@impl_methods $name { $($field: $field_type,)* });
    };

    {
        [element($size:expr, $type:ty)]
        struct $name:ident {
            $(
                $([Documentation($doc_str:expr)])?
                $field:ident: $field_type:ty,
            )* $(,)?
        }
    } => {
        #[derive(::binrw::BinWrite, ::binrw::BinRead, Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name {
            $(
                $(
                    #[doc = $doc_str]
                )?
                $field:$field_type,
            )*
        }

        $crate::define_readonly_struct!(@impl_vector_element $name $size $type);
        $crate::define_readonly_struct!(@impl_methods $name { $($field: $field_type,)* });
    };

    {
        struct $name:ident {
            $(
                $([Documentation($doc_str:expr)])?
                $field:ident: $field_type:ty,
            )* $(,)?
        }
    } => {
        #[derive(::binrw::BinWrite, ::binrw::BinRead, Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name {
            $(
                $(
                    #[doc = $doc_str]
                )?
                $field:$field_type,
            )*
        }

        $crate::define_readonly_struct!(@impl_methods $name { $($field: $field_type,)* });
    };

    {
        @impl_vector_element $name:ident $size:tt $type:ty
    } => {
        impl $crate::utils::VectorElement for $name {
            const MAX_SIZE: usize = $size;
            type RangeType = $type;
        }
    };

    {
        @impl_methods $name:ident { $($field:ident: $field_type:ty,)* }
    } => {
        impl $name {
            pub const fn new($($field: $field_type,)*) -> Self {
                Self { $($field,)* }
            }

            $(
                #[allow(clippy::len_without_is_empty)]
                pub const fn $field(&self) -> $field_type {
                    self.$field
                }
            )*
        }
    };
}
