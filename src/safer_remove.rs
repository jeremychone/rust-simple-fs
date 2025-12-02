use crate::SPath;
use crate::error::{Cause, PathAndCause};
use crate::{Error, Result};
use std::fs;

/// Safely deletes a directory if it passes safety checks.
///
/// Safety checks:
/// - The directory path must be below the current directory
/// - The directory path must contain one of the allowed substrings
///
/// Returns Ok(true) if the directory was deleted, Ok(false) if it didn't exist.
/// Returns an error if safety checks fail or deletion fails.
pub fn safer_remove_dir(dir_path: &SPath, allowed_patterns: &[&str]) -> Result<bool> {
	// If path doesn't exist, just return false
	if !dir_path.exists() {
		return Ok(false);
	}

	check_path_for_deletion_safety::<true>(dir_path, allowed_patterns)?;

	// Perform the deletion
	fs::remove_dir_all(dir_path.as_std_path()).map_err(|e| {
		Error::DirNotSafeToRemove(PathAndCause {
			path: dir_path.to_string(),
			cause: Cause::Io(Box::new(e)),
		})
	})?;

	Ok(true)
}

/// Safely deletes a file if it passes safety checks.
///
/// Safety checks:
/// - The file path must be below the current directory
/// - The file path must contain one of the allowed substrings
///
/// Returns Ok(true) if the file was deleted, Ok(false) if it didn't exist.
/// Returns an error if safety checks fail or deletion fails.
pub fn safer_remove_file(file_path: &SPath, allowed_patterns: &[&str]) -> Result<bool> {
	// If path doesn't exist, just return false
	if !file_path.exists() {
		return Ok(false);
	}

	check_path_for_deletion_safety::<false>(file_path, allowed_patterns)?;

	// Perform the deletion
	fs::remove_file(file_path.as_std_path()).map_err(|e| {
		Error::FileNotSafeToRemove(PathAndCause {
			path: file_path.to_string(),
			cause: Cause::Io(Box::new(e)),
		})
	})?;

	Ok(true)
}

// region:    --- Support

/// Performs safety checks before deletion:
/// 1. Path must be resolvable and below the current working directory.
/// 2. Path must contain one of the allowed substrings/patterns.
///
/// The const generic IS_DIR determines whether this is checking a directory (true) or file (false).
fn check_path_for_deletion_safety<const IS_DIR: bool>(path: &SPath, allowed_patterns: &[&str]) -> Result<()> {
	// Resolve the path to absolute
	let resolved = path.canonicalize()?;
	let resolved_str = resolved.as_str();

	// Get current directory and resolve it
	let current_dir = std::env::current_dir().map_err(|e| {
		// Use appropriate error based on item type
		let pac = PathAndCause {
			path: path.to_string(),
			cause: Cause::Io(Box::new(e)),
		};
		if IS_DIR {
			Error::DirNotSafeToRemove(pac)
		} else {
			Error::FileNotSafeToRemove(pac)
		}
	})?;
	let current_dir_path = SPath::from_std_path_buf(current_dir)?;
	let current_resolved = current_dir_path.canonicalize()?;
	let current_str = current_resolved.as_str();

	// -- Safety checks
	let mut error_causes = Vec::new();

	// Check that the path is below current directory
	if !resolved_str.starts_with(current_str) {
		error_causes.push(format!("is not below current directory '{current_resolved}'"));
	}

	// Check that the path contains one of the allowed substrings
	let path_str = path.as_str();
	let allowed = allowed_patterns.iter().any(|s| path_str.contains(s));
	if !allowed {
		error_causes.push(format!(
			"does not contain any of the allowed patterns: {allowed_patterns:?}"
		));
	}

	if !error_causes.is_empty() {
		let cause_msg = format!("Safety check failed: {}", error_causes.join("; "));
		let path_and_cause = PathAndCause {
			path: path.to_string(),
			cause: Cause::Custom(cause_msg),
		};

		if IS_DIR {
			return Err(Error::DirNotSafeToRemove(path_and_cause));
		} else {
			return Err(Error::FileNotSafeToRemove(path_and_cause));
		}
	}

	Ok(())
}

// endregion: --- Support
