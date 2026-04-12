# Architecture

## Why v1 is scan-based
This project is a learning exercise focused on Rust systems programming and fast filesystem crawling. A scan-based v1 keeps the implementation grounded in traversal, matching, error handling, and concurrency without prematurely introducing a daemon or persistent index.

## Stable seam for future indexed backends
A future indexed backend only needs to satisfy these contracts:
- `SearchQuery`: root path, pattern, match target, case-sensitivity mode, hidden-file policy, mount-boundary policy, symlink policy
- `SearchResult`: matched path, file type hint, and backend-agnostic display/accounting metadata
- `SearchEngine`: produce `SearchResult` items plus structured non-fatal diagnostics from a `SearchQuery`
- CLI output contract: one match per stdout line, non-fatal diagnostics on stderr, non-zero exit codes only for fatal invocation/setup errors

## Execution paths
### Parallel shipping backend
The default backend uses a bounded worker queue. Workers enumerate directories, enqueue subdirectories, and emit matches inline to minimize extra passes. Concurrency is intentionally internal to the scan backend.

### Sequential reference path
The reference implementation traverses the tree single-threaded using the same query, matcher, and result normalization semantics as the parallel backend. It exists to:
- provide deterministic correctness checks
- compare normalized results against the shipping backend
- keep concurrency bugs from hiding behind performance work

## Filesystem policy
- Directory symlinks are not followed in v1
- Symlink entries themselves may still be reported if they match
- Permission-denied directories are reported as non-fatal diagnostics
- Hidden files are included by default
- Mount-boundary enforcement uses device-ID checks so traversal stays on the root filesystem by default
- Output order favors traversal completion speed; deterministic normalized comparison is used in tests

## Why there is no platform layer yet
The project is Linux-first, but v1 does not introduce a dedicated platform module until a concrete Linux-specific primitive forces that separation. Avoiding speculative layers keeps the code small and instructional.
