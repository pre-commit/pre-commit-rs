use cfgv::Cfgv;
use cfgv_derive::Cfgv;
use pre_commit_rs_derive::make_config_hook;

#[allow(dead_code)]
#[derive(Debug, Cfgv)]
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

#[allow(dead_code)]
#[make_config_hook]
pub(crate) struct ConfigHook;

#[allow(dead_code)]
#[derive(Cfgv, Debug)]
pub(crate) struct Repo {
    #[cfgv_id]
    repo: String,
    rev: String,
    hooks: Vec<ConfigHook>,
}

#[allow(dead_code)]
#[derive(Cfgv, Debug)]
pub(crate) struct Config {
    repos: Vec<Repo>,

    #[cfgv_default_expr(vec!["pre-commit".into()])]
    default_install_hook_types: Vec<String>,

    // TODO: idk what this should be
    // default_language_version: ...,

    // TODO: idk how to set the default nicely
    // #[cfgv_default_expr=...]
    // default_stages: Vec<String>>,
    #[cfgv_default]
    files: String,
    #[cfgv_default_expr("^$".into())]
    exclude: String,
    #[cfgv_default]
    fail_fast: bool,
    #[cfgv_default_expr("0".into())]
    minimum_pre_commit_version: String,
    // TODO: allow any mapping here
    // ci: ...,
}
