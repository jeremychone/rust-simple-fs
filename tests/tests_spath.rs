use simple_fs::SPath;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_spath_new_sibling() -> Result<()> {
	// -- Setup & Fixtures

	let fx_data = &[
		// (original_path, sibling_leaf_path, expected_path)
		("/some/path/to/file.txt", "new_file.md", "/some/path/to/new_file.md"),
		("some/path/to/file.txt", "new_file.md", "some/path/to/new_file.md"),
		("/some/path/to/file.txt", "file.txt", "/some/path/to/file.txt"),
		("./file.txt", "new_file.md", "./new_file.md"),
		("file.txt", "new_file.md", "new_file.md"),
	];

	// -- Exec & Check
	for data in fx_data.iter() {
		let original_path = SPath::new(data.0)?;
		let sibling_leaf_path = SPath::new(data.1)?;
		let expected_path = SPath::new(data.2)?;

		let actual_path = original_path.new_sibling(&sibling_leaf_path)?;

		assert_eq!(actual_path.to_str(), expected_path.to_str());
	}

	Ok(())
}
