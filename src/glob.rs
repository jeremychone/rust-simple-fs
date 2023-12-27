use crate::{Error, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};

pub const DEFAULT_EXCLUDE_GLOBS: &[&str] = &["**/.git", "**/target", "**/node_modules"];

pub fn get_glob_set(globs: &[&str]) -> Result<GlobSet> {
	let mut builder = GlobSetBuilder::new();
	for &glob_str in globs {
		let glob = Glob::new(glob_str).map_err(|e| Error::GlobCantNew {
			glob: glob_str.to_string(),
			cause: e,
		})?;
		builder.add(glob);
	}

	let glob_set = builder.build().map_err(|e| Error::GlobSetCantBuild {
		globs: globs.iter().map(|&v| v.to_string()).collect(),
		cause: e,
	})?;

	Ok(glob_set)
}
