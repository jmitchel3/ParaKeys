//! ParaKeys CLI: like Apple Passwords, but for dotenv.

mod agent;
mod cli;
mod commands;
mod config;
mod envfile;
mod error;
mod keywallet;
mod status;
mod vault;

use anyhow::Result;
use clap::Parser;

use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::dispatch(cli)
}
