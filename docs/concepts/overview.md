# NROS Conceptual Overview

> **Purpose:** Provide the high-level mental model for NROS before readers enter architecture, implementation, or verification details.

NROS (Native Robotics Operating System) is a robotics software project centered on a native, systems-oriented foundation for composing robot applications from typed components, communication mechanisms, execution infrastructure, and hardware-facing interfaces.

The project emphasizes explicit system boundaries and evidence-aware engineering. Claims about determinism, performance, safety, hardware behavior, or production readiness are intentionally separated from the conceptual model and must be supported by implementation and verification evidence.

## 1. What problem NROS addresses

Robotics software crosses several boundaries at once:

```text
Application logic
      ↓
Component communication
      ↓
Execution / scheduling
      ↓
Operating-system services
      ↓
Hardware interfaces
      ↓
Physical system
```

Each boundary introduces different requirements for:

- data representation;
- latency and timing;
- concurrency;
- resource ownership;
- failure handling;
- observability;
- portability;
- hardware interaction.

NROS provides a conceptual and implementation framework for making those boundaries explicit rather than treating the entire robot software stack as one undifferentiated runtime.

## 2. Conceptual stack

At a high level:

```text
┌──────────────────────────────┐
│ Robot applications           │
├──────────────────────────────┤
│ APIs, tools, and interfaces   │
├──────────────────────────────┤
│ Core services                │
├──────────────────────────────┤
│ Communication substrate      │
├──────────────────────────────┤
│ Runtime / execution          │
├──────────────────────────────┤
│ Hardware abstraction         │
├──────────────────────────────┤
│ OS / target hardware         │
└──────────────────────────────┘
```

This is a conceptual stack. It MUST NOT be interpreted as proof that every layer is implemented, integrated, or production-ready in the current repository.

## 3. Three questions

NROS documentation separates three fundamentally different questions:

### What should exist?

Answered by specifications, architecture, requirements, and design documents.

### What exists?

Answered by current source code, manifests, APIs, generated artifacts, and repository state.

### What has been demonstrated?

Answered by executed tests, benchmarks, integration runs, hardware evidence, and validation records.

```text
Intent
  ↓
Implementation
  ↓
Evidence
```

These stages MUST NOT be collapsed into one claim.

## 4. Core system boundaries

NROS can be reasoned about through several boundaries:

### Component boundary

A node or other component owns a defined portion of behavior and communicates through explicit interfaces.

### Communication boundary

Messages, topics, services, and actions describe communication semantics. Transport mechanisms implement the movement of data across a particular boundary.

### Execution boundary

Scheduling, callbacks, tasks, lifecycle transitions, and runtime mechanisms determine when and how work executes.

### Hardware boundary

Hardware abstraction isolates target-specific mechanisms from higher-level software where the architecture requires such separation.

### Evidence boundary

Verification and validation distinguish documented intent from observed behavior.

## 5. What NROS is not

The conceptual model does not imply that NROS is automatically:

- a complete operating-system kernel;
- a replacement for every host operating system;
- a particular process model;
- a particular network protocol;
- hard real-time software;
- zero-copy end-to-end software;
- hardware-certified software;
- production-qualified software.

Those statements require specific implementation and evidence.

## 6. Determinism and timing

NROS treats timing and execution behavior as explicit engineering concerns.

However, the phrase **"NROS is deterministic"** is too broad to be a useful technical claim without scope.

Possible claims include:

```text
Deterministic transformation
Deterministic state transition
Deterministic ordering
Bounded scheduling behavior
Repeatable execution under defined conditions
```

Likewise:

```text
Low latency
      ≠
Bounded latency
      ≠
Deadline compliance
      ≠
Worst-case timing guarantee
```

The applicable claim and its evidence must be stated explicitly.

## 7. Communication model

The conceptual communication model consists of typed information and defined interaction patterns:

```text
Messages
   │
   ├── Topics   → message streams
   ├── Services → request / response
   └── Actions  → goal-oriented interactions
```

This describes communication semantics. It does not prescribe a single transport or imply a particular serialization format.

## 8. Runtime model

The runtime is the infrastructure that coordinates supported NROS execution mechanisms.

It may encompass scheduling, callback execution, lifecycle coordination, resource management, and other runtime services depending on the implemented architecture.

The term **runtime** does not by itself establish:

- a scheduler algorithm;
- a process model;
- a thread model;
- an operating-system implementation;
- real-time guarantees.

Those properties belong to more specific architecture and verification documents.

## 9. Hardware model

Hardware-facing behavior is treated as an explicit boundary:

```text
Application / component
          ↓
      NROS API
          ↓
 Hardware abstraction
          ↓
 Target-specific implementation
          ↓
       Hardware
```

The existence of an abstraction layer does not by itself prove that a physical target is supported. Hardware support requires target-specific implementation and evidence.

## 10. Current repository reality

The repository may contain different maturity states at the same time:

```text
PROPOSED
SPECIFIED
SCAFFOLDED
SIMULATED
IMPLEMENTED
TESTED
BENCHMARKED
INTEGRATION-TESTED
HARDWARE-VALIDATED
PRODUCTION-READY
```

A capability may occupy one state while another dimension remains unverified. Readers should therefore use the [Verification](../verification/README.md) documentation when evaluating claims about current behavior.

## 11. Documentation authority

Use the documentation layers according to the question:

| Question | Primary source |
|---|---|
| What does the concept mean? | Concepts |
| How should the system be structured? | Architecture |
| What MUST be true? | Specifications |
| What exists now? | Reference / source |
| How do I use it? | Getting Started / Reference |
| What has been demonstrated? | Verification |
| Does it satisfy the use case? | Validation |

If a conceptual statement conflicts with current implementation evidence, the conflict should be recorded and resolved rather than silently treating the conceptual page as proof of implementation.

## 12. Next reading

- [Design Principles](design-principles.md)
- [Concepts](README.md)
- [Architecture](../architecture/README.md)
- [Specifications](../specifications/README.md)
- [Reference](../reference/README.md)
- [Verification](../verification/README.md)
