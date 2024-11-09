/// Note: In the future, the lifetime might go away, and the iter_files will take Option<&ListOptions>
#[derive(Default)]
pub struct ListOptions<'a> {
	pub exclude_globs: Option<Vec<&'a str>>,

	/// When this is true,
	/// - the glob will be relative to the dir of the list, rather than including it.
	///
	/// By default it is false.
	pub relative_glob: bool,
}

/// Constructors
impl<'a> ListOptions<'a> {
	pub fn new(globs: Option<&'a [&'a str]>) -> Self {
		ListOptions {
			exclude_globs: globs.map(|v| v.to_vec()),
			relative_glob: false,
		}
	}

	pub fn from_relative_glob(val: bool) -> Self {
		ListOptions {
			exclude_globs: None,
			relative_glob: val,
		}
	}
}

/// Setters
impl<'a> ListOptions<'a> {
	pub fn with_exclude_globs(mut self, globs: &'a [&'a str]) -> Self {
		self.exclude_globs = Some(globs.to_vec());
		self
	}

	pub fn with_relative_glob(mut self) -> Self {
		self.relative_glob = true;
		self
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
			relative_glob: false,
		}
	}
}

impl<'a> From<Option<&'a [&'a str]>> for ListOptions<'a> {
	fn from(globs: Option<&'a [&'a str]>) -> Self {
		ListOptions {
			exclude_globs: globs.map(|v| v.to_vec()),
			relative_glob: false,
		}
	}
}

impl<'a> From<Vec<&'a str>> for ListOptions<'a> {
	fn from(globs: Vec<&'a str>) -> Self {
		let globs_ref: Vec<&'a str> = globs.to_vec();
		ListOptions {
			exclude_globs: Some(globs_ref),
			relative_glob: false,
		}
	}
}

// endregion: --- Froms
