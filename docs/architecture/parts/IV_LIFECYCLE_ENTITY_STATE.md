# Part IV — Lifecycle & Entity State

> **Series:** NROS Architecture Series  
> **Part:** IV  
> **Role:** Entity lifecycle and runtime state semantics  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part III established the runtime/kernel boundary. Part IV defines how runtime entities exist, transition between states, fail, recover, and become observable.

The central rule is:

> **A runtime state is a contract, not a label. A transition is valid only when its prerequisites and effects are defined.**

## 2. Entity Model

A runtime entity is any object whose lifecycle is managed or observed by NROS.

Examples include:

```text
Component
Node
Service
Action
Resource
Device
Runtime worker
Deployment unit
Agent
```

Conceptually:

```text
Entity
├── Identity
├── Type
├── Generation
├── Lifecycle state
├── Configuration
├── Dependencies
├── Resources
├── Capabilities
├── Health
└── Fault state
```

Not every entity type needs every field.

## 3. State Is Typed

NROS should avoid one overloaded "status" field.

At minimum, state should distinguish:

```text
Lifecycle state
Execution state
Health state
Fault state
Admission state
```

For example:

```text
Lifecycle: RUNNING
Execution: IDLE
Health: DEGRADED
Fault: NONE
Admission: ADMITTED
```

This avoids ambiguous states such as `RUNNING_BUT_FAILED`.

## 4. Canonical Lifecycle

A baseline lifecycle is:

```text
                    ┌──────────────┐
                    │    CREATED   │
                    └──────┬───────┘
                           │ configure
                           ▼
                    ┌──────────────┐
                    │  CONFIGURED  │
                    └──────┬───────┘
                           │ activate
                           ▼
                    ┌──────────────┐
                    │    READY     │
                    └──────┬───────┘
                           │ admit
                           ▼
                    ┌──────────────┐
                    │   ADMITTED   │
                    └──────┬───────┘
                           │ start
                           ▼
                    ┌──────────────┐
                    │   RUNNING    │
                    └──────┬───────┘
                           │ stop
                           ▼
                    ┌──────────────┐
                    │   STOPPING   │
                    └──────┬───────┘
                           ▼
                    ┌──────────────┐
                    │   STOPPED    │
                    └──────────────┘
```

This is a conceptual baseline. Concrete entity types may require additional states.

## 5. Transition Contract

A lifecycle transition has four parts:

```text
Transition
├── Source state
├── Trigger
├── Preconditions
├── State mutation
├── Side effects
└── Result / event
```

For example:

```text
READY → ADMITTED

Trigger:
    admission request

Preconditions:
    configuration valid
    dependencies satisfied
    required resources available
    policy permits admission

Effects:
    admission recorded
    runtime event emitted
```

A transition is not complete merely because a state field was changed.

## 6. Invalid Transitions

The runtime should reject transitions that violate the lifecycle contract.

Examples:

```text
CREATED → RUNNING          invalid
STOPPED → EXECUTING        invalid
FAULTED → RUNNING          invalid without recovery
```

The exact transition graph may evolve, but invalid transitions must have defined behavior.

Possible outcomes:

```text
Reject
Return error
Emit diagnostic
Record attempted transition
```

## 7. Configuration Boundary

Configuration establishes the parameters under which an entity may operate.

```text
CREATED
   ↓
Configuration
   ↓
Validation
   ↓
CONFIGURED
```

Configuration should be distinguished from runtime state.

```text
Configuration
   ≠
Current state
```

A valid configuration does not guarantee that runtime resources are currently available.

## 8. Readiness

`READY` means the entity satisfies the prerequisites for admission/start according to its contract.

It should not mean:

```text
READY = healthy forever
READY = resource guaranteed forever
READY = running
```

Readiness may depend on:

- configuration;
- dependencies;
- capabilities;
- resource availability;
- platform support;
- policy.

## 9. Admission

Admission is the point where the runtime decides whether work may enter active execution.

```text
READY
  ↓
Requirements
  ↓
Policy
  ↓
Resource availability
  ↓
Dependency state
  ↓
ADMITTED
```

Admission and scheduling are different:

```text
Admission
→ Is this entity/work allowed to enter execution?

Scheduling
→ When and where does eligible work execute?
```

## 10. Running and Execution

`RUNNING` describes lifecycle participation in the active runtime.

It does not imply that the entity is currently consuming CPU.

An entity may be:

```text
RUNNING + IDLE
RUNNING + EXECUTING
RUNNING + BLOCKED
RUNNING + WAITING
```

This is why lifecycle and execution state are separate dimensions.

## 11. Stopping

Stopping is a transition, not an instantaneous state mutation.

```text
RUNNING
   ↓
STOPPING
   ├── cancel active work
   ├── release resources
   ├── close channels
   ├── flush required state
   └── finalize observation
   ↓
STOPPED
```

The runtime should define which cleanup operations are mandatory, optional, or bounded by a deadline.

## 12. Fault State

Fault state represents abnormal conditions independently from lifecycle.

Conceptually:

```text
FAULT
├── NONE
├── DEGRADED
├── FAULTED
├── ISOLATED
└── FATAL
```

A fault may coexist with a lifecycle state.

For example:

```text
Lifecycle = RUNNING
Fault     = DEGRADED
```

or:

```text
Lifecycle = STOPPED
Fault     = FAULTED
```

## 13. Fault Transition

A fault path should be explicit:

```text
RUNNING
   │
   ▼
FAULT DETECTED
   │
   ├── recover
   │      ↓
   │   RECOVERING
   │      ↓
   │   READY / RUNNING
   │
   ├── isolate
   │      ↓
   │   ISOLATED
   │
   └── fatal
          ↓
       STOPPED / SAFE STATE
```

Recovery is not guaranteed. The policy must specify what happens when recovery itself fails.

## 14. Recovery

Recovery may involve:

```text
Restart component
Reinitialize state
Reacquire resources
Reconnect channels
Replay checkpoint
Restore persisted state
Reconcile dependencies
Escalate fault
```

These mechanisms belong to later persistence and supervision architecture; Part IV defines only the lifecycle boundary they must respect.

## 15. Generation and Restart Identity

Restarted entities must not be confused with their previous incarnation.

Conceptually:

```text
EntityId = stable identity
Generation = incarnation
```

Example:

```text
camera-01 / generation 7
camera-01 / generation 8
```

The stable identity can remain constant while the generation changes.

This is important for stale messages, recovery, leases, and distributed coordination.

## 16. Dependencies

Lifecycle transitions may depend on other entities.

```text
Component A
   ↓ requires
Component B
```

The runtime should be able to distinguish:

```text
Dependency declared
Dependency discovered
Dependency ready
Dependency healthy
Dependency available
```

These states are not interchangeable.

## 17. State Invariants

The lifecycle model establishes several invariants.

### L1 — Valid transitions only

Every lifecycle transition must originate from a permitted source state.

### L2 — Preconditions precede state claims

An entity must not claim a state whose prerequisites have not been satisfied.

### L3 — Terminal states are explicit

Stopping/failure/recovery paths must eventually produce an explicit result.

### L4 — Generation changes identify reincarnation

A restarted entity must be distinguishable from its previous execution generation.

### L5 — Lifecycle and execution are separate

`RUNNING` does not mean continuously executing.

### L6 — Fault is orthogonal

Fault state must not be encoded solely through lifecycle state.

### L7 — State transitions are observable where required

Transitions relevant to correctness, recovery, safety, or auditability must have an observation path.

## 18. State Machine Model

A formal transition can be represented as:

```text
T = (S, E, G, A, S')
```

where:

```text
S  = source state
E  = triggering event
G  = guard / precondition
A  = transition actions
S' = destination state
```

A transition is valid iff:

```text
S is current
AND
E is permitted
AND
G evaluates true
```

The transition then produces `S'` and its defined effects.

## 19. Event Ordering

Lifecycle events should carry enough identity to distinguish transitions across concurrent or distributed execution.

Conceptually:

```text
LifecycleEvent
├── event_id
├── entity_id
├── generation
├── previous_state
├── new_state
├── cause
├── timestamp
└── sequence / ordering metadata
```

A timestamp alone does not establish a total ordering.

Distributed ordering requirements belong to later coordination architecture.

## 20. Concurrency

Lifecycle transitions may race.

Example:

```text
Thread A: stop()
Thread B: restart()
Thread C: fault()
```

The runtime therefore needs a defined serialization or arbitration mechanism.

The architecture must answer:

- who owns lifecycle state;
- whether transitions are atomic;
- how concurrent requests are ordered;
- whether stale requests are rejected;
- how generation changes invalidate old operations.

## 21. Stale Operations

An operation associated with an old generation must not silently affect a newer generation.

```text
Operation generation = 7
Current generation   = 8

→ reject / invalidate
```

This rule becomes particularly important when messages, leases, recovery actions, or commands may arrive after a restart.

## 22. Safe State

Some domains require a defined safe state after unrecoverable failure.

```text
FAULT
  ↓
Recovery attempts exhausted
  ↓
SAFE STATE
```

The actual safe state is domain-specific.

For a motor controller it may mean disabling actuation; for a monitoring service it may simply mean stopping data publication.

NROS must provide a lifecycle boundary for such policies without claiming that the runtime itself determines the correct safety behavior for every robot.

## 23. Verification Matrix

| Property | Verification question |
|---|---|
| Transition validity | Are illegal transitions rejected? |
| Preconditions | Can missing prerequisites prevent admission? |
| Atomicity | Can concurrent transitions leave an impossible state? |
| Generation | Are stale operations rejected? |
| Fault isolation | Can fault state be distinguished from lifecycle state? |
| Recovery | Does each recovery path reach a defined terminal outcome? |
| Cancellation | Does stop/cancel reach its required terminal state? |
| Dependencies | Are dependency prerequisites enforced? |
| Events | Are lifecycle transitions observable with correct identity? |
| Safe state | Is the required domain-specific safe-state action invoked? |

## 24. What Part IV Does Not Claim

Part IV does not claim that the current NROS implementation already provides:

- complete lifecycle enforcement;
- distributed lifecycle consensus;
- automatic fault recovery;
- safety-certified safe-state behavior;
- universal atomic state transitions;
- complete generation-aware message invalidation.

Those capabilities require repository-specific implementation and verification evidence.

## 25. Transition to Part V

Part IV establishes the entity state machine.

Part V should define how entities communicate across the runtime boundary while preserving explicit contracts for ownership, ordering, delivery, reliability, backpressure, and transport independence.

```text
Part III
Runtime / kernel boundary
        ↓
Part IV
Lifecycle + entity state
        ↓
Part V
Communication + transport contract
```

## Canonical rule

> **NROS lifecycle state is explicit, transition-driven, generation-aware, and independently observable; execution, health, and fault state must not be collapsed into a single status label.**
