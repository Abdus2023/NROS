# NROS — Deep Analysis & Verification — Pass 20 to 23 (Re-verification, Branch Integrity, CI Provenance, Invariant Audit)

Branch: `arena/01a0188d-nros`

---

## Pass 20 — Re-verification against the current branch

### Current workspace architecture

The branch now contains 12 crates:

```
nros-types
nros-core
nros-node
nros-hal
nros-transport
nros-distributed
nros-cli
nros-sim
nros-studio
nros-macros
nros
nros-audit
```

Intended dependency direction much healthier:

```
nros-types
                 /   |   \
                /    |    \
               ▼     ▼     ▼
          nros-core node  HAL
               │      │
               │      ├── transport
               │      └── sim
               │
               └──── runtime/facade
```

### nros-types is now genuinely canonical

New crate explicitly describes itself as “single source of truth for messages, time, geometry” and fixes earlier nros-core::Twist ≠ nros-node::Twist duplication. Contains WallTimestamp, MonotonicInstant, Timestamp, MonotonicTimestamp, Vector3, Twist, MotorCommand, Odometry, Point3D, PointCloud, Image, ImuData, ExecutionStats.

**Status: 🟢 Canonical message layer: implemented**

### nros-core now depends on nros-types

Core manifest explicitly has `nros-types = { path = "../nros-types" }` — correct dependency direction.

### nros-node has also migrated

Node crate now imports `pub use nros_types::{WallTimestamp, MonotonicInstant, Vector3, Twist, MotorCommand, Odometry}` and explicitly states now single source of truth nros-types. Earlier finding nros-core::Twist ≠ nros-node::Twist no longer applicable.

**Corrected verdict: 🟢 Node/core message duplication: remediated**

### Time-domain separation substantially better

nros-types explicitly distinguishes Wall time (WallTimestamp based on SystemTime for wire protocol, external timestamps, ROS time) and Monotonic time (MonotonicInstant based on Instant for latency, deadlines, elapsed time). Directly addresses earlier cross-crate time ambiguity.

**Status: 🟢 Time-domain model: substantially improved**

### Core has made serious safety refactor

Core source now explicitly implements `WriteGuard<Uninit> -> InitializedWriteGuard -> commit()` — very good direction. Old dangerous pattern exposing `&mut T` over uninitialized storage removed from safe API. Consumer similarly has `ReadGuard -> Deref<Target=T>` with no DerefMut.

### But still P0 safety concern — init_with()

New API contains:

```rust
pub fn init_with<F>(self, f: F) -> InitializedWriteGuard<'a, T>
where F: FnOnce(&mut MaybeUninit<T>)
```

and after callback returns, unconditionally constructs InitializedWriteGuard. Source itself admits “We cannot enforce at compile time” and recommends write_value() for 100% safety. This is not sufficient for sound safe Rust API. Consider:

```rust
let guard = producer.allocate().unwrap();
let initialized = guard.init_with(|_| { // do nothing
});
initialized.commit();
```

Closure can return without initializing MaybeUninit<T>. Type returned nevertheless InitializedWriteGuard<T> and commit() publishes it. Consumer then dereferences slot as T. That can become UB.

**Severity: 🔴 P0 / safety-critical**

Correct fix: Simplest safe API is `pub fn write_value(self, value: T) -> InitializedWriteGuard<'a, T>` and remove safe init_with() entirely. If field-by-field initialization required, expose constructor based on `MaybeUninit::write` with type-level proof strategy, or keep operation explicitly unsafe: `pub unsafe fn init_with_unchecked(...)` with documented requirement closure MUST fully initialize value. Do not have safe method that claims initialization when compiler cannot verify it.

### Another core concern: SPSC abstraction has legacy escape hatch

New Producer<T>/Consumer<T> is deliberately not Clone, which is good. But legacy API still exposes `Publisher<T>::ring() -> Arc<RingBuffer<T>>` and `Publisher::from_ring(...)` — means code can still obtain underlying ring and construct additional endpoints. So effectively two models: new API capability-restricted SPSC, legacy API raw Arc<RingBuffer>. Second weakens first.

**Recommended core API:** Eventually make RingBuffer, Producer, Consumer internal implementation details, expose only `let (tx, rx) = channel::<Twist>(64);` with Producer/Consumer as only capabilities. If compatibility requires Publisher/Subscriber, they should wrap those capabilities rather than expose ring.

### Core still performs manual memory management

RingBuffer uses alloc, dealloc, Layout, MaybeUninit, raw pointers, unsafe — justified for high-performance ring buffer, but verification burden accordingly high. Next required evidence is not another unit test, but Miri, Loom, stress/concurrency testing, drop testing, wraparound testing, panic-path testing.

### And more — see full Pass 20-23 document for complete analysis of time-domain separation,_execution stats semantic mismatch, node implementation vs node runtime, controller realtime claim qualification, Instant usage, parameters control-plane vs data-plane, branch has better architecture than previous audit suggested, verification bar should now become stricter, required end-to-end proof using canonical types nros_types::Twist → Producer → runtime → Consumer → VelocityController → MotorCommand → simulation, current verification matrix, priority corrections P0 fix init_with(), close raw ring escape hatch, establish runtime kernel, P1 connect macros to descriptors, canonical E2E test, Miri/Loom, updated architecture target, bottom line branch materially improved, canonical type unification implemented, but safe initialization invariant still not enforced because init_with() can manufacture InitializedWriteGuard without proving initialization, and runtime kernel missing.

---

## Pass 21 — Verification Strategy and Remaining Proof Gaps

Continuing from current branch state, with key corrections from Pass 20 retained: nros-types is now canonical message/time layer, nros-node has migrated, core ring has substantially improved type-state API, safe initialization invariant still not fully enforced, runtime/executor remains principal missing integration layer. Also attempted to resolve GitHub Actions evidence directly — available GitHub interface not exposing branch's workflow-run endpoint cleanly, so will not claim CI passed or failed without actual run evidence.

### Verification hierarchy needs to change now

At this stage, source inspection alone insufficient. NROS should be verified in four progressively stronger layers:
- L0 — Static structure
- L1 — Unit/component correctness
- L2 — Cross-crate integration
- L3 — Executable runtime evidence
- L4 — Safety/concurrency evidence

Repository currently has substantial L0/L1 evidence, major missing evidence is L2–L4.

### L0 — Architecture

Current result: 🟢 / 🟠 — dependency direction becoming coherent: nros-types → nros-core, nros-node, nros-hal → runtime → transport/sim/studio. Missing runtime box remains central issue.

### L1 — Component correctness

- nros-types 🟢 Strong — canonical Twist, Vector3, MotorCommand, Odometry, Timestamp, ExecutionStats
- nros-core 🟠 Strong prototype — ring/channel implementation sophisticated enough to warrant formal safety verification
- nros-node 🟠 Substantial — controller/lifecycle/parameter/statistics abstractions real code
- Studio 🟠 Substantial infrastructure, but live-runtime provenance remains unresolved

### L2 — Cross-crate integration is real next gate

Crucial test no longer does nros-core compile, but can nros-core and nros-node communicate through canonical nros-types without adapters? Desired proof: `nros_types::Twist → nros_core::Producer<Twist> → runtime → nros_core::Consumer<Twist> → VelocityController`. If this path doesn't exist yet, architecture remains modular but not integrated.

### L2 — Canonical message test — minimal test conceptually `let (tx, rx) = channel::<Twist>(16); tx.write_value(Twist::default()).commit(); let msg = rx.try_read().unwrap();` Then feed same Twist into actual node callback. Important: No conversion between nros-core::Twist and nros-node::Twist — there should only be one Twist.

### L2 — Node callback integration — publish → queue → executor → callback → state mutation

### Strongest current missing test — `tests/canonical_runtime_vertical_slice.rs` with 10 phases: construct runtime, register controller, create /cmd_vel publisher/subscriber, publish Twist, executor dispatches callback, controller produces MotorCommand, simulation receives, simulator updates state, telemetry records.

### L3 — Runtime existence — Runtime needs concrete public object `let runtime = Runtime::builder().build()?; runtime.register(controller)?; runtime.spin()?;` Or `nros::init()?; let node = Controller::new(...); nros::register(node)?; nros::spin()?;` Important point is spin() must not mean `println!("spinning"); return;` It must own actual execution loop.

### L3 — Telemetry provenance — DemoDataProvider simulated, supposed LiveNrosDataProvider still needs proven against actual runtime data. Desired chain: Executor → callback count, execution duration, deadline misses, node state → TelemetrySnapshot → Studio. Until exists, Studio must not report synthetic values as live runtime measurements.

### L4 — Core memory safety — intended invariant Reserved → Initialized → Published, dangerous API is init_with() because its type signature allows MaybeUninit<T> to become InitializedWriteGuard<T> without compile-time proof. Recommended API correction: safe path write_value only, unsafe init_with_unchecked for expert escape hatch. Then run Miri for write/read/drop/abort/wraparound/full/empty/zero capacity/single/multiple/panic, plus DropProbe counter. Loom second essential proof for atomic synchronization Release/Acquire, model smallest state machine Producer → slot state → Consumer, required Loom assertions: No double publication, No read before publication, No duplicate consumption, Visibility producer write → commit Release → consumer Acquire → observes write, Index ordering monotonic, SPSC semantics formally decided strict SPSC vs serialized MPSC vs separate primitives SpscRing/MpscQueue/MpmcQueue recommendation C, don't market serialized MPSC as SPSC.

### Miri/Loom, raw-ring escape hatch should disappear — public API should not permit Publisher → Arc<RingBuffer> because that makes possible to construct endpoint ownership outside SPSC discipline. Prefer Channel<T> { Producer<T>, Consumer<T> } with actual ring private.

### Realtime claims must be downgraded until measured — current code has <1ms target, deadline_misses, execution duration useful but proper terminology is deadline monitoring not verified realtime execution until runtime controls and measures scheduler, thread priority, allocation behavior, blocking, CPU affinity, memory faults, execution jitter. Proper realtime benchmark needs distributions N=1M min/mean/median/p95/p99/p99.9/max/deadline miss count under idle/CPU load/I/O load/multiple nodes/multiple publishers/queue saturation. NROS should separate Functional (callback eventually executes correctly) vs Performance (callback normally completes within X μs) vs Realtime (system provides bounded execution/scheduling guarantee under defined conditions).

### Runtime shutdown deterministic protocol, runtime ownership explicit, CLI thin client, launch configuration compile into RuntimeConfig, canonical graph model RuntimeGraph containing NodeSpec/TopicSpec/PublisherSpec/SubscriberSpec/QoSSpec/ExecutorSpec/LifecycleSpec, nros-audit should become evidence gate, evidence states strict PLANNED→SCAFFOLDED→IMPLEMENTED→TESTED→INTEGRATION-VERIFIED→SAFETY-VERIFIED.

### Current branch status after this pass: Architecture Improving rapidly, Type system Canonicalized, Core Sophisticated but safety proof incomplete, Node subsystem Substantial, Runtime still principal missing layer, Macros Need runtime metadata integration, Studio live path Needs actual runtime telemetry, Realtime guarantees Not demonstrated, CI evidence Not asserted without actual branch workflow run.

### Most important correction before further feature work: Stop feature expansion temporarily and enforce sequence: Fix init_with() → Close raw RingBuffer escape hatch → Miri → Loom → Runtime kernel → Executor → Node registration → Topic registration → Canonical vertical slice → Runtime telemetry → Studio live path → CI evidence. This order minimizes architectural rework.

---

## Pass 22 — Reconciling Audit Against Actual Implementation

Branch identity: `arena/01a0188d-nros` and repository's own Pass 20 audit identifies baseline as `c3d3a87` with subsequent local/type-state/canonical-type/claim-linter changes. Branch also contains substantial evidence/audit corpus: AUDIT.md, AUDIT_PASS_8_12.md, AUDIT_PASS_13_19.md, AUDIT_PASS_20.md, EVIDENCE_REGISTRY.md, DESIGN.md, COMPARISON.md. That is actually useful characteristic: project attempting to preserve own verification history rather than relying exclusively on README claims.

### First correction: init_with() really is safety problem

Actual implementation:

```rust
pub fn init_with<F>(self, f: F) -> InitializedWriteGuard<'a, T>
where
    F: FnOnce(&mut MaybeUninit<T>)
```

After callback returns, unconditionally constructs InitializedWriteGuard. Source says Safety: closure must have initialized MaybeUninit and acknowledges cannot be enforced at compile time. Not merely documentation weakness. Because init_with() is safe, caller can legally provide closure that does nothing, resulting InitializedWriteGuard can be committed, consumer dereferences slot as T → UB. Therefore 🔴 init_with() is genuine safe-API soundness defect.

### Correct fix: write_value() as safe primitive, then either remove init_with() or make unsafe init_with_unchecked

Even cleaner safe ergonomic API would return T: `pub fn init_with<F>(self, f: F) -> InitializedWriteGuard where F: FnOnce() -> T` but loses true field-by-field in-place initialization. For this project, explicit unsafe is preferable to deceptively safe zero-copy API.

### Another important issue: as_mut_ptr() is public

Current API has `pub fn as_mut_ptr(&self) -> *mut T` — source calls this unsafe escape hatch but method itself not declared unsafe. Returning raw pointer is not itself UB, so function can technically be safe, but API makes it extremely easy to create invalid T. Prefer `pub unsafe fn as_mut_ptr(&self) -> *mut T` and document caller must completely initialize T before commit.

### Current type-state model otherwise good

Intended state machine: WriteGuard<Uninit> --write_value()--> InitializedWriteGuard<Init> --commit()--> Published. Old failure mode genuinely eliminated: reserve() → commit() because WriteGuard does not expose commit(). Pass 20 correctly identifies as substantial remediation of CORE-014. So verdict: 🟢 Type-state architecture: good, 🔴 Safe initialization escape: still unsafe — both can simultaneously be true.

### abort_initialized() manual drop + forget — potentially correct but not proven, needs Miri

### RingBuffer::Drop walks [read, write) and drop_in_place, correct if every slot in interval initialized exactly once, but proof relies on Published ⇒ Initialized universally true, but init_with() weakens invariant.

### Canonical-type migration incomplete: HAL still owns own Timestamp and Vector3 duplicates

Actual HAL source confirms local `pub struct Timestamp` at beginning. Therefore 🟠 Canonical types: partial migration, not complete. Proper rule: Domain data types belong in nros-types; device-specific implementation state belongs in HAL. Belongs in nros-types: Timestamp, Vector3, Twist, MotorCommand, Odometry, Image, ImageFormat, ImuData, PointCloud. Belongs in nros-hal: CameraDriver, DeviceInfo, SensorConfig, DMA implementation, V4L2 handles, hardware ownership, device capabilities. That boundary much cleaner.

### HAL DMA architecture conceptually good but claims must remain qualified: simulated zero-copy sharing vs real hardware zero-copy, type-state ownership CPUOwned/DeviceOwned excellent direction but cache-coherency operations simulated.

### Project has surprisingly strong simulation honesty pattern: Instead of pretending RealDmaBuffer, RaftElection, BulletPhysicsEngine, LiveNrosDataProvider are complete, code separates SIMULATED, SCAFFOLDED, REAL. Pass 20 audit documents distinction across transport, distributed, simulation, studio, HAL — exactly right direction.

### But evidence registry now becomes critical: Once dozens of IMPLEMENTED/SIMULATED/SCAFFOLDED claims, documentation itself becomes potential source of false confidence. Evidence model should therefore require claim → implementation → test → execution → artifact rather than claim → source exists.

### Benchmark improved from synthetic 1000ns to VecDeque<Instant> real latency via publish queue but still placeholder, remains TEMPLATE rather than independently generated evidence. Methodological weakness: separate VecDeque<Instant> to associate publication and reception adds external synchronization to measurement mechanism. Stronger benchmark would embed monotonic timestamp in message or maintain explicit side-channel designed solely for instrumentation.

### Vertical slice exists — Twist -> channel() -> Producer -> WriteGuard -> InitializedWriteGuard -> commit() -> ReadGuard -> VelocityController -> MotorCommand -> Simulator — previous characterization of no meaningful vertical slice too harsh, there is one. Remaining issue: direct component-level vertical slice, not yet proof that full runtime/executor/topic-discovery architecture drives path. Gives two different vertical slices: Existing manual/component slice vs Required runtime slice Topic -> TopicRegistry -> Executor -> Subscription -> Controller -> MotorCommand topic -> Simulator. Second is what converts project from collection of interoperable components into actual ROS-like runtime.

### Macros remain major gap: procedural macros are currently largely passthrough/scaffolded, so #[nros::node] doesn't necessarily generate node descriptor, subscriptions, publishers, parameters, lifecycle hooks, executor metadata yet. Significant architecture gap because macros appear central to intended developer experience. Correct macro maturity model: syntax → metadata extraction → descriptor generation → runtime registration → executor wiring, current state roughly syntax 🟢, metadata 🔴, registration 🔴, executor wiring 🔴.

### CLI is in same situation: nros init template was reportedly fixed so it generates compilable Rust rather than referencing nonexistent workspace crates — good, but generated project compiles does not mean generated project is NROS application, next gate: nros init -> generated project -> cargo check -> cargo test -> nros runtime -> node actually executes.

### CI is still evidence blocker: Pass 20 audit says workflow exists locally but was not pushed because GitHub integration lacked workflow permission. Repository's state therefore remains: CI specification exists ≠ CI has executed. We should not convert former into latter. Current CI evidence state: .github/workflows/ci.yml as Defined/locally prepared but Remote execution evidence: unverified until actual GitHub Actions run on audited branch visible. Consistent with repository's own audit. Repository's audit discipline is actually working: project repeatedly uses IMPLEMENTED/SCAFFOLDED/SIMULATED/TEMPLATE/CI-PENDING instead of collapsing everything into done. That is good engineering governance. Problem is several high-level documents still require careful cross-checking against source and actual execution.

### Updated architecture maturity after this pass: Canonical basic types 🟢, Time model 🟢/🟠, SPSC core concept 🟢, SPSC safety proof 🔴, HAL type-state ownership 🟢, HAL real DMA 🔴, Transport basic implementation 🟠, Real network zero-copy 🔴, Distributed runtime 🔴, Physics simulation 🟠, Studio live provider 🔴, Macro codegen 🔴, CLI scaffolding 🟠, Runtime executor 🔴, Runtime graph 🔴, E2E component slice 🟢/🟠, Full runtime E2E 🔴, CI execution evidence ⚪. Next architectural bottleneck three independent blockers: Safety (Miri/Loom/API), Runtime (Executor/Registry), Integration (canonical migration, HAL/transport).

### Gate A — Safety: Before expanding ring: remove/unsafe-qualify init_with, unsafe raw pointer API, Miri, Loom, panic tests, drop tests, wraparound, multiple endpoint abuse tests

### Gate B — Integration: Complete nros-types → nros-core → nros-node → nros-hal → nros-transport → nros-sim without duplicate Timestamp/Vector3/Image/ImageFormat/Twist/MotorCommand definitions. Should become compile-time architecture rule, not merely audit recommendation.

### Gate C — Runtime: Then implement Runtime, Executor, NodeRegistry, TopicRegistry, TimerRegistry, LifecycleManager and prove publisher → topic → subscription → executor → callback with real runtime ownership.

### Recommended architectural dependency rule: nros-types ↑ {core, node, hal, transport, sim, distributed}, runtime ↑ {node, transport, sim, CLI}, studio ↑ runtime telemetry API and prohibit hal→node, transport→studio, studio→core internals, node→concrete transport unless explicit reason.

### Most important newly confirmed finding: Branch contains more implementation than superficial audit would suggest. In particular: type-state SPSC, DMA ownership state, simulation separation, transport capability model, latency model, vertical slice, claim/evidence infrastructure are real architectural work. So correct classification is not NROS is mostly scaffolding — it is NROS contains several genuinely implemented architectural kernels surrounded by partially integrated/scaffolded runtime infrastructure. Final Pass 22 verdict: Strongest areas Rust ownership/type-state architecture, simulation-vs-real separation, canonical domain-type direction, DMA ownership abstraction, SPSC conceptual design, audit/evidence discipline. Critical blockers: Safe init_with() violates initialization proof boundary, Public raw-pointer escape should be explicitly unsafe, HAL still duplicates canonical types, Miri/Loom evidence absent, Runtime executor/registry remains incomplete, Macros mostly don't generate runtime behavior, Studio live path is not proven, Real DMA/network zero-copy remains scaffolded, GitHub CI execution on this branch remains unverified.

---

## Pass 23 — Branch-integrity, CI provenance, and invariant audit

### 1. CI claim needs correction

New commit exists: `dafa7220 — Add CI workflow for NROS project` dated 2026-08-19, cryptographically verified, adds `.github/workflows/Ci.yml` with jobs build-gate, bench, safety-gate, nros-init-compile including cargo fmt --check, cargo check --workspace --all-targets, cargo test --workspace, cargo clippy, Miri, core safety tests, nros-init compilation. So CI commit exists in repository's Git history, but does not establish workflow belongs to requested branch.

### 2. Critical branch discrepancy

Fetched actual requested branch `arena/01a0188d-nros` — root contains AUDIT.md, AUDIT_PASS_8_12.md, AUDIT_PASS_13_19.md, AUDIT_PASS_20.md, COMPARISON.md, Cargo.toml, DESIGN.md, EVIDENCE_REGISTRY.md, README.md, benchmarks/, crates/, docs/, implementations/ but no .github/ directory appears in branch root listing. Therefore: CI commit exists in repository 🟢, CI workflow exists somewhere in repo 🟢, CI workflow proven on target branch 🔴, CI workflow execution proven 🔴. There is now evidence of branch/provenance mismatch.

### 3. CI commit's parent revealing

GitHub reports `dafa7220` parent `467ce30` Initial commit direct child of repository's Initial commit rather than demonstrably descendant of audited arena/01a0188d-nros history. So we currently have repository with two histories: arena/01a0188d-nros full NROS tree vs dafa7220 CI commit — relationship needs reconciliation.

### 4. New P0 governance issue — CI provenance / branch-integrity reconciliation required

Project's intended rule should be: CI evidence is valid only when workflow and source being tested are descendants of same audited commit lineage. Otherwise we can accidentally get CI passes on one tree, audit says verified for another tree. Unacceptable for safety-oriented runtime.

**New finding: 🔴 CI provenance / branch-integrity reconciliation required**

### 5. Workflow itself has another important problem — safety gate contains `|| echo`

Safety gate contains `cargo miri test -p nros-core --lib -- --nocapture || echo "Miri check attempted"` — Miri failure does not fail job, command fails and shell proceeds because of `|| echo`. Therefore Miri failure → echo → job continues — not safety gate, but safety attempt. Especially problematic because Miri is being used to establish soundness, workflow labels job Safety Gate — nros-core soundness but one important checks is non-blocking. Should either be Miri verification and remain advisory, or Safety Gate with failure fatal. For repository with unsafe memory-management code, strongly recommend latter.

### 6. Benchmark job explicitly non-gating

Workflow says `continue-on-error: true` for benchmarks — perfectly reasonable, benchmarks should not normally block correctness CI, but evidence status must be benchmark executed rather than benchmark verified unless generated results persisted and independently inspected.

### 7. nros-init job not actually testing nros init

More serious: Job called nros init must produce buildable project but shown script creates /tmp/test_robot manually and writes `fn main() { println!("Hello NROS - compilable template"); }` then runs `cargo check` — proves plain Rust project → cargo check PASS, not proves `nros init → generated project → Cargo.toml → nros dependencies → generated macros → generated source → cargo check`. Classic evidence mismatch. Verdict 🔴 Golden test does not currently exercise claimed behavior.

### 8. Correct nros init CI test

Should literally execute something equivalent to `cargo run -p nros-cli -- init /tmp/test_robot` or `nros init /tmp/test_robot`, then `cd /tmp/test_robot && cargo check`, then `cargo test`, preserve generated Cargo.toml src/ cargo metadata cargo check output.

### 9. CI workflow therefore has three evidence classes

Real checks: cargo fmt, check, test, clippy — Potentially non-gating checks: benchmarks — Misnamed/weak checks: Miri failure swallowed, nros init CLI not actually invoked — valuable finding because CI can otherwise create false sense of completion.

### 10. Branch-level CI gate should be made explicit

For target branch, need: `arena/01a0188d-nros → commit SHA → workflow file exists at SAME SHA → workflow executes → run conclusion = success`, anything less is CI-PENDING.

### 11-23. Invariant Audit I-001 to I-012

- I-001 Published data is initialized Desired Published<T> ⇒ initialized T Current 🔴 NOT PROVEN Reason init_with() can produce InitializedWriteGuard without compiler proof
- I-002 Only one producer Desired SPSC<T> ⇒ exactly one Producer<T> — new API deliberately avoids cloning producer, Status 🟢/🟠 good capability design but public access to underlying ring/reconstruction APIs weakens guarantee
- I-003 Only one consumer same 🟢/🟠 good capability model, needs removal/restriction of raw-ring reconstruction paths
- I-004 Every initialized value is dropped exactly once Desired Initialized → exactly one drop, manually manages drop_in_place mem::forget ring Drop, Status 🟠 unproven precisely where Miri must become mandatory
- I-005 Abandoned reservation cannot expose garbage Desired reserve → producer disappears → slot never becomes Published, repository already has dedicated abandoned-reservation test, Status 🟢 component test exists but 🟠 formal memory-model proof still missing
- I-006 ReadGuard cannot outlive queue incorrectly lifetime structure designed to tie ReadGuard<'a> to ring lifetime, good Rust design, Status 🟢 strong static protection
- I-007 Domain types have one canonical identity Desired Twist == nros_types::Twist etc, Status 🟠 incomplete HAL still contains duplicate types
- I-008 Hardware ownership is represented by types DMA state machine CPU-owned → Device-owned → CPU-owned represented using Rust types, Status 🟢 architecturally strong but actual cache-coherency operations remain hardware-dependent/scaffolded, so ownership model 🟢 hardware coherence 🔴
- I-009 Simulation must not masquerade as hardware Current architecture explicitly separates simulated and real implementations, Status 🟢 good
- I-010 Runtime telemetry must originate from runtime execution Desired executor → callback metrics → telemetry → studio, Current 🔴 not proven as complete path synthetic/demo providers still exist
- I-011 Macro metadata must correspond to runtime behavior Desired #[node] #[subscribe] #[publish] → generated metadata → runtime registration, Current 🔴 not implemented end-to-end
- I-012 CI result must correspond to audited source Desired audited SHA = tested SHA, Current 🔴 not currently established, new CI commit and requested branch currently present different observable histories

**Consolidated invariant matrix:**

| ID | Invariant | Status |
|----|-----------|--------|
| I-001 | Published ⇒ initialized | 🔴 |
| I-002 | Single producer | 🟢/🟠 |
| I-003 | Single consumer | 🟢/🟠 |
| I-004 | Exactly-once drop | 🟠 |
| I-005 | Abandoned reservation safe | 🟢/🟠 |
| I-006 | Read lifetime safety | 🟢 |
| I-007 | Canonical domain types | 🟠 |
| I-008 | DMA ownership | 🟢 |
| I-009 | Simulation honesty | 🟢 |
| I-010 | Live telemetry provenance | 🔴 |
| I-011 | Macro→runtime integration | 🔴 |
| I-012 | CI/source provenance | 🔴 |

**Overall maturity:**

```
Architecture          ████████░░  80%
Domain model           ████████░░  80%
Unsafe-core design     ██████░░░░  60%
Safety evidence        ███░░░░░░░  30%
Runtime                ████░░░░░░  40%
Integration            █████░░░░░  50%
Hardware realization   ██░░░░░░░░  20%
CI evidence            ███░░░░░░░  30%
Governance/evidence    ███████░░░  70%
```

Biggest risk now evidence drift: source evolves → audit says X → CI tests Y → documentation claims Z without common SHA/evidence identity. Repository needs single verification manifest `docs/audit/verification.json` with repository/branch/audited_sha/ci_sha/verification_timestamp and gates fmt/check/test/clippy/miri/loom/hardware and safety miri/loom/unsafe_api_audit required, types canonical_types required, runtime executor/topic_registry/vertical_slice required, ci fmt/check/test/clippy required, then every audit report can state Verification manifest SHA: Source SHA: CI workflow SHA: CI run, CI itself should emit machine-readable evidence instead of only PASS generate JSON source_sha workflow_sha rust_toolchain fmt PASS check PASS test PASS clippy PASS miri PASS vertical_slice PASS then store as CI artifact, safety gate should be redesigned Miri failure → STOP and separately advisory benchmarks failure → report only, makes CI semantics unambiguous, next milestone should not be more features, project is now at point where another 10 crates or 20 APIs would make verification harder, next milestone should be NROS Verification Gate 1 with only four objectives: Make unsafe core sound, Finish canonical type migration, Establish runtime vertical slice, Establish SHA-bound CI evidence, concrete remediation order P0-A Core soundness remove/unsafe-qualify init_with, unsafe raw pointer API, Miri mandatory, drop tests, Loom, P0-B Branch/CI reconciliation identify branch HEAD, identify CI commit ancestry, put workflow on target branch, run it, record SHA, P1-A Canonical migration HAL Timestamp → nros-types, HAL Vector3 → nros-types, HAL Image → nros-types, P1-B Runtime Executor, TopicRegistry, NodeRegistry, P1-C Real vertical slice Twist → topic → executor → controller → MotorCommand → simulator, current final verdict requested branch is not production-verified, but considerably more mature than conventional scaffold, most accurate description: NROS is serious architectural prototype with several implemented low-level kernels, meaningful component-level vertical slice, and unusually detailed self-audit documentation — but unsafe core, runtime integration, canonical-type migration, and CI provenance are not yet sufficiently proven for strong system-level verification claim, and there is now particularly important new fact: CI workflow was committed on August 19, 2026, but requested arena branch does not currently expose .github/workflows tree. CI commit is direct child of Initial commit. Therefore CI workflow must be reconciled with target branch before its results can be used as evidence for this audit. That is highest-priority repository-integrity issue discovered in Pass 23.

---

## Appendix: Full evidence taxonomy per EVIDENCE_REGISTRY.md

See previous audits for full taxonomy and remediation. This file extends audit with Pass 20-23 re-verification after remediation, type-state safety, canonical types, runtime missing, branch integrity, CI provenance, invariant audit and additional analysis of current branch state.
