use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::envfile::load_env_file;
use crate::keywallet::{load_local_key, project_root};
use crate::status::{classify_key, status_label, KeyStatus};
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

    let mut names: BTreeSet<String> = vault.keys.keys().cloned().collect();
    let mut env_values = std::collections::BTreeMap::<String, String>::new();
    let env_path = root.join(".env");
    if env_path.is_file() {
        let env = load_env_file(&env_path).context("read .env")?;
        for (k, v) in env.assignments() {
            names.insert(k.to_string());
            env_values.insert(k.to_string(), v.to_string());
        }
    }

    if names.is_empty() {
        println!("(no keys in vault or .env)");
        return Ok(());
    }

    for name in names {
        let env_v = env_values.get(&name).map(String::as_str);
        let st = classify_key(&vault, &name, env_v);
        if reveal {
            match st {
                KeyStatus::SetInVault => {
                    let val = vault.get(&name).unwrap_or("");
                    println!("{name}={val}");
                }
                KeyStatus::NotSet => println!("{name}={}", status_label(st)),
                KeyStatus::PlaintextOnDisk => {
                    println!("{name}={}  # warning: plaintext still in .env", env_v.unwrap_or(""));
                }
            }
        } else {
            println!("{name}={}", status_label(st));
        }
    }
    Ok(())
}
