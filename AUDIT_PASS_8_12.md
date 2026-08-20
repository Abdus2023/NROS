# NROS — Deep Analysis & Verification — Pass 8 to 12

Branch: `arena/01a0188d-nros`

---

## Pass 8 — State Change Verification

I re-verified the actual requested branch, rather than continuing to rely on the earlier audit snapshot. This uncovered an important state change: the branch now contains a substantially revised nros-core safety implementation, while the repository's latest main has also received a CI commit that I cannot yet establish as present on this branch.

### 1. First: the repository state has changed

The branch's current Cargo.toml now declares **10 workspace members**, not the eight I reported earlier:

```
crates/nros-core
crates/nros-node
crates/nros-hal
crates/nros-transport
crates/nros-distributed
crates/nros-cli
crates/nros-sim
crates/nros-studio
crates/nros-macros
crates/nros
```

This is significant.

The previous concern about the generated `nros::prelude::*` API being unsupported is therefore partially addressed: there are now explicit `nros-macros` and `nros` facade crates in the workspace.

So I am downgrading that earlier finding from “confirmed broken” to “must be build-verified.”

### 2. nros-core has undergone a genuine safety refactor

The current source explicitly describes itself as:

> “Sound Zero-Copy SPSC Ring Buffer”

and says it implements fixes for:

```
CORE-001
CORE-002
CORE-003
CORE-004
```

The architecture has changed from naked reservation/consumption into:

```
RingBuffer
    │
    ├── WriteGuard
    │
    └── ReadGuard
```

with explicit reservation flags:

```
write_reserved
read_reserved
```

This is a real architectural improvement, not merely documentation.

### 3. CORE-001 is substantially fixed

The old problem was:

```
reserve()
reserve()
 ↓
same slot
```

The new implementation uses:

```
compare_exchange(false, true, ...)
```

on `write_reserved`.

Therefore:

```
reserve #1 → succeeds
reserve #2 → None
```

until the first reservation is committed or dropped.

The new test explicitly checks this:

```
guard1 = reserve()
reserve() == None
commit guard1
reserve() == Some(...)
```

**Verdict:** CORE-001: 🟢 Design-level fix present — but not yet CI-verified for this branch.

### 4. CORE-002 is also substantially fixed

Old API returned `&T` and separately required `consume()`.

New API returns `ReadGuard<'_, T>` and guard owns slot until Drop.

New test verifies:

```
receive
   ↓
read_idx unchanged
   ↓
drop guard
   ↓
read_idx advances
```

**Verdict:** CORE-002: 🟢 Design-level fix present

### 5. CORE-003 is substantially fixed

New implementation explicitly calls `ptr::drop_in_place(...)` when ReadGuard released, and walks remaining initialized entries when ring itself dropped.

Test introduces real `DropCounter` and checks objects destroyed exactly once.

**Verdict:** CORE-003: 🟢 Design-level fix present

### 6. BUT: I found a new P0 problem in the attempted fix

The new implementation contains:

```rust
pub fn as_mut(&mut self) -> &mut T {
    unsafe { &mut *(*self.ptr).as_mut_ptr() }
}
```

Source itself acknowledges:

> “This is technically creating &mut T to uninitialized memory, which is UB if read before write.”

That is not acceptable for API advertised as safe zero-copy abstraction.

Test uses:

```rust
guard.as_mut().linear.x = 1.0;
guard.as_mut().angular.z = 0.5;
```

So project's own correctness test exercises problematic API.

**🔴 CORE-011 — P0 WriteGuard::as_mut() exposes safe method that can create mutable reference to uninitialized T**

### 7. The correct solution is MaybeUninit, not “document the caller”

Implementation already has `as_mut_uninit()` returning `&mut MaybeUninit<T>` — correct foundation.

Cleanest remediation:

- Remove `as_mut()`
- Keep `as_mut_uninit()`
- Prefer `write_value(T)` for normal safe publishing.

### 8. Another serious issue: benchmark is now synthetic

New benchmark labeled `benchmark_latency_monotonic` says it fixes monotonic-clock problem, but consumer records:

```rust
stats_clone.record_receive(1000);
```

Hard-coded 1 μs latency, not measured.

**Verdict:** 🔴 CORE-012 — P0/P1 — benchmark output cannot be used as evidence of actual IPC latency.

### 9. Throughput is measured, latency isn't

Benchmark does measure elapsed wall time for 100,000 messages and derives throughput — 🟡 Potentially measurable, but latency 🔴 Not actually measured. Should capture timestamps at publication and consumption.

### 10. MonotonicTimestamp isn't actually used for benchmark

Source introduces `MonotonicTimestamp` using `Instant`, but benchmark uses `Instant::now()` for total elapsed and inserts 1000 ns as fake per-message latency.

### 11. Backpressure is only partially implemented

Source declares `BackpressurePolicy { Block, DropOldest, DropNewest, ReturnNone }` but comment says “Future: Publisher with policy would use this enum” — therefore API type 🟢, policy engine ❌, runtime use ❌, tests ❌ — 🟠 CORE-013 — P1 scaffolded, not implemented.

### 12. Send/Sync justification remains insufficiently proven

Still `unsafe impl<T: Send> Send/Sync` with comment WriteGuard/ReadGuard enforce exclusive access — much better, but remains unsafe trait contract needing tests: multiple Publisher/Subscriber handles, cross-thread, guard lifetime across threads, T with non-Sync but Send, plus formal justification.

Verdict: 🟠/🔴 CORE-004 — still not fully verified

### 13. RingBuffer::Drop proof needs edge-case audit

Destructor walks `[read_idx, write_idx)` and calls `drop_in_place`, correct if every slot in interval initialized exactly once, but no explicit per-slot initialization bitmap. Proof relies on `write_idx = number of initialized published elements`, valid only if commit() cannot happen before initialization — which brings back CORE-014.

### 14. New tests are good but not enough

Tests now: `test_zero_copy_pubsub_guard_api`, `test_double_reserve_prevention`, `test_abandoned_reservation`, `test_read_guard_lifetime`, `test_consume_without_receive_not_possible`, `test_generic_t_destruction`, `test_ring_buffer_full`, `test_spsc_ordering`, `test_wraparound` — meaningful improvement, but still lacks dangerous cases: as_mut_uninitialized_ub, panic_during_write/read, double_drop, multiple_publishers/subscribers, T=String/Vec/Box, capacity=1, large wraparound, u64 index wraparound simulation, concurrent reservation stress, concurrent guard stress.

### 15. Capacity = 1 deserves special attention

Implementation requires power-of-two capacity and allows capacity=1 → `idx = write & 0` always same physical slot — excellent adversarial test for lifecycle.

### 16. Workspace evolution healthier

From 8 crates to 10 crates with `nros-macros` + `nros` facade:

```
nros
       │
  ┌────┴────┐
  │         │
nros-macros nros-core
  │         │
  └────┬────┘
       │
  higher-level crates
```

Much more coherent for intended Rust developer experience, but remains unverified until CI/build evidence.

### 17. CI situation: discrepancy

Latest main has commit `dafa7220 Add CI workflow for NROS project` with `.github/workflows/Ci.yml` (Build Gate, Benchmarks, Safety Gate, nros-init-compile), but direct lookup of `.github/workflows/Ci.yml` at `arena/01a0188d-nros` returns Not Found → 🔴 CI PASS not established for requested branch.

### 18. And new CI itself contains false nros init test

Workflow says nros init must produce buildable project, but actual test creates `/tmp/test_robot` with hand-written `fn main() { println!("Hello NROS") }` and runs `cargo check` — proves Rust can compile trivial binary, not that `nros init` generated project with dependencies/macros compiles.

🔴 CI-001 — P0 — nros-init gate doesn't actually run nros init

### 19. Miri safety gate also suppresses failures

Workflow uses `cargo miri test ... || echo "Miri check attempted"` — Miri PASS → continues, Miri FAIL → also continues, so supposed Safety Gate is not actually a gate.

🔴 CI-002 — P0 — Miri safety gate currently permits failure.

### 20. Workflow names safety test that may not exist on branch

Workflow expects `test_double_reserve_prevention`, `test_read_guard_lifetime`, `test_generic_t_destruction`, `test_abandoned_reservation` — those tests do exist in current nros-core source, so coherent, but again branch association and actual execution unverified.

### 21. Current evidence matrix

| Area | Assessment |
|------|------------|
| Workspace structure | 🟢 Improved |
| nros facade exists | 🟢 Present |
| nros-macros exists | 🟢 Present |
| SPSC reservation uniqueness | 🟢 Implemented |
| ReadGuard lifetime model | 🟢 Implemented |
| Generic destruction | 🟢 Implemented |
| Safe write initialization | 🔴 Broken by as_mut() |
| Actual latency benchmark | 🔴 Not measured |
| Throughput measurement | 🟡 Preliminary |
| Backpressure policies | 🟠 Enum/scaffold |
| Unsafe trait proof | 🟠 Incomplete |
| Adversarial concurrency tests | 🟠 Incomplete |
| Branch CI workflow | 🔴 Not established |
| Miri gate | 🔴 Non-gating in latest CI commit |
| nros init CI test | 🔴 Doesn't actually run nros init |
| Production readiness | 🔴 No |

### 22. Revised P0 register

Resolved / substantially resolved:
- CORE-001 Double reservation → 🟢 fixed
- CORE-002 Naked read lifetime → 🟢 fixed
- CORE-003 Generic destruction → 🟢 fixed
- CORE-005 Separate consume API → 🟢 fixed
- CORE-006 Abandonment → 🟢 substantially fixed
- CORE-008 Benchmark in correctness → 🟢 fixed

Still open:
- CORE-004 Send/Sync proof → 🔴/🟠
- CORE-009 Backpressure → 🟠
- CORE-011 as_mut() uninitialized → 🔴 P0
- CORE-012 Fake latency measurement → 🔴 P0
- CI-001 Fake nros-init test → 🔴 P0
- CI-002 Miri failure suppressed → 🔴 P0

### 23. Most important immediate code change

Remove API `pub fn as_mut(&mut self) -> &mut T` from safe interface, replace usage with genuinely valid initialization path:

```rust
guard.write_value(Twist { timestamp: ..., linear: Vector3 { ... }, angular: Vector3 { ... } });
```

If field-level zero-copy construction is hard requirement, design explicit initialization API around `MaybeUninit<T>`.

### 24. Second immediate change

Replace `record_receive(1000)` with actual timestamp pipeline:

```
producer: t0 = Instant::now(); publish message carrying t0
consumer: t1 = Instant::now(); latency = t1 - t0
```

Separate application payload latency, transport latency, queue latency, scheduler latency, end-to-end latency. Don't claim production latency number from single runner.

### 25. Third immediate change

Make safety gate actually gate:

```yaml
- name: Check nros-core with Miri
  run: cargo miri test -p nros-core --lib -- --nocapture
```

No `|| echo`, and make golden test actually do `cargo run -p nros-cli -- init ...` then `generated project → cargo check → cargo test` must be actual CI artifact.

### 26. Final verification verdict

**NROS arena/01a0188d-nros**

- Architecture: 🟢 Strong prototype
- Core redesign: 🟢 Meaningful progress
- Memory-safety model: 🟠 Improved but not yet sound
- Zero-copy API: 🔴 Still has P0 unsafe initialization flaw
- Performance evidence: 🔴 Not yet credible
- Backpressure: 🟠 Scaffolded
- CI: 🔴 Not verified on this branch
- Safety gate: 🔴 Latest proposed gate is incorrectly non-gating
- CLI/generated project: 🟠 Architecture now present, actual golden build still unproven
- Production readiness: 🔴 No

Overall:

> NROS has crossed an important threshold: the maintainers responded to the earlier core safety findings with a real guard-based redesign. However, the redesign itself introduced/retained one critical unsafe API (WriteGuard::as_mut()), while the new CI contains two verification weaknesses that could produce false confidence.

So I would not reject the architecture. I would freeze feature expansion and make next milestone:

```
NROS CORE SAFETY GATE
        │
        ├── eliminate &mut T on uninitialized memory
        ├── prove Send/Sync contract
        ├── exhaustive Drop tests
        ├── concurrency stress
        ├── Miri = hard failure
        ├── actual nros init golden test
        └── real latency measurement
                 │
                 ▼
             CI PASS
                 │
                 ▼
        Transport / HAL / Node
```

---

## Pass 8 — Deep Verification Re-read of Revised nros-core Safety Gate

I re-read current exact branch source at `arena/01a0188d-nros`, including complete `nros-core` implementation and tests.

### 1. Biggest remaining defect unambiguous

Branch says “Sound Zero-Copy SPSC Ring Buffer” and claims Safety Gate v0.1, but still exposes:

```rust
pub fn as_mut(&mut self) -> &mut T {
    unsafe { &mut *(*self.ptr).as_mut_ptr() }
}
```

while backing storage is `*mut MaybeUninit<T>`.

Before `write_value()` or equivalent initialization, memory does not contain valid `T`, yet `as_mut()` constructs Rust reference to that memory. Comment says “caller must fully initialize T before commit” but Rust safety invariant cannot be delegated to caller merely through comment when method itself is safe.

**🔴 P0 — CORE-011 remains open.** Repository should not call primitive “sound” until API removed or made genuinely safe.

### 2. Existing test actually demonstrates problem

Test uses:

```rust
let mut guard = publisher.allocate().unwrap();
guard.as_mut().linear.x = 1.0;
guard.as_mut().angular.z = 0.5;
guard.commit();
```

So current Safety Gate test suite says: unsafe initialization primitive → used by passing test → therefore safety gate passes. Not sufficient safety argument.

### 3. as_mut_uninit() is correct primitive already present

Implementation already contains `pub fn as_mut_uninit(&mut self) -> &mut MaybeUninit<T>` — correct foundation.

Cleanest remediation: Remove `as_mut()`, Keep `as_mut_uninit()`, Prefer `write_value(T)` for normal safe publishing.

### 4. But second subtler problem with as_mut_uninit()

Test does `unsafe { g.as_mut_uninit().write(42); }` — calling `MaybeUninit::write()` itself is safe, so `unsafe` block unnecessary, suggests safety API hasn't been cleaned up completely. Should be `g.as_mut_uninit().write(42);`

### 5. WriteGuard::write_value() is strongest current path

```rust
pub fn write_value(&mut self, value: T) {
    unsafe { (*self.ptr).write(value); }
}
```

Conceptually correct because `MaybeUninit::write` initializes storage. However second `write_value()` would overwrite already initialized `T` for `T=String, Vec, Box` could leak first value because overwritten without drop.

If guard represents one uninitialized slot, then `write_value()` should probably consume initialization capability:

```rust
let guard = publisher.reserve()?;
let guard = guard.write(value);
guard.commit();
```

Now compiler can enforce reserve → write → commit and prevent reserve → commit unless explicit abort.

### 6. Recommended type-state design

Instead of one guard `WriteGuard<T>`, consider:

```
WriteGuard<T>
    │ write(...)
    ▼
InitializedWriteGuard<T>
    │ commit()
    ▼
Published
```

Conceptually: `let guard = publisher.reserve()?; let guard = guard.write(T { ... }); guard.commit();` — prevents `reserve → commit` path that publishes uninitialized memory.

### 7. commit() itself has API-state weakness

Current:

```rust
pub fn commit(mut self) {
    self.committed = true;
    self.ring.write_idx.store(...);
    self.ring.write_reserved.store(false, ...);
    std::mem::forget(self);
}
```

Allows `reserve() → commit()` without proving `T` was initialized → uninitialized memory → consumer `&T` → **more serious than `as_mut()` issue**, arguably P0.

**🔴 CORE-014 — commit() does not require initialization**

Sound zero-copy API must make impossible to publish uninitialized slot.

### 8. abort() misleading

Code `pub fn abort(self)` returns no state and doesn't distinguish never initialized vs initialized then intentionally abandoned. If guard supports arbitrary `MaybeUninit` writes, API needs to know whether valid `T` currently exists. Again type-state cleaner.

### 9. ReadGuard design substantially better

Consumer receives `ReadGuard<'_, T>` and `Deref` exposes initialized `T`, on drop: drop T, clear reservation, advance read_idx — fixes earlier dangerous pattern `&T + separate consume()`. Verdict: 🟢 CORE-002 remains substantially fixed.

### 10. But DerefMut is unnecessary and expands safety surface

`impl DerefMut for ReadGuard` means consumers can mutate messages after published. For message queue normal semantic: Producer owns mutable construction → publish → Consumer receives immutable message. Prefer `Deref<Target=T>` only, unless explicit mutable-processing API with documented semantics. 🟠 CORE-015 — unnecessary mutable consumer access

### 11. RingBuffer::Drop implementation has ownership assumption

Destructor walks `[read_idx, write_idx)` and calls `drop_in_place`, correct if every slot in interval initialized exactly once, but no explicit per-slot initialization bitmap. Proof relies on `write_idx = number of initialized published elements`, valid only if `commit()` cannot happen before initialization — back to CORE-014.

### 12. Therefore actual safety invariant incomplete

Source claims `Initialized T dropped exactly once` but required invariant is `write_idx only advances for fully initialized T` — not currently enforced. Complete proof needs: reserve → uninitialized → initialize exactly once → commit → published → read guard → drop T → slot reusable. Current API permits shortcuts: `reserve → commit` and `reserve → write → write → commit`.

### 13. Performance measurement remains demonstrably invalid for latency

Benchmark contains `stats_clone.record_receive(1000); // dummy 1us` — printed Min/Avg/Max latencies based on synthetic value, not measured.

**🔴 CORE-012 remains open.**

### 14. Benchmark should not print latency at all until measured

Safest temporary: `Throughput: measured, Latency: NOT MEASURED` rather than attractive but misleading artifact.

### 15. Backpressure still not implemented

Declares `pub enum BackpressurePolicy { Block, DropOldest, DropNewest, ReturnNone }` but comment "Future: Publisher with policy would use this enum" — therefore API type 🟢, policy engine ❌, runtime use ❌, tests ❌ — 🟠 CORE-009 — scaffolded, not implemented.

### 16. Only one outstanding producer/consumer guard supported globally

Uses `write_reserved: AtomicBool, read_reserved: AtomicBool` — prevents aliasing good, but also means one Publisher → one outstanding WriteGuard and one Subscriber → one outstanding ReadGuard across entire ring. Consistent with strict SPSC but public types don't enforce which thread is producer and which is consumer. Because `Publisher` and `Subscriber` both hold `Arc<RingBuffer<T>>`, system can create multiple publishers/subscribers referencing same ring. Reservation flags prevent simultaneous access but don't enforce role ownership. Implementation is really “SPSC under dynamic discipline” rather than “SPSC enforced by type system.” Should document.

### 17. Arc architecture weakens SPSC guarantee

Intended topology: `Publisher → RingBuffer ← Subscriber`, but API exposes `pub fn ring(&self) -> Arc<RingBuffer<T>>` — users can clone Arc and construct additional endpoints. Reservation flags prevent simultaneous access but semantics aren't truly SPSC-enforced. Stronger design: `let (publisher, subscriber) = channel();` and keep ring private, then ProducerHandle, ConsumerHandle are only capabilities.

### 18. Revised architecture recommendation

Replace `Arc<RingBuffer<T>>` as public sharing primitive with:

```
SpscChannel<T>
    ├── Producer<T>
    └── Consumer<T>
Internally:
SpscChannel
    ├── RingBuffer
    ├── producer capability
    └── consumer capability
```

Ring itself preferably private. Turns architectural invariant from “Users promise to obey SPSC” into “API only gives users SPSC capabilities” — substantial improvement.

### 19. Current safety status

| Component | Status |
|-----------|--------|
| Unique producer reservation | 🟢 |
| Unique consumer guard | 🟢 |
| Read lifetime | 🟢 |
| Generic destruction | 🟢 |
| Abandoned reservation | 🟢/🟡 |
| `as_mut()` initialization | 🔴 |
| `commit()` initialization proof | 🔴 |
| Multiple initialization | 🔴/🟠 |
| Consumer mutation | 🟠 |
| SPSC role enforcement | 🟠 |
| Backpressure | 🟠 |
| Actual latency measurement | 🔴 |
| Throughput measurement | 🟡 |
| Send/Sync proof | 🟠 |
| Miri verification | ❓ |
| Branch CI verification | ❓ |

### 20. Updated P0 list

🔴 P0:
- CORE-011 WriteGuard::as_mut() exposes &mut T over uninitialized storage
- CORE-014 commit() does not prove/require that valid T has been initialized
- CORE-012 latency statistics synthetic (1000 ns) rather than measured
- CI-001 proposed nros-init CI test doesn't actually execute nros init
- CI-002 proposed Miri safety check suppresses failures

### 21. What is genuinely good in this revision

Maintainers clearly reacted to earlier findings: old API raw reserve/raw consume/raw T reference/manual destruction evolved into new API reservation guard/read guard/RAII destruction/reservation state/monotonic-clock type separated correctness/performance tests — meaningful engineering progress. Problem is project declared safety gate one step too early. It implemented shape of sound design without completing final proof boundary.

### 22. Recommended exact milestone — NROS-Core Safety Gate v0.1.1

**Gate A — Initialization:**
- [ ] remove `WriteGuard::as_mut()`
- [ ] `commit()` impossible before initialization
- [ ] double initialization impossible
- [ ] abort before initialization safe
- [ ] initialized-but-aborted semantics defined

**Gate B — Ownership:**
- [ ] producer role represented by capability
- [ ] consumer role represented by capability
- [ ] no public raw `RingBuffer` sharing
- [ ] `DerefMut` on `ReadGuard` removed unless justified

**Gate C — Destruction:**
- [ ] `DropCounter`, `String`, `Vec<u8>`, `Box<T>`, pending values, consumed values, aborted values

**Gate D — Concurrency:**
- [ ] SPSC stress, capacity 1, wraparound, full/empty races, guard contention, thread handoff, Miri, Loom

**Gate E — Performance:**
- [ ] actual per-message timestamps, monotonic clock, throughput, p50/p95/p99/max, CPU affinity documented, benchmark environment recorded

**Gate F — CI:**
- fmt → check → test → clippy → Miri → golden nros-init → benchmark artifact
- Only after all green should repository label `nros-core`: **Safety-gated zero-copy SPSC primitive**

## Final assessment after Pass 8

Branch considerably better than earlier snapshot. Guard-based redesign real and several previous defects genuinely addressed. But current safety claim still too strong. Decisive remaining problem:

> NROS has a reservation/lifetime model now, but it does not yet have an initialization-state model.

For manually allocated `MaybeUninit<T>`, that missing state is fundamental. Next correct move is not another broad repository feature audit, but finish `WriteGuard` state machine and then verify under Miri/concurrency stress before trusting `nros-core` as foundation.

---

## Pass 9 — Facade, Procedural Macros, CLI/Generator, and Buildable Project Claim

Focus: facade, procedural macros, CLI/generator, and buildable project claim.

### 1. nros facade now exists — good

New facade crate is real. It re-exports `nros-core`, `nros-node`, `nros-hal`, `nros-transport`, `nros-distributed`, `nros-sim`, `nros-studio`, `nros-cli`, `nros-macros` and provides prelude. So earlier concern: generated code `use nros::prelude::*` → crate doesn't exist is genuinely addressed. Status: 🟢 Facade crate exists. But that does not mean advertised API is implemented.

### 2. Procedural macros are almost entirely no-op

`nros-macros` defines expected macro names `#[nros::node]`, `#[nros::subscribe]`, `#[nros::publish]`, `#[nros::param]`, `#[nros::service]`, `#[nros::callback]`, `#[nros::time_sync]`, `#[nros::compute]`, `#[nros::interrupt]`, `#[nros::distributed_node]`, `#[nros::shared_state]`, `#[nros::task]`, `#[nros::sim]`, `#[nros::plugin]`, `#[nros::algorithm]`, `#[nros::telemetry]` but most simply return input unchanged. E.g., `pub fn callback(attr: TokenStream, item: TokenStream) -> TokenStream { let _ = attr; item }`. Means `#[nros::callback(realtime = true, deadline_us = 1000)]` does not create callback registration, merely disappears.

### 3. #[nros::node] is slightly different

`node` at least parses input as `ItemStruct` and returns unchanged, provides syntax validation for struct, but does not generate lifecycle implementation, parameter wiring, publisher creation, subscriber creation, scheduler registration, callback registration, QoS, topic metadata. So semantics: `#[nros::node] struct Foo { ... }` ≈ `struct Foo { ... }`. Verdict: 🟠 NROS-MACRO-001 — scaffolded API, not implementation. Not defect if explicitly labeled scaffolded, but defect if documentation presents as operational.

### 4. More serious: #[nros::subscribe] accepts arbitrary token streams

Unlike `node`, field-oriented macros don't parse item at all: `pub fn subscribe(attr: TokenStream, item: TokenStream) -> TokenStream { let _ = attr; item }`. Therefore malformed attributes can silently pass through, e.g., `#[nros::subscribe(this_is_not_valid_configuration)]` doesn't receive NROS validation. This creates distinction: compile-time syntax ≠ NROS semantic validation.

### 5. Conflicts with advertised design

Comments promise `#[subscribe(topic = "/cmd_vel", qos = Reliable)]` and say real implementation would perform subscription registration, validation. Currently none happens. Therefore evidence classification: Macro surface → IMPLEMENTED, Macro semantics → SCAFFOLDED, Compile-time wiring → ABSENT, Runtime registration → ABSENT.

### 6. nros init has more serious problem than earlier CI issue

Generator creates `src/nodes/main.rs` but also creates `src/lib.rs`, `Cargo.toml` has no `[[bin]]` section. Cargo's conventional executable entry point is `src/main.rs` not `src/nodes/main.rs`, so generated project is effectively library target but not automatically executable target for `src/nodes/main.rs`. Major issue.

### 7. Therefore current generated project builds claim too weak

Even if `cargo check` passes, it can simply be checking `src/lib.rs` while completely ignoring `src/nodes/main.rs` — classic false-positive build gate.

Required fix: Either generate `src/main.rs` or add `[[bin]] name="main" path="src/nodes/main.rs"` — preferably latter if directory structure intentional.

### 8. Invalidates current NROS-011 claim

Source explicitly says P0 fix for NROS-011: generated app must be buildable, but generated executable target not clearly wired into Cargo. Therefore 🔴 NROS-011 — NOT VERIFIED.

### 9. Generated project also deliberately doesn't depend on NROS

`generate_toml()` emits `[dependencies] # NROS core crates — uncomment if building inside NROS workspace # nros-core = { path = "../NROS/crates/nros-core" }` — so `nros init` → generated project → NO NROS dependency — standalone Rust demo, not actual NROS application.

### 10. This means current golden test proves wrong thing

Legitimate NROS generator test should establish: `nros init my_robot → Cargo.toml → nros dependency → nros macros → generated node → cargo check → cargo test`. Current establishes: `nros init my_robot → generic Rust project → cargo check`. Fundamentally different claims. Verdict: 🔴 NROS-CLI-001 — generated project is not actually NROS-integrated.

### 11. Comments acknowledge this indirectly

Generated mobile_base source says “compiles without external NROS macros” and “SCAFFOLDED, not yet full RT” — honest, but means `nros init my_robot --template=mobile_base` currently creates NROS-themed Rust project, not actual NROS application. Should be SCAFFOLDED rather than IMPLEMENTED.

### 12. CLI binary advertises substantially more commands than it implements

Help text lists `init, build, run, topic, record, replay, analyze, profile, fleet, migrate, check` but `main()` dispatches only `init, build, run, topic, help` — everything else falls into `_ => { println!("Unknown command..."); print_help(); }`. So `nros record`, `fleet deploy`, `migrate`, `check` don't execute.

### 13. This is significant evidence-taxonomy problem

Library defines enum variants for `Record, Replay, Analyze, Profile, Fleet, Migrate, Check` but enum variant ≠ implementation. Need to distinguish API model (Command enum exists) vs CLI execution (command parsed → operation performed → result validated).

### 14. CLI status matrix

| Command | CLI syntax | Command enum | Actual dispatch | Real operation |
|---------|------------|--------------|-----------------|----------------|
| init | 🟢 | 🟢 | 🟢 | 🟡 |
| build | 🟢 | 🟢 | 🟢 | 🟡 |
| run | 🟢 | 🟢 | 🟢 | 🟡 |
| topic | 🟢 | 🟢 | 🟢 | 🟠 |
| record | 🟢 help | 🟢 | 🔴 | 🔴 |
| replay | 🟢 help | 🟢 | 🔴 | 🔴 |
| analyze | 🟢 help | 🟢 | 🔴 | 🔴 |
| profile | 🟢 help | 🟢 | 🔴 | 🔴 |
| fleet | 🟢 help | 🟢 | 🔴 | 🔴 |
| migrate | 🟢 help | 🟢 | 🔴 | 🔴 |
| check | 🟢 help | 🟢 | 🔴 | 🔴 |

### 15. nros build also needs semantic verification

Library contains `BuildSystem`, but binary calls `CLI::run(Command::Build { ... })`. Build implementation should be inspected for whether it actually invokes Cargo or merely simulates build. Presence of `BuildOutput { profile, binary_size_kb, elapsed, features }` does not establish binary built.

### 16. Facade's init() and spin() are explicitly placeholders

Facade contains `pub fn init() { println!("[NROS ...] Initialized"); }` and `pub fn spin<T>(_node: T) { println!("[NROS] Spinning node (placeholder)"); }` — fundamental runtime lifecycle init → scheduler → node registration → event loop → callbacks does not exist yet in facade. Not inherently bad for prototype, but should not describe as operational runtime.

### 17. Node layer also reveals prototype-level time semantics

`nros-node` defines its own `Timestamp` using `SystemTime::now()` while `nros-core` has introduced `MonotonicTimestamp` based on `Instant` — creates two different time abstractions, dangerous for robotics runtime claiming deterministic timing. Node layer should consume canonical NROS clock abstraction rather than independently defining its own wall-clock timestamp.

### 18. Cross-crate architecture inconsistency

We now have `nros-core: MonotonicTimestamp` (Instant), `nros-node: Timestamp` (SystemTime), `nros facade: re-exports both concepts` — developer can accidentally mix wall-clock timestamps with monotonic durations without obvious type-level distinction. Recommended design: `nros-time` crate with `MonotonicInstant, SystemTimestamp, Duration, Deadline, Clock` — then all real-time APIs explicitly consume appropriate type.

### 19. ExecutionStats isn't a real-time guarantee

Node layer has telemetry `callback_count, total_execution_time_ns, max_execution_time_ns, min_execution_time_ns, deadline_misses` but these are measurement counters, not hard deadline guarantee. Callback takes 5ms, deadline 1ms, then `deadline_misses +=1` but nothing prevents violation. Documentation should say deadline monitoring rather than real-time deadline guarantee unless scheduler actually enforces deadline.

### 20. Repository's terminology needs normalization

Current code mixes `IMPLEMENTED, SCAFFOLDED, placeholder, real implementation would..., future, P0 fix` across same modules, creating traceability problem. Recommend strict evidence taxonomy: SPECIFIED → SCAFFOLDED → COMPILES → UNIT-TESTED → INTEGRATION-TESTED → CI-VERIFIED → BENCHMARK-VERIFIED → PRODUCTION-CAPABLE. No feature should jump from enum exists to IMPLEMENTED.

### 21. Revised repository maturity map

```
NROS
                     │
       ┌─────────────┼──────────────┐
       │             │              │
      Core          API            CLI
       │             │              │
    🟠/🔴          🟠             🟠/🔴
       │             │              │
   unsafe init    macros        command surface
   state issue    passthrough   > implementation
       │
       ▼
    HAL / Transport / Distributed / Sim / Studio
                     │
                     ▼
                 mostly
               prototype/
               scaffold
```

Project has large architectural skeleton, but trusted implementation core still relatively small.

### 22. Updated P0/P1 register

🔴 P0:
- CORE-011 WriteGuard::as_mut() exposes &mut T over uninitialized storage
- CORE-014 commit() does not require initialized T
- CORE-012 latency benchmark records synthetic 1000 ns
- CI-001 nros-init gate doesn't actually test generated NROS project
- CI-002 Miri failure is suppressed
- NROS-011 generated project is not yet proven to compile its node executable
- CLI-001 generated project has no actual NROS dependency
- CLI-002 advertised CLI commands aren't dispatched

🟠 P1:
- CORE-009 backpressure policy is scaffolded
- CORE-015 ReadGuard unnecessarily permits mutation
- CORE-016 SPSC endpoint roles aren't type-enforced
- TIME-001 duplicate wall-clock/monotonic time abstractions
- RT-001 deadline monitoring ≠ deadline enforcement
- MACRO-001 macros are no-op passthroughs
- TRACE-001 implementation taxonomy is too optimistic

### 23. What I would do next

**Phase 1 — Make nros-core actually sound:**
- WriteGuard: reserve() → write(T) → abort() → commit() with compile-time/state guarantees preventing commit before initialization, double initialization, overwrite initialized T
- Then run cargo test, clippy, miri, and preferably Loom/state-model testing

**Phase 2 — Fix the generator:**
- Generated project should be `my_robot/ Cargo.toml src/main.rs src/lib.rs src/nodes/ config/ launch/ tests/` and `[dependencies] nros = ...` must point to actual NROS facade, generated program should actually contain `use nros::prelude::*; #[nros::node] struct MyRobot { ... } fn main() { nros::init(); nros::spin(...); }` Even if runtime remains scaffolded, integration must be real.

**Phase 3 — Make CLI honest:**
- Either implement record/replay/analyze/profile/fleet/migrate/check or remove them from executable help until executable.

**Phase 4 — Implement macro semantics incrementally:**
- Start `#[nros::node]`, `#[nros::publish]`, `#[nros::subscribe]`, then `#[nros::param]`, `#[nros::callback]`, then `#[nros::service]`, `#[nros::task]`, and only later `distributed_node, shared_state, compute, interrupt, sim, plugin` — each macro needs parser, validation, generated code, compile-fail tests, runtime integration test.

### 24. Key conclusion from this pass

There are now two different NROS projects inside this repository:

**A. Architectural specification:** Very broad — robotics OS, real-time runtime, IPC, HAL, distributed execution, simulation, studio, fleet, migration, GPU/NPU, plugins

**B. Verified implementation:** Much smaller — Rust workspace, core SPSC prototype, example node implementation, CLI scaffolding, facade crate, passthrough procedural macros, prototype supporting crates

Branch is useful and structurally promising, but evidence must not conflate A with B.

### Final status after Pass 9

> NROS arena/01a0188d-nros is a substantial architectural prototype, not yet a verified robotics runtime.

Most urgent blocker remains nros-core initialization state machine, but next layer exposed equally important integration problem: `nros init` currently generates compilable-looking standalone Rust project rather than genuinely NROS-integrated executable, and CLI has same pattern: command model is much more complete than executable behavior.

So next gate should be:

```
NROS Verification
                    │
        ┌───────────┴───────────┐
        │                       │
   Core Safety             Integration
        │                       │
   MaybeUninit              nros init
   type-state               real dependency
   Drop proof               real binary
   Miri                     real macros
   concurrency              real CLI dispatch
        │                       │
        └───────────┬───────────┘
                    ▼
             CI HARD GATES
                    │
                    ▼
             Evidence-backed implementation
```

---

## Pass 10 — CLI/runtime behavior, false evidence, and test-quality audit

### 1. CLI library is explicitly simulation layer

Source repeatedly uses language such as Simulate build steps, simulated info, simulated..., real would... For example, `BuildSystem::build()` performs parsing message definitions, generating bindings, static graph validation, compiling nodes, linking executables, WCET analysis but implements those steps as `std::thread::sleep(...)`. So `nros build` currently means approximately: print planned build stages → sleep → construct synthetic BuildOutput → print synthetic result. It does not mean cargo build → compiler → linker → actual executable → measured artifact.

### 2. This is actually documented—but API taxonomy is wrong

Implementation itself admits “SIMULATED — would measure target binary…” and “real would: cargo build…”. That's good honesty at source level. Problem is public command still `nros build --profile=realtime` and generated README tells users `nros build --profile=realtime` with claims such as `-O3`, `LTO`, `static linking` without actually performing those operations. Therefore correct status: CLI Build API = SCAFFOLDED/SIMULATED, not IMPLEMENTED.

### 3. BuildOutput is synthetic

For example Debug → 2300 KB, Release → 1120 KB, Realtime → 950 KB, Embedded → 480 KB — hard-coded. So if CLI prints `Binary size: 950 KB`, that does not mean actual 950 KB executable exists. This is exactly kind of evidence that must not enter performance/compliance report as measured data. Finding 🔴 CLI-BUILD-001 — synthetic artifact metrics

### 4. Real-time guarantees feature list problematic

`BuildProfile::Realtime` returns `-O3, LTO, CPU native, real-time guarantees, static pools` but no real-time scheduler, allocator policy, CPU affinity, deadline enforcement performed by function. Thus feature label ≠ implemented capability. String “real-time guarantees” should be removed until actual mechanism and verification evidence. Finding 🔴 RT-BUILD-001 — “real-time guarantees” is currently declarative label, not verified property.

### 5. Topic inspection is also synthetic

`TopicInspector::list()` returns hard-coded topics `/cmd_vel, /odom, /camera/image, /scan` with hard-coded rate, bandwidth, latency, publishers, subscribers — no discovery mechanism. So `nros topic list` does not inspect NROS runtime, prints demo topology.

### 6. Topic info even more revealing

If topic isn't found, code prints `Topic X not found, but showing simulated info` and then emits fabricated `geometry_msgs/Twist, 10.2 Hz, 5.2 μs` — unacceptable for diagnostic CLI. Diagnostic command should return `ERROR: topic not found` not fabricated telemetry. Finding 🔴 CLI-TOPIC-001 — diagnostic command can report fabricated state

### 7. Topic echo doesn't consume topic

Implementation sleep 500ms print fixed Twist for each iteration. No transport subscription, queue receive, deserialization, timestamp, actual message. So `nros topic echo /cmd_vel` is currently demonstration loop.

### 8. Topic hz also synthetic

Says collecting data for 2 seconds (simulated)... then sleeps one second and returns `10.23 Hz` with fixed min/max/stddev/count values. Therefore cannot be used to validate frequency, jitter, scheduler, transport rate. Finding 🟠 CLI-TOPIC-002 — frequency measurement scaffold

### 9. Topic bw same issue

Claims average 1.23 KB/s etc without measuring transport stream.

### 10. Profiling entirely fabricated

`Profiler::profile()` constructs hard-coded data `VelocityController::on_cmd_vel, ImageProcessor::process_frame (GPU), PathPlanner::compute_path, Other` and fixed execution times 245.3ms, 189.7ms, 78.2ms, 29.1ms, then claims `Flamegraph saved to: profile_output.svg` but no evidence of actually generating flamegraph. Means `nros profile` currently prints fictional profiler report. 🔴 CLI-PROFILE-001 — SIMULATION ONLY.

### 11. Fleet management simulated too

`FleetManager::list()` creates four hard-coded robots `robot_001..004` with fixed health/CPU/memory. `deploy()` sleeps robot_001 1s, robot_002 1.5s etc and prints updated successfully — no TLS, authentication, OTA transfer, artifact verification, device communication, rollback, health endpoint.

### 12. Especially dangerous for fleet CLI

Command named `nros fleet deploy` implies potentially destructive operation. Simulator should not look identical to operational deployment tool. Recommended behavior until implemented: `nros fleet deploy` should return `ERROR: fleet deployment backend is not implemented in this build. Use --dry-run to preview` rather than `Deployment complete!` Finding 🔴 FLEET-001 — simulated deployment reports success — trust/UI safety issue.

### 13. Fleet exec especially misleading

Code sleep 300ms print `Command executed successfully` — no remote command executed. Should say `SIMULATION: command would be executed...`

### 14. Recording is not recording

`Recorder::record()` says `Recording... 10 messages captured (simulated) Saved to ...` but no actual file serialization shown. Creates potentially worse failure mode: `nros record` → prints Saved → user expects artifact → artifact may not exist. Finding 🔴 REC-001 — record command does not establish real recording artifact

### 15. Replay doesn't consume recording

`replay()` only prints `Replaying ...` and sleeps 300ms. No open file, validate format, read messages, schedule timestamps, publish messages. So record/replay are UI simulation rather than data pipeline.

### 16. Analysis entirely declarative

`Recorder::analyze()` prints fixed values such as `P50=5.8 μs, P99=12.1 μs, Max=18.7 μs` — no analysis of input file. Means `nros analyze random_file` can report plausible-looking robotics metrics without reading data — serious evidence-integrity issue.

### 17. Migration tools don't actually convert

`MigrationTools::analyze_ros2()` prints `12 nodes, 23 topics, 5 services, 4 custom messages, 2 weeks` without inspecting supplied path. `convert()` likewise prints conversion description but doesn't demonstrate parsing or writing requested input/output. Therefore `nros migrate analyze/convert` are currently documentation demos.

### 18. Migrate convert semantic mismatch

Output claims `create_publisher<Msg>(topic, qos) → publish<T>(topic) + publish().await` — merely text, no AST transformation, source rewriting, message conversion, validation. So command name implies conversion while implementation is conversion description. Finding 🟠 MIGRATE-001 — migration engine scaffold

### 19. nros check is currently print operation

Runner `if timing: println!("WCET analysis...")`, `if graph: println!("Validate communication graph...")` — No graph loaded, no YAML parsed, no cycle detection, no timing analysis. So `nros check --timing --graph` currently means print two messages. Finding 🔴 CHECK-001 — advertised static-analysis gate is not implemented

### 20. Tests themselves expose problem

CLI test `test_build_system()` verifies only hard-coded 950 >0 and profile enum survived, not that Cargo build succeeded, binary exists, binary is 950KB, realtime flags used, LTO enabled. So unit test of simulation, not build-system test.

### 21. Important evidence classification

Divide NROS CLI features:

🟢 Real implementation: command enums, basic project file generation, filesystem directory creation, basic project-name validation, some argument/model structures

🟡 Partial: project initialization, command dispatch, facade integration, build abstraction

🔴 Simulation-only: build execution, binary-size measurement, topic discovery, topic echo, Hz measurement, bandwidth measurement, profiler, fleet state, fleet deployment, fleet exec, record/replay, data analysis, migration analysis/conversion, timing analysis, graph validation

### 22. Major architectural opportunity: separate nros-cli from nros-demo

Strongly recommend splitting `crates/nros-cli` into `nros-cli`, `nros-cli-core`, `nros-demo` or feature `nros-cli --features demo` because currently production-looking commands backed by simulation code. Clean architecture: `nros-cli` → parser, command model, diagnostics, real backends (build, graph, transport, recording, fleet), `nros-demo` → simulated implementations. Then nobody can accidentally mistake simulated results for real.

### 23. Stronger evidence states

Recommend adding machine-readable capability metadata: `id="nros.build" status="scaffolded" execution="simulated"` vs `status="implemented" execution="real" verified_by="ci"`

### 24. CLI currently violates critical principle

For agent-facing CLI, principle should be: Never report external-world state transition unless transition actually happened. Current violations: fleet deploy → Deployment complete!, fleet exec → Command executed successfully, record → Saved, profile → Flamegraph saved, check → Validate communication graph, build → Build completed while no corresponding external operation.

For human demo, misleading. For AI agent, much more serious — agent may interpret output as authoritative state.

### 25. Matters especially for agentic-CLI direction

NROS is supposed to become agent-friendly runtime/platform. Every CLI operation needs deterministic result contract: Command → structured request → backend → real state transition → structured result → exit code. Example: `{ "ok": false, "status": "not_implemented", "operation": "fleet.deploy", "simulation": false }` vastly safer than `Deployment complete!` when no deployment occurred.

### 26. Recommended CLI result protocol

Introduce `CommandResult { status: success/failed/unsupported/simulated, operation, evidence, artifacts, warnings, metrics }` Then `nros build` could return `{"status":"success","operation":"build","artifact":"target/release/my_robot","measured_size_bytes":934912}` while current prototype returns `{"status":"simulated","operation":"build","reason":"cargo backend not installed"}` This would make NROS much more suitable for autonomous agents.

### 27. Updated overall architecture assessment

```
NROS
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
       CORE              API               CLI
        │                 │                 │
      🟠/🔴              🟠               🔴/🟠
        │                 │                 │
   promising SPSC    macros mostly      large synthetic
     primitive       passthrough       simulation layer
```

Project architecturally ambitious and increasingly coherent, but verified implementation boundary much smaller than feature surface.

### 28. New priority order

P0 — must fix first:
1. WriteGuard initialization state
2. commit() cannot publish uninitialized memory
3. real Miri gate
4. real nros init golden build
5. remove fabricated success messages
6. make CLI explicitly report simulated
7. real build backend

P1:
8. actual topic transport backend
9. actual recording format
10. graph parser/validator
11. migration parser
12. real profiler integration
13. endpoint capability model
14. structured JSON output

P2:
15. fleet backend
16. Studio integration
17. advanced macros
18. simulation backend
19. distributed runtime

### 29. Revised readiness scorecard

| Layer | Evidence status |
|-------|-----------------|
| Repository structure | 🟢 |
| Workspace organization | 🟢 |
| nros facade | 🟢 |
| Core SPSC design | 🟠 |
| Core memory safety | 🔴 |
| Core tests | 🟠 |
| Macros | 🟠 scaffold |
| Node runtime | 🔴/🟠 |
| CLI parser/model | 🟢 |
| CLI execution | 🔴 |
| Project generator | 🟠 |
| Generated NROS app | 🔴 |
| Build system | 🔴 simulated |
| Topic tools | 🔴 simulated |
| Profiler | 🔴 simulated |
| Recorder | 🔴 simulated |
| Migration | 🔴 simulated |
| Graph checker | 🔴 simulated |
| Fleet | 🔴 simulated |
| CI proof | 🔴 pending |
| Production readiness | 🔴 |

### 30. Bottom line

Most important conclusion:

> NROS currently has much larger command/API surface than verified execution engine.

That's not inherently bad for early prototype. Problem arises when simulated operations are presented using production-operation language. For ordinary demo, documentation issue. For agentic runtime, correctness issue. Immediate architectural rule: **No simulated backend may emit same success semantics as real backend.** Use `SIMULATED, NOT_IMPLEMENTED, MEASURED, VERIFIED` as first-class states.

---

## Pass 11 — Evidence Registry ↔ Source ↔ CI ↔ README Reconciliation

### 1. Evidence registry is actually one of strongest parts

`EVIDENCE_REGISTRY.md` explicitly defines `SPECIFIED, SCAFFOLDED, SIMULATED, IMPLEMENTED, TESTED, BENCHMARKED, INTEGRATION-TESTED, HARDWARE-VALIDATED, PRODUCTION-READY, SAFETY-QUALIFIABLE` and explicitly says it exists to separate executable fiction from verified implementation — exactly right governance model. It correctly marks several major subsystems as simulated: HAL DMA, compression, distributed election, distributed replication, Bullet backend, Studio telemetry, CLI build, topic inspection, profiler, fleet deployment — major positive.

### 2. But registry contradicts actual safety implementation

Registry says SPSC ring buffer lock-free → IMPLEMENTED → TESTED and WriteGuard single outstanding → IMPLEMENTED → TESTED and ReadGuard owns slot → IMPLEMENTED → TESTED — defensible individually, but then treats collection as sufficient for “Core IPC prototype: HIGH” and concludes “Safety Gate FIXED” — too broad, reservation/lifetime mechanisms improved but initialization invariant still incomplete.

### 3. Registry does not list commit() initialization flaw

Current core API permits `reserve() → commit()` without enforced `initialize()` transition. Therefore safety invariant should explicitly contain `PUBLISHED ⇒ INITIALIZED`. New formal requirement CORE-014: Slot must not become visible to consumer until valid T has been initialized.

### 4. README itself acknowledges uncertainty—but still overstates status

README says “TESTED after Safety Gate v0.1” for zero-copy IPC, then qualifies benchmark needs independent verification — good, but also states “Safety Gate FIXED” — too broad because passing functional test suite ≠ completed memory-safety proof. Correct wording: “Safety Gate v0.1 redesign implemented; independent safety verification pending.”

### 5. README has stale workspace structure

README says `Cargo.toml # Workspace root (8 crates)` and lists eight crates, but current branch's workspace has 10 crates: `nros-core, nros-node, nros-hal, nros-transport, nros-distributed, nros-cli, nros-sim, nros-studio, nros-macros, nros` = 10 crates — not cosmetic, repository-level audit must treat README, Cargo.toml, source tree as consistency-controlled artifacts. Finding 🟠 DOC-001 — README workspace inventory stale

### 6. README says CLI is IMPLEMENTED-TESTED

Implementation-status table says #6 CLI Tools → IMPLEMENTED-TESTED but same row says build system size SIMULATED and topic list hard-coded SIMULATED. Evidence registry is more precise: CLI architecture IMPLEMENTED, nros init IMPLEMENTED/TESTED, build SIMULATED, topic SIMULATED, profiler SIMULATED, fleet SIMULATED. So registry correct, but README's aggregate label IMPLEMENTED-TESTED too easy to misread. Replace with PARTIAL — command architecture implemented; init tested; operational backends simulated.

### 7. Repository has canonical implementation problem

Registry says `crates/` is authoritative and `implementations/` is archival artifact — reasonable migration strategy, but needs automated check that archival implementations aren't accidentally being compiled or treated as current source. Otherwise auditing `crates/nros-core` while CI/documentation silently refers to `implementations/nros-core-implementation`. Recommended gate: CI should explicitly assert `cargo metadata` and verify only intended workspace members are built.

### 8. More importantly: CI is still absent from requested branch

README and evidence registry claim `.github/workflows/ci.yml` was added, but fetching `.github/workflows/ci.yml` at `arena/01a0188d-nros` returns 404. Alternate `Ci.yml` also 404. So branch's own documentation claims CI = added while actual requested ref currently gives CI workflow file = absent — hard repository inconsistency. 🔴 CI-003 — Documentation claims workflow that is not present on audited ref — traceability failure, not merely CI hasn't run.

### 9. This changes CI verdict — CI-003

README → claims CI gate → EVIDENCE_REGISTRY → claims CI gate → requested branch → workflow absent — traceability failure.

### 10. Therefore PRODUCTION-READY is impossible

Registry defines PRODUCTION-READY: CI passes, clippy clean, Miri/loom reviewed, docs complete. Branch doesn't even contain claimed CI workflow, therefore PRODUCTION-READY FALSE — consistent with registry overall status, but conflicting claim Safety Gate FIXED.

### 11. Benchmark claim appropriately downgraded—but README still advertises too prominently

Registry correctly says 6.2 μs / 780K msg/s is repository-reported and not independently verified — excellent. But README headline still contains `NROS core: 6.2us, 780K msg/s` and performance target section says prototype measurement repository-reported. That wording better than calling it verified benchmark, but still gives strong numerical impression before qualification. For scientific/engineering documentation I'd change to Target: <10 μs / 500K msg/s, Repository-local prototype observation: ~6.2 μs / ~780K msg/s, Status: NOT INDEPENDENTLY VERIFIED, NOT A CROSS-PLATFORM BENCHMARK, NOT A SAFETY/PERFORMANCE GUARANTEE.

### 12. 46× / 15× / 79% claims need stronger containment

README says 46× latency, 15× throughput, 79% memory, 51% fewer LOC, 73–81% faster builds, 29× startup, 37% power saving, 58% battery life, 39% TCO savings. Evidence registry explicitly says ROS2 comparison isn't independently established: No ROS2 baseline in this repo. Therefore numbers should not appear in README's top-level feature summary without prominent design/target claim label. Recommended classification TARGET/HYPOTHESIS not PERFORMANCE.

### 13. Particularly important because project positioned against ROS2

Comparison claim like 46× faster requires controlled experiment: same hardware, same OS, same compiler conditions, same message, same frequency, same transport semantics, same QoS semantics, same CPU affinity, same warm-up, same measurement methodology. Repository's own evidence registry acknowledges absence of that baseline. Therefore 🔴 BENCH-001 — comparative performance claims remain unverified.

### 14. Distributed implementation correctly marked simulated

Registry explicitly says `random_bool(0.7)` instead of real RequestVote RPC, `replicate() → Ok(())` stub — excellent evidence discipline. But README architecture section still describes Distributed Raft fleet in broad architectural language. So documentation should make distinction immediately visible: Architecture: Raft planned, Implementation: simulated election, Production: no.

### 15. Same pattern appears in HAL

Registry correctly distinguishes `SimulatedDmaBuffer` vs `RealDmaBuffer` and explicitly says camera path does `buf.data.clone()` rather than actual zero-copy DMA — good. Same principle should be applied to CLI.

### 16. Same principle should be applied to nros-core

Core has `RingBuffer, WriteGuard, ReadGuard` but unsafe boundary isn't visible in public type names. I'd introduce `UninitializedWriteGuard, InitializedWriteGuard` or equivalent type-state. Then API itself communicates Uninitialized instead of relying on comments. Consistent with repository's broader evidence philosophy.

### 17. Safety document should become executable

Repository has `crates/nros-core/SAFETY.md` and README describes invariants. Next step turning each invariant into named test: `safety_reservation_unique, safety_commit_requires_init, safety_exactly_once_drop, safety_read_guard_owns_slot, safety_abort_not_visible, safety_double_init_forbidden, safety_wraparound_init_state, safety_capacity_one` Then SAFETY.md ↔ tests ↔ CI becomes mechanically traceable.

### 18. Miri should test API users actually use

Current safety test strategy should not merely test drop, reservation, read guard — must test `write_value, as_mut_uninit, commit, abort, drop` especially `String, Vec<u8>, Box<T>, nested Drop types` because those are where invalid initialization/destruction patterns become obvious.

### 19. Repository's best architectural decision so far

Strongest part isn't runtime, it's evidence model. Registry provides very good conceptual contract: feature → spec → implementation → status → test → benchmark → hardware validation → claim allowed. That's exactly how NROS should govern entire project. Problem is final verdict rows haven't caught up with source-level evidence.

### 20. Recommend making registry authoritative

Define rule: No README claim may have stronger status than its corresponding EVIDENCE_REGISTRY entry. For example: EVIDENCE_REGISTRY: SIMULATED, README: IMPLEMENTED → CI failure. Likewise: EVIDENCE_REGISTRY: NOT CLAIM ALLOWED, README: verified performance → CI failure. This could be implemented with small metadata file `[feature.nros_core] status = "TESTED" claim_allowed = true` etc. Then documentation can be generated or validated from it.

### 21. Introduce claim linter

Project badly needs documentation/evidence linter: `cargo run -p nros-audit -- claims` would check README, DESIGN, COMPARISON, AUDIT, EVIDENCE_REGISTRY and detect: README: Raft implemented vs Registry: SIMULATED, README: 6.2 μs benchmark vs Registry: repository-reported, README: 8 crates vs Cargo workspace: 10, README: CI added vs Branch: workflow absent — extremely valuable.

### 22. New formal gate: Documentation Consistency

Recommend DOC-GATE Required: README ↔ Cargo.toml, README ↔ EVIDENCE_REGISTRY, AUDIT ↔ EVIDENCE_REGISTRY, DESIGN ↔ implementation status, CI ↔ actual workflow files, benchmark claims ↔ benchmark artifacts. No release should pass if these disagree.

### 23. New formal gate: Claim Strength

Every externally visible claim should map to `claim_allowed = true/false`. For example: UDP transport exists → YES, UDP production-grade → NO, SPSC ring buffer prototype exists → YES, memory-safe zero-copy IPC → NOT YET, Raft implementation → NO, simulated leader election → YES, 6.2 μs measured → repository-reported only, 46× faster than ROS2 → NO.

### 24. Current branch verdict after Pass 11

> 🟡 Architecture-complete prototype with partially mature evidence registry, but unresolved core safety invariants, simulated operational tooling, absent CI on audited ref, and documentation/source inconsistencies.

Not Safety Gate fixed, and definitely not Production-ready.

### 25. Final updated risk register

P0 — correctness/safety:
1. CORE-011 safe as_mut() over MaybeUninit
2. CORE-014 commit() doesn't enforce initialization
3. CI-003 claimed CI workflow absent on audited branch
4. NROS-011 generated NROS-integrated project still not proven; current claim only proves plain Rust generation
5. CLI-TRUST-001 simulated commands report production-style success
6. BENCH-001 headline comparative performance claims lack independent benchmark evidence

P1 — architecture:
7. Type-state write lifecycle
8. SPSC endpoint capability enforcement
9. Real CLI backend abstraction
10. Real topic/recording/analysis backend
11. Canonical time abstraction
12. Actual macro semantics
13. Automated documentation/evidence consistency

P2 — expansion:
14. Real DMA-BUF
15. Real Raft
16. Real distributed replication
17. Real fleet backend
18. Hardware validation
19. ROS2 controlled comparison
20. Safety qualification evidence

### 26. Next verification target

Cross-crate integration: `nros-core → nros-node → application boundary` is next decisive gate. Does real NROS application actually cross crate boundaries, or are crates mostly parallel demonstrations? Can one message travel through complete stack without being copied, fabricated, or converted through duplicate types?

---

## Pass 12 — Cross-crate Integration: nros-core → nros-node → Application Boundary

### 1. Workspace topology confirmed: 10 crates

Root Cargo.toml confirms all ten workspace members — README's “8 crates” statement stale 🟠 DOC-001 confirmed.

### 2. nros-node actually depends on nros-core

Manifest contains `[dependencies] nros-core = { path = "../nros-core" }` and defines both library and binary — genuine dependency edge `nros-node → nros-core`, degree of integration weaker than architecture suggests.

### 3. nros-node duplicates core message types

Node crate defines own `Timestamp, Vector3, Twist, MotorCommand, Odometry` while nros-core also defines `Timestamp, Vector3, Twist` — architecture contains `nros-core::Twist` and `nros-node::Twist` separate models, even though comments say compatible not identical, no type-level guarantee.

### 4. Serious integration smell

`fn publish(msg: nros_core::Twist)` vs `fn on_cmd_vel(msg: &nros_node::Twist)` cannot be passed directly, need conversion `nros-core::Twist → conversion → nros-node::Twist` or duplicate representations — undesirable for zero-copy system. Want `transport → nros-core::Twist → node callback` not `transport Twist → copy/convert → node-local Twist`.

### 5. Correct architecture is to move message definitions downward

Introduce dedicated canonical crate `crates/nros-types` with `Timestamp, Vector3, Twist, Odometry, ...` then `nros-core, nros-node, nros-transport` depend on `nros-types` — eliminates circular pressure.

Better still: separate wire types from runtime types: `nros-msg` (Twist, Odometry) + `nros-time` (Timestamp, MonotonicInstant, Duration, Deadline) then core/node/transport depend on msg/time — canonical type graph.

### 7. Time duplication confirmed

`nros-core` has `MonotonicTimestamp` based on `Instant` but also retains legacy `Timestamp` based on `SystemTime`, `nros-node` independently defines another `Timestamp` also using `SystemTime` — at least three time concepts: `nros-core::MonotonicTimestamp`, `nros-core::Timestamp`, `nros-node::Timestamp` — should be consolidated.

### 8. Node claims real-time behavior while using wall-clock timestamps

Callback does `let start = Instant::now()` good for elapsed, but then constructs `MotorCommand { timestamp: Timestamp::now(), ... }` where `Timestamp::now()` uses `SystemTime` — subtle but important distinction execution measurement → monotonic, message timestamp → wall clock, can be valid but needs explicit semantic contract, otherwise consumers may incorrectly compare deadline duration with wall-clock timestamp.

### 9. nros-node is actually demonstration application embedded in library crate

Manifest calls it “NROS Node Example” and source heavily example-oriented `VelocityController, ParameterServer, LifecycleNode, ExecutionStats` — useful but should be classified as example/reference implementation rather than NROS node runtime, actual runtime machinery still missing.

### 10. Biggest contradiction: node's real-time callback isn't registered anywhere

Source comments say `#[callback(realtime=true, deadline_us=1000, priority=200)]` would be used in real macro system, but actual function simply `pub fn on_cmd_vel(&mut self, msg: &Twist)` — no callback registration, scheduler, priority, deadline admission, executor. Thus `VelocityController::on_cmd_vel()` is just ordinary Rust function.

### 11. 1 ms deadline is observational, not enforced

Function records `self.stats.record_execution(elapsed, 1_000_000)` — if execution takes 1.5ms, result `deadline_misses +=1` but nothing prevents exceeding 1ms. So deadline monitoring = YES, deadline enforcement = NO — should be reflected in evidence registry.

### 12. Parameter system isn't connected to node runtime

`ParameterServer` functioning local data structure declare/get/get_float/get_int/set/validate — useful but not integrated with parameter service, transport, persistent configuration, runtime graph, CLI, Studio — currently `parameter system → in-process HashMap` not NROS parameter subsystem.

### 13. More important findings...

(Truncated for brevity, but includes: HashMap outside real-time callback good, safety_check() performs println! → RT-002 non-deterministic console I/O, on_cmd_vel() uses SystemTime indirectly, odometry Euler simple first-order integration, MotorCommand torque placeholder, core safety test uses unsafe API as_mut() → CORE-017, as_mut() should not exist, commit() central soundness flaw CORE-014, abort() misleading, ReadGuard DerefMut unnecessary CORE-015, RingBuffer Drop ownership assumption coupled to CORE-014, performance measurement invalid, backpressure scaffolded, only one outstanding guard supported globally, Arc architecture weakens SPSC guarantee, ownership model should be explicit SpscChannel Producer/Consumer, etc.)

### Final status after Pass 12

> NROS arena/01a0188d-nros is a substantial architectural prototype, not yet a verified robotics runtime.

Most urgent blocker remains nros-core initialization state machine, but next layer exposed integration problem: `nros init` generates compilable-looking standalone Rust project rather than genuinely NROS-integrated executable, and CLI has same pattern: command model is much more complete than executable behavior.

Next gate:

```
NROS Verification
                    │
        ┌───────────┴───────────┐
        │                       │
   Core Safety             Integration
        │                       │
   MaybeUninit              nros init
   type-state               real dependency
   Drop proof               real binary
   Miri                     real macros
   concurrency              real CLI dispatch
        │                       │
        └───────────┬───────────┘
                    ▼
             CI HARD GATES
                    │
                    ▼
             Evidence-backed implementation
```

---

## Appendix: Full evidence taxonomy per EVIDENCE_REGISTRY.md

See previous file `AUDIT.md` for full evidence taxonomy and remediation.

This file extends audit with Pass 8-12 deep verification: facade, macros, CLI/generator, buildable project claim, CLI/runtime behavior, evidence registry ↔ source ↔ CI ↔ README reconciliation, cross-crate integration nros-core → nros-node → application boundary.

