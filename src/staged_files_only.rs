use gix::index::entry::Flags;
use gix::index::entry::Mode;
use std::borrow::Borrow;
use std::fs;
use std::path;
use std::process;
use std::time;

pub(crate) const SKIP_POST_CHECKOUT: &str = "_PRE_COMMIT_SKIP_POST_CHECKOUT";

const EMPTY_BLOB: gix::ObjectId = gix::ObjectId::empty_blob(gix::hash::Kind::Sha1);

fn _discard_changes() -> anyhow::Result<()> {
    // without forcing submodule.recurse=0, changes in nested submodules will
    // be discarded if `submodule.recurse=1` is configured
    // we choose this instead of `--no-recurse-submodules` because it works on
    // versions of git before that option was added to `git checkout`
    let output = process::Command::new("git")
        .args(["-c", "submodule.recurse=0", "checkout", "--", "."])
        .env(SKIP_POST_CHECKOUT, "1")
        .output()?;

    // TODO: improve error message
    // TODO: and/or `.exit_ok()?` when stabilized
    if !output.status.success() {
        anyhow::bail!(
            "checkout failed\n\nstdout: {:?}\nstderr: {:?}\n",
            output.stdout,
            output.stderr
        )
    } else {
        Ok(())
    }
}

fn _git_apply(patch: &str) -> anyhow::Result<()> {
    let output = process::Command::new("git")
        .args(["apply", "--whitespace=nowarn", patch])
        .output()?;
    // retry with autocrlf=false == see #570
    if !output.status.success() {
        let retried = process::Command::new("git")
            .args([
                "-c",
                "core.autocrlf=false",
                "apply",
                "--whitespace=nowarn",
                patch,
            ])
            .output()?;

        if !retried.status.success() {
            anyhow::bail!(
                "apply failed\n\nstdout: {:?}\nstderr: {:?}\n",
                output.stdout,
                output.stderr
            )
        }
    }
    Ok(())
}

struct IntentToAdd<'a> {
    repo: &'a gix::Repository,
    info: Vec<(Mode, bstr::BString)>,
}

impl<'a> IntentToAdd<'a> {
    fn new(repo: &'a gix::Repository) -> anyhow::Result<Option<Self>> {
        let mut info: Vec<(Mode, bstr::BString)> = Vec::new();
        let mut idx = repo.open_index()?;
        for (entry, p) in &mut idx.entries_mut_with_paths() {
            if entry.flags.contains(Flags::INTENT_TO_ADD) {
                info.push((entry.mode, p.into()));
                entry.flags.set(Flags::REMOVE, true);
                entry.flags.remove(Flags::INTENT_TO_ADD);
            }
        }
        if !info.is_empty() {
            idx.write(Default::default())?;
            Ok(Some(IntentToAdd { repo, info }))
        } else {
            Ok(None)
        }
    }
}

impl Drop for IntentToAdd<'_> {
    fn drop(&mut self) {
        let mut idx = self.repo.open_index().unwrap();
        for (mode, p) in self.info.iter() {
            if idx.entry_index_by_path_and_stage(p.borrow(), 0).is_some() {
                continue;
            }

            // an intent-to-add object is always: 0 stat, empty file
            idx.dangerously_push_entry(
                Default::default(),
                EMPTY_BLOB,
                Flags::EXTENDED | Flags::INTENT_TO_ADD,
                *mode,
                p.borrow(),
            );
        }
        idx.sort_entries();
        idx.verify_entries().unwrap();
        idx.write(Default::default()).unwrap();
    }
}

struct UnstagedCleared {
    patch: String,
}

impl UnstagedCleared {
    fn new<P: AsRef<path::Path>>(p: P) -> anyhow::Result<Option<Self>> {
        // TODO: rewrite this using gix once easy
        let tree_res = process::Command::new("git").arg("write-tree").output()?;
        let tree = String::from_utf8(tree_res.stdout)?;

        let diff = process::Command::new("git")
            .args([
                "diff-index",
                "--ignore-submodules",
                "--binary",
                "--exit-code",
                "--no-color",
                "--no-ext-diff",
            ])
            .arg(tree.trim_end())
            .arg("--")
            .output()?;

        if diff.status.success() {
            Ok(None)
        } else if diff.status.code() == Some(1) && diff.stdout.is_empty() {
            // due to behaviour (probably a bug?) in git with crlf endings and
            // autocrlf set to either `true` or `input` somestimes git will
            // refuse to show a crlf-only diff to us :(
            Ok(None)
        } else if diff.status.code() == Some(1) && !diff.stdout.is_empty() {
            let now = time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)?
                .as_millis();
            let patch_filename = format!("patch{now}-{}", process::id());
            let patch_p = p.as_ref().join(patch_filename);
            let patch = patch_p.to_string_lossy().into();

            // TODO: logging / coloring
            println!("Unstaged files detected.");
            println!("Stashing unstaged files to {patch}");

            fs::create_dir_all(p)?;
            fs::write(&patch_p, diff.stdout)?;

            _discard_changes()?;

            Ok(Some(Self { patch }))
        } else {
            // TODO: properly format this!
            anyhow::bail!("pre-commit failed to diff -- perhaps due to permissions?")
        }
    }
}

impl Drop for UnstagedCleared {
    fn drop(&mut self) {
        let apply_attempt = _git_apply(&self.patch);
        // TODO: specific error types?
        if apply_attempt.is_err() {
            // TODO: logging / coloring
            println!("Stashed changes conflicted with hook auto-fixes... Rolling back fixes...");
            _discard_changes().unwrap();
            _git_apply(&self.patch).unwrap();
        }

        // TODO: logging / coloring
        println!("Restored changes from {}.", self.patch);
    }
}

pub(crate) struct StagedFilesOnly<'a> {
    // ordering here is important, Drop teardown happens in declaration order
    #[allow(dead_code)] // for Drop
    unstaged_cleared: Option<UnstagedCleared>,
    #[allow(dead_code)] // for Drop
    intent_to_add: Option<IntentToAdd<'a>>,
}

impl<'a> StagedFilesOnly<'a> {
    pub(crate) fn new<P: AsRef<path::Path>>(
        repo: &'a gix::Repository,
        p: P,
    ) -> anyhow::Result<Self> {
        // ordering here is important
        let intent_to_add = IntentToAdd::new(repo)?;
        let unstaged_cleared = UnstagedCleared::new(p)?;
        Ok(StagedFilesOnly {
            unstaged_cleared,
            intent_to_add,
        })
    }
}
