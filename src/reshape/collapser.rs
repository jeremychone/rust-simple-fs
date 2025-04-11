//! Collpase Camino Utf8Path paths similarly to canonicalize, but without performing I/O.
//!
//! Adapted from [cargo-binstall](https://github.com/cargo-bins/cargo-binstall/blob/main/crates/normalize-path/src/lib.rs)

use crate::SPath;
use camino::{Utf8Component, Utf8Path, Utf8PathBuf};

/// Collpase a path without performing I/O.
///
/// - Resolved the "../"
/// - And the "./"
///
/// All redundant separator and up-level references are collapsed.
///
/// However, this does not resolve links.
pub fn collapse(path: impl AsRef<Utf8Path>) -> SPath {
	let path = path.as_ref();
	let mut components = path.components().peekable();
	let mut ret = if let Some(c @ Utf8Component::Prefix(..)) = components.peek() {
		let buf = Utf8PathBuf::from(c);
		components.next();
		buf
	} else {
		Utf8PathBuf::new()
	};

	for component in components {
		match component {
			Utf8Component::Prefix(..) => unreachable!(),
			Utf8Component::RootDir => {
				ret.push(component);
			}
			Utf8Component::CurDir => {}
			Utf8Component::ParentDir => {
				ret.pop();
			}
			Utf8Component::Normal(c) => {
				ret.push(c);
			}
		}
	}

	ret.into()
}

/// Same as [`collapse`] except that if
/// `Component::Prefix`/`Component::RootDir` is encountered,
/// or if the path points outside of current dir, returns `None`.
pub fn try_collapse(path: impl AsRef<Utf8Path>) -> Option<SPath> {
	let path = path.as_ref();
	let mut ret = Utf8PathBuf::new();

	for component in path.components() {
		match component {
			Utf8Component::Prefix(..) | Utf8Component::RootDir => return None,
			Utf8Component::CurDir => {}
			Utf8Component::ParentDir => {
				if !ret.pop() {
					return None;
				}
			}
			Utf8Component::Normal(c) => {
				ret.push(c);
			}
		}
	}

	Some(ret.into())
}

/// Return `true` if the path is collapsed.
///
/// Note that if the path contains `/./` or `\.\` it will return false
///
pub fn is_collapsed(path: impl AsRef<Utf8Path>) -> bool {
	let path = path.as_ref();
	let path_str = path.as_str();
	if path_str.contains("/./") || path.as_str().contains("\\.\\") {
		return false;
	}

	for component in path.components() {
		match component {
			//  Note: The CurDir is proabbly not necessary since the above
			Utf8Component::CurDir | Utf8Component::ParentDir => {
				return false;
			}
			_ => continue,
		}
	}

	true
}
// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_collapser_collpase_ok() -> Result<()> {
		// -- Setup & Fixtures
		let data = &[
			//
			("some/path/../to/file.rs", "some/to/file.rs"),
			("/some/../file.txt", "/file.txt"),
			("some/../file.txt", "file.txt"),
			("some/./file.txt", "some/file.txt"),
			("/../../some-dir", "/some-dir"),
		];

		// -- Exec & Check
		for (path, expected) in data {
			// assert!(is_collapsed(path), "Should be collapsed '{path}'");
			let res_path = collapse(path);
			let expected = Utf8PathBuf::from(expected);

			// NOTE: exepcted is now utf8pathbuf, with / all the time, but when doing Eq, it will compare on components
			assert_eq!(res_path.as_str(), expected, "Wrong normalization for path '{path}'");
		}

		Ok(())
	}

	#[test]
	fn test_collapser_is_collpased_true() -> Result<()> {
		// -- Setup & Fixtures
		let data = &[
			//
			"some/path/to/file.rs",
			"/some/file.txt",
			"some/dir",
			"dir",
			"/root",
		];

		// -- Exec & Check
		for path in data {
			assert!(is_collapsed(path), "Should be collapsed '{path}'");
		}

		Ok(())
	}

	#[test]
	fn test_collapser_is_collpased_false() -> Result<()> {
		// -- Setup & Fixtures
		let data = &[
			//
			"some/path/../to/file.rs",
			"../to/file.rs",
			"some/path/./to/file.rs",
		];

		// -- Exec & Check
		for path in data {
			assert!(!is_collapsed(path), "Should NOT be collapsed '{path}'");
		}

		Ok(())
	}
}

// endregion: --- Tests
