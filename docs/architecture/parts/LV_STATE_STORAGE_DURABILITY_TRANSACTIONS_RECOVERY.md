# Part LV — State, Storage, Durability, Transactions & Recovery

> **Series:** NROS Architecture Series  
> **Part:** LV  
> **Role:** Logical state, storage ownership, mutation, transactions, persistence, snapshots, durability, recovery, consistency, and reconciliation  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LIV established workload and execution semantics. Part LV defines how NROS represents, mutates, persists, recovers, and reconciles state across memory, storage, execution, and failure boundaries.

The central rule is:

> **NROS must distinguish in-memory state, proposed state, committed state, durable state, recovered state, and externally observed state.**

## 2. Fundamental State Distinctions

```text
In-Memory State
    ≠
Proposed State
    ≠
Committed State
    ≠
Durable State
    ≠
Recovered State
    ≠
Observed External State
```

These states may coincide, but they must not be assumed equivalent without evidence.

## 3. Logical State

Logical state represents authoritative domain information independently of its physical storage representation.

Examples include:

```text
workload state
resource allocation state
configuration state
identity state
scheduler state
execution state
```

## 4. State Ownership

Every authoritative state domain should have an explicit owner or authority.

```text
state domain
 ↓
authority
 ↓
mutation rights
```

## 5. State Identity

State should be identifiable using appropriate metadata:

```text
state_id
revision
owner
schema_version
epoch
```

## 6. Revision

Mutations should advance an observable revision when the state model requires versioning.

```text
Revision 10
 ↓ mutation
Revision 11
```

## 7. Revision Ordering

A revision may establish ordering within one authority domain. It does not automatically establish global distributed ordering.

## 8. State Mutation

A mutation should have an explicit lifecycle:

```text
Read
 ↓
Validate
 ↓
Propose
 ↓
Commit
 ↓
Persist
 ↓
Publish
```

The exact ordering may vary by consistency model but must be defined.

## 9. Read Semantics

A read should identify its consistency semantics where ambiguity matters:

```text
local
cached
committed
linearizable
snapshot
stale-acceptable
```

## 10. Stale Reads

A stale read must not be silently treated as authoritative current state.

## 11. Write Authority

Only authorized actors may mutate authoritative state.

```text
Authenticated
 ∧
Authorized
 ∧
CorrectEpoch
```

## 12. Transactions

A transaction groups mutations under a declared atomicity and consistency boundary.

## 13. Transaction Identity

Each transaction may carry:

```text
transaction_id
actor
start_revision
commit_revision
scope
outcome
```

## 14. Atomicity

For an atomic transaction:

```text
all mutations commit
OR
none become authoritative
```

Partial visibility must not occur where atomicity is promised.

## 15. Isolation

Isolation defines what concurrent readers and writers can observe.

Possible semantics include:

```text
serializable
snapshot
read committed
read uncommitted
```

The implementation must not imply stronger semantics than actually provided.

## 16. Consistency

Consistency is a property of a declared state model, not a generic synonym for correctness.

## 17. Durability

Durability means that a committed state survives the failures covered by the declared durability contract.

```text
Commit
  ≠
theoretical persistence
```

The actual persistence boundary must be explicit.

## 18. Commit Point

A state mutation needs a well-defined commit point.

```text
Before commit
 → not authoritative

After commit
 → authoritative according to the transaction contract
```

## 19. Persistence Boundary

The architecture must define when committed state becomes durable under the applicable storage contract.

## 20. Write-Ahead Logging

Where used, a WAL can establish an ordered recovery record:

```text
Intent
 ↓
Log
 ↓
Commit
 ↓
Apply
```

The WAL is an implementation mechanism; the durability contract is the architectural requirement.

## 21. Journal

A journal may contain:

```text
sequence
operation
actor
revision
payload reference
integrity metadata
```

## 22. Journal Ordering

Ordering must be explicit within the authority domain.

## 23. Journal Integrity

Durability records may require integrity protection:

```text
hash
checksum
authenticated record
chain
```

The selected mechanism depends on the threat and failure model.

## 24. Snapshot

A snapshot captures a coherent state at a defined revision or consistency boundary.

```text
Snapshot S
    ↓
Revision R
```

## 25. Snapshot Consistency

A snapshot must not combine unrelated state revisions while claiming to represent one coherent state unless the model explicitly permits it.

## 26. Incremental Snapshot

Large state domains may use incremental snapshots with explicit base relationships.

```text
Snapshot 10
   ↓ delta
Snapshot 11
```

## 27. Snapshot Integrity

Snapshots should carry sufficient metadata to verify:

```text
identity
revision
schema
integrity
source
```

## 28. Schema Version

Persisted state must identify the schema version required to interpret it.

## 29. Schema Migration

State migrations should be explicit:

```text
Schema N
 ↓ migration
Schema N+1
```

Migration must not silently reinterpret incompatible data.

## 30. Forward Compatibility

Readers may support older persisted schemas where explicitly designed.

## 31. Backward Compatibility

Writers must not produce state that older required readers cannot safely interpret when backward compatibility is part of the contract.

## 32. Recovery

Recovery reconstructs authoritative state after failure.

```text
Durable Snapshot
      ↓
Replay Journal
      ↓
Reconstruct State
      ↓
Validate
      ↓
Publish Recovered State
```

## 33. Recovery Point

Recovery should identify the durable point from which reconstruction begins.

## 34. Recovery Point Objective

If an RPO is defined, it describes the maximum acceptable loss of committed state under the specified failure model.

## 35. Recovery Time Objective

If an RTO is defined, it describes the required recovery-time bound.

These are requirements, not evidence of achievement.

## 36. Crash Recovery

After process failure:

```text
Detect
 ↓
Fence stale writer
 ↓
Load durable state
 ↓
Replay / reconcile
 ↓
Validate
 ↓
Resume authority
```

## 37. Storage Failure

Storage failures require explicit handling:

```text
retry
failover
read-only mode
quarantine
fail
```

## 38. Partial Write

A partially written record must not be interpreted as a valid committed state unless the storage protocol guarantees that property.

## 39. Torn State

Recovery must detect or reject incomplete state according to the storage integrity contract.

## 40. Double Commit

A transaction must not accidentally become two distinct authoritative mutations because of retries or uncertain acknowledgement.

## 41. Idempotent Mutation

Mutations that may be retried should support idempotency keys or equivalent transaction identity.

## 42. Commit Acknowledgement Loss

If the client loses the commit response, it must be possible to determine whether the transaction committed without blindly issuing a duplicate mutation.

## 43. Compare-and-Swap

Optimistic state mutation may use:

```text
expected_revision
new_state
```

and reject stale writers.

## 44. Conflict

```text
Writer A based on Revision 10
Writer B commits Revision 11
Writer A submits based on 10
        ↓
Conflict
```

The conflict policy must be explicit.

## 45. Conflict Resolution

Possible policies include:

```text
reject
retry
merge
last-writer-wins
authority-defined resolution
```

Last-writer-wins must not be used where it can violate safety invariants.

## 46. Leases

State ownership may use leases, but lease expiry must fence stale mutation authority.

## 47. Epoch-Bound State

Authoritative state mutations can require a valid epoch:

```text
writer epoch 4
current epoch 5
      ↓
reject mutation
```

## 48. Replication

Replicated state should distinguish:

```text
local copy
replica acknowledged
quorum committed
authoritative committed
```

## 49. Replication Lag

A replica can be behind the authoritative revision.

```text
Authority: R100
Replica:   R97
```

## 50. Read-From-Replica

A read from a lagging replica must expose or constrain its staleness where required by the caller.

## 51. Quorum

If quorum semantics are used, the quorum definition must be explicit:

```text
members
threshold
failure assumptions
commit rule
```

## 52. Split Brain

Two partitions must not both independently claim authoritative ownership of the same state domain when the consistency model forbids it.

## 53. Fencing

Authority transfer requires fencing of stale writers before the new authority performs protected mutations where necessary.

## 54. Reconciliation

After recovery or partition healing:

```text
Observed State
      ↓
Compare
      ↓
Authoritative State
      ↓
Reconcile
```

## 55. Reconciliation Policy

Differences may result in:

```text
repair
accept authoritative state
quarantine
manual intervention
failure
```

## 56. External State

Some state exists outside NROS:

```text
hardware
external database
remote service
filesystem
network peer
```

NROS cannot claim ownership merely because it issued an operation.

## 57. Desired vs Observed State

```text
Desired State
    ≠
Observed External State
```

This mirrors the configuration distinction established in Part LII.

## 58. Reconciliation Loop

```text
Observe
 ↓
Compare
 ↓
Plan
 ↓
Mutate
 ↓
Observe
```

Reconciliation must be bounded and policy-controlled.

## 59. Convergence

A reconciliation controller should define what convergence means.

## 60. Non-Convergent State

Repeated mutation without convergence may indicate:

```text
configuration conflict
external interference
insufficient authority
unstable policy
fault
```

## 61. Tombstones

Deleted distributed state may require tombstones so that stale replicas do not resurrect removed objects.

## 62. Garbage Collection

Tombstones, journals, snapshots, and historical state may require retention and garbage-collection policies.

## 63. Retention

Retention should define:

```text
minimum lifetime
maximum lifetime
legal/operational constraints
space policy
```

## 64. Compaction

Compaction may reduce historical storage while preserving the recovery guarantees of the state model.

## 65. Compaction Safety

Compaction must not remove information still required for:

```text
recovery
replication
reconciliation
audit
```

## 66. State Garbage Collection

Garbage collection must respect references from active workloads, snapshots, checkpoints, and recovery mechanisms.

## 67. Transaction Timeout

Long-running transactions should have explicit timeout or cancellation semantics where supported.

## 68. Deadlocks

If transactional locking exists, deadlock handling must be defined.

Possible responses:

```text
avoidance
detection
abort
retry
```

## 69. Lock Ownership

Locks should identify their owner and, where appropriate, lease or epoch information.

## 70. Lock Recovery

A crashed owner must not permanently retain a lock unless retention is intentional and recoverable.

## 71. Atomic Multi-Domain Mutation

When one logical operation spans multiple state domains, NROS must explicitly define whether it provides:

```text
single transaction
saga/compensation
eventual convergence
best-effort coordination
```

It must not imply global atomicity accidentally.

## 72. Eventual Consistency

Where eventual consistency is selected, the architecture should define:

```text
convergence target
staleness bounds where applicable
conflict policy
failure behavior
```

## 73. Linearizability

If a state API claims linearizable operations, the implementation must provide a defensible linearization point and evidence model.

## 74. Snapshot Reads

Snapshot reads should identify the snapshot revision or equivalent consistency boundary.

## 75. MVCC

Multi-version state may support concurrent readers and writers without forcing all readers onto the newest state.

## 76. Version Retention

MVCC versions require bounded retention or explicit historical-state policy.

## 77. Cache

Caches are accelerators, not automatically authoritative state.

```text
Cache
 ≠
Source of Truth
```

## 78. Cache Invalidation

Where cache correctness matters, invalidation or version validation must be explicit.

## 79. Write-Through / Write-Back

If caching is combined with writes, the architecture must define whether persistence occurs:

```text
before cache acknowledgement
or
after deferred persistence
```

## 80. State Publication

Committed state may be published to observers through events or watches.

Publication is not itself the commit point unless explicitly defined.

## 81. Watch Semantics

Watch streams should define:

```text
starting revision
ordering
replay
loss behavior
resynchronization
```

## 82. Event Loss

Consumers must be able to detect missed state events and recover through snapshot or replay mechanisms.

## 83. Snapshot + Stream

A robust observation pattern is:

```text
Snapshot at R
 ↓
Stream from R+1
```

This prevents consumers from depending on an infinitely retained event stream.

## 84. State Checkpoints

Execution checkpoints from Part LIV are state artifacts and therefore inherit:

```text
schema
integrity
ownership
retention
compatibility
```

## 85. Durable Workload Completion

A workload should not claim durable completion before the required result state crosses its declared persistence boundary.

## 86. Commit vs Acknowledgement

```text
Client acknowledgement
    ≠
Durable commit
```

unless the API explicitly guarantees equivalence.

## 87. Recovery Authority

Recovery must establish who owns authoritative state after reconstruction.

## 88. Recovery Epoch

Recovered state may enter a new epoch to fence stale pre-failure actors.

## 89. Recovery Validation

Recovered state should be checked for:

```text
schema validity
integrity
revision consistency
security invariants
resource references
workload references
```

## 90. Recovery Ordering

Dependent state should recover in an order consistent with its authority relationships.

For example:

```text
identity / authority
 ↓
configuration
 ↓
resource state
 ↓
workload state
 ↓
execution state
```

The exact implementation order may differ, but dependencies must be explicit.

## 91. Recovery and External Effects

Durable local state cannot prove that an external side effect completed.

External effect reconciliation may require:

```text
query
idempotency key
transaction protocol
compensation
manual reconciliation
```

## 92. Recovery Ambiguity

When state is uncertain, NROS should represent uncertainty rather than fabricate success.

## 93. Unknown Outcome

```text
Operation issued
 ↓
Response lost
 ↓
Outcome unknown
```

The state model should permit an explicit unknown/reconciling state where required.

## 94. Recovery Safety Gate

Before declaring a recovered subsystem operational:

```text
Durable State Loaded
 ∧
Schema Valid
 ∧
Authority Established
 ∧
Stale Writers Fenced
 ∧
Required Reconciliation Complete
```

## 95. State Evidence

Evidence should distinguish:

```text
written
committed
durable
recovered
observed
```

## 96. State Metrics

Useful metrics include:

```text
commit latency
persistence latency
journal size
snapshot age
replication lag
recovery duration
recovery point
reconciliation backlog
conflict count
```

## 97. Formal Commit Invariant

```text
Committed(T)
    ⇒
Authorized(T)
 ∧
AtomicitySatisfied(T)
```

## 98. Formal Revision Invariant

```text
Write(ExpectedRevision = R)
    ∧
CurrentRevision != R
    ⇒
RejectOrExplicitlyResolveConflict
```

## 99. Formal Recovery Invariant

```text
Recovered(S)
    ⇒
SchemaValid(S)
 ∧
IntegrityValid(S)
 ∧
AuthorityEstablished(S)
```

## 100. Formal Fencing Invariant

```text
OldEpoch < CurrentEpoch
    ⇒
Reject(AuthoritativeMutation)
```

## 101. Formal Durability Invariant

```text
Durable(T)
    ⇒
SurvivesDeclaredFailureModel(T)
```

The declared failure model is part of the claim.

## 102. Verification Matrix

| Property | Verification question |
|---|---|
| Ownership | Is every authoritative state domain governed by an owner? |
| Revision | Are stale writes detectable? |
| Transactions | Are atomicity and isolation semantics explicit? |
| Commit | Is the authoritative commit point defined? |
| Durability | What failure model does durability cover? |
| Persistence | Is the persistence boundary observable? |
| WAL/Journal | Can recovery reconstruct the required state? |
| Snapshot | Is snapshot consistency defined? |
| Schema | Can persisted state be interpreted safely? |
| Migration | Are schema changes explicit and validated? |
| Replication | Are replica and authority states distinguished? |
| Split brain | Can two writers become authoritative? |
| Fencing | Are stale writers prevented from mutation? |
| Recovery | Can state be reconstructed after failure? |
| Reconciliation | Can state divergence be detected and resolved? |
| External state | Are external side effects distinguished from local commit? |
| Unknown outcome | Can ambiguous operations be represented safely? |
| Caching | Is cache state distinguished from source-of-truth state? |
| Watches | Can consumers recover from event loss? |
| Retention | Are historical records retained safely? |
| Evidence | Can claims of commit/durability/recovery be independently supported? |

## 103. What Part LV Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production durable state store;
- universal transactional semantics;
- linearizable state across every subsystem;
- production-grade replication for every state domain;
- complete snapshot/restore implementation;
- automatic schema migration for all persisted objects;
- complete external side-effect reconciliation;
- universal exactly-once durability semantics;
- complete disaster-recovery automation.

Those require implementation-specific evidence.

## 104. Transition to Part LVI

Part LV establishes the durable-state foundation.

Part LVI should define **messaging, eventing, queues, delivery semantics, ordering, correlation, replay, backpressure, and cross-domain communication**.

```text
Part LIV
Workloads + execution + supervision
        ↓
Part LV
State + storage + durability + recovery
        ↓
Part LVI
Messaging + events + delivery + communication
```

## Canonical rule

> **NROS never equates memory with authority or persistence: state mutations, commits, durability, recovery, and external observation are distinct architectural events governed by explicit ownership, revision, consistency, integrity, failure, and evidence semantics.**
