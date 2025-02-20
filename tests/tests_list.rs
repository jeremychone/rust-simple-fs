use simple_fs::{list_files, ListOptions};

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_list_one_level_dotted() -> Result<()> {
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
fn test_list_rel_one_level_dotted() -> Result<()> {
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
fn test_list_rel_one_level_no_file() -> Result<()> {
	// -- Exec
	let res = list_files("./", Some(&["./*.rs"]), Some(ListOptions::from_relative_glob(true)))?;

	// -- Check
	assert_eq!(res.len(), 0, "Should have 0 files with *.rs in base dir");

	Ok(())
}

#[test]
fn test_list_one_level_no_file() -> Result<()> {
	// -- Exec
	let res = list_files("./", Some(&["./*.rs"]), None)?;

	// -- Check
	assert_eq!(res.len(), 0, "Should have 0 files with *.rs in base dir");

	Ok(())
}

#[test]
fn test_list_one_file_dotted() -> Result<()> {
	// -- Exec
	let res = list_files(".", Some(&["./Cargo.toml"]), None)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.to_str()).collect::<Vec<_>>();
	assert_eq!(res.len(), 1, "Should have 1 file");
	assert!(res_paths.contains(&"./Cargo.toml"), " Should contain Cargo.toml");

	Ok(())
}

#[test]
fn test_list_sub_level_dotted() -> Result<()> {
	// -- Exec
	let res = list_files("./", Some(&["./tests/**/*.rs"]), None)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.to_str()).collect::<Vec<_>>();
	assert_tests_res(&res_paths);

	Ok(())
}

#[test]
fn test_list_sub_dir_full_path() -> Result<()> {
	// -- Exec
	let res = list_files("./tests/", Some(&["./tests/tests*.rs"]), None)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.to_str()).collect::<Vec<_>>();
	assert_tests_res(&res_paths);

	Ok(())
}

/// Here the globs are lative to the base dir given (here `./tests/`)
#[test]
fn test_list_sub_dir_rel_glob() -> Result<()> {
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
fn test_list_absolute_wildcard() -> Result<()> {
	// Get the absolute path to the "src" directory.
	let src_abs = std::fs::canonicalize("src")?;
	let src_abs_str = src_abs.to_str().unwrap();

	// Construct a glob pattern that should match the "spath.rs" file.
	let pattern = format!("{}/{}", src_abs_str, "*path.rs");

	// Execute list_files using the absolute src directory and the wildcard pattern.
	let files = list_files(&src_abs, Some(&[pattern.as_str()]), None)?;

	// Check that at least one file's path ends with "spath.rs"
	let found = files.iter().any(|p| p.to_str().ends_with("spath.rs"));
	assert!(found, "Expected to find spath.rs file with wildcard absolute pattern");

	Ok(())
}

#[test]
fn test_list_absolute_direct() -> Result<()> {
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
