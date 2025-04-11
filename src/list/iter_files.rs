use crate::{ListOptions, Result, SFile};
use std::path::Path;

pub fn iter_files(
	dir: impl AsRef<Path>,
	include_globs: Option<&[&str]>,
	list_options: Option<ListOptions<'_>>,
) -> Result<super::globs_file_iter::GlobsFileIter> {
	super::globs_file_iter::GlobsFileIter::new(dir, include_globs, list_options)
}

pub fn list_files(
	dir: impl AsRef<Path>,
	include_globs: Option<&[&str]>,
	list_options: Option<ListOptions<'_>>,
) -> Result<Vec<SFile>> {
	let sfiles_iter = iter_files(dir, include_globs, list_options)?;
	Ok(sfiles_iter.collect())
}
