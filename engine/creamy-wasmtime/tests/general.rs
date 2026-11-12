#![allow(unused)]
#![allow(clippy::missing_errors_doc)]

use std::path::PathBuf;

use anyhow::Context;
use creamy::define_engine;
use creamy_wasmtime::{
    WTRuntime,
    loader::{Loader, config::LoaderConfig},
};

define_engine! {
    max_messages: 1024,
    max_protocols: 32,
    max_subscribers: 32,
}

pub async fn init_engine() -> anyhow::Result<PluginEngine<WTRuntime, ()>> {
    const HEAP_SIZE: u32 = 67_108_864;
    let loader = Loader::new(
        LoaderConfig {
            parallel_downloads: 4,
            plugin_directory: "$CREAMY_TEST_PLUGIN_DIR".into(),
        }
        .into_valid()?,
        tokio::runtime::Handle::current(),
    )
    .await?;

    Ok(PluginEngine::new(WTRuntime::new(HEAP_SIZE, loader)?))
}

#[must_use]
pub fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/output")
}

pub struct TestContext {
    _tempdir: tempfile::TempDir,
    pub plugin_path: PathBuf,
    pub fixtures: PathBuf,
}

impl TestContext {
    pub fn new() -> anyhow::Result<Self> {
        let _ = tracing_subscriber::fmt().with_target(false).try_init();

        let tempdir = tempfile::tempdir()?;

        let fixtures = fixtures_path();
        unsafe { std::env::set_var("CREAMY_TEST_PLUGIN_DIR", tempdir.path().as_os_str()) };

        Ok(Self {
            plugin_path: tempdir.path().into(),
            _tempdir: tempdir,
            fixtures,
        })
    }

    pub fn copy_to_plugin_dir(&self, other: &str) -> anyhow::Result<()> {
        std::fs::copy(self.fixtures.join(other), self.plugin_path.join(other)).with_context(
            || {
                format!(
                    "From: {}. To: {}",
                    self.fixtures.display(),
                    self.plugin_path.display()
                )
            },
        )?;
        Ok(())
    }
}
