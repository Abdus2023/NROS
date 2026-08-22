# Part LXVIII — Storage, Persistence, Durability, Crash Consistency & Transactions

> **Series:** NROS Architecture Series  
> **Part:** LXVIII  
> **Role:** Persistence semantics, durability, write ordering, crash consistency, transactions, journals, storage recovery, retention, compaction, and durable-state governance  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LXVII established distributed state, replication, consistency, and convergence. Part LXVIII defines the persistence substrate on which authoritative state survives process failure, node failure, restart, partial writes, and storage faults.

The central rule is:

> **NROS must distinguish in-memory state, persisted state, durable state, committed state, and externally observable state; a successful write operation must never imply stronger durability or atomicity than the storage contract actually provides.**

## 2. Persistence Model

```text
Application State
 ↓
Mutation
 ↓
Persistence Boundary
 ↓
Storage
 ↓
Durability
 ↓
Recovery
```

## 3. State Classes

```text
Volatile
Persisted
Durable
Committed
Replicated
Observed
```

These classes are not interchangeable.

## 4. Persistence Boundary

Every durable state transition should identify the boundary at which the system considers data recoverable after the relevant failure model.

## 5. Write

A write records or attempts to record state in a persistence substrate.

```text
WriteRequested
    ≠
WritePersisted
    ≠
WriteDurable
```

## 6. Flush

A flush requests movement of buffered state toward the underlying persistence layer.

Flush semantics must be defined by the storage implementation.

## 7. Sync

A synchronization primitive may provide stronger durability than ordinary buffered writes, but the exact guarantee remains substrate-dependent.

## 8. Durability

Durability means that state survives the declared failure model.

```text
Durable(S,F)
    ⇒
Recover(S) after F
```

within the documented scope.

## 9. Failure Model

Every durability claim should identify whether it covers:

```text
process crash
runtime restart
node failure
OS failure
power loss
storage-device failure
replica loss
corruption
```

## 10. Atomicity

Atomicity means a transaction's declared state transition is observed as all-or-nothing within its boundary.

## 11. Transaction Boundary

```text
Transaction
 ↓
Defined Persistence Scope
 ↓
Commit / Abort
```

The boundary must be explicit.

## 12. Commit

A commit establishes that the transaction's required persistence conditions have been satisfied according to the storage contract.

## 13. Abort

An aborted transaction must not expose a partial committed state within the declared atomicity boundary.

## 14. Write Ordering

When ordering affects recovery, writes must have explicit ordering semantics.

```text
A → B
```

must not be interpreted as durable ordering unless the persistence protocol guarantees it.

## 15. Write-Ahead Logging

A WAL-style protocol records mutation intent before exposing dependent durable state.

```text
Mutation
 ↓
WAL
 ↓
Data Pages
 ↓
Commit
```

## 16. Journal

A journal records state transitions or recovery information required to reconstruct consistent state.

## 17. Journal Sequence

Journal entries should carry an explicit sequence, epoch, or equivalent ordering identity.

## 18. Journal Integrity

Recovery should validate journal integrity before applying entries.

Possible mechanisms include:

```text
checksum
hash chain
MAC
signature
sequence validation
```

## 19. Torn Write

A crash may leave a partially written storage unit.

The persistence layer must detect or prevent torn state where correctness depends on atomic writes.

## 20. Crash Consistency

Crash consistency defines which states are recoverable after interruption at arbitrary points in the write sequence.

## 21. Crash Point Analysis

For a multi-step update:

```text
A
 ↓
B
 ↓
C
```

recovery semantics should be defined for crashes after A, after B, and after C.

## 22. Recovery Point

Recovery reconstructs the latest state that satisfies the persistence contract rather than simply the latest bytes observed on disk.

## 23. Checkpoint

A checkpoint captures a recoverable state at a declared version or sequence.

## 24. Snapshot

A snapshot provides a compact representation of state from which recovery can begin.

## 25. Snapshot Validation

Before use, a snapshot should be validated for:

```text
integrity
schema version
sequence
epoch
completeness
compatibility
```

## 26. Snapshot + Journal

```text
Snapshot(V)
    +
Journal(V+1...N)
    =
Recoverable State(N)
```

## 27. Recovery Ordering

Journal replay must respect declared mutation ordering and dependency constraints.

## 28. Idempotent Replay

Recovery operations should be idempotent or use durable progress markers to avoid duplicate state transitions.

## 29. Recovery Marker

A durable marker may record the highest safely applied journal sequence.

## 30. Commit Record

Where required, a commit record should distinguish prepared state from committed state.

## 31. Two-Phase Commit Boundary

If distributed transactions are used, NROS must explicitly model:

```text
prepare
commit
abort
recovery
```

and the associated failure states.

## 32. Distributed Transaction Limits

A local storage transaction does not automatically provide atomicity across independent storage systems.

## 33. External Persistence

External systems require their own transaction, idempotency, or reconciliation contracts.

## 34. Read Consistency

A read should identify the consistency level under which its value is returned.

Possible states include:

```text
local
cached
snapshot
replica
quorum
authoritative
```

## 35. Stale Reads

A stale read is valid only when the consumer's consistency contract permits it.

## 36. Read-Your-Writes

When required, a caller should not observe a state older than its own acknowledged writes.

## 37. Monotonic Reads

A consumer should not move backward to an older state version within a contract requiring monotonic reads.

## 38. Versioned Reads

Reads may carry an expected or minimum state version.

```text
Read(min_version = V)
```

## 39. Compare-and-Swap

Conditional persistence operations can prevent lost updates:

```text
ExpectedVersion = V
 ↓
Write
 ↓
Version = V+1
```

## 40. Optimistic Concurrency

Optimistic concurrency detects conflicting updates through versions, epochs, or comparable tokens.

## 41. Lost Update

A storage protocol must prevent or explicitly tolerate concurrent writes that overwrite each other unintentionally.

## 42. Serialization

Persisted data requires explicit serialization semantics.

```text
Logical State
 ↓
Encoding
 ↓
Bytes
```

## 43. Schema Version

Every persisted format requiring evolution should identify its schema version.

## 44. Migration

Storage migrations should define:

```text
source version
target version
compatibility
rollback
failure recovery
validation
```

## 45. Migration Atomicity

A failed migration must not leave the persistent store in an unrecognized state.

## 46. Forward Compatibility

Readers may support newer fields or formats only when explicitly designed to do so.

## 47. Backward Compatibility

New writers should not silently invalidate readers that remain within the supported compatibility contract.

## 48. Corruption

Storage corruption must be detectable where integrity is required.

## 49. Corruption Response

Possible responses include:

```text
reject
restore snapshot
replay journal
fail over
quarantine
repair
```

## 50. Quarantine

Corrupted or unverifiable persistence artifacts should not automatically become authoritative state.

## 51. Replication Interaction

Persistence and replication must define ordering between local durability and replica publication.

Possible policies include:

```text
persist → replicate
replicate → persist
persist + replicate transactionally
```

Each has different failure semantics.

## 52. Durable Before Publish

For critical state, NROS may require local durability before publishing an externally visible event.

## 53. Publish Before Durable

If publication precedes durability, the protocol must account for observers receiving state that may disappear after crash.

## 54. Outbox Integration

A durable outbox can connect state transitions with outgoing messages:

```text
Transaction
 ↓
State + Outbox Record
 ↓
Commit
 ↓
Publisher
```

## 55. Inbox Integration

A durable inbox can connect accepted incoming messages with recoverable processing state.

## 56. Garbage Collection

Obsolete persistent state may be reclaimed only after no active contract requires it.

## 57. Retention

Retention policy must consider:

```text
recovery
replication
audit
rollback
legal / operational policy
```

## 58. Compaction

Compaction may replace historical records with a newer equivalent representation.

## 59. Compaction Safety

Compaction must preserve all state required by active recovery, replication, and audit contracts.

## 60. Log Truncation

A journal may be truncated only after all required consumers have safely advanced beyond the removed range.

## 61. Watermarks

A durable watermark can identify the earliest sequence that remains required.

## 62. Storage Quotas

Persistent storage consumption must be bounded by explicit quota or capacity policy where exhaustion could threaten system safety.

## 63. Full Storage

Storage exhaustion must be a first-class failure state.

Possible policies:

```text
reject writes
shed optional data
compact
pause workloads
fail over
alert
```

## 64. Priority Under Storage Pressure

Critical state must not necessarily be evicted merely because optional data arrived later.

## 65. Temporary Storage

Temporary state should be clearly distinguished from durable state.

## 66. Cache

Caches must not be mistaken for authoritative persistence unless the cache contract explicitly provides durability.

## 67. Memory-Mapped State

Memory mapping does not by itself establish durability semantics.

## 68. Filesystem Semantics

Filesystem behavior must not be generalized across platforms without evidence.

## 69. Object Storage

Object stores may provide different atomicity, consistency, and overwrite semantics from local filesystems.

## 70. Rename / Replace

Atomic replacement assumptions must be validated against the actual storage substrate.

## 71. Directory Metadata

File content durability and directory-entry durability may have different guarantees.

## 72. Permissions

Persistent artifacts must retain or re-establish required authorization and ownership semantics after recovery.

## 73. Encryption at Rest

Sensitive persistent state may require encryption at rest according to the trust model.

## 74. Key Dependency

Encrypted state recovery depends on key availability and validity.

## 75. Key Rotation

Storage encryption key rotation must preserve access to historical encrypted data required by retention and recovery contracts.

## 76. Secure Deletion

Deletion semantics must identify whether removal means logical deletion, cryptographic erasure, physical reclamation, or merely dereferencing.

## 77. Auditability

Critical persistence operations should provide evidence for:

```text
write
commit
migration
recovery
repair
compaction
retention
purge
```

## 78. Recovery Evidence

Recovery should record the snapshot, journal range, schema version, and policy versions used to reconstruct state where auditability is required.

## 79. Deterministic Recovery

Given the same valid snapshot, journal, schema, and recovery policy, recovery should produce the same logical state where deterministic recovery is required.

## 80. Recovery Failure

If recovery cannot establish a valid state, NROS must not silently promote uncertain data to authoritative state.

## 81. Read-Only Recovery

A system may enter read-only degraded mode when reads remain safe but writes cannot be trusted.

## 82. Fail-Safe Storage

When persistence integrity is uncertain, safety-sensitive state should prefer explicit refusal over silent corruption.

## 83. Transaction Timeout

A transaction timeout does not prove that the transaction did not commit remotely.

The result may be unknown and require reconciliation.

## 84. Unknown Commit Outcome

```text
Commit
 ↓
Connection Loss
 ↓
Unknown
```

The caller must reconcile before unsafe retry of non-idempotent operations.

## 85. Transaction Retry

Retrying a timed-out transaction requires a transaction identity or equivalent idempotency mechanism where duplicate commit is dangerous.

## 86. Storage Failure Isolation

A failing storage subsystem should not automatically corrupt unrelated persistence domains.

## 87. Failure Domains

Persistent resources should identify their failure domain where replication and recovery depend on independence.

## 88. Replica Durability

A replicated write should specify whether success requires:

```text
local durability
one replica
quorum
all replicas
```

## 89. Durability vs Replication

```text
Replication
    ≠
Durability
```

Multiple volatile replicas do not automatically establish durable persistence.

## 90. Durability vs Availability

Stronger durability requirements can reduce availability under partition or storage failure.

The tradeoff must be explicit.

## 91. Persistence Priority

Persistence policies may classify state as:

```text
critical
important
recoverable
reconstructable
ephemeral
```

## 92. Reconstructable State

State that can be deterministically regenerated need not necessarily receive the same persistence guarantees as irreplaceable state.

## 93. Irreplaceable State

State that cannot be reconstructed requires stronger retention and recovery guarantees.

## 94. Durable State Ownership

Each persistent artifact should have an owning lifecycle and authority model.

## 95. Lifecycle Interaction

When an owning workload terminates, persistent state should follow explicit policy:

```text
retain
archive
transfer
revoke
purge
```

## 96. Persistence Security

Storage recovery must not bypass the capability and authorization boundaries defined by earlier Parts.

## 97. Formal Durability Invariant

```text
Durable(S)
    ⇒
Recover(S, DeclaredFailureModel)
```

## 98. Formal Commit Invariant

```text
Committed(T)
    ⇒
AllRequiredPersistenceConditions(T)
```

## 99. Formal Recovery Invariant

```text
Recover(S)
    ⇒
ValidSchema(S)
 ∧
ValidIntegrity(S)
 ∧
ValidSequence(S)
 ∧
ValidEpoch(S)
```

## 100. Formal Compaction Invariant

```text
Compact(R)
    ⇒
NoActiveContractRequires(R)
```

## 101. Verification Matrix

| Property | Verification question |
|---|---|
| Persistence | Is the persistence boundary explicit? |
| Durability | Is the failure model defined? |
| Atomicity | Is the transaction boundary explicit? |
| Ordering | Are write-order guarantees documented? |
| WAL | Can crash recovery reconstruct committed state? |
| Integrity | Can corruption be detected? |
| Recovery | Are snapshot and journal semantics defined? |
| Migration | Is schema evolution recoverable? |
| Concurrency | Are lost updates prevented or explicit? |
| Replication | Is durability distinguished from replication? |
| Outbox | Can state/message consistency survive crashes? |
| Inbox | Can accepted messages survive receiver failure? |
| Quota | Is storage exhaustion handled? |
| Retention | Are recovery and audit dependencies respected? |
| Compaction | Can required history be reclaimed safely? |
| Security | Are authorization and encryption preserved? |
| Key lifecycle | Can historical encrypted state remain recoverable? |
| Recovery evidence | Can reconstruction be independently explained? |
| Unknown outcomes | Are commit uncertainties reconciled? |
| Fail-safe | Can uncertain persistence avoid becoming authoritative? |

## 102. What Part LXVIII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- universal transactional persistence;
- complete crash-consistent storage;
- production WAL infrastructure;
- universal filesystem atomicity;
- distributed transactions across arbitrary stores;
- complete corruption detection and repair;
- universal durable inbox/outbox support;
- production-grade storage encryption and key rotation;
- complete retention and compaction automation.

Those require implementation-specific evidence.

## 103. Transition to Part LXIX

Part LXVIII establishes storage and persistence semantics.

Part LXIX should define the **observability, telemetry, tracing, metrics, logging, evidence, diagnostics, and audit plane** required to make NROS behavior measurable and independently verifiable.

```text
Part LXVII
Distributed state + replication + consistency + convergence
        ↓
Part LXVIII
Storage + persistence + durability + crash consistency
        ↓
Part LXIX
Observability + telemetry + tracing + evidence + audit
```

## Canonical rule

> **NROS persistence is a contract over recoverability: writes, commits, durability, ordering, transactions, replication, snapshots, journals, migrations, retention, and recovery must each expose only the guarantees actually established by their persistence boundary and declared failure model.**
