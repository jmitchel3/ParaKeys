use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::keywallet::{load_local_key, project_root};
use crate::vault::{default_vault_path, load_vault};

pub fn run(reveal: bool, path: Option<PathBuf>) -> Result<()> {
    let root = project_root(path).context("resolve project path")?;
    let vault_path = default_vault_path(&root);
    if !vault_path.is_file() {
        bail!(
            "no vault at {} — run `parakeys init` first",
            vault_path.display()
        );
    }

    let vault_key = load_local_key(&root).context("load local key")?;
    let vault = load_vault(&root, &vault_key).context("decrypt vault")?;

    if vault.keys.is_empty() {
        println!("(vault is empty)");
        return Ok(());
    }

    for (name, value) in &vault.keys {
        if reveal {
            println!("{name}={value}");
        } else {
            println!("{name}=<set in parakeys>");
        }
    }
    Ok(())
}
