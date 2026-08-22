# NROS Concepts

> **Purpose:** Define the conceptual vocabulary used throughout NROS documentation.

This section explains **what the NROS concepts mean** without requiring the reader to know their implementation. Conceptual definitions describe the model; they do not automatically establish implementation or verification status.

## 1. Conceptual model

NROS is described in terms of cooperating components that exchange typed information, expose operations, and participate in managed execution lifecycles.

The core vocabulary is:

```text
Node
 ├── publishes / receives Messages
 ├── participates in Topics
 ├── may expose Services
 ├── may participate in Actions
 └── may have a managed Lifecycle
```

These are conceptual relationships. The exact implementation surface is defined by the current repository and its reference documentation.

## 2. Core concepts

- [Overview](overview.md) — the problem domain, scope, and conceptual boundaries.
- [Design Principles](design-principles.md) — principles that guide NROS architecture.
- [Nodes](nodes.md) — the conceptual component/execution boundary.
- [Messages](messages.md) — typed data exchanged between components.
- [Topics](topics.md) — named communication relationships for message streams.
- [Services](services.md) — request/response interactions.
- [Actions](actions.md) — longer-running goal-oriented interactions.
- [Lifecycle](lifecycle.md) — managed component states and transitions.

## 3. Vocabulary boundaries

### Node

A **node** is a conceptual NROS component boundary. It represents a participant in the system rather than automatically implying a particular operating-system process, thread, or task.

### Message

A **message** is typed data exchanged between participants. The conceptual term does not by itself specify serialization, transport, allocation, ownership, or wire representation.

### Topic

A **topic** is a named communication relationship for message-oriented data flow. A topic describes the communication abstraction; it does not by itself prove a particular transport implementation.

### Service

A **service** represents a request/response interaction with a defined interface and response relationship.

### Action

An **action** represents a longer-running goal-oriented interaction in which progress, completion, cancellation, feedback, or related state may matter.

### Lifecycle

A **lifecycle** describes managed component state and permitted transitions. It is a state-management concept, not merely a process start/stop mechanism.

## 4. Important non-equivalences

NROS documentation MUST NOT silently equate these concepts:

```text
Node        ≠ OS process
Node        ≠ thread
Message     ≠ serialized byte buffer
Topic       ≠ network socket
Service     ≠ arbitrary RPC implementation
Action      ≠ ordinary function call
Lifecycle   ≠ process lifetime
Runtime     ≠ operating system
Transport   ≠ communication abstraction
```

A concrete implementation may map one concept onto another mechanism, but that mapping must be documented explicitly.

## 5. Data and execution

NROS has two related but distinct conceptual dimensions:

```text
Data plane
  Messages
  Topics
  Services
  Actions

Execution plane
  Nodes
  Lifecycle
  Scheduling / callbacks
  Runtime mechanisms
```

The separation is useful when reasoning about correctness. A message contract does not automatically define scheduling behavior, and a scheduling mechanism does not automatically define communication semantics.

## 6. Determinism

**Determinism** MUST always be scoped.

A statement such as "NROS is deterministic" is incomplete unless it identifies the behavior, inputs, scheduling assumptions, execution environment, and evidence supporting the claim.

Possible scopes include:

- deterministic data transformation;
- deterministic state transition;
- deterministic ordering;
- bounded scheduling behavior;
- repeatable benchmark behavior.

These are different properties.

## 7. Real-time

**Real-time** is a timing requirement, not a synonym for fast execution.

Distinguish:

```text
Low latency
Bounded latency
Deadline compliance
Worst-case timing bound
Hard real-time qualification
```

Evidence for one does not automatically establish the next.

## 8. Zero-copy

**Zero-copy** is a property of a defined data path, not a blanket property of a system.

The documentation should identify which boundaries are claimed to avoid copying and which operations may still copy or allocate.

```text
Zero-copy at one boundary
        ≠
Zero-copy end-to-end
```

## 9. Runtime

The **runtime** is the execution infrastructure responsible for coordinating supported NROS execution mechanisms. The term does not imply a particular operating system, process model, scheduler, or transport unless explicitly specified.

## 10. Transport

A **transport** is the mechanism that moves data between communication participants or boundaries. Transport is an implementation dimension of communication; it should not be confused with the higher-level message/topic/service/action abstractions.

## 11. Implementation boundary

Conceptual documentation intentionally stops short of claiming implementation maturity.

```text
Concept
  ↓
Specification
  ↓
Implementation
  ↓
Verification
  ↓
Validation
```

Each transition requires its own evidence.

## 12. Authority and evidence

Concept pages define vocabulary and conceptual intent. They do not establish that a feature exists.

For implementation questions, consult [Reference](../reference/README.md).

For demonstrated behavior, consult [Verification](../verification/README.md).

For use-case acceptance, consult [Validation](../verification/validation.md).

## 13. Related documentation

- [Documentation Hub](../README.md)
- [Architecture](../architecture/README.md)
- [Specifications](../specifications/README.md)
- [Reference](../reference/README.md)
- [Verification](../verification/README.md)
