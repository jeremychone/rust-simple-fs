// region:    --- Modules

mod dir;
mod error;
mod file;
mod glob;
mod globs_iter;
mod iter_files;
mod list_options;
mod sfile;
mod spath;
mod watch;

#[cfg(feature = "bin-nums")]
mod bin_nums;
#[cfg(feature = "with-json")]
mod with_json;
#[cfg(feature = "with-toml")]
mod with_toml;

pub use self::error::{Error, Result};

// -- Re-export everything for the root crate

pub use dir::*;
pub use file::*;
pub use glob::*;
pub use iter_files::*;
pub use list_options::*;
pub use sfile::*;
pub use spath::*;
pub use watch::*;

#[cfg(feature = "with-json")]
pub use with_json::*;

#[cfg(feature = "with-toml")]
pub use with_toml::*;

#[cfg(feature = "bin-nums")]
pub use bin_nums::*;

// endregion: --- Modules
