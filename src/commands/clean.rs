pub(crate) fn cmd(store: crate::store::Store) -> anyhow::Result<()> {
    rm_rf::remove(&store.directory)?;
    println!("Cleaned {}.", store.directory.display());
    Ok(())
}
