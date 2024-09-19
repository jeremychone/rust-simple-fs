use crate::glob::{get_glob_set, DEFAULT_EXCLUDE_GLOBS};
use crate::{Result, SFile};
use std::path::Path;
use walkdir::WalkDir;

pub fn iter_files(
	dir: impl AsRef<Path>,
	include_globs: Option<&[&str]>,
	exclude_globs: Option<&[&str]>,
) -> Result<impl Iterator<Item = SFile>> {
	iter_files_impl(dir.as_ref(), include_globs, exclude_globs)
}

pub fn list_files(
	dir: impl AsRef<Path>,
	include_globs: Option<&[&str]>,
	exclude_globs: Option<&[&str]>,
) -> Result<Vec<SFile>> {
	let sfiles_iter = iter_files_impl(dir.as_ref(), include_globs, exclude_globs)?;
	Ok(sfiles_iter.collect())
}

/// The implementation of the iter_files
fn iter_files_impl(
	dir: &Path,
	include_globs: Option<&[&str]>,
	exclude_globs: Option<&[&str]>,
) -> Result<impl Iterator<Item = SFile>> {
	// -- Determine recursive depth
	let depth = include_globs
		.map(|globs| globs.iter().any(|&g| g.contains("**")))
		.map(|v| if v { 100 } else { 1 })
		.unwrap_or(1);

	// -- Prep globs
	let include_globs = include_globs.map(get_glob_set).transpose()?;
	let exclude_globs = exclude_globs.or(Some(DEFAULT_EXCLUDE_GLOBS)).map(get_glob_set).transpose()?;

	// -- Build file iterator
	let walk_dir_it = WalkDir::new(dir)
		.max_depth(depth)
		.into_iter()
		.filter_entry(move |e|
			// If dir, check the exclude.
			// Note: Important not to check the includes for directories, as they will always fail.
			if e.file_type().is_dir() {
				if let Some(exclude_globs) = exclude_globs.as_ref() {
					!exclude_globs.is_match(e.path())
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
					Some(globs) => globs.is_match(e.path()),
					None => true,
				}
			}
		)
		.filter_map(|e| e.ok().filter(|e| e.file_type().is_file()));

	let sfile_iter = walk_dir_it.filter_map(SFile::from_walkdir_entry_ok);

	Ok(sfile_iter)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	type Error = Box<dyn std::error::Error>;
	type Result<T> = core::result::Result<T, Error>;

	#[test]
	fn test_iter_files_simple_ok() -> Result<()> {
		let excludes = [
			//
			DEFAULT_EXCLUDE_GLOBS,
			&["*.lock", "**/w*.rs"],
		]
		.concat();

		// FIXME: See to have a bug here if `**/s*.rs`, all `src/...` will be excluded

		// TODO: Implement more complete tests.

		let iter = iter_files("./", Some(&["./src/**/*.rs"]), Some(&excludes))?;
		let count = iter.count();
		// Very trivial check.
		assert_eq!(count, 9);

		Ok(())
	}
}

// endregion: --- Tests
