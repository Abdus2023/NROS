# Part LIV — Workload, Execution, Supervision, Lifecycle & Failure Semantics

> **Series:** NROS Architecture Series  
> **Part:** LIV  
> **Role:** Workload identity, execution attempts, lifecycle, supervision, cancellation, checkpointing, retry, failure, completion, and side-effect semantics  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LIII established resource management and scheduling. Part LIV defines what happens after a workload is admitted and allocated: how execution is identified, supervised, retried, cancelled, checkpointed, completed, and reconciled.

The central rule is:

> **A workload is durable intent; an execution attempt is one concrete realization of that intent; a process is only one possible implementation mechanism; a result is the observed outcome.**

## 2. Fundamental Distinctions

```text
Workload
   ≠
Execution Attempt
   ≠
Process
   ≠
Result
```

A workload may have multiple attempts while retaining one logical identity.

## 3. Workload Identity

A workload should have a stable identity:

```text
workload_id
owner
origin
policy context
resource request
lifecycle policy
```

## 4. Execution Attempt Identity

Each concrete attempt should have its own identity:

```text
attempt_id
attempt_number
workload_id
start context
resource allocation
executor identity
```

## 5. Process Identity

A process, thread, container, VM, or remote worker may implement an attempt.

Process identity must not be confused with workload identity.

## 6. Result Identity

An execution result should identify:

```text
workload_id
attempt_id
status
outputs
failure information
completion evidence
```

## 7. Lifecycle

A canonical lifecycle is:

```text
Created
  ↓
Admitted
  ↓
Scheduled
  ↓
Allocated
  ↓
Starting
  ↓
Running
  ↓
Completing
  ↓
Completed
```

Failure and cancellation introduce additional terminal or transitional states.

## 8. Lifecycle State Authority

Only an authorized lifecycle owner may make authoritative transitions.

Observed process state alone does not automatically establish logical workload state.

## 9. State Transition Rule

Every transition should define:

```text
source state
event
guards
authority
result state
evidence
```

## 10. Starting

Starting bridges allocation and execution:

```text
Allocated
 ↓
Starting
 ↓
Running
```

Startup failure must remain distinguishable from runtime failure.

## 11. Running

Running means the execution attempt has reached the declared execution state.

It does not guarantee successful completion.

## 12. Completion

Completion should distinguish:

```text
success
failure
cancelled
preempted
expired
aborted
```

## 13. Terminal State

A terminal execution attempt must not silently return to Running.

A new attempt receives a new attempt identity.

## 14. Workload vs Attempt Retry

Retry normally creates:

```text
Workload W
 ├── Attempt 1
 ├── Attempt 2
 └── Attempt 3
```

The workload identity remains stable while attempt identities differ.

## 15. Retry Policy

Retry behavior should define:

```text
maximum attempts
retryable failures
backoff
jitter
resource limits
time budget
side-effect policy
```

## 16. Retry Classification

Failures may be:

```text
retryable
non-retryable
conditionally retryable
unknown
```

Unknown failures must not automatically imply unlimited retry.

## 17. Retry Budget

Retries consume a bounded budget to prevent retry storms.

## 18. Backoff

Backoff can be:

```text
fixed
exponential
policy-defined
```

The selected semantics must be explicit.

## 19. Retry Storm Prevention

Repeated failures must not create unbounded resource consumption or admission pressure.

## 20. Supervision

A supervisor monitors an execution attempt or workload according to policy.

```text
Supervisor
    ↓
Observe
    ↓
Evaluate
    ↓
Act
```

## 21. Supervision Actions

Possible actions include:

```text
continue
notify
restart
cancel
preempt
quarantine
escalate
```

## 22. Supervisor Authority

Supervision actions require explicit authority and must obey security and resource policy.

## 23. Health

Health is a multidimensional concept:

```text
process liveness
progress
resource health
dependency health
policy compliance
```

A live process is not necessarily a healthy workload.

## 24. Progress

Progress may be represented using:

```text
heartbeats
sequence numbers
checkpoints
progress counters
phase transitions
```

## 25. Stalled Execution

A workload may be alive but make no meaningful progress.

Supervision may classify this as:

```text
healthy
slow
stalled
failed
```

according to policy.

## 26. Heartbeats

Heartbeats provide liveness evidence but do not prove correctness.

```text
heartbeat
   ≠
progress
   ≠
success
```

## 27. Timeout

Timeouts should distinguish:

```text
startup timeout
execution timeout
idle timeout
heartbeat timeout
deadline expiration
```

## 28. Cancellation

Cancellation is an explicit lifecycle request:

```text
Running
 ↓ cancel requested
Stopping
 ↓
Cancelled
```

## 29. Cooperative Cancellation

Workloads should receive an opportunity to release resources and finalize state.

## 30. Forced Cancellation

If cooperation exceeds its grace period, policy may permit forced termination.

## 31. Cancellation Authority

Cancellation must be authorized and scoped.

## 32. Preemption vs Cancellation

```text
Cancellation
 → workload-directed termination

Preemption
 → resource-directed reclamation
```

They may use similar mechanisms but have different semantics.

## 33. Checkpointing

A checkpoint captures sufficient state to resume or recover according to workload semantics.

## 34. Checkpoint Identity

A checkpoint should identify:

```text
workload_id
attempt_id
checkpoint_id
logical position
state revision
integrity information
```

## 35. Checkpoint Correctness

A checkpoint must not be treated as valid merely because it was written.

Its integrity and compatibility must be established.

## 36. Resume

Resume may create a new attempt:

```text
Attempt 1
 ↓ checkpoint
Attempt 1 terminates
 ↓
Attempt 2 resumes from checkpoint
```

## 37. Checkpoint Compatibility

A checkpoint may depend on:

```text
software revision
schema version
configuration revision
resource model
external dependencies
```

## 38. Stale Checkpoint

A checkpoint incompatible with current authority or schema must not be silently resumed.

## 39. Execution Environment

An attempt should have an explicit execution context:

```text
software revision
configuration revision
policy revision
resource allocation
identity
environment
```

## 40. Isolation

Execution may be isolated using:

```text
process
container
sandbox
VM
remote worker
```

The mechanism is implementation-dependent; the security contract is not.

## 41. Side Effects

Execution can produce external side effects:

```text
files
network requests
messages
database mutations
hardware actions
```

## 42. Side-Effect Classification

Effects may be:

```text
reversible
idempotent
non-idempotent
compensatable
irreversible
```

Retry policy must consider this classification.

## 43. At-Least-Once Execution

Retries may produce repeated execution.

Therefore:

```text
attempt count > 1
    ⇒
possible repeated side effects
```

## 44. Idempotency

For retryable operations, idempotency keys or equivalent mechanisms should prevent unintended duplication where possible.

## 45. Exactly-Once Claims

Exactly-once behavior must not be claimed merely because a scheduler issued one logical request.

It requires end-to-end evidence across execution and side-effect boundaries.

## 46. Compensation

Irreversible effects may require compensating actions rather than rollback.

```text
Effect A
 ↓ failure
Compensation A'
```

## 47. External Dependency Failure

A workload can fail because a dependency is unavailable even if the local executor is healthy.

## 48. Failure Domains

Failures should be classified by domain:

```text
workload
executor
resource
network
dependency
policy
configuration
platform
```

## 49. Failure Propagation

A local failure should not automatically become a global failure unless policy defines the dependency.

## 50. Failure Containment

Supervision should limit cascading failures through:

```text
budgets
bulkheads
rate limits
circuit breakers
quarantine
```

## 51. Quarantine

Repeatedly failing workloads or execution environments may enter quarantine to prevent repeated damage.

## 52. Restart

Restart creates a new execution attempt unless the runtime explicitly defines transparent continuation semantics.

## 53. Crash Recovery

After executor crash:

```text
Detect
 ↓
Fence stale execution
 ↓
Reconcile resources
 ↓
Recover / Retry / Fail
```

## 54. Stale Executor

An executor that loses authority must not continue making authoritative state transitions.

## 55. Execution Epoch

Execution attempts may be bound to an authority epoch:

```text
attempt epoch 9
current epoch 10
      ↓
reject stale control action
```

## 56. Resource Reconciliation

When an attempt terminates unexpectedly, allocations must be reconciled with actual resource state.

## 57. Completion Evidence

Completion should have evidence sufficient to establish:

```text
attempt identity
terminal status
resource release
outputs
side-effect status
```

## 58. Output Ownership

Outputs should have explicit ownership and lifecycle semantics.

## 59. Partial Output

A failed attempt may leave partial outputs.

These must not automatically be presented as complete successful results.

## 60. Result Validity

```text
Produced Output
    ≠
Valid Result
```

Result validity depends on workload semantics and completion status.

## 61. Artifact Integrity

Artifacts should have integrity information such as:

```text
hash
size
producer
revision
```

## 62. Artifact Provenance

Where required, outputs should identify:

```text
workload
attempt
software
configuration
input references
```

## 63. Input Provenance

Execution should preserve sufficient input identity to make the result interpretable and reproducible where required.

## 64. Determinism

Deterministic workloads may expose reproducibility metadata.

Non-deterministic execution should not be falsely represented as deterministic.

## 65. Execution Budget

Attempts may have bounded:

```text
time
CPU
memory
I/O
network
retry count
cost
```

## 66. Budget Exhaustion

Budget exhaustion is a defined terminal or policy transition, not an infrastructure mystery.

## 67. Deadline

A deadline applies to the workload according to policy and should account for queueing, execution, retries, or explicitly declared phases.

## 68. Queue Time vs Execution Time

These should remain separately measurable:

```text
wait time
startup time
execution time
recovery time
```

## 69. Lifecycle Events

Lifecycle transitions should generate correlatable events:

```text
created
admitted
scheduled
allocated
started
running
checkpointed
cancelled
failed
completed
```

## 70. Event Ordering

Event timestamps alone do not establish causality.

Use correlation IDs, sequence numbers, epochs, or other ordering mechanisms where required.

## 71. Execution Logs

Logs should be linked to:

```text
workload_id
attempt_id
execution context
```

Sensitive values must remain protected.

## 72. Execution Metrics

Useful metrics include:

```text
attempt count
runtime
queue time
restart count
checkpoint count
failure count
resource consumption
completion latency
```

## 73. Supervisor Observability

Supervisor actions should themselves be observable and auditable when privileged.

## 74. Operator Intervention

Manual intervention should become part of the workload history:

```text
actor
action
reason
scope
result
```

## 75. Manual Retry

Manual retry should not bypass normal workload and resource policy.

## 76. Priority Changes During Execution

Policy may allow priority changes, but the change must be authorized and observable.

## 77. Resource Changes During Execution

Elastic workloads may receive allocation changes through the scheduler rather than direct uncontrolled mutation.

## 78. Migration

An execution may migrate between resources if workload semantics permit it.

Migration requires:

```text
state transfer
resource transfer
authority continuity
side-effect safety
```

## 79. Migration vs Restart

```text
Migration
 → continuity-oriented transfer

Restart
 → new execution attempt
```

## 80. Shutdown

Graceful system shutdown should provide a policy for active workloads:

```text
finish
checkpoint
cancel
preempt
migrate
```

## 81. Drain Mode

A node or scheduler domain may enter drain mode:

```text
No new placement
 ↓
Existing workloads drain
 ↓
Resources become reclaimable
```

## 82. Maintenance

Maintenance can require workload evacuation before resource withdrawal.

## 83. Failure During Drain

Drain operations must remain recoverable if a workload fails during evacuation.

## 84. Supervisor Hierarchy

Supervision may be hierarchical:

```text
System Supervisor
 ↓
Node Supervisor
 ↓
Workload Supervisor
 ↓
Attempt Supervisor
```

Authority must remain non-ambiguous.

## 85. Parent-Child Workloads

A workload may create child workloads.

Parent-child lifecycle semantics must define:

```text
ownership
cancellation
failure propagation
resource accounting
```

## 86. Structured Concurrency

Where supported, child execution should remain associated with the lifecycle of its parent scope.

## 87. Orphaned Children

Children whose parent disappears must have an explicit policy:

```text
cancel
adopt
continue
quarantine
```

## 88. Failure Aggregation

Composite workloads should distinguish:

```text
child failure
partial success
aggregate failure
```

## 89. Completion Policy

A workload may complete when:

```text
all children complete
one child succeeds
threshold reached
deadline reached
policy condition satisfied
```

## 90. Terminal Result

A terminal workload result should be immutable or versioned once finalized.

## 91. Result Reconciliation

If an executor reports success but durable state indicates failure, the authoritative result requires reconciliation rather than blind acceptance.

## 92. Split-Brain Execution

Two active attempts must not both be treated as authoritative when the workload contract requires single execution authority.

## 93. Fencing

Fencing prevents stale attempts from continuing protected actions after authority changes.

## 94. Fencing + Side Effects

Fencing is especially important before retrying non-idempotent operations.

## 95. Retry Gate

Before retry:

```text
Previous attempt terminated
 ∧
Resources reconciled
 ∧
Stale authority fenced
 ∧
Retry policy permits
```

## 96. Formal Workload Invariant

```text
Workload(W)
    ⇒
StableIdentity(W)
```

## 97. Formal Attempt Invariant

```text
Attempt(A)
    ⇒
UniqueIdentity(A)
 ∧
BelongsTo(A, W)
```

## 98. Formal Retry Invariant

```text
Retry(W)
    ⇒
NewAttempt(W)
 ∧
RetryPolicyAllows(W)
```

## 99. Formal Completion Invariant

```text
Completed(A)
    ⇒
Terminal(A)
 ∧
CompletionEvidence(A)
```

## 100. Formal Resource Invariant

```text
Terminal(A)
    ⇒
Eventually Reconciled(Allocation(A))
```

## 101. Formal Fencing Invariant

```text
StaleAttempt(A)
    ⇒
Reject(ProtectedAction(A))
```

## 102. Verification Matrix

| Property | Verification question |
|---|---|
| Identity | Are workload and attempt identities distinct? |
| Lifecycle | Are transitions explicit and guarded? |
| Retry | Are retry limits and classifications defined? |
| Supervision | Are supervisor actions authorized? |
| Cancellation | Is cooperative vs forced cancellation explicit? |
| Checkpoint | Is checkpoint integrity and compatibility verified? |
| Side effects | Are retry and duplication hazards modeled? |
| Failure | Are failure domains distinguishable? |
| Fencing | Can stale attempts perform protected actions? |
| Resources | Are allocations reconciled after termination? |
| Outputs | Are partial outputs distinguished from valid results? |
| Provenance | Can results be traced to execution context? |
| Budgets | Are execution budgets bounded? |
| Shutdown | Is active-work policy explicit? |
| Children | Are parent/child lifecycle semantics defined? |
| Evidence | Can completion be independently established? |
| Split brain | Is duplicate authority prevented? |
| Recovery | Can crashed attempts be safely retried or failed? |
| Determinism | Are reproducibility claims evidence-backed? |
| Security | Are execution actions governed by identity and policy? |

## 103. What Part LIV Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production workload supervisor;
- universal checkpoint/resume support;
- exactly-once side-effect semantics;
- complete execution fencing;
- universal distributed retry coordination;
- automatic migration of every workload;
- complete parent-child workload orchestration;
- production-grade execution provenance for every artifact.

Those require implementation-specific evidence.

## 104. Transition to Part LV

Part LIV establishes workload and execution semantics.

Part LV should define **storage, state, durability, transactions, snapshots, recovery, and consistency architecture** so that workload execution can establish durable state without confusing memory, persistence, and externally observable completion.

```text
Part LIII
Resources + admission + scheduling
        ↓
Part LIV
Workloads + execution + supervision
        ↓
Part LV
State + storage + durability + recovery
```

## Canonical rule

> **NROS treats workload intent, execution attempts, process instances, and results as distinct entities; every execution lifecycle, retry, cancellation, recovery, and completion transition must preserve identity, authority, resource accounting, side-effect semantics, and evidence.**
