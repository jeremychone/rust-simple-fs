use crate::{get_buf_reader, Error, Result};
use std::fs::File;
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
	let file = File::create(file_path).map_err(|e| Error::FileCantCreate((file_path, e).into()))?;

	let res = if pretty {
		serde_json::to_writer_pretty(file, data)
	} else {
		serde_json::to_writer(file, data)
	};

	res.map_err(|e| Error::JsonCantWrite((file_path, e).into()))?;

	Ok(())
}
