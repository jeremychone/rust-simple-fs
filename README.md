# simple-fs

Simple File System APIs built on `std::fs`, `walkdir`, `globset`, and serde's `json` and `toml`, along with their respective `with-json` and `with-toml` features.

This crate does not intend to meet all file system access needs or replace the above crates. Instead, it focuses on providing  convenient, simple API for common operations such as:

- Loading/saving JSON or TOML files.
- Loading a string from a path with `filename` in the error message when a file is not found.
- Getting a `buf_reader` with `filename` in the error message when a file is not found.
- Iterating over specific files in a directory with globs (includes and excludes).
- Simplifying the retrieval of `&str` from `path`, `file_name`, `file_stem`, `extension` using the `SFile` construct.

For more control, it is recommended to use `std::fs`, `walkdir`, `globset`, and other crates directly.

This is a very early implementation, with more to come.

Happy coding!