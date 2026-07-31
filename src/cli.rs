//! Clap command definitions for the ParaKeys binary.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Like Apple Passwords, but for dotenv.
#[derive(Debug, Parser)]
#[command(name = "parakeys", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create a new vault and recovery code, or restore a local key from recovery
    Init {
        /// Project directory (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,

        /// Restore the local key from a recovery code (does not create a new vault)
        #[arg(long)]
        recover: Option<String>,

        /// Overwrite an existing vault and/or local key
        #[arg(long)]
        force: bool,
    },

    /// Import a plaintext .env into the vault and rewrite placeholders
    Import {
        /// Path to the .env file to import
        #[arg(default_value = ".env")]
        env_file: PathBuf,

        /// Project directory (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Set a key in the vault
    Set {
        /// KEY=value or KEY followed by --value
        key: String,

        /// Value when not using KEY=value form
        #[arg(long)]
        value: Option<String>,

        /// Project directory (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Remove a key from the vault
    Unset {
        key: String,

        /// Project directory (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// List key names and set/missing status (values hidden by default)
    List {
        /// Print secret values (use with care)
        #[arg(long)]
        reveal: bool,

        /// Project directory (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Run a command with vault secrets injected into the process environment
    Run {
        /// Command and arguments after `--`
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
        command: Vec<String>,

        /// Project directory (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Check vault, key wallet, and .env hygiene
    Doctor {
        /// Project directory (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Manage the placeholder `.env` manifest
    Manifest {
        #[command(subcommand)]
        action: ManifestCmd,
    },

    /// Agent recipient tooling (grants / keygen)
    Agent {
        #[command(subcommand)]
        action: AgentCmd,
    },

    /// Create encrypted grants for agents
    Grant {
        #[command(subcommand)]
        action: GrantCmd,
    },
}

#[derive(Debug, Subcommand)]
pub enum ManifestCmd {
    /// Rewrite/create `.env` from vault key names (placeholders only)
    Sync {
        /// Project directory (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,

        /// Env file path relative to project (default: .env)
        #[arg(long, default_value = ".env")]
        env_file: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum AgentCmd {
    /// Generate agent recipient keypair under `.parakeys-agent/`
    Keygen {
        /// Project directory (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Store a grant file for local agent use (no human vault key required)
    Apply {
        /// Path to grant.enc
        grant: PathBuf,

        /// Project directory (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum GrantCmd {
    /// Encrypt allowlisted vault keys to an agent public key
    Create {
        /// Path to agent.pub (recipient key)
        #[arg(long = "to")]
        to: PathBuf,

        /// Comma-separated key names to include
        #[arg(long = "keys")]
        keys: String,

        /// Output grant file path
        #[arg(long = "out", default_value = "grant.enc")]
        out: PathBuf,

        /// Project directory (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },
}
