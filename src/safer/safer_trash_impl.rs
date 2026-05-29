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

	perform_trash(dir_path)?;

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

	perform_trash(file_path)?;

	Ok(true)
}

// region:    --- Trash Support

/// Performs the trash operation for a file or directory.
///
/// On macOS, attempts a silent trash move first (to avoid the system trash sound);
/// if unavailable or it fails (e.g., cross-device), falls back to the `trash` crate.
/// On other platforms, always uses the `trash` crate.
fn perform_trash(path: &SPath) -> Result<()> {
	let trash_done = {
		#[cfg(target_os = "macos")]
		{
			crate::safer::safer_trash_mac_support::try_silent_trash_move(path)?
		}
		#[cfg(not(target_os = "macos"))]
		{
			false
		}
	};

	if !trash_done {
		trash::delete(path.as_std_path()).map_err(|e| {
			Error::CantTrash(PathAndCause {
				path: path.to_string(),
				cause: Cause::Custom(e.to_string()),
			})
		})?;
	}

	Ok(())
}

// endregion: --- Trash Support
