use simple_fs::{iter_files, list_files, ListOptions};

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_list_files_one_level_dotted() -> Result<()> {
	// NOTE F"*.toml" it won't work since absolute
	//      Use relative glob below
	// -- Exec
	let res = list_files("./", Some(&["./*.toml"]), None)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.to_str()).collect::<Vec<_>>();
	assert_eq!(res.len(), 2, "Should have 2 files with *.toml");
	assert!(res_paths.contains(&"./Cargo.toml"), " Should contain Cargo.toml");
	assert!(res_paths.contains(&"./rustfmt.toml"), " Should contain rustfmt.toml");

	Ok(())
}

#[test]
fn test_list_files_rel_one_level_dotted() -> Result<()> {
	// NOTE With relative_glob, "*.toml" now works
	// -- Exec
	let res = list_files("./", Some(&["*.toml"]), Some(ListOptions::from_relative_glob(true)))?;

	// -- Check
	let res_paths = res.iter().map(|p| p.to_str()).collect::<Vec<_>>();
	assert_eq!(res.len(), 2, "Should have 2 files with *.toml");
	assert!(res_paths.contains(&"./Cargo.toml"), " Should contain Cargo.toml");
	assert!(res_paths.contains(&"./rustfmt.toml"), " Should contain rustfmt.toml");

	Ok(())
}

#[test]
fn test_list_files_rel_one_level_no_file() -> Result<()> {
	// -- Exec
	let res = list_files("./", Some(&["./*.rs"]), Some(ListOptions::from_relative_glob(true)))?;

	// -- Check
	assert_eq!(res.len(), 0, "Should have 0 files with *.rs in base dir");

	Ok(())
}

#[test]
fn test_list_files_one_level_no_file() -> Result<()> {
	// -- Exec
	let res = list_files("./", Some(&["./*.rs"]), None)?;

	// -- Check
	assert_eq!(res.len(), 0, "Should have 0 files with *.rs in base dir");

	Ok(())
}

#[test]
fn test_list_files_one_file_dotted() -> Result<()> {
	// -- Exec
	let res = list_files(".", Some(&["./Cargo.toml"]), None)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.to_str()).collect::<Vec<_>>();
	assert_eq!(res.len(), 1, "Should have 1 file");
	assert!(res_paths.contains(&"./Cargo.toml"), " Should contain Cargo.toml");

	Ok(())
}

#[test]
fn test_list_files_sub_level_dotted() -> Result<()> {
	// -- Exec
	let res = list_files("./", Some(&["./tests/**/*.rs"]), None)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.to_str()).collect::<Vec<_>>();
	assert_tests_res(&res_paths);

	Ok(())
}

#[test]
fn test_list_files_sub_dir_full_path() -> Result<()> {
	// -- Exec
	let res = list_files("./tests/", Some(&["./tests/tests*.rs"]), None)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.to_str()).collect::<Vec<_>>();
	assert_tests_res(&res_paths);

	Ok(())
}

/// Here the globs are relative to the base dir given (here `./tests/`)
#[test]
fn test_list_files_sub_dir_rel_glob() -> Result<()> {
	// -- Exec
	let res = list_files(
		"./tests/",
		Some(&["tests*.rs"]),
		Some(ListOptions::from_relative_glob(true)),
	)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.to_str()).collect::<Vec<_>>();
	assert_tests_res(&res_paths);

	Ok(())
}

#[test]
fn test_list_files_absolute_wildcard() -> Result<()> {
	// TODO: Need to support absolute path out of the base dir of the list
	// Get the absolute path to the "src" directory.
	let src_abs = std::fs::canonicalize("./")?;
	let src_abs_str = src_abs.to_str().unwrap();

	// Get the parent directory of the file.
	let parent_dir = src_abs.parent().expect("Should be parent dir");

	// Construct a glob pattern that should match the "spath.rs" file.
	let pattern = format!("{}/{}", src_abs_str, "**/*path.rs");

	// Execute list_files using the absolute src directory and the wildcard pattern.
	let files = list_files("src/", Some(&[pattern.as_str()]), None)?;

	// Check that at least one file's path ends with "spath.rs"
	let found = files.iter().any(|p| p.to_str().ends_with("spath.rs"));
	assert!(found, "Expected to find spath.rs file with wildcard absolute pattern");

	Ok(())
}

#[test]
fn test_list_files_absolute_direct() -> Result<()> {
	// TODO: Need to support absolute path out of the base dir of the list
	// Get the absolute path to "src/spath.rs".
	let file_abs = std::fs::canonicalize("src/spath.rs")?;
	let file_abs_str = file_abs.to_str().unwrap();

	// Get the parent directory of the file.
	let parent_dir = file_abs.parent().unwrap();

	// Execute list_files using the parent directory and an exact match glob for the file.
	let files = list_files(parent_dir, Some(&[file_abs_str]), None)?;
	assert_eq!(files.len(), 1, "Should have exactly one file with exact match");

	let returned_path = files[0].to_str();
	assert_eq!(
		returned_path, file_abs_str,
		"The file path should match the absolute file path"
	);

	Ok(())
}

#[test]
fn test_list_files_mixed_absolute_and_relative_globs() -> Result<()> {
	// Mix an absolute glob and a relative glob in the same call.
	let abs_pattern = std::fs::canonicalize("./tests/tests_list.rs")?;
	let patterns = [abs_pattern.to_str().unwrap(), "tests/tests_spath.rs"];
	let res = list_files("./", Some(&patterns), None)?;
	let res_paths: Vec<&str> = res.iter().map(|p| p.to_str()).collect();
	assert_eq!(res.len(), 2, "Expected both files to be found using mixed patterns");
	assert!(
		res_paths.iter().any(|&p| p.ends_with("tests_list.rs")),
		"Should contain tests_list.rs"
	);
	assert!(
		res_paths.iter().any(|&p| p.ends_with("tests_spath.rs")),
		"Should contain tests_spath.rs"
	);
	Ok(())
}

#[test]
fn test_list_files_mixed_absolute_and_relative_globs_with_relative_option() -> Result<()> {
	// Mix an absolute glob and a relative glob with the relative_glob option enabled.
	let abs_pattern = std::fs::canonicalize("./tests/tests_spath.rs")?;
	let patterns = ["tests/tests_list.rs", abs_pattern.to_str().unwrap()];
	let res = list_files("./", Some(&patterns), Some(ListOptions::from_relative_glob(true)))?;
	let res_paths: Vec<&str> = res.iter().map(|p| p.to_str()).collect();
	assert_eq!(
		res.len(),
		2,
		"Expected both files to be found using mixed patterns with relative_glob option"
	);
	assert!(
		res_paths.iter().any(|&p| p.ends_with("tests_list.rs")),
		"Should contain tests_list.rs"
	);
	assert!(
		res_paths.iter().any(|&p| p.ends_with("tests_spath.rs")),
		"Should contain tests_spath.rs"
	);
	Ok(())
}

#[test]
fn test_list_iter_files_simple_glob_ok() -> Result<()> {
	let iter = iter_files("./", Some(&["./src/s*.rs"]), None)?;
	let count = iter.count();
	assert_eq!(count, 2, "Expected 2 files matching pattern");
	Ok(())
}

#[test]
fn test_list_iter_files_nested_and_exclude_ok() -> Result<()> {
	let excludes = [simple_fs::DEFAULT_EXCLUDE_GLOBS, &["**/.devai", "*.lock", "**/w*.rs"]].concat();
	let iter = iter_files("./", Some(&["./src/**/*.rs"]), Some(excludes.into()))?;
	let count = iter.count();
	assert_eq!(count, 11, "Expected 11 files matching pattern");
	Ok(())
}

// region:    --- Support

/// Reusable function for the the result of `./tests/tests*.rs`
fn assert_tests_res(res_paths: &[&str]) {
	assert_eq!(res_paths.len(), 2, "Should have 2 test files with *.rs");
	assert!(
		res_paths.contains(&"./tests/tests_list.rs"),
		" Should contain './tests/tests_list.rs'"
	);
	assert!(
		res_paths.contains(&"./tests/tests_spath.rs"),
		" Should contain './tests/tests_spath.rs'"
	);
}

// endregion: --- Support
