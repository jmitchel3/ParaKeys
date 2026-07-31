use std::path::PathBuf;

use anyhow::{bail, Result};

pub fn run(_command: Vec<String>, _path: Option<PathBuf>) -> Result<()> {
    bail!("not implemented yet: `parakeys run` (see MVP #5)")
}
