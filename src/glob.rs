use crate::{Error, Result};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};

pub const DEFAULT_EXCLUDE_GLOBS: &[&str] = &["**/.git", "**/target", "**/node_modules"];

pub fn get_glob_set(globs: &[&str]) -> Result<GlobSet> {
	let mut builder = GlobSetBuilder::new();

	for &glob_str in globs {
		let glob = GlobBuilder::new(glob_str)
			// NOTE: Important to set to true, otherwise single "*" will pass through "/".
			.literal_separator(true)
			.build()
			.map_err(|e| Error::GlobCantNew {
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

pub fn longest_base_path_wild_free(pattern: &str) -> PathBuf {
	let path = Path::new(pattern);
	let mut base_path = PathBuf::new();

	for component in path.components() {
		let component_str = component.as_os_str().to_string_lossy();
		if component_str.contains('*') || component_str.contains('?') {
			break;
		}
		base_path.push(component);
	}

	base_path
}
