# Part XXVII — Persistence, Durability & Crash Consistency

> **Series:** NROS Architecture Series  
> **Part:** XXVII  
> **Role:** Persistent state, write paths, durability, crash consistency, journals, WAL, snapshots, replication at rest, recovery, compaction, garbage collection, and durable state lifecycle  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXVI defined networking and transport. Part XXVII defines how NROS state survives process termination, host failure, restart, partial writes, storage errors, and recovery.

The central rule is:

> **NROS must distinguish persisted, durable, committed, replicated, recoverable, and consistent state; a successful write must have an explicitly defined durability point and recovery contract.**

## 2. Fundamental Distinctions

```text
persisted
  ≠
durable
  ≠
committed
  ≠
replicated
  ≠
recoverable
  ≠
consistent
```

These properties may coincide in some implementations but must never be assumed equivalent.

## 3. Persistence Boundary

The architectural path is:

```text
Application State
      ↓
Write Intent
      ↓
Validation
      ↓
Persistence Layer
      ↓
Storage
      ↓
Durability Boundary
```

The durability boundary must be observable through an explicit contract.

## 4. Persistent State

Persistent state may include:

```text
configuration
identity metadata
runtime state
queues
checkpoints
protocol state
membership state
execution journals
application data
security metadata
```

Each category should define retention, integrity, confidentiality, and recovery requirements.

## 5. State Classes

NROS may distinguish:

```text
volatile state
recoverable state
persistent state
durable state
replicated durable state
```

A state class should specify what failure modes it survives.

## 6. Write Path

A conceptual write path is:

```text
Request
 ↓
Validate
 ↓
Transform
 ↓
Journal / WAL
 ↓
Storage update
 ↓
Durability barrier
 ↓
Commit / acknowledge
```

Actual ordering may differ by storage engine, but the externally visible semantics must remain explicit.

## 7. Write Intent vs Commit

A requested state change is not necessarily committed merely because it was accepted.

```text
requested
  ↓
validated
  ↓
accepted
  ↓
persisted
  ↓
durable
  ↓
committed
```

The implementation may collapse stages, but the contract must identify the observable point.

## 8. Durability

Durability answers:

> After the system reports success, which failures may occur without losing the acknowledged state?

Possible failure boundaries include:

```text
process crash
runtime restart
OS crash
host power loss
storage restart
single-device failure
replica failure
```

## 9. Durability Contract

A durability contract should define:

```text
acknowledgement point
storage medium
flush semantics
ordering guarantees
failure model
recovery procedure
replication requirements
```

## 10. Flush and Sync

A storage API's “write completed” signal is not automatically equivalent to durable media persistence.

The contract must distinguish:

```text
buffered
submitted
written to cache
flushed
persisted to medium
replicated
```

## 11. Atomicity

Atomicity defines whether a state transition is observed as:

```text
old state
or
new state
```

rather than an invalid intermediate state.

Atomicity scope must be explicit:

```text
field
record
file
transaction
journal entry
snapshot
multi-resource operation
```

## 12. Crash Consistency

A crash can occur between any two persistence steps:

```text
write A
 ↓
CRASH
 ↓
write B never completes
```

Recovery must produce a state allowed by the persistence contract.

## 13. Write-Ahead Logging

A WAL can represent intended changes before applying them:

```text
Intent
 ↓
WAL
 ↓
Data pages
 ↓
Commit
```

The WAL contract must specify ordering, durability, record identity, truncation, and recovery.

## 14. Journal Records

A journal record may contain:

```text
record identity
sequence number
transaction / operation identity
state transition
schema version
integrity metadata
commit marker
```

Sensitive data must follow Part XXII security requirements.

## 15. Journal Ordering

Journal sequence numbers can establish durable ordering:

```text
J100
 ↓
J101
 ↓
J102
```

The sequence should not be confused with wall-clock time.

## 16. Recovery Journal

Recovery may replay committed or otherwise valid journal entries:

```text
Snapshot S
 + J100
 + J101
 + J102
 ↓
Recovered state
```

The replay semantics must be deterministic where required by Part XXIV.

## 17. Idempotent Recovery

Recovery may encounter repeated or partially processed records.

Where required:

```text
Apply(J, Apply(J,S))
    ≡
Apply(J,S)
```

This connects Part XXVI retry semantics with persistent recovery.

## 18. Transactions

A transaction may provide:

```text
atomicity
consistency
isolation
durability
```

but NROS must define which guarantees actually apply and at what scope.

## 19. Local vs Distributed Transactions

Local persistence guarantees do not automatically extend across nodes.

```text
local commit
   ≠
distributed commit
```

Part XXV governs distributed agreement and commit semantics.

## 20. Snapshots

A snapshot captures a consistent state at a defined boundary:

```text
State at T
   ↓
Snapshot S
```

The snapshot contract should define consistency point, metadata, version, integrity, and recovery semantics.

## 21. Incremental Snapshots

Large state may use incremental snapshots:

```text
Full S0
 + Δ1
 + Δ2
 + Δ3
```

Recovery must validate the complete dependency chain.

## 22. Snapshot Integrity

Snapshots should include sufficient metadata for:

```text
identity
version
schema
configuration
integrity
provenance
creation boundary
```

Corrupt or incompatible snapshots must not silently become trusted state.

## 23. Checkpoint vs Snapshot

The concepts are related but distinct:

```text
checkpoint
    → execution/runtime continuation

snapshot
    → persistent state image
```

A system may combine them, but the semantics should remain explicit.

## 24. Replication at Rest

Durable state may be replicated:

```text
Primary
 ↓
Replica A
Replica B
```

Replication guarantees must specify whether replicas are:

```text
asynchronous
synchronous
quorum-confirmed
transactionally coupled
```

## 25. Replication vs Durability

Replication does not automatically guarantee durability.

For example, multiple replicas sharing one failure domain may fail together.

Therefore durability analysis must include:

```text
failure domain
replication distance
acknowledgement rule
storage medium
```

## 26. Commit vs Visibility

A state may be durably committed but not yet visible to every reader.

```text
committed
   ↓
propagated
   ↓
visible
```

Read consistency is governed by the relevant contract.

## 27. Recovery Point Objective

A persistence design may define how much acknowledged or unacknowledged state can be lost after a failure.

The allowed loss boundary must be explicit rather than implied by “persistent”.

## 28. Recovery Time Objective

Recovery also has a time requirement:

```text
failure
 ↓
recovery start
 ↓
state reconstruction
 ↓
service restoration
```

Recovery duration is a resource and availability property.

## 29. Recovery Ordering

Recovery should define ordering between:

```text
configuration
identity
schema migration
journal replay
state reconstruction
network rejoin
external effect reconciliation
service admission
```

A node should not become fully operational before required safety state is reconstructed.

## 30. External Effects

Persistent recovery may interact with external effects:

```text
journal says effect pending
        ↓
recovery
        ↓
retry / reconcile / suppress
```

Part XXIV replay semantics and Part XXV fencing must determine whether an effect is safe to repeat.

## 31. Exactly-Once State Transitions

Exactly-once durable semantics generally require:

```text
stable operation identity
transactional state transition
deduplication
commit marker
recovery rules
```

Storage durability alone does not establish exactly-once application behavior.

## 32. Compaction

Journals and snapshots may require compaction:

```text
J1 J2 J3 J4 J5
      ↓
Snapshot S
      ↓
retain only required journal suffix
```

Compaction must preserve all information required for recovery and audit contracts.

## 33. Compaction Safety

A journal entry may be removed only when the system can prove that its semantics are preserved by retained state.

This boundary should be explicit and verifiable.

## 34. Garbage Collection

Persistent garbage collection may remove:

```text
obsolete snapshots
expired journals
unreferenced blobs
old checkpoints
retired schema versions
```

Deletion must respect retention, legal/policy, recovery, and audit requirements where applicable.

## 35. Tombstones

Deletion may require durable tombstones to prevent an older replica or stale state source from resurrecting deleted data.

```text
Delete X
 ↓
Tombstone(X)
```

Tombstone retention must be tied to replication and recovery semantics.

## 36. Schema Evolution

Part XXIII governs persistent schema evolution.

A storage migration should define:

```text
old schema
new schema
migration
rollback boundary
mixed-version behavior
historical data interpretation
```

## 37. Migration Safety

A migration should not destroy the only recoverable representation before the new representation has been validated.

Possible strategy:

```text
Old state
 ↓
Copy / transform
 ↓
Validate new state
 ↓
Commit migration
 ↓
Retire old state
```

## 38. Corruption Detection

Persistent state should support integrity detection where required:

```text
checksums
hashes
Merkle structures
authenticated records
replica comparison
```

Detection does not automatically provide correction.

## 39. Corruption Recovery

On detected corruption, policy may choose:

```text
repair
restore snapshot
replay journal
use replica
quarantine
fail closed
```

The chosen behavior depends on state criticality.

## 40. Storage Failure

Storage failures should be classified rather than treated as one generic error:

```text
transient I/O error
capacity exhaustion
permission failure
corruption
device failure
latency degradation
filesystem failure
```

Different failures require different recovery behavior.

## 41. Capacity Exhaustion

Persistent storage exhaustion is a resource-control event:

```text
storage usage
   ↓ threshold
admission control
   ↓
compaction / eviction / rejection
```

Part XXI resource economics therefore applies directly to persistence.

## 42. Backpressure from Storage

Slow storage can propagate backpressure:

```text
storage slows
 ↓
write queue grows
 ↓
application throttled
 ↓
upstream backpressure
```

This connects Part XIII flow control with durable-state capacity.

## 43. Observability

Part XIV should expose facts such as:

```text
journal position
snapshot identity
write latency
flush latency
queue depth
storage capacity
recovery progress
replication lag
compaction state
corruption events
```

These measurements should not be presented as causal explanations without evidence.

## 44. Security

Part XXII applies to persistent data:

```text
confidentiality
integrity
access control
key management
secure deletion
backup protection
snapshot protection
```

A durable copy is still a security-relevant copy.

## 45. Deterministic Recovery

Part XXIV requires recovery to be deterministic where the recovery contract demands it:

```text
same valid snapshot
+
same valid journal
+
same recovery rules
 ↓
equivalent recovered state
```

## 46. Formal Persistence Model

A conceptual state transition is:

```text
Sₙ + Wₙ
   ↓
Persist(Wₙ)
   ↓
Sₙ₊₁
```

A crash may occur before, during, or after persistence.

The recovery function should map every allowed crash point to an allowed state:

```text
Recover(storage_after_crash)
    ∈ ValidRecoveryStates
```

## 47. Durability Invariant

A possible contract is:

```text
AcknowledgedDurable(W)
    ⇒
W survives every failure class included in the durability contract.
```

The failure set must always be stated explicitly.

## 48. Verification Matrix

| Property | Verification question |
|---|---|
| Persistence | What state is actually stored? |
| Durability | Which failures can acknowledged state survive? |
| Commit | What exactly is the commit point? |
| Atomicity | Can partial state become visible? |
| WAL | Is journal ordering and recovery explicit? |
| Snapshot | Is the snapshot internally consistent? |
| Replication | What replication guarantee exists? |
| Recovery | Can every allowed crash point recover safely? |
| Migration | Are schema transitions recoverable? |
| Compaction | Can data be safely discarded? |
| Deletion | Are tombstones/retention rules sufficient? |
| Corruption | Can corruption be detected and handled? |
| Capacity | Is storage exhaustion bounded? |
| Backpressure | Can slow storage throttle producers? |
| Security | Are persistent copies protected? |
| Determinism | Is recovery reproducible where required? |
| Observability | Is recovery state measurable? |
| Formal assurance | Are durability invariants explicit? |

## 49. What Part XXVII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production WAL;
- crash-safe persistence;
- formally verified recovery;
- distributed durable transactions;
- automatic corruption repair;
- production snapshot management;
- topology-independent durability;
- fully reproducible recovery;
- complete storage fault injection.

Those require implementation-specific evidence.

## 50. Transition to Part XXVIII

Part XXVII defines durable state and recovery from storage failures.

Part XXVIII should define **resource lifecycle and memory/storage ownership, allocation, quotas, accounting, reclamation, isolation, and leak prevention**, connecting persistent state to the broader NROS resource model.

```text
Part XXVI
Networking + transport + congestion + topology
        ↓
Part XXVII
Persistence + durability + crash consistency
        ↓
Part XXVIII
Resource lifecycle + ownership + allocation + reclamation
```

## Canonical rule

> **NROS treats durability as a scoped failure-survival contract: every acknowledged state transition must have an explicit persistence, commit, recovery, and integrity boundary, while journals, snapshots, replicas, migrations, compaction, and garbage collection must preserve the state required by the declared correctness and recovery model.**
