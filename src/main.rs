use clap::{Parser, Subcommand};

mod clientlib;
mod commands;
mod store;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Clean out pre-commit files
    Clean,
    /// Produce a sample .pre-commit-config.yaml file
    SampleConfig,
    /// Validate .pre-commit-config.yaml files
    ValidateConfig { filenames: Vec<String> },
    /// Validate .pre-commit-hooks.yaml files
    ValidateManifest { filenames: Vec<String> },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let store = store::Store::new()?;
    println!(
        "got store! @{:?} (readonly? {})",
        store.directory, store.readonly
    );

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Clean) => {
            commands::clean::cmd(&store)?;
        }
        Some(Commands::SampleConfig) => {
            commands::sample_config::cmd();
        }
        Some(Commands::ValidateConfig { filenames }) => {
            commands::validate_config::cmd(filenames)?;
        }
        Some(Commands::ValidateManifest { filenames }) => {
            commands::validate_manifest::cmd(filenames)?;
        }
        None => {
            println!("default")
        }
    }
    Ok(())
}
