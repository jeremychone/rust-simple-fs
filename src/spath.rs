use crate::{Error, Result, SFile};
use core::fmt;
use pathdiff::diff_paths;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// An SPath can be constructed from a Path, io::DirEntry, or walkdir::DirEntry
/// and guarantees the following:
///
/// - The full path is UTF-8 valid.
/// - It does NOT have to have a file NAME (.e.g, './'). If no file_name, .file_name() will return ""
#[derive(Debug, Clone)]
pub struct SPath {
	pub(crate) path_buf: PathBuf,
}

/// Constructors that guarantee the SPath contract described in the struct
impl SPath {
	/// Constructor for SPath accepting anything that implements Into<PathBuf>.
	///
	/// Note: This is quite ergonomic and allows for avoiding a PathBuf allocation
	///       if a PathBuf is provided.
	pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
		let path = path.into();

		validate_spath_for_result(&path)?;

		Ok(Self { path_buf: path })
	}

	/// Constructor from Path and all impl AsRef<Path>.
	///
	/// Returns Result<SPath>
	///
	/// Note: Prefer the use of the SPath::try_from(...) or new when available as it might
	///       avoid a PathBuf allocation.
	pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
		let path = path.as_ref();

		validate_spath_for_result(path)?;

		Ok(Self {
			path_buf: path.to_path_buf(),
		})
	}

	/// Constructor for anything that implements AsRef<Path>.
	///
	/// Returns Option<SFile>. Useful for filter_map.
	///
	/// Note: Favor using concrete type functions like `SPath::from_path_buf_ok`
	///       when available.
	pub fn from_path_ok(path: impl AsRef<Path>) -> Option<Self> {
		let path = path.as_ref();
		validate_spath_for_option(path)?;

		Some(Self {
			path_buf: path.to_path_buf(),
		})
	}

	/// Constructed from PathBuf returns an Option, none if validation fails.
	/// Useful for filter_map.
	pub fn from_path_buf_ok(path_buf: PathBuf) -> Option<Self> {
		validate_spath_for_option(&path_buf)?;
		Some(Self { path_buf })
	}

	/// Constructor from fs::DirEntry returning an Option, none if validation fails.
	/// Useful for filter_map.
	pub fn from_fs_entry_ok(fs_entry: fs::DirEntry) -> Option<Self> {
		let path_buf = fs_entry.path();
		validate_spath_for_option(&path_buf)?;
		Some(Self { path_buf })
	}

	/// Constructor from walkdir::DirEntry returning an Option, none if validation fails.
	/// Useful for filter_map.
	pub fn from_walkdir_entry_ok(wd_entry: walkdir::DirEntry) -> Option<Self> {
		let path = wd_entry.path();
		validate_spath_for_option(path)?;
		Some(Self {
			path_buf: wd_entry.into_path(),
		})
	}
}

/// Public into path
impl SPath {
	/// Consumes the SPath and returns its PathBuf.
	pub fn into_path_buf(self) -> PathBuf {
		self.path_buf
	}

	/// Returns a reference to the internal Path.
	pub fn path(&self) -> &Path {
		&self.path_buf
	}
}

/// Public getters
impl SPath {
	/// Returns the &str of the path.
	///
	/// NOTE: We know that this must be Some() since the SPath constructor guarantees that
	///       the path.to_str() is valid.
	pub fn to_str(&self) -> &str {
		self.path_buf.to_str().unwrap_or_default()
	}

	/// Returns the Option<&str> representation of the `path.file_name()`
	///
	/// Note: if the `OsStr` cannot be made into utf8 will be None
	///
	pub fn file_name(&self) -> Option<&str> {
		self.path_buf.file_name().and_then(|n| n.to_str())
	}

	/// Returns the &str representation of the `path.file_name()`
	///
	/// Note: If no file name (e.g., `./`) or `OsStr` no utf8, will be empty string
	pub fn name(&self) -> &str {
		self.file_name().unwrap_or_default()
	}

	/// Returns the Option<&str> representation of the file_stem()
	///
	/// Note: if the `OsStr` cannot be made into utf8 will be None
	pub fn file_stem(&self) -> Option<&str> {
		self.path_buf.file_stem().and_then(|n| n.to_str())
	}

	/// Returns the &str representation of the `file_name()`
	///
	/// Note: If no file name (e.g., `./`) or `OsStr` no utf8, will be empty string
	pub fn stem(&self) -> &str {
		self.file_stem().unwrap_or_default()
	}

	/// Returns the Option<&str> representation of the extension().
	///
	/// NOTE: This should never be a non-UTF-8 string
	///       as the path was validated during SPath construction.
	pub fn extension(&self) -> Option<&str> {
		self.path_buf.extension().and_then(|os_str| os_str.to_str())
	}

	/// Returns the extension or "" if no extension
	pub fn ext(&self) -> &str {
		self.extension().unwrap_or_default()
	}

	/// Returns true if the path represents a directory.
	pub fn is_dir(&self) -> bool {
		self.path_buf.is_dir()
	}

	/// Returns true if the path represents a file.
	pub fn is_file(&self) -> bool {
		self.path_buf.is_file()
	}

	/// Checks if the path exists.
	pub fn exists(&self) -> bool {
		self.path_buf.exists()
	}

	/// Returns the path.metadata modified.
	pub fn modified(&self) -> Result<SystemTime> {
		let path = self.path();
		let metadata = fs::metadata(path).map_err(|ex| Error::CantGetMetadata((path, ex).into()))?;
		let last_modified = metadata
			.modified()
			.map_err(|ex| Error::CantGetMetadataModified((path, ex).into()))?;
		Ok(last_modified)
	}

	/// Returns the epoch duration in microseconds.
	/// Note: The maximum UTC date would be approximately `292277-01-09 04:00:54 UTC`.
	///       Thus, for all intents and purposes, it is far enough to not worry.
	pub fn modified_us(&self) -> Result<i64> {
		let modified = self.modified()?;
		let since_the_epoch = modified
			.duration_since(UNIX_EPOCH)
			.map_err(Error::CantGetDurationSystemTimeError)?;

		let modified_us = since_the_epoch.as_micros().min(i64::MAX as u128) as i64;

		Ok(modified_us)
	}
}

/// Transformers
impl SPath {
	/// Returns the parent directory as an Option<SPath>.
	pub fn parent(&self) -> Option<SPath> {
		self.path().parent().and_then(SPath::from_path_ok)
	}

	/// Joins the provided leaf_path with the current path and returns an SPath.
	pub fn join(&self, leaf_path: impl AsRef<Path>) -> Result<SPath> {
		let leaf_path = leaf_path.as_ref();
		let joined = self.path().join(leaf_path);
		validate_spath_for_result(&joined)?;

		Ok(SPath { path_buf: joined })
	}

	/// Creates a new sibling SPath with the given leaf_path.
	pub fn new_sibling(&self, leaf_path: impl AsRef<Path>) -> Result<SPath> {
		let leaf_path = leaf_path.as_ref();

		match self.path().parent() {
			Some(parent_dir) => SPath::new(parent_dir.join(leaf_path)),
			None => SPath::from_path(leaf_path),
		}
	}

	// Return a clean version of the path (meaning resolve the "../../")
	// Note: This does not resolve the path to the file system.
	//       So safe to use on non existant path.
	pub fn clean(&self) -> SPath {
		let path_buf = path_clean::clean(self);
		// Note: Here we can assume the result PathBuf is UTF8 compatible
		//       So, return empty Path cannot make it a spath
		SPath::from_path_buf_ok(path_buf).unwrap_or_else(|| SPath {
			path_buf: Path::new("").into(),
		})
	}

	pub fn diff(&self, base: impl AsRef<Path>) -> Result<SPath> {
		let base = base.as_ref();

		let diff_path = diff_paths(self, base).ok_or_else(|| Error::CannotDiff {
			path: self.to_string(),
			base: base.to_string_lossy().to_string(),
		})?;

		SPath::new(diff_path)
	}
}

// region:    --- Std Traits Impls

impl AsRef<Path> for SPath {
	fn as_ref(&self) -> &Path {
		self.path_buf.as_ref()
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

// region:    --- Froms

impl From<SFile> for SPath {
	fn from(sfile: SFile) -> Self {
		SPath { path_buf: sfile.into() }
	}
}

impl From<SPath> for PathBuf {
	fn from(val: SPath) -> Self {
		val.into_path_buf()
	}
}

impl From<&SPath> for PathBuf {
	fn from(val: &SPath) -> Self {
		val.path_buf.clone()
	}
}
// endregion: --- Froms

// region:    --- TryFrom

impl TryFrom<&str> for SPath {
	type Error = Error;
	fn try_from(path: &str) -> Result<SPath> {
		let path = Path::new(path);
		validate_spath_for_result(path)?;

		Ok(Self {
			path_buf: path.to_path_buf(),
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

		Ok(Self { path_buf })
	}
}

impl TryFrom<fs::DirEntry> for SPath {
	type Error = Error;
	fn try_from(fs_entry: fs::DirEntry) -> Result<SPath> {
		let path_buf = fs_entry.path();
		validate_spath_for_result(&path_buf)?;

		Ok(Self { path_buf })
	}
}

impl TryFrom<walkdir::DirEntry> for SPath {
	type Error = Error;
	fn try_from(wd_entry: walkdir::DirEntry) -> Result<SPath> {
		let path = wd_entry.path();

		validate_spath_for_result(path)?;

		Ok(Self {
			path_buf: wd_entry.into_path(),
		})
	}
}

// endregion: --- TryFrom

// region:    --- Path Validation

pub(crate) fn validate_spath_for_result(path: &Path) -> Result<()> {
	if path.to_str().is_none() {
		return Err(Error::PathNotUtf8(path.to_string_lossy().to_string()));
	}

	Ok(())
}

/// Validate but without generating an error (good for the _ok constructors)
pub(crate) fn validate_spath_for_option(path: &Path) -> Option<()> {
	path.to_str()?;

	Some(())
}

// endregion: --- Path Validation
