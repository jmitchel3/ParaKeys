//! Classify env key status for list/doctor (non-secret metadata).

use crate::envfile::{is_placeholder, PLACEHOLDER_NOT_SET, PLACEHOLDER_SET};
use crate::vault::VaultData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStatus {
    /// Present in vault with a value.
    SetInVault,
    /// Declared in `.env` as not set / missing from vault.
    NotSet,
    /// Declared only in `.env` with a non-placeholder value still on disk.
    PlaintextOnDisk,
}

/// Classify a key by vault membership and optional .env assignment value.
pub fn classify_key(vault: &VaultData, key: &str, env_value: Option<&str>) -> KeyStatus {
    let in_vault = vault.get(key).is_some();
    match env_value {
        None => {
            if in_vault {
                KeyStatus::SetInVault
            } else {
                KeyStatus::NotSet
            }
        }
        Some(v) => {
            let v = v.trim();
            if is_placeholder(v) {
                if v == PLACEHOLDER_SET && in_vault {
                    KeyStatus::SetInVault
                } else if v == PLACEHOLDER_NOT_SET || !in_vault {
                    KeyStatus::NotSet
                } else {
                    KeyStatus::NotSet
                }
            } else if !v.is_empty() {
                KeyStatus::PlaintextOnDisk
            } else if in_vault {
                KeyStatus::SetInVault
            } else {
                KeyStatus::NotSet
            }
        }
    }
}

pub fn status_label(s: KeyStatus) -> &'static str {
    match s {
        KeyStatus::SetInVault => "<set in parakeys>",
        KeyStatus::NotSet => "<not set in parakeys>",
        KeyStatus::PlaintextOnDisk => "<plaintext on disk>",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::VaultData;

    #[test]
    fn classifies_set_and_not_set() {
        let mut v = VaultData::new();
        v.set("A", "1");
        assert_eq!(
            classify_key(&v, "A", Some(PLACEHOLDER_SET)),
            KeyStatus::SetInVault
        );
        assert_eq!(
            classify_key(&v, "B", Some(PLACEHOLDER_NOT_SET)),
            KeyStatus::NotSet
        );
        assert_eq!(
            classify_key(&v, "C", Some("sk-live-plaintext-token-zzzz")),
            KeyStatus::PlaintextOnDisk
        );
        assert_eq!(classify_key(&v, "A", None), KeyStatus::SetInVault);
    }
}
