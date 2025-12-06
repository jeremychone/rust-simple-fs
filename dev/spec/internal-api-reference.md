# simple-fs – Internal APIs

This document describes internal types, structs, functions, and constants primarily used for internal logic, implementation details, and module coordination.

## Core / Constants

### `src/lib.rs`

```rust
const TOP_MAX_DEPTH: usize = 100;
```

## `src/error.rs`

### Types

```rust
pub enum Cause {
	Custom(String),
	Io(Box<io::Error>),
	#[cfg(feature = "with-json")]
	SerdeJson(Box<serde_json::Error>),
	#[cfg(feature = "with-toml")]
	TomlDe(Box<toml::de::Error>),
	#[cfg(feature = "with-toml")]
	TomlSer(Box<toml::ser::Error>),
}
```

```rust
pub struct PathAndCause {
	pub path: String,
	pub cause: Cause,
}
```

### PathAndCause Conversions

```rust
impl From<(&Path, io::Error)> for PathAndCause
impl From<(&SPath, io::Error)> for PathAndCause
impl From<(&SPath, std::time::SystemTimeError)> for PathAndCause

// Requires feature = "with-json"
impl From<(&Path, serde_json::Error)> for PathAndCause

// Requires feature = "with-toml"
impl From<(&Path, toml::de::Error)> for PathAndCause
impl From<(&Path, toml::ser::Error)> for PathAndCause
```

## `src/spath.rs` / `src/sfile.rs` (Validation)

### `src/spath.rs`

```rust
pub(crate) fn validate_spath_for_result(path: impl Into<PathBuf>) -> Result<Utf8PathBuf>
pub(crate) fn validate_spath_for_option(path: impl Into<PathBuf>) -> Option<Utf8PathBuf>
```

### `src/sfile.rs`

```rust
fn validate_sfile_for_result(path: &SPath) -> Result<()>
fn validate_sfile_for_option(path: &SPath) -> Option<()>
```

## `src/common/pretty.rs`

### Functions

```rust
pub fn pretty_size_with_options(size_in_bytes: u64, options: impl Into<PrettySizeOptions>) -> String
```

### Constants

```rust
const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"]; // Internal to pretty_size_with_options impl
```

## `src/featured` (Feature Gated)

### `src/featured/bin_nums.rs` (Requires `bin-nums`)

Internal implementation functions for loading binary numbers.

```rust
// Generic loader function templates
fn load_f64(file_path: &Path, read_fn: fn(buf: &[u8]) -> f64) -> Result<Vec<f64>>
fn save_f64(file_path: &Path, data: &[f64], write_fn: fn(buf: &mut [u8], n: f64)) -> Result<()>
// ... Similar functions exist for f32, u64, u32, u16, i64, i32, i16 types.
```

### `src/featured/with_json/save.rs` (Requires `with-json`)

```rust
const JSON_LINES_BUFFER_SIZE: usize = 100;
fn save_json_impl<T>(file_path: &Path, data: &T, pretty: bool) -> Result<()>
where
	T: serde::Serialize,
```

## `src/list` (Iteration & Globbing)

### Iterators

```rust
pub struct GlobsDirIter {
	inner: Box<dyn Iterator<Item = SPath>>,
}
```

```rust
pub struct GlobsFileIter {
	inner: Box<dyn Iterator<Item = SFile>>,
}
```

```rust
struct GlobGroup {
	base: SPath,
	patterns: Vec<String>,
	prefixes: Vec<String>,
}
```

### Support Functions (`src/list/globs_file_iter.rs`)

```rust
fn process_globs(main_base: &SPath, globs: &[&str]) -> Result<Vec<GlobGroup>>
fn relative_from_absolute(glob: &SPath, group_base: &SPath) -> String
fn directory_matches_allowed_prefixes(path: &SPath, base: &SPath, prefixes: &[String]) -> bool
fn glob_literal_prefixes(pattern: &str) -> Vec<String>
fn expand_brace_segment(segment: &str) -> Option<Vec<String>>
fn segment_contains_wildcard(segment: &str) -> bool
fn append_adjusted(target: &mut Vec<String>, values: &[String])
fn normalize_prefixes(prefixes: &mut Vec<String>)
```

### Support Functions (`src/list/sort.rs`)

```rust
fn match_index_for_path(path: &SPath, matchers: &[(usize, GlobMatcher)], end_weighted: bool) -> usize
```

## `src/safer_remove/safer_remove_impl.rs` (Safety Checks)

```rust
fn check_path_for_deletion_safety<const IS_DIR: bool>(path: &SPath, options: &SaferRemoveOptions<'_>) -> Result<()>
```

## `src/span` (IO Implementation)

### `src/span/csv_spans.rs`

```rust
fn csv_row_spans_from_reader<R: Read>(r: &mut R) -> io::Result<Vec<(usize, usize)>>
```

### `src/span/line_spans.rs`

```rust
fn line_spans_from_reader<R: Read>(r: &mut R) -> io::Result<Vec<(usize, usize)>>
```

### `src/span/read_span.rs`

```rust
fn read_exact_at(file: &File, offset: u64, len: usize) -> io::Result<Vec<u8>>
```

## `src/watch.rs` (File Watching)

### Types

```rust
struct EventHandler {
	tx: Sender<Vec<SEvent>>,
}
```

```rust
#[derive(Hash, Eq, PartialEq)]
struct SEventKey {
	spath_string: String,
	skind: SEventKind,
}
```

### Functions

```rust
fn build_sevents(events: Vec<DebouncedEvent>) -> Vec<SEvent>
```

### Constants

```rust
const WATCH_DEBOUNCE_MS: u64 = 200;
```
