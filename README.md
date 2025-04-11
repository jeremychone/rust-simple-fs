# simple-fs

[simple-fs](https://github.com/jeremychone/rust-simple-fs) is a crate that provides a set of convenient and common file APIs built on `std::fs`, [walkdir](https://crates.io/crates/walkdir), and [globset](https://crates.io/crates/globset).

## Concept

`simple-fs` operates under the assumption that paths that are not `utf8` are not visible to the API, simplifying many of the path-related APIs.

The main construct of `simple-fs` is the `SPath` structure, which contains a `Utf8PathBuf` and ensures the following:

- It is UTF-8 by contract.
- Posix normalize the path, meaning only `/` (no `\`), and redundant `//` or `/./` are collapsed to one `./`.
- It does not have any `\\?\` on Windows.

The `SFile` is a File struct that contains a `SPath`.

By applying the above rules, path/file APIs can be drastically simplified, and both structs offer many Path functions, with `&str` as the return type.

This crate also offers a simple and scalable way to list or iterate on files, given a glob:

- `iter_files(dir, include_globs: Option<&[&str]>, list_options: Option<ListOptions>) -> Result<impl Iter SFile>`
- `list_files(dir, include_globs: Option<&[&str]>, list_options: Option<ListOptions>) -> Result<Vec<SFile>>`
- `ensure_dir(dir_path)` make sure all the dir paths are created.
- `ensure_file_dir(file_path)` makes sure the file directory exists.

The crate also includes other convenient, common APIs:

- `read_to_string`, which reports the file path if not found.
- `get_buf_reader`, which also reports the file path if not found or in case of an error.

For more control, it is recommended to use `std::fs`, `walkdir`, `globset`, and other crates directly.

This is a very early implementation, with more to come.

Happy coding!

## Cargo Features

| Feature     | Functions Included                               |
|-------------|--------------------------------------------------|
| `with-json` | `load_json`, `save_json`, `save_json_pretty`     |
| `with-toml` | `load_toml`, `save_toml`                         |
| `bin-nums`  | `save_be_f64`, `load_be_f64`, `save_le_f64`, ... |
| `full`      | All the above.                                   |
| default     | None of the above. See below.                    |

## Changes

- `0.6.x` (rc for now)
  - NOW uses Utf8 by default; std path moved to `..std_path..` naming.
  - NOW normalizes all SPath to be Posix based (i.e., `/` and remove redundant `//` and `/./`)
  + `SPath/SFile`.
  - `!` deprecate '.to_str()' now '.as_str()'
  - `!` .diff(..) - Now take AsRef Utf8Path, return Option<SPath> (use try_diff(..) for Result)
  - `+` Add `collapse`, `into_collapsed`, `is_collapsed`, `try_collapse`
  - `!` `.clean(..)` is replaced by `.collapse()`
  - `+` list/iter files/dirs - add support for negative glob patterns in include_globs (convenience)
  - `!` API CHANGE - now all default to Utf8Path (from camino crate). `std_path...()` for the std path
  - `^` sfile/spath - add is_absolute/is_relative passthrough
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
