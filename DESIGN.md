# NROS Design

> **Status: Legacy design entry point.**
>
> This file is retained for historical traceability. It is no longer the authoritative location for the complete NROS architecture, API surface, implementation status, or verification status.

## Current documentation

Use the domain-oriented documentation system instead:

- [Concepts](docs/concepts/README.md)
- [Architecture](docs/architecture/README.md)
- [Specifications](docs/specifications/README.md)
- [Getting Started](docs/getting-started/README.md)
- [Reference](docs/reference/README.md)
- [Development](docs/development/README.md)
- [Verification](docs/verification/README.md)
- [Safety](docs/safety/README.md)
- [Operations](docs/operations/README.md)
- [Governance](docs/governance/README.md)
- [Migration](docs/migration/README.md)

## Why this document is legacy

The historical design mixed several different categories of information:

1. architectural intent;
2. proposed APIs and illustrative code;
3. implementation targets;
4. performance targets;
5. safety/compliance aspirations;
6. roadmap items.

Those categories are not equivalent evidence.

A code example in a design document does not establish that the API exists. A performance target does not establish a benchmark. A safety aspiration does not establish validation or certification.

## Status model

NROS documentation uses the following progression:

```text
PROPOSED
   ↓
SPECIFIED
   ↓
SCAFFOLDED
   ↓
SIMULATED
   ↓
IMPLEMENTED
   ↓
TESTED
   ↓
BENCHMARKED
   ↓
INTEGRATION-TESTED
   ↓
HARDWARE-VALIDATED
   ↓
PRODUCTION-READY
```

A higher status requires evidence appropriate to that status.

## Migration map

| Historical material | Current authority |
|---|---|
| System architecture | `docs/architecture/` |
| Runtime and scheduling | `docs/architecture/runtime.md`, `docs/architecture/scheduling.md` |
| IPC and transport | `docs/architecture/ipc.md`, `docs/architecture/transport.md` |
| Hardware abstraction | `docs/architecture/hardware.md` |
| Distributed operation | `docs/architecture/distributed.md` |
| Simulation and Studio | `docs/architecture/simulation.md`, `docs/architecture/studio.md` |
| Type and protocol contracts | `docs/specifications/` |
| Safety model | `docs/specifications/safety.md`, `docs/safety/` |
| Concrete interfaces | `docs/reference/` |
| Evidence and claims | `docs/verification/` |
| Historical migration | `docs/migration/` |

## Historical material that requires evidence before promotion

The legacy design contains examples and claims involving zero-copy IPC, deterministic real-time guarantees, automatic hardware discovery, DMA, GPU/NPU dispatch, ROS 2 bridging, Studio functionality, OTA deployment, and safety compliance.

These remain **design material** unless corresponding implementation and verification evidence establishes a stronger status.

Likewise, historical performance tables are targets/design objectives, not current universal guarantees. See [COMPARISON.md](COMPARISON.md).

## Preservation

The previous full design document remains available through Git history. New architectural or normative material should be added to the focused documentation pages rather than expanding this compatibility entry point.

For the migration policy, see [Documentation Migration](docs/migration/README.md).
