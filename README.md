# rs-find

`rs-find` is a Linux-first Rust CLI file finder built as a systems-programming learning project. It stays scan-based in v1, but it keeps a minimal seam for a future indexed backend.

## Features
- Recursive local filesystem search
- Default **name-only** matching
- Optional **full-path** matching via `--path`
- Internal parallel scan backend
- Sequential reference path for deterministic correctness checks
- Non-fatal diagnostics for permission-denied paths
- Hidden files included by default
- Mount-boundary enforcement via device-ID checks

## Usage
```bash
cargo run -- <query> <root>
cargo run -- --path <query> <root>
cargo run -- --ignore-case <query> <root>
```

## Filesystem policy
- Directory symlinks are not followed
- Matching symlink entries can still be reported
- Permission-denied directories emit warnings to stderr without aborting the full search
- Hidden files are included by default in v1
- Traversal stays on the root filesystem by default via device-ID checks
- Output ordering is traversal-completion order; tests normalize results before comparing

## Benchmarking
The project includes a lightweight `cargo bench` entrypoint.

```bash
cargo build --release
BENCH_ROOT=/usr BENCH_QUERY=lib cargo bench
```

If `fd` is installed, the bench runner compares `rs-find` and `fd` on the same workload and prints median timings.

## Verification
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo bench
```

## Future direction
V1 is intentionally scan-based. A future indexed backend should only need to implement the stable `SearchQuery` / `SearchResult` / `SearchEngine` contract described in `docs/architecture.md`.
