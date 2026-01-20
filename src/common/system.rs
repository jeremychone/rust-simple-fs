use crate::{Error, Result, SPath};
use std::path::Path;

/// Returns the home directory of the current user as an `SPath`.
pub fn home_dir() -> Result<SPath> {
	let path = std::env::home_dir().ok_or(Error::HomeDirNotFound)?;
	SPath::from_std_path(path)
}

/// Returns the current directory as an `SPath`.
pub fn current_dir() -> Result<SPath> {
	let path = std::env::current_dir().map_err(|e| Error::FileCantRead((Path::new("."), e).into()))?;
	SPath::from_std_path_buf(path)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_common_system_home_dir_simple() -> Result<()> {
		// -- Exec
		let home = home_dir()?;

		// -- Check
		assert!(home.exists(), "Home directory should exist");
		assert!(home.is_absolute(), "Home directory should be an absolute path");

		Ok(())
	}

	#[test]
	fn test_common_system_current_dir_simple() -> Result<()> {
		// -- Exec
		let current = current_dir()?;

		// -- Check
		assert!(current.exists(), "Current directory should exist");
		assert!(current.is_absolute(), "Current directory should be an absolute path");

		Ok(())
	}
}

// endregion: --- Tests
