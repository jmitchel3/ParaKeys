use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::envfile::{
    load_env_file, save_env_file, EnvFile, EnvLine, PLACEHOLDER_SET,
};
use crate::keywallet::{load_local_key, project_root};
use crate::vault::{default_vault_path, load_vault, save_vault};

pub fn run(key_arg: String, value_flag: Option<String>, path: Option<PathBuf>) -> Result<()> {
    let (key, value) = parse_key_value(&key_arg, value_flag)?;
    if key.is_empty() {
        bail!("key name must not be empty");
    }

    let root = project_root(path).context("resolve project path")?;
    let vault_path = default_vault_path(&root);
    if !vault_path.is_file() {
        bail!(
            "no vault at {} — run `parakeys init` first",
            vault_path.display()
        );
    }

    let vault_key = load_local_key(&root).context("load local key")?;
    let mut vault = load_vault(&root, &vault_key).context("decrypt vault")?;
    vault.set(key.clone(), value);
    save_vault(&root, &vault, &vault_key).context("write vault")?;

    // Keep .env placeholder in sync when the file exists (or create minimal entry).
    let env_path = root.join(".env");
    if env_path.is_file() {
        let mut env = load_env_file(&env_path).context("read .env")?;
        upsert_placeholder(&mut env, &key, PLACEHOLDER_SET);
        save_env_file(&env_path, &env).context("rewrite .env")?;
    }

    println!("Set `{key}` in vault (value not printed).");
    Ok(())
}

fn parse_key_value(key_arg: &str, value_flag: Option<String>) -> Result<(String, String)> {
    if let Some(v) = value_flag {
        if let Some((k, rest)) = key_arg.split_once('=') {
            if !rest.is_empty() {
                bail!("use either KEY=value or KEY --value, not both");
            }
            return Ok((k.to_string(), v));
        }
        return Ok((key_arg.to_string(), v));
    }
    if let Some((k, v)) = key_arg.split_once('=') {
        if k.is_empty() {
            bail!("invalid KEY=value form");
        }
        return Ok((k.to_string(), v.to_string()));
    }
    bail!("provide a value as KEY=value or KEY --value <value>");
}

fn upsert_placeholder(env: &mut EnvFile, key: &str, placeholder: &str) {
    for line in &mut env.lines {
        if let EnvLine::Assignment {
            key: k, value, ..
        } = line
        {
            if k == key {
                *value = placeholder.to_string();
                return;
            }
        }
    }
    env.lines.push(EnvLine::Assignment {
        key: key.to_string(),
        value: placeholder.to_string(),
        export: false,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_key_eq_value() {
        let (k, v) = parse_key_value("FOO=bar", None).unwrap();
        assert_eq!(k, "FOO");
        assert_eq!(v, "bar");
    }

    #[test]
    fn parse_key_with_flag() {
        let (k, v) = parse_key_value("FOO", Some("bar".into())).unwrap();
        assert_eq!(k, "FOO");
        assert_eq!(v, "bar");
    }
}
