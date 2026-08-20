# NROS Runtime

> **Status:** Active architectural documentation.
>
> This document defines the conceptual responsibilities and boundaries of the NROS runtime. Concrete implementation and verification status must be established from repository evidence.

## 1. Runtime responsibility

The runtime is the coordination layer between application-facing entities and lower-level execution and communication mechanisms.

```text
Application
    ↓
Nodes / Entities
    ↓
Runtime
 ┌──┼───────────┐
 ↓  ↓           ↓
Exec Lifecycle Communication
 ↓              ↓
Scheduler      IPC / Transport
```

The runtime should coordinate application entities without unnecessarily coupling application code to a particular scheduler, transport, or hardware backend.

## 2. Lifecycle

A runtime lifecycle can be modeled as:

```text
Created
  ↓
Configured
  ↓
Initialized
  ↓
Running
  ↓
Stopping
  ↓
Stopped
  ↓
Finalized
```

The exact states, transitions, and error behavior are normative only when established by the relevant specification or implementation contract.

Lifecycle transitions must account for resources owned by nodes, entities, executors, communication endpoints, and adapters.

## 3. Execution coordination

The runtime coordinates executable work such as callbacks, timers, events, lifecycle transitions, and other scheduled operations.

```text
Event / Timer / Message
          ↓
     Ready work
          ↓
      Scheduler
          ↓
      Executor
          ↓
       Callback
          ↓
    State / Output
```

This model does not imply a particular scheduling algorithm or timing guarantee.

## 4. Ownership and shutdown

Shutdown is an ownership and synchronization problem, not merely a boolean state change.

```text
Stop accepting new work
        ↓
Quiesce execution
        ↓
Close communication
        ↓
Release runtime resources
        ↓
Finalize adapters
        ↓
Stopped / Finalized
```

The actual ordering must follow implementation-specific lifetime and synchronization rules.

## 5. Error boundaries

Runtime errors should preserve their originating context. Examples include invalid lifecycle transitions, initialization failures, executor failures, communication setup failures, transport failures, adapter failures, and shutdown/finalization failures.

Error handling must not silently convert an unavailable capability into a successful runtime state.

## 6. Concurrency boundary

The runtime may coordinate multiple concurrent activities. Documentation must distinguish:

```text
Concurrency
    ≠
Parallelism
    ≠
Determinism
    ≠
Real-time guarantees
```

Any stronger claim requires evidence for the relevant target, scheduler, workload, and environment.

## 7. Extensibility

Schedulers, executors, transports, and hardware adapters should be replaceable behind explicit contracts where supported by the implementation.

Extensibility means a boundary permits alternative implementations; it does not prove that those implementations currently exist.

## 8. Verification requirements

| Claim | Required evidence |
|---|---|
| API exists | Source/interface inspection |
| Lifecycle works | Executed lifecycle tests |
| Shutdown is safe | Relevant tests plus ownership/lifetime evidence |
| Scheduler is deterministic | Reproducible timing evidence |
| Real-time behavior | Target-specific timing validation |
| Runtime survives transport failure | Failure-path integration tests |
| Production readiness | Broader integration and operational evidence |

## 9. Related documents

- [Architecture Overview](overview.md)
- [System Model](system-model.md)
- [IPC](ipc.md)
- [Scheduling](scheduling.md)
- [Transport](transport.md)
- [Specifications](../specifications/README.md)
- [Verification](../verification/README.md)
- [Safety](../safety/README.md)
