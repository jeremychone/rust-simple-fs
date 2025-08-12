use std::cmp::Ordering;

use globset::{Glob, GlobMatcher};

use crate::{Error, Result, SPath};

/// Sort files by glob priority, then by full path.
///
/// - Builds a Vec of Glob (no GlobSet).
/// - The "glob index" used for ordering is chosen as:
///   - end_weighted = false: first matching glob index (from the beginning).
///   - end_weighted = true: last matching glob index (from the end).
/// - Files are ordered by (glob_index, full_path). Non-matches get `usize::MAX`.
pub fn sort_by_globs<T>(mut items: Vec<T>, globs: &[&str], end_weighted: bool) -> Result<Vec<T>>
where
	T: AsRef<SPath>,
{
	// Build individual Glob matchers in order.
	let mut matchers: Vec<(usize, GlobMatcher)> = Vec::with_capacity(globs.len());
	for (idx, pat) in globs.iter().enumerate() {
		let gm = Glob::new(pat).map_err(Error::sort_by_globs)?.compile_matcher();
		matchers.push((idx, gm));
	}

	items.sort_by(|a, b| {
		// Get paths from either SFile or SPath via AsRef<SPath>.
		let ap: &SPath = a.as_ref();
		let bp: &SPath = b.as_ref();

		let ai = match_index_for_path(ap, &matchers, end_weighted);
		let bi = match_index_for_path(bp, &matchers, end_weighted);

		match ai.cmp(&bi) {
			Ordering::Equal => {
				// Tiebreaker: by full path from SPath.
				let an = ap.as_str();
				let bn = bp.as_str();
				an.cmp(bn)
			}
			other => other,
		}
	});

	Ok(items)
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
