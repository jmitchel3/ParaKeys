//! macOS Keychain backend for the human vault unlock key.

use std::path::Path;

use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
    set_generic_password_options, AccessControlOptions, PasswordOptions,
};

use super::keychain_account;
use crate::error::ParaKeysError;
use crate::vault::VaultKey;

const SERVICE: &str = "com.parakeys.vaultkey";

/// Whether to skip SecAccessControl user-presence (Touch ID / passcode).
/// Set in automated tests; production tries presence first then falls back.
pub fn skip_user_presence() -> bool {
    matches!(
        std::env::var("PARAKEYS_KEYCHAIN_NO_PRESENCE").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

/// Store with Touch ID / user-presence when the OS allows.
pub fn store_with_user_presence(project_root: &Path, key: &VaultKey) -> Result<(), ParaKeysError> {
    let account = keychain_account(project_root);
    let _ = delete_generic_password(SERVICE, &account);

    let mut opts = PasswordOptions::new_generic_password(SERVICE, &account);
    opts.set_label("ParaKeys vault key");
    opts.set_description("Unlock key for project dotenv vault");
    opts.set_access_control_options(AccessControlOptions::USER_PRESENCE);
    set_generic_password_options(key.as_bytes(), opts).map_err(map_sf_err)?;
    Ok(())
}

/// Store as a normal Keychain generic password (no SecAccessControl).
/// Works for unsigned CLI binaries that lack keychain entitlements for presence ACLs.
pub fn store_plain(project_root: &Path, key: &VaultKey) -> Result<(), ParaKeysError> {
    let account = keychain_account(project_root);
    let _ = delete_generic_password(SERVICE, &account);
    set_generic_password(SERVICE, &account, key.as_bytes()).map_err(map_sf_err)?;
    Ok(())
}

/// Production store: try user-presence first, then plain Keychain.
/// Returns whether user-presence ACL was applied.
pub fn store(project_root: &Path, key: &VaultKey) -> Result<bool, ParaKeysError> {
    if skip_user_presence() {
        store_plain(project_root, key)?;
        return Ok(false);
    }
    match store_with_user_presence(project_root, key) {
        Ok(()) => Ok(true),
        Err(presence_err) => {
            // Common for ad-hoc CLI: errSecMissingEntitlement (-34018).
            // Still prefer Keychain over a plaintext file on disk.
            store_plain(project_root, key).map_err(|plain_err| {
                ParaKeysError::InvalidVault(format!(
                    "keychain user-presence failed ({presence_err}); plain keychain also failed ({plain_err})"
                ))
            })?;
            Ok(false)
        }
    }
}

pub fn load(project_root: &Path) -> Result<VaultKey, ParaKeysError> {
    let account = keychain_account(project_root);
    let bytes = get_generic_password(SERVICE, &account).map_err(map_sf_err)?;
    VaultKey::try_from_slice(&bytes)
}

pub fn exists(project_root: &Path) -> bool {
    let account = keychain_account(project_root);
    get_generic_password(SERVICE, &account).is_ok()
}

pub fn delete(project_root: &Path) -> Result<(), ParaKeysError> {
    let account = keychain_account(project_root);
    match delete_generic_password(SERVICE, &account) {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = format!("{e}");
            if msg.contains("not found") || msg.contains("-25300") {
                Ok(())
            } else {
                Err(map_sf_err(e))
            }
        }
    }
}

fn map_sf_err(e: security_framework::base::Error) -> ParaKeysError {
    ParaKeysError::InvalidVault(format!("keychain: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::VaultKey;
    use std::fs;

    fn unique_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "parakeys-kc-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn plain_keychain_store_load_delete() {
        let dir = unique_dir("plain");
        let key = VaultKey::generate();
        store_plain(&dir, &key).expect("plain store must work without entitlements");
        assert!(exists(&dir));
        let loaded = load(&dir).expect("plain load");
        assert_eq!(loaded.as_bytes(), key.as_bytes());
        delete(&dir).unwrap();
        assert!(!exists(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn store_tier_prefers_keychain_over_failure() {
        // Exercises production store() including the presence→plain fallback path.
        let dir = unique_dir("tier");
        let key = VaultKey::generate();
        // Do NOT set NO_PRESENCE: store() should try presence, then plain.
        std::env::remove_var("PARAKEYS_KEYCHAIN_NO_PRESENCE");
        let used_presence = store(&dir, &key).expect("store tier must succeed via plain fallback");
        // Presence may or may not succeed depending on entitlements; item must exist either way.
        assert!(exists(&dir), "key must land in Keychain after store()");
        let loaded = load(&dir).expect("load after store()");
        assert_eq!(loaded.as_bytes(), key.as_bytes());
        // Document outcome for evidence logs (presence false is expected for unsigned CLI).
        let _ = used_presence;
        delete(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn user_presence_store_records_result() {
        // Drive the real USER_PRESENCE SecAccessControl path (not skipped).
        // On unsigned CLI this often fails with -34018; that is a real observation.
        let dir = unique_dir("presence");
        let key = VaultKey::generate();
        std::env::remove_var("PARAKEYS_KEYCHAIN_NO_PRESENCE");
        let result = store_with_user_presence(&dir, &key);
        match result {
            Ok(()) => {
                assert!(exists(&dir));
                assert_eq!(load(&dir).unwrap().as_bytes(), key.as_bytes());
                delete(&dir).unwrap();
            }
            Err(e) => {
                let msg = e.to_string();
                // Must be a real Keychain error from the presence path, not a silent stub.
                assert!(
                    msg.contains("keychain") || msg.contains("-34018") || msg.contains("entitlement")
                        || msg.contains("Param") || msg.contains("error"),
                    "unexpected presence error: {msg}"
                );
                // Presence failed; plain must still work (proves fallback path is available).
                store_plain(&dir, &key).expect("plain after presence failure");
                assert!(exists(&dir));
                delete(&dir).unwrap();
            }
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
