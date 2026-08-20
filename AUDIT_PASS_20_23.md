# NROS — Deep Analysis & Verification — Pass 20 to 23

---

## Pass 20 — Re-verification against the current branch

I re-checked the branch itself rather than relying on the earlier audit narrative. There is an important update:

> The repository has moved materially forward since the earlier passes. The canonical nros-types layer now exists and nros-node has migrated to it.

So one of my earlier conclusions — that the workspace still had duplicated Twist/Vector3/Timestamp definitions across the main crates — is now stale for the current branch.

The remaining problem is deeper: the type architecture has improved, but the executable runtime architecture is still incomplete, and the core SPSC safety surface still contains a potentially unsound initialization escape.

---

### 1. Current workspace architecture

The branch now contains:

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

The workspace manifest explicitly lists nros-types and nros-audit, confirming that the architecture has evolved since the earlier snapshot.

Current intended dependency direction is therefore much healthier:

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

This is a significant improvement.

### 2. nros-types is now genuinely canonical

The new crate explicitly describes itself as:

> “single source of truth for messages, time, geometry”

and specifically says it fixes the earlier nros-core::Twist ≠ nros-node::Twist duplication.

It contains:

- WallTimestamp, MonotonicInstant, Timestamp, MonotonicTimestamp
- Vector3, Twist, MotorCommand, Odometry
- Point3D, PointCloud
- Image, ImuData
- ExecutionStats

This is exactly the architectural refactoring previously recommended.

**Status: 🟢 Canonical message layer: implemented**

### 3. nros-core now depends on nros-types

The core manifest explicitly has:

```
nros-types = { path = "../nros-types" }
```

That is the correct dependency direction.

The core is now positioned as:

```
nros-types
    ↑
nros-core
```

rather than owning application message definitions itself.

### 4. nros-node has also migrated

The node crate now imports:

```rust
pub use nros_types::{
    WallTimestamp,
    MonotonicInstant,
    Vector3,
    Twist,
    MotorCommand,
    Odometry,
};
```

and explicitly states it now uses single source of truth nros-types.

So earlier Pass 18 finding `nros-core::Twist ≠ nros-node::Twist` is no longer applicable.

**Corrected verdict: 🟢 Node/core message duplication: remediated**

### 5. But there is a subtle problem with Timestamp

The node creates `type Timestamp = WallTimestamp` and then attempts to provide compatibility methods. The architectural intention is good: legacy Timestamp → WallTimestamp, but this should be checked carefully at compilation because inherent implementation on alias of type defined in another crate can run into Rust's orphan/inherent-impl restrictions. More importantly, compatibility layer encourages old code to continue thinking of Timestamp as undifferentiated time type. Canonical API should encourage WallTimestamp, MonotonicInstant directly.

### 6. Time-domain separation is now substantially better

nros-types explicitly distinguishes Wall time (WallTimestamp based on SystemTime for wire protocol, external timestamps, ROS time) and Monotonic time (MonotonicInstant based on Instant for latency, deadlines, elapsed time). This directly addresses earlier cross-crate time ambiguity.

**Status: 🟢 Time-domain model: substantially improved**

### 7. However, canonical types contain semantic mismatch

ExecutionStats is defined inside nros-types: callback_count, total_execution_time_ns, max_execution_time_ns, min_execution_time_ns, deadline_misses, but node's actual runtime statistics use AtomicU64 for these counters. So there are now potentially two representations: `nros_types::ExecutionStats` plain u64 snapshot vs `nros_node::ExecutionStats` AtomicU64 live counters. That isn't necessarily wrong, but architecture should explicitly distinguish ExecutionStatsSnapshot from ExecutionStatsCounters, otherwise canonical type isn't actually canonical for live telemetry path.

### 8. Core has made serious safety refactor

Core source now explicitly implements:

```
WriteGuard<Uninit>
        ↓
InitializedWriteGuard
        ↓
commit()
```

Old dangerous pattern exposing `&mut T` over uninitialized storage removed from safe API. Consumer similarly has `ReadGuard → Deref<Target=T>` with no DerefMut. That is strong improvement.

### 9. But still P0 safety concern

New API contains:

```rust
pub fn init_with<F>(self, f: F) -> InitializedWriteGuard<'a, T>
where
    F: FnOnce(&mut MaybeUninit<T>)
```

and after callback returns, unconditionally constructs InitializedWriteGuard. Source itself admits "We cannot enforce at compile time" and recommends write_value() for 100% safety. This is not sufficient for sound safe Rust API. Consider:

```rust
let guard = producer.allocate().unwrap();
let initialized = guard.init_with(|_| {
    // do nothing
});
initialized.commit();
```

Closure can return without initializing MaybeUninit<T>. Type returned nevertheless InitializedWriteGuard<T> and commit() publishes it. Consumer then dereferences slot as T. That can become UB.

**Severity: 🔴 P0 / safety-critical**

### 10. Most important current finding

Core comments claim `Published ⇒ Initialized` but `init_with()` provides safe route that can violate that invariant. Therefore invariant currently only holds if callers use `write_value()` and not `init_with()`. That's not complete safety guarantee.

**Correct fix:** Simplest safe API is `pub fn write_value(self, value: T) -> InitializedWriteGuard` and remove safe `init_with()` entirely. If field-by-field initialization required, expose constructor based on `MaybeUninit::write` with type-level proof strategy, or keep operation explicitly unsafe: `pub unsafe fn init_with_unchecked(...)` with documented requirement closure MUST fully initialize value. Do not have safe method that claims initialization when compiler cannot verify it.

### 11. Another core concern: safe initialization invariant still not fully enforced

New API:

```
WriteGuard
    ├── write_value(T)
    ├── abort()
    └── commit() [should not exist on Uninit]
InitializedWriteGuard
    └── commit()
```

Old failure mode genuinely eliminated: `reserve() → commit()` because WriteGuard does not expose commit(). Good. But `init_with()` still allows uninitialized → initialized transition without proof.

### 12. Legacy escape hatch

New Producer/Consumer is deliberately not Clone, which is good. Comments say this enforces one producer one consumer, but legacy API still exposes `Publisher::ring() -> Arc<RingBuffer<T>>` and `Publisher::from_ring(...)` — means code can still obtain underlying ring and construct additional endpoints. So effectively two models: new API capability-restricted SPSC, legacy API raw Arc<RingBuffer>. Second weakens first.

### 13. Why this matters

SPSC safety is not merely RingBuffer is thread-safe, it relies on exactly one producer, one consumer. If arbitrary endpoint creation remains public, API no longer enforces claimed ownership discipline. Therefore new endpoint model is good, but old public escape hatch prevents calling SPSC discipline fully enforced by type system.

**Recommended core API:** Eventually make RingBuffer, Producer, Consumer internal implementation details, expose only `let (tx, rx) = channel::<Twist>(64);` with Producer/Consumer as only capabilities. If compatibility requires Publisher/Subscriber, they should wrap those capabilities rather than expose ring.

### 14. Arc<RingBuffer<T>> should not become public primitive

Desired: `Publisher<T> → Producer<T> → private RingBuffer<T>` not `Publisher<T> → Arc<RingBuffer<T>> → arbitrary producer, arbitrary consumer, arbitrary endpoint`. Especially important if NROS wants to make strong realtime/ownership claims.

### 15. Core still performs manual memory management

RingBuffer uses `alloc, dealloc, Layout, MaybeUninit, raw pointers, unsafe` — justified for high-performance ring buffer, but verification burden accordingly high. Next required evidence is not another unit test, but Miri, Loom, stress/concurrency testing, drop testing, wraparound testing, panic-path testing.

### 16. Drop behavior needs dedicated verification

Ring destructor walks `[read, write)` and drops every element. Reasonable only if every slot in that range is definitely initialized and no element already dropped. State machine appears designed around that invariant, but because `init_with()` can publish uninitialized value, destructor inherits same soundness problem. This is why initialization issue is not isolated, it contaminates commit/read/drop semantics.

### 17. abort_initialized() deserves scrutiny

Method manually calls `drop_in_place(T)` then manually clears reservation and forgets guard. Can be correct, but ownership protocol manually encoded. Any future change to Drop can accidentally create double drop or leaked value. Safer design centralize state transition.

### 18. Core should have formal slot state model

Instead of relying mainly on comments, define:

```
SlotState
├── Free
├── Reserved
├── Initialized
└── Published
```

Then map operations: try_reserve Free→Reserved, write_value Reserved→Initialized, commit Initialized→Published, try_read Published→ConsumerOwned, ReadGuard::drop ConsumerOwned→Free. Implementation may remain optimized lock-free, but state model becomes explicit enough to test.

### 19. Current core architecture much stronger than Pass 18

Project gone from approximately raw ring buffer to type-state publication protocol + private SPSC capabilities + canonical message types — meaningful engineering progress. Remaining problem not conceptual design, but proving every public API path preserves intended invariants.

### 20. nros-types itself appropriately lightweight

Its dependency section empty — excellent. Canonical type layer should remain small, dependency-light, portable, deterministic. Should not become dumping ground for runtime functionality.

### 21. One thing would change: split types by semantic domain eventually

Current crate manageable, but already contains time, geometry, messages, images, IMU, execution statistics. Future architecture could use `nros-types` → `nros-time, nros-geometry, nros-msg, nros-observability` or keep single crate for v0.1. Would not split yet — current single crate preferable until runtime stabilizes.

### 22. Bigger missing piece remains runtime

New canonical types do not by themselves establish node registration, topic registry, executor, scheduler, callback dispatch, lifecycle orchestration. So system remains: types 🟢, core 🟠, node 🟠, transport 🟠, HAL 🟠, simulation 🟠, Studio 🟠, runtime 🔴. This is still dominant architectural gap.

### 23. Important distinction: node implementation ≠ node runtime

nros-node now contains fairly serious VelocityController with parameters, lifecycle, emergency stop, safety limits, execution statistics, deadline monitoring, odometry state — valuable. But VelocityController::on_cmd_vel() being implemented does not mean NROS automatically calls it. Need Twist publication → subscription registration → executor → on_cmd_vel(). Last three are still critical integration boundary.

### 24. Controller's realtime claim also needs qualification

Source says Target: <1ms execution, deadline: 1000us, but measuring execution duration and incrementing atomics does not prove realtime guarantee. True claim requires controlling allocation, blocking, scheduler behavior, CPU affinity, priority, preemption, page faults, logging, memory access, interrupt effects. Therefore <1 ms target is performance requirement, not evidence of realtime guarantee. Distinction should be enforced in documentation.

### 25. Instant usage inside callback acceptable for measurement, not proof

Controller uses `let start = Instant::now();` and later records elapsed time — fine as instrumentation, but measurement clock doesn't provide scheduler guarantee or deadline enforcement. Runtime must eventually own scheduling policy.

### 26. Parameters still fundamentally control-plane infrastructure

Controller uses `HashMap<String, Parameter>`, `String`, mutable parameter state — appropriate outside hard realtime path. Source already caches robot configuration into fields `wheel_base, wheel_radius, max_speed, max_angular_speed` which is correct direction. Invariant should be: parameter service → validated configuration → atomic/runtime-safe update → cached realtime state, not callback → HashMap lookup.

### 27. Current branch therefore has better architecture than previous audit suggested

Previously: duplicated message types + incomplete time model + core + node. Now: canonical types + explicit time domains + core type-state ring + node consuming canonical types. That is real remediation. Branch should not be judged using old duplicate types everywhere finding.

### 28. But verification bar should now become stricter

Because architecture improving, next question no longer: Does project have right abstractions? It is: Can we prove abstractions compose without violating invariants? That means integration tests now more important than adding more standalone modules.

### 29. Required end-to-end proof

First genuine system test should now use new canonical types: `nros_types::Twist → nros_core::Producer<Twist> → runtime → nros_core::Consumer<Twist> → VelocityController::on_cmd_vel() → nros_types::MotorCommand → simulation`. That test would validate major architectural refactor.

### 30. Current verification matrix

| Area | Verdict | Confidence |
|------|---------|------------|
| Workspace structure | 🟢 | High |
| Canonical message types | 🟢 | High |
| Node uses canonical types | 🟢 | High |
| Time-domain separation | 🟢 | High |
| Core type-state concept | 🟢 | High |
| Core memory safety | 🔴 | Requires Miri/Loom + API fix |
| SPSC capability isolation | 🟠 | Legacy escape hatch |
| Node implementation | 🟠 | Good subsystem prototype |
| Realtime guarantee | 🔴 | Not proven |
| Executor/runtime | 🔴 | Main gap |
| Macro integration | 🔴 | Still needs runtime wiring |
| Transport integration | 🟠 | Needs E2E proof |
| Simulation integration | 🟠 | Needs E2E proof |
| Studio live telemetry | 🔴 | Needs actual runtime |
| Hardware/DMA | 🔴 | Still scaffold/simulation |

### 31. Priority corrections

**P0 — Fix init_with():** Do not allow safe API to convert arbitrary MaybeUninit<T> into initialized state.

**P0 — Close raw ring escape hatch:** Remove or restrict `Publisher::ring()`, `Publisher::from_ring()` unless explicitly marked unsafe/internal.

**P0 — Establish actual runtime kernel:** Implement Runtime, Executor, NodeRegistry, TopicRegistry

**P1 — Connect macros to descriptors:** Implement `#[node]`, `#[publish]`, `#[subscribe]` as metadata generation

**P1 — Canonical E2E test:** Use nros-types all way through runtime

**P1 — Miri/Loom:** Prove initialization, drop, concurrency, wraparound, publication, consumption

---

## Pass 21 — Verification Strategy and Remaining Proof Gaps

Continuing from current branch state, with key corrections from Pass 20 retained: nros-types is now canonical message/time layer, nros-node has migrated onto those canonical types, core ring has substantially improved type-state API, safe initialization invariant still not fully enforced, runtime/executor remains principal missing integration layer. Also attempted to resolve GitHub Actions evidence directly — available GitHub interface not exposing branch's workflow-run endpoint cleanly, so will not claim CI passed or failed without actual run evidence.

### 1. Verification hierarchy needs to change now

At this stage, source inspection alone insufficient. NROS should be verified in four progressively stronger layers:

- L0 — Static structure
- L1 — Unit/component correctness
- L2 — Cross-crate integration
- L3 — Executable runtime evidence
- L4 — Safety/concurrency evidence

Repository currently has substantial L0/L1 evidence, major missing evidence is L2–L4.

### 2. L0 — Architecture

Current result: 🟢 / 🟠 — dependency direction becoming coherent: nros-types → nros-core, nros-node, nros-hal, then runtime, then transport/sim/studio. Missing runtime box remains central issue.

### 3. L1 — Component correctness

- nros-types 🟢 Strong — establishes canonical Twist, Vector3, MotorCommand, Odometry, Timestamp, ExecutionStats layer
- nros-core 🟠 Strong prototype — ring/channel implementation sophisticated enough to warrant formal safety verification
- nros-node 🟠 Substantial — controller/lifecycle/parameter/statistics abstractions real code rather than merely API declarations
- Studio 🟠 Substantial infrastructure, but live-runtime provenance remains unresolved

### 4. L2 — Cross-crate integration is real next gate

Crucial test no longer does nros-core compile or does nros-node compile, but can nros-core and nros-node communicate through canonical nros-types without adapters or duplicate types? Desired proof: `nros_types::Twist → nros_core::Producer<Twist> → runtime → nros_core::Consumer<Twist> → VelocityController`. If this path doesn't exist yet, architecture remains modular but not integrated.

### 5. L2 — Canonical message test

Minimal test conceptually:

```rust
let (tx, rx) = channel::<Twist>(16);
tx.write_value(Twist::default()).commit();
let msg = rx.try_read().unwrap();
assert_eq!(msg.linear.x, ...);
```

Then feed same Twist into actual node callback. Important part: No conversion between nros-core::Twist and nros-node::Twist — there should only be one Twist. That would formally close old type-identity problem.

### 6. L2 — Node callback integration

Next proof should establish publish → queue → executor → callback → state mutation. For velocity controller: Twist { linear.x, angular.z } → safety limits → wheel commands → MotorCommand. This should be real execution path, not test calls on_cmd_vel() directly. Direct callback tests prove controller, not NROS.

### 7. Strongest current missing test

Add `tests/canonical_runtime_vertical_slice.rs` with phases: construct runtime, register controller, create /cmd_vel publisher, create /cmd_vel subscriber, publish Twist, executor dispatches callback, controller produces MotorCommand, simulation receives MotorCommand, simulator updates state, telemetry records callback execution.

### 8. L3 — Runtime existence

Runtime needs concrete public object:

```rust
let runtime = Runtime::builder().build()?;
runtime.register(controller)?;
runtime.spin()?;
```

Or `nros::init()?; let node = Controller::new(...); nros::register(node)?; nros::spin()?;`

Important point is spin() must no longer mean `println!("spinning"); return;` It must own actual execution loop.

### 9. Executor minimum viable design

Do not jump directly to distributed realtime scheduler. First runtime should be single process, single executor, bounded queues, explicit wakeups, deterministic shutdown. Something like Executor with callbacks, timers, lifecycle.

### 10. Executor should not depend on Studio

Correct dependency: runtime → execution → state → telemetry → Studio. Never Studio → runtime execution. Studio is observer.

### 11. L3 — Telemetry provenance

Earlier finding remains important: DemoDataProvider is simulated, supposed LiveNrosDataProvider still needs to be proven against actual runtime data. Desired chain: Executor → callback count, execution duration, deadline misses, node state → TelemetrySnapshot → Studio. Until exists, Studio must not report synthetic values as live runtime measurements.

### 12. L4 — Core memory safety

Intended invariant: Reserved → Initialized → Published. Dangerous API is init_with() because its type signature allows MaybeUninit<T> to become InitializedWriteGuard<T> without compile-time proof. Should be fixed before claiming ring is memory-safe.

### 13. Recommended API correction

Make safe path `pub fn write_value(self, value: T) -> InitializedWriteGuard` only. If expert-level escape hatch needed: `pub unsafe fn init_with_unchecked(...)`. Then safety contract explicit.

### 14. Then run Miri

Ring implementation should have dedicated Miri tests for write, read, drop, abort, wraparound, full queue, empty queue, zero capacity, single element, multiple elements, panic during producer operation, panic during consumer operation, especially MaybeUninit<T> with nontrivial Drop type.

### 15. Loom is second essential proof

Miri verifies memory-model/UB issues, Loom verifies concurrent interleavings. SPSC ring should be tested under adversarial scheduling: Producer reserve/write/commit, Consumer observe/read/release including producer stalls, consumer stalls, wraparound, queue full/empty, rapid alternating operations.

### 16. Raw-ring escape hatch should disappear

Public API should not permit `Publisher → Arc<RingBuffer>` because that makes possible to construct endpoint ownership outside intended SPSC discipline. Prefer `Channel<T> { Producer<T>, Consumer<T> }` with actual ring private.

### 17. L4 — Drop correctness

Explicit test matrix for queue containing A B C D: read A drop A, read B drop B, destroy queue and verify A,B,C,D dropped exactly once, then repeat around ring wraparound. Especially important because implementation uses manual memory management.

### 18. Realtime claims must be downgraded until measured

Current code has <1ms target, deadline_misses, execution duration — useful, but proper terminology is deadline monitoring, not verified realtime execution, until runtime controls and measures scheduler, thread priority, allocation behavior, blocking, CPU affinity, memory faults, execution jitter.

### 19. Proper realtime benchmark needs distributions

Don't report callback = 300 μs as proof. Collect N=1,000,000 callbacks and report min, mean, median, p95, p99, p99.9, max, deadline miss count, then repeat under idle, CPU load, I/O load, multiple nodes, multiple publishers, queue saturation.

### 20. NROS should separate three claims

- Functional: callback eventually executes correctly
- Performance: callback normally completes within X μs
- Realtime: system provides bounded execution/scheduling guarantee under defined conditions

These are different claims. Current NROS can reasonably pursue first two, third requires substantially stronger evidence.

### 21. Runtime shutdown is another hidden requirement

Executor needs deterministic shutdown protocol: Running → StopRequested → StopAcceptingWork → Drain/Cancel → StopNodes → ReleaseResources → Stopped. Avoid `std::process::exit()` inside runtime components.

### 22. Runtime ownership should be explicit

Good hierarchy: Runtime → Executor, NodeRegistry, TopicRegistry, TimerRegistry, Transport, Simulation, Telemetry. Nodes should not own executor, Studio should not own runtime state, CLI should not own runtime state. One authoritative lifecycle.

### 23. CLI should become thin client

Eventually: `nros run` should do load project → build RuntimeConfig → construct Runtime → load nodes → start executor → attach telemetry → wait, not implement second runtime. Likewise `nros topic` should query runtime's canonical TopicRegistry.

### 24. Launch configuration should compile into RuntimeConfig

Instead of separate semantic models nros.toml, launch.yaml, CLI flags, runtime config, compile them into RuntimeConfig with validation before execution.

### 25. Canonical graph model

Next important type should be `RuntimeGraph` containing NodeSpec, TopicSpec, PublisherSpec, SubscriberSpec, QoSSpec, ExecutorSpec, LifecycleSpec. Then CLI, Studio, validator, runtime, recorder, replayer all consume same graph representation.

### 26. This also solves static graph checking

Graph checker can validate before runtime: publisher topic exists, subscriber topic exists, message types match, QoS compatible, node names unique, executor assignment valid. Then runtime receives validated graph.

### 27. nros-audit should become evidence gate

Presence of audit crate is promising. Strongest future model: nros-audit with API invariants, graph invariants, safety invariants, configuration invariants, evidence reports. Should not merely report module exists, should establish property P → test T → evidence E → status VERIFIED.

### 28. Evidence states should be strict

PLANNED → SCAFFOLDED → IMPLEMENTED → TESTED → INTEGRATION-VERIFIED → SAFETY-VERIFIED. Do not let IMPLEMENTED implicitly mean VERIFIED. Especially important for unsafe Rust.

### 29. Current branch status after this pass

Architecture 🟢 Improving rapidly, Type system 🟢 Canonicalized, Core 🟠 Sophisticated but safety proof incomplete, Node subsystem 🟠 Substantial, Runtime 🔴 Still principal missing layer, Macros 🔴 Need runtime metadata integration, Studio live path 🔴 Needs actual runtime telemetry, Realtime guarantees 🔴 Not demonstrated, CI evidence ⚪ Not asserted without actual branch workflow run.

### 30. Most important correction before further feature work

Stop feature expansion temporarily and enforce sequence:

1. Fix init_with() → 2. Close raw ring escape hatch → 3. Miri → 4. Loom → 5. Runtime kernel → 6. Executor → 7. Node registration → 8. Topic registration → 9. Canonical vertical slice → 10. Runtime telemetry → 11. Studio live path → 12. CI evidence

This order minimizes architectural rework.

### 31. Decisive acceptance gate — NROS-RUNTIME-GATE-01

Passes only if:
- [ ] nros-types is sole message definition source
- [ ] nros-core safe publication cannot publish uninitialized T
- [ ] no safe raw-ring endpoint duplication exists
- [ ] Miri passes
- [ ] Loom passes
- [ ] Runtime exists
- [ ] Executor exists
- [ ] NodeRegistry exists
- [ ] TopicRegistry exists
- [ ] canonical Twist path executes end-to-end
- [ ] MotorCommand reaches simulator
- [ ] runtime telemetry is real
- [ ] Studio reports real telemetry
- [ ] shutdown is deterministic
- [ ] CI executes these tests

Only after that should project move aggressively toward distributed, DMA, hardware, hard realtime, GPU/NPU, fleet.

### 32. Final assessment

Latest branch better than earlier audit suggested. Project has successfully moved one major architectural layer forward: duplicated domain types → canonical nros-types → core + node converge. Next transition is much larger: independent subsystems → unified runtime. And there is one issue that should be treated as blocking rather than cosmetic: safe init_with() path must not be able to manufacture initialized guard without proving initialization. Until fixed, ring cannot receive strong memory-safety verdict. Central engineering objective for next phase is therefore now very clear: NROS Safety kernel + Runtime kernel → Miri/Loom → Executor/Registry → Canonical E2E Slice → Twist → Node → MotorCommand → Simulator → Telemetry → Studio. Once this passes on actual GitHub CI, NROS will have evidence for being integrated runtime rather than merely collection of increasingly mature subsystem prototypes.

---

## Pass 22 — Reconciling Audit Against Actual Implementation

Branch identity: arena/01a0188d-nros and repository's own Pass 20 audit identifies baseline as c3d3a87 with subsequent local/type-state/canonical-type/claim-linter changes. Branch also contains substantial evidence/audit corpus: AUDIT.md, AUDIT_PASS_8_12.md, AUDIT_PASS_13_19.md, AUDIT_PASS_20.md, EVIDENCE_REGISTRY.md, DESIGN.md, COMPARISON.md. That is actually useful characteristic: project attempting to preserve own verification history rather than relying exclusively on README claims.

### First correction: init_with() really is safety problem

Actual implementation:

```rust
pub fn init_with<F>(self, f: F) -> InitializedWriteGuard<'a, T>
where
    F: FnOnce(&mut MaybeUninit<T>)
```

After callback returns, unconditionally constructs InitializedWriteGuard. Source says Safety: closure must have initialized MaybeUninit and acknowledges cannot be enforced at compile time. That is not merely documentation weakness. Because init_with() is safe, caller can legally provide closure that does nothing. Resulting InitializedWriteGuard can then be committed, after which consumer dereferences slot as T. Therefore 🔴 init_with() is genuine safe-API soundness defect. Should remain P0 until corrected.

### Correct fix

Cleanest fix: `pub fn write_value(self, value: T) -> InitializedWriteGuard` as safe initialization primitive. Then either remove init_with() or make it `pub unsafe fn init_with_unchecked(...)` with precise contract: closure MUST fully initialize MaybeUninit<T>. Even cleaner safe ergonomic API would return T: `pub fn init_with<F>(self, f: F) -> InitializedWriteGuard where F: FnOnce() -> T` but loses true field-by-field in-place initialization. For this project, explicit unsafe is preferable to deceptively safe zero-copy API.

### Another important issue: as_mut_ptr() is public

Current API has `pub fn as_mut_ptr(&self) -> *mut T` — source calls this unsafe escape hatch but method itself not declared unsafe. Returning raw pointer is not itself UB, so function can technically be safe, but API makes it extremely easy to create invalid T. Prefer `pub unsafe fn as_mut_ptr(&self) -> *mut T` and document caller must completely initialize T before commit. Makes safety boundary visible to Rust tooling and reviewers.

### Current type-state model otherwise good

Intended state machine: WriteGuard<Uninit> --write_value()--> InitializedWriteGuard<Init> --commit()--> Published. Old failure mode genuinely eliminated: reserve() → commit() because WriteGuard does not expose commit(). Pass 20 correctly identifies this as substantial remediation of CORE-014. So verdict: 🟢 Type-state architecture: good, 🔴 Safe initialization escape: still unsafe — both can simultaneously be true.

### abort_initialized() also needs scrutiny

Implementation does `drop_in_place(T)` then manually clears reservation and forgets guard. Can be correct, but ownership protocol manually encoded. Any future change to Drop can accidentally create double drop or leaked value. Safer design centralize state transition.

### RingBuffer::Drop deserves same scrutiny

Destructor assumes `[read_idx, write_idx)` contains initialized objects. Valid only if Published ⇒ Initialized universally true. But init_with() currently weakens invariant. So safety problem propagates init_with() → InitializedWriteGuard → commit() → write_idx → RingBuffer::Drop → drop_in_place(). Same defect can affect destruction, not only reads.

### Canonical-type migration incomplete

Earlier described as essentially complete, project's own Pass 20 says otherwise: nros-hal/src/lib.rs still defines own Timestamp and Vector3 duplicates. And actual HAL source confirms local `pub struct Timestamp` at beginning of file. Therefore 🟠 Canonical types: partial migration, not complete.

### HAL still owns own time type

HAL currently has `pub struct Timestamp { sec, nanosec }` with now(), to_millis(), from_millis(), elapsed_ms(), difference_ms() but nros-types already provides canonical wall-time abstraction. Therefore two semantic universes: nros-types::WallTimestamp canonical vs nros-hal::Timestamp duplicate. INTEGRATION-001 migration not finished.

### HAL Image is also duplicated

HAL defines own `ImageFormat` and `Image` and its Image uses `timestamp: Timestamp` plus width, height, format, data, frame_id, dma_buffer_id. Meanwhile nros-types already has image-related types. Should be reconciled. Otherwise architecture eventually becomes nros-types::Image ≠ nros-hal::Image which recreates exact integration problem canonical crate was introduced to solve.

### Proper rule for nros-types

Repository should enforce: Domain data types belong in nros-types; device-specific implementation state belongs in HAL. Therefore belongs in nros-types: Timestamp, Vector3, Twist, MotorCommand, Odometry, Image, ImageFormat, ImuData, PointCloud. Belongs in nros-hal: CameraDriver, DeviceInfo, SensorConfig, DMA implementation, V4L2 handles, hardware ownership, device capabilities. That boundary much cleaner.

### HAL DMA architecture conceptually good but claims must remain qualified

HAL makes very useful distinction SimulatedDmaBuffer vs RealDmaBuffer. Simulated implementation uses Arc<Vec<u8>> and therefore shares memory without copying byte buffer when cloned. That's legitimate in-process shared ownership, but it is not hardware DMA, DMA-BUF, IOMMU mapping, camera → GPU zero-copy. Source explicitly acknowledges real implementation remains scaffolded. So 🟢 simulated zero-copy sharing, 🔴 real hardware zero-copy — distinction should remain enforced throughout documentation.

### Particularly interesting HAL state machine

HAL has `DmaBufferState<OwnedByCpu>` and `DmaBufferState<OwnedByDevice>` with CPUOwned → submit() → DeviceOwned → complete() → CPUOwned. This is very good application of Rust's type system. Device-owned state deliberately lacks `as_mut_slice()` while CPU-owned state has it. Architecture verdict 🟢 Excellent direction.

### But DMA state machine currently simulates cache coherence

Source comments indicate submit() → cache clean → memory barrier → DMA fence and complete() → DMA fence → cache invalidate → memory barrier but these operations currently comments, not actual hardware synchronization. Therefore type-state ownership = implemented, cache-coherency operations = simulated — distinction belongs in evidence registry.

### Project has surprisingly strong simulation honesty pattern

Instead of pretending RealDmaBuffer, RaftElection, BulletPhysicsEngine, LiveNrosDataProvider are complete, code separates SIMULATED, SCAFFOLDED, REAL. Pass 20 audit documents this distinction across transport, distributed, simulation, studio, HAL. That is exactly right direction for research/runtime project.

### But evidence registry now becomes critical

Once repository has dozens of IMPLEMENTED/SIMULATED/SCAFFOLDED claims, documentation itself becomes potential source of false confidence. Evidence model should therefore require claim → implementation → test → execution → artifact rather than claim → source exists.

### Benchmark is another example

Pass 20 says benchmark improved from `stats_clone.record_receive(1000);` to actual producer/consumer timestamp measurement using VecDeque<Instant>. That is meaningful correction, but audit also says benchmarks/results.json remains TEMPLATE rather than independently generated evidence. Therefore 🟠 Benchmark implementation: improved, 🔴 Benchmark artifact: not independently verified — distinction should be preserved.

### Benchmark still has methodological weakness

Current measurement architecture uses separate VecDeque<Instant> to associate publication and reception. That measures producer timestamp → ring → consumer but adds external synchronization/data structure to measurement mechanism. Stronger benchmark would embed monotonic timestamp in message or maintain explicit side-channel designed solely for instrumentation. Audit itself suggests direction via MonotonicInstant.

### Vertical slice exists — but not whole NROS runtime

This is important positive finding. Pass 20 reports vertical slice: Twist → channel() → Producer → WriteGuard → InitializedWriteGuard → commit() → ReadGuard → VelocityController → MotorCommand → Simulator. That means previous characterization of no meaningful vertical slice was too harsh. There is one. Remaining issue: It is direct component-level vertical slice, not yet proof that full runtime/executor/topic-discovery architecture drives path.

### This gives two different vertical slices

Existing: Manual/component slice Twist → channel → controller → simulator

Required: Runtime slice Topic → TopicRegistry → Executor → Subscription → Controller → MotorCommand topic → Simulator

Second is what converts project from collection of interoperable components into actual ROS-like runtime.

### Macros remain major gap

Pass 20 reports procedural macros are currently largely passthrough/scaffolded. That means `#[nros::node] struct MyNode;` doesn't necessarily generate node descriptor, subscriptions, publishers, parameters, lifecycle hooks, executor metadata yet. This is significant architecture gap because macros appear central to intended developer experience.

### Correct macro maturity model

Project should explicitly track macro syntax → metadata extraction → descriptor generation → runtime registration → executor wiring. Current state roughly syntax 🟢, metadata 🔴, registration 🔴, executor wiring 🔴. So macro compilation is not equivalent to macro functionality.

### CLI is in same situation

nros init template was reportedly fixed so it generates compilable Rust rather than referencing nonexistent workspace crates — good. But generated project compiles does not mean generated project is NROS application. Next gate: nros init → generated project → cargo check → cargo test → nros runtime → node actually executes.

### CI is still evidence blocker

Pass 20 audit says workflow exists locally but was not pushed because GitHub integration lacked workflow permission. This is extremely important. Repository's state therefore remains: CI specification exists ≠ CI has executed. We should not convert former into latter.

### Current CI evidence state

I therefore classify `.github/workflows/ci.yml` as 🟡 Defined / locally prepared but ⚪ Remote execution evidence: unverified until actual GitHub Actions run on audited branch visible. Consistent with repository's own audit.

### Repository's audit discipline is actually working

Project repeatedly uses language such as IMPLEMENTED, SCAFFOLDED, SIMULATED, TEMPLATE, CI-PENDING instead of collapsing everything into done. That is good engineering governance. Problem is several high-level documents still require careful cross-checking against source and actual execution.

### Updated architecture maturity

- Canonical basic types 🟢
- Time model 🟢/🟠
- SPSC core concept 🟢
- SPSC safety proof 🔴
- HAL type-state ownership 🟢
- HAL real DMA 🔴
- Transport basic implementation 🟠
- Real network zero-copy 🔴
- Distributed runtime 🔴
- Physics simulation 🟠
- Studio live provider 🔴
- Macro codegen 🔴
- CLI scaffolding 🟠
- Runtime executor 🔴
- Runtime graph 🔴
- E2E component slice 🟢/🟠
- Full runtime E2E 🔴
- CI execution evidence ⚪

### Next architectural bottleneck is now obvious

There are actually three independent blockers:

- Safety: Miri, Loom, API
- Runtime: Executor, Registry
- Integration: canonical migration, HAL/transport

They should be handled independently.

### Gate A — Safety

Before expanding ring: remove/unsafe-qualify init_with, unsafe raw pointer API, Miri, Loom, panic tests, drop tests, wraparound, multiple endpoint abuse tests

### Gate B — Integration

Complete nros-types → nros-core → nros-node → nros-hal → nros-transport → nros-sim without duplicate Timestamp, Vector3, Image, ImageFormat, Twist, MotorCommand definitions. This should become compile-time architecture rule, not merely audit recommendation.

### Gate C — Runtime

Then implement Runtime, Executor, NodeRegistry, TopicRegistry, TimerRegistry, LifecycleManager and prove publisher → topic → subscription → executor → callback with real runtime ownership.

### Recommended architectural dependency rule

```
nros-types
   ↑
   ├── core
   ├── node
   ├── hal
   ├── transport
   ├── sim
   └── distributed

runtime
   ↑
   ├── node
   ├── transport
   ├── sim
   └── CLI

studio
   ↑
   └── runtime telemetry API
```

and prohibit hal → node, transport → studio, studio → core internals, node → concrete transport implementation unless very explicit reason.

### Most important newly confirmed finding

Branch contains more implementation than superficial audit would suggest. In particular: type-state SPSC, DMA ownership state, simulation separation, transport capability model, latency model, vertical slice, claim/evidence infrastructure are real architectural work. So correct classification is not "NROS is mostly scaffolding" — it is "NROS contains several genuinely implemented architectural kernels surrounded by partially integrated/scaffolded runtime infrastructure." That is much more accurate assessment.

### Final Pass 22 verdict

Strongest areas: Rust ownership/type-state architecture, simulation-vs-real separation, canonical domain-type direction, DMA ownership abstraction, SPSC conceptual design, audit/evidence discipline. Critical blockers: Safe init_with() violates initialization proof boundary, Public raw-pointer escape should be explicitly unsafe, HAL still duplicates canonical types, Miri/Loom evidence absent, Runtime executor/registry remains incomplete, Macros mostly don't generate runtime behavior, Studio live path is not proven, Real DMA/network zero-copy remains scaffolded, GitHub CI execution on this branch remains unverified.

---

## Pass 23 — Branch-integrity, CI provenance, and invariant audit

**Focus:** Branch-integrity, CI provenance, and invariant audit — uncovered significant repository-state discrepancy that changes verification status. Most important finding is not in Rust code, but in Git history / branch provenance.

### 1. CI claim needs to be corrected

A new commit exists: `dafa7220 — Add CI workflow for NROS project` dated 2026-08-19 16:59:57 UTC, cryptographically verified, adds `.github/workflows/Ci.yml` with jobs build-gate, bench, safety-gate, nros-init-compile including cargo fmt --check, cargo check --workspace --all-targets, cargo test --workspace, cargo clippy, Miri, core safety tests, nros-init compilation. So there is now CI commit in repository's Git history. But that does not establish workflow belongs to requested branch.

### 2. Critical branch discrepancy

Fetched actual requested branch `arena/01a0188d-nros` — root contains `AUDIT.md, AUDIT_PASS_8_12.md, AUDIT_PASS_13_19.md, AUDIT_PASS_20.md, COMPARISON.md, Cargo.toml, DESIGN.md, EVIDENCE_REGISTRY.md, README.md, benchmarks/, crates/, docs/, implementations/` but no `.github/` directory appears in branch root listing. Therefore: CI commit exists in repository 🟢, CI workflow exists somewhere in repo 🟢, CI workflow proven on target branch 🔴, CI workflow execution proven 🔴 — supersedes simpler statement from Pass 22 that CI execution is merely unverified. There is now evidence of branch/provenance mismatch.

### 3. CI commit's parent revealing

GitHub reports `dafa7220` parent `467ce30` — CI commit is direct child of repository's Initial commit, rather than demonstrably descendant of audited arena/01a0188d-nros history. So we currently have:

```
repository
                      │
          ┌───────────┴───────────┐
          │                       │
   arena/01a0188d-nros       dafa7220
          │                       │
     full NROS tree          CI commit
          │                       │
          └──────── ? ────────────┘
```

Relationship between histories needs to be explicitly reconciled.

### 4. New P0 governance issue — CI provenance / branch-integrity reconciliation required

Project's intended rule should be: CI evidence is valid only when workflow and source being tested are descendants of same audited commit lineage. Otherwise we can accidentally get CI passes on one tree, audit says verified for another tree. Unacceptable for safety-oriented runtime project. New finding 🔴 CI provenance / branch-integrity reconciliation required.

### 5. Workflow itself has another important problem — safety gate contains `|| echo`

Safety gate contains `cargo miri test -p nros-core --lib -- --nocapture || echo "Miri check attempted"` — Miri failure does not fail job, command fails and shell proceeds because of `|| echo`. Therefore Miri failure → echo → job continues — not a safety gate, but safety attempt. Especially problematic because Miri is being used to establish soundness, workflow labels job Safety Gate — nros-core soundness but one of most important checks is non-blocking. Should either be Miri verification and remain advisory, or Safety Gate with failure fatal. For repository with unsafe memory-management code, strongly recommend latter.

### 6. Benchmark job explicitly non-gating

Workflow says `continue-on-error: true` for benchmarks — perfectly reasonable, benchmarks should not normally block correctness CI, but evidence status must be benchmark executed rather than benchmark verified unless generated results persisted and independently inspected.

### 7. nros-init job not actually testing nros init

More serious: Job called nros init must produce buildable project but shown script creates `/tmp/test_robot` manually and writes `fn main() { println!("Hello NROS - compilable template"); }` then runs `cargo check` — proves plain Rust project → cargo check PASS, not proves `nros init → generated project → Cargo.toml → nros dependencies → generated macros → generated source → cargo check`. Classic evidence mismatch. Verdict 🔴 Golden test does not currently exercise claimed behavior.

### 8. Correct nros init CI test

Should literally execute something equivalent to `cargo run -p nros-cli -- init /tmp/test_robot` or `nros init /tmp/test_robot`, then `cd /tmp/test_robot && cargo check`, then `cargo test`, preserve generated Cargo.toml, src/, cargo metadata, cargo check output.

### 9. CI workflow therefore has three evidence classes

Real checks: cargo fmt, check, test, clippy — Potentially non-gating checks: benchmarks — Misnamed / weak checks: Miri → failure swallowed, nros init → CLI not actually invoked — valuable finding because CI can otherwise create false sense of completion.

### 10. Branch-level CI gate should be made explicit

For target branch, need: `arena/01a0188d-nros → commit SHA → workflow file exists at SAME SHA → workflow executes → run conclusion = success` — Anything less is CI-PENDING.

### 11-36. Invariant Audit (Detailed)

*Detailed invariant audit per Pass 23 covering I-001 to I-012: Published data is initialized, Only one producer, Only one consumer, Every initialized value is dropped exactly once, Abandoned reservation cannot expose garbage, ReadGuard cannot outlive queue incorrectly, Domain types have one canonical identity, Hardware ownership is represented by types, Simulation must not masquerade as hardware, Runtime telemetry must originate from runtime execution, Macro metadata must correspond to runtime behavior, CI result must correspond to audited source.*

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

Biggest risk now evidence drift: source evolves → audit says X → CI tests Y → documentation claims Z without common SHA/evidence identity. Repository needs single verification manifest `docs/audit/verification.json` with repository/branch/audited_sha/ci_sha/verification_timestamp and gates fmt/check/test/clippy/miri/loom/hardware, and claim ledger Claim/Source/Evidence/Status.

**Current final verdict:** Requested branch is not production-verified, but considerably more mature than conventional scaffold. Most accurate description: **NROS is a serious architectural prototype with several implemented low-level kernels, a meaningful component-level vertical slice, and unusually detailed self-audit documentation — but its unsafe core, runtime integration, canonical-type migration, and CI provenance are not yet sufficiently proven for strong system-level verification claim.**

**Highest-priority repository-integrity issue:** CI workflow committed on August 19, 2026, but requested arena branch does not currently expose .github/workflows tree. CI commit is direct child of Initial commit. Therefore CI workflow must be reconciled with target branch before its results can be used as evidence for this audit.
