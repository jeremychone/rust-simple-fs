use crate::{Error, Result};
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

pub fn read_to_string(file_path: impl AsRef<Path>) -> Result<String> {
	let file_path = file_path.as_ref();

	if !file_path.is_file() {
		return Err(Error::FileNotFound(file_path.to_string_lossy().to_string()));
	}

	let content = fs::read_to_string(file_path).map_err(|e| Error::FileCantRead((file_path, e).into()))?;

	Ok(content)
}

pub fn get_buf_reader(file: impl AsRef<Path>) -> Result<BufReader<File>> {
	let file = file.as_ref();

	let file = File::open(file).map_err(|e| Error::FileCantOpen((file, e).into()))?;

	Ok(BufReader::new(file))
}
