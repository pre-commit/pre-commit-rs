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

// from `git rev-parse --local-env-vars`
// TODO: use gix for this?
const _LOCAL_ENV_VARS: &[&str] = &[
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_CONFIG",
    "GIT_CONFIG_COUNT",
    "GIT_CONFIG_PARAMETERS",
    "GIT_DIR",
    "GIT_GRAFT_FILE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_INTERNAL_SUPER_PREFIX",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_OBJECT_DIRECTORY",
    "GIT_PREFIX",
    "GIT_REPLACE_REF_BASE",
    "GIT_SHALLOW_FILE",
    "GIT_WORK_TREE",
];

pub(crate) trait GitNoLocalEnv {
    fn git_no_local_env(&mut self) -> &mut Self;
}

impl GitNoLocalEnv for process::Command {
    fn git_no_local_env(&mut self) -> &mut Self {
        for var in _LOCAL_ENV_VARS {
            self.env_remove(var);
        }
        self
    }
}

pub(crate) fn git_no_fs_monitor() -> process::Command {
    let mut ret = process::Command::new("git");
    ret.args(["-c", "core.useBuiltinFSMonitor=false"]);
    ret
}

pub(crate) fn init_repo(path: &str, remote: &str) -> anyhow::Result<()> {
    // TODO:
    // if os.path.isdir(remote):
    //      remote = os.path.abspath(remote)

    // TODO: add this to gix?
    // avoid the user's template so that hooks do not recurse
    git_no_fs_monitor()
        .git_no_local_env()
        .args(["init", "--template=", path])
        .output()?;
    git_no_fs_monitor()
        .git_no_local_env()
        .current_dir(path)
        .args(["remote", "add", "origin", remote])
        .output()?;

    Ok(())
}
