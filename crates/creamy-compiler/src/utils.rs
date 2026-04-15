use compiler_utils::strpool::{StringId, StringPool};

pub trait StringPoolResolver {
    fn resolve<'a>(&'a self, pool: &'a StringPool) -> &'a str;
}

pub trait StringPoolIntern {
    fn intern(&self, pool: &mut StringPool) -> StringId;
}

impl StringPoolIntern for String {
    fn intern(&self, pool: &mut StringPool) -> StringId {
        pool.get_id(self)
    }
}

impl StringPoolIntern for &str {
    fn intern(&self, pool: &mut StringPool) -> StringId {
        pool.get_id(self)
    }
}

impl StringPoolResolver for StringId {
    fn resolve<'a>(&'a self, pool: &'a StringPool) -> &'a str {
        pool.get_string(*self)
    }
}

impl StringPoolResolver for &StringId {
    fn resolve<'a>(&'a self, pool: &'a StringPool) -> &'a str {
        pool.get_string(**self)
    }
}
