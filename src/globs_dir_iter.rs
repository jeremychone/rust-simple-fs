use crate::{Error, ListOptions, Result, SPath};
use globset::{Glob, GlobSetBuilder};
use std::path::Path;
use walkdir::WalkDir;

pub struct GlobsDirIter {
	inner: walkdir::IntoIter,
	globset: Option<globset::GlobSet>,
}

impl GlobsDirIter {
	/// Create a new GlobsDirIter for directories.
	///
	/// - `dir`: the starting directory.
	/// - `include_globs`: optional slice of glob patterns. If provided, only directories whose
	///    full path matches at least one pattern will be returned.
	/// - `list_options`: optional list options, e.g., limiting recursion depth.
	///
	/// Returns a Result with GlobsDirIter or an appropriate Error.
	pub fn new(
		dir: impl AsRef<Path>,
		include_globs: Option<&[&str]>,
		list_options: Option<ListOptions<'_>>,
	) -> Result<Self> {
		// Build the GlobSet from provided patterns if any.
		let globset = if let Some(globs) = include_globs {
			let mut builder = GlobSetBuilder::new();
			for pattern in globs {
				builder.add(Glob::new(pattern).map_err(|e| Error::GlobCantNew {
					glob: pattern.to_string(),
					cause: e,
				})?);
			}
			Some(builder.build().map_err(|e| Error::GlobSetCantBuild {
				globs: globs.iter().map(|s| s.to_string()).collect(),
				cause: e,
			})?)
		} else {
			None
		};

		// Create the walkdir iterator.
		let walker = WalkDir::new(dir.as_ref());
		let walker = if let Some(opts) = list_options {
			// If list_options defines a maximum depth, use it.
			if let Some(depth) = opts.depth {
				walker.max_depth(depth)
			} else {
				walker
			}
		} else {
			walker
		};

		Ok(Self {
			inner: walker.into_iter(),
			globset,
		})
	}
}

impl Iterator for GlobsDirIter {
	type Item = SPath;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.by_ref().find_map(|entry_result| {
			let entry = entry_result.ok()?;
			if !entry.file_type().is_dir() {
				return None;
			}
			if !self.globset.as_ref().is_none_or(|globset| globset.is_match(entry.path())) {
				return None;
			}
			SPath::from_path_ok(entry.path())
		})
	}
}
