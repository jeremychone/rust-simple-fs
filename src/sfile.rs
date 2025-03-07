use crate::SPath;
use crate::{Error, Result};
use camino::{Utf8Path, Utf8PathBuf};
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
	/// Constructor for SFile accepting anything that implements Into<Utf8PathBuf>.
	pub fn new(path: impl Into<Utf8PathBuf>) -> Result<Self> {
		let path = SPath::new(path);
		validate_sfile_for_result(&path)?;
		Ok(Self { path })
	}

	/// Constructor from standard PathBuf.
	pub fn from_std_path_buf(path_buf: PathBuf) -> Result<Self> {
		let path = SPath::from_std_path_buf(path_buf)?;
		validate_sfile_for_result(&path)?;
		Ok(Self { path })
	}

	/// Constructor from standard Path and all impl AsRef<Path>.
	pub fn from_std_path(path: impl AsRef<Path>) -> Result<Self> {
		let path = SPath::from_std_path(path)?;
		validate_sfile_for_result(&path)?;
		Ok(Self { path })
	}

	/// Constructor from walkdir::DirEntry
	pub fn from_walkdir_entry(wd_entry: walkdir::DirEntry) -> Result<Self> {
		let path = SPath::from_walkdir_entry(wd_entry)?;
		validate_sfile_for_result(&path)?;
		Ok(Self { path })
	}

	/// Constructors for anything that implements AsRef<Path>.
	///
	/// Returns Option<SFile>. Useful for filter_map.
	pub fn from_std_path_ok(path: impl AsRef<Path>) -> Option<Self> {
		let path = SPath::from_std_path_ok(path)?;
		validate_sfile_for_option(&path)?;
		Some(Self { path })
	}

	/// Constructor from PathBuf returning an Option, none if validation fails.
	/// Useful for filter_map.
	pub fn from_std_path_buf_ok(path_buf: PathBuf) -> Option<Self> {
		let path = SPath::from_std_path_buf_ok(path_buf)?;
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

/// Public into path
impl SFile {
	/// Consumes the SFile and returns its PathBuf.
	pub fn into_std_path_buf(self) -> PathBuf {
		self.path.into_std_path_buf()
	}

	/// Returns a reference to the internal standard Path.
	pub fn std_path(&self) -> &Path {
		self.path.std_path()
	}

	/// Returns a reference to the internal Utf8Path.
	pub fn path(&self) -> &SPath {
		&self.path
	}
}

/// Public getters
impl SFile {
	/// Returns the &str of the path.
	///
	/// NOTE: We know that this must be Some() since the SFile constructor guarantees that
	///       the path.to_str() is valid.
	pub fn to_str(&self) -> &str {
		self.path.to_str()
	}

	/// Returns the Option<&str> representation of the `path.file_name()`.
	pub fn file_name(&self) -> Option<&str> {
		self.path.file_name()
	}

	/// Returns the &str representation of the `path.file_name()`.
	///
	/// Note: If no file name will be an empty string
	pub fn name(&self) -> &str {
		self.path.name()
	}

	/// Returns the parent name, and empty static &str if no present
	pub fn parent_name(&self) -> &str {
		self.path.parent_name()
	}

	/// Returns the Option<&str> representation of the file_stem().
	pub fn file_stem(&self) -> Option<&str> {
		self.path.file_stem()
	}

	/// Returns the &str representation of the `file_name()`.
	///
	/// Note: If no stem, will be an empty string
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
	/// Note: The maximum UTC date would be approximately `2262-04-11`.
	///       Thus, for all intents and purposes, it is far enough not to worry.
	pub fn modified_us(&self) -> Result<i64> {
		self.path.modified_us()
	}

	/// Returns the file size in bytes as `i64`.
	/// Note: In the highly unlikely event that the size exceeds `i64::MAX`,
	///       `i64::MAX` is returned. `i64::MAX` represents 8,388,607 terabytes,
	///       providing ample margin before it becomes a concern.
	pub fn file_size(&self) -> Result<i64> {
		let path = self.std_path();
		let metadata = fs::metadata(path).map_err(|ex| Error::CantGetMetadata((path, ex).into()))?;
		let size = match metadata.len().try_into() {
			Ok(v) => v,
			Err(_) => i64::MAX,
		};
		Ok(size)
	}

	/// Returns true if the internal path is absolute.
	pub fn is_absolute(&self) -> bool {
		self.path.is_absolute()
	}

	/// Returns true if the internal path is relative.
	pub fn is_relative(&self) -> bool {
		self.path.is_relative()
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
	pub fn parent(&self) -> Option<SPath> {
		self.path.parent()
	}

	/// Joins the current path with the specified leaf_path.
	///
	/// This method creates a new path by joining the existing path with a specified leaf_path
	/// and returns the result as an SPath.
	pub fn join(&self, leaf_path: impl Into<Utf8PathBuf>) -> SPath {
		self.path.join(leaf_path)
	}

	/// Joins a standard Path to the path of this SFile.
	pub fn join_std_path(&self, leaf_path: impl AsRef<Path>) -> Result<SPath> {
		self.path.join_std_path(leaf_path)
	}

	/// Creates a new sibling path with the specified leaf_path.
	///
	/// Generates a new path in the same parent directory as the current file, appending the leaf_path.
	pub fn new_sibling(&self, leaf_path: &str) -> SPath {
		self.path.new_sibling(leaf_path)
	}

	/// Creates a new sibling path with the specified standard path.
	pub fn new_sibling_std_path(&self, leaf_path: impl AsRef<Path>) -> Result<SPath> {
		self.path.new_sibling_std_path(leaf_path)
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

// region:    --- AsRefs

impl AsRef<SFile> for SFile {
	fn as_ref(&self) -> &SFile {
		self
	}
}

impl AsRef<Utf8Path> for SFile {
	fn as_ref(&self) -> &Utf8Path {
		self.path.as_ref()
	}
}

impl AsRef<str> for SFile {
	fn as_ref(&self) -> &str {
		self.to_str()
	}
}

impl AsRef<SPath> for SFile {
	fn as_ref(&self) -> &SPath {
		&self.path
	}
}

// endregion: --- AsRefs

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
		val.into_std_path_buf()
	}
}

impl From<&SFile> for PathBuf {
	fn from(val: &SFile) -> Self {
		val.std_path().to_path_buf()
	}
}

impl From<SFile> for Utf8PathBuf {
	fn from(val: SFile) -> Self {
		val.path.path_buf
	}
}

impl From<SFile> for SPath {
	fn from(val: SFile) -> Self {
		val.path
	}
}

impl From<&SFile> for SPath {
	fn from(val: &SFile) -> Self {
		val.path.clone()
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
		SFile::from_std_path_buf(path_buf)
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
		Err(Error::FileNotFound(path.to_str().to_string()))
	}
}

/// Validate but without generating an error (good for the _ok constructors)
fn validate_sfile_for_option(path: &SPath) -> Option<()> {
	if path.is_file() { Some(()) } else { None }
}

// endregion: --- File Validation
