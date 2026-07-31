//! Non-secret project metadata (safe to commit).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::ParaKeysError;

pub const CONFIG_REL: &str = ".parakeys/config.toml";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectConfig {
    pub format_version: u32,
    pub env_name: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            format_version: 0,
            env_name: "default".to_string(),
        }
    }
}

pub fn config_path(project_root: &Path) -> PathBuf {
    project_root.join(CONFIG_REL)
}

pub fn save_config(project_root: &Path, cfg: &ProjectConfig) -> Result<PathBuf, ParaKeysError> {
    let path = config_path(project_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(ParaKeysError::Io)?;
    }
    let text = toml::to_string_pretty(cfg)
        .map_err(|e| ParaKeysError::InvalidVault(format!("serialize config: {e}")))?;
    let header = "# ParaKeys project config (non-secret; safe to commit)\n";
    fs::write(&path, format!("{header}{text}")).map_err(ParaKeysError::Io)?;
    Ok(path)
}

pub fn load_config(project_root: &Path) -> Result<ProjectConfig, ParaKeysError> {
    let path = config_path(project_root);
    let text = fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ParaKeysError::VaultNotFound(path.display().to_string())
        } else {
            ParaKeysError::Io(e)
        }
    })?;
    toml::from_str(&text)
        .map_err(|e| ParaKeysError::InvalidVault(format!("parse config.toml: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trip() {
        let dir = std::env::temp_dir().join(format!("pk-cfg-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let cfg = ProjectConfig {
            format_version: 0,
            env_name: "acme-local".into(),
        };
        save_config(&dir, &cfg).unwrap();
        let loaded = load_config(&dir).unwrap();
        assert_eq!(loaded, cfg);
        let raw = fs::read_to_string(config_path(&dir)).unwrap();
        assert!(raw.contains("safe to commit"));
        assert!(!raw.contains("sk-") && !raw.contains("password="));
        let _ = fs::remove_dir_all(&dir);
    }
}
