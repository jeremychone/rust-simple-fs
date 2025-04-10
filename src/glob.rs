use crate::{Error, Result, TOP_MAX_DEPTH};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};

pub const DEFAULT_EXCLUDE_GLOBS: &[&str] = &["**/.git", "**/.DS_Store", "**/target", "**/node_modules"];

pub fn get_glob_set(globs: &[&str]) -> Result<GlobSet> {
	let mut builder = GlobSetBuilder::new();

	for &glob_str in globs {
		let glob = GlobBuilder::new(glob_str)
			// NOTE: Important to set to true, otherwise single "*" will pass through "/".
			.literal_separator(true)
			.build()
			.map_err(|e| Error::GlobCantNew {
				glob: glob_str.to_string(),
				cause: e,
			})?;
		builder.add(glob);
	}

	let glob_set = builder.build().map_err(|e| Error::GlobSetCantBuild {
		globs: globs.iter().map(|&v| v.to_string()).collect(),
		cause: e,
	})?;

	Ok(glob_set)
}

pub fn longest_base_path_wild_free(pattern: &str) -> PathBuf {
	let path = Path::new(pattern);
	let mut base_path = PathBuf::new();

	for component in path.components() {
		let component_str = component.as_os_str().to_string_lossy();
		if component_str.contains('*') || component_str.contains('?') {
			break;
		}
		base_path.push(component);
	}

	base_path
}

/// Computes the maximum depth required for a set of glob patterns.
///
/// Logic:
/// 1) If a depth is provided via the argument, it is returned directly.
/// 2) Otherwise, if any pattern contains "**", returns TOP_MAX_DEPTH.
/// 3) Else, calculates the maximum folder level from patterns (using the folder count),
///    regardless if they contain a single "*" or only "/".
///
/// Returns at least 1.
pub fn get_depth(patterns: &[&str], depth: Option<usize>) -> usize {
	if let Some(user_depth) = depth {
		return user_depth;
	}
	for &g in patterns {
		if g.contains("**") {
			return TOP_MAX_DEPTH;
		}
	}
	let mut max_depth = 0;
	for &g in patterns {
		let depth_count = g.matches(|c| c == '\\' || c == '/').count() + 1;
		if depth_count > max_depth {
			max_depth = depth_count;
		}
	}
	max_depth.max(1)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	#[test]
	fn test_glob_get_depth_no_depth_simple() -> Result<()> {
		// -- Setup & Fixtures
		let test_cases: &[(&[&str], usize)] = &[
			(&["*/*"], 2),
			(&["some/path/**/and*/"], TOP_MAX_DEPTH),
			(&["*"], 1),
			(&["a/b", "c/d/e/f"], 4),
			(&[], 1),
		];

		// -- Exec & Check
		for &(patterns, expected) in test_cases {
			// -- Exec: Call get_depth without a provided depth
			let depth = get_depth(patterns, None);
			// -- Check: Verify returned depth matches expected value
			assert_eq!(
				depth, expected,
				"For patterns {:?}, expected depth {}, got {}",
				patterns, expected, depth
			);
		}
		Ok(())
	}

	#[test]
	fn test_glob_get_depth_with_depth_custom() -> Result<()> {
		// -- Setup & Fixtures
		let test_cases: &[(&[&str], usize, usize)] = &[
			(&["*/*"], 5, 5),
			(&["some/path/**/and*/"], 10, 10),
			(&["*"], 3, 3),
			(&["a/b", "c/d/e/f"], 7, 7),
			(&[], 4, 4),
		];

		// -- Exec & Check
		for &(patterns, provided_depth, expected) in test_cases {
			// -- Exec: Call get_depth with the provided depth value
			let depth = get_depth(patterns, Some(provided_depth));
			// -- Check: Verify returned depth equals expected value
			assert_eq!(
				depth, expected,
				"For patterns {:?} with provided depth {}, expected depth {}, got {}",
				patterns, provided_depth, expected, depth
			);
		}
		Ok(())
	}
}

// endregion: --- Tests
