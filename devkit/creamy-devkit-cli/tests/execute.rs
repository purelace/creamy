use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn init() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("creamy")?;

    cmd.arg("init");

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));

    Ok(())
    //run()
}
