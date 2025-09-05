use crate::{Error, Result};
use crate::{get_buf_reader, get_buf_writer};
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use std::io::{Read, Write};
use std::path::Path;

// region:    --- Loaders

macro_rules! generate_load_functions {
    ( $( $type:ty, $size:expr, $load_be_fn_name:ident, $load_le_fn_name:ident, $load_fn:ident, $byteorder_read_fn:ident );* $(;)? ) => {
        $(
						pub fn $load_be_fn_name(file_path: impl AsRef<Path>) -> Result<Vec<$type>> {
							$load_fn(file_path.as_ref(), BigEndian::$byteorder_read_fn)
						}

						pub fn $load_le_fn_name(file_path: impl AsRef<Path>) -> Result<Vec<$type>> {
							$load_fn(file_path.as_ref(), LittleEndian::$byteorder_read_fn)
						}

            fn $load_fn(file_path: &Path, read_fn: fn(buf: &[u8]) -> $type) -> Result<Vec<$type>> {
                let mut reader = get_buf_reader(file_path)?;

                let mut data = Vec::new();
                let mut buf = [0u8; $size];
                while let Ok(()) = reader.read_exact(&mut buf) {
                    let val = read_fn(&buf);
                    data.push(val);
                }

                Ok(data)
            }
        )*
    };
}

generate_load_functions!(
	f64, 8, load_be_f64, load_le_f64, load_f64, read_f64;
	f32, 4, load_be_f32, load_le_f32, load_f32, read_f32;
	u64, 8, load_be_u64, load_le_u64, load_u64, read_u64;
	u32, 4, load_be_u32, load_le_u32, load_u32, read_u32;
	u16, 2, load_be_u16, load_le_u16, load_u16, read_u16;
	i64, 8, load_be_i64, load_le_i64, load_i64, read_i64;
	i32, 4, load_be_i32, load_le_i32, load_i32, read_i32;
	i16, 2, load_be_i16, load_le_i16, load_i16, read_i16;
);

// endregion: --- Loaders

// region:    --- Savers

macro_rules! generate_save_functions {
    ( $( $type:ty, $size:expr, $save_be_fn_name:ident, $save_le_fn_name:ident, $save_fn:ident, $byteorder_write_fn:ident );* $(;)? ) => {
$(
		pub fn $save_be_fn_name(file_path: impl AsRef<Path>, data: &[$type]) -> Result<()> {
			$save_fn(file_path.as_ref(), data, BigEndian::$byteorder_write_fn)
		}

		pub fn $save_le_fn_name(file_path: impl AsRef<Path>, data: &[$type]) -> Result<()> {
			$save_fn(file_path.as_ref(), data, LittleEndian::$byteorder_write_fn)
		}

		fn $save_fn(file_path: &Path, data: &[$type], write_fn: fn(buf: &mut [u8], n: $type)) -> Result<()> {
			let mut writer = get_buf_writer(file_path)?;

			let mut buf = [0; $size];
			for value in data {
				write_fn(&mut buf, *value);
				writer
					.write_all(&buf)
					.map_err(|e| Error::FileCantWrite((file_path, e).into()))?;
			}

			writer.flush().map_err(|e| Error::FileCantWrite((file_path, e).into()))?;
			Ok(())
		}
)*
    };
}

generate_save_functions!(
	f64, 8, save_be_f64, save_le_f64, save_f64, write_f64;
	f32, 4, save_be_f32, save_le_f32, save_f32, write_f32;
	u64, 8, save_be_u64, save_le_u64, save_u64, write_u64;
	u32, 4, save_be_u32, save_le_u32, save_u32, write_u32;
	u16, 2, save_be_u16, save_le_u16, save_u16, write_u16;
	i64, 8, save_be_i64, save_le_i64, save_i64, write_i64;
	i32, 4, save_be_i32, save_le_i32, save_i32, write_i32;
	i16, 2, save_be_i16, save_le_i16, save_i16, write_i16;
);

// endregion: --- Savers
