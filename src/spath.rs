use crate::Result;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug)]
pub struct SPath {
	path: PathBuf,
}

impl SPath {
	pub fn from_entry(entry: DirEntry) -> Option<Self> {
		Self::from_path(entry.path())
	}

	pub fn from_path(path: impl Into<PathBuf>) -> Option<Self> {
		let path = path.into();
		if path.to_str().is_some() {
			Some(SPath { path })
		} else {
			None
		}
	}

	pub fn path(&self) -> &Path {
		&self.path
	}

	pub fn take(self) -> PathBuf {
		self.path
	}

	pub fn file_name(&self) -> &str {
		self.path.file_name().and_then(|n| n.to_str()).unwrap_or_default()
	}

	pub fn file_stem(&self) -> &str {
		self.path.file_stem().and_then(|n| n.to_str()).unwrap_or_default()
	}

	/// "" if no extension
	pub fn ext(&self) -> &str {
		self.path.extension().and_then(|n| n.to_str()).unwrap_or_default()
	}

	pub fn modified(&self) -> Result<SystemTime> {
		let modified = fs::metadata(&self.path).and_then(|m| m.modified())?;
		Ok(modified)
	}
}
