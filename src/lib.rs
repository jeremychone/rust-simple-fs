// region:    --- Modules

mod dir;
mod error;
mod featured;
mod file;
mod list;
mod reshape;
mod sfile;
mod spath;
mod watch;

pub use self::error::{Error, Result};

// -- Re-export everything for the root crate

pub use dir::*;
#[allow(unused)]
pub use featured::*;
pub use file::*;
pub use list::*;
pub use reshape::*;
pub use sfile::*;
pub use spath::*;
pub use watch::*;

// endregion: --- Modules

const TOP_MAX_DEPTH: usize = 100;
