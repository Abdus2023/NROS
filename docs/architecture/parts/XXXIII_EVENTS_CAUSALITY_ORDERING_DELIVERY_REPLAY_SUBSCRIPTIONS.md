# Part XXXIII — Events, Causality, Ordering, Delivery, Replay & Subscriptions

> **Series:** NROS Architecture Series  
> **Part:** XXXIII  
> **Role:** Event semantics, causality, ordering, delivery guarantees, acknowledgement, deduplication, replay, subscriptions, durable event logs, and state reconstruction  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXXII established data contracts and representation semantics. Part XXXIII defines the temporal and reactive semantics of NROS: what constitutes an event, how events relate causally, how ordering is represented, what delivery guarantees mean, how consumers acknowledge and deduplicate events, and how durable histories can be replayed to reconstruct state.

The central rule is:

> **NROS must distinguish an event from a message, command, notification, and state: an event records an occurrence, carries an explicit identity and causal context, and has defined ordering, durability, delivery, replay, and subscription semantics.**

## 2. Fundamental Distinctions

```text
event
  ≠
message
  ≠
command
  ≠
notification
  ≠
state
  ≠
cause
  ≠
effect
```

## 3. Event

An event represents an occurrence that has happened or has been committed under a defined contract:

```text
Event
 ├─ event ID
 ├─ event type
 ├─ producer identity
 ├─ timestamp / logical time
 ├─ causal context
 ├─ sequence / ordering metadata
 ├─ schema/version
 └─ payload
```

An event's payload describes the occurrence; it is not automatically a command to perform another operation.

## 4. Command vs Event

A command requests an action:

```text
Command → perform operation
```

An event reports an occurrence:

```text
Event → operation occurred / state changed
```

Conflating the two can create incorrect retry and audit semantics.

## 5. Notification vs Event

A notification may be transient and advisory.

An event may be durable, replayable, and part of an authoritative history.

```text
notification
  ≠
durable event
```

The contract must state which semantics apply.

## 6. Event Identity

Every durable or deduplicatable event should have a stable identifier within its authority scope:

```text
Event ID
 + producer identity
 + generation/incarnation
```

Event IDs must not be reused in a way that can cause distinct occurrences to be mistaken for duplicates.

## 7. Event Type

An event type identifies its semantic contract:

```text
resource.created
resource.released
agent.started
agent.stopped
session.established
policy.changed
```

Event types should be governed and versioned.

## 8. Producer Identity

Events should preserve the identity of their producer where required:

```text
Producer
 + incarnation
 + event ID
```

The producer identity is metadata and must not automatically imply authority over the event's subject.

## 9. Subject

An event may identify the resource, agent, session, or entity affected:

```text
Producer → Event → Subject
```

Producer and subject may be different identities.

## 10. Event Time

An event may carry several temporal concepts:

```text
occurrence time
creation time
commit time
delivery time
processing time
```

These must not be silently treated as interchangeable.

## 11. Logical Time

Distributed events may require logical timestamps such as:

```text
Lamport-style logical time
vector-style causal time
sequence numbers
```

The chosen mechanism must match the required ordering guarantees.

## 12. Causality

Causality expresses that one event contributed to another:

```text
Event A
  ↓
causal relation
  ↓
Event B
```

Causality does not necessarily imply physical-time ordering.

## 13. Causal Context

An event may carry:

```text
parent event ID
causal span/context
logical clock
trace context
producer sequence
```

This enables reconstruction of distributed relationships.

## 14. Partial Ordering

Distributed events may be only partially ordered:

```text
A → C
B → D
```

without requiring:

```text
A < B < C < D
```

The protocol must not invent total ordering when only partial ordering is available.

## 15. Total Ordering

A system may establish a total sequence:

```text
1: A
2: B
3: C
```

A total order requires an explicit authority or algorithm, such as a log leader or consensus mechanism.

## 16. Ordering Scope

Ordering must state its scope:

```text
per producer
per subject
per stream
per partition
per session
per log
per cluster
```

“Ordered” without a scope is insufficient.

## 17. Producer Sequence

A producer may attach a monotonically increasing sequence:

```text
Producer P
 1 → E1
 2 → E2
 3 → E3
```

Gaps may indicate loss, filtering, or producer failure depending on the contract.

## 18. Event Log

A durable event log records events in a defined sequence:

```text
Log
 ├─ offset 0 → E0
 ├─ offset 1 → E1
 ├─ offset 2 → E2
 └─ ...
```

The log's ordering semantics must be explicit.

## 19. Offset

An offset identifies a position in a log or partition:

```text
partition P
 offset 42
```

An offset is not necessarily globally unique outside its partition/context.

## 20. Partitioning

A log may be partitioned:

```text
Partition A
Partition B
Partition C
```

Ordering is then typically guaranteed within a partition unless a stronger contract exists.

## 21. Delivery Semantics

NROS should explicitly distinguish:

```text
at-most-once
at-least-once
exactly-once effect / processing
```

“Exactly once delivery” should not be claimed merely because duplicates are rare or hidden.

## 22. At-Most-Once

The consumer receives an event zero or one time:

```text
send → maybe delivered
```

This may sacrifice availability or durability guarantees for lower duplication complexity.

## 23. At-Least-Once

An event may be delivered multiple times:

```text
E
 ↓
E, E, E
```

Consumers therefore need deduplication or idempotent processing when duplicates are unsafe.

## 24. Exactly-Once Effects

Exactly-once effects require coordination between event consumption and side-effect commitment:

```text
consume
 ↓
process
 ↓
commit effect + progress atomically
```

A transport alone cannot establish exactly-once application effects.

## 25. Acknowledgement

Acknowledgement indicates a defined processing milestone:

```text
received
processed
persisted
committed
```

The protocol must specify which milestone an acknowledgement represents.

## 26. Negative Acknowledgement

A consumer may explicitly reject or defer an event:

```text
ACK
NACK
RETRY / DEFER
```

Failure semantics must prevent infinite retry loops.

## 27. Consumer Position

A durable consumer may maintain:

```text
consumer ID
partition
last acknowledged offset
processing state
```

Position state should obey Part XXVII persistence semantics.

## 28. Deduplication

Deduplication may use:

```text
event ID
producer + sequence
partition + offset
idempotency key
```

The deduplication key must match the event's uniqueness contract.

## 29. Deduplication Window

Deduplication may be bounded:

```text
seen IDs
 ↓ expiry
forget old IDs
```

The expiry window must be compatible with the maximum expected redelivery/replay interval.

## 30. Replay

Replay re-delivers historical events:

```text
Log
 ↓ from offset N
Replay
 ↓
Consumer
```

Replay is distinct from ordinary redelivery of an unacknowledged event.

## 31. Replay Position

A replay request may specify:

```text
offset
sequence
timestamp
checkpoint
snapshot + offset
```

The interpretation must be deterministic within the log contract.

## 32. Replay Safety

Replay must not automatically repeat irreversible side effects.

Consumers should support:

```text
read-only reconstruction
idempotent processing
side-effect suppression
transactional replay mode
```

## 33. Snapshot + Replay

State reconstruction can use:

```text
Snapshot at N
      ↓
Events N+1 ... M
      ↓
Current state
```

This bounds replay cost while preserving event history semantics.

## 34. Event Sourcing Boundary

If events are authoritative state history:

```text
Events
 ↓ fold/reduce
State
```

If events are merely notifications:

```text
Authoritative State
 ↓
Notification
```

The architecture must explicitly identify which model applies.

## 35. Event Ordering and State Reconstruction

A state reducer must consume events according to the ordering contract required by the state machine.

```text
Correct order
 ↓
valid state transition
```

Receiving events out of order may require buffering, reordering, or rejection.

## 36. Out-of-Order Events

Consumers may encounter:

```text
E3 arrives
E1 arrives
E2 arrives
```

The consumer must know whether to:

```text
buffer
reorder
apply with causal checks
reject
```

## 37. Missing Events

A sequence gap may indicate:

```text
loss
partitioning
filtering
producer restart
retention
```

The consumer should not silently assume missing events are irrelevant.

## 38. Retention

Event history may have retention policies:

```text
retention duration
maximum bytes
maximum events
compaction rules
archival policy
```

Retention must be compatible with replay and audit requirements.

## 39. Compaction

Compaction may remove redundant historical events while preserving reconstructable state:

```text
E1 E2 E3 E4
    ↓ compact
Snapshot / reduced history
```

Compaction is safe only when the resulting history preserves the declared semantics.

## 40. Tombstones

For compacted state, deletion may require an explicit tombstone:

```text
Object X
 ↓
Tombstone(X)
```

Without a tombstone, older replicas may incorrectly resurrect deleted state.

## 41. Subscription

A subscription defines what events a consumer wants:

```text
Subscribe
 ├─ event types
 ├─ subjects
 ├─ filters
 ├─ starting position
 └─ delivery contract
```

Subscription authorization remains governed by Part XXIX.

## 42. Subscription Lifecycle

```text
Requested
 ↓
Authorized
 ↓
Active
 ↓
Paused
 ↓
Resumed
 ↓
Cancelled
```

Subscription state must be durable when durable delivery is required.

## 43. Filtering

Filtering may occur at:

```text
producer
broker/log
consumer
```

The architecture must define whether filtered events still advance consumer positions.

## 44. Backpressure

A slow subscriber must not cause unbounded global buffering:

```text
slow consumer
 ↓
bounded queue
 ↓
backpressure / drop / disconnect / persist
```

The chosen behavior must be explicit.

## 45. Event Priorities

If priorities exist, they must not accidentally violate required causal or ordering guarantees.

Priority is a scheduling property, not automatically a semantic ordering rule.

## 46. Event Authorization

Consumers must be authorized to receive protected events:

```text
Subscription
 + identity
 + capability
 + policy
      ↓
Delivery permission
```

Event visibility must follow the same isolation and capability boundaries as other resources.

## 47. Event Redaction

Sensitive event payloads may require:

```text
field redaction
subject filtering
scope-specific projection
```

Redaction must preserve enough metadata for safe interpretation without leaking protected data.

## 48. Event Integrity

Durable event records may require integrity protection:

```text
event
 + schema/version
 + causal context
 + sequence
      ↓
integrity metadata
```

This helps detect alteration or accidental corruption.

## 49. Event Provenance

An event may preserve:

```text
producer
origin
causal parent
transformation history
schema version
```

Derived events should not falsely appear to originate from their source event producer.

## 50. Event Transformation

A processor may transform:

```text
E1
 ↓ processor
E2
```

E2 should retain provenance linking it to E1 where traceability matters.

## 51. Event Batching

Events may be batched:

```text
Batch
 ├─ E1
 ├─ E2
 └─ E3
```

Batch acknowledgement semantics must specify whether acknowledgement covers all events or a subset.

## 52. Event Transactions

A transaction may atomically produce multiple events:

```text
Transaction T
 ├─ E1
 ├─ E2
 └─ E3
```

Consumers need explicit visibility semantics for partially committed transactions.

## 53. Transaction Visibility

Possible models include:

```text
read uncommitted
read committed events
atomic transaction visibility
```

The selected model must be explicit.

## 54. Event and Persistence

Part XXVII governs durable event logs, offsets, snapshots, and recovery:

```text
Event
 ↓ durable commit
Log
 ↓ checkpoint
Consumer state
```

Crash recovery must not lose acknowledged events or incorrectly acknowledge uncommitted processing.

## 55. Event and Distributed Coordination

Part XXV may provide ordering or consensus for authoritative event streams:

```text
Consensus / leader
       ↓
ordered log
       ↓
events
```

Not every event stream requires consensus; the required guarantee must be justified.

## 56. Event and Networking

Part XXVI provides transport delivery, but application delivery guarantees remain separate:

```text
Transport delivery
      ≠
Event processing guarantee
```

A reliable transport does not automatically create exactly-once event effects.

## 57. Event and Protocol Sessions

Part XXXI establishes session semantics; events operate within or across those sessions:

```text
Session
 ↓
Event stream
 ↓
Consumer
```

Session termination must define whether event delivery resumes, restarts, or terminates.

## 58. Event and Data Contracts

Part XXXII defines event schema and encoding:

```text
Event semantics
 ↓
Schema
 ↓
Canonical encoding
 ↓
Transport / persistence
```

Schema evolution must preserve declared event semantics or explicitly version the event type.

## 59. Event and Agents

Agent execution can consume events:

```text
Event
 ↓ Observe
 ↓ Plan
 ↓ Execute
 ↓ Reflect
 ↓ Checkpoint
```

Events may therefore become inputs to the NROS agent scheduling model without becoming implicit commands.

## 60. Causal Traceability

A distributed trace may represent:

```text
Cause A
 ↓
Event B
 ↓
Action C
 ↓
Event D
```

Stable IDs and causal metadata should permit reconstruction where required.

## 61. Eventual Consistency

In eventually consistent systems:

```text
Event observed at node A
      ↓
later observed at node B
```

Consumers must not infer global completion merely from local observation.

## 62. Watermarks

A stream may expose a watermark indicating that events up to a defined point are expected to be complete under a contract:

```text
watermark = W
```

Watermark semantics must define whether late events remain possible.

## 63. Late Events

Late events may arrive after a watermark or expected ordering point.

The system should define whether they are:

```text
accepted
retracted
recomputed
quarantined
ignored
```

## 64. Event Failure Handling

Failed processing may result in:

```text
retry
park / dead-letter
skip with audit
rollback
operator intervention
```

Automatic retry must be bounded.

## 65. Dead-Letter Events

A dead-letter record should preserve enough context for diagnosis:

```text
original event ID
failure class
attempt count
consumer identity
time
source position
```

Sensitive payloads remain subject to Part XXIX controls.

## 66. Poison Events

An event that repeatedly fails processing must not permanently block unrelated progress unless ordering semantics require it.

Partitioning and quarantine strategies should be considered.

## 67. Event Lifecycle

A durable event may transition through:

```text
Created
 ↓
Committed
 ↓
Available
 ↓
Delivered
 ↓
Acknowledged
 ↓
Retained / Archived
 ↓
Expired / Compacted
```

These states must not be conflated.

## 68. Formal Event Identity Invariant

```text
DistinctOccurrences(E1, E2)
    ⇒
EventIdentity(E1) ≠ EventIdentity(E2)
```

within the event identity scope.

## 69. Delivery Invariant

For an at-least-once stream:

```text
Committed(E)
    ⇒
E remains available for redelivery
until the retention/acknowledgement contract permits removal.
```

## 70. Replay Invariant

```text
Replay(Log, position=N)
```

must produce a sequence consistent with the log's declared ordering and retention semantics.

## 71. Causality Invariant

If the event contract records a causal dependency:

```text
Cause(E1, E2)
    ⇒
E1 precedes E2 in the declared causal relation.
```

This does not require physical timestamps to be ordered in the same way.

## 72. Verification Matrix

| Property | Verification question |
|---|---|
| Event identity | Is every event uniquely identifiable within scope? |
| Semantics | Is event meaning distinct from commands and notifications? |
| Producer | Is origin attributable? |
| Subject | Is affected entity distinguishable from producer? |
| Time | Are occurrence, commit, delivery, and processing times distinct? |
| Causality | Can causal relationships be represented? |
| Ordering | Is ordering scope explicit? |
| Delivery | Is at-most/at-least/exactly-once semantics explicit? |
| ACK | Is acknowledgement meaning defined? |
| Deduplication | Is duplicate identity unambiguous? |
| Replay | Can historical events be replayed safely? |
| State | Can state reconstruction semantics be established? |
| Retention | Is replay availability bounded explicitly? |
| Compaction | Does compaction preserve declared semantics? |
| Subscription | Are filters, positions, and authorization explicit? |
| Backpressure | Are slow consumers bounded? |
| Security | Are event visibility and redaction enforced? |
| Persistence | Are log and consumer checkpoints crash-consistent? |
| Distribution | Are ordering/consensus requirements justified? |
| Observability | Can event provenance and processing state be reconstructed? |
| Formal assurance | Are event/causality/delivery invariants explicit? |

## 73. What Part XXXIII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a universal event bus;
- durable event sourcing everywhere;
- exactly-once application effects;
- global total ordering;
- universal replay;
- complete subscription management;
- production-grade consumer checkpointing;
- formally verified causal ordering;
- universal event retention/compaction;
- complete dead-letter handling.

Those require implementation-specific evidence.

## 74. Transition to Part XXXIV

Part XXXIII defines event semantics and temporal behavior.

Part XXXIV should define **state machines, transitions, invariants, reconciliation, convergence, conflict handling, and derived state**, connecting event streams to authoritative state and distributed recovery.

```text
Part XXXII
Serialization + schemas + encoding + validation + data evolution
        ↓
Part XXXIII
Events + causality + ordering + delivery + replay + subscriptions
        ↓
Part XXXIV
State machines + transitions + invariants + reconciliation + convergence
```

## Canonical rule

> **NROS treats events as explicit temporal facts: every durable event has defined identity, provenance, causal context, ordering scope, delivery semantics, and retention behavior; replay and subscription mechanisms must preserve those contracts, while consumers must not confuse delivery with processing, observation with global truth, or an event with an implicit command.**
