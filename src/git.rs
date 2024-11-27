use std::ffi;
use std::path;
use std::process;

pub(crate) fn repo<P: AsRef<path::Path>>(p: P) -> anyhow::Result<gix::Repository> {
    // TODO: handle Trust?
    let repo = gix::ThreadSafeRepository::discover_with_environment_overrides(p)?.to_thread_local();
    if matches!(repo.kind(), gix::repository::Kind::Bare) {
        anyhow::bail!("pre-commit needs a worktree, not a bare repo");
    }
    Ok(repo)
}

pub(crate) fn has_unmerged_paths(repo: &gix::Repository) -> anyhow::Result<bool> {
    for entry in repo.index()?.entries() {
        if entry.flags.stage() != gix::index::entry::Stage::Unconflicted {
            return Ok(true);
        }
    }
    Ok(false)
}

trait CmdKv {
    fn arg_kv<K: AsRef<ffi::OsStr>, V: AsRef<ffi::OsStr>>(&mut self, k: K, v: V) -> &mut Self;
}

impl CmdKv for process::Command {
    fn arg_kv<K: AsRef<ffi::OsStr>, V: AsRef<ffi::OsStr>>(&mut self, k: K, v: V) -> &mut Self {
        self.arg(k).arg(v)
    }
}

pub(crate) fn has_unstaged_config(repo: &gix::Repository, config: &str) -> anyhow::Result<bool> {
    // TODO: need `write-tree` / `diff`: Byron/gitoxide#301
    let retc = process::Command::new("git")
        .arg_kv("-C", repo.work_dir().unwrap())
        .args(["diff", "--quiet", "--no-ext-diff", config])
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status()?
        .code()
        .unwrap_or(255);

    Ok(retc == 1)
}
