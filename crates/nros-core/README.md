# nros-core — Sound Zero-Copy SPSC Ring Buffer

Lock-free SPSC ring buffer with guard-based ownership — Safety Gate v0.1 per AUDIT.md

**Fixes P0 findings CORE-001..004:**
- CORE-001 double reservation aliasing → prevented via `write_reserved: AtomicBool` CAS, at most one `WriteGuard` outstanding
- CORE-002 `&T` can outlive slot → `ReadGuard<'a,T>` owns slot, `Deref` while alive, `Drop` advances `read_idx` and `drop_in_place`
- CORE-003 generic `T` destruction missing → `MaybeUninit<T>` buffer, `ReadGuard::drop` + `RingBuffer::drop` drains `[read,write)` with `drop_in_place`
- CORE-004 Send/Sync proof absent → documented invariants + `unsafe impl<T: Send> Send/Sync` with SPSC discipline
- CORE-005/006 abandonment & consume state → `WriteGuard` Drop clears flag without advancing, no separate `consume()` API — illegal transitions unrepresentable
- CORE-007 wall clock → `MonotonicTimestamp` using `Instant::now()` monotonic
- CORE-008 benchmark mixed into tests → benchmarks `#[ignore]`, correctness only in `cargo test`
- CORE-009 busy-spin sole backpressure → `BackpressurePolicy::ReturnNone` + `is_full()` check, future Block/DropOldest

## Architecture

```
Publisher::allocate() -> WriteGuard<'a,T>
  -> as_mut() / as_mut_uninit() / write_value() writes directly into shared memory (no memcpy)
  -> commit() : Release store write_idx (committed) + clear reserved ~20ns

Subscriber::try_recv() -> ReadGuard<'a,T> owns slot
  -> Deref to &T direct ref to shared memory (zero-copy)
  -> Drop: drop_in_place(T) + Release store read_idx (no separate consume())

RingBuffer<T>:
  - buffer: *mut MaybeUninit<T> (Layout::array)
  - write_idx/read_idx each on own 64-byte cache line (AlignedU64)
  - write_reserved/read_reserved: AlignedBool CAS prevents aliasing
  - capacity power-of-two masking idx = write & (cap-1)
  - Invariants: one producer reservation per slot MUST, one consumer owner MUST, no &T after release MUST, Drop exactly once MUST, etc. (see SAFETY.md)
```

## API (guard-based, sound)

```rust
use nros_core::{Publisher, Subscriber, Twist};

let publisher = Publisher::<Twist>::new("/cmd_vel", 1024);
let subscriber = Subscriber::new(publisher.ring(), "/cmd_vel");

// Zero-copy publish — at most one outstanding WriteGuard (CORE-001 fixed)
let mut guard = publisher.allocate().unwrap();
guard.as_mut().linear.x = 1.0; // &mut T to uninit memory — must fully init before commit
// or guard.write_value(Twist::default());
guard.commit(); // Release store, makes visible

// Second allocate while first outstanding would fail:
// let g2 = publisher.allocate(); // None

// Zero-copy subscribe — ReadGuard owns slot (CORE-002 fixed)
if let Some(guard) = subscriber.try_recv() {
    println!("{}", guard.linear.x); // Deref to &T
    // No separate consume() — Drop does drop_in_place + advance read_idx
}
// Old API `subscriber.consume()` no longer exists — compile-time prevented (CORE-006)
```

## Safety Invariants (see SAFETY.md)

| Invariant | Enforcement |
|-----------|-------------|
| One producer reservation per slot | `write_reserved` AtomicBool CAS |
| One consumer owner per slot | `read_reserved` AtomicBool CAS |
| No &T after release | ReadGuard owns, Drop releases |
| Drop exactly once | ReadGuard Drop + RingBuffer Drop drains [read,write) |
| Producer cannot overwrite acquired | Full check `write - read >= cap` |
| Consumer cannot consume unacquired | Empty check + read_reserved |
| Indices never backwards | Monotonic AtomicU64 wrapping_add |
| Wraparound safe | Power-of-two masking |
| Ordering proven | Release/Acquire: write Release → read Acquire sees T |
| Send/Sync justified | T: Send, SPSC discipline, reservation flags |

## Performance Monitoring — Monotonic Clock

`PerformanceStats` uses `Instant::elapsed()` not `SystemTime` (fixes CORE-007), <1μs overhead relaxed atomics: messages_sent/received, total/max/min latency, throughput.

```rust
let stats = PerformanceStats::new();
stats.record_send();
stats.record_receive(latency_ns);
stats.print_summary(elapsed); // no assert in correctness tests
```

## Tests — Correctness only, benchmarks ignored

- `test_zero_copy_pubsub_guard_api` — basic WriteGuard/ReadGuard
- `test_double_reserve_prevention` — second reserve fails while outstanding (CORE-001)
- `test_abandoned_reservation` — Drop without commit allows retry, len stays 0 (CORE-005)
- `test_read_guard_lifetime` — read_idx not advanced while guard alive, advanced after drop (CORE-002)
- `test_consume_without_receive_not_possible` — old consume() API removed, compile-time prevented (CORE-006)
- `test_generic_t_destruction` — DropCounter asserts exact drop count, RingBuffer Drop drains remaining (CORE-003)
- `test_ring_buffer_full`, `test_spsc_ordering`, `test_wraparound`
- Benchmarks: `mod benchmarks { #[ignore] fn benchmark_latency_monotonic() }` — run via `cargo test -- --ignored --nocapture`, no assert (fixes CORE-008)

Run:
```bash
cargo test -p nros-core -- --nocapture          # correctness only, must pass always
cargo test -p nros-core -- --ignored --nocapture # benchmarks, may have perf variance, not CI gate
cargo miri test -p nros-core --lib               # unsafe review
```

## Relation to Design Doc

Implements §14.1 Zero-Copy Shared Memory Architecture, §25 Artifact #1, and Safety Gate v0.1 remediation for AUDIT.md P0.

Future per AUDIT Option C:
- `TypedRing<T>` — Rust ownership (current)
- `RawMessageRing` — fixed-layout POD, no Drop, validated byte ABI for shared memory cross-process

Extensions:
- Shared memory via memfd_create + mmap for cross-process (FD passing)
- MPMC variant with reservation queue + commit queue
- Integration with NROS microkernel scheduler deadline monitoring
- Backpressure policies Block/DropOldest/DropNewest
