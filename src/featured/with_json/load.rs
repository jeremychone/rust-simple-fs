use crate::{Error, Result, get_buf_reader};
use serde_json::Value;
use std::path::Path;

pub fn load_json<T>(file: impl AsRef<Path>) -> Result<T>
where
	T: serde::de::DeserializeOwned,
{
	let file = file.as_ref();

	let buf_reader = get_buf_reader(file)?;
	let val = serde_json::from_reader(buf_reader).map_err(|ex| Error::JsonCantRead((file, ex).into()))?;

	Ok(val)
}

/// Loads a ndjson (newline delimited json) file returning `Result<Vec<Value>>`.
/// Empty lines will be skipped.
pub fn load_ndjson(file: impl AsRef<Path>) -> Result<Vec<Value>> {
	let file = file.as_ref();
	let buf_reader = get_buf_reader(file)?;
	super::parse_ndjson_from_reader(buf_reader)
}

/// Returns an iterator over each line parsed as json `Result<Value>`.
/// Empty lines will be skipped.
pub fn stream_ndjson(file: impl AsRef<Path>) -> Result<impl Iterator<Item = Result<Value>>> {
	let file = file.as_ref();
	let buf_reader = get_buf_reader(file)?;
	Ok(super::parse_ndjson_iter_from_reader(buf_reader))
}
