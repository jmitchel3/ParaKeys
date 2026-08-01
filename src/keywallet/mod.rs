//! Human unlock key storage: macOS Keychain (primary) with file-wallet fallback.
//!
//! The vault key is 32 random bytes. The recovery code is base32 of that key
//! (shown once at init). Day-to-day unlock uses Keychain + user presence (Touch ID /
//! passcode) on macOS when available; otherwise `.parakeys/local.key`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use data_encoding::BASE32_NOPAD;

use crate::error::ParaKeysError;
use crate::vault::VaultKey;

#[cfg(target_os = "macos")]
mod keychain;

/// Relative path of the local key file (must stay out of git).
pub const LOCAL_KEY_REL: &str = ".parakeys/local.key";

/// Recovery codes are Crockford-ish base32 without padding, grouped for reading.
const RECOVERY_GROUP: usize = 4;

/// Which backend holds the unlock key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletBackend {
    /// macOS Keychain (may require Touch ID / user presence on load).
    Keychain,
    /// File at `.parakeys/local.key` (mode 0600).
    File,
}

impl WalletBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Keychain => "keychain",
            Self::File => "file",
        }
    }
}

pub fn local_key_path(project_root: &Path) -> PathBuf {
    project_root.join(LOCAL_KEY_REL)
}

/// Encode a vault key as a recovery code string (`xxxx-xxxx-...`).
pub fn encode_recovery_code(key: &VaultKey) -> String {
    let encoded = BASE32_NOPAD.encode(key.as_bytes());
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

/// Force file wallet when set (tests / explicit override).
pub fn force_file_wallet() -> bool {
    matches!(
        std::env::var("PARAKEYS_FORCE_FILE_WALLET").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

/// Prefer Keychain on macOS unless forced to file.
pub fn prefer_keychain() -> bool {
    #[cfg(target_os = "macos")]
    {
        !force_file_wallet()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Write the vault key to the local key file with mode 0600.
pub fn store_local_key(project_root: &Path, key: &VaultKey) -> Result<PathBuf, ParaKeysError> {
    let path = local_key_path(project_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(ParaKeysError::Io)?;
    }

    let mut opts = fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }

    let mut file = opts.open(&path).map_err(ParaKeysError::Io)?;
    file.write_all(key.as_bytes()).map_err(ParaKeysError::Io)?;
    file.sync_all().map_err(ParaKeysError::Io)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path).map_err(ParaKeysError::Io)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms).map_err(ParaKeysError::Io)?;
    }

    Ok(path)
}

/// Load the vault key from the local key file only.
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

/// Remove the local key file.
pub fn clear_local_key(project_root: &Path) -> Result<(), ParaKeysError> {
    let path = local_key_path(project_root);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(ParaKeysError::Io(e)),
    }
}

/// Store unlock key: Keychain first on macOS (when preferred), else file.
/// Returns which backend was used.
pub fn store_unlock_key(project_root: &Path, key: &VaultKey) -> Result<WalletBackend, ParaKeysError> {
    if prefer_keychain() {
        #[cfg(target_os = "macos")]
        {
            match keychain::store(project_root, key) {
                Ok(()) => {
                    // Prefer Keychain as primary: remove file copy if present so
                    // day-to-day unlock goes through Keychain user presence.
                    let _ = clear_local_key(project_root);
                    return Ok(WalletBackend::Keychain);
                }
                Err(e) => {
                    // Fall through to file.
                    let _ = e;
                }
            }
        }
    }
    store_local_key(project_root, key)?;
    Ok(WalletBackend::File)
}

/// Load unlock key: try Keychain first (macOS), then file.
pub fn load_unlock_key(project_root: &Path) -> Result<VaultKey, ParaKeysError> {
    if prefer_keychain() {
        #[cfg(target_os = "macos")]
        {
            if let Ok(key) = keychain::load(project_root) {
                return Ok(key);
            }
        }
    }
    load_local_key(project_root)
}

/// True if either Keychain or file wallet has a key for this project.
pub fn has_unlock_key(project_root: &Path) -> bool {
    if prefer_keychain() {
        #[cfg(target_os = "macos")]
        {
            if keychain::exists(project_root) {
                return true;
            }
        }
    }
    has_local_key(project_root)
}

/// Which backend currently has the key (Keychain preferred if both).
pub fn detect_backend(project_root: &Path) -> Option<WalletBackend> {
    if prefer_keychain() {
        #[cfg(target_os = "macos")]
        {
            if keychain::exists(project_root) {
                return Some(WalletBackend::Keychain);
            }
        }
    }
    if has_local_key(project_root) {
        return Some(WalletBackend::File);
    }
    None
}

/// Clear unlock material from Keychain (if any) and file.
pub fn clear_unlock_key(project_root: &Path) -> Result<(), ParaKeysError> {
    #[cfg(target_os = "macos")]
    {
        let _ = keychain::delete(project_root);
    }
    clear_local_key(project_root)
}

/// Resolve project root from an optional `--path` (default: cwd).
pub fn project_root(path: Option<PathBuf>) -> Result<PathBuf, ParaKeysError> {
    match path {
        Some(p) => Ok(p),
        None => std::env::current_dir().map_err(ParaKeysError::Io),
    }
}

/// Account string used for Keychain items (canonical path when possible).
pub fn keychain_account(project_root: &Path) -> String {
    std::fs::canonicalize(project_root)
        .unwrap_or_else(|_| project_root.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::{decrypt, encrypt, VaultData};

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
    fn unlock_key_file_fallback_store_load() {
        let dir = std::env::temp_dir().join(format!(
            "parakeys-key-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let key = VaultKey::generate();
        // Exercise file backend API directly (no env races with parallel Keychain tests).
        store_local_key(&dir, &key).unwrap();
        assert!(has_local_key(&dir));
        assert!(has_unlock_key(&dir));
        let loaded = load_unlock_key(&dir).unwrap();
        assert_eq!(loaded.as_bytes(), key.as_bytes());

        let mut data = VaultData::new();
        data.set("K", "v");
        let envelope = encrypt(&data, &key).unwrap();
        clear_local_key(&dir).unwrap();
        // After file cleared, unlock may still succeed via Keychain if a prior
        // test left an item; clear all unlock material for this path.
        let _ = clear_unlock_key(&dir);
        assert!(!has_local_key(&dir));

        let recovered = decode_recovery_code(&encode_recovery_code(&key)).unwrap();
        store_local_key(&dir, &recovered).unwrap();
        let again = decrypt(&envelope, &load_unlock_key(&dir).unwrap()).unwrap();
        assert_eq!(again.get("K"), Some("v"));

        let _ = clear_unlock_key(&dir);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_wallet_api_round_trip_direct() {
        let dir = std::env::temp_dir().join(format!("parakeys-file-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let key = VaultKey::generate();
        store_local_key(&dir, &key).unwrap();
        assert!(has_local_key(&dir));
        assert_eq!(load_local_key(&dir).unwrap().as_bytes(), key.as_bytes());
        clear_local_key(&dir).unwrap();
        assert!(!has_local_key(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn keychain_store_load_round_trip() {
        // Use no user-presence for non-interactive test; still real Keychain API.
        std::env::set_var("PARAKEYS_KEYCHAIN_NO_PRESENCE", "1");
        std::env::remove_var("PARAKEYS_FORCE_FILE_WALLET");

        let dir = std::env::temp_dir().join(format!(
            "parakeys-kc-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let key = VaultKey::generate();
        keychain::store(&dir, &key).expect("keychain store");
        assert!(keychain::exists(&dir));
        let loaded = keychain::load(&dir).expect("keychain load");
        assert_eq!(loaded.as_bytes(), key.as_bytes());

        // Decrypt vault using key from Keychain.
        let mut data = VaultData::new();
        data.set("KC", "from-keychain");
        let env = encrypt(&data, &key).unwrap();
        let plain = decrypt(&env, &loaded).unwrap();
        assert_eq!(plain.get("KC"), Some("from-keychain"));

        keychain::delete(&dir).expect("keychain delete");
        assert!(!keychain::exists(&dir));

        // File fallback after Keychain cleared.
        store_local_key(&dir, &key).unwrap();
        assert_eq!(load_unlock_key(&dir).unwrap().as_bytes(), key.as_bytes());

        clear_unlock_key(&dir).unwrap();
        std::env::remove_var("PARAKEYS_KEYCHAIN_NO_PRESENCE");
        let _ = fs::remove_dir_all(&dir);
    }
}
