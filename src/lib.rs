// region:    --- Modules

mod dir;
mod error;
mod glob;
mod iter_files;
mod reader;
mod sfile;
mod spath;

#[cfg(feature = "with-json")]
mod with_json;
#[cfg(feature = "with-toml")]
mod with_toml;

pub use self::error::{Error, Result};

// -- re-export all for root crate
pub use dir::*;
pub use glob::*;
pub use iter_files::*;
pub use reader::*;
pub use sfile::*;
pub use spath::*;

#[cfg(feature = "with-json")]
pub use with_json::*;

#[cfg(feature = "toml")]
pub use with_toml::*;

// endregion: --- Modules
