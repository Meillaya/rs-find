# Benchmark Evidence

## 2026-04-13 22:55:26 -04:00

- Workload: local `/usr` tree on the development machine
- Query: `lib`
- Iterations: `3`
- Cache notes: warm-ish cache after local verification run
- Command:

```bash
cargo build --release
BENCH_ROOT=/usr BENCH_QUERY=lib BENCH_ITERATIONS=3 BENCH_CACHE_NOTES="warm-ish cache after local verification run" cargo bench
```

- Result:
  - `rs-find` median: `135.13ms`
  - `fd` median: `79.67ms`
  - `rs-find/fd` ratio: `1.70x`
  - Verdict: within the documented v1 target band

## Notes

- This workload is intentionally local and scan-based; it measures recursive name search against the shipped v1 backend.
- The ratio moved versus the earlier OMX-session evidence because the benchmark is sensitive to machine load and cache warmth; the important check is whether the tool remains in the same practical band as `fd`.
