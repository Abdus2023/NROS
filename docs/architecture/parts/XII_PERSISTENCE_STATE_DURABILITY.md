# Part XII — Persistence, State & Durability

> **Series:** NROS Architecture Series  
> **Part:** XII  
> **Role:** State ownership, persistence, checkpoints, journals, crash recovery, consistency, and durability  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part XI established security, trust, and authorization. Part XII defines how NROS represents runtime state, determines which state must survive failure, persists it, restores it, and verifies that recovered state is valid.

The central rule is:

> **Persisted state is not automatically durable, and restored state is not automatically valid; durability and recovery require explicit guarantees and verification.**

## 2. State Taxonomy

NROS distinguishes at least:

```text
State
├── Volatile
├── Recoverable
├── Persistent
├── Durable
├── Reconstructible
└── External authoritative state
```

The classification belongs to the state contract, not merely to the storage mechanism.

## 3. Volatile State

Volatile state exists only during the current runtime incarnation.

Examples:

```text
scheduler queues
transient caches
in-flight execution context
temporary buffers
connection-local state
```

Loss of volatile state may be acceptable or may require reconstruction.

## 4. Persistent State

Persistent state is intended to survive a runtime restart or process lifetime boundary.

```text
RAM
  ↓ persistence operation
Storage
```

Persistence alone does not establish that the state has reached stable storage.

## 5. Durability

Durability is a property of a completed persistence operation under a defined failure model.

```text
Persist requested
      ↓
Write accepted
      ↓
Write completed
      ↓
Durability boundary
      ↓
Failure
      ↓
State survives according to contract
```

The durability boundary must be explicit.

## 6. State Ownership

Every persistent state item should have an owner.

```text
StateItem
├── owner
├── schema
├── version
├── lifecycle
├── consistency model
└── durability requirement
```

Shared state without an explicit ownership model creates ambiguous recovery semantics.

## 7. Authoritative State

The runtime should distinguish authoritative state from derived state.

```text
Authoritative
    ↓ source of truth
Derived
    ↓ reconstructed from authoritative state
Cache
    ↓ optimization
```

Caches should not silently become authoritative merely because the original source is unavailable.

## 8. State Versioning

Persistent state should carry schema/version information.

```text
State v1
   ↓ migration
State v2
```

A runtime must know whether it can:

```text
read
migrate
reject
rollback
```

an older or newer state representation.

## 9. Checkpoints

A checkpoint captures a defined state boundary.

```text
Running
  ↓ checkpoint
Checkpoint N
  ↓ continue
Running
```

A checkpoint should specify:

```text
scope
state version
creation time
generation
consistency boundary
storage location
integrity metadata
```

## 10. Consistent Checkpoint

A checkpoint is useful for recovery only if its internal consistency is defined.

Possible models:

```text
single-entity snapshot
transactional snapshot
coordinated distributed snapshot
application-defined checkpoint
```

A collection of individually valid files does not automatically constitute a globally consistent checkpoint.

## 11. Journaling

A journal records state-changing operations or recovery-relevant events.

```text
Operation 1
Operation 2
Operation 3
   ↓
Journal
```

Journaling can support crash recovery, auditability, and reconstruction, but the exact guarantees depend on ordering and persistence semantics.

## 12. Write-Ahead Logging

With write-ahead logging:

```text
Intent / log record
      ↓ durable boundary
State mutation
      ↓
Commit
```

The system must define when a log record is considered durable and how incomplete records are handled after failure.

## 13. Atomicity

An atomic state transition should not expose an invalid intermediate representation where atomicity is required.

```text
Before
  ↓ transaction
After
```

Atomicity is a property of the state operation and its storage semantics, not merely of a function name such as `save()`.

## 14. Ordering

Persistence systems must define ordering where recovery depends on it.

```text
A → B → C
```

must not be replayed as:

```text
B → C → A
```

unless the state model explicitly permits reordering.

## 15. Crash Recovery

A conceptual recovery sequence is:

```text
Crash
 ↓
Open persistent state
 ↓
Validate integrity
 ↓
Identify committed state
 ↓
Replay / rollback as required
 ↓
Reconstruct runtime state
 ↓
Verify invariants
 ↓
Resume
```

Recovery must not treat every persisted byte as trusted valid state.

## 16. Recovery Verification

Restoration requires postconditions.

```text
State restored
      ↓
Schema validation
      ↓
Integrity validation
      ↓
Semantic validation
      ↓
Runtime invariant validation
      ↓
Recovered-valid
```

Therefore:

```text
Loaded
  ≠
Valid
  ≠
Operational
```

## 17. Integrity

Persistent state may use integrity metadata such as:

```text
checksum
hash
MAC
authenticated record
storage-level integrity mechanism
```

Integrity verification detects corruption according to the mechanism's threat and failure model; it does not prove semantic correctness.

## 18. Corruption

Possible corruption states include:

```text
truncated
partially written
checksum mismatch
invalid schema
invalid references
inconsistent transaction
unknown version
```

Recovery policy should distinguish recoverable corruption from terminal corruption.

## 19. Rollback

Rollback restores a previously accepted state boundary.

```text
State N
 ↓ updates
State N+1
 ↓ failure
rollback
 ↓
State N
```

Rollback must define what happens to external side effects that occurred after the rollback point.

## 20. Checkpoint + Generation

Checkpoints should be associated with the relevant entity generation where stale restoration would be dangerous.

```text
Generation 7
   ↓ checkpoint
Checkpoint G7

Generation 8
   ↓ restart
```

A G7 checkpoint must not automatically be applied to G8 without an explicit compatibility and recovery policy.

## 21. Persistence and Lifecycle

Persistent state participates in lifecycle transitions:

```text
RUNNING
  ↓ checkpoint
RUNNING

RUNNING
  ↓ shutdown
PERSISTING
  ↓
STOPPED
```

The exact lifecycle states are entity-specific, but persistence work must have explicit ownership and completion semantics.

## 22. Persistence and Resources

Storage operations consume resources:

```text
CPU
memory
storage capacity
I/O bandwidth
latency budget
network bandwidth
```

Persistence cannot be treated as free background work when its resource impact affects scheduling or deadlines.

## 23. Persistence and Scheduling

Checkpointing and journal writes may be scheduled work.

They may therefore require:

```text
priority
budget
deadline
I/O class
backpressure
```

A persistence subsystem must avoid creating unbounded queues under sustained write pressure.

## 24. Backpressure

When persistence cannot keep up:

```text
Producer
   ↓
Persistence queue
   ↓
Storage
```

The system needs an explicit policy:

```text
block
throttle
drop noncritical data
coalesce
spill to alternate storage
fail operation
```

Silent unbounded buffering is not a durability strategy.

## 25. Storage Failure

Storage may become:

```text
slow
full
unavailable
read-only
corrupt
inaccessible
```

Recovery policy should distinguish these conditions where the distinction affects safe behavior.

## 26. External State

Some state is authoritative outside NROS.

Examples:

```text
device controller
external database
remote service
hardware register
operator system
```

NROS must not claim durable ownership over state it cannot control.

## 27. State Synchronization

When state exists in multiple locations:

```text
NROS state
     ↕
External state
```

the consistency relationship must be explicit.

Possible models include:

```text
authoritative external source
NROS authoritative source
eventual consistency
transactional coordination
reconciliation
```

## 28. Reconciliation

After restart or partition, divergent state may require reconciliation.

```text
Local state
    ↘
     reconcile
    ↗
Remote state
```

Reconciliation must have conflict-resolution rules rather than relying on accidental last-writer behavior.

## 29. Idempotent Recovery

Recovery actions should be idempotent where practical.

Repeated restoration should not progressively corrupt state.

```text
restore(checkpoint)
restore(checkpoint)
```

should produce the same defined result when the operation is specified as idempotent.

## 30. Garbage Collection of State

Old checkpoints and journal records require lifecycle policies:

```text
retention
compaction
archival
deletion
```

Deletion must respect recovery, audit, compliance, and dependency requirements.

## 31. Encryption

Persistent state may require confidentiality protection.

The architecture distinguishes:

```text
encryption at rest
key management
access authorization
integrity protection
```

Encryption does not itself guarantee integrity, authorization, or recoverability.

## 32. Persistence and Security

Part XI security policy applies to persistent state operations:

```text
create checkpoint
read state
restore state
rollback
compact
export
purge
```

Sensitive state must not become readable merely because it is persistent.

## 33. Observability

Persistence events should be observable:

```text
CheckpointStarted
CheckpointCommitted
CheckpointFailed
JournalAppended
JournalCommitted
RecoveryStarted
RecoveryCompleted
RecoveryFailed
StateCorruptionDetected
RollbackStarted
RollbackCompleted
```

Evidence should identify entity and generation where applicable.

## 34. Verification Matrix

| Property | Verification question |
|---|---|
| Ownership | Is every persistent state item assigned to an owner? |
| Versioning | Can state schema versions be identified? |
| Atomicity | Are required transitions atomic? |
| Ordering | Is required persistence ordering preserved? |
| Durability | Is the durability boundary explicitly defined? |
| Integrity | Can corruption be detected according to policy? |
| Recovery | Can crash recovery reconstruct a valid state? |
| Checkpoints | Are checkpoint boundaries consistent? |
| Generation | Can stale-generation state be rejected? |
| Rollback | Are external side effects handled correctly? |
| Backpressure | Is persistence overload bounded? |
| Storage failure | Are storage faults classified and handled? |
| Security | Are state operations authorized and protected? |
| Reconciliation | Are divergent replicas resolved by explicit policy? |
| Retention | Can old state be safely retired? |

## 35. What Part XII Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- a universal persistent store;
- crash-safe durability across arbitrary platforms;
- transactional distributed persistence;
- automatic state migration;
- corruption recovery in every failure mode;
- transparent rollback of external side effects;
- encrypted persistent state by default;
- consensus-backed replicated state.

Those properties require implementation and verification evidence.

## 36. Transition to Part XIII

Part XII defines state persistence and recovery semantics.

Part XIII should define **dataflow, message semantics, event streams, buffering, backpressure, and flow control**, connecting the communication model from Part V with resource and scheduling constraints from Parts VII–VIII.

```text
Part XI
Security + trust + authorization
        ↓
Part XII
Persistence + state + durability
        ↓
Part XIII
Dataflow + events + flow control
```

## Canonical rule

> **NROS distinguishes volatile, persistent, and durable state, and requires integrity, consistency, and recovery verification before restored state is treated as operationally valid.**
