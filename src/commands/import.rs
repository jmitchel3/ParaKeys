use std::path::PathBuf;

use anyhow::{bail, Result};

pub fn run(_env_file: PathBuf, _path: Option<PathBuf>) -> Result<()> {
    bail!("not implemented yet: `parakeys import` (see MVP #4)")
}
