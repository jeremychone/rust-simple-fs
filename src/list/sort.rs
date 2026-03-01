use std::cmp::Ordering;

use globset::{Glob, GlobMatcher};

use crate::{Error, Result, SPath};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortByGlobsOptions {
	pub end_weighted: bool,
	pub no_match_position: NoMatchPosition,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoMatchPosition {
	Start,
	#[default]
	End,
}

impl Default for SortByGlobsOptions {
	fn default() -> Self {
		Self {
			end_weighted: false,
			no_match_position: NoMatchPosition::End,
		}
	}
}

impl From<bool> for SortByGlobsOptions {
	fn from(end_weighted: bool) -> Self {
		Self {
			end_weighted,
			..Default::default()
		}
	}
}

/// Sort files by glob priority, then by full path.
///
/// - Builds a Vec of Glob (no GlobSet).
/// - The "glob index" used for ordering is chosen as:
///   - end_weighted = false: first matching glob index (from the beginning).
///   - end_weighted = true: last matching glob index (from the end).
/// - Files are ordered by (glob_index, full_path). Non-matches are preserved in their original order.
pub fn sort_by_globs<T>(items: Vec<T>, globs: &[&str], options: impl Into<SortByGlobsOptions>) -> Result<Vec<T>>
where
	T: AsRef<SPath>,
{
	let options = options.into();

	// Build individual Glob matchers in order.
	let mut matchers: Vec<(usize, GlobMatcher)> = Vec::with_capacity(globs.len());
	for (idx, pat) in globs.iter().enumerate() {
		let gm = Glob::new(pat).map_err(Error::sort_by_globs)?.compile_matcher();
		matchers.push((idx, gm));
	}

	let mut matched = Vec::with_capacity(items.len());
	let mut unmatched = Vec::with_capacity(items.len());

	for (orig_idx, item) in items.into_iter().enumerate() {
		let glob_idx = match_index_for_path(item.as_ref(), &matchers, options.end_weighted);
		if glob_idx == usize::MAX {
			unmatched.push(item);
		} else {
			matched.push((glob_idx, orig_idx, item));
		}
	}

	// Sort matched.
	matched.sort_by(|(ai, a_orig, a_item), (bi, b_orig, b_item)| {
		match ai.cmp(bi) {
			Ordering::Equal => {
				// Tiebreaker: by full path from SPath.
				let an = a_item.as_ref().as_str();
				let bn = b_item.as_ref().as_str();
				match an.cmp(bn) {
					Ordering::Equal => a_orig.cmp(b_orig),
					other => other,
				}
			}
			other => other,
		}
	});

	let matched: Vec<T> = matched.into_iter().map(|(_, _, item)| item).collect();

	let mut res = Vec::with_capacity(matched.len() + unmatched.len());
	match options.no_match_position {
		NoMatchPosition::Start => {
			res.extend(unmatched);
			res.extend(matched);
		}
		NoMatchPosition::End => {
			res.extend(matched);
			res.extend(unmatched);
		}
	}

	Ok(res)
}

// region:    --- Support

#[inline]
fn match_index_for_path(path: &SPath, matchers: &[(usize, GlobMatcher)], end_weighted: bool) -> usize {
	if matchers.is_empty() {
		return usize::MAX;
	}

	// Normalize the input used for matching: many callers produce paths that start with "./".
	// Glob patterns typically don't include that leading "./", so strip it for matching purposes.
	let s = path.as_str();
	let match_input = s.strip_prefix("./").unwrap_or(s);

	if end_weighted {
		// Use the last matching glob index (from the end).
		let mut found: Option<usize> = None;
		for (idx, gm) in matchers.iter().map(|(i, m)| (*i, m)) {
			if gm.is_match(match_input) {
				found = Some(idx);
			}
		}
		found.unwrap_or(usize::MAX)
	} else {
		// Use the first matching glob index (from the beginning).
		for (idx, gm) in matchers.iter().map(|(i, m)| (*i, m)) {
			if gm.is_match(match_input) {
				return idx;
			}
		}
		usize::MAX
	}
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::list_files;

	#[test]
	fn test_list_sort_sort_files_by_globs_end_weighted_true() -> Result<()> {
		// -- Setup & Fixtures
		let globs = ["src/**/*", "src/common/**/*.*", "src/list/sort.rs"];
		let files = list_files("./", Some(&globs), None)?;

		// -- Exec
		let files = sort_by_globs(files, &globs, true)?;

		// -- Check
		let file_names = files.into_iter().map(|v| v.to_string()).collect::<Vec<_>>();
		let last_file = file_names.last().ok_or("Should have a least one")?;

		assert_eq!(last_file, "./src/list/sort.rs");

		Ok(())
	}
}

// endregion: --- Tests
