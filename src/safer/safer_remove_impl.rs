use crate::SPath;
use crate::error::{Cause, PathAndCause};
use crate::safer::SaferRemoveOptions;
use crate::{Error, Result};
use std::fs;

/// Safely deletes a directory if it passes safety checks.
///
/// Safety checks (based on options):
/// - If `restrict_to_current_dir` is true, the directory path must be below the current directory
/// - If `must_contain_any` is set, the path must contain at least one of the specified patterns
/// - If `must_contain_all` is set, the path must contain all of the specified patterns
///
/// Returns Ok(true) if the directory was deleted, Ok(false) if it didn't exist.
/// Returns an error if safety checks fail or deletion fails.
pub fn safer_remove_dir<'a>(dir_path: &SPath, options: impl Into<SaferRemoveOptions<'a>>) -> Result<bool> {
	let options = options.into();

	// If path doesn't exist, just return false
	if !dir_path.exists() {
		return Ok(false);
	}

	check_path_for_deletion_safety::<true>(dir_path, &options)?;

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
/// Safety checks (based on options):
/// - If `restrict_to_current_dir` is true, the file path must be below the current directory
/// - If `must_contain_any` is set, the path must contain at least one of the specified patterns
/// - If `must_contain_all` is set, the path must contain all of the specified patterns
///
/// Returns Ok(true) if the file was deleted, Ok(false) if it didn't exist.
/// Returns an error if safety checks fail or deletion fails.
pub fn safer_remove_file<'a>(file_path: &SPath, options: impl Into<SaferRemoveOptions<'a>>) -> Result<bool> {
	let options = options.into();

	// If path doesn't exist, just return false
	if !file_path.exists() {
		return Ok(false);
	}

	check_path_for_deletion_safety::<false>(file_path, &options)?;

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

/// Performs safety checks before deletion based on the provided options:
/// 1. If `restrict_to_current_dir` is true, path must be below the current working directory.
/// 2. If `must_contain_any` is set, path must contain at least one of those patterns.
/// 3. If `must_contain_all` is set, path must contain all of those patterns.
///
/// The const generic IS_DIR determines whether this is checking a directory (true) or file (false).
fn check_path_for_deletion_safety<const IS_DIR: bool>(path: &SPath, options: &SaferRemoveOptions<'_>) -> Result<()> {
	// Resolve the path to absolute
	let resolved = path.canonicalize()?;
	let resolved_str = resolved.as_str();
	let path_str = path.as_str();

	// -- Safety checks
	let mut error_causes = Vec::new();

	// Check that the path is below current directory (if enabled)
	if options.restrict_to_current_dir {
		let current_dir = std::env::current_dir().map_err(|e| {
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

		if !resolved_str.starts_with(current_str) {
			error_causes.push(format!("is not below current directory '{current_resolved}'"));
		}
	}

	// Check must_contain_any
	if let Some(patterns) = options.must_contain_any {
		if patterns.is_empty() {
			error_causes.push("must_contain_any cannot be an empty list (use None to disable)".to_string());
		} else {
			let has_any = patterns.iter().any(|s| path_str.contains(s));
			if !has_any {
				error_causes.push(format!("does not contain any of the required patterns: {patterns:?}"));
			}
		}
	}

	// Check must_contain_all
	if let Some(patterns) = options.must_contain_all {
		if patterns.is_empty() {
			error_causes.push("must_contain_all cannot be an empty list (use None to disable)".to_string());
		} else {
			let missing: Vec<_> = patterns.iter().filter(|s| !path_str.contains(*s)).collect();
			if !missing.is_empty() {
				error_causes.push(format!("does not contain all required patterns, missing: {missing:?}"));
			}
		}
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
