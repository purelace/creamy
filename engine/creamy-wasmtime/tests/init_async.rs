use std::{num::NonZeroU8, time::Duration};

use creamy::define_engine;
use creamy_wasmtime::{
    WTRuntime,
    loader::{Loader, config::LoaderConfig},
};

define_engine! {
    max_messages: 1024,
    max_groups: 32,
    max_subscribers: 32,
}

async fn init_engine() -> anyhow::Result<PluginEngine<WTRuntime, ()>> {
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

    let runtime = WTRuntime::new(HEAP_SIZE, loader)?;
    Ok(PluginEngine::new(runtime))
}

const ROUNDTRIP: NonZeroU8 = NonZeroU8::new(2).unwrap();

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preload_plugin() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt().with_target(false).try_init();

    let tempdir = tempfile::tempdir()?;
    let plugin_path = tempdir.path().join("ping.cmy");
    std::fs::copy("fixtures/output/ping.cmy", plugin_path.clone())?;
    unsafe { std::env::set_var("CREAMY_TEST_PLUGIN_DIR", tempdir.path().as_os_str()) };

    let mut engine = init_engine().await?;

    loop {
        engine.tick(ROUNDTRIP);
        if engine.loaded_plugins() == 2 {
            break;
        }
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn load_plugin() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt().with_target(false).try_init();

    let tempdir = tempfile::tempdir()?;
    let plugin_path = tempdir.path().join("ping.cmy");
    unsafe { std::env::set_var("CREAMY_TEST_PLUGIN_DIR", tempdir.path().as_os_str()) };

    let mut engine = init_engine().await?;
    let mut loaded = false;
    let mut buffer = Vec::new();

    loop {
        engine.tick(ROUNDTRIP);
        engine
            .runtime_mut()
            .loader_mut()
            .get_in_progress(&mut buffer);
        if engine.loaded_plugins() == 2 {
            break;
        }
        if !loaded {
            loaded = true;
            std::fs::copy("fixtures/output/ping.cmy", plugin_path.clone())?;
        }

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
