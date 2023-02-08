pub(crate) fn repo() -> anyhow::Result<git_repository::Repository> {
    // TODO: handle Trust?
    let repo = git_repository::ThreadSafeRepository::discover_with_environment_overrides(".")?;
    if matches!(repo.kind(), git_repository::Kind::Bare) {
        anyhow::bail!("pre-commit needs a worktree, not a bare repo");
    }
    Ok(repo.to_thread_local())
}

pub(crate) fn has_unmerged_paths(repo: &git_repository::Repository) -> anyhow::Result<bool> {
    for entry in repo.index()?.entries() {
        if entry.flags.stage() != 0 {
            return Ok(true);
        }
    }
    Ok(false)
}
