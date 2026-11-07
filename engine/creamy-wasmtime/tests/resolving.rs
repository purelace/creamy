//TODO: move this tests to engine tests
mod general;

use std::{num::NonZeroU8, time::Duration};

use self::general::{TestContext, init_engine};

const ROUNDTRIP: NonZeroU8 = NonZeroU8::new(2).unwrap();

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[should_panic(expected = "Group 'none' from protocol 'empty' is already provided by 'empty'")]
async fn try_provide_already_provided_group() {
    group_provided_runner().await.unwrap();
}

async fn group_provided_runner() -> anyhow::Result<()> {
    let context = TestContext::new()?;
    context.copy_to_plugin_dir("empty.cmy")?;

    let mut engine = init_engine().await?;

    let mut seconds = 16;
    while seconds != 0 {
        engine.tick(ROUNDTRIP);
        if engine.loaded_plugins() == 2 {
            context.copy_to_plugin_dir("empty.cmy")?;
        }

        println!("Loaded plugins: {}", engine.loaded_plugins());
        if let Some(e) = engine.errors().drain(..).next() {
            panic!("{e}");
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
        seconds -= 1;
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[should_panic(expected = "Group 'none' from protocol 'empty' is busy")]
async fn try_consume_busy_group() {
    group_busy_runner().await.unwrap();
}

async fn group_busy_runner() -> anyhow::Result<()> {
    let context = TestContext::new()?;
    context.copy_to_plugin_dir("empty.cmy")?;

    let mut engine = init_engine().await?;

    let mut ticks = 16;
    while ticks != 0 {
        engine.tick(ROUNDTRIP);
        if engine.loaded_plugins() == 2 {
            context.copy_to_plugin_dir("empty_consumer.cmy")?;
        }

        if engine.loaded_plugins() == 3 {
            context.copy_to_plugin_dir("empty_consumer.cmy")?;
        }

        println!("Loaded plugins: {}", engine.loaded_plugins());
        if let Some(e) = engine.errors().drain(..).next() {
            panic!("{e}");
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
        ticks -= 1;
    }

    Ok(())
}
