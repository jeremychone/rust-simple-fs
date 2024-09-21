pub struct ListOptions<'a> {
	pub exclude_globs: Option<Vec<&'a str>>,
}

/// Constructors
impl<'a> ListOptions<'a> {
	pub fn new(globs: Option<&'a [&'a str]>) -> Self {
		ListOptions {
			exclude_globs: globs.map(|v| v.to_vec()),
		}
	}
}

/// Getters
impl<'a> ListOptions<'a> {
	pub fn exclude_globs(&'a self) -> Option<&'a [&'a str]> {
		self.exclude_globs.as_deref()
	}
}

// region:    --- Froms

impl<'a> From<&'a [&'a str]> for ListOptions<'a> {
	fn from(globs: &'a [&'a str]) -> Self {
		ListOptions {
			exclude_globs: Some(globs.to_vec()),
		}
	}
}

impl<'a> From<Option<&'a [&'a str]>> for ListOptions<'a> {
	fn from(globs: Option<&'a [&'a str]>) -> Self {
		ListOptions {
			exclude_globs: globs.map(|v| v.to_vec()),
		}
	}
}

impl<'a> From<Vec<&'a str>> for ListOptions<'a> {
	fn from(globs: Vec<&'a str>) -> Self {
		let globs_ref: Vec<&'a str> = globs.to_vec();
		ListOptions {
			exclude_globs: Some(globs_ref),
		}
	}
}

// endregion: --- Froms
