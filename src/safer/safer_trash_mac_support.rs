use crate::SPath;
use crate::error::{Cause, PathAndCause};
use crate::{Error, Result};

#[cfg(target_os = "macos")]
pub(crate) fn try_silent_trash_move(path: &SPath) -> Result<bool> {
	use std::{
		fs,
		time::{SystemTime, UNIX_EPOCH},
	};

	// Find the trash directory
	let home = std::env::var("HOME").ok();
	let trash_dir = match home {
		Some(home) => {
			let tp = SPath::from(format!("{home}/.Trash"));
			if tp.is_dir() {
				tp
			} else {
				return Ok(false); // trash dir not found, fallback
			}
		}
		None => return Ok(false),
	};

	// Generate a unique destination name using a millisecond-precision timestamp
	let stem = path.file_stem().unwrap_or("unnamed");
	let ext = path.extension().unwrap_or("");
	let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
	let ts = format_trash_timestamp(now_ms);
	let dest_name = if ext.is_empty() {
		format!("{stem}--{ts}")
	} else {
		format!("{stem}--{ts}.{ext}")
	};
	let dest = trash_dir.join(dest_name);

	// Attempt the move
	match fs::rename(path.as_std_path(), dest.as_std_path()) {
		Ok(()) => {
			// Set the "Put Back" metadata so Finder can restore the item to its original location.
			// We intentionally ignore failures here: the item is already safely in the Trash,
			// and the only consequence of a failure is that "Put Back" won't be available.
			let _ = set_putback_metadata(path, &dest);
			Ok(true)
		}
		Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
			Ok(false) // fallback to trash crate
		}
		Err(e) => Err(Error::CantTrash(PathAndCause {
			path: path.to_string(),
			cause: Cause::Custom(format!("Silent trash move failed: {e}")),
		})),
	}
}

/// Sets the macOS "Put Back" extended attribute on the trashed item so Finder can
/// restore it to its original location (under its trashed name).
///
/// Finder reads the `com.apple.metadata:com.apple.trash.putback` extended attribute,
/// whose value is a binary plist encoding the original absolute path as a string.
///
/// `original_path` is the path the item lived at before being trashed, and `dest`
/// is its current location inside the Trash (where the attribute must be written).
#[cfg(target_os = "macos")]
fn set_putback_metadata(original_path: &SPath, dest: &SPath) -> Result<()> {
	// Resolve the original path to an absolute path. Finder expects the full original path.
	// At this point the original file no longer exists, so canonicalize its parent and re-join.
	let original_abs = resolve_original_abs_path(original_path);

	let plist = encode_binary_plist_string(&original_abs);

	let name = std::ffi::CString::new("com.apple.metadata:com.apple.trash.putback").map_err(|e| {
		Error::CantTrash(PathAndCause {
			path: dest.to_string(),
			cause: Cause::Custom(format!("Cannot build xattr name: {e}")),
		})
	})?;

	let dest_c = path_to_cstring(dest.as_std_path()).map_err(|e| {
		Error::CantTrash(PathAndCause {
			path: dest.to_string(),
			cause: Cause::Custom(format!("Cannot build dest path for xattr: {e}")),
		})
	})?;

	// SAFETY: All pointers are valid for the duration of the call. `plist` outlives the call,
	// and lengths are derived from the owning buffers.
	#[allow(unsafe_code)]
	let ret = unsafe {
		setxattr(
			dest_c.as_ptr(),
			name.as_ptr(),
			plist.as_ptr() as *const std::ffi::c_void,
			plist.len(),
			0,
			0,
		)
	};

	if ret != 0 {
		let err = std::io::Error::last_os_error();
		return Err(Error::CantTrash(PathAndCause {
			path: dest.to_string(),
			cause: Cause::Custom(format!("Cannot set put-back xattr: {err}")),
		}));
	}

	Ok(())
}

/// Resolves the original path to an absolute path string for the put-back metadata.
///
/// Since the file has typically already been moved out of its original location by the
/// time we record metadata, we canonicalize the original parent directory and re-append
/// the original file name. Falls back to a best-effort absolute join if that fails.
#[cfg(target_os = "macos")]
fn resolve_original_abs_path(original_path: &SPath) -> String {
	// Try parent canonicalization + file name (parent still exists after the move).
	if let (Some(parent), Some(file_name)) = (original_path.parent(), original_path.file_name())
		&& let Ok(parent_canon) = parent.canonicalize()
	{
		return parent_canon.join(file_name).to_string();
	}

	// Fallback: if already absolute, use as-is; otherwise join with current dir.
	if original_path.as_str().starts_with('/') {
		original_path.to_string()
	} else if let Ok(cwd) = std::env::current_dir()
		&& let Ok(cwd_spath) = SPath::from_std_path_buf(cwd)
	{
		cwd_spath.join(original_path.as_str()).to_string()
	} else {
		original_path.to_string()
	}
}

/// Encodes a single UTF-8 string as an Apple binary plist (`bplist00`).
///
/// The put-back metadata value is a binary plist whose root object is the original path
/// string. This produces the minimal valid bplist for that single-string case, which is
/// what Finder expects to read back.
#[cfg(target_os = "macos")]
fn encode_binary_plist_string(value: &str) -> Vec<u8> {
	// Binary plist layout:
	//   header:        "bplist00"
	//   object table:  the encoded objects (here, a single ASCII/Unicode string)
	//   offset table:  one offset per object
	//   trailer:       32 bytes describing the tables
	let mut buf: Vec<u8> = Vec::new();
	buf.extend_from_slice(b"bplist00");

	// Offset of the single (root) object within the file.
	let object_offset = buf.len() as u64;

	// Encode the string object. ASCII strings use the 0x5n marker (n = length),
	// non-ASCII strings use the 0x6n marker (UTF-16BE). We use ASCII when possible.
	if value.is_ascii() {
		write_plist_length_header(&mut buf, 0x50, value.len());
		buf.extend_from_slice(value.as_bytes());
	} else {
		let utf16: Vec<u16> = value.encode_utf16().collect();
		write_plist_length_header(&mut buf, 0x60, utf16.len());
		for unit in utf16 {
			buf.extend_from_slice(&unit.to_be_bytes());
		}
	}

	// Offset table: one entry (the root object). Each offset is 1 byte here since the
	// file is tiny; offset_size in the trailer reflects this.
	let offset_table_start = buf.len() as u64;
	let offset_size: u8 = 1;
	buf.push(object_offset as u8);

	// Trailer (32 bytes).
	let num_objects: u64 = 1;
	let top_object: u64 = 0;
	buf.extend_from_slice(&[0u8; 5]); // 5 unused bytes
	buf.push(0); // sort version
	buf.push(offset_size); // offset int size
	buf.push(8); // object ref size (8 is a safe default)
	buf.extend_from_slice(&num_objects.to_be_bytes());
	buf.extend_from_slice(&top_object.to_be_bytes());
	buf.extend_from_slice(&offset_table_start.to_be_bytes());

	buf
}

/// Writes a binary plist object marker and length.
///
/// For lengths < 15, the low nibble of `marker_base` carries the length directly.
/// For lengths >= 15, the low nibble is 0xF and an integer object encodes the length.
#[cfg(target_os = "macos")]
fn write_plist_length_header(buf: &mut Vec<u8>, marker_base: u8, len: usize) {
	if len < 15 {
		buf.push(marker_base | (len as u8));
	} else {
		buf.push(marker_base | 0x0F);
		// Encode the length as a big-endian integer object (0x1n, n = log2(byte count)).
		if len <= u8::MAX as usize {
			buf.push(0x10);
			buf.push(len as u8);
		} else if len <= u16::MAX as usize {
			buf.push(0x11);
			buf.extend_from_slice(&(len as u16).to_be_bytes());
		} else {
			buf.push(0x12);
			buf.extend_from_slice(&(len as u32).to_be_bytes());
		}
	}
}

/// Converts a filesystem path to a `CString` for FFI calls.
#[cfg(target_os = "macos")]
fn path_to_cstring(path: &std::path::Path) -> std::io::Result<std::ffi::CString> {
	use std::os::unix::ffi::OsStrExt;
	std::ffi::CString::new(path.as_os_str().as_bytes())
		.map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
}

// macOS `setxattr(2)` from libc (linked by default on macOS targets).
#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
unsafe extern "C" {
	fn setxattr(
		path: *const std::ffi::c_char,
		name: *const std::ffi::c_char,
		value: *const std::ffi::c_void,
		size: usize,
		position: u32,
		options: std::ffi::c_int,
	) -> std::ffi::c_int;
}

/// Formats a Unix timestamp in milliseconds (UTC) as `YYYY-MM-DD-HH-mm-ss-ms`,
/// with a 24-hour clock and all components zero-padded.
///
/// The date computation uses the civil-from-days algorithm (Howard Hinnant)
/// so it does not require any external date/time dependency.
#[cfg(target_os = "macos")]
fn format_trash_timestamp(epoch_ms: u64) -> String {
	let total_secs = epoch_ms / 1_000;
	let millis = epoch_ms % 1_000;

	let secs_of_day = total_secs % 86_400;
	let days = (total_secs / 86_400) as i64;

	let hour = secs_of_day / 3_600;
	let minute = (secs_of_day % 3_600) / 60;
	let second = secs_of_day % 60;

	// -- Civil date from days since Unix epoch (Howard Hinnant's algorithm)
	let z = days + 719_468;
	let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
	let doe = z - era * 146_097; // [0, 146096]
	let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
	let year_base = yoe + era * 400;
	let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
	let mp = (5 * doy + 2) / 153; // [0, 11]
	let day = (doy - (153 * mp + 2) / 5 + 1) as u64; // [1, 31]
	let month = if mp < 10 { mp + 3 } else { mp - 9 } as u64; // [1, 12]
	let year = (year_base + if month <= 2 { 1 } else { 0 }) as u64;

	format!("{year:04}-{month:02}-{day:02}-{hour:02}-{minute:02}-{second:02}-{millis:03}")
}
