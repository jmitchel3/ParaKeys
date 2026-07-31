//! Local key material storage (file wallet for v0; Keychain later).
//!
//! The vault key is 32 random bytes. The recovery code is a Crockford base32
//! encoding of that key (shown once at init). The same bytes are stored in a
//! local, gitignored key file for day-to-day unlock.

use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use data_encoding::BASE32_NOPAD;

use crate::error::ParaKeysError;
use crate::vault::VaultKey;

/// Relative path of the local key file (must stay out of git).
pub const LOCAL_KEY_REL: &str = ".parakeys/local.key";

/// Recovery codes are Crockford-ish base32 without padding, grouped for reading.
const RECOVERY_GROUP: usize = 4;

pub fn local_key_path(project_root: &Path) -> PathBuf {
    project_root.join(LOCAL_KEY_REL)
}

/// Encode a vault key as a recovery code string (`xxxx-xxxx-...`).
pub fn encode_recovery_code(key: &VaultKey) -> String {
    let encoded = BASE32_NOPAD.encode(key.as_bytes());
    // data-encoding BASE32 uses A-Z2-7; lowercase for display comfort.
    let lower = encoded.to_ascii_lowercase();
    lower
        .as_bytes()
        .chunks(RECOVERY_GROUP)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect::<Vec<_>>()
        .join("-")
}

/// Decode a recovery code back into a vault key.
pub fn decode_recovery_code(code: &str) -> Result<VaultKey, ParaKeysError> {
    let compact: String = code
        .chars()
        .filter(|c| *c != '-' && !c.is_whitespace())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    let bytes = BASE32_NOPAD
        .decode(compact.as_bytes())
        .map_err(|e| ParaKeysError::InvalidVault(format!("invalid recovery code: {e}")))?;
    VaultKey::try_from_slice(&bytes)
}

/// Write the vault key to the local key file with mode 0600.
pub fn store_local_key(project_root: &Path, key: &VaultKey) -> Result<PathBuf, ParaKeysError> {
    let path = local_key_path(project_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(ParaKeysError::Io)?;
    }

    // Overwrite securely enough for v0: truncate and write raw bytes.
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    opts.mode(0o600);

    let mut file = opts.open(&path).map_err(ParaKeysError::Io)?;
    file.write_all(key.as_bytes()).map_err(ParaKeysError::Io)?;
    file.sync_all().map_err(ParaKeysError::Io)?;

    // Ensure permissions even if umask interfered on create.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path).map_err(ParaKeysError::Io)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms).map_err(ParaKeysError::Io)?;
    }

    Ok(path)
}

/// Load the vault key from the local key file.
#[allow(dead_code)] // used by import/run/list in later MVP cards
pub fn load_local_key(project_root: &Path) -> Result<VaultKey, ParaKeysError> {
    let path = local_key_path(project_root);
    let bytes = fs::read(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ParaKeysError::VaultNotFound(format!(
                "local key missing at {} (run `parakeys init` or `parakeys init --recover <code>`)",
                path.display()
            ))
        } else {
            ParaKeysError::Io(e)
        }
    })?;
    VaultKey::try_from_slice(&bytes)
}

/// True if a local key file exists.
pub fn has_local_key(project_root: &Path) -> bool {
    local_key_path(project_root).is_file()
}

/// Remove the local key file (for tests / explicit reset).
#[allow(dead_code)]
pub fn clear_local_key(project_root: &Path) -> Result<(), ParaKeysError> {
    let path = local_key_path(project_root);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(ParaKeysError::Io(e)),
    }
}

/// Resolve project root from an optional `--path` (default: cwd).
pub fn project_root(path: Option<PathBuf>) -> Result<PathBuf, ParaKeysError> {
    match path {
        Some(p) => Ok(p),
        None => std::env::current_dir().map_err(ParaKeysError::Io),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::{encrypt, decrypt, VaultData};

    #[test]
    fn recovery_code_round_trip() {
        let key = VaultKey::generate();
        let code = encode_recovery_code(&key);
        assert!(code.contains('-'), "expected grouped code, got {code}");
        let restored = decode_recovery_code(&code).unwrap();
        assert_eq!(restored.as_bytes(), key.as_bytes());
    }

    #[test]
    fn recovery_code_ignores_case_and_spaces() {
        let key = VaultKey::generate();
        let code = encode_recovery_code(&key);
        let messy = code.to_ascii_uppercase().replace('-', " - ");
        let restored = decode_recovery_code(&messy).unwrap();
        assert_eq!(restored.as_bytes(), key.as_bytes());
    }

    #[test]
    fn local_key_store_and_load() {
        let dir = std::env::temp_dir().join(format!("parakeys-key-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let key = VaultKey::generate();
        store_local_key(&dir, &key).unwrap();
        assert!(has_local_key(&dir));
        let loaded = load_local_key(&dir).unwrap();
        assert_eq!(loaded.as_bytes(), key.as_bytes());

        // After clear, recovery can restore access for vault decrypt.
        let mut data = VaultData::new();
        data.set("K", "v");
        let envelope = encrypt(&data, &key).unwrap();
        clear_local_key(&dir).unwrap();
        assert!(!has_local_key(&dir));

        let recovered = decode_recovery_code(&encode_recovery_code(&key)).unwrap();
        store_local_key(&dir, &recovered).unwrap();
        let again = decrypt(&envelope, &load_local_key(&dir).unwrap()).unwrap();
        assert_eq!(again.get("K"), Some("v"));

        let _ = fs::remove_dir_all(&dir);
    }
}
