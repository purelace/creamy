use creamy_engine_async_loader::config::LoaderConfig;
use garde::Validate;
use serde::{Deserialize, Serialize};

/*
#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct PerformanceConfig {
    #[garde(skip)]
    pub heap_size: u32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            heap_size: 67_108_864,
        }
    }
}

#[derive(Default, Deserialize, Validate, Clone)]
pub struct EngineConfig {
    #[garde(dive)]
    pub loader: LoaderConfig,

    #[garde(dive)]
    pub performance: PerformanceConfig,
}
*/
