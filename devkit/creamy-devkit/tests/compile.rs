use std::io::Write;

use creamy_devkit::{BinaryPlugin, compile_to_binary};
use creamy_template::{Id, Template};
use semver::Version;

const PROTOCOL: &str = r#"
<?xml version="1.0" encoding="UTF-8" ?>
<protocol name="testcase" version="1.0.0">
    <group name="opachki" access="Private">
        <message kind="0" name="Execute">
            <field name="value" type="u64"/>
        </message>
    </group>
</protocol>
"#;

#[test]
fn compile_write_read() -> Result<(), Box<dyn core::error::Error + Send + Sync>> {
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

    let mut file =
        std::fs::File::create(dir.path().join("example/.creamy/definitions/testcase.xml"))?;
    write!(file, "{PROTOCOL}")?;

    let package0 = compile_to_binary(dir.path().join("example"), vec![])?;
    assert_eq!(package0.version(), &Version::new(0, 1, 0));
    assert_eq!(package0.definitions.len(), 1);
    assert!(package0.core().is_empty());

    package0.write_to_file(dir.path().join("package.cmy"))?;

    let package1 = BinaryPlugin::load_from_file(dir.path().join("package.cmy"))?;
    let package2 = BinaryPlugin::load_from_bytes(&std::fs::read(dir.path().join("package.cmy"))?)?;

    assert_eq!(package0, package1);
    assert_eq!(package0, package2);
    assert_eq!(package1, package2);

    Ok(())
}
