use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::agent::{create_grant, load_recipient_key_from_pub_file, save_grant};
use crate::keywallet::{load_local_key, project_root};
use crate::vault::{default_vault_path, load_vault};

pub fn create(to: PathBuf, keys: String, out: PathBuf, path: Option<PathBuf>) -> Result<()> {
    let root = project_root(path).context("resolve project path")?;
    let vault_path = default_vault_path(&root);
    if !vault_path.is_file() {
        bail!("no vault at {}", vault_path.display());
    }
    let allowlist: Vec<String> = keys
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if allowlist.is_empty() {
        bail!("provide at least one key via --keys A,B");
    }

    let human = load_local_key(&root).context("load human local key")?;
    let vault = load_vault(&root, &human).context("decrypt vault")?;
    let recipient = load_recipient_key_from_pub_file(&to).context("load agent.pub")?;
    let grant = create_grant(&vault, &allowlist, &recipient).context("create grant")?;

    let out_path = if out.is_absolute() {
        out
    } else {
        root.join(out)
    };
    save_grant(&out_path, &grant).context("write grant")?;

    let raw = std::fs::read_to_string(&out_path)?;
    for name in &allowlist {
        if let Some(v) = vault.get(name) {
            if !v.is_empty() && raw.contains(v) {
                bail!("secret value for `{name}` leaked into grant file");
            }
        }
    }

    println!(
        "Wrote grant for {} key(s) to {}",
        allowlist.len(),
        out_path.display()
    );
    for k in &allowlist {
        println!("  {k}");
    }
    Ok(())
}
