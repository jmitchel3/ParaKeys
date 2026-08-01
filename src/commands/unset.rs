use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::envfile::{
    load_env_file, save_env_file, EnvLine, PLACEHOLDER_NOT_SET,
};
use crate::keywallet::{load_unlock_key, project_root};
use crate::vault::{default_vault_path, load_vault, save_vault};

pub fn run(key: String, path: Option<PathBuf>) -> Result<()> {
    if key.is_empty() {
        bail!("key name must not be empty");
    }

    let root = project_root(path).context("resolve project path")?;
    let vault_path = default_vault_path(&root);
    if !vault_path.is_file() {
        bail!(
            "no vault at {} — run `parakeys init` first",
            vault_path.display()
        );
    }

    let vault_key = load_unlock_key(&root).context("load local key")?;
    let mut vault = load_vault(&root, &vault_key).context("decrypt vault")?;
    if !vault.unset(&key) {
        bail!("key `{key}` is not in the vault");
    }
    save_vault(&root, &vault, &vault_key).context("write vault")?;

    let env_path = root.join(".env");
    if env_path.is_file() {
        let mut env = load_env_file(&env_path).context("read .env")?;
        let mut found = false;
        for line in &mut env.lines {
            if let EnvLine::Assignment {
                key: k, value, ..
            } = line
            {
                if k == &key {
                    *value = PLACEHOLDER_NOT_SET.to_string();
                    found = true;
                }
            }
        }
        if found {
            save_env_file(&env_path, &env).context("rewrite .env")?;
        }
    }

    println!("Unset `{key}` from vault.");
    Ok(())
}
