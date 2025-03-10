use crate::{Error, Result};
use camino::{Utf8Path, Utf8PathBuf};
use core::fmt;
use pathdiff::diff_utf8_paths;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// An SPath can be constructed from a String, Path, io::DirEntry, or walkdir::DirEntry
/// and guarantees the path is UTF-8, simplifying many apis.
#[derive(Debug, Clone)]
pub struct SPath {
	pub(crate) path_buf: Utf8PathBuf,
}

/// Constructors that guarantee the SPath contract described in the struct
impl SPath {
	/// Constructor for SPath accepting anything that implements Into<Utf8PathBuf>.
	pub fn new(path: impl Into<Utf8PathBuf>) -> Self {
		Self { path_buf: path.into() }
	}

	/// Constructor from standard PathBuf.
	pub fn from_std_path_buf(path_buf: PathBuf) -> Result<Self> {
		let path_buf = validate_spath_for_result(path_buf)?;
		Ok(SPath::from(path_buf))
	}

	/// Constructor from standard Path and all impl AsRef<Path>.
	pub fn from_std_path(path: impl AsRef<Path>) -> Result<Self> {
		let path = path.as_ref();
		let path_buf = validate_spath_for_result(path)?;
		Ok(SPath::from(path_buf))
	}

	/// Constructor from walkdir::DirEntry
	pub fn from_walkdir_entry(wd_entry: walkdir::DirEntry) -> Result<Self> {
		let path = wd_entry.into_path();
		let path_buf = validate_spath_for_result(path)?;
		Ok(SPath::from(path_buf))
	}

	/// Constructor for anything that implements AsRef<Path>.
	///
	/// Returns Option<SPath>. Useful for filter_map.
	pub fn from_std_path_ok(path: impl AsRef<Path>) -> Option<Self> {
		let path = path.as_ref();
		let path_buf = validate_spath_for_option(path)?;
		Some(SPath::from(path_buf))
	}

	/// Constructed from PathBuf returns an Option, none if validation fails.
	/// Useful for filter_map.
	pub fn from_std_path_buf_ok(path_buf: PathBuf) -> Option<Self> {
		let path_buf = validate_spath_for_option(&path_buf)?;
		Some(SPath::from(path_buf))
	}

	/// Constructor from fs::DirEntry returning an Option, none if validation fails.
	/// Useful for filter_map.
	pub fn from_fs_entry_ok(fs_entry: fs::DirEntry) -> Option<Self> {
		let path_buf = fs_entry.path();
		let path_buf = validate_spath_for_option(&path_buf)?;
		Some(SPath::from(path_buf))
	}

	/// Constructor from walkdir::DirEntry returning an Option, none if validation fails.
	/// Useful for filter_map.
	pub fn from_walkdir_entry_ok(wd_entry: walkdir::DirEntry) -> Option<Self> {
		let path_buf = validate_spath_for_option(wd_entry.path())?;
		Some(SPath::from(path_buf))
	}
}

/// Public into path
impl SPath {
	/// Consumes the SPath and returns its PathBuf.
	pub fn into_std_path_buf(self) -> PathBuf {
		self.path_buf.into()
	}

	/// Returns a reference to the internal std Path.
	pub fn std_path(&self) -> &Path {
		self.path_buf.as_std_path()
	}

	/// Returns a reference to the internal Utf8Path.
	pub fn path(&self) -> &Utf8Path {
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
		self.path_buf.as_str()
	}

	/// Returns the Option<&str> representation of the `path.file_name()`
	///
	pub fn file_name(&self) -> Option<&str> {
		self.path_buf.file_name()
	}

	/// Returns the &str representation of the `path.file_name()`
	///
	/// Note: If no file name will be an empty string
	pub fn name(&self) -> &str {
		self.file_name().unwrap_or_default()
	}

	/// Returns the parent name, and empty static &str if no present
	pub fn parent_name(&self) -> &str {
		self.path_buf.parent().and_then(|p| p.file_name()).unwrap_or_default()
	}

	/// Returns the Option<&str> representation of the file_stem()
	///
	/// Note: if the `OsStr` cannot be made into utf8 will be None
	pub fn file_stem(&self) -> Option<&str> {
		self.path_buf.file_stem()
	}

	/// Returns the &str representation of the `file_name()`
	///
	/// Note: If no stem, will be an empty string
	pub fn stem(&self) -> &str {
		self.file_stem().unwrap_or_default()
	}

	/// Returns the Option<&str> representation of the extension().
	///
	/// NOTE: This should never be a non-UTF-8 string
	///       as the path was validated during SPath construction.
	pub fn extension(&self) -> Option<&str> {
		self.path_buf.extension()
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
		let path = self.std_path();
		let metadata = fs::metadata(path).map_err(|ex| Error::CantGetMetadata((path, ex).into()))?;
		let last_modified = metadata
			.modified()
			.map_err(|ex| Error::CantGetMetadataModified((path, ex).into()))?;
		Ok(last_modified)
	}

	/// Returns the epoch duration in microseconds.
	/// Note: The maximum UTC date would be approximately `2262-04-11`.
	///       Thus, for all intents and purposes, it is far enough to not worry.
	pub fn modified_us(&self) -> Result<i64> {
		let modified = self.modified()?;
		let since_the_epoch = modified
			.duration_since(UNIX_EPOCH)
			.map_err(Error::CantGetDurationSystemTimeError)?;

		let modified_us = since_the_epoch.as_micros().min(i64::MAX as u128) as i64;

		Ok(modified_us)
	}

	/// Returns true if the internal path is absolute.
	pub fn is_absolute(&self) -> bool {
		self.path_buf.is_absolute()
	}

	/// Returns true if the internal path is relative.
	pub fn is_relative(&self) -> bool {
		self.path_buf.is_relative()
	}
}

/// Transformers
impl SPath {
	/// This perform a OS Canonicalization.
	pub fn canonicalize(&self) -> Result<SPath> {
		let path = self
			.path_buf
			.canonicalize_utf8()
			.map_err(|err| Error::CannotCanonicalize((self.std_path(), err).into()))?;
		Ok(SPath::new(path))
	}

	/// Normalize a path without performing I/O.
	///
	/// All redundant separator and up-level references are collapsed.
	///
	/// However, this does not resolve links.
	pub fn normalize(&self) -> SPath {
		crate::normalize(self)
	}

	/// Same as [`normalize`] but consume and create a new SPath only if needed
	pub fn into_normalized(self) -> SPath {
		if self.is_normalized() { self } else { self.normalize() }
	}

	/// Same as [`normalize`] except that if
	/// `Component::Prefix`/`Component::RootDir` is encountered,
	/// or if the path points outside of current dir, returns `None`.
	pub fn try_normalize(&self) -> Option<SPath> {
		crate::try_normalize(self)
	}

	/// Return `true` if the path is normalized.
	///
	/// # Quirk
	///
	/// If the path does not start with `./` but contains `./` in the middle,
	/// then this function might returns `true`.
	pub fn is_normalized(&self) -> bool {
		crate::is_normalized(self)
	}

	/// Returns the parent directory as an Option<SPath>.
	pub fn parent(&self) -> Option<SPath> {
		self.path_buf.parent().map(SPath::from)
	}

	/// Joins the provided path with the current path and returns an SPath.
	pub fn join(&self, leaf_path: impl Into<Utf8PathBuf>) -> SPath {
		let path_buf = self.path_buf.join(leaf_path.into());
		SPath::from(path_buf)
	}

	/// Joins a standard Path to the path of this SPath.
	pub fn join_std_path(&self, leaf_path: impl AsRef<Path>) -> Result<SPath> {
		let leaf_path = leaf_path.as_ref();
		let joined = self.std_path().join(leaf_path);
		let path_buf = validate_spath_for_result(joined)?;
		Ok(SPath::from(path_buf))
	}

	/// Creates a new sibling SPath with the given leaf_path.
	pub fn new_sibling(&self, leaf_path: impl AsRef<str>) -> SPath {
		let leaf_path = leaf_path.as_ref();
		match self.path_buf.parent() {
			Some(parent_dir) => SPath::new(parent_dir.join(leaf_path)),
			None => SPath::new(leaf_path),
		}
	}

	/// Creates a new sibling SPath with the given standard path.
	pub fn new_sibling_std_path(&self, leaf_path: impl AsRef<Path>) -> Result<SPath> {
		let leaf_path = leaf_path.as_ref();

		match self.std_path().parent() {
			Some(parent_dir) => SPath::from_std_path(parent_dir.join(leaf_path)),
			None => SPath::from_std_path(leaf_path),
		}
	}

	pub fn diff(&self, base: impl AsRef<Utf8Path>) -> Option<SPath> {
		let base = base.as_ref();

		let diff_path = diff_utf8_paths(self, base);

		diff_path.map(SPath::from)
	}

	pub fn try_diff(&self, base: impl AsRef<Utf8Path>) -> Result<SPath> {
		self.diff(&base).ok_or_else(|| Error::CannotDiff {
			path: self.to_string(),
			base: base.as_ref().to_string(),
		})
	}
}

// region:    --- Std Traits Impls

impl fmt::Display for SPath {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_str())
	}
}

// endregion: --- Std Traits Impls

// region:    --- AsRefs

impl AsRef<SPath> for SPath {
	fn as_ref(&self) -> &SPath {
		self
	}
}

impl AsRef<Path> for SPath {
	fn as_ref(&self) -> &Path {
		self.path_buf.as_ref()
	}
}

impl AsRef<Utf8Path> for SPath {
	fn as_ref(&self) -> &Utf8Path {
		self.path_buf.as_ref()
	}
}

impl AsRef<str> for SPath {
	fn as_ref(&self) -> &str {
		self.to_str()
	}
}

// endregion: --- AsRefs

// region:    --- Froms (into other types)

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

impl From<SPath> for PathBuf {
	fn from(val: SPath) -> Self {
		val.into_std_path_buf()
	}
}

impl From<&SPath> for PathBuf {
	fn from(val: &SPath) -> Self {
		val.path_buf.clone().into()
	}
}

impl From<SPath> for Utf8PathBuf {
	fn from(val: SPath) -> Self {
		val.path_buf
	}
}

// endregion: --- Froms (into other types)

// region:    --- Froms

impl From<Utf8PathBuf> for SPath {
	fn from(path_buf: Utf8PathBuf) -> Self {
		SPath { path_buf }
	}
}

impl From<&Utf8Path> for SPath {
	fn from(path: &Utf8Path) -> Self {
		SPath { path_buf: path.into() }
	}
}

impl From<String> for SPath {
	fn from(path: String) -> Self {
		Self {
			path_buf: Utf8PathBuf::from(path),
		}
	}
}

impl From<&String> for SPath {
	fn from(path: &String) -> Self {
		Self {
			path_buf: Utf8PathBuf::from(path),
		}
	}
}

impl From<&str> for SPath {
	fn from(path: &str) -> Self {
		Self {
			path_buf: Utf8PathBuf::from(path),
		}
	}
}

// endregion: --- Froms

// region:    --- TryFrom

impl TryFrom<PathBuf> for SPath {
	type Error = Error;
	fn try_from(path_buf: PathBuf) -> Result<SPath> {
		SPath::from_std_path_buf(path_buf)
	}
}

impl TryFrom<fs::DirEntry> for SPath {
	type Error = Error;
	fn try_from(fs_entry: fs::DirEntry) -> Result<SPath> {
		SPath::from_std_path_buf(fs_entry.path())
	}
}

impl TryFrom<walkdir::DirEntry> for SPath {
	type Error = Error;
	fn try_from(wd_entry: walkdir::DirEntry) -> Result<SPath> {
		SPath::from_std_path(wd_entry.path())
	}
}

// endregion: --- TryFrom

// region:    --- Path Validation

pub(crate) fn validate_spath_for_result(path: impl Into<PathBuf>) -> Result<Utf8PathBuf> {
	let path = path.into();
	let path_buf =
		Utf8PathBuf::from_path_buf(path).map_err(|err| Error::PathNotUtf8(err.to_string_lossy().to_string()))?;
	Ok(path_buf)
}

/// Validate but without generating an error (good for the _ok constructors)
pub(crate) fn validate_spath_for_option(path: impl Into<PathBuf>) -> Option<Utf8PathBuf> {
	Utf8PathBuf::from_path_buf(path.into()).ok()
}

// endregion: --- Path Validation
