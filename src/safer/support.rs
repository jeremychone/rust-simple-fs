use crate::error::{Cause, PathAndCause};
use crate::{Error, Result, SPath};

/// Performs safety checks before deletion or trashing based on the provided options.
/// Returns a list of error causes if safety checks fail.
pub(crate) fn check_path_safety_causes(
	path: &SPath,
	restrict_to_current_dir: bool,
	must_contain_any: Option<&[&str]>,
	must_contain_all: Option<&[&str]>,
) -> Result<Vec<String>> {
	let mut error_causes = Vec::new();

	// Resolve the path to absolute
	let resolved = path.canonicalize()?;
	let resolved_str = resolved.as_str();
	let path_str = path.as_str();

	// Check that the path is below current directory (if enabled)
	if restrict_to_current_dir {
		let current_dir = std::env::current_dir().map_err(|e| {
			Error::CantGetMetadata(PathAndCause {
				path: path.to_string(),
				cause: Cause::Io(Box::new(e)),
			})
		})?;
		let current_dir_path = SPath::from_std_path_buf(current_dir)?;
		let current_resolved = current_dir_path.canonicalize()?;
		let current_str = current_resolved.as_str();

		if !resolved_str.starts_with(current_str) {
			error_causes.push(format!("is not below current directory '{current_resolved}'"));
		}
	}

	// Check must_contain_any
	if let Some(patterns) = must_contain_any {
		if patterns.is_empty() {
			error_causes.push("must_contain_any cannot be an empty list (use None to disable)".to_string());
		} else {
			let has_any = patterns.iter().any(|s| path_str.contains(s));
			if !has_any {
				error_causes.push(format!("does not contain any of the required patterns: {patterns:?}"));
			}
		}
	}

	// Check must_contain_all
	if let Some(patterns) = must_contain_all {
		if patterns.is_empty() {
			error_causes.push("must_contain_all cannot be an empty list (use None to disable)".to_string());
		} else {
			let missing: Vec<_> = patterns.iter().filter(|s| !path_str.contains(*s)).collect();
			if !missing.is_empty() {
				error_causes.push(format!("does not contain all required patterns, missing: {missing:?}"));
			}
		}
	}

	Ok(error_causes)
}
