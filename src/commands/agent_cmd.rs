use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::agent;
use crate::keywallet::project_root;

pub fn keygen(path: Option<PathBuf>) -> Result<()> {
    let root = project_root(path).context("resolve project path")?;
    let (key_path, pub_path, pub_b64) = agent::keygen(&root).context("agent keygen")?;
    println!("Agent private key: {} (mode 0600; do not commit)", key_path.display());
    println!("Agent public key:  {}", pub_path.display());
    println!();
    println!("Share the public key file with the human vault owner to create grants:");
    println!("{pub_b64}");
    Ok(())
}
