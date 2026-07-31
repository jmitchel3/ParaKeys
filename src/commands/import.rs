use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::envfile::{load_env_file, save_env_file};
use crate::keywallet::{load_local_key, project_root};
use crate::vault::{default_vault_path, load_vault, save_vault};

pub fn run(env_file: PathBuf, path: Option<PathBuf>) -> Result<()> {
    let root = project_root(path).context("resolve project path")?;
    let vault_path = default_vault_path(&root);
    if !vault_path.is_file() {
        bail!(
            "no vault at {} — run `parakeys init` first",
            vault_path.display()
        );
    }

    let env_path = if env_file.is_absolute() {
        env_file
    } else {
        root.join(env_file)
    };
    if !env_path.is_file() {
        bail!("env file not found: {}", env_path.display());
    }

    let key = load_local_key(&root).context("load local key")?;
    let mut vault = load_vault(&root, &key).context("decrypt vault")?;
    let mut env = load_env_file(&env_path).context("read env file")?;

    let candidates = env.secret_candidates();
    if candidates.is_empty() {
        bail!(
            "no plaintext secret values found in {} (already placeholders or empty)",
            env_path.display()
        );
    }

    let mut imported_keys = Vec::new();
    for (k, v) in candidates {
        vault.set(k.clone(), v);
        imported_keys.push(k);
    }

    save_vault(&root, &vault, &key).context("write vault")?;
    env.rewrite_placeholders(&imported_keys);
    save_env_file(&env_path, &env).context("rewrite env file")?;

    println!(
        "Imported {} key(s) into {}",
        imported_keys.len(),
        vault_path.display()
    );
    for k in &imported_keys {
        println!("  {k}");
    }
    println!(
        "Rewrote {} with placeholders (values no longer on disk).",
        env_path.display()
    );
    Ok(())
}
