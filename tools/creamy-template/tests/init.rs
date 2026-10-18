use creamy_template::{Id, Template};

#[test]
fn init_project() -> Result<(), Box<dyn core::error::Error>> {
    let template = Template {
        id: Id::new("org.creamy.example")?,
        name: "Example".into(),
        description: "Template example".into(),
        repository: "https://github.com/purelace/creamy".into(),
    };

    let dir = tempfile::TempDir::new()?;
    creamy_template::generate_template(dir.path(), &template)?;

    let creamy = dir.path().join("Example").join(".creamy");
    assert!(std::fs::exists(&creamy)?);
    assert!(std::fs::exists(creamy.join("definitions"))?);
    assert!(std::fs::exists(creamy.join("manifest.toml"))?);
    assert!(std::fs::exists(creamy.join("build.toml"))?);

    Ok(())
}
