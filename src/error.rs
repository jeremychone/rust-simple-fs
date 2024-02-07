use derive_more::From;
use std::io;
use std::path::Path;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	// -- Path
	PathNotUtf8(String),
	PathHasNoFileName(String),

	// -- File
	FileNotFound(String),
	FileCantOpen(PathAndCause),
	FileCantRead(PathAndCause),
	FileCantWrite(PathAndCause),
	FileCantCreate(PathAndCause),
	FileHasNoParent(String),

	// -- Meta
	CantGetMetadata(PathAndCause),
	CantGetMetadataModified(PathAndCause),

	// -- Dir
	DirCantCreateAll(PathAndCause),

	// -- Path Validations
	PathNotValidForPath(PathAndCause),

	// -- Glob
	GlobCantNew {
		glob: String,
		cause: globset::Error,
	},
	GlobSetCantBuild {
		globs: Vec<String>,
		cause: globset::Error,
	},

	// -- Watch
	FailToWatch {
		path: String,
		cause: String,
	},
	CantWatchPathNotFound(String),

	// -- with-json
	#[cfg(feature = "with-json")]
	JsonCantRead(PathAndCause),
	#[cfg(feature = "with-json")]
	JsonCantWrite(PathAndCause),

	// -- with-toml
	#[cfg(feature = "with-toml")]
	TomlCantRead(PathAndCause),
	#[cfg(feature = "with-toml")]
	TomlCantWrite(PathAndCause),
}

// region:    --- Cause Types

#[derive(Debug)]
pub enum Cause {
	Io(Box<io::Error>),

	#[cfg(feature = "with-json")]
	SerdeJson(Box<serde_json::Error>),
	#[cfg(feature = "with-toml")]
	TomlDe(Box<toml::de::Error>),
	#[cfg(feature = "with-toml")]
	TomlSer(Box<toml::ser::Error>),
}

#[derive(Debug)]
#[allow(unused)]
pub struct PathAndCause {
	path: String,
	cause: Cause,
}

// endregion: --- Cause Types

// region:    --- Io

impl From<(&Path, io::Error)> for PathAndCause {
	fn from(val: (&Path, io::Error)) -> Self {
		PathAndCause {
			path: val.0.to_string_lossy().to_string(),
			cause: Cause::Io(Box::new(val.1)),
		}
	}
}

// endregion: --- Io

// region:    --- Json

#[cfg(feature = "with-json")]
impl From<(&Path, serde_json::Error)> for PathAndCause {
	fn from(val: (&Path, serde_json::Error)) -> Self {
		PathAndCause {
			path: val.0.to_string_lossy().to_string(),
			cause: Cause::SerdeJson(Box::new(val.1)),
		}
	}
}

// endregion: --- Json

// region:    --- Toml

#[cfg(feature = "with-toml")]
impl From<(&Path, toml::de::Error)> for PathAndCause {
	fn from(val: (&Path, toml::de::Error)) -> Self {
		PathAndCause {
			path: val.0.to_string_lossy().to_string(),
			cause: Cause::TomlDe(Box::new(val.1)),
		}
	}
}

#[cfg(feature = "with-toml")]
impl From<(&Path, toml::ser::Error)> for PathAndCause {
	fn from(val: (&Path, toml::ser::Error)) -> Self {
		PathAndCause {
			path: val.0.to_string_lossy().to_string(),
			cause: Cause::TomlSer(Box::new(val.1)),
		}
	}
}

// endregion: --- Toml

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
