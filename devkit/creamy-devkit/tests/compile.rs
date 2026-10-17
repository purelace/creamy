use std::io::Write;

use creamy_binary::{BinaryReader, BinaryWriter};
use creamy_devkit::{BinaryPlugin, compilation::compile_to_binary};
use creamy_template::{Id, Template};
use semver::Version;

const PROTOCOL: &str = r#"
<?xml version="1.0" encoding="UTF-8" ?>
<protocol name="testcase" version="1.0.0">
    <group name="opachki" access="Private">
        <message kind="0" name="Execute" direction="Duplex">
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

    let package0 = compile_to_binary(dir.path().join("example"), Box::default())?;
    assert_eq!(&package0.version, &Version::new(0, 1, 0));
    assert_eq!(package0.models.len(), 1);
    assert_eq!(package0.core(), &[] as &[u8]);

    let file = std::fs::File::create(dir.path().join("package.cmy"))?;
    let mut file = creamy_binary::adapters::FromStd::new(file);
    package0.write_ne(&mut file)?;

    let file = std::fs::File::open(dir.path().join("package.cmy"))?;
    let mut file = creamy_binary::adapters::FromStd::new(file);

    let package1 = BinaryPlugin::read_ne(&mut file)?;

    assert_eq!(package0, package1);

    Ok(())
}
