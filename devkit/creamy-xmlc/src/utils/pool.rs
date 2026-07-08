#![cfg_attr(coverage_nightly, coverage(off))]
use creamy_utils::strpool::{NonZeroStringId, StringId, StringPool};

pub trait StringPoolResolver {
    fn resolve<'a>(&self, pool: &'a StringPool) -> &'a str;
}

pub trait StringPoolIntern {
    fn intern(&self, pool: &mut StringPool) -> StringId;
    fn intern_non_zero(&self, pool: &mut StringPool) -> NonZeroStringId;
}

impl StringPoolIntern for String {
    fn intern(&self, pool: &mut StringPool) -> StringId {
        pool.get_id_or_add(self)
    }

    fn intern_non_zero(&self, pool: &mut StringPool) -> NonZeroStringId {
        pool.get_non_zero_id(self)
    }
}

impl StringPoolIntern for &str {
    fn intern(&self, pool: &mut StringPool) -> StringId {
        pool.get_id_or_add(self)
    }

    fn intern_non_zero(&self, pool: &mut StringPool) -> NonZeroStringId {
        pool.get_non_zero_id(self)
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

impl StringPoolResolver for NonZeroStringId {
    fn resolve<'a>(&self, pool: &'a StringPool) -> &'a str {
        pool.get_string(self.as_string_id())
    }
}

impl StringPoolResolver for &NonZeroStringId {
    fn resolve<'a>(&self, pool: &'a StringPool) -> &'a str {
        pool.get_string(self.as_string_id())
    }
}
