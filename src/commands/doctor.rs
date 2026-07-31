use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::{config_path, load_config};
use crate::envfile::{is_placeholder, load_env_file};
use crate::keywallet::{has_local_key, local_key_path, project_root};
use crate::vault::default_vault_path;

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

    let env_path = root.join(".env");
    if env_path.is_file() {
        let env = load_env_file(&env_path).context("read .env")?;
        let mut leak_keys = Vec::new();
        for (k, v) in env.assignments() {
            if looks_like_secret(v) {
                leak_keys.push(k.to_string());
            }
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
}
