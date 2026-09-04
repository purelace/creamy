use creamy_devkit::compile_to_binary;
use creamy_template::{Id, Template};

#[test]
fn missing_definitions_directory() -> Result<(), Box<dyn core::error::Error>> {
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

    std::fs::remove_dir(
        dir.path()
            .join("example")
            .join(".creamy")
            .join("definitions"),
    )?;

    compile_to_binary(dir.path().join("example"), vec![])?;
    Ok(())
}
