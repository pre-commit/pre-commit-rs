use crate::clientlib::{Config, LocalRepo, ManifestHook, MetaRepo, RemoteRepo, Repo};
use crate::store::Store;
use std::path;

#[derive(Debug)]
pub(crate) struct Hook {
    src: String,
    prefix: path::PathBuf,
    hook: ManifestHook,
}

fn _local_hooks(repo: LocalRepo, store: &Store) -> anyhow::Result<Vec<Hook>> {
    anyhow::bail!("not implemented!")
}

fn _meta_hooks(repo: MetaRepo, store: &Store) -> anyhow::Result<Vec<Hook>> {
    anyhow::bail!("not implemented!")
}

fn _cloned_hooks(repo: RemoteRepo, store: &Store) -> anyhow::Result<Vec<Hook>> {
    let path = store.clone_repo(&repo.repo, &repo.rev)?;
    anyhow::bail!("not implemented!")
}

pub(crate) fn all_hooks(config: Config, store: &Store) -> anyhow::Result<Vec<Hook>> {
    let mut ret: Vec<Hook> = Vec::new();

    for repo in config.repos.into_iter() {
        match repo {
            Repo::Local(local_repo) => ret.extend(_local_hooks(local_repo, store)?),
            Repo::Meta(meta_repo) => ret.extend(_meta_hooks(meta_repo, store)?),
            Repo::Remote(repo) => ret.extend(_cloned_hooks(repo, store)?),
        }
    }

    Ok(ret)
}
