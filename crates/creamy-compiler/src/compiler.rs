use compiler_utils::strpool::StringPool;

use crate::{model::types::Protocol, resolver::Resolver, tree::types::ProtocolTree};

pub struct ProtocolCompiler {
    resolver: Resolver,
}

impl Default for ProtocolCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolCompiler {
    pub fn new() -> Self {
        Self {
            resolver: Resolver::new(),
        }
    }

    pub fn compile(&mut self, content: &str) -> Protocol {
        let document = roxmltree::Document::parse(content).unwrap();
        let root = document.root().first_child().unwrap();
        let tree = ProtocolTree::new(root, &mut self.resolver.pool);
        self.resolver.run(tree)
    }

    pub fn get_pool(&self) -> &StringPool {
        &self.resolver.pool
    }
}
