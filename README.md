# simple-fs

[simple-fs](https://github.com/jeremychone/rust-simple-fs) is a crate that provides a set of convenient and common file APIs built on `std::fs`, [walkdir](https://crates.io/crates/walkdir), and [globset](https://crates.io/crates/globset).

## Concept 

`simple-fs` operates under the assumption that paths which are not `utf8` are not visible to the API, which simplifies many of the path-related APIs.

The two constructs that follow this assumption are:

- `SPath`, which ensures that the contained path is a valid UTF-8 path and includes a file name.

- `SFile`, which carries the same guarantees as `SPath` but also checks if the file `is_file()`, confirming the file's existence at the time of the `SFile` construction.

By establishing these rules, APIs such as `.file_name()`, `.file_stem()`, and `.to_str()` are much simpler, as they all return `&str`.

The crate also includes other convenient, common APIs:

- `read_to_string`, which reports the file path if not found.
- `get_buf_reader`, which also reports the file path if not found or in case of an error.
- `load_json` and `save_json` with the `with-json` feature.
- `load_toml` and `save_toml` with the `with-toml` feature.
- `iter_files(dir, include_globs: &[&str], exclude_globs: &[&str]) -> Result<impl Iter...>`

For more control, it is recommended to use `std::fs`, `walkdir`, `globset`, and other crates directly.


## Cargo Features

- `with-json` - For `load_json`, `save_json`, `save_json_pretty`
- `with-toml` - For `load_toml`, `save_toml`
- `bin-nums` - For `save_be_f64`, `save_le_f64`, `load_be_f64`, ...


This is a very early implementation, with more to come.

Happy coding!