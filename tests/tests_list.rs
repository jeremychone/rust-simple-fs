use simple_fs::{list_files, ListOptions};

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_list_one_level_dotted() -> Result<()> {
	// -- Exec
	let res = list_files("./", Some(&["*.toml"]), None)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.to_str()).collect::<Vec<_>>();
	assert_eq!(res.len(), 2, "Should have 2 files with *.toml");
	assert!(res_paths.contains(&"./Cargo.toml"), " Should contain Cargo.toml");
	assert!(res_paths.contains(&"./rustfmt.toml"), " Should contain rustfmt.toml");

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
