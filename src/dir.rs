use crate::{Error, Result};
use std::fs;
use std::path::Path;

pub fn ensure_dir(dir: impl AsRef<Path>) -> Result<bool> {
	let dir = dir.as_ref();
	if dir.is_dir() {
		Ok(false)
	} else {
		fs::create_dir_all(dir).map_err(|e| Error::DirCantCreateAll((dir, e).into()))?;
		Ok(true)
	}
}

pub fn ensure_file_dir(file_path: impl AsRef<Path>) -> Result<bool> {
	let file_path = file_path.as_ref();
	let dir = file_path
		.parent()
		.ok_or_else(|| Error::FileHasNoParent(file_path.to_string_lossy().to_string()))?;

	if dir.is_dir() {
		Ok(false)
	} else {
		fs::create_dir_all(dir).map_err(|e| Error::DirCantCreateAll((dir, e).into()))?;
		Ok(true)
	}
}
