use simple_fs::{ListOptions, SFile, SPath, iter_files, list_files};

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_list_files_one_level_dotted() -> Result<()> {
	// -- Exec
	let res = list_files("./tests-data/", Some(&["./tests-data/*.txt"]), None)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert_eq!(res.len(), 1, "Should have 1 file with *.txt in tests-data");
	assert!(
		res_paths.contains(&"./tests-data/file2.txt"),
		"Should contain file2.txt"
	);
	assert!(
		res_paths.iter().any(|p| p.ends_with("file2.txt")),
		"Should contain file2.txt"
	);

	Ok(())
}

#[test]
fn test_list_files_rel_one_level_dotted() -> Result<()> {
	// NOTE With relative_glob, "*.txt" now works
	// -- Exec
	let res = list_files(
		"./tests-data/",
		Some(&["*.txt"]),
		Some(ListOptions::from_relative_glob(true)),
	)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert_eq!(res.len(), 1, "Should have 1 file with *.txt in tests-data");
	assert!(
		res_paths.iter().any(|p| p.ends_with("file2.txt")),
		"Should contain file2.txt"
	);

	Ok(())
}

#[test]
fn test_list_files_rel_one_level_no_file() -> Result<()> {
	// -- Exec
	let res = list_files(
		"./tests-data/",
		Some(&["*.rs"]),
		Some(ListOptions::from_relative_glob(true)),
	)?;

	// -- Check
	assert_eq!(res.len(), 0, "Should have 0 files with *.rs in tests-data dir");

	Ok(())
}

#[test]
fn test_list_files_one_level_no_file() -> Result<()> {
	// -- Exec
	let res = list_files("./tests-data/", Some(&["./tests-data/*.rs"]), None)?;

	// -- Check
	assert_eq!(res.len(), 0, "Should have 0 files with *.rs in tests-data dir");

	Ok(())
}

#[test]
fn test_list_files_one_file_dotted() -> Result<()> {
	// -- Exec
	let res = list_files("./tests-data", Some(&["./tests-data/file1.md"]), None)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert_eq!(res.len(), 1, "Should have 1 file");
	assert!(res_paths.contains(&"./tests-data/file1.md"), "Should contain file1.md");

	Ok(())
}

#[test]
fn test_list_files_sub_level_dotted() -> Result<()> {
	// -- Exec
	let res = list_files("./tests-data/", Some(&["./tests-data/**/*.md"]), None)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert_md_files_res(&res_paths);

	Ok(())
}

#[test]
fn test_list_files_sub_dir_full_path() -> Result<()> {
	// -- Exec
	let res = list_files("./tests-data/dir1/", Some(&["./tests-data/dir1/**/*.md"]), None)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert_eq!(res_paths.len(), 3, "Should have 3 markdown files in dir1");
	assert!(
		res_paths.contains(&"./tests-data/dir1/file3.md"),
		"Should contain dir1/file3.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/dir1/dir2/file5.md"),
		"Should contain dir1/dir2/file5.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/dir1/dir2/dir3/file7.md"),
		"Should contain dir1/dir2/dir3/file7.md"
	);

	Ok(())
}

/// Here the globs are relative to the base dir given (here `./tests-data/`)
#[test]
fn test_list_files_sub_dir_rel_glob() -> Result<()> {
	// -- Exec
	let res = list_files(
		"./tests-data/",
		Some(&["**/*.md"]),
		Some(ListOptions::from_relative_glob(true)),
	)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert_md_files_res(&res_paths);

	Ok(())
}

#[test]
fn test_list_files_absolute_wildcard() -> Result<()> {
	// Get the absolute path to the "tests-data" directory.
	let test_data_abs = SPath::new("./tests-data");
	let test_data_abs_str = test_data_abs.as_str();

	// Construct a glob pattern that should match the "file1.md" file.
	let pattern = format!("{test_data_abs_str}/**/*1.md");

	// -- Exec
	// Execute list_files using the tests-data directory and the wildcard pattern.
	let files = list_files("./tests-data/", Some(&[pattern.as_str()]), None)?;

	// -- Check
	// Check that at least one file's path ends with "file1.md"
	let found = files.iter().any(|p| p.as_str().ends_with("file1.md"));
	assert!(found, "Expected to find file1.md file with wildcard absolute pattern");

	Ok(())
}

#[test]
fn test_list_files_absolute_direct() -> Result<()> {
	// Get the absolute path to "tests-data/file1.md".
	let file_abs = std::fs::canonicalize("tests-data/file1.md")?;
	let file_abs = SPath::from_std_path_buf(file_abs)?;

	// Get the parent directory of the file.
	let parent_dir = file_abs.parent().ok_or("Should have parent dir")?;

	// -- Exec
	// Execute list_files using the parent directory and an exact match glob for the file.
	let files = list_files(parent_dir, Some(&[file_abs.as_str()]), None)?;

	// -- Check
	assert_eq!(files.len(), 1, "Should have exactly one file with exact match");

	let returned_path = files[0].as_str();
	assert_eq!(
		returned_path,
		file_abs.as_str(),
		"The file path should match the absolute file path"
	);

	Ok(())
}

#[test]
fn test_list_files_mixed_absolute_and_relative_globs() -> Result<()> {
	// -- Exec
	// Mix an absolute glob and a relative glob in the same call.
	let abs_pattern = SPath::new("./tests-data/file1.md").canonicalize()?;
	let patterns = [abs_pattern.as_str(), "tests-data/file2.txt"];
	let res = list_files("./", Some(&patterns), None)?;

	// -- Check
	let res_paths: Vec<&str> = res.iter().map(|p| p.as_str()).collect();

	assert_eq!(res.len(), 2, "Expected both files to be found using mixed patterns");
	assert!(
		res_paths.iter().any(|&p| p.ends_with("file1.md")),
		"Should contain file1.md"
	);
	assert!(
		res_paths.iter().any(|&p| p.ends_with("file2.txt")),
		"Should contain file2.txt"
	);
	Ok(())
}

#[test]
fn test_list_files_mixed_absolute_and_relative_globs_with_relative_option() -> Result<()> {
	// -- Exec
	// Mix an absolute glob and a relative glob with the relative_glob option enabled.
	let abs_pattern = SPath::new("./tests-data/file1.md");
	let patterns = ["**/*.txt", abs_pattern.as_str()];
	let res = list_files(
		"./tests-data/",
		Some(&patterns),
		Some(ListOptions::from_relative_glob(true)),
	)?;

	// -- Check
	let res_paths: Vec<&str> = res.iter().map(|p| p.as_str()).collect();
	assert!(
		res.len() >= 6,
		"Expected at least 6 files to be found using mixed patterns with relative_glob option"
	);
	assert!(
		res_paths.iter().any(|&p| p.ends_with("file1.md")),
		"Should contain file1.md"
	);
	assert!(
		res_paths.iter().any(|&p| p.ends_with("file2.txt")),
		"Should contain file2.txt"
	);
	Ok(())
}

#[test]
fn test_list_iter_files_simple_glob_ok() -> Result<()> {
	// -- Exec
	let iter = iter_files("./tests-data/", Some(&["./tests-data/*.md"]), None)?;
	let res: Vec<SFile> = iter.collect();

	// -- Check
	let count = res.len();
	assert_eq!(count, 1, "Expected 1 file matching pattern");
	Ok(())
}

#[test]
fn test_list_iter_files_nested_and_exclude_ok() -> Result<()> {
	// -- Exec
	let excludes = [simple_fs::DEFAULT_EXCLUDE_GLOBS, &["**/.devai", "*.lock", "**/dir2/**"]].concat();
	let iter = iter_files("./tests-data/", Some(&["./tests-data/**/*.md"]), Some(excludes.into()))?;

	// -- Check
	let count = iter.count();
	assert_eq!(count, 5, "Expected 5 files matching pattern after exclusions");
	Ok(())
}

#[test]
fn test_list_files_with_negative_glob() -> Result<()> {
	// -- Exec
	// Include all markdown files but exclude those in dir2
	let res = list_files(
		"./tests-data/",
		Some(&["./tests-data/**/*.md", "!./tests-data/**/dir2/**"]),
		None,
	)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert_eq!(res.len(), 5, "Should have 5 markdown files (excluding dir2)");

	assert!(res_paths.contains(&"./tests-data/file1.md"), "Should contain file1.md");
	assert!(
		res_paths.contains(&"./tests-data/dir1/file3.md"),
		"Should contain dir1/file3.md"
	);
	assert!(
		!res_paths.contains(&"./tests-data/dir1/dir2/file5.md"),
		"Should not contain dir1/dir2/file5.md"
	);
	assert!(
		!res_paths.contains(&"./tests-data/dir1/dir2/dir3/file7.md"),
		"Should not contain dir1/dir2/dir3/file7.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/another-dir/notes.md"),
		"Should contain another-dir/notes.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/another-dir/sub-dir/example.md"),
		"Should contain another-dir/sub-dir/example.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/another-dir/sub-dir/deep-folder/final.md"),
		"Should contain another-dir/sub-dir/deep-folder/final.md"
	);

	Ok(())
}

#[test]
fn test_list_files_with_multiple_negative_globs() -> Result<()> {
	// -- Exec
	// Include all markdown files but exclude multiple patterns
	let res = list_files(
		"./tests-data/",
		Some(&[
			"./tests-data/**/*.md",       // Include all markdown files
			"!./tests-data/**/dir2/**",   // Exclude dir2 files
			"!./tests-data/**/*final.md", // Exclude final.md files
		]),
		None,
	)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert_eq!(res.len(), 4, "Should have 4 markdown files after multiple exclusions");

	assert!(res_paths.contains(&"./tests-data/file1.md"), "Should contain file1.md");
	assert!(
		res_paths.contains(&"./tests-data/dir1/file3.md"),
		"Should contain dir1/file3.md"
	);
	assert!(
		!res_paths.contains(&"./tests-data/dir1/dir2/file5.md"),
		"Should not contain dir1/dir2/file5.md"
	);
	assert!(
		!res_paths.contains(&"./tests-data/dir1/dir2/dir3/file7.md"),
		"Should not contain dir1/dir2/dir3/file7.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/another-dir/notes.md"),
		"Should contain another-dir/notes.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/another-dir/sub-dir/example.md"),
		"Should contain another-dir/sub-dir/example.md"
	);
	assert!(
		!res_paths.contains(&"./tests-data/another-dir/sub-dir/deep-folder/final.md"),
		"Should not contain another-dir/sub-dir/deep-folder/final.md"
	);

	Ok(())
}

#[test]
fn test_list_files_with_only_negative_globs() -> Result<()> {
	// -- Exec
	// Only use negative patterns (should default to ** for includes)
	let res = list_files(
		"./tests-data/",
		Some(&[
			"!./tests-data/**/*.txt", // Exclude all txt files
		]),
		None,
	)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert!(res.len() >= 7, "Should have at least 7 files after excluding txt files");

	// Ensure no txt files are included
	assert!(
		!res_paths.iter().any(|p| p.ends_with(".txt")),
		"Should not contain any .txt files"
	);

	// Check for md files as a sanity check
	assert!(res_paths.iter().any(|p| p.ends_with(".md")), "Should contain .md files");

	Ok(())
}

#[test]
fn test_list_files_relative_negative_glob() -> Result<()> {
	// -- Exec
	// Use relative globs with negative patterns
	let res = list_files(
		"./tests-data/",
		Some(&[
			"**/*.md",     // Include all markdown files
			"!**/dir2/**", // Exclude dir2 files (using relative glob)
		]),
		Some(ListOptions::from_relative_glob(true)),
	)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert_eq!(res.len(), 5, "Should have 5 markdown files (excluding dir2)");

	assert!(res_paths.contains(&"./tests-data/file1.md"), "Should contain file1.md");
	assert!(
		res_paths.contains(&"./tests-data/dir1/file3.md"),
		"Should contain dir1/file3.md"
	);
	assert!(
		!res_paths.contains(&"./tests-data/dir1/dir2/file5.md"),
		"Should not contain dir1/dir2/file5.md"
	);
	assert!(
		!res_paths.contains(&"./tests-data/dir1/dir2/dir3/file7.md"),
		"Should not contain dir1/dir2/dir3/file7.md"
	);

	Ok(())
}

#[test]
fn test_list_files_with_combined_exclusion_methods() -> Result<()> {
	// -- Exec
	// Combine both ListOptions exclude_globs and negative patterns in include_globs
	let list_options = ListOptions::default()
		.with_exclude_globs(&["**/deep-folder/**"]) // Exclude deep-folder files via ListOptions
		.with_relative_glob(); // Use relative glob mode

	let res = list_files(
		"./tests-data/",
		Some(&[
			"**/*.md",     // Include all markdown files
			"!**/dir2/**", // Exclude dir2 files via negative pattern
		]),
		Some(list_options),
	)?;

	// -- Check
	let res_paths = res.iter().map(|p| p.as_str()).collect::<Vec<_>>();
	assert_eq!(res.len(), 4, "Should have 4 markdown files after combined exclusions");

	// Files that should be included
	assert!(res_paths.contains(&"./tests-data/file1.md"), "Should contain file1.md");
	assert!(
		res_paths.contains(&"./tests-data/dir1/file3.md"),
		"Should contain dir1/file3.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/another-dir/notes.md"),
		"Should contain another-dir/notes.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/another-dir/sub-dir/example.md"),
		"Should contain another-dir/sub-dir/example.md"
	);

	// Files that should be excluded by negative pattern
	assert!(
		!res_paths.contains(&"./tests-data/dir1/dir2/file5.md"),
		"Should not contain dir1/dir2/file5.md (excluded by negative pattern)"
	);
	assert!(
		!res_paths.contains(&"./tests-data/dir1/dir2/dir3/file7.md"),
		"Should not contain dir1/dir2/dir3/file7.md (excluded by negative pattern)"
	);

	// Files that should be excluded by ListOptions
	assert!(
		!res_paths.contains(&"./tests-data/another-dir/sub-dir/deep-folder/final.md"),
		"Should not contain another-dir/sub-dir/deep-folder/final.md (excluded by ListOptions)"
	);

	Ok(())
}

// region:    --- Support

/// Reusable function for checking markdown files in test-data directory
fn assert_md_files_res(res_paths: &[&str]) {
	assert_eq!(res_paths.len(), 7, "Should have 7 markdown files in total");
	assert!(res_paths.contains(&"./tests-data/file1.md"), "Should contain file1.md");
	assert!(
		res_paths.contains(&"./tests-data/dir1/file3.md"),
		"Should contain dir1/file3.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/dir1/dir2/file5.md"),
		"Should contain dir1/dir2/file5.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/dir1/dir2/dir3/file7.md"),
		"Should contain dir1/dir2/dir3/file7.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/another-dir/notes.md"),
		"Should contain another-dir/notes.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/another-dir/sub-dir/deep-folder/final.md"),
		"Should contain another-dir/sub-dir/deep-folder/final.md"
	);
	assert!(
		res_paths.contains(&"./tests-data/another-dir/sub-dir/example.md"),
		"Should contain another-dir/sub-dir/example.md"
	);
}

// endregion: --- Support
