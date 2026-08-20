# NROS Safety & Concurrency Threat Model — per AUDIT.md Pass 6 Next Step

> Formal: Invariant → Unsafe boundary → Possible violation → Exploitability → Test required → Fix → Verification gate

## 1. nros-core — Ring Buffer Unsafe Boundary

| Invariant | Unsafe Boundary | Possible Violation | Exploitability | Test Required | Fix | Verification Gate |
|-----------|-----------------|--------------------|----------------|---------------|-----|-------------------|
| One producer reservation per slot MUST | `try_reserve()` reads `write_idx`/`read_idx` and returns `*mut T` | Two `allocate()` return same slot → two `&mut T` aliasing | Safe Rust code can create UB via aliasing `&mut` | `test_double_reserve_prevention` — second reserve must fail while outstanding | `write_reserved: AtomicBool` CAS, at most one `WriteGuard` | `cargo test test_double_reserve_prevention -- --nocapture` + Miri |
| No &T after release MUST | `try_read()` returns `&T` + separate `consume()` | `let x = try_recv(); consume(); use(x)` — old &T outlives slot reuse, producer overwrites while old &T alive | Safe Rust can cause use-after-free / aliasing violation via raw pointer write | `test_read_guard_lifetime` — read_idx not advanced while guard alive, advanced after drop | `ReadGuard<'a,T>` owns slot, `Deref` only while alive, `Drop` advances | `cargo test test_read_guard_lifetime` |
| Initialized T dropped exactly once MUST | `alloc(Layout::array::<T>)` + `dealloc` only, no `drop_in_place` | `T=String` with heap allocation leaks, `RingBuffer` Drop doesn't call `drop_in_place` for remaining initialized slots | Resource leak, not UB but safety-relevant for generic API | `test_generic_t_destruction` with `DropCounter(Arc<AtomicUsize>)` asserts count 1, test remaining drain on Drop | Buffer `*mut MaybeUninit<T>`, `ReadGuard::drop` calls `drop_in_place`, `RingBuffer::drop` drains `[read,write)` | `cargo test test_generic_t_destruction` |
| Producer cannot overwrite acquired slot MUST | Full check `write - read >= cap` uses Relaxed/Acquire ordering | If ordering wrong, producer may see stale read_idx and overwrite slot still owned by consumer | Data race, corruption | `test_ring_buffer_full` + stress test `test_spsc_ordering` 100 msgs order preserved | Acquire load read_idx in try_reserve, Release store write_idx on commit | `cargo test test_ring_buffer_full` + Miri |
| Send/Sync justified MUST | `unsafe impl<T: Send> Send/Sync for RingBuffer<T>` | If T not Send but Sync impl allows sharing via Arc, consumer could access non-Send T from another thread | Data race via interior mutability | Review Send/Sync bounds, document justification, test with `T=Sync` and `T=Send` only | Document invariants, keep `T: Send` for both Send/Sync with SPSC discipline, reservation flags prevent aliasing | `cargo test` + `cargo miri test` |

## 2. nros-transport — Checksum & Multicast

| Invariant | Unsafe Boundary | Violation | Exploitability | Test | Fix | Gate |
|-----------|-----------------|-----------|----------------|------|-----|------|
| Payload integrity MUST | `with_checksum` computed but not verified in receive | Corruption undetected, deserialization of corrupted data | Silent data corruption in robotics (safety-critical) | Loopback test with corrupted byte flips checksum mismatch | Added `verify_checksum()` and enforced in `UdpTransport::receive` + `TcpTransport::receive` before decompress | `cargo test test_udp_loopback` + inject corruption test |
| Multicast group join MUST | `multicast_group()` stub println only | Claims multicast but doesn't join, one-to-many without per-subscriber overhead not demonstrated | Feature claim without implementation | No test before, now real join via `set_multicast_ttl_v4` + `join_multicast_v4` | Implemented real multicast join parsing IP from `224.0.0.1:5000`, `Ipv4Addr::UNSPECIFIED` | Manual test with `cargo test` and network capture |

## 3. nros-distributed — Raft & Replication

| Invariant | Boundary | Violation | Exploitability | Test | Fix | Gate |
|-----------|----------|-----------|----------------|------|-----|------|
| Leader election must be Raft RequestVote RPC with log up-to-date check | `should_grant_vote` uses `random_bool(0.7)` | Claims Raft but uses random, split-brain not prevented, no log replication | Distributed consensus unsound, fleet may have multiple leaders | `test_leader_election` only checks role transitions, not Raft correctness | Separated `SimulatedElection` (random) vs `RaftElection` scaffolded with `current_term, voted_for, log, commit_index, request_vote_rpc, append_entries_rpc` placeholders, trait `ElectionEngine { is_simulated() }` | Need real Raft lib (e.g., `raft` crate) or implement Raft §5.2 |
| Replication must persist to majority | `replicate()` returns `Ok(())` stub | Claims replication factor 3 but only local HashMap | Data loss on node failure, fleet param not replicated | `test_distributed_state` only local | Added `ReplicationMode::{Simulated, Real}`, `is_simulated()`, `set()` logs SIMULATED vs REAL distinction | Need consistent hash ring + Raft log replication |

## 4. nros-hal — DMA

| Invariant | Boundary | Violation | Exploitability | Test | Fix | Gate |
|-----------|----------|-----------|----------------|------|-----|------|
| Zero-copy DMA buffer must be memfd + mmap + DMA-BUF GPU-accessible | `DmaBuffer { data: Vec<u8> }` | Claims zero-copy DMA but uses Vec + clone() | Copy overhead, not zero-copy, GPU cannot DMA into Vec | `test_camera_dma` checks id not DMA | Separated `SimulatedDmaBuffer(Vec<u8>)` SIMULATED vs `RealDmaBuffer` SCAFFOLDED would use memfd_create + mmap + dma_buf, trait `DmaBufferTrait`, type alias `DmaBuffer = SimulatedDmaBuffer` | Need `memfd_create`, `mmap`, `dma_buf` via `nix` crate + hardware validation on Jetson |

## 5. nros-studio — Telemetry

| Invariant | Boundary | Violation | Exploitability | Test | Fix | Gate |
|-----------|----------|-----------|----------------|------|-----|------|
| Metrics must be from real nodes | `to_metric_json()` uses `SystemTime + pseudo_rand()` | Dashboard looks live but data synthetic, cannot be used as benchmark evidence | Misleading benchmark claims 6.2μs, 780K msg/s if using Studio screenshot | Manual preview shows synthetic | Separated `DemoDataProvider` SIMULATED pseudo_rand hard-coded vs `LiveNrosDataProvider` SCAFFOLDED would collect `PerformanceStats`, trait `DataProvider`, JSON includes provider name + simulated bool | Need to wire real `PerformanceStats` + `ExecutionStats` + sysinfo crate |

## 6. nros-cli — Build System

| Invariant | Boundary | Violation | Exploitability | Test | Fix | Gate |
|-----------|----------|-----------|----------------|------|-----|------|
| Binary size must be measured not simulated | `BuildSystem::build()` prints 950KB/480KB hard-coded | Claims embedded 480KB but never measures `target/` binary via `fs::metadata` | Misleading deployment claim 500KB binary 2MB RAM | `test_build_system` only checks size>0 | Labeled as `[SIMULATED — would measure target/{profile}/binary]` per EVIDENCE_REGISTRY, added note | Need real `cargo build --profile` + `fs::metadata` measurement |
| Topic inspector must read real NROS topics | `TopicInspector::list()` hard-coded topics with latency 5.2μs | Claims live topic list but hard-coded | User sees fake topics | None | Labeled SIMULATED per evidence registry, future would query `nros-core` Publisher/Subscriber registry |

## Verification Gates

- [x] Safety Gate v0.1 — ownership + lifetime + destruction + concurrency proof for core (implemented)
- [ ] CI Gate — `.github/workflows/ci.yml` fmt, check, test correctness only, clippy, bench ignored, safety-gate Miri + specific tests, nros-init golden test (file exists locally, needs manual addition via GitHub UI due to workflows permission)
- [ ] Miri Gate — `cargo miri test -p nros-core` best effort
- [ ] Loom Gate — concurrency stress with loom crate
- [ ] Benchmark Artifact Gate — `cargo run -p nros-core --bin bench -- --output benchmarks/results.json` with env info CPU model, OS, kernel, rustc version, commit, affinity, message_size, latency distribution
- [ ] Hardware Gate — V4L2, LiDAR, Jetson validation for HAL
- [ ] Safety-Qualifiable Gate — ISO 26262 / IEC 61508 process, MC/DC, formal proof

## References

- AUDIT.md Pass 6-7 risk register NROS-001..012, CORE-001..010
- SAFETY.md invariant table
- EVIDENCE_REGISTRY.md status taxonomy
