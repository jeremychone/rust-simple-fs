use super::glob::{DEFAULT_EXCLUDE_GLOBS, get_glob_set, longest_base_path_wild_free};
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
		for GlobGroup {
			base: group_base,
			patterns,
			prefixes,
		} in groups.into_iter()
		{
			// Compute maximum depth among the group's relative glob patterns
			let pats: Vec<&str> = patterns.iter().map(|s| s.as_str()).collect();
			let depth = get_depth(&pats, max_depth);

			// Build the globset for the group from its relative patterns
			let globset = get_glob_set(&pats)?;

			let allowed_prefixes = Arc::new(prefixes);

			// Clone group_base for use in closures
			let base_clone_for_dirs = group_base.clone();
			let exclude_globs_set_clone = exclude_globs_set.clone();
			let allowed_prefixes_for_dirs = allowed_prefixes.clone();
			let iter = WalkDir::new(group_base.path())
				.max_depth(depth)
				.into_iter()
				.filter_entry(move |e| {
					let Ok(path) = SPath::from_std_path(e.path()) else {
						return false;
					};

					// This uses the walkdir file_type which does not make a system call
					let is_dir = e.file_type().is_dir();

					if is_dir {
						if let Some(exclude_globs) = exclude_globs_set_clone.as_ref() {
							if use_relative_glob {
								if let Some(rel_path) = path.diff(&base_clone_for_dirs)
									&& exclude_globs.is_match(&rel_path)
								{
									return false;
								}
							} else if exclude_globs.is_match(&path) {
								return false;
							}
						}

						if !allowed_prefixes_for_dirs.is_empty()
							&& !directory_matches_allowed_prefixes(
								&path,
								&base_clone_for_dirs,
								allowed_prefixes_for_dirs.as_ref(),
							) {
							return false;
						}
					}

					true
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
						if let Some(rel_path) = sfile.diff(&main_base_clone)
							&& exclude.is_match(&rel_path)
						{
							return false;
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
			.scan(HashSet::<SPath>::new(), |seen, file| {
				let path = file.path().clone();
				if seen.insert(path) {
					Some(Some(file))
				} else {
					Some(None)
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

struct GlobGroup {
	base: SPath,
	patterns: Vec<String>,
	prefixes: Vec<String>,
}

// region:    --- Support

/// Processes the provided globs into groups with collapsed base directories.
/// For relative globs, the pattern is adjusted to be relative to main_base.
/// Groups glob patterns by their longest shared base directory.
///
/// # Example
///
/// ```text
/// inputs: main_base="/project", globs=["/project/src/**/*.rs", "*.md"]
/// output: [GlobGroup { base="/project/src", patterns=["**/*.rs"], .. }, GlobGroup { base="/project", patterns=["*.md"], .. }]
/// ```
fn process_globs(main_base: &SPath, globs: &[&str]) -> Result<Vec<GlobGroup>> {
	let mut groups: Vec<(SPath, Vec<String>)> = Vec::new();
	let mut relative_patterns: Vec<String> = Vec::new();

	for &glob in globs {
		let path_glob = SPath::new(glob);
		if path_glob.is_absolute() {
			let abs_base = longest_base_path_wild_free(&path_glob);
			let rel_pattern = relative_from_absolute(&path_glob, &abs_base);

			// Add to groups: if exists with same base, push; else create new.
			if let Some((_, patterns)) = groups.iter_mut().find(|(b, _)| b.as_str() == abs_base.as_str()) {
				patterns.push(rel_pattern);
			} else {
				groups.push((abs_base, vec![rel_pattern]));
			}
		} else {
			// Remove any leading "./" from the glob
			let cleaned = glob.trim_start_matches("./").to_string();
			// Collapse the relative glob by stripping the main_base prefix if present.
			let base_candidate: &str = main_base.as_str();
			let base_str_cleaned = {
				let s = base_candidate.trim_start_matches("./");
				if s.is_empty() {
					String::new()
				} else {
					let mut t = s.to_string();
					if !t.ends_with("/") {
						t.push('/');
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
	let mut final_groups: Vec<GlobGroup> = Vec::new();
	for (base, patterns) in groups {
		let mut merged = false;
		for existing_group in final_groups.iter_mut() {
			if existing_group.base.starts_with(&base) {
				// 'base' is a subdirectory of 'existing_base'
				let diff = base.diff(&existing_group.base).map(|p| p.to_string()).unwrap_or_default();
				for pat in patterns.iter() {
					let new_pat = if diff.is_empty() {
						pat.to_string()
					} else {
						SPath::new(&diff).join(pat).to_string()
					};
					existing_group.patterns.push(new_pat.clone());
				}
				
				// Recalculate prefixes for the merged pattern set
				let mut new_prefixes = Vec::new();
				let mut full_traversal_needed = false;
				for pat in existing_group.patterns.iter() {
					let pfx = glob_literal_prefixes(pat);
					if pfx.is_empty() {
						full_traversal_needed = true;
						break;
					}
					append_adjusted(&mut new_prefixes, &pfx);
				}

				if full_traversal_needed {
					existing_group.prefixes.clear();
				} else {
					normalize_prefixes(&mut new_prefixes);
					existing_group.prefixes = new_prefixes;
				}

				merged = true;
				break;
			} else if base.starts_with(&existing_group.base) {
				// 'existing_base' is a prefix of 'base'
				let diff = existing_group.base.diff(&base).map(|p| p.to_string()).unwrap_or_default();
				let mut new_patterns = patterns.clone();

				// Adjust and merge existing patterns (which were relative to the shorter base)
				for pat in existing_group.patterns.iter() {
					let new_pat = if diff.is_empty() {
						pat.clone()
					} else {
						SPath::new(&diff).join(pat).to_string()
					};
					new_patterns.push(new_pat.clone());
				}

				// Recalculate prefixes for all new patterns (incoming + adjusted existing)
				let mut new_prefixes = Vec::new();
				let mut full_traversal_needed = false;
				for pat in new_patterns.iter() {
					let pfx = glob_literal_prefixes(pat);
					if pfx.is_empty() {
						full_traversal_needed = true;
						break;
					}
					append_adjusted(&mut new_prefixes, &pfx);
				}

				existing_group.base = base.clone();
				existing_group.patterns = new_patterns;
				
				if full_traversal_needed {
					existing_group.prefixes.clear();
				} else {
					normalize_prefixes(&mut new_prefixes);
					existing_group.prefixes = new_prefixes;
				}

				merged = true;
				break;
			}
		}
		if !merged {
			let mut prefixes = Vec::new();
			let mut full_traversal_needed = false;

			for pat in patterns.iter() {
				let pfx = glob_literal_prefixes(pat);
				if pfx.is_empty() {
					full_traversal_needed = true;
					break;
				}
				append_adjusted(&mut prefixes, &pfx);
			}

			if full_traversal_needed {
				prefixes.clear();
			} else {
				normalize_prefixes(&mut prefixes);
			}

			final_groups.push(GlobGroup {
				base,
				patterns,
				prefixes,
			});
		}
	}

	Ok(final_groups)
}

/// Given an absolute glob pattern and its computed base, returns the relative glob
/// by removing the base prefix and any leading path separator.
/// Rewrites an absolute glob so it becomes relative to `group_base`.
///
/// # Example
///
/// ```text
/// inputs: glob="/root/a/**/*.txt", group_base="/root/a"
/// output: "**/*.txt"
/// ```
fn relative_from_absolute(glob: &SPath, group_base: &SPath) -> String {
	glob.diff(group_base).map(|p| p.to_string()).unwrap_or_else(|| glob.to_string())
}

/// Checks whether a directory path aligns with one of the candidate prefixes.
///
/// # Example
///
/// ```text
/// inputs: path="/root/a/b", base="/root", prefixes=["a", "docs"]
/// output: true
/// ```
fn directory_matches_allowed_prefixes(path: &SPath, base: &SPath, prefixes: &[String]) -> bool {
	if prefixes.is_empty() {
		return true;
	}
	if path.as_str() == base.as_str() {
		return true;
	}

	let Some(mut rel_path) = path.diff(base.path()) else {
		return true;
	};

	{
		let rel_str = rel_path.as_str();

		if let Some(stripped) = rel_str.strip_prefix("./") {
			if stripped.is_empty() {
				return true;
			}
			rel_path = SPath::new(stripped);
		} else if rel_str.is_empty() {
			return true;
		}
	}

	prefixes.iter().any(|prefix| {
		let prefix = prefix.as_str();
		if prefix.is_empty() {
			return true;
		}

		let prefix_spath = SPath::new(prefix);

		rel_path.starts_with(&prefix_spath) || prefix_spath.starts_with(&rel_path)
	})
}

/// Extracts literal directory prefixes from a glob pattern.
///
/// # Example
///
/// ```text
/// input: "assets/images/*.png"
/// output: ["assets", "assets/images"]
/// ```
fn glob_literal_prefixes(pattern: &str) -> Vec<String> {
	let clean = pattern.trim_start_matches("./");
	if clean.is_empty() {
		return Vec::new();
	}

	let segments: Vec<&str> = clean.split('/').filter(|s| !s.is_empty() && *s != ".").collect();

	// If there are no segments or only one segment (just a filename), no directory prefixes
	if segments.len() <= 1 {
		return Vec::new();
	}

	let mut prefixes = vec![String::new()];

	// Process all segments except the last one (which is the filename/pattern)
	for &segment in segments.iter().take(segments.len() - 1) {
		if segment == ".." || segment_contains_wildcard(segment) {
			break;
		}

		let mut next = Vec::new();
		if let Some(options) = expand_brace_segment(segment) {
			for prefix in &prefixes {
				for option in options.iter() {
					let new_prefix = if prefix.is_empty() {
						option.clone()
					} else {
						SPath::new(prefix).join(option).to_string()
					};
					next.push(new_prefix);
				}
			}
		} else if segment.contains('{') || segment.contains('}') {
			break;
		} else {
			for prefix in &prefixes {
				let new_prefix = if prefix.is_empty() {
					segment.to_string()
				} else {
					SPath::new(prefix).join(segment).to_string()
				};
				next.push(new_prefix);
			}
		}

		if next.is_empty() {
			break;
		}

		prefixes = next;
	}

	// If we only have the empty string, return empty
	if prefixes.len() == 1 && prefixes[0].is_empty() {
		Vec::new()
	} else {
		prefixes
	}
}

/// Expands a single `{a,b}` brace segment into concrete options.
///
/// # Example
///
/// ```text
/// input: "{foo,bar}"
/// output: Some(["foo", "bar"])
/// ```
fn expand_brace_segment(segment: &str) -> Option<Vec<String>> {
	if segment.starts_with('{') && segment.ends_with('}') {
		let inner = &segment[1..segment.len() - 1];
		if inner.contains('{') || inner.contains('}') {
			return None;
		}
		let options: Vec<String> = inner
			.split(',')
			.map(|s| s.trim())
			.filter(|s| !s.is_empty())
			.map(|s| s.to_string())
			.collect();
		if options.is_empty() { None } else { Some(options) }
	} else {
		None
	}
}

/// Reports whether the provided segment contains glob wildcards.
///
/// # Example
///
/// ```text
/// input: "src*"
/// output: true
/// ```
fn segment_contains_wildcard(segment: &str) -> bool {
	segment.contains('*') || segment.contains('?') || segment.contains('[')
}

/// Appends cloned prefix values into the running list.
///
/// # Example
///
/// ```text
/// inputs: target=["a"], values=["b","c"]
/// result: target=["a","b","c"]
/// ```
fn append_adjusted(target: &mut Vec<String>, values: &[String]) {
	for value in values {
		target.push(value.to_string());
	}
}

/// Normalizes prefix candidates by removing empties and duplicates.
///
/// # Example
///
/// ```text
/// input: ["", "a", "a"]
/// output: []
/// ```
fn normalize_prefixes(prefixes: &mut Vec<String>) {
	if prefixes.is_empty() {
		return;
	}
	if prefixes.iter().any(|p| p.is_empty()) {
		prefixes.clear();
		return;
	}
	prefixes.sort();
	prefixes.dedup();
}

// endregion: --- Support
