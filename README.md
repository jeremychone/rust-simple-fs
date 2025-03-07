# simple-fs

[simple-fs](https://github.com/jeremychone/rust-simple-fs) is a crate that provides a set of convenient and common file APIs built on `std::fs`, [walkdir](https://crates.io/crates/walkdir), and [globset](https://crates.io/crates/globset).

## Cargo Features

| Feature     | Functions Included                              |
|-------------|--------------------------------------------------|
| `with-json` | `load_json`, `save_json`, `save_json_pretty`     |
| `with-toml` | `load_toml`, `save_toml`                         |
| `bin-nums`  | `save_be_f64`, `load_be_f64`, `save_le_f64`, ... |
| `full`      | All the above.                                   |
| default     | None of the above. See below.                    |

## API Changes

- `0.6.x` (rc for now)
  - Now use Utf8 by default, std path moved to `..std_path..` naming. 
- `0.5.0`
  - Internal use camino, utf8path. 
  - Reimplementation of the iter_files iterator, supporting absolute path globs out of the base directory. 
- `0.4.0`
  - Update to `notify 8` (should not have any API changes)
  - API CHANGE - SPath - Now `SPath::from(&str/&String,String)` (no need for `try_from`)
- `0.3.1` from `0.3.0`
  - This is a fix; however, it can change behavior on `list/iter` files. 
    - Previously, the glob `*` was traversing into subfolders `/`, which was not the intended behavior. 
    - Now, in `0.3.1`, it uses the glob `literal_separator = true`, so it won't descend further. 
    - You can now use `*.rs` to list direct descendants and `**/*.rs` for nested files. 
    - The glob also needs to include the `dir` given in the list; otherwise, set `ListOptions.relative_glob = true` to make it relative. 
- `0.3.x` from `0.2.x`
  - API CHANGE - watch - changes the rx to be [flume](https://crates.io/crates/flume) based (works on both sync/async).
- `0.2.0` from `0.1.x`
  - API CHANGE - now .file_name() and .file_stem() return Option<&str>; use .name() or .stem() to get &str.

## Concept

`simple-fs` operates under the assumption that paths that are not `utf8` are not visible to the API, simplifying many of the path-related APIs.

The two constructs that follow this assumption are (both are just wrappers of PathBuf with some guarantees):

- `SPath`, which ensures that the contained path is a valid UTF-8 path and includes a file name.
- `SFile`, which carries the same guarantees as `SPath` but also checks if the file `is_file()`, confirming the file's existence at the time of `SFile` construction.

By establishing these rules, APIs such as `.file_name()`, `.file_stem()`, and `.to_str()` are much simpler, as they all return `&str`.

The crate also includes other convenient, common APIs:

- `read_to_string`, which reports the file path if not found.
- `get_buf_reader`, which also reports the file path if not found or in case of an error.
- `iter_files(dir, include_globs: &[&str], exclude_globs: &[&str]) -> Result<impl Iter SFile>`
- `list_files(..same as iter_files..) -> Result<Vec<SFile>>`
- `ensure_dir(dir_path)`, `ensure_file_dir(file_path)`
- With features, see above. 

For more control, it is recommended to use `std::fs`, `walkdir`, `globset`, and other crates directly.

This is a very early implementation, with more to come.

Happy coding!
