use crate::file::create_file;
use crate::{read_to_string, Error, Result};
use std::fs;
use std::path::Path;

pub fn load_toml<T>(file_path: impl AsRef<Path>) -> Result<T>
where
	T: serde::de::DeserializeOwned,
{
	let file_path = file_path.as_ref();
	let content = read_to_string(file_path)?;

	let res = toml::from_str(&content).map_err(|e| Error::TomlCantRead((file_path, e).into()))?;

	Ok(res)
}

pub fn save_toml<T>(file_path: impl AsRef<Path>, data: &T) -> Result<()>
where
	T: serde::Serialize,
{
	let file_path = file_path.as_ref();
	create_file(file_path)?;

	let toml_string = toml::to_string(data).map_err(|e| Error::TomlCantWrite((file_path, e).into()))?;
	fs::write(file_path, toml_string).map_err(|e| Error::TomlCantWrite((file_path, e).into()))?;

	Ok(())
}
