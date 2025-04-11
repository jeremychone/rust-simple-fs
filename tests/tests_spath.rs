use simple_fs::SPath;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_spath_spath_new_sibling() -> Result<()> {
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
		let original_path = SPath::new(data.0).into_normalized();
		let sibling_leaf_path = SPath::new(data.1).into_normalized();
		let expected_path = SPath::new(data.2).into_normalized();

		let actual_path = original_path.new_sibling(sibling_leaf_path).into_normalized();

		assert_eq!(actual_path.as_str(), expected_path.as_str());
	}

	Ok(())
}

#[test]
fn test_spath_spath_diff() -> Result<()> {
	// -- Setup & Fixtures
	let fx_data = &[
		// (base_path, target_path, expected_path)
		(
			"/some/base/path",
			"/some/base/path/sub_dir/some_file.md",
			"sub_dir/some_file.md",
		),
		(
			"/some/base/path/sub_dir/some_file.md",
			"/some/base/path/some/other-file.md",
			"../../some/other-file.md",
		),
	];

	// -- Exec & Check
	for data in fx_data.iter() {
		let base_path = SPath::new(data.0).into_normalized();
		let target_path = SPath::new(data.1).into_normalized();
		let expected_path = SPath::new(data.2).into_normalized();

		let diff_path = target_path.diff(&base_path).ok_or("Should have diff")?.into_normalized();
		let rejoined_path = base_path.join(&diff_path).collapse().into_normalized();

		assert_eq!(diff_path.as_str(), expected_path.as_str());
		assert_eq!(rejoined_path.as_str(), target_path.as_str());
	}

	Ok(())
}
