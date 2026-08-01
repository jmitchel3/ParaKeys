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

fn no_presence() -> bool {
    matches!(
        std::env::var("PARAKEYS_KEYCHAIN_NO_PRESENCE").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    ) || cfg!(test)
}

pub fn store(project_root: &Path, key: &VaultKey) -> Result<(), ParaKeysError> {
    let account = keychain_account(project_root);
    // Replace any existing item first.
    let _ = delete_generic_password(SERVICE, &account);

    if no_presence() {
        set_generic_password(SERVICE, &account, key.as_bytes()).map_err(map_sf_err)?;
    } else {
        let mut opts = PasswordOptions::new_generic_password(SERVICE, &account);
        opts.set_label("ParaKeys vault key");
        opts.set_description("Unlock key for project dotenv vault");
        // Touch ID / device passcode when OS allows.
        opts.set_access_control_options(AccessControlOptions::USER_PRESENCE);
        set_generic_password_options(key.as_bytes(), opts).map_err(map_sf_err)?;
    }
    Ok(())
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
            // Not found is fine.
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
