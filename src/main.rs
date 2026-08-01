//! ParaKeys CLI: like Apple Passwords, but for dotenv.

use anyhow::Result;
use clap::Parser;
use parakeys::cli::Cli;
use parakeys::commands;

fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::dispatch(cli)
}
