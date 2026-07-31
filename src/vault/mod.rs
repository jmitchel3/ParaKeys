//! Encrypted project vault, format v0.
//!
//! On-disk envelope is JSON (pretty) with base64 fields so the file is
//! inspectable as opaque ciphertext and safe to commit. Plaintext payload is
//! JSON mapping environment key names to values.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::error::ParaKeysError;

/// Current on-disk format version.
pub const FORMAT_VERSION: u32 = 0;

/// Magic / format id written into the envelope.
pub const FORMAT_ID: &str = "parakeys-vault";

/// Symmetric key length (ChaCha20-Poly1305).
pub const KEY_LEN: usize = 32;

/// Nonce length for ChaCha20-Poly1305.
pub const NONCE_LEN: usize = 12;

/// Relative path of the vault file inside a project.
pub const DEFAULT_VAULT_REL: &str = ".parakeys/vault.enc";

/// Decrypted vault contents for one environment (v0: single map of keys).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultData {
    /// Environment variable name -> secret value.
    #[serde(default)]
    pub keys: BTreeMap<String, String>,
}

impl VaultData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.keys.get(key).map(String::as_str)
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.keys.insert(key.into(), value.into());
    }

    pub fn unset(&mut self, key: &str) -> bool {
        self.keys.remove(key).is_some()
    }
}

/// Versioned on-disk envelope (ciphertext only; no key material).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEnvelope {
    pub format: String,
    pub version: u32,
    pub algorithm: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
}

impl VaultEnvelope {
    pub fn to_pretty_json(&self) -> Result<String, ParaKeysError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| ParaKeysError::InvalidVault(e.to_string()))
    }

    pub fn from_json(s: &str) -> Result<Self, ParaKeysError> {
        let env: Self = serde_json::from_str(s)
            .map_err(|e| ParaKeysError::InvalidVault(format!("parse envelope: {e}")))?;
        if env.format != FORMAT_ID {
            return Err(ParaKeysError::InvalidVault(format!(
                "unknown format id {:?}",
                env.format
            )));
        }
        if env.version != FORMAT_VERSION {
            return Err(ParaKeysError::InvalidVault(format!(
                "unsupported vault version {} (this binary supports {FORMAT_VERSION})",
                env.version
            )));
        }
        if env.algorithm != "chacha20poly1305" {
            return Err(ParaKeysError::InvalidVault(format!(
                "unsupported algorithm {:?}",
                env.algorithm
            )));
        }
        Ok(env)
    }
}

/// 32-byte vault key.
#[derive(Clone)]
pub struct VaultKey([u8; KEY_LEN]);

impl VaultKey {
    pub fn from_bytes(bytes: [u8; KEY_LEN]) -> Self {
        Self(bytes)
    }

    pub fn generate() -> Self {
        let mut bytes = [0u8; KEY_LEN];
        rand::rng().fill_bytes(&mut bytes);
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn try_from_slice(slice: &[u8]) -> Result<Self, ParaKeysError> {
        if slice.len() != KEY_LEN {
            return Err(ParaKeysError::InvalidVault(format!(
                "vault key must be {KEY_LEN} bytes, got {}",
                slice.len()
            )));
        }
        let mut bytes = [0u8; KEY_LEN];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }
}

impl std::fmt::Debug for VaultKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VaultKey([redacted])")
    }
}

/// Encrypt plaintext vault data into a versioned envelope.
pub fn encrypt(data: &VaultData, key: &VaultKey) -> Result<VaultEnvelope, ParaKeysError> {
    let plaintext = serde_json::to_vec(data)
        .map_err(|e| ParaKeysError::InvalidVault(format!("serialize vault data: {e}")))?;

    let key_arr = Key::from(*key.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key_arr);
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_ref())
        .map_err(|_| ParaKeysError::InvalidVault("encryption failed".into()))?;

    Ok(VaultEnvelope {
        format: FORMAT_ID.to_string(),
        version: FORMAT_VERSION,
        algorithm: "chacha20poly1305".to_string(),
        nonce_b64: B64.encode(nonce_bytes),
        ciphertext_b64: B64.encode(ciphertext),
    })
}

/// Decrypt an envelope with the vault key.
pub fn decrypt(envelope: &VaultEnvelope, key: &VaultKey) -> Result<VaultData, ParaKeysError> {
    // Re-validate fields (callers may construct envelopes without from_json).
    if envelope.format != FORMAT_ID {
        return Err(ParaKeysError::InvalidVault(format!(
            "unknown format id {:?}",
            envelope.format
        )));
    }
    if envelope.version != FORMAT_VERSION {
        return Err(ParaKeysError::InvalidVault(format!(
            "unsupported vault version {}",
            envelope.version
        )));
    }

    let nonce_raw = B64
        .decode(envelope.nonce_b64.as_bytes())
        .map_err(|e| ParaKeysError::InvalidVault(format!("nonce base64: {e}")))?;
    if nonce_raw.len() != NONCE_LEN {
        return Err(ParaKeysError::InvalidVault(format!(
            "nonce must be {NONCE_LEN} bytes, got {}",
            nonce_raw.len()
        )));
    }
    let ciphertext = B64
        .decode(envelope.ciphertext_b64.as_bytes())
        .map_err(|e| ParaKeysError::InvalidVault(format!("ciphertext base64: {e}")))?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(&nonce_raw);
    let key_arr = Key::from(*key.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key_arr);
    let nonce = Nonce::from(nonce_bytes);
    let plaintext = cipher
        .decrypt(&nonce, ciphertext.as_ref())
        .map_err(|_| {
            ParaKeysError::InvalidVault(
                "decryption failed (wrong key or corrupt vault)".into(),
            )
        })?;

    serde_json::from_slice(&plaintext)
        .map_err(|e| ParaKeysError::InvalidVault(format!("vault plaintext JSON: {e}")))
}

/// Load envelope JSON from disk.
pub fn load_envelope(path: &Path) -> Result<VaultEnvelope, ParaKeysError> {
    let text = fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ParaKeysError::VaultNotFound(path.display().to_string())
        } else {
            ParaKeysError::Io(e)
        }
    })?;
    VaultEnvelope::from_json(&text)
}

/// Write envelope JSON to disk (creates parent dirs).
pub fn save_envelope(path: &Path, envelope: &VaultEnvelope) -> Result<(), ParaKeysError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(ParaKeysError::Io)?;
    }
    let text = envelope.to_pretty_json()?;
    fs::write(path, format!("{text}\n")).map_err(ParaKeysError::Io)?;
    Ok(())
}

/// Default vault path under a project root.
pub fn default_vault_path(project_root: &Path) -> PathBuf {
    project_root.join(DEFAULT_VAULT_REL)
}

/// Encrypt and write vault data to the default path.
pub fn save_vault(project_root: &Path, data: &VaultData, key: &VaultKey) -> Result<PathBuf, ParaKeysError> {
    let path = default_vault_path(project_root);
    let envelope = encrypt(data, key)?;
    save_envelope(&path, &envelope)?;
    Ok(path)
}

/// Load and decrypt vault from the default path.
pub fn load_vault(project_root: &Path, key: &VaultKey) -> Result<VaultData, ParaKeysError> {
    let path = default_vault_path(project_root);
    let envelope = load_envelope(&path)?;
    decrypt(&envelope, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_round_trip() {
        let key = VaultKey::generate();
        let mut data = VaultData::new();
        data.set("DATABASE_URL", "postgres://secret");
        data.set("OPENAI_API_KEY", "sk-test");

        let envelope = encrypt(&data, &key).expect("encrypt");
        assert_eq!(envelope.format, FORMAT_ID);
        assert_eq!(envelope.version, FORMAT_VERSION);
        assert!(!envelope.ciphertext_b64.is_empty());

        let decoded = decrypt(&envelope, &key).expect("decrypt");
        assert_eq!(decoded, data);
    }

    #[test]
    fn wrong_key_fails() {
        let key_a = VaultKey::generate();
        let key_b = VaultKey::generate();
        let mut data = VaultData::new();
        data.set("X", "1");

        let envelope = encrypt(&data, &key_a).expect("encrypt");
        let err = decrypt(&envelope, &key_b).expect_err("wrong key");
        match err {
            ParaKeysError::InvalidVault(msg) => {
                assert!(msg.contains("decryption failed"), "msg was: {msg}");
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn envelope_json_round_trip() {
        let key = VaultKey::generate();
        let mut data = VaultData::new();
        data.set("A", "b");
        let envelope = encrypt(&data, &key).unwrap();
        let json = envelope.to_pretty_json().unwrap();
        let parsed = VaultEnvelope::from_json(&json).unwrap();
        let decoded = decrypt(&parsed, &key).unwrap();
        assert_eq!(decoded.get("A"), Some("b"));
    }

    #[test]
    fn reject_unknown_format_version() {
        let key = VaultKey::generate();
        let envelope = encrypt(&VaultData::new(), &key).unwrap();
        let mut bad = envelope.clone();
        bad.version = 99;
        let json = serde_json::to_string(&bad).unwrap();
        let err = VaultEnvelope::from_json(&json).unwrap_err();
        assert!(matches!(err, ParaKeysError::InvalidVault(_)));
    }

    #[test]
    fn save_and_load_file() {
        let dir = std::env::temp_dir().join(format!("parakeys-vault-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let key = VaultKey::generate();
        let mut data = VaultData::new();
        data.set("TOKEN", "abc");
        save_vault(&dir, &data, &key).unwrap();

        let path = default_vault_path(&dir);
        assert!(path.is_file());
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("parakeys-vault"));
        assert!(!text.contains("abc"), "plaintext must not appear in file");

        let loaded = load_vault(&dir, &key).unwrap();
        assert_eq!(loaded.get("TOKEN"), Some("abc"));

        let _ = fs::remove_dir_all(&dir);
    }
}
