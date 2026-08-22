# Part XLIII — Storage, Persistence, Durability, Consistency & Replication

> **Series:** NROS Architecture Series  
> **Part:** XLIII  
> **Role:** Storage, persistence, durability, transactions, consistency, replication, snapshots, recovery points, crash consistency, and data lifecycle  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XLII established distributed communication semantics. Part XLIII defines how NROS state becomes persistent, durable, consistent, replicated, recoverable, and eventually retired.

The central rule is:

> **NROS never treats a successful write call as proof of durable state: memory, persisted state, durable state, replicated state, committed state, and recoverable state are distinct properties with explicit guarantees.**

## 2. Fundamental Distinctions

```text
memory
  ≠
persistent state
  ≠
durable state
  ≠
replicated state
  ≠
committed state
  ≠
recoverable state
```

## 3. State Pipeline

```text
Application State
      ↓
Write Intent
      ↓
Storage Buffer
      ↓
Persistent Representation
      ↓
Durability Boundary
      ↓
Replication
      ↓
Commit / Visibility
      ↓
Recovery Point
```

Each transition requires explicit semantics.

## 4. State Classes

NROS state can include:

```text
configuration
workflow state
agent state
scheduler state
identity metadata
leases
checkpoints
telemetry evidence
cache state
persistent application data
```

Not every state requires identical durability.

## 5. Volatile State

Volatile state may disappear after process or node failure:

```text
RAM
 ↓ crash
lost
```

Volatility is acceptable only when the state is reconstructible or intentionally disposable.

## 6. Persistent State

Persistent state survives process restart when stored through an appropriate persistence mechanism.

Persistence alone does not prove crash durability.

## 7. Durable State

Durability means the system's declared recovery model guarantees that committed state survives the specified failure boundary.

```text
Commit
 ↓
Durability boundary
 ↓
Recovery guarantee
```

## 8. Durability Boundary

Every durable operation should identify the boundary it guarantees:

```text
process
node
storage device
replica set
region
```

A write durable on one node is not automatically durable across node loss.

## 9. Write Acknowledgement

An acknowledgement should identify what it confirms:

```text
buffered
persisted
flushed
replicated
committed
```

Ambiguous acknowledgements create unsafe recovery assumptions.

## 10. Commit

Commit defines when a state transition becomes authoritative under the relevant consistency model.

```text
Proposed state
 ↓
Validation
 ↓
Commit
 ↓
Authoritative state
```

## 11. Atomicity

A transaction should define whether a set of changes becomes visible atomically:

```text
A + B + C
   ↓
all visible
or
none visible
```

Atomicity scope must be explicit.

## 12. Transaction Identity

Transactions should have stable identifiers:

```text
transaction_id
operation_id
principal
policy_version
```

This supports correlation, deduplication, and auditability.

## 13. Idempotent Writes

Retryable writes should be idempotent where possible.

```text
Write(X)
retry Write(X)
 ↓
same logical result
```

## 14. Write-Ahead Logging

Where WAL is used:

```text
Intent
 ↓
WAL
 ↓
Data update
 ↓
Commit marker
```

Recovery can replay or roll back according to protocol semantics.

## 15. Crash Consistency

A crash may occur between any persistence steps.

```text
write A
 ↓
CRASH
 ↓
partial state
```

Storage formats must define recovery behavior for interrupted operations.

## 16. Torn State

Multi-record or multi-block updates require protection against partially persisted state.

Possible mechanisms include:

```text
journaling
checksums
copy-on-write
atomic rename
transaction logs
versioned records
```

## 17. Checksums

Persistent records may carry integrity metadata:

```text
Record
 ↓
Checksum
 ↓
Validate
```

Corruption must be detected rather than silently interpreted as valid state.

## 18. Versioned State

State records can carry:

```text
version
sequence
epoch
schema version
```

This supports stale-write detection and migration.

## 19. Compare-and-Swap Semantics

Conditional writes may require:

```text
expected_version
new_state
```

The write succeeds only if the expected version still matches.

## 20. Lost Update Protection

Concurrent writers must not silently overwrite each other when the application requires conflict detection.

```text
Read v7
Writer A → v8
Writer B based on v7 → reject/conflict
```

## 21. Consistency

Consistency describes which state observations are permitted relative to concurrent updates.

It is not synonymous with durability.

```text
consistent
    ≠
durable
```

## 22. Visibility

A committed state may become visible according to a defined consistency model:

```text
local
session
causal
strong
eventual
```

The selected model must be explicit per subsystem where necessary.

## 23. Linearization

Operations requiring a single authoritative order need a defined linearization point.

```text
invoke
 ↓
linearization point
 ↓
return
```

## 24. Serializability

Where transactional serializability is required, concurrent transactions must have an outcome equivalent to an allowed serial execution.

## 25. Snapshot Isolation

A snapshot can provide a stable read view while concurrent writes proceed.

Snapshot semantics must define visibility and conflict behavior.

## 26. Read-Your-Writes

A session may require:

```text
Write(X)
 ↓
Read(X)
 ↓
observe X
```

This is a consistency guarantee, not merely a storage guarantee.

## 27. Causal Consistency

If operation B depends causally on A:

```text
A → B
```

observers satisfying causal consistency should not observe B without the relevant causal history.

## 28. Eventual Consistency

Replicas may temporarily disagree:

```text
Replica A = v8
Replica B = v7
```

Convergence requires explicit assumptions and conflict rules.

## 29. Conflict Resolution

Concurrent divergent writes require deterministic handling:

```text
reject
merge
last-writer policy
application-defined resolution
```

Arbitrary conflict resolution is unsafe for critical state.

## 30. Replication

Replication copies state across failure domains:

```text
Primary
 ├── Replica A
 └── Replica B
```

Replication improves availability and durability only under defined assumptions.

## 31. Replication Factor

Replication factor should be explicit:

```text
N replicas
```

A replication factor of N does not itself prove that N independent failure domains exist.

## 32. Failure Domains

Replicas should be distributed according to the intended failure model:

```text
process
node
rack
zone
region
```

Co-located replicas may fail together.

## 33. Synchronous Replication

A write may require remote acknowledgement before commit:

```text
Write
 ↓
Replica ACK
 ↓
Commit
```

Latency and availability implications must be explicit.

## 34. Asynchronous Replication

A primary may commit before replicas receive the update:

```text
Commit primary
 ↓
replication later
```

This creates a replication lag window.

## 35. Replication Lag

Lag should be observable:

```text
primary version = 100
replica version = 97
lag = 3
```

Critical reads may reject replicas whose freshness is insufficient.

## 36. Replica Eligibility

A replica may be excluded from serving reads when:

```text
stale
suspected
corrupt
wrong epoch
under recovery
```

## 37. Primary Election

If leadership changes:

```text
Primary A
 ↓ failure
Election
 ↓
Primary B
```

The old primary must be fenced before it can safely resume writes.

## 38. Split-Brain Storage

Two writers must not both believe they are authoritative.

Epochs, leases, quorum, or fencing must establish authority.

## 39. Storage Epoch

Storage leadership can be fenced with epochs:

```text
Epoch 12
 ↓
Epoch 13
```

Writes carrying stale epochs are rejected.

## 40. Quorum Writes

A quorum write requires an explicitly defined acknowledgement threshold.

```text
N replicas
 ↓
W acknowledgements
 ↓
commit
```

Read quorum requirements must be defined separately.

## 41. Quorum Reads

A read quorum can provide stronger consistency depending on the replication protocol.

Quorum intersection assumptions must be documented rather than inferred.

## 42. Read Repair

A read may detect stale replicas and trigger repair:

```text
Read
 ↓
version mismatch
 ↓
repair
```

Repair must remain bounded and authorized.

## 43. Anti-Entropy

Replicas can periodically compare summaries and synchronize divergent state.

```text
Digest A
 ↕
Digest B
 ↓
Difference
 ↓
Repair
```

## 44. Snapshot

A snapshot captures a defined state view:

```text
State @ epoch E
 ↓
Snapshot
```

Snapshot consistency requirements must be explicit.

## 45. Snapshot Metadata

Snapshots should identify:

```text
snapshot_id
state version
epoch
creation time
schema version
source
integrity metadata
```

## 46. Snapshot Atomicity

A multi-component snapshot must define whether it represents:

```text
single instant
consistent cut
best-effort collection
```

## 47. Incremental Snapshot

Incremental snapshots store changes since a base snapshot:

```text
Base S0
 ↓
Delta S1
 ↓
Delta S2
```

Dependency chains require integrity and retention rules.

## 48. Recovery Point

A recovery point identifies state from which recovery can proceed:

```text
RecoveryPoint
 = known recoverable state
```

A backup that cannot be restored is not sufficient recovery evidence.

## 49. Recovery Point Objective

RPO describes acceptable data loss relative to a failure event.

It must be stated per workload where requirements differ.

## 50. Recovery Time Objective

RTO describes acceptable restoration time.

RTO and RPO are operational requirements, not storage implementation details alone.

## 51. Restore

Restore should validate:

```text
integrity
schema
version
authorization
compatibility
```

before activating recovered state.

## 52. Recovery Validation

```text
Snapshot
 ↓
Integrity check
 ↓
Schema validation
 ↓
Semantic validation
 ↓
Recovery staging
 ↓
Activation
```

## 53. Recovery Isolation

Recovered state should initially be isolated from production authority when corruption or compromise is possible.

## 54. Backup

Backups should define:

```text
scope
frequency
retention
integrity
encryption
restore procedure
failure domain
```

## 55. Backup Independence

A backup stored in the same failure domain as the primary may not protect against that domain's failure.

## 56. Backup Encryption

Sensitive persistent data should be protected according to the security model in Part XLI.

## 57. Key Dependency

Encrypted backups require recoverable key-management procedures.

```text
Backup
 + key availability
 = recoverable backup
```

## 58. Data Lifecycle

Persistent data should have explicit states:

```text
Created
 ↓
Active
 ↓
Archived
 ↓
Expired
 ↓
Deleted
```

## 59. Retention

Retention policies should define:

```text
minimum lifetime
maximum lifetime
legal/operational hold
cleanup behavior
```

## 60. Deletion

Deletion semantics must distinguish:

```text
logical deletion
physical reclamation
replica deletion
backup expiry
```

A logical delete does not necessarily mean immediate physical erasure.

## 61. Tombstones

Replicated systems may need tombstones to prevent deleted state from reappearing during synchronization.

## 62. Garbage Collection

Storage reclamation must not delete state still required by:

```text
replicas
snapshots
transactions
recovery
replication logs
```

## 63. Compaction

Compaction can reduce storage overhead but must preserve required recovery and consistency semantics.

## 64. Storage Pressure

Storage capacity is a resource:

```text
storage pressure
 ↓
admission / compaction / retention / shedding
```

Part XXXVII applies.

## 65. Out-of-Space Failure

The architecture must define behavior when storage cannot accept new state:

```text
Write
 ↓
No capacity
 ↓
Reject / degrade / emergency retention policy
```

Silent partial writes are unacceptable.

## 66. Write Ordering

If operation B depends on A:

```text
A → B
```

storage must preserve or explicitly reconstruct the required ordering relationship.

## 67. Flush Semantics

A flush operation must define what it guarantees:

```text
buffer → OS
OS → device
metadata
all prior writes
```

The implementation must not claim stronger semantics than the underlying mechanism provides.

## 68. Fsync / Durable Commit

Where durable commit is required, the storage layer must establish a concrete durability boundary rather than assuming that a successful API return implies persistence.

## 69. Metadata Durability

File or record data and metadata may have different persistence semantics.

Recovery requirements must account for both.

## 70. Corruption Detection

Corruption checks should cover the state needed for recovery:

```text
records
indexes
metadata
snapshots
logs
```

## 71. Corruption Response

Detected corruption should cause:

```text
Reject
 ↓
Quarantine
 ↓
Repair / Restore
 ↓
Validate
```

rather than silent propagation.

## 72. Schema Migration

Persistent schemas evolve:

```text
Schema N
 ↓ migration
Schema N+1
```

Migration must define compatibility, rollback, and failure recovery.

## 73. Online Migration

Online migration should avoid exposing partially migrated state to incompatible readers.

## 74. Migration Versioning

Records may carry schema versions so readers can select the correct interpretation.

## 75. Storage API

A storage abstraction should distinguish operations such as:

```text
read
write
append
update
compare-and-swap
commit
snapshot
restore
```

## 76. Storage Capability

Access to persistent state should be capability-scoped according to Part XLI:

```text
Principal
 ↓
Storage capability
 ↓
Resource / namespace
 ↓
Operation
```

## 77. Namespace Isolation

Storage namespaces may isolate:

```text
tenants
workflows
agents
services
system state
```

Cross-namespace access requires explicit authority.

## 78. Multi-Tenant Storage

Shared storage must preserve tenant isolation at the storage and authorization layers.

## 79. Cache vs Source of Truth

Caches must be explicitly distinguished from authoritative state:

```text
Cache
 ≠
Source of truth
```

Cache invalidation and staleness semantics must be explicit.

## 80. Derived State

Derived indexes or projections can be reconstructed from authoritative state where possible.

```text
Authoritative state
 ↓
Projection
```

This reduces recovery complexity.

## 81. Event Sourcing

If event sourcing is used:

```text
Event log
 ↓
Projection
 ↓
Current state
```

Events become durable domain facts and require lifecycle, integrity, and schema policies.

## 82. Checkpointing

Long-running execution may periodically persist checkpoints:

```text
Execution
 ↓
Checkpoint
 ↓
Continue
```

Checkpoints should identify execution epoch and relevant configuration/policy versions.

## 83. Agent State

Agent checkpoints may include:

```text
workflow position
approved state
tool-operation identifiers
memory references
policy context
resource state
```

Sensitive information remains subject to Part XLI controls.

## 84. Exactly-Once State Transition

Exactly-once externally observable effects cannot be assumed from storage or transport alone.

It requires an end-to-end protocol connecting:

```text
identity
operation ID
transaction
commit
external effect
reconciliation
```

## 85. External Side Effects

Persistent commit and external side effects can diverge:

```text
DB commit
 ↓
external API fails
```

or:

```text
external API succeeds
 ↓
DB commit fails
```

The architecture must explicitly handle such ambiguity.

## 86. Outbox Pattern

Where appropriate:

```text
State change + outbox record
        ↓ atomic commit
Outbox dispatcher
        ↓
External message
```

This separates durable intent from eventual external delivery.

## 87. Inbox Pattern

Receivers can persist operation identity before applying side effects:

```text
Message
 ↓
Inbox record
 ↓
deduplicate
 ↓
apply effect
```

## 88. Reconciliation

When external state is uncertain:

```text
Local record
 ↕
External state
 ↓
Reconciliation
```

Reconciliation must be deterministic and auditable.

## 89. Storage Observability

Storage should expose:

```text
write latency
read latency
queue depth
flush latency
replication lag
commit latency
errors
corruption indicators
capacity
```

Part XL defines observability requirements.

## 90. Storage Evidence

Durability claims should identify:

```text
operation
storage layer
acknowledgement
failure boundary
replica state
recovery test
```

## 91. Formal Durability Invariant

```text
DurableCommit(X)
    ⇒
Recoverable(X, DeclaredFailureBoundary)
```

## 92. Formal Version Invariant

```text
Write(ExpectedVersion = V)
    ⇒
CurrentVersion = V
```

otherwise the write must fail or explicitly resolve the conflict.

## 93. Formal Stale-Writer Invariant

```text
WriterEpoch < CurrentStorageEpoch
    ⇒
Reject(Write)
```

## 94. Formal Replication Invariant

```text
Commit(X, RequiredReplication=N)
    ⇒
ReplicationRequirementSatisfied(X, N)
```

where the system explicitly defines what counts as an independent replica.

## 95. Formal Recovery Invariant

```text
Restore(Snapshot)
    ⇒
IntegrityVerified(Snapshot)
 ∧
SchemaCompatible(Snapshot)
```

## 96. Verification Matrix

| Property | Verification question |
|---|---|
| Persistence | Does state survive the declared restart boundary? |
| Durability | Does committed state survive the declared failure boundary? |
| Atomicity | Are required multi-state updates atomic? |
| Crash consistency | Can interrupted writes be safely recovered? |
| Integrity | Can corruption be detected? |
| Versioning | Can stale writers be rejected? |
| Consistency | Is the visibility model explicit? |
| Replication | Are replicas actually independent failure domains? |
| Lag | Is replication freshness observable? |
| Election | Can stale leaders be fenced? |
| Snapshot | Does the snapshot represent the promised consistency boundary? |
| Restore | Can snapshots actually be restored and validated? |
| Backup | Are backups independent and recoverable? |
| Retention | Is lifecycle policy explicit? |
| Deletion | Are logical and physical deletion distinguished? |
| Capacity | Is storage pressure bounded and observable? |
| Security | Is storage capability-scoped? |
| Migration | Can schema upgrades fail and recover safely? |
| External effects | Are ambiguous outcomes reconciled? |
| Evidence | Can durability claims be independently verified? |

## 97. What Part XLIII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- production transactional storage;
- a universal WAL implementation;
- consensus-backed replication;
- production snapshots and restore;
- formally verified crash consistency;
- complete backup infrastructure;
- universal exactly-once external effects;
- complete schema migration tooling;
- production multi-tenant storage isolation.

Those require implementation-specific evidence.

## 98. Transition to Part XLIV

Part XLIII establishes persistent-state semantics.

Part XLIV should define **scheduling, concurrency, work distribution, priorities, fairness, admission, deadlines, preemption, and deterministic execution across the NROS runtime**.

```text
Part XLII
Networking + communication + discovery + transport + partitions
        ↓
Part XLIII
Storage + persistence + durability + consistency + replication
        ↓
Part XLIV
Scheduling + concurrency + fairness + deadlines + execution order
```

## Canonical rule

> **NROS treats durable state as an explicit contract: a successful write becomes authoritative only at a defined commit boundary, survives only the failure domains explicitly guaranteed, and remains recoverable, integrity-checked, versioned, and governed by the same security, resource, observability, and temporal constraints as the runtime that produced it.**
