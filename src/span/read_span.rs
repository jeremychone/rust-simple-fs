use crate::{Error, Result, SPath, open_file};
use std::fs::File;
use std::io::{self, ErrorKind};

#[cfg(unix)]
use std::os::unix::fs::FileExt as _;
#[cfg(windows)]
use std::os::windows::fs::FileExt as _;

/// Read a (start,end) half-open span and return a string.
pub fn read_span(path: impl Into<SPath>, start: usize, end: usize) -> Result<String> {
	let len = end.checked_sub(start).ok_or(Error::SpanInvalidStartAfterEnd)?;

	let path = path.into();
	let file = open_file(&path)?;

	let res = read_exact_at(&file, start as u64, len).map_err(|err| Error::FileCantRead((&path, err).into()))?;

	let txt = String::from_utf8(res).map_err(|_| Error::SpanInvalidUtf8)?;

	Ok(txt)
}

// region:    --- Support

/// Read exactly `len` bytes starting at absolute file offset `offset` into a Vec.
fn read_exact_at(file: &File, offset: u64, len: usize) -> io::Result<Vec<u8>> {
	let mut buf = vec![0u8; len];
	let mut filled = 0usize;

	while filled < len {
		#[cfg(unix)]
		let n = file.read_at(&mut buf[filled..], offset + filled as u64)?;
		#[cfg(windows)]
		let n = file.seek_read(&mut buf[filled..], offset + filled as u64)?;

		if n == 0 {
			return Err(io::Error::new(
				ErrorKind::UnexpectedEof,
				"span exceeds file size (hit EOF)",
			));
		}
		filled += n;
	}
	Ok(buf)
}

// endregion: --- Support
