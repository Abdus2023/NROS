# Part XLIV — Scheduling, Concurrency, Fairness, Deadlines & Deterministic Execution

> **Series:** NROS Architecture Series  
> **Part:** XLIV  
> **Role:** Scheduling, concurrency, work distribution, priorities, fairness, admission, deadlines, preemption, execution order, and deterministic runtime behavior  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XLIII established persistent-state semantics. Part XLIV defines how NROS decides what work may execute, when it executes, where it executes, how concurrent work interacts, and which execution-order guarantees are observable.

The central rule is:

> **NROS treats scheduling as an explicit policy over admitted work: concurrency, parallelism, priority, fairness, deadlines, preemption, execution, completion, and commit are distinct concepts and must not be conflated.**

## 2. Fundamental Distinctions

```text
concurrency
  ≠
parallelism
  ≠
scheduling
  ≠
execution
  ≠
completion
  ≠
commit
```

## 3. Work Lifecycle

```text
Created
 ↓
Admitted
 ↓
Queued
 ↓
Runnable
 ↓
Selected
 ↓
Executing
 ↓
Completed / Failed / Cancelled
 ↓
Committed
```

Not every work item reaches commit.

## 4. Work Identity

Every schedulable unit should have a stable identity:

```text
work_id
task_id
workflow_id
principal
priority
deadline
epoch
```

Identity supports tracing, cancellation, deduplication, and recovery.

## 5. Scheduling Unit

A scheduling unit is the smallest unit for which the scheduler makes an execution decision.

Depending on the runtime this may be:

```text
job
task
actor
agent
workflow step
fiber
thread
process
```

## 6. Concurrency

Concurrency means multiple units may be in progress during overlapping intervals.

```text
A: █████
B:   █████
```

Concurrency does not require simultaneous physical execution.

## 7. Parallelism

Parallelism means independent work executes simultaneously on multiple execution resources.

```text
CPU 0: █████
CPU 1: █████
```

Parallelism is an implementation/resource property, not a synonym for concurrency.

## 8. Scheduler

The scheduler maps runnable work to execution opportunities:

```text
Runnable Set
    ↓
Scheduling Policy
    ↓
Execution Slot
    ↓
Worker
```

## 9. Admission

A work item should not become runnable merely because it exists.

```text
Created
 ↓
Admission checks
 ↓
Admitted / Rejected
```

Admission may consider:

```text
resource quotas
security authority
deadline
capacity
backpressure
policy
```

## 10. Admission vs Scheduling

```text
Authorized
    ≠
Admitted
    ≠
Runnable
    ≠
Selected
```

A valid principal may still have to wait for resources.

## 11. Runnable State

Runnable means the work can execute if an appropriate execution slot becomes available.

It does not mean execution is guaranteed immediately.

## 12. Queueing

Queues should have explicit bounds and ownership.

```text
Runnable Queue
 ↓ capacity
Admission pressure
```

Part XXXVII resource constraints apply.

## 13. Queue Classes

Queues may be separated by:

```text
priority
principal
tenant
workload class
resource class
control/data plane
```

Isolation prevents one workload from monopolizing scheduler capacity.

## 14. Priority

Priority expresses relative scheduling preference.

```text
P0 > P1 > P2 > P3
```

Priority does not automatically override security, admission, or resource limits.

## 15. Priority Inversion

A high-priority task may wait on a lower-priority task holding a required resource.

Mitigation may include:

```text
priority inheritance
priority ceiling
resource partitioning
```

## 16. Starvation

A work item starves when scheduling policy repeatedly prevents it from receiving execution opportunity.

Fairness policies should explicitly bound starvation where required.

## 17. Fairness

Fairness defines how execution opportunity is distributed among competing work.

Possible models:

```text
FIFO
round robin
weighted fair
fair-share
quota-based
priority-aware fairness
```

## 18. Fairness Scope

Fairness must identify its domain:

```text
worker
queue
node
tenant
principal
cluster
```

Global fairness cannot be inferred from local fairness.

## 19. Weighted Fairness

Weights may determine relative service:

```text
Tenant A weight = 2
Tenant B weight = 1
```

The implementation must define whether weights represent capacity, frequency, or another quantity.

## 20. Fairness vs Efficiency

Maximum throughput and strict fairness may conflict.

NROS should define the intended trade-off for each scheduler class.

## 21. Deadline

A deadline defines a latest acceptable time boundary for an operation.

```text
start
 ↓
work
 ↓
deadline
```

Missing a deadline should produce explicit semantics.

## 22. Deadline vs Timeout

A deadline is an absolute temporal boundary.

A timeout is usually a relative waiting limit.

```text
deadline
 ≠
timeout
```

Part XXXVI defines temporal semantics.

## 23. Deadline Propagation

A parent deadline can propagate to child work:

```text
Parent deadline
 ↓
Child A
 ↓
Child B
```

Children must not receive more effective time than the parent permits.

## 24. Admission by Deadline

A scheduler may reject work that cannot plausibly complete before its deadline when the system contract permits early rejection.

## 25. Deadline Miss

When a deadline is missed, the system should classify the outcome:

```text
completed late
cancelled
failed
best-effort result
escalated
```

## 26. Cancellation

Cancellation requests should identify the work unit:

```text
cancel(work_id)
```

Cancellation is a control request whose completion semantics must be explicit.

## 27. Cancellation Propagation

```text
Parent cancelled
 ↓
Child cancellation
 ↓
Resource release
```

Propagation may be best-effort unless stronger guarantees are explicitly defined.

## 28. Cooperative Cancellation

Tasks may periodically observe cancellation state and stop safely.

```text
Running
 ↓ cancellation observed
Cleanup
 ↓
Stopped
```

## 29. Forced Preemption

Some runtimes may support externally forced interruption.

Forced preemption requires safe boundaries because arbitrary interruption can corrupt state or violate invariants.

## 30. Preemption Points

Preemption can occur at explicitly safe points:

```text
await
checkpoint
scheduler yield
transaction boundary
cooperative poll
```

## 31. Non-Preemptible Regions

Critical sections may temporarily prohibit preemption.

Their maximum duration should be bounded where latency guarantees matter.

## 32. Execution Quantum

Time-sliced schedulers may assign execution quanta:

```text
Task A → quantum
Task B → quantum
Task C → quantum
```

Quantum size affects latency and throughput.

## 33. Work Stealing

Workers may rebalance local queues:

```text
Worker A: many tasks
Worker B: idle
       ↓
    steal work
```

Stealing must preserve ownership, synchronization, and security context.

## 34. Work Sharing

Alternatively, workers may draw from shared queues.

This can simplify balancing but introduces contention and shared-state costs.

## 35. Affinity

Work may prefer a specific:

```text
CPU
NUMA node
worker
node
cache domain
```

Affinity should be treated as a preference unless the contract requires strict placement.

## 36. Placement Constraints

Some work requires placement restrictions:

```text
hardware capability
security boundary
data locality
resource class
tenant isolation
```

Placement must respect Part XLI authorization and Part XXXVII resources.

## 37. Resource-Aware Scheduling

Scheduling should consider resource requirements:

```text
CPU
memory
GPU
I/O
network
storage
special device
```

A task should not be scheduled where its required resources are unavailable.

## 38. Resource Reservation

Critical work may reserve capacity:

```text
Reserved capacity
 ↓
Protected execution
```

Reservations must themselves be bounded to avoid capacity hoarding.

## 39. Backpressure Integration

When downstream capacity falls:

```text
Downstream pressure
 ↓
Admission reduction
 ↓
Queue growth prevented
```

Scheduling and communication backpressure therefore form one control loop.

## 40. Load Shedding

When capacity is insufficient, the scheduler may shed work according to explicit policy:

```text
critical work
   ↓ preserve
best-effort work
   ↓ shed first
```

## 41. Scheduler Overload

The scheduler itself consumes resources.

Scheduler metadata, queues, timers, and coordination must have bounded growth.

## 42. Scheduler Hierarchy

Large systems may use hierarchical scheduling:

```text
Cluster
 ↓
Node
 ↓
Worker
 ↓
Local queue
```

Each level must define ownership and fairness boundaries.

## 43. Hierarchical Quotas

Quota allocation can follow hierarchy:

```text
Global quota
 ↓
Tenant quota
 ↓
Workflow quota
 ↓
Task quota
```

Lower-level limits cannot exceed higher-level authority.

## 44. Fairness and Quotas

A quota is a bound; fairness is a distribution policy.

```text
quota
 ≠
fairness
```

## 45. Control Plane Priority

Scheduler control operations may require reserved capacity so that overload does not prevent:

```text
cancellation
recovery
security response
reconfiguration
```

## 46. Determinism

Determinism means repeated execution under equivalent conditions produces an equivalent specified outcome.

It does not necessarily require identical physical timing.

## 47. Deterministic Scheduling

A deterministic scheduler needs explicit tie-breaking:

```text
priority
 ↓
arrival sequence
 ↓
work_id
```

Without a defined tie-breaker, equal-priority execution order may remain nondeterministic.

## 48. Stable Ordering

Where required, runnable work should have a stable total or partial order.

```text
Order(A,B)
```

must be defined by the scheduler contract rather than accidental queue behavior.

## 49. Nondeterministic Parallelism

Parallel execution may produce different completion orders even when scheduling decisions are deterministic.

```text
Schedule deterministic
    ≠
Completion order deterministic
```

## 50. Deterministic Replay

Replay requires more than recording task IDs.

Relevant inputs may include:

```text
scheduler decisions
randomness
external events
clock observations
resource outcomes
message order
```

## 51. Logical Time

Deterministic simulations or replay may use logical time rather than wall-clock time.

Part XXXVI temporal semantics apply.

## 52. Randomness

If scheduling uses randomness, reproducibility requires a controlled seed or equivalent recorded entropy source where deterministic replay is required.

## 53. External Inputs

External events can break deterministic replay unless their relevant observations are captured as evidence.

Part XL observability semantics apply.

## 54. Scheduler State

Scheduler state may include:

```text
run queues
priority state
fairness counters
tokens
leases
timers
worker assignments
```

If recovery is required, relevant scheduler state must be checkpointable or reconstructible.

## 55. Scheduling Epoch

Scheduler decisions may be associated with an epoch:

```text
Scheduler epoch 12
 ↓ reconfiguration
Scheduler epoch 13
```

Stale scheduling decisions may then be rejected.

## 56. Reconfiguration

Part XXXIX configuration changes can alter scheduling policy.

The transition should be controlled:

```text
Proposed policy
 ↓
Validated
 ↓
Activated
 ↓
New scheduler epoch
```

## 57. Scheduler Drain

Before shutdown or major policy change, work may enter draining state:

```text
Accepting
 ↓
Draining
 ↓
No new work
 ↓
Existing work completes/cancels
```

## 58. Worker Failure

If a worker fails:

```text
Worker
 ↓ crash
In-flight work
 ↓
recover / retry / fail
```

Work ownership must be recoverable or explicitly lossy.

## 59. Work Lease

A worker may hold a lease for a task:

```text
Task
 ↓
Lease
 ↓
Worker
```

Lease expiry prevents permanently abandoned ownership.

## 60. Stale Worker

A restarted worker must not continue authority over work owned by its previous incarnation:

```text
Worker epoch 7
 ↓ restart
Worker epoch 8
```

Stale epoch 7 operations are rejected where fencing applies.

## 61. Duplicate Execution

Retries and worker recovery can produce duplicate execution attempts.

```text
Task T
 ├─ Worker A
 └─ Worker B retry
```

Side effects therefore require idempotency or ownership fencing.

## 62. Work Ownership

Ownership should identify:

```text
worker_id
worker_epoch
lease
work_id
```

## 63. Completion

Completion means the execution body reached its defined terminal condition.

It does not necessarily mean durable commit.

```text
Completed
    ≠
Committed
```

## 64. Failure

Failures should distinguish:

```text
application error
resource denial
timeout
cancellation
security denial
worker failure
communication failure
storage failure
```

## 65. Retry Classification

The scheduler should retry only when policy allows:

```text
transient → potentially retry
permanent → fail
unknown → reconcile / cautious retry
```

## 66. Retry Budget

Retries consume resources and must be bounded:

```text
RetryBudget
 ↓
exhausted
 ↓
terminal outcome
```

## 67. Retry and Deadline

A retry should inherit or respect the remaining deadline.

```text
Original deadline
 ↓
Attempt 1
 ↓
Attempt 2
 ↓
remaining time
```

A retry cannot reset an expired parent deadline unless explicitly defined.

## 68. Retry and Security

A retry remains subject to current authorization and capability validity.

```text
Previously authorized
    ≠
currently authorized
```

## 69. Scheduler and Persistence

Critical scheduler transitions may require persistent state:

```text
Work ownership
 ↓
Checkpoint / transaction
 ↓
Recoverable scheduling state
```

Part XLIII defines durability semantics.

## 70. Scheduler and Communication

Distributed scheduling requires explicit communication semantics:

```text
Work offer
 ↓
Acceptance
 ↓
Lease
 ↓
Execution
 ↓
Completion
```

Part XLII defines transport and partition semantics.

## 71. Scheduler and Security

Scheduling decisions must preserve authority context:

```text
Principal
 ↓
Authorized work
 ↓
Scheduler
 ↓
Worker
```

A worker should not gain broader authority merely because it executes scheduled work.

## 72. Scheduler and Observability

The scheduler should expose:

```text
queue depth
wait time
service time
priority
fairness
deadline misses
preemptions
retries
worker utilization
```

Part XL defines evidence and diagnostics.

## 73. Latency

Scheduling latency is distinct from execution latency:

```text
arrival
 ↓
queue wait
 ↓
start
 ↓
execution
 ↓
completion
```

Each interval should be measurable where required.

## 74. Tail Latency

Average latency can hide severe outliers.

Critical scheduling systems should monitor tail behavior where SLOs require it.

## 75. Admission Latency

Admission itself can become a bottleneck.

Admission queues and control paths must remain bounded.

## 76. Fairness Measurement

Fairness should be measurable according to the selected policy rather than inferred from aggregate throughput.

## 77. Priority Aging

A scheduler may increase the effective priority of waiting work to reduce starvation:

```text
waiting longer
 ↓
priority aging
```

Aging rules must remain bounded and deterministic where required.

## 78. Deadline-Aware Priority

Urgency may be derived from remaining deadline:

```text
less remaining time
 ↓
higher scheduling urgency
```

This must not bypass security or hard resource limits.

## 79. Admission Fairness

Fairness can apply before queueing, not only after work becomes runnable.

Otherwise rejected tenants may be starved despite fair runnable scheduling.

## 80. Tenant Isolation

A tenant must not consume scheduler capacity beyond its configured authority.

```text
Tenant A quota
    ↓
bounded execution share
```

## 81. Priority Abuse

Untrusted principals must not be able to obtain unlimited high priority merely by requesting it.

Priority itself can be security-sensitive policy.

## 82. Scheduler Policy Version

Scheduling decisions should be attributable to a policy version where reproducibility matters:

```text
Decision
 ↓
Scheduler policy vN
```

## 83. Scheduling Evidence

Important scheduling decisions can record:

```text
work_id
scheduler_epoch
policy_version
queue
priority
selected_worker
reason
outcome
```

## 84. Formal Admission Invariant

```text
Scheduled(W)
    ⇒
Admitted(W)
```

## 85. Formal Authority Invariant

```text
Execute(W)
    ⇒
Authorized(Principal(W), W, Context(W))
```

## 86. Formal Deadline Invariant

```text
DeadlineExpired(W)
    ⇒
NoNewExecution(W)
```

unless an explicitly defined late-execution policy applies.

## 87. Formal Quota Invariant

```text
Usage(Tenant)
    ≤
EffectiveQuota(Tenant)
```

## 88. Formal Ownership Invariant

```text
Epoch(WorkerLease) < CurrentWorkerEpoch
    ⇒
Reject(WorkerAction)
```

## 89. Formal Retry Invariant

```text
Attempts(W)
    ≤
RetryBudget(W)
```

## 90. Formal Fairness Invariant

For a scheduler with a bounded-starvation contract:

```text
Runnable(W) ∧ Eligible(W)
    ⇒
EventuallyScheduled(W)
```

subject to explicitly stated resource and failure assumptions.

## 91. Formal Determinism Invariant

For deterministic scheduling mode:

```text
EquivalentInputs + EquivalentSchedulerState
    ⇒
EquivalentSchedulingDecisions
```

## 92. Verification Matrix

| Property | Verification question |
|---|---|
| Admission | Can unauthorized or impossible work enter execution? |
| Priority | Is priority bounded and policy-controlled? |
| Fairness | Is starvation behavior explicitly defined? |
| Quotas | Can a tenant exceed its scheduling allocation? |
| Deadlines | Are deadline semantics explicit and propagated? |
| Cancellation | Is cancellation behavior observable and bounded? |
| Preemption | Are preemption points safe? |
| Placement | Are security/resource placement constraints enforced? |
| Ownership | Can stale workers be fenced? |
| Retry | Are attempts bounded and classified? |
| Determinism | Are tie-breaking rules explicit? |
| Replay | Are relevant scheduling inputs recorded? |
| Recovery | Can scheduler state be reconstructed? |
| Communication | Are distributed work offers and acknowledgements explicit? |
| Persistence | Are critical scheduling transitions durable where required? |
| Observability | Can scheduling decisions be reconstructed? |
| Security | Does scheduling preserve authority scope? |
| Resources | Are scheduler queues and metadata bounded? |
| Failure | Is worker failure behavior explicit? |
| Formal assurance | Are admission, quota, ownership, deadline, fairness, and determinism invariants defined? |

## 93. What Part XLIV Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production scheduler;
- a formally verified fairness algorithm;
- universal deterministic execution;
- complete distributed work stealing;
- hard real-time scheduling guarantees;
- production deadline enforcement;
- complete preemptive cancellation;
- formally verified scheduler recovery;
- production multi-tenant scheduling isolation.

Those require implementation-specific evidence.

## 94. Transition to Part XLV

Part XLIV establishes scheduling and concurrency semantics.

Part XLV should define **execution semantics, state machines, task/agent lifecycle, cancellation, failure propagation, supervision interaction, transactional boundaries, and the formal model of an NROS work unit from creation through terminal state**.

```text
Part XLIII
Storage + persistence + durability + consistency + replication
        ↓
Part XLIV
Scheduling + concurrency + fairness + deadlines + determinism
        ↓
Part XLV
Execution semantics + work lifecycle + cancellation + failure propagation
```

## Canonical rule

> **NROS schedules only admitted work, executes it under explicit authority and resource constraints, bounds fairness, deadlines, retries, and ownership, and distinguishes scheduling decisions from physical execution, completion, and durable commit so that concurrency cannot silently invalidate security, recovery, or determinism guarantees.**
