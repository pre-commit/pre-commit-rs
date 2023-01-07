use cfgv::Cfgv;
use cfgv_derive::Cfgv;

#[derive(Debug, Cfgv)]
#[allow(dead_code)]
pub(crate) struct ManifestHook {
    #[cfgv_id]
    id: String,
    name: String,
    entry: String,
    language: String,
    #[cfgv_default]
    alias: String,

    #[cfgv_default]
    files: String,
    #[cfgv_default_expr("^$".into())]
    exclude: String,
    #[cfgv_default_expr(vec!["file".into()])]
    types: Vec<String>,
    #[cfgv_default]
    types_or: Vec<String>,
    #[cfgv_default]
    exclude_types: Vec<String>,

    #[cfgv_default]
    additional_dependencies: Vec<String>,
    #[cfgv_default]
    args: Vec<String>,
    #[cfgv_default]
    always_run: bool,
    #[cfgv_default]
    fail_fast: bool,
    #[cfgv_default_expr(true)]
    pass_filenames: bool,
    #[cfgv_default]
    description: String,
    #[cfgv_default_expr("default".into())]
    language_version: String,
    #[cfgv_default]
    log_file: String,
    #[cfgv_default_expr("0".into())]
    minimum_pre_commit_version: String,
    #[cfgv_default]
    require_serial: bool,
    #[cfgv_default]
    stages: Vec<String>,
    #[cfgv_default]
    verbose: bool,
}
