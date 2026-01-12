#[derive(Debug, Clone)]
pub struct SaferTrashOptions<'a> {
	pub must_contain_any: Option<&'a [&'a str]>,
	pub must_contain_all: Option<&'a [&'a str]>,
	pub restrict_to_current_dir: bool,
}

// region:    --- Default

impl Default for SaferTrashOptions<'_> {
	fn default() -> Self {
		Self {
			must_contain_any: None,
			must_contain_all: None,
			restrict_to_current_dir: true,
		}
	}
}

// endregion: --- Default

// region:    --- Froms

impl From<()> for SaferTrashOptions<'_> {
	fn from(_: ()) -> Self {
		Self::default()
	}
}

impl<'a> From<&'a [&'a str]> for SaferTrashOptions<'a> {
	fn from(patterns: &'a [&'a str]) -> Self {
		Self {
			must_contain_any: Some(patterns),
			..Default::default()
		}
	}
}

// endregion: --- Froms

// region:    --- Fluent API

impl<'a> SaferTrashOptions<'a> {
	pub fn with_must_contain_any(mut self, patterns: &'a [&'a str]) -> Self {
		self.must_contain_any = Some(patterns);
		self
	}

	pub fn with_must_contain_all(mut self, patterns: &'a [&'a str]) -> Self {
		self.must_contain_all = Some(patterns);
		self
	}

	pub fn with_restrict_to_current_dir(mut self, val: bool) -> Self {
		self.restrict_to_current_dir = val;
		self
	}
}

// endregion: --- Fluent API
