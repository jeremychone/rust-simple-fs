/// A simplified file metadata structure with common, normalized fields.
/// All fields are guaranteed to be present.
#[derive(Debug, Clone)]
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
