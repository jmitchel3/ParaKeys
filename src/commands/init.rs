use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::config::{save_config, ProjectConfig};
use crate::keywallet::{
    clear_unlock_key, decode_recovery_code, detect_backend, encode_recovery_code, has_unlock_key,
    project_root, store_unlock_key, WalletBackend,
};
use crate::vault::{default_vault_path, load_vault, save_vault, VaultData, VaultKey};

pub fn run(path: Option<PathBuf>, recover: Option<String>, force: bool) -> Result<()> {
    let root = project_root(path).context("resolve project path")?;
    let vault_path = default_vault_path(&root);

    if let Some(code) = recover {
        return recover_key(&root, &code, force);
    }

    if vault_path.is_file() && !force {
        bail!(
            "vault already exists at {} (use --force to recreate, or --recover CODE to restore unlock)",
            vault_path.display()
        );
    }
    if has_unlock_key(&root) && !force {
        bail!("unlock key already present for this project (use --force to overwrite)");
    }
    if force {
        let _ = clear_unlock_key(&root);
    }

    let key = VaultKey::generate();
    let data = VaultData::new();
    save_vault(&root, &data, &key).context("write empty vault")?;
    let outcome = store_unlock_key(&root, &key).context("store unlock key")?;
    let env_name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("default")
        .to_string();
    let cfg = ProjectConfig {
        format_version: 0,
        env_name: env_name.clone(),
    };
    let cfg_path = save_config(&root, &cfg).context("write config.toml")?;
    let code = encode_recovery_code(&key);

    println!("Created ParaKeys vault at {}", vault_path.display());
    println!(
        "Project config (non-secret) at {} (env_name={env_name})",
        cfg_path.display()
    );
    print_backend(&outcome.backend, &root);
    for note in &outcome.notes {
        eprintln!("note: {note}");
    }
    println!();
    println!("RECOVERY CODE (store offline; shown once):");
    println!("{code}");
    println!();
    println!("Anyone with this code can decrypt the vault. Do not commit it or paste it into agent chat.");
    println!("Add `.parakeys/local.key` to .gitignore if using the file wallet.");

    ensure_gitignore_hint(&root)?;

    Ok(())
}

fn recover_key(root: &std::path::Path, code: &str, force: bool) -> Result<()> {
    let vault_path = default_vault_path(root);
    if !vault_path.is_file() {
        bail!(
            "no vault at {} — clone/pull the project ciphertext first, then re-run with --recover",
            vault_path.display()
        );
    }
    if has_unlock_key(root) && !force {
        bail!("unlock key already present (use --force to replace it from the recovery code)");
    }
    if force {
        let _ = clear_unlock_key(root);
    }

    let key = decode_recovery_code(code).context("decode recovery code")?;
    let _data = load_vault(root, &key).context(
        "recovery code does not decrypt this vault (wrong code or corrupt project)",
    )?;
    let outcome = store_unlock_key(root, &key).context("store unlock key")?;

    print_backend(&outcome.backend, root);
    for note in &outcome.notes {
        eprintln!("note: {note}");
    }
    if let Some(b) = detect_backend(root) {
        println!("Active backend: {}", b.as_str());
    }
    println!("Vault unlocks successfully.");
    Ok(())
}

fn print_backend(backend: &WalletBackend, root: &std::path::Path) {
    match backend {
        WalletBackend::KeychainUserPresence => {
            println!(
                "Unlock key stored in macOS Keychain with user presence (Touch ID / passcode)."
            );
        }
        WalletBackend::Keychain => {
            println!("Unlock key stored in macOS Keychain.");
        }
        WalletBackend::File => {
            println!(
                "Unlock key stored in file wallet at {}",
                crate::keywallet::local_key_path(root).display()
            );
        }
    }
}

fn ensure_gitignore_hint(root: &std::path::Path) -> Result<()> {
    let gi = root.join(".gitignore");
    if !gi.is_file() {
        return Ok(());
    }
    let text = std::fs::read_to_string(&gi).unwrap_or_default();
    if text.lines().any(|l| {
        let t = l.trim();
        t == ".parakeys/local.key" || t == "**/.parakeys/local.key" || t.ends_with("local.key")
    }) {
        return Ok(());
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&gi) {
        use std::io::Write;
        let _ = writeln!(
            f,
            "\n# ParaKeys local unlock key fallback (never commit)\n.parakeys/local.key"
        );
        println!("Appended `.parakeys/local.key` to .gitignore");
    }
    Ok(())
}
