// region:    --- Modules

mod common;
mod dir;
mod error;
mod featured;
mod file;
mod list;
mod reshape;
mod safer;
mod sfile;
mod span;
mod spath;
mod watch;

pub use self::error::{Error, Result};

// -- Re-export everything for the root crate

pub use common::*;
pub use dir::*;
pub use file::*;
pub use list::*;
pub use reshape::*;
pub use safer::*;
pub use sfile::*;
pub use span::*;
pub use spath::*;
pub use watch::*;

#[allow(unused)]
pub use featured::*;

// endregion: --- Modules

const TOP_MAX_DEPTH: usize = 100;
