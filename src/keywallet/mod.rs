//! Human unlock key storage: macOS Keychain (primary) with file-wallet fallback.
//!
//! The vault key is 32 random bytes. The recovery code is base32 of that key
//! (shown once at init). Day-to-day unlock uses Keychain on macOS when possible
//! (Touch ID / user presence when entitlements allow); otherwise `.parakeys/local.key`.

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

const RECOVERY_GROUP: usize = 4;

/// Which backend holds the unlock key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletBackend {
    /// macOS Keychain with user-presence ACL (Touch ID / passcode).
    KeychainUserPresence,
    /// macOS Keychain without presence ACL (unsigned CLI friendly).
    Keychain,
    /// File at `.parakeys/local.key` (mode 0600).
    File,
}

impl WalletBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::KeychainUserPresence => "keychain+user-presence",
            Self::Keychain => "keychain",
            Self::File => "file",
        }
    }
}

/// Result of storing an unlock key, including any degraded-mode notes.
#[derive(Debug, Clone)]
pub struct StoreOutcome {
    pub backend: WalletBackend,
    /// Human-readable notes (e.g. presence failed, fell back to plain Keychain).
    pub notes: Vec<String>,
}

pub fn local_key_path(project_root: &Path) -> PathBuf {
    project_root.join(LOCAL_KEY_REL)
}

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

pub fn force_file_wallet() -> bool {
    matches!(
        std::env::var("PARAKEYS_FORCE_FILE_WALLET").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

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

pub fn has_local_key(project_root: &Path) -> bool {
    local_key_path(project_root).is_file()
}

pub fn clear_local_key(project_root: &Path) -> Result<(), ParaKeysError> {
    let path = local_key_path(project_root);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(ParaKeysError::Io(e)),
    }
}

/// Store unlock key: Keychain (presence → plain) on macOS, else file.
pub fn store_unlock_key(project_root: &Path, key: &VaultKey) -> Result<StoreOutcome, ParaKeysError> {
    let mut notes = Vec::new();

    if prefer_keychain() {
        #[cfg(target_os = "macos")]
        {
            match keychain::store(project_root, key) {
                Ok(used_presence) => {
                    let _ = clear_local_key(project_root);
                    let backend = if used_presence {
                        WalletBackend::KeychainUserPresence
                    } else {
                        if !keychain::skip_user_presence() {
                            notes.push(
                                "Keychain user-presence (Touch ID) ACL unavailable for this binary \
                                 (often errSecMissingEntitlement on unsigned CLI); stored in Keychain without presence."
                                    .into(),
                            );
                        }
                        WalletBackend::Keychain
                    };
                    return Ok(StoreOutcome { backend, notes });
                }
                Err(e) => {
                    notes.push(format!("Keychain store failed: {e}"));
                    notes.push("Falling back to file wallet (.parakeys/local.key).".into());
                }
            }
        }
    } else if force_file_wallet() {
        notes.push("PARAKEYS_FORCE_FILE_WALLET set; using file wallet.".into());
    }

    store_local_key(project_root, key)?;
    Ok(StoreOutcome {
        backend: WalletBackend::File,
        notes,
    })
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

pub fn detect_backend(project_root: &Path) -> Option<WalletBackend> {
    if prefer_keychain() {
        #[cfg(target_os = "macos")]
        {
            if keychain::exists(project_root) {
                // Cannot distinguish presence ACL after the fact without item attrs;
                // report generic Keychain when item exists.
                return Some(WalletBackend::Keychain);
            }
        }
    }
    if has_local_key(project_root) {
        return Some(WalletBackend::File);
    }
    None
}

pub fn clear_unlock_key(project_root: &Path) -> Result<(), ParaKeysError> {
    #[cfg(target_os = "macos")]
    {
        let _ = keychain::delete(project_root);
    }
    clear_local_key(project_root)
}

pub fn project_root(path: Option<PathBuf>) -> Result<PathBuf, ParaKeysError> {
    match path {
        Some(p) => Ok(p),
        None => std::env::current_dir().map_err(ParaKeysError::Io),
    }
}

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
        assert!(code.contains('-'));
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
        store_local_key(&dir, &key).unwrap();
        assert!(has_local_key(&dir));
        assert!(has_unlock_key(&dir));
        assert_eq!(load_unlock_key(&dir).unwrap().as_bytes(), key.as_bytes());

        let mut data = VaultData::new();
        data.set("K", "v");
        let envelope = encrypt(&data, &key).unwrap();
        let _ = clear_unlock_key(&dir);

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
    fn store_unlock_key_uses_keychain_not_file_when_possible() {
        // Real store_unlock_key path: must land in Keychain (plain or presence), not only file.
        std::env::remove_var("PARAKEYS_FORCE_FILE_WALLET");
        std::env::remove_var("PARAKEYS_KEYCHAIN_NO_PRESENCE");

        let dir = std::env::temp_dir().join(format!(
            "parakeys-store-tier-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let key = VaultKey::generate();
        let outcome = store_unlock_key(&dir, &key).expect("store_unlock_key");
        assert!(
            matches!(
                outcome.backend,
                WalletBackend::Keychain | WalletBackend::KeychainUserPresence
            ),
            "expected Keychain backend, got {:?} notes={:?}",
            outcome.backend,
            outcome.notes
        );
        // File should not be required primary.
        assert!(
            keychain::exists(&dir),
            "Keychain item must exist after store_unlock_key"
        );
        // load goes through shipped API
        assert_eq!(load_unlock_key(&dir).unwrap().as_bytes(), key.as_bytes());

        let mut data = VaultData::new();
        data.set("X", "from-kc");
        let env = encrypt(&data, &key).unwrap();
        assert_eq!(
            decrypt(&env, &load_unlock_key(&dir).unwrap())
                .unwrap()
                .get("X"),
            Some("from-kc")
        );

        // After clearing Keychain, file fallback still works when we store file.
        keychain::delete(&dir).unwrap();
        assert!(!keychain::exists(&dir));
        store_local_key(&dir, &key).unwrap();
        assert_eq!(load_unlock_key(&dir).unwrap().as_bytes(), key.as_bytes());

        let _ = clear_unlock_key(&dir);
        let _ = fs::remove_dir_all(&dir);
    }
}
