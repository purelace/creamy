use std::collections::HashMap;

use creamy_engine_core::devkit::{utils::strpool::StringPool, xmlc::ProtocolDefinition};

#[derive(Default)]
pub struct ProtocolRegistry {
    pool: StringPool,
    map: HashMap<Box<str>, ProtocolDefinition>,
}

impl ProtocolRegistry {
    pub const fn pool(&self) -> &StringPool {
        &self.pool
    }

    pub fn replace_strings(&mut self, pool: &StringPool, definition: &mut ProtocolDefinition) {
        definition.replace_identifiers(pool, &mut self.pool);
    }

    pub fn compare_models(&self, path: &str, definition: &ProtocolDefinition) -> bool {
        match self.map.get(path) {
            Some(def) => def == definition,
            None => false,
        }
    }

    pub fn declare_protocol(&mut self, path: impl Into<Box<str>>, definition: ProtocolDefinition) {
        let path = path.into();
        tracing::info!(
            "Protocol '{}@{}' declared",
            definition.name().resolve(&self.pool),
            definition.version()
        );
        //TODO: chech if contains
        assert!(self.map.insert(path, definition).is_none());
    }

    pub fn get_model(&self, path: &str) -> Option<&ProtocolDefinition> {
        self.map.get(path)
    }

    //pub fn is_provided_and(&mut self, path: &str, mut f: impl FnMut(Version)) -> Result<bool, > {
    //    match self.map.get(path) {
    //        Some(def) => {
    //            f(def.version());
    //            true
    //        }
    //        None => false,
    //    }
    //}
}
