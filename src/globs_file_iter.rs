use crate::glob::{DEFAULT_EXCLUDE_GLOBS, get_glob_set, longest_base_path_wild_free};
use crate::{ListOptions, Result, SFile, SPath, get_depth};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;

pub struct GlobsFileIter {
	inner: Box<dyn Iterator<Item = SFile>>,
}

impl GlobsFileIter {
	pub fn new(
		dir: impl AsRef<Path>,
		include_globs: Option<&[&str]>,
		list_options: Option<ListOptions<'_>>,
	) -> Result<Self> {
		// main_base for relative globs comes from the directory passed in
		let main_base = SPath::from_std_path(dir.as_ref())?;

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
			(vec!["**"], Vec::new())
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

		// Process the globs into groups: each group is a (base_dir, Vec<relative glob>)
		let groups = process_globs(&main_base, &include_patterns)?;

		// Get the relative_glob setting from list_options
		let use_relative_glob = list_options.as_ref().is_some_and(|o| o.relative_glob);

		// Prepare exclude globs applied uniformly on each group
		let exclude_globs_raw: Option<&[&str]> = list_options.as_ref().and_then(|o| o.exclude_globs());
		let exclude_globs_set = exclude_globs_raw
			.or(Some(DEFAULT_EXCLUDE_GLOBS))
			.map(get_glob_set)
			.transpose()?;

		// For each group, create a WalkDir iterator with its own base and globset
		let mut group_iterators: Vec<Box<dyn Iterator<Item = SFile>>> = Vec::new();

		let max_depth = list_options.and_then(|o| o.depth);

		let exclude_globs_set = Arc::new(exclude_globs_set);
		for (group_base, patterns) in groups.into_iter() {
			// Compute maximum depth among the group's relative glob patterns
			let pats: Vec<&str> = patterns.iter().map(|s| s.as_str()).collect();
			let depth = get_depth(&pats, max_depth);

			// Build the globset for the group from its relative patterns
			let globset = get_glob_set(&pats)?;

			// Clone group_base for use in closures
			let base_clone = group_base.clone();
			let exclude_globs_set_clone = exclude_globs_set.clone();
			let iter = WalkDir::new(group_base.path())
				.max_depth(depth)
				.into_iter()
				.filter_entry(move |e| {
					let Ok(path) = SPath::from_std_path(e.path()) else {
						return false;
					};
					// This uses the walkdir file_type which does not make a system call
					let is_dir = e.file_type().is_dir();

					// here we are filtering only the dir.
					if is_dir {
						// If it's a directory, check the excludes.
						// NOTE 1: It is important not to glob check the includes for directories, as they will always fail.
						if let Some(exclude_globs) = exclude_globs_set_clone.as_ref() {
							// Check with proper path handling based on relative_glob setting
							if use_relative_glob {
								if let Some(rel_path) = path.diff(&base_clone) {
									let stop = exclude_globs.is_match(&rel_path);
									return !stop;
								}
							}
							// Check with absolute path if relative path not available or relative_glob is false
							let stop = exclude_globs.is_match(&path);
							!stop
						} else {
							true
						}
					} else {
						true
					}
				})
				.filter_map(|entry| entry.ok())
				.filter(|entry| entry.file_type().is_file())
				.filter_map(SFile::from_walkdir_entry_ok);

			let exclude_globs_set_clone = exclude_globs_set.clone();
			let main_base_clone = main_base.clone();
			let base_clone = group_base.clone();
			let iter = iter.filter(move |sfile| {
				// First check if the file should be excluded by the exclude_globs
				if let Some(exclude) = exclude_globs_set_clone.as_ref() {
					// Use appropriate path based on relative_glob setting
					if use_relative_glob {
						if let Some(rel_path) = sfile.diff(&main_base_clone) {
							if exclude.is_match(&rel_path) {
								return false;
							}
						}
					} else if exclude.is_match(sfile) {
						return false;
					}
				}

				// Always compute the relative path based on the group base
				let rel_path = match sfile.diff(base_clone.path()) {
					Some(p) => p,
					None => return false,
				};

				// Accept only those files that match the group's globset
				globset.is_match(rel_path)
			});
			group_iterators.push(Box::new(iter));
		}

		// Combine all group iterators into one combined iterator
		let combined_iter = group_iterators.into_iter().fold(
			Box::new(std::iter::empty()) as Box<dyn Iterator<Item = SFile>>,
			|acc, iter| Box::new(acc.chain(iter)) as Box<dyn Iterator<Item = SFile>>,
		);

		// Use scan to keep track of absolute file paths and remove duplicates.
		let dedup_iter = combined_iter
			.scan(HashSet::new(), |seen, file| {
				let path_str = file.to_string();
				if seen.contains(&path_str) {
					Some(None)
				} else {
					seen.insert(path_str);
					Some(Some(file))
				}
			})
			.flatten();

		Ok(GlobsFileIter {
			inner: Box::new(dedup_iter),
		})
	}
}

impl Iterator for GlobsFileIter {
	type Item = SFile;
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

/// Processes the provided globs into groups with collapsed base directories.
/// For relative globs, the pattern is adjusted to be relative to main_base.
fn process_globs(main_base: &SPath, globs: &[&str]) -> Result<Vec<(SPath, Vec<String>)>> {
	let mut groups: Vec<(SPath, Vec<String>)> = Vec::new();
	let mut relative_patterns: Vec<String> = Vec::new();

	for &glob in globs {
		let path = Path::new(glob);
		if path.is_absolute() {
			let base_path = longest_base_path_wild_free(glob);
			let abs_base = SPath::from_std_path(base_path)?;
			let rel_pattern = relative_from_absolute(glob, &abs_base);
			// Add to groups: if exists with same base, push; else create new.
			if let Some((_, patterns)) = groups.iter_mut().find(|(b, _)| b.as_str() == abs_base.as_str()) {
				patterns.push(rel_pattern);
			} else {
				groups.push((abs_base, vec![rel_pattern]));
			}
		} else {
			// Remove any leading "./" from the glob
			let cleaned = glob.trim_start_matches("./").to_string();
			// Collpase the relative glob by stripping the main_base prefix if present.
			let base_candidate: &str = main_base.as_str();
			let base_str_cleaned = {
				let s = base_candidate.trim_start_matches("./");
				if s.is_empty() {
					String::new()
				} else {
					let mut t = s.to_string();
					if !t.ends_with(std::path::MAIN_SEPARATOR) {
						t.push(std::path::MAIN_SEPARATOR);
					}
					t
				}
			};
			if !base_str_cleaned.is_empty() && cleaned.starts_with(&base_str_cleaned) {
				let relative = cleaned[base_str_cleaned.len()..].to_string();
				relative_patterns.push(relative);
			} else {
				relative_patterns.push(cleaned);
			}
		}
	}
	if !relative_patterns.is_empty() {
		groups.push((main_base.clone(), relative_patterns));
	}

	// Merge groups with common base directories.
	// Sort groups by base path length (shorter first).
	groups.sort_by_key(|(base, _)| base.as_str().len());
	let mut final_groups: Vec<(SPath, Vec<String>)> = Vec::new();
	for (base, patterns) in groups {
		let mut merged = false;
		for (existing_base, existing_patterns) in final_groups.iter_mut() {
			if is_prefix(existing_base, &base) {
				// 'base' is a subdirectory of 'existing_base'
				let diff = safe_diff(&base, existing_base);
				for pat in patterns.iter() {
					let new_pat = if diff.is_empty() {
						pat.to_string()
					} else {
						format!("{}/{}", diff, pat)
					};
					existing_patterns.push(new_pat);
				}
				merged = true;
				break;
			} else if is_prefix(&base, existing_base) {
				// 'existing_base' is a subdirectory of 'base'
				let diff = safe_diff(existing_base, &base);
				let mut new_patterns = patterns.clone();
				for pat in existing_patterns.iter() {
					let new_pat = if diff.is_empty() {
						pat.clone()
					} else {
						format!("{}/{}", diff, pat)
					};
					new_patterns.push(new_pat);
				}
				*existing_base = base.clone();
				*existing_patterns = new_patterns;
				merged = true;
				break;
			}
		}
		if !merged {
			final_groups.push((base, patterns));
		}
	}

	Ok(final_groups)
}

/// Helper function to check if 'prefix' is a prefix of 'path'
fn is_prefix(prefix: &SPath, path: &SPath) -> bool {
	let prefix_str = prefix.as_str();
	let path_str = path.as_str();
	if path_str == prefix_str {
		return true;
	}
	if !path_str.starts_with(prefix_str) {
		return false;
	}
	let remainder = &path_str[prefix_str.len()..];
	remainder.starts_with(std::path::MAIN_SEPARATOR)
}

/// Helper function to safely compute the diff, returning an empty string on error.
fn safe_diff(path: &SPath, base: &SPath) -> String {
	path.diff(base.path()).map(|p| p.as_str().to_string()).unwrap_or_default()
}

/// Given an absolute glob pattern and its computed base, returns the relative glob
/// by removing the base prefix and any leading path separator.
fn relative_from_absolute(glob: &str, group_base: &SPath) -> String {
	let base_str = group_base.as_str();
	let rel = glob.strip_prefix(base_str).unwrap_or(glob);
	rel.trim_start_matches(std::path::MAIN_SEPARATOR).to_string()
}
