use crate::{Error, Result};
use std::fs;
use std::path::Path;

pub fn ensure_dir(dir: &Path) -> Result<bool> {
	if dir.is_dir() {
		Ok(false)
	} else {
		fs::create_dir_all(dir).map_err(|e| Error::DirCantCreateAll((dir, e).into()))?;
		Ok(true)
	}
}
