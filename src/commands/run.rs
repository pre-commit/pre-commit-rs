pub(crate) fn cmd(
    config: String,
    repo: git_repository::Repository,
    store: crate::store::Store,
    cmd: crate::Run,
) -> anyhow::Result<()> {
    let stash = !cmd.all_files && cmd.files.is_empty();

    if stash && crate::git::has_unmerged_paths(&repo)? {
        anyhow::bail!("Unmerged files.  Resolve before committing.");
    } else if stash && crate::git::has_unstaged_config(&repo, &config)? {
        anyhow::bail!(
            "Your pre-commit configuration is unstaged.\n`git add {config}` to fix this."
        );
    }
    Ok(())
}
