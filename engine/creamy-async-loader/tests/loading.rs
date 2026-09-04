use std::{num::NonZeroU8, time::Duration};

use creamy::engine::PluginEngine;
use creamy_async_loader::{AsyncLoader, config::LoaderConfig};
use creamy_engine_core::Constants;
use creamy_wasmtime::WasmtimeRuntime;

fn compile_plugin() -> anyhow::Result<()> {
    std::process::Command::new("creamy")
        .arg("build")
        .current_dir("../../examples/ping")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("CARGO_LLVM_COV")
        .env_remove("__CARGO_LLVM_COV_RUSTC_WRAPPER")
        .env_remove("__CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES")
        .env_remove("__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS")
        .spawn()?
        .wait()?;

    Ok(())
}

async fn init_engine() -> anyhow::Result<PluginEngine<WasmtimeRuntime, AsyncLoader>> {
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

    let engine = PluginEngine::new(
        Constants {
            heap_size: HEAP_SIZE,
        },
        runtime,
        loader,
    );

    Ok(engine)
}

const ROUNDTRIP: NonZeroU8 = NonZeroU8::new(2).unwrap();

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preload_plugin() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt().with_target(false).try_init();

    let tempdir = tempfile::tempdir()?;
    {
        compile_plugin()?;
        let plugin_path = tempdir.path().join("ping.cmy");
        std::fs::copy("../../target/creamy/ping.cmy", plugin_path.clone())?;
    }

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
