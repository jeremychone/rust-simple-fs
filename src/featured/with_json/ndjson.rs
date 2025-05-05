use crate::{Error, Result};
use serde_json::Value;
use std::io::{BufRead, Cursor};

// From &str using Cursor (reuses above)
pub fn parse_ndjson_iter(input: &str) -> impl Iterator<Item = Result<Value>> {
	let reader = Cursor::new(input);
	parse_ndjson_iter_from_reader(reader)
}

pub fn parse_ndjson(input: &str) -> Result<Vec<Value>> {
	let reader = Cursor::new(input);
	parse_ndjson_from_reader(reader)
}

// Full collector from BufRead
pub fn parse_ndjson_from_reader<R: BufRead>(reader: R) -> Result<Vec<Value>> {
	parse_ndjson_iter_from_reader(reader).collect()
}

// Core streaming parser
pub fn parse_ndjson_iter_from_reader<R: BufRead>(reader: R) -> impl Iterator<Item = Result<Value>> {
	reader.lines().enumerate().filter_map(|(index, line_result)| {
		match line_result {
			Ok(line) if line.trim().is_empty() => None, // skip empty
			Ok(line) => Some(serde_json::from_str::<Value>(&line).map_err(|e| {
				Error::NdJson(format!(
					"aip.file.load_ndjson - Failed to parse JSON on line {}. Cause: {}",
					index + 1,
					e
				))
			})),
			Err(e) => Some(Err(Error::NdJson(format!(
				"aip.file.load_ndjson - Failed to read line {}. Cause: {}",
				index + 1,
				e
			)))),
		}
	})
}
