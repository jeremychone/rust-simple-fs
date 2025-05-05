use crate::file::create_file;
use crate::{Error, Result};
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
