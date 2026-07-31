use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::keywallet::{
    decode_recovery_code, encode_recovery_code, has_local_key, project_root, store_local_key,
};
use crate::vault::{
    default_vault_path, load_vault, save_vault, VaultData, VaultKey,
};

pub fn run(path: Option<PathBuf>, recover: Option<String>, force: bool) -> Result<()> {
    let root = project_root(path).context("resolve project path")?;
    let vault_path = default_vault_path(&root);

    if let Some(code) = recover {
        return recover_key(&root, &code, force);
    }

    if vault_path.is_file() && !force {
        bail!(
            "vault already exists at {} (use --force to recreate, or --recover CODE to restore the local key)",
            vault_path.display()
        );
    }
    if has_local_key(&root) && !force {
        bail!(
            "local key already exists at {} (use --force to overwrite)",
            crate::keywallet::local_key_path(&root).display()
        );
    }

    let key = VaultKey::generate();
    let data = VaultData::new();
    save_vault(&root, &data, &key).context("write empty vault")?;
    store_local_key(&root, &key).context("store local key")?;
    let code = encode_recovery_code(&key);

    println!("Created ParaKeys vault at {}", vault_path.display());
    println!("Local key stored at {}", crate::keywallet::local_key_path(&root).display());
    println!();
    println!("RECOVERY CODE (store offline; shown once):");
    println!("{code}");
    println!();
    println!("Anyone with this code can decrypt the vault. Do not commit it or paste it into agent chat.");
    println!("Add `.parakeys/local.key` to .gitignore if it is not already ignored.");

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
    if has_local_key(root) && !force {
        bail!(
            "local key already present (use --force to replace it from the recovery code)"
        );
    }

    let key = decode_recovery_code(code).context("decode recovery code")?;
    // Prove the code opens this vault before writing the local key.
    let _data = load_vault(root, &key).context(
        "recovery code does not decrypt this vault (wrong code or wrong project)",
    )?;
    store_local_key(root, &key).context("store local key")?;

    println!(
        "Local key restored at {}",
        crate::keywallet::local_key_path(root).display()
    );
    println!("Vault unlocks successfully.");
    Ok(())
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
    // Best-effort append; not fatal.
    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&gi) {
        use std::io::Write;
        let _ = writeln!(f, "\n# ParaKeys local unlock key (never commit)\n.parakeys/local.key");
        println!("Appended `.parakeys/local.key` to .gitignore");
    }
    Ok(())
}
