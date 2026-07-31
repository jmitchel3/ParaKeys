use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::envfile::{
    load_env_file, save_env_file, EnvFile, EnvLine, PLACEHOLDER_NOT_SET, PLACEHOLDER_SET,
};
use crate::keywallet::{load_local_key, project_root};
use crate::vault::{default_vault_path, load_vault};

pub fn sync(path: Option<PathBuf>, env_file: PathBuf) -> Result<()> {
    let root = project_root(path).context("resolve project path")?;
    let vault_path = default_vault_path(&root);
    if !vault_path.is_file() {
        bail!(
            "no vault at {} — run `parakeys init` first",
            vault_path.display()
        );
    }

    let key = load_local_key(&root).context("load local key")?;
    let vault = load_vault(&root, &key).context("decrypt vault")?;
    let vault_keys: BTreeSet<String> = vault.keys.keys().cloned().collect();

    let env_path = if env_file.is_absolute() {
        env_file
    } else {
        root.join(env_file)
    };

    let mut env = if env_path.is_file() {
        load_env_file(&env_path).context("read .env")?
    } else {
        EnvFile::default()
    };

    // Update existing assignment lines.
    let mut seen = BTreeSet::new();
    for line in &mut env.lines {
        if let EnvLine::Assignment { key: k, value, .. } = line {
            seen.insert(k.clone());
            if vault_keys.contains(k) {
                *value = PLACEHOLDER_SET.to_string();
            } else {
                // Declared in .env but no longer in vault
                *value = PLACEHOLDER_NOT_SET.to_string();
            }
        }
    }

    // Append missing vault keys.
    for k in &vault_keys {
        if !seen.contains(k) {
            env.lines.push(EnvLine::Assignment {
                key: k.clone(),
                value: PLACEHOLDER_SET.to_string(),
                export: false,
            });
        }
    }

    save_env_file(&env_path, &env).context("write .env")?;

    // Safety: no secret values from vault appear in file.
    let rendered = env.render();
    for (_k, v) in &vault.keys {
        if !v.is_empty() && rendered.contains(v) {
            bail!("internal error: secret value leaked into .env render");
        }
    }

    println!(
        "Synced {} vault key(s) into {} (placeholders only).",
        vault_keys.len(),
        env_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keywallet::store_local_key;
    use crate::vault::{save_vault, VaultData, VaultKey};
    use std::fs;

    #[test]
    fn sync_writes_placeholders_only() {
        let dir = std::env::temp_dir().join(format!("pk-manifest-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let key = VaultKey::generate();
        store_local_key(&dir, &key).unwrap();
        let mut data = VaultData::new();
        data.set("SECRET_TOKEN", "super-secret-value-xyz");
        data.set("OTHER", "other-secret-abc");
        save_vault(&dir, &data, &key).unwrap();

        fs::write(dir.join(".env"), "STALE=old-plain\n").unwrap();
        sync(Some(dir.clone()), PathBuf::from(".env")).unwrap();
        let text = fs::read_to_string(dir.join(".env")).unwrap();
        assert!(text.contains("SECRET_TOKEN=<set in parakeys>"));
        assert!(text.contains("OTHER=<set in parakeys>"));
        assert!(text.contains("STALE=<not set in parakeys>"));
        assert!(!text.contains("super-secret-value-xyz"));
        assert!(!text.contains("other-secret-abc"));
        let _ = fs::remove_dir_all(&dir);
    }
}
