use crate::SPath;
use derive_more::{Display, From};
use std::io;
use std::path::Path;
use std::time::SystemTimeError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Display)]
pub enum Error {
	// -- Path
	#[display("Path is not valid UTF-8: '{_0}'")]
	PathNotUtf8(String),
	#[display("Path has no file name: '{_0}'")]
	PathHasNoFileName(String),
	#[display("Strip Prefix fail. Path '{path}' does not a base of '{prefix}'")]
	StripPrefix {
		prefix: String,
		path: String,
	},

	// -- File
	#[display("File not found at path: '{_0}'")]
	FileNotFound(String),
	#[display("Cannot open file '{}'\nCause: {}", _0.path, _0.cause)]
	FileCantOpen(PathAndCause),
	#[display("Cannot read path '{}'\nCause: {}", _0.path, _0.cause)]
	FileCantRead(PathAndCause),
	#[display("Cannot write file '{}'\nCause: {}", _0.path, _0.cause)]
	FileCantWrite(PathAndCause),
	#[display("Cannot create file '{}'\nCause: {}", _0.path, _0.cause)]
	FileCantCreate(PathAndCause),
	#[display("File path has no parent directory: '{_0}'")]
	FileHasNoParent(String),

	// -- Remove
	#[display("File not safe to remove.\nPath: '{}'\nCause: {}", _0.path, _0.cause)]
	FileNotSafeToRemove(PathAndCause),
	#[display("Directory not safe to remove.\nPath: '{}'\nCause: {}", _0.path, _0.cause)]
	DirNotSafeToRemove(PathAndCause),

	// -- Sort
	#[display("Cannot sort by globs.\nCause: {cause}")]
	SortByGlobs {
		cause: String,
	},

	// -- Metadata
	#[display("Cannot get metadata for path '{}'\nCause: {}", _0.path, _0.cause)]
	CantGetMetadata(PathAndCause),
	#[display("Cannot get 'modified' metadata for path '{}'\nCause: {}", _0.path, _0.cause)]
	CantGetMetadataModified(PathAndCause),

	// -- Time
	#[display("Cannot get duration from system time. Cause: {_0}")]
	CantGetDurationSystemTimeError(SystemTimeError),

	// -- Directory
	#[display("Cannot create directory (and parents) '{}'\nCause: {}", _0.path, _0.cause)]
	DirCantCreateAll(PathAndCause),

	// -- Path Validations
	#[display("Path is invalid: '{}'\nCause: {}",_0.path, _0.cause)]
	PathNotValidForPath(PathAndCause),

	// -- Glob
	#[display("Cannot create glob pattern '{glob}'.\nCause: {cause}")]
	GlobCantNew {
		glob: String,
		cause: globset::Error,
	},
	#[display("Cannot build glob set from '{globs:?}'.\nCause: {cause}")]
	GlobSetCantBuild {
		globs: Vec<String>,
		cause: globset::Error,
	},

	// -- Watch
	#[display("Failed to watch path '{path}'.\nCause: {cause}")]
	FailToWatch {
		path: String,
		cause: String,
	},
	#[display("Cannot watch path because it was not found: '{_0}'")]
	CantWatchPathNotFound(String),

	// -- Span
	SpanInvalidStartAfterEnd,
	SpanOutOfBounds,
	SpanInvalidUtf8,

	// -- Other
	#[display("Cannot compute relative path from '{base}' to '{path}'")]
	CannotDiff {
		path: String,
		base: String,
	},
	#[display("Cannot Canonicalize path '{}'\nCause: {}", _0.path, _0.cause)]
	CannotCanonicalize(PathAndCause),

	// -- with-json
	#[cfg(feature = "with-json")]
	#[display("Cannot read json path '{}'\nCause: {}", _0.path, _0.cause)]
	JsonCantRead(PathAndCause),
	#[cfg(feature = "with-json")]
	#[display("Cannot write JSON to path '{}'\nCause: {}", _0.path, _0.cause)]
	JsonCantWrite(PathAndCause),
	#[cfg(feature = "with-json")]
	#[display("Error processing NDJSON: {_0}")]
	NdJson(String),

	// -- with-toml
	#[cfg(feature = "with-toml")]
	#[display("Cannot read TOML from path '{}'\nCause: {}", _0.path, _0.cause)]
	TomlCantRead(PathAndCause),
	#[cfg(feature = "with-toml")]
	#[display("Cannot write TOML to path '{}'\nCause: {}", _0.path, _0.cause)]
	TomlCantWrite(PathAndCause),
}

impl Error {
	pub fn sort_by_globs(cause: impl std::fmt::Display) -> Error {
		Error::SortByGlobs {
			cause: cause.to_string(),
		}
	}
}

// region:    --- Cause Types

#[derive(Debug, Display, From)]
pub enum Cause {
	#[from]
	Custom(String),

	#[from]
	Io(Box<io::Error>),

	#[cfg(feature = "with-json")]
	SerdeJson(Box<serde_json::Error>),

	#[cfg(feature = "with-toml")]
	TomlDe(Box<toml::de::Error>),

	#[cfg(feature = "with-toml")]
	TomlSer(Box<toml::ser::Error>),
}

#[derive(Debug)]
pub struct PathAndCause {
	pub path: String,
	pub cause: Cause,
}

// endregion: --- Cause Types

// region:    --- IO

impl From<(&Path, io::Error)> for PathAndCause {
	fn from(val: (&Path, io::Error)) -> Self {
		PathAndCause {
			path: val.0.to_string_lossy().to_string(),
			cause: Cause::Io(Box::new(val.1)),
		}
	}
}

impl From<(&SPath, io::Error)> for PathAndCause {
	fn from(val: (&SPath, io::Error)) -> Self {
		PathAndCause {
			path: val.0.to_string(),
			cause: Cause::Io(Box::new(val.1)),
		}
	}
}

//std::time::SystemTimeError
impl From<(&SPath, std::time::SystemTimeError)> for PathAndCause {
	fn from(val: (&SPath, std::time::SystemTimeError)) -> Self {
		PathAndCause {
			path: val.0.to_string(),
			cause: Cause::Custom(val.1.to_string()),
		}
	}
}

// endregion: --- IO

// region:    --- JSON

#[cfg(feature = "with-json")]
impl From<(&Path, serde_json::Error)> for PathAndCause {
	fn from(val: (&Path, serde_json::Error)) -> Self {
		PathAndCause {
			path: val.0.to_string_lossy().to_string(),
			cause: Cause::SerdeJson(Box::new(val.1)),
		}
	}
}

// endregion: --- JSON

// region:    --- TOML

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

// endregion: --- TOML

// region:    --- Error Boilerplate

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
