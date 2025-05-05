use crate::file::create_file;
use crate::{Error, Result};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn save_json<T>(file: impl AsRef<Path>, data: &T) -> Result<()>
where
	T: serde::Serialize,
{
	save_json_impl(file.as_ref(), data, false)
}

pub fn save_json_pretty<T>(file: impl AsRef<Path>, data: &T) -> Result<()>
where
	T: serde::Serialize,
{
	save_json_impl(file.as_ref(), data, true)
}

fn save_json_impl<T>(file_path: &Path, data: &T, pretty: bool) -> Result<()>
where
	T: serde::Serialize,
{
	let file = create_file(file_path)?;

	let res = if pretty {
		serde_json::to_writer_pretty(file, data)
	} else {
		serde_json::to_writer(file, data)
	};

	res.map_err(|e| Error::JsonCantWrite((file_path, e).into()))?;

	Ok(())
}

/// Appends a `serde_json::Value` as a JSON line to the specified file.
/// Creates the file if it doesn't exist.
pub fn append_json_line<T: Serialize>(file: impl AsRef<Path>, value: &T) -> Result<()> {
	let file_path = file.as_ref();

	// Serialize the value to a JSON string first.
	let json_string = serde_json::to_string(value).map_err(|e| Error::JsonCantWrite((file_path, e).into()))?;

	// Open the file in append mode, creating it if necessary.
	let mut file = OpenOptions::new()
		.create(true) // Create the file if it doesn't exist
		.append(true) // Set append mode
		.open(file_path)
		.map_err(|e| Error::FileCantOpen((file_path, e).into()))?;

	// Write the JSON string followed by a newline character.
	writeln!(file, "{}", json_string).map_err(|e| Error::FileCantWrite((file_path, e).into()))?;

	Ok(())
}
