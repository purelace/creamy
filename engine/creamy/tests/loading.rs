use creamy::{
    config::{EngineConfig, GeneralConfig},
    engine::PluginEngine,
};

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
    let mut engine = PluginEngine::new(&EngineConfig {
        general: GeneralConfig {
            plugin_directory: "$CREAMY_TEST_PLUGIN_DIR".into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .await
    .unwrap();
    loop {
        engine.run();
        if engine.loaded_plugins() == 2 {
            break;
        }
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn load_plugin() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt().with_target(false).try_init();

    let tempdir = tempfile::tempdir()?;
    compile_plugin()?;

    unsafe { std::env::set_var("CREAMY_TEST_PLUGIN_DIR", tempdir.path().as_os_str()) };
    let mut engine = PluginEngine::new(&EngineConfig {
        general: GeneralConfig {
            plugin_directory: "$CREAMY_TEST_PLUGIN_DIR".into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .await
    .unwrap();

    let plugin_path = tempdir.path().join("ping.cmy");
    std::fs::copy("../../target/creamy/ping.cmy", plugin_path.clone())?;

    loop {
        engine.run();
        if engine.loaded_plugins() == 2 {
            break;
        }
    }

    Ok(())
}
