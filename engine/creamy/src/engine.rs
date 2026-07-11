use garde::Validate;

use crate::{config::CreamyConfig, loader::PluginLoader};

pub struct PluginEngine {
    loader: PluginLoader,
}

#[allow(dead_code, unused)]
impl PluginEngine {
    pub fn new(config: CreamyConfig) -> Result<Self, Box<dyn std::error::Error>> {
        config.validate()?;
        Ok(Self {
            loader: PluginLoader::new(config)?,
        })
    }

    pub async fn run(&mut self) {
        self.loader.run().await;
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use creamy_devkit::BinaryPlugin;
    use memmap2::Mmap;

    use crate::{config::CreamyConfig, engine::PluginEngine};

    #[tokio::test]
    #[ignore = "Only runs locally"]
    async fn test() {
        let config = CreamyConfig::new("/home/selrisu/creamy_test");
        let mut engine = PluginEngine::new(config).unwrap();
        loop {
            engine.run().await;
        }
    }

    #[test]
    #[ignore = "Only runs locally"]
    fn mmap() {
        let file = File::open("/home/selrisu/creamy_test/ping.cmy").unwrap();

        let mmap = unsafe { Mmap::map(&file).unwrap() };
    }
}
