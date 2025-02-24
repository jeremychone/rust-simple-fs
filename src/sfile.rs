use crate::SPath;
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
#[derive(Debug, Clone)]
pub struct SFile {
	path: SPath,
}

/// Constructors that guarantee the SFile contract described in the struct
impl SFile {
	/// Constructor for SFile accepting anything that implements Into<&PathBuf>.
	///
	/// Note: This is quite ergonomic and allows for avoiding a PathBuf allocation
	///       if a PathBuf is provided.
	pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
		let path = SPath::new(path)?;

		validate_sfile_for_result(&path)?;

		Ok(Self { path })
	}

	/// Constructor from File and all impl AsRef<&Path>.
	///
	/// Returns Result<SFile>
	///
	/// Note: Prefers the use of the SPath::try_from(...) when available, as it might
	///       avoid a PathBuf allocation.
	pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
		let path = SPath::from_path(path)?;

		validate_sfile_for_result(&path)?;

		Ok(Self { path })
	}

	/// Constructor from walkdir::DirEntry
	pub fn from_walkdir_entry(wd_entry: walkdir::DirEntry) -> Result<Self> {
		let path = SPath::from_walkdir_entry(wd_entry)?;
		validate_sfile_for_result(&path)?;
		Ok(Self { path })
	}

	/// Constructors for anything that implements AsRef<&Path>.
	///
	/// Returns Option<SFile>. Useful for filter_map.
	///
	/// Note: Favor using concrete type functions like `SPath::from_path_buf_ok`
	///       when available.
	pub fn from_path_ok(path: impl AsRef<Path>) -> Option<Self> {
		let path = SPath::from_path_ok(path)?;
		validate_sfile_for_option(&path)?;

		Some(Self { path })
	}

	/// Constructor from PathBuf returning an Option, none if validation fails.
	/// Useful for filter_map.
	pub fn from_path_buf_ok(path_buf: PathBuf) -> Option<Self> {
		let path = SPath::from_path_buf_ok(path_buf)?;

		validate_sfile_for_option(&path)?;
		Some(Self { path })
	}

	/// Constructor from fs::DirEntry returning an Option; none if validation fails.
	/// Useful for filter_map.
	pub fn from_fs_entry_ok(fs_entry: fs::DirEntry) -> Option<Self> {
		let path = SPath::from_fs_entry_ok(fs_entry)?;
		validate_sfile_for_option(&path)?;
		Some(Self { path })
	}

	/// Constructor from walkdir::DirEntry returning an Option; none if validation fails.
	/// Useful for filter_map.
	pub fn from_walkdir_entry_ok(wd_entry: walkdir::DirEntry) -> Option<Self> {
		let path = SPath::from_walkdir_entry_ok(wd_entry)?;
		validate_sfile_for_option(&path)?;
		Some(Self { path })
	}
}

/// Public return Path constructs.
impl SFile {
	/// Converts SFile into a PathBuf.
	///
	/// Takes ownership of the SFile and returns the underlying PathBuf.
	pub fn into_path_buf(self) -> PathBuf {
		self.path.path_buf
	}

	/// Returns a reference to the Path.
	///
	/// Accesses the internal path of the SFile without transferring ownership.
	pub fn path(&self) -> &Path {
		self.path.path()
	}
}

/// Public file components as str methods.
impl SFile {
	/// Returns the &str of the path.
	///
	/// NOTE: We know that this must be Some() since the SFile constructor guarantees that
	///       the path.to_str() is valid.
	pub fn to_str(&self) -> &str {
		self.path.to_str()
	}

	/// Returns the Option<&str> representation of the `path.file_name()`.
	///
	/// Note: if the `OsStr` cannot be made into UTF-8, will be None.
	///
	pub fn file_name(&self) -> Option<&str> {
		self.path.file_name()
	}

	/// Returns the &str representation of the `path.file_name()`.
	///
	/// Note: If no file name (e.g., `./`) or `OsStr` is not UTF-8, will be an empty string
	pub fn name(&self) -> &str {
		self.file_name().unwrap_or_default()
	}

	/// Returns the parent name, and empty static &str if no present
	pub fn parent_name(&self) -> &str {
		self.path()
			.parent()
			.and_then(|p| p.file_name())
			.and_then(|n| n.to_str())
			.unwrap_or_default()
	}

	/// Returns the Option<&str> representation of the file_stem().
	///
	/// Note: if the `OsStr` cannot be made into UTF-8, will be None.
	pub fn file_stem(&self) -> Option<&str> {
		self.path.file_stem()
	}

	/// Returns the &str representation of the `file_name()`.
	///
	/// Note: If no file name (e.g., `./`) or `OsStr` is not UTF-8, will be an empty string
	pub fn stem(&self) -> &str {
		self.path.stem()
	}

	/// Returns the Option<&str> representation of the extension().
	///
	/// NOTE: This should never be a non-UTF-8 string
	///       as the path was validated during SFile construction.
	pub fn extension(&self) -> Option<&str> {
		self.path.extension()
	}

	/// Same as `.extension()` but returns "" if no extension.
	pub fn ext(&self) -> &str {
		self.path.ext()
	}

	/// Returns the path.metadata modified as SystemTime.
	pub fn modified(&self) -> Result<SystemTime> {
		self.path.modified()
	}

	/// Returns the epoch duration in microseconds.
	/// Note: The maximum UTC date would be approximately `292277-01-09 04:00:54 UTC`.
	///       Thus, for all intents and purposes, it is far enough not to worry.
	pub fn modified_us(&self) -> Result<i64> {
		self.path.modified_us()
	}

	/// Returns the file size in bytes as `i64`.
	/// Note: In the highly unlikely event that the size exceeds `i64::MAX`,
	///       `i64::MAX` is returned. `i64::MAX` represents 8,388,607 terabytes,
	///       providing ample margin before it becomes a concern.
	pub fn file_size(&self) -> Result<i64> {
		let path = self.path();
		let metadata = fs::metadata(path).map_err(|ex| Error::CantGetMetadata((path, ex).into()))?;
		let size = match metadata.len().try_into() {
			Ok(v) => v,
			Err(_) => i64::MAX,
		};
		Ok(size)
	}
}

/// Transformers
impl SFile {
	pub fn canonicalize(&self) -> Result<SFile> {
		let path = self.path.canonicalize()?;
		// Note: here since the previous path was valid, if the spath canonicalization passes,
		//       we are ok.
		Ok(SFile { path })
	}

	/// Returns the parent directory as SPath, if available.
	///
	/// If the SFile has a parent directory, converts it to SPath and returns it.
	pub fn parent(&self) -> Option<SPath> {
		self.path.parent()
	}

	/// Joins the current path with the specified leaf_path.
	///
	/// This method creates a new path by joining the existing path with a specified leaf_path
	/// and returns the result as an SPath.
	pub fn join(&self, leaf_path: impl AsRef<Path>) -> Result<SPath> {
		self.path.join(leaf_path)
	}

	/// Creates a new sibling path with the specified leaf_path.
	///
	/// Generates a new path in the same parent directory as the current file, appending the leaf_path.
	pub fn new_sibling(&self, leaf_path: impl AsRef<Path>) -> Result<SPath> {
		self.path.new_sibling(leaf_path)
	}

	// Return a clean version of the path (meaning resolve the "../../").
	// Note: This does not resolve the path to the file system.
	//       So safe to use on a non-existent path.
	pub fn clean(&self) -> SPath {
		self.path.clean()
	}

	pub fn diff(&self, base: impl AsRef<Path>) -> Result<SPath> {
		self.path.diff(base)
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

// endregion: --- Std Traits Impls

// region:    --- Froms

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

impl From<SFile> for PathBuf {
	fn from(val: SFile) -> Self {
		val.into_path_buf()
	}
}

impl From<&SFile> for PathBuf {
	fn from(val: &SFile) -> Self {
		val.path.path_buf.clone()
	}
}

// endregion: --- Froms

// region:    --- TryFroms

impl TryFrom<&str> for SFile {
	type Error = Error;
	fn try_from(path: &str) -> Result<SFile> {
		let path = SPath::from(path);
		validate_sfile_for_result(&path)?;

		Ok(Self { path })
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
		let path = SPath::new(path_buf)?;
		validate_sfile_for_result(&path)?;

		Ok(Self { path })
	}
}

impl TryFrom<fs::DirEntry> for SFile {
	type Error = Error;
	fn try_from(fs_entry: fs::DirEntry) -> Result<SFile> {
		let path = SPath::try_from(fs_entry)?;
		validate_sfile_for_result(&path)?;

		Ok(Self { path })
	}
}

impl TryFrom<walkdir::DirEntry> for SFile {
	type Error = Error;
	fn try_from(wd_entry: walkdir::DirEntry) -> Result<SFile> {
		let path = SPath::try_from(wd_entry)?;
		validate_sfile_for_result(&path)?;

		Ok(Self { path })
	}
}

impl TryFrom<SPath> for SFile {
	type Error = Error;
	fn try_from(path: SPath) -> Result<SFile> {
		validate_sfile_for_result(&path)?;

		Ok(Self { path })
	}
}
// endregion: --- TryFroms

// region:    --- File Validation

fn validate_sfile_for_result(path: &SPath) -> Result<()> {
	if path.is_file() {
		Ok(())
	} else {
		Err(Error::FileNotFound(path.path_buf.to_string_lossy().to_string()))
	}
}

/// Validate but without generating an error (good for the _ok constructors)
fn validate_sfile_for_option(path: &SPath) -> Option<()> {
	if path.is_file() {
		Some(())
	} else {
		None
	}
}

// endregion: --- File Validation
