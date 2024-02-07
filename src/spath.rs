use crate::{Error, Result};
use core::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// An SPath can be constructed from a Path, io::DirEntry, or walkdir::DirEntry
/// and guarantees the following:
///
/// - The full path is UTF-8 valid.
/// - It has a file name.
#[derive(Debug)]
pub struct SPath {
	path: PathBuf,
}

/// Constructors that guarantees the SPath contract describe in the struct
impl SPath {
	/// Constructor from Path and all impl AsRef<Path>.
	///
	/// Returns Result<SPath>
	///
	/// Note: Prefer the use of the SPath::try_from(...) when available as it might
	///       avoid a PathBuf allocation.
	pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
		let path = path.as_ref();

		validate_spath_for_result(path)?;

		Ok(Self {
			path: path.to_path_buf(),
		})
	}

	/// Constructor for anything that impl AsRef<Path>.
	///
	/// Returns Option<SFile>. Useful for filter_map.
	///
	/// Note: Favor using concrete type functions like `SPath::from_path_buf_ok`
	///       when available.
	pub fn from_path_ok(path: impl AsRef<Path>) -> Option<Self> {
		let path = path.as_ref();
		validate_spath_for_option(path)?;

		Some(Self {
			path: path.to_path_buf(),
		})
	}

	/// Constructor from PathBuf returning an Option, none if validation fail.
	/// Useful for filter_map.
	pub fn from_path_buf_ok(path_buf: PathBuf) -> Option<Self> {
		validate_spath_for_option(&path_buf)?;
		Some(Self { path: path_buf })
	}

	/// Constructor from fs::DirEntry returning an Option, none if validation fail.
	/// Useful for filter_map.
	pub fn from_fs_entry_ok(fs_entry: fs::DirEntry) -> Option<Self> {
		let path_buf = fs_entry.path();
		validate_spath_for_option(&path_buf)?;
		Some(Self { path: path_buf })
	}

	/// Constructor from walkdir::DirEntry returning an Option, none if validation fail.
	/// Useful for filter_map.
	pub fn from_walkdir_entry_ok(wd_entry: walkdir::DirEntry) -> Option<Self> {
		let path = wd_entry.path();
		validate_spath_for_option(path)?;
		Some(Self {
			path: wd_entry.into_path(),
		})
	}
}

/// Public return Path constructs.
impl SPath {
	pub fn into_path_buf(self) -> PathBuf {
		self.path
	}

	pub fn path(&self) -> &Path {
		&self.path
	}
}

/// Public file components as str methods.
impl SPath {
	/// Returns the &str of the path.
	///
	/// NOTE: We know that this must be Some() since the SPath constructor guarantees that
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
	///       as the path was validated during SPath construction.
	pub fn extension(&self) -> Option<&str> {
		self.path.extension().and_then(|os_str| os_str.to_str())
	}
}

// region:    --- Std Traits Impls

impl AsRef<Path> for SPath {
	fn as_ref(&self) -> &Path {
		self.path.as_ref()
	}
}

impl fmt::Display for SPath {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_str())
	}
}

impl From<SPath> for String {
	fn from(val: SPath) -> Self {
		val.to_str().to_string()
	}
}

impl From<&SPath> for String {
	fn from(val: &SPath) -> Self {
		val.to_str().to_string()
	}
}

// endregion: --- Std Traits Impls

// region:    --- TryFrom

impl TryFrom<&str> for SPath {
	type Error = Error;
	fn try_from(path: &str) -> Result<SPath> {
		let path = Path::new(path);
		validate_spath_for_result(path)?;

		Ok(Self {
			path: path.to_path_buf(),
		})
	}
}

impl TryFrom<String> for SPath {
	type Error = Error;
	fn try_from(path: String) -> Result<SPath> {
		SPath::try_from(path.as_str())
	}
}

impl TryFrom<&String> for SPath {
	type Error = Error;
	fn try_from(path: &String) -> Result<SPath> {
		SPath::try_from(path.as_str())
	}
}

impl TryFrom<PathBuf> for SPath {
	type Error = Error;
	fn try_from(path_buf: PathBuf) -> Result<SPath> {
		validate_spath_for_result(&path_buf)?;

		Ok(Self { path: path_buf })
	}
}

impl TryFrom<fs::DirEntry> for SPath {
	type Error = Error;
	fn try_from(fs_entry: fs::DirEntry) -> Result<SPath> {
		let path_buf = fs_entry.path();
		validate_spath_for_result(&path_buf)?;

		Ok(Self { path: path_buf })
	}
}

impl TryFrom<walkdir::DirEntry> for SPath {
	type Error = Error;
	fn try_from(wd_entry: walkdir::DirEntry) -> Result<SPath> {
		let path = wd_entry.path();

		validate_spath_for_result(path)?;

		Ok(Self {
			path: wd_entry.into_path(),
		})
	}
}

// endregion: --- TryFrom

// region:    --- Path Validation

pub(crate) fn validate_spath_for_result(path: &Path) -> Result<()> {
	if path.to_str().is_none() {
		return Err(Error::PathNotUtf8(path.to_string_lossy().to_string()));
	}
	if path.file_name().is_none() {
		return Err(Error::PathHasNoFileName(path.to_string_lossy().to_string()));
	}

	Ok(())
}

/// Validate but without generating an error (good for the _ok constructors)
pub(crate) fn validate_spath_for_option(path: &Path) -> Option<()> {
	path.to_str()?;
	path.file_name()?;

	Some(())
}

// endregion: --- Path Validation
