pub(crate) fn cmd(cmd: crate::ValidateFiles) -> anyhow::Result<()> {
    for filename in cmd.filenames {
        cfgv::load_file::<Vec<crate::clientlib::ManifestHook>>(&filename)?;
    }
    Ok(())
}
