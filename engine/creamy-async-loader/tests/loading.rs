/*
use std::{num::NonZeroU8, time::Duration};

use creamy::define_engine;
use creamy_async_loader::{AsyncLoader, config::LoaderConfig};
use creamy_wasmtime::WasmtimeRuntime;

define_engine! {
    max_messages: 1024,
    max_groups: 32,
    max_subscribers: 32,
}

async fn init_engine() -> anyhow::Result<PluginEngine<WasmtimeRuntime, AsyncLoader, ()>> {
    const HEAP_SIZE: u32 = 67_108_864;
    let runtime = WasmtimeRuntime::new(HEAP_SIZE)?;
    let loader = AsyncLoader::new(
        LoaderConfig {
            parallel_downloads: 4,
            plugin_directory: "$CREAMY_TEST_PLUGIN_DIR".into(),
        }
        .into_valid()?,
        tokio::runtime::Handle::current(),
    )
    .await?;

    let engine = PluginEngine::new(runtime, loader);

    Ok(engine)
}

const ROUNDTRIP: NonZeroU8 = NonZeroU8::new(2).unwrap();

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preload_plugin() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt().with_target(false).try_init();

    let tempdir = tempfile::tempdir()?;
    let plugin_path = tempdir.path().join("ping.cmy");
    std::fs::copy("../../target/creamy/ping.cmy", plugin_path.clone())?;

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
    compile_plugin()?;

    unsafe { std::env::set_var("CREAMY_TEST_PLUGIN_DIR", tempdir.path().as_os_str()) };
    let mut engine = init_engine().await?;

    let plugin_path = tempdir.path().join("ping.cmy");
    std::fs::copy("../../target/creamy/ping.cmy", plugin_path.clone())?;

    loop {
        engine.tick(ROUNDTRIP);
        if engine.loaded_plugins() == 2 {
            break;
        }
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

*/
