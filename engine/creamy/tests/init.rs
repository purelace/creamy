use std::{env, num::NonZeroU8};

use creamy::{
    core::{Constants, devkit::semver::Version},
    engine::PluginEngine,
};
use creamy_engine_core::{
    bus::define_bus_config, devkit::compiler::model::strpool::StringPoolResolver,
};
use creamy_loader::Loader;
use creamy_wasmtime::WasmtimeRuntime;
use pathenv::to_absolute_path;

const ROUNDTRIP: NonZeroU8 = NonZeroU8::new(2).unwrap();

fn run_custom_builder() -> anyhow::Result<()> {
    let mut cmd = std::process::Command::new("creamy");
    cmd.current_dir("../../examples/ping");
    cmd.arg("build");
    cmd.env_clear();

    if let Some(value) = env::var_os("TERM") {
        cmd.env("TERM", value);
    }

    if let Some(value) = env::var_os("COLORTERM") {
        cmd.env("COLORTERM", value);
    }

    if let Some(path) = env::var_os("PATH") {
        cmd.env("PATH", path);
    }

    if let Some(cargo_home) = env::var_os("CARGO_HOME") {
        cmd.env("CARGO_HOME", cargo_home);
    }
    if let Some(rustup_home) = env::var_os("RUSTUP_HOME") {
        cmd.env("RUSTUP_HOME", rustup_home);
    }

    if cfg!(windows) {
        if let Some(userprofile) = env::var_os("USERPROFILE") {
            cmd.env("USERPROFILE", userprofile);
        }
        if let Some(systemroot) = env::var_os("SystemRoot") {
            cmd.env("SystemRoot", systemroot);
        }
    } else if let Some(home) = env::var_os("HOME") {
        cmd.env("HOME", home);
    }

    cmd.spawn()?.wait()?;
    Ok(())
}

fn compile_plugin() -> anyhow::Result<()> {
    run_custom_builder()?;

    Ok(())
}

pub const M: usize = 1024;
pub const S: usize = 32;
define_bus_config! {
    Legacy,
    max_subscribers: 32,
    max_messages: 1024,
    max_groups: 32,
}

fn init_engine() -> anyhow::Result<PluginEngine<Legacy, WasmtimeRuntime, Loader, (), S, M>> {
    const HEAP_SIZE: u32 = 67_108_864;
    let runtime = WasmtimeRuntime::new(HEAP_SIZE)?;
    let loader = Loader::new(to_absolute_path("$CREAMY_TEST_PLUGIN_DIR").unwrap())?;

    let engine = PluginEngine::new(
        Constants {
            heap_size: HEAP_SIZE,
        },
        runtime,
        loader,
    );

    Ok(engine)
}

#[test]
fn init() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        //.with_max_level(LevelFilter::DEBUG)
        .with_target(true)
        .with_thread_names(false)
        .with_thread_ids(false)
        .try_init();

    let tempdir = tempfile::tempdir()?;
    compile_plugin()?;

    unsafe { std::env::set_var("CREAMY_TEST_PLUGIN_DIR", tempdir.path().as_os_str()) };
    let mut engine = init_engine()?;

    let plugin_path = tempdir.path().join("ping.cmy");
    //std::fs::copy(
    //    "/run/media/selrisu/SSD/fusionwm/creamy/target/creamy/ping.cmy",
    //    plugin_path.clone(),
    //)?;
    std::fs::copy("../../target/creamy/ping.cmy", plugin_path.clone())?;

    engine.tick(ROUNDTRIP);

    assert_eq!(engine.loaded_plugins(), 2);
    assert!(engine.errors().is_empty());

    let registry = engine.protocol_registry();
    let result = registry.get_protocol_context_by_str("ping");
    if let Some(model) = result {
        assert_eq!(model.model().name().resolve(registry.pool()), "ping");
        assert_eq!(model.model().version(), &Version::new(1, 0, 0));
    } else {
        panic!("result is_none() == true");
    }

    //engine.unload();

    Ok(())
}
