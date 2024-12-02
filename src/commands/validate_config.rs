use crate::clientlib;

pub(crate) fn cmd(cmd: crate::ValidateFiles) -> anyhow::Result<()> {
    for filename in cmd.filenames {
        clientlib::load_config(&filename)?;
    }
    Ok(())
}
