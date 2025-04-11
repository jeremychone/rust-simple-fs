use simple_fs::{ListOptions, iter_dirs, list_dirs};

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_iter_dirs_list_dirs() -> Result<()> {
	// -- Exec: List directories in the current working directory, no filtering.
	let dirs = list_dirs("./", None, None)?;

	// -- Check: Ensure that at least one directory is found.
	assert!(
		!dirs.is_empty(),
		"Expected to find at least one directory, but found none."
	);

	// -- Check: Each returned entry must be a directory.
	for dir in dirs {
		assert!(dir.is_dir(), "Expected {} to be a directory", dir.as_str());
	}

	Ok(())
}

#[test]
fn test_list_dirs_one_level_dotted() -> Result<()> {
	// -- Exec: List directories in the tests-data directory, no filtering.
	let dirs = list_dirs("./tests-data/", None, None)?;

	// -- Check: Ensure we find the expected directories.
	let dir_paths = dirs.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert!(dir_paths.contains(&"./tests-data/dir1"), "Should contain dir1");
	assert!(
		dir_paths.contains(&"./tests-data/another-dir"),
		"Should contain another-dir"
	);

	Ok(())
}

#[test]
fn test_list_dirs_with_glob_pattern() -> Result<()> {
	// -- Exec: List directories in tests-data directory matching a glob pattern.
	let dirs = list_dirs("./tests-data/", Some(&["./tests-data/dir1"]), None)?;

	// -- Check: Ensure we only find directories matching the pattern.
	let dir_paths = dirs.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert_eq!(dirs.len(), 1, "Should have 1 directory matching 'dir1'");
	assert!(dir_paths.contains(&"./tests-data/dir1"), "Should contain dir1");
	assert!(
		!dir_paths.contains(&"./tests-data/another-dir"),
		"Should not contain another-dir"
	);

	Ok(())
}

#[test]
fn test_list_dirs_with_relative_glob() -> Result<()> {
	// -- Exec: List directories using relative glob pattern.
	let dirs = list_dirs(
		"./tests-data/",
		Some(&["another-dir"]),
		Some(ListOptions::default().with_relative_glob()),
	)?;

	// -- Check: Ensure we only find directories matching the pattern.
	let dir_paths = dirs.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert_eq!(dirs.len(), 1, "Should have 1 directory matching 'another-dir'");
	assert!(
		dir_paths.contains(&"./tests-data/another-dir"),
		"Should contain another-dir"
	);
	assert!(!dir_paths.contains(&"./tests-data/dir1"), "Should not contain dir1");

	Ok(())
}

#[test]
fn test_list_dirs_recursive() -> Result<()> {
	// -- Exec: List all directories recursively in tests-data.
	let dirs = list_dirs("./tests-data/", Some(&["./tests-data/**"]), None)?;

	// -- Check: Ensure we find all the expected directories.
	let dir_paths = dirs.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	
	assert!(dir_paths.contains(&"./tests-data/dir1"), "Should contain dir1");
	assert!(
		dir_paths.contains(&"./tests-data/dir1/dir2"),
		"Should contain dir1/dir2"
	);
	assert!(
		dir_paths.contains(&"./tests-data/dir1/dir2/dir3"),
		"Should contain dir1/dir2/dir3"
	);
	assert!(
		dir_paths.contains(&"./tests-data/another-dir"),
		"Should contain another-dir"
	);
	assert!(
		dir_paths.contains(&"./tests-data/another-dir/sub-dir"),
		"Should contain another-dir/sub-dir"
	);
	assert!(
		dir_paths.contains(&"./tests-data/another-dir/sub-dir/deep-folder"),
		"Should contain another-dir/sub-dir/deep-folder"
	);

	Ok(())
}

#[test]
fn test_list_dirs_with_exclude_option() -> Result<()> {
	// -- Exec: List directories with exclusion pattern.
	let list_options = ListOptions::default()
		.with_exclude_globs(&["**/dir2", "**/dir2/**"])
		.with_relative_glob(); // Add relative_glob for proper pattern matching

	let dirs = list_dirs("./tests-data/", Some(&["**"]), Some(list_options))?;

	// -- Check: Ensure excluded directories are not in the results.
	let dir_paths = dirs.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert!(dir_paths.contains(&"./tests-data/dir1"), "Should contain dir1");
	assert!(
		!dir_paths.contains(&"./tests-data/dir1/dir2"),
		"Should not contain dir1/dir2"
	);
	assert!(
		!dir_paths.contains(&"./tests-data/dir1/dir2/dir3"),
		"Should not contain dir1/dir2/dir3"
	);
	assert!(
		dir_paths.contains(&"./tests-data/another-dir"),
		"Should contain another-dir"
	);

	Ok(())
}

#[test]
fn test_iter_dirs_functionality() -> Result<()> {
	// -- Exec: Use iter_dirs to create an iterator over directories.
	let dir_iter = iter_dirs("./tests-data/", None, None)?;

	// -- Check: Convert iterator to vector and verify results.
	let dirs: Vec<_> = dir_iter.collect();
	assert!(dirs.len() >= 2, "Should have at least 2 directories");

	let dir_paths = dirs.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert!(dir_paths.contains(&"./tests-data/dir1"), "Should contain dir1");
	assert!(
		dir_paths.contains(&"./tests-data/another-dir"),
		"Should contain another-dir"
	);

	Ok(())
}

#[test]
fn test_list_dirs_absolute_path() -> Result<()> {
	// -- Exec: Get absolute path and use it for listing directories.
	let test_data_abs = std::fs::canonicalize("./tests-data/")?;
	let dirs = list_dirs(&test_data_abs, None, None)?;

	// -- Check: Verify that we can find directories using absolute path.
	assert!(
		dirs.len() >= 2,
		"Should have at least 2 directories using absolute path"
	);

	// Get the directory names for easier comparison
	let dir_names: Vec<_> = dirs.iter().map(|p| p.name()).collect();
	assert!(dir_names.contains(&"dir1"), "Should contain dir1");
	assert!(dir_names.contains(&"another-dir"), "Should contain another-dir");

	Ok(())
}

#[test]
fn test_list_dirs_with_negative_glob() -> Result<()> {
	// -- Exec: List directories with a negative (exclusion) pattern in include_globs.
	let dirs = list_dirs(
		"./tests-data/",
		Some(&["**", "!**/dir2"]), // Include all directories but exclude dir2
		None,
	)?;

	// -- Check: Ensure excluded directories are not in the results.
	let dir_paths = dirs.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert!(dir_paths.contains(&"./tests-data/dir1"), "Should contain dir1");
	assert!(
		!dir_paths.contains(&"./tests-data/dir1/dir2"),
		"Should not contain dir1/dir2"
	);
	assert!(
		dir_paths.contains(&"./tests-data/another-dir"),
		"Should contain another-dir"
	);
	assert!(
		dir_paths.contains(&"./tests-data/another-dir/sub-dir"),
		"Should contain another-dir/sub-dir"
	);

	Ok(())
}

#[test]
fn test_list_dirs_with_multiple_negative_globs() -> Result<()> {
	// -- Exec: List directories with multiple negative patterns in include_globs.
	let dirs = list_dirs(
		"./tests-data/",
		Some(&[
			"**",              // Include all directories
			"!**/dir2",        // Exclude dir2
			"!**/deep-folder", // Exclude deep-folder
		]),
		None,
	)?;

	// -- Check: Ensure all excluded directories are not in the results.
	let dir_paths = dirs.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert!(dir_paths.contains(&"./tests-data/dir1"), "Should contain dir1");
	assert!(
		!dir_paths.contains(&"./tests-data/dir1/dir2"),
		"Should not contain dir1/dir2"
	);
	assert!(
		dir_paths.contains(&"./tests-data/another-dir"),
		"Should contain another-dir"
	);
	assert!(
		dir_paths.contains(&"./tests-data/another-dir/sub-dir"),
		"Should contain another-dir/sub-dir"
	);
	assert!(
		!dir_paths.contains(&"./tests-data/another-dir/sub-dir/deep-folder"),
		"Should not contain another-dir/sub-dir/deep-folder"
	);

	Ok(())
}

#[test]
fn test_list_dirs_with_only_negative_globs() -> Result<()> {
	// -- Exec: List directories with only negative patterns (should default to "**" for includes).
	let dirs = list_dirs(
		"./tests-data/",
		Some(&["!**/dir2", "!**/deep-folder"]), // Only exclusion patterns
		None,
	)?;

	// -- Check: Verify filtering works with only negative patterns.
	let dir_paths = dirs.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert!(dir_paths.contains(&"./tests-data/dir1"), "Should contain dir1");
	assert!(
		!dir_paths.contains(&"./tests-data/dir1/dir2"),
		"Should not contain dir1/dir2"
	);
	assert!(
		!dir_paths.contains(&"./tests-data/another-dir/sub-dir/deep-folder"),
		"Should not contain another-dir/sub-dir/deep-folder"
	);

	Ok(())
}
