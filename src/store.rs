use std::fs;
use std::path;

use crate::env_ext;

fn _store_dir_impl(
    pre_commit_home: Option<path::PathBuf>,
    xdg_cache_home: Option<path::PathBuf>,
    home: Option<path::PathBuf>,
) -> anyhow::Result<path::PathBuf> {
    if let Some(ret) = pre_commit_home {
        Ok(ret)
    } else if let Some(mut ret) = xdg_cache_home {
        ret.push("pre-commit");
        Ok(ret)
    } else if let Some(mut ret) = home {
        ret.push(".cache");
        ret.push("pre-commit");
        Ok(ret)
    } else {
        anyhow::bail!("no candidate for pre-commit home");
    }
}

fn _store_dir() -> anyhow::Result<path::PathBuf> {
    _store_dir_impl(
        env_ext::var_os_nonempty("PRE_COMMIT_HOME").map(path::PathBuf::from),
        env_ext::var_os_nonempty("XDG_CACHE_HOME").map(path::PathBuf::from),
        dirs::home_dir(),
    )
}

#[cfg(windows)]
fn _readonly(d: &path::Path) -> bool {
    false
}

#[cfg(not(windows))]
fn _readonly(d: &path::Path) -> bool {
    use faccess::{AccessMode, PathExt};
    d.exists() && d.access(AccessMode::WRITE).is_err()
}

trait MakeRepo {
    fn make(&self, dest: &str) -> anyhow::Result<()>;
}

struct LocalRepoMaker {}

impl MakeRepo for LocalRepoMaker {
    fn make(&self, dest: &str) -> anyhow::Result<()> {
        anyhow::bail!("not implemented!");
    }
}

struct ClonedRepoMaker<'a> {
    repo: &'a str,
    rev: &'a str,
}

impl MakeRepo for ClonedRepoMaker<'_> {
    fn make(&self, dest: &str) -> anyhow::Result<()> {
        anyhow::bail!("not implemented!");
    }
}

pub(crate) struct Store {
    pub(crate) directory: path::PathBuf,
    pub(crate) readonly: bool,
}

impl Store {
    fn _exclusive_lock(&self) -> anyhow::Result<fslock::LockFile> {
        if self.readonly {
            anyhow::bail!("attempted a write on a readonly store");
        }

        let mut lock = fslock::LockFile::open(&self.directory.join(".lock"))?;
        if !lock.try_lock()? {
            println!("[INFO] Locking pre-commit directory");
            lock.lock()?;
        }
        Ok(lock)
    }

    fn _db_path(&self) -> path::PathBuf {
        self.directory.join("db.db")
    }

    fn _create_config_table(&self, conn: &rusqlite::Connection) -> anyhow::Result<()> {
        conn.execute_batch(indoc::indoc! {"
            CREATE TABLE IF NOT EXISTS configs (
               path TEXT NOT NULL,
               PRIMARY KEY (path)
            );
        "})?;
        Ok(())
    }

    fn _ensure_created(&self) -> anyhow::Result<()> {
        if !self._db_path().exists() {
            fs::create_dir_all(&self.directory)?;

            {
                let lock = self._exclusive_lock()?;
                // another process may have already completed this work
                if self._db_path().exists() {
                    return Ok(());
                }

                let tmp = tempfile::Builder::new()
                    .suffix(".db")
                    .tempfile_in(&self.directory)?;
                {
                    let conn = rusqlite::Connection::open(&tmp)?;
                    conn.execute_batch(indoc::indoc! {"
                        CREATE TABLE repos (
                            repo TEXT NOT NULL,
                            ref TEXT NOT NULL,
                            path TEXT NOT NULL,
                            PRIMARY KEY (repo, ref)
                        );
                    "})?;
                    self._create_config_table(&conn)?;
                }

                tmp.persist(self._db_path())?;

                drop(lock);
            }
        }

        Ok(())
    }

    pub(crate) fn new() -> anyhow::Result<Self> {
        let directory = _store_dir()?;
        let readonly = _readonly(&directory);
        let ret = Store {
            directory,
            readonly,
        };
        ret._ensure_created()?;
        Ok(ret)
    }

    pub(crate) fn clone_repo(&self, repo: &str, rev: &str) -> anyhow::Result<String> {
        anyhow::bail!("not implemented!");
    }

    pub(crate) fn mark_config_used(&self, path: &str) -> anyhow::Result<()> {
        if self.readonly {
            return Ok(());
        }
        if let Ok(p) = fs::canonicalize(path) {
            let pstr = p.to_string_lossy();
            let conn = rusqlite::Connection::open(self._db_path())?;
            self._create_config_table(&conn)?;
            conn.execute("INSERT OR IGNORE INTO configs VALUES (?)", (pstr,))?;
        }
        Ok(())
    }
}
