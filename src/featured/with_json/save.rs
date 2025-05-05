use crate::file::create_file;
use crate::{Error, Result};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::Path;

const JSON_LINES_BUFFER_SIZE: usize = 100;

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

/// Appends multiple `serde_json::Value` items as JSON lines to the specified file.
/// Creates the file if it doesn't exist. Writes in batches for efficiency.
pub fn append_json_lines<'a, T, I>(file: impl AsRef<Path>, values: I) -> Result<()>
where
	T: Serialize + 'a,
	I: IntoIterator<Item = &'a T>,
{
	let file_path = file.as_ref();

	// Open the file in append mode, creating it if necessary.
	let file = OpenOptions::new()
		.create(true) // Create the file if it doesn't exist
		.append(true) // Set append mode
		.open(file_path)
		.map_err(|e| Error::FileCantOpen((file_path, e).into()))?;

	let mut writer = BufWriter::new(file);
	let mut count = 0;

	for value in values {
		// Serialize the value to a JSON string.
		let json_string = serde_json::to_string(value).map_err(|e| Error::JsonCantWrite((file_path, e).into()))?;

		// Write the JSON string followed by a newline character to the buffer.
		writeln!(writer, "{}", json_string).map_err(|e| Error::FileCantWrite((file_path, e).into()))?;

		count += 1;

		// Flush the buffer periodically.
		if count % JSON_LINES_BUFFER_SIZE == 0 {
			writer.flush().map_err(|e| Error::FileCantWrite((file_path, e).into()))?;
		}
	}

	// Ensure any remaining lines in the buffer are written.
	writer.flush().map_err(|e| Error::FileCantWrite((file_path, e).into()))?;

	Ok(())
}
