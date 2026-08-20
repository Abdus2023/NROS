# NROS — Deep Verification Pass 13 to 19

## The next decisive pass

### NROS — Deep Verification Pass 13

The next decisive area is the transport → HAL → node → distributed boundary. This is where the project’s architecture either becomes a real runtime or remains a collection of mostly independent prototypes.

The next pass should move above individual crates and inspect the actual transport/HAL/distributed boundaries:

```
nros-core
    ↓
nros-transport
    ↓
nros-hal
    ↓
nros-node
    ↓
nros-distributed
    ↓
nros-sim
    ↓
nros-studio
    ↓
nros facade
```

The key questions will be:

- Is transport actually backed by the core ring?
- Does HAL preserve zero-copy semantics or clone?
- Does distributed transport preserve message ownership?
- Are async boundaries compatible with the claimed realtime model?
- Are Send/Sync assumptions valid across all layers?
- Does simulation use the same interfaces as hardware?
- Does Studio observe real runtime state or synthetic telemetry?
- Can one message travel through the complete stack without being copied, fabricated, or converted through duplicate types?

That is the real NROS integration gate.

---

## Pass 14 — Transport/HAL boundary and the zero-copy claim

The key question in this pass is:

**Does NROS preserve the `nros-core` ownership/zero-copy model once a message leaves the core ring and enters transport/HAL?**

The answer is currently **no demonstrated end-to-end proof**. The architecture has the right interfaces, but the evidence boundary stops well before a verified hardware/transport pipeline.

### 1. The critical distinction: zero-copy inside the ring ≠ zero-copy system-wide

NROS currently has an important conceptual separation:

```
              nros-core
                   │
           zero-copy slot
                   │
                   ▼
              consumer
```

That can legitimately be called zero-copy **inside the ring**.

But larger architecture claims much stronger pipeline:

```
application
    ↓
node
    ↓
transport
    ↓
HAL
    ↓
DMA/device
```

For full claim to hold, message must remain same owned memory region throughout path. That requires evidence at every boundary.

### 2. Actual proof obligation

Project should formally define:

```
ZERO-COPY-001
For a message M:
    allocation(M) = A
and for every layer L:
     address_L(M) == A
```

until ownership explicitly transferred or message crosses boundary where copying unavoidable.

Then distinguish:

```
INTRA_PROCESS_ZERO_COPY
INTER_THREAD_ZERO_COPY
DMA_ZERO_COPY
NETWORK_ZERO_COPY
```

These are completely different claims.

### 3. `nros-core` can support first category

Ring's intended design is producer → slot<T> → consumer with consumer taking ownership through `ReadGuard`. That is legitimate basis for **in-process ownership-transfer / zero-copy messaging** assuming initialization and lifetime problems are fixed. But nothing about this automatically proves DMA zero-copy, network zero-copy, GPU zero-copy.

### 4. HAL must not silently clone

Evidence registry already identifies important issue in camera path: `buf.data.clone()` rather than true DMA-backed ownership. That means current camera pipeline is effectively:

```
device buffer
     ↓
copy
     ↓
Vec<u8>
     ↓
NROS message
```

rather than:

```
DMA buffer
     ↓
NROS-owned buffer view
```

**Verdict: 🔴 HAL zero-copy = not implemented**

### 5. Correct HAL abstraction needs ownership, not just bytes

Weak abstraction: `trait Buffer { fn data(&self) -> &[u8]; }` because caller has no information about allocation source, alignment, physical address, DMA ownership, cache state, lifetime, synchronization, mutability.

For real hardware integration, buffer needs something closer to:

```
DmaBuffer
├── virtual_addr
├── physical_addr / IOVA
├── len
├── alignment
├── cache_state
├── ownership
└── lifetime
```

Exact fields depend on platform, but ownership contract must exist.

### 6. DMA ownership is fundamentally different from Rust ownership

Rust can establish one owner of `&mut T` but not automatically `CPU owns buffer vs DMA engine owns buffer`. You need explicit state machine:

```
CPUOwned
    │
    ├── submit()
    ▼
DMAOwned
    │
    ├── complete()
    ▼
CPUOwned
```

This is exactly kind of invariant that should be represented in types.

### 7. Recommended HAL type-state

Something like:

```rust
DmaBuffer<OwnedByCpu>
DmaBuffer<OwnedByDevice>
```

with transitions `submit()` → `DmaBuffer<OwnedByDevice>` `complete()` → `DmaBuffer<OwnedByCpu>`. Then compiler prevents CPU modifies DMA-owned memory without explicit unsafe escape hatch.

### 8. Cache coherency must be part of contract

On embedded systems, zero-copy is not sufficient. You also need:

```
CPU cache
        ↕
memory
        ↕
DMA
```

with explicit synchronization when required: `cache clean, cache invalidate, memory barrier, DMA fence` may be necessary. So actual HAL safety contract should include `ownership + cache coherency + memory ordering + lifetime`.

### 9. Transport boundary even harder

Inside one process `Arc<RingBuffer<T>>` can preserve Rust ownership. Across network:

```
robot A
    │
    │ Ethernet
    ▼
robot B
```

There is no shared Rust memory. Therefore architecture must distinguish:

- zero-copy local transport
- serialization-free network transport
- zero-copy receive into preallocated buffer

These are different properties.

### 10. Network transport necessarily changes ownership model

Network message normally becomes `T → wire representation → bytes → network → bytes → T`. So even if serialization optimized, you cannot honestly call this **zero-copy end-to-end** unless architecture doing something unusual such as shared-memory transport or specialized hardware. Evidence terminology should therefore say: Local: zero-copy candidate, Network: serialization-based, DMA: zero-copy planned until proven otherwise.

### 11. Distributed NROS must not inherit local zero-copy claim

Correct capability matrix:

| Transport | Copy semantics | Status |
|-----------|----------------|--------|
| same-thread | direct | possible |
| same-process SPSC | zero-copy candidate | prototype |
| shared memory | zero-copy candidate | unverified |
| UDP/TCP | serialization | prototype/planned |
| DMA | zero-copy candidate | simulated |
| GPU | specialized | simulated |
| distributed Raft | control-plane messages | simulated |

This prevents architecture from accidentally implying all NROS communication is zero-copy.

### 12. Distributed layer should have different message path

Don't force `nros-core::RingBuffer<T>` to become universal transport abstraction. Instead:

```
              Message
                  │
           TransportEnvelope
                  │
       ┌──────────┴──────────┐
       │                     │
  LocalTransport       NetworkTransport
       │                     │
  RingBuffer             serializer
```

Application should depend on capability-oriented transport interface.

### 13. Capability negotiation needed

Useful NROS concept: `TransportCapabilities { zero_copy, bounded_latency, ordered, reliable, lossy, shared_memory, dma, multicast, serialization }`. Then node can request `requires: bounded_latency, zero_copy` and runtime can reject transport that cannot satisfy it. Far better than assuming every transport has identical semantics.

### 14. Realtime requirements must propagate across transport

Suppose node declares `deadline = 1000 μs`, runtime must determine whether `node execution + queue delay + transport delay + scheduler delay` can fit within deadline. Currently node only measures own callback execution — that's execution latency not end-to-end latency.

### 15. Introduce end-to-end latency model

Useful model: `L_total = L_publish + L_queue + L_transport + L_schedule + L_callback + L_output`. For each term define min, mean, P99, P99.9, max and importantly measurement source. Otherwise number like `6.2 μs` has no clear semantic meaning.

### 16. WCET is not same as average latency

CLI currently advertises WCET analysis. But `average = 6.2 μs` does not imply `WCET < 10 μs`, even `P99.9 < 10 μs` does not imply hard upper bound. For hard realtime claims, you need either formal bound or validated bounded execution under defined assumptions and clearly documented environment.

### 17. This affects node's deadline_misses

Node currently counts deadline miss if `elapsed > deadline` — useful telemetry, but should not be interpreted as runtime guarantees deadlines, merely means application detected callback exceeded configured target. Should be encoded in evidence registry.

### 18. Scheduling is currently missing layer

Architecture needs explicit executor: `Executor { realtime worker, normal worker, background worker, IO worker }` with scheduling policy priority, deadline, budget, affinity, period. Then node callback becomes scheduled entity rather than normal method call. Until that exists `#[callback(realtime=true)]` cannot honestly imply real-time scheduling.

### 19. Macro semantics should eventually compile into executor model

Intended macro `#[callback(realtime = true, deadline_us = 1000, priority = 200)]` should generate `CallbackDescriptor { function, class: Realtime, deadline: 1ms, priority: 200 }` and register with Executor. Then runtime can actually enforce or at least monitor priority, deadline, budget. Right now metadata mostly aspirational.

### 20. HAL and transport need same evidence taxonomy

Evidence registry should not simply say `HAL = IMPLEMENTED`. Instead:
```
HAL API → IMPLEMENTED
Simulated DMA → SIMULATED
Real DMA → SPECIFIED
Cache coherency → NOT VERIFIED
Zero-copy camera → NOT IMPLEMENTED
```
Likewise: `Transport API → IMPLEMENTED, Local SPSC → PROTOTYPE, Network transport → IMPLEMENTED/SERIALIZED, Network zero-copy → NOT APPLICABLE, Shared memory → SPECIFIED`. Produces more useful engineering evidence.

### 21. Simulator should become reference integration backend

Instead of treating `nros-sim` as another feature, use it as **first complete backend**:

```
Application
     │
     ▼
NROS Runtime API
     │
     ├───────────────┐
     ▼               ▼
Simulator       Hardware backend
          backend
```

Both must implement exactly same traits. Then end-to-end test can run entirely in simulation while validating node → transport → scheduler → messages → lifecycle → diagnostics without claiming hardware validation.

### 22. This would dramatically improve CI

CI could then run: `cargo test --workspace → NROS simulator integration → end-to-end message flow → failure injection → lifecycle tests → record/replay`, then hardware CI can be added later: software CI → simulator → QEMU → real hardware. Much more credible verification ladder.

### 23. Recommended integration test

First golden test should be:

```
cmd_vel
    ↓
Twist
    ↓
transport
    ↓
VelocityController
    ↓
MotorCommand
    ↓
simulated motor
    ↓
Odometry
    ↓
odom
```

Entire loop should run without mock transport, fake messages, hard-coded metrics, manual intervention. Then validate message correctness, ordering, timestamps, deadline accounting, ownership, drop count.

### 24. Add failure injection

Serious runtime test must also test: transport unavailable, queue full, consumer stalled, deadline exceeded, device timeout, malformed message, node shutdown restart. For example: consumer stalls → producer reserves slot → queue fills → publish returns QueueFull → verify no memory leak, no duplicate publication, no lost ownership, no deadlock. Far more valuable than another happy-path unit test.

### 25. Important: queue-full semantics need to be specified

For real-time runtime, queue exhaustion is not incidental error. Define explicitly: `QueueFullPolicy { DropNewest, DropOldest, Block, Reject, Overwrite }`. For hard real-time operation, Block can be dangerous. Policy must be part of node/transport contract.

### 26. Backpressure should be explicit

Node graph: `camera 30 Hz → detector 30 Hz → planner 10 Hz → controller 100 Hz` — runtime needs explicit semantics for sampling, queue depth, latest-value, deadline, drop policy. Otherwise real-time becomes ambiguous.

### 27. This is where NROS can differentiate itself from ordinary ROS-style middleware

Project's strongest potential architectural idea is not simply “ROS but in Rust.” It is **communication contracts that encode latency, ownership, scheduling, and capability requirements.** For example: `Channel<T> where T: Message, Transport: ZeroCopy + BoundedLatency` — could make runtime much more explicit than generic middleware abstraction.

### 28. But don't encode impossible guarantees in traits

Avoid `trait HardRealtimeTransport {}` unless trait has meaningful verification contract. Prefer capability metadata: `TransportCapabilities { max_latency: Option<Duration>, zero_copy: bool, bounded: bool, reliability: Reliability }` and make runtime distinguish declared capability from verified capability.

### 29. New verification statuses needed

Existing evidence registry is good, but for runtime capabilities recommend: `DECLARED, IMPLEMENTED, TESTED, MEASURED, BOUNDED, INTEGRATION-TESTED, HARDWARE-VALIDATED`. Distinction between MEASURED and BOUNDED especially important. Measured maximum is not automatically bound.

### 30. Updated architecture after this pass

Verified architecture currently:

```
                    NROS
                      │
              ┌───────┴────────┐
              │                │
         nros-core          nros-node
              │                │
           SPSC          reference/demo
              │                │
              └──────┬─────────┘
                     │
               integration
                 incomplete
                     │
         ┌───────────┼────────────┐
         │           │            │
      transport      HAL      distributed
         │           │            │
      partial      simulated     simulated/scaffolded
```

Intended architecture much stronger:

```
                         NROS
                           │
                     Runtime API
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
       Node             Transport          HAL
         │                 │                 │
         └─────────────────┼─────────────────┘
                           │
                      Scheduler
                           │
                      Core IPC
                           │
                   ┌───────┴────────┐
                   │                │
                 Sim              HW/DMA
```

Second diagram is architecture to build toward.

### 31. New findings

🔴 P0:
- **ZERO-COPY-001** Full-stack zero-copy is not demonstrated.
- **HAL-001** Current camera path clones buffer data.
- **RT-003** No actual scheduler enforcement for callback realtime metadata.
- **E2E-001** No verified end-to-end application pipeline across core/node/transport/HAL.

🟠 P1:
- **TRANSPORT-001** Transport capability semantics need explicit modeling.
- **DMA-001** CPU/device ownership state machine absent.
- **CACHE-001** DMA cache-coherency contract absent.
- **LATENCY-001** No end-to-end latency measurement model.
- **QUEUE-001** Queue-full/backpressure semantics insufficiently formalized.

### 32. Current NROS maturity

| Area | Status |
|------|--------|
| Architecture | 🟢 strong |
| Evidence model | 🟢 strong |
| Core IPC | 🟠 prototype / safety work required |
| Node API | 🟠 reference implementation |
| Message model | 🔴 duplicated |
| Scheduler | 🔴 missing |
| Transport | 🟠/🔴 |
| HAL | 🟠 API / simulated implementation |
| DMA | 🔴 |
| Distributed | 🔴 simulated/scaffold |
| CLI | 🔴 simulation-heavy |
| Simulator | 🟠 potential integration foundation |
| End-to-end pipeline | 🔴 unproven |
| CI gate | 🔴 unresolved |
| Production readiness | 🔴 |

### 33. Highest-value next step

Next implementation target should **not** be Raft, Studio, fleet management, or more CLI commands. It should be one minimal vertical slice:

```
┌─────────────────────────────────────────┐
│          NROS Vertical Slice            │
├─────────────────────────────────────────┤
│                                         │
│  Twist                                  │
│    ↓                                    │
│  Canonical message type                 │
│    ↓                                    │
│  Producer                               │
│    ↓                                    │
│  nros-core SPSC                         │
│    ↓                                    │
│  Consumer                               │
│    ↓                                    │
│  Node callback                          │
│    ↓                                    │
│  MotorCommand                           │
│    ↓                                    │
│  Simulator backend                      │
│    ↓                                    │
│  Odometry                               │
│                                         │
└─────────────────────────────────────────┘
```

Then prove it with: `cargo test, cargo clippy, Miri, deterministic integration test, failure injection, artifact generation`. Once that slice passes, rest of architecture has real foundation.

---

## Pass 15 — Vertical-slice feasibility, transport contracts, and path to verifiable runtime

### 1. First vertical slice should be deliberately tiny

Do not start by implementing entire ROS-like runtime. First target:

```
Twist
  ↓
Publisher
  ↓
SPSC
  ↓
Subscriber
  ↓
VelocityController
  ↓
MotorCommand
  ↓
Simulator
```

Nothing else. No fleet, Raft, Studio, DMA, network transport, migration, advanced CLI, GPU. Goal is to establish one real, inspectable, end-to-end execution path.

### 2. Canonical message problem must be solved first

Currently: `nros-core::Twist` and `nros-node::Twist` are separate types. That should become `nros-types::Twist` used everywhere. Recommended structure:

```
crates/
├── nros-types
├── nros-core
├── nros-node
├── nros-transport
└── nros
```

Dependency graph:

```
nros-types
            /    |     \
           /     |      \
       core     node   transport
         \       |       /
          \      |      /
             nros
```

This should be P0 architectural change before further integration.

### 3. Don't put application messages inside nros-core

`nros-core` should contain mechanisms: `RingBuffer, Producer, Consumer, MessageSlot, Executor primitives` not application-specific robotics structures like `Twist, Odometry`. Otherwise `nros-core` becomes large coupling point. Better: `nros-types` contains `Twist, Odometry, MotorCommand`, `nros-core` contains `RingBuffer<T>, Producer<T>, Consumer<T>`.

### 4. Define minimum Message contract

Runtime needs very small foundational trait conceptually:

```rust
trait Message: Send + 'static {
    const TYPE_NAME: &'static str;
}
```

Do not immediately require `Serialize, Deserialize, Clone, Debug` — those would unnecessarily force all messages into network-oriented semantics. Local zero-copy should not require serialization.

### 5. Separate local and wire representations

Essential. `Message` → Local representation and Wire representation. For example `Twist` local and only when crossing network boundary `Twist → TwistWire → bytes`. Avoids forcing `Clone, Serialize, Deserialize` into realtime local path.

### 6. Transport should be generic over message

Transport abstraction conceptually like `Publisher<T>, Subscriber<T>` rather than `Publisher` internally knows `Twist`. Then `Publisher<Twist>, Publisher<Odometry>, Publisher<MotorCommand>` all use same mechanism.

### 7. Transport capabilities need to be first-class

Introduce `TransportCapabilities { local, zero_copy, bounded_latency, ordered, reliable, lossy, shared_memory, serialized }`. Then runtime can distinguish Local SPSC from UDP without pretending identical properties.

### 8. Channel should expose semantics

For example `ChannelConfig { capacity, overflow_policy, delivery, deadline }` Possible overflow policies: Reject, DropNewest, DropOldest, OverwriteLatest, Block. For realtime paths, Block should generally require explicit permission.

### 9. Most useful realtime primitive may actually be latest value

Many robotics signals don't require every sample, e.g., `cmd_vel` often has semantics newest command. In that case capacity=1 policy=OverwriteLatest can be more appropriate than conventional FIFO. NROS should make explicit rather than treating every topic as generic queue.

### 10. Define channel semantics before implementation

Recommend formally specifying `Channel<T>` with guarantees: Ordering published(A) before published(B) ⇒ consumer sees A before B, Ownership published(T) ⇒ exactly one consumer owns T, Visibility consumer-visible(T) ⇒ T initialized, Drop if message dropped drop(T) destructor executes exactly once, Full queue full ⇒ defined policy, No implicit behavior.

### 11. This immediately exposes core commit() flaw

Required invariant: `consumer_visible(T) ⇒ initialized(T)` but current `commit()` can conceptually transition reserved → committed without initialized. Therefore state machine must become `Reserved<Uninit> → write(T) → Reserved<Init> → commit() → Published<T>`. Cleanest resolution.

### 12. Make initialization type-state transition

Conceptually:

```rust
struct WriteGuard<T, State> { ... }
struct Uninit; struct Init;
WriteGuard<T, Uninit> has write(value) returning WriteGuard<T, Init>
Only WriteGuard<T, Init> gets commit()
```

Now compiler can enforce reserve → write → commit and prevent reserve → commit unless explicit abort.

### 13. ReadGuard should be immutable

Previous finding remains: `DerefMut` should disappear, use `Deref<T>` only. Consumer should not silently mutate published data.

### 14. SPSC should be represented by endpoint ownership

Instead of `RingBuffer<T>` freely clonable through `Arc`, create `let (producer, consumer) = channel();` Then Producer<T> cannot produce another producer, Consumer<T> cannot produce another consumer. Conceptually Channel<T> → Producer<T> + Consumer<T>. This gives type system chance to enforce SPSC contract.

### 15. Don't confuse endpoint ownership with internal Arc

Internally implementation may still use `Arc<Inner<T>>` if necessary, but public API should enforce one Producer, one Consumer. Implementation mechanism can remain shared internally.

### 16. Then build simulator against exact same API

Bad: ProductionNode → Transport A, SimulationNode → completely different simulation API. Good: Node → Transport trait → SpscTransport and SimTransport both satisfy same contract.

### 17. Simulator must not return synthetic measurements

Simulation can legitimately produce simulated time, sensor, motor, transport, but output must explicitly identify execution_mode = simulation, e.g., `{ "execution_mode": "simulation", "message_count": 1000, "deadline_misses": 0 }` not `{ "message_count": 1000 }` which could be mistaken for hardware evidence.

### 18. Introduce execution-mode contract

At runtime: ExecutionMode { Simulation, Native, Hardware, Distributed }. Every diagnostic artifact should record it. Example run.json with execution_mode, target, compiler, git_revision, configuration, measurements, evidence_level — dramatically strengthen reproducibility.

### 19. Every benchmark must contain provenance

Performance result should never just be `6.2 μs`, should be `{ "metric": "callback_latency", "value_ns": 6200, "execution_mode": "native", "target": "x86_64", "os": "...", "rustc": "...", "commit": "...", "iterations": 1000000, "warmup": 10000, "measurement": "wall_clock", "status": "repository_reported" }`. Then evidence registry can reference artifact. This solves benchmark controversy cleanly: Repository artifact: 6.2 μs, Independent verification: NO, Hardware validation: NO, Reproducibility: PENDING.

### 20. This solves benchmark controversy

Instead of arguing whether 6.2 μs is real, project can say: Repository artifact: 6.2 μs, Independent verification: NO, Hardware validation: NO, Reproducibility: PENDING — much stronger engineering communication.

### 21. Executor should be added only after message path works

Don't build sophisticated scheduler before proving message → channel → callback → output. First use `std::thread` or simple deterministic executor in vertical slice, then introduce `RealtimeExecutor` as separate milestone.

### 22. Executor milestone

Once basic vertical slice passes: Executor { TaskId, priority, deadline, period, budget, affinity, execution statistics }. Then callback metadata can finally become meaningful.

### 23. Realtime execution must avoid allocation

Callback path should eventually be audited for Vec, String, HashMap, Box, Arc clone, println!, format!, filesystem, blocking lock, SystemTime — any occurrence classified FORBIDDEN, ALLOWED, CONDITIONALLY-ALLOWED depending on realtime class.

### 24. NROS should have explicit execution classes

Define `ExecutionClass { HardRealtime, SoftRealtime, Normal, Background }` — each callback belongs to exactly one, e.g., cmd_vel → HardRealtime, odometry → SoftRealtime, parameter RPC → Normal, logging → Background. Executor can enforce different rules for each.

### 25. This makes current println! finding concrete

For HardRealtime, runtime could reject println!, allocation, blocking I/O, unbounded locks during static analysis or review. Turns architectural philosophy into enforceable rule.

### 26. Add realtime lint layer

Eventually `nros check --realtime` should inspect callback source → dependency graph → known forbidden operations → allocation paths → blocking paths and produce ERROR RT001: HardRealtime callback calls std::println, ERROR RT002: HardRealtime callback allocates Vec, WARNING RT003: SystemTime used in realtime path. Current CLI doesn't do this; future implementation target.

### 27. First integration test should have precise contract

Input: `Twist { linear.x = 1.0, angular.z = 0.2 }`, Controller: `VelocityController::on_cmd_vel()`, Output: `MotorCommand`, Assertions: message received exactly once, message initialized before publication, message order preserved, no duplicate drop, no leak, expected motor values, execution time recorded, Runtime: Simulation, Evidence: INTEGRATION-TESTED.

### 28. Then run exact test under Miri

Miri should specifically exercise reserve → write → commit → receive → drop and reserve → abort → drop and queue full → failed reservation with String, Vec<u8>, Box, nested Drop messages. Important test is not Twist { f64 fields } because trivial scalar types can hide lifetime/destructor problems.

### 29. Add adversarial message type

Create `struct DropProbe { id: usize, ... }` whose destructor increments atomic counter. Then verify publish → receive → exactly one Drop and reserve → abort → exactly one Drop and queue full → rejected → exactly one Drop. Strongest tests for ring lifecycle.

### 30. Add initialization sentinel

Use `MaybeUninit<DropProbe>` and deliberately test that commit without write cannot compile after type-state refactor.

### 31. Repository should distinguish compile-time and runtime safety

Use two evidence columns: Invariant | Compile-time | Runtime — publish requires initialization YES | —, exactly-one consumer YES | —, destructor exactly once PARTIAL | TEST, queue capacity — | TEST, deadline — | TEST, DMA ownership YES | TEST, cache state PARTIAL | HARDWARE TEST. Makes safety argument rigorous.

### 32. Vertical slice becomes new release gate

Before adding more subsystems: NROS-GATE-01 must pass: cargo fmt --check, cargo check --workspace, cargo test --workspace, cargo clippy --workspace, Miri core lifecycle, E2E simulator pipeline, documentation/evidence consistency. Only then NROS-GATE-02 can begin scheduler and later NROS-GATE-03 hardware.

### 33. Revised roadmap

Phase A — Safety foundation: canonical message crate, type-state WriteGuard, immutable ReadGuard, type-enforced SPSC endpoints, lifecycle/Miri tests

Phase B — Vertical slice: publisher, subscriber, controller, simulator, end-to-end test

Phase C — Runtime: executor, execution classes, deadlines, budgets, realtime diagnostics

Phase D — Hardware: DMA ownership, cache semantics, HAL implementation, hardware test

Phase E — Distributed: serialized transport, network semantics, discovery, Raft, fleet

### 34. Updated verdict

NROS repository is not failing because architecture is wrong. Opposite: **Architecture is currently ahead of implementation.** Most valuable work now is not adding features, but closing vertical slice and turning architectural claims into executable evidence. Three highest-priority actions: 1. Fix WriteGuard initialization soundness, 2. Unify message/time types, 3. Build one real core → node → simulator pipeline.

---

## Pass 16 — Verification Gates: CI, Miri/Loom, Reproducibility, and Evidence Closure

### 1. Current gate status

| Gate | Status | Meaning |
|------|--------|---------|
| Repository integrity | 🟢 | Branch/source can be inspected |
| Workspace structure | 🟢 | 10-crate workspace confirmed |
| Core implementation | 🟠 | substantial implementation exists |
| Core memory safety | 🔴 | as_mut() / initialization invariant unresolved |
| SPSC semantics | 🟠 | implementation exists, contract not sufficiently enforced |
| Node integration | 🟠 | dependency exists, canonical types duplicated |
| End-to-end pipeline | 🔴 | not independently demonstrated |
| CI | 🔴 | executable GitHub Actions workflow not established |
| Miri | 🔴 | not executed evidence |
| Loom | 🔴 | not executed evidence |
| Benchmarks | 🟠 | repository-reported, independently unverified |
| HAL/DMA | 🔴 | simulated/copying rather than demonstrated hardware zero-copy |
| Distributed | 🔴 | non-production status |
| Production readiness | 🔴 | gate remains open |

Crucial distinction: **Source existence is not verification.**

### 2. CI is now first infrastructure blocker

No executable `.github/workflows/ci.yml` on audited branch → claims such as `cargo test --workspace, cargo clippy, cargo fmt, Miri` cannot be treated as GitHub-verified results. Evidence-chain defect: project needs executable workflow before can truthfully say `CI PASS`.

### 3. CI should be staged, not one giant workflow

Recommend gate hierarchy: CI-01 format+compile → CI-02 unit/integration tests → CI-03 clippy → CI-04 Miri → CI-05 Loom → CI-06 benchmark/reproducibility. Makes failures attributable.

### 4. First workflow: deterministic baseline

First executable workflow should establish only: `cargo fmt --all -- --check, cargo check --workspace, cargo test --workspace, cargo clippy --workspace --all-targets -- -D warnings`. Important not mark subsequent safety gates as passed merely because this workflow exists. Each gate requires its own observed result.

### 5. Toolchain must be pinned

For reproducibility, record `rustc --version, cargo --version, rustup show, uname -a` and ideally pin toolchain through `rust-toolchain.toml` or repository-controlled mechanism. Result artifact should contain `rustc, cargo, target, commit`. Otherwise two green runs can represent different toolchains.

### 6. Miri is decisive test for nros-core

Core ring uses `MaybeUninit, raw pointers, unsafe impl, atomic synchronization, manual Drop` — exactly class of code where ordinary tests insufficient. Required Miri target is not merely `cargo test` but focused lifecycle suite: reserve → initialize → commit → receive → drop and reserve → abort → drop and queue full → failed reservation.

### 7. First Miri test should be nontrivial destructor

Use message containing owned resources: `struct Probe { bytes: Vec<u8>, text: String, boxed: Box<u64>, }` — because `Twist { f64, f64, f64 }` doesn't exercise most dangerous part of manual storage management. Test must establish initialized value → exactly one destructor, aborted reservation → no destructor of uninitialized storage, published value → exactly one destructor.

### 8. as_mut() issue must be closed before Miri is considered safety gate

Current conceptual API `WriteGuard { as_mut() -> &mut T }` is unsafe when backing storage still uninitialized. Correct design: `WriteGuard<Uninit> { as_uninit(), as_mut_ptr(), write_value(T) }` then `WriteGuard<Init> { commit() }`. Raw pointer operation can remain available as explicitly unsafe escape hatch. But safe Rust must not permit uninitialized storage → &mut T.

### 9. commit() must become impossible before initialization

Current: `Reserved → commit()` desired: `Reserved<Uninit> → write → Reserved<Init> → commit → Published`. Not merely style improvement — removes entire class of invalid states from safe API.

### 10. Loom addresses different problem

Miri answers: Is this memory operation/lifetime behavior valid? Loom answers: Are atomic synchronization assumptions correct under possible thread interleavings? NROS needs both. Ring uses atomics and memory ordering, so ordinary tests cannot establish Release and Acquire placed correctly for every relevant execution ordering.

### 11. Loom model should target smallest possible state machine

Do not immediately model whole runtime. Model Producer → slot state → Consumer with reserve, write, commit, receive, release and tiny capacity 1 or 2. Small state spaces much easier for exhaustive exploration.

### 12. Required Loom assertions

Prove at least: No double publication one reservation → at most one commit, No read before publication consumer cannot observe unpublished slot, No duplicate consumption one committed slot → at most one consumer release, Visibility producer write → commit Release → consumer Acquire → consumer observes write, Index ordering monotonic producer sequence and monotonic consumer sequence.

### 13. SPSC semantics must be formally decided

Three possible designs: A — strict SPSC one Producer one Consumer (best for current architecture), B — serialized MPSC multiple producers allowed but reservations serialized, C — separate primitives `SpscRing<T>, MpscQueue<T>, MpmcQueue<T>` (cleanest long-term). Recommendation C, with `SpscRing<T>` as highly optimized realtime primitive.

### 14. Don't market serialized MPSC implementation as SPSC

If arbitrary producers can concurrently access `Arc<RingBuffer<T>>` then type-level semantics don't match name. Affects performance claims and correctness assumptions because strict SPSC algorithm can make stronger assumptions than MPSC algorithm.

### 15. Benchmark verification must also be reconstructed

README's `6.2 μs, 780K msg/s` must remain classified as repository-reported benchmark values until executable benchmark artifact reproduces them. Repository should not convert them to verified performance.

### 16. Benchmark methodology needs to be part of result

At minimum: CPU, OS, Rust version, optimization level, target, message size, queue capacity, producer count, consumer count, warmup, iterations, statistical method, clock, commit SHA. Without these, performance comparisons weak.

### 17. Throughput and latency are different metrics

Do not combine `780K messages/sec` with `6.2 μs` as though they establish same property. Throughput messages/second, Latency time per operation. System can have high throughput and poor tail latency. For NROS, report throughput, mean latency, P50, P95, P99, P99.9, max observed separately.

### 18. Tail latency matters more than mean latency for realtime

If claim is 1 ms deadline then mean 20 μs almost irrelevant if P99.99 = 3 ms. Evidence should prioritize worst observed tail distribution, deadline misses over averages.

### 19. But maximum observed is still not WCET

Distinction explicit: max observed ≠ WCET. Benchmark can report max observed = 87 μs without proving execution ≤ 87 μs. Latter requires stronger argument.

### 20. Evidence registry should encode distinction

Recommend `EvidenceLevel: SOURCE_ONLY, IMPLEMENTED, UNIT_TESTED, INTEGRATION_TESTED, MEASURED, REPRODUCED, BOUNDED, HARDWARE_VALIDATED`. Then `6.2 μs` could currently be MEASURED only if executable benchmark produced it. If merely written in README: SOURCE_ONLY or REPOSITORY_REPORTED.

### 21. Evidence should be tied to commit

Every verification artifact should record repository = Abdus2023/NROS, branch = arena/01a0188d-nros, commit = exact SHA. This matters because branch can change after report. Result from commit A must never silently become evidence for commit B.

### 22. Add machine-readable verification manifest

For example `docs/audit/verification.json` with conceptual fields `{ "repository": "Abdus2023/NROS", "revision": "...", "gates": { "fmt": "PASS", "check": "PASS", "test": "PASS", "clippy": "PASS", "miri": "NOT_RUN", "loom": "NOT_RUN", "hardware": "NOT_RUN" } }`. Prevents documentation from drifting away from reality.

### 23. Documentation should never manually claim green gate

Instead: README → verification manifest → CI artifact. README can say `Safety verification status: See verification manifest` rather than embedding stale results. Particularly important because project has already experienced documentation/code discrepancies.

### 24. Add claim ledger

High-value addition. Example:

| Claim | Source | Evidence | Status |
|-------|--------|----------|--------|
| zero-copy SPSC | nros-core | tests | 🟠 |
| 6.2 μs latency | README | benchmark | 🟠 |
| 780K msg/s | README | benchmark | 🟠 |
| DMA zero-copy | HAL docs | simulation | 🔴 |
| realtime callback | node API | no executor | 🔴 |
| distributed replication | distributed crate | simulation | 🔴 |

Gives maintainers single authoritative view.

### 25. Claim ledger should distinguish implemented from verified

Example: Implemented: RingBuffer<T> does not mean Verified: RingBuffer<T> is memory-safe under all modeled interleavings. Latter requires Miri + Loom + tests + review depending on claim.

### 26. Security and safety review should follow same model

For unsafe Rust: `unsafe block → SAFETY comment → invariant → test → Miri/Loom`. Every unsafe block should have explicit invariant. For example: `SAFETY: slot is initialized before dereference because WriteGuard<Init> is only type exposing commit/read`. That statement becomes true only after type-state refactor.

### 27. Current unsafe impl Send/Sync deserves dedicated audit

Ring contains unsafe concurrency traits. High-value review targets because `unsafe impl Send, Sync` make promises about entire type. Audit should explicitly answer Send: Can ownership of ring safely move between threads? Sync: Can references to ring safely be shared concurrently? Internal state: Are all mutations synchronized? T: Send: Is that bound sufficient? Destruction: Can Drop occur concurrently with access? Until answers documented and tested, unsafe impls should remain review-required.

### 28. Add negative compile tests

NROS should test things that must not compile: two Producers from one SPSC channel should fail, commit uninitialized WriteGuard should fail, mutable ReadGuard should fail. These tests extremely valuable for type-state API. Tool such as `trybuild` could be appropriate.

### 29. This creates much stronger safety story

Instead of “We tested that it works.” NROS can eventually claim: The safe API makes invalid lifecycle states unrepresentable. Miri checks unsafe implementation. Loom explores concurrency model. Runtime tests verify behavior. Negative compile tests verify API restrictions. That's credible Rust safety architecture.

### 30. Final verification ladder

```
                    NROS CLAIM
                         │
                         ▼
                 Source inspection
                         │
                         ▼
                 Compile-time checks
                         │
                         ▼
                  Unit tests
                         │
                         ▼
               Integration tests
                         │
              ┌──────────┴──────────┐
              ▼                     ▼
            Miri                   Loom
              │                     │
              └──────────┬──────────┘
                         ▼
                   Benchmarks
                         │
                         ▼
                Simulator E2E
                         │
                         ▼
                    QEMU/HIL
                         │
                         ▼
                  Real hardware
```

Different claims stop at different levels. Correct way to avoid overclaiming.

### 31. Revised Gate A

Core memory safety must satisfy: [ ] no safe API creates &mut T to uninitialized memory, [ ] commit requires initialization, [ ] ReadGuard immutable, [ ] destructor exactly once, [ ] abort/drop semantics tested, [ ] Miri PASS, [ ] Loom PASS, [ ] unsafe Send/Sync reviewed. Until all satisfied: **Gate A = OPEN**

### 32. Gate B — Core messaging

[ ] strict SPSC contract selected, [ ] endpoint ownership enforced, [ ] queue-full semantics defined, [ ] ordering specified, [ ] memory visibility specified, [ ] failure/drop semantics specified, [ ] adversarial message tests

### 33. Gate C — Vertical runtime

[ ] canonical message types, [ ] publisher, [ ] subscriber, [ ] node callback, [ ] simulator, [ ] end-to-end test, [ ] provenance artifact

### 34. Gate D — Realtime

[ ] executor, [ ] execution classes, [ ] deadline accounting, [ ] allocation policy, [ ] blocking policy, [ ] realtime diagnostics, [ ] tail-latency measurements

### 35. Gate E — Hardware

[ ] DMA ownership, [ ] cache coherency, [ ] real HAL, [ ] device lifecycle, [ ] hardware-in-loop, [ ] reproducible hardware benchmark

### 36. Gate F — Distributed

[ ] serialized transport, [ ] explicit wire format, [ ] discovery, [ ] network failure semantics, [ ] replication tests, [ ] partition tests, [ ] recovery tests

### 37. Priority ordering very clear

P0 — close immediately:
1. Remove WriteGuard::as_mut().
2. Make commit() initialization-safe.
3. Decide strict SPSC vs separate MPSC.
4. Add executable CI.
5. Establish Miri.
6. Establish Loom.

P1:
1. Canonicalize Twist, Timestamp, Odometry, etc.
2. Remove ReadGuard::DerefMut.
3. Build vertical simulator pipeline.
4. Create claim/evidence ledger.

P2:
1. Executor.
2. realtime classes.
3. HAL ownership model.
4. DMA/cache contracts.

P3:
1. distributed transport.
2. fleet/Raft.
3. Studio.
4. advanced hardware integrations.

### 38. Most important conclusion from Pass 16

NROS does **not** need more architectural breadth right now. It needs **evidence depth**. Repository has already accumulated enough architecture to support meaningful runtime. Bottleneck is proving most dangerous foundations actually correct. Decisive transition: Prototype → remove unsafe lifecycle hole → Memory-safe core → Miri + Loom → Verified core → real vertical slice → Verified runtime → hardware validation → Credible NROS platform

Current overall verdict: Architecture 🟢 strong, Prototype implementation 🟢/🟠 substantial, Core safety 🔴 Gate A open, Concurrency verification 🔴 not yet demonstrated, CI evidence 🔴 not established, End-to-end runtime 🔴 not demonstrated, Hardware/zero-copy 🔴 not demonstrated, Distributed production readiness 🔴 not demonstrated. Next pass should be repository-level verification of exact unsafe blocks, synchronization primitives, and test inventory, producing line-by-line Core Safety Matrix.

---

## Pass 17 — Actual Core Safety Matrix — Branch-Level Source Verification

Branch identity confirmed: `arena/01a0188d-nros`, workspace 10 crates: nros-core, nros-node, nros-hal, nros-transport, nros-distributed, nros-cli, nros-sim, nros-studio, nros-macros, nros. Core source substantial ~28 KB lib.rs, rather than skeleton.

### Critical correction

Repository's own SAFETY.md says Status: IMPLEMENTED → TESTED → Needs CI verification + Miri/loom and lists major ownership/lifetime fixes as implemented. Useful evidence of project's remediation intent. But source still contains:

```rust
pub fn as_mut(&mut self) -> &mut T {
    unsafe { &mut *(*self.ptr).as_mut_ptr() }
}
```

Surrounding comment acknowledges creating mutable reference to potentially uninitialized memory. Therefore P0 initialization soundness issue NOT actually closed. Documentation cannot make operation sound. Gate A remains OPEN.

### Core Safety Matrix

| ID | Invariant | Implementation | Verification | Verdict |
|----|-----------|----------------|--------------|---------|
| CORE-001 | one producer reservation | write_reserved CAS | unit test | 🟠 |
| CORE-002 | one consumer reservation | read_reserved CAS | unit test | 🟠 |
| CORE-003 | T destroyed exactly once | ReadGuard::drop, ring drain | DropCounter tests | 🟠 |
| CORE-004 | Send/Sync safety | unsafe impl | documentation/tests | 🔴 |
| CORE-005 | abandoned write safe | guard Drop | test | 🟠 |
| CORE-006 | no separate consume lifetime | ReadGuard | test | 🟠 |
| CORE-007 | monotonic timing | Instant | source inspection | 🟢 |
| CORE-008 | benchmark isolated | ignored benchmark | source/docs | 🟢 |
| CORE-009 | full behavior | ReturnNone | test | 🟢 |
| INIT-001 | safe initialization | MaybeUninit + write_value | as_mut() bypass exists | 🔴 |
| CONC-001 | atomic ordering | Acquire/Release | no Loom evidence | 🔴 |
| MIRI-001 | unsafe memory correctness | unsafe allocation/access | Miri not run | 🔴 |
| E2E-001 | full runtime integration | partial | no proven E2E gate | 🔴 |

### Additional Findings

- CORE-001 substantially improved: `write_reserved: AtomicBool` CAS prevents second outstanding write reservation, test `test_double_reserve_prevention`
- CORE-002 materially improved: `read_reserved` CAS, lifetime tied to ReadGuard Drop advancing read_idx, not separate receive()/consume()
- But ReadGuard still exposes mutable access via `DerefMut` with `&mut *as_mut_ptr()` — consumer can mutate published object, undermines clean semantic model, should remove `DerefMut`, keep `Deref` only
- CORE-003 real improvement: ring Drop walks remaining initialized entries, ReadGuard drop destroys consumed T, DropCounter test, but proof assumes `[read_idx, write_idx)` contains exactly initialized undropped slots, needs to remain true under wraparound, aborted writes, active ReadGuard, panic during initialization/processing — tests cover some, Miri/Loom essential
- RingBuffer::Drop comment exposes lifecycle assumption: read_reserved slot still in range and will be dropped, but `Arc<RingBuffer>` ownership topology needs explicit explanation of ownership
- `unsafe impl Sync` deserves stronger scrutiny: Can any execution produce simultaneous access to same slot where one side has mutable and other immutable? Need to prove producer slot ≠ consumer slot whenever both active, depends on write_idx, read_idx, capacity, reservation flags, memory ordering — therefore Sync cannot be treated as independently proven until entire state machine modeled
- Atomic ordering deserves formal model: producer `read_idx.load(Acquire)`, `write_idx.store(Release)`, consumer `write_idx.load(Acquire)`, `read_idx.store(Release)` correct kind for SPSC publication, but additional states `write_reserved`, `read_reserved` introduce additional transitions — exactly where Loom should be applied, reservation CAS ordering questionable enough to test aggressively
- Reservation flags are global: `write_reserved` one boolean for entire ring, not per slot, likewise `read_reserved` — enforces one outstanding producer/consumer guard at any time, valid for strict SPSC, but should be documented as hard semantic property
- Public type does not enforce SPSC ownership: `RingBuffer<T>` can be wrapped in `Arc` and passed to multiple publishers/subscribers, flags prevent simultaneous access but runtime serialization, not type-level SPSC enforcement — should be `SpscChannel<T> { Producer<T>, Consumer<T> }` with ring private
- `Publisher::ring() -> Arc<RingBuffer<T>>` weakens encapsulation, makes it harder to enforce one publisher/one subscriber, should prefer Channel as public abstraction
- `Publisher::new(topic, capacity)` creates its own `Arc<RingBuffer<T>>` while `Subscriber::new(ring, topic)` requires ring separately — low-level API rather than middleware channel abstraction, cleaner `let (publisher, subscriber) = channel("cmd_vel", 64);`
- Timestamp remains duplicated conceptually: core defines `Timestamp` and `Twist`, support earlier architectural concern core mechanism + application message definitions still coupled, workspace would benefit from canonical `nros-types` crate
- Monotonic clock remediation good but semantics need clarification: `MonotonicTimestamp` not suitable as wire timestamp, architecture should explicitly distinguish `WallTimestamp` vs `MonotonicInstant` vs `NetworkTimestamp`
- `PerformanceStats` still not realtime-safe telemetry primitive: uses atomics good, but `print_summary` does `println!` and floating-point formatting, must not be called from hard realtime callback, API should separate `RealtimeStats` vs `HumanReadableReporter`
- Benchmark evidence remains insufficient: source correctly separates correctness tests from ignored benchmarks, safety doc says benchmark artifacts still need environment info, until GitHub Actions or independently reproducible run produces artifact, `6.2 μs, 780K msg/s` remain repository-reported not independently verified
- Repository's own safety document confirms missing verification: `loom / Miri = future`, `CI workflow = unchecked`, `benchmark artifact = unchecked` — documentation more conservative than earlier architectural summaries — good, audit should preserve distinction
- Updated Gate A: Core ownership/lifetime 🟠 substantially remediated, Initialization soundness 🔴 still OPEN because `as_mut() -> &mut T` exists on potentially uninitialized storage, Read immutability 🔴 still OPEN because `DerefMut` for `ReadGuard` exists, Destruction 🟠 strong implementation + tests, Concurrency 🔴 No Loom evidence, Unsafe Send/Sync 🔴 Review not closed, CI 🔴 Not verified on GitHub Actions
- Minimum patch set for Gate A: Remove `WriteGuard::as_mut()`, Keep only `as_mut_uninit()`, `as_mut_ptr()` unsafe, `write_value()`, Remove `DerefMut` for ReadGuard, Introduce `SpscChannel<T>` or otherwise make producer/consumer endpoint ownership explicit, Add Miri tests, Add Loom model tests, Review Send/Sync proof against actual endpoint topology
- Desired API: `let mut g = publisher.allocate()?; g.as_mut_uninit().write(value); g.commit();` or `let mut g = publisher.allocate()?; g.write_value(value); g.commit();` For truly in-place initialization, safe API should expose operations on `MaybeUninit<T>` rather than pretending object already exists
- If ergonomic field-by-field construction required, don't reintroduce `&mut T` to uninitialized memory, instead introduce constructor callback `guard.init_with(|slot| { ... })` or `MaybeUninit<T>` builder, invariant should remain no safe `&mut T` until `T` is initialized
- Subtle issue: `write_value()` permits repeated initialization `write_value(A)`, `write_value(B)`, `commit()` with no explicit destruction of A — for `T` with destructor could leak — another reason type-state API preferable: `WriteGuard<Uninit>.write_value(A) → WriteGuard<Init>.commit()` would make write twice impossible
- `abort()` is also semantically redundant, body does nothing relying on Drop, creates two concepts abort() and drop() that do same thing, cleaner API could simply say dropping uncommitted WriteGuard aborts reservation and reserve abort() for explicit readability
- `commit()` implementation should not need `mem::forget` — current code `committed=true, write_idx.store(...), write_reserved.store(false), mem::forget(self)` works around Drop, cleaner design commit(self) → perform state transition → Drop sees committed state or use dedicated internal state machine, `mem::forget` in unsafe infrastructure deserves extra scrutiny
- Wraparound issue: implementation uses `u64` `wrapping_sub` for occupancy and `wrapping_add` for indices, common technique, but proof needs explicit assumption `capacity << 2^63` and producer/consumer distance must never become ambiguous across wraparound, add mathematical property test around boundary `u64::MAX - N → u64::MAX → 0 → N`

### Current Core Safety conclusion

Actual branch better than superficial audit would suggest, real remediation: reservation guards, RAII read ownership, destructor handling, monotonic timing, benchmark separation, backpressure policy — meaningful engineering improvements, but remaining problems concentrated exactly where they matter most: unsafe initialization, unsafe mutable read access, unsafe Send/Sync proof, atomic interleavings, repeated initialization, absence of Miri/Loom evidence. So correct status: **NROS core is an advanced safety-remediation prototype, not yet a verified safe zero-copy primitive.**

---

## Pass 18 — Cross-crate ownership-flow audit

Key finding: **nros-core has been partially hardened, but higher layers still mostly operate as parallel implementations around it rather than as single canonical runtime.** Strongest guarantees established in core are not yet propagating through NROS stack.

### Dependency Topology

Workspace 10 crates: `nros-core, nros-node, nros-hal, nros-transport, nros-distributed, nros-cli, nros-sim, nros-studio, nros-macros, nros` as confirmed by workspace manifest.

But important distinction between declared dependencies and actual type integration:

- `nros-node` → `nros-core` dependency exists, but source still defines its own message/math types
- `nros-sim` → `nros-core` dependency exists, but still defines own message/math types

### Node Duplicates Core Types

`nros-node` defines own `Timestamp, Vector3, Twist, MotorCommand, Odometry` rather than using canonical core definitions — comment says compatible not identical, no type-level guarantee. Suppose `nros_core::Twist` published through core transport, node callback expects `nros_node::Twist` — Rust sees unrelated types, need conversion, undesirable for zero-copy system. Correct solution: dedicated canonical crate `nros-types` containing `Timestamp, Vector3, Twist, Odometry, Point3D, PointCloud, Image metadata`, then `nros-core, nros-node, nros-hal, nros-transport, nros-sim, nros-types` — mechanism → types not each subsystem → its own types. Better still: separate wire types from runtime types `nros-msg` (Twist, Odometry) + `nros-time` (Timestamp, MonotonicInstant, Duration, Deadline).

### Time Duplication

`nros-core` has `MonotonicTimestamp` based on `Instant` but also retains legacy `Timestamp` based on `SystemTime`, `nros-node` independently defines another `Timestamp` also using `SystemTime` — at least three time concepts: `nros-core::MonotonicTimestamp`, `nros-core::Timestamp`, `nros-node::Timestamp` — should be consolidated.

### Node Claims Real-time While Using Wall-clock Timestamps

Callback does `let start = Instant::now()` good for elapsed, but then constructs `MotorCommand { timestamp: Timestamp::now(), ... }` where `Timestamp::now()` uses `SystemTime` — subtle but important distinction execution measurement → monotonic, message timestamp → wall clock, needs explicit semantic contract.

### Node is Demonstration App Embedded in Library Crate

Manifest calls it “NROS Node Example” and source heavily example-oriented `VelocityController, ParameterServer, LifecycleNode, ExecutionStats` — useful but should be classified as example/reference implementation rather than NROS node runtime, actual runtime machinery still missing.

### Biggest Contradiction: Node's Real-time Callback Isn't Registered Anywhere

Source comments say `#[callback(realtime=true, deadline_us=1000, priority=200)]` would be used in real macro system, but actual function simply `pub fn on_cmd_vel(&mut self, msg: &Twist)` — no callback registration, scheduler, priority, deadline admission, executor. Thus `VelocityController::on_cmd_vel()` is just ordinary Rust function.

### 1 ms Deadline is Observational, Not Enforced

Records `self.stats.record_execution(elapsed, 1_000_000)` — if execution takes 1.5ms, result `deadline_misses +=1` but nothing prevents exceeding 1ms. So deadline monitoring = YES, deadline enforcement = NO.

### Parameter System Not Connected to Node Runtime

`ParameterServer` functioning local data structure declare/get/get_float/get_int/set/validate — useful but not integrated with parameter service, transport, persistent configuration, runtime graph, CLI, Studio — currently in-process HashMap not NROS parameter subsystem. Parameter updates not automatically propagated — `reload_parameters()` copies values into cached realtime fields, reasonable design for realtime paths because you don't want mutex/hash-map lookup on every callback, but architecture should make transaction explicit: configuration update → validate → stage → atomic/runtime-safe snapshot → callback sees new configuration.

### Safety_check() Performs I/O

Periodic safety callback contains `println!(...)` when timeout occurs — for hard real-time callback, standard output is not acceptable as deterministic operation, can involve locking, allocation, buffering, syscalls, scheduler interaction. Finding RT-002 — real-time safety path performs non-deterministic console I/O. Fix: safety callback → atomic/event flag → non-RT diagnostics task → logging.

### Odometry Mathematical Issue

`self.odom_theta = self.odom_theta.sin().atan2(self.odom_theta.cos())` intended to normalize angle, mathematically equivalent to `atan2(sin(theta), cos(theta))` reasonable, but integration uses `self.odom_x += linear_vel * dt * self.odom_theta.cos()` after updating `odom_theta` — simple Euler approximation using new orientation rather than midpoint orientation. For prototype acceptable, for robotics runtime should be documented as first-order integration not high-accuracy odometry.

### Core Safety Test Itself Uses Unsafe API

Core test `guard.as_mut().linear.x = 1.0; guard.as_mut().angular.z = 0.5;` uses `WriteGuard::as_mut()` — source comment itself admits this creates mutable reference to potentially uninitialized memory. Means project's main zero-copy correctness test exercises exact API it admits can be UB. 🔴 CORE-017 — correctness test relies on explicitly acknowledged UB-prone API. Should use `as_mut_uninit().write(...)` or `write_value(...)` instead.

### as_mut() Should Probably Not Exist

Offers three write mechanisms `as_mut_ptr(), as_mut_uninit(), as_mut()` — first two can be justified, third fundamentally dangerous. Much safer API: `as_uninit() -> &mut MaybeUninit<T>` plus `write_value()`. Then compiler prevents common accidental pattern.

### commit() Remains Central Soundness Flaw

API allows `let guard = ring.try_reserve().unwrap(); guard.commit();` without preceding initialization. Because commit() only does `write_idx +=1` and makes slot visible, this violates invariant visible slot ⇒ initialized T. No comment can make this sound.

### Best Fix: Eliminate Public commit(self) on Uninitialized State

Use type-state `WriteGuard<T> → write_value(T) → InitializedWriteGuard<T> → commit()` conceptually `let guard = ring.try_reserve()?; let guard = guard.write_value(value); guard.commit();` Now WriteGuard::commit() doesn't exist.

### ReadGuard::DerefMut Unnecessarily Powerful

Consumer gets `impl DerefMut for ReadGuard` means subscriber can mutate already-published message — for message transport, better API is `ReadGuard → Deref<T>` only. If mutation needed, provide explicit copy-on-write or transformation mechanism. Finding 🟠 CORE-018 — consumer ownership permits mutation without explicit intent.

### SPSC Abstraction Isn't Actually Type-Enforced as SPSC

`RingBuffer<T>` is `unsafe impl<T: Send> Send, Sync` and can be wrapped in `Arc<RingBuffer<T>>` by arbitrary publishers/subscribers, flags prevent multiple simultaneous reservations but type doesn't establish exactly one producer, one consumer. So implementation is really closer to shared ring with serialized reservations than type-enforced SPSC channel. Should be `SpscChannel<T> { Producer<T>, Consumer<T> }` and keep ring private/internal.

### Cross-Crate Ownership Matrix

| Boundary | Current mechanism | Verification |
|----------|-------------------|--------------|
| node → core | dependency exists | 🟠 |
| node message → core message | duplicate types | 🔴 |
| HAL → core | conceptual compatibility | 🔴 |
| transport → core | independent serialization types | 🔴 |
| sim → core | dependency exists | 🟠 |
| DMA → Image | clone | 🔴 for zero-copy |
| transport → wire | Vec<u8> serialization | 🟠 |
| sim → runtime | incomplete integration | 🔴 |
| node → transport | not canonicalized | 🔴 |
| HAL → transport | no unified data plane | 🔴 |

### Most Important Architectural Refactor

Do not make `nros-core` canonical home for every application type. Instead `nros-types` → messages, timestamps, geometry, identifiers and `nros-core` → channel, executor primitives, ownership, synchronization. Then `nros-node` = runtime/application API, `nros-transport` = wire/network implementation, `nros-hal` = device implementation, `nros-sim` = simulation implementation. Dependency graph: `nros-types / \ / \ ▼ ▼ ▼ nros-core nros-hal transport │ │ │ └────┬────┴────┬─────┘ ▼ ▼ nros-node nros-sim \ / \ / nros`. Important property: No subsystem defines own incompatible version of shared message.

---

## Pass 19 — Top-level Runtime, CLI, Macros, and Studio: System vs Collection of Subsystems

Pass confirms concern from Pass 18: **Current NROS branch is much closer to well-documented/scaffolded platform prototype than to unified executable robotics operating system.** Evidence unusually explicit in source itself.

### 1. Top-level nros crate is explicitly facade

Top-level crate describes itself as “NROS facade crate — aggregates all core crates + macros” and says macros currently passthrough and real code generation future work. That is decisive architectural evidence. Crate re-exports `nros-core`, `nros-node`, `nros-hal`, `nros-transport`, `nros-distributed`, `nros-sim`, `nros-studio`, `nros-cli` but aggregation is not integration.

### 2. nros::init() is not runtime initialization

Actual implementation `pub fn init() { println!("[NROS {}] Initialized (facade v{})", NROS_VERSION, VERSION); }` — source explicitly says real would initialize scheduler, HAL, logging, etc. Therefore claim `NROS runtime initialized` actual behavior `print initialization message` → Verdict 🔴 Runtime initialization: SCAFFOLDED

### 3. nros::spin() is explicitly placeholder

Implementation `pub fn spin<T>(_node: T) { println!("[NROS] Spinning node (placeholder) — real would start scheduler event loop"); }` — arguably single most important finding. Robotics runtime needs `spin(node) → executor → wait → wake → callback → deadline accounting → repeat`, current behavior `spin(node) → println! → return`. Therefore **no demonstrated NROS executor at public facade level**.

### 4. This invalidates several downstream claims

Without actual executor: `#[nros::node]` cannot mean node registered, node scheduled, callbacks dispatched, deadlines enforced. Likewise `nros::spin(node)` cannot mean runtime event loop active. Distinction needs to be reflected throughout documentation.

### 5. Macro system confirms it

Macro crate extraordinarily clear: “SCAFFOLDED — provides #[nros::node] etc as no-op passthrough” and says real implementation would generate lifecycle, parameter handling, wiring, QoS, etc. Actual `node` macro does parse struct then return same struct, no runtime machinery generated.

### 6. #[subscribe] does nothing

Implementation effectively `pub fn subscribe(attr, item) -> TokenStream { let _ = attr; item }` — does not create subscription registration, callback, topic lookup, type checking, QoS, simply leaves annotated item unchanged. Verdict 🔴 NROS-DSL semantics: NOT IMPLEMENTED

### 7. #[publish] is also no-op

Same pattern `#[publish(...)] → original item unchanged` — facade presents sophisticated declarative programming model that compiler currently does not implement, acceptable as scaffolding provided every external claim calls it scaffolding.

### 8. Problem not that macros are unfinished

Unfinished macros normal, architectural problem is `nros-core` already contains real low-level channel implementation, while `nros-node` contains node abstractions, and `nros-macros` contains proposed declarative API — but no demonstrated bridge `macro → generated node metadata → registration → executor → core channel` — missing runtime spine.

### 9. Correct architecture for macro layer

Eventually `#[nros::node] struct Controller { #[subscribe(topic="/cmd_vel")] cmd: Subscriber<Twist>, #[publish(topic="/motor")] motor: Publisher<MotorCommand>, }` should generate `ControllerMetadata { subscriptions, publications, parameters, callback descriptors, priorities, deadlines }` → Executor → NodeRuntime. Current macros generate none.

### 10. CLI is also more facade than runtime

CLI entry point advertises `nros init, build, run, topic, record, replay, analyze, profile, fleet, migrate` but actual command implementation needs to be classified independently. CLI command existing in enum is not evidence underlying subsystem exists.

### 11. nros init is actually one of stronger pieces

Project initializer really does create directories, write nros.toml, write source, write launch configuration, write robot config, write README — legitimate implementation. Verdict 🟢 Project scaffolding: implemented

### 12. But generated project deliberately doesn't use NROS

Generated Cargo.toml contains comments `# nros-core = ... # nros-node = ...` and generated sample node is ordinary Rust. Source explicitly says “compiles without external NROS macros” and describes full NROS API as future functionality. Therefore `nros init` currently proves `NROS can generate a compilable Rust project` but not `NROS can generate a working NROS application`. Verdict 🟢 generator 🔴 NROS application integration

### 13. This is actually good P0 remediation

Repository previously identified generated-project build problem, current strategy fixes it by generating standalone Rust program — technically sound for P0: generated project must compile, but changes semantics of generated NROS project into generated Rust project prepared for future NROS integration. Distinction should be explicit.

### 14. CLI run command therefore needs special scrutiny

Help says `nros run --inspect` and suggests NROS Studio live graph, but if `nros::spin()` is only print-and-return placeholder, then runtime cannot currently provide genuine live node execution graph. So chain `nros run → node → runtime → Studio` is broken at runtime layer.

### 15. NROS Studio exposes even stronger example

Studio source contains two providers `DemoDataProvider, LiveNrosDataProvider` — good architecture for distinguishing evidence. Demo provider explicitly marked SIMULATED — excellent. But supposedly live provider is itself still scaffolded.

### 16. LiveNrosDataProvider isn't live

Its comments say it would collect `nros-core PerformanceStats, nros-node ExecutionStats, OS metrics` but actual implementation delegates nodes/topics to `DemoDataProvider` and generates synthetic metric values. Therefore `LiveNrosDataProvider` is actually SCAFFOLDED LIVE PROVIDER not live telemetry. More serious: `is_simulated()` returns false while provider's own documentation says “Currently still synthetic but labeled as real path and scaffolded” — real audit finding 🔴 STUDIO-001 — telemetry provenance misclassification. System exposes `is_simulated() == false` for data source that actually produces synthetic metrics — can create false confidence downstream. Consider future dashboard: Live metrics Latency: 2.4 μs Throughput: 500 kmsg/s CPU: 38% — if provider reports `is_simulated() == false`, UI may present those numbers as measured runtime data, but they are generated constants/synthetic values. Violates repository's otherwise good evidence taxonomy. Required fix: provider should have explicit state `enum TelemetryProvenance { Simulated, Recorded, Live }` then `DemoDataProvider → Simulated, ReplayProvider → Recorded, LiveNrosDataProvider → Live only after actual integration` and UI should display provenance. Never infer provenance from boolean.

### 17-19 not repeated for brevity but include JSON manual generation fragility, Mutex in Studio, clean system decomposition Studio observability/debug vs Runtime control/data plane, etc.

### 20. Studio architecture issues

- JSON manually generated fragile, should use serde_json typed serialization — P1 robustness issue
- State uses `Arc, Mutex` reasonable for HTTP/SSE dashboard but must not be confused with realtime execution plane, Studio should live entirely in CONTROL/OBSERVABILITY PLANE and never sit on callback critical path

### 21-22 Clean system decomposition

NROS should explicitly have `NROS Studio (observability/debug)` → telemetry API → `NROS Runtime` with Executor, Node API, Channels, HAL/Sim, Transport. Studio observes runtime, must not pretend to be runtime.

### 23. New critical finding: there is no runtime spine

Branch has real channel primitive YES, real node abstractions YES, real HAL abstractions YES, real transport abstractions YES, real simulation engine YES, CLI command model YES, Studio HTTP/backend YES, macro declarations YES. But unified executor NO, node registration NO, macro → runtime wiring NO, runtime → channel wiring NO, runtime → HAL lifecycle NO, runtime → simulator backend NO, runtime → Studio telemetry NO. Therefore missing component is not another library, it is **the NROS Runtime Kernel**.

### 24. What runtime kernel must own

Minimal real implementation should own: `Runtime { Executor, NodeRegistry, TopicRegistry, ServiceRegistry, TimerRegistry, LifecycleManager, Scheduler, Clock, Telemetry, ShutdownCoordinator }` Then `nros::init()` constructs it, `nros::spin()` runs it, and `#[nros::node]` registers components with it.

### 25. Minimal runtime state machine

Node should have Created → Configured → Registered → Inactive → Active → Stopping → Stopped, runtime owns transitions, current `LifecycleNode` abstractions not yet equivalent to runtime-enforced lifecycle.

### 26. Minimal executor model

First production-worthy executor does not need to be distributed or heterogeneous. Start with single-process, single-host, SPSC channels, bounded queues, deterministic executor. For example Executor with ready queue, timers, subscriptions, shutdown. Then prove it. Only afterward add multi-thread RT priority distributed GPU/NPU.

### 27. This would immediately unlock macros

Once runtime exists: `#[nros::node]` can generate `NodeDescriptor` and `#[subscribe]` generates `SubscriptionDescriptor<T>` while `#[publish]` generates `PublisherDescriptor<T>`, runtime can then consume those descriptors. Much safer than having macros directly manipulate global runtime state.

### 28. Macro-generated metadata should be declarative

Recommended: struct Controller { cmd_vel: Subscriber<Twist>, motor: Publisher<MotorCommand> } → Macro output ControllerSpec { subscriptions: ["/cmd_vel"], publications: ["/motor"], callbacks: [...], parameters: [...] } → Then runtime performs registration. Makes architecture testable without proc-macro magic.

### 29. This also enables static graph validation

CLI already advertises `nros check --graph`, generated launch files already contain graph declarations, but graph checker should operate against same canonical runtime descriptors. Desired flow: macro → NodeSpec → GraphCompiler → validation → RuntimePlan → Executor. Currently these concepts exist largely as separate scaffolding.

### 30. Runtime should become single source of truth

Currently Studio → synthetic graph, CLI → command model, launch YAML → graph model, node crate → node model, transport → topic model — multiple representations. Instead RuntimeGraph should be canonical, then CLI, Studio, recording, debugger, static analyzer all consume same graph representation.

### 31. This solves Studio problem too

Once runtime owns NodeRegistry, TopicRegistry, PerformanceStats, ExecutionStats, Studio becomes Runtime → TelemetrySnapshot → Studio and no longer needs synthetic fake runtime state.

### 32. Evidence taxonomy should become machine-readable

Repository already uses terms IMPLEMENTED, SCAFFOLDED, SIMULATED — valuable, formalize enum `ImplementationStatus { Implemented, Scaffolded, Simulated, Planned }` and attach to subsystem capabilities: NROS runtime executor: Scaffolded, channel: Implemented, DMA: Simulated, telemetry: Scaffolded, macros: Scaffolded. Prevents documentation drift.

### 33. Updated system maturity matrix

| Subsystem | Current state |
|-----------|---------------|
| Core ring/channel | 🟠 real but unsafe proof incomplete |
| Node API | 🟠 substantial prototype |
| Executor | 🔴 absent |
| Runtime initialization | 🔴 placeholder |
| spin() | 🔴 placeholder |
| Proc macros | 🔴 passthrough |
| Project generator | 🟢 real |
| CLI command model | 🟠 real API/scaffold |
| CLI runtime integration | 🔴 incomplete |
| Transport | 🟠 partial |
| HAL | 🟠 simulated/scaffolded |
| Simulator | 🟠 substantial isolated subsystem |
| Studio UI/backend | 🟠 substantial |
| Live telemetry | 🔴 scaffolded |
| Distributed runtime | 🔴 not demonstrated |
| Real DMA | 🔴 absent |
| Canonical message model | 🔴 absent |

### 34. Overall maturity split into two numbers

Subsystem sophistication ~7/10, System integration ~3/10 — distinction explains repository accurately.

### 35. Revised claim taxonomy

Defensible:
- NROS has substantial Rust robotics platform prototype.
- NROS has functioning project generator.
- NROS has substantial SPSC/zero-copy channel prototype.
- NROS has simulation, HAL, transport, CLI, Studio subsystem prototypes.

Not yet defensible as verified implementation claims:
- NROS is complete robotics OS.
- `nros::spin()` runs NROS scheduler.
- `#[nros::node]` generates runtime integration.
- NROS Studio displays live NROS telemetry.
- NROS performs hardware DMA.
- NROS provides verified realtime execution.

### 36. Immediate P0/P1 roadmap

**P0 — Runtime spine:** Implement Runtime, Executor, NodeRegistry, TopicRegistry, init(), spin(), shutdown()

**P0 — Core safety:** Close WriteGuard initialization, ReadGuard DerefMut, Loom, Miri, Send/Sync proof

**P1 — Canonical types:** Create nros-types and remove duplicate Twist, Vector3, Timestamp

**P1 — Macro integration:** Generate metadata, not magic

**P1 — Runtime telemetry:** Connect Studio to actual runtime counters

**P1 — Canonical vertical slice:** Twist → Publisher → Executor → Subscriber → Controller → MotorCommand → Simulation

### 37. Decisive acceptance test

Strongest single test we can add: `cargo test --test canonical_runtime_slice` which starts actual NROS runtime and verifies: initialize runtime, register node, register publisher/subscriber, publish Twist, executor wakes callback, callback produces MotorCommand, simulator consumes command, state changes, telemetry records execution, Studio provider sees actual telemetry, shutdown completes. If passes without mocks replacing runtime components, NROS crosses major architectural threshold.

### 38. Final verdict — Pass 19

Repository is not merely collection of random stubs. Contains several serious subsystem prototypes. But top-level evidence now makes missing piece undeniable: NROS facade → Core, Node, HAL → Transport, Sim, Distributed → ??? Runtime Kernel is central missing artifact. Until runtime kernel exists, correct status for NROS is **Advanced, extensively scaffolded robotics-platform prototype — not yet an integrated operating system/runtime.** And most efficient next engineering move is not more feature implementation, but to build smallest real runtime kernel and use it to connect already-existing core, node, simulator, transport, Studio components into one verified vertical slice.

---

## Appendix — Full Verification Summary

See `AUDIT.md` for Pass 1-7 and `EVIDENCE_REGISTRY.md` for taxonomy. This file extends audit with Pass 13-19 covering transport/HAL boundary zero-copy claim, vertical-slice feasibility, transport contracts, verification gates (CI, Miri/Loom, reproducibility, evidence closure), core safety matrix branch-level source verification, cross-crate ownership-flow audit, top-level runtime facade vs system.

