use compiler_utils::strpool::StringId;
use enum_dispatch::enum_dispatch;

use crate::table::TypeTable;

#[enum_dispatch]
pub trait Layout {
    fn size_of(&self, tt: &TypeTable) -> usize;
    fn align_of(&self, tt: &TypeTable) -> usize;
}

#[enum_dispatch]
pub trait ResolvedType: Layout {
    fn name(&self) -> StringId;
}
