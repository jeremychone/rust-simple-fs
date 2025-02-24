use crate::{Error, Result, SPath};
use std::path::Path;
use walkdir::WalkDir;

/// Lists directories under the `base_path` up to a given `depth`.
///
/// - If `base_path` does not exist, returns an appropriate error.
/// - Only directories are returned; files are skipped.
/// - If an entry cannot be converted to an `SPath`, it is skipped.
pub fn list_dirs(base_path: impl AsRef<Path>, depth: usize) -> Result<Vec<SPath>> {
	let base = base_path.as_ref();

	// Check if the base path exists.
	if !base.exists() {
		return Err(Error::FileNotFound(base.to_string_lossy().to_string()));
	}

	let mut dirs = Vec::new();

	// WalkDir::new iterates from the base, with the specified max_depth.
	// Note that if depth is less than 1, we treat it as 1 to at least check the base.
	for entry in WalkDir::new(base).max_depth(depth).into_iter().filter_map(|e| e.ok()) {
		// Only process directories.
		if entry.file_type().is_dir() {
			if let Some(s_path) = SPath::from_walkdir_entry_ok(entry) {
				dirs.push(s_path);
			}
		}
	}

	Ok(dirs)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_iter_dirs_list_dirs_one_level() -> Result<()> {
		// -- Exec: list directories one level deep.
		let dirs = list_dirs("./", 1)?;

		// -- Check: Ensure that there are more than 3 directories
		assert!(
			dirs.len() > 3,
			"Expected more than 3 directories, but got {}",
			dirs.len()
		);

		// Create a vector of directory names from the list
		let dir_names: Vec<String> = dirs
			.iter()
			.filter_map(|d| d.path().file_name().and_then(|n| n.to_str()).map(String::from))
			.collect();

		// Check that expected directories are present.
		assert!(
			dir_names.contains(&"src".to_string()),
			"Expected directory 'src' to be present"
		);
		assert!(
			dir_names.contains(&"tests".to_string()),
			"Expected directory 'tests' to be present"
		);
		assert!(
			dir_names.contains(&"target".to_string()),
			"Expected directory 'target' to be present"
		);

		Ok(())
	}
}

// endregion: --- Tests
