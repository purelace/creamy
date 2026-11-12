//TODO: move this tests to engine tests
mod general;

use std::{fs::File, num::NonZeroU8};

use creamy_binary::BinaryReader;
use creamy_engine_core::package::BinaryPlugin;
use creamy_wasmtime::WTRuntime;

use self::general::{PluginEngine, TestContext, fixtures_path, init_engine};

const ROUNDTRIP: NonZeroU8 = NonZeroU8::new(4).unwrap();

fn read_plugin(plugin: &str) -> anyhow::Result<BinaryPlugin> {
    let path = fixtures_path().join(plugin);
    let mut adapter = creamy_binary::adapters::FromStd::new(File::open(path)?);
    Ok(BinaryPlugin::read_ne(&mut adapter)?)
}

fn load_and_wait(engine: &mut PluginEngine<WTRuntime, ()>, plugin: &str) -> anyhow::Result<()> {
    engine.runtime_mut().load_manually(read_plugin(plugin)?);
    engine.tick(ROUNDTRIP);
    println!("Loaded plugins: {}", engine.loaded_plugins());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[should_panic(expected = "Protocol 'empty' is already provided by 'empty'")]
async fn try_provide_already_provided_protocol() {
    protocol_provided_runner().await.unwrap();
}

async fn protocol_provided_runner() -> anyhow::Result<()> {
    let _c = TestContext::new()?;
    let mut engine = init_engine().await?;
    load_and_wait(&mut engine, "empty.cmy")?;
    load_and_wait(&mut engine, "empty.cmy")?;

    if let Some(e) = engine.errors().drain(..).next() {
        panic!("{e}");
    }

    panic!("no error occurred")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[should_panic(expected = "Protocol 'empty' is busy")]
async fn try_consume_busy_protocol() {
    protocol_busy_runner().await.unwrap();
}

async fn protocol_busy_runner() -> anyhow::Result<()> {
    let _c = TestContext::new()?;
    let mut engine = init_engine().await?;
    load_and_wait(&mut engine, "empty.cmy")?;
    load_and_wait(&mut engine, "empty_consumer.cmy")?;
    load_and_wait(&mut engine, "empty_consumer.cmy")?;

    if let Some(e) = engine.errors().drain(..).next() {
        panic!("{e}");
    }

    panic!("no error occurred")
}
