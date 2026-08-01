//! ParaKeys library: vault, wallet, CLI command handlers (shared with GUI).

pub mod agent;
pub mod cli;
pub mod commands;
pub mod config;
pub mod envfile;
pub mod error;
pub mod keywallet;
pub mod status;
pub mod vault;

pub use error::ParaKeysError;
pub use keywallet::{
    decode_recovery_code, detect_backend, encode_recovery_code, has_unlock_key, load_unlock_key,
    project_root, store_unlock_key, WalletBackend,
};
pub use vault::{default_vault_path, load_vault, save_vault, VaultData, VaultKey};
