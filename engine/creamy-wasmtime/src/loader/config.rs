use std::path::PathBuf;

use garde::Validate;
use pathenv::to_absolute_path;
use serde::Deserialize;

#[allow(clippy::trivially_copy_pass_by_ref)]
fn check_path(path: &str, _: &()) -> garde::Result {
    match to_absolute_path(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(garde::Error::new(e)),
    }
}

#[derive(Deserialize, Validate, Clone)]
pub struct LoaderConfig {
    #[cfg(feature = "async-loader")]
    #[cfg_attr(feature = "async-loader", garde(range(min = 1)))]
    pub parallel_downloads: u8,

    #[garde(custom(check_path))]
    pub plugin_directory: String,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            #[cfg(feature = "async-loader")]
            parallel_downloads: 4,
            plugin_directory: "plugins".into(),
        }
    }
}

impl LoaderConfig {
    /// Validates `Self`, returning an `Err` with an aggregate of all errors if
    /// the validation failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if config is not valid.
    #[allow(clippy::missing_panics_doc)]
    pub fn into_valid(self) -> Result<ValidLoaderConfig, garde::Report> {
        self.validate()?;

        Ok(ValidLoaderConfig {
            #[cfg(feature = "async-loader")]
            parallel_downloads: self.parallel_downloads,
            plugin_directory: to_absolute_path(&self.plugin_directory)
                .expect("plugin_directory must be valid because self.validate() already passed"),
        })
    }
}

#[derive(Clone)]
pub struct ValidLoaderConfig {
    #[cfg(feature = "async-loader")]
    pub(crate) parallel_downloads: u8,
    pub(crate) plugin_directory: PathBuf,
}
