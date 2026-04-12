# rs-find

`rs-find` is a Linux-first Rust CLI file finder built as a systems-programming learning project. It stays scan-based in v1, but it keeps a minimal seam for a future indexed backend.

rs-find performs recursive local filesystem search with default **name-only** matching, optional **full-path** matching via `--path`, optional case-insensitive matching via `--ignore-case`, an internal parallel scan backend, and a sequential reference path for deterministic correctness checks; its v1 filesystem policy does **not** follow directory symlinks, still reports matching symlink entries, includes hidden files by default, emits permission-denied paths as non-fatal stderr warnings, and stays on the root filesystem via device-ID boundary checks.

## Usage
```bash
cargo run -- <query> <root>
cargo run -- --path <query> <root>
cargo run -- --ignore-case <query> <root>
```

## Demo
Run the demo script to search a small fixture tree committed in `demo/fixtures/`:

```bash
bash demo/run.sh
```

Or try the commands directly:

```bash
cargo run -- target demo/fixtures
cargo run -- --path rs-find/architecture demo/fixtures
cargo run -- --ignore-case BORROW demo/fixtures
cargo run -- link demo/fixtures
```

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

## References
- [The Rust Programming Language](https://doc.rust-lang.org/book/) — broad Rust background for ownership, modules, errors, and CLI structure.
- [`std::fs::read_dir`](https://doc.rust-lang.org/stable/std/fs/fn.read_dir.html) — directory iteration semantics and ordering caveats relevant to traversal behavior.
- [`std::thread::scope`](https://doc.rust-lang.org/beta/std/thread/fn.scope.html) — scoped-thread model used to structure the parallel walker safely.
- [`std::sync::Mutex`](https://doc.rust-lang.org/beta/std/sync/struct.Mutex.html) — shared-state coordination used in the parallel search path.
- [`std::sync::Condvar`](https://doc.rust-lang.org/std/sync/struct.Condvar.html) — condition-variable primitive used by the bounded work queue.
- [`std::os::unix::fs::MetadataExt`](https://doc.rust-lang.org/std/os/unix/fs/trait.MetadataExt.html) — Unix metadata access used for device-ID filesystem boundary checks.
- [`std::fs` module docs](https://doc.rust-lang.org/stable/std/fs/) — general filesystem API reference for metadata and traversal operations.
- [`sharkdp/fd`](https://github.com/sharkdp/fd) — comparison target and a useful reference point for search-tool behavior and performance tradeoffs.
