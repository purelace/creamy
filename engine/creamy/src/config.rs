use garde::Validate;
use serde::Deserialize;

fn path_validator(value: &str, _ctx: &()) -> garde::Result {
    //std::fs::exists(value).map_err(|err| garde::Error::new(err))?;
    if std::fs::metadata(value)
        .map_err(garde::Error::new)?
        .is_dir()
    {
        return Ok(());
    }

    Err(garde::Error::new(format!("{value}: not a directory")))
}

#[derive(Deserialize, Validate, Clone)]
pub struct CreamyConfig {
    #[garde(range(min = 1))]
    parallel_downloads: u8,

    #[garde(custom(path_validator))]
    plugin_directory: String,
}

impl CreamyConfig {
    pub fn new(directory: impl Into<String>) -> Self {
        Self {
            parallel_downloads: 4,
            plugin_directory: directory.into(),
        }
    }

    pub const fn parallel_downloads(&self) -> usize {
        self.parallel_downloads as usize
    }

    pub const fn plugin_directory(&self) -> &str {
        self.plugin_directory.as_str()
    }
}
