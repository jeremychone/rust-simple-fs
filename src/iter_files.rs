use crate::glob::{get_glob_set, DEFAULT_EXCLUDE_GLOBS};
use crate::{ListOptions, Result, SFile, SPath};
use std::path::Path;
use std::sync::Arc;
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

/// The implementation of the iter_files function.
fn iter_files_impl(
	dir: &Path,
	include_globs: Option<&[&str]>,
	list_options: Option<ListOptions<'_>>,
) -> Result<impl Iterator<Item = SFile>> {
	let ref_dir = Arc::new(SPath::from_path(dir)?);

	// -- Determine recursive depth
	let depth = include_globs.as_ref().map_or(1, |globs| get_depth(globs));

	// -- Prepare globs
	// will default to false
	let dir_relative = list_options.as_ref().map(|l| l.relative_glob).unwrap_or_default();
	let include_globs = include_globs.map(get_glob_set).transpose()?;

	let exclude_globs: Option<&[&str]> = list_options
		.as_ref() // Borrow list_options to ensure it remains valid
		.and_then(|o| o.exclude_globs()); // No flatten needed as it's Option<&[&str]>

	let exclude_globs = exclude_globs
		.or(Some(DEFAULT_EXCLUDE_GLOBS)) // Use the static reference, no allocation
		.map(get_glob_set) // Pass directly as &[&str]
		.transpose()?;

	// -- Build file iterator
	let dir = ref_dir.clone();
	let walk_dir_it = WalkDir::new(&*dir)
		.max_depth(depth)
		.into_iter()
		.filter_entry(move |e| {
			// This uses the walkdir file_type which does not make a system call
			let is_dir = e.file_type().is_dir();

			let Ok(path) = SPath::from_path(e.path()) else {
				return false;
			};

			let path = if dir_relative {
				let Ok(path) = path.diff(&*dir) else {
					return false;
				};
				path
			} else {
				path
			};

			// If it's a directory, check the excludes.

			// NOTE 1: It is important not to glob check the includes for directories, as they will always fail.
			if is_dir {
				if let Some(exclude_globs) = exclude_globs.as_ref() {
					!exclude_globs.is_match(path)
				} else {
					true
				}
			}
			// Else, for files, we apply the globs.
			else {
				// First, evaluate the excludes.
				if let Some(exclude_globs) = exclude_globs.as_ref() {
					if exclude_globs.is_match(&path) {
						return false;
					}
				}

				// Then, evaluate the includes.
				let res = match include_globs.as_ref() {
					Some(globs) => globs.is_match(&path),
					None => true,
				};
				res
			}
		})
		// only take ok entry that are files
		.filter_map(|e| e.ok().filter(|e| e.file_type().is_file()));

	// Now, the final iteration
	// TODO: Here we could do an optimization if SFile would allow a backdoor for setting the path,
	//       as we know it is a file.
	// IMPORTANT: Here we will have the path including the "dir", meaning relative to current_dir, so that it can be loaded.
	//            Otherwise, it would not be a valid path to load.
	let walk_dir_it = walk_dir_it.filter_map(SFile::from_walkdir_entry_ok);

	Ok(walk_dir_it)
}

// region:    --- Support

/// Determine the depth of the walk from the globs.
///
/// Rules:
/// 1. If any glob contains "**", the depth is set to 100.
/// 2. Otherwise, the depth is determined by the maximum number of path separators in the globs.
///
/// Note: It might not be perfect, but we will fine-tune it later.
fn get_depth(include_globs: &[&str]) -> usize {
	let depth = include_globs.iter().fold(1, |acc, &g| {
		if g.contains("**") {
			return 100;
		}
		let sep_count = g.matches(std::path::MAIN_SEPARATOR).count() + 1;
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
		// TODO: Implement more comprehensive tests.

		// -- Execute
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

		// FIXME: There seems to be a bug here if `**/s*.rs`, all `src/...` will be excluded.

		// TODO: Implement more comprehensive tests.

		let iter = iter_files("./", Some(&["./src/**/*.rs"]), Some(excludes.into()))?;
		let count = iter.count();
		// Very trivial check.
		assert_eq!(count, 10);

		Ok(())
	}
}

// endregion: --- Tests
