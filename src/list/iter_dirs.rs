use crate::{ListOptions, Result, SPath};

use std::path::Path;

/// Returns an iterator over directories in the specified `dir` filtered optionally by `include_globs`
/// and `list_options`. This implementation uses the internal GlobsDirIter.
pub fn iter_dirs(
	dir: impl AsRef<Path>,
	include_globs: Option<&[&str]>,
	list_options: Option<ListOptions<'_>>,
) -> Result<impl Iterator<Item = SPath>> {
	let iter = super::globs_dir_iter::GlobsDirIter::new(dir, include_globs, list_options)?;
	Ok(iter)
}

/// Collects directories from `iter_dirs` into a Vec<SPath>
pub fn list_dirs(
	dir: impl AsRef<Path>,
	include_globs: Option<&[&str]>,
	list_options: Option<ListOptions<'_>>,
) -> Result<Vec<SPath>> {
	let iter = iter_dirs(dir, include_globs, list_options)?;
	Ok(iter.collect())
}
