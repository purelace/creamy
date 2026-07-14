use garde::Validate;
use serde::{Deserialize, Serialize};

use crate::utils::to_absolute_path;

#[allow(clippy::trivially_copy_pass_by_ref)]
fn check_path(path: &str, _: &()) -> garde::Result {
    match to_absolute_path(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(garde::Error::new(e)),
    }
}

#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct GeneralConfig {
    #[garde(range(min = 1))]
    pub parallel_downloads: u8,

    #[garde(custom(check_path))]
    pub plugin_directory: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            parallel_downloads: 4,
            plugin_directory: "plugins".into(),
        }
    }
}

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
    pub general: GeneralConfig,

    #[garde(dive)]
    pub performance: PerformanceConfig,
}
