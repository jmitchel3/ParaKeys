//! Agent recipient keys and grant helpers (phase 2 start).
//!
//! v0 uses a 32-byte recipient key stored under `.parakeys-agent/`.
//! The `.pub` file is the same key material base64-encoded for grant encryption
//! (pre-shared recipient key, not classic asymmetric crypto).

use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::error::ParaKeysError;
use crate::vault::{VaultData, VaultKey, KEY_LEN, NONCE_LEN};

pub const AGENT_DIR: &str = ".parakeys-agent";
pub const AGENT_KEY_FILE: &str = "agent.key";
pub const AGENT_PUB_FILE: &str = "agent.pub";
pub const AGENT_GRANT_FILE: &str = "grant.enc";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantEnvelope {
    pub format: String,
    pub version: u32,
    pub algorithm: String,
    pub allowlist: Vec<String>,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
}

pub fn agent_dir(project_root: &Path) -> PathBuf {
    project_root.join(AGENT_DIR)
}

pub fn agent_key_path(project_root: &Path) -> PathBuf {
    agent_dir(project_root).join(AGENT_KEY_FILE)
}

pub fn agent_pub_path(project_root: &Path) -> PathBuf {
    agent_dir(project_root).join(AGENT_PUB_FILE)
}

pub fn agent_grant_path(project_root: &Path) -> PathBuf {
    agent_dir(project_root).join(AGENT_GRANT_FILE)
}

pub fn keygen(project_root: &Path) -> Result<(PathBuf, PathBuf, String), ParaKeysError> {
    let dir = agent_dir(project_root);
    fs::create_dir_all(&dir).map_err(ParaKeysError::Io)?;

    let key = VaultKey::generate();
    let key_path = agent_key_path(project_root);
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    opts.mode(0o600);
    let mut f = opts.open(&key_path).map_err(ParaKeysError::Io)?;
    f.write_all(key.as_bytes()).map_err(ParaKeysError::Io)?;
    f.sync_all().map_err(ParaKeysError::Io)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&key_path).map_err(ParaKeysError::Io)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&key_path, perms).map_err(ParaKeysError::Io)?;
    }

    let pub_b64 = B64.encode(key.as_bytes());
    let pub_path = agent_pub_path(project_root);
    fs::write(&pub_path, format!("{pub_b64}\n")).map_err(ParaKeysError::Io)?;

    Ok((key_path, pub_path, pub_b64))
}

pub fn load_agent_key(project_root: &Path) -> Result<VaultKey, ParaKeysError> {
    let path = agent_key_path(project_root);
    let bytes = fs::read(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ParaKeysError::VaultNotFound(format!(
                "agent key missing at {} (run `parakeys agent keygen`)",
                path.display()
            ))
        } else {
            ParaKeysError::Io(e)
        }
    })?;
    VaultKey::try_from_slice(&bytes)
}

pub fn load_recipient_key_from_pub_file(path: &Path) -> Result<VaultKey, ParaKeysError> {
    let text = fs::read_to_string(path).map_err(ParaKeysError::Io)?;
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = B64
        .decode(compact.as_bytes())
        .map_err(|e| ParaKeysError::InvalidVault(format!("agent.pub base64: {e}")))?;
    VaultKey::try_from_slice(&bytes)
}

pub fn create_grant(
    vault: &VaultData,
    allowlist: &[String],
    recipient: &VaultKey,
) -> Result<GrantEnvelope, ParaKeysError> {
    let mut subset = VaultData::new();
    for name in allowlist {
        match vault.get(name) {
            Some(v) => subset.set(name.clone(), v.to_string()),
            None => {
                return Err(ParaKeysError::InvalidVault(format!(
                    "allowlist key `{name}` not in vault"
                )));
            }
        }
    }
    let plaintext = serde_json::to_vec(&subset)
        .map_err(|e| ParaKeysError::InvalidVault(format!("serialize grant: {e}")))?;

    let key_arr = Key::from(*recipient.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key_arr);
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_ref())
        .map_err(|_| ParaKeysError::InvalidVault("grant encryption failed".into()))?;

    Ok(GrantEnvelope {
        format: "parakeys-grant".into(),
        version: 0,
        algorithm: "chacha20poly1305".into(),
        allowlist: allowlist.to_vec(),
        nonce_b64: B64.encode(nonce_bytes),
        ciphertext_b64: B64.encode(ciphertext),
    })
}

pub fn decrypt_grant(envelope: &GrantEnvelope, agent_key: &VaultKey) -> Result<VaultData, ParaKeysError> {
    if envelope.format != "parakeys-grant" {
        return Err(ParaKeysError::InvalidVault(format!(
            "unknown grant format {:?}",
            envelope.format
        )));
    }
    let nonce_raw = B64
        .decode(envelope.nonce_b64.as_bytes())
        .map_err(|e| ParaKeysError::InvalidVault(format!("grant nonce: {e}")))?;
    if nonce_raw.len() != NONCE_LEN {
        return Err(ParaKeysError::InvalidVault("bad grant nonce length".into()));
    }
    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(&nonce_raw);
    let ciphertext = B64
        .decode(envelope.ciphertext_b64.as_bytes())
        .map_err(|e| ParaKeysError::InvalidVault(format!("grant ciphertext: {e}")))?;

    let key_arr = Key::from(*agent_key.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key_arr);
    let nonce = Nonce::from(nonce_bytes);
    let plaintext = cipher
        .decrypt(&nonce, ciphertext.as_ref())
        .map_err(|_| ParaKeysError::InvalidVault("grant decryption failed".into()))?;
    serde_json::from_slice(&plaintext)
        .map_err(|e| ParaKeysError::InvalidVault(format!("grant JSON: {e}")))
}

pub fn save_grant(path: &Path, envelope: &GrantEnvelope) -> Result<(), ParaKeysError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(ParaKeysError::Io)?;
    }
    let text = serde_json::to_string_pretty(envelope)
        .map_err(|e| ParaKeysError::InvalidVault(e.to_string()))?;
    fs::write(path, format!("{text}\n")).map_err(ParaKeysError::Io)?;
    Ok(())
}

pub fn load_grant(path: &Path) -> Result<GrantEnvelope, ParaKeysError> {
    let text = fs::read_to_string(path).map_err(ParaKeysError::Io)?;
    serde_json::from_str(&text)
        .map_err(|e| ParaKeysError::InvalidVault(format!("parse grant: {e}")))
}

/// Silence unused KEY_LEN if only used in docs.
#[allow(dead_code)]
const _KEY_LEN: usize = KEY_LEN;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keygen_and_grant_round_trip() {
        let dir = std::env::temp_dir().join(format!("pk-agent-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let (key_path, pub_path, _) = keygen(&dir).unwrap();
        assert!(key_path.is_file());
        assert!(pub_path.is_file());
        let agent = load_agent_key(&dir).unwrap();
        let recipient = load_recipient_key_from_pub_file(&pub_path).unwrap();
        assert_eq!(agent.as_bytes(), recipient.as_bytes());

        let mut vault = VaultData::new();
        vault.set("A", "secret-a");
        vault.set("B", "secret-b");
        let grant = create_grant(&vault, &["A".into()], &recipient).unwrap();
        assert!(!serde_json::to_string(&grant).unwrap().contains("secret-a"));
        let data = decrypt_grant(&grant, &agent).unwrap();
        assert_eq!(data.get("A"), Some("secret-a"));
        assert!(data.get("B").is_none());

        let _ = fs::remove_dir_all(&dir);
    }
}
