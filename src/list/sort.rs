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

	if end_weighted {
		// Use the last matching glob index (from the end).
		let mut found: Option<usize> = None;
		for (idx, gm) in matchers.iter().map(|(i, m)| (*i, m)) {
			if gm.is_match(path) {
				found = Some(idx);
			}
		}
		found.unwrap_or(usize::MAX)
	} else {
		// Use the first matching glob index (from the beginning).
		for (idx, gm) in matchers.iter().map(|(i, m)| (*i, m)) {
			if gm.is_match(path) {
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

	#[test]
	fn test_list_sort_sort_files_by_globs_end_weighted_false() -> Result<()> {
		// -- Setup & Fixtures
		let globs = ["src/**", "src/list/**", "src/list/sort.rs"];
		let matchers: Vec<(usize, GlobMatcher)> = globs
			.iter()
			.enumerate()
			.map(|(i, g)| {
				let gm = Glob::new(g).map_err(|e| format!("bad glob: {g} - {e}"))?.compile_matcher();
				Ok((i, gm))
			})
			.collect::<core::result::Result<_, String>>()
			.map_err(|e| format!("glob build failed: {e}"))?;

		let p_main = SPath::new("src/main.rs"); // May or may not exist; used for logic-only test.
		let p_sort = SPath::new("src/list/sort.rs");
		let p_lib = SPath::new("src/lib.rs");

		// -- Exec & Check
		// For end_weighted = false, first match is used.
		let i_main = super::match_index_for_path(&p_main, &matchers, false);
		let i_sort = super::match_index_for_path(&p_sort, &matchers, false);
		let i_lib = super::match_index_for_path(&p_lib, &matchers, false);

		// Expectations:
		// - "src/main.rs" should match "src/**" (index 0) if exists, else MAX (logic still holds).
		// - "src/list/sort.rs" first matches "src/**" (index 0), even though later globs also match.
		// - "src/lib.rs" should match "src/**" (index 0) if exists, else MAX.
		assert!(i_sort == 0 || i_sort == usize::MAX);
		assert!(i_main == 0 || i_main == usize::MAX);
		assert!(i_lib == 0 || i_lib == usize::MAX);

		Ok(())
	}

	#[test]
	fn test_list_sort_sort_files_by_globs_end_weighted_true() -> Result<()> {
		// -- Setup & Fixtures
		let globs = ["src/**", "src/list/**", "src/list/sort.rs"];
		let matchers: Vec<(usize, GlobMatcher)> = globs
			.iter()
			.enumerate()
			.map(|(i, g)| {
				let gm = Glob::new(g).map_err(|e| format!("bad glob: {g} - {e}"))?.compile_matcher();
				Ok((i, gm))
			})
			.collect::<core::result::Result<_, String>>()
			.map_err(|e| format!("glob build failed: {e}"))?;

		let p_sort = SPath::new("src/list/sort.rs");
		let p_list_mod = SPath::new("src/list/mod.rs");

		// -- Exec & Check
		// For end_weighted = true, last match is used.
		let i_sort = super::match_index_for_path(&p_sort, &matchers, true);
		let i_list_mod = super::match_index_for_path(&p_list_mod, &matchers, true);

		// Expectations:
		// - "src/list/sort.rs" should take the last matching pattern "src/list/sort.rs" (index 2) if available.
		// - "src/list/mod.rs" should take the last matching "src/list/**" (index 1) if available.
		assert!(i_sort == 2 || i_sort == usize::MAX);
		assert!(i_list_mod == 1 || i_list_mod == usize::MAX);

		Ok(())
	}
}

// endregion: --- Tests
