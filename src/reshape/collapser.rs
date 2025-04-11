//! Collapse Camino Utf8Path paths similarly to canonicalize, but without performing I/O.
//!
//! Adapted from [cargo-binstall](https://github.com/cargo-bins/cargo-binstall/blob/main/crates/normalize-path/src/lib.rs)
//! and Rust's `path::normalize`.

use camino::{Utf8Component, Utf8Path, Utf8PathBuf};

/// Collapses a path buffer without performing I/O.
///
/// - Resolves `../` segments where possible.
/// - Removes `./` segments, except when leading
/// - All redundant separators and up-level references are collapsed.
///
/// Example:
/// - `a/b/../c` becomes `a/c`
/// - `a/../../c` becomes `../c`
/// - `./some` becomes `./some`
/// - `./some/./path` becomes `./some/path`
/// - `/a/../c` becomes `/c`
/// - `/a/../../c` becomes `/c`
///
/// However, this does not resolve symbolic links.
/// It consumes the input `Utf8PathBuf` and returns a new one.
pub fn into_collapsed(path: impl Into<Utf8PathBuf>) -> Utf8PathBuf {
	let path_buf = path.into();

	// For empty paths, return empty path
	if path_buf.as_str().is_empty() {
		return path_buf;
	}

	// Fast path: if the path is already collapsed, return it as is
	if is_collapsed(&path_buf) {
		return path_buf;
	}

	let mut components = Vec::new();
	let mut normal_seen = false;

	// Process each component
	for component in path_buf.components() {
		match component {
			Utf8Component::Prefix(prefix) => {
				components.push(Utf8Component::Prefix(prefix));
			}
			Utf8Component::RootDir => {
				components.push(Utf8Component::RootDir);
				normal_seen = false; // Reset after root dir
			}
			Utf8Component::CurDir => {
				// Only keep current dir at the beginning of a relative path
				if components.is_empty() {
					components.push(component);
				}
				// Otherwise, ignore it (it's redundant)
			}
			Utf8Component::ParentDir => {
				// If we've seen a normal component and we're not at the root,
				// pop the last component instead of adding the parent
				if normal_seen && !components.is_empty() {
					match components.last() {
						Some(Utf8Component::Normal(_)) => {
							components.pop();
							normal_seen = components.iter().any(|c| matches!(c, Utf8Component::Normal(_)));
							continue;
						}
						Some(Utf8Component::ParentDir) => {}
						Some(Utf8Component::RootDir) | Some(Utf8Component::Prefix(_)) => {
							// For absolute paths, we can discard parent dirs that
							// would go beyond the root
							continue;
						}
						_ => {}
					}
				}
				components.push(component);
			}
			Utf8Component::Normal(name) => {
				components.push(Utf8Component::Normal(name));
				normal_seen = true;
			}
		}
	}

	// If we've collapsed everything away, return "." or "" appropriately
	if components.is_empty() {
		if path_buf.as_str().starts_with("./") {
			return Utf8PathBuf::from(".");
		} else {
			return Utf8PathBuf::from("");
		}
	}

	// Reconstruct the path from the collapsed components
	let mut result = Utf8PathBuf::new();
	for component in components {
		result.push(component.as_str());
	}

	result
}

/// Same as [`into_collapsed`] except that if `Component::Prefix` or `Component::RootDir`
/// is encountered in a path that is supposed to be relative, or if the path attempts
/// to navigate above its starting point using `..`, it returns `None`.
///
/// Useful for ensuring a path stays within a certain relative directory structure.
pub fn try_into_collapsed(path: impl Into<Utf8PathBuf>) -> Option<Utf8PathBuf> {
	let path_buf = path.into();

	// Fast path: if the path is already collapsed and doesn't contain problematic components,
	// return it as is
	if is_collapsed(&path_buf) && !contains_problematic_components(&path_buf) {
		return Some(path_buf);
	}

	let mut components = Vec::new();
	let mut normal_seen = false;
	let mut parent_count = 0;

	// Process each component
	for component in path_buf.components() {
		match component {
			Utf8Component::Prefix(_) => {
				// A prefix indicates this is not a relative path
				return None;
			}
			Utf8Component::RootDir => {
				// A root directory indicates this is not a relative path
				return None;
			}
			Utf8Component::CurDir => {
				// Only keep current dir at the beginning of a relative path
				if components.is_empty() {
					components.push(component);
				}
				// Otherwise, ignore it (it's redundant)
			}
			Utf8Component::ParentDir => {
				if normal_seen {
					// If we've seen a normal component, pop the last component
					if let Some(Utf8Component::Normal(_)) = components.last() {
						components.pop();
						normal_seen = components.iter().any(|c| matches!(c, Utf8Component::Normal(_)));
						continue;
					}
				} else {
					// If we haven't seen a normal component, this is a leading ".."
					parent_count += 1;
				}
				components.push(component);
			}
			Utf8Component::Normal(name) => {
				components.push(Utf8Component::Normal(name));
				normal_seen = true;
			}
		}
	}

	// If there are any parent dirs still in the path, check if they would try to go
	// beyond the starting dir
	if parent_count > 0 && components.iter().filter(|c| matches!(c, Utf8Component::Normal(_))).count() < parent_count {
		return None;
	}

	// If we've collapsed everything away, return "." or "" appropriately
	if components.is_empty() {
		if path_buf.as_str().starts_with("./") {
			return Some(Utf8PathBuf::from("."));
		} else {
			return Some(Utf8PathBuf::from(""));
		}
	}

	// Reconstruct the path from the collapsed components
	let mut result = Utf8PathBuf::new();
	for component in components {
		result.push(component.as_str());
	}

	Some(result)
}

/// Returns `true` if the path is already collapsed.
///
/// A path is considered collapsed if it contains no `.` components
/// and no `..` components that immediately follow a normal component.
/// Leading `..` components in relative paths are allowed.
/// Absolute paths should not contain `..` at all after the root/prefix.
pub fn is_collapsed(path: impl AsRef<Utf8Path>) -> bool {
	let path = path.as_ref();
	let mut components = path.components().peekable();
	let mut is_absolute = false;
	let mut previous_was_normal = false;

	while let Some(component) = components.next() {
		match component {
			Utf8Component::Prefix(_) | Utf8Component::RootDir => {
				is_absolute = true;
			}
			Utf8Component::CurDir => {
				// Current dir components are allowed only at the beginning of a relative path
				if previous_was_normal || is_absolute || components.peek().is_some() {
					return false;
				}
			}
			Utf8Component::ParentDir => {
				// In absolute paths, parent dir components should never appear
				if is_absolute {
					return false;
				}
				// In relative paths, parent dir should not follow a normal component
				if previous_was_normal {
					return false;
				}
			}
			Utf8Component::Normal(_) => {
				previous_was_normal = true;
			}
		}
	}

	true
}

// Helper function for try_into_collapsed
fn contains_problematic_components(path: &Utf8Path) -> bool {
	let mut has_parent_after_normal = false;
	let mut has_prefix_or_root = false;
	let mut normal_seen = false;

	for component in path.components() {
		match component {
			Utf8Component::Prefix(_) | Utf8Component::RootDir => {
				has_prefix_or_root = true;
			}
			Utf8Component::ParentDir => {
				if normal_seen {
					has_parent_after_normal = true;
				}
			}
			Utf8Component::Normal(_) => {
				normal_seen = true;
			}
			_ => {}
		}
	}

	has_prefix_or_root || has_parent_after_normal
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	// -- Tests for into_collapsed

	#[test]
	fn test_reshape_collapser_into_collapsed_simple() -> Result<()> {
		// -- Setup & Fixtures
		let data = &[
			// Basic cases
			("a/b/c", "a/b/c"),
			("a/./b", "a/b"),
			("./a/b", "./a/b"),
			("./a/b", "./a/b"),
			("a/./b/.", "a/b"),
			("/a/./b/.", "/a/b"),
			("a/../b", "b"),
			("../a/b", "../a/b"),         // Keep leading ..
			("../a/b/..", "../a"),        // Keep leading ..
			("../a/b/../../..", "../.."), // Keep leading ..
			("a/b/..", "a"),
			("a/b/../..", ""),          // Collapses to current dir
			("../../a/b", "../../a/b"), // Keep multiple leading ..
			(".", "."),                 // "."
			("..", ".."),               // ".." stays ".."
		];

		// -- Exec & Check
		for (input, expected) in data {
			let input_path = Utf8PathBuf::from(input);
			let result_path = into_collapsed(input_path);
			let expected_path = Utf8PathBuf::from(expected);
			assert_eq!(
				result_path, expected_path,
				"Input: '{}', Expected: '{}', Got: '{}'",
				input, expected, result_path
			);
		}

		Ok(())
	}
}

// endregion: --- Tests
