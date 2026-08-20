# Crate Reference

> **Status:** Repository-grounded reference.
>
> Inventory verified against `Cargo.toml` and the crate manifests on `arena/documentation-rewrite`.

## 1. Workspace inventory

The NROS workspace currently declares **12 crates**:

| Crate | Role | Kind | Current dependency notes |
|---|---|---|---|
| `nros-types` | Canonical data types | Library | No dependencies |
| `nros-core` | Core IPC/runtime primitives | Library + demo binary | Depends on `nros-types` |
| `nros-node` | Node/example lifecycle layer | Library + demo binary | Depends on `nros-core`, `nros-types` |
| `nros-hal` | Hardware abstraction layer | Library + demo binary | Currently local domain types; no workspace dependencies |
| `nros-transport` | Network transport | Library + demo binary | Own wire types; optional compression/checksum dependencies |
| `nros-distributed` | Distributed coordination | Library + demo binary | No workspace dependencies currently declared |
| `nros-cli` | CLI tooling | Library + binaries | No workspace dependencies currently declared |
| `nros-sim` | Simulation engine | Library + demo binary | Own simulation domain types; no workspace dependencies currently declared |
| `nros-studio` | Monitoring/visualization | Library + binary | No workspace dependencies currently declared |
| `nros-macros` | Procedural macros | Proc-macro library | `proc-macro2`, `quote`, `syn` |
| `nros` | Public facade | Library | Aggregates the NROS workspace crates |
| `nros-audit` | Documentation/claim audit tooling | Binary | No dependencies currently declared |

The workspace uses Cargo resolver 2 and workspace version `0.1.0`. fileciteturn94file0

## 2. Crate details

### `nros-types`

Canonical shared types such as `Twist`, `Odometry`, `Vector3`, `Timestamp`, `MonotonicTimestamp`, `MotorCommand`, and `PointCloud` are identified by the manifest as the crate's intended domain. fileciteturn95file0

This crate currently has no declared dependencies.

### `nros-core`

Provides core IPC-oriented functionality, with the manifest describing lock-free/zero-copy IPC and ring-buffer/pub-sub functionality. It depends on `nros-types` and also exposes the `nros-core-demo` binary. fileciteturn96file0

Performance descriptions in the manifest are descriptive targets/claims and should not be interpreted as verified latency guarantees without corresponding benchmark evidence.

### `nros-node`

Provides the node/example layer, with manifest-level responsibilities including lifecycle, parameters, velocity-control, safety, and deadline monitoring. It depends on `nros-core` and `nros-types` and exposes `nros-node-demo`. fileciteturn97file0

### `nros-hal`

Provides the hardware-abstraction layer. Its manifest describes sensor interfaces, DMA-oriented data paths, synchronization, and hardware triggers. It currently declares no workspace dependencies and explicitly notes that local domain types remain pending canonical migration. fileciteturn98file0

### `nros-transport`

Provides network transport functionality described by the manifest as UDP/TCP, serialization, compression, discovery, and multicast. It currently maintains its own wire types. Optional features are `real-compression` and `real-checksum`, backed by `lz4_flex` and `crc32fast` respectively. fileciteturn99file0

### `nros-distributed`

Provides distributed coordination functionality described by the manifest as leader election, replicated state, task scheduling, and fleet coordination. It currently declares no workspace dependencies. fileciteturn100file0

### `nros-cli`

Provides CLI tooling for project management, multi-profile build, topic inspection, profiling, and fleet deployment. The package exposes the `nros` executable and a `nros-cli-demo` binary. fileciteturn101file0

### `nros-sim`

Provides the simulation-engine surface described by the manifest, including physics, rendering, sensor simulation, deterministic replay, and simulation/reality parity. It currently keeps local simulation domain types rather than depending on `nros-types`. fileciteturn102file0

### `nros-studio`

Provides the Studio monitoring/visualization surface, including the manifest's stated live monitoring, visualization, debugging, 3D transform, timeline, metrics, and remote-breakpoint responsibilities. It currently declares no workspace dependencies. fileciteturn103file0

### `nros-macros`

A procedural-macro crate. Its manifest identifies the macro surface as scaffolded/no-op passthrough functionality and declares `proc-macro2`, `quote`, and `syn`. fileciteturn104file0

### `nros`

The facade crate aggregates the workspace's principal crates and is intended to support a unified `nros`/prelude-facing experience. Its manifest currently depends on the other ten NROS functional/macro crates and exposes `real-time` and `gpu-acceleration` feature names, both currently empty. fileciteturn105file0

### `nros-audit`

A standalone audit/claim-linting binary. Its manifest describes checks involving documentation claims, evidence registry data, workspace inventory, workflows, and benchmark artifacts. fileciteturn106file0

## 3. Dependency topology

The currently declared workspace relationships include:

```text
nros-types
    ↑
nros-core
    ↑
nros-node

nros
 ├── nros-types
 ├── nros-core
 ├── nros-node
 ├── nros-hal
 ├── nros-transport
 ├── nros-distributed
 ├── nros-cli
 ├── nros-sim
 ├── nros-studio
 └── nros-macros
```

The topology above represents **declared Cargo dependencies**, not proof that all intended runtime relationships are implemented.

## 4. Important integration boundary

Several crates currently maintain local domain/wire types instead of depending on `nros-types`, notably `nros-hal`, `nros-transport`, and `nros-sim`. Their manifests explicitly identify canonical migration as future work. fileciteturn98file0turn99file0turn102file0

Therefore documentation must not claim that `nros-types` is already the single implementation-level source of every domain type across the workspace.

## 5. Source of truth

The authoritative sources for this page are:

1. root `Cargo.toml` workspace membership;
2. each crate's `Cargo.toml`;
3. crate source for actual public modules/APIs;
4. executable tests and CI for behavioral evidence.

Manifest descriptions are useful orientation, but they are not by themselves proof that every advertised capability is implemented.

## 6. Verification boundary

```text
Crate listed in workspace
        ↓
Crate compiles
        ↓
API exists
        ↓
Behavior tested
        ↓
Integration verified
        ↓
Performance / production claim verified
```

These are independent states.

## 7. Related documentation

- [Reference Index](README.md)
- [API](api.md)
- [Configuration](configuration.md)
- [Environment](environment.md)
- [Architecture](../architecture/README.md)
- [Specifications](../specifications/README.md)
- [Verification](../verification/README.md)
