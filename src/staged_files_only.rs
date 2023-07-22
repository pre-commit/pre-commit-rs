use gix::index::entry::Flags;
use gix::index::entry::Mode;
use std::borrow::Borrow;
use std::path;
use std::process;

pub(crate) const SKIP_POST_CHECKOUT: &str = "_PRE_COMMIT_SKIP_POST_CHECKOUT";

const EMPTY_BLOB: gix::ObjectId = gix::ObjectId::empty_blob(gix::hash::Kind::Sha1);

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
        } else if diff.status.code() == Some(1) && diff.stdout.len() == 0 {
            // due to behaviour (probably a bug?) in git with crlf endings and
            // autocrlf set to either `true` or `input` somestimes git will
            // refuse to show a crlf-only diff to us :(
            Ok(None)
        } else if diff.status.code() == Some(1) && diff.stdout.len() > 0 {
            println!("todo: stash and such");
            Ok(None)
        } else {
            // TODO: properly format this!
            anyhow::bail!("failed to diff")
        }
    }
}

impl Drop for UnstagedCleared {
    fn drop(&mut self) {}
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
