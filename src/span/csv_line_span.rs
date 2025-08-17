use crate::spath::SPath;
use crate::{Error, Result, open_file};
use std::io::{self, Read};

/// CSV-aware record spans: returns byte ranges [start, end) for each *record*.
/// - Treats '\n' as a record separator only when **not** inside quotes.
/// - For CRLF, the '\r' is excluded from the end bound.
/// - Supports `""` as an escaped quote inside quoted fields.
/// - Streams in chunks; does *not* read the whole file into memory.
pub fn csv_spans(path: impl Into<SPath>) -> Result<Vec<(usize, usize)>> {
	let path = path.into();
	let mut f = open_file(&path)?;
	csv_spans_from_reader(&mut f).map_err(|err| Error::FileCantRead((&path, err).into()))
}

// region:    --- Support

fn csv_spans_from_reader<R: Read>(r: &mut R) -> io::Result<Vec<(usize, usize)>> {
	let mut spans: Vec<(usize, usize)> = Vec::new();

	// 64 KiB chunks: good balance of cacheability vs syscalls.
	let mut buf = [0u8; 64 * 1024];

	// Absolute position of start of `buf` in file.
	let mut file_pos: usize = 0;
	// Absolute start offset of the current record.
	let mut rec_start: usize = 0;

	// CSV quote state across chunk boundaries.
	let mut in_quotes: bool = false;
	// We saw a '"' at the end of the previous byte; need to decide if it’s
	// a closing quote or the first of a `""` escape when we see the next byte.
	let mut quote_pending: bool = false;

	// Track CR immediately before '\n' across chunk boundary.
	let mut prev_byte_is_cr: bool = false;

	loop {
		let n = r.read(&mut buf)?;
		if n == 0 {
			break;
		}
		let chunk = &buf[..n];

		let mut i = 0usize;
		while i < n {
			let b = chunk[i];

			// Resolve a pending quote (from previous byte/chunk) if any.
			if quote_pending {
				if b == b'"' {
					// Escaped quote "" inside a quoted field.
					// Consume this byte as the second quote of the escape.
					quote_pending = false;
					// Stay in_quotes; the pair represents a literal '"'.
					i += 1;
					prev_byte_is_cr = false;
					continue;
				} else {
					// Previous '"' was a closing quote.
					in_quotes = false;
					quote_pending = false;
					// Fall through to process current byte normally.
				}
			}

			match b {
				b'"' => {
					if in_quotes {
						// Might be closing quote, but need lookahead to disambiguate "".
						quote_pending = true;
					} else {
						// Enter quoted field.
						in_quotes = true;
						// No pending: we only set pending when *inside* quotes.
					}
				}
				b'\n' => {
					if !in_quotes && !quote_pending {
						// This is a record delimiter. Compute end (exclude preceding \r).
						let abs_nl = file_pos + i;
						let end = if i > 0 {
							if chunk[i - 1] == b'\r' { abs_nl - 1 } else { abs_nl }
						} else if prev_byte_is_cr {
							abs_nl - 1
						} else {
							abs_nl
						};
						spans.push((rec_start, end));
						rec_start = abs_nl + 1;
					}
				}
				_ => { /* regular byte */ }
			}

			prev_byte_is_cr = b == b'\r';
			i += 1;
		}

		// If chunk ended with a '"' inside quotes, we have to defer the decision.
		// `quote_pending` already encodes that state correctly.
		// If chunk ended with '\r', remember it for CRLF spanning chunks:
		// handled via `prev_byte_is_cr` above.

		file_pos += n;
	}

	// End-of-file: close any pending quote decision (treat as closing if still pending).
	#[allow(unused)]
	if quote_pending {
		in_quotes = false;
		quote_pending = false;
	}

	// Final record if file doesn’t end with '\n'
	if rec_start < file_pos {
		spans.push((rec_start, file_pos));
	}

	Ok(spans)
}

// endregion: --- Support
