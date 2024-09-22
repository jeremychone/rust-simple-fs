use crate::glob::{get_glob_set, DEFAULT_EXCLUDE_GLOBS};
use crate::{ListOptions, Result, SFile};
use std::path::Path;
use walkdir::WalkDir;

pub fn iter_files(
	dir: impl AsRef<Path>,
	include_globs: Option<&[&str]>,
	list_options: Option<ListOptions<'_>>,
) -> Result<impl Iterator<Item = SFile>> {
	iter_files_impl(dir.as_ref(), include_globs, list_options)
}

pub fn list_files(
	dir: impl AsRef<Path>,
	include_globs: Option<&[&str]>,
	list_options: Option<ListOptions<'_>>,
) -> Result<Vec<SFile>> {
	let sfiles_iter = iter_files_impl(dir.as_ref(), include_globs, list_options)?;
	Ok(sfiles_iter.collect())
}

/// The implementation of the iter_files
fn iter_files_impl(
	dir: &Path,
	include_globs: Option<&[&str]>,
	list_options: Option<ListOptions<'_>>,
) -> Result<impl Iterator<Item = SFile>> {
	// -- Determine recursive depth

	let depth = include_globs.as_ref().map_or(1, |globs| get_depth(globs));

	// -- Prep globs
	let include_globs = include_globs.map(get_glob_set).transpose()?;
	let exclude_globs: Option<&[&str]> = list_options
		.as_ref() // Borrow list_options to ensure it stays valid
		.and_then(|o| o.exclude_globs()); // No flatten needed as it's Option<&[&str]>

	let exclude_globs = exclude_globs
		.or(Some(DEFAULT_EXCLUDE_GLOBS)) // Use the static reference, no allocation
		.map(get_glob_set) // Pass directly as &[&str]
		.transpose()?;

	// -- Build file iterator
	let walk_dir_it = WalkDir::new(dir)
		.max_depth(depth)
		.into_iter()
		.filter_entry(move |e|
			// If dir, check the exclude.
			// Note: It is important not to check the includes for directories, as they will always fail.
			if e.file_type().is_dir() {
				if let Some(exclude_globs) = exclude_globs.as_ref() {
					let do_not_exclude = !exclude_globs.is_match(e.path());
					do_not_exclude
				} else {
					true
				}
			}
			// Else file, we apply the globs.
			else {
				// First, evaluate the exclude.
				if let Some(exclude_globs) = exclude_globs.as_ref() {
					if exclude_globs.is_match(e.path()) {
						return false;
					}
				}
				// And then, evaluate the include.
				match include_globs.as_ref() {
					Some(globs) => {
						let does_match = globs.is_match(e.path());
						does_match
					},
					None => true,
				}
			}
		)
		.filter_map(|e| e.ok().filter(|e| e.file_type().is_file()));

	let sfile_iter = walk_dir_it.filter_map(SFile::from_walkdir_entry_ok);

	Ok(sfile_iter)
}

// region:    --- Support

/// Determine the depth of the walk from the globs.
///
/// Rules:
/// 1. If any glob contains "**", the depth is set to 100.
/// 2. Otherwise, the depth is determined by the maximum number of path separators in the globs.
///
/// Note: It might not be perfect, but will fine-tune later.
fn get_depth(include_globs: &[&str]) -> usize {
	let depth = include_globs.iter().fold(0, |acc, &g| {
		if g.contains("**") {
			return 100;
		}
		let sep_count = g.matches(std::path::MAIN_SEPARATOR).count();
		if sep_count > acc {
			sep_count
		} else {
			acc
		}
	});
	depth.max(1)
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	type Error = Box<dyn std::error::Error>;
	type Result<T> = core::result::Result<T, Error>;

	#[test]
	fn test_iter_files_simple_glob_ok() -> Result<()> {
		// TODO: Implement more complete tests.

		// -- Exec
		let iter = iter_files("./", Some(&["./src/s*.rs"]), None)?;

		// -- Check
		let count = iter.count();
		assert_eq!(count, 2);

		Ok(())
	}

	#[test]
	fn test_iter_files_nested_and_exclude_ok() -> Result<()> {
		let excludes = [
			//
			DEFAULT_EXCLUDE_GLOBS,
			&["*.lock", "**/w*.rs"],
		]
		.concat();

		// FIXME: There seems to be a bug here if `**/s*.rs`, all `src/...` will be excluded

		// TODO: Implement more complete tests.

		let iter = iter_files("./", Some(&["./src/**/*.rs"]), Some(excludes.into()))?;
		let count = iter.count();
		// Very trivial check.
		assert_eq!(count, 10);

		Ok(())
	}
}

// endregion: --- Tests
