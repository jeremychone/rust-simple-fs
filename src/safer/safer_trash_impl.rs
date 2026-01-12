use crate::SPath;
use crate::error::{Cause, PathAndCause};
use crate::safer::{SaferTrashOptions, support};
use crate::{Error, Result};

/// Safely moves a directory to the system trash if it passes safety checks.
///
/// Safety checks (based on options):
/// - If `restrict_to_current_dir` is true, the directory path must be below the current directory
/// - If `must_contain_any` is set, the path must contain at least one of the specified patterns
/// - If `must_contain_all` is set, the path must contain all of the specified patterns
///
/// Returns Ok(true) if the directory was trashed, Ok(false) if it didn't exist.
/// Returns an error if safety checks fail or trashing fails.
pub fn safer_trash_dir<'a>(dir_path: &SPath, options: impl Into<SaferTrashOptions<'a>>) -> Result<bool> {
	let options = options.into();

	// If path doesn't exist, just return false
	if !dir_path.exists() {
		return Ok(false);
	}

	let causes = support::check_path_safety_causes(
		dir_path,
		options.restrict_to_current_dir,
		options.must_contain_any,
		options.must_contain_all,
	)?;

	if !causes.is_empty() {
		return Err(Error::DirNotSafeToTrash(PathAndCause {
			path: dir_path.to_string(),
			cause: Cause::Custom(format!("Safety check failed: {}", causes.join("; "))),
		}));
	}

	// Perform the trash operation
	trash::delete(dir_path.as_std_path()).map_err(|e| {
		Error::CantTrash(PathAndCause {
			path: dir_path.to_string(),
			cause: Cause::Custom(e.to_string()),
		})
	})?;

	Ok(true)
}

/// Safely moves a file to the system trash if it passes safety checks.
///
/// Safety checks (based on options):
/// - If `restrict_to_current_dir` is true, the file path must be below the current directory
/// - If `must_contain_any` is set, the path must contain at least one of the specified patterns
/// - If `must_contain_all` is set, the path must contain all of the specified patterns
///
/// Returns Ok(true) if the file was trashed, Ok(false) if it didn't exist.
/// Returns an error if safety checks fail or trashing fails.
pub fn safer_trash_file<'a>(file_path: &SPath, options: impl Into<SaferTrashOptions<'a>>) -> Result<bool> {
	let options = options.into();

	// If path doesn't exist, just return false
	if !file_path.exists() {
		return Ok(false);
	}

	let causes = support::check_path_safety_causes(
		file_path,
		options.restrict_to_current_dir,
		options.must_contain_any,
		options.must_contain_all,
	)?;

	if !causes.is_empty() {
		return Err(Error::FileNotSafeToTrash(PathAndCause {
			path: file_path.to_string(),
			cause: Cause::Custom(format!("Safety check failed: {}", causes.join("; "))),
		}));
	}

	// Perform the trash operation
	trash::delete(file_path.as_std_path()).map_err(|e| {
		Error::CantTrash(PathAndCause {
			path: file_path.to_string(),
			cause: Cause::Custom(e.to_string()),
		})
	})?;

	Ok(true)
}
