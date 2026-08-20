# NROS Repository-Level Verification — Branch `arena/01a0188d-nros`

> This document is a repository-level verification of `Abdus2023/NROS`, specifically the branch `arena/01a0188d-nros`, rather than treating its README claims as verified facts.

---

## Initial findings

### 1. Branch is real and structurally substantial

The requested branch exists:

`arena/01a0188d-nros`

The repository is public, active, unarchived, and uses `main` as its default branch. The branch currently contains:

- `README.md`
- `DESIGN.md`
- `COMPARISON.md`
- `Cargo.toml`
- `crates/`
- `implementations/`

The workspace declares **8 Rust crates**:
`nros-core nros-node nros-hal nros-transport nros-distributed nros-cli nros-sim nros-studio`

### 2. The architecture is ambitious

The README describes NROS as a robotics middleware/OS architecture centered around:
```
Application
    ↓
High-Level APIs / Tools
    ↓
Core Services
    ↓
Communication Substrate
    ↓
Microkernel / Scheduler
    ↓
HAL
```
The intended differentiators are:

- zero-copy IPC
- deterministic real-time scheduling
- hardware abstraction
- Rust memory safety
- distributed robotics
- simulation
- CLI tooling
- visual Studio/dashboard

That is considerably broader than merely implementing a ROS2 alternative transport layer.

### 3. Important verification warning: README claims ≠ demonstrated evidence

There is already a significant evidence distinction.

The README says:

> “Achieved: 6.2 μs mean latency, 780K msg/s throughput”

and marks all eight listed implementation artifacts as **Complete**.

However, the repository evidence I have inspected so far does **not yet justify treating those numbers as independently verified benchmark results**.

For example, `nros-core` contains an SPSC lock-free ring buffer and benchmark test, which is genuine implementation code rather than merely documentation.

But a benchmark embedded in a Rust test is not automatically equivalent to a reproducible, statistically rigorous robotics middleware benchmark.

That distinction will be central to the deeper audit.

# First technical finding: `nros-core`

The core implementation uses:

- `AtomicU64`
- `AtomicUsize`
- `Acquire`/`Release` ordering
- cache-line alignment
- manually allocated storage
- SPSC ring-buffer semantics
- RAII-style `ReservedSlot`
- direct mutable access to a reserved slot
- `Arc<RingBuffer<T>>`
- publisher/subscriber abstractions

The design is clearly attempting a genuine zero-copy producer/consumer path.

But there are several things I want to verify before accepting the implementation as production-safe.

### A. Unsafe memory lifecycle

The implementation manually performs:
```
alloc(Layout::array::<T>(capacity))
```
and later:
```
dealloc(...)
```
while individual `T` values are written using raw pointers.

That immediately raises a critical ownership question:

**Are initialized `T` objects properly dropped before the backing allocation is released?**

For `Twist`, this happens to be harmless because it is composed of primitive copyable fields. But `RingBuffer<T>` is generic, and the implementation claims a generic API.

For arbitrary `T`, the current destruction model needs much more scrutiny.

### B. `try_read()` lifetime semantics

The consumer receives:
```
Option<&T>
```
directly into the ring.

Then consumption is performed separately:
```
subscriber.consume()
```
This creates an important API invariant:

> The referenced object must not be overwritten while the consumer still holds the reference.

The type system does not obviously encode that protocol.

That is a potential soundness boundary that deserves dedicated concurrency tests.

### C. `consume()` is extremely permissive

The consumer can call `consume()` without first successfully receiving a message.

The API itself does not appear to enforce:
```
try_recv()
    ↓
use reference
    ↓
consume()
```
as a state machine.

That may be acceptable for an internal primitive, but it is dangerous if presented as a safe high-level abstraction.

### D. SPSC is not automatically MPSC/MPMC

The implementation explicitly identifies itself as:

**SPSC — Single Producer Single Consumer.**

Yet the surrounding NROS architecture is positioned as a general robotics middleware.

Therefore we need to establish exactly where:
```
1 producer → 1 consumer
```
is guaranteed and where NROS requires:
```
N producers M consumers
```
because simply wrapping the SPSC queue in `Arc` does **not** make it MPSC/MPMC-safe.

### E. Benchmark methodology needs auditing

The test uses a producer and consumer thread and records timestamps using `SystemTime`.

That is useful as a demonstration, but it is not enough to establish a robust real-time latency claim.

For a serious `<10 μs` claim we should distinguish:

- mean latency
- median
- p95
- p99
- p99.9
- maximum
- scheduler interference
- CPU affinity
- CPU frequency scaling
- thermal throttling
- OS scheduling
- NUMA
- cache topology
- allocation effects
- debug vs release
- compiler optimization
- message size
- queue depth
- producer/consumer CPU placement

The claimed **6.2 μs** therefore remains **repository-reported**, not yet independently verified.

---

# Architectural verification matrix

My current evidence status is:

| Area | Current assessment |
|------|-------------------|
| Repository exists | ✅ Verified |
| Requested branch exists | ✅ Verified |
| Rust workspace | ✅ Verified |
| 8 workspace crates | ✅ Verified |
| Core zero-copy implementation exists | ✅ Verified |
| SPSC lock-free mechanism exists | ✅ Verified |
| Publisher/subscriber abstraction exists | ✅ Verified |
| Benchmark code exists | ✅ Verified |
| 6.2 μs result | 🟡 Needs reproducible execution |
| 780K msg/s result | 🟡 Needs reproducible execution |
| Production-grade memory safety | 🔴 Not established yet |
| Real-time determinism | 🔴 Not established yet |
| 100 KHz RT claim | 🔴 Not established yet |
| ROS2 performance comparison | 🔴 Not independently established |
| ISO 26262 / IEC 61508 readiness | 🔴 Claim requires substantial evidence |
| Distributed/Raft correctness | ⏳ Not audited yet |
| HAL correctness | ⏳ Not audited yet |
| Network transport correctness | ⏳ Not audited yet |
| Simulation determinism | ⏳ Not audited yet |
| Studio architecture | ⏳ Not audited yet |
| CI/reproducibility | ⏳ Not audited yet |

## Most important conclusion so far

**NROS is not merely a documentation-only repository anymore.**

There is real Rust implementation behind the architecture, particularly in `nros-core`.

But the README currently mixes three different evidence levels:
```
DESIGN SPECIFICATION
        ↓
IMPLEMENTATION EXISTS
        ↓
PERFORMANCE / SAFETY CLAIM VERIFIED
```
Those are **not equivalent**, and the branch currently needs a systematic audit to determine how much of the third level is actually supported.

The next useful pass is to go **crate-by-crate**, then cross-check each implementation against `DESIGN.md`, `COMPARISON.md`, tests, unsafe code, dependencies, benchmarks, and actual GitHub Actions evidence—especially looking for places where the documentation claims more than the executable implementation proves.

---

## NROS deep audit — Pass 2: architecture vs implementation

The second pass reveals a more important pattern: **NROS currently has a credible prototype substrate, but the repository’s implementation maturity is materially below the guarantees described by `DESIGN.md`.**

The design document explicitly promises a preemptive RT scheduler, CPU affinity/NUMA awareness, memory pools, DMA coordination, compile-time graph validation, and other OS-level facilities.

The code inspected so far is instead primarily **library-level demonstrations and prototype implementations**.

### 1. The biggest architectural gap: “microkernel” vs library

`DESIGN.md` describes:

- NROS Microkernel/Scheduler
- preemptive priority scheduling
- CPU affinity
- NUMA awareness
- interrupt handling
- DMA coordination
- resource management
- RTOS-like execution guarantees

But the workspace manifest currently consists of eight ordinary Rust crates; there is no obvious dedicated kernel/RT scheduler crate in the workspace manifest.

That doesn't mean the architecture cannot eventually provide these capabilities, but it means the terminology needs to be separated:

**Current reality:** robotics middleware/runtime prototype.

**Design target:** native robotics operating system / microkernel-oriented runtime.

That distinction is important for the project's credibility.

# 2. `nros-node`: substantial prototype, but not the promised programming model

The node crate is real and significantly more than scaffolding.

It contains:

- lifecycle states
- lifecycle traits
- parameters
- runtime parameter validation
- `Twist`
- `MotorCommand`
- `Odometry`
- timing utilities
- node-related infrastructure

For example, lifecycle state is explicitly represented as:
```
Unconfigured
Inactive
Active
Finalized
```
and the implementation exposes lifecycle callbacks such as `on_configure`, `on_activate`, `on_deactivate`, `on_cleanup`, and `on_shutdown`.

That's good prototype architecture.

### But there is a major mismatch

The design document presents a programming model like:
```rust
#[nros::node]
struct VelocityController {
    #[subscribe(...)]
    cmd_vel: Subscriber<Twist>, 
    #[publish(...)]
    motor_pub: Publisher<MotorCmd>, 
    #[param(...)]
    max_speed: f64,
}
```
and:
```rust
#[callback(realtime = true, deadline_us = 1000)]
async fn on_cmd_vel(...)
```
The implementation inspected so far is **not that programming model**.

There is no demonstrated procedural macro system establishing those annotations as compile-time guarantees.

So this currently falls into:

> **Design/API specification exists → partial underlying primitives exist → promised ergonomic/compile-time layer is not yet demonstrated.**

That is a recurring pattern we should track throughout the repository.

# 3. Parameter validation is runtime, not compile-time

The node implementation has a useful `Parameter` abstraction.

It validates:

- parameter type
- numeric minimum
- numeric maximum

at runtime.

That's legitimate functionality.

But the design claims a considerably stronger model:

> compile-time bounds checking

and a message-definition system with units, ranges, versioning and hashes.

Those are fundamentally different guarantees.

### Evidence classification

| Capability | Evidence |
|------------|----------|
| Runtime parameter storage | ✅ Implemented |
| Runtime type validation | ✅ Implemented |
| Runtime range validation | ✅ Implemented |
| Compile-time parameter constraints | ❌ Not demonstrated |
| MDL compiler | ❌ Not demonstrated |
| Compile-time units | ❌ Not demonstrated |
| Compile-time graph validation | ❌ Not demonstrated |

This distinction should eventually be reflected in the project's implementation-status table.

# 4. `nros-core`: the unsafe boundary is the highest-priority audit target

The core ring buffer manually manages memory and exposes raw pointers.

That is reasonable for a high-performance primitive, but it means the project cannot simply rely on Rust's normal safety guarantees.

The critical questions are now:

### Ownership

Who owns each initialized `T`?

### Destruction

Who invokes `drop` on initialized elements?

### Abandonment

What happens when a `ReservedSlot` is dropped without `commit()`?

### Reuse

Can an old `&T` coexist with a producer overwriting the same slot?

### Concurrency

Are the `Send`/`Sync` implementations valid for **all** `T: Send`?

### Queue semantics

What happens if the consumer calls `consume()` incorrectly?

These need dedicated adversarial tests, not just happy-path tests.

# 5. SPSC semantics are an architectural constraint

This is particularly important.

The core implementation explicitly implements an:

**SPSC — Single Producer / Single Consumer**

ring.

That's a very good primitive for deterministic communication.

But NROS's broader design talks about:

- one-to-many
- multicast
- distributed systems
- network transport
- multiple nodes
- fleet management

Therefore the architecture needs an explicit topology model:
```
SPSC
  ↓
MPSC
  ↓
SPMC
  ↓
MPMC
  ↓
multicast
  ↓
distributed transport
```
Otherwise there is a risk of accidentally treating an excellent SPSC primitive as though it were the complete NROS communication substrate.

# 6. The benchmark claims need a strict evidence gate

The README reports:

**6.2 μs mean latency**

and:

**780K messages/sec**

as achieved values.

The repository does contain benchmark code, so the claim isn't fabricated merely from documentation.

But **benchmark code ≠ independently reproducible benchmark evidence**.

For NROS, I recommend adopting a strict evidence classification:
```
DESIGN
  ↓
IMPLEMENTED
  ↓
UNIT TESTED
  ↓
BENCHMARK EXECUTED
  ↓
REPRODUCIBLE
  ↓
INDEPENDENTLY VERIFIED
  ↓
SAFETY-RELEVANT CLAIM
```
The README currently jumps too quickly from implementation to achieved performance.

# 7. Another important issue: time source

Both the core and node implementations use:

`SystemTime::now()`

with Unix epoch timestamps.

For a serious real-time runtime, this deserves redesign.

There should be a clear distinction between:
```
wall clock
monotonic clock
steady clock
hardware timestamp
PTP synchronized clock
simulation clock
```
For latency measurement and deadline scheduling, monotonic/steady time is generally the relevant abstraction.

The design also talks about deterministic execution and safety-critical timing, making the clock abstraction a foundational component rather than an incidental utility.

# 8. NROS currently has three architectural layers that should be separated

My current model of the repository is:
```
┌──────────────────────────────────────┐
│         NROS Vision / OS             │
│  Microkernel / RT / HAL / Graph      │
└──────────────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────┐
│       NROS Middleware Prototype      │
│ IPC / Nodes / Transport / Services   │
└──────────────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────┐
│       Demonstration Artifacts        │
│ Benchmarks / demos / examples        │
└──────────────────────────────────────┘
```
The current repository has meaningful implementation in the middle layer.

The upper layer is substantially more aspirational.

The bottom layer is useful evidence of feasibility, but shouldn't automatically be presented as production readiness.

# 9. Preliminary maturity score

Based on the files inspected so far:

| Dimension | Assessment |
|-----------|-----------|
| Architectural vision | **9/10** |
| Repository organization | **8/10** |
| Rust implementation presence | **7/10** |
| Core IPC prototype | **7/10** |
| Node prototype | **7/10** |
| API completeness | **4/10** |
| Real-time implementation | **3/10** |
| Compile-time safety model | **3/10** |
| OS/kernel implementation | **2/10** |
| Benchmark evidence | **4/10** |
| Production readiness | **2–3/10** |
| Safety-certification readiness | **1–2/10** |

These are **audit ratings, not project quality judgments**. They measure how much of the claimed architecture is presently supported by inspectable implementation evidence.

## Highest-priority next investigation

The next pass should move downward through the remaining crates:
```
nros-hal
      ↓
nros-transport
      ↓
nros-distributed
      ↓
nros-cli
      ↓
nros-sim
      ↓
nros-studio
```
while simultaneously checking:
```
DESIGN.md requirement
        ↓
claimed feature
        ↓
source implementation
        ↓
tests
        ↓
benchmark
        ↓
CI execution
        ↓
actual evidence
```
The **HAL + transport + distributed layers** are especially important because they will tell us whether NROS is becoming a coherent robotics runtime or remains a collection of independently demonstrated prototypes.

---

## Pass 3 — HAL, Transport, and Distributed layer

This pass substantially changes the evidence picture.

The repository contains a lot of code, but several features are explicitly simulations/placeholders, and some of those placeholders sit directly underneath claims of zero-copy, DMA, compression, distributed consensus, and network reliability.

### 1. nros-hal: the “DMA zero-copy” implementation is explicitly simulated

The HAL has a sensible abstraction structure:

```
Sensor
 ├── Camera
 ├── LiDAR
 ├── IMU
 ├── GPS
 ├── Radar
 └── Ultrasonic
```

It defines:

- DeviceInfo
- SensorCapabilities
- SensorConfig
- trigger modes
- sensor traits
- image/point-cloud data structures
- camera and LiDAR drivers

That is a good middleware API foundation.

But the critical detail is here:

**DmaBuffer uses `Vec<u8>` rather than actual DMA memory.**

The source itself says that the real implementation would use mechanisms such as `memfd_create`, `mmap`, `DMA-BUF` attachment, and GPU-accessible memory.

So:

> NROS currently has a DMA abstraction, not a DMA implementation.

That's an important distinction.

### 2. The camera “zero-copy” path actually copies

This is the most concrete finding in this pass.

The camera capture code obtains the simulated DMA buffer and then does:

```
buf.data.clone()
```

The source itself acknowledges that this is a safety/demo copy and that real NROS would expose a zero-copy view.

Therefore the actual execution path is:

```
simulated DMA buffer
        ↓
Vec<u8>
        ↓
CLONE
        ↓
Image
```

not:

```
camera DMA
     ↓
shared DMA-BUF
     ↓
consumer
```

Evidence status:

| HAL capability | Status |
|----------------|--------|
| Unified sensor trait | ✅ |
| Device metadata | ✅ |
| Sensor configuration | ✅ |
| Trigger abstraction | ✅ |
| Camera prototype | ✅ |
| LiDAR prototype | ✅ |
| DMA abstraction | ✅ |
| Real DMA | ❌ |
| DMA-BUF | ❌ |
| V4L2 integration | ❌ |
| GPU memory sharing | ❌ |
| Camera zero-copy | ❌ |
| Actual hardware driver | ❌ |

This should be classified as SIMULATED, not COMPLETE.

### 3. Transport layer has an even larger evidence problem

The transport implementation claims to demonstrate:

- UDP
- TCP
- serialization
- discovery
- compression
- multicast
- QoS

UDP itself is genuinely implemented using `UdpSocket`.

But several supposedly advanced features are placeholders.

#### 3.1 Compression is not compression

The source describes the compression engine as:

> “Simplified compression — real: lz4::compress”

and the implementation simply prefixes the original data with a flag.

So `compress(data)` currently produces approximately:

```
[1] + data
```

No LZ4 compression occurs.

Yet the code reports:

```
estimated_ratio() -> 0.6
```

which represents an assumed 40% saving.

That is not measured compression.

**Classification: Compression — ❌ placeholder, not ✅ implemented**

### 4. Checksum is generated but apparently not verified

The transport header contains a checksum field.

The sender computes a checksum-like value via `header.with_checksum(&final_payload)`, but the receiver's visible validation path validates:

- magic
- protocol version

and does not appear to verify the payload checksum.

This means corruption detection is incomplete.

A proper implementation should do:

```
receive
 ↓
validate header
 ↓
validate payload length
 ↓
verify checksum/MAC
 ↓
decompress
 ↓
deserialize
```

### 5. “Zero-copy network serialization” is not implemented

The transport documentation describes a FlatBuffers-style zero-copy model.

But the implementation does:

```
Vec<u8>
  ↓
serialize()
  ↓
payload Vec
  ↓
packet Vec
  ↓
UDP send
```

and on reception:

```
UDP buffer
  ↓
payload slice
  ↓
decompression Vec
  ↓
deserialize()
  ↓
new T
```

`LargePayload::deserialize()` explicitly creates a new `Vec<u8>`.

Therefore:

> The transport layer is serialized-copy based, not zero-copy.

### 6. Multicast is currently a stub

The transport API contains `multicast_group(group, ttl)` but the implementation only prints a message.

It does not actually perform multicast socket configuration.

**Multicast: ❌ simulated rather than implemented.**

### 7. Distributed layer: this is the most serious discrepancy so far

The crate calls itself a distributed computing system and advertises:

- leader election
- Raft-like consensus
- distributed state
- task distribution
- fleet coordination.

There are indeed structures representing these concepts.

But the source explicitly reveals that critical operations are simulated.

#### 7.1 It is not Raft

`LeaderElection` contains:

- current term
- candidate/follower/leader roles
- votes
- peers
- heartbeat timers

That's a useful skeleton.

But `start_election()` does not actually perform Raft `RequestVote` RPCs.

Instead it calls `should_grant_vote(...)` which uses pseudo-random behavior `rand::random_bool(0.7)`.

Therefore:

> This is Raft-like state-machine scaffolding, not a Raft implementation.

### 8. Distributed replication is also a stub

`DistributedState::set()` stores the value locally and then calls `replicate(...)` but `replicate()` simply returns success.

The source comments explicitly say that a real implementation would use a consistent hash ring and Raft log entries.

So the current behavior is:

```
set()
 ↓
local HashMap
 ↓
version++
 ↓
replicate()
 ↓
Ok(())
```

There is no actual remote replication.

Consequently:

| Distributed feature | Status |
|---------------------|--------|
| Robot identity | ✅ |
| Node roles | ✅ |
| Election state | ✅ |
| Term tracking | ✅ |
| Peer registry | ✅ |
| Election simulation | ✅ |
| Real Raft RPC | ❌ |
| Raft log replication | ❌ |
| Commit index | ❌ |
| Majority persistence | ❌ |
| Split-brain prevention | ❌ |
| Distributed state replication | ❌ |
| Consistent hash abstraction | 🟡 |
| Fleet coordination | 🟡/❌ |

This is currently prototype simulation, not distributed systems infrastructure.

### 9. This exposes a systematic repository-status problem

HAL, Transport, Distributed all show same pattern:

- HAL: Real hardware → SIMULATED
- Transport: Real compression / zero-copy / multicast → PARTIAL / SIMULATED
- Distributed: Real Raft / replication → SIMULATED

The project needs a formal implementation-status taxonomy. I recommend:

```
SPECIFIED
SCAFFOLDED
IMPLEMENTED
TESTED
BENCHMARKED
INTEGRATION-TESTED
HARDWARE-VALIDATED
PRODUCTION-READY
SAFETY-QUALIFIABLE
```

Every feature in README should carry one of these states.

### 10. Revised maturity assessment after this pass

| Layer | Maturity |
|-------|----------|
| Architecture/design | 9/10 |
| Core IPC prototype | 7/10 |
| Node runtime prototype | 6–7/10 |
| HAL abstraction | 6/10 |
| Real hardware integration | 1–2/10 |
| Network transport | 5/10 |
| Network zero-copy | 2/10 |
| Compression | 1/10 |
| Multicast | 1/10 |
| Distributed state | 2–3/10 |
| Raft | 1–2/10 |
| RTOS/kernel | 2/10 |
| Production readiness | 2–3/10 |

### 11. The repository is still valuable

There is a coherent architectural prototype here. The strongest part is the separation of concerns: `nros-core`, `nros-node`, `nros-hal`, `nros-transport`, `nros-distributed`, `nros-cli`, `nros-sim`, `nros-studio` and abstractions converging around common robotics runtime model. The problem is claim/evidence alignment, not lack of engineering direction.

### 12. Current audit verdict

> Architecture-complete prototype / implementation-in-progress — NOT feature-complete NROS OS.

More specifically:

```
Architecture                 █████████░  90%
Core prototype               ███████░░░  70%
Node middleware              ██████░░░░  65%
HAL abstraction              ██████░░░░  60%
Actual hardware              ██░░░░░░░░  20%
Transport                    █████░░░░░  50%
Distributed systems          ██░░░░░░░░  20%
Real-time kernel             ██░░░░░░░░  20%
Verification evidence        ███░░░░░░░  30%
Production readiness         ██░░░░░░░░  20%
```

---

## Pass 4 — Simulation, Studio, and CLI

This pass exposes another important layer: **the tooling is real, but much of the “live” data is currently synthetic.**

### 1. `nros-sim`: this is the strongest of the three

The simulator contains genuine mathematical/physics infrastructure:

- `Vector3`, `Quaternion`, `Transform`, `RigidBody`, collision shapes, entities, fixed timestep, accumulated simulation time, entity management

This is considerably more substantial than a simple mock.

**Fixed-step simulation** is a good design choice for deterministic simulation.

### 2. “Bullet integration” is not yet established

The simulator's documentation says “Physics engine integration (Bullet)” and code describes rigid bodies as being “per Bullet engine.” But inspected implementation contains its own physics structures and integration machinery. I have **not yet found evidence that the actual Bullet engine is being invoked**.

So correct classification: **Physics model: implemented, Bullet backend: not yet proven**

### 3. Simulation/reality parity remains a design target

The README/design language promises `SIM ↕ REAL` with shared node/message interfaces. The simulator's data structures are compatible in spirit, but that alone does not establish parity.

A strong parity test would need same node, same message types, same timing contract, same parameters, same control loop → SIM result ≈ REAL result and ideally automated tests proving both paths satisfy same interface.

### 4. `nros-studio`: visually impressive architecture, but telemetry is synthetic

`StudioState::new()` creates hard-coded nodes:

```
velocity_controller
camera_driver
lidar_processor
path_planner
```

with fixed metrics such as CPU, memory, rate, priority and deadline misses. It also hard-codes topics `/cmd_vel`, `/odom`, `/camera/image`, `/scan` with latency numbers `avg_us: 5.2 p99_us: 12.1 max_us: 18.7`.

These are **demo fixtures**, not measured runtime telemetry.

### 5. The metrics endpoint explicitly generates fake metrics

`to_metric_json()` computes latency, throughput, CPU and memory using `SystemTime` + `pseudo_rand()` rather than collecting from running NROS nodes.

So dashboard metrics (Latency, Throughput, CPU, Memory, Deadline misses) is currently a **synthetic observability demonstration**.

This means we must not use Studio screenshots/endpoint output as evidence for previously claimed 6.2 μs latency, 780K msg/s, etc. unless those values come from independently instrumented runtime.

**Critical audit finding:** The UI makes NROS look like it is monitoring a live robotics system, but the current backend is capable of generating that system entirely from predetermined/synthetic state.

### 6. Live parameter editing isn't actually connected to nodes

`update_param()` modifies `StudioState.nodes[node].params` and prints “hot-reload per §17.3 validation” but this changes Studio's in-memory representation only.

It does **not demonstrate that the underlying NROS node receives the parameter change**.

### 7. Studio is therefore “observability UI prototype”

| Studio feature | Evidence |
|----------------|----------|
| HTTP server | ✅ |
| Dashboard serving | ✅ |
| Node model | ✅ |
| Topic model | ✅ |
| TF model | ✅ |
| Metrics API | ✅ |
| SSE-style streaming architecture | 🟡 |
| Real node telemetry | ❌ |
| Real latency telemetry | ❌ |
| Real CPU/memory telemetry | ❌ |
| Live parameter control | ❌ |
| Real breakpoints | ❌ |
| Real graph introspection | ❌ |

The UI architecture is useful, but its current data source is synthetic.

### 8. `nros-cli`: good command architecture, but many commands are ahead of implementation

The CLI exposes broad command model: `init build run topic service node record replay analyze profile fleet migrate check`. This is strong interface design providing clear operational model: Develop → Build → Run → Observe → Record → Replay → Analyze → Profile → Deploy.

### 9. But `nros init` generates APIs that don't exist yet

The generated project uses `use nros::prelude::*;` and attributes `#[nros::node] #[subscribe] #[publish] #[param] #[callback]` But workspace does not establish corresponding `nros` facade/procedural-macro ecosystem.

Generated `Cargo.toml` also references packages such as `nros-stdlib`, `nros-navigation`, `nros-manipulation`, `nros-vision`, `nros-control`, `nros-balance`. Those are not demonstrated by the eight-crate workspace.

This means `nros init` can currently generate a project that **looks like the intended NROS API but may not actually compile**.

### 10. CLI has a “design surface” much larger than the runtime

The pattern is now very clear:

```
                 NROS
        │                     │
   Implemented           Advertised
        │                     │
        ▼                     ▼
   Rust primitives       Full ROS-like OS
   node abstractions     macro API
   SPSC IPC              real-time kernel
   UDP                   DMA
   simulator             Raft
   HTTP Studio           live telemetry
                          fleet/cloud
                          migration
```

### 11. A new critical category: “Executable fiction”

**Executable fiction:** code that successfully demonstrates an API or workflow but uses synthetic, placeholder, or simulated internals while presenting the interface as if it were backed by the production subsystem.

Examples:

- Studio: Live metrics → `pseudo_rand()`
- Compression: `compress()` → prefix flag + original data
- Raft: `RequestVote` → `random_bool(0.7)`
- Distributed replication: `replicate()` → `Ok(())`
- DMA: `DMA buffer` → `Vec<u8>`
- Multicast: `join multicast` → `println!()`

These are legitimate techniques for early prototypes. The problem is **status labeling**.

### 12. Repository-wide evidence level after four passes

| Architecture/specification | HIGH |
| Rust scaffolding | HIGH |
| Core IPC prototype | HIGH |
| Node abstractions | MEDIUM-HIGH |
| Simulation | MEDIUM |
| CLI | MEDIUM |
| Studio | LOW-MEDIUM |
| Hardware integration | LOW |
| Network advanced features | LOW |
| Distributed consensus | VERY LOW |
| Real-time kernel | VERY LOW |
| Production evidence | LOW |

### 13. Most important remediation

I would **not** recommend immediately adding more features. The repository first needs an **Evidence/Capability Registry** for every advertised feature.

### 14. Current red flags

- 🔴 P0 — Claims: README should not present simulated/placeholder functionality as completed production features.
- 🔴 P0 — Benchmark provenance: 6.2 μs / 780K figures need machine-generated benchmark artifacts tied to a commit and environment.
- 🔴 P0 — Generated project compilation: `nros init` must produce a project that actually builds.
- 🔴 P0 — Distributed layer: Do not call current election mechanism “Raft” without implementing actual protocol.
- 🟠 P1 — Studio: Separate `DemoDataProvider` from `LiveNrosDataProvider` so synthetic data cannot be confused with runtime telemetry.
- 🟠 P1 — HAL: Separate `SimulatedDmaBuffer` from `DmaBuf` and make distinction visible.
- 🟠 P1 — Transport: Separate `MockCompression` from `Lz4Compression` and actually verify checksums.

### 15. Updated overall verdict

> **NROS is a well-structured architectural prototype with meaningful Rust implementation, but the branch is not yet a complete Native Robotics Operating System and several “complete” artifacts are actually simulations or placeholders.**

---

## Pass 5 — Build integrity, dependency reality, CI, and repository topology

### 1. The workspace is extremely dependency-light

The root `Cargo.toml` declares exactly eight workspace members and workspace dependency section is effectively empty. That is striking because architecture describes functionality normally requiring substantial external infrastructure: networking, compression, serialization, discovery, simulation, hardware access, distributed consensus, visualization, real-time facilities.

Yet, for example, `nros-transport` depends only on `nros-core`. `nros-distributed` also depends only on `nros-core`. `nros-studio` likewise only depends on `nros-core`. And `nros-cli` only depends on `nros-core`.

### 2. This explains many of the placeholders discovered earlier

The dependency graph itself provides strong corroborating evidence for implementation status. Transport: A genuinely integrated implementation of LZ4, mDNS, FlatBuffers, advanced multicast would normally require either external crates or substantial in-house implementations. The manifest has neither. Distributed: There is no Raft dependency. That reinforces earlier conclusion.

### 3. The repository has two parallel implementation hierarchies

The branch contains:

- `crates/` and also `implementations/` — `nros-cli-tools`, `nros-core-implementation`, `nros-distributed-system`, `nros-hal-sensors`, `nros-network-transport`, `nros-node-example`, `nros-simulation-engine`

That creates ambiguity: Which directory is authoritative? If `crates/nros-transport` is product implementation while `implementations/nros-network-transport` is artifact, repository needs to state explicitly.

### 4. This is especially important for the “Artifact #N” language

### 5. CI evidence: currently there is no .github/workflows directory visible on this branch

I queried `.github/workflows` on requested branch and GitHub returned Not Found. Likewise, no root-level tests/ directory visible. Branch root currently exposes: `COMPARISON.md`, `Cargo.toml`, `DESIGN.md`, `README.md`, `crates/`, `implementations/`, `AUDIT.md` etc. Therefore cannot mark CI as PASS. Cannot claim `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` have been executed by GitHub Actions for this branch.

### 6. This invalidates a very common evidence shortcut

`#[test]` ≠ GitHub Actions PASS, `cargo test` locally ≠ reproducible CI verification, benchmark function exists ≠ 6.2 μs independently verified.

### 7. The CLI dependency graph reveals another likely build problem

Generated project from `nros init` uses `nros::prelude::*` and procedural attributes `#[nros::node]`, `#[subscribe]`, `#[publish]`, `#[param]`, `#[callback]` But actual CLI crate itself depends only on `nros-core` and workspace has no `nros` facade crate among its eight members. Therefore generated application is not currently demonstrated to be buildable.

### 8. The project versioning is also revealing

Workspace says version `0.1.0`, yet generated projects use version `1.0.0` and `nros_version = "0.1"` — three distinct version semantics, needs explicit versioning policy.

### 9. The generated dependencies are another integrity problem

Generator emits dependencies such as `nros-stdlib`, `nros-navigation`, `nros-manipulation`, `nros-vision`, `nros-control`, `nros-balance` but current workspace only contains eight crates. So generated mobile-base project could request `nros-navigation = "0.1"` without that package being part of repository. This should be treated as 🔴 P0 developer workflow defect unless those packages are intentionally external.

### 10. Repository maturity asymmetric

### 11. Revised verification matrix

| Claim / subsystem | Source | Test evidence | CI evidence | Verdict |
|-------------------|--------|---------------|-------------|---------|
| Workspace exists | ✅ | — | — | Verified |
| 8 crates | ✅ | — | — | Verified |
| Core SPSC | ✅ | ✅ source tests | ❌ | Implemented / CI-unverified |
| Node lifecycle | ✅ | ✅ | ❌ | Implemented / CI-unverified |
| HAL abstraction | ✅ | 🟡 | ❌ | Prototype |
| Real DMA | ❌ | ❌ | ❌ | Not implemented |
| UDP | ✅ | 🟡 | ❌ | Prototype |
| LZ4 | ❌ | ❌ | ❌ | Not implemented |
| Multicast | ❌ | ❌ | ❌ | Stub |
| Raft | ❌ | ❌ | ❌ | Simulation |
| Distributed replication | ❌ | ❌ | ❌ | Stub |
| Physics simulation | ✅ | 🟡 | ❌ | Prototype |
| Live Studio telemetry | ❌ | ❌ | ❌ | Synthetic |
| CLI architecture | ✅ | 🟡 | ❌ | Prototype |
| nros init generated app | 🟡 | ❌ | ❌ | Not proven buildable |
| 6.2 μs benchmark | 🟡 | ❌ | ❌ | Unverified |
| 780K msg/s | 🟡 | ❌ | ❌ | Unverified |

### 12. What I would change before adding features

**P0 — Establish a canonical build gate:** Add `.github/workflows/ci.yml` with at least `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`. Do not mark PASS until GitHub actually executes.

**P0 — Make nros init compile:** Golden test should be `nros init test_robot && cd test_robot && cargo check && cargo test`. If fails, generator cannot be called complete.

**P0 — Establish one canonical implementation:** Choose `crates/` or `implementations/` as authoritative source. If `implementations/` is archival/artifact-oriented, state explicitly.

**P1 — Introduce feature status:** Every major feature should carry SPECIFIED → SCAFFOLDED → IMPLEMENTED → TESTED → CI-VERIFIED → BENCHMARKED → HARDWARE-VERIFIED.

**P1 — Separate simulation from production APIs:** Examples `SimulatedDmaBuffer` vs `RealDmaBuffer`, `DemoTelemetryProvider` vs `RuntimeTelemetryProvider`, `MockCompression` vs `Lz4Compression`, `SimulatedElection` vs `RaftElection`.

### 13. Current branch verdict after Pass 5

Overall: 🟠 **ARCHITECTURALLY PROMISING / IMPLEMENTATION PROTOTYPE / VERIFICATION INCOMPLETE**

Not 🟢 production-ready and definitely not yet 🟢 verified real-time robotics OS.

---

## Pass 6 — Core safety audit: a much more serious finding

The central SPSC ring buffer is not merely “in need of more tests.” Its current safe API appears to permit memory-safety violations under ordinary safe Rust usage. That is a P0 blocker if `nros-core` is intended to be a safe foundational primitive.

### 1. P0 — Multiple outstanding reservations can alias the same slot

Producer does `try_reserve()` which reads `write_idx` and `read_idx` and returns `ReservedSlot`. Crucially, `try_reserve()` does not advance or otherwise reserve producer position. Write index advanced only when `commit()` happens. Therefore safe code can conceptually do:

```rust
let mut a = publisher.allocate().unwrap();
let mut b = publisher.allocate().unwrap();
let x = a.as_mut();
let y = b.as_mut();
```

Before either handle is committed, both reservations refer to same `write_idx` and same physical slot. API can create two simultaneous mutable references to same object. That's aliasing Rust ownership supposed to prevent.

**Severity: 🔴 P0 — potential unsoundness**

### 2. P0 — ReservedSlot abandonment doesn't actually roll back the reservation

Comment says “If commit() wasn't called, the write is abandoned. Slot remains uncommitted, producer can retry” but implementation doesn't maintain reservation state. Drop is empty. That means `allocate() → write partial data → drop handle` doesn't establish new state from which producer can safely recover. The slot was never atomically reserved.

The abstraction is closer to *calculate current write position* than *reserve unique queue position*.

### 3. P0 — RingBuffer<T> has an unsafe Sync implementation that is too weak

Code declares `unsafe impl<T: Send> Send for RingBuffer<T> {}` and `Sync`, but API returns `&T` from `try_read()` and `&mut T` through reservation. Sync impl only requires `T: Send` not `T: Sync`. Implementation relies on external SPSC discipline, but Rust type system doesn't enforce.

### 4. P0 — The consumer reference can outlive the queue slot's logical ownership

API does:
```rust
let received = subscriber.try_recv();
subscriber.consume();
```
and returns normal Rust reference `Option<&T>`. Once `consume()` advances read pointer, producer allowed to reuse slot. So sequence possible:
- consumer: `let x = try_recv();`
- consumer: `consume();`
- producer: writes next `T` into same slot
- consumer: continues using `x`

The lifetime of `x` is not tied to logical lifetime of queue entry. This is fundamental zero-copy queue API problem. Reference needs guard/token whose lifetime extends until slot released, e.g. `ReceiveGuard<'a, T>` with Deref and Drop → consume rather than exposing naked `&T` plus separately callable `consume()`.

### 5. P0 — Generic T destruction is missing

Backing memory is allocated using `alloc(Layout::array::<T>(capacity))` and freed using `dealloc(...)`. But initialized objects are never explicitly dropped. For `T = Twist` (copy-like primitives) not observable, but for `T = String`, `Vec<u8>`, `Box<_>`, `Arc<_>` can own external resources, queue can leak.

### 6. P0 — The queue has no explicit initialization bitmap/state

Allocation itself contains uninitialized memory. Yet implementation determines whether slot contains valid `T` entirely through atomic indices. That can work for carefully designed SPSC queue—but only if lifecycle invariants rigorously maintained. Current reservation API breaks assumption because multiple outstanding reservations possible. So memory-safety proof incomplete.

### 7. The benchmark is also a correctness gate, not merely a benchmark

Benchmark test contains `assert!(stats.avg_latency_us() < 10.0)`. This creates CI problem: performance benchmark embedded directly into `cargo test` means machine's scheduling characteristics can determine whether functional test suite passes. Performance benchmarking should be separated: `cargo test` → correctness only, `cargo bench` → performance, explicit benchmark environment.

### 8. The benchmark measures end-to-end scheduling noise

Benchmark uses `Timestamp::now()` and `SystemTime`, while source says real implementation should use `CLOCK_MONOTONIC`. So claimed latency affected by system clock, thread scheduling, OS preemption, CPU frequency, migration, cache state, contention and isn't deterministic IPC latency measurement. Furthermore benchmark doesn't pin producer and consumer to CPUs.

### 9. The benchmark itself is weak evidence for “zero-copy”

Benchmark constructs `Twist` directly inside reserved memory, which is good, but measured operation includes `Timestamp::now()` on every message. Therefore measured latency isn't purely reserve→write→commit→observe, also includes clock acquisition.

### 10. There is no backpressure policy

Benchmark explicitly says “real NROS would have backpressure policy”. Current behavior when queue full is loop forever `try_reserve()`. This is busy-spin. No demonstrated block, drop-oldest, drop-newest, deadline, priority, bounded wait, producer cancellation, overload telemetry policy. For robotics middleware, architectural requirement rather than implementation detail.

### 11. consume() itself has no state validation

Consumer can simply call `subscriber.consume();` without establishing that it successfully received a message first. So API permits `consume()`, `consume()`, `consume()` without corresponding messages. Underlying atomic index can advance beyond published region. Abstraction depends on caller discipline rather than encoding state machine.

### 12. The correct redesign is not “add more atomics”

API needs proper ownership model: ProducerHandle → reserve() → WriteGuard<T> &mut T → commit() → ConsumerHandle → receive() → ReadGuard<T> &T → Drop → released. Critical invariant: WriteGuard owns producer reservation, ReadGuard owns consumer reservation rather than exposing independent operations caller can misuse.

### 13. This changes the project priority order

Previously, CI at top. After source-level inspection, priority becomes:

- P0: Make nros-core sound before claiming safe Rust zero-copy IPC.
- P0: Add concurrency/adversarial tests: double reserve, abandoned reservation, reserve after full, wraparound reservation, drop_pending_message, drop_consumed_message, overwrite_requires_drop, read_guard_lifetime, producer_consumer_stress, queue_wraparound_stress, memory_reordering_stress, consume_without_receive, multiple_read_attempts, multiple_reservations, generic-type tests using String, Vec<u8>, Box<T>, DropCounter.
- P0: Establish Miri/loom-style verification where applicable.
- P0: Only then establish CI enforcing corrected invariants.

### 14. Revised NROS risk register

| ID | Finding | Severity | Current state |
|----|---------|----------|---------------|
| NROS-001 | Multiple outstanding producer reservations can alias | 🔴 P0 | Open |
| NROS-002 | Sync bound appears insufficient for safe shared API | 🔴 P0 | Open |
| NROS-003 | Read reference can outlive logical queue ownership | 🔴 P0 | Open |
| NROS-004 | Generic T destruction not performed | 🔴 P0 | Open |
| NROS-005 | Reservation cancellation has no state rollback | 🔴 P0 | Open |
| NROS-006 | consume() lacks state enforcement | 🔴 P1 | Open |
| NROS-007 | Benchmark uses SystemTime | 🟡 P1 | Open |
| NROS-008 | Benchmark embedded in test suite | 🟡 P1 | Open |
| NROS-009 | Busy-spin is sole demonstrated backpressure | 🟡 P1 | Open |
| NROS-010 | CI evidence absent on branch | 🔴 P0 | Open |
| NROS-011 | nros init API not demonstrated buildable | 🔴 P0 | Open |
| NROS-012 | Simulation/stub status mixed with completed claims | 🔴 P0 | Open |

### 15. This is now the key conclusion

The NROS core should not yet be advertised as a safe zero-copy IPC primitive. A more accurate description at this exact branch state is:

> Experimental SPSC zero-copy prototype using manually managed memory and unsafe synchronization primitives; safety invariants and lifecycle ownership are not yet formally established.

That is still valuable work. But distinction crucial because nros-core is foundation beneath Node → Transport → Distributed → Simulation → Studio → CLI. If core ownership model is unsound, every higher layer inherits risk.

#### Next pass: security/soundness sweep

The logical next step is to audit every unsafe, atomic ordering, raw pointer, allocation, thread boundary, and Send/Sync implementation across entire branch, then produce formal Safety & Concurrency Threat Model with Invariant → Unsafe boundary → Possible violation → Exploitability → Test required → Fix → Verification gate.

---

## Pass 7 — Correctness proof of nros-core: separating confirmed defects from hypotheses

Executive result: I would now classify nros-core as:

> 🔴 Unsafe experimental prototype — not yet suitable as a generally safe Rust zero-copy IPC abstraction.

There are three independent P0-level design problems:

1. producer reservations are not uniquely claimed;
2. the consumer exposes a reference whose validity isn't tied to slot ownership;
3. initialized T values are never destroyed.

There are also additional P1 issues around queue state validation, timing, and benchmark methodology.

### 1. Confirmed P0 — reservation is not actually a reservation

Implementation reads `write = write_idx.load(Relaxed)`, `read = read_idx.load(Acquire)`, checks `write - read >= capacity`, then `idx = write & (capacity-1)` and returns `ReservedSlot` containing `write_idx`. Nothing changes `write_idx` during `try_reserve()`. Only `commit()` does `store(write_idx+1, Release)`. Therefore:

```
try_reserve() → write=N → slot N
try_reserve() → write=N → slot N again
```

Before first commits. This is not merely theoretical multi-producer problem. It can happen with one producer calling `allocate()` twice.

```rust
let mut a = publisher.allocate().unwrap();
let mut b = publisher.allocate().unwrap();
a.as_mut();
b.as_mut();
```

Both handles point at same physical slot. Type system cannot protect caller because both operations are legal safe Rust.

**Verdict: NROS-CORE-001 — P0 — Producer reservation abstraction does not establish unique ownership of a queue slot.**

### 2. The SPSC label does not save this

SPSC means one producer thread, one consumer thread. It does not mean producer can only have one outstanding Rust reservation. That second property must be enforced by API or documented as mandatory invariant. Currently neither enforced nor represented in type system.

### 3. Confirmed P0 — published references can survive slot reuse

`try_read()` returns `Option<&T>` and `consume()` is independent. This permits:

```rust
let msg = subscriber.try_recv().unwrap();
subscriber.consume();
// later use(msg);
```

Rust borrow checker permits because `consume()` takes `&self`, not mutable borrow. Now queue considers slot available again. Producer can eventually write new `T` into same physical memory. Therefore same physical slot → old `&T` and new `&mut T` coexist. Producer's raw-pointer write is outside Rust's normal aliasing protections.

**Why this matters:** Old `&T` remains alive while producer overwrites object, potentially violating Rust's aliasing rules.

**Correct abstraction:** Consumer should receive ownership guard `ReadGuard<'a, T>` whose Drop releases slot, rather than `try_recv()` + separate `consume()`.

**Verdict: NROS-CORE-002 — P0**

### 4. Confirmed P0 — generic objects are leaked

Buffer allocates raw storage `alloc(Layout::array::<T>(capacity))` and on destruction only `dealloc(...)`. No iteration over initialized slots calling `ptr::drop_in_place(...)`. For `T = Twist` no heap-owned field, but `T = String`, `Vec<u8>`, `Box<_>`, `Arc<_>` can own external resources. Queue doesn't destroy those objects.

**Verdict: NROS-CORE-003 — P0**

### 5. This suggests an important design decision

NROS needs to decide which kind of IPC primitive it actually wants:

- Option A: typed Rust objects — must correctly support Drop, ownership, lifetimes, Send/Sync, initialization, aliasing, panic safety
- Option B: fixed-layout POD messages — constrain API to deliberately defined wire/message representation (fixed ABI layout → initialized bytes → validated message) much easier for shared memory and cross-process IPC
- Option C: both — best long-term: `nros-core` with `TypedRing<T>` (Rust ownership) and `RawMessageRing` (validated byte/message ABI)

### 6. Confirmed P1 — consume() has no corresponding receive token

API allows `subscriber.consume();` without preceding successful `try_recv()`. No stored state indicating message currently acquired. Public state machine effectively ANY → consume() → read_idx+1 rather than EMPTY → AVAILABLE → ACQUIRED → RELEASED.

### 7. Confirmed P1 — abandonment semantics are misleading

`ReservedSlot::Drop` is empty, comment says “producer can retry” but implementation doesn't have reservation state to roll back. Because `write_idx` never moved, slot simply remains current slot. Happens to allow retrying in some simple cases, but semantics problematic once multiple handles or partially initialized objects considered.

### 8. Confirmed P1 — publish_copy() uses raw initialization without explicit initialized-state model

`publish_copy()` does `ptr::write(handle.slot.ptr, msg); handle.commit();` appropriate for writing into uninitialized memory if and only if slot uniquely owned and guaranteed uninitialized. But current reservation implementation doesn't establish invariant robustly. So correctness of `ptr::write(...)` depends on broken reservation contract.

### 9. Send/Sync: refine previous finding

Implementation says `unsafe impl<T: Send> Send for RingBuffer<T> {}` and `Sync`. Important point not simply Sync should require T: Sync. Because NROS intentionally wants cross-thread ownership, exact bound depends on API's aliasing guarantees. Real problem: implementation has not established sound synchronization/aliasing contract justifying these unsafe trait implementations.

So record as **NROS-CORE-004 — P0 review required** rather than asserting simplistic T: Sync fix. Proper bound should emerge from redesigned ownership model.

### 10. Memory ordering deserves formal proof

Current uses Producer: `write_idx.load(Relaxed)`, `read_idx.load(Acquire)`, `write_idx.store(Release)`; Consumer: `read_idx.load(Relaxed)`, `write_idx.load(Acquire)`, `read_idx.store(Release)`. Resembles standard SPSC release/acquire structure. So would not label ordering itself incorrect yet. Bigger problem queue's ownership invariants broken around it.

### 11. Timestamp implementation has another correctness issue

`Timestamp::now()` uses `SystemTime::now()` against UNIX epoch. Source admits intended real implementation should use `CLOCK_MONOTONIC`. For latency measurement, wall-clock time wrong abstraction. NROS needs separate clock types: WallClock, MonotonicClock, SteadyClock, HardwareClock, SimulationClock and latency APIs should accept monotonic/steady clock.

### 12. Benchmark claim needs downgrading

Source says target `<10 μs latency`, `500K+ msg/s` and test named `benchmark_latency` with 100k-message workload. But not sufficient to establish published performance claim. Why? `Timestamp::now()` + OS scheduler + thread migration + CPU frequency + cache state + busy-spin + test harness all influence measurement. Also benchmark is under `#[test]`, so performance coupled to ordinary test suite.

Required separation: `cargo test` → correctness only, `cargo bench` → performance, benchmark artifact → CPU model, OS, compiler, commit, affinity, iterations, distribution. Only last artifact should support quantitative performance claim.

### 13. New test suite I would require

Before NROS-Core can graduate from experimental status:

- Ownership tests: double_reserve, abandoned_reservation, reserve_after_full, wraparound_reservation
- Lifetime tests: drop_pending_message, drop_consumed_message, overwrite_requires_drop, read_guard_lifetime
- Concurrency tests: producer_consumer_stress, queue_wraparound_stress, memory_reordering_stress
- API misuse tests: consume_without_receive, multiple_read_attempts, multiple_reservations
- Generic-type tests: use String, Vec<u8>, Box<T>, DropCounter not only Twist. DropCounter particularly useful to assert every initialized object destroyed exactly once.

### 14. The key invariant table

Redesigned core should explicitly document:

| Invariant | Required |
|-----------|----------|
| One producer reservation per slot | MUST |
| One consumer owner per slot | MUST |
| No &T after release | MUST |
| Initialized T dropped exactly once | MUST |
| Producer cannot overwrite acquired slot | MUST |
| Consumer cannot consume unacquired slot | MUST |
| Queue indices never move backwards | MUST |
| Wraparound is safe | MUST |
| Release/acquire ordering proven | MUST |
| Send/Sync justified by proof | MUST |
| Full-buffer behavior defined | MUST |

### 15. Updated risk register after re-verification

| ID | Finding | Priority | Confidence |
|----|---------|----------|------------|
| CORE-001 | try_reserve() doesn't uniquely reserve | 🔴 P0 | Confirmed |
| CORE-002 | &T can outlive queue ownership | 🔴 P0 | Confirmed |
| CORE-003 | Generic T destruction missing | 🔴 P0 | Confirmed |
| CORE-004 | Send/Sync unsafe-contract proof absent | 🔴 P0 | High |
| CORE-005 | consume() lacks ownership token | 🟠 P1 | Confirmed |
| CORE-006 | Abandonment isn't transactional | 🟠 P1 | Confirmed |
| CORE-007 | Wall clock used for latency | 🟠 P1 | Confirmed |
| CORE-008 | Benchmark mixed into tests | 🟠 P1 | Confirmed |
| CORE-009 | Backpressure = busy spin | 🟠 P1 | Confirmed |
| CORE-010 | CI verification absent | 🔴 P0 | Previously confirmed |

### 16. The remediation architecture

Refactor into:

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

And make queue's public API guard-based:

```
Producer
  │
  └── reserve()
         │
         ▼
    WriteGuard<T>
         │
       commit
         │
         ▼
    Consumer
         │
         └── receive()
                │
                ▼
           ReadGuard<T>
                │
              Drop
                │
                ▼
             released
```

### Bottom line

The important discovery isn't simply “there are bugs.” It's that NROS's central zero-copy abstraction currently relies on safety properties that are not encoded by its API. That becomes especially important because every higher-level claim—low latency, zero-copy messaging, deterministic execution, transport efficiency—depends on this primitive. So correct next milestone is no longer another feature crate. It is:

> NROS-Core v0.1 Safety Gate — ownership + lifetime + destruction + concurrency proof, followed by CI verification.

Only after that gate passes should project use nros-core as trusted foundation for HAL, transport, distributed execution, simulation, and Studio telemetry.

---

## Appendix: Full evidence taxonomy

For more details, see the preceding passes and the implementation files in `crates/` and `implementations/` directories. This audit was conducted on branch `arena/01a0188d-nros` at commit range including DESIGN.md v1.0 and subsequent implementation artifacts.
