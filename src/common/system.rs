use crate::{Error, Result, SPath};

/// Returns the home directory of the current user as an `SPath`.
pub fn home_dir() -> Result<SPath> {
	#[allow(deprecated)]
	let path = std::env::home_dir().ok_or(Error::HomeDirNotFound)?;
	SPath::from_std_path(path)
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
}

// endregion: --- Tests
