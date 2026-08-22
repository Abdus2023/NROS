# Part LIII — Resource Management, Admission, Scheduling & Allocation

> **Series:** NROS Architecture Series  
> **Part:** LIII  
> **Role:** Resource discovery, capacity, quotas, reservations, admission control, priorities, fairness, placement, preemption, backpressure, allocation, and accounting  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LII established governed configuration and control state. Part LIII defines how NROS reasons about finite resources and turns authorized workload intent into bounded, schedulable, observable allocation.

The central rule is:

> **A resource request is not an allocation: NROS must distinguish requested, admitted, reserved, allocated, consumed, released, and accounted resource state.**

## 2. Resource Model

A resource has:

```text
resource_type
resource_id
capacity
availability
scope
constraints
```

Examples:

```text
CPU
memory
storage
network bandwidth
accelerator
file descriptors
process slots
worker slots
```

## 3. Resource Dimensions

Resources may be:

```text
scalar
vector
counted
exclusive
shareable
consumable
replenishing
```

The accounting model must match the resource semantics.

## 4. Capacity

Capacity describes what a resource can provide under the declared model.

```text
capacity
 ≠
current availability
```

## 5. Availability

Availability may be reduced by:

```text
existing allocations
reservations
maintenance
policy
faults
headroom
```

## 6. Request

A workload request expresses desired resources:

```text
requested CPU
requested memory
requested slots
constraints
priority
lifetime
```

A request creates intent, not entitlement.

## 7. Admission

Admission determines whether work is allowed into the scheduling domain.

```text
Request
 ↓
Policy
 ↓
Quota
 ↓
Capacity
 ↓
Admission Decision
```

## 8. Admission vs Scheduling

```text
Admitted
    ≠
Scheduled
    ≠
Allocated
```

Admission permits participation; scheduling selects placement; allocation grants resources.

## 9. Quotas

Quotas constrain aggregate consumption:

```text
per principal
per tenant
per project
per workload class
per resource
```

## 10. Reservations

A reservation represents protected future or conditional capacity.

```text
available
 ↓
reserved
 ↓
allocated
```

Reservation semantics must define expiry and ownership.

## 11. Reservation Expiry

Expired reservations return to the applicable availability pool unless policy specifies another outcome.

## 12. Allocation

Allocation is the point where resource ownership/usage rights become effective for a workload.

```text
reservation
 ↓
allocation
```

## 13. Consumption

Consumption measures actual use:

```text
allocated
 ↓
consumed
```

A workload can consume less than its allocation.

## 14. Release

When work completes or allocation is revoked:

```text
consumed
 ↓
released
 ↓
available
```

Release must be observable and bounded.

## 15. Accounting

Accounting should distinguish:

```text
requested
reserved
allocated
consumed
released
```

## 16. Overcommit

Overcommit permits aggregate requests to exceed physical capacity under explicit assumptions.

It must never silently violate hard safety guarantees.

## 17. Hard vs Soft Guarantees

Resources may provide:

```text
hard reservation
soft reservation
best-effort availability
```

The scheduler must preserve the declared guarantee.

## 18. Resource Headroom

Safety-critical systems may reserve headroom for:

```text
recovery
control traffic
emergency actions
system daemons
```

## 19. Control-Plane Reserve

Control operations should not become impossible merely because workload demand consumes all nominal capacity.

## 20. Workload Classes

Workloads may be classified by policy:

```text
system
control
interactive
batch
best-effort
emergency
```

## 21. Priority

Priority determines relative scheduling preference, not unrestricted resource entitlement.

## 22. Priority Inversion

The architecture should detect or mitigate cases where low-priority work blocks critical high-priority work.

## 23. Fairness

Fair scheduling can be scoped by:

```text
principal
tenant
project
workload class
resource
```

## 24. Fairness Models

Possible models include:

```text
weighted fair sharing
hierarchical fairness
quota-aware fairness
round robin
```

The chosen model must be explicit.

## 25. Starvation

A scheduler must define whether indefinite starvation is possible and what mechanisms prevent it.

## 26. Aging

Aging can increase effective priority as wait time increases.

It must remain bounded and deterministic where required.

## 27. Placement

Placement selects an execution location satisfying:

```text
resource requirements
policy
topology
capabilities
affinity
anti-affinity
```

## 28. Affinity

Affinity expresses preference for co-location or proximity.

## 29. Anti-Affinity

Anti-affinity expresses separation requirements:

```text
failure domains
security domains
resource contention
```

## 30. Topology Awareness

Placement may consider:

```text
node
rack
zone
region
network domain
accelerator locality
```

## 31. Capability-Aware Placement

A workload requiring a capability must not be placed onto a resource that merely resembles a capable resource.

## 32. Resource Constraints

Constraints may be:

```text
hard
soft
preferred
required
```

## 33. Scheduling Cycle

A scheduling cycle can be modeled as:

```text
Discover
 ↓
Filter
 ↓
Score
 ↓
Select
 ↓
Reserve
 ↓
Commit
 ↓
Allocate
```

## 34. Filter

Filtering removes placements that violate hard constraints.

## 35. Score

Scoring ranks remaining candidates according to policy.

## 36. Select

Selection chooses a candidate without violating declared guarantees.

## 37. Reservation During Scheduling

Where races are possible, the scheduler should reserve capacity before committing placement.

## 38. Scheduling Race

```text
Scheduler A sees capacity
Scheduler B sees capacity
Both allocate
```

Without coordination this can create over-allocation.

## 39. Allocation Commit

Allocation should have a clear commit point:

```text
proposal
 ↓
reservation
 ↓
commit
 ↓
effective allocation
```

## 40. Distributed Scheduling

Distributed schedulers require explicit ownership or coordination for contested resources.

## 41. Scheduler Authority

The architecture must define who may make authoritative allocation decisions:

```text
central scheduler
partition leader
resource owner
coordinated schedulers
```

## 42. Scheduler Epoch

Scheduling authority may be bound to an epoch to prevent stale schedulers from allocating after authority changes.

## 43. Stale Scheduler

```text
scheduler epoch 7
current epoch 8
       ↓
reject allocation
```

## 44. Preemption

Preemption allows higher-priority or safety-critical work to reclaim resources.

## 45. Preemption Safety

Preemption must define:

```text
victim selection
checkpoint behavior
grace period
forced termination
resource reclamation
```

## 46. Cooperative Preemption

Workloads may receive a cancellation or checkpoint request before forced reclamation.

## 47. Forced Preemption

Forced termination must be explicit and must preserve resource-accounting correctness.

## 48. Eviction

Eviction removes work from a placement because of:

```text
resource pressure
fault
policy
maintenance
security
```

## 49. Backpressure

When demand exceeds capacity:

```text
Demand
 ↓
Bounded Queue
 ↓
Admission / Scheduling
```

The system must avoid unbounded work accumulation.

## 50. Queue Limits

Queues should define:

```text
maximum depth
maximum age
maximum memory
rejection behavior
```

## 51. Queue Semantics

Queue ordering may be:

```text
FIFO
priority
deadline
fair-share
policy-defined
```

## 52. Deadline Scheduling

If deadlines influence scheduling, the scheduler must distinguish:

```text
requested deadline
estimated completion
feasibility
```

An impossible deadline should not create false guarantees.

## 53. Admission Rejection

Rejection should explain machine-readable reasons such as:

```text
quota_exceeded
capacity_unavailable
policy_denied
invalid_request
no_feasible_placement
```

## 54. Retry Semantics

A rejected request may be:

```text
retryable
non-retryable
retry-after
requires modification
```

## 55. Resource Leasing

Some allocations may use leases:

```text
lease
 ↓ renewal
lease
 ↓ expiry
release
```

## 56. Lease Safety

Expired authority must not permit continued privileged allocation.

## 57. Resource Revocation

Revocation should define whether it is:

```text
cooperative
immediate
scheduled
conditional
```

## 58. Resource Pressure

Pressure states may be represented as:

```text
normal
constrained
critical
exhausted
```

## 59. Pressure Response

Responses can include:

```text
slow admission
shed work
preempt
scale
reclaim
enter degraded mode
```

## 60. Resource Isolation

Resource controls should prevent one workload from consuming resources assigned to another workload or reserved for the control plane.

## 61. CPU Accounting

CPU accounting should distinguish:

```text
requested
reserved
allocated
runtime consumed
```

## 62. Memory Accounting

Memory semantics must distinguish:

```text
limit
reservation
allocation
resident usage
peak usage
```

## 63. Storage Accounting

Storage may require separate accounting for:

```text
capacity
quota
reserved space
actual usage
IOPS/bandwidth
```

## 64. Network Accounting

Network resources can include:

```text
bandwidth
connections
queues
packets
```

## 65. Accelerator Accounting

Accelerators may require exclusive or partitioned allocation:

```text
whole device
partition
share
```

## 66. Composite Resources

A workload may require a vector:

```text
CPU + memory + storage + network
```

Placement must satisfy the combination, not each dimension independently.

## 67. Fragmentation

Resources can be technically available but unusable because capacity is fragmented.

Schedulers should distinguish:

```text
aggregate free capacity
vs
feasible contiguous/compatible capacity
```

## 68. Resource Locality

Moving work can incur costs:

```text
network
storage
cache
checkpoint
startup
```

Placement policies may account for these costs.

## 69. Scheduling Cost

Optimization must not violate hard safety or authorization constraints merely to improve utilization.

## 70. Utilization

Utilization is an observation, not necessarily a target to maximize without bound.

## 71. Efficiency vs Safety

```text
higher utilization
    ≠
higher system safety
```

Headroom may intentionally reduce utilization.

## 72. Admission Control + Security

A workload must pass both:

```text
security authorization
AND
resource admission
```

Neither substitutes for the other.

## 73. Admission Control + Configuration

Admission behavior is governed by current authoritative configuration revision.

## 74. Admission Control + Epoch

Stale admission decisions must not survive authority changes when the policy requires epoch binding.

## 75. Scheduler + Evidence

A scheduling decision should be reconstructable from:

```text
workload request
resource state
policy revision
scheduler identity
scheduler epoch
decision
```

## 76. Explainability

The scheduler should expose a machine-readable explanation for rejection or placement where practical.

## 77. Determinism

Safety-critical scheduling decisions should avoid uncontrolled nondeterminism where deterministic behavior is required for verification.

## 78. Randomized Scheduling

If randomized selection is used, its seed or equivalent reproducibility mechanism should be defined where auditability requires it.

## 79. Scheduler Failover

When scheduler authority fails:

```text
detect
 ↓
fence stale authority
 ↓
transfer authority
 ↓
reconcile allocations
```

## 80. Allocation Reconciliation

The scheduler should compare:

```text
authoritative allocation state
vs
observed resource state
```

and detect drift.

## 81. Allocation Drift

Possible responses:

```text
repair
revoke
quarantine
alert
```

## 82. Resource Leakage

Resources must not remain allocated after their owning workload has permanently terminated unless retention is intentional.

## 83. Orphaned Allocation

An orphaned allocation requires reconciliation rather than silent indefinite retention.

## 84. Graceful Completion

Work completion should trigger:

```text
final accounting
release
reservation cleanup
queue update
observability
```

## 85. Cancellation

Cancellation should interact with allocation explicitly:

```text
cancel request
 ↓
work termination
 ↓
allocation release
```

## 86. Checkpoint + Preemption

Checkpoint-capable workloads can preserve useful state before resource reclamation.

## 87. Resource-Aware Retry

Retries should account for the resource cost of repeated execution.

## 88. Retry Storm Prevention

Scheduler and admission policies should limit retry amplification under resource pressure.

## 89. Burst Handling

Burst capacity should be explicitly modeled rather than assumed.

## 90. Elasticity

Elastic workloads may adjust allocation within declared bounds:

```text
minimum
requested
maximum
```

## 91. Elastic Allocation

Allocation changes must preserve quotas, policy, and scheduler authority.

## 92. Scale-Up / Scale-Down

Scaling should define:

```text
trigger
rate
limits
stability criteria
rollback behavior
```

## 93. Resource Accounting Evidence

Resource reports should identify their measurement basis:

```text
source
sampling
interval
revision
scope
```

## 94. Resource Metrics

Useful metrics include:

```text
capacity
available
reserved
allocated
consumed
queue depth
wait time
preemptions
rejections
utilization
```

## 95. Formal Admission Invariant

```text
Admit(W)
    ⇒
Authorized(W)
 ∧
QuotaSatisfied(W)
 ∧
FeasibleUnderDeclaredPolicy(W)
```

## 96. Formal Allocation Invariant

```text
Allocate(W, R)
    ⇒
ReservedOrOtherwiseAuthorized(R)
 ∧
WithinCapacity(R)
 ∧
ValidSchedulerAuthority
```

## 97. Formal Release Invariant

```text
Terminal(W)
    ⇒
Eventually Released(Allocation(W))
```

subject to explicitly declared retention semantics.

## 98. Formal Epoch Invariant

```text
SchedulerEpoch < CurrentEpoch
    ⇒
Reject(NewAuthoritativeAllocation)
```

## 99. Formal Accounting Invariant

```text
Consumed(R)
    ≤
AuthorizedEffectiveAllocation(R)
```

for resources with hard usage bounds.

## 100. Verification Matrix

| Property | Verification question |
|---|---|
| Capacity | Is capacity explicitly modeled? |
| Admission | Are security, quota, and feasibility checks enforced? |
| Reservation | Is protected capacity distinguishable from availability? |
| Allocation | Is the allocation commit point defined? |
| Accounting | Are requested/reserved/allocated/consumed/released states distinct? |
| Quotas | Are aggregate limits enforced? |
| Fairness | Can one principal starve others? |
| Priority | Are priority semantics explicit? |
| Placement | Are hard constraints enforced before scoring? |
| Topology | Is locality modeled where required? |
| Preemption | Is reclamation bounded and auditable? |
| Backpressure | Are queues bounded? |
| Leases | Are expired allocations fenced? |
| Epoch | Can stale schedulers allocate? |
| Failover | Is scheduler authority transferred safely? |
| Drift | Are allocation mismatches detectable? |
| Leakage | Are orphaned resources reclaimed? |
| Evidence | Can scheduling decisions be reconstructed? |
| Determinism | Is nondeterminism controlled where required? |
| Recovery | Can allocations be reconciled after restart? |

## 101. What Part LIII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production cluster scheduler;
- universal multi-resource fair scheduling;
- complete preemption support;
- distributed reservations for every resource type;
- production quota enforcement across every workload class;
- complete topology-aware placement;
- universal resource accounting at hardware level;
- automatic elasticity;
- complete scheduler failover evidence.

Those require implementation-specific evidence.

## 102. Transition to Part LIV

Part LIII establishes the resource-control and scheduling plane.

Part LIV should define **workload and execution architecture: workload identity, execution attempts, lifecycle, cancellation, checkpoints, retries, supervision, isolation, side effects, completion, and failure semantics**.

```text
Part LII
Configuration + control state
        ↓
Part LIII
Resources + admission + scheduling + allocation
        ↓
Part LIV
Workload + execution + supervision + completion
```

## Canonical rule

> **NROS never treats scheduling intent as resource ownership: requested, admitted, reserved, allocated, consumed, and released resources are distinct states governed by explicit authority, policy, capacity, lifecycle, and accounting semantics.**
