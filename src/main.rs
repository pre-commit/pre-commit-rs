use clap::{Args, Parser, Subcommand, ValueEnum};
use pre_commit_rs_derive::PreCommitEnv;
use std::env;
use std::path;

mod clientlib;
mod commands;
mod env_ext;
mod git;
mod staged_files_only;
mod store;

#[derive(ValueEnum, Clone, Debug)]
enum HookType {
    PreCommit,
    PreMergeCommit,
    PrePush,
    PrepareCommitMsg,
    CommitMsg,
    PostCommit,
    PostCheckout,
    PostMerge,
    PostRewrite,
}

#[derive(ValueEnum, Clone, Debug)]
enum Stage {
    Commit,
    MergeCommit,
    Push,
    PrepareCommitMsg,
    CommitMsg,
    PostCommit,
    PostCheckout,
    PostMerge,
    PostRewrite,
    Manual,
}

#[derive(Args, Debug)]
struct Autoupdate {
    /// Update to the bleeding edge of `HEAD` instead of the latest tagged
    /// version (the default behaviour)
    #[arg(long)]
    bleeding_edge: bool,
    /// Store "frozen" hashes in `rev` instead of tag names
    #[arg(long)]
    freeze: bool,
    /// Only update this repo -- may be specified multiple times
    #[arg(long = "repo")]
    repos: Vec<String>,
}

#[derive(Args, Debug)]
struct InitTemplatedir {
    /// The directory in which to write the hook script
    directory: String,
    /// Can be specified multiple times
    #[arg(short = 't', long)]
    hook_type: Vec<HookType>,
}

#[derive(Args, Debug)]
struct Install {
    /// Overwrite existing hooks / remove migration mode
    #[arg(short = 'f', long)]
    overwrite: bool,
    /// Whether to install hook environments for all environments in the config
    /// file
    #[arg(long)]
    install_hooks: bool,
    /// Can be specified multiple times
    #[arg(short = 't', long)]
    hook_type: Vec<HookType>,
    /// Whether to allow a missing `pre-commit` configuration file or exit with
    /// a failure code
    #[arg(long)]
    allow_missing_config: bool,
}

trait PreCommitEnv {
    fn set_pre_commit_env_vars(&self);
}

#[derive(Args, Debug, PreCommitEnv)]
#[clap(group = clap::ArgGroup::new("file-args").multiple(false))]
struct Run {
    /// A single hook-id to run
    hook: Option<String>,
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Run on all the files in the repo
    #[arg(short = 'a', long, group = "file-args")]
    all_files: bool,
    /// Specific filenames to run hooks on
    #[arg(long, required = false, num_args = 0.., group = "file-args")]
    files: Vec<String>,
    /// When hooks fail, run `git diff` directly afterward
    #[arg(long)]
    show_diff_on_failure: bool,
    /// (for usage with `--to-ref`) -- this option represents the original ref
    /// in a `from_ref...to_ref` diff expression.
    /// For `pre-push` hooks this represents the branch you are pushing to.
    /// For `post-checkout` hooks this represents the branch that was
    /// previously checked out.
    #[arg(long, short_alias = 's', alias = "source", requires = "to_ref")]
    #[pre_commit_env_var("PRE_COMMIT_ORIGIN")] // deprecated name
    #[pre_commit_env_var("PRE_COMMIT_FROM_REF")]
    from_ref: Option<String>,
    /// (for usage with `--from-ref`) -- this option represents the destination
    /// ref in a `from_ref...to_ref` diff expressions.
    /// For `pre-push` hooks this represents the branch being pushed.
    /// For `post-checkout` hooks this represents the branch that is now checked out.
    #[arg(long, short_alias = 'o', alias = "origin", requires = "from_ref")]
    #[pre_commit_env_var("PRE_COMMIT_SOURCE")] // deprecated name
    #[pre_commit_env_var("PRE_COMMIT_TO_REF")]
    to_ref: Option<String>,
    /// The stage during which the hook is fired
    #[arg(value_enum, long, default_value_t = Stage::Commit)]
    hook_stage: Stage,
    /// Remote branch ref used by `git push`
    #[arg(long)]
    #[pre_commit_env_var("PRE_COMMIT_REMOTE_BRANCH")]
    remote_branch: Option<String>,
    /// Local branch ref used by `git push`
    #[arg(long)]
    #[pre_commit_env_var("PRE_COMMIT_LOCAL_BRANCH")]
    local_branch: Option<String>,
    /// Filename to check when running during `commit-msg`
    #[arg(long, requires = "hook_stage")]
    #[arg(required_if_eq("hook_stage", "commit-msg"))]
    #[arg(required_if_eq("hook_stage", "prepare-commit-msg"))]
    commit_msg_filename: Option<String>,
    /// Source of the commit message (typically the second argument to
    /// .git/hooks/prepare-commit-msg)
    #[arg(long)]
    #[pre_commit_env_var("PRE_COMMIT_COMMIT_MSG_SOURCE")]
    prepare_commit_message_source: Option<String>,
    /// Commit object name (typically the third argument to
    /// .git/hooks/prepare-commit-msg)
    #[arg(long)]
    #[pre_commit_env_var("PRE_COMMIT_COMMIT_OBJECT_NAME")]
    commit_object_name: Option<String>,
    /// Remote name used by `git push`
    #[arg(long)]
    #[pre_commit_env_var("PRE_COMMIT_REMOTE_NAME")]
    remote_name: Option<String>,
    /// Remote url used by `git push`
    #[arg(long)]
    #[pre_commit_env_var("PRE_COMMIT_REMOTE_URL")]
    remote_url: Option<String>,
    /// Indicates whether the checkout was a branch checkout
    /// (changing branchesm, flag=1) or a file checkout (retrieving a file from
    /// the index, flag=0)]
    #[arg(long)] // TODO: can only be '0' or '1'?
    #[pre_commit_env_var("PRE_COMMIT_CHECKOUT_TYPE")]
    checkout_type: Option<String>,
    /// During a post-merge hook, indicates whether the merge was a squash
    /// merge
    #[arg(long)] // TODO: can only bo '0' or '1'?
    #[pre_commit_env_var("PRE_COMMIT_IS_SQUASH_MERGE")]
    is_squash_merge: Option<String>,
    /// During a post-rewrite hook, specifies the command that invoked the
    /// rewrite
    #[arg(long)]
    #[pre_commit_env_var("PRE_COMMIT_REWRITE_COMMAND")]
    rewrite_command: Option<String>,
}

#[derive(Args, Debug)]
struct TryRepo {
    /// Repository to source hooks from
    repo: String,
    /// Manually select a rev to run against, otherwise the `HEAD` revision
    /// will be used
    #[arg(long = "ref", visible_alias = "rev")]
    rev: Option<String>,
    #[clap(flatten)]
    run: Run,
}

#[derive(Args, Debug)]
struct Uninstall {
    /// Can be specified multiple times
    #[arg(short = 't', long)]
    hook_type: Vec<HookType>,
}

#[derive(Args, Debug)]
struct ValidateFiles {
    filenames: Vec<String>,
}

#[derive(Args, Debug)]
struct HookImpl {
    #[arg(long)]
    hook_type: HookType,
    #[arg(long)]
    hook_dir: String,
    #[arg(long)]
    skip_on_missing_config: bool,
    rest: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Auto-update pre-commit config to the latest repos' versions
    Autoupdate(Autoupdate),
    /// Clean out pre-commit files
    Clean,
    /// Clean unused cached repos
    Gc,
    /// Install hook script in a directory intended for use with `git config init.templateDir`
    InitTemplatedir(InitTemplatedir),
    /// Install the pre-commit script
    Install(Install),
    /// Install hook environments for all environments in the config file.
    /// You may find `pre-commit install --install-hooks` more useful.
    InstallHooks,
    /// Migrate list configuration to new map configuration
    MigrateConfig,
    /// Run hooks
    Run(Run),
    /// Produce a sample .pre-commit-config.yaml file
    SampleConfig,
    /// Try the hooks in a repository, useful for developing new hooks
    TryRepo(TryRepo),
    /// Uninstall the pre-commit script
    Uninstall(Uninstall),
    /// Validate .pre-commit-config.yaml files
    ValidateConfig(ValidateFiles),
    /// Validate .pre-commit-hooks.yaml files
    ValidateManifest(ValidateFiles),
    #[clap(hide = true)]
    HookImpl(HookImpl),
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(long, global = true)]
    color: Option<String>,
    #[arg(short = 'c', long, default_value = ".pre-commit-config.yaml")]
    config: String,
}

struct Chdir {
    orig: path::PathBuf,
    new: path::PathBuf,
}

fn _chdir_path<P: AsRef<path::Path>>(p: P, chdir: &Chdir) -> String {
    pathdiff::diff_paths(chdir.orig.join(p), &chdir.new)
        .unwrap() // guaranteed chdir.new is abspath
        .to_string_lossy()
        .into()
}

fn _chdir_to_git_root(config: String) -> anyhow::Result<(String, gix::Repository, Option<Chdir>)> {
    let orig = env::current_dir()?;
    let repo = git::repo(&orig)?;
    let new = repo.work_dir().unwrap();

    if orig == new {
        Ok((config, repo, None))
    } else {
        let chdir = Chdir {
            orig,
            new: new.to_path_buf(),
        };
        let config = if path::Path::new(&config).exists() {
            _chdir_path(config, &chdir)
        } else {
            config
        };

        env::set_current_dir(&chdir.new)?;
        Ok((config, repo, Some(chdir)))
    }
}

fn _adjust_run(cmd: &mut Run, chdir: &Chdir) {
    for f in cmd.files.iter_mut() {
        *f = _chdir_path(&f, chdir);
    }
    if let Some(f) = &cmd.commit_msg_filename {
        cmd.commit_msg_filename = Some(_chdir_path(f, chdir));
    }
}

fn main() -> anyhow::Result<()> {
    let res = Cli::parse();
    let cmd = res.command.unwrap_or_else(|| {
        let argv = vec![env::args().next().unwrap(), "run".into()];
        Cli::parse_from(argv).command.unwrap()
    });

    let store = store::Store::new()?;

    // these commands do not use the git repo
    match cmd {
        Commands::Clean => {
            return commands::clean::cmd(store);
        }
        Commands::Gc => {
            panic!("not implemented");
        }
        Commands::InitTemplatedir(_) => {
            panic!("not implemented");
        }
        Commands::SampleConfig => {
            return commands::sample_config::cmd();
        }
        Commands::ValidateConfig(cmd) => {
            return commands::validate_config::cmd(cmd);
        }
        Commands::ValidateManifest(cmd) => {
            return commands::validate_manifest::cmd(cmd);
        }
        _ => (),
    }

    let (config, repo, chdir) = _chdir_to_git_root(res.config)?;
    store.mark_config_used(&config)?;

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match cmd {
        Commands::Autoupdate(_) => {
            panic!("not implemented!");
        }
        Commands::Install(_) => {
            panic!("not implemented!");
        }
        Commands::InstallHooks => {
            panic!("not implemented!");
        }
        Commands::MigrateConfig => {
            panic!("not implemented!");
        }
        Commands::Run(mut cmd) => {
            if let Some(chdir) = chdir {
                _adjust_run(&mut cmd, &chdir);
            }
            commands::run::cmd(config, repo, store, cmd)
        }
        Commands::TryRepo(mut cmd) => {
            if let Some(chdir) = chdir {
                cmd.repo = _chdir_path(&cmd.repo, &chdir);
                _adjust_run(&mut cmd.run, &chdir);
            }
            panic!("not implemented!");
        }
        Commands::Uninstall(_) => {
            panic!("not implemented!");
        }
        Commands::HookImpl(_) => {
            panic!("not implemented!");
        }
        _ => unreachable!(),
    }
}
