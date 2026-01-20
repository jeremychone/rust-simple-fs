# simple-fs – Public APIs (by category)

Note: Root crate re-exports most modules, so items are accessible from the crate root unless noted.

```toml
simple-fs = "0.9.1"
# or with features
simple-fs = {version = "0.9.1", features = ["with-json", "with-toml", "bin-nums"]}
# or `features = ["full"]
```

## Core

- Type alias: `Result<T> = core::result::Result<T, Error>`

- Error type: `Error`


## Paths (SPath)

- Type: `SPath` (UTF-8, normalized posix-style path, `/` separators)

- Constructors
  - `SPath::new(path: impl Into<Utf8PathBuf>) -> SPath` (always normalizes)
  
  - `SPath::from_std_path_buf(path_buf: PathBuf) -> Result<SPath>`
  
  - `SPath::from_std_path(path: impl AsRef<Path>) -> Result<SPath>`
  
  - `SPath::from_walkdir_entry(wd_entry: walkdir::DirEntry) -> Result<SPath>`
  
  - `SPath::from_std_path_ok(path: impl AsRef<Path>) -> Option<SPath>`
  
  - `SPath::from_std_path_buf_ok(path_buf: PathBuf) -> Option<SPath>`
  
  - `SPath::from_fs_entry_ok(fs_entry: fs::DirEntry) -> Option<SPath>`
  
  - `SPath::from_walkdir_entry_ok(wd_entry: walkdir::DirEntry) -> Option<SPath>`

- Conversions (consuming / views)
  - `SPath::into_std_path_buf(self) -> PathBuf`
  
  - `SPath::std_path(&self) -> &Path`
  
  - `SPath::path(&self) -> &Utf8Path`
  
  - `SPath::as_std_path(&self) -> &Path`

- Getters
  - `SPath::to_str(&self) -> &str` (deprecated)
  
  - `SPath::as_str(&self) -> &str`
  
  - `SPath::file_name(&self) -> Option<&str>`
  
  - `SPath::name(&self) -> &str`
  
  - `SPath::parent_name(&self) -> &str`
  
  - `SPath::file_stem(&self) -> Option<&str>`
  
  - `SPath::stem(&self) -> &str`
  
  - `SPath::extension(&self) -> Option<&str>`
  
  - `SPath::ext(&self) -> &str`
  
  - `SPath::is_dir(&self) -> bool`
  
  - `SPath::is_file(&self) -> bool`
  
  - `SPath::exists(&self) -> bool`
  
  - `SPath::is_absolute(&self) -> bool`
  
  - `SPath::is_relative(&self) -> bool`

- Metadata
  - `SPath::meta(&self) -> Result<SMeta>`
  
  - `SPath::metadata(&self) -> Result<Metadata>`
  
  - `SPath::modified(&self) -> Result<SystemTime>` (deprecated)
  
  - `SPath::modified_us(&self) -> Result<i64>` (deprecated)

- Transformers
  - `SPath::canonicalize(&self) -> Result<SPath>`
  
  - `SPath::collapse(&self) -> SPath`
  
  - `SPath::into_collapsed(self) -> SPath`
  
  - `SPath::is_collapsed(&self) -> bool`

- Parent & Join
  - `SPath::parent(&self) -> Option<SPath>`
  
  - `SPath::append_suffix(&self, suffix: &str) -> SPath`
  
  - `SPath::join(&self, leaf_path: impl Into<Utf8PathBuf>) -> SPath`
  
  - `SPath::join_std_path(&self, leaf_path: impl AsRef<Path>) -> Result<SPath>`
  
  - `SPath::new_sibling(&self, leaf_path: impl AsRef<str>) -> SPath`
  
  - `SPath::new_sibling_std_path(&self, leaf_path: impl AsRef<Path>) -> Result<SPath>`

- Diff
  - `SPath::diff(&self, base: impl AsRef<Utf8Path>) -> Option<SPath>`
  
  - `SPath::try_diff(&self, base: impl AsRef<Utf8Path>) -> Result<SPath>`

- Replace
  - `SPath::replace_prefix(&self, base: impl AsRef<str>, with: impl AsRef<str>) -> SPath`
  
  - `SPath::into_replace_prefix(self, base: impl AsRef<str>, with: impl AsRef<str>) -> SPath`

- Extensions
  - `SPath::into_ensure_extension(self, ext: &str) -> SPath`
  
  - `SPath::ensure_extension(&self, ext: &str) -> SPath`
  
  - `SPath::append_extension(&self, ext: &str) -> SPath`

- Other
  - `SPath::strip_prefix(&self, prefix: impl AsRef<Path>) -> Result<SPath>`
  
  - `SPath::starts_with(&self, base: impl AsRef<Path>) -> bool`
  
  - `SPath::dir_before_glob(&self) -> Option<SPath>`

- Traits (summary)
  - Display
  
  - AsRef<SPath> / AsRef<Path> / AsRef<Utf8Path> / AsRef<str>
  
  - From<Utf8PathBuf> / From<&Utf8Path> / From<String> / From<&String> / From<&str>
  
  - From<SPath> for String / PathBuf / Utf8PathBuf
  
  - From<&SPath> for String / PathBuf
  
  - TryFrom<PathBuf> / TryFrom<fs::DirEntry> / TryFrom<walkdir::DirEntry>

## File I/O

- `create_file(file_path: impl AsRef<Path>) -> Result<File>`

- `read_to_string(file_path: impl AsRef<Path>) -> Result<String>`

- `open_file(path: impl AsRef<Path>) -> Result<File>`

- `get_buf_reader(file: impl AsRef<Path>) -> Result<BufReader<File>>`

- `get_buf_writer(file_path: impl AsRef<Path>) -> Result<BufWriter<File>>`


## Spans

- `read_span(path: impl AsRef<SPath>, start: usize, end: usize) -> Result<String>`

- `line_spans(path: impl AsRef<SPath>) -> Result<Vec<(usize, usize)>>`

- `csv_row_spans(path: impl AsRef<SPath>) -> Result<Vec<(usize, usize)>>`


## Directories

- `ensure_dir(dir: impl AsRef<Path>) -> Result<bool>`

- `ensure_file_dir(file_path: impl AsRef<Path>) -> Result<bool>`


## Listing & Globbing

- Directory iteration
  - `iter_dirs(dir: impl AsRef<Path>, include_globs: Option<&[&str]>, list_options: Option<ListOptions<'_>>) -> Result<impl Iterator<Item = SPath>>`
  
  - `list_dirs(dir: impl AsRef<Path>, include_globs: Option<&[&str]>, list_options: Option<ListOptions<'_>>) -> Result<Vec<SPath>>`

- File iteration
  - `iter_files(dir: impl AsRef<Path>, include_globs: Option<&[&str]>, list_options: Option<ListOptions<'_>>) -> Result<GlobsFileIter>`
  
  - `list_files(dir: impl AsRef<Path>, include_globs: Option<&[&str]>, list_options: Option<ListOptions<'_>>) -> Result<Vec<SPath>>`
  
  - Type: `GlobsFileIter` (Iterator<Item = SPath>)

- Options
  - `ListOptions<'a> { exclude_globs: Option<Vec<&'a str>>, relative_glob: bool, depth: Option<usize> }`
  - Defaults: `relative_glob: false`, `exclude_globs: None` (but `iter_files` applies `DEFAULT_EXCLUDE_GLOBS` if `None`)
  
  - `ListOptions::new(globs: Option<&'a [&'a str]>) -> ListOptions<'a>`
  
  - `ListOptions::from_relative_glob(val: bool) -> ListOptions<'a>`
  
  - `ListOptions::with_exclude_globs(self, globs: &'a [&'a str]) -> Self`
  
  - `ListOptions::with_relative_glob(self) -> Self`
  
  - `ListOptions::exclude_globs(&'a self) -> Option<&'a [&'a str]>`
  
  - From conversions: `From<&'a [&'a str]>`, `From<Option<&'a [&'a str]>>`, `From<Vec<&'a str>>`

- Glob utilities
  - `DEFAULT_EXCLUDE_GLOBS: &[&str]`
  
  - `get_glob_set(globs: &[&str]) -> Result<globset::GlobSet>`
  
  - `longest_base_path_wild_free(pattern: &SPath) -> SPath`
  
  - `get_depth(patterns: &[&str], depth: Option<usize>) -> usize`

- Sorting
  - `sort_by_globs<T>(items: Vec<T>, globs: &[&str], end_weighted: bool) -> Result<Vec<T>> where T: AsRef<SPath>`


## Reshape / Normalize

- Normalizer
  - `needs_normalize(path: &Utf8Path) -> bool`
  
  - `into_normalized(path: Utf8PathBuf) -> Utf8PathBuf`

- Collapser
  - `into_collapsed(path: impl Into<Utf8PathBuf>) -> Utf8PathBuf`
  
  - `try_into_collapsed(path: impl Into<Utf8PathBuf>) -> Option<Utf8PathBuf>`
  
  - `is_collapsed(path: impl AsRef<Utf8Path>) -> bool`


## Safer Remove

- Function: `safer_remove_dir(dir_path: &SPath, options: impl Into<SaferRemoveOptions<'a>>) -> Result<bool>`
- Function: `safer_remove_file(file_path: &SPath, options: impl Into<SaferRemoveOptions<'a>>) -> Result<bool>`
- Note: `options` cannot be `None`. Use `SaferRemoveOptions::default()` or `()` for default safety.

- Type: `SaferRemoveOptions<'a>`
  - `SaferRemoveOptions::default()`: `restrict_to_current_dir: true` (path must be below CWD), others `None`.
  - `SaferRemoveOptions::with_must_contain_any(self, patterns: &'a [&'a str]) -> Self`
  - `SaferRemoveOptions::with_must_contain_all(self, patterns: &'a [&'a str]) -> Self`
  - `SaferRemoveOptions::with_restrict_to_current_dir(self, val: bool) -> Self`


## Safer Trash

- Function: `safer_trash_dir(dir_path: &SPath, options: impl Into<SaferTrashOptions<'a>>) -> Result<bool>`

- Function: `safer_trash_file(file_path: &SPath, options: impl Into<SaferTrashOptions<'a>>) -> Result<bool>`

- Note: `options` cannot be `None`. Use `SaferTrashOptions::default()` or `()` for default safety.

- Type: `SaferTrashOptions<'a>`
  - `SaferTrashOptions::default()`: `restrict_to_current_dir: true` (path must be below CWD), others `None`.
  - `SaferTrashOptions::with_must_contain_any(self, patterns: &'a [&'a str]) -> Self`
  - `SaferTrashOptions::with_must_contain_all(self, patterns: &'a [&'a str]) -> Self`
  - `SaferTrashOptions::with_restrict_to_current_dir(self, val: bool) -> Self`


## Common

- `home_dir() -> Result<SPath>`

- `current_dir() -> Result<SPath>`

- Pretty size (Fixed width 9 chars, right aligned number, unit aligned)
  - `struct PrettySizeOptions { lowest_unit: SizeUnit }`
  
  - `enum SizeUnit { B, KB, MB, GB, TB }`
  
  - `SizeUnit::new(val: &str) -> SizeUnit`
  
  - `pretty_size(size_in_bytes: u64) -> String`
  
  - `pretty_size_with_options(size_in_bytes: u64, options: impl Into<PrettySizeOptions>) -> String`

- File metadata
  - `struct SMeta { created_epoch_us: i64, modified_epoch_us: i64, size: u64, is_file: bool, is_dir: bool }`


## Watch

- `watch(path: impl AsRef<Path>) -> Result<SWatcher>` (Debounced: ~200ms)

- `struct SWatcher { rx: flume::Receiver<Vec<SEvent>>, /* keeps internal debouncer alive */ }`

- `struct SEvent { spath: SPath, skind: SEventKind }`

- `enum SEventKind { Create, Modify, Remove, Other }`

- Re-export: `DebouncedEvent` (from `notify_debouncer_full`)


## Feature-gated: with-json

- Load
  - `load_json<T: serde::de::DeserializeOwned>(file: impl AsRef<Path>) -> Result<T>`
  
  - `load_ndjson(file: impl AsRef<Path>) -> Result<Vec<serde_json::Value>>`
  
  - `stream_ndjson(file: impl AsRef<Path>) -> Result<impl Iterator<Item = Result<serde_json::Value>>>`

- Parse (NDJSON)
  - `parse_ndjson_iter(input: &str) -> impl Iterator<Item = Result<serde_json::Value>>`
  
  - `parse_ndjson(input: &str) -> Result<Vec<serde_json::Value>>`
  
  - `parse_ndjson_from_reader<R: std::io::BufRead>(reader: R) -> Result<Vec<serde_json::Value>>`
  
  - `parse_ndjson_iter_from_reader<R: std::io::BufRead>(reader: R) -> impl Iterator<Item = Result<serde_json::Value>>`

- Save
  - `save_json<T: serde::Serialize>(file: impl AsRef<Path>, data: &T) -> Result<()>`
  
  - `save_json_pretty<T: serde::Serialize>(file: impl AsRef<Path>, data: &T) -> Result<()>`
  
  - `append_json_line<T: serde::Serialize>(file: impl AsRef<Path>, value: &T) -> Result<()>`
  
  - `append_json_lines<'a, T: serde::Serialize + 'a, I: IntoIterator<Item = &'a T>>(file: impl AsRef<Path>, values: I) -> Result<()>`


## Feature-gated: with-toml

- `load_toml<T: serde::de::DeserializeOwned>(file_path: impl AsRef<Path>) -> Result<T>`

- `save_toml<T: serde::Serialize>(file_path: impl AsRef<Path>, data: &T) -> Result<()>`


## Feature-gated: bin-nums

- Load (binary)
  - `load_be_f64(file) -> Result<Vec<f64>>`, `load_le_f64(file) -> Result<Vec<f64>>`
  
  - `load_be_f32(file) -> Result<Vec<f32>>`, `load_le_f32(file) -> Result<Vec<f32>>`
  
  - `load_be_u64(file) -> Result<Vec<u64>>`, `load_le_u64(file) -> Result<Vec<u64>>`
  
  - `load_be_u32(file) -> Result<Vec<u32>>`, `load_le_u32(file) -> Result<Vec<u32>>`
  
  - `load_be_u16(file) -> Result<Vec<u16>>`, `load_le_u16(file) -> Result<Vec<u16>>`
  
  - `load_be_i64(file) -> Result<Vec<i64>>`, `load_le_i64(file) -> Result<Vec<i64>>`
  
  - `load_be_i32(file) -> Result<Vec<i32>>`, `load_le_i32(file) -> Result<Vec<i32>>`
  
  - `load_be_i16(file) -> Result<Vec<i16>>`, `load_le_i16(file) -> Result<Vec<i16>>`

- Save (binary)
  - `save_be_f64(file, data: &[f64]) -> Result<()>`, `save_le_f64(file, data: &[f64]) -> Result<()>`
  
  - `save_be_f32(file, data: &[f32]) -> Result<()>`, `save_le_f32(file, data: &[f32]) -> Result<()>`
  
  - `save_be_u64(file, data: &[u64]) -> Result<()>`, `save_le_u64(file, data: &[u64]) -> Result<()>`
  
  - `save_be_u32(file, data: &[u32]) -> Result<()>`, `save_le_u32(file, data: &[u32]) -> Result<()>`
  
  - `save_be_u16(file, data: &[u16]) -> Result<()>`, `save_le_u16(file, data: &[u16]) -> Result<()>`
  
  - `save_be_i64(file, data: &[i64]) -> Result<()>`, `save_le_i64(file, data: &[i64]) -> Result<()>`
  
  - `save_be_i32(file, data: &[i32]) -> Result<()>`, `save_le_i32(file, data: &[i32]) -> Result<()>`
  
  - `save_be_i16(file, data: &[i16]) -> Result<()>`, `save_le_i16(file, data: &[i16]) -> Result()`
