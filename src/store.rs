use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

fn _store_dir_impl(
    pre_commit_home: Option<PathBuf>,
    xdg_cache_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> anyhow::Result<PathBuf> {
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

fn _to_buf(s: Option<OsString>) -> Option<PathBuf> {
    s.and_then(|s| if s.is_empty() { None } else { Some(s) })
        .map(PathBuf::from)
}

fn _store_dir() -> anyhow::Result<PathBuf> {
    _store_dir_impl(
        _to_buf(env::var_os("PRE_COMMIT_HOME")),
        _to_buf(env::var_os("XDG_CACHE_HOME")),
        dirs::home_dir(),
    )
}

#[cfg(windows)]
fn _readonly(d: &PathBuf) -> bool {
    false
}

#[cfg(not(windows))]
fn _readonly(d: &PathBuf) -> bool {
    use faccess::{AccessMode, PathExt};
    d.exists() && !d.access(AccessMode::WRITE).is_ok()
}

pub(crate) struct Store {
    pub(crate) directory: PathBuf,
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

    fn _db_path(&self) -> PathBuf {
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
}