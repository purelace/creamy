use creamy_devkit::{Error, compile_to_binary};
use creamy_template::{Id, Template};

#[test]
fn missing_directory_error() -> Result<(), Box<dyn core::error::Error>> {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir(dir.path().join("example"))?;
    let result = compile_to_binary(dir.path().join("example"), vec![]);
    assert!(matches!(result, Err(Error::MissingDirectory)));

    Ok(())
}

#[test]
fn not_a_file() -> Result<(), Box<dyn core::error::Error>> {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir_all(dir.path().join("example/.creamy/manifest.toml"))?;
    let result = compile_to_binary(dir.path().join("example"), vec![]);
    assert!(matches!(result, Err(Error::IO(_))));

    Ok(())
}

#[test]
fn not_a_directory() -> Result<(), Box<dyn core::error::Error>> {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir_all(dir.path().join("example/.creamy"))?;
    std::fs::File::create_new(dir.path().join("example/.creamy/definitions"))?;
    let result = compile_to_binary(dir.path().join("example"), vec![]);
    assert!(matches!(result, Err(Error::IO(_))));

    Ok(())
}

#[test]
fn missing_manifest() -> Result<(), Box<dyn core::error::Error>> {
    let dir = tempfile::tempdir()?;
    creamy_template::generate_template(
        dir.path(),
        &Template {
            id: Id::new("org.creamy.example")?,
            name: "example".into(),
            description: String::new(),
            repository: "https://github.com/purelace/creamy".into(),
        },
    )?;

    std::fs::remove_file(
        dir.path()
            .join("example")
            .join(".creamy")
            .join("manifest.toml"),
    )?;

    let result = compile_to_binary(dir.path().join("example"), vec![]);
    assert!(matches!(result, Err(Error::IO(_))));
    Ok(())
}
