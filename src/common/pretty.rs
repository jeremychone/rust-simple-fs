// region:    --- Pretty Size

use derive_more::From;

#[derive(Debug, Default, Clone, From)]
pub struct PrettySizeOptions {
	#[from]
	lowest_unit: SizeUnit,
}

impl From<&str> for PrettySizeOptions {
	fn from(val: &str) -> Self {
		SizeUnit::new(val).into()
	}
}

impl From<&String> for PrettySizeOptions {
	fn from(val: &String) -> Self {
		SizeUnit::new(val).into()
	}
}

impl From<String> for PrettySizeOptions {
	fn from(val: String) -> Self {
		SizeUnit::new(&val).into()
	}
}

#[derive(Debug, Clone, Default)]
pub enum SizeUnit {
	#[default]
	B,
	KB,
	MB,
	GB,
	TB,
}

impl SizeUnit {
	/// Will return
	pub fn new(val: &str) -> Self {
		match val.to_uppercase().as_str() {
			"B" => Self::B,
			"KB" => Self::KB,
			"MB" => Self::MB,
			"GB" => Self::GB,
			"TB" => Self::TB,
			_ => Self::B,
		}
	}

	/// Index of the unit in the `UNITS` array used by [`pretty_size_with_options`].
	#[inline]
	pub fn idx(&self) -> usize {
		match self {
			Self::B => 0,
			Self::KB => 1,
			Self::MB => 2,
			Self::GB => 3,
			Self::TB => 4,
		}
	}
}

impl From<&str> for SizeUnit {
	fn from(val: &str) -> Self {
		Self::new(val)
	}
}

impl From<&String> for SizeUnit {
	fn from(val: &String) -> Self {
		Self::new(val)
	}
}

impl From<String> for SizeUnit {
	fn from(val: String) -> Self {
		Self::new(&val)
	}
}

/// Formats a byte size as a pretty, fixed-width (9 char) string with unit alignment.
/// The output format is tailored to align nicely in monospaced tables.
///
/// - Number is always 6 character, always right aligned.
/// - Empty char
/// - Unit is always 2 chars, left aligned. So, for Byte, "B", it will be "B "
/// - When below 1K Byte, do not have any digits
/// - Otherwise, always 2 digit, rounded
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
	pretty_size_with_options(size_in_bytes, PrettySizeOptions::default())
}

/// Formats a byte size as a pretty, fixed-width (9 char) string with unit alignment.
/// The output format is tailored to align nicely in monospaced tables.
///
/// - Number is always 6 character, always right aligned.
/// - Empty char
/// - Unit is always 2 chars, left aligned. So, for Byte, "B", it will be "B "
/// - When below 1K Byte, do not have any digits
/// - Otherwise, always 2 digit, rounded
///
/// ### PrettySizeOptions
///
/// - `lowest_unit`
///   Define the lowest unit to consider,
///   For example, if `MB`, then, B and KB will be expressed in decimal
///   following the formatting rules.
///
/// NOTE: From String, &str, .. are implemented, so `PrettySizeOptions::from("MB")` will default to
///       `PrettySizeOptions { lowest_unit: SizeUnit::MB }` (if string not match, will default to `SizeUnit::MB`)
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
pub fn pretty_size_with_options(size_in_bytes: u64, options: impl Into<PrettySizeOptions>) -> String {
	let options = options.into();

	const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];

	// -- Step 1: shift the value so that we start at the minimum unit requested.
	let min_unit_idx = options.lowest_unit.idx();
	let mut size = size_in_bytes as f64;
	for _ in 0..min_unit_idx {
		size /= 1000.0;
	}
	let mut unit_idx = min_unit_idx;

	// -- Step 2: continue bubbling up if the number is >= 1000.
	while size >= 1000.0 && unit_idx < UNITS.len() - 1 {
		size /= 1000.0;
		unit_idx += 1;
	}

	let unit_str = UNITS[unit_idx];

	// -- Step 3: formatting
	if unit_idx == 0 {
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

		Ok(())
	}

	#[test]
	fn test_pretty_size_with_lowest_unit() -> Result<()> {
		// -- Setup
		let options = PrettySizeOptions::from("MB");
		let cases = [
			//
			(88777, "  0.09 MB"),
			(888777, "  0.89 MB"),
			(1_234_567, "  1.23 MB"),
		];

		// -- Exec / Check
		for &(input, expected) in &cases {
			let actual = pretty_size_with_options(input, options.clone());
			assert_eq!(actual, expected, "input: {input}");
		}

		Ok(())
	}
}

// endregion: --- Tests
