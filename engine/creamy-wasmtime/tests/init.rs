#![cfg(not(feature = "async-loader"))]

use std::num::NonZeroU8;

use creamy::{core::package::version::Version, define_engine};
use creamy_engine_core::package::pool::StringPoolResolver;
use creamy_wasmtime::{
    WTRuntime,
    loader::{Loader, config::LoaderConfig},
};

const ROUNDTRIP: NonZeroU8 = NonZeroU8::new(2).unwrap();

define_engine! {
    max_messages: 1024,
    max_groups: 32,
    max_subscribers: 32,
}

fn init_engine() -> anyhow::Result<PluginEngine<WTRuntime, ()>> {
    const HEAP_SIZE: u32 = 67_108_864;
    let cfg = LoaderConfig {
        plugin_directory: "$CREAMY_TEST_PLUGIN_DIR".into(),
    };
    let runtime = WTRuntime::new(HEAP_SIZE, Loader::new(cfg.into_valid()?)?)?;
    Ok(PluginEngine::new(runtime))
}

#[test]
fn init() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_names(false)
        .with_thread_ids(false)
        .try_init();

    let tempdir = tempfile::tempdir()?;
    unsafe { std::env::set_var("CREAMY_TEST_PLUGIN_DIR", tempdir.path().as_os_str()) };
    let mut engine = init_engine()?;

    let plugin_path = tempdir.path().join("ping.cmy");
    std::fs::copy("fixtures/output/ping.cmy", plugin_path.clone())?;
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
