# NROS Agent & Workflow (Part CI–CX)

We now enter the boundary where distributed runtimes become genuinely difficult:

> **What should NROS do when it cannot determine whether an operation happened?**

Examples:

```text
Controller → START → Agent
                  X
             network failure
```

The controller does not know whether:

```text
START never arrived
```

or:

```text
START arrived and execution started
```

or:

```text
START executed, but ACK was lost
```

This is the **uncertainty problem**.

# 1. The Fundamental Rule

NROS must distinguish:

```text
NOT_EXECUTED
EXECUTED
UNKNOWN
```

The third state is essential.

Do not collapse:

```text
UNKNOWN → NOT_EXECUTED
```

because that can cause duplicate side effects.

# 2. The Ambiguous Operation

Consider:

```text
Controller
   │
   │ START(command-17)
   ↓
 Agent
   │
   │ starts process
   ↓
 Process
   │
   X
 network failure
```

The Controller receives no response.

Correct state:

```text
START(command-17) = UNKNOWN
```

not:

```text
START(command-17) = FAILED
```

# 3. Three Delivery States

For commands:

```text
UNSENT
SENT
UNKNOWN
```

After authoritative evidence:

```text
ACCEPTED
EXECUTED
REJECTED
```

# 4. Command State Machine

```text
CREATED
   ↓
AUTHORIZED
   ↓
DISPATCHED
   ↓
   ├── ACKED → ACCEPTED
   │
   ├── REJECTED
   │
   └── TIMEOUT → UNKNOWN
```

Unknown must trigger reconciliation, not blind retry.

# 5. Why Blind Retry Is Dangerous

Suppose:

```text
START command
```

times out.

Controller sends another:

```text
START command
```

Agent may already have started the first execution.

Result:

```text
Execution A
Execution B
```

Duplicate execution.

# 6. Idempotency

Commands therefore require an idempotency identity:

```text
idempotency_key
```

Example:

```text
start:execution-123:generation-7
```

# 7. Agent Deduplication

Agent maintains:

```text
command_id
idempotency_key
result
```

If the same command arrives again:

```text
same key
```

the Agent returns the previously recorded result instead of executing again.

# 8. Durable Deduplication

For crash safety, deduplication state cannot always remain only in RAM.

Bad:

```text
command received
 ↓
RAM remembers command
 ↓
process crashes
 ↓
same command arrives
 ↓
executed twice
```

# 9. Durable Command Record

Persist:

```text
CommandRecord {
    command_id
    idempotency_key
    state
    execution_id
    generation
    result
}
```

before or atomically with the side effect according to the execution protocol.

# 10. The Hard Boundary

The hardest case is:

```text
persist command record
       ↓
execute external side effect
       ↓
crash
       ↓
persist result
```

If the process crashes between those steps, the system may know:

```text
command existed
```

but not:

```text
side effect completed
```

# 11. Exactly-Once Is Not Magic

A distributed system cannot simply declare:

```text
exactly_once = true
```

The guarantee depends on the boundary.

For example:

```text
NROS database
```

may support atomic transactions, while:

```text
arbitrary external HTTP server
```

may not.

# 12. Transaction Boundary

Strong semantics require a common transactional boundary.

For example:

```text
NROS state
+
event
+
deduplication record
```

may be committed atomically if they share a transactional store.

# 13. External Side Effects

Consider:

```text
POST /charge
```

NROS cannot assume the remote service participated in its transaction.

Therefore:

```text
local commit ≠ external commit
```

# 14. Idempotency-Key Protocol

External APIs that support idempotency should receive:

```text
Idempotency-Key: execution-123/payment-1
```

Repeated requests then resolve to the same logical operation.

# 15. Side-Effect Record

NROS can track:

```text
SideEffect {
    operation_id
    idempotency_key
    target
    request_hash
    state
    response_hash
}
```

# 16. Side-Effect States

```text
PLANNED
DISPATCHED
UNKNOWN
CONFIRMED
FAILED
COMPENSATED
```

# 17. Unknown Side Effect

If the network fails after transmission:

```text
DISPATCHED
   ↓
network timeout
   ↓
UNKNOWN
```

Do not immediately:

```text
FAILED
```

# 18. Reconciliation of Side Effects

The system should attempt:

```text
query status
```

using:

```text
operation_id
idempotency_key
```

# 19. Status Query

External system may answer:

```text
CONFIRMED
```

Then:

```text
UNKNOWN → CONFIRMED
```

# 20. External System Without Status API

If no query mechanism exists:

```text
UNKNOWN
```

may remain unresolved.

The correct response is not to invent certainty.

Possible policy:

```text
manual verification
compensation
quarantine
```

# 21. Compensation

When an operation cannot safely be retried:

```text
original side effect
      ↓
compensating operation
```

Example:

```text
reserve resource
      ↓
compensate
      ↓
release resource
```

# 22. Compensation ≠ Rollback

A rollback attempts to restore prior state.

A compensation is a new operation intended to counteract an earlier effect.

They are not mathematically identical.

# 23. Saga Model

For multi-step operations:

```text
A
 ↓
B
 ↓
C
```

compensation may be:

```text
C⁻¹
 ↓
B⁻¹
 ↓
A⁻¹
```

But compensation can itself fail.

# 24. Saga Failure

Example:

```text
A succeeded
B succeeded
C failed
```

Then:

```text
compensate B
compensate A
```

may be attempted.

If compensation B fails:

```text
PARTIALLY_COMPENSATED
```

is a legitimate state.

# 25. NROS Should Not Pretend Rollback Is Universal

Some operations cannot be reversed:

```text
send message
publish artifact
external notification
physical actuator movement
```

For these:

```text
compensation
```

must be explicitly designed.

# 26. Operation Classification

Each operation should declare:

```text
idempotent
retryable
reversible
compensatable
transactional
```

# 27. Operation Contract

Conceptually:

```text
OperationSemantics {
    idempotent
    retryable
    reversible
    compensatable
    external
}
```

This gives recovery logic the information it needs.

# 28. At-Most-Once

At-most-once means:

```text
operation executed ≤ 1 time
```

But it may never execute.

Useful where duplicate side effects are unacceptable.

# 29. At-Least-Once

At-least-once means:

```text
operation eventually attempted ≥ 1 time
```

but duplicates may occur.

Useful when operations are idempotent.

# 30. Effectively-Once

The practical pattern:

```text
at-least-once delivery
+
stable idempotency key
+
deduplication
```

produces one logical effect despite duplicate delivery.

# 31. Exactly-Once Processing

Inside a single transactional boundary:

```text
consume event
+
update state
+
record processed event
```

can be atomic.

This is stronger than claiming exactly-once execution across arbitrary systems.

# 32. Transactional Inbox

A useful pattern:

```text
Incoming Event
      ↓
Inbox Table
      ↓
Transaction
 ├── deduplicate
 ├── update state
 └── record processing
      ↓
COMMIT
```

If the transaction fails:

```text
everything rolls back
```

# 33. Transactional Outbox

For outbound events:

```text
State Change
+
Outbox Event
```

are committed together.

Then:

```text
Outbox Publisher
       ↓
Event Broker
```

publishes the event.

# 34. Outbox Prevents Lost Events

Without outbox:

```text
update DB
   ↓
crash
   ↓
publish event never happens
```

Now state changed but evidence is missing.

With outbox:

```text
DB transaction
 ├── state update
 └── outbox event
```

both survive together.

# 35. Inbox + Outbox

A strong NROS component may use both:

```text
           INBOX
             ↓
         PROCESS
             ↓
      STATE + OUTBOX
             ↓
         PUBLISH
```

This creates reliable event-driven boundaries.

# 36. Event Publication Failure

If publication fails:

```text
outbox = PENDING
```

The publisher retries.

# 37. Publisher Crash

Suppose:

```text
publish event
 ↓
broker accepts
 ↓
publisher crashes before marking sent
```

The event may be published twice.

Therefore consumers must tolerate duplicates.

# 38. Delivery Guarantee

The realistic default:

```text
at-least-once event delivery
```

with:

```text
idempotent consumers
```

# 39. Duplicate Event

Event:

```text
event_id = E42
```

arrives twice.

Consumer stores:

```text
processed(E42) = true
```

and ignores the duplicate.

# 40. Event Ordering

At-least-once delivery can also produce:

```text
E1
E3
E2
```

Therefore consumers may need:

```text
sequence
causation
buffering
reordering
```

# 41. Per-Stream Ordering

A practical guarantee:

```text
events within one stream
```

are ordered.

Across streams:

```text
no global order assumed
```

# 42. Causal Ordering

If:

```text
E2.causation_id = E1
```

then E1 must logically precede E2.

This provides a stronger semantic relationship than wall-clock ordering.

# 43. Lost Acknowledgement

Classic case:

```text
Agent executes
 ↓
ACK sent
 ↓
ACK lost
```

Controller sees:

```text
UNKNOWN
```

but the Agent has:

```text
EXECUTED
```

# 44. Query-Based Reconciliation

Controller asks:

```text
status(command_id)
```

Agent responds:

```text
EXECUTED
execution_id = E17
```

Now uncertainty resolves.

# 45. Command Journal

Agent should maintain a durable command journal:

```text
command_id
state
execution_id
result
timestamp
```

This enables recovery after Agent restart.

# 46. Agent Crash During Command

Sequence:

```text
receive command
 ↓
persist command
 ↓
crash
```

On restart:

```text
command state = ACCEPTED
```

Agent can reconcile whether execution exists.

# 47. Agent Crash After Side Effect

Sequence:

```text
receive
 ↓
execute
 ↓
crash
 ↓
no result record
```

The Agent must discover existing execution state rather than blindly rerun.

# 48. Execution Fingerprint

Where possible, execution can expose:

```text
execution_id
command_id
generation
process identity
```

This helps detect an already-running operation.

# 49. Process Identity

For OS-level execution:

```text
PID
start_time
command_hash
```

can form an execution fingerprint.

PID alone is insufficient because it can be reused.

# 50. Stronger Process Fingerprint

Conceptually:

```text
process_fingerprint =
hash(
    executable
    arguments
    environment_identity
    start_time
    execution_id
)
```

# 51. Duplicate Detection

Before starting a replacement:

```text
search existing execution
 ↓
match fingerprint
 ↓
adopt if valid
```

rather than:

```text
always start
```

# 52. Execution Attempt Record

Each attempt should contain:

```text
attempt_id
execution_id
agent_incarnation
generation
start_evidence
terminal_evidence
result
```

# 53. Attempt Lifecycle

```text
CREATED
 ↓
DISPATCHED
 ↓
ACCEPTED
 ↓
STARTED
 ↓
RUNNING
 ↓
TERMINAL
```

Unknown states can occur between any two externally observed transitions.

# 54. Partial Result Commit

Suppose:

```text
execution finishes
 ↓
result written
 ↓
controller crashes
 ↓
commit event missing
```

Now:

```text
artifact exists
execution state uncertain
```

This is another reconciliation problem.

# 55. Result Provenance

The result should identify:

```text
execution_id
attempt_id
generation
producer
content_hash
created_at
```

# 56. Result Adoption

After restart:

```text
find artifact
 ↓
verify provenance
 ↓
verify hash
 ↓
associate with execution
```

Then NROS may safely produce:

```text
RESULT_RECONCILED
```

# 57. Never Adopt by Filename Alone

Bad:

```text
output.json exists
```

Good:

```text
artifact identity
+
content hash
+
producer identity
+
execution provenance
```

# 58. Partial Resource Commit

Suppose:

```text
reserve CPU
reserve memory
```

CPU succeeds:

```text
CPU = reserved
```

Memory fails:

```text
MEMORY = rejected
```

The reservation is partially committed.

# 59. Resource Transaction

If the resource manager supports atomic allocation:

```text
CPU + memory
```

should be one transaction.

Otherwise:

```text
partial allocation
```

must be explicitly represented.

# 60. Compensation

If memory allocation fails:

```text
release CPU
```

then:

```text
RESOURCE_ROLLED_BACK
```

or:

```text
RESOURCE_PARTIAL
```

if release cannot complete.

# 61. Resource State Machine

```text
AVAILABLE
   ↓
RESERVATION_PENDING
   ↓
RESERVED
   ↓
ALLOCATED
   ↓
RELEASE_PENDING
   ↓
RELEASED
```

Failure paths must be explicit.

# 62. Reservation vs Allocation

Reservation:

> This resource is committed for this future operation.

Allocation:

> This resource is currently assigned/consumed.

They must not be represented by one ambiguous flag:

```text
allocated = true
```

# 63. Resource Double Allocation

The invariant:

```text
allocated(resource) ≤ capacity(resource)
```

must hold according to the resource's semantics.

# 64. Oversubscription

If NROS intentionally allows:

```text
allocated > physical capacity
```

that must be an explicit policy rather than accidental drift.

# 65. Resource Evidence

Every important allocation should have:

```text
reservation_id
resource_id
execution_id
generation
amount
authority
```

# 66. Failure Matrix

| Failure | Correct immediate interpretation |
|---|---|
| command timeout | UNKNOWN |
| ACK lost | UNKNOWN |
| agent disappeared | UNKNOWN |
| lease expired | authority lost |
| result exists | evidence of result |
| result missing | absence of evidence |
| event duplicated | deduplicate |
| event reordered | reorder/hold |
| resource partially allocated | PARTIAL |
| controller restarted | RECOVERY |
| agent restarted | new incarnation |
| ownership stale | FENCE |
| external side effect uncertain | UNKNOWN |

# 67. The Most Important Distinction

NROS must separate:

```text
absence of evidence
```

from:

```text
evidence of absence
```

These are not equivalent.

# 68. Absence of Evidence

Example:

```text
no heartbeat received
```

means:

```text
heartbeat not observed
```

not automatically:

```text
process dead
```

# 69. Evidence of Absence

A stronger observation:

```text
resource manager confirms process terminated
```

is evidence supporting:

```text
TERMINATED
```

# 70. Failure Detector

Failure detection therefore produces:

```text
suspected_failure
```

before necessarily producing:

```text
confirmed_failure
```

# 71. Suspicion State

Useful lifecycle:

```text
HEALTHY
 ↓
SUSPECTED
 ↓
CONFIRMED_LOST
```

# 72. Failure Detector Tuning

The transition:

```text
HEALTHY → SUSPECTED
```

may use:

```text
heartbeat interval
timeout
network state
lease state
historical behavior
```

But the final authority semantics must remain explicit.

# 73. False Positive

A network partition can create:

```text
Agent healthy
Controller suspects failure
```

Therefore recovery actions must be fenced before dangerous duplicate execution.

# 74. Recovery Safety

If the controller wants to restart an uncertain execution:

```text
1. establish new generation
2. fence old generation
3. verify resource ownership
4. then create replacement attempt
```

This is safer than:

```text
timeout
 ↓
start another
```

# 75. Generation-Fenced Recovery

```text
Generation 7
   ↓
failure suspected
   ↓
Generation 8 issued
   ↓
Generation 7 fenced
   ↓
Generation 8 recovery attempt
```

# 76. External Fencing

The strongest recovery requires the resource itself to understand fencing:

```text
Resource
   ↑
token=8 → accepted
token=7 → rejected
```

Without external fencing, stale processes may continue side effects.

# 77. Unfenced Resources

If a resource cannot enforce fencing:

```text
NROS cannot honestly claim stale-side-effect prevention.
```

It must use a weaker recovery policy.

# 78. Recovery Capability Classification

Resources can declare:

```text
FENCED
IDEMPOTENT
COMPENSATABLE
NON_REVERSIBLE
UNKNOWN
```

This affects recovery strategy.

# 79. Recovery Safety Classes

For example:

```text
Class A:
fenced + idempotent

Class B:
idempotent

Class C:
compensatable

Class D:
non-reversible

Class E:
unknown semantics
```

Higher-risk classes require stronger safeguards.

# 80. Side-Effect Policy

An execution policy may specify:

```text
side_effect_policy {
    retry_mode
    fencing_required
    idempotency_required
    compensation_allowed
    manual_intervention_threshold
}
```

# 81. Exactly-Once Claim Boundary

NROS documentation should state guarantees precisely:

```text
Exactly-once state transition:
    possible within transactional boundary.

Exactly-once event publication:
    not assumed; consumers must deduplicate.

Exactly-once execution:
    not guaranteed across arbitrary agents.

Exactly-once external side effect:
    requires cooperation from external system.
```

This avoids misleading architecture claims.

# 82. Failure Semantics Matrix

| Boundary | Strongest realistic guarantee |
|---|---|
| local state transaction | atomic |
| state + outbox | atomic state/event creation |
| event broker | at-least-once |
| command delivery | at-least-once + idempotency |
| agent execution | protocol-dependent |
| external API with idempotency key | effectively-once |
| arbitrary external side effect | potentially UNKNOWN |
| fenced resource | stale-owner prevention |
| non-fenced resource | weaker recovery guarantee |

# 83. Failure Invariants

```text
1. UNKNOWN is a first-class state.

2. Command timeout does not prove command failure.

3. Lost ACK does not prove operation absence.

4. Duplicate commands must be deduplicated where possible.

5. Deduplication state must survive required failure boundaries.

6. Event consumers must tolerate duplicate delivery.

7. Event consumers must tolerate defined reordering semantics.

8. State and outbox records should commit atomically where required.

9. External side effects require explicit semantics.

10. Exactly-once claims must identify their transaction boundary.

11. Idempotency keys must remain stable across retries.

12. Side-effect status must be reconcilable where possible.

13. Unresolvable side effects remain UNKNOWN rather than being guessed.

14. Compensation is explicit.

15. Compensation failure is itself represented.

16. Resource reservations and allocations are distinct.

17. Partial resource operations are explicit.

18. Controller recovery cannot blindly restart uncertain executions.

19. Generation fencing precedes dangerous replacement execution.

20. Agent incarnations distinguish old and new process instances.

21. Orphan adoption requires provenance and authority validation.

22. Orphan reclamation requires fencing/ownership verification.

23. Terminal execution history is immutable.

24. Failure detection distinguishes suspicion from confirmed failure.

25. Absence of evidence is not evidence of absence.

26. Recovery actions are policy-driven.

27. Recovery loops are detectable.

28. Non-convergent executions can be quarantined.

29. Every ambiguous operation remains auditable.

30. The system never claims stronger guarantees than its infrastructure can actually provide.
```

# 84. Unified Failure-Safe Execution Model

```text
                    COMMAND
                       │
                       ↓
                ┌─────────────┐
                │ IDEMPOTENCY │
                │   CHECK     │
                └──────┬──────┘
                       ↓
                ┌─────────────┐
                │   AGENT     │
                └──────┬──────┘
                       ↓
                  SIDE EFFECT
                       │
             ┌─────────┴─────────┐
             ↓                   ↓
          CONFIRMED            UNKNOWN
             │                   │
             ↓                   ↓
          COMMIT             RECONCILE
                                 │
                       ┌─────────┴─────────┐
                       ↓                   ↓
                    CONFIRMED           UNKNOWN
                       │                   │
                       ↓                   ↓
                    COMMIT            QUARANTINE/
                                      COMPENSATE/
                                      MANUAL
```

# 85. NROS Reliability Principle

The architecture should optimize not for:

```text
"never fail"
```

but for:

```text
"fail without losing the ability to determine what happened."
```

That is a much stronger and more realistic distributed-runtime objective.

# Part CII — Durable State, Journaling & Storage Semantics

The next layer should define the persistence substrate underneath all of this:

```text
event journal
command journal
state store
snapshot store
outbox
inbox
artifact metadata
lease state
ownership state
recovery checkpoints
```

The central question becomes:

> **What must be persisted atomically, what can be reconstructed, what can be cached, and what must survive a total process restart?**

# NROS — Part CII: Durable State, Journaling & Storage Semantics

The next architectural boundary is **durability**.

Everything established previously—authority, generations, reconciliation, recovery, idempotency, fencing, and failure semantics—depends on one question:

> **Which facts must survive process failure, and what is the minimum durable record required to reconstruct truth?**

A runtime that has correct in-memory semantics but weak persistence is not actually fault tolerant.

## 1. Persistence Is Not One Thing

NROS should distinguish at least:

```text
VOLATILE
CACHE
DURABLE
IMMUTABLE EVIDENCE
```

These have different guarantees.

### Volatile

Can disappear on restart:

```text
in-memory queues
temporary scheduling state
connection pools
metrics buffers
```

### Cache

Reconstructible from authoritative state:

```text
derived indexes
recent observations
computed summaries
```

### Durable

Must survive the relevant failure boundary:

```text
execution state
authority generation
command deduplication
leases
recovery checkpoints
```

### Immutable evidence

Historical facts that should not be rewritten:

```text
events
audit records
terminal results
authority transitions
```

# 2. Source of Truth

Every important datum should have an explicit authority.

For example:

```text
Execution state
    → state store

Historical transition
    → event journal

Current observation
    → observation store/cache

Artifact content
    → artifact store

Artifact identity/provenance
    → metadata store
```

Avoid having five competing "truths."

# 3. State vs Event

NROS needs both:

```text
CURRENT STATE
+
HISTORICAL EVENTS
```

They solve different problems.

Current state answers:

> What is true now?

Events answer:

> How did we get here?

# 4. State Store

Conceptually:

```text
Execution {
    execution_id
    work_id
    attempt_id
    generation
    state
    authority
    owner
    result
    version
}
```

This is optimized for current-state queries.

# 5. Event Journal

Conceptually:

```text
Event {
    event_id
    stream_id
    sequence
    type
    payload
    causation_id
    correlation_id
    timestamp
}
```

The event journal is optimized for historical reconstruction and auditability.

# 6. Event Sourcing Is Not Mandatory

NROS does **not** necessarily need pure event sourcing.

A practical architecture may use:

```text
durable state
+
append-only event journal
```

This avoids forcing every read to replay the complete history.

# 7. Hybrid Model

```text
             ┌───────────────┐
             │   COMMAND     │
             └───────┬───────┘
                     ↓
              ┌─────────────┐
              │ TRANSACTION │
              └──────┬──────┘
                     │
             ┌───────┴────────┐
             ↓                ↓
       CURRENT STATE       EVENT LOG
```

Both are committed according to the same consistency boundary.

# 8. Versioned State

Every mutable state record should have a version:

```text
version = 41
```

Next mutation:

```text
version = 42
```

This enables optimistic concurrency.

# 9. Compare-and-Swap

Conceptually:

```text
UPDATE execution
SET state = RUNNING,
    version = 42
WHERE execution_id = E
  AND version = 41
```

If zero rows are modified:

```text
CONCURRENT_MODIFICATION
```

The controller must reconcile rather than overwrite blindly.

# 10. Lost Update Prevention

Without versioning:

```text
Controller A reads version 41
Controller B reads version 41

A writes RUNNING
B writes CANCELLED

B accidentally destroys A's update.
```

Version checks prevent this.

# 11. Optimistic Concurrency

The general pattern:

```text
READ(version=N)
      ↓
COMPUTE
      ↓
WRITE(version=N → N+1)
      ↓
success / conflict
```

This works particularly well for control-plane state.

# 12. State Version vs Generation

These are different.

```text
generation
```

means:

> Which authority/execution incarnation?

```text
version
```

means:

> Which revision of the current record?

Example:

```text
generation = 7
version    = 143
```

# 13. Epoch vs Generation vs Version

Keep the meanings explicit:

| Concept | Purpose |
|---|---|
| epoch | ownership/control-plane era |
| generation | execution authority incarnation |
| version | record revision |
| sequence | ordered event position |
| incarnation | process instance |

These must not be collapsed into one counter.

# 14. Event Sequence

For a stream:

```text
stream = execution:E17
```

events may be:

```text
sequence 1
sequence 2
sequence 3
sequence 4
```

The sequence gives deterministic ordering **within that stream**.

# 15. Global Ordering

NROS should avoid assuming a globally ordered event stream unless its storage system explicitly provides one.

Prefer:

```text
per-stream ordering
+
causal relationships
```

over an artificial global timestamp ordering.

# 16. Causation

An event should identify what caused it.

Example:

```text
RETRY_SCHEDULED
caused_by = COMMAND_TIMEOUT
```

This enables causal reconstruction.

# 17. Correlation

Events can also share:

```text
correlation_id
```

for an entire logical operation.

Example:

```text
correlation = work-123
```

Events from several components can then be connected.

# 18. Event Identity

Every event needs a stable:

```text
event_id
```

because duplicate delivery is possible.

Example:

```text
event_id = 01J...
```

A consumer can record:

```text
processed(event_id)
```

to provide deduplication.

# 19. Durable Inbox

The inbox pattern:

```text
Incoming event
      ↓
┌──────────────┐
│    INBOX     │
└──────┬───────┘
       ↓
processing transaction
```

The inbox prevents repeated delivery from repeatedly applying state changes.

# 20. Inbox Record

```text
InboxEntry {
    event_id
    received_at
    source
    status
    processed_at
}
```

Possible states:

```text
RECEIVED
PROCESSING
PROCESSED
FAILED
QUARANTINED
```

# 21. Outbox

The outbound equivalent:

```text
OutboxEntry {
    event_id
    destination
    payload
    state
    attempts
    next_attempt_at
}
```

# 22. Outbox Lifecycle

```text
PENDING
  ↓
DISPATCHING
  ↓
PUBLISHED
```

Failure:

```text
DISPATCHING
    ↓
RETRY_PENDING
```

Permanent failure:

```text
FAILED
```

# 23. Outbox Must Be Durable

If the process crashes after:

```text
state update
```

but before:

```text
event publication
```

the outbox ensures the event remains available.

# 24. Atomic State + Event

The strongest local invariant is:

```text
state transition
+
corresponding event
```

are committed atomically when they represent one logical transition.

# 25. Example

Instead of:

```text
UPDATE execution
...
COMMIT

publish(EXECUTION_STARTED)
```

use:

```text
TRANSACTION
    update execution
    append event
    append outbox
COMMIT
```

Then publish asynchronously.

# 26. Crash Safety

If crash occurs before commit:

```text
neither state nor event exists
```

If crash occurs after commit:

```text
both exist
```

The outbox can later publish the event.

# 27. Snapshotting

A long event stream can become expensive to replay.

Therefore:

```text
events:
E1 E2 E3 ... E100000
```

can periodically produce:

```text
snapshot @ E100000
```

# 28. Snapshot

A snapshot may contain:

```text
ExecutionSnapshot {
    stream_id
    sequence
    state
    generation
    authority
    resources
    result
}
```

# 29. Replay

Recovery becomes:

```text
load snapshot @ 100000
        ↓
replay E100001
        ↓
E100002
        ↓
...
```

rather than replaying the entire history.

# 30. Snapshot Is Not History

A snapshot is derived state.

It should not replace the event history unless the system explicitly accepts loss of historical detail.

Therefore:

```text
snapshot ≠ audit log
```

# 31. Snapshot Integrity

Snapshots need:

```text
sequence
version
schema_version
checksum
created_at
```

so NROS can verify they are valid.

# 32. Snapshot Schema Version

Storage structures evolve.

Therefore:

```text
schema_version = 4
```

must be explicit.

A new runtime may need:

```text
v4 → v5 migration
```

before loading the snapshot.

# 33. Migration

A migration must be:

```text
deterministic
auditable
restart-safe
versioned
```

# 34. Migration Must Not Rewrite History

Suppose:

```text
event schema v1
```

becomes:

```text
event schema v2
```

The original historical event should remain identifiable.

Prefer:

```text
original schema
+
compatible decoding/migration layer
```

rather than silently rewriting historical meaning.

# 35. Durable Checkpoint

Agents may periodically persist:

```text
checkpoint {
    execution_id
    generation
    logical_position
    state_hash
    artifacts
}
```

This enables execution recovery.

# 36. Checkpoint vs Snapshot

A **snapshot** describes a persistent state representation.

A **checkpoint** represents recoverable progress of an operation.

They are related but not identical.

# 37. Checkpoint Semantics

Suppose an operation processes:

```text
items 1..100000
```

Checkpoint:

```text
position = 65000
```

After restart:

```text
resume from 65001
```

only if the operation semantics permit that safely.

# 38. Checkpoint Safety

The checkpoint must not claim:

```text
item 65000 processed
```

unless the corresponding side effect is durable or idempotently repeatable.

Otherwise:

```text
checkpoint says done
but side effect never happened
```

causes data loss.

# 39. Atomic Progress

Ideal:

```text
process item
+
commit progress
```

within one transaction.

If impossible:

```text
idempotent item processing
```

is required.

# 40. Journal vs Checkpoint

The journal records:

```text
what happened
```

The checkpoint records:

```text
where recovery can safely resume
```

# 41. Durable Lease State

Lease state must survive controller restart where necessary.

At minimum:

```text
lease_id
subject
generation
owner
expiry
state
```

# 42. Lease Recovery

After restart, the controller must determine:

```text
Which leases are still valid?
Which expired?
Which belong to the current epoch?
```

It cannot simply assume:

```text
all previous leases remain valid
```

# 43. Ownership Record

Ownership should be durable:

```text
Ownership {
    resource
    owner
    epoch
    generation
    state
}
```

This allows safe recovery.

# 44. Persistent Fencing

A new owner should advance:

```text
epoch
```

or another monotonic fencing token.

Example:

```text
owner A → fence token 40
owner B → fence token 41
```

All protected operations carrying token 40 become stale.

# 45. Monotonicity

Fencing tokens must never move backward:

```text
40 → 41 → 42
```

never:

```text
42 → 39
```

# 46. Durable Monotonic Counters

Counters used for:

```text
epoch
generation
sequence
```

must have persistence semantics strong enough to prevent reuse after restart.

# 47. Counter Reuse Hazard

Bad:

```text
restart
counter resets to 0
```

Old messages may suddenly appear valid again.

This can produce extremely dangerous stale-message acceptance.

# 48. Monotonic Identity

Prefer durable allocation of:

```text
epoch
generation
sequence
```

or use globally unique identifiers combined with explicit ordering metadata.

# 49. Durable Command Journal

Every externally significant command should have a durable identity.

```text
Command {
    command_id
    execution_id
    generation
    idempotency_key
    state
    created_at
}
```

# 50. Command Recovery

On restart:

```text
load commands
      ↓
find UNKNOWN
      ↓
query agents
      ↓
reconcile
      ↓
resolve
```

Unknown commands should not simply disappear.

# 51. Garbage Collection

Durable state grows indefinitely unless retention is defined.

NROS therefore needs explicit retention classes.

For example:

```text
LIVE
RECENT
ARCHIVED
EXPIRED
PURGED
```

# 52. Event Retention

Not all events need the same retention period.

Potential classes:

```text
operational telemetry → short
execution lifecycle → long
security/audit events → very long
terminal evidence → policy-defined
```

# 53. Never Purge Active Evidence

An event required to prove:

```text
current authority
current ownership
active execution
```

must not be garbage-collected.

# 54. Tombstones

Deletion itself may need evidence.

Instead of silently removing:

```text
Execution E17
```

record:

```text
EXECUTION_TOMBSTONED
```

# 55. Tombstone Purpose

Tombstones prevent an old message from recreating deleted state.

Example:

```text
E17 deleted
```

then stale:

```text
START(E17)
```

arrives.

The tombstone allows NROS to reject it.

# 56. Tombstone Retention

Tombstones must remain long enough to cover the maximum relevant stale-message window.

This is a protocol property, not merely a storage cleanup preference.

# 57. Corruption Detection

Durable records should support integrity checking.

Examples:

```text
checksum
hash
schema validation
length validation
```

# 58. Event Hashing

An event can contain:

```text
payload_hash
```

or even:

```text
previous_hash
```

for stronger tamper-evidence.

# 59. Hash Chain

Conceptually:

```text
E1
 ↓ hash
E2
 ↓ hash
E3
 ↓ hash
E4
```

If E2 is modified, downstream integrity checks fail.

This is useful for audit-oriented journals.

# 60. Hash Chain Is Not Consensus

A hash chain proves linkage/integrity of stored records.

It does **not** by itself prove:

```text
who was authoritative
```

or:

```text
whether the event was true
```

Those require authority and provenance.

# 61. Evidence Envelope

A strong NROS evidence record can contain:

```text
Evidence {
    evidence_id
    type
    subject
    producer
    producer_incarnation
    authority
    generation
    observed_at
    received_at
    sequence
    payload_hash
    provenance
}
```

# 62. Evidence Levels

The earlier evidence model can be formalized:

```text
UNKNOWN
   ↓
OBSERVED
   ↓
VALIDATED
   ↓
AUTHORITY-VALIDATED
   ↓
COMMITTED
```

Not every observation reaches the highest level.

# 63. Observation vs Commitment

Example:

```text
Agent says:
"execution finished"
```

This is:

```text
OBSERVED
```

After validation:

```text
execution result verified
```

may become:

```text
VALIDATED
```

After durable state transition:

```text
COMMITTED
```

# 64. Evidence Provenance

Every committed fact should answer:

```text
WHO?
WHAT?
WHEN?
UNDER WHICH AUTHORITY?
FROM WHICH GENERATION?
BASED ON WHICH EVIDENCE?
```

# 65. Durable Audit Record

For important transitions:

```text
AuditEntry {
    event_id
    actor
    authority
    operation
    subject
    previous_state
    new_state
    reason
    evidence
}
```

# 66. Reason Codes

Do not rely solely on free-form text:

```text
reason = "because it looked stale"
```

Use structured:

```text
reason_code = LEASE_EXPIRED
```

with optional explanation.

# 67. Policy Version

A reconciliation decision should record:

```text
policy_version
```

because future policy changes should not make historical decisions uninterpretable.

# 68. Deterministic Replay

A major architectural goal:

```text
historical state
+
events
+
policy version
```

should reproduce the decision that was made.

# 69. Replayability

This enables:

```text
incident investigation
testing
debugging
migration
audit
```

# 70. Replay Must Be Side-Effect Free

Historical replay must never accidentally execute:

```text
START
STOP
DELETE
DEPLOY
```

Replay should calculate state, not perform real-world actions.

# 71. Simulation Mode

NROS can expose:

```text
RECONCILE_SIMULATE
```

which computes:

```text
classification
decision
expected transition
```

without executing the action.

# 72. Dry Run

Example:

```text
desired = RUNNING
observed = STOPPED

simulation:
    classification = DRIFT
    proposed_action = START
```

No START is dispatched.

# 73. Storage Failure

Now consider the storage system itself failing.

```text
Controller
    ↓
State Store
    X
```

NROS must not assume:

```text
write succeeded
```

because the request was sent.

# 74. Storage Write Ambiguity

Exactly the same uncertainty problem applies:

```text
WRITE
 ↓
timeout
```

Could mean:

```text
not committed
committed
unknown
```

Therefore storage operations need transaction identifiers or safe retry semantics.

# 75. Commit Tokens

Where supported, a transaction can expose:

```text
commit_id
```

After timeout, NROS can query:

```text
status(commit_id)
```

rather than repeating the transaction blindly.

# 76. Durable Store Requirements

The state store should ideally support:

```text
atomic transactions
conditional writes
durable commits
monotonic versions
unique constraints
range queries
durable ordering where required
```

# 77. Minimal Persistence Contract

The NROS persistence abstraction should define semantics rather than exposing database-specific behavior.

Conceptually:

```text
trait StateStore {
    read()
    conditional_write()
    transaction()
    append_event()
    load_snapshot()
}
```

The exact API can vary.

# 78. Persistence Capability Model

A backend can advertise:

```text
PersistenceCapabilities {
    transactions
    compare_and_swap
    durable_append
    ordering
    snapshots
    conditional_delete
    unique_constraints
}
```

# 79. Backend Qualification

NROS should not silently assume every backend provides the same guarantees.

A weak backend must produce a weaker runtime capability profile.

# 80. Capability-Gated Features

For example:

```text
if transactions == false:
    exactly_atomic_state_event = unavailable
```

Likewise:

```text
if conditional_write == false:
    optimistic_concurrency = unavailable
```

# 81. No False Capability Claims

This follows the broader NROS principle:

> **A feature is not operational merely because an interface exists for it.**

The underlying persistence guarantee must be demonstrated.

# 82. Bootstrap Recovery

On controller startup:

```text
BOOT
 ↓
OPEN STORE
 ↓
VERIFY SCHEMA
 ↓
VERIFY JOURNAL
 ↓
LOAD SNAPSHOTS
 ↓
REPLAY DELTA
 ↓
RECONCILE ACTIVE STATE
 ↓
RECOVERY COMPLETE
 ↓
NORMAL OPERATION
```

# 83. Recovery Barrier

Again:

```text
NORMAL_OPERATION
```

must not begin before:

```text
RECOVERY_COMPLETE
```

# 84. Recovery Failure

If storage integrity fails:

```text
STORE_CORRUPTION
```

NROS should enter:

```text
DEGRADED
```

or:

```text
RECOVERY_BLOCKED
```

rather than inventing current state.

# 85. Read-Only Recovery

A useful mode:

```text
RECOVERY_READ_ONLY
```

allows:

```text
inspect
verify
export evidence
```

while preventing unsafe mutations.

# 86. Write Protection

During uncertain recovery:

```text
scheduling = disabled
mutations = restricted
side_effects = blocked
```

until authority is re-established.

# 87. Disaster Recovery

Process restart is not the only failure.

Consider:

```text
storage loss
machine loss
region loss
backup corruption
```

NROS should define recovery tiers.

# 88. Recovery Tiers

```text
Tier 0 — process restart
Tier 1 — node failure
Tier 2 — storage failure
Tier 3 — control-plane failure
Tier 4 — disaster recovery
```

Each requires different evidence.

# 89. Backup

Backups should preserve:

```text
state
events
metadata
schema version
integrity information
```

not merely current tables.

# 90. Restore Verification

A restored database is not automatically trustworthy.

Perform:

```text
restore
 ↓
integrity check
 ↓
schema check
 ↓
journal consistency
 ↓
snapshot verification
 ↓
reconciliation
```

# 91. Recovery Point Objective

NROS documentation should define:

```text
RPO
```

How much durable history may be lost?

# 92. Recovery Time Objective

Also:

```text
RTO
```

How quickly must the control plane return to operational state?

# 93. These Are Architecture Constraints

RPO/RTO cannot be decided only operationally.

They affect:

```text
journal frequency
snapshot frequency
replication
storage selection
checkpoint strategy
```

# 94. Replication

If durable state is replicated:

```text
Primary
   ↓
Replica A
Replica B
```

the system must define when a write is considered durable.

For example:

```text
local commit
```

versus:

```text
quorum commit
```

are materially different guarantees.

# 95. Replication Is Not Backup

Replication protects availability.

Backup protects against:

```text
corruption
accidental deletion
logical destruction
```

A corrupted primary can replicate corruption to every replica.

# 96. Audit Immutability

If audit history matters, consider append-only storage semantics:

```text
append
```

rather than:

```text
update
delete
```

# 97. Storage Security Boundary

Durability also introduces security requirements:

```text
encryption
access control
key management
integrity verification
tenant isolation
```

These are part of the persistence contract.

# 98. Multi-Tenant Isolation

Every durable record should have an explicit ownership scope where applicable:

```text
tenant_id
workspace_id
project_id
```

Queries must enforce isolation at the storage boundary.

# 99. No Ambient Tenant Context

Dangerous:

```text
store.get(execution_id)
```

if execution IDs alone are not globally scoped.

Safer:

```text
store.get(tenant, execution_id)
```

or use globally unique identifiers with enforced authorization.

# 100. Durable State Invariants

```text
1. Every authoritative mutable state has one defined source of truth.

2. Volatile state must be reconstructible.

3. Cache contents must never outrank authoritative state.

4. Durable state carries explicit versions.

5. Event streams carry stable identities.

6. Per-stream ordering is explicit.

7. Causation is recorded where relevant.

8. Correlation is recorded where relevant.

9. State transitions and required events commit atomically where supported.

10. Outbox entries survive process failure.

11. Inbox entries enable duplicate suppression.

12. Snapshots are derived state, not historical truth.

13. Snapshots are versioned and integrity-checked.

14. Checkpoints cannot claim progress that is not safely recoverable.

15. Epochs, generations, and versions are distinct.

16. Monotonic fencing values cannot regress.

17. Command identities survive restart.

18. Unknown commands remain discoverable.

19. Tombstones prevent stale recreation.

20. Historical evidence is not silently rewritten.

21. Garbage collection respects active authority and stale-message windows.

22. Important evidence has provenance.

23. Policy versions accompany consequential decisions.

24. Replay is side-effect free.

25. Storage capability limits runtime guarantees.

26. Backend limitations are exposed rather than hidden.

27. Recovery starts behind a barrier.

28. Recovery failure produces an explicit degraded state.

29. Backups include both state and required historical evidence.

30. Restore operations perform integrity and reconciliation checks.

31. Replication and backup are treated as different mechanisms.

32. Tenant isolation is enforced by durable-state access semantics.

33. Durability guarantees are explicitly scoped.

34. No operation is declared committed merely because a request was sent.

35. Storage ambiguity is handled with the same UNKNOWN semantics used elsewhere.
```

# 101. Unified NROS Persistence Model

```text
                         ┌──────────────────┐
                         │     COMMAND      │
                         └────────┬─────────┘
                                  ↓
                         ┌──────────────────┐
                         │    TRANSACTION   │
                         └────────┬─────────┘
                                  │
                  ┌───────────────┼────────────────┐
                  ↓               ↓                ↓
             STATE STORE       EVENT LOG        OUTBOX
                  │               │                │
                  ↓               ↓                ↓
             SNAPSHOT         JOURNAL          BROKER
                  │
                  ↓
             CHECKPOINT
                  │
                  ↓
              RECOVERY
                  │
                  ↓
             RECONCILIATION
```

# 102. The Persistence Principle

The core principle is:

> **Persist enough information to distinguish what was intended, what was observed, what was authorized, what was committed, and what remains uncertain.**

That gives NROS a durable foundation for the reconciliation model established in Parts CI–CII.

# Part CIII — Scheduling Semantics & Resource Arbitration

The next layer should define the **scheduler itself**:

```text
Work admission
→ priority
→ dependencies
→ resource requests
→ placement
→ reservations
→ fairness
→ preemption
→ dispatch
→ backpressure
→ cancellation
→ rescheduling
```

The key question becomes:

> **How does NROS decide which work is allowed to execute next without violating authority, resource, dependency, fairness, and safety constraints?**

# NROS — Part CIII: Scheduling Semantics & Resource Arbitration

We now move from **durability and recovery** into the scheduling core.

The scheduler should not be modeled as:

```text
queue.pop()
→ execute()
```

That is too weak for NROS.

A correct scheduler must answer simultaneously:

```text
Is this Work admissible?
Is it authorized?
Are its dependencies satisfied?
Are resources available?
Is placement valid?
Does policy permit execution now?
Would scheduling it violate fairness?
Can it be safely cancelled/preempted?
```

# 1. Scheduling Is a Decision Function

Conceptually:

```text
ScheduleDecision =
    f(
        desired_work,
        dependencies,
        resources,
        authority,
        policy,
        fairness,
        placement,
        current_state
    )
```

The scheduler produces a **decision**, not merely a queue operation.

# 2. Work vs Execution

Keep the distinction:

```text
WORK
  ↓
EXECUTION
  ↓
ATTEMPT
```

Work represents the logical request.

Execution represents one execution lifecycle.

Attempt represents one concrete execution attempt.

# 3. Scheduler Input

A scheduling candidate may contain:

```text
WorkCandidate {
    work_id
    priority
    deadline
    dependencies
    resource_request
    placement_constraints
    retry_policy
    execution_policy
    tenant
}
```

# 4. Admission Before Scheduling

Not every submitted Work should immediately enter the runnable queue.

Pipeline:

```text
SUBMITTED
   ↓
VALIDATED
   ↓
AUTHORIZED
   ↓
ADMITTED
   ↓
READY
```

# 5. Admission Control

Admission verifies:

```text
schema
policy
authority
quotas
dependencies
resource request
```

Only then:

```text
READY
```

# 6. READY ≠ RUNNING

This distinction is fundamental.

```text
READY
```

means:

> The Work may run when resources and scheduling policy permit it.

It does not mean:

> Resources have already been assigned.

# 7. Scheduling Pipeline

```text
SUBMITTED
    ↓
VALIDATED
    ↓
AUTHORIZED
    ↓
ADMITTED
    ↓
READY
    ↓
SELECTED
    ↓
RESERVED
    ↓
DISPATCHED
    ↓
STARTING
    ↓
RUNNING
```

# 8. Selection vs Reservation

These must remain separate.

### Selected

Scheduler chooses:

```text
Work W
```

### Reserved

Resources are committed:

```text
CPU = 2
MEM = 4GiB
```

Selection can fail to produce reservation.

# 9. Why?

Between:

```text
SELECT
```

and:

```text
RESERVE
```

another scheduler may consume the resource.

Therefore reservation requires an atomic or conditional resource operation.

# 10. Scheduling Race

Two controllers:

```text
A → sees 4 CPUs
B → sees 4 CPUs
```

Both select:

```text
Work X = 3 CPUs
```

Without conditional reservation:

```text
A reserves 3
B reserves 3
```

Result:

```text
6 CPUs allocated from 4
```

# 11. Reservation Token

A reservation should have identity:

```text
Reservation {
    reservation_id
    work_id
    execution_id
    resource
    amount
    owner
    epoch
    state
}
```

# 12. Reservation Lifecycle

```text
REQUESTED
   ↓
RESERVED
   ↓
BOUND
   ↓
CONSUMED
   ↓
RELEASE_PENDING
   ↓
RELEASED
```

Failure:

```text
RESERVED
   ↓
EXPIRED
```

# 13. Reservation TTL

Reservations should not remain forever.

A failed controller must not permanently consume:

```text
CPU
memory
GPU
storage
ports
```

Therefore:

```text
reservation_expiry
```

must be explicit.

# 14. Reservation Expiration

Expiration does not automatically mean:

```text
Work failed
```

It means:

```text
reservation authority expired
```

The scheduler then reconciles Work.

# 15. Resource Request

Resource requests should be structured:

```text
ResourceRequest {
    cpu
    memory
    storage
    accelerator
    network
    custom
}
```

# 16. Hard vs Soft Requirements

Example:

```text
CPU >= 2
```

may be hard.

While:

```text
prefer locality = true
```

may be soft.

# 17. Constraints

Placement constraints may include:

```text
required_labels
forbidden_labels
architecture
region
zone
capability
security_domain
data_locality
```

# 18. Constraints vs Preferences

Keep them distinct:

```text
constraint → MUST satisfy

preference → SHOULD satisfy
```

Otherwise the scheduler cannot reason about feasibility correctly.

# 19. Feasibility

A candidate is feasible only if:

```text
authority
AND
dependencies
AND
hard constraints
AND
resource availability
AND
policy
```

are all satisfied.

# 20. Candidate Filtering

Conceptually:

```text
Candidates
   ↓
authorized?
   ↓
dependencies?
   ↓
resources?
   ↓
placement?
   ↓
policy?
   ↓
fairness?
   ↓
eligible set
```

# 21. Eligibility vs Priority

A high-priority Work that is impossible to execute should not block everything else.

Example:

```text
Priority 100
requires GPU
```

while:

```text
Priority 50
requires CPU
```

is runnable.

The GPU job remains:

```text
WAITING_FOR_RESOURCE
```

while CPU work proceeds.

# 22. Priority

Priority is one scheduling dimension, not the entire scheduler.

Potential ordering:

```text
priority
deadline
fairness debt
age
dependency criticality
resource efficiency
```

# 23. Strict Priority Problem

Naive:

```text
always run highest priority
```

can starve lower-priority Work.

# 24. Aging

Waiting Work can gain effective priority:

```text
effective_priority =
base_priority
+
aging_factor × wait_time
```

This reduces starvation.

# 25. Fairness

NROS may need fairness across:

```text
tenant
user
project
queue
agent
resource pool
```

# 26. Weighted Fairness

For tenants:

```text
Tenant A weight = 4
Tenant B weight = 1
```

A may receive approximately four times the scheduling share under sustained contention, subject to constraints.

# 27. Fairness Is Not Equality

A scheduler may intentionally provide:

```text
different weights
different quotas
different priorities
```

Fairness means adherence to the defined policy, not equal resource distribution.

# 28. Quota

Quota defines maximum permitted usage.

Example:

```text
Tenant A
CPU quota = 100
```

Even if the cluster has 500 CPUs available, A cannot exceed its quota unless policy permits borrowing.

# 29. Quota vs Capacity

These are distinct:

```text
capacity = what exists

quota = what this principal may consume
```

# 30. Quota vs Reservation

Quota controls permission.

Reservation controls allocation.

Example:

```text
quota = 100 CPU
reserved = 30 CPU
```

Remaining available quota:

```text
70 CPU
```

subject to policy.

# 31. Hierarchical Quotas

Possible structure:

```text
Organization
 ├── Team A
 │    ├── Project A1
 │    └── Project A2
 └── Team B
```

Effective quota may be constrained at every level.

# 32. Resource Accounting

NROS should track:

```text
requested
reserved
allocated
used
released
```

These are not interchangeable.

# 33. Requested vs Reserved

```text
requested = 8 CPU
reserved  = 8 CPU
```

means resources have been committed.

# 34. Reserved vs Used

The execution may actually consume:

```text
used = 3 CPU
```

while:

```text
reserved = 8 CPU
```

This difference matters for utilization analysis.

# 35. Overcommit

Some systems allow:

```text
reserved > physical capacity
```

under controlled assumptions.

If NROS supports this, it must be an explicit resource policy.

# 36. Backpressure

If resources are exhausted:

```text
READY
  ↓
WAITING_FOR_RESOURCE
```

rather than repeatedly attempting dispatch.

# 37. Retry Storm

Bad scheduler:

```text
try
fail
try
fail
try
fail
```

This wastes control-plane resources.

Better:

```text
WAITING
+
event-driven wakeup
```

or controlled backoff.

# 38. Resource Events

When resources change:

```text
RESOURCE_AVAILABLE
```

the scheduler can reconsider waiting candidates.

# 39. Backoff

If polling is necessary:

```text
100ms
200ms
400ms
800ms
...
```

with a maximum bound and jitter.

# 40. Backoff Is Not Scheduling Policy

Backoff determines:

> When should I reconsider?

Scheduling policy determines:

> What should I choose?

Keep them separate.

# 41. Dependency Graph

Work may depend on other Work:

```text
A
↓
B
↓
C
```

B is not READY until A reaches an acceptable terminal state.

# 42. Dependency Types

Possible semantics:

```text
SUCCESS
COMPLETION
FAILURE
ARTIFACT_AVAILABLE
STATE_MATCH
RESOURCE_AVAILABLE
```

# 43. Success Dependency

```text
B runs only if A = COMPLETED
```

# 44. Completion Dependency

```text
B runs when A reaches any terminal state.
```

# 45. Conditional Dependency

```text
if A.result == SUCCESS
    → B
else
    → C
```

This makes the dependency graph a control-flow structure.

# 46. Dependency Cycles

A cycle:

```text
A → B
B → C
C → A
```

must be detected.

Otherwise all three remain:

```text
WAITING
```

forever.

# 47. Cycle Detection

Admission should reject cycles where the dependency model requires an acyclic graph.

```text
graph validation
→ cycle
→ reject
```

# 48. Dynamic Dependencies

Some systems discover dependencies during execution.

Then:

```text
RUNNING
 ↓
new dependency discovered
```

must not violate the current execution's state invariants.

This requires explicit dynamic-dependency semantics.

# 49. Critical Path

For dependency graphs, scheduler can prioritize work on the critical path.

Example:

```text
A ──→ C ──→ D
 \
  → B ─────→ D
```

Work affecting D's completion may receive higher scheduling value.

# 50. Deadline

A Work may specify:

```text
deadline
```

Scheduler can calculate:

```text
slack =
deadline - expected_completion_time
```

# 51. Deadline Miss

A Work that cannot meet its deadline may be:

```text
DEADLINE_AT_RISK
```

rather than silently continuing under the assumption that the deadline remains achievable.

# 52. Deadline Policy

Possible policies:

```text
BEST_EFFORT
MUST_START_BEFORE
MUST_COMPLETE_BEFORE
CANCEL_IF_MISSED
```

# 53. Preemption

Preemption means:

> Temporarily or permanently removing execution from its current resource allocation.

It should never be treated as universally safe.

# 54. Preemption Capability

An execution can declare:

```text
PREEMPTABLE
CHECKPOINTABLE
MIGRATABLE
NON_PREEMPTABLE
```

# 55. Cooperative Preemption

Preferred when possible:

```text
scheduler
   ↓
PREEMPT_REQUEST
   ↓
agent
   ↓
checkpoint
   ↓
release resources
```

# 56. Forced Preemption

If the agent refuses or becomes unresponsive:

```text
FORCE_TERMINATE
```

may be required.

This should require stronger authority and evidence.

# 57. Preemption Is Not Cancellation

Preemption:

```text
pause / relocate / suspend
```

Cancellation:

```text
terminate logical Work
```

These must remain separate.

# 58. Suspension

An execution may support:

```text
RUNNING
 ↓
SUSPENDED
 ↓
RESUMING
 ↓
RUNNING
```

Only if the underlying workload supports safe suspension.

# 59. Migration

Migration:

```text
Agent A
   ↓
checkpoint
   ↓
Agent B
   ↓
resume
```

requires checkpoint correctness and authority transfer.

# 60. Resource Rebalancing

Scheduler may migrate work when:

```text
node overloaded
```

to:

```text
underutilized node
```

but migration must preserve execution identity semantics.

# 61. Placement

Placement can be modeled:

```text
PlacementDecision {
    agent
    resource_pool
    zone
    reason
    score
}
```

# 62. Scoring

Candidate placement might calculate:

```text
score =
resource_fit
+
locality
+
fairness
+
priority
+
load_balance
-
migration_cost
```

The exact formula is policy-dependent.

# 63. Determinism

Given identical scheduler inputs:

```text
same state
same policy
same candidate set
```

the scheduler should ideally produce the same decision.

This greatly improves:

```text
testing
replay
debugging
auditability
```

# 64. Tie Breaking

If two candidates have equal score, use deterministic tie-breaking:

```text
priority
→ deadline
→ creation sequence
→ stable ID
```

Never depend on hash-map iteration order.

# 65. Scheduler Clock

Time-dependent scheduling must use an explicit clock abstraction.

This enables:

```text
deterministic tests
```

without depending on real wall-clock time.

# 66. Queue Semantics

A queue should not itself represent truth.

Example:

```text
READY queue
```

is a scheduling index.

The authoritative Work state remains in the durable state model.

# 67. Stale Queue Entry

A queue can contain:

```text
Work W
```

while W has already become:

```text
CANCELLED
```

Scheduler must revalidate before dispatch.

# 68. Queue Revalidation

```text
pop candidate
 ↓
load authoritative state
 ↓
verify eligibility
 ↓
schedule
```

not:

```text
pop
→ blindly execute
```

# 69. Queue Deduplication

The same Work may accidentally appear multiple times.

Use:

```text
work_id
version
```

or a durable scheduling key to avoid duplicate scheduling decisions.

# 70. Scheduler Epoch

A scheduler instance can carry:

```text
scheduler_epoch
```

to identify its control-plane incarnation.

A new scheduler epoch fences stale scheduler instances.

# 71. Multiple Schedulers

If NROS permits multiple scheduler instances:

```text
Scheduler A
Scheduler B
```

they need explicit coordination.

Possible models:

```text
single leader
partition ownership
distributed optimistic scheduling
```

# 72. Leader Scheduler

Simplest strong model:

```text
ONE ACTIVE SCHEDULER
```

with:

```text
leader lease
```

Other instances remain standby.

# 73. Partitioned Scheduler

For scale:

```text
Partition 1 → Scheduler A
Partition 2 → Scheduler B
```

Each scheduler owns only its partition.

# 74. Partition Ownership

The ownership protocol established earlier applies directly:

```text
partition
owner
epoch
```

must be durable and fenced.

# 75. Distributed Optimistic Scheduling

Multiple schedulers may independently propose decisions:

```text
Scheduler A → reserve
Scheduler B → reserve
```

The resource manager/state store resolves conflicts using conditional transactions.

# 76. Scheduler Does Not Own Resources

Important architectural distinction:

```text
Scheduler
    → decides

Resource Manager
    → commits allocation
```

The scheduler should not pretend its local view constitutes allocation.

# 77. Two-Phase Scheduling

Conceptually:

```text
PHASE 1
candidate selection

PHASE 2
resource reservation
```

If reservation fails:

```text
selection invalidated
```

and another candidate is considered.

# 78. Avoid Distributed 2PC Unless Necessary

A full distributed two-phase commit across scheduler, resource manager, and agents may become expensive and fragile.

Prefer:

```text
local transactional authority
+
idempotent asynchronous dispatch
+
reconciliation
```

where possible.

# 79. Scheduler Backpressure

The scheduler should limit:

```text
dispatch concurrency
```

so it does not flood Agents or resource managers.

# 80. Dispatch Window

Example:

```text
max_inflight_dispatches = N
```

When full:

```text
READY remains READY
```

rather than creating unbounded pending commands.

# 81. Agent Capacity

Agents advertise:

```text
capacity
available
capabilities
current_load
```

But these observations are not automatically authoritative.

The resource manager remains authoritative for committed allocations when applicable.

# 82. Capacity Changes

An Agent may report:

```text
available CPU = 2
```

while the resource manager knows:

```text
reserved CPU = 4
```

Scheduler must reconcile these views.

# 83. Resource Hierarchy

Resources may be hierarchical:

```text
Cluster
 └── Node
      └── NUMA
           └── CPU
```

Placement decisions may therefore require multiple constraints.

# 84. Dominant Resource

For multi-resource fairness, the scheduler may need to reason about:

```text
CPU
MEM
GPU
IO
```

rather than a single scalar capacity.

# 85. Multi-Dimensional Fit

A Work requiring:

```text
4 CPU
16 GB RAM
```

cannot be scheduled merely because:

```text
CPU ≥ 4
```

Memory must also fit.

# 86. Fragmentation

Available resources may total:

```text
8 CPU
```

but be distributed as:

```text
2 + 2 + 2 + 2
```

while Work requires:

```text
4 contiguous CPUs
```

if topology requires contiguity.

The scheduler must understand resource topology where relevant.

# 87. Affinity

Affinity says:

```text
prefer/require co-location
```

Example:

```text
database + cache
```

may benefit from the same node.

# 88. Anti-Affinity

Anti-affinity:

```text
do not place together
```

can increase fault tolerance.

Example:

```text
replica A ≠ replica B node
```

# 89. Fault Domains

Placement may consider:

```text
rack
zone
region
power domain
network domain
```

This prevents correlated failures.

# 90. Data Locality

Moving computation to data may be cheaper than moving data to computation.

Scheduler can therefore include:

```text
data_locality_score
```

as a placement preference.

# 91. Security Placement

Some Work may require:

```text
trusted execution environment
specific security domain
restricted network
```

These should be hard constraints when mandated.

# 92. Capability Matching

Agent:

```text
capabilities = [GPU, CUDA, WASM]
```

Work:

```text
requires = [GPU]
```

is compatible.

# 93. Capability Version

Capability names may need versions:

```text
CUDA >= 13
```

rather than merely:

```text
CUDA = true
```

# 94. Resource Classes

NROS can classify resources:

```text
COMPUTE
MEMORY
STORAGE
NETWORK
ACCELERATOR
DEVICE
LICENSE
CUSTOM
```

# 95. Consumable vs Non-Consumable

CPU is generally consumable:

```text
2 / 8
```

A capability like:

```text
supports_wasm = true
```

is non-consumable.

# 96. Exclusive Resources

Some devices require exclusive ownership:

```text
GPU 0
```

must not be simultaneously allocated to incompatible Work.

# 97. Shared Resources

Other resources can be shared:

```text
network bandwidth
```

with policy-defined limits.

# 98. Scheduler Decision Record

Every significant scheduling decision should be explainable:

```text
SchedulingDecision {
    work_id
    selected_agent
    resource_request
    priority
    score
    policy_version
    constraints_checked
    reservation_id
    reason
}
```

# 99. Explainability

A scheduler should be able to answer:

> Why didn't this Work run?

Possible structured reasons:

```text
WAITING_DEPENDENCY
WAITING_RESOURCE
QUOTA_EXCEEDED
NO_ELIGIBLE_AGENT
POLICY_BLOCKED
DEADLINE_POLICY
PREEMPTION_BLOCKED
AUTHORITY_INVALID
```

# 100. Scheduling Invariants

```text
1. Work and Execution are distinct identities.

2. Admission precedes readiness.

3. READY does not imply resource allocation.

4. Selection does not imply reservation.

5. Reservation must be authoritative.

6. Resource allocation must not rely solely on scheduler-local observations.

7. Hard constraints cannot be violated by scoring preferences.

8. Preferences cannot override mandatory policy.

9. Priority alone must not imply unlimited resource access.

10. Fairness is policy-defined.

11. Quotas constrain permitted consumption.

12. Capacity describes actual available resources.

13. Requested, reserved, allocated, and used are distinct states.

14. Reservations have explicit identity and lifecycle.

15. Reservations expire safely.

16. Expired reservations do not automatically imply Work failure.

17. Stale queue entries are revalidated before dispatch.

18. Duplicate queue entries cannot create duplicate executions.

19. Scheduler decisions are deterministic under identical inputs where practical.

20. Tie-breaking is deterministic.

21. Dependency cycles are rejected or explicitly handled.

22. Dependency state is authoritative.

23. Preemption requires explicit capability.

24. Preemption is distinct from cancellation.

25. Migration requires checkpoint/recovery semantics.

26. Resource topology can constrain placement.

27. Affinity and anti-affinity are explicit.

28. Security placement constraints cannot be bypassed by optimization.

29. Scheduler instances require explicit authority.

30. Scheduler epochs prevent stale scheduler actions.

31. Partition ownership is fenced.

32. Dispatch concurrency is bounded.

33. Backpressure prevents retry storms.

34. Resource managers, not scheduler guesses, determine committed allocation.

35. Scheduling decisions are auditable.

36. Every blocked Work has a structured reason.

37. Scheduling does not bypass authorization.

38. Scheduling does not bypass recovery barriers.

39. Scheduling does not create side effects before required reservation/authority.

40. Scheduler behavior remains compatible with reconciliation semantics.
```

# 101. Unified Scheduling Architecture

```text
                    ┌───────────────┐
                    │     WORK      │
                    └───────┬───────┘
                            ↓
                    ┌───────────────┐
                    │   ADMISSION   │
                    └───────┬───────┘
                            ↓
                    ┌───────────────┐
                    │     READY     │
                    └───────┬───────┘
                            ↓
                 ┌─────────────────────┐
                 │ ELIGIBILITY FILTER  │
                 └──────────┬──────────┘
                            ↓
                 ┌─────────────────────┐
                 │ POLICY / FAIRNESS   │
                 └──────────┬──────────┘
                            ↓
                 ┌─────────────────────┐
                 │ PLACEMENT / SCORING │
                 └──────────┬──────────┘
                            ↓
                 ┌─────────────────────┐
                 │ RESOURCE RESERVATION│
                 └──────────┬──────────┘
                            ↓
                 ┌─────────────────────┐
                 │     DISPATCH        │
                 └──────────┬──────────┘
                            ↓
                 ┌─────────────────────┐
                 │     RECONCILE       │
                 └─────────────────────┘
```

# 102. Scheduler's Fundamental Contract

The scheduler should therefore promise:

> **Given authoritative state, policy, available resources, and valid authority, produce a deterministic, explainable decision that does not violate resource, dependency, fairness, or execution invariants.**

It should **not** promise that every selected Work will successfully execute.

That belongs to the execution and reconciliation layers.

# Part CIV — Agent Execution Contract & Data-Plane Semantics

The next layer should descend into the **Agent** itself:

```text
registration
→ capability negotiation
→ command acceptance
→ execution isolation
→ process lifecycle
→ stdout/stderr
→ heartbeats
→ checkpoints
→ result publication
→ cancellation
→ fencing
→ crash recovery
```

The key question becomes:

> **What exactly does an NROS Agent promise when it receives a command, and how does the control plane distinguish acceptance, execution, completion, and durable result?**

# NROS — Part CIV: Agent Execution Contract & Data-Plane Semantics

We now cross the boundary from **control-plane scheduling** into the **data plane**.

The scheduler answers:

> *Which Work should execute, where, and under what authority?*

The Agent answers:

> *What does it mean to actually execute that Work, report progress, handle interruption, and produce a trustworthy result?*

The crucial rule is:

> **Receiving a command is not the same thing as executing it, and execution is not the same thing as durable completion.**

# 1. Agent Model

An NROS Agent should be treated as an execution authority subordinate to the control plane.

```text
Control Plane
      │
      │ command
      ↓
   Agent
      │
      ├── process
      ├── resources
      ├── I/O
      ├── checkpoint
      └── result
```

The Agent should not independently redefine global Work state.

# 2. Agent Lifecycle

An Agent itself has a lifecycle:

```text
CREATED
   ↓
REGISTERING
   ↓
READY
   ↓
DRAINING
   ↓
OFFLINE
```

Failure:

```text
READY
  ↓
UNRESPONSIVE
```

# 3. Registration

Registration establishes:

```text
agent_id
incarnation
capabilities
resource inventory
protocol version
runtime version
security identity
```

# 4. Agent Incarnation

Every Agent process should have an incarnation identifier:

```text
agent_id = A17
incarnation = I42
```

After restart:

```text
agent_id = A17
incarnation = I43
```

This prevents old messages from being confused with the new process.

# 5. Agent Epoch

If Agent ownership itself participates in fencing:

```text
agent_epoch
```

can identify the current control-plane authority for that Agent.

# 6. Registration Is Not Readiness

An Agent can register successfully but remain:

```text
INITIALIZING
```

until:

```text
capabilities verified
resources verified
runtime healthy
security checks complete
```

Only then:

```text
READY
```

# 7. Capability Advertisement

Agent capabilities should be structured:

```text
Capability {
    name
    version
    attributes
    limits
}
```

Example:

```text
rust-runtime
version = 1.91
targets = [x86_64-linux]
```

# 8. Capability Verification

Self-reported capabilities are observations.

The control plane may require verification:

```text
ADVERTISED
   ↓
PROBED
   ↓
VALIDATED
```

# 9. Resource Advertisement

Agent may report:

```text
CPU = 8
memory = 32 GiB
GPU = 1
```

But the scheduler/resource manager must distinguish:

```text
physical capacity
reserved capacity
available capacity
observed utilization
```

# 10. Command Identity

Every command must have a unique identity:

```text
command_id
```

The command should also identify:

```text
work_id
execution_id
attempt_id
generation
```

# 11. Command Envelope

Conceptually:

```text
Command {
    command_id
    work_id
    execution_id
    attempt_id
    generation
    issuer
    operation
    payload
    deadline
    idempotency_key
}
```

# 12. Command Authentication

The Agent must validate:

```text
issuer identity
authority
generation
command signature/authentication
```

before executing.

# 13. Fencing at the Agent

Suppose:

```text
generation = 12
```

is current.

A stale controller sends:

```text
generation = 11
```

The Agent must reject it:

```text
STALE_GENERATION
```

# 14. Command Acceptance

A command lifecycle should distinguish:

```text
RECEIVED
VALIDATED
ACCEPTED
STARTING
RUNNING
```

Acceptance means:

> The Agent has durably or otherwise reliably recorded responsibility for the command.

It does **not** necessarily mean the workload has started.

# 15. Acceptance vs Execution

Example:

```text
ACCEPTED
```

may be followed by:

```text
STARTING
```

for several seconds.

The control plane must not interpret acceptance as successful execution.

# 16. Durable Command Inbox

An Agent should maintain an inbox where necessary:

```text
command_id
state
received_at
attempt
```

This supports duplicate suppression.

# 17. Duplicate Command

The same command may arrive twice:

```text
START(command=42)
START(command=42)
```

The Agent should recognize the second occurrence.

Correct behavior:

```text
return existing command state
```

not:

```text
start second process
```

# 18. Idempotency Scope

Idempotency keys must have a defined scope.

For example:

```text
tenant + operation + idempotency_key
```

or:

```text
agent + command_id
```

depending on the protocol.

# 19. Process Identity

Once execution starts, the Agent should record:

```text
process_id
execution_id
attempt_id
started_at
```

where applicable.

# 20. Attempt Identity

Retries must create distinct attempts:

```text
execution E17
    ├── attempt A1
    └── attempt A2
```

This prevents one failed process from being confused with its replacement.

# 21. Attempt Lifecycle

```text
CREATED
 ↓
STARTING
 ↓
RUNNING
 ↓
COMPLETING
 ↓
COMPLETED
```

Failure:

```text
RUNNING
 ↓
FAILED
```

Cancellation:

```text
RUNNING
 ↓
CANCELLING
 ↓
CANCELLED
```

# 22. Process State Is Not Execution State

A process can disappear while the logical execution remains:

```text
UNKNOWN
```

The control plane must reconcile.

It should not immediately invent:

```text
FAILED
```

unless evidence justifies that conclusion.

# 23. Start Boundary

A particularly important transition:

```text
STARTING → RUNNING
```

should have clear semantics.

Possible evidence:

```text
process created
process acknowledged
health probe succeeded
first output received
```

NROS must specify which event constitutes **RUNNING**.

# 24. Process Created ≠ Healthy

A process can exist but be unusable:

```text
PID exists
```

while:

```text
application failed initialization
```

Therefore lifecycle state should not rely solely on PID existence.

# 25. Readiness

Long-running Agents may expose:

```text
STARTED
READY
```

separately.

Example:

```text
STARTING
 ↓
RUNNING
 ↓
READY
```

if application-level readiness matters.

# 26. Health

Health is another dimension:

```text
healthy
degraded
unhealthy
unknown
```

It should not necessarily replace execution state.

# 27. Orthogonal State Dimensions

A useful model:

```text
Execution state:
    STARTING / RUNNING / STOPPING / TERMINAL

Health:
    HEALTHY / DEGRADED / UNHEALTHY / UNKNOWN

Authority:
    CURRENT / STALE / UNKNOWN

Connectivity:
    CONNECTED / DISCONNECTED
```

This avoids combinatorial explosion.

# 28. Heartbeats

Agents can periodically report:

```text
heartbeat {
    agent_id
    incarnation
    timestamp
    load
    active_executions
}
```

# 29. Heartbeat Semantics

A heartbeat proves:

> The Agent communicated at this time.

It does not necessarily prove:

> Every execution on the Agent is healthy.

# 30. Execution Heartbeat

For long-running Work, separate execution liveness may be useful:

```text
execution_heartbeat {
    execution_id
    attempt_id
    progress
    timestamp
}
```

# 31. Heartbeat Loss

If heartbeats stop:

```text
last_seen = T
```

the control plane may classify:

```text
SUSPECT
```

before:

```text
UNRESPONSIVE
```

# 32. Failure Detector

A failure detector should be explicit about uncertainty.

Bad:

```text
heartbeat missed
→ process dead
```

Better:

```text
heartbeat missed
→ liveness uncertain
→ reconciliation required
```

# 33. Network Partition

Agent may be:

```text
alive but unreachable
```

Therefore:

```text
CONTROL_PLANE_DISCONNECTED
```

does not prove:

```text
PROCESS_TERMINATED
```

# 34. Split-Brain Protection

During partition, stale Agents must not continue accepting commands from stale authority.

This is where:

```text
generation
epoch
fencing
```

become execution safety mechanisms.

# 35. Command Lease

Long-running commands may carry:

```text
command_lease
```

The Agent executes only while the lease remains valid.

# 36. Lease Renewal

```text
LEASE
 ↓
RENEW
 ↓
RENEW
 ↓
...
```

If renewal stops:

```text
LEASE_EXPIRED
```

Agent can transition to a safe state according to policy.

# 37. Lease Expiration Is Not Always Kill

For some Work:

```text
lease expiration → terminate
```

For others:

```text
lease expiration → freeze
```

or:

```text
lease expiration → isolate
```

Policy must specify the behavior.

# 38. Cancellation Contract

Cancellation should be a protocol operation:

```text
CANCEL {
    execution_id
    attempt_id
    generation
    reason
}
```

# 39. Cancellation Is Intent

Receiving:

```text
CANCEL
```

does not mean:

```text
process already stopped
```

It means:

> The Agent has been instructed to attempt cancellation.

# 40. Cancellation Lifecycle

```text
RUNNING
 ↓
CANCEL_REQUESTED
 ↓
CANCELLING
 ↓
TERMINATED
```

# 41. Cancellation Timeout

If graceful cancellation does not complete:

```text
CANCELLING
    ↓ timeout
FORCE_TERMINATION
```

if policy permits.

# 42. Cancellation Evidence

Agent should record:

```text
requested_at
acknowledged_at
terminated_at
method
exit_status
```

This enables later diagnosis.

# 43. Shutdown

Agent shutdown should be stateful:

```text
READY
 ↓
DRAINING
 ↓
STOPPING
 ↓
OFFLINE
```

# 44. Draining

During:

```text
DRAINING
```

the Agent:

```text
accepts no new Work
```

but may allow existing executions to complete.

# 45. Forced Drain

If shutdown deadline expires:

```text
remaining executions
```

are reconciled according to their interruption policy.

# 46. Result Model

An execution result should be structured:

```text
Result {
    status
    exit_code
    signal
    outputs
    artifacts
    metrics
    error
}
```

# 47. Result Status

Avoid mapping everything into success/failure.

Useful distinctions:

```text
SUCCEEDED
FAILED
CANCELLED
TIMED_OUT
PREEMPTED
INTERRUPTED
UNKNOWN
```

# 48. UNKNOWN Result

If the Agent disappears before result publication:

```text
result = UNKNOWN
```

until reconciliation obtains sufficient evidence.

# 49. Exit Code

An exit code is evidence about process termination.

It does not necessarily determine logical Work status.

For example:

```text
exit_code = 0
```

may still fail a higher-level validation step.

# 50. Result Validation

Control plane may perform:

```text
process result
 ↓
artifact verification
 ↓
schema validation
 ↓
policy validation
```

before committing logical success.

# 51. Artifact Model

Artifacts should have identities:

```text
Artifact {
    artifact_id
    type
    uri
    size
    digest
    producer
}
```

# 52. Digest

A result claiming:

```text
artifact = output.tar
```

is insufficient.

Prefer:

```text
sha256 = ...
```

or another approved digest.

# 53. Artifact Publication

Artifact lifecycle:

```text
CREATED
 ↓
UPLOADING
 ↓
STORED
 ↓
VERIFIED
 ↓
PUBLISHED
```

# 54. Result Before Artifact

Do not commit:

```text
SUCCESS
```

if required artifacts have not been durably published.

Otherwise logical success can point to missing evidence.

# 55. Output Streams

Stdout/stderr are data streams, not authoritative lifecycle state.

```text
stdout
stderr
logs
metrics
events
```

may all be emitted independently.

# 56. Stream Ordering

Within one stream:

```text
sequence 1
sequence 2
sequence 3
```

should be ordered where the protocol promises ordering.

Do not assume stdout and stderr have a globally meaningful combined ordering unless explicitly captured.

# 57. Log Loss

If log transport fails:

```text
execution may still succeed
```

unless logs themselves are a required output.

This distinction is important.

# 58. Backpressure on Output

A noisy process must not crash the Agent because output buffers fill.

Agent needs:

```text
bounded buffering
streaming
spooling
sampling
truncation policy
```

# 59. Output Limits

Execution policy may define:

```text
max_stdout_bytes
max_stderr_bytes
max_log_rate
```

Exceeding the limit should produce a structured event:

```text
OUTPUT_LIMIT_EXCEEDED
```

# 60. Artifact vs Log

Large binary output should generally be:

```text
artifact
```

rather than pushed through the event stream.

# 61. Checkpoint

Agent may periodically create:

```text
Checkpoint {
    execution_id
    attempt_id
    position
    state_digest
    artifacts
}
```

# 62. Checkpoint Authority

The Agent creates the checkpoint, but the control plane decides whether it is acceptable for recovery.

Thus:

```text
created ≠ committed recovery point
```

# 63. Checkpoint Commit

A checkpoint can transition:

```text
CREATED
 ↓
UPLOADED
 ↓
VERIFIED
 ↓
COMMITTED
```

# 64. Crash Recovery

Suppose Agent dies:

```text
execution A2
checkpoint C7
```

exists.

New Agent can:

```text
load C7
→ validate generation
→ create A3
→ resume
```

# 65. Attempt Replacement

A resumed attempt must not reuse the old attempt identity.

```text
A2 → failed/interrupted
A3 → resumed from checkpoint C7
```

This preserves history.

# 66. Side-Effect Safety

Checkpoint recovery is dangerous when Work has external side effects.

Example:

```text
charge credit card
```

followed by:

```text
checkpoint
```

A retry could charge twice.

Therefore external effects need:

```text
idempotency
transactionality
or explicit compensation
```

# 67. Execution Sandbox

Agent should execute Work within a defined isolation boundary.

Potential dimensions:

```text
filesystem
process tree
network
credentials
environment
CPU
memory
devices
```

# 68. Credential Scope

Work should receive only credentials required for its execution.

Prefer:

```text
short-lived scoped credential
```

over:

```text
Agent-wide permanent secret
```

# 69. Environment Reproducibility

Execution should record relevant environment identity:

```text
runtime version
image/container identity
OS
architecture
toolchain
configuration digest
```

# 70. Reproducibility

A result is more useful when NROS can answer:

> Under what execution environment was this produced?

# 71. Execution Manifest

Before starting, create:

```text
ExecutionManifest {
    execution_id
    attempt_id
    generation
    resources
    environment
    command
    inputs
    policy_version
}
```

# 72. Manifest Immutability

Once execution begins, the manifest should be immutable.

If something changes:

```text
new attempt
```

or:

```text
explicit revision
```

must be created.

# 73. Agent-Side State Machine

```text
RECEIVED
   ↓
VALIDATED
   ↓
ACCEPTED
   ↓
STARTING
   ↓
RUNNING
   ├──→ CANCELLING
   ├──→ PREEMPTING
   └──→ FAILED
   ↓
COMPLETING
   ↓
RESULT_READY
   ↓
RESULT_COMMITTED
```

# 74. Important Distinction

The Agent can reach:

```text
RESULT_READY
```

while the control plane remains:

```text
RESULT_UNCOMMITTED
```

until durable validation occurs.

# 75. Completion Boundary

A strong completion protocol is:

```text
process terminated
      ↓
result constructed
      ↓
artifacts published
      ↓
result verified
      ↓
result committed
      ↓
execution COMPLETE
```

# 76. Exactly-Once Execution

The Agent should **not** promise universal exactly-once execution.

Distributed failures make this extremely difficult.

Instead promise:

```text
at-most-once command acceptance
+
idempotent retry
+
attempt identity
+
reconciliation
```

where appropriate.

# 77. At-Least-Once Delivery

Command delivery may naturally be:

```text
at-least-once
```

Therefore duplicate handling is mandatory.

# 78. Exactly-Once Effects

Where external systems support idempotency keys, NROS can approach:

```text
exactly-once logical effect
```

without claiming exactly-once network execution.

# 79. Agent Failure

When Agent crashes:

```text
Agent A
   X
```

the control plane should classify active attempts:

```text
UNKNOWN
```

then reconcile.

# 80. Reconciliation Sources

Possible evidence:

```text
Agent restart journal
process supervisor
resource manager
artifact store
external system
checkpoint store
```

# 81. Agent Journal

A local durable journal can record:

```text
command accepted
process started
checkpoint committed
process exited
result published
```

This greatly improves crash recovery.

# 82. Local Journal Durability

However, local disk durability is itself a capability.

If the Agent runs on ephemeral infrastructure:

```text
local journal
```

may disappear with the node.

Critical recovery evidence should therefore be replicated or externally persisted where required.

# 83. Agent Recovery Protocol

```text
Agent starts
    ↓
load local journal
    ↓
register new incarnation
    ↓
receive active executions
    ↓
reconcile previous attempts
    ↓
resume / terminate / report UNKNOWN
```

# 84. New Incarnation Must Not Inherit Authority Blindly

Old incarnation:

```text

```

new incarnation:

```text

```

The new Agent must prove that it is authorized to recover the execution.

# 85. Recovery Claim

Conceptually:

```text
Claim {
    execution_id
    previous_attempt
    new_attempt
    generation
    agent_incarnation
}
```

Control plane validates the claim.

# 86. Fenced Recovery

If another Agent already owns the execution:

```text
CLAIM_REJECTED
```

This prevents duplicate execution.

# 87. Agent Resource Release

When execution ends:

```text
process terminated
 ↓
resource release
 ↓
reservation released
```

The release itself must be idempotent.

# 88. Double Release

Two recovery paths may both attempt:

```text
release reservation R
```

The resource manager must safely treat repeated release as:

```text
already released
```

rather than corrupting accounting.

# 89. Agent Execution Contract

The Agent should guarantee:

```text
1. Commands have stable identity.

2. Stale generations are rejected.

3. Duplicate commands are detected.

4. Acceptance is distinct from execution.

5. Attempt identity is unique.

6. Process state is not blindly equated with logical state.

7. Cancellation is an explicit lifecycle.

8. Preemption is capability-dependent.

9. Heartbeats prove communication, not necessarily health.

10. Network loss creates uncertainty rather than automatic failure.

11. Results are structured.

12. Artifacts have verifiable identity.

13. Required artifacts are durable before logical completion.

14. Checkpoints have explicit lifecycle.

15. Checkpoints do not automatically imply safe recovery.

16. External side effects require idempotency or compensation.

17. Output streams are bounded and backpressured.

18. Execution environments are identifiable.

19. Execution manifests become immutable after start.

20. Agent incarnations are unique.

21. Recovery uses explicit authority.

22. Resource release is idempotent.

23. Agent shutdown supports draining.

24. Forceful termination is explicitly observable.

25. Unknown outcomes remain UNKNOWN until sufficient evidence exists.

26. The Agent never silently upgrades an observation into authoritative global state.
```

# 90. Complete Control/Data Plane

The architecture now becomes:

```text
                 CONTROL PLANE
────────────────────────────────────────────

 Work
   ↓
 Admission
   ↓
 Scheduler
   ↓
 Resource Reservation
   ↓
 Command
   ↓
 Fencing / Authority
   │
   │
   ▼
────────────────────────────────────────────
                 DATA PLANE

              ┌───────────┐
              │   AGENT   │
              └─────┬─────┘
                    │
              ┌─────┴─────┐
              │ Execution │
              └─────┬─────┘
                    │
          ┌─────────┼─────────┐
          ↓         ↓         ↓
       Process    Output   Checkpoint
          │         │         │
          └─────────┼─────────┘
                    ↓
                 Result
                    ↓
────────────────────────────────────────────
                 CONTROL PLANE
                    ↓
               Validation
                    ↓
                Commit
                    ↓
              Reconciliation
```

# 91. The Critical Boundary

The most important invariant introduced here is:

```text
COMMAND ACCEPTED
        ≠
PROCESS STARTED
        ≠
PROCESS HEALTHY
        ≠
PROCESS TERMINATED
        ≠
RESULT READY
        ≠
RESULT COMMITTED
        ≠
WORK COMPLETED
```

Each boundary requires its own evidence.

# 92. NROS Execution Truth Model

The resulting truth hierarchy is:

```text
Intent
   ↓
Authorized Command
   ↓
Agent Acceptance
   ↓
Execution Evidence
   ↓
Result Evidence
   ↓
Artifact Verification
   ↓
Durable Commit
   ↓
Logical Completion
```

This prevents the common distributed-runtime mistake of treating a single process status as the entire truth model.

# Part CV — Protocol & Wire-Level Semantics

The next layer should formalize the actual **NROS protocol**:

```text
message envelope
→ identity
→ authentication
→ version negotiation
→ capability negotiation
→ request/response
→ streaming
→ acknowledgements
→ errors
→ retries
→ idempotency
→ ordering
→ flow control
→ compatibility
→ protocol evolution
```

The central question becomes:

> **What exact guarantees does an NROS message carry across an unreliable network, and how can two different protocol versions communicate without silently weakening the system's invariants?**

# NROS — Part CV: Protocol & Wire-Level Semantics

We now formalize the **wire protocol** beneath the scheduler and Agent execution model.

The protocol must preserve the guarantees established above even when:

- messages are duplicated,
- messages arrive out of order,
- connections disappear,
- peers restart,
- protocol versions differ,
- requests time out,
- acknowledgements are lost,
- streams are interrupted,
- and multiple controllers compete for authority.

The protocol therefore cannot be merely a serialization format.

It is a **distributed state-transition protocol**.

## 1. Protocol Layers

Separate the protocol into layers:

```text
┌───────────────────────────────┐
│ Application Semantics         │
│ Work / Execution / Result     │
├───────────────────────────────┤
│ Control Semantics             │
│ Scheduling / Authority        │
├───────────────────────────────┤
│ Message Semantics             │
│ IDs / Ordering / Ack / Retry   │
├───────────────────────────────┤
│ Security                      │
│ Identity / Auth / Integrity   │
├───────────────────────────────┤
│ Transport                     │
│ TCP / QUIC / Unix socket / …  │
└───────────────────────────────┘
```

Do not allow transport assumptions to leak into application semantics.

# 2. Transport Independence

NROS should conceptually support multiple transports:

```text
Unix socket
TCP
QUIC
local IPC
embedded transport
```

The protocol should define:

```text

```

independently from:

```text

```

# 3. Message Envelope

Every protocol message should have a common envelope.

Conceptually:

```text
Envelope {
    protocol_version
    message_type
    message_id
    correlation_id
    sender
    receiver
    epoch
    sequence
    timestamp
    flags
    payload
}
```

# 4. Message Identity

Every message requires a stable:

```text
message_id
```

It identifies the specific message instance.

This allows duplicate detection.

# 5. Correlation Identity

For request/response protocols:

```text
request.message_id = M42

response.correlation_id = M42
```

The response does not need to reuse the request's identity.

# 6. Conversation Identity

Long-running interactions may also require:

```text
conversation_id
```

Example:

```text
handshake
→ capability negotiation
→ authentication
→ session establishment
```

All messages belong to one protocol session.

# 7. Sender Identity

Sender identity should be explicit:

```text
sender {
    principal_id
    incarnation
}
```

An Agent restart therefore changes:

```text
incarnation
```

without necessarily changing:

```text
principal_id
```

# 8. Receiver Identity

Messages may specify:

```text
receiver
```

when addressed to a specific peer.

Broadcast or discovery messages can omit a concrete receiver if the protocol permits it.

# 9. Epoch

Control-plane messages should carry authority context:

```text
epoch
```

This is critical for fencing stale controllers.

# 10. Sequence Numbers

A connection or logical stream can carry:

```text
sequence = 1
sequence = 2
sequence = 3
```

Sequence numbers support:

```text
gap detection
duplicate detection
replay detection
```

# 11. Sequence Scope

Never leave sequence scope ambiguous.

Specify whether sequence numbers are scoped to:

```text
connection
session
sender/receiver pair
stream
conversation
```

A strong default is:

```text

```

combined with a stable stream identity.

# 12. Ordering

The protocol should explicitly define ordering guarantees.

Possible:

```text
ORDERED
UNORDERED
PARTIALLY_ORDERED
```

Do not assume transport ordering automatically gives application ordering after reconnects.

# 13. Reconnection

Suppose:

```text
messages 100–105
```

were sent.

Connection dies.

On reconnect the peers need to determine:

```text
which messages were processed?
which were merely transmitted?
which must be resent?
```

# 14. Acknowledgement Semantics

An ACK must have a precise meaning.

Possible meanings:

```text
RECEIVED
VALIDATED
ACCEPTED
COMMITTED
```

These are fundamentally different.

# 15. Transport ACK

A transport-level ACK means:

> Bytes reached the peer's transport stack.

It does **not** mean:

> NROS accepted the command.

# 16. Protocol ACK

An NROS ACK can mean:

```text
message parsed and accepted by protocol layer
```

Again, this is not execution.

# 17. Command ACK

For a command:

```text
COMMAND_ACCEPTED
```

means:

> The Agent has accepted responsibility for the command.

It does not mean:

```text
execution succeeded
```

# 18. Commit ACK

A stronger acknowledgement:

```text
COMMITTED
```

means the receiver has durably recorded the relevant state.

This distinction is essential for retry behavior.

# 19. ACK Matrix

| ACK | Meaning |
|---|---|
| RECEIVED | Message arrived |
| PARSED | Envelope understood |
| VALIDATED | Semantically valid |
| ACCEPTED | Receiver accepted responsibility |
| STARTED | Execution began |
| COMMITTED | State durably recorded |
| COMPLETED | Operation reached terminal state |

Never collapse these into one generic `OK`.

# 20. Request Lifecycle

A request may therefore follow:

```text
SENT
 ↓
RECEIVED
 ↓
PARSED
 ↓
VALIDATED
 ↓
ACCEPTED
 ↓
PROCESSING
 ↓
COMMITTED
 ↓
COMPLETED
```

# 21. Timeout Semantics

Timeout means:

> The sender did not receive expected evidence within the configured interval.

It does **not** mean:

> The receiver definitely failed.

# 22. Timeout Example

Controller sends:

```text
START
```

Agent starts successfully.

ACK is lost.

Controller times out.

The actual state may be:

```text
RUNNING
```

Therefore retry must be idempotent.

# 23. Retry Protocol

Retry should use:

```text
same logical operation
same idempotency identity
new transport attempt
```

not:

```text
brand-new command identity
```

unless a new logical attempt is actually intended.

# 24. Transport Retry vs Execution Retry

These must remain separate.

```text
transport retry
```

means:

> resend the message.

Whereas:

```text
execution retry
```

means:

> create another execution attempt.

# 25. Example

```text
START(command=42)
```

times out.

Retry:

```text
START(command=42)
```

should normally address the same execution.

But if the execution itself failed:

```text
attempt A1 → FAILED
```

then:

```text
attempt A2
```

is a new logical execution attempt.

# 26. Idempotency Key

Commands that can cause side effects should carry:

```text
idempotency_key
```

The receiver stores the outcome associated with that key.

# 27. Idempotency Record

Conceptually:

```text
IdempotencyRecord {
    key
    operation
    request_digest
    state
    result
    created_at
    expires_at
}
```

# 28. Idempotency Conflict

If the same key arrives with different content:

```text
key = K42
payload = A
```

then later:

```text
key = K42
payload = B
```

the Agent must reject it.

Example:

```text
IDEMPOTENCY_CONFLICT
```

# 29. Message Digest

For important commands, include:

```text
payload_digest
```

This allows the receiver to detect accidental or malicious mutation.

# 30. Authentication

NROS authentication should establish:

```text
who sent this?
```

Integrity establishes:

```text
was it modified?
```

Authorization establishes:

```text
may this principal perform this operation?
```

These are distinct.

# 31. Authentication ≠ Authorization

An authenticated Agent can still be forbidden from:

```text
executing privileged Work
```

Authentication merely establishes identity.

# 32. Message Integrity

Messages should be protected against unauthorized modification through the selected secure transport or message-level cryptographic mechanism.

The protocol should not silently assume:

```text
trusted network = trusted message
```

# 33. Replay Protection

An attacker or stale peer could resend:

```text
CANCEL(execution=17)
```

from an earlier session.

Replay protection requires some combination of:

```text
epoch
incarnation
sequence
nonce
expiry
idempotency
```

# 34. Freshness

Security-sensitive commands should have a defined freshness rule:

```text
issued_at
expires_at
epoch
```

A command outside its validity window should be rejected.

# 35. Protocol Version

Every connection should negotiate:

```text
protocol_version
```

# 36. Version Negotiation

Example:

```text
Agent supports:
v1, v2, v3

Controller supports:
v2, v3
```

Negotiated:

```text
v3
```

# 37. Version Compatibility

Compatibility should distinguish:

```text
wire compatibility
semantic compatibility
feature compatibility
```

Two peers can parse the same message while disagreeing about its meaning.

That is dangerous.

# 38. Feature Negotiation

Features can be negotiated separately:

```text
checkpoint-v2
streaming-results
artifact-digests
lease-fencing
```

# 39. Capability Negotiation

A peer may advertise:

```text
features {
    checkpoint
    migration
    compression
}
```

The protocol should never invoke an optional feature without establishing support.

# 40. Unknown Fields

For forward compatibility, implementations should generally tolerate unknown optional fields.

But unknown **required semantics** must result in an explicit incompatibility response.

# 41. Unknown Message Type

If an Agent receives:

```text
message_type = FUTURE_OPERATION
```

it should return:

```text
UNSUPPORTED_MESSAGE_TYPE
```

rather than interpreting it incorrectly.

# 42. Error Model

Errors should be structured.

```text
Error {
    code
    category
    retryable
    message
    details
}
```

# 43. Error Categories

Useful categories:

```text
PROTOCOL
AUTHENTICATION
AUTHORIZATION
VALIDATION
CONFLICT
RESOURCE
TIMEOUT
UNAVAILABLE
STALE
INTERNAL
```

# 44. Retryability

Every transient error should expose whether retry is appropriate:

```text
retryable = true
```

But retryability does not mean:

> retry immediately.

Backoff remains necessary.

# 45. Error Codes Must Be Stable

Human-readable:

```text
"resource busy"
```

is not a reliable API contract.

Use:

```text
RESOURCE_UNAVAILABLE
```

with structured details.

# 46. Error Details

Example:

```text
{
    code: RESOURCE_UNAVAILABLE,
    resource: "gpu",
    requested: 1,
    available: 0,
    retryable: true
}
```

# 47. Streaming

Long-running operations should not require one giant response.

Use:

```text
REQUEST
 ↓
EVENT*
 ↓
FINAL_RESPONSE
```

# 48. Stream Identity

Each stream should have:

```text
stream_id
```

and sequence numbers.

# 49. Stream Events

Possible events:

```text
STARTED
PROGRESS
OUTPUT
CHECKPOINT
WARNING
HEARTBEAT
STATE_CHANGED
COMPLETED
```

# 50. Final Event

The stream must define exactly one terminal event:

```text
COMPLETED
FAILED
CANCELLED
```

or equivalent.

# 51. Stream Resume

If connection disappears after event 72:

```text
last_received = 72
```

the client can request:

```text
resume_from = 73
```

if the server retains sufficient history.

# 52. Event Retention

Stream resumability requires a retention policy:

```text
event buffer
or
durable event log
```

Without retention, resume is impossible.

# 53. Flow Control

An Agent must not be overwhelmed by a controller.

Likewise:

```text
Agent output
```

must not overwhelm the controller.

Flow control is therefore bidirectional.

# 54. Credits

A simple model:

```text
receiver grants N credits
sender may transmit N units
```

Credits can be defined in:

```text
messages
events
```

# 55. Backpressure

When credits reach zero:

```text
sender pauses
```

rather than buffering infinitely.

# 56. Resource Limits

Protocol implementations should enforce:

```text
max_message_size
max_header_size
max_streams
max_inflight_requests
max_event_buffer
```

# 57. Large Payloads

Do not place massive artifacts directly into control messages.

Instead:

```text
control message
   ↓
artifact reference
   ↓
artifact transport/store
```

# 58. Payload Reference

Example:

```text
ArtifactRef {
    artifact_id
    uri
    digest
    size
}
```

# 59. Compression

Compression should be negotiated.

Potential:

```text
none
zstd
gzip
```

depending on implementation goals.

Do not compress blindly because:

```text
already-compressed artifacts
```

may become less efficient.

# 60. Fragmentation

If the transport cannot carry the entire message, fragmentation must preserve:

```text
message_id
fragment_index
fragment_count
payload_digest
```

Reassembly must have bounded memory.

# 61. Connection Lifecycle

A connection can use:

```text
CONNECTING
 ↓
NEGOTIATING
 ↓
AUTHENTICATING
 ↓
ESTABLISHED
 ↓
DRAINING
 ↓
CLOSED
```

# 62. Handshake

Handshake should establish:

```text
protocol version
features
identity
authentication
limits
session parameters
```

# 63. Session Identity

After handshake:

```text
session_id
```

should identify the logical connection session.

A reconnect creates a new session unless the protocol explicitly supports session resumption.

# 64. Session Resumption

If supported:

```text
old session
 ↓
disconnect
 ↓
resume token
 ↓
new transport
 ↓
same logical session
```

But resumption must include replay protection.

# 65. Connection Loss

Connection loss should generate a local event:

```text
PEER_DISCONNECTED
```

It should not directly mutate remote execution state.

# 66. Reconciliation After Reconnect

After reconnection:

```text
handshake
 ↓
exchange state watermarks
 ↓
detect missing events
 ↓
replay/reconcile
```

# 67. Watermarks

Peers can exchange:

```text
last_received_sequence
last_committed_sequence
last_event_sequence
```

This is much stronger than saying:

```text
"we're synchronized"
```

# 68. State Digest

For large state sets, peers may exchange:

```text
state_digest
```

to detect divergence.

# 69. Divergence

If:

```text
local_digest != remote_digest
```

do not automatically overwrite one side.

Instead:

```text
DIVERGENCE_DETECTED
→ reconciliation
```

# 70. Protocol Transactions

Some operations may require:

```text
BEGIN
→ operations
→ COMMIT
```

But transactions should remain scoped.

Do not create a giant transaction spanning:

```text
Agent
artifact store
external system
```

unless unavoidable.

# 71. Command + Event Model

A strong architecture is:

```text
COMMAND
   ↓
AGENT
   ↓
EVENTS
   ↓
STATE REDUCER
```

Commands express intent.

Events provide evidence.

# 72. Commands Are Not Events

Command:

```text
START_EXECUTION
```

means:

> Please perform this action.

Event:

```text
EXECUTION_STARTED
```

means:

> The system observed that execution started.

Do not conflate them.

# 73. Event Immutability

Once emitted:

```text
EXECUTION_STARTED
```

should not be edited into:

```text
EXECUTION_FAILED
```

Instead emit another event.

# 74. Event Ordering

Events should include enough information to reconstruct causality:

```text
event_id
sequence
causation_id
correlation_id
timestamp
```

# 75. Causation

Example:

```text
START_COMMAND
```

causes:

```text
EXECUTION_STARTED
```

The event should reference the command:

```text
causation_id = START_COMMAND
```

# 76. Correlation

All events for one execution can share:

```text
correlation_id = execution_id
```

This makes tracing substantially easier.

# 77. Trace Context

Protocol messages may carry:

```text
trace_id
span_id
parent_span_id
```

This is observability metadata, not execution authority.

# 78. Timestamp Semantics

Define timestamp types:

```text
wall_clock_time
monotonic_elapsed_time
logical_sequence
```

Never use wall-clock timestamps as the sole mechanism for ordering.

# 79. Logical Ordering

When correctness matters:

```text
sequence
generation
causation
```

should establish ordering.

Clock time is supplementary evidence.

# 80. Clock Skew

Two Agents may report:

```text
A = 12:00:00
B = 11:59:57
```

Do not infer causal ordering solely from timestamps.

# 81. Protocol State Machine

A robust connection:

```text
DISCONNECTED
     ↓
CONNECTING
     ↓
HANDSHAKE
     ↓
AUTHENTICATION
     ↓
NEGOTIATION
     ↓
ESTABLISHED
     ├──→ DRAINING
     ├──→ RECONNECTING
     └──→ FAILED
```

# 82. Message State Machine

```text
CREATED
 ↓
QUEUED
 ↓
SENT
 ↓
ACKNOWLEDGED
 ↓
COMMITTED
```

Failure path:

```text
SENT
 ↓
TIMEOUT
 ↓
RETRYING
```

# 83. Protocol Invariants

```text
1. Every message has stable identity.

2. Requests and responses have explicit correlation.

3. Sequence scope is defined.

4. Ordering guarantees are explicit.

5. ACK semantics are explicit.

6. Transport ACK is not command acceptance.

7. Command acceptance is not execution success.

8. Timeout does not imply remote failure.

9. Transport retry is distinct from execution retry.

10. Idempotency is explicit for side-effecting operations.

11. Idempotency conflicts are detected.

12. Authentication and authorization remain distinct.

13. Replay protection is mandatory for privileged commands.

14. Epoch/generation fences stale authority.

15. Protocol versions are negotiated.

16. Optional capabilities are negotiated.

17. Unknown optional fields can be tolerated where safe.

18. Unsupported mandatory semantics fail explicitly.

19. Errors use stable machine-readable codes.

20. Retryability is explicit.

21. Streaming has sequence and termination semantics.

22. Flow control prevents unbounded buffering.

23. Large payloads use references where appropriate.

24. Connection loss does not automatically mutate remote state.

25. Reconnection performs reconciliation.

26. Event history is immutable.

27. Commands express intent.

28. Events express observed state transitions.

29. Causation and correlation are distinguishable.

30. Logical ordering does not depend solely on wall-clock time.

31. State divergence is detected and reconciled rather than silently overwritten.

32. Protocol implementation limits are explicit.

33. Security failures are distinguishable from transient transport failures.

34. A peer incarnation is distinct from its stable principal identity.

35. Session identity changes across reconnect unless resumption is explicitly negotiated.
```

# 84. Canonical NROS Message Shape

The conceptual wire representation becomes:

```text
Envelope
{
    version,
    type,

    message_id,
    correlation_id,
    causation_id,

    sender {
        principal_id,
        incarnation
    },

    receiver,

    session_id,
    stream_id,

    epoch,
    sequence,

    issued_at,
    expires_at,

    flags,

    payload,

    payload_digest,

    authentication
}
```

Not every transport needs every field physically encoded in exactly this form, but the **semantic model** should remain stable.

# 85. Canonical Command

```text
Command
{
    command_id,
    work_id,
    execution_id,
    attempt_id,

    operation,

    generation,
    idempotency_key,

    deadline,

    resource_context,

    payload
}
```

# 86. Canonical Event

```text
Event
{
    event_id,

    event_type,

    execution_id,
    attempt_id,

    correlation_id,
    causation_id,

    sequence,
    generation,

    occurred_at,

    payload
}
```

# 87. Canonical Error

```text
Error
{
    code,
    category,
    retryable,

    correlation_id,

    message,

    details
}
```

# 88. Protocol Design Principle

The protocol should make illegal states difficult to represent.

For example, avoid an API where callers can simply send:

```text
status = "completed"
```

Instead completion should require the protocol flow that establishes:

```text
execution evidence
+
result
+
artifact validation
+
commit authority
```

# 89. The Most Important Separation

NROS now has three different semantic directions:

```text
COMMAND
    ↓
intent

EVENT
    ↓
evidence

STATE
    ↓
derived authoritative interpretation
```

This separation gives the runtime a much stronger foundation for:

```text
reconciliation
auditing
testing
recovery
debugging
```

# Part CVI — Durable State & Event Log

The next layer should now formalize the **persistence substrate**:

```text
event log
→ snapshots
→ state reducers
→ transactions
→ commit points
→ idempotency records
→ leases
→ reservations
→ recovery journal
→ compaction
→ retention
→ corruption detection
→ crash consistency
```

The central question becomes:

> **What exact state is authoritative, where is it persisted, and how can NROS reconstruct the same logical state after a complete control-plane crash?**

# NROS — Part CVI: Durable State, Event Log & Crash-Recovery Semantics

We now reach the persistence boundary.

The scheduler, Agent, and protocol layers all generate state transitions, but none of those transitions become authoritative merely because they happened in memory.

NROS therefore needs a durable state model capable of surviving:

- process crashes,
- machine reboots,
- controller replacement,
- network partitions,
- duplicate commands,
- partial writes,
- replay,
- recovery,
- and state-store corruption.

The fundamental principle is:

> **If NROS cannot reconstruct a state transition after a crash, that transition must not be treated as durable truth.**

# 1. Persistence Is Part of the Runtime Semantics

Persistence is not merely an implementation detail.

It determines whether NROS can answer:

```text
What Work existed?
What Execution existed?
Who owned it?
Which command was accepted?
Which resources were reserved?
Which result was committed?
Which authority epoch was current?
```

after a crash.

Therefore:

```text
runtime semantics
    ↓
durability semantics
```

must be designed together.

# 2. Authoritative State

NROS should identify one authoritative state representation for each domain.

For example:

```text
Work state
Execution state
Agent ownership
Resource reservation
Scheduler epoch
Idempotency record
Checkpoint commitment
Artifact commitment
```

Each must have a clearly defined authority.

# 3. Memory Is Not Authority

A controller may have:

```text
READY
RUNNING
RESERVED
```

in memory.

After process termination:

```text
memory = gone
```

Therefore those values are not durable unless persisted.

# 4. Durable State Model

Conceptually:

```text
             ┌──────────────┐
             │   COMMAND    │
             └──────┬───────┘
                    ↓
             ┌──────────────┐
             │ EVENT / TX   │
             └──────┬───────┘
                    ↓
             ┌──────────────┐
             │ DURABLE LOG  │
             └──────┬───────┘
                    ↓
             ┌──────────────┐
             │ STATE REDUCER│
             └──────┬───────┘
                    ↓
             ┌──────────────┐
             │ CURRENT STATE│
             └──────────────┘
```

# 5. Event Log

The event log records durable transitions.

Example:

```text
WORK_CREATED
WORK_ADMITTED
WORK_READY
EXECUTION_SELECTED
RESOURCE_RESERVED
COMMAND_ACCEPTED
EXECUTION_STARTED
EXECUTION_COMPLETED
RESULT_COMMITTED
WORK_COMPLETED
```

# 6. Event Is Immutable

Once committed:

```text
EXECUTION_STARTED
```

must never be edited into:

```text
EXECUTION_FAILED
```

Instead:

```text
EXECUTION_STARTED
EXECUTION_FAILED
```

are two separate events.

# 7. Event Identity

Every event needs:

```text
event_id
```

and preferably:

```text
aggregate_id
sequence
generation
causation_id
correlation_id
```

# 8. Aggregate Sequence

For an Execution:

```text
Execution E17

sequence 1 → CREATED
sequence 2 → STARTING
sequence 3 → RUNNING
sequence 4 → COMPLETED
```

The sequence establishes logical ordering within the aggregate.

# 9. Global Sequence

A durable event store may additionally provide:

```text
global_sequence
```

which orders events across aggregates.

This is useful for:

```text
replay
subscriptions
auditing
incremental snapshots
```

but should not replace aggregate-level invariants.

# 10. Aggregate

An aggregate is a consistency boundary.

Possible NROS aggregates:

```text
Work
Execution
Agent
Reservation
Lease
```

Avoid placing the entire runtime into one giant aggregate.

# 11. Why Aggregate Boundaries Matter

If every transition required one global transaction:

```text
Work
+
Scheduler
+
Agent
+
Resource manager
+
Artifact store
```

the system becomes tightly coupled.

Prefer smaller authoritative boundaries plus explicit reconciliation.

# 12. State Reducer

A reducer transforms:

```text
state + event
```

into:

```text
new_state
```

Conceptually:

```text
state' = reduce(state, event)
```

# 13. Deterministic Reducer

Given identical:

```text
initial state
event sequence
```

the reducer should produce identical state.

This gives NROS:

```text
replayability
testability
debuggability
```

# 14. Reducer Must Not Perform Side Effects

A reducer should not:

```text
start process
send network command
allocate resource
delete file
```

It should derive state.

Side effects belong to explicit effect handlers.

# 15. Command Handler

A command handler can:

```text
validate command
check current state
authorize operation
produce events
```

Example:

```text
START_EXECUTION
    ↓
validate
    ↓
authorize
    ↓
produce EXECUTION_START_REQUESTED
```

# 16. Event Handler

The event handler/reducer then applies:

```text
EXECUTION_START_REQUESTED
```

to durable state.

# 17. Effects

A separate effect subsystem can observe committed events:

```text
EXECUTION_START_REQUESTED
        ↓
dispatch START command to Agent
```

This is safer than sending the command before the durable intent exists.

# 18. Transactional Outbox

This leads naturally to an outbox pattern.

Within one durable transaction:

```text
state change
+
outbox message
```

are committed together.

Then:

```text
outbox
 ↓
transport
 ↓
Agent
```

# 19. Why Outbox Matters

Without an outbox:

```text
1. update state
2. send command
```

may crash between steps.

Or:

```text
1. send command
2. update state
```

may crash in the opposite order.

The outbox makes the intended side effect durable.

# 20. Outbox Lifecycle

```text
PENDING
   ↓
DISPATCHING
   ↓
SENT
   ↓
ACKNOWLEDGED
   ↓
COMPLETED
```

Failures may return to:

```text
RETRYABLE
```

# 21. Outbox Is Not Execution Truth

An outbox record saying:

```text
START command sent
```

does not prove:

```text
Execution started
```

The Agent must provide evidence.

# 22. Inbox Pattern

On the Agent side:

```text
incoming command
      ↓
durable inbox
      ↓
deduplication
      ↓
execution
```

This creates:

```text
Outbox → transport → Inbox
```

with explicit idempotency.

# 23. Outbox + Inbox

The complete flow:

```text
Controller
   │
   ├── durable state
   └── outbox
          │
          ↓
       network
          │
          ↓
        inbox
          │
          ↓
        Agent
```

# 24. Exactly-Once Illusion

This architecture does not make network delivery literally exactly-once.

Instead it provides:

```text
durable intent
+
at-least-once delivery
+
deduplication
+
idempotent effects
+
reconciliation
```

which can produce exactly-once **logical outcomes** where the operation itself supports them.

# 25. Commit Point

NROS must define the exact point at which an event becomes durable.

Conceptually:

```text
append
 ↓
flush
 ↓
durable commit
```

The meaning of `commit` depends on the underlying storage system.

# 26. Durable Does Not Mean Visible

A write may be:

```text
persisted
```

but not yet:

```text
replicated
```

If NROS requires replication before acknowledging durability, that requirement must be explicit.

# 27. Durability Levels

Potential durability classes:

```text
LOCAL_DURABLE
REPLICATED
QUORUM_COMMITTED
EXTERNALY_COMMITTED
```

Do not call all of them simply:

```text
DURABLE
```

without defining semantics.

# 28. Replication

A replicated event log may look like:

```text
Leader
  ↓
Follower A
Follower B
```

An event becomes quorum-committed after sufficient replicas acknowledge it.

# 29. Leader Failure

If the leader crashes after:

```text
local append
```

but before:

```text
quorum commit
```

the event may or may not survive leader replacement.

The protocol must define this explicitly.

# 30. Commit Index

A replicated log commonly maintains:

```text
last_appended
last_replicated
last_committed
```

These are different values.

# 31. Applied Index

There is another boundary:

```text
last_applied
```

Meaning:

> The reducer has applied committed events to the materialized state.

Therefore:

```text
appended
≤ replicated
≤ committed
≤ applied
```

under the chosen consistency model.

# 32. Materialized State

The current state can be stored separately:

```text
Event Log
    ↓
Reducer
    ↓
Materialized State
```

This avoids replaying the entire history for every query.

# 33. Snapshot

Periodically:

```text
event 1
event 2
...
event 100000
```

can become:

```text
snapshot @ 100000
```

Then recovery only needs:

```text
snapshot
+
events 100001...
```

# 34. Snapshot Metadata

A snapshot should include:

```text
snapshot_id
log_position
state_schema_version
created_at
state_digest
```

# 35. Snapshot Must Be Self-Describing

Recovery needs to know:

```text
which schema?
which reducer version?
which log position?
```

A raw serialized state blob is insufficient.

# 36. Snapshot Consistency

A snapshot must represent a coherent state.

It cannot combine:

```text
Work state @ event 100
Execution state @ event 103
Reservation state @ event 97
```

unless the snapshot format explicitly supports cross-domain consistency.

# 37. Snapshot Boundary

A strong snapshot boundary is:

```text
global_commit_index = N
```

with all included materialized state derived from:

```text
events ≤ N
```

# 38. Snapshot Verification

Every snapshot should have integrity metadata:

```text
state_digest
```

and ideally:

```text
manifest_digest
```

for its constituent sections.

# 39. Snapshot Failure

If snapshot creation crashes halfway:

```text
partial snapshot
```

must never replace the previous valid snapshot.

Use:

```text
temporary
 ↓
complete
 ↓
verify
 ↓
atomic publish
```

# 40. Snapshot Publication

Conceptually:

```text
snapshot.tmp
      ↓
verify
      ↓
snapshot.complete
      ↓
manifest update
```

The manifest update becomes the publication point.

# 41. Recovery Algorithm

A clean recovery path:

```text
1. Load latest valid snapshot.
2. Verify snapshot integrity.
3. Read snapshot log position.
4. Load events after that position.
5. Verify event integrity.
6. Replay committed events.
7. Rebuild materialized state.
8. Restore timers/leases requiring reconstruction.
9. Reconcile external systems.
10. Mark control plane READY.
```

# 42. Recovery Must Precede Scheduling

A recovering scheduler must not immediately dispatch Work.

Correct:

```text
PROCESS START
   ↓
LOAD STATE
   ↓
REPLAY
   ↓
RECONCILE
   ↓
RESTORE AUTHORITY
   ↓
ENABLE SCHEDULING
```

# 43. Recovery Barrier

Introduce an explicit state:

```text
RECOVERING
```

and only transition to:

```text
SERVING
```

after required recovery barriers pass.

# 44. No State Transition Before Prerequisite

A strong invariant:

```text
NO OBSERVED PREREQUISITE
        →
NO STATE TRANSITION
```

For example:

```text
Agent unreachable
```

does not justify:

```text
Execution FAILED
```

without sufficient evidence.

# 45. Recovery Classification

After replay, active executions may be:

```text
KNOWN_RUNNING
KNOWN_COMPLETED
KNOWN_FAILED
UNKNOWN
REQUIRES_RECONCILIATION
```

# 46. Unknown State

Unknown should be a first-class state.

It prevents NROS from converting missing information into fabricated certainty.

# 47. Recovery Actions

For an UNKNOWN execution:

```text
query Agent
query supervisor
query resource manager
query artifact store
query checkpoint store
```

Only after evidence is collected should NROS transition it.

# 48. Lease Reconstruction

Leases require special handling after restart.

The system must determine:

```text
lease owner
lease epoch
expiration
renewal status
```

using authoritative storage.

# 49. Never Trust Memory Timers

A timer that existed before crash is gone.

Therefore:

```text
in-memory timeout
```

cannot be the source of truth.

Use durable:

```text
deadline
lease expiration
```

and recompute timers during recovery.

# 50. Time-Based State

For any time-dependent state, persist:

```text
deadline
```

rather than:

```text
remaining_duration
```

Example:

```text
deadline = 12:30:00
```

After restart at 12:25:

```text
remaining = 5 min
```

can be recomputed.

# 51. Clock Semantics

Wall-clock deadlines can be affected by clock changes.

For high-integrity timing, NROS should distinguish:

```text
wall-clock deadline
monotonic timer
logical expiration
```

# 52. Event Log Integrity

Events should be protected against:

```text
truncation
corruption
reordering
unexpected mutation
```

# 53. Hash Chaining

One possible integrity mechanism:

```text
event_1
  ↓ hash
event_2
  ↓ hash
event_3
```

Each event carries:

```text
previous_event_hash
```

This makes unexpected history mutation detectable.

# 54. Hash Chain Is Not Authentication

A hash chain proves internal consistency only if the root is trusted.

For stronger authenticity:

```text
signed checkpoint
trusted root
authenticated storage
```

may be required.

# 55. Corruption Detection

If replay encounters:

```text
invalid checksum
```

the runtime should enter:

```text
CORRUPT
```

or:

```text
RECOVERY_FAILED
```

rather than silently skipping the event.

# 56. Never Silently Skip Events

This would produce:

```text
false state
```

which is more dangerous than:

```text
system unavailable
```

for a correctness-oriented runtime.

# 57. Partial Write

Suppose a crash occurs during:

```text
event append
```

The storage layer must detect whether the final record is:

```text
complete
```

or:

```text
partial
```

and recover according to its journal/record format.

# 58. Atomic Record

Each event should have a framing structure allowing recovery to identify complete records.

Conceptually:

```text
length
header
payload
checksum
```

# 59. Append-Only Principle

The event log should ideally be:

```text
append-only
```

with corrections represented by new events.

This dramatically simplifies auditability.

# 60. Compaction

The event log can grow indefinitely.

Therefore:

```text
snapshot
+
compaction
```

may eventually remove old events from the hot store.

But only when retention policy allows it.

# 61. Compaction Safety

Never compact events required for:

```text
active recovery
audit retention
legal retention
reconciliation
debugging
```

unless another durable representation preserves the necessary semantics.

# 62. Snapshot + Compaction

Example:

```text
events 1–1,000,000
snapshot @ 900,000
```

After safe compaction:

```text
snapshot @ 900,000
events 900,001–1,000,000
```

Events before 900,000 can be removed from the active log if policy permits.

# 63. Event Retention Classes

Not every event needs the same retention.

Possible classes:

```text
EPHEMERAL
OPERATIONAL
AUDIT
SECURITY
FORENSIC
```

# 64. Audit Events

Security-sensitive events should generally receive stronger retention:

```text
AUTHORIZATION_GRANTED
AUTHORIZATION_DENIED
EPOCH_CHANGED
FENCE_REJECTED
PRIVILEGED_COMMAND
```

# 65. Idempotency Store

The persistence layer should retain idempotency records long enough to cover the retry window.

Otherwise:

```text
old command
```

may arrive after its deduplication record has been deleted and accidentally execute twice.

# 66. Idempotency Expiration

The expiration period must be based on:

```text
maximum retry window
+
network delay assumptions
+
recovery window
```

not an arbitrary short TTL.

# 67. Transaction Boundaries

Example transaction:

```text
BEGIN

append EXECUTION_START_REQUESTED
insert outbox command

COMMIT
```

This ensures:

```text
durable intent
+
durable dispatch obligation
```

exist together.

# 68. Agent Inbox Transaction

Agent may perform:

```text
BEGIN

insert command_id into inbox
record command state

COMMIT
```

before starting the process.

Then if the process crashes:

```text
command remains known
```

and recovery can determine what happened.

# 69. Inbox and Process Start

A stronger pattern:

```text
durable inbox
      ↓
execution record
      ↓
process start
```

with explicit recovery state.

Do not rely solely on:

```text
process PID
```

to identify command ownership.

# 70. Durable Execution Record

An Agent execution record might contain:

```text
execution_id
attempt_id
command_id
process_identity
start_time
state
checkpoint_id
result_id
```

# 71. Crash Window

Consider:

```text
1. create process
2. crash
3. before recording PID
```

Without recovery-aware design:

```text
orphan process
```

may exist.

Therefore process supervisors and durable execution records should be integrated where possible.

# 72. Process Supervisor

A supervisor can maintain:

```text
execution_id → process
```

independently from the Agent controller process.

This improves crash recovery.

# 73. Orphan Detection

On Agent restart:

```text
enumerate supervised processes
```

then match them against:

```text
durable execution records
```

Unmatched processes become:

```text
ORPHAN
```

and require policy-defined handling.

# 74. Orphan Is Not Automatically Kill

An orphan process may represent:

```text
valid execution whose metadata was lost
```

or:

```text
stale execution
```

Therefore:

```text
ORPHAN
```

should trigger reconciliation, not blind termination.

# 75. Resource Accounting Recovery

After crash:

```text
reserved resources
```

must be reconstructed from authoritative reservations.

Never reconstruct resource usage merely from:

```text
old in-memory scheduler queues
```

# 76. Reservation Reconciliation

For each reservation:

```text
ACTIVE
EXPIRED
RELEASED
UNKNOWN
```

the resource manager determines the authoritative state.

# 77. Scheduler Queue Reconstruction

Queues can be rebuilt from authoritative state:

```text
Work state
+
dependency state
+
resource state
+
policy
```

rather than persisted as unquestionable truth.

# 78. Queue Is Derived State

This is an important architectural rule:

```text
authoritative state
       ↓
scheduler index/queue
```

not:

```text
queue
       ↓
authoritative state
```

# 79. Rebuilding READY

After recovery:

```text
all nonterminal Work
        ↓
evaluate eligibility
        ↓
rebuild READY index
```

This avoids stale queue entries surviving crashes.

# 80. Recovery Idempotency

Running recovery twice should not produce:

```text
duplicate reservations
duplicate executions
duplicate commands
```

Therefore recovery operations must be idempotent.

# 81. Recovery as a Reconciliation Pass

Conceptually:

```text
replay durable history
        ↓
inspect external observations
        ↓
derive discrepancies
        ↓
emit corrective events
        ↓
repeat until converged
```

# 82. Convergence

A healthy recovery system should converge toward:

```text
authoritative state
≈
observed external state
```

subject to uncertainty.

# 83. Reconciliation Loop

```text
OBSERVE
   ↓
COMPARE
   ↓
CLASSIFY
   ↓
CORRECT
   ↓
PERSIST
   ↓
OBSERVE AGAIN
```

# 84. Correction Must Be Evidence-Based

If an Agent reports:

```text
execution unknown
```

the controller should not manufacture:

```text
FAILED
```

just to make the state machine progress.

Instead:

```text
UNKNOWN
```

remains until evidence changes the classification.

# 85. Recovery Completion

Recovery should have explicit barriers:

```text
STATE_REPLAY_COMPLETE
SNAPSHOT_VALIDATED
LEASES_RECONSTRUCTED
RESERVATIONS_RECONCILED
AGENTS_RECONCILED
QUEUES_REBUILT
OUTBOX_RESTORED
```

Only then:

```text
CONTROL_PLANE_READY
```

# 86. Control-Plane Readiness

A controller that has loaded its database but has not reconciled Agents is not necessarily ready to schedule.

Therefore:

```text
database_loaded ≠ runtime_ready
```

# 87. Persistence Invariants

```text
1. Authoritative state is explicitly defined.

2. Memory is never treated as durable authority.

3. Events are immutable.

4. Event identity is stable.

5. Event ordering semantics are explicit.

6. Reducers are deterministic.

7. Reducers do not perform uncontrolled side effects.

8. Commands express intent.

9. Events express observed transitions.

10. Durable state and required dispatch intent can be committed atomically where required.

11. Outbox delivery is retryable.

12. Agent inbox processing is deduplicated.

13. Transport retry does not automatically create execution retry.

14. Execution attempts have independent identities.

15. Snapshots have explicit log positions.

16. Snapshots are integrity-verified.

17. Partial snapshots never replace valid snapshots.

18. Recovery starts from a verified snapshot or genesis.

19. Only committed events are replayed into authoritative state.

20. Time-based state is reconstructed from durable deadlines.

21. In-memory timers are never recovery authority.

22. Event corruption is detected.

23. Corrupt events are not silently skipped.

24. Compaction preserves all required recovery semantics.

25. Idempotency records survive the required retry window.

26. Scheduler queues are derived state.

27. Queue state can be rebuilt.

28. Resource reservations are reconciled from authoritative resource state.

29. Agent incarnations are reconstructed explicitly.

30. Orphan executions require reconciliation.

31. Orphan processes require policy-defined handling.

32. Recovery is idempotent.

33. Recovery precedes scheduling.

34. Reconciliation is evidence-driven.

35. UNKNOWN remains a valid state.

36. State corrections are themselves durable events.

37. Control-plane readiness requires all mandatory recovery barriers.

38. Durable commit semantics are explicitly defined.

39. Replication guarantees are explicitly defined.

40. Persistence failures fail closed rather than fabricating state.
```

# 88. Full Persistence Architecture

The resulting model is:

```text
                         COMMAND
                            │
                            ↓
                    ┌──────────────┐
                    │ Command      │
                    │ Handler      │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │ Transaction  │
                    └──────┬───────┘
                           │
              ┌────────────┴────────────┐
              ↓                         ↓
       ┌──────────────┐          ┌──────────────┐
       │ Event Log    │          │ Outbox       │
       └──────┬───────┘          └──────┬───────┘
              ↓                         ↓
       ┌──────────────┐             Transport
       │ Reducer      │                 ↓
       └──────┬───────┘               Agent
              ↓
       ┌──────────────┐
       │ Materialized │
       │ State        │
       └──────┬───────┘
              ↓
        Scheduler / API
```

# 89. Crash-Recovery Architecture

```text
                  CONTROL PLANE CRASH
                          │
                          ↓
                  ┌──────────────┐
                  │ Restart      │
                  └──────┬───────┘
                         ↓
                  Load Snapshot
                         ↓
                  Verify Snapshot
                         ↓
                  Replay Events
                         ↓
                  Restore State
                         ↓
              ┌──────────┴──────────┐
              ↓                     ↓
       Restore Outbox       Reconcile Agents
              │                     │
              └──────────┬──────────┘
                         ↓
                Reconcile Resources
                         ↓
                  Rebuild Queues
                         ↓
                 Restore Authority
                         ↓
                  READY TO SERVE
```

# 90. The Persistence Boundary

The strongest conceptual boundary is:

```text
BEFORE COMMIT
    =
intent / proposal

AFTER COMMIT
    =
durable fact
```

This distinction should be reflected throughout NROS.

# 91. The Complete NROS Truth Chain

We can now connect all previous layers:

```text
WORK INTENT
    ↓
ADMISSION
    ↓
SCHEDULING DECISION
    ↓
RESOURCE RESERVATION
    ↓
DURABLE COMMAND INTENT
    ↓
NETWORK DELIVERY
    ↓
AGENT ACCEPTANCE
    ↓
EXECUTION EVIDENCE
    ↓
RESULT
    ↓
ARTIFACT VERIFICATION
    ↓
DURABLE COMMIT
    ↓
LOGICAL COMPLETION
    ↓
RECONCILIATION
```

Each arrow is a **semantic boundary**.

None should be silently collapsed.

# Part CVII — Public API & Control-Plane Surface

The next layer should define the external NROS API:

```text
Work submission
→ Work inspection
→ Execution inspection
→ scheduling controls
→ Agent registration
→ capability discovery
→ command submission
→ event streaming
→ artifact references
→ cancellation
→ retries
→ administrative operations
→ authorization boundaries
→ API versioning
```

The central question becomes:

> **What can an external client ask NROS to do, what can it observe, and which operations are guaranteed to be durable, idempotent, asynchronous, or eventually consistent?**

# NROS — Part CVII: Public API & Control-Plane Surface

We now expose the internal runtime through a **stable control-plane API**.

The API must not become a thin wrapper around internal structs. It should expose **semantic operations** with explicit guarantees around:

- identity,
- authorization,
- durability,
- idempotency,
- asynchronous execution,
- observation,
- cancellation,
- retries,
- event streaming,
- pagination,
- versioning,
- and consistency.

## 1. API Architecture

The external surface should conceptually be:

```text
                    External Client
                           │
                           ▼
                  ┌─────────────────┐
                  │ API Gateway     │
                  └────────┬────────┘
                           │
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
        Work API      Execution API    Agent API
            │              │              │
            └──────────────┼──────────────┘
                           ▼
                  Control Plane Core
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
      Scheduler        Event Store       Outbox
```

The public API should communicate with the control plane through domain operations rather than manipulating storage directly.

# 2. API Principles

The API should satisfy:

```text
1. Explicit semantics
2. Stable identifiers
3. Idempotent mutation where possible
4. Async-first execution
5. Observable state
6. Explicit consistency
7. Versioned contracts
8. Structured errors
9. Pagination for collections
10. No hidden state transitions
```

# 3. Resource Model

Core resources:

```text
Work
Execution
Attempt
Agent
Capability
Reservation
Artifact
Event
Lease
Checkpoint
```

# 4. Resource Relationships

```text
Work
 ├── Execution
 │     ├── Attempt
 │     ├── Event*
 │     ├── Artifact*
 │     └── Checkpoint*
 │
 └── Policy / Metadata

Agent
 ├── Capability*
 ├── Execution*
 └── Lease*
```

# 5. Stable Resource IDs

Every resource receives a stable ID:

```text
work_id
execution_id
attempt_id
agent_id
artifact_id
checkpoint_id
event_id
reservation_id
lease_id
```

IDs must remain stable across process restarts.

# 6. ID Ownership

The API should define who creates identifiers.

Two useful models:

```text
CLIENT_GENERATED
SERVER_GENERATED
```

For idempotent submission, client-generated request identities are particularly useful.

# 7. Work Submission

Conceptual operation:

```text
POST /v1/works
```

Request:

```text
CreateWorkRequest {
    request_id
    work_spec
    priority
    constraints
    metadata
}
```

Response should identify the durable Work.

# 8. Submission Semantics

The API must explicitly state whether successful creation means:

```text
received
```

or:

```text
durably accepted
```

For NROS, creation should normally mean:

> The Work admission decision and required durable state have been committed.

# 9. Idempotent Work Creation

A client retrying:

```text
request_id = R42
```

must not accidentally create:

```text
Work A
Work B
```

The same logical request should resolve to the same Work identity.

# 10. Work State

The API should expose states such as:

```text
SUBMITTED
ADMITTED
BLOCKED
READY
QUEUED
RUNNING
SUCCEEDED
FAILED
CANCEL_REQUESTED
CANCELLED
UNKNOWN
```

The exact state machine should be defined centrally.

# 11. Work State Is Not Execution State

A Work can have multiple executions:

```text
Work W42
 ├── Execution E1 → FAILED
 └── Execution E2 → RUNNING
```

Therefore:

```text
Work.status
```

must not simply mirror:

```text
Execution.status
```

# 12. Execution API

Conceptual operations:

```text
POST /v1/works/{work_id}/executions
GET  /v1/executions/{execution_id}
POST /v1/executions/{execution_id}/cancel
POST /v1/executions/{execution_id}/retry
```

# 13. Execution Creation

Creating an execution should be explicit.

This prevents accidental coupling between:

```text
Work submission
```

and:

```text
execution attempt
```

# 14. Automatic Scheduling

NROS may automatically create executions when Work becomes eligible.

Therefore the API should expose:

```text
execution_mode
```

or equivalent semantics:

```text
AUTO
MANUAL
DEFERRED
```

# 15. Execution Identity

An Execution identifies the logical execution.

An Attempt identifies a concrete try.

```text
Execution E42
    ├── Attempt A1 → FAILED
    ├── Attempt A2 → FAILED
    └── Attempt A3 → SUCCEEDED
```

# 16. Retry API

Retry should normally create:

```text
new attempt
```

rather than:

```text
new Work
```

The API should preserve the parent Execution relationship.

# 17. Retry Policy

The request can optionally specify:

```text
retry_policy
```

with:

```text
max_attempts
backoff
retryable_errors
```

But the server remains responsible for enforcing global safety limits.

# 18. Cancellation

Cancellation is a request:

```text
POST /v1/executions/{id}/cancel
```

It does not necessarily mean immediate termination.

# 19. Cancellation States

A useful model:

```text
RUNNING
   ↓
CANCEL_REQUESTED
   ↓
CANCELLING
   ↓
CANCELLED
```

Or, if cancellation cannot be confirmed:

```text
CANCEL_REQUESTED
   ↓
UNKNOWN
```

# 20. Cancellation Must Be Durable

The cancellation intent should be persisted before NROS claims:

```text
CANCEL_REQUESTED
```

This prevents cancellation intent from disappearing during a controller crash.

# 21. Forced Cancellation

Administrative APIs may expose:

```text
force = true
```

but this must be heavily authorized.

Forced cancellation may have stronger consequences:

```text
terminate process
release resources
invalidate lease
```

# 22. Agent Registration

Agents should register explicitly:

```text
POST /v1/agents
```

Registration establishes:

```text
agent_id
incarnation
capabilities
protocol version
resources
health state
```

# 23. Agent Incarnation

An Agent restart should produce:

```text
agent_id = A42
incarnation = 17
```

rather than pretending the old runtime instance still exists.

# 24. Agent Lifecycle

Possible states:

```text
REGISTERING
HEALTHY
DEGRADED
DRAINING
UNAVAILABLE
QUARANTINED
RETIRED
```

# 25. Agent Heartbeats

Heartbeat proves communication activity.

It does not necessarily prove:

```text
healthy execution
```

An Agent can be responsive while its execution subsystem is broken.

# 26. Health Model

Separate:

```text
transport health
control-plane health
execution health
resource health
```

Example:

```text
Agent:
transport = HEALTHY
execution = DEGRADED
GPU = UNAVAILABLE
```

# 27. Capability API

Clients may inspect:

```text
GET /v1/agents/{agent_id}/capabilities
```

Capabilities should be structured.

Example:

```text
Capability {
    name
    version
    constraints
    availability
}
```

# 28. Capability Is Not Permission

An Agent may advertise:

```text
rust-execution
```

without every client being authorized to request it.

Capability:

> What the Agent can do.

Authorization:

> What the caller may ask it to do.

# 29. Capability Matching

Scheduling can compute:

```text
Work requirements
        ×
Agent capabilities
        ×
Resource availability
        ×
policy
```

to determine eligibility.

# 30. Resource API

Reservations may be observable:

```text
GET /v1/reservations/{reservation_id}
```

But arbitrary clients should not necessarily be allowed to mutate them.

# 31. Reservation Ownership

A reservation should identify:

```text
reservation_id
execution_id
agent_id
resource
quantity
epoch
state
expires_at
```

# 32. Lease API

Administrative inspection:

```text
GET /v1/leases/{lease_id}
```

Lease mutation should generally remain internal or strongly privileged.

# 33. Event API

Events are first-class observability resources.

Possible:

```text
GET /v1/events
GET /v1/executions/{id}/events
GET /v1/works/{id}/events
```

# 34. Event Filtering

Filtering may include:

```text
resource_id
event_type
time_range
sequence
correlation_id
severity
```

# 35. Pagination

Never return unbounded event lists.

Use cursor pagination:

```text
GET /v1/events?cursor=...
```

A cursor should represent a stable traversal position rather than simply:

```text

```

# 36. Cursor Semantics

A cursor should encode enough information to preserve:

```text
filter context
position
```

and ideally expire when necessary.

# 37. Event Streaming

For live observation:

```text
GET /v1/events/stream
```

could provide:

```text
event
event
event
...
```

through an appropriate streaming transport.

# 38. Resume

A client disconnects after:

```text
sequence = 842
```

and reconnects with:

```text
resume_from = 843
```

if retained history permits.

# 39. Event Stream Guarantees

Document whether the stream is:

```text
at-most-once
at-least-once
best-effort
replayable
ordered
```

NROS should prefer:

```text
ordered + replayable
```

for authoritative control-plane events where practical.

# 40. API Consistency

Every GET operation should define its consistency model.

Examples:

```text
STRONG
COMMITTED
EVENTUAL
LOCAL
```

# 41. Strong vs Eventual

A client requesting:

```text
GET /executions/E42
```

immediately after:

```text
POST /executions/E42/cancel
```

needs to know whether it will see:

```text
CANCEL_REQUESTED
```

immediately or only after propagation.

# 42. Mutation Response

A mutation response can include:

```text
operation_id
resource_id
accepted_at
commit_index
state
```

This gives clients a durable observation point.

# 43. Asynchronous Operations

Long-running administrative actions should return an operation resource:

```text
Operation O42
```

rather than blocking indefinitely.

# 44. Operation Resource

Conceptually:

```text
Operation {
    operation_id
    type
    state
    created_at
    started_at
    completed_at
    result
    error
}
```

# 45. Operation States

```text
PENDING
RUNNING
SUCCEEDED
FAILED
CANCELLED
```

# 46. Operation Polling

Clients can query:

```text
GET /v1/operations/{operation_id}
```

This provides a generic mechanism for asynchronous API operations.

# 47. Artifact API

Artifacts should be referenced rather than embedded in normal control responses.

```text
GET /v1/artifacts/{artifact_id}
```

may return metadata:

```text
size
digest
media_type
created_at
execution_id
```

# 48. Artifact Integrity

An artifact should have:

```text
artifact_id
digest
size
```

The digest provides content identity.

# 49. Artifact Upload

Uploading should not automatically mean:

```text
RESULT_COMMITTED
```

The artifact must be validated and associated with the relevant Execution.

# 50. Checkpoint API

Checkpoint inspection:

```text
GET /v1/executions/{execution_id}/checkpoints
```

Each checkpoint may expose:

```text
checkpoint_id
sequence
digest
created_at
status
```

# 51. Checkpoint Semantics

A checkpoint means:

> A recoverable execution state has been durably recorded.

It does not automatically mean:

> The execution can resume successfully.

# 52. Resume API

If supported:

```text
POST /v1/executions/{id}/resume
```

must specify whether it means:

```text
resume from latest checkpoint
```

or:

```text
create a new attempt from checkpoint
```

The latter is generally safer.

# 53. API Authorization

Authorization should operate at multiple levels:

```text
principal
role
resource
operation
```

# 54. Example Permission Model

```text
work:read
work:create
work:cancel

execution:read
execution:start
execution:retry
execution:cancel

agent:read
agent:register

admin:reconcile
admin:drain
admin:force_cancel
```

# 55. Resource-Level Authorization

A caller may have:

```text
execution:read
```

but only for:

```text
tenant = T42
```

Therefore permission checks need resource context.

# 56. Authorization Before Mutation

Correct:

```text
authenticate
   ↓
authorize
   ↓
validate
   ↓
mutate
```

Not:

```text
mutate
   ↓
check permission
```

# 57. Validation vs Authorization

These should remain distinct.

Validation asks:

> Is this request structurally and semantically valid?

Authorization asks:

> Is this principal allowed to perform it?

# 58. Tenant Isolation

If NROS supports multi-tenancy, every tenant-owned resource should carry:

```text
tenant_id
```

Authorization must enforce tenant boundaries.

# 59. Cross-Tenant References

A request must not reveal whether another tenant owns:

```text
Work W42
```

through distinguishable error behavior if isolation requirements prohibit such disclosure.

# 60. API Error Envelope

All API errors should have a stable shape:

```text
{
    code,
    category,
    message,
    retryable,
    request_id,
    details
}
```

# 61. Request ID

Every incoming API request should receive:

```text
request_id
```

This enables tracing across:

```text
API
→ command handler
→ event log
→ outbox
→ Agent
```

# 62. Correlation ID

Clients can optionally provide:

```text
correlation_id
```

to associate multiple API calls with one business operation.

# 63. Idempotency Header / Field

Mutation endpoints should support an idempotency mechanism.

Conceptually:

```text
Idempotency-Key: K42
```

The API must define:

```text
scope
retention
conflict behavior
```

# 64. Idempotency Conflict

Same key:

```text
K42
```

with different request body should produce:

```text
IDEMPOTENCY_CONFLICT
```

not a second mutation.

# 65. Rate Limiting

API limits should be explicit:

```text
requests/sec
concurrent operations
stream count
payload size
```

# 66. Rate Limit Response

A throttled client should receive a structured error indicating:

```text
retryable = true
```

and, where appropriate, a retry hint.

# 67. Backpressure at API Boundary

The API must not allow external clients to generate unlimited:

```text
Work
executions
events
streams
```

faster than the control plane can process them.

# 68. Admission Control

Before accepting Work:

```text
validate
authorize
quota check
resource feasibility
policy
durability
```

# 69. Admission ≠ Scheduling

A Work can be:

```text
ADMITTED
```

but remain:

```text
BLOCKED
```

because its dependencies or resources are unavailable.

# 70. Query APIs Must Not Mutate

A GET-like observation operation must not silently:

```text
start execution
renew lease
allocate resources
```

This preserves predictability.

# 71. Administrative API

Administrative operations may include:

```text
drain agent
resume agent
reconcile agent
pause scheduler
resume scheduler
trigger recovery
inspect state
```

These require stronger authorization.

# 72. Scheduler Pause

Pausing scheduling should mean:

```text
no new scheduling decisions
```

It should not necessarily mean:

```text
terminate running executions
```

# 73. Agent Drain

Drain semantics:

```text
DRAIN_REQUESTED
    ↓
stop assigning new Work
    ↓
existing executions continue
    ↓
active count = 0
    ↓
DRAINED
```

# 74. Force Drain

A force-drain operation may additionally:

```text
cancel
migrate
terminate
```

depending on policy.

It must never be implied by normal drain.

# 75. Reconciliation Endpoint

An administrator may request:

```text
POST /v1/admin/reconcile/agents/{id}
```

The API should return an asynchronous Operation if reconciliation is long-running.

# 76. API Versioning

The public contract should be versioned:

```text
/v1/
```

or an equivalent explicit versioning mechanism.

# 77. Semantic Versioning Is Not Enough

API compatibility should distinguish:

```text
backward-compatible additions
breaking changes
behavioral changes
```

A field addition may be syntactically compatible but semantically disruptive.

# 78. Compatibility Rule

Existing clients should continue to work when:

```text
new optional field
new event type
new capability
```

is introduced, provided the old semantics remain valid.

# 79. Breaking Changes

Examples:

```text
renaming required field
changing identifier meaning
changing state transition semantics
changing authorization rules
changing idempotency behavior
```

should require explicit versioning or migration.

# 80. OpenAPI / Schema Contract

The API contract should have machine-readable schemas.

Possible formats:

```text
OpenAPI
JSON Schema
Protocol Buffers
```

depending on transport.

The authoritative schemas should live with the repository rather than being generated only in runtime memory.

# 81. Schema Validation

CI should validate:

```text
schema syntax
request examples
response examples
compatibility
generated clients
```

# 82. Contract Tests

Contract tests should verify:

```text
client request
→ API
→ expected semantic response
```

rather than merely testing serialization.

# 83. API State-Machine Tests

Important tests:

```text
create Work
→ admit
→ schedule
→ execute
→ complete
```

and:

```text
create Work
→ cancel
→ restart controller
→ verify cancellation persists
```

# 84. Failure Tests

At minimum:

```text
API crash
storage failure
outbox retry
Agent disconnect
duplicate request
duplicate command
lost ACK
reconnect
stale epoch
authorization failure
```

# 85. Public API Invariants

```text
1. Every resource has a stable identity.

2. Work identity is distinct from Execution identity.

3. Execution identity is distinct from Attempt identity.

4. Mutation semantics are explicit.

5. Successful mutation has defined durability semantics.

6. Side-effecting requests support idempotency where appropriate.

7. Idempotency conflicts are detected.

8. Authentication precedes authorization.

9. Authorization precedes mutation.

10. Validation and authorization remain separate.

11. GET-style observation does not mutate state.

12. Long-running operations are asynchronous where appropriate.

13. Operation resources have explicit lifecycle states.

14. Event streams define ordering and replay semantics.

15. Pagination is mandatory for unbounded collections.

16. Cursors represent stable traversal positions.

17. API consistency guarantees are explicit.

18. Cancellation is a durable intent.

19. Cancellation acknowledgement does not imply termination unless explicitly defined.

20. Retry creates a new attempt when execution retry is intended.

21. Agent incarnation changes across restart.

22. Agent capability is distinct from authorization.

23. Resource reservations have explicit ownership.

24. Artifact identity is content-verifiable.

25. Checkpoint identity is distinct from execution identity.

26. Administrative operations require stronger authorization.

27. Scheduler pause does not imply execution termination.

28. Agent drain does not imply cancellation.

29. API errors use stable machine-readable codes.

30. Request IDs support end-to-end tracing.

31. API versioning protects semantic compatibility.

32. Schemas are machine-verifiable.

33. Contract tests verify semantics, not only encoding.

34. Rate limits provide bounded admission.

35. Public API cannot bypass durable state transitions.
```

# 86. Canonical External Flow

The complete client flow becomes:

```text
CLIENT
  │
  │ Create Work
  ▼
API
  │
  ▼
AUTHENTICATE
  │
  ▼
AUTHORIZE
  │
  ▼
VALIDATE
  │
  ▼
ADMIT
  │
  ▼
DURABLE COMMIT
  │
  ▼
WORK CREATED
  │
  ▼
SCHEDULER
  │
  ▼
EXECUTION CREATED
  │
  ▼
OUTBOX
  │
  ▼
AGENT
  │
  ▼
EXECUTION
  │
  ▼
EVENTS
  │
  ▼
RESULT
  │
  ▼
COMMIT
  │
  ▼
CLIENT OBSERVES
```

# 87. API Is an Observation Boundary

The API should expose enough information to answer:

```text
What happened?
When?
Why?
Who requested it?
Who authorized it?
Which execution performed it?
Which attempt produced it?
Which Agent executed it?
Which event proves it?
Which artifact proves the result?
```

That is substantially stronger than exposing only:

```text
status = "success"
```

# 88. API Must Preserve Evidence

A completed response should ideally be traceable through:

```text
request_id
      ↓
command_id
      ↓
execution_id
      ↓
attempt_id
      ↓
event sequence
      ↓
artifact/checkpoint
      ↓
commit event
```

This creates an end-to-end evidence chain.

# 89. Control Plane vs Data Plane

NROS should preserve the distinction:

```text
CONTROL PLANE
    Work
    scheduling
    authorization
    execution state
    commands
    events
    leases

DATA PLANE
    process I/O
    artifacts
    model data
    large payloads
    execution streams
```

The public control API should not become a giant data-transfer channel.

# 90. Final API Principle

The most important rule is:

> **The public API exposes durable state transitions and observations; it does not bypass the runtime's authority, persistence, scheduling, or reconciliation mechanisms.**

This gives NROS a coherent chain from:

```text
API request
→ authenticated intent
→ authorized command
→ durable event
→ scheduled execution
→ Agent evidence
→ durable result
→ observable state
```

# Part CVIII — Security & Trust Architecture

The next layer should formalize the **NROS trust model**:

```text
principal identity
→ authentication
→ authorization
→ capabilities
→ trust domains
→ credentials
→ key rotation
→ message integrity
→ replay protection
→ lease fencing
→ privileged operations
→ tenant isolation
→ audit evidence
→ compromise recovery
```

The central question becomes:

> **When NROS receives a command, exactly why should it trust the sender, trust the message, trust the requested operation, and trust the resulting state?**

# NROS — Part CVIII: Security & Trust Architecture

Security in NROS is not an isolated authentication layer.

It is a **trust architecture** spanning:

```text
identity
→ authentication
→ authorization
→ capability
→ message integrity
→ freshness
→ execution authority
→ resource authority
→ audit
→ recovery
```

The fundamental rule is:

> **No identity, credential, capability, or message should grant more authority than its explicitly defined trust boundary permits.**

# 1. Security Model

NROS should distinguish five questions:

```text
WHO?
    ↓
Is the principal authenticated?

WHAT?
    ↓
What operation is being requested?

MAY?
    ↓
Is the principal authorized?

WHICH?
    ↓
Which resource/domain does the authority cover?

WHEN?
    ↓
Is the authority still valid?
```

This prevents authentication from being mistaken for authorization.

# 2. Principal

A principal is an entity that can act within NROS.

Examples:

```text
human operator
service
controller
Agent
scheduler
automation client
administrative identity
```

Every security-sensitive operation should resolve to a principal.

# 3. Principal Identity

A principal identity should be stable enough for auditing.

Conceptually:

```text
principal_id
principal_type
issuer
tenant
status
credential_set
```

# 4. Authentication

Authentication establishes:

> This request was presented by a principal possessing acceptable credentials.

It does **not** establish:

> This principal is allowed to perform the requested operation.

# 5. Authentication Methods

Depending on deployment, NROS may support:

```text
mTLS
signed tokens
OIDC/OAuth2
service credentials
local Unix credentials
hardware-backed credentials
```

The protocol should abstract the mechanism from the authorization model.

# 6. Credential Identity

A credential should have an independent identity:

```text
credential_id
principal_id
issuer
created_at
expires_at
status
```

This enables precise revocation and audit.

# 7. Credential Rotation

Credentials must be replaceable without changing the principal identity.

Correct:

```text
Principal P42
    │
    ├── Credential C1
    └── Credential C2
```

During rotation:

```text
C1 → C2
```

rather than:

```text
P42 → P43
```

unless the principal itself changed.

# 8. Key Rotation

A service should be able to rotate signing keys without invalidating historical evidence.

Events should retain enough metadata to identify:

```text
key_id
issuer
algorithm
signature
```

when signatures are used.

# 9. Credential Expiration

Expiration should be explicit.

A credential with:

```text
expires_at
```

must not silently become permanent merely because the local system clock changed.

# 10. Revocation

A revoked credential must cease to authorize new operations according to the system's revocation propagation guarantee.

The architecture must define whether revocation is:

```text
immediate
bounded-delay
eventually consistent
```

# 11. Trust Domains

NROS should separate trust domains.

For example:

```text
┌─────────────────────────────┐
│ Control Plane               │
│                             │
│ Scheduler / API / State     │
└──────────────┬──────────────┘
               │
               │ authenticated protocol
               ▼
┌─────────────────────────────┐
│ Agent Domain                │
│                             │
│ Execution / Supervisor      │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ Workload Domain             │
│                             │
│ Untrusted / restricted code │
└─────────────────────────────┘
```

# 12. Trust Boundary

The workload itself should normally be considered less trusted than the Agent.

Therefore:

```text
Workload compromise
≠
Agent compromise
```

and:

```text
Agent compromise
≠
Control-plane compromise
```

should be architectural goals.

# 13. Least Authority

Each component receives only the permissions required for its function.

For example:

```text
Scheduler
    may schedule

Agent
    may execute assigned work

Artifact service
    may store/retrieve artifacts

API client
    may access authorized resources
```

No component should receive unrestricted control-plane authority merely for convenience.

# 14. Capability vs Permission

A capability describes:

```text
what can be done
```

A permission describes:

```text
what this principal may do
```

A request is valid only when both align.

```text
principal permission
        ×
target capability
        ×
resource policy
        ↓
authorized operation
```

# 15. Resource-Scoped Authority

Authority should preferably be scoped.

Instead of:

```text
execution:cancel:any
```

prefer:

```text
execution:cancel
tenant=T42
work=W17
```

where operationally appropriate.

# 16. Time-Scoped Authority

Some authority should expire.

Examples:

```text
lease
temporary delegation
break-glass access
execution token
registration session
```

This limits the impact of stolen credentials.

# 17. Delegation

A principal may delegate limited authority:

```text
Controller
   ↓
Agent
```

But delegation must preserve:

```text
original principal
delegating principal
delegated scope
expiration
constraints
```

# 18. Delegation Chain

An authorization record may conceptually contain:

```text
principal = P1
delegated_by = P0
scope = execution:start
resource = E42
expires_at = T
```

This permits audit reconstruction.

# 19. No Ambient Authority

A component should not gain authority simply because it is running inside the NROS process.

Avoid implicit rules such as:

```text
"local process = trusted"
```

Instead use explicit credentials and authorization contexts.

# 20. Agent Authentication

Agent registration should authenticate the Agent before accepting:

```text
agent_id
capabilities
resource claims
```

The Agent must not be allowed to select an arbitrary identity.

# 21. Agent Identity Binding

A registration should bind:

```text
authenticated identity
        ↔
agent_id
```

If the credential claims:

```text
agent-A
```

but the request attempts to register:

```text
agent-B
```

the request must fail unless explicit authority permits the mapping.

# 22. Agent Incarnation Security

Every Agent process instance should receive an incarnation identity:

```text
agent_id = A42
incarnation = 91
```

A new process becomes:

```text
incarnation = 92
```

This prevents stale messages from incarnation 91 from controlling incarnation 92.

# 23. Epoch Fencing

NROS should use monotonically increasing epochs for authority transitions.

Example:

```text
epoch 41 → old controller
epoch 42 → new controller
```

Messages carrying:

```text
epoch = 41
```

must not mutate resources governed by:

```text
epoch = 42
```

# 24. Stale Command

A stale command:

```text
command.epoch < current_epoch
```

should produce:

```text
FENCED
```

rather than being executed.

# 25. Fencing Is Stronger Than Authentication

A perfectly authenticated message can still be invalid because it is stale.

Therefore:

```text
authenticated
```

does not imply:

```text
currently authoritative
```

# 26. Replay Protection

A captured valid command must not be reusable indefinitely.

Commands should contain sufficient freshness information, such as:

```text
command_id
timestamp/deadline
nonce
epoch
sender identity
```

where appropriate.

# 27. Command Identity

Every side-effecting command should have a unique:

```text
command_id
```

The receiving Agent persists command identity for the required deduplication window.

# 28. Replay Detection

If:

```text
command_id = C42
```

was already processed, a duplicate request should resolve according to its existing result rather than executing again.

# 29. Duplicate vs Replay

A duplicate can be legitimate:

```text
transport retry
```

A replay may be malicious:

```text
captured old command
```

Both require deduplication, but replay protection additionally requires freshness and authority checks.

# 30. Message Integrity

Control-plane messages should provide integrity protection.

Conceptually:

```text
header
+
payload
+
authentication/integrity data
```

A modified command must fail verification before semantic processing.

# 31. Confidentiality

Not every message necessarily requires identical confidentiality.

Classify data:

```text
public
internal
sensitive
secret
```

Then apply appropriate transport/storage protections.

# 32. Transport Security

Control-plane communication should normally use an authenticated encrypted transport.

For example:

```text
client
  ⇅
TLS/mTLS
  ⇅
NROS control plane
```

The exact mechanism remains deployment-dependent.

# 33. Message Security vs Transport Security

Transport security protects:

```text
connection
```

Message-level security can additionally protect:

```text
message
```

across intermediate components.

This distinction matters when messages traverse:

```text
queues
brokers
proxies
relays
```

# 34. Secure Serialization

Input decoding must be treated as an attack surface.

Reject:

```text
malformed data
unexpected fields where forbidden
oversized payloads
invalid encodings
resource-exhausting structures
```

# 35. Resource Limits

Every public endpoint should define bounded limits for:

```text
payload size
metadata size
event size
batch size
stream count
pagination size
concurrent operations
```

# 36. Parser Safety

Untrusted input must not trigger:

```text
unbounded recursion
unbounded allocation
pathological parsing
```

before admission control.

# 37. Authorization Evaluation

A useful model:

```text
allow =
    authenticated(principal)
    AND
    credential_valid
    AND
    permission_exists
    AND
    resource_scope_matches
    AND
    capability_matches
    AND
    epoch_valid
    AND
    policy_allows
```

# 38. Deny by Default

If an authorization rule is missing:

```text
ALLOW
```

should not be inferred.

The result should be:

```text
DENY
```

# 39. Explicit Privilege Escalation

Administrative operations should never be accessible through ordinary execution permissions.

For example:

```text
execution:run
```

must not imply:

```text
agent:drain
scheduler:pause
credential:rotate
```

# 40. Break-Glass Access

Highly privileged emergency access may exist, but should be:

```text
explicit
time-limited
audited
strongly authenticated
```

# 41. Break-Glass Audit

Every emergency operation should record:

```text
principal
reason
scope
start
expiration
operation
result
```

# 42. Audit Log

Security-sensitive actions should produce immutable audit events.

Examples:

```text
AUTHENTICATION_SUCCEEDED
AUTHENTICATION_FAILED
AUTHORIZATION_GRANTED
AUTHORIZATION_DENIED
CREDENTIAL_CREATED
CREDENTIAL_REVOKED
KEY_ROTATED
PRIVILEGED_OPERATION
EPOCH_CHANGED
FENCE_REJECTED
```

# 43. Audit vs Operational Events

Operational events describe runtime behavior.

Audit events describe security-relevant decisions.

They may share infrastructure but should retain distinct semantics.

# 44. Audit Evidence

An audit record should answer:

```text
who?
what?
which resource?
when?
from where?
under which authority?
with which credential?
what was the result?
```

# 45. Audit Immutability

Audit records should not be casually editable.

Corrections should be represented by:

```text
correction event
```

rather than destructive modification.

# 46. Sensitive Data

Logs must not accidentally expose:

```text
credentials
private keys
authentication tokens
session secrets
```

or unnecessary sensitive payloads.

# 47. Redaction

Structured logging should support explicit fields such as:

```text
secret
token
credential
```

that are automatically redacted.

Do not rely exclusively on developers remembering to redact strings manually.

# 48. Secret Storage

Secrets should not be stored directly inside:

```text
event payloads
configuration files
source code
ordinary metadata
```

unless the security model explicitly permits it.

Prefer a dedicated secret-management mechanism.

# 49. Secret Reference

NROS may store:

```text
secret_ref
```

rather than:

```text
secret_value
```

This allows execution components to obtain secrets under controlled authorization.

# 50. Secret Scope

A workload should receive only the secrets it requires.

For example:

```text
Execution E42
    → secret S7
```

not:

```text
Execution E42
    → all tenant secrets
```

# 51. Secret Lifetime

Credentials injected into workloads should preferably have bounded lifetimes.

Avoid permanent secrets when short-lived credentials can satisfy the operation.

# 52. Artifact Security

Artifacts may contain sensitive execution output.

Access should therefore be controlled independently of:

```text
execution:read
```

when artifact confidentiality differs from execution metadata.

# 53. Artifact Authorization

A principal might be permitted to inspect:

```text
execution metadata
```

without being permitted to download:

```text
private artifact
```

# 54. Checkpoint Security

Checkpoints can contain complete execution state and therefore may be more sensitive than ordinary metadata.

Protect:

```text
checkpoint storage
checkpoint references
checkpoint download
checkpoint restoration
```

with explicit policy.

# 55. Tenant Isolation

For multi-tenant deployments:

```text
tenant A
    ↕
strict boundary
    ↕
tenant B
```

should apply to:

```text
Work
Execution
Agent assignments
Artifacts
Events
Secrets
Quotas
Audit views
```

# 56. Cross-Tenant Scheduling

If shared Agents can execute Work from multiple tenants, the scheduler must explicitly enforce isolation.

A shared Agent should not automatically imply:

```text
tenant data sharing
```

# 57. Workload Isolation

Where workload isolation is required, use operating-system or runtime mechanisms such as:

```text
process isolation
containers
namespaces
resource limits
sandboxing
```

according to deployment needs.

# 58. Resource Isolation

Prevent one tenant from exhausting shared resources through:

```text
CPU quotas
memory quotas
storage quotas
execution concurrency
API rate limits
queue limits
```

# 59. Scheduler Security

The scheduler must not trust workload-provided claims such as:

```text
"I require 1 CPU"
"I am safe"
"I am authorized"
```

without validating them against policy.

# 60. Capability Claims

Agent-reported capabilities should be authenticated.

Otherwise a malicious Agent could claim:

```text
GPU_ADMIN
SECRET_ACCESS
PRIVILEGED_EXECUTION
```

and receive workloads it should not receive.

# 61. Capability Attestation

Where high assurance is required, capability claims may be tied to:

```text
authenticated Agent identity
software version
deployment policy
attestation evidence
```

# 62. Software Identity

Security decisions may depend on the Agent software version.

A deployment policy could require:

```text
agent_version >= approved_version
```

before allowing privileged workloads.

# 63. Quarantine

An Agent may be moved to:

```text
QUARANTINED
```

when:

```text
credential compromise suspected
unexpected capability claims
protocol violation
integrity failure
repeated fencing
```

# 64. Quarantine Semantics

Quarantine should normally prevent:

```text
new Work assignments
```

while preserving enough access to:

```text
diagnose
collect evidence
revoke credentials
```

according to policy.

# 65. Compromised Agent

If an Agent is suspected compromised:

```text
1. Fence current incarnation.
2. Stop new assignments.
3. Revoke/rotate credentials.
4. Preserve audit evidence.
5. Reconcile active executions.
6. Evaluate artifact integrity.
7. Establish a new trusted incarnation.
```

# 66. Credential Compromise

A compromised credential should not require destroying unrelated principal identities.

Instead:

```text
revoke credential C17
issue credential C18
```

and preserve the audit relationship.

# 67. Epoch Rotation After Compromise

When control authority changes after compromise:

```text
epoch 100
   ↓
fence
   ↓
epoch 101
```

All stale commands from epoch 100 become invalid.

# 68. Control-Plane Compromise

If the control plane itself is compromised, ordinary application-level authorization may no longer be sufficient.

High-assurance deployments may therefore require:

```text
external trust anchor
independent audit
key isolation
multi-party authorization
```

for critical operations.

# 69. Multi-Party Authorization

Critical actions could require:

```text
principal A
+
principal B
```

before execution.

Potential examples:

```text
destroy tenant
rotate root authority
restore from untrusted backup
force release protected resource
```

# 70. Recovery Security

Recovery data must be treated as security-sensitive.

An attacker able to modify:

```text
event log
snapshot
lease state
epoch
```

could potentially manufacture runtime authority.

Therefore recovery inputs require integrity verification.

# 71. Snapshot Trust

A snapshot should only become authoritative if:

```text
format valid
integrity valid
schema supported
trust metadata valid
```

# 72. Backup Security

Backups must preserve:

```text
confidentiality
integrity
retention
access control
```

A backup is not trustworthy merely because it can be successfully restored.

# 73. Restore Verification

After restoration:

```text
verify snapshot
verify event chain
verify schema
verify authority epoch
verify credentials
reconcile external systems
```

before returning to service.

# 74. Security State Machine

A principal/resource relationship may move through:

```text
UNKNOWN
   ↓
AUTHENTICATED
   ↓
AUTHORIZED
   ↓
ACTIVE
   ↓
REVOKED
```

with:

```text
DENIED
QUARANTINED
EXPIRED
```

as explicit terminal or blocking conditions where applicable.

# 75. Security Invariants

```text
1. Authentication never implies authorization.

2. Authorization is deny-by-default.

3. Every privileged action has an identifiable principal.

4. Credentials have explicit lifecycle semantics.

5. Credential rotation does not require principal replacement.

6. Revocation semantics are explicitly bounded.

7. Trust domains are explicit.

8. Workloads are less trusted than control components by default.

9. Least authority is the default allocation strategy.

10. Capabilities are distinct from permissions.

11. Authority may be resource-scoped.

12. Authority may be time-scoped.

13. Delegation preserves the originating authority chain.

14. No ambient authority is assumed.

15. Agent identity is bound to authenticated identity.

16. Agent incarnations are distinct.

17. Epochs fence stale authority.

18. Authenticated stale commands are still rejected.

19. Side-effecting commands have stable identities.

20. Replay protection is explicit.

21. Duplicate commands are deduplicated.

22. Message integrity is verified before semantic processing.

23. Transport confidentiality is not confused with message authenticity.

24. Input sizes and resource consumption are bounded.

25. Secrets are not exposed through ordinary logs.

26. Secret values are not unnecessarily persisted.

27. Artifact access is independently authorized.

28. Checkpoint access is independently authorized.

29. Tenant isolation applies to control and data resources.

30. Agent capability claims are authenticated.

31. Suspicious Agents can be quarantined.

32. Compromised authority can be fenced.

33. Security events are auditable.

34. Audit history is tamper-evident or otherwise integrity-protected.

35. Emergency privileges are explicit and time-limited.

36. Recovery inputs are integrity-verified.

37. Restored state is reconciled before scheduling resumes.

38. Security failures fail closed where safety permits.

39. Security state transitions are durable.

40. No security decision depends solely on volatile memory.
```

# 76. End-to-End Trust Chain

The complete security chain becomes:

```text
                  CLIENT
                    │
                    ▼
             ┌──────────────┐
             │ Identity     │
             └──────┬───────┘
                    ▼
             ┌──────────────┐
             │ Authenticate │
             └──────┬───────┘
                    ▼
             ┌──────────────┐
             │ Authorize    │
             └──────┬───────┘
                    ▼
             ┌──────────────┐
             │ Freshness    │
             │ + Epoch      │
             └──────┬───────┘
                    ▼
             ┌──────────────┐
             │ Validate     │
             └──────┬───────┘
                    ▼
             ┌──────────────┐
             │ Durable      │
             │ Commit       │
             └──────┬───────┘
                    ▼
             ┌──────────────┐
             │ Outbox       │
             └──────┬───────┘
                    ▼
             ┌──────────────┐
             │ Agent        │
             │ Authenticate │
             └──────┬───────┘
                    ▼
             ┌──────────────┐
             │ Fence Check  │
             └──────┬───────┘
                    ▼
             ┌──────────────┐
             │ Execute      │
             └──────┬───────┘
                    ▼
             ┌──────────────┐
             │ Evidence     │
             └──────┬───────┘
                    ▼
             ┌──────────────┐
             │ Durable      │
             │ Result       │
             └──────────────┘
```

# 77. Security as a Runtime Property

Security should not be implemented as:

```text
API authentication middleware
```

alone.

It must exist across:

```text
API
scheduler
event store
outbox
Agent protocol
execution supervisor
artifact store
checkpoint store
recovery
administration
```

# 78. Trust Is Time-Bounded

A particularly important NROS principle is:

```text
trust
+
identity
+
authority
+
freshness
+
epoch
```

A message can be authentic and still invalid because its authority has expired.

# 79. Trust Is Evidence-Bounded

Similarly:

```text
authenticated sender
```

does not prove:

```text
execution succeeded
```

The result still requires execution evidence.

Thus the security model and evidence model converge on the same principle:

> **Identity establishes who acted; evidence establishes what actually happened.**

# 80. Security Boundary Summary

NROS now has four major semantic boundaries:

```text
1. API boundary
   external intent → authenticated request

2. Authority boundary
   authenticated request → authorized operation

3. Execution boundary
   authorized command → Agent execution

4. Evidence boundary
   execution claim → verified durable result
```

These boundaries should remain explicit throughout implementation.

# Part CIX — Observability, Telemetry & Evidence Architecture

The next layer should define how NROS makes its behavior **measurable, diagnosable, and independently verifiable**:

```text
logs
→ metrics
→ traces
→ events
→ audit records
→ execution evidence
→ health signals
→ diagnostics
→ anomaly detection
→ forensic reconstruction
```

The key question becomes:

> **Can an operator reconstruct what NROS believed, why it made a decision, what actually happened, and where the evidence for that conclusion came from?**

# NROS — Part CIX: Observability, Telemetry & Evidence Architecture

Observability in NROS must go beyond collecting logs.

The runtime should make it possible to reconstruct:

```text
what happened
why it happened
when it happened
which component decided it
which identity caused it
which state existed at the time
what evidence supports the conclusion
```

The core model is:

```text
Metrics
   +
Logs
   +
Traces
   +
Events
   +
Audit
   +
Execution Evidence
   ↓
Operational Truth
```

# 1. Observability Model

NROS should distinguish six complementary signal classes:

```text
1. Metrics
2. Logs
3. Traces
4. Domain Events
5. Audit Events
6. Evidence Records
```

They answer different questions.

# 2. Metrics

Metrics answer:

> How much? How often? How long? How many?

Examples:

```text
work_submissions_total
executions_started_total
executions_completed_total
execution_duration_seconds
scheduler_queue_depth
agent_count
active_leases
event_lag
```

# 3. Logs

Logs answer:

> What did a component observe or decide?

A structured log should contain fields such as:

```text
timestamp
level
component
message
request_id
correlation_id
work_id
execution_id
attempt_id
agent_id
event_id
```

# 4. Traces

Traces answer:

> How did one operation propagate through the system?

Example:

```text
API request
   │
   ├── authorization
   ├── admission
   ├── persistence
   ├── scheduling
   ├── dispatch
   ├── Agent execution
   └── result commit
```

# 5. Domain Events

Domain events answer:

> What authoritative state transition occurred?

Examples:

```text
WorkCreated
WorkAdmitted
ExecutionCreated
AttemptStarted
AttemptFinished
ExecutionSucceeded
ExecutionFailed
AgentRegistered
LeaseAcquired
LeaseReleased
```

These should have semantic meaning independent of logging.

# 6. Audit Events

Audit events answer:

> What security-sensitive action occurred, under whose authority?

Examples:

```text
AuthorizationDenied
CredentialRevoked
AgentQuarantined
AdministrativeOperation
PolicyChanged
```

# 7. Evidence Records

Evidence answers:

> What proves that a runtime claim is true?

Examples:

```text
process exit status
resource allocation record
checkpoint digest
artifact digest
Agent acknowledgement
execution transcript
verification result
```

# 8. Signals Must Not Be Confused

A log saying:

```text
"execution completed successfully"
```

is not itself proof of successful execution.

The authoritative evidence should originate from the execution state machine and durable result commit.

# 9. Canonical Observability Graph

```text
                 Request
                    │
                    ▼
               Trace Root
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
      Logs        Metrics      Audit
        │           │           │
        └───────────┼───────────┘
                    ▼
               Domain Event
                    │
                    ▼
              State Transition
                    │
                    ▼
              Evidence Record
                    │
                    ▼
              Durable Result
```

# 10. Correlation Identity

All related signals should share stable correlation identifiers where possible.

Minimum useful identifiers:

```text
request_id
correlation_id
command_id
work_id
execution_id
attempt_id
agent_id
event_id
```

# 11. Request ID

`request_id` identifies one externally received request.

Example:

```text
R-8f32
```

It should remain available through the request lifecycle.

# 12. Correlation ID

`correlation_id` groups multiple requests belonging to one higher-level operation.

Example:

```text
deployment-2026-08-21-0042
```

# 13. Command ID

`command_id` identifies a side-effecting command.

It is especially important for:

```text
deduplication
replay detection
audit
Agent acknowledgement
```

# 14. Resource IDs

Domain resources provide the strongest semantic correlation:

```text
work_id
execution_id
attempt_id
agent_id
artifact_id
checkpoint_id
lease_id
```

# 15. Trace IDs

Distributed tracing may introduce:

```text
trace_id
span_id
parent_span_id
```

These should complement, not replace, domain identifiers.

# 16. Trace Context

Propagation should preserve trace context across:

```text
API
→ scheduler
→ command bus
→ Agent
→ workload supervisor
```

where supported.

# 17. Trace Boundaries

Not every internal function needs a span.

Good span boundaries usually correspond to meaningful operations:

```text
admit_work
schedule_execution
dispatch_attempt
start_process
collect_result
commit_execution
```

# 18. Avoid Trace Explosion

Creating spans for every tiny function can generate enormous telemetry overhead.

Tracing should prioritize:

```text
cross-component operations
slow operations
state transitions
failures
security decisions
```

# 19. Metrics Taxonomy

Metrics should be organized by domain.

```text
API
Scheduler
Execution
Agent
Storage
Events
Security
Resources
Recovery
```

# 20. API Metrics

Examples:

```text
api_requests_total
api_request_duration_seconds
api_request_errors_total
api_request_rejected_total
api_rate_limited_total
```

# 21. Scheduler Metrics

Examples:

```text
scheduler_cycles_total
scheduler_decisions_total
scheduler_queue_depth
scheduler_blocked_work
scheduler_dispatch_latency
scheduler_reconciliation_total
```

# 22. Execution Metrics

Examples:

```text
executions_started_total
executions_succeeded_total
executions_failed_total
executions_cancelled_total
execution_duration_seconds
attempts_total
retry_total
```

# 23. Agent Metrics

Examples:

```text
agents_registered
agents_healthy
agents_degraded
agents_draining
agents_quarantined
agent_heartbeat_latency
agent_command_failures
```

# 24. Storage Metrics

Examples:

```text
storage_reads_total
storage_writes_total
storage_errors_total
commit_latency_seconds
transaction_rollbacks_total
```

# 25. Event Metrics

Examples:

```text
events_written_total
events_processed_total
event_processing_latency
event_lag
event_replay_total
duplicate_events_total
```

# 26. Security Metrics

Examples:

```text
authentication_failures_total
authorization_denials_total
replay_rejections_total
fencing_rejections_total
credential_revocations_total
quarantined_agents
```

# 27. Recovery Metrics

Examples:

```text
recovery_attempts_total
recovery_success_total
recovery_failures_total
reconciliation_duration_seconds
stale_state_detected_total
```

# 28. Resource Metrics

Examples:

```text
cpu_allocated
memory_allocated
storage_allocated
active_executions
queue_capacity
resource_utilization
```

# 29. Metric Cardinality

Avoid placing high-cardinality identifiers directly into metric labels.

Bad:

```text
execution_duration{execution_id="E123456"}
```

Better:

```text
execution_duration{result="success"}
```

while keeping `execution_id` in traces/logs/events.

# 30. Cardinality Budget

NROS should explicitly define a cardinality budget for metric labels.

Identifiers such as:

```text
work_id
execution_id
request_id
artifact_id
```

should generally not become unbounded metric dimensions.

# 31. Histograms

Latency should generally be represented as distributions rather than only averages.

Important histograms include:

```text
API latency
queue latency
dispatch latency
execution startup latency
commit latency
event processing latency
```

# 32. Percentiles

Operational dashboards may expose:

```text
p50
p90
p95
p99
```

but percentile calculations should be derived from an appropriate histogram/distribution rather than manually aggregated averages.

# 33. Saturation

Observability should measure not only failures but approaching limits.

Examples:

```text
queue utilization
worker utilization
storage utilization
memory pressure
event backlog
API concurrency
```

# 34. Four Golden Signals

For API/control-plane operations, track:

```text
Latency
Traffic
Errors
Saturation
```

These provide a baseline operational view.

# 35. Runtime Health

Health should not be one boolean.

Instead expose dimensions such as:

```text
control_plane
scheduler
storage
event_pipeline
agent_connectivity
execution
```

# 36. Liveness vs Readiness

Liveness asks:

> Is this process functioning sufficiently to remain alive?

Readiness asks:

> Should this instance receive new work?

These must remain distinct.

# 37. Degraded State

A component can be alive but not ready.

Example:

```text
process = alive
storage = reachable
scheduler = paused
readiness = false
```

# 38. Health Endpoint

Conceptually:

```text
GET /health
```

should provide a minimal process-level signal.

A richer endpoint can expose:

```text
GET /health/details
```

with authorized diagnostic information.

# 39. Health Must Not Leak Secrets

Health responses should never reveal:

```text
credentials
private keys
tokens
sensitive configuration
tenant data
```

# 40. Event Sequence

Authoritative domain events should carry a monotonic ordering value within their defined scope.

For example:

```text
event_sequence = 842
```

# 41. Event Ordering Scope

The system must define whether ordering applies to:

```text
global stream
tenant
resource
partition
aggregate
```

Do not imply global ordering if the implementation only guarantees per-resource ordering.

# 42. Event Envelope

A canonical envelope may contain:

```text
Event {
    event_id
    event_type
    schema_version
    timestamp
    sequence
    producer
    resource_type
    resource_id
    correlation_id
    causation_id
    payload
}
```

# 43. Causation ID

`causation_id` identifies the event or command that caused the current event.

Example:

```text
Command C42
    ↓
WorkAdmitted E100
    ↓
ExecutionCreated E101
```

Then:

```text
E101.causation_id = E100
```

# 44. Event Graph

This creates a causal graph:

```text
request
  ↓
command
  ↓
state transition
  ↓
execution
  ↓
attempt
  ↓
result
```

This is extremely valuable for forensic reconstruction.

# 45. Event Schema Version

Events must carry an explicit schema version.

Example:

```text
event_type = "ExecutionFinished"
schema_version = 2
```

Consumers can then handle evolution explicitly.

# 46. Event Immutability

Authoritative events should be append-only.

Corrections should produce new events.

Do not silently rewrite historical state.

# 47. Derived State

Operational read models may be reconstructed from authoritative events.

Conceptually:

```text
Event Store
     │
     ├── Scheduler View
     ├── API View
     ├── Metrics
     └── Diagnostic View
```

# 48. Read Model Failure

If a derived read model becomes corrupted:

```text
authoritative state
      ↓
rebuild
      ↓
read model
```

This is an important resilience property.

# 49. Observability During Recovery

Recovery itself must generate evidence.

Examples:

```text
RecoveryStarted
RecoveryCheckpointLoaded
StateReconciled
StaleLeaseDetected
LeaseFenced
RecoveryCompleted
```

# 50. Reconciliation Evidence

A reconciliation operation should report:

```text
objects examined
objects corrected
objects fenced
objects abandoned
objects recovered
errors encountered
```

# 51. Execution Evidence

An execution should accumulate evidence through its lifecycle.

```text
AttemptStarted
    ↓
ProcessSpawned
    ↓
ResourceBound
    ↓
OutputObserved
    ↓
ProcessExited
    ↓
ResultValidated
    ↓
ResultCommitted
```

# 52. Evidence Levels

A useful evidence hierarchy:

```text
UNKNOWN
   ↓
OBSERVED
   ↓
CORRELATED
   ↓
VALIDATED
   ↓
COMMITTED
```

# 53. UNKNOWN

The system has no trustworthy evidence.

Example:

```text
Agent disconnected before reporting process state.
```

Do not automatically convert this to:

```text
FAILED
```

# 54. OBSERVED

A component observed something but it has not yet been independently validated.

Example:

```text
Agent reports exit code 1.
```

# 55. CORRELATED

Multiple signals agree.

Example:

```text
process exit
+
Agent report
+
resource release
```

# 56. VALIDATED

The control plane has checked the evidence against its state model.

# 57. COMMITTED

The resulting state has been durably committed.

This should be the strongest normal operational state.

# 58. Evidence Provenance

Every important evidence record should identify:

```text
source
timestamp
producer
resource
observation type
confidence/validation state
```

# 59. Evidence Digest

Large evidence objects should be content-addressed or digest-protected.

Example:

```text
sha256:<digest>
```

The digest allows later verification that the evidence was not modified.

# 60. Evidence Storage

Separate:

```text
metadata
```

from:

```text
large evidence payload
```

The event can contain:

```text
artifact_id
digest
size
```

while the artifact store holds the large object.

# 61. Evidence Retention

Retention policies should distinguish:

```text
operational telemetry
audit events
domain events
execution evidence
artifacts
```

They may have different retention requirements.

# 62. Telemetry Sampling

Tracing may use sampling to control cost.

However, security/audit events should not be dropped merely because tracing is sampled.

# 63. Sampling Rule

A useful distinction:

```text
metrics
    aggregated

traces
    sampled

logs
    policy-filtered

domain events
    authoritative

audit
    authoritative

evidence
    authoritative
```

# 64. Failure Telemetry

Failed operations should generally receive higher observability priority.

Useful enrichment:

```text
error_code
failure_stage
retryable
resource_state
epoch
agent_incarnation
```

# 65. Failure Stage

Instead of only:

```text
FAILED
```

record:

```text
admission_failed
authorization_failed
scheduling_failed
dispatch_failed
startup_failed
execution_failed
validation_failed
commit_failed
```

This drastically improves diagnosis.

# 66. Error Taxonomy

Errors should distinguish:

```text
validation
authorization
authentication
capacity
dependency
transient
permanent
protocol
storage
execution
integrity
fencing
```

# 67. Retry Classification

Every retryable failure should have an explicit reason.

For example:

```text
retryable = true
category = transient
```

rather than relying on message text.

# 68. Anomaly Detection

NROS can detect patterns such as:

```text
repeated Agent crashes
rapid retry loops
unexpected queue growth
event lag
credential failures
fencing spikes
resource starvation
```

# 69. Alerting

Alerts should be based on operational conditions, not individual noisy log messages.

Example:

```text
event_lag > threshold
for sustained interval
```

is better than:

```text
one "event lag" log line
```

# 70. Alert Severity

Use explicit levels:

```text
INFO
WARNING
ERROR
CRITICAL
```

Security and operational severity should remain distinguishable when necessary.

# 71. Diagnostic Bundles

For difficult incidents, NROS should be able to produce a bounded diagnostic bundle containing:

```text
relevant events
state snapshots
metrics
logs
trace references
Agent status
version information
configuration fingerprints
```

# 72. Diagnostic Bundle Security

Diagnostic bundles can contain sensitive information.

Therefore they require:

```text
authorization
redaction
integrity protection
retention policy
```

# 73. Configuration Fingerprint

Instead of dumping full configuration into every event, record a fingerprint:

```text
config_digest
```

This allows operators to determine whether two executions ran under the same configuration.

# 74. Software Version

Every execution evidence chain should identify relevant software versions:

```text
control_plane_version
scheduler_version
agent_version
protocol_version
workload_version
```

# 75. Reproducibility Metadata

Where deterministic behavior matters, record:

```text
configuration digest
input digest
software version
protocol version
policy version
```

This provides the basis for later reproduction.

# 76. Clock Handling

Timestamps are useful but clocks can disagree.

NROS should avoid using wall-clock timestamps as the sole ordering mechanism.

Prefer:

```text
sequence
logical ordering
causation
epoch
```

where available.

# 77. Monotonic Timing

Duration measurements should use monotonic clocks where the platform supports them.

Do not calculate runtime durations solely from wall-clock timestamps.

# 78. Clock Skew

Observability should account for clock skew across:

```text
API nodes
scheduler
Agents
storage systems
```

A timestamp mismatch should not automatically imply causal inconsistency.

# 79. Time Source

Deployments requiring stronger temporal guarantees may record clock synchronization status.

For example:

```text
clock_offset_estimate
clock_sync_state
```

where useful.

# 80. Privacy

Observability systems can accidentally become data-exfiltration systems.

Therefore telemetry should follow:

```text
minimum necessary data
```

principles.

# 81. Payload Redaction

Do not automatically put complete Work payloads into:

```text
logs
traces
metrics
```

Prefer identifiers and controlled references.

# 82. Tenant-Aware Observability

Operators should only see telemetry they are authorized to access.

A global monitoring system must not accidentally expose:

```text
tenant A execution data
```

to:

```text
tenant B
```

# 83. Observability API

Useful endpoints include:

```text
GET /v1/metrics
GET /v1/events
GET /v1/works/{id}/events
GET /v1/executions/{id}/events
GET /v1/executions/{id}/trace
GET /v1/executions/{id}/evidence
GET /v1/agents/{id}/diagnostics
```

Access to detailed endpoints should be permission-controlled.

# 84. Evidence Query

A powerful diagnostic query should be able to answer:

```text
Why is Execution E42 marked FAILED?
```

and return:

```text
final state
failure stage
causing event
attempt
Agent
relevant evidence
artifact references
commit event
```

# 85. Reconstruction Algorithm

Conceptually:

```text
load execution
    ↓
load attempts
    ↓
load causal events
    ↓
load relevant Agent events
    ↓
load evidence references
    ↓
verify integrity
    ↓
reconstruct state transition
    ↓
produce explanation
```

# 86. Explainability

NROS should eventually be able to provide structured explanations such as:

```text
Execution E42 failed because:

Attempt A3
→ Agent A7
→ process exited with code 137
→ memory limit exceeded
→ resource evidence validated
→ execution failure committed
```

This is much more useful than a raw log dump.

# 87. State Explanation

Similarly:

```text
Why is Work W42 BLOCKED?
```

could produce:

```text
dependency D17 unresolved
AND
required capability "gpu" unavailable
```

with references to the underlying state/events.

# 88. Decision Evidence

Scheduling decisions should optionally retain:

```text
candidate Agents
rejected candidates
rejection reasons
selected Agent
policy version
resource snapshot
```

This is especially important for debugging scheduling behavior.

# 89. Scheduler Explainability

Example:

```text
Work W42
  ├─ Agent A1 → rejected: capability mismatch
  ├─ Agent A2 → rejected: draining
  ├─ Agent A3 → rejected: memory unavailable
  └─ Agent A4 → selected
```

This turns the scheduler from a black box into an inspectable system.

# 90. Evidence Cost Control

Not every decision requires maximum evidence.

NROS can define evidence levels:

```text
minimal
standard
diagnostic
forensic
```

Higher levels may increase storage and runtime cost.

# 91. Forensic Mode

For critical incidents, an authorized operator may enable enhanced evidence collection.

This might capture:

```text
additional traces
decision inputs
extended event context
Agent diagnostics
```

The mode itself must be audited.

# 92. Observability Invariants

```text
1. Metrics measure aggregates.

2. Logs describe observations and decisions.

3. Traces describe causal propagation.

4. Domain events describe authoritative state transitions.

5. Audit events describe security-sensitive actions.

6. Evidence records support runtime claims.

7. These signal types must not be conflated.

8. Every important operation has stable correlation identity.

9. Domain events are versioned.

10. Authoritative events are append-only.

11. Derived state can be rebuilt where designed.

12. Event ordering guarantees are explicit.

13. High-cardinality identifiers do not become unbounded metric labels.

14. Duration measurements use monotonic timing where appropriate.

15. Wall-clock timestamps are not the sole ordering mechanism.

16. Failed operations expose structured failure stages.

17. Retryability is explicit.

18. Diagnostic bundles are access-controlled.

19. Telemetry is tenant-aware.

20. Sensitive payloads are redacted.

21. Audit records are retained according to policy.

22. Evidence has provenance.

23. Evidence integrity can be verified.

24. Execution conclusions are backed by evidence.

25. UNKNOWN state is preserved when evidence is insufficient.

26. Recovery generates observable evidence.

27. Reconciliation generates observable evidence.

28. Scheduler decisions can be explained.

29. Observability cannot silently mutate runtime state.

30. Security-sensitive telemetry cannot be disabled through ordinary workload controls.
```

# 93. Unified Evidence Chain

The resulting architecture is:

```text
                    REQUEST
                       │
                       ▼
                 TRACE CONTEXT
                       │
             ┌─────────┼─────────┐
             ▼         ▼         ▼
           LOGS      METRICS    AUDIT
             │         │         │
             └─────────┼─────────┘
                       ▼
                  COMMAND
                       │
                       ▼
                 DOMAIN EVENT
                       │
                       ▼
                 STATE CHANGE
                       │
                       ▼
                  EXECUTION
                       │
             ┌─────────┼─────────┐
             ▼         ▼         ▼
          TRACE     LOGS      EVIDENCE
             │         │         │
             └─────────┼─────────┘
                       ▼
                RESULT VALIDATION
                       │
                       ▼
                DURABLE COMMIT
                       │
                       ▼
                  FINAL STATE
```

# 94. Operational Truth

NROS should treat operational truth as the intersection of:

```text
declared intent
        ∩
authorized action
        ∩
observed execution
        ∩
validated evidence
        ∩
durable state
```

Not merely:

```text
latest log message
```

# 95. Final Observability Principle

> **If NROS cannot explain a state transition, correlate it with its cause, and identify the evidence supporting it, that transition is operationally opaque.**

The architecture should therefore make observability a first-class runtime capability rather than an afterthought.

# Part CX — Reliability, Fault Tolerance & Recovery Architecture

The next layer should formalize how NROS behaves when things fail:

```text
process crash
→ network partition
→ storage failure
→ Agent loss
→ controller restart
→ duplicate delivery
→ stale state
→ partial commit
→ corrupted evidence
→ recovery
→ reconciliation
```

The central question becomes:

> **After arbitrary component failure, what state can NROS prove, what state must it treat as unknown, and how does it safely converge back to a valid state?**

# NROS — Part CX: Reliability, Fault Tolerance & Recovery Architecture

Reliability in NROS should not mean merely:

```text
process does not crash
```

It should mean:

> **When failures occur, NROS preserves the invariants that matter, avoids inventing facts it cannot prove, fences stale authority, and converges toward a valid state through deterministic recovery and reconciliation.**

The core model is:

```text
failure
   ↓
detect
   ↓
classify
   ↓
fence unsafe authority
   ↓
preserve durable truth
   ↓
recover
   ↓
reconcile
   ↓
validate
   ↓
resume
```

# 1. Failure Is a Normal Runtime Condition

NROS must assume that any component can fail:

```text
API
scheduler
controller
Agent
workload
storage
network
event consumer
artifact store
checkpoint store
```

No component should be treated as permanently available.

# 2. Failure Domains

Failures should be classified by domain:

```text
PROCESS
NETWORK
STORAGE
RESOURCE
PROTOCOL
AUTHORITY
EXECUTION
DATA
CONTROL PLANE
EXTERNAL DEPENDENCY
```

This classification determines recovery behavior.

# 3. Process Failure

Examples:

```text
panic
abort
OOM kill
machine reboot
container termination
power loss
```

The system should distinguish:

```text
process disappeared
```

from:

```text
operation failed
```

A crashed process does not automatically prove that its operation failed.

# 4. Network Failure

A network partition can produce:

```text
controller cannot reach Agent
```

without proving:

```text
Agent stopped executing
```

This distinction is fundamental.

# 5. The Unknown State

When evidence is insufficient:

```text
UNKNOWN
```

is preferable to guessing:

```text
FAILED
```

or:

```text
SUCCEEDED
```

# 6. Failure Knowledge Hierarchy

NROS should distinguish:

```text
KNOWN_SUCCESS
KNOWN_FAILURE
KNOWN_CANCELLED
UNKNOWN
```

rather than forcing every disconnected operation into a binary result.

# 7. Storage Failure

Storage failures can occur at multiple levels:

```text
read failure
write failure
transaction failure
timeout
corruption
unavailable database
partial infrastructure failure
```

Each requires different handling.

# 8. Durable Commit Boundary

The runtime must define exactly when a state transition becomes durable.

Conceptually:

```text
prepare
   ↓
validate
   ↓
commit
   ↓
durable
```

Only after the defined commit point may NROS claim durable state.

# 9. No False Success

If persistence fails:

```text
state transition
      ↓
storage error
```

NROS must not return:

```text
SUCCESS
```

merely because the in-memory mutation succeeded.

# 10. Transactional Mutation

Where supported, logically related changes should commit atomically.

Example:

```text
ExecutionCreated
+
scheduler assignment
+
lease ownership
```

must not accidentally leave the system believing only half of the operation happened.

# 11. Atomicity Boundary

If full atomicity is impossible across subsystems, define an explicit boundary:

```text
authoritative transaction
        ↓
outbox
        ↓
external side effect
```

The transaction establishes what NROS knows; the outbox drives external effects.

# 12. Transaction + Outbox

A reliable command path:

```text
BEGIN
  write state
  write domain event
  write outbox entry
COMMIT
```

Then:

```text
outbox worker
   ↓
deliver command
```

This prevents:

```text
state committed
but command forgotten
```

# 13. Outbox Failure

If delivery fails:

```text
outbox entry remains pending
```

and can be retried.

The outbox must not disappear merely because one delivery attempt failed.

# 14. Duplicate Delivery

A retry can produce:

```text
Command C42
Command C42
```

The receiving side must be able to deduplicate according to command identity.

# 15. Exactly-Once Illusion

NROS should avoid claiming universal exactly-once execution.

Across distributed boundaries, the safer model is generally:

```text
durable intent
+
at-least-once delivery
+
idempotent processing
+
deduplication
```

# 16. Command Processing

A command processor should conceptually:

```text
receive
  ↓
authenticate
  ↓
validate
  ↓
check epoch
  ↓
check command identity
  ↓
deduplicate
  ↓
execute
  ↓
record result
```

# 17. Command Result Persistence

If a command has already completed, its result should be recoverable.

Example:

```text
C42
status = COMPLETED
result = ACKNOWLEDGED
```

A duplicate request can then return the prior result.

# 18. Crash Between Side Effect and ACK

Consider:

```text
Agent executes command
      ↓
side effect succeeds
      ↓
Agent crashes
      ↓
ACK never reaches controller
```

The controller cannot assume:

```text
side effect failed
```

This is a classic uncertainty boundary.

# 19. Recovery From Uncertain Command

NROS should use:

```text
command_id
execution identity
Agent state
resource evidence
reconciliation
```

to determine whether the command should be:

```text
confirmed
retried
fenced
marked unknown
```

# 20. Agent Failure

When an Agent disappears:

```text
heartbeat timeout
       ↓
Agent suspected unavailable
       ↓
stop new assignments
       ↓
fence incarnation
       ↓
reconcile active executions
```

# 21. Heartbeat Is Not Proof of Execution

Heartbeat means:

```text
Agent is communicating
```

It does not prove:

```text
specific execution is healthy
```

Execution state requires separate evidence.

# 22. Agent Failure Detection

Failure detection should use bounded suspicion rather than instantaneous assumptions.

Conceptually:

```text
HEALTHY
   ↓
SUSPECTED
   ↓
UNAVAILABLE
```

The exact thresholds should be configurable.

# 23. False Positives

A temporary network partition may make a healthy Agent appear unavailable.

Therefore recovery must be safe even when failure detection is wrong.

This is why fencing is essential.

# 24. Incarnation Fencing

Suppose:

```text
Agent A42
incarnation 10
```

loses connectivity.

A replacement instance starts:

```text
Agent A42
incarnation 11
```

The controller must reject stale commands belonging to incarnation 10.

# 25. Lease Expiration

Execution authority can be represented by a lease:

```text
lease_id
owner
epoch
expires_at
```

When the lease expires, the owner must no longer assume authority.

# 26. Lease Renewal

Lease renewal must be explicit.

A late renewal should not resurrect authority that has already been transferred to a newer epoch.

# 27. Split-Brain Prevention

The critical invariant is:

```text
at most one current authority
```

for a resource within the defined authority domain.

Epochs and fencing should enforce this.

# 28. Scheduler Failure

If the scheduler crashes:

```text
running executions
```

should not automatically be terminated.

The scheduler can restart and reconstruct:

```text
pending Work
active executions
leases
Agent states
```

from durable state.

# 29. Scheduler Restart

Recovery flow:

```text
load durable state
    ↓
load latest authoritative events
    ↓
reconstruct scheduler state
    ↓
validate leases
    ↓
fence stale authority
    ↓
reconcile Agents
    ↓
resume scheduling
```

# 30. Scheduler Must Not Double-Schedule

After restart, it must not create duplicate executions merely because it forgot an earlier scheduling decision.

Stable:

```text
work_id
execution_id
command_id
```

and durable state are required for deduplication.

# 31. Controller Restart

The controller should treat volatile memory as reconstructable.

Critical truth must live in:

```text
durable state
durable events
durable command/result records
```

not only in process memory.

# 32. Restart Recovery Invariant

After restart:

```text
reconstructed_state
```

must be semantically equivalent to the state that existed immediately before the crash, subject to explicitly documented external uncertainty.

# 33. Event Replay

If event sourcing or event-backed reconstruction is used:

```text
snapshot
+
event suffix
```

can reconstruct current state.

# 34. Snapshot + Event Log

Conceptually:

```text
Snapshot S100
    +
Events 101..150
    ↓
Current State
```

This avoids replaying the entire history on every restart.

# 35. Snapshot Integrity

Before use, verify:

```text
format
schema
digest
version
authority metadata
```

# 36. Snapshot Version

Snapshots should identify:

```text
snapshot_version
event_sequence
schema_version
created_at
```

so the replay boundary is unambiguous.

# 37. Corrupted Snapshot

If validation fails:

```text
discard snapshot
   ↓
load earlier valid snapshot
   ↓
replay events
```

or, if required:

```text
rebuild from authoritative event history
```

# 38. Event Corruption

If authoritative event history itself is corrupted, NROS must not silently continue as though nothing happened.

It should enter a controlled recovery state:

```text
DEGRADED
RECOVERY_REQUIRED
```

according to policy.

# 39. Event Integrity

Event integrity may be strengthened through:

```text
sequence numbers
hashes
signatures
content digests
append-only storage
```

depending on assurance requirements.

# 40. Recovery Checkpoints

Recovery can emit explicit checkpoints:

```text
RecoveryStarted
StateLoaded
EventsReplayed
AgentsReconciled
LeasesValidated
RecoveryCompleted
```

This makes recovery observable and auditable.

# 41. Reconciliation

Recovery reconstructs internal state.

Reconciliation compares that state against external reality.

```text
reconstruct
    ↓
compare
    ↓
identify divergence
    ↓
repair/fence
```

# 42. Reconciliation Is Not Replay

Replay answers:

> What did our durable history say?

Reconciliation answers:

> Does the external system still match what our history says?

Both are required.

# 43. Agent Reconciliation

The controller may query an Agent for:

```text
active commands
active executions
resource ownership
process identities
incarnation
```

and compare them against durable state.

# 44. Reconciliation Outcomes

Possible outcomes:

```text
MATCH
MISSING
EXTRA
CONFLICT
UNKNOWN
```

# 45. MATCH

Internal and external state agree.

No correction required.

# 46. MISSING

NROS believes an execution exists, but the Agent reports no corresponding execution.

Possible causes:

```text
completed during disconnect
crashed
manually terminated
state lost
```

Additional evidence may be required.

# 47. EXTRA

Agent reports an execution NROS does not know about.

This is dangerous.

Possible response:

```text
fence
quarantine
inspect
recover
```

depending on authority and safety policy.

# 48. CONFLICT

Both sides report the execution but disagree.

Example:

```text
Controller: RUNNING
Agent: EXITED
```

The system must not arbitrarily choose one without applying the defined evidence hierarchy.

# 49. Unknown Reconciliation

If neither side has sufficient evidence:

```text
UNKNOWN
```

preserve uncertainty and escalate according to policy.

# 50. Resource Reconciliation

Resources can drift too.

Example:

```text
Controller:
CPU allocation = 4

Agent:
CPU allocation = 6
```

The reconciliation layer should identify the discrepancy.

# 51. Lease Reconciliation

A lease should be checked for:

```text
owner
epoch
expiration
resource
execution
```

Stale leases must be fenced or released according to policy.

# 52. Recovery Ordering

A safe recovery sequence is generally:

```text
1. Establish local process integrity.
2. Load trusted configuration.
3. Load durable state.
4. Validate storage integrity.
5. Reconstruct state.
6. Establish new authority epoch if required.
7. Fence stale owners.
8. Reconnect Agents.
9. Reconcile executions.
10. Reconcile resources.
11. Rebuild derived state.
12. Enable scheduling.
```

# 53. Scheduling Must Wait

The scheduler should not immediately dispatch new Work while recovery is still discovering stale state.

Otherwise:

```text
unknown old execution
+
new execution
=
duplicate execution
```

could occur.

# 54. Recovery Gate

Use an explicit state:

```text
STARTING
   ↓
RECOVERING
   ↓
RECONCILING
   ↓
READY
```

Only `READY` should permit normal scheduling unless a special degraded mode is explicitly supported.

# 55. Degraded Operation

Some failures may permit partial operation.

Example:

```text
artifact store unavailable
```

might allow:

```text
metadata inspection
```

while preventing:

```text
new artifact-producing executions
```

# 56. Capability-Based Degradation

Instead of one global:

```text
DOWN
```

represent subsystem availability:

```text
API = READY
READS = READY
SCHEDULING = BLOCKED
ARTIFACTS = DEGRADED
RECOVERY = READY
```

# 57. Backpressure

When downstream systems cannot keep up:

```text
producer
   ↓
queue
   ↓
consumer
```

the producer must eventually receive backpressure.

Otherwise memory or storage exhaustion becomes the failure mechanism.

# 58. Queue Limits

Queues require explicit limits:

```text
maximum entries
maximum bytes
maximum age
maximum concurrency
```

# 59. Backpressure Policy

When capacity is exhausted, NROS may:

```text
reject
delay
shed low-priority work
pause scheduling
```

according to policy.

# 60. Priority

Priority should not bypass safety constraints.

A high-priority Work still requires:

```text
authorization
capacity
resource validity
dependency validity
```

# 61. Starvation Prevention

Priority scheduling can starve low-priority Work.

NROS may use:

```text
aging
fair-share
quotas
weighted scheduling
```

to prevent indefinite starvation.

# 62. Retry Storm Prevention

If a dependency fails, unlimited retries can amplify the outage.

Use:

```text
bounded attempts
backoff
jitter
circuit breakers
```

where appropriate.

# 63. Backoff

Retry timing should increase according to a controlled policy:

```text
attempt 1 → short delay
attempt 2 → longer delay
attempt 3 → longer delay
...
```

with a maximum bound.

# 64. Jitter

When many executions fail simultaneously, deterministic retry timing can synchronize them.

Jitter prevents:

```text
retry storm
```

by distributing retry attempts.

# 65. Circuit Breaking

If an external dependency is consistently failing:

```text
healthy
   ↓
degraded
   ↓
open
```

NROS can temporarily stop issuing operations to that dependency.

# 66. Circuit Recovery

After a cooldown:

```text
OPEN
 ↓
HALF_OPEN
 ↓
test
 ↓
CLOSED
```

if the dependency recovers.

# 67. Failure Domains

A failure in one Agent should not automatically stop unrelated Agents.

Likewise:

```text
tenant A
```

failure should not necessarily halt:

```text
tenant B
```

unless a shared dependency makes that unavoidable.

# 68. Bulkheads

Use isolation boundaries around:

```text
tenants
Agents
queues
resource pools
external dependencies
```

where operationally justified.

# 69. Resource Exhaustion

Memory exhaustion is especially dangerous because it can terminate the control plane itself.

Protect against:

```text
unbounded event buffers
unbounded queues
unbounded logs
unbounded metadata
unbounded concurrent requests
```

# 70. Graceful Shutdown

A controlled shutdown should:

```text
stop admission
   ↓
stop new scheduling
   ↓
persist necessary state
   ↓
flush durable events
   ↓
finish/release safe operations
   ↓
close resources
```

# 71. Shutdown vs Crash

A graceful shutdown may provide guarantees that a crash cannot.

Therefore the system should never assume graceful shutdown occurred.

Crash recovery remains mandatory.

# 72. Draining

A node entering shutdown should advertise:

```text
DRAINING
```

before refusing new assignments.

# 73. Active Executions During Shutdown

Policy must define whether active executions:

```text
continue
pause
checkpoint
cancel
migrate
```

The default should not be accidental.

# 74. Data Loss Policy

Every subsystem should define acceptable loss:

```text
telemetry
logs
domain events
audit
execution state
artifacts
checkpoints
```

For example:

```text
telemetry may be sampled
```

while:

```text
authoritative execution state must not be silently lost
```

# 75. Durability Classes

A useful classification:

```text
VOLATILE
RECONSTRUCTABLE
DURABLE
AUTHORITATIVE
IMMUTABLE/AUDIT
```

Each data category should have one explicit class.

# 76. Volatile Data

Examples:

```text
in-memory caches
temporary metrics aggregation
connection state
```

Loss is acceptable if reconstructable.

# 77. Reconstructable Data

Examples:

```text
derived read models
scheduler caches
diagnostic indexes
```

These can be rebuilt from authoritative sources.

# 78. Durable Data

Examples:

```text
Work state
Execution state
commands
outbox records
domain events
```

These must survive expected failures.

# 79. Authoritative Data

The system should explicitly identify the sources of truth.

For example:

```text
domain event/state store
```

rather than:

```text
API cache
```

# 80. Recovery Authority

Only authoritative data should determine final recovered state.

Caches and stale telemetry may provide hints but should not override authoritative state.

# 81. Disaster Recovery

For larger failures, NROS should support recovery from:

```text
backup
replica
snapshot
event history
```

with documented recovery objectives.

# 82. RPO

Recovery Point Objective answers:

> How much authoritative data can be lost?

Example:

```text
RPO = 0
```

means no committed authoritative state may be lost within the defined failure model.

# 83. RTO

Recovery Time Objective answers:

> How long may recovery take before the system must return to an operational state?

RTO should be defined per deployment class.

# 84. Recovery Classes

Possible deployment targets:

```text
development
single-node
production
high-availability
mission-critical
```

Each can impose different RPO/RTO guarantees.

# 85. Replication

High-availability deployments may replicate:

```text
authoritative state
event log
metadata
```

The replication protocol must define:

```text
consistency
leader election
failover
fencing
```

# 86. Leader Failover

When a new controller becomes leader:

```text
old epoch
   ↓
fence
   ↓
new epoch
   ↓
new leader
```

The new leader must not reuse the old authority epoch.

# 87. Leader Election

Leader election must avoid two leaders simultaneously believing they are authoritative.

Therefore:

```text
leader identity
+
epoch
+
fencing
```

must be treated as one security/reliability mechanism.

# 88. Split-Brain Recovery

If two controllers were active due to a partition, recovery must determine:

```text
which authority is valid
```

and fence the stale authority.

This cannot safely be solved by simply comparing wall-clock timestamps.

# 89. External Side Effects

The hardest reliability problem is usually:

```text
NROS
  ↕
external system
```

where NROS cannot atomically commit together with the external side effect.

# 90. Side-Effect Uncertainty

Example:

```text
NROS sends command
    ↓
external system executes
    ↓
network fails
    ↓
NROS receives no response
```

The correct state may be:

```text
UNKNOWN
```

until reconciliation proves otherwise.

# 91. Idempotent External Operations

Where possible, external commands should carry an idempotency key:

```text
operation_id
```

so retries do not create duplicate side effects.

# 92. Reconciliation API

External integrations should provide a queryable state whenever possible:

```text
submit(operation_id)
query(operation_id)
```

rather than only:

```text
fire_and_forget()
```

# 93. Compensation

If an operation cannot be rolled back atomically, NROS may use a compensating action.

Example:

```text
Action A
   ↓
Action B fails
   ↓
Compensating Action A'
```

Compensation is not identical to rollback and must be modeled explicitly.

# 94. Compensation Evidence

A compensated operation should preserve:

```text
original action
failure
compensation
final state
```

rather than hiding the failed action.

# 95. Recovery Does Not Rewrite History

If:

```text
Attempt A1
```

failed and:

```text
Attempt A2
```

succeeded, history remains:

```text
A1 = FAILED
A2 = SUCCEEDED
```

Recovery should not rewrite A1 into success merely because the overall Work eventually succeeded.

# 96. Final Work State

Work-level state can represent aggregate outcome:

```text
Work W42
    A1 FAILED
    A2 SUCCEEDED

Work = SUCCEEDED
```

while preserving the complete attempt history.

# 97. Recovery Idempotency

Running recovery twice should not produce different final states when no external conditions changed.

Conceptually:

```text
recover(S)
    →
S'

recover(S')
    →
S'
```

This is a crucial property.

# 98. Reconciliation Idempotency

Likewise:

```text
reconcile(S)
    →
S'
```

and repeating reconciliation should produce:

```text
reconcile(S')
    →
S'
```

unless external reality changed.

# 99. Recovery Invariants

```text
1. Process failure does not automatically imply operation failure.

2. Network failure does not automatically imply remote execution failure.

3. Unknown state is preserved when evidence is insufficient.

4. Durable state defines authoritative internal truth.

5. Volatile state is reconstructable or explicitly disposable.

6. Outbox entries survive delivery failure.

7. Commands have stable identities.

8. Duplicate commands do not create unintended duplicate side effects.

9. Stale authority is fenced.

10. Agent incarnations are distinct.

11. Leases expire.

12. Expired authority cannot silently resurrect.

13. Scheduler restart cannot double-schedule existing Work.

14. Recovery precedes normal scheduling.

15. Reconciliation compares internal state with external reality.

16. Conflicts are explicit.

17. Recovery never silently rewrites historical events.

18. Retry policies are bounded.

19. Backoff prevents retry amplification.

20. Resource exhaustion is bounded.

21. Shutdown and crash have separate semantics.

22. Critical state has explicit durability guarantees.

23. Snapshots are integrity-verified.

24. Event history is protected against silent corruption.

25. Recovery operations are observable.

26. Recovery operations are auditable where security-sensitive.

27. RPO and RTO are deployment-specific and explicit.

28. Failover changes authority epoch.

29. Split-brain requires fencing.

30. External side-effect uncertainty is not converted into fabricated certainty.

31. Compensation is explicitly represented.

32. Recovery is idempotent.

33. Reconciliation is idempotent.

34. Independent failure domains remain isolated where possible.

35. A recovered system must return through an explicit readiness gate.
```

# 100. Canonical Failure-Recovery Flow

The complete model becomes:

```text
                     FAILURE
                        │
                        ▼
                ┌───────────────┐
                │ Detect        │
                └───────┬───────┘
                        ▼
                ┌───────────────┐
                │ Classify      │
                └───────┬───────┘
                        ▼
                ┌───────────────┐
                │ Fence unsafe  │
                │ authority     │
                └───────┬───────┘
                        ▼
                ┌───────────────┐
                │ Preserve      │
                │ durable truth │
                └───────┬───────┘
                        ▼
                ┌───────────────┐
                │ Recover       │
                └───────┬───────┘
                        ▼
                ┌───────────────┐
                │ Reconstruct   │
                └───────┬───────┘
                        ▼
                ┌───────────────┐
                │ Reconcile     │
                └───────┬───────┘
                        ▼
                ┌───────────────┐
                │ Validate      │
                └───────┬───────┘
                        ▼
                ┌───────────────┐
                │ READY         │
                └───────────────┘
```

# 101. Reliability Principle

The deepest reliability rule for NROS is:

> **Never confuse absence of evidence with evidence of failure, and never allow uncertain authority to continue operating without fencing.**

That produces the two complementary safety rules:

```text
UNKNOWN when truth is unavailable
FENCED when authority is uncertain
```

Together they prevent two of the most dangerous distributed-runtime errors:

```text
inventing state
```

and:

```text
allowing stale actors to mutate state
```

# Part CXI — Determinism, Scheduling Semantics & Runtime Correctness

The next layer should formalize the actual runtime execution model:

```text
Work
→ eligibility
→ scheduling
→ admission
→ placement
→ resource reservation
→ dispatch
→ execution
→ completion
→ retry
→ checkpoint
→ finalization
```

with particular attention to:

```text
determinism
fairness
priority
preemption
dependencies
resource constraints
ordering
concurrency
idempotency
```

The central question becomes:

> **Given the same authoritative state and policy inputs, can NROS explain and, where required, reproduce why a particular Work was scheduled, where it ran, and how its execution state evolved?**
