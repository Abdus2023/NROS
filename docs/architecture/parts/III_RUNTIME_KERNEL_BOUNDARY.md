# Part III — NROS Runtime & Kernel Boundary

> **Series:** NROS Architecture Series  
> **Part:** III  
> **Role:** Runtime boundary and execution substrate  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part II defined the conceptual primitives of NROS. Part III defines the boundary that turns those primitives into a runtime model.

The central question is:

> **What is the minimum execution substrate required to create, schedule, execute, observe, and terminate NROS work without making the core dependent on a particular operating system, transport, executor, or application framework?**

The answer is a small, explicit runtime boundary with replaceable platform adapters.

## 2. Kernel Is a Boundary, Not a Monolith

The NROS kernel/runtime should not become a second operating system.

Its responsibility is to own semantics that must remain consistent across deployments:

```text
Identity
Lifecycle
Activation
Scheduling contract
Execution state
Resource/accounting metadata
Cancellation
Deadline metadata
Capability context
Events
Observation hooks
```

Platform-specific mechanisms remain below or beside that boundary:

```text
NROS Runtime / Kernel Boundary
              │
      ┌───────┼────────┐
      ▼       ▼        ▼
     OS      HAL    Transport
      │       │        │
      └───────┼────────┘
              ▼
           Hardware
```

## 3. Runtime Responsibilities

The runtime is responsible for coordinating work, not for implementing every mechanism itself.

### Required semantic responsibilities

- create and identify runtime entities;
- manage lifecycle transitions;
- represent activations;
- expose scheduling constraints;
- coordinate execution;
- manage cancellation and deadlines;
- expose resource requirements and accounting hooks;
- carry capability/policy context;
- publish runtime events;
- provide observation points;
- define failure boundaries.

### Explicitly outside the universal kernel

Depending on deployment, the following may be adapters rather than kernel functionality:

- Linux networking;
- RTOS primitives;
- device drivers;
- DMA engines;
- filesystem implementation;
- GPU runtime;
- TCP/UDP implementation;
- DDS or other external middleware;
- board-specific interrupt controllers.

The architecture may define contracts for these mechanisms without embedding one implementation into the core.

## 4. Minimal Runtime Object Model

A minimal conceptual runtime can be expressed as:

```text
Runtime
├── EntityRegistry
├── LifecycleManager
├── ActivationManager
├── Scheduler
├── Executor
├── ResourceRegistry
├── CapabilityContext
├── EventBus
├── Clock
└── ObservationSink
```

Not every item must be a separate crate or process. These are semantic responsibilities.

## 5. Entity Identity

Every runtime entity that participates in observable execution needs a stable identity within its defined scope.

Conceptually:

```text
EntityId
├── namespace
├── local_id
└── generation
```

Identity scope must be explicit.

```text
Local identity
      ≠
Process identity
      ≠
Node identity
      ≠
Cluster identity
```

A distributed deployment may therefore require both local and globally meaningful identifiers.

## 6. Runtime State

Runtime state should be represented explicitly rather than inferred from incidental data structures.

```text
RuntimeState
├── lifecycle
├── execution
├── health
├── resource
├── capability
└── fault
```

State transitions should produce observable events where the transition is relevant to correctness, recovery, or auditability.

## 7. Activation Lifecycle

The activation path is the core execution boundary:

```text
CAUSE
  ↓
CREATE ACTIVATION
  ↓
ADMISSION
  ↓
QUEUE / PLACE
  ↓
START
  ↓
EXECUTE
  ├── checkpoint
  ├── effect
  └── observation
  ↓
COMPLETE
  ↓
ACCOUNT / RECORD
```

Failure paths must remain explicit:

```text
START / EXECUTE
      │
      ├── cancel
      ├── deadline miss
      ├── resource failure
      ├── component fault
      └── runtime fault
                ↓
             TERMINATE
                ↓
        RECOVERY / ESCALATION
```

## 8. Scheduler Boundary

The scheduler decides **when and where eligible work may execute**.

The scheduler should consume explicit metadata:

```text
Activation
├── priority
├── deadline
├── period
├── budget
├── affinity
├── execution class
├── resource requirements
└── dependencies
```

The scheduler must not be confused with the executor.

```text
Scheduler
→ selects / orders work

Executor
→ performs work
```

A scheduling algorithm is an implementation choice beneath the contract unless the architecture explicitly requires a particular algorithm.

## 9. Executor Boundary

The executor maps admitted work onto execution mechanisms.

```text
Activation
   ↓
Scheduler decision
   ↓
Executor
   ↓
Thread / task / interrupt / worker
   ↓
User computation
```

The executor may use:

- OS threads;
- async tasks;
- dedicated workers;
- embedded loops;
- interrupt-driven mechanisms.

The runtime contract should not silently equate any one mechanism with a particular real-time guarantee.

## 10. Platform Adapter Boundary

Platform adaptation should be explicit:

```text
                Runtime Contract
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
   Platform API     HAL API       Transport API
        │              │              │
        ▼              ▼              ▼
      Linux          MCU/SoC       UDP/SHM/etc.
```

Adapters translate runtime contracts into platform mechanisms.

An adapter may expose capabilities or limitations back to the runtime.

## 11. Resource Accounting

The runtime should be able to represent resource consumption even when allocation/control is delegated to the platform.

Examples:

```text
CPU time
Memory
I/O
Network bandwidth
Device handles
GPU time
Storage
Energy
```

The distinction is important:

```text
Accounting
   ≠
Enforcement
   ≠
Admission
```

A runtime may observe CPU usage without being able to enforce a CPU budget.

## 12. Cancellation and Deadlines

Cancellation and deadline metadata must have explicit semantics.

```text
Deadline
├── absolute time
├── clock domain
└── policy

Cancellation
├── requested
├── acknowledged
├── cooperative
└── forced (if supported)
```

A cancellation request does not prove that work has stopped.

Likewise:

```text
Deadline configured
      ≠
Deadline enforced
      ≠
Deadline always met
```

## 13. Capability Context

Runtime execution carries the authority context required for effects.

```text
Activation
   ↓
CapabilityContext
   ↓
Policy decision
   ↓
Effect request
```

The context may include:

- identity;
- granted capabilities;
- resource scope;
- security domain;
- delegation metadata;
- policy version.

This creates a stable integration point for later security and policy architecture.

## 14. Event Boundary

Important runtime transitions should be representable as events.

Example event vocabulary:

```text
EntityCreated
LifecycleChanged
ActivationCreated
ActivationAdmitted
ActivationStarted
ActivationCompleted
ActivationCancelled
DeadlineMissed
ResourceFault
CapabilityDenied
EffectRequested
EffectCompleted
ComponentFaulted
RecoveryStarted
RecoveryCompleted
```

Events are observations of runtime state transitions; they are not automatically a durable event log.

```text
Runtime event
      ≠
Persistent journal
```

Durability belongs to the persistence architecture introduced later in the series.

## 15. Clock Boundary

The runtime should consume an abstract clock interface rather than directly binding core semantics to wall-clock APIs.

```text
Clock
├── monotonic
├── realtime
├── logical
└── simulation
```

Deadline and duration semantics should identify the clock domain they depend on.

This enables simulation and replay without redefining the runtime API.

## 16. Failure Containment

The kernel boundary should define where failures are contained.

```text
Application fault
      ↓
Component boundary
      ↓
Runtime containment
      ↓
Platform boundary
```

Not every failure can or should be recovered by the runtime.

The architecture should distinguish:

```text
Recoverable
Isolatable
Escalatable
Fatal
```

A failure policy must remain domain-specific where safety requirements differ.

## 17. Observation Boundary

The runtime should expose observation hooks without forcing a particular telemetry implementation.

```text
Runtime state
     │
     ├── metrics
     ├── events
     ├── traces
     ├── logs
     └── evidence records
```

Observation must not silently change the semantics being observed, particularly for real-time-sensitive paths.

Therefore the architecture should account for instrumentation overhead.

## 18. Determinism Boundary

The runtime can provide mechanisms that support deterministic behavior, but determinism remains a property requiring precise definition and evidence.

Possible properties include:

```text
Deterministic state transition
Deterministic scheduling
Deterministic replay
Bounded latency
Stable resource usage
```

These must not be collapsed into one generic claim of "deterministic runtime."

## 19. Core / Adapter Separation

The architectural dependency direction should remain:

```text
Application
    ↓
NROS APIs
    ↓
Runtime contracts
    ↓
Core semantics
    ↓
Adapter contracts
    ↓
Platform mechanisms
```

The core must not require a concrete network stack, board, filesystem, or operating system merely to represent its abstract runtime semantics.

## 20. Rust Mapping

Rust provides a natural implementation vocabulary for the boundary:

```text
trait Clock
trait Executor
trait Scheduler
trait Transport
trait ResourceProvider
trait CapabilityProvider
trait ObservationSink
trait Platform
```

The exact trait set is an implementation decision and should be derived from actual repository needs.

Traits must not be introduced merely to mirror this document.

## 21. Repository Boundary Rule

The architecture document must never be used to infer that a corresponding implementation exists.

The repository representation should be able to answer independently:

```text
Does this runtime concept exist in source?
Is it tested?
Was CI executed?
Was it benchmarked?
Was it simulated?
Was it validated on hardware?
```

If the answer is unknown, the claim remains unknown.

## 22. Verification Matrix

| Runtime boundary | Verification question |
|---|---|
| Entity identity | Are identity collisions/reuse handled correctly? |
| Lifecycle | Are invalid transitions rejected? |
| Activation | Are terminal states complete and unambiguous? |
| Scheduler | Are ordering/admission invariants enforced? |
| Executor | Are execution errors propagated correctly? |
| Resource accounting | Are measurements attributable to the correct entity? |
| Cancellation | Does cancellation reach the defined terminal state? |
| Deadline | Is the clock domain explicit and the miss observable? |
| Capability | Are denied effects actually rejected? |
| Events | Are event ordering and identity semantics defined? |
| Clock | Are monotonic and logical semantics preserved? |
| Observation | Is instrumentation bounded and non-destructive? |
| Adapter boundary | Can platform differences be isolated without changing core semantics? |

## 23. What Part III Does Not Claim

This Part does not establish that the repository currently provides:

- a complete kernel;
- a production scheduler;
- hard real-time guarantees;
- universal platform portability;
- complete capability enforcement;
- deterministic execution;
- production-grade telemetry;
- fault containment for every failure class.

Those claims require implementation and verification evidence.

## 24. Transition to Part IV

Part III defines the runtime boundary.

Part IV should make the execution model concrete by defining **lifecycle, entity state, and transition semantics** in enough detail to support implementation, testing, and later supervision/recovery architecture.

```text
Part I
Foundation
  ↓
Part II
Core concepts
  ↓
Part III
Runtime / kernel boundary
  ↓
Part IV
Lifecycle + entity state
```

## Canonical rule

> **The NROS kernel is the smallest stable semantic boundary that coordinates execution; operating-system, hardware, and transport mechanisms remain replaceable adapters beneath explicit contracts.**
