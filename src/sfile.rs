use crate::{validate_spath_for_option, validate_spath_for_result, SPath};
use crate::{Error, Result};
use core::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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
	/// Constructor for SFile accepting anything that implements Into<PathBuf>.
	///
	/// Note: This is quite ergonomic and allows for avoiding a PathBuf allocation
	///       if a PathBuf is provided.
	pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
		let path = path.into();

		validate_sfile_for_result(&path)?;

		Ok(Self { path })
	}

	/// Constructor from File and all impl AsRef<Path>.
	///
	/// Returns Result<SFile>
	///
	/// Note: Prefer the use of the SPath::try_from(...) when available as it might
	///       avoid a PathBuf allocation.
	pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
		let path = path.as_ref();

		validate_sfile_for_result(path)?;

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
		validate_sfile_for_option(path)?;

		Some(Self {
			path: path.to_path_buf(),
		})
	}

	/// Constructor from PathBuf returning an Option, none if validation fail.
	/// Useful for filter_map.
	pub fn from_path_buf_ok(path_buf: PathBuf) -> Option<Self> {
		validate_sfile_for_option(&path_buf)?;
		Some(Self { path: path_buf })
	}

	/// Constructor from fs::DirEntry returning an Option, none if validation fail.
	/// Useful for filter_map.
	pub fn from_fs_entry_ok(fs_entry: fs::DirEntry) -> Option<Self> {
		let path_buf = fs_entry.path();
		validate_sfile_for_option(&path_buf)?;
		Some(Self { path: path_buf })
	}

	/// Constructor from walkdir::DirEntry returning an Option, none if validation fail.
	/// Useful for filter_map.
	pub fn from_walkdir_entry_ok(wd_entry: walkdir::DirEntry) -> Option<Self> {
		let path = wd_entry.path();
		validate_sfile_for_option(path)?;
		Some(Self {
			path: wd_entry.into_path(),
		})
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

	/// Returns the &str representation of the file_stem()
	///
	/// NOTE: According to the constructors' contract, this method will never return ""
	///       as a file_name() is required for construction, and stem is always part of it.
	pub fn file_stem(&self) -> &str {
		self.path.file_stem().and_then(|n| n.to_str()).unwrap_or_default()
	}

	#[deprecated = "use file_stem(..)"]
	pub fn stem(&self) -> &str {
		self.file_stem()
	}

	/// Returns the Option<&str> representation of the extension().
	///
	/// NOTE: This should never be a non-UTF-8 string
	///       as the path was validated during SFile construction.
	pub fn extension(&self) -> Option<&str> {
		self.path.extension().and_then(|os_str| os_str.to_str())
	}

	/// Returns the extension or "" if no extension
	pub fn ext(&self) -> &str {
		self.extension().unwrap_or_default()
	}

	pub fn is_dir(&self) -> bool {
		self.path.is_dir()
	}

	pub fn is_file(&self) -> bool {
		self.path.is_file()
	}

	// Returns the path.metadata modified.
	pub fn modified(&self) -> Result<SystemTime> {
		let path = self.path();
		let metadata = fs::metadata(path).map_err(|ex| Error::CantGetMetadata((path, ex).into()))?;
		let last_modified = metadata
			.modified()
			.map_err(|ex| Error::CantGetMetadataModified((path, ex).into()))?;
		Ok(last_modified)
	}
}

// region:    --- Std Traits Impls

impl AsRef<Path> for SFile {
	fn as_ref(&self) -> &Path {
		self.path.as_ref()
	}
}

impl fmt::Display for SFile {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_str())
	}
}

impl From<SFile> for String {
	fn from(val: SFile) -> Self {
		val.to_str().to_string()
	}
}

impl From<&SFile> for String {
	fn from(val: &SFile) -> Self {
		val.to_str().to_string()
	}
}

// endregion: --- Std Traits Impls

// region:    --- TryFroms

impl TryFrom<&str> for SFile {
	type Error = Error;
	fn try_from(path: &str) -> Result<SFile> {
		let path = Path::new(path);
		validate_sfile_for_result(path)?;

		Ok(Self {
			path: path.to_path_buf(),
		})
	}
}

impl TryFrom<String> for SFile {
	type Error = Error;
	fn try_from(path: String) -> Result<SFile> {
		SFile::try_from(path.as_str())
	}
}

impl TryFrom<&String> for SFile {
	type Error = Error;
	fn try_from(path: &String) -> Result<SFile> {
		SFile::try_from(path.as_str())
	}
}

impl TryFrom<PathBuf> for SFile {
	type Error = Error;
	fn try_from(path_buf: PathBuf) -> Result<SFile> {
		validate_sfile_for_result(&path_buf)?;

		Ok(Self { path: path_buf })
	}
}

impl TryFrom<fs::DirEntry> for SFile {
	type Error = Error;
	fn try_from(fs_entry: fs::DirEntry) -> Result<SFile> {
		let path_buf = fs_entry.path();
		validate_sfile_for_result(&path_buf)?;

		Ok(Self { path: path_buf })
	}
}

impl TryFrom<walkdir::DirEntry> for SFile {
	type Error = Error;
	fn try_from(wd_entry: walkdir::DirEntry) -> Result<SFile> {
		let path = wd_entry.path();

		validate_sfile_for_result(path)?;

		Ok(Self {
			path: wd_entry.into_path(),
		})
	}
}

impl TryFrom<SPath> for SFile {
	type Error = Error;
	fn try_from(spath: SPath) -> Result<SFile> {
		let path = spath.path();

		validate_sfile_for_result(path)?;

		Ok(Self {
			path: spath.into_path_buf(),
		})
	}
}
// endregion: --- TryFroms

// region:    --- File Validation

fn validate_sfile_for_result(path: &Path) -> Result<()> {
	validate_spath_for_result(path)?;

	if path.is_file() {
		Ok(())
	} else {
		Err(Error::FileNotFound(path.to_string_lossy().to_string()))
	}
}

/// Validate but without generating an error (good for the _ok constructors)
fn validate_sfile_for_option(path: &Path) -> Option<()> {
	validate_spath_for_option(path)?;

	if path.is_file() {
		Some(())
	} else {
		None
	}
}

// endregion: --- File Validation
