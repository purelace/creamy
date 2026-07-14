use std::path::PathBuf;

pub fn get_workdir(workdir: Option<String>) -> anyhow::Result<PathBuf> {
    Ok(if let Some(workdir) = workdir {
        PathBuf::from(workdir)
    } else {
        std::env::current_dir()?
    })
}
