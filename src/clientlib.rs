use crate::Stage;
use cfgv::Cfgv;
use cfgv_derive::Cfgv;
use pre_commit_rs_derive::make_config_hook;

#[allow(dead_code)]
#[derive(Cfgv, Debug)]
pub(crate) struct ManifestHook {
    #[cfgv_id]
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) entry: String,
    pub(crate) language: String,
    #[cfgv_default]
    pub(crate) alias: String,

    #[cfgv_default]
    pub(crate) files: String,
    #[cfgv_default_expr("^$".into())]
    pub(crate) exclude: String,
    #[cfgv_default_expr(vec!["file".into()])]
    pub(crate) types: Vec<String>,
    #[cfgv_default]
    pub(crate) types_or: Vec<String>,
    #[cfgv_default]
    pub(crate) exclude_types: Vec<String>,

    #[cfgv_default]
    pub(crate) additional_dependencies: Vec<String>,
    #[cfgv_default]
    pub(crate) args: Vec<String>,
    #[cfgv_default]
    pub(crate) always_run: bool,
    #[cfgv_default]
    pub(crate) fail_fast: bool,
    #[cfgv_default_expr(true)]
    pub(crate) pass_filenames: bool,
    #[cfgv_default]
    pub(crate) description: String,
    #[cfgv_default_expr("default".into())]
    pub(crate) language_version: String,
    #[cfgv_default]
    pub(crate) log_file: String,
    #[cfgv_default_expr("0".into())]
    pub(crate) minimum_pre_commit_version: String,
    #[cfgv_default]
    pub(crate) require_serial: bool,
    #[cfgv_default]
    pub(crate) stages: Vec<Stage>,
    #[cfgv_default]
    pub(crate) verbose: bool,
}

#[allow(dead_code)]
#[make_config_hook]
pub(crate) struct ConfigHook;

#[allow(dead_code)]
#[derive(Cfgv, Debug)]
pub(crate) struct LocalRepo {
    #[cfgv_id]
    pub(crate) repo: String,
    pub(crate) hooks: Vec<ManifestHook>,
}

#[allow(dead_code)]
#[derive(Cfgv, Debug)]
pub(crate) struct MetaRepo {
    #[cfgv_id]
    pub(crate) repo: String,
    pub(crate) hooks: Vec<ConfigHook>,
}

#[allow(dead_code)]
#[derive(Cfgv, Debug)]
pub(crate) struct RemoteRepo {
    #[cfgv_id]
    pub(crate) repo: String,
    pub(crate) rev: String,
    pub(crate) hooks: Vec<ConfigHook>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum Repo {
    Local(LocalRepo),
    Meta(MetaRepo),
    Remote(RemoteRepo),
}

impl Cfgv for Repo {
    fn cfgv_validate(ctx: &mut Vec<String>, v: &serde_yaml::Value) -> anyhow::Result<Self> {
        let mut repo = None;
        if let serde_yaml::Value::Mapping(m) = v {
            if let Some(serde_yaml::Value::String(repo_s)) = m.get("repo") {
                repo = Some(repo_s.as_str())
            }
        }
        match repo {
            Some("local") => Ok(Self::Local(LocalRepo::cfgv_validate(ctx, v)?)),
            Some("meta") => Ok(Self::Meta(MetaRepo::cfgv_validate(ctx, v)?)),
            _ => Ok(Self::Remote(RemoteRepo::cfgv_validate(ctx, v)?)),
        }
    }
}

#[allow(dead_code)]
#[derive(Cfgv, Debug)]
pub(crate) struct Config {
    pub(crate) repos: Vec<Repo>,

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

pub(crate) fn load_config(filename: &str) -> anyhow::Result<Config> {
    cfgv::load_file::<crate::clientlib::Config>(filename)
}
