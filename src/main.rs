use clap::{Parser, Subcommand};

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
    /// Produce a sample .pre-commit-config.yaml file
    SampleConfig,
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
        Some(Commands::SampleConfig) => {
            commands::sample_config::cmd();
        }
        None => {
            println!("default")
        }
    }
    Ok(())
}
