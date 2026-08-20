# NROS Repository Representation — Canonical Architecture & Workspace Overview

This document provides a highly structured, authoritative representation of the `Abdus2023/NROS` repository. It outlines the directory hierarchy, workspace dependency tree, data flow pipelines, implementation entry points, and the mapping from conceptual specs to active code.

---

## 1. Directory Tree & Architecture Mapping

The repository organizes the 12 workspace crates under `crates/`, separate from archival/original single-file implementation demonstrations under `implementations/`.

```
NROS/ (Workspace Root)
├── Cargo.toml                  # Cargo workspace manifest (12 crates)
├── README.md                   # Project overview & roadmap
├── DESIGN.md                   # Full 25-section architecture specification
├── COMPARISON.md               # NROS vs ROS2 benchmark details & metrics
├── AUDIT.md                    # Core repository-level audit reports (Pass 1 to 7)
├── AUDIT_PASS_8_12.md          # State verification & core refactoring report
├── AUDIT_PASS_13_19.md         # Zero-copy, validation slices, and build-gate reports
├── AUDIT_PASS_20_23.md         # re-verification, branch integrity, and invariant reports
├── AUDIT_PASS_24.md            # Soundness patches & hard E0116/E0252/Ord-Eq compilation fixes
├── AUDIT_PASS_25.md            # SPSC endpoint locks & non-blocking framing accumulator report
├── AUDIT_PASS_26.md            # Fixed-timestep physics sub-stepping & equivalence principle IMU checks
├── EVIDENCE_REGISTRY.md        # Feature → Specification → Implementation → Status registry
│
├── .github/
│   └── workflows/
│       └── ci.yml              # CI verification pipeline (Fmt, Check, Tests, Miri, Linter)
│
├── crates/                     # ── Authoritative Current Runtime Implementations ──
│   ├── nros-types/             # Canonical domain messages, wall time, monotonic clocks, geometry
│   ├── nros-core/              # Zero-copy SPSC lock-free ring-buffer & priority-queue executor
│   ├── nros-node/              # Node middleware, lifecycle state machine, ParameterServer
│   ├── nros-hal/               # Hardware Abstraction Layer sensor synchronizer (Camera, LiDAR, IMU)
│   ├── nros-transport/         # Networking layer: UDP/TCP, service discovery, Lz4, sum-checksums
│   ├── nros-distributed/       # Raft election scaffolds, FNV-1a consistent hash shards, consensus
│   ├── nros-sim/               # Fixed-timestep physics sub-stepping & simulated sensor raycasting
│   ├── nros-studio/            # HTTP dashboard server, Three.js 3D TF rendering, metrics SSE stream
│   ├── nros-macros/            # Ergonomic procedural macro attribute wrappers (e.g. #[nros::node])
│   ├── nros/                   # Aggregate facade library & unified prelude exports
│   └── nros-audit/             # Structural safety-linter / CI doc-gate check tool
│
├── implementations/            # ── Archival Single-File Prototype Artifacts (§25) ──
│   ├── nros-core-implementation/
│   ├── nros-node-example/
│   ├── nros-hal-sensors/
│   ├── nros-network-transport/
│   ├── nros-distributed-system/
│   ├── nros-cli-tools/
│   └── nros-simulation-engine/
│
└── benchmarks/
    └── results.json            # Machine-generated statistical benchmark results
```

---

## 2. Workspace Dependency DAG (Directed Acyclic Graph)

The workspace enforces a strict, layered acyclic import flow. Higher-level abstractions aggregate components from lower-level core/message libraries, with `nros-types` acting as the absolute single source of truth for canonical models.

```
                  [ nros-cli ] (User Command Tool)
                       │
                       ▼
                  [   nros   ] (Aggregate Facade)
                  /    │     \
                 /     │      \
                ▼      ▼       ▼
         [ nros-node ] │  [ nros-macros ] (Procedural macros)
          /      │     │
         /       │     │
        ▼        ▼     ▼
  [ transport ] [ nros-core ] (SPSC Ring-Buffer / Real-Time Priority Executor)
        │              │
        \              /
         ▼            ▼
         [ nros-types ] (Canonical Messages, Vectors, Monotonic/Wall Clocks)
```

### Dependency Inventory Table

| Crate Name | Absolute Path | Direct Workspace Dependencies | Role & Target |
|:---|:---|:---|:---|
| `nros-types` | `crates/nros-types` | None | Geometry, Clocks, Twist, Odometry, Image |
| `nros-core` | `crates/nros-core` | `nros-types` | SPSC raw buffers, guards, scheduling priority |
| `nros-node` | `crates/nros-node` | `nros-types` | Lifecycle, runtime ParameterServer, timing metrics |
| `nros-macros` | `crates/nros-macros` | None | Attributes: `#[nros::node]`, `#[subscribe]`, `#[publish]` |
| `nros-hal` | `crates/nros-hal` | None | SensorSync, DMA buffers state machines |
| `nros-transport`| `crates/nros-transport`| None | Network UDP/TCP, service broadcast, frame packaging |
| `nros-distributed`| `crates/nros-distributed`| None | Raft simulation, FNV consistent shards |
| `nros-sim` | `crates/nros-sim` | None | Semi-implicit Euler integration, sensor raycasting |
| `nros-studio` | `crates/nros-studio` | None | SSE telemetry streamer, force layout, TF rendering |
| `nros` | `crates/nros` | `nros-types`, `nros-core`, `nros-node`, `nros-macros` | Facade aggregate for end-user imports |
| `nros-audit` | `crates/nros-audit` | None | Static safety linter, regression-testing runner |

---

## 3. Core Data Flow & Lifecycle Pipelines

### 3.1 Zero-Copy Publication Lifecycle (crates/nros-core)

NROS-Core ensures safe, double-reserve protected zero-copy publications using a guard-based type-state machine:

```
[Producer Thread]
       │
       │  1. try_reserve()
       ▼
 ┌────────────┐  Is "write_reserved" AtomicBool CAS false?
 │ WriteGuard │ ───────────────────────────────────────► [Yes] Allocates pointer offset
 └────────────┘                                            │
       │                                                   │
       │  2. write_value(T)                                 ▼
       ▼                                            Is "T" moved into guard?
 ┌────────────────────────┐                                │
 │ InitializedWriteGuard  │ ◄──────────────────────────────┘
 └────────────────────────┘
       │
       │  3. commit()
       ▼
 [Read Index Advanced] (Visible to Consumer thread via Release load order)
```

```
[Consumer Thread]
       │
       │  1. try_recv()
       ▼
 ┌───────────┐  Is "read_reserved" AtomicBool CAS false?
 │ ReadGuard │ ───────────────────────────────────────► [Yes] Yields immutable &T
 └───────────┘                                             │
       │                                                   │
       │  2. drop() (RAII cleanup)                         ▼
       ▼                                            Is "drop_in_place" run exactly once?
 [Read Index Incremented] ◄────────────────────────────────┘
```

---

## 4. Specification Entry Points Map

To eliminate evidence-drift, this directory map links the primary specification sections in `DESIGN.md` to their authoritative codebase locations:

| DESIGN.md Specification Section | Canonical Source Path | Key Structs / Modules |
|:---|:---|:---|
| **§3.1 Node Lifecycle** | `crates/nros-node/src/lib.rs` | `enum LifecycleState`, `trait LifecycleNode` |
| **§4.1 Real-Time Deadlines** | `crates/nros-node/src/lib.rs` | `struct ExecutionStats` (deadline tracking) |
| **§4.3 Thread Profiling** | `crates/nros-cli/src/lib.rs` | `struct Profiler` (synthetic profiling) |
| **§6 Hardware Integration** | `crates/nros-hal/src/lib.rs` | `struct SensorSynchronizer`, `trait DmaBufferTrait` |
| **§7.1 Command Line interface** | `crates/nros-cli/src/lib.rs` | `struct CLI`, `struct ProjectInitializer` |
| **§7.2 Studio Visualization** | `crates/nros-studio/src/lib.rs` | `struct StudioServer`, `trait DataProvider` |
| **§7.3 Simulation & Replay** | `crates/nros-sim/src/lib.rs` | `struct SimulationWorld`, `struct SimulatedCamera` |
| **§14.1 SPSC Ring-Buffer** | `crates/nros-core/src/lib.rs` | `struct RingBuffer<T>`, `struct ReadGuard` |
| **§14.3 Communication Bridging** | `crates/nros-transport/src/lib.rs`| `struct MessageHeader`, `struct UdpTransport` |
| **§17.1 Raft Leader Elections** | `crates/nros-distributed/src/lib.rs`| `struct LeaderElection`, `trait ElectionEngine` |
| **§17.3 Parameter Systems** | `crates/nros-node/src/lib.rs` | `struct Parameter`, `enum ParameterValue` |
