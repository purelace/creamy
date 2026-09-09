#![cfg_attr(coverage_nightly, coverage(off))]
use creamy_utils::strpool::{StringId, StringPool};

pub trait StringPoolResolver {
    fn resolve<'a>(&self, pool: &'a StringPool) -> &'a str;
}

pub trait StringPoolIntern {
    fn intern(&self, pool: &mut StringPool) -> StringId;
}

impl StringPoolIntern for String {
    fn intern(&self, pool: &mut StringPool) -> StringId {
        pool.get_id_or_add(self)
    }
}

impl StringPoolIntern for &str {
    fn intern(&self, pool: &mut StringPool) -> StringId {
        pool.get_id_or_add(self)
    }
}

impl StringPoolResolver for StringId {
    fn resolve<'a>(&self, pool: &'a StringPool) -> &'a str {
        pool.get_string(*self)
    }
}

impl StringPoolResolver for &StringId {
    fn resolve<'a>(&self, pool: &'a StringPool) -> &'a str {
        pool.get_string(**self)
    }
}
