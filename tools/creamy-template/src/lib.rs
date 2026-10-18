pub mod error;
mod id;

use std::path::Path;

use self::error::TemplateError;
pub use self::id::Id;

/// Represents a template configuration used to generate a new project structure.
///
/// This struct contains all the metadata required to create the initial
/// manifest file and directory structure for a new template.
pub struct Template {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub repository: String,
}

impl Template {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        write!(
            writer,
            r#"
[package]
id = "{id}"
name = "{name}"
version = "0.1.0"
description = "{desc}"
repository = "{repo}"
authors = [""]

[protocols]
            "#,
            id = self.id,
            name = self.name,
            desc = self.description,
            repo = self.repository,
        )
    }
}

/// Generates a template directory with a manifest file.
///
/// # Errors
///
/// Returns an error if the directory cannot be created or the manifest file
/// cannot be written.
///
/// # Examples
///
/// ```
/// use creamy_template::{Id, Template, generate_template};
///
/// fn main() -> Result<(), Box<dyn core::error::Error>> {
///     let template = Template {
///         id: Id::new("org.creamy.template")?,
///         name: "name".to_string(),
///         description: "desc".to_string(),
///         repository: "repo".to_string(),
///     };
///
///     let dir = tempfile::tempdir()?;
///     generate_template(dir.path(), &template)?;
///
///     let creamy = dir.path().join("name").join(".creamy");
///     assert!(std::fs::exists(&creamy)?);
///     assert!(std::fs::exists(creamy.join("definitions"))?);
///     assert!(std::fs::exists(creamy.join("manifest.toml"))?);
///     assert!(std::fs::exists(creamy.join("build.toml"))?);
///
///     Ok(())
/// }
///
/// ```
pub fn generate_template(
    outdir: impl AsRef<Path>,
    template: &Template,
) -> Result<(), TemplateError> {
    let outdir = outdir.as_ref();
    let rootdir = outdir.join(&template.name);
    let creamy_path = rootdir.join(".creamy");
    std::fs::create_dir_all(&creamy_path)?;
    std::fs::create_dir(creamy_path.join("definitions"))?;
    std::fs::File::create(creamy_path.join("build.toml"))?;
    let mut manifest_file = std::fs::File::create(creamy_path.join("manifest.toml"))?;
    template.write_to(&mut manifest_file)?;

    Ok(())
}
