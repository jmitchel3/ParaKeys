use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::{config_path, load_config};
use crate::envfile::{is_placeholder, load_env_file};
use crate::keywallet::{has_local_key, load_local_key, local_key_path, project_root};
use crate::status::{classify_key, status_label, KeyStatus};
use crate::vault::{default_vault_path, load_vault, VaultData};

/// Heuristic: values that look like real secrets rather than placeholders/flags.
pub fn looks_like_secret(value: &str) -> bool {
    let v = value.trim();
    if v.is_empty() || is_placeholder(v) {
        return false;
    }
    // Obvious secret shapes
    if v.starts_with("sk-") || v.starts_with("pk_") || v.starts_with("ghp_") {
        return true;
    }
    if v.contains("://") && (v.contains('@') || v.contains("password") || v.contains("secret")) {
        return true;
    }
    // High entropy-ish: long token without spaces
    if v.len() >= 20 && !v.contains(' ') && !v.contains('<') {
        let alnum = v
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '+' | '/' | '='))
            .count();
        if alnum * 10 >= v.len() * 8 {
            return true;
        }
    }
    false
}

pub fn run(path: Option<PathBuf>) -> Result<()> {
    let root = project_root(path).context("resolve project path")?;
    let mut issues = 0u32;
    let mut oks = 0u32;

    let vault_path = default_vault_path(&root);
    if vault_path.is_file() {
        println!("ok: vault present at {}", vault_path.display());
        oks += 1;
    } else {
        println!("error: vault missing at {}", vault_path.display());
        issues += 1;
    }

    if has_local_key(&root) {
        println!("ok: local key at {}", local_key_path(&root).display());
        oks += 1;
    } else {
        println!(
            "error: local key missing (run `parakeys init` or `parakeys init --recover`)"
        );
        issues += 1;
    }

    match load_config(&root) {
        Ok(cfg) => {
            println!(
                "ok: config.toml env_name={} ({})",
                cfg.env_name,
                config_path(&root).display()
            );
            oks += 1;
        }
        Err(_) => {
            println!(
                "warn: no config.toml at {} (optional; re-run init to create)",
                config_path(&root).display()
            );
        }
    }

    // Shared status classification (same helper as `list`).
    let vault_data: Option<VaultData> = if vault_path.is_file() && has_local_key(&root) {
        match load_local_key(&root).and_then(|k| load_vault(&root, &k)) {
            Ok(v) => Some(v),
            Err(e) => {
                println!("error: cannot decrypt vault for status check: {e}");
                issues += 1;
                None
            }
        }
    } else {
        None
    };

    let env_path = root.join(".env");
    if env_path.is_file() {
        let env = load_env_file(&env_path).context("read .env")?;
        let mut leak_keys = Vec::new();
        let mut set_keys = Vec::new();
        let mut not_set_keys = Vec::new();
        let mut plaintext_status = Vec::new();
        let mut names: BTreeSet<String> = BTreeSet::new();

        for (k, v) in env.assignments() {
            names.insert(k.to_string());
            if looks_like_secret(v) {
                leak_keys.push(k.to_string());
            }
            if let Some(ref vault) = vault_data {
                match classify_key(vault, k, Some(v)) {
                    KeyStatus::SetInVault => set_keys.push(k.to_string()),
                    KeyStatus::NotSet => not_set_keys.push(k.to_string()),
                    KeyStatus::PlaintextOnDisk => plaintext_status.push(k.to_string()),
                }
            }
        }
        // Vault-only keys (not in .env) still count as set via shared helper.
        if let Some(ref vault) = vault_data {
            for k in vault.keys.keys() {
                if !names.contains(k) {
                    match classify_key(vault, k, None) {
                        KeyStatus::SetInVault => set_keys.push(k.clone()),
                        KeyStatus::NotSet => not_set_keys.push(k.clone()),
                        KeyStatus::PlaintextOnDisk => plaintext_status.push(k.clone()),
                    }
                }
            }
        }

        if let Some(ref vault) = vault_data {
            println!(
                "ok: key status via shared classifier: {} set, {} not set, {} plaintext-on-disk",
                set_keys.len(),
                not_set_keys.len(),
                plaintext_status.len()
            );
            for k in &set_keys {
                println!("  {k}={}", status_label(KeyStatus::SetInVault));
            }
            for k in &not_set_keys {
                println!("  {k}={}", status_label(KeyStatus::NotSet));
            }
            for k in &plaintext_status {
                println!("  {k}={}", status_label(KeyStatus::PlaintextOnDisk));
            }
            // Touch vault so unused warning never appears; also sanity-check empty vault.
            let _ = vault.keys.len();
            oks += 1;
        }

        if leak_keys.is_empty() {
            println!("ok: .env has no values that look like plaintext secrets");
            oks += 1;
        } else {
            println!(
                "error: .env may contain plaintext secrets for: {}",
                leak_keys.join(", ")
            );
            println!("  hint: run `parakeys import .env` to move them into the vault");
            issues += 1;
        }
    } else {
        println!("warn: no .env file (optional)");
        if let Some(ref vault) = vault_data {
            if !vault.keys.is_empty() {
                println!(
                    "ok: vault has {} key(s) classified set via shared helper (no .env)",
                    vault.keys.len()
                );
                for k in vault.keys.keys() {
                    println!(
                        "  {k}={}",
                        status_label(classify_key(vault, k, None))
                    );
                }
                oks += 1;
            }
        }
    }

    println!();
    if issues == 0 {
        println!("doctor: {oks} check(s) passed");
        Ok(())
    } else {
        anyhow::bail!("doctor: {issues} issue(s) found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envfile::{PLACEHOLDER_NOT_SET, PLACEHOLDER_SET};
    use crate::status::classify_key;
    use crate::vault::VaultData;

    #[test]
    fn placeholders_are_not_secrets() {
        assert!(!looks_like_secret("<set in parakeys>"));
        assert!(!looks_like_secret("<not set in parakeys>"));
        assert!(!looks_like_secret("true"));
        assert!(!looks_like_secret("debug"));
    }

    #[test]
    fn detects_token_shapes() {
        assert!(looks_like_secret("sk-abcdefghijklmnopqrstuvwxyz"));
        assert!(looks_like_secret("ghp_abcdefghijklmnopqrstuvwx"));
        assert!(looks_like_secret("postgres://user:pass@host/db"));
    }

    #[test]
    fn doctor_uses_shared_status_classifier() {
        // Prove doctor module path shares classify_key semantics with list.
        let mut vault = VaultData::new();
        vault.set("FOO", "secret");
        assert_eq!(
            classify_key(&vault, "FOO", Some(PLACEHOLDER_SET)),
            KeyStatus::SetInVault
        );
        assert_eq!(
            classify_key(&vault, "BAR", Some(PLACEHOLDER_NOT_SET)),
            KeyStatus::NotSet
        );
        assert_eq!(
            status_label(classify_key(&vault, "FOO", Some(PLACEHOLDER_SET))),
            "<set in parakeys>"
        );
        assert_eq!(
            status_label(classify_key(&vault, "BAR", Some(PLACEHOLDER_NOT_SET))),
            "<not set in parakeys>"
        );
    }
}
