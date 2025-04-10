//! Path normalization functions
//!
//! Normalize path strings by collapsing redundant separators and handling platform-specific quirks.

use camino::{Utf8Path, Utf8PathBuf};

pub fn needs_normalize(path: &Utf8Path) -> bool {
	// Check if the path contains any backslashes, multiple consecutive slashes,
	// single dots (except at the start), or the Windows-specific `\\?\` prefix.
	let path_str = path.as_str();
	path_str.contains('\\') || path_str.contains("//") || path_str.contains("/.") || path_str.starts_with(r"\\?\")
}

/// Normalizes a path by:
/// - Converting backslashes to forward slashes
/// - Collapsing multiple consecutive slashes to single slashes
/// - Removing single dots except at the start
/// - Removing Windows-specific `\\?\` prefix
///
/// The function performs a quick check to determine if normalization is actually needed.
/// If no normalization is required, it returns the original path to avoid unnecessary allocations.
pub fn into_normalized(path: Utf8PathBuf) -> Utf8PathBuf {
	// Quick check to see if any normalization is needed
	let path_str = path.as_str();

	// Check for conditions that require normalization
	let needs_normalization = needs_normalize(&path);

	if !needs_normalization {
		return path;
	}

	// Perform normalization
	let mut result = String::with_capacity(path_str.len());
	let mut chars = path_str.chars().peekable();
	let mut last_was_slash = false;

	// Handle Windows UNC path prefix (\\?\)
	if path_str.starts_with(r"\\?\") {
		for _ in 0..4 {
			chars.next(); // Skip the first 4 chars
		}
	}

	while let Some(c) = chars.next() {
		match c {
			'\\' | '/' => {
				// Convert backslash to forward slash and collapse consecutive slashes
				if !last_was_slash {
					result.push('/');
					last_was_slash = true;
				}
			}
			'.' => {
				// Special handling for dots
				if last_was_slash {
					// Look ahead to check if this is a "/./" pattern
					match chars.peek() {
						Some(&'/') | Some(&'\\') => {
							// Skip single dot if it's not at the start
							if !result.is_empty() {
								chars.next(); // Skip the next slash
								continue;
							}
						}
						// Check if it's a "../" pattern (which we want to keep)
						Some(&'.') => {
							result.push('.');
							last_was_slash = false;
						}
						// Something else
						_ => {
							result.push('.');
							last_was_slash = false;
						}
					}
				} else {
					result.push('.');
					last_was_slash = false;
				}
			}
			_ => {
				result.push(c);
				last_was_slash = false;
			}
		}
	}

	// If the original path ended with a slash, ensure the normalized path does too
	if (path_str.ends_with('/') || path_str.ends_with('\\')) && !result.ends_with('/') {
		result.push('/');
	}

	Utf8PathBuf::from(result)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_normalizer_into_normalize_backslashes() -> Result<()> {
		// -- Setup & Fixtures
		let paths = [
			(r"C:\Users\name\file.txt", "C:/Users/name/file.txt"),
			(r"path\to\file.txt", "path/to/file.txt"),
			(r"mixed/path\style", "mixed/path/style"),
		];

		// -- Exec & Check
		for (input, expected) in paths {
			let path = Utf8PathBuf::from(input);
			let normalized = into_normalized(path);
			assert_eq!(
				normalized.as_str(),
				expected,
				"Failed to normalize backslashes in '{}'",
				input
			);
		}

		Ok(())
	}

	#[test]
	fn test_normalizer_into_normalize_multiple_slashes() -> Result<()> {
		// -- Setup & Fixtures
		let paths = [
			("//path//to///file.txt", "/path/to/file.txt"),
			("path////file.txt", "path/file.txt"),
			(r"\\server\\share\\file.txt", "/server/share/file.txt"),
		];

		// -- Exec & Check
		for (input, expected) in paths {
			let path = Utf8PathBuf::from(input);
			let normalized = into_normalized(path);
			assert_eq!(
				normalized.as_str(),
				expected,
				"Failed to collapse multiple slashes in '{}'",
				input
			);
		}

		Ok(())
	}

	#[test]
	fn test_normalizer_into_normalize_single_dots() -> Result<()> {
		// -- Setup & Fixtures
		let paths = [
			("path/./file.txt", "path/file.txt"),
			("./path/./to/./file.txt", "./path/to/file.txt"),
			("path/to/./././file.txt", "path/to/file.txt"),
		];

		// -- Exec & Check
		for (input, expected) in paths {
			let path = Utf8PathBuf::from(input);
			let normalized = into_normalized(path);
			assert_eq!(
				normalized.as_str(),
				expected,
				"Failed to handle single dots correctly in '{}'",
				input
			);
		}

		Ok(())
	}

	#[test]
	fn test_normalizer_into_normalize_preserve_parent_dirs() -> Result<()> {
		// -- Setup & Fixtures
		let paths = [
			("path/../file.txt", "path/../file.txt"),
			("../path/file.txt", "../path/file.txt"),
			("path/../../file.txt", "path/../../file.txt"),
		];

		// -- Exec & Check
		for (input, expected) in paths {
			let path = Utf8PathBuf::from(input);
			let normalized = into_normalized(path);
			assert_eq!(
				normalized.as_str(),
				expected,
				"Should preserve parent directory references in '{}'",
				input
			);
		}

		Ok(())
	}

	#[test]
	fn test_normalizer_into_normalize_windows_prefix() -> Result<()> {
		// -- Setup & Fixtures
		let paths = [
			(r"\\?\C:\Users\name\file.txt", "C:/Users/name/file.txt"),
			(r"\\?\UNC\server\share", "UNC/server/share"),
		];

		// -- Exec & Check
		for (input, expected) in paths {
			let path = Utf8PathBuf::from(input);
			let normalized = into_normalized(path);
			assert_eq!(
				normalized.as_str(),
				expected,
				"Failed to remove Windows prefix in '{}'",
				input
			);
		}

		Ok(())
	}

	#[test]
	fn test_normalizer_into_normalize_no_change_needed() -> Result<()> {
		// -- Setup & Fixtures
		let paths = ["path/to/file.txt", "/absolute/path/file.txt", "../parent/dir", "file.txt"];

		// -- Exec & Check
		for input in paths {
			let path = Utf8PathBuf::from(input);
			let path_clone = path.clone();
			let normalized = into_normalized(path);
			// This should be a simple identity return with no changes
			assert_eq!(
				normalized, path_clone,
				"Path should not change when normalization not needed"
			);
		}

		Ok(())
	}

	#[test]
	fn test_normalizer_into_normalize_trailing_slash() -> Result<()> {
		// -- Setup & Fixtures
		let paths = [
			("path/to/dir/", "path/to/dir/"),
			(r"path\to\dir\", "path/to/dir/"),
			("path//to///dir///", "path/to/dir/"),
		];

		// -- Exec & Check
		for (input, expected) in paths {
			let path = Utf8PathBuf::from(input);
			let normalized = into_normalized(path);
			assert_eq!(
				normalized.as_str(),
				expected,
				"Should preserve trailing slash in '{}'",
				input
			);
		}

		Ok(())
	}

	#[test]
	fn test_normalizer_into_normalize_complex_paths() -> Result<()> {
		// -- Setup & Fixtures
		let paths = [
			(
				r"C:\Users\.\name\..\admin\//docs\file.txt",
				"C:/Users/name/../admin/docs/file.txt",
			),
			(
				r"\\?\C:\Program Files\\.\multiple//slashes",
				"C:/Program Files/multiple/slashes",
			),
			("./current/dir/./file.txt", "./current/dir/file.txt"),
		];

		// -- Exec & Check
		for (input, expected) in paths {
			let path = Utf8PathBuf::from(input);
			let normalized = into_normalized(path);
			assert_eq!(
				normalized.as_str(),
				expected,
				"Failed to normalize complex path '{}'",
				input
			);
		}

		Ok(())
	}
}

// endregion: --- Tests
