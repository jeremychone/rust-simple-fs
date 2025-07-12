/// A simplified file metadata structure with common, normalized fields.
/// All fields are guaranteed to be present.
pub struct SMeta {
	/// Creation time since the Unix epoch in microseconds.
	/// If unavailable, this may fall back to the modification time.
	pub created_epoch_us: i64,

	/// Last modification time since the Unix epoch in microseconds.
	pub modified_epoch_us: i64,

	/// File size in bytes. Will be 0 for directories or when unavailable.
	pub size: u64,

	/// Whether the path is a regular file.
	pub is_file: bool,

	/// Whether the path is a directory.
	pub is_dir: bool,
}

// region:    --- Pretty Size

/// Formats a byte size as a pretty, fixed-width (10 char) string with unit alignment.
/// The output format is tailored to align nicely in monospaced tables.
///
/// - Number is always 6 character, always right aligned.
/// - Unit is always 2 chars, left aligned. So, for Byte, "B", it will be "B "
/// - When below 1K Byte, do not have any digits
/// - Otehrwise, always 2 digit, rounded
///
/// ### Examples
///
/// `777`           -> `"   777 B "`
/// `8777`          -> `"  8.78 KB"`
/// `88777`         -> `" 88.78 KB"`
/// `888777`        -> `"888.78 KB"`
/// `2_345_678_900` -> `"  2.35 GB"`
///
/// NOTE: if in simple-fs, migh call it pretty_size()
pub fn pretty_size(size_in_bytes: u64) -> String {
	const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
	let mut size = size_in_bytes as f64;
	let mut unit = 0;

	// Determine which unit to use
	while size >= 1000.0 && unit < UNITS.len() - 1 {
		size /= 1000.0;
		unit += 1;
	}

	let unit_str = UNITS[unit];

	// Note: The logic is derived from the test cases, which have specific formatting rules.
	if unit == 0 {
		// Bytes: integer, pad to 6, then add " B "
		let number_str = format!("{size_in_bytes:>6}");
		format!("{number_str} {unit_str} ")
	} else {
		// Units KB or above: 2 decimals, pad to width, then add " unit"
		let number_str = format!("{size:>6.2}");
		format!("{number_str} {unit_str}")
	}
}

// endregion: --- Pretty Size

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_pretty_size() -> Result<()> {
		// -- Setup & Fixtures
		let cases = [
			(777, "   777 B "),
			(8777, "  8.78 KB"),
			(88777, " 88.78 KB"),
			(888777, "888.78 KB"),
			(888700, "888.70 KB"),
			(200000, "200.00 KB"),
			(2_000_000, "  2.00 MB"),
			(900_000_000, "900.00 MB"),
			(2_345_678_900, "  2.35 GB"),
			(1_234_567_890_123, "  1.23 TB"),
			(2_345_678_900_123_456, "  2.35 PB"),
			(0, "     0 B "),
		];

		// -- Exec
		for &(input, expected) in &cases {
			let actual = pretty_size(input);
			assert_eq!(actual, expected, "input: {input}");
		}

		// -- Check

		Ok(())
	}
}

// endregion: --- Tests
