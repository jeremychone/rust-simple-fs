# simple-fs

[simple-fs](https://github.com/jeremychone/rust-simple-fs) is a crate that provides a set of convenient and common file APIs built on `std::fs`, [walkdir](https://crates.io/crates/walkdir), and [globset](https://crates.io/crates/globset).


## Cargo Features

| Feature     | Functions Included                               |
|-------------|--------------------------------------------------|
| `with-json` | `load_json`, `save_json`, `save_json_pretty`     |
| `with-toml` | `load_toml`, `save_toml`                         |
| `bin-nums`  | `save_be_f64`, `load_be_f64`, `save_le_f64`, ... |
| `full`      | All the above.                                   |
| default     | None of the above. See below.                    |

## API Change `0.2.0` from `0.1.x`

- API CHANGE - now .file_name() and .file_stem() returns Option<&str> use .name(), .stem() to have &str

## Concept 

`simple-fs` operates under the assumption that paths which are not `utf8` are not visible to the API, which simplifies many of the path-related APIs.

The two constructs that follow this assumption are (both are just wrappers of PathBuf with some guarantees):

- `SPath`, which ensures that the contained path is a valid UTF-8 path and includes a file name.

- `SFile`, which carries the same guarantees as `SPath` but also checks if the file `is_file()`, confirming the file's existence at the time of the `SFile` construction.

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