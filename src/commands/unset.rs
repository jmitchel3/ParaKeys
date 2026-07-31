use std::path::PathBuf;

use anyhow::{bail, Result};

pub fn run(_key: String, _path: Option<PathBuf>) -> Result<()> {
    bail!("not implemented yet: `parakeys unset` (see MVP #7)")
}
