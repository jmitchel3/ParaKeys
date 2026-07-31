//! Command handlers. Implementations land in later MVP cards.

mod doctor;
mod import;
mod init;
mod list;
mod run;
mod set;
mod unset;

use anyhow::Result;

use crate::cli::{Cli, Commands};

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init { path } => init::run(path),
        Commands::Import { env_file, path } => import::run(env_file, path),
        Commands::Set { key, value, path } => set::run(key, value, path),
        Commands::Unset { key, path } => unset::run(key, path),
        Commands::List { reveal, path } => list::run(reveal, path),
        Commands::Run { command, path } => run::run(command, path),
        Commands::Doctor { path } => doctor::run(path),
    }
}
