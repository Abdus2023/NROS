# Benchmark Artifacts — per AUDIT.md Pass 7 §12

This directory should contain machine-generated benchmark artifacts tied to commit and environment, not just repository-reported numbers.

## Required Evidence Gate for Performance Claims

Per AUDIT Pass 7 §12, a quantitative performance claim (e.g., 6.2 μs mean latency, 780K msg/s) must be supported by:

```
cargo bench
    │
    └── benchmark artifact
          │
          ├── CPU model
          ├── OS
          ├── kernel
          ├── compiler (rustc version)
          ├── commit (git rev-parse HEAD)
          ├── timestamp
          ├── capacity
          ├── iterations
          ├── affinity (CPU pinning)
          ├── message_size
          ├── throughput
          ├── latency distribution: min, p50, p95, p99, p99.9, max, mean, stddev
          └── total time
```

Only the artifact with all above fields should support a claim like "6.2 μs mean latency".

## How to Generate

```bash
# Build and run benchmark artifact generator (uses monotonic clock per AUDIT CORE-007)
cargo run -p nros-core --bin bench -- --iterations 100000 --capacity 1024 --output benchmarks/results.json

# For real compression benchmark
cargo run -p nros-transport --bin bench --features real-compression,real-checksum -- --iterations 10000 --output benchmarks/transport.json

# With CPU affinity (requires taskset or core_affinity crate)
taskset -c 2,3 cargo run -p nros-core --bin bench -- --iterations 100000 --output benchmarks/results_pinned.json
```

## Current Status

- `crates/nros-core/src/bin/bench.rs` — artifact generator using monotonic clock (Instant), guard-based SPSC, separated from correctness tests per CORE-008, generates JSON with env info
- `results.json` — placeholder, to be generated on actual hardware, not synthetic
- Previously, `BENCHMARK` was embedded in `#[test] benchmark_latency` with `assert!(avg < 10.0)` — this coupled performance to correctness gate and used SystemTime wall clock (CORE-007, CORE-008) — now fixed: benchmarks are `#[ignore]` and use monotonic clock

## Reproducibility

To reproduce COMPARISON.md numbers:

```bash
# On target hardware (e.g., x86_64, Ubuntu 22.04, rustc 1.75)
cargo test -p nros-core -- --ignored --nocapture  # old benchmark (for reference)
cargo run -p nros-core --bin bench -- --iterations 100000 --output benchmarks/results_$(hostname)_$(date +%Y%m%d).json
cat benchmarks/results_*.json
```

Then update COMPARISON.md with actual measured numbers and link to artifact file commit.

## TODO

- [ ] Add criterion.rs dependency for statistically rigorous benchmarking (optional)
- [ ] Add benchmark for transport with real LZ4 and CRC32
- [ ] Add benchmark for HAL DMA zero-copy vs copy path
- [ ] Add benchmark for node control loop 1 KHz with deadline monitoring
- [ ] CI: benchmark job currently `continue-on-error: true` — should upload artifact to GitHub Actions artifacts, not fail build on perf variance

## References

- AUDIT.md Pass 6-7: benchmark methodology needs p50/p95/p99/p99.9/max/mean/stddev, CPU affinity, frequency scaling, NUMA, cache topology, allocation effects, debug vs release, message size, queue depth, producer/consumer CPU placement
- DESIGN.md §18 Benchmarks & Validation
- EVIDENCE_REGISTRY.md: Benchmark column
