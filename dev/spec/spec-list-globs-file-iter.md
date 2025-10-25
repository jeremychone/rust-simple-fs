
## list-globs-file-iter

### Goal

Provide a flexible file iteration utility that accepts include and exclude glob patterns, supports both absolute and relative patterns, and walks the filesystem efficiently while avoiding duplicate outputs. The iterator cooperates with `ListOptions` to respect caller-specified exclusions, relative matching, and depth limits.

### Module Parts

- `GlobsFileIter`  
  Wraps the boxed iterator that yields `SFile` values. The `new` constructor orchestrates pattern preprocessing, directory walking, filtering, and final deduplication.
- `GlobGroup`  
  Internal struct `{ base: SPath, patterns: Vec<String>, prefixes: Vec<String> }` describing a single walk scope. Each group anchors a subset of glob patterns to a concrete base directory and records literal directory prefixes that can prune traversal.
- `process_globs(main_base, globs)`  
  Builds `GlobGroup` instances. Absolute patterns are normalized via `longest_base_path_wild_free` and rewritten relative to the derived base. Relative patterns are cleaned of leading `./` and stripped of the caller’s base prefix. Overlapping bases are merged so that shared directories are walked once, and brace-expanded literal prefixes are captured for pruning.
- Support helpers (`relative_from_absolute`, `directory_matches_allowed_prefixes`, `glob_literal_prefixes`, `expand_brace_segment`, `segment_contains_wildcard`, `append_adjusted`, `normalize_prefixes`)  
  These routines transform patterns into relative forms, expand brace syntax, compute safe directory prefixes, and keep prefix lists deduplicated for the pruning logic used during traversal.

### Public Module API

- `GlobsFileIter::new(dir: impl AsRef<Path>, include_globs: Option<&[&str]>, list_options: Option<ListOptions<'_>>) -> Result<GlobsFileIter>`  
  - Splits provided patterns into positive includes and negated excludes (prefixed with `!`). Negated entries are funneled into `ListOptions::exclude_globs`. When no includes remain, the iterator defaults to `"**"` so the walk still yields files.  
  - Creates `GlobGroup` batches, one per collapsed base directory, each with an individual `globset` matcher. Depth is chosen by `get_depth` per group, overridden by `ListOptions::depth` when present.  
  - Honors `ListOptions::relative_glob` by matching include and exclude globs against paths relative to the root directory; otherwise matches use absolute `SPath`s.  
  - Applies excludes from caller options or the module default (`DEFAULT_EXCLUDE_GLOBS`) to both directory entries (during traversal) and files (before emission).  
  - Limits directory traversal with recorded literal prefixes so directories outside eligible glob prefixes are skipped early.  
  - Chains iterators from every group, then deduplicates results via a `HashSet<SPath>` which ensures each file path is yielded at most once even if multiple patterns match it.

- `impl Iterator for GlobsFileIter<Item = SFile>`  
  Delegates to the boxed iterator assembled by `new`, exposing standard iterator behavior for consumers of the module.
