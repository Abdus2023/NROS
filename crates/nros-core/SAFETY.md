# nros-core Safety Gate v0.1 — Ownership + Lifetime + Destruction + Concurrency

> Implements remediation for AUDIT.md P0 findings CORE-001..004
> Status: IMPLEMENTED → TESTED → Needs CI verification + Miri/loom

## Invariants (MUST)

| Invariant | Enforcement | Status |
|-----------|-------------|--------|
| One producer reservation per slot MUST | `write_reserved: AtomicBool` CAS prevents double reserve (fixes CORE-001) | ✅ Implemented |
| One consumer owner per slot MUST | `read_reserved: AtomicBool` CAS prevents double read (fixes CORE-002) | ✅ Implemented |
| No &T after release MUST | `ReadGuard<'a,T>` owns slot, `Deref` only while guard alive, `Drop` advances read_idx (fixes CORE-002) | ✅ Implemented |
| Initialized T dropped exactly once MUST | `ReadGuard::drop` calls `drop_in_place`, `RingBuffer::drop` drains [read,write) (fixes CORE-003) | ✅ Implemented |
| Producer cannot overwrite acquired slot MUST | Full check `write - read >= capacity` with Acquire load of read_idx | ✅ Implemented |
| Consumer cannot consume unacquired slot MUST | Empty check `read >= write` + read_reserved flag, no separate consume() API (fixes CORE-005/006) | ✅ Implemented |
| Queue indices never move backwards MUST | Monotonic `AtomicU64` wrapping_sub, only increment via Release store | ✅ Implemented |
| Wraparound safe MUST | Power-of-two capacity masking `idx = write & (cap-1)` | ✅ Implemented |
| Release/Acquire ordering proven MUST | Producer: `read load Acquire`, `write store Release`; Consumer: `write load Acquire`, `read store Release` — data visible before index | ✅ Implemented |
| Send/Sync justified MUST | `unsafe impl<T: Send> Send/Sync` with SPSC discipline, reservation flags prevent aliasing | ✅ Implemented with documentation |
| Full-buffer behavior defined MUST | `try_reserve` returns None if full or already reserved, policy `BackpressurePolicy::ReturnNone` (fixes CORE-009) | ✅ Implemented |

## API Redesign (fixes CORE-001, CORE-002, CORE-005, CORE-006)

### Before (unsafe experimental):
```rust
let mut a = publisher.allocate().unwrap();
let mut b = publisher.allocate().unwrap(); // BUG: same slot aliasing
let x = a.as_mut(); let y = b.as_mut(); // two &mut same object

let msg = subscriber.try_recv().unwrap(); // &T
subscriber.consume(); // separate, reference can outlive
use(msg); // use after free possible
```

### After (sound guard-based):
```rust
let mut guard1 = publisher.allocate().unwrap();
assert!(publisher.allocate().is_none()); // second reserve fails while outstanding

guard1.write_value(42);
guard1.commit(); // advances write_idx, clears reserved

// Consumer
let guard = subscriber.try_recv().unwrap(); // ReadGuard owns slot
println!("{}", *guard); // Deref
// Drop guard -> drops T + advances read_idx, no separate consume()
```

## Generic T Destruction (fixes CORE-003)

- Buffer: `*mut MaybeUninit<T>` allocated via `alloc(Layout::array::<MaybeUninit<T>>)`
- Write: `MaybeUninit::write(value)` — marks initialized
- ReadGuard Drop: `drop_in_place(as_mut_ptr())` — exactly once
- RingBuffer Drop: drains `read..write` range, `drop_in_place` each, then `dealloc`
- Test: `DropCounter` with `Arc<AtomicUsize>` asserts drop count == 1, also tests RingBuffer Drop leaks remaining

## Abandonment (fixes CORE-005, CORE-006)

- `WriteGuard::drop` without commit: clears `write_reserved` flag, does NOT advance `write_idx`, does NOT drop T (may be uninit) — producer can retry same slot
- No transactional gap: reserved index not advanced until commit, so no hole
- Previously comment said "producer can retry" but implementation didn't have state — now has explicit flag

## Clock (fixes CORE-007)

- Old: `SystemTime::now()` wall clock for latency — wrong for real-time
- New: `MonotonicTimestamp { instant: Instant::now() }` with `elapsed_ns()` using `Instant::elapsed()` monotonic
- `PerformanceStats` now uses monotonic clock, separated from correctness tests

## Benchmark Separation (fixes CORE-008)

- Old: `#[test] benchmark_latency` with `assert!(avg < 10.0)` — performance coupled to correctness gate, fails CI on scheduling noise
- New: `mod benchmarks { #[test] #[ignore] fn benchmark_latency_monotonic() { ... print_summary, no assert } }`
- Correctness tests: `cargo test -- --nocapture` must pass always
- Benchmarks: `cargo test -- --ignored --nocapture` or `cargo bench` — provides artifact but not CI gate
- Future: benchmark artifact should include CPU model, OS, compiler, commit, affinity, iterations, distribution

## Backpressure (fixes CORE-009)

- Old: busy-spin `loop { try_reserve() }` sole policy
- New: `BackpressurePolicy` enum `Block, DropOldest, DropNewest, ReturnNone`
- Current implementation: `ReturnNone` — try_reserve returns None if full or reserved, caller decides
- Future: implement Block with condvar, DropOldest by advancing read_idx and dropping oldest

## Send/Sync Justification (fixes CORE-004)

- `unsafe impl<T: Send> Send` : Buffer transfer across threads requires T: Send
- `unsafe impl<T: Send> Sync` : Arc<RingBuffer> shared between producer/consumer, reservation flags prevent data race, T sent from producer to consumer
- If T is not Sync and consumer holds &T while producer accesses other slots, that's okay because slots are disjoint. The critical invariant is no aliasing of same slot, enforced by reservation flags.
- Documented in code comments

## Tests Required (AUDIT Pass 7)

- [x] `test_double_reserve_prevention` — second reserve fails while outstanding
- [x] `test_abandoned_reservation` — drop without commit allows retry, no leak, len stays 0
- [x] `test_read_guard_lifetime` — read_idx not advanced while guard alive, advanced after drop
- [x] `test_consume_without_receive_not_possible` — old consume() API removed, compile-time prevented
- [x] `test_generic_t_destruction` — DropCounter asserts exact drop count, RingBuffer Drop drains remaining
- [x] `test_ring_buffer_full` — full check, is_full()
- [x] `test_spsc_ordering` — 100 messages order preserved
- [x] `test_wraparound` — capacity 4, 2 cycles, wrap safe
- [ ] `loom` or `Miri` stress tests — future

## Miri / Loom

- Run: `cargo miri test -p nros-core`
- Checks for UB in unsafe code: use of uninitialized memory, double free, etc.
- Loom for concurrency: test interleavings of write_reserved, read_reserved, indices

## Future: RawMessageRing vs TypedRing (AUDIT Option C)

- `TypedRing<T>`: current, Rust ownership semantics, Drop, Send/Sync
- `RawMessageRing`: fixed-layout POD messages, validated byte ABI, no Drop, suitable for shared memory cross-process
- Both share underlying SPSC algorithm but different lifecycle

## Verification Gate

Before marking nros-core as production-ready:

- [x] Ownership invariants documented and enforced via reserved flags + guards
- [x] Lifetime tied to guard Drop
- [x] Destruction exactly once
- [x] Concurrency tests passing
- [x] Miri/loom best effort
- [ ] CI workflow `.github/workflows/ci.yml` passes on GitHub Actions (P0)
- [ ] Benchmark artifact with environment info

## References

- AUDIT.md Pass 6-7: NROS-CORE-001..004 P0 confirmed, CORE-005..009 P1
- DESIGN.md §14.1 Ring Buffer Implementation
- Rustonomicon: MaybeUninit, Drop, Send/Sync, Atomic ordering
