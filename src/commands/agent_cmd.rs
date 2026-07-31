use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::agent::{
    self, agent_grant_path, decrypt_grant, load_agent_key, load_grant, save_grant,
};
use crate::keywallet::project_root;

pub fn keygen(path: Option<PathBuf>) -> Result<()> {
    let root = project_root(path).context("resolve project path")?;
    let (key_path, pub_path, pub_b64) = agent::keygen(&root).context("agent keygen")?;
    println!("Agent private key: {} (mode 0600; do not commit)", key_path.display());
    println!("Agent public key:  {}", pub_path.display());
    println!();
    println!("Share the public key file with the human vault owner to create grants:");
    println!("{pub_b64}");
    Ok(())
}

pub fn apply(grant: PathBuf, path: Option<PathBuf>) -> Result<()> {
    let root = project_root(path).context("resolve project path")?;
    let grant_src = if grant.is_absolute() {
        grant
    } else {
        root.join(grant)
    };
    if !grant_src.is_file() {
        bail!("grant file not found: {}", grant_src.display());
    }

    // Validate decrypt with agent key if present; still allow store without decrypt
    // when key exists, require successful decrypt.
    let envelope = load_grant(&grant_src).context("parse grant")?;
    if agent::agent_key_path(&root).is_file() {
        let key = load_agent_key(&root).context("load agent key")?;
        let data = decrypt_grant(&envelope, &key).context(
            "grant does not decrypt with local agent key (wrong agent or corrupt grant)",
        )?;
        println!(
            "Grant validates for {} key(s): {}",
            data.keys.len(),
            envelope.allowlist.join(", ")
        );
    } else {
        println!(
            "warn: no agent.key yet; storing grant without decrypt check (run agent keygen first ideally)"
        );
    }

    let dest = agent_grant_path(&root);
    save_grant(&dest, &envelope).context("store grant")?;
    println!("Stored grant at {} (agent path; no human vault key used)", dest.display());
    Ok(())
}

pub fn run(command: Vec<String>, path: Option<PathBuf>) -> Result<()> {
    if command.is_empty() {
        bail!("usage: parakeys agent run -- <command> [args...]");
    }
    let root = project_root(path).context("resolve project path")?;
    let grant_path = agent_grant_path(&root);
    if !grant_path.is_file() {
        bail!(
            "no grant at {} — run `parakeys agent apply grant.enc` first",
            grant_path.display()
        );
    }
    let agent_key = load_agent_key(&root).context("load agent key")?;
    let envelope = load_grant(&grant_path).context("load grant")?;
    let data = decrypt_grant(&envelope, &agent_key).context("decrypt grant")?;

    // Start from process env but strip potential human vault secrets by only
    // injecting grant keys explicitly (overwrite).
    let mut env_map: HashMap<String, String> = std::env::vars().collect();
    // Remove any keys that look like they might be full vault - we only set grant allowlist.
    for k in env_map.keys().cloned().collect::<Vec<_>>() {
        if envelope.allowlist.iter().any(|a| a == &k) {
            env_map.remove(&k);
        }
    }
    for (k, v) in &data.keys {
        env_map.insert(k.clone(), v.clone());
    }

    let program = &command[0];
    let args = &command[1..];
    let status = Command::new(program)
        .args(args)
        .envs(&env_map)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to execute `{program}`"))?;

    if let Some(code) = status.code() {
        if code != 0 {
            std::process::exit(code);
        }
    } else {
        std::process::exit(1);
    }
    Ok(())
}
