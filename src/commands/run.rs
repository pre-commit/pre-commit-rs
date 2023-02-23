use crate::env_ext;
use crate::git;
use crate::staged_files_only;
use crate::store;
use crate::PreCommitEnv;
use crate::Run;
use crate::Stage;

pub(crate) fn cmd(
    config: String,
    repo: gix::Repository,
    store: store::Store,
    cmd: Run,
) -> anyhow::Result<()> {
    // prevent recursive post-checkout hooks (#1418)
    if matches!(cmd.hook_stage, Stage::PostCommit)
        && env_ext::var_os_nonempty(staged_files_only::SKIP_POST_CHECKOUT).is_some()
    {
        return Ok(());
    }

    let stash = !cmd.all_files && cmd.files.is_empty();

    if stash && git::has_unmerged_paths(&repo)? {
        anyhow::bail!("Unmerged files.  Resolve before committing.");
    } else if stash && git::has_unstaged_config(&repo, &config)? {
        anyhow::bail!(
            "Your pre-commit configuration is unstaged.\n`git add {config}` to fix this."
        );
    }

    cmd.set_pre_commit_env_vars();

    let mut ctx: Option<staged_files_only::StagedFilesOnly> = None;
    if stash {
        ctx = Some(staged_files_only::StagedFilesOnly::new(
            &repo,
            store.directory,
        )?);
    }

    drop(ctx);
    Ok(())
}
