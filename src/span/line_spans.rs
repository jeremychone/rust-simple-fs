use crate::spath::SPath;
use crate::{Error, Result, open_file};
use memchr::memchr_iter;
use std::io::{self, Read};

/// Return byte ranges [start, end) for each line in the file at `path`,
/// splitting on '\n' and trimming a preceding '\r' (CRLF) even across chunk boundaries.
/// Runs in O(n) time, streaming; does not allocate the whole file.
pub fn line_spans(path: impl AsRef<SPath>) -> Result<Vec<(usize, usize)>> {
	let path = path.as_ref();
	let mut f = open_file(path)?;
	let res = line_spans_from_reader(&mut f).map_err(|err| Error::FileCantRead((path, err).into()))?;
	Ok(res)
}

// region:    --- Support

/// Same logic over any `Read` (useful for pipes).
fn line_spans_from_reader<R: Read>(r: &mut R) -> io::Result<Vec<(usize, usize)>> {
	let mut spans: Vec<(usize, usize)> = Vec::new();

	// 64 KiB chunks are a good balance for cache and syscalls.
	let mut buf = [0u8; 64 * 1024];

	let mut file_pos: usize = 0; // absolute offset of start of `buf`
	let mut line_start: usize = 0; // absolute start of current line
	let mut prev_byte_is_cr = false; // was the byte immediately before this chunk a '\r'?

	loop {
		let n = r.read(&mut buf)?;
		if n == 0 {
			break;
		}
		let chunk = &buf[..n];

		// Find all '\n' quickly.
		for nl_idx in memchr_iter(b'\n', chunk) {
			let abs_nl = file_pos + nl_idx;

			// If the byte just before '\n' is '\r', trim it. Handle chunk boundary.
			let end = if nl_idx > 0 {
				if chunk[nl_idx - 1] == b'\r' { abs_nl - 1 } else { abs_nl }
			} else if prev_byte_is_cr {
				abs_nl - 1
			} else {
				abs_nl
			};

			spans.push((line_start, end));
			line_start = abs_nl + 1; // next line starts after '\n'
		}

		// Prepare for next chunk.
		prev_byte_is_cr = chunk[n - 1] == b'\r';
		file_pos += n;
	}

	// Final line if file doesn't end with '\n'
	if line_start < file_pos {
		spans.push((line_start, file_pos));
	}

	Ok(spans)
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_span_line_span_line_spans_simple() -> Result<()> {
		// -- Setup & Fixtures
		let path = SPath::from("tests-data/example.csv");

		// -- Exec
		let spans = line_spans(&path)?;

		// -- Check
		assert_eq!(spans.len(), 5, "should find 5 physical lines");

		let expected = [
			"name,age,comment",
			"Alice,30,\"hello, world\"",
			"Bob,25,\"Line with \"\"quote\"\"\"",
			"Carol,28,\"multi",
			"line with \"\"quotes\"\" inside\"",
		];

		for (i, exp) in expected.iter().enumerate() {
			let (s, e) = spans.get(i).copied().ok_or("missing expected line span")?;
			let got = crate::read_span(&path, s, e)?;
			assert_eq!(&got, exp);
		}

		Ok(())
	}
}

// endregion: --- Tests
