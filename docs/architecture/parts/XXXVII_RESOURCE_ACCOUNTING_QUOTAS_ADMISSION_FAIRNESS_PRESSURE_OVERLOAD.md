# Part XXXVII — Resource Accounting, Quotas, Admission, Fairness, Pressure & Overload

> **Series:** NROS Architecture Series  
> **Part:** XXXVII  
> **Role:** Finite-resource semantics, accounting, quotas, reservations, allocation, admission control, fairness, backpressure, pressure propagation, overload, degradation, and recovery  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXXVI established temporal semantics. Part XXXVII defines how NROS reasons about finite resources and what happens when demand exceeds available capacity.

The central rule is:

> **NROS treats resource capacity as finite, measurable, scoped, and enforceable: usage, reservation, allocation, quota, limit, and pressure are distinct states; admission precedes execution; overload must produce bounded behavior rather than uncontrolled failure propagation.**

## 2. Fundamental Distinctions

```text
capacity
  ≠
quota
  ≠
reservation
  ≠
allocation
  ≠
usage
  ≠
limit
  ≠
pressure
```

## 3. Resource Model

A resource is an explicitly identified consumable or constrainable system capability:

```text
Resource
 ├─ identity
 ├─ class
 ├─ capacity
 ├─ unit
 ├─ scope
 ├─ accounting policy
 └─ enforcement policy
```

## 4. Resource Classes

NROS may account for:

```text
CPU
memory
storage
I/O
network bandwidth
connections
file descriptors
agent slots
concurrency slots
GPU / accelerator capacity
energy / battery budgets
```

The same semantic model can apply to additional resource classes.

## 5. Capacity

Capacity is the amount available under a declared resource model:

```text
Capacity = C
```

Capacity may be static or dynamically changing.

## 6. Usage

Usage represents currently consumed resources:

```text
Usage = U
```

Usage should be measurable according to the resource's accounting contract.

## 7. Reservation

A reservation holds capacity for future use:

```text
Capacity
 ├─ Reserved
 └─ Unreserved
```

Reservation does not necessarily mean the resource is currently consumed.

## 8. Allocation

Allocation grants a consumer the right to use capacity:

```text
Admission
 ↓
Allocation
 ↓
Execution
```

Allocation must have an owner and lifecycle.

## 9. Quota

A quota limits consumption by a scope:

```text
Tenant A ≤ 4 CPU
Workflow B ≤ 2 GiB
Agent C ≤ 8 concurrent tasks
```

Quota is a policy boundary, not a measurement of physical capacity.

## 10. Limit

A limit constrains a specific operation or execution context:

```text
max memory
max concurrency
max request size
max duration
```

Limits may be nested beneath broader quotas.

## 11. Resource Scope

Resource accounting should identify scope:

```text
system
cluster
node
process
tenant
identity
workflow
task
agent
```

A resource claim without scope is incomplete.

## 12. Accounting Identity

Usage must be attributable:

```text
Resource Usage
      ↓
Principal / Workflow / Task
```

This enables quota enforcement, fairness, diagnostics, and chargeback where applicable.

## 13. Accounting Accuracy

The system should distinguish:

```text
requested
reserved
allocated
observed
estimated
```

An estimate must not be presented as an exact measurement.

## 14. Resource Generations

Dynamic resources may change generation:

```text
Resource R
 Generation 7
      ↓
resize / replacement
      ↓
 Generation 8
```

Consumers holding stale assumptions must revalidate.

## 15. Admission Control

Before execution:

```text
Request
 ↓
Validate
 ↓
Authorize
 ↓
Quota Check
 ↓
Capacity Check
 ↓
Admit / Reject / Defer
```

Admission prevents the system from accepting work it cannot safely sustain.

## 16. Admission States

A request may become:

```text
Admitted
Rejected
Deferred
Throttled
Expired
```

These states should remain distinguishable.

## 17. Reservation Before Admission

Some systems require reservation before commitment:

```text
Request
 ↓
Reserve
 ↓
Admit
 ↓
Execute
```

Reservation failure must not leave phantom allocations.

## 18. Atomic Resource Accounting

Where multiple resources must be acquired together:

```text
CPU + Memory + Agent Slot
```

partial allocation must either be explicitly supported or rolled back.

## 19. Resource Transactions

A resource transaction can provide:

```text
prepare
 ↓
commit
or
abort
```

This prevents silent partial admission when atomicity is required.

## 20. Resource Leakage

Every allocation requires a release path:

```text
Allocate
 ↓
Use
 ↓
Release
```

Failure recovery must reclaim abandoned allocations.

## 21. Ownership

Resource allocations should identify:

```text
owner
allocation ID
generation
created time
expiry / lease
```

Part XXVIII governs broader resource lifecycle and ownership semantics.

## 22. Lease-Bound Allocation

Temporary allocations may use leases:

```text
Allocation
 ↓
Lease
 ↓
Expiry
 ↓
Reclaim
```

Part XXXVI governs temporal and fencing semantics.

## 23. Hierarchical Quotas

Quotas may form a hierarchy:

```text
Organization
 ├─ Team A
 │   ├─ Workflow 1
 │   └─ Workflow 2
 └─ Team B
```

A child allocation must not violate an ancestor quota.

## 24. Quota Invariant

```text
Usage(scope) + Reserved(scope)
    ≤
EffectiveQuota(scope)
```

where the exact accounting treatment is defined by the resource contract.

## 25. Borrowing

Hierarchical systems may permit borrowing unused capacity:

```text
Team A under quota
Team B needs capacity
        ↓
Borrow according to policy
```

Borrowing must not silently become permanent quota expansion.

## 26. Fairness

Fairness distributes scarce resources according to an explicit policy:

```text
consumer A
consumer B
consumer C
      ↓
Fair scheduler
```

Fairness requires a defined population and measurement window.

## 27. Fairness Dimensions

Possible dimensions include:

```text
identity
tenant
workflow
priority
resource class
time window
```

A system can be fair in one dimension and unfair in another.

## 28. Priority

Priority influences allocation order but must not bypass:

```text
authorization
safety limits
hard quotas
resource ownership
```

## 29. Starvation

A fair scheduler should define starvation prevention:

```text
long-waiting request
      ↓
priority adjustment / aging
```

Aging is policy, not a universal requirement.

## 30. Concurrency Limits

Concurrency can be bounded independently of CPU/memory:

```text
MaxConcurrentTasks = N
```

This is especially important for agents, external APIs, and connection-heavy workloads.

## 31. Backpressure

When downstream capacity is constrained:

```text
Producer
 ↓
Backpressure
 ↓
Slow / Pause / Reject
```

The system should prefer bounded pressure over unbounded queue growth.

## 32. Pressure Propagation

Pressure can travel upstream:

```text
Storage saturated
      ↓
Worker slows
      ↓
Scheduler queues
      ↓
Producer throttles
```

Propagation should be explicit rather than emerging accidentally from resource exhaustion.

## 33. Queue Bounds

Queues should define:

```text
maximum depth
maximum bytes
retention
ordering
overflow behavior
```

An unlimited queue is not a safe default.

## 34. Overflow Policy

When a queue is full:

```text
reject
block
shed
sample
spill
prioritize
```

The chosen policy must be explicit.

## 35. Load Shedding

Under severe pressure, the system may discard work according to policy:

```text
low priority → shed
high priority → preserve
```

Shedding must be observable and auditable where required.

## 36. Graceful Degradation

A service may reduce optional functionality:

```text
Full
 ↓ pressure
Reduced
 ↓ severe pressure
Minimal
```

Core safety and correctness properties must survive degradation.

## 37. Admission Under Pressure

As pressure increases:

```text
Normal → Throttled → Deferred → Rejected
```

Transitions should have measurable thresholds and recovery rules.

## 38. Pressure Hysteresis

To prevent oscillation:

```text
enter degraded mode at High
exit only below Low
```

where `Low < High`.

## 39. Resource Reservation Fairness

Reservations can cause fragmentation:

```text
capacity reserved
but idle
```

Reservation policies should define expiry, reclaimability, and fairness.

## 40. Fragmentation

Capacity may be technically available but unusable because it is divided into incompatible units.

Resource allocators should distinguish:

```text
physical capacity
allocatable capacity
usable capacity
```

## 41. Multi-Resource Scheduling

Tasks may require a vector:

```text
CPU = 2
Memory = 4 GiB
Network = 20 MB/s
Slots = 1
```

Admission must evaluate the complete resource vector.

## 42. Dominant Constraints

A task may be constrained primarily by one resource:

```text
CPU abundant
Memory scarce
```

The scheduler should identify the effective bottleneck rather than optimizing one resource in isolation.

## 43. Resource Locality

Capacity may exist on one node but not another:

```text
Node A → memory available
Node B → CPU available
```

Scheduling must account for placement constraints.

## 44. Affinity / Anti-Affinity

Policies may require:

```text
same host
same zone
near resource
separate hosts
separate failure domains
```

These constraints interact with capacity and fairness.

## 45. Pressure Metrics

Useful indicators include:

```text
utilization
queue depth
allocation latency
rejection rate
memory pressure
I/O wait
CPU saturation
network saturation
```

Metrics should distinguish symptoms from actual resource exhaustion.

## 46. Saturation

High utilization does not always mean failure:

```text
utilization = high
latency = stable
```

The system should model saturation together with latency, queueing, and error behavior.

## 47. Resource Exhaustion

When a hard resource limit is reached:

```text
Allocation request
 ↓
No capacity
 ↓
Reject / defer / shed
```

The system must avoid uncontrolled allocation failure cascades.

## 48. Cascading Failure

```text
Resource exhaustion
 ↓
Retries
 ↓
More load
 ↓
More exhaustion
```

Retry budgets, admission control, and backpressure must cooperate to prevent this loop.

## 49. Retry and Capacity

A retry should consume the same resource accounting as a normal attempt:

```text
Retry
 ↓
Admission
 ↓
Quota / capacity check
```

Retries must not bypass capacity policy.

## 50. Workflow Resource Budgets

A workflow may receive a resource budget:

```text
Workflow
 ├─ CPU budget
 ├─ memory budget
 ├─ network budget
 └─ concurrency budget
```

Child tasks inherit or explicitly receive bounded portions according to policy.

## 51. Agent Resource Budgets

Autonomous agents require explicit bounds:

```text
agent
 ├─ max active tasks
 ├─ compute budget
 ├─ tool-call budget
 ├─ memory budget
 └─ network budget
```

Autonomy must not imply unlimited resource authority.

## 52. Tool-Call Budgets

Agent tools may be constrained by:

```text
calls / workflow
calls / minute
bytes
cost
concurrency
```

A budget breach should produce an explicit policy outcome.

## 53. Cost Accounting

Some resources can be represented as economic cost:

```text
compute cost
storage cost
network cost
external API cost
```

Cost is distinct from physical resource usage but may participate in admission policy.

## 54. Budget Exhaustion

When a budget is exhausted:

```text
Continue
  ↓
unsafe / unauthorized
```

The system should transition to a defined state such as deferred, degraded, or rejected.

## 55. Resource Pressure and Time

Pressure interacts with deadlines:

```text
High queue delay
      ↓
Deadline approaches
      ↓
Early rejection
```

Part XXXVI temporal semantics govern the deadline calculation.

## 56. Resource Pressure and Persistence

Under storage pressure:

```text
Normal writes
 ↓ pressure
Admission control
 ↓ severe pressure
Reject / compact / archive
```

Persistence safety must not be sacrificed merely to maintain throughput.

## 57. Resource Pressure and Networking

Network saturation may require:

```text
rate limiting
queue bounds
priority scheduling
connection limits
load shedding
```

Part XXVI governs transport-level behavior.

## 58. Resource Pressure and Events

Event streams must define behavior under subscriber overload:

```text
slow consumer
 ↓
backpressure / buffer / drop / replay
```

Part XXXIII defines event delivery semantics.

## 59. Resource Pressure and Security

Security controls should remain active under overload.

An emergency degradation mode must not silently disable authorization, isolation, or audit requirements.

## 60. Resource Reclamation

Reclamation should be triggered by:

```text
completion
cancellation
lease expiry
failure recovery
quota correction
administrative action
```

## 61. Reclamation Verification

A released allocation should not remain counted indefinitely:

```text
Release
 ↓
Accounting update
 ↓
Capacity restored
```

Accounting lag should be measurable.

## 62. Orphan Detection

An allocation without a valid owner or lease becomes an orphan candidate:

```text
Allocation
 ↓
Owner unavailable
 ↓
Orphan detection
 ↓
Reclaim / quarantine
```

## 63. Resource Quarantine

Unsafe or uncertain resources may be quarantined:

```text
Available
 ↓ anomaly
Quarantined
 ↓ verification
Available / Retired
```

Quarantine prevents potentially corrupted capacity from re-entering normal scheduling.

## 64. Recovery

After overload:

```text
Overloaded
 ↓
Pressure decreases
 ↓
Recovery checks
 ↓
Normal operation
```

Recovery should be gradual enough to avoid a second overload wave.

## 65. Recovery Ramp

A system may use:

```text
low admission rate
 ↓
increase gradually
 ↓
observe
 ↓
expand capacity
```

## 66. Resource Reservation During Recovery

Recovery should not consume all newly available capacity immediately if critical workloads require reserved capacity.

## 67. Critical Workloads

A policy may reserve capacity for:

```text
safety-critical tasks
control-plane tasks
recovery tasks
health monitoring
```

Critical status must be explicitly defined and authorized.

## 68. Control Plane Protection

NROS should preserve enough resources for control-plane operations to recover data-plane overload:

```text
Data Plane saturated
        ↓
Control Plane remains viable
        ↓
Recovery / admission decisions
```

## 69. Noisy Neighbor Isolation

One consumer must not exhaust shared capacity and starve others.

Isolation may use:

```text
quotas
reservations
concurrency limits
fair queues
resource partitions
```

## 70. Resource Accounting Invariant

```text
CountedUsage
    ≤
EffectiveCapacity
```

except during explicitly defined transient accounting states that are themselves bounded and observable.

## 71. Admission Invariant

```text
Admitted(Request)
    ⇒
Authorized(Request)
 ∧
QuotaAllows(Request)
 ∧
CapacityAllows(Request)
```

## 72. Release Invariant

```text
AllocationReleased(A)
    ⇒
A cannot continue consuming authoritative capacity
```

unless a new allocation is established.

## 73. Backpressure Invariant

```text
DownstreamCapacity < Demand
    ⇒
UpstreamRate must not grow without bound
```

## 74. Quota Invariant

```text
Usage(scope) + Reserved(scope)
    ≤ EffectiveQuota(scope)
```

under the declared accounting model.

## 75. Overload Invariant

```text
Pressure ↑
    ⇒
System remains within defined safety bounds
```

through throttling, deferral, rejection, shedding, degradation, or other explicit mechanisms.

## 76. Verification Matrix

| Property | Verification question |
|---|---|
| Resource identity | Is each resource class and scope explicit? |
| Capacity | Is available capacity measurable? |
| Usage | Is actual consumption distinguishable from estimates? |
| Reservation | Are reserved resources tracked and reclaimable? |
| Allocation | Does every allocation have ownership/lifecycle? |
| Quota | Are hierarchical quota rules explicit? |
| Admission | Are authorization and capacity checked before execution? |
| Atomicity | Are multi-resource allocations atomic or compensatable? |
| Fairness | Is fairness defined by population and time window? |
| Backpressure | Are queues bounded? |
| Pressure | Is pressure propagated explicitly? |
| Overload | Are degradation and load shedding defined? |
| Reclamation | Can abandoned allocations be recovered? |
| Isolation | Can noisy neighbors be contained? |
| Recovery | Is recovery gradual and bounded? |
| Agents | Are autonomous agents resource-bounded? |
| Retries | Do retries consume normal resource budgets? |
| Observability | Can resource decisions be explained? |
| Formal assurance | Are accounting/admission/backpressure invariants explicit? |

## 77. What Part XXXVII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a universal resource accounting subsystem;
- global quota enforcement;
- production-grade fair scheduling;
- complete multi-resource admission control;
- universal pressure propagation;
- automatic overload recovery;
- exact resource measurements for every environment;
- guaranteed economic cost accounting;
- formally verified resource isolation.

Those require implementation-specific evidence.

## 78. Transition to Part XXXVIII

Part XXXVII establishes finite-resource semantics.

Part XXXVIII should define **execution isolation, fault containment, supervision, process/agent boundaries, failure domains, restart policies, health semantics, and recovery supervision**, connecting resource limits to safe execution under faults.

```text
Part XXXVI
Time + clocks + timers + deadlines + leases + temporal correctness
        ↓
Part XXXVII
Resource accounting + quotas + admission + fairness + pressure + overload
        ↓
Part XXXVIII
Isolation + supervision + fault containment + failure domains + recovery
```

## Canonical rule

> **NROS treats resource pressure as a first-class state: every execution consumes bounded capacity under explicit ownership, quota, admission, and scheduling rules; overload produces controlled backpressure, throttling, degradation, shedding, or rejection rather than unbounded queues, retry storms, or silent loss of safety guarantees.**
