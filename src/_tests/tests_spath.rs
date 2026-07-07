use crate::SPath;

#[test]
fn test_spath_is_likely_text_ext_false() {
	// -- Setup & Fixtures
	#[rustfmt::skip]
	let cases: &[&str] = &[
		// media
		"png", "jpg", "jpeg", "gif", "webp", "mp3", "mp4", "ttf", "woff",
		// exec
	 "exe", "so", "dll",
		// db
		"db", "db3", "sqlite", "sqlite3",
		// doc
		"pdf", "docx","doc"
	];

	// -- Exec & Check
	for ext in cases {
		let filename = format!("file.{}", ext);
		let spath = SPath::new(&filename);
		let result = spath.is_likely_text();
		if result {
			let mimes = mime_guess::from_path(spath);
			panic!("is_likely_text({filename:?}) expected false but got {result:?}. mime guess: {mimes:?}",)
		}
	}
}

#[test]
fn test_spath_is_likely_text_ext_true() {
	// -- Setup & Fixtures
	#[rustfmt::skip]
	let cases: &[&str] = &[
		"md", "markdown", "csv", "toml", "yaml", "yml", "json", "jsonc", "jsonl", "ndjson", "ldjson",
		"xml", "html", "htm", "css", "scss", "sass", "less", "js", "mjs", "cjs", "ts", "tsx", "jsx",
		"rs", "py", "rb", "go", "java", "c", "cpp", "h", "hpp", "sh", "bash", "zsh", "fish", "php",
		"lua", "ini", "cfg", "conf", "sql", "graphql", "gql", "svg", "log", "env", "txt", "dart", "tsv",
		"tex", "scala", "vue", "svelte", "hbs", "astro", "cs", "kt", "kotlin", "aip",
	];

	// -- Exec & Check
	for ext in cases {
		let filename = format!("file.{}", ext);
		let spath = SPath::new(&filename);
		let result = spath.is_likely_text();
		assert!(
			result,
			"is_likely_text({:?}) expected true but got {:?}",
			filename, result
		);
	}
}

#[test]
fn test_spath_is_likely_text_filename_true() {
	// -- Setup & Fixtures
	let cases: &[&str] = &["Dockerfile", "Makefile", "LICENSE", ".gitignore", ".env"];

	// -- Exec & Check
	for filename in cases {
		let spath = SPath::new(*filename);
		let result = spath.is_likely_text();
		assert!(
			result,
			"is_likely_text({:?}) expected true but got {:?}",
			filename, result
		);
	}
}
