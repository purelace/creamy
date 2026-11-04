use std::path::PathBuf;

use anyhow::Context;
use cargo_metadata::MetadataCommand;
use xshell::{Shell, cmd};

struct Fixtures {
    root: PathBuf,
    inner: Vec<PathBuf>,
}

struct Project {
    name: String,
    root: PathBuf,
    fixtures: Option<Fixtures>,
}

struct Projects {
    inner: Vec<Project>,
    has_fixtures: bool,
}

fn collect_projects() -> anyhow::Result<Projects> {
    let metadata = MetadataCommand::new().exec()?;

    let mut has_fixtures = false;
    let mut inner = vec![];

    for package in metadata.workspace_packages() {
        let package_root = PathBuf::from(&package.manifest_path)
            .parent()
            .unwrap()
            .to_path_buf();

        let fixtures_dir = package_root.join("fixtures");
        if !fixtures_dir.is_dir() {
            inner.push(Project {
                name: package.name.to_string(),
                root: package_root,
                fixtures: None,
            });

            continue;
        }

        let mut fixtures = Fixtures {
            root: fixtures_dir,
            inner: vec![],
        };

        if let Ok(entries) = std::fs::read_dir(&fixtures.root) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() && path.join("Cargo.toml").exists() {
                    has_fixtures = true;
                    println!("* Found fixture: {}", path.display());
                    fixtures.inner.push(path);
                }
            }
        }
    }

    Ok(Projects {
        inner,
        has_fixtures,
    })
}

pub fn run_all_tests() -> anyhow::Result<()> {
    let sh = Shell::new()?;
    let Projects {
        inner,
        has_fixtures,
    } = collect_projects()?;

    if has_fixtures {
        cmd!(
            sh,
            "cargo +stable install creamy-devkit-cli --path tools/creamy-devkit-cli"
        )
        .run()?;
    }

    for project in inner {
        if let Some(fixtures) = project.fixtures.as_ref() {
            sh.change_dir(&fixtures.root);
            cmd!(sh, "creamy build").run()?;
        }
    }

    let sh = Shell::new()?;
    cmd!(
        sh,
        "cargo +nightly llvm-cov --all-features --doctests --html"
    )
    .run()?;

    Ok(())
}

pub fn run_project_tests(project_name: &str) -> anyhow::Result<()> {
    let Projects { inner, .. } = collect_projects()?;

    let project = inner
        .into_iter()
        .find(|p| p.name == project_name)
        .with_context(|| format!("Project '{project_name}' not found in workspace"))?;

    let sh = Shell::new()?;

    if let Some(fixtures) = project.fixtures.as_ref() {
        cmd!(
            sh,
            "cargo +stable install creamy-devkit-cli --path tools/creamy-devkit-cli"
        )
        .run()?;

        sh.change_dir(&fixtures.root);
        cmd!(sh, "creamy build").run()?;
    }

    sh.change_dir(project.root);
    cmd!(
        sh,
        "cargo +nightly llvm-cov --all-features --doctests --html"
    )
    .run()
    .with_context(|| format!("Project: {project_name}"))?;

    Ok(())
}
