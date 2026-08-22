# Part XLV — Execution Semantics, Work Lifecycle, Cancellation & Failure Propagation

> **Series:** NROS Architecture Series  
> **Part:** XLV  
> **Role:** Execution semantics, work-unit state machines, task and agent lifecycle, yielding, checkpoints, cancellation, failure propagation, supervision interaction, transactional boundaries, and terminal-state semantics  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XLIV defined how NROS selects work for execution. Part XLV defines what it means for that selected work to execute and transition through its lifecycle.

The central rule is:

> **NROS treats execution as an explicit state transition system: scheduling a work unit does not start it, starting it does not complete it, completion does not imply commit, and cancellation or failure must produce explicit, recoverable terminal or intermediate states.**

## 2. Fundamental Distinctions

```text
scheduled
  ≠
started
  ≠
running
  ≠
yielded
  ≠
completed
  ≠
committed
  ≠
recovered
```

## 3. Canonical Work Lifecycle

```text
Created
 ↓
Admitted
 ↓
Scheduled
 ↓
Started
 ↓
Running
 ├─→ Yielded ─→ Running
 ├─→ Checkpointed ─→ Running
 ├─→ CancelRequested ─→ Cancelling
 ├─→ FailureDetected ─→ Failing
 └─→ CompletionReady ─→ Completing
                         ↓
                    Completed
                         ↓
                      Commit
```

The exact transitions are policy-controlled.

## 4. Work Unit Identity

Every execution unit should have stable identity:

```text
work_id
task_id
execution_id
attempt_id
workflow_id
principal
policy_version
epoch
```

`work_id` identifies logical work while `attempt_id` distinguishes execution attempts.

## 5. State Machine

A work-unit state machine should reject invalid transitions.

```text
Created → Running
```

is invalid when admission and scheduling are mandatory.

Likewise:

```text
Cancelled → Running
```

is invalid unless an explicit resurrection/retry protocol creates a new attempt.

## 6. State Ownership

The component responsible for each transition must be explicit:

```text
Admission controller
Scheduler
Executor
Supervisor
Persistence layer
```

No component should silently mutate another component's authoritative state.

## 7. Start

Start establishes that an execution attempt has acquired an execution context.

```text
Scheduled
 ↓
Start
 ↓
Running
```

Start should establish relevant identity, authority, resources, and tracing context.

## 8. Execution Context

An execution context can include:

```text
principal
capabilities
scheduler policy
resource allocation
deadline
cancellation token
storage context
trace context
configuration epoch
```

## 9. Authority Preservation

Execution must preserve the authority under which work was admitted.

```text
Authorized(W, Context A)
 ↓
Execute(W, Context B)
```

is valid only if B remains within the authority explicitly granted to W.

## 10. Resource Acquisition

Execution may require resources before entering Running:

```text
Scheduled
 ↓
Acquire resources
 ↓
Start
```

Failure to acquire resources must produce an explicit outcome rather than indefinite hidden waiting.

## 11. Resource Lifetime

Resources should be tied to execution lifetime where possible:

```text
Start
 ↓ acquire
Running
 ↓
Complete / Cancel / Fail
 ↓ release
```

## 12. Running

Running means the execution attempt currently owns an execution opportunity and may perform its permitted work.

It does not imply progress is guaranteed.

## 13. Progress

Long-running work should expose progress where required:

```text
started
 ↓
progress
 ↓
progress
 ↓
complete
```

Progress must not be confused with successful completion.

## 14. Yield

Yield voluntarily returns execution opportunity to the scheduler:

```text
Running
 ↓ yield
Runnable
```

Yield should preserve logical work identity.

## 15. Cooperative Yielding

Cooperative execution can yield at safe boundaries:

```text
await
checkpoint
poll
explicit yield
```

## 16. Checkpoint

A checkpoint records sufficient state to resume or recover according to the workload contract.

```text
Running
 ↓
Checkpoint
 ↓
Running
```

Checkpoint persistence follows Part XLIII durability semantics.

## 17. Checkpoint Identity

A checkpoint should identify:

```text
checkpoint_id
work_id
attempt_id
execution epoch
policy version
state version
```

## 18. Checkpoint Completeness

A checkpoint is useful only if it captures all state required by the declared recovery model.

A pointer to volatile state is not itself a recoverable checkpoint.

## 19. Cancellation Request

Cancellation begins with an explicit request:

```text
Running
 ↓
cancel(work_id)
 ↓
CancelRequested
```

The request is not equivalent to immediate termination.

## 20. Cancellation Observation

Cooperative tasks observe cancellation at defined safe points.

```text
CancelRequested
 ↓
observe
 ↓
cleanup
 ↓
Cancelled
```

## 21. Cancellation Deadline

Cancellation itself may require a deadline:

```text
CancelRequested
 ↓
cleanup window
 ↓
Cancelled
```

Failure to stop within the window requires explicit escalation semantics.

## 22. Forced Cancellation

Forced termination may exist for containment, but it requires explicit guarantees about resource and state cleanup.

It must not be assumed equivalent to cooperative cancellation.

## 23. Cancellation Idempotency

Repeated cancellation requests should converge:

```text
cancel(W)
retry cancel(W)
```

without producing contradictory lifecycle state.

## 24. Failure Detection

Execution failure may arise from:

```text
application error
panic/fault
resource exhaustion
timeout
security denial
storage failure
communication failure
worker loss
```

## 25. Failure State

Failure should become an explicit lifecycle state before final policy resolution:

```text
Running
 ↓
FailureDetected
 ↓
Failing
 ↓
Retry / Recover / Abort
```

## 26. Failure Classification

Failures should be classified as:

```text
transient
permanent
unknown
policy-induced
operator-induced
```

Classification determines whether retry or recovery is permitted.

## 27. Failure Propagation

A child failure may propagate to a parent according to policy:

```text
Child failure
 ↓
Supervisor
 ↓
Parent policy
```

Propagation must not be assumed universal.

## 28. Failure Domains

Failure propagation should respect boundaries:

```text
task
workflow
agent
service
node
cluster
```

A task failure does not automatically imply cluster failure.

## 29. Supervisory Relationship

A supervisor can own lifecycle decisions for child work:

```text
Supervisor
 ├─ Child A
 ├─ Child B
 └─ Child C
```

Supervisor semantics should define restart, stop, isolate, or escalate behavior.

## 30. Restart Policy

Restart policies may include:

```text
never
once
bounded
until deadline
until budget exhausted
operator-directed
```

## 31. Restart Identity

A restart should create a new execution attempt while preserving logical work identity:

```text
work_id = W
attempt_id = 1
attempt_id = 2
```

## 32. Restart Safety

Restart is safe only when state and side effects are controlled.

External effects require idempotency, fencing, reconciliation, or transactional coordination.

## 33. Retry vs Restart

```text
retry
 ≠
restart
```

A retry may repeat one operation; a restart may reconstruct a larger execution context.

## 34. Recovery

Recovery restores execution from a valid state:

```text
Failure
 ↓
Recovery point
 ↓
Validate
 ↓
Resume / Restart
```

Part XLIII governs persistent recovery points.

## 35. Recovery Validation

Recovered state must validate:

```text
integrity
schema
configuration epoch
authority
resource compatibility
```

## 36. Stale Execution Context

A recovered execution must not continue under stale configuration or authority when the contract requires fencing.

```text
Execution epoch < Current epoch
        ↓
        stop
```

## 37. Completion

Completion means the execution body reached its defined successful terminal condition.

```text
Running
 ↓
Completing
 ↓
Completed
```

Completion may still require durable commit.

## 38. Commit

Commit establishes the authoritative persistent outcome when required:

```text
Completed
 ↓
Commit
 ↓
Committed
```

Thus:

```text
Completed ≠ Committed
```

## 39. Commit Failure

If execution completed but commit failed:

```text
Completed
 ↓
Commit failure
 ↓
CommitPending / RecoveryRequired
```

The system must not falsely report the logical operation as durably committed.

## 40. Exactly-Once Effects

Exactly-once behavior is an end-to-end property.

Execution alone cannot establish it.

Required mechanisms may include:

```text
operation identity
idempotency
transaction
outbox/inbox
fencing
reconciliation
```

## 41. External Side Effects

The execution model must account for:

```text
side effect succeeded
local state failed
```

and:

```text
local state committed
side effect failed
```

These are ambiguous outcomes requiring reconciliation.

## 42. Transaction Boundary

Execution may define transactional boundaries around state transitions:

```text
prepare
 ↓
execute
 ↓
commit
```

The boundary must be explicit.

## 43. Atomic Work Unit

If a work unit promises atomic outcome:

```text
success → all required state committed
failure → no partially visible outcome
```

The implementation must provide evidence for that guarantee.

## 44. Partial Completion

Some work naturally produces partial progress.

Such workloads should explicitly model:

```text
partial state
checkpoint
resume point
```

rather than falsely representing partial work as complete.

## 45. Compensation

When rollback is impossible, compensation may reverse or neutralize an external effect:

```text
Effect A
 ↓
Failure
 ↓
Compensating action
```

Compensation is not equivalent to atomic rollback.

## 46. Terminal States

Canonical terminal states may include:

```text
Committed
Failed
Cancelled
Expired
Rejected
Aborted
```

The exact set depends on workload policy.

## 47. Terminal-State Immutability

Once a logical work unit reaches a terminal state, later events must not silently rewrite its history.

Corrections should be represented as new events or reconciliation actions.

## 48. Event Ordering

Lifecycle events should carry ordering metadata:

```text
event_id
sequence
execution epoch
timestamp/logical time
```

Part XXXVI governs temporal semantics.

## 49. Duplicate Events

Lifecycle processing must tolerate duplicate delivery where the communication model permits it.

```text
Event E
retry E
```

must not produce duplicate terminal transitions.

## 50. Out-of-Order Events

Events arriving out of order require sequence, epoch, causal, or state-validation rules.

## 51. Invalid Transition

Invalid lifecycle transitions should be rejected and observable:

```text
Cancelled → Running
```

is rejected unless an explicit new attempt is created.

## 52. State Machine Persistence

For recoverable work, lifecycle state may need persistence:

```text
State transition
 ↓
durable record
 ↓
recoverable lifecycle
```

Part XLIII defines durability guarantees.

## 53. State Machine Recovery

After restart:

```text
Persistent state
 ↓
reconstruct lifecycle
 ↓
validate leases/epochs
 ↓
resume or terminate
```

## 54. Lease Expiry

A work lease that expires should cause ownership to become invalid unless renewed.

```text
Lease expired
 ↓
Old worker loses authority
```

## 55. Worker Loss

When a worker disappears:

```text
Worker loss
 ↓
lease timeout
 ↓
recover / retry / fail
```

The system must avoid indefinite orphaned work.

## 56. Orphan Detection

Orphan detection should use explicit evidence such as lease state, worker epoch, or heartbeat policy rather than assuming absence of a response proves failure.

## 57. Heartbeats

Heartbeats can indicate liveness:

```text
worker → heartbeat
```

but heartbeat presence does not prove correct application-level progress.

## 58. Progress vs Liveness

```text
liveness
 ≠
progress
```

A worker can remain alive while making no useful progress.

## 59. Hung Work

Hung work can be detected using:

```text
progress deadlines
heartbeats
execution watchdogs
resource observations
```

Detection should lead to explicit policy, not arbitrary termination.

## 60. Watchdog

A watchdog may monitor execution without becoming the execution authority itself.

```text
Executor
   ↑
Watchdog
```

Its actions remain policy-controlled.

## 61. Backpressure During Execution

If downstream systems become saturated, running work may need to:

```text
pause
yield
checkpoint
slow down
cancel
```

This connects execution to Part XLII communication semantics.

## 62. Resource Revocation

Execution may lose a resource due to:

```text
quota change
security revocation
resource failure
operator action
```

The execution model must define whether work pauses, degrades, retries, or fails.

## 63. Security Revocation

If authority is revoked during execution:

```text
Running
 ↓
Authority revoked
 ↓
Stop / contain
```

Continued execution requires explicit policy.

## 64. Configuration Change

A running work unit may encounter a new configuration epoch:

```text
Config epoch 4
 ↓
Config epoch 5
```

Policy must define whether execution continues under the old snapshot, transitions, or terminates.

## 65. Policy Snapshot

A work unit should record the policy/configuration version under which its behavior is evaluated when reproducibility matters.

## 66. Deterministic Execution

Deterministic execution requires controlling relevant nondeterminism:

```text
input order
randomness
clock
message order
resource outcomes
scheduler decisions
```

Part XLIV defines scheduling determinism.

## 67. Replay

Replay should reconstruct enough execution context to explain or reproduce behavior.

```text
execution record
 + inputs
 + policy
 + scheduler evidence
 → replay context
```

## 68. Execution Trace

A useful trace can include:

```text
created
admitted
scheduled
started
yielded
checkpointed
resumed
cancelled/failed/completed
committed
```

## 69. Observability

Execution metrics may include:

```text
start latency
run time
CPU time
wait time
yield count
checkpoint latency
retry count
failure class
completion latency
commit latency
```

Part XL defines observability semantics.

## 70. Security Evidence

Execution records should preserve enough identity context to answer:

```text
Who initiated this work?
Under which authority?
Which policy was active?
Which worker executed it?
Which resources were used?
```

## 71. Resource Accounting

Execution should attribute relevant resource consumption:

```text
CPU
memory
I/O
network
storage
GPU
```

Part XXXVII defines resource accounting principles.

## 72. Nested Work

An execution unit may create child work:

```text
Parent
 ├─ Child A
 └─ Child B
```

Parent-child relationships should be explicit for cancellation, authorization, accounting, and failure propagation.

## 73. Detached Work

Detached work must have an explicit owner or lifecycle authority.

Otherwise it risks becoming orphaned work.

## 74. Structured Concurrency

Where applicable, child work should remain within the parent's lifecycle scope:

```text
Parent scope
 ├─ Child A
 └─ Child B
```

Parent termination can then define child termination behavior.

## 75. Supervisor Restart

A supervisor restart may restart children according to policy:

```text
Supervisor failure
 ↓
Supervisor restart
 ↓
Child reconciliation
```

Recovered children must validate ownership before resuming.

## 76. Failure Escalation

Failure may escalate through defined levels:

```text
Task
 ↓
Workflow
 ↓
Agent
 ↓
Service
 ↓
Node
```

Escalation must be bounded and policy-driven.

## 77. Failure Containment

Containment can prevent one failure from cascading:

```text
Faulty task
 ↓
quarantine
 ↓
healthy tasks continue
```

## 78. Circuit Breaking

Repeated execution failures against a dependency may open a circuit:

```text
Closed
 ↓ failures
Open
 ↓
Half-open
 ↓
Closed / Open
```

## 79. Execution Admission After Failure

A failing workload may require a cooldown or explicit operator approval before new attempts are admitted.

## 80. Deadline + Failure

Failure recovery must respect the remaining deadline:

```text
Failure
 ↓
Recovery attempt
 ↓
remaining deadline
```

An expired deadline should prevent new work unless late recovery is explicitly allowed.

## 81. Cancellation + Failure Race

Cancellation and failure can occur concurrently:

```text
CancelRequested
      ╲
       race
      ╱
FailureDetected
```

The state machine needs deterministic precedence or an explicit combined outcome.

## 82. Completion + Cancellation Race

Similarly:

```text
CompletionReady
      ╲
       race
      ╱
CancelRequested
```

The system must define the linearization point that determines the authoritative outcome.

## 83. Commit + Cancellation Race

Once commit becomes authoritative, cancellation must not falsely report that committed state was undone unless compensation actually occurred.

## 84. Linearization Point

Lifecycle operations requiring a single outcome should define a linearization point:

```text
request
 ↓
linearization point
 ↓
observable state
```

## 85. Idempotent Lifecycle Commands

Commands such as:

```text
cancel
retry
resume
checkpoint
```

should be idempotent or explicitly return a conflict when repeated.

## 86. Recovery and Duplicate Effects

After restart, the system may not know whether an external effect occurred.

It must use operation identity and reconciliation rather than blindly repeating unsafe effects.

## 87. Terminal Evidence

A terminal result should identify:

```text
work_id
attempt_id
terminal_state
reason
policy_version
execution_epoch
commit status
```

## 88. Failure Evidence

Failure records should preserve the original failure classification and relevant causal context without silently replacing history.

## 89. Causal Chain

Execution failures should be traceable:

```text
Parent
 ↓
Child
 ↓
Dependency
 ↓
Root cause evidence
```

Causality should be represented rather than inferred only from timestamps.

## 90. Formal Lifecycle Invariant

```text
Terminal(W)
    ⇒
¬Transition(W, Running)
```

unless a new execution attempt is explicitly created.

## 91. Formal Authorization Invariant

```text
Execute(W)
    ⇒
Authorized(Context(W), W)
```

## 92. Formal Cancellation Invariant

```text
Cancelled(W)
    ⇒
NoFutureExecutionAttempt(W, SameAttempt)
```

## 93. Formal Ownership Invariant

```text
ValidLease(W, WorkerEpoch)
    ⇒
WorkerMayExecute(W)
```

and stale epochs are rejected.

## 94. Formal Commit Invariant

```text
Committed(W)
    ⇒
DurableCommit(W, DeclaredBoundary)
```

when the workload contract requires durable commit.

## 95. Formal Recovery Invariant

```text
Resume(W)
    ⇒
ValidRecoveryPoint(W)
 ∧
CompatiblePolicy(W)
 ∧
ValidAuthority(W)
```

## 96. Verification Matrix

| Property | Verification question |
|---|---|
| Lifecycle | Are valid and invalid transitions explicit? |
| Identity | Are logical work and attempts distinguishable? |
| Authority | Is execution bounded by the admitted authority? |
| Resources | Are resource acquisition and release observable? |
| Yield | Can work safely return execution opportunity? |
| Checkpoint | Is checkpoint state actually recoverable? |
| Cancellation | Is cancellation bounded and idempotent? |
| Failure | Are failure classes explicit? |
| Propagation | Are parent/child failure rules defined? |
| Restart | Are retries/restarts distinguished and bounded? |
| Recovery | Can execution resume from a validated state? |
| Ownership | Can stale workers be fenced? |
| Completion | Is completion distinct from commit? |
| Commit | Is durable commit explicitly evidenced? |
| Side effects | Can ambiguous external outcomes be reconciled? |
| Races | Are cancellation/failure/completion races deterministic? |
| Replay | Is enough context preserved for reconstruction? |
| Observability | Can execution history be reconstructed? |
| Security | Can authority revocation stop execution safely? |
| Failure containment | Can local faults avoid cascading? |

## 97. What Part XLV Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a complete production work-state machine;
- universal structured concurrency;
- formally verified cancellation;
- production supervision and restart orchestration;
- universal deterministic replay;
- complete exactly-once external side effects;
- formally verified failure propagation;
- production checkpoint/recovery for every workload.

Those require implementation-specific evidence.

## 98. Transition to Part XLVI

Part XLV establishes the execution lifecycle.

Part XLVI should define **supervision, fault domains, resilience, restart orchestration, isolation, circuit breaking, recovery coordination, and system-level failure containment**.

```text
Part XLIV
Scheduling + concurrency + fairness + deadlines + determinism
        ↓
Part XLV
Execution semantics + work lifecycle + cancellation + failure propagation
        ↓
Part XLVI
Supervision + resilience + fault domains + recovery orchestration
```

## Canonical rule

> **NROS execution is a state machine, not an implicit function call: every start, yield, checkpoint, cancellation, failure, recovery, completion, and commit has an explicit lifecycle meaning, ownership boundary, authority context, and observable outcome.**
