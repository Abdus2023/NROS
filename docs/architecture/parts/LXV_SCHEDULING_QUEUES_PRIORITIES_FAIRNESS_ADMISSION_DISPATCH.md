# Part LXV — Scheduling, Queues, Priorities, Fairness, Admission & Dispatch

> **Series:** NROS Architecture Series  
> **Part:** LXV  
> **Role:** Scheduling, queue semantics, admission control, priorities, fairness, deadlines, dispatch, preemption, backpressure, execution ordering, and scheduler governance  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LXIV established temporal semantics. Part LXV defines how NROS decides which eligible work may execute, when it executes, with what priority, under which resource constraints, and how it behaves when capacity is unavailable.

The central rule is:

> **NROS scheduling is an authorized resource-allocation decision, not merely queue ordering: work becomes dispatchable only when policy, authority, temporal constraints, dependencies, isolation, and required resources are simultaneously satisfied.**

## 2. Scheduling Model

```text
Work
 ↓
Admission
 ↓
Queue
 ↓
Eligibility
 ↓
Priority / Fairness
 ↓
Resource Fit
 ↓
Temporal Fit
 ↓
Dispatch
 ↓
Execution
 ↓
Completion / Requeue / Preemption
```

## 3. Work Item Identity

Every schedulable item should have an explicit identity where ordering, cancellation, retries, or reconciliation depend on it.

```text
work_id
workload_id
attempt_id
priority
deadline
```

## 4. Scheduling vs Execution

```text
Schedulable
    ≠
Executing
```

A work item can be eligible yet remain queued because resources are unavailable.

## 5. Admission Control

Admission determines whether work may enter the scheduling system.

Admission should evaluate:

```text
authorization
quota
resource feasibility
isolation policy
dependencies
lifecycle state
temporal budget
```

## 6. Admission Failure

Admission failure should be explicit and distinguishable from temporary scheduling delay.

## 7. Queue

A queue represents work awaiting dispatch under a defined ordering and eligibility policy.

## 8. Queue Scope

Queues may be scoped by:

```text
workload
service
namespace
tenant
resource class
priority domain
fault domain
```

## 9. Queue Ordering

Ordering must be explicitly defined rather than inferred from implementation details.

Possible ordering dimensions include:

```text
priority
arrival time
deadline
fairness state
dependency readiness
```

## 10. FIFO

FIFO provides arrival-order semantics within a declared queue scope.

FIFO alone does not guarantee fairness across queues.

## 11. Priority

Priority expresses relative scheduling importance.

Priority must not bypass:

```text
authorization
isolation
hard quotas
resource safety
security policy
```

## 12. Priority Domains

Priority values require an explicit comparison domain.

A priority from one scheduler domain should not automatically be comparable with a priority from another domain.

## 13. Priority Inversion

Priority inversion occurs when higher-priority work is indirectly blocked by lower-priority work.

Mitigation may include:

```text
priority inheritance
priority ceiling
resource partitioning
bounded critical sections
```

## 14. Fairness

Fairness prevents eligible work from being indefinitely starved.

Fairness must be defined quantitatively or procedurally where it is a correctness requirement.

## 15. Fairness vs Priority

```text
Priority
    ≠
Fairness
```

Priority may influence selection while fairness constrains long-term service distribution.

## 16. Weighted Fairness

Queues may receive service according to configured weights.

```text
ServiceShare(Q) ≈ Weight(Q) / ΣWeights
```

subject to resource and policy constraints.

## 17. Starvation

A scheduler should detect or bound starvation where fairness guarantees apply.

## 18. Aging

Aging can increase the effective priority of waiting work over time to reduce starvation.

Aging must remain bounded to avoid unbounded priority escalation.

## 19. Deadline Scheduling

Deadlines may influence scheduling when the work contract requires temporal completion.

```text
Earlier Deadline
    →
Potentially Higher Urgency
```

A deadline does not itself grant additional authority.

## 20. Deadline Admission

Work should not be admitted when its required deadline cannot reasonably be met under declared resource and scheduling guarantees.

## 21. Remaining Budget

Schedulers should operate on remaining temporal budget rather than repeatedly resetting independent timeouts.

## 22. Deadline Miss

A missed deadline is a defined scheduling outcome and should be distinguished from execution failure.

## 23. Temporal Feasibility

Scheduling may evaluate:

```text
remaining budget
estimated execution time
resource availability
queue delay
dependency delay
```

## 24. Resource Fit

A work item is dispatchable only if its required resource allocation can be satisfied.

```text
Eligible
 ∧
ResourcesAvailable
```

is necessary but may not be sufficient.

## 25. Resource Reservation

Schedulers may use reservations for predictable workloads, but reservations must remain bounded and reclaimable according to Part LXIII.

## 26. Dependency Readiness

Work blocked on dependencies should not consume execution slots unnecessarily.

## 27. Dependency Graph

```text
A
 ↓
B
 ↓
C
```

B should become dispatchable only when required conditions from A are satisfied.

## 28. Queue Backpressure

When downstream capacity is exhausted, upstream producers should receive explicit backpressure rather than creating unbounded hidden queues.

## 29. Queue Capacity

Queues should have bounded capacity or an explicit overflow policy.

## 30. Overflow Policy

Possible policies include:

```text
reject
block
spill
sample
coalesce
prioritize
shed load
```

The policy must be explicit.

## 31. Load Shedding

Under overload, lower-priority or deadline-infeasible work may be rejected or cancelled according to policy.

## 32. Backpressure Propagation

Backpressure should propagate through declared dependency and transport boundaries where supported.

## 33. Dispatch

Dispatch assigns eligible work to an execution resource.

```text
Work Item
 ↓
Scheduler Decision
 ↓
Execution Slot
```

## 34. Dispatch Authority

The dispatcher must possess authority to allocate the required resources and activate the workload.

## 35. Dispatch Atomicity

Where partial dispatch could create unsafe states, scheduling and resource allocation should be atomic or compensatable.

## 36. Dispatch Failure

If dispatch fails after resources are reserved, the system must reconcile or roll back the reservation.

## 37. Execution Slot

An execution slot represents schedulable capacity rather than an unconditional guarantee of completion.

## 38. Preemption

Preemption temporarily removes executing work from an execution resource.

```text
Running
 ↓ preempt
Runnable
```

## 39. Cooperative Preemption

Cooperative preemption allows work to yield voluntarily at defined safe points.

## 40. Forced Preemption

Forced preemption may interrupt execution where platform and workload semantics permit it.

## 41. Preemption Safety

Preemption must not violate:

```text
memory safety
resource ownership
transaction semantics
security boundaries
lifecycle invariants
```

## 42. Preemption Cost

Schedulers should consider context-switch and cache/state costs where relevant.

## 43. Preemption Budget

Repeated preemption should be bounded to prevent scheduler thrashing.

## 44. Scheduler Thrashing

Rapid alternation among workloads can reduce useful execution and should be detectable.

## 45. Runnable State

A work item is runnable when it is eligible to execute but not currently executing.

## 46. Blocked State

A work item is blocked when a declared prerequisite prevents dispatch.

## 47. Suspended State

Suspension is an explicit lifecycle or policy state distinct from failure.

## 48. Cancelled State

Cancellation should be represented separately from completion and failure.

## 49. Completion

Completion should record whether work:

```text
succeeded
failed
cancelled
expired
preempted
```

## 50. Requeue

Failed or preempted work may be requeued only when retry policy permits it.

## 51. Retry Identity

A retry should receive an explicit attempt identity.

```text
work_id = W
attempt_id = 1 → 2 → 3
```

## 52. Duplicate Execution

Schedulers must prevent unintended simultaneous execution of singleton work.

## 53. Singleton Constraint

A singleton work item may require:

```text
AtMostOneAuthoritativeAttempt(W)
```

## 54. Fencing

Distributed schedulers may use epochs or fencing tokens to prevent stale dispatchers from activating obsolete attempts.

## 55. Scheduler Leadership

A distributed scheduling authority requires explicit ownership, lease, or consensus semantics where concurrent decisions could conflict.

## 56. Stale Scheduler

```text
Old Scheduler
    ⇏
Current Dispatch Authority
```

## 57. Scheduler Epoch

An epoch identifies the current authoritative scheduler generation.

## 58. Work Epoch

Work may carry an execution epoch to reject stale dispatch or completion messages.

## 59. Completion Validation

Completion should verify that the reporting attempt is still authoritative where stale results could corrupt state.

## 60. Queue Reconciliation

After scheduler restart, queued and running work should be reconciled against observed execution state.

## 61. Durable Queue State

Critical queue state should be durable or reconstructable when loss could cause unsafe duplication or permanent work loss.

## 62. Queue Recovery

Recovery should distinguish:

```text
queued
running
unknown
completed
cancelled
```

rather than assuming that an absent process means failed work.

## 63. Unknown Execution Outcome

```text
Dispatch
 ↓
Connection Loss
 ↓
Unknown Outcome
```

The scheduler should reconcile before retrying non-idempotent work.

## 64. Fairness Accounting

Fairness state may track:

```text
service received
wait time
weight
priority
resource usage
preemption history
```

## 65. Fairness Isolation

A high-priority queue must not obtain unauthorized access to another queue's reserved resources merely because it is urgent.

## 66. Resource Partitioning

Resources may be partitioned to provide stronger isolation between scheduling domains.

## 67. Capacity Sharing

Shared capacity requires explicit accounting and ownership semantics from Part LXIII.

## 68. Admission vs Preemption

Admission decides whether work may enter execution management.

Preemption decides whether currently executing work may be displaced.

They are separate policy decisions.

## 69. Emergency Priority

Emergency scheduling classes may exist, but they remain subject to safety and authorization constraints.

## 70. Priority Abuse

Untrusted workloads must not be able to self-assign arbitrary priority escalation.

## 71. Priority Authorization

Priority should be assigned by an authorized policy source or bounded by declared workload policy.

## 72. Queue Isolation

One tenant or workload should not be able to monopolize a shared queue without explicit policy.

## 73. Multi-Level Scheduling

Schedulers may combine:

```text
global fairness
 ↓
queue priority
 ↓
work-item ordering
 ↓
resource fit
 ↓
deadline urgency
```

## 74. Scheduling Policy Version

Scheduling decisions should identify the policy version when reproducibility or auditability requires it.

## 75. Policy Update

Policy changes should define how queued and running work are affected.

Possible behaviors:

```text
continue
re-evaluate
requeue
preempt
cancel
```

## 76. Scheduler Determinism

When multiple candidates are equivalent under policy, a deterministic tie-breaker should be used where reproducibility matters.

## 77. Tie Breaking

Possible deterministic keys include:

```text
arrival sequence
work_id
attempt_id
queue sequence
```

## 78. Randomization

Randomized scheduling may be used for load distribution but should remain bounded and observable where correctness depends on reproducibility.

## 79. Scheduler Observability

The scheduler should expose evidence for:

```text
admission
queueing
delay
selection
preemption
dispatch
completion
requeue
rejection
```

## 80. Scheduling Metrics

Useful metrics include:

```text
queue depth
queue latency
dispatch latency
execution latency
deadline miss rate
starvation duration
preemption count
rejection rate
resource utilization
```

## 81. Scheduling Trace

A scheduling decision may record:

```text
work_id
queue
policy_version
priority
deadline
resource_requirements
eligibility
selected_reason
scheduler_epoch
```

## 82. Explainability

A scheduler should be able to answer why a work item was dispatched, delayed, rejected, or preempted when evidence requirements apply.

## 83. Safety over Throughput

When safety and throughput conflict, safety and authorization constraints take precedence.

## 84. Fairness over Opportunistic Throughput

Where fairness is a declared requirement, short-term throughput optimization must not create indefinite starvation.

## 85. Deadline over Fairness

Where a hard deadline is part of the contract, the scheduler may prioritize deadline feasibility subject to safety, authority, and policy.

## 86. Backpressure over Unbounded Growth

When capacity is unavailable, explicit backpressure or load shedding is preferred to uncontrolled queue growth.

## 87. Admission Control and Resource Exhaustion

Admission must protect resource quotas and fault domains against overload.

## 88. Scheduler Failure

If the scheduler becomes unavailable, the system should define whether existing workloads:

```text
continue
freeze
self-schedule
fail over
terminate
```

## 89. Scheduler Failover

Failover must preserve authority ordering and prevent two schedulers from dispatching conflicting work simultaneously.

## 90. Scheduler Recovery

A recovering scheduler should reconcile observed execution before issuing new dispatch decisions.

## 91. Resource Revocation

If a resource lease expires while work is queued or executing, scheduling must respect the revocation semantics rather than assuming continued access.

## 92. Temporal Revocation

An expired deadline should prevent further execution unless the work is explicitly re-admitted under a new contract.

## 93. Security Boundary

Scheduling must not cross isolation or capability boundaries merely to improve utilization.

## 94. Formal Eligibility Invariant

```text
Dispatchable(W)
    ⇒
Authorized
 ∧
Isolated
 ∧
DependenciesReady
 ∧
ResourcesAvailable
 ∧
TemporalContractValid
```

## 95. Formal Quota Invariant

```text
Allocated(W)
    ⇒
UsageAfterAllocation ≤ EffectiveQuota(W)
```

## 96. Formal Deadline Invariant

```text
DeadlineExpired(W)
    ⇒
NoDispatch(W)
```

unless explicit re-admission creates a new temporal contract.

## 97. Formal Singleton Invariant

```text
Singleton(W)
    ⇒
AtMostOneAuthoritativeAttempt(W)
```

## 98. Formal Stale-Scheduler Invariant

```text
SchedulerEpoch(S) ≠ CurrentEpoch
    ⇒
Dispatch(S) = Forbidden
```

## 99. Formal Reconciliation Invariant

```text
SchedulerRestart
    ⇒
ReconcileObservedExecution
```

before unsafe redispatch.

## 100. Verification Matrix

| Property | Verification question |
|---|---|
| Admission | Are authorization and resource feasibility checked? |
| Queue | Is ordering explicit? |
| Priority | Is priority bounded and authorized? |
| Fairness | Is starvation prevented where required? |
| Deadline | Are temporal constraints enforced? |
| Resources | Is dispatch resource-feasible? |
| Dependencies | Are blocked workloads excluded from dispatch? |
| Backpressure | Is queue growth bounded? |
| Dispatch | Is allocation atomic or compensatable? |
| Preemption | Is interruption safe? |
| Retry | Are attempts explicitly identified? |
| Singleton | Are duplicate authoritative attempts prevented? |
| Fencing | Can stale schedulers be rejected? |
| Recovery | Is execution reconciled before redispatch? |
| Determinism | Are equivalent choices deterministically resolved where required? |
| Observability | Can scheduling decisions be explained? |
| Failover | Can two schedulers conflict? |
| Revocation | Are expired resources and deadlines respected? |
| Security | Can scheduling bypass authority boundaries? |
| Evidence | Can scheduling decisions be independently verified? |

## 101. What Part LXV Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production multi-tenant scheduler;
- complete weighted fairness;
- universal deadline-aware scheduling;
- distributed scheduler consensus;
- complete fencing for every dispatch path;
- universal preemption;
- complete queue durability;
- automatic starvation detection;
- complete scheduling explainability;
- production-grade load shedding.

Those require implementation-specific evidence.

## 102. Transition to Part LXVI

Part LXV establishes scheduling and dispatch semantics.

Part LXVI should define **event ordering, messaging, queues, delivery guarantees, backpressure protocols, acknowledgment, retry, deduplication, and exactly-once claims** across NROS execution boundaries.

```text
Part LXIV
Time + clocks + deadlines + temporal consistency
        ↓
Part LXV
Scheduling + queues + priorities + fairness + dispatch
        ↓
Part LXVI
Messaging + delivery + ordering + acknowledgement + deduplication
```

## Canonical rule

> **NROS scheduling is a constrained authorization decision: a work item is dispatchable only when its authority, isolation, dependencies, resources, quota, and temporal contract are valid, while fairness, backpressure, bounded preemption, fencing, reconciliation, and evidence preserve system-wide correctness under contention and failure.**
