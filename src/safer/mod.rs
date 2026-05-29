// region:    --- Modules

mod safer_remove_impl;
mod safer_remove_options;
mod safer_trash_impl;
#[cfg(target_os = "macos")]
mod safer_trash_mac_support;
mod safer_trash_options;
mod support;

pub use safer_remove_impl::*;
pub use safer_remove_options::*;
pub use safer_trash_impl::*;
pub use safer_trash_options::*;

// endregion: --- Modules
