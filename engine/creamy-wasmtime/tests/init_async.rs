mod general;

use std::{num::NonZeroU8, time::Duration};

use self::general::{TestContext, init_engine};

const ROUNDTRIP: NonZeroU8 = NonZeroU8::new(2).unwrap();

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preload_plugin() -> anyhow::Result<()> {
    let context = TestContext::new()?;
    context.copy_to_plugin_dir("ping.cmy")?;

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
    let context = TestContext::new()?;

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
            context.copy_to_plugin_dir("ping.cmy")?;
        }

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
