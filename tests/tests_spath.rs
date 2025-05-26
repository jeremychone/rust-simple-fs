use simple_fs::SPath;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_spath_starts_with_simple() -> Result<()> {
	// -- Setup & Fixtures
	let fx_data = &[
		// (path_str, prefix_str, expected_bool)
		// Exact matches
		("~/passwd", "~/", true),
		("~/passwd", "~", true),
		("~passwd", "~", false), // because `~` is not a path component
		("/etc/passwd", "/etc/passwd", true),
		("src/main.rs", "src/main.rs", true),
		// Prefix matches
		("/etc/passwd", "/etc", true),
		("/etc/passwd", "/etc/", true),
		("some/path/to/file", "some/path", true),
		("some/path/to/file", "some/path/", true),
		// Prefix matches with extra slashes in prefix
		("/etc/passwd", "/etc/passwd/", true),   // extra slash is okay
		("/etc/passwd", "/etc/passwd///", true), // multiple extra slashes are okay
		// Non-matches
		("/etc/passwd", "/e", false),              // partial component
		("/etc/passwd", "/etc/passwd.txt", false), // different file
		("src/main.rs", "src/main", false),        // partial component
		("file.txt", "another-file.txt", false),
		("data/project/file.txt", "data/project/files", false), // prefix is longer in component name
		// Relative paths
		("relative/path", "relative", true),
		("relative/path", "relative/", true),
		("./config/settings.toml", "./config", true),
		("./config/settings.toml", "./config/", true),
		// Edge cases
		("file", "file", true),
		("file", "f", false),
		("/", "/", true),
		("/a/b", "/", true),
		("a/b", "a", true),
		// Non-match when base is longer
		("path/to/file", "path/to/file/extra", false),
	];

	// -- Exec & Check
	for &(path_str, prefix_str, expected_bool) in fx_data.iter() {
		let spath = SPath::new(path_str);
		let prefix_path = SPath::new(prefix_str); // SPath can take Path as AsRef
		let actual_bool = spath.starts_with(prefix_path.std_path()); // SPath.starts_with takes AsRef<Path>

		assert_eq!(
			actual_bool, expected_bool,
			"Path: '{}', Prefix: '{}'. Expected: {}, Got: {}",
			path_str, prefix_str, expected_bool, actual_bool
		);

		// Test with &str directly
		let actual_bool_str_prefix = spath.starts_with(prefix_str);
		assert_eq!(
			actual_bool_str_prefix, expected_bool,
			"Path: '{}', Prefix (str): '{}'. Expected: {}, Got: {}",
			path_str, prefix_str, expected_bool, actual_bool_str_prefix
		);
	}

	Ok(())
}

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
		let original_path = SPath::new(data.0);
		let sibling_leaf_path = SPath::new(data.1);
		let expected_path = SPath::new(data.2);

		let actual_path = original_path.new_sibling(sibling_leaf_path);

		assert_eq!(actual_path.as_str(), expected_path.as_str());
	}

	Ok(())
}

#[test]
fn test_spath_replace_prefix_simple() -> Result<()> {
	// -- Setup & Fixtures
	let fx_data = &[
		// (original_path, base_prefix, replacement, expected_path)
		// Basic replacement
		(
			"/data/proj/src/main.rs",
			"/data/proj/",
			"/archive/v1/",
			"/archive/v1/src/main.rs",
		),
		// Base without trailing slash, replacement without trailing slash
		(
			"/data/proj/src/main.rs",
			"/data/proj",
			"/archive/v1",
			"/archive/v1/src/main.rs",
		),
		// Relative paths
		("src/main.rs", "src/", "lib/", "lib/main.rs"),
		// Replacement with trailing slash, base without
		("src/main.rs", "src", "lib/", "lib/main.rs"),
		// Prefix not found
		(
			"/data/proj/src/main.rs",
			"/nonexistent/",
			"/foo/",
			"/data/proj/src/main.rs",
		),
		// Empty base prefix (should prepend replacement and a slash)
		("file.txt", "", "prefix", "prefix/file.txt"),
		// Root base prefix
		("/file.txt", "/", "/new_root", "/new_root/file.txt"),
		// Full path replacement
		(
			"/data/project/file.txt",
			"/data/project/file.txt",
			"/archive/doc.md",
			"/archive/doc.md/",
		),
		// Base longer than path (no replacement)
		("path/to/file", "path/to/file/extra", "new", "path/to/file"),
		// Base is identical to path, replacement is empty
		// ("config.toml", "config.toml", "", ""), // "" + "/" + ""
		// Base is part of path, replacement is empty
		("project/config.toml", "project/", "", "config.toml"),
	];

	// -- Exec & Check
	for &(original_str, base_str, with_str, expected_str) in fx_data.iter() {
		let original_path = SPath::new(original_str);
		let expected_path = SPath::new(expected_str);

		let actual_path = original_path.replace_prefix(base_str, with_str);

		assert_eq!(
			actual_path.as_str(),
			expected_path.as_str(),
			"Failed for: original='{}', base='{}', with='{}'",
			original_str,
			base_str,
			with_str
		);
	}

	Ok(())
}

#[test]
fn test_spath_into_replace_prefix_simple() -> Result<()> {
	// -- Setup & Fixtures
	let fx_data = &[
		// (original_path, base_prefix, replacement, expected_path)
		// Basic replacement
		(
			"/data/proj/src/main.rs",
			"/data/proj/",
			"/archive/v1/",
			"/archive/v1/src/main.rs",
		),
		// Base without trailing slash, replacement without trailing slash
		(
			"/data/proj/src/main.rs",
			"/data/proj",
			"/archive/v1",
			"/archive/v1/src/main.rs",
		),
		// Relative paths
		("src/main.rs", "src/", "lib/", "lib/main.rs"),
		// Replacement with trailing slash, base without
		("src/main.rs", "src", "lib/", "lib/main.rs"),
		// Prefix not found (original path should be returned)
		(
			"/data/proj/src/main.rs",
			"/nonexistent/",
			"/foo/",
			"/data/proj/src/main.rs",
		),
		// Empty base prefix (should prepend replacement and a slash)
		("file.txt", "", "prefix", "prefix/file.txt"),
		// Root base prefix
		("/file.txt", "/", "/new_root", "/new_root/file.txt"),
		// Full path replacement
		(
			"/data/project/file.txt",
			"/data/project/file.txt",
			"/archive/doc.md",
			"/archive/doc.md/",
		),
		// Base longer than path (no replacement)
		("path/to/file", "path/to/file/extra", "new", "path/to/file"),
		// Base is identical to path, replacement is empty
		("config.toml", "config.toml", "", ""),
		// Base is part of path, replacement is empty
		("project/config.toml", "project/", "", "config.toml"),
	];

	// -- Exec & Check
	for &(original_str, base_str, with_str, expected_str) in fx_data.iter() {
		let original_path = SPath::new(original_str);
		let original_path_for_check = SPath::new(original_str); // Clone for comparison if no change
		let expected_path = SPath::new(expected_str);

		let actual_path = original_path.into_replace_prefix(base_str, with_str);

		assert_eq!(
			actual_path.as_str(),
			expected_path.as_str(),
			"Failed for: original='{}', base='{}', with='{}'",
			original_str,
			base_str,
			with_str
		);

		// Check if the original path was indeed consumed or returned if unchanged
		if original_str == expected_str {
			// This check is a bit indirect for "consumed".
			// If no change, the `into_` version might return `self`.
			// We are primarily testing the transformation logic here.
			assert_eq!(actual_path.as_str(), original_path_for_check.as_str());
		}
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
		let base_path = SPath::new(data.0);
		let target_path = SPath::new(data.1);
		let expected_path = SPath::new(data.2);

		let diff_path = target_path.diff(&base_path).ok_or("Should have diff")?;
		let rejoined_path = base_path.join(&diff_path).collapse();

		assert_eq!(diff_path.as_str(), expected_path.as_str());
		assert_eq!(rejoined_path.as_str(), target_path.as_str());
	}

	Ok(())
}
