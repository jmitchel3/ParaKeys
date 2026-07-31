//! ParaKeys CLI: like Apple Passwords, but for dotenv.

mod cli;
mod commands;
mod envfile;
mod error;
mod keywallet;
mod vault;

use anyhow::Result;
use clap::Parser;

use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::dispatch(cli)
}
