use std::fs;
use std::path::{Path, PathBuf};

/// An SFile can be constructed from a Path, io::DirEntry, or walkdir::DirEntry
/// and guarantees the following:
///
/// - The entry is a file (exists).
/// - It has a file name.
/// - The full path is UTF-8 valid.
#[derive(Debug)]
pub struct SFile {
	path: PathBuf,
}

/// Constructors that guarantees the SFile contract describe in the struct
impl SFile {
	/// Constructor from Path.
	pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
		let path = path.as_ref();

		// Result the SFile only the entry is a file and the path is utf8 compatible
		if path.is_file() && path.to_str().is_some() && path.file_name().is_some() {
			Some(Self {
				path: path.to_path_buf(),
			})
		} else {
			None
		}
	}

	/// Constructor from fs DirEntry.
	pub fn from_fs_entry(fs_entry: fs::DirEntry) -> Option<Self> {
		let is_file = fs_entry.file_type().map(|ft| ft.is_file()).unwrap_or(false);
		let path = fs_entry.path();
		// Result the SFile only the entry is a file and the path is utf8 compatible
		if is_file && path.to_str().is_some() && path.file_name().is_some() {
			Some(Self {
				path: path.to_path_buf(),
			})
		} else {
			None
		}
	}

	/// Constructor from walkdir entry.
	pub fn from_walkdir_entry(wd_entry: walkdir::DirEntry) -> Option<Self> {
		let path = wd_entry.path();
		// Result the SFile only the entry is a file and the path is utf8 compatible
		if wd_entry.file_type().is_file() && path.to_str().is_some() && path.file_name().is_some() {
			Some(Self {
				path: wd_entry.into_path(),
			})
		} else {
			None
		}
	}
}

/// Public return Path constructs.
impl SFile {
	pub fn into_path_buf(self) -> PathBuf {
		self.path
	}

	pub fn path(&self) -> &Path {
		&self.path
	}
}

/// Public file components as str methods.
impl SFile {
	/// Returns the &str of the path.
	///
	/// NOTE: We know that this must be Some() since the SFile constructor guarantees that
	///       the path.to_str() is valid.
	pub fn to_str(&self) -> &str {
		self.path.to_str().unwrap_or_default()
	}

	/// Returns the &str representation of the file_name()
	///
	/// NOTE: According to the constructors' contract, this method will never return ""
	///       as a file_name() is required for construction.
	pub fn file_name(&self) -> &str {
		self.path.file_name().and_then(|n| n.to_str()).unwrap_or_default()
	}

	/// Returns the &str representation of the file_name()
	///
	/// NOTE: According to the constructors' contract, this method will never return ""
	///       as a file_name() is required for construction, and stem is always part of it.
	pub fn stem(&self) -> &str {
		self.path.file_stem().and_then(|n| n.to_str()).unwrap_or_default()
	}

	/// Returns the Option<&str> representation of the extension().
	///
	/// NOTE: This should never be a non-UTF-8 string
	///       as the path was validated during SFile construction.
	pub fn extension(&self) -> Option<&str> {
		self.path.extension().and_then(|os_str| os_str.to_str())
	}
}
