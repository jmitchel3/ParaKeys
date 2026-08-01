use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::envfile::{is_placeholder, load_env_file, PLACEHOLDER_SET};
use crate::keywallet::{load_unlock_key, project_root};
use crate::vault::{default_vault_path, load_vault};

pub fn run(command: Vec<String>, path: Option<PathBuf>) -> Result<()> {
    if command.is_empty() {
        bail!("usage: parakeys run -- <command> [args...]");
    }

    let root = project_root(path).context("resolve project path")?;
    let vault_path = default_vault_path(&root);
    if !vault_path.is_file() {
        bail!(
            "no vault at {} — run `parakeys init` first",
            vault_path.display()
        );
    }

    let key = load_unlock_key(&root).context("load local key")?;
    let vault = load_vault(&root, &key).context("decrypt vault")?;

    // Start from the current process environment, then overlay vault + .env merge.
    let mut env_map: HashMap<String, String> = std::env::vars().collect();

    // All vault keys are available to the child by default.
    for (k, v) in &vault.keys {
        env_map.insert(k.clone(), v.clone());
    }

    // If a .env exists, resolve placeholders and pass through non-placeholder values.
    let env_path = root.join(".env");
    if env_path.is_file() {
        let file = load_env_file(&env_path).context("read .env")?;
        for (k, v) in file.assignments() {
            if is_placeholder(v) {
                if v.trim() == PLACEHOLDER_SET {
                    match vault.get(k) {
                        Some(secret) => {
                            env_map.insert(k.to_string(), secret.to_string());
                        }
                        None => {
                            bail!(
                                "key `{k}` is `{PLACEHOLDER_SET}` in .env but missing from the vault"
                            );
                        }
                    }
                }
                // `<not set in parakeys>`: leave unset / do not inject
            } else if !v.is_empty() {
                // Plaintext non-secret still on disk: pass through (may override vault).
                env_map.insert(k.to_string(), v.to_string());
            }
        }
    }

    let program = &command[0];
    let args = &command[1..];

    let status = Command::new(program)
        .args(args)
        .envs(&env_map)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to execute `{program}`"))?;

    if let Some(code) = status.code() {
        if code != 0 {
            std::process::exit(code);
        }
    } else {
        // Terminated by signal
        std::process::exit(1);
    }
    Ok(())
}
