# NROS System Model

> **Status:** Active architectural documentation.
>
> This document defines the logical system components and their boundaries. It does not assert that every component or backend is fully implemented.

## 1. Logical system

The NROS system can be understood as a set of cooperating layers:

```text
Application
    │
    ▼
Node Graph / Entities
    │
    ├──────────────┐
    ▼              ▼
Execution       Communication
    │              │
    │          ┌───┴────┐
    │          ▼        ▼
    │         IPC    Network Transport
    │
    └──────────┬──────────┘
               ▼
       Hardware Abstraction
          │            │
          ▼            ▼
       Hardware     Simulation
          │
          └──────┬──────┘
                 ▼
        Tooling / Observability
```

This is a logical model. Repository crates, binaries, adapters, and services may realize only a subset of these components at a given revision.

## 2. Component responsibilities

| Component | Primary responsibility | Boundary |
|---|---|---|
| Application | Robot/domain behavior | Uses public runtime interfaces |
| Node graph | Logical participants and relationships | Separates graph semantics from execution |
| Entities | Publishers, subscribers, services, actions, timers, parameters | Runtime-facing objects |
| Execution | Callback and lifecycle progression | Scheduling semantics are independently verified |
| Communication | Typed data and interaction contracts | Independent from concrete transport |
| IPC | Local process/thread communication | Requires concrete ownership and synchronization evidence |
| Network transport | Cross-process/machine delivery | Requires concrete protocol/backend evidence |
| Hardware abstraction | Stable device boundary | Driver coverage is target-specific |
| Simulation | Controlled virtual resources | Does not imply physical validation |
| Tooling | CLI, Studio, diagnostics, telemetry | Live operation requires runtime/provider evidence |

## 3. Ownership boundaries

The architecture treats several forms of ownership as separate concerns:

```text
Logical ownership
    ≠ memory ownership
    ≠ transport ownership
    ≠ lifecycle ownership
    ≠ hardware ownership
```

Documentation should identify which ownership model a component actually implements before making zero-copy, lifetime, or safety claims.

## 4. Control flow

A typical application flow is conceptually:

```text
Create / configure
        ↓
Create node and entities
        ↓
Register execution work
        ↓
Start runtime
        ↓
Execute callbacks / events
        ↓
Exchange data
        ↓
Observe / diagnose
        ↓
Shutdown / finalize
```

The exact lifecycle and scheduling behavior are defined by the implementation and corresponding specifications, not by this conceptual sequence alone.

## 5. Data flow

```text
Producer
   ↓
Logical message
   ↓
Serialization / representation
   ↓
Transport or IPC
   ↓
Deserialization / representation
   ↓
Consumer
```

Implementations may optimize this path through borrowing, shared memory, batching, or other mechanisms. Such optimizations require concrete evidence and must not be inferred from the logical model.

## 6. Failure boundaries

Failures should remain attributable to the layer that owns them:

- application errors belong to application logic;
- runtime lifecycle/execution failures belong to runtime coordination;
- communication failures belong to communication/transport mechanisms;
- device failures belong to hardware adapters or drivers;
- simulation failures belong to simulation infrastructure;
- tooling failures belong to the relevant tooling/provider boundary.

This separation improves diagnostics and prevents a failure in one layer from being described as a property of the entire system without evidence.

## 7. Configuration boundary

Configuration should select or parameterize implementations without silently changing the conceptual contract. Configuration claims must distinguish:

```text
Supported option
    ≠
Implemented option
    ≠
Validated option
```

## 8. Verification boundary

The system model is not a capability matrix. For each component, verification should establish:

1. whether the interface exists;
2. whether the implementation is present;
3. whether relevant tests execute;
4. whether integration is demonstrated;
5. whether performance or safety claims have appropriate evidence.

See [Verification](../verification/README.md) and [Evidence Registry](../../EVIDENCE_REGISTRY.md).

## 9. Related documents

- [Architecture Overview](overview.md)
- [Runtime](runtime.md)
- [IPC](ipc.md)
- [Scheduling](scheduling.md)
- [Transport](transport.md)
- [Hardware](hardware.md)
- [Distributed](distributed.md)
- [Simulation](simulation.md)
