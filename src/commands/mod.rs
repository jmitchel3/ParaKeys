//! Command handlers. Implementations land in later MVP cards.

mod agent_cmd;
mod doctor;
mod grant_cmd;
mod import;
mod init;
mod list;
mod manifest;
mod run;
mod set;
mod unset;

use anyhow::Result;

use crate::cli::{AgentCmd, Cli, Commands, GrantCmd, ManifestCmd};

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init {
            path,
            recover,
            force,
        } => init::run(path, recover, force),
        Commands::Import { env_file, path } => import::run(env_file, path),
        Commands::Set { key, value, path } => set::run(key, value, path),
        Commands::Unset { key, path } => unset::run(key, path),
        Commands::List { reveal, path } => list::run(reveal, path),
        Commands::Run { command, path } => run::run(command, path),
        Commands::Doctor { path } => doctor::run(path),
        Commands::Manifest { action } => match action {
            ManifestCmd::Sync { path, env_file } => manifest::sync(path, env_file),
        },
        Commands::Agent { action } => match action {
            AgentCmd::Keygen { path } => agent_cmd::keygen(path),
            AgentCmd::Apply { grant, path } => agent_cmd::apply(grant, path),
            AgentCmd::Run { command, path } => agent_cmd::run(command, path),
        },
        Commands::Grant { action } => match action {
            GrantCmd::Create { to, keys, out, path } => grant_cmd::create(to, keys, out, path),
        },
    }
}
