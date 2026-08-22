# Part XXXIV — State Machines, Transitions, Invariants, Reconciliation & Convergence

> **Series:** NROS Architecture Series  
> **Part:** XXXIV  
> **Role:** Authoritative state, transitions, invariants, snapshots, derived state, reconciliation, conflict handling, convergence, and recovery  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXXIII established event semantics, causality, ordering, delivery, replay, and subscriptions. Part XXXIV defines how NROS represents authoritative state, permits state transitions, derives state from events, detects invalid or divergent state, reconciles replicas, resolves conflicts, and establishes convergence where the architecture requires it.

The central rule is:

> **NROS treats state as governed by explicit transition systems: every authoritative transition has defined preconditions, effects, invariants, provenance, and recovery semantics; derived and replicated state must never be mistaken for authoritative truth merely because it is locally observable.**

## 2. Fundamental Distinctions

```text
state
  ≠
event
  ≠
transition
  ≠
snapshot
  ≠
derived state
  ≠
authoritative state
```

## 3. State

State is the set of values that describe a system, resource, agent, session, or domain object at a defined point in its lifecycle.

```text
State
 ├─ identity
 ├─ version/generation
 ├─ fields
 ├─ lifecycle state
 ├─ invariants
 └─ provenance
```

## 4. Authoritative State

Authoritative state is state whose source is recognized by the architecture as having authority to establish the current truth for its scope.

```text
Authoritative Source
        ↓
Authoritative State
```

Local observation does not automatically confer authority.

## 5. Derived State

Derived state is computed from another authoritative source:

```text
Authoritative State / Events
          ↓
       Derivation
          ↓
     Derived State
```

Derived state can become stale and must have an explicit refresh or reconciliation path.

## 6. State Machine

A state machine defines:

```text
States
 + Events/Inputs
 + Preconditions
 + Transitions
 + Effects
 + Invariants
```

A transition is valid only when the applicable preconditions and invariants are satisfied.

## 7. Transition

Conceptually:

```text
S0 + Input
   ↓
Preconditions
   ↓
Transition
   ↓
S1 + Effects
```

The transition should be atomic with respect to the authority that owns the state.

## 8. Preconditions

Preconditions may include:

```text
current state
identity
capability
resource ownership
version/generation
required dependencies
policy
```

A failed precondition must prevent the transition rather than merely producing a warning.

## 9. Postconditions

A successful transition establishes explicit postconditions:

```text
Transition(S0, E) = S1
```

The implementation must not claim success when postconditions are not established.

## 10. Invariants

Invariants are properties that must remain true across valid transitions:

```text
ValidState(S)
```

and:

```text
ValidState(S0)
 ∧ ValidTransition(S0, E)
    ⇒
ValidState(S1)
```

## 11. State Versioning

State should have a monotonic version or generation where stale updates must be detected:

```text
Generation 7
    ↓
Transition
    ↓
Generation 8
```

Generation values are scoped and must not be assumed globally unique.

## 12. Compare-and-Swap Semantics

Optimistic state updates may use:

```text
Expected generation = 7
Actual generation   = 8
        ↓
Reject stale update
```

This prevents lost updates when multiple actors modify the same state.

## 13. State Ownership

Every authoritative state domain should have an explicit owner or authority:

```text
State Domain
     ↓
Authority
```

Multiple writers require a defined coordination model rather than accidental last-writer-wins behavior.

## 14. Single-Authority State

For simple domains:

```text
One authority
     ↓
Serialized transitions
     ↓
Authoritative state
```

This minimizes conflict complexity.

## 15. Multi-Authority State

When multiple authorities can modify related state:

```text
Authority A ─┐
             ├─ shared domain
Authority B ─┘
```

NROS must explicitly define coordination, partitioning, or conflict resolution.

## 16. State and Events

A state transition may produce an event:

```text
S0
 ↓ transition
S1
 ↓
event
```

The event should describe the committed occurrence rather than merely the attempted transition.

## 17. Event-Driven State Reconstruction

Where events are authoritative history:

```text
Initial State
    ↓
Event 1
    ↓
Event 2
    ↓
Event 3
    ↓
Current State
```

The reducer must obey event ordering and schema contracts.

## 18. Reducer

A reducer computes state transitions from events:

```text
Reduce(S, E) → S'
```

A deterministic reducer is required when replicas are expected to reconstruct equivalent state.

## 19. Deterministic Reduction

For the same valid initial state and equivalent ordered event history:

```text
Reduce(S0, History)
    =
Reduce(S0, History)
```

across conforming implementations.

## 20. Snapshot

A snapshot captures state at a defined position:

```text
Snapshot @ N
```

It must identify the corresponding event/log position or equivalent state version when replay correctness depends on it.

## 21. Snapshot Integrity

A snapshot should preserve:

```text
state version
generation
source/log position
schema version
creation metadata
integrity metadata
```

Without this context, a snapshot may be impossible to safely reconcile.

## 22. State Checkpoints

Checkpoints establish recoverable progress:

```text
Process
 ↓
Checkpoint @ N
 ↓
Continue
```

Crash recovery can resume from the latest valid checkpoint subject to Part XXVII persistence rules.

## 23. State Transition Errors

Invalid transitions should have stable categories:

```text
invalid current state
precondition failure
stale generation
unauthorized
resource unavailable
invariant violation
conflict
```

## 24. Illegal State Prevention

Prefer preventing illegal states at transition boundaries:

```text
Input
 ↓ validate
 ↓ authorize
 ↓ transition
 ↓ invariant check
 ↓ commit
```

rather than allowing invalid intermediate states to become authoritative.

## 25. Atomicity

A state transition should not expose a partially committed authoritative state:

```text
Before
  ↓
Atomic transition
  ↓
After
```

Crash consistency is governed jointly by Parts XXVII and XXXIII.

## 26. Idempotent Transitions

A transition may be idempotent when repeated application has the same state effect:

```text
T(T(S)) = T(S)
```

Only transitions whose semantics actually satisfy this property should be marked idempotent.

## 27. Idempotency Keys

For externally retried operations:

```text
Request ID / Idempotency Key
          ↓
Already applied?
          ↓
Return existing result / execute once
```

The key's retention scope must cover the relevant retry window.

## 28. Conflict Detection

Conflicts occur when concurrent changes cannot be safely merged:

```text
Base S
 ├─ update A → SA
 └─ update B → SB
```

The architecture must identify whether A and B commute, conflict, or require an explicit merge.

## 29. Conflict Classes

Conflicts may be:

```text
version conflict
ownership conflict
semantic conflict
resource conflict
authorization conflict
ordering conflict
schema conflict
```

## 30. Last-Writer-Wins

Last-writer-wins is not a universal conflict-resolution strategy.

If used, the ordering authority and timestamp semantics must be explicit, and semantic loss must be accepted by the domain.

## 31. Merge Functions

A merge function combines concurrent states:

```text
Merge(SA, SB) → SM
```

It must preserve domain invariants or reject the merge.

## 32. Commutativity

Operations may safely reorder when:

```text
A(B(S)) = B(A(S))
```

Commutativity can reduce coordination requirements but must be proven for the relevant state domain.

## 33. Associativity

For merge/reduction operations:

```text
Merge(A, Merge(B, C))
 =
Merge(Merge(A, B), C)
```

Associativity enables deterministic distributed aggregation where applicable.

## 34. Idempotence of Merge

A safe merge may require:

```text
Merge(S, S) = S
```

This is particularly useful under repeated synchronization.

## 35. Convergence

Replicas converge when, after receiving equivalent authoritative information and applying the defined reconciliation rules, they reach equivalent state:

```text
Replica A ─┐
           ├─ reconciliation ─→ equivalent state
Replica B ─┘
```

## 36. Strong Convergence

Strong convergence requires equivalent replicas to become equivalent without requiring identical delivery timing, when the model permits it.

The exact guarantee must be stated rather than assumed.

## 37. Eventual Convergence

Under eventual delivery and stable inputs:

```text
No new conflicting updates
        ↓
Repeated reconciliation
        ↓
Convergent state
```

The assumptions are part of the guarantee.

## 38. Reconciliation

Reconciliation compares observed state with authoritative or peer state:

```text
Local State
     ↓ compare
Reference State
     ↓
Diff
     ↓
Reconciliation Plan
```

## 39. Reconciliation Authority

The system must identify which side is authoritative when states differ:

```text
Local ≠ Remote
      ↓
Authority decision
      ↓
repair / merge / reject
```

Without an authority model, “reconciliation” can simply overwrite valid state arbitrarily.

## 40. Repair

Repair transforms invalid or stale state toward the authoritative state:

```text
Invalid / stale
      ↓
Repair
      ↓
Valid state
```

Repairs should be observable and auditable where they affect important resources.

## 41. Reconciliation Safety

Reconciliation must not bypass:

```text
authorization
resource ownership
capability checks
state invariants
schema validation
```

A synchronization channel is not a privileged bypass by default.

## 42. Split-Brain

If two authorities independently accept conflicting state:

```text
Authority A → SA
Authority B → SB
```

NROS must detect and resolve the divergence according to the domain's coordination model.

## 43. Fencing

Generation or epoch numbers can fence stale authorities:

```text
Epoch 10 → valid
Epoch 9  → rejected
```

This prevents a disconnected former leader from continuing to mutate authoritative state.

## 44. Leases

Leases can bound authority in time:

```text
Lease granted
 ↓
Authority valid
 ↓
Lease expires
 ↓
Authority invalid
```

Lease expiry must be based on a defined clock model and safety margin.

## 45. Epochs

Epochs identify authority generations:

```text
Epoch 1
 ↓
Epoch 2
 ↓
Epoch 3
```

An old epoch must not silently regain authority after a newer epoch has been established.

## 46. State Reconciliation and Events

Reconciliation may produce events:

```text
Conflict detected
 ↓
Resolution
 ↓
Reconciled state
 ↓
Reconciliation event
```

The event should identify whether it records observation, decision, or committed repair.

## 47. Derived State Refresh

Derived state may be rebuilt:

```text
Authoritative source
       ↓
Recompute
       ↓
Derived state
```

A derived cache should be disposable when its source remains authoritative.

## 48. Cache Correctness

Caches must define:

```text
freshness
invalidation
version binding
staleness tolerance
rebuild strategy
```

A cache hit must not be mistaken for authoritative truth unless explicitly defined as such.

## 49. State and Resources

Part XXVIII resource generations should be reflected where stale references can affect state transitions:

```text
Resource R
Generation 4
```

An update referencing generation 3 should be rejected or explicitly reconciled.

## 50. State and Capabilities

A valid state transition requires both:

```text
ValidStateTransition
 ∧
AuthorizedActor
```

A technically valid transition is not automatically an authorized transition.

## 51. State and Identity

State changes should preserve attribution:

```text
Actor identity
 + authority
 + transition
 + resulting state
```

Identity is not merely a display label; it can be part of audit and authorization semantics.

## 52. State and Sessions

Session state is itself a state machine:

```text
Negotiating
 ↓
Established
 ↓
Active
 ↓
Draining
 ↓
Closed
```

Part XXXI defines the session protocol; Part XXXIV generalizes state-machine semantics across NROS domains.

## 53. State and Persistence

Authoritative transitions must respect durable commit semantics:

```text
Transition
 ↓
persist / commit
 ↓
acknowledge
```

An operation must not be reported as durably committed before the persistence contract establishes it.

## 54. State and Recovery

Recovery may use:

```text
snapshot
 + event log
 + checkpoints
 + reconciliation
```

The recovery process must restore a state satisfying the same invariants as normal execution.

## 55. Recovery from Corruption

If state fails validation:

```text
Detect
 ↓
Quarantine
 ↓
Recover from trusted source
 ↓
Validate
 ↓
Commit repaired state
```

Corrupt state must not automatically become the new authority.

## 56. State Machine Composition

Complex state machines can be composed:

```text
System
 ├─ Session state machine
 ├─ Resource state machine
 ├─ Agent state machine
 └─ Policy state machine
```

Cross-machine transitions require explicit coordination semantics.

## 57. Atomic Cross-Domain Transitions

When multiple state domains must change together:

```text
Domain A
 + Domain B
 + Domain C
       ↓
Atomic transaction / coordination
```

If atomicity is impossible, the architecture must define compensating or saga-like semantics instead of pretending the update is atomic.

## 58. Compensation

A failed multi-step operation may execute a compensating transition:

```text
T1 → T2 → T3 fails
             ↓
          Compensate
             ↓
          Safe state
```

Compensation is not necessarily equivalent to rollback.

## 59. Saga Semantics

Long-running workflows may use:

```text
Step 1
 ↓
Step 2
 ↓
Step 3
 ↓ failure
Compensating actions
```

Each compensation must have explicit semantics and authorization.

## 60. Monotonic State

Some domains can guarantee monotonic growth:

```text
S0 ⊆ S1 ⊆ S2
```

Monotonic state can simplify distributed convergence.

## 61. Tombstones and Deletion

Deletion may require explicit tombstones to prevent stale replicas from resurrecting removed state:

```text
Object X
 ↓
Tombstone X
```

Tombstones require retention long enough to cover the stale-replica window.

## 62. Garbage Collection of State

State and tombstones may eventually be collected only when the system can establish that no valid replica or replay path requires them.

## 63. State Authority Transfer

Authority may move:

```text
Authority A
 ↓ handoff
Authority B
```

A safe handoff requires fencing or equivalent mechanisms preventing both authorities from accepting conflicting writes.

## 64. Reconciliation Scheduling

Reconciliation may be:

```text
periodic
triggered by conflict
triggered by reconnect
operator initiated
continuously streaming
```

The schedule must match freshness and convergence requirements.

## 65. Bounded Reconciliation

Reconciliation work must respect resource limits:

```text
max records
max bytes
max duration
max retries
```

A pathological divergence must not consume unlimited system resources.

## 66. Observability

State transitions should expose safe diagnostics:

```text
state/version
transition type
actor identity
precondition result
conflict class
reconciliation result
source event/offset
```

Sensitive state values should not be logged merely for observability.

## 67. Auditability

Important authoritative transitions should be attributable:

```text
Who
What
When
From which version
To which version
Why / triggering event
```

Audit records are themselves data contracts governed by Part XXXII.

## 68. Formal Transition Invariant

```text
ValidState(S0)
 ∧ Preconditions(S0, E)
 ∧ Authorized(E)
 ∧ ValidTransition(S0, E)
    ⇒
ValidState(S1)
```

## 69. Formal Stale-Write Invariant

```text
ExpectedGeneration ≠ ActualGeneration
    ⇒
RejectOrReconcile(Update)
```

A stale update must not silently overwrite newer authoritative state.

## 70. Formal Convergence Invariant

Under the declared convergence assumptions:

```text
EquivalentInputs(A, B)
 ∧ StableAuthority
 ∧ EventualDelivery
 ∧ DeterministicReconciliation
    ⇒
EquivalentState(A, B)
```

The assumptions are part of the guarantee.

## 71. Formal Recovery Invariant

```text
Recover(Snapshot, Events)
    ⇒
ValidState(Result)
```

Recovery is not successful merely because a process restarts; it must restore an invariant-valid state.

## 72. Verification Matrix

| Property | Verification question |
|---|---|
| Authority | Is the authoritative source explicit? |
| State | Are state domains and versions defined? |
| Transition | Are preconditions/effects/postconditions explicit? |
| Invariants | Can invalid states be detected/prevented? |
| Generation | Are stale writes fenced? |
| Events | Are committed transitions distinguishable from attempts? |
| Reducer | Is event-to-state reduction deterministic where required? |
| Snapshot | Is snapshot position/version explicit? |
| Recovery | Can state be reconstructed safely? |
| Conflict | Are conflict classes defined? |
| Merge | Are merge semantics justified? |
| Reconciliation | Is authority explicit during divergence? |
| Convergence | Are convergence assumptions and guarantees explicit? |
| Split-brain | Are stale authorities fenced? |
| Caches | Is derived-state staleness bounded and recoverable? |
| Persistence | Are durable transitions crash-consistent? |
| Security | Do reconciliation and transitions preserve authorization? |
| Observability | Are important transitions diagnosable safely? |
| Formal assurance | Are transition, stale-write, recovery, and convergence invariants explicit? |

## 73. What Part XXXIV Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a universal state-machine framework;
- formally verified transition systems;
- automatic conflict-free merging;
- global strong convergence;
- universal distributed transactions;
- automatic split-brain recovery;
- complete state reconciliation services;
- formally verified reducers for every event stream;
- production-grade saga orchestration;
- universal authoritative-state replication.

Those require implementation-specific evidence.

## 74. Transition to Part XXXV

Part XXXIV establishes authoritative state and convergence semantics.

Part XXXV should define **workflow and orchestration semantics: long-running operations, jobs, tasks, dependencies, retries, compensation, scheduling, cancellation, deadlines, and durable workflow state**, connecting NROS state machines with execution coordination.

```text
Part XXXIII
Events + causality + ordering + delivery + replay + subscriptions
        ↓
Part XXXIV
State machines + transitions + invariants + reconciliation + convergence
        ↓
Part XXXV
Workflows + orchestration + jobs + retries + compensation + scheduling
```

## Canonical rule

> **NROS treats authoritative state as a governed state machine: transitions require explicit preconditions and authorization, preserve invariants, advance scoped generations, emit committed events where required, reject or reconcile stale writes, and use deterministic reconciliation and recovery rules whenever replicas or derived state diverge.**
