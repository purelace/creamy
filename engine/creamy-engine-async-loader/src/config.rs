use std::path::PathBuf;

use garde::Validate;
use serde::Deserialize;

use crate::utils::to_absolute_path;

#[allow(clippy::trivially_copy_pass_by_ref)]
fn check_path(path: &str, _: &()) -> garde::Result {
    match to_absolute_path(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(garde::Error::new(e)),
    }
}

#[derive(Deserialize, Validate, Clone)]
pub struct LoaderConfig {
    #[garde(range(min = 1))]
    pub parallel_downloads: u8,

    #[garde(custom(check_path))]
    pub plugin_directory: String,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            parallel_downloads: 4,
            plugin_directory: "plugins".into(),
        }
    }
}

impl LoaderConfig {
    pub fn into_valid(self) -> Result<ValidLoaderConfig, garde::Report> {
        self.validate()?;

        Ok(ValidLoaderConfig {
            parallel_downloads: self.parallel_downloads,
            plugin_directory: to_absolute_path(&self.plugin_directory).unwrap(),
        })
    }
}

#[derive(Clone)]
pub struct ValidLoaderConfig {
    pub(crate) parallel_downloads: u8,
    pub(crate) plugin_directory: PathBuf,
}
