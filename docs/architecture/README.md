# NROS Architecture

> **Status:** Active architectural documentation.
>
> This page is the architectural map for NROS. It describes system boundaries, relationships, and design responsibilities. It does not by itself prove implementation completeness, timing guarantees, hardware support, or production readiness.

## 1. Architectural intent

NROS is organized around a Rust-oriented robotics runtime with explicit boundaries between application entities, execution, communication, hardware abstraction, simulation, tooling, and verification.

```text
Application / Robotics Graph
            │
            ▼
      Node & Entities
            │
      ┌─────┴─────┐
      ▼           ▼
 Execution     Communication
      │           │
 Scheduler    Transport / IPC
      │           │
      └─────┬─────┘
            ▼
     Hardware Abstraction
            │
      ┌─────┴─────┐
      ▼           ▼
   Hardware    Simulation
            │
            ▼
     Tooling / Observability
```

## 2. Architectural boundaries

### Application boundary

Application code expresses robotics behavior through nodes, entities, messages, parameters, and lifecycle concepts where those facilities are implemented.

### Runtime boundary

The runtime owns execution coordination, scheduling, lifecycle coordination, and integration points between application entities and lower layers.

### Communication boundary

Communication separates the logical data contract from transport mechanics. Serialization, memory ownership, allocation, transport, and delivery scheduling are distinct concerns.

### Hardware boundary

Hardware abstraction isolates target-specific device behavior from higher-level runtime logic. A declared abstraction does not imply that every target has a working driver.

### Simulation boundary

Simulation supplies controlled substitutes for physical resources. Simulation is a validation environment, not evidence that the same behavior has been demonstrated on physical hardware.

### Verification boundary

Verification documentation records what is actually demonstrated. Architecture documents define intent and relationships; they do not upgrade evidence state.

## 3. Architectural layers

| Layer | Responsibility | Evidence boundary |
|---|---|---|
| Application | Robotics logic and graph composition | Depends on implemented APIs |
| Node / Entity | Runtime participants and communication objects | Interface presence does not prove backend completeness |
| Execution | Scheduling, callbacks, lifecycle progression | Determinism requires timing evidence |
| Communication | Logical messaging and IPC | Transport properties require implementation evidence |
| Transport | Data movement and ownership | Zero-copy requires concrete ownership/transport evidence |
| Hardware | Devices and target integration | Requires target-specific validation |
| Simulation | Controlled development/validation environment | Does not establish hardware behavior |
| Tooling | CLI, Studio, diagnostics, observability | Operational status requires live-provider evidence |

## 4. Architecture documents

- [Overview](overview.md) — system-level architecture.
- [System Model](system-model.md) — boundaries, components, and dependencies.
- [Runtime](runtime.md) — runtime and execution model.
- [IPC](ipc.md) — local communication and ownership model.
- [Scheduling](scheduling.md) — execution and timing model.
- [Transport](transport.md) — network communication model.
- [Hardware](hardware.md) — hardware abstraction boundaries.
- [Distributed](distributed.md) — multi-node and fleet concepts.
- [Simulation](simulation.md) — simulation architecture.
- [Studio](studio.md) — observability and development tooling.

## 5. Architecture versus implementation

```text
Architecture
     ↓
Specification
     ↓
Implementation
     ↓
Tests / Evidence
     ↓
Validation
```

Each subsystem must be evaluated through the evidence model rather than inferred from an architecture diagram.

## 6. Architectural invariants

1. Higher-level layers must not silently assume unavailable lower-layer capabilities.
2. Logical communication contracts are distinct from transport implementation.
3. Simulation boundaries must remain distinguishable from hardware boundaries.
4. Safety requirements constrain implementation and integration choices.
5. Performance claims require reproducible measurements.
6. Real-time guarantees require target-specific timing evidence.
7. Architecture documentation must not be used as implementation evidence.

## 7. Related documentation

- [Conceptual Model](../concepts/model.md)
- [Specifications](../specifications/README.md)
- [Reference](../reference/README.md)
- [Verification](../verification/README.md)
- [Safety](../safety/README.md)
- [Repository Representation](../REPOSITORY_REPRESENTATION.md)
