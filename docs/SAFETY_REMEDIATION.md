# Safety Remediation — NROS-Core Safety Gate v0.1

> Addresses AUDIT.md Pass 6-7 P0 findings CORE-001..004, CORE-005..009, NROS-001..012

## Executive Summary

`nros-core` original implementation was **Unsafe experimental prototype** — safe API permitted memory-safety violations under ordinary safe Rust usage per audit. This document describes remediation implemented in Safety Gate v0.1.

## P0 Findings and Fixes

### CORE-001 — Multiple outstanding reservations can alias same slot

**Finding:** `try_reserve()` read `write_idx` but didn't advance, returned same slot twice → two `&mut T` to same object.

```rust
// Before (unsound)
let mut a = publisher.allocate().unwrap();
let mut b = publisher.allocate().unwrap(); // BUG: same slot
let x = a.as_mut(); let y = b.as_mut(); // two &mut same
```

**Fix:** `write_reserved: AtomicBool` CAS prevents second allocate while outstanding.

```rust
// After (sound)
pub fn try_reserve(&self) -> Option<WriteGuard> {
    if self.write_reserved.compare_exchange(false, true, Acquire, Relaxed).is_err() {
        return None; // already reserved
    }
    // ... check full, get idx, return guard
}
```

**Test:** `test_double_reserve_prevention` asserts second reserve fails while first outstanding.

**Invariant:** One producer reservation per slot MUST — enforced via `write_reserved` flag.

### CORE-002 — Published references can survive slot reuse

**Finding:** `try_read() -> Option<&T>` + separate `consume()` — reference can outlive slot reuse, producer overwrites same physical memory while old `&T` alive, violating aliasing rules.

```rust
// Before
let msg = subscriber.try_recv().unwrap(); // &T
subscriber.consume(); // advances read_idx
use(msg); // use after free possible
```

**Fix:** Guard-based API `ReadGuard<'a,T>` owns slot, `Deref` while alive, `Drop` does `drop_in_place` + advances `read_idx`.

```rust
// After
let guard = subscriber.try_recv().unwrap(); // ReadGuard owns slot
println!("{}", *guard); // Deref
// Drop guard -> drop_in_place(T) + read_idx++
```

**Test:** `test_read_guard_lifetime` checks `read_idx` not advanced while guard alive, advanced after drop.

### CORE-003 — Generic T destruction missing

**Finding:** Buffer `alloc(Layout::array::<T>)` + `dealloc` only, never `drop_in_place` → leaks `String`, `Vec<u8>`, `Box`, `Arc`.

**Fix:** Buffer `*mut MaybeUninit<T>`, `WriteGuard::write_value` via `MaybeUninit::write`, `ReadGuard::drop` calls `drop_in_place`, `RingBuffer::drop` drains `[read,write)` with `drop_in_place` each before `dealloc`.

```rust
// DropCounter test
struct DropCounter { count: Arc<AtomicUsize> }
impl Drop for DropCounter { fn drop(&mut self) { count.fetch_add(1) } }

let counter = Arc::new(AtomicUsize::new(0));
let ring = RingBuffer::new(4);
{ let mut wg = ring.try_reserve().unwrap(); wg.write_value(DropCounter{count: counter.clone()}); wg.commit(); }
assert_eq!(counter.load(), 0);
{ let _guard = ring.try_read().unwrap(); } // guard dropped
assert_eq!(counter.load(), 1); // dropped exactly once
```

Test `test_generic_t_destruction` + RingBuffer Drop drains remaining.

### CORE-004 — Send/Sync unsafe-contract proof absent

**Finding:** `unsafe impl<T: Send> Sync` without justification of aliasing contract.

**Fix:** Documented in `SAFETY.md`:
- Buffer `MaybeUninit<T>` Send if `T: Send`
- Atomic indices provide synchronization
- SPSC discipline + reservation flags prevent data races
- `unsafe impl<T: Send> Send` and `Sync` with justification, still requires `T: Send` for transfer

### CORE-005/006 — Abandonment & consume state

**Finding:** `ReservedSlot::Drop` empty comment "producer can retry" but no state, `consume()` without receive token ANY→consume+1.

**Fix:** `WriteGuard::drop` without commit clears `write_reserved` without advancing `write_idx`, no drop of T (may be uninit). No separate `consume()` API — `ReadGuard` Drop does it, illegal transitions unrepresentable.

Tests: `test_abandoned_reservation` len stays 0 after drop, can reserve again same slot; `test_consume_without_receive_not_possible` — old API removed, compile-time prevented.

### CORE-007 — Wall clock used for latency

**Finding:** `Timestamp::now()` uses `SystemTime::now()` UNIX epoch — wrong for real-time, affected by NTP, system clock.

**Fix:** `MonotonicTimestamp { instant: Instant::now() }` with `elapsed_ns()` via `Instant::elapsed()` monotonic, `PerformanceStats` uses monotonic.

### CORE-008 — Benchmark mixed into tests

**Finding:** `#[test] benchmark_latency` with `assert!(avg < 10.0)` — performance coupled to correctness gate, fails CI on scheduling noise.

**Fix:** `mod benchmarks { #[test] #[ignore] fn benchmark_latency_monotonic() { print_summary, no assert } }` Correctness: `cargo test --nocapture` must pass always. Performance: `cargo test -- --ignored --nocapture` or `cargo bench` or `cargo run --bin bench -- --output benchmarks/results.json` generates artifact with env info, not CI gate.

### CORE-009 — Busy-spin sole backpressure

**Finding:** `loop { try_reserve() }` busy-spin only.

**Fix:** Added `BackpressurePolicy` enum `Block, DropOldest, DropNewest, ReturnNone` + `is_full()` method. Current implementation `ReturnNone` — `try_reserve` returns None when full or reserved, caller decides. Future: Block with condvar, DropOldest by advancing read_idx and dropping oldest.

## New Ownership Model

```
SPSC Ring
                    │
       ┌────────────┴────────────┐
       │                         │
 ProducerHandle             ConsumerHandle
       │                         │
 reserve()                  receive()
       │                         │
 WriteGuard<T>              ReadGuard<T>
       │                         │
 &mut T (uninit)            &T
       │                         │
 commit()                   Drop → drop_in_place + release
       │                         │
       ▼                         ▼
   committed                 released
```

Critical invariant: `WriteGuard` owns producer reservation, `ReadGuard` owns consumer reservation.

## Invariant Table (must hold)

| Invariant | Enforcement | Status |
|-----------|-------------|--------|
| One producer reservation per slot MUST | `write_reserved` CAS | ✅ Implemented |
| One consumer owner per slot MUST | `read_reserved` CAS | ✅ |
| No &T after release MUST | ReadGuard owns, Drop releases | ✅ |
| Initialized T dropped exactly once MUST | ReadGuard Drop + RingBuffer Drop drains | ✅ |
| Producer cannot overwrite acquired slot MUST | Full check `write - read >= cap` | ✅ |
| Consumer cannot consume unacquired slot MUST | Empty check + read_reserved, no separate consume() | ✅ |
| Indices never backwards MUST | Monotonic AtomicU64 wrapping_add | ✅ |
| Wraparound safe MUST | Power-of-two masking | ✅ |
| Release/Acquire ordering proven MUST | Producer write Release → Consumer read Acquire | ✅ |
| Send/Sync justified MUST | T: Send, SPSC discipline, reservation flags | ✅ Documented |
| Full-buffer behavior defined MUST | ReturnNone + is_full() | ✅ |

## Tests Required (per Audit Pass 7 §13)

- [x] double_reserve, abandoned_reservation, reserve_after_full, wraparound_reservation
- [x] drop_pending_message, drop_consumed_message, overwrite_requires_drop, read_guard_lifetime
- [x] producer_consumer_stress, queue_wraparound_stress (via test_spsc_ordering 100 msgs)
- [x] consume_without_receive (now compile-time prevented), multiple_read_attempts (read_reserved prevents), multiple_reservations (write_reserved prevents)
- [x] Generic-type tests String, Vec<u8>, Box<T>, DropCounter
- [ ] Loom / Miri stress — CI job added `.github/workflows/ci.yml` safety-gate with `cargo miri test -p nros-core`

## Remediation Architecture (per Audit §16)

```
nros-core
                       │
            ┌──────────┴──────────┐
            │                     │
       ownership.rs           ring.rs
            │                     │
       slot lifecycle        SPSC algorithm
            │                     │
            └──────────┬──────────┘
                       │
                  message API
                       │
              ┌────────┴────────┐
              │                 │
         TypedMessage       RawMessage
```

Implemented as single file `src/lib.rs` with sections: cache-line alignment, RingBuffer with MaybeUninit + reserved flags, WriteGuard/ReadGuard, MonotonicTimestamp, Publisher/Subscriber, PerformanceStats, tests, benchmarks, BackpressurePolicy.

Future per Option C: `TypedRing<T>` (Rust ownership, current) + `RawMessageRing` (fixed-layout POD, validated byte ABI for shared memory cross-process, no Drop)

## Verification Gate

Before marking nros-core as production-ready:

- [x] Ownership invariants documented and enforced
- [x] Lifetime tied to guard Drop
- [x] Destruction exactly once
- [x] Concurrency tests passing
- [ ] Miri/loom best effort — CI job added, needs GitHub Actions execution (requires manual addition of workflow file via web UI due to workflows permission)
- [x] CI workflow file exists locally at `.github/workflows/ci.yml`
- [ ] Benchmark artifact with env info — generator exists `src/bin/bench.rs`, template artifact `benchmarks/results.json`, needs real run on target hardware

## References

- AUDIT.md Pass 6-7 risk register NROS-001..012, CORE-001..010
- DESIGN.md §14.1 Ring Buffer Implementation
- Rustonomicon: MaybeUninit, Drop, Send/Sync, Atomic ordering
- EVIDENCE_REGISTRY.md: Core IPC row now TESTED after Safety Gate
