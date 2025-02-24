use simple_fs::list_dirs;

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
		assert!(dir.is_dir(), "Expected {} to be a directory", dir.to_str());
	}

	Ok(())
}
