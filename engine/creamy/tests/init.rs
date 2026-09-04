use std::num::NonZeroU8;

use creamy::{
    core::{
        Constants,
        devkit::{semver::Version, xmlc::StringPoolResolver},
    },
    engine::PluginEngine,
};
use creamy_loader::Loader;
use creamy_wasmtime::WasmtimeRuntime;
use pathenv::to_absolute_path;

const ROUNDTRIP: NonZeroU8 = NonZeroU8::new(2).unwrap();

fn compile_plugin() -> anyhow::Result<()> {
    std::process::Command::new("creamy")
        .arg("build")
        //.current_dir("/mnt/ssd/fusionwm/creamy/examples/ping")
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

fn init_engine() -> anyhow::Result<PluginEngine<WasmtimeRuntime, Loader, ()>> {
    const HEAP_SIZE: u32 = 67_108_864;
    let runtime = WasmtimeRuntime::new(HEAP_SIZE)?;
    let loader = Loader::new(to_absolute_path("$CREAMY_TEST_PLUGIN_DIR").unwrap())?;

    let engine = PluginEngine::new(
        Constants {
            heap_size: HEAP_SIZE,
        },
        runtime,
        loader,
        (),
    );

    Ok(engine)
}

#[test]
fn init() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_names(false)
        .with_thread_ids(false)
        .try_init();

    let tempdir = tempfile::tempdir()?;
    compile_plugin()?;

    unsafe { std::env::set_var("CREAMY_TEST_PLUGIN_DIR", tempdir.path().as_os_str()) };
    let mut engine = init_engine()?;

    let plugin_path = tempdir.path().join("ping.cmy");
    std::fs::copy("../../target/creamy/ping.cmy", plugin_path.clone())?;
    //std::fs::copy(
    //    "/mnt/ssd/fusionwm/creamy/target/creamy/ping.cmy",
    //    plugin_path.clone(),
    //)?;

    engine.tick(ROUNDTRIP);

    assert_eq!(engine.loaded_plugins(), 2);

    let registry = engine.protocol_registry();
    let result = registry.get_model("ping");
    if let Some(model) = result {
        assert_eq!(model.name().resolve(registry.pool()), "ping");
        assert_eq!(model.version(), &Version::new(1, 0, 0));
    } else {
        panic!("result is_none() == true");
    }
    Ok(())
}
