use crate::{Error, Result, SMeta, reshape};
use camino::{Utf8Path, Utf8PathBuf};
use core::fmt;
use pathdiff::diff_utf8_paths;
use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// An SPath is a posix normalized Path using camino Utf8PathBuf as strogate.
/// It can be constructed from a String, Path, io::DirEntry, or walkdir::DirEntry
///
/// - It's Posix normalized `/`, all redundant `//` and `/./` are removed
/// - Garanteed to be UTF8
#[derive(Debug, Clone)]
pub struct SPath {
	pub(crate) path_buf: Utf8PathBuf,
}

/// Constructors that guarantee the SPath contract described in the struct
impl SPath {
	/// Constructor for SPath accepting anything that implements Into<Utf8PathBuf>.
	/// IMPORTANT: This will normalize the path (posix style
	pub fn new(path: impl Into<Utf8PathBuf>) -> Self {
		let path_buf = path.into();
		let path_buf = reshape::into_normalized(path_buf);
		Self { path_buf }
	}

	/// Constructor from standard PathBuf.
	pub fn from_std_path_buf(path_buf: PathBuf) -> Result<Self> {
		let path_buf = validate_spath_for_result(path_buf)?;
		Ok(SPath::new(path_buf))
	}

	/// Constructor from standard Path and all impl AsRef<Path>.
	pub fn from_std_path(path: impl AsRef<Path>) -> Result<Self> {
		let path = path.as_ref();
		let path_buf = validate_spath_for_result(path)?;
		Ok(SPath::new(path_buf))
	}

	/// Constructor from walkdir::DirEntry
	pub fn from_walkdir_entry(wd_entry: walkdir::DirEntry) -> Result<Self> {
		let path = wd_entry.into_path();
		let path_buf = validate_spath_for_result(path)?;
		Ok(SPath::new(path_buf))
	}

	/// Constructor for anything that implements AsRef<Path>.
	///
	/// Returns Option<SPath>. Useful for filter_map.
	pub fn from_std_path_ok(path: impl AsRef<Path>) -> Option<Self> {
		let path = path.as_ref();
		let path_buf = validate_spath_for_option(path)?;
		Some(SPath::new(path_buf))
	}

	/// Constructed from PathBuf returns an Option, none if validation fails.
	/// Useful for filter_map.
	pub fn from_std_path_buf_ok(path_buf: PathBuf) -> Option<Self> {
		let path_buf = validate_spath_for_option(&path_buf)?;
		Some(SPath::new(path_buf))
	}

	/// Constructor from fs::DirEntry returning an Option, none if validation fails.
	/// Useful for filter_map.
	pub fn from_fs_entry_ok(fs_entry: fs::DirEntry) -> Option<Self> {
		let path_buf = fs_entry.path();
		let path_buf = validate_spath_for_option(&path_buf)?;
		Some(SPath::new(path_buf))
	}

	/// Constructor from walkdir::DirEntry returning an Option, none if validation fails.
	/// Useful for filter_map.
	pub fn from_walkdir_entry_ok(wd_entry: walkdir::DirEntry) -> Option<Self> {
		let path_buf = validate_spath_for_option(wd_entry.path())?;
		Some(SPath::new(path_buf))
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
	///       the path.as_str() is valid.
	#[deprecated(note = "use as_str()")]
	pub fn to_str(&self) -> &str {
		self.path_buf.as_str()
	}

	/// Returns the &str of the path.
	pub fn as_str(&self) -> &str {
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

	/// Returns true if the internal path is absolute.
	pub fn is_absolute(&self) -> bool {
		self.path_buf.is_absolute()
	}

	/// Returns true if the internal path is relative.
	pub fn is_relative(&self) -> bool {
		self.path_buf.is_relative()
	}
}

/// Meta
impl SPath {
	/// Get a Simple Metadata structure `SMeta` with
	/// created_epoch_us, modified_epoch_us, and size (all i64)
	/// (size will be '0' for any none file)
	#[allow(clippy::fn_to_numeric_cast)]
	pub fn meta(&self) -> Result<SMeta> {
		let path = self;

		let metadata = self.metadata()?;

		// -- Get modified (failed if it cannot)
		let modified = metadata.modified().map_err(|ex| Error::CantGetMetadata((path, ex).into()))?;
		let modified_epoch_us: i64 = modified
			.duration_since(UNIX_EPOCH)
			.map_err(|ex| Error::CantGetMetadata((path, ex).into()))?
			.as_micros()
			.min(i64::max as u128) as i64;

		// -- Get created (If not found, will get modified)
		let created_epoch_us = metadata
			.modified()
			.ok()
			.and_then(|c| c.duration_since(UNIX_EPOCH).ok())
			.map(|c| c.as_micros().min(i64::max as u128) as i64);
		let created_epoch_us = created_epoch_us.unwrap_or(modified_epoch_us);

		// -- Get size
		let size = if metadata.is_file() { metadata.len() } else { 0 };

		Ok(SMeta {
			created_epoch_us,
			modified_epoch_us,
			size,
			is_file: metadata.is_file(),
			is_dir: metadata.is_dir(),
		})
	}

	/// Returns the std metadata
	pub fn metadata(&self) -> Result<Metadata> {
		fs::metadata(self).map_err(|ex| Error::CantGetMetadata((self, ex).into()))
	}

	/// Returns the path.metadata modified SystemTime
	///
	#[deprecated = "use spath.meta()"]
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
	#[deprecated = "use spath.meta()"]
	pub fn modified_us(&self) -> Result<i64> {
		Ok(self.meta()?.modified_epoch_us)
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

	// region:    --- Collapse

	/// Collapse a path without performing I/O.
	///
	/// All redundant separator and up-level references are collapsed.
	///
	/// However, this does not resolve links.
	pub fn collapse(&self) -> SPath {
		let path_buf = crate::into_collapsed(self.path_buf.clone());
		SPath::new(path_buf)
	}

	/// Same as [`collapse`] but consume and create a new SPath only if needed
	pub fn into_collapsed(self) -> SPath {
		if self.is_collapsed() { self } else { self.collapse() }
	}

	/// Return `true` if the path is collapsed.
	///
	/// # Quirk
	///
	/// If the path does not start with `./` but contains `./` in the middle,
	/// then this function might returns `true`.
	pub fn is_collapsed(&self) -> bool {
		crate::is_collapsed(self)
	}

	// endregion: --- Collapse

	// region:    --- Parent & Join

	/// Returns the parent directory as an Option<SPath>.
	pub fn parent(&self) -> Option<SPath> {
		self.path_buf.parent().map(SPath::from)
	}

	/// Returns a new SPath with the given suffix appended to the filename (after the eventual extension)
	///
	/// Use [`join`] to join path segments.
	///
	/// Example:
	/// - `foo.rs` + `_backup` → `foo.rs_backup`
	pub fn append_suffix(&self, suffix: &str) -> SPath {
		SPath::new(format!("{self}{suffix}"))
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

	// endregion: --- Parent & Join

	// region:    --- Diff

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

	// endregion: --- Diff

	// region:    --- Replace

	pub fn replace_prefix(&self, base: impl AsRef<str>, with: impl AsRef<str>) -> SPath {
		let base = base.as_ref();
		let with = with.as_ref();
		let s = self.as_str();
		if let Some(stripped) = s.strip_prefix(base) {
			// Avoid introducing double slashes (is with.is_empty() because do not want to add a / if empty)
			let joined = if with.is_empty() || with.ends_with('/') || stripped.starts_with('/') {
				format!("{with}{stripped}")
			} else {
				format!("{with}/{stripped}")
			};
			SPath::new(joined)
		} else {
			self.clone()
		}
	}

	pub fn into_replace_prefix(self, base: impl AsRef<str>, with: impl AsRef<str>) -> SPath {
		let base = base.as_ref();
		let with = with.as_ref();
		let s = self.as_str();
		if let Some(stripped) = s.strip_prefix(base) {
			let joined = if with.is_empty() || with.ends_with('/') || stripped.starts_with('/') {
				format!("{with}{stripped}")
			} else {
				format!("{with}/{stripped}")
			};
			SPath::new(joined)
		} else {
			self
		}
	}

	// endregion: --- Replace
}

/// Path/UTF8Path/Camino passthrough
impl SPath {
	pub fn as_std_path(&self) -> &Path {
		self.std_path()
	}

	/// Returns a path that, when joined onto `base`, yields `self`.
	///
	/// # Errors
	///
	/// If `base` is not a prefix of `self`
	pub fn strip_prefix(&self, prefix: impl AsRef<Path>) -> Result<SPath> {
		let prefix = prefix.as_ref();
		let new_path = self.path_buf.strip_prefix(prefix).map_err(|_| Error::StripPrefix {
			prefix: prefix.to_string_lossy().to_string(),
			path: self.to_string(),
		})?;

		Ok(new_path.into())
	}

	/// Determines whether `base` is a prefix of `self`.
	///
	/// Only considers whole path components to match.
	///
	/// # Examples
	///
	/// ```
	/// use camino::Utf8Path;
	///
	/// let path = Utf8Path::new("/etc/passwd");
	///
	/// assert!(path.starts_with("/etc"));
	/// assert!(path.starts_with("/etc/"));
	/// assert!(path.starts_with("/etc/passwd"));
	/// assert!(path.starts_with("/etc/passwd/")); // extra slash is okay
	/// assert!(path.starts_with("/etc/passwd///")); // multiple extra slashes are okay
	///
	/// assert!(!path.starts_with("/e"));
	/// assert!(!path.starts_with("/etc/passwd.txt"));
	///
	/// assert!(!Utf8Path::new("/etc/foo.rs").starts_with("/etc/foo"));
	/// ```
	pub fn starts_with(&self, base: impl AsRef<Path>) -> bool {
		self.path_buf.starts_with(base)
	}
}

/// Extensions
impl SPath {
	/// Consumes the SPath and returns one with the given extension ensured:
	/// - Sets the extension if not already equal.
	/// - Returns self if the extension is already present.
	///
	/// ## Params
	/// - `ext` e.g. `html` (not . prefixed)
	pub fn into_ensure_extension(mut self, ext: &str) -> Self {
		if self.extension() != Some(ext) {
			self.path_buf.set_extension(ext);
		}
		self
	}

	/// Returns a new SPath with the given extension ensured.
	///
	/// - Since this takes a reference, it will return a Clone no matter what.
	/// - Use [`into_ensure_extension`] to consume and create a new SPath only if needed.
	///
	/// Delegates to `into_ensure_extension`.
	///
	/// ## Params
	/// - `ext` e.g. `html` (not . prefixed)
	pub fn ensure_extension(&self, ext: &str) -> Self {
		self.clone().into_ensure_extension(ext)
	}

	/// Appends the extension, even if one already exists or is the same.
	///
	/// ## Params
	/// - `ext` e.g. `html` (not . prefixed)
	pub fn append_extension(&self, ext: &str) -> Self {
		SPath::new(format!("{self}.{ext}"))
	}
}

/// Other
impl SPath {
	/// Returns a new SPath for the eventual directory before the first glob expression.
	///
	/// If not a glob, will return none
	///
	/// ## Examples
	/// - `/some/path/**/src/*.rs` → `/some/path`
	/// - `**/src/*.rs` → `""`
	/// - `/some/{src,doc}/**/*` → `/some`
	pub fn dir_before_glob(&self) -> Option<SPath> {
		let path_str = self.as_str();
		let mut last_slash_idx = None;

		for (i, c) in path_str.char_indices() {
			if c == '/' {
				last_slash_idx = Some(i);
			} else if matches!(c, '*' | '?' | '[' | '{') {
				return Some(SPath::from(&path_str[..last_slash_idx.unwrap_or(0)]));
			}
		}

		None
	}
}

// region:    --- Std Traits Impls

impl fmt::Display for SPath {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
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
		self.as_str()
	}
}

// endregion: --- AsRefs

// region:    --- Froms (into other types)

impl From<SPath> for String {
	fn from(val: SPath) -> Self {
		val.as_str().to_string()
	}
}

impl From<&SPath> for String {
	fn from(val: &SPath) -> Self {
		val.as_str().to_string()
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
		SPath::new(path_buf)
	}
}

impl From<&Utf8Path> for SPath {
	fn from(path: &Utf8Path) -> Self {
		SPath::new(path)
	}
}

impl From<String> for SPath {
	fn from(path: String) -> Self {
		SPath::new(path)
	}
}

impl From<&String> for SPath {
	fn from(path: &String) -> Self {
		SPath::new(path)
	}
}

impl From<&str> for SPath {
	fn from(path: &str) -> Self {
		SPath::new(path)
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
