pub(crate) fn cmd(filenames: &Vec<String>) -> anyhow::Result<()> {
    for filename in filenames {
        cfgv::load_file::<crate::clientlib::Config>(&filename)?;
    }
    Ok(())
}
