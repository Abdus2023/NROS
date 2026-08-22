# Part II — NROS Core Concepts

> **Series:** NROS Architecture Series  
> **Part:** II  
> **Role:** Core computational model  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part II defines the minimum conceptual vocabulary required to reason about NROS as a runtime rather than merely a ROS-compatible middleware layer.

The central shift is:

```text
ROS-oriented model
Node + callback + topic

NROS-oriented model
Component + activation + channel + state + resource
```

The goal is not to replace useful ROS concepts for their own sake. The goal is to make execution semantics explicit enough that scheduling, resource ownership, lifecycle, failure handling, and verification can be reasoned about systematically.

## 2. Component

The primary application/runtime entity is a **Component**.

A component has an explicit contract:

```text
Component
├── Identity
├── Inputs
├── Outputs
├── State
├── Services
├── Actions
├── Resources
├── Lifecycle
├── Execution constraints
└── Dependencies
```

Example:

```text
Component: lidar_processor

Inputs:
    LaserScan

Outputs:
    ObstacleSet

State:
    calibration
    filtering
    diagnostics

Execution:
    periodic / event-driven

Constraints:
    period   = 10 ms
    deadline = 5 ms
    memory   = bounded
```

The example is architectural. It does not assert that the current repository implements every field or guarantee.

## 3. Activation

The fundamental runtime occurrence is an **activation**.

Instead of treating execution as an implicit consequence of a callback, NROS models the path explicitly:

```text
Event
  ↓
Activation
  ↓
Admission
  ↓
Scheduling
  ↓
Execution
  ↓
Completion
  ↓
Observation
```

An activation represents a request for a component to perform a bounded unit of work.

Conceptually:

```text
Activation
├── activation_id
├── component
├── cause
├── priority
├── deadline
├── budget
├── timestamp
└── cancellation
```

Possible causes include:

```text
Message
Timer
Goal
ServiceRequest
LifecycleTransition
ExternalEvent
Recovery
```

## 4. Execution classes

NROS may classify work according to its execution requirements:

```text
ExecutionClass
├── HardRealTime
├── SoftRealTime
├── Periodic
├── EventDriven
├── Background
└── BestEffort
```

These labels are meaningful only when their semantics are specified and the required properties are actually verified.

For example:

```text
MotorController → real-time-oriented
SensorFusion    → periodic
Diagnostics     → best-effort
Persistence     → background
```

An execution class is therefore a **constraint vocabulary**, not a performance guarantee.

## 5. Channel

NROS uses a typed **Channel** as the principal continuous communication abstraction.

```text
Channel<T>
├── Type
├── Capacity
├── Ordering
├── Reliability
├── Delivery policy
├── Ownership
├── Backpressure
├── QoS
└── Transport binding
```

This makes communication semantics independent from the mechanism used to move data.

For example:

```text
Channel<ImageFrame>
```

may be realized by an in-process queue, shared memory, or a network transport without changing the logical application contract.

## 6. Backpressure

A bounded runtime must define what happens when producers outpace consumers.

Possible policies include:

```text
Backpressure
├── Block
├── DropNewest
├── DropOldest
├── LatestOnly
├── Reject
└── Buffer
```

The policy is part of the channel contract.

Unbounded buffering should never be assumed to be safe for real-time-oriented workloads.

## 7. Ownership and data movement

Rust allows NROS to make data ownership explicit.

Conceptually, a message may be:

```text
Owned
Shared
Borrowed
Moved
Serialized
ZeroCopy-capable
```

These are different mechanisms with different lifetime and synchronization requirements.

In particular:

```text
Zero-copy data structure
        ≠
zero-copy end-to-end system
```

The latter requires evidence across the complete data path.

## 8. Local and distributed communication

The logical channel should remain stable across deployment boundaries:

```text
                    Channel<T>
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
       InProcess     SharedMemory   Network
```

The transport can change while the application-facing contract remains conceptually stable.

This allows the same component model to support:

- same-thread execution;
- same-process execution;
- inter-process execution;
- embedded/local IPC;
- cross-machine communication.

The performance, capability set, and failure model may differ by transport and must therefore remain explicit.

## 9. Runtime graph

The NROS graph extends beyond communication topology.

```text
Runtime Graph
├── Components
├── Channels
├── Resources
├── Lifecycle state
├── Execution constraints
├── Dependencies
└── Health/fault state
```

The graph is therefore a candidate **runtime system model**, not merely a visualization of topic connections.

## 10. Lifecycle

A component lifecycle should be explicit and transition-driven.

One conceptual model is:

```text
CREATED
   ↓
CONFIGURED
   ↓
READY
   ↓
ADMITTED
   ↓
RUNNING
   ↓
STOPPING
   ↓
STOPPED
```

Fault and recovery paths are part of the lifecycle model:

```text
RUNNING
   │
   ├── normal stop → STOPPING
   │
   └── fault → FAULTED → RECOVER / ISOLATE / SAFE STATE
```

Exact states may evolve; explicit transition semantics are the invariant.

## 11. Fault model

NROS should distinguish failure classes instead of collapsing them into "node crashed":

```text
Failure
├── ComponentFailure
├── ChannelFailure
├── TransportFailure
├── DeadlineMiss
├── ResourceExhaustion
├── InvalidInput
├── HardwareFailure
└── RuntimeFailure
```

Policies may include:

```text
Retry
Restart
Isolate
Degrade
FailSafe
Shutdown
Escalate
```

The appropriate policy is domain-specific and must be tied to an explicit safety/operational requirement.

## 12. Resource model

Resources are first-class runtime objects.

Examples:

```text
Resource
├── CPU
├── Memory
├── GPU
├── Camera
├── LiDAR
├── Motor
├── Network
├── Storage
└── Device
```

A component can declare requirements and the runtime can perform admission checks before activation.

```text
Component
   ↓
Resource requirements
   ↓
Availability / policy check
   ↓
Admission decision
```

This is a design mechanism, not evidence that the current implementation already performs complete resource admission.

## 13. Services

Services model bounded request/response interactions:

```text
Service<Request, Response>
```

A request should have enough metadata to support distributed execution, such as:

```text
request_id
source
timestamp
deadline
payload
```

The response should preserve correlation and status.

## 14. Actions / goals

Long-running work requires more than request/response.

NROS therefore treats a goal/action as managed work:

```text
Action<Goal, Feedback, Result>
```

Its lifecycle can include:

```text
Created
  ↓
Submitted
  ↓
Admitted
  ↓
Queued
  ↓
Executing
  ├── Feedback
  ├── Checkpoints
  └── Cancellation
  ↓
Succeeded / Failed / Cancelled
```

This integrates naturally with scheduling, resources, deadlines, lifecycle, and fault policy.

## 15. Time

NROS should distinguish at least:

```text
Wall time
Monotonic time
Logical/simulation time
```

The runtime should use monotonic time for elapsed-time and deadline calculations and allow virtual/logical clocks for simulation, replay, and deterministic testing.

Conceptually:

```text
Clock
├── monotonic()
├── realtime()
├── simulation()
└── logical()
```

## 16. Unified execution model

The concepts above converge on one runtime path:

```text
Component
    │
    ├── Message
    ├── Timer
    ├── Service request
    ├── Goal
    ├── Lifecycle event
    └── Recovery event
            │
            ▼
        Activation
            │
            ▼
         Scheduler
            │
            ▼
         Executor
            │
            ▼
        Computation
            │
            ▼
       Proposed effect
            │
            ▼
       Policy / capability
            │
            ▼
          Resource
            │
            ▼
       Observation / trace
```

This is the central conceptual model of Part II.

## 17. Effects

Runtime computation may produce external effects:

```text
Motor command
Network transmission
File write
Device operation
State mutation
```

NROS should conceptually distinguish computation from effects:

```text
Activation
   ↓
Computation
   ↓
Proposed Effect
   ↓
Policy / Capability Check
   ↓
External Effect
   ↓
Observation
```

This gives safety, replay, and verification systems an explicit point at which effects can be inspected.

## 18. Capability boundary

A component should only be able to exercise capabilities it currently possesses.

```text
Component
   ↓
Capability
   ↓
Effect
   ↓
Policy
   ↓
Resource
```

Capability possession is therefore distinct from effect execution.

```text
Capability granted
        ≠
operation executed
        ≠
operation succeeded
```

## 19. Determinism boundary

The runtime should distinguish:

```text
Functional determinism
Scheduling determinism
Temporal predictability
Replay determinism
```

These are separate properties.

For example:

```text
same inputs → same output
```

does not prove:

```text
same inputs → same execution timing
```

Likewise, a repeatable benchmark does not by itself establish a worst-case real-time guarantee.

## 20. Architectural invariants

Part II establishes these principles:

### C1 — Execution is explicit

Runtime work is represented by activations rather than being defined solely by callback invocation.

### C2 — Communication is contract-driven

A channel defines semantics independently from its transport mechanism.

### C3 — Resources are bounded architectural objects

Resource requirements and ownership must be representable independently of application convention.

### C4 — Lifecycle transitions are explicit

A component cannot legitimately claim a lifecycle state without satisfying that state's prerequisites.

### C5 — Effects are observable

External effects should have a representation that can participate in policy, tracing, and verification.

### C6 — Capability is distinct from effect

Authorization to access a resource does not imply that an operation occurred or succeeded.

### C7 — Runtime guarantees require evidence

Scheduling metadata, bounded queues, or Rust types are mechanisms; they are not themselves proof of real-time, safety, or distributed-system guarantees.

## 21. Implementation boundary

Part II defines concepts, not a mandatory crate decomposition.

A future implementation may expose these semantics through crates such as:

```text
nros-core
nros-types
nros-channel
nros-event
nros-runtime
nros-executor
nros-scheduler
nros-time
nros-lifecycle
nros-service
nros-action
nros-transport
nros-safety
nros-observability
```

The actual repository topology remains authoritative for what exists.

## 22. Verification implications

Each major concept creates a corresponding verification question:

| Concept | Evidence question |
|---|---|
| Component | Is the declared contract enforced? |
| Activation | Can execution causes and state transitions be observed? |
| Channel | Are type, capacity, ordering, and delivery semantics enforced? |
| Lifecycle | Are invalid transitions rejected? |
| Resource | Are ownership/admission rules enforced? |
| Goal | Are cancellation, deadline, and terminal states correct? |
| Clock | Is monotonic/logical behavior correct under test? |
| Effect | Can external operations be identified and traced? |
| Capability | Can unauthorized operations be rejected? |
| Determinism | What exact property is being measured or proven? |

The verification framework under `docs/verification/` governs how those claims are evidenced.

## 23. What Part II does not claim

This Part does **not** claim that the repository currently provides:

- a complete deterministic scheduler;
- hard real-time guarantees;
- universal zero-copy communication;
- complete resource admission;
- production-grade fault containment;
- complete capability security;
- deterministic distributed replay;
- safety certification.

Those require implementation-specific evidence and, where appropriate, validation/qualification evidence.

## 24. Transition to Part III

Part II defines the conceptual primitives.

Part III should therefore answer the next concrete question:

> **What is the minimal NROS runtime/kernel boundary, and how should those semantics map onto the actual Rust workspace without coupling the core to a particular OS, executor, transport, or middleware?**

```text
Part I
ROS foundation + proposition
        ↓
Part II
Core concepts
        ↓
Part III
Core runtime / kernel boundary
        ↓
Part IV+
Workspace, lifecycle, transport, time,
safety, goals, and unified execution
```

## Canonical rule

> **NROS treats execution, communication, state, resources, lifecycle, and effects as explicit runtime semantics; implementation mechanisms are replaceable beneath those contracts.**
