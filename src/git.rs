use git_repository::discover;

pub(crate) fn root() -> anyhow::Result<std::path::PathBuf> {
    match discover::upwards(std::env::current_dir()?) {
        // TODO: handle Trust?
        Ok((discover::repository::Path::LinkedWorkTree { work_dir, .. }, _)) => Ok(work_dir),
        Ok((discover::repository::Path::WorkTree(work_dir), _)) => Ok(work_dir),
        _ => {
            anyhow::bail!("git failed. are you in a Git repository?")
        }
    }
}
