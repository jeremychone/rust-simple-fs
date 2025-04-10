use crate::{Error, ListOptions, Result, SPath};
use globset::{Glob, GlobSetBuilder};
use std::path::Path;
use walkdir::WalkDir;

pub struct GlobsDirIter {
	inner: Box<dyn Iterator<Item = SPath>>,
}

impl GlobsDirIter {
	/// Create a new GlobsDirIter for directories.
	///
	/// - `dir`: the starting directory.
	/// - `include_globs`: optional slice of glob patterns. If provided, only directories whose
	///   full path matches at least one pattern will be returned. Patterns starting with `!`
	///   are treated as exclusion patterns.
	/// - `list_options`: optional list options, e.g., limiting recursion depth.
	///
	/// Returns a Result with GlobsDirIter or an appropriate Error.
	pub fn new(
		dir: impl AsRef<Path>,
		include_globs: Option<&[&str]>,
		list_options: Option<ListOptions<'_>>,
	) -> Result<Self> {
		let base_dir = SPath::from_std_path(dir.as_ref())?;

		// Process include_globs to separate includes and negated excludes (starting with !)
		let (include_patterns, negated_excludes) = if let Some(globs) = include_globs {
			let mut includes = Vec::new();
			let mut excludes = Vec::new();

			for &pattern in globs {
				if let Some(negative_pattern) = pattern.strip_prefix("!") {
					excludes.push(negative_pattern);
				} else {
					includes.push(pattern);
				}
			}

			// If all patterns were negated, use a default include pattern
			if includes.is_empty() && !excludes.is_empty() {
				(vec!["**"], excludes)
			} else {
				(includes, excludes)
			}
		} else {
			(vec![], Vec::new())
		};

		// Create or extend the ListOptions with negated_excludes
		let list_options = if !negated_excludes.is_empty() {
			match list_options {
				Some(opts) => {
					let mut new_opts = ListOptions {
						exclude_globs: opts.exclude_globs.clone(),
						relative_glob: opts.relative_glob,
						depth: opts.depth,
					};

					if let Some(existing_excludes) = &mut new_opts.exclude_globs {
						// Append negated excludes to existing excludes
						let mut combined = existing_excludes.clone();
						combined.extend(negated_excludes);
						new_opts.exclude_globs = Some(combined);
					} else {
						// Create new excludes from negated patterns
						new_opts.exclude_globs = Some(negated_excludes);
					}

					Some(new_opts)
				}
				None => {
					// Create a new ListOptions with just the negated excludes
					Some(ListOptions {
						exclude_globs: Some(negated_excludes),
						relative_glob: false,
						depth: None,
					})
				}
			}
		} else {
			list_options
		};

		// Build the include GlobSet from provided patterns if any
		let include_globset = if !include_patterns.is_empty() {
			let mut builder = GlobSetBuilder::new();
			for pattern in include_patterns.iter() {
				builder.add(Glob::new(pattern).map_err(|e| Error::GlobCantNew {
					glob: pattern.to_string(),
					cause: e,
				})?);
			}
			Some(builder.build().map_err(|e| Error::GlobSetCantBuild {
				globs: include_patterns.iter().map(|&s| s.to_string()).collect(),
				cause: e,
			})?)
		} else {
			None
		};

		// Extract exclude patterns from list_options if present
		let exclude_globset = if let Some(opts) = &list_options {
			if let Some(exclude_globs) = opts.exclude_globs() {
				let mut builder = GlobSetBuilder::new();
				for pattern in exclude_globs {
					builder.add(Glob::new(pattern).map_err(|e| Error::GlobCantNew {
						glob: pattern.to_string(),
						cause: e,
					})?);
				}
				Some(builder.build().map_err(|e| Error::GlobSetCantBuild {
					globs: exclude_globs.iter().map(|s| s.to_string()).collect(),
					cause: e,
				})?)
			} else {
				None
			}
		} else {
			None
		};

		// Determine whether to use relative globs
		let use_relative_glob = list_options.as_ref().is_some_and(|o| o.relative_glob);

		// Determine the maximum depth
		let depth = list_options.as_ref().and_then(|o| o.depth);

		// Create the walkdir iterator
		let walker = WalkDir::new(base_dir.path());
		let walker = if let Some(depth) = depth {
			walker.max_depth(depth)
		} else {
			walker
		};

		// Build the final iterator
		let iter = walker
			.into_iter()
			.filter_map(|entry_result| entry_result.ok())
			.filter(|entry| entry.file_type().is_dir())
			.filter_map(|entry| SPath::from_std_path_ok(entry.path()))
			.filter(move |path| {
				// Skip paths that match exclude patterns
				if let Some(ref exclude_set) = exclude_globset {
					// Handle relative or absolute paths for exclude patterns
					if use_relative_glob {
						if let Some(rel_path) = path.diff(&base_dir) {
							if exclude_set.is_match(rel_path) {
								return false;
							}
						}
					} else if exclude_set.is_match(path) {
						return false;
					}
				}

				// Only include paths that match include patterns (if specified)
				if let Some(ref include_set) = include_globset {
					// Handle relative or absolute paths for include patterns
					if use_relative_glob {
						if let Some(rel_path) = path.diff(&base_dir) {
							include_set.is_match(rel_path)
						} else {
							false
						}
					} else {
						include_set.is_match(path)
					}
				} else {
					true // No include patterns specified, include all paths
				}
			});

		Ok(Self { inner: Box::new(iter) })
	}
}

impl Iterator for GlobsDirIter {
	type Item = SPath;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}
