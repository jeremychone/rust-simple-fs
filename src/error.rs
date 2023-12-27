use derive_more::From;
use std::io;
use std::path::Path;

pub type Result<T> = core::result::Result<T, Error>;

// region:    --- Types

#[derive(Debug)]
pub struct PathAndIoCause {
	file: String,
	cause: io::Error,
}
impl From<(&Path, io::Error)> for PathAndIoCause {
	fn from(val: (&Path, io::Error)) -> Self {
		PathAndIoCause {
			file: val.0.to_string_lossy().to_string(),
			cause: val.1,
		}
	}
}

// -- with-json
#[cfg(feature = "with-json")]
#[derive(Debug)]
pub struct PathAndSerdeCause {
	file: String,
	cause: serde_json::Error,
}
#[cfg(feature = "with-json")]
impl From<(&Path, serde_json::Error)> for PathAndSerdeCause {
	fn from(val: (&Path, serde_json::Error)) -> Self {
		PathAndSerdeCause {
			file: val.0.to_string_lossy().to_string(),
			cause: val.1,
		}
	}
}

// -- with-toml
#[cfg(feature = "with-toml")]
#[derive(Debug, From)]
pub enum TomlError {
	De(toml::de::Error),
	Ser(toml::ser::Error),
}

#[cfg(feature = "with-toml")]
#[derive(Debug)]
pub struct PathAndTomlCause {
	file: String,
	cause: TomlError,
}
#[cfg(feature = "with-toml")]
impl From<(&Path, toml::de::Error)> for PathAndTomlCause {
	fn from(val: (&Path, toml::de::Error)) -> Self {
		PathAndTomlCause {
			file: val.0.to_string_lossy().to_string(),
			cause: val.1.into(),
		}
	}
}
#[cfg(feature = "with-toml")]
impl From<(&Path, toml::ser::Error)> for PathAndTomlCause {
	fn from(val: (&Path, toml::ser::Error)) -> Self {
		PathAndTomlCause {
			file: val.0.to_string_lossy().to_string(),
			cause: val.1.into(),
		}
	}
}

// endregion: --- Types

#[derive(Debug)]
pub enum Error {
	// -- File
	FileNotFound(String),
	FileCantOpen(PathAndIoCause),
	FileCantRead(PathAndIoCause),
	FileCantCreate(PathAndIoCause),

	// -- Dir
	DirCantCreateAll(PathAndIoCause),

	// -- Glob
	GlobCantNew {
		glob: String,
		cause: globset::Error,
	},
	GlobSetCantBuild {
		globs: Vec<String>,
		cause: globset::Error,
	},

	// -- with-json
	#[cfg(feature = "with-json")]
	JsonCantRead(PathAndSerdeCause),
	#[cfg(feature = "with-json")]
	JsonCantWrite(PathAndSerdeCause),

	// -- with-toml
	#[cfg(feature = "with-toml")]
	TomlCantRead(PathAndTomlCause),
	#[cfg(feature = "with-toml")]
	TomlCantWrite(PathAndTomlCause),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate
