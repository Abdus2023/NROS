# Part XXI — Capacity, Admission, Overload & Resource Economics

> **Series:** NROS Architecture Series  
> **Part:** XXI  
> **Role:** Capacity, quotas, budgets, reservation, admission control, utilization, saturation, overload, backpressure, fairness, load shedding, and sustainable operation  
> **Status:** Architectural design document — not capacity measurement evidence

## 1. Purpose

Part XX defined resilience under failure. Part XXI defines how NROS behaves when demand approaches or exceeds available resources.

The central rule is:

> **NROS must make resource availability an explicit control boundary: demand is admitted only when the required resources and policy permit it, overload is detected before uncontrolled collapse where possible, pressure is propagated or absorbed according to contract, and resource consumption remains bounded by declared budgets.**

## 2. Fundamental Distinctions

```text
Capacity
  ≠
Allocation
  ≠
Reservation
  ≠
Admission
  ≠
Utilization
  ≠
Saturation
  ≠
Overload
```

### Capacity
The amount of resource that can be provided under defined conditions.

### Allocation
Resource assigned to an entity or operation.

### Reservation
Capacity held for future or conditional use.

### Admission
Decision to allow work to enter a controlled execution domain.

### Utilization
Observed consumption relative to available capacity.

### Saturation
A resource has reached a limiting operating condition.

### Overload
Demand exceeds the service's ability to satisfy its declared operating contract.

## 3. Resource Classes

NROS may manage:

```text
CPU
memory
storage
network bandwidth
file descriptors
threads/tasks
queues
connections
IOPS
GPU / accelerator capacity
energy / power budgets
logical concurrency
```

The exact resource model is domain-specific.

## 4. Capacity Model

A capacity claim must identify:

```text
resource
unit
measurement conditions
available capacity
reserved capacity
allocatable capacity
safety margin
```

Capacity without measurement conditions is ambiguous.

## 5. Demand

Demand may be expressed as:

```text
requests / second
concurrent operations
bytes / second
CPU time
memory footprint
queue occupancy
storage growth
```

Demand can be instantaneous, average, bursty, or forecasted.

## 6. Admission Control

The admission decision is conceptually:

```text
Demand
  ↓
Policy check
  ↓
Resource check
  ↓
Quota check
  ↓
Priority check
  ↓
ADMIT / REJECT / DEFER / SHED
```

Admission prevents uncontrolled work from entering the system.

## 7. Admission vs Scheduling

Admission answers:

```text
“May this work enter?”
```

Scheduling answers:

```text
“When and where should admitted work execute?”
```

A scheduler should not be forced to solve an admission problem after overload has already occurred.

## 8. Quotas

Quotas constrain resource use by scope:

```text
organization
cluster
node
process
component
entity
principal
operation class
```

Quota semantics must define whether a limit is:

```text
hard
soft
borrowable
shared
reserved
```

## 9. Budgets

Budgets represent bounded consumption over a defined dimension.

Examples:

```text
CPU budget
memory budget
retry budget
request budget
recovery budget
network budget
storage budget
```

Part XX recovery budgets therefore become a specialization of the broader resource model.

## 10. Reservation

Reservation protects capacity for a future obligation:

```text
capacity
 ↓ reserve
reserved
 ↓ consume
allocated
```

Reservations must not silently exceed physical or contractual capacity.

## 11. Overcommit

Overcommit permits logical allocation above immediately available physical capacity.

It requires explicit semantics for:

```text
maximum overcommit
contention
reclamation
failure behavior
priority
```

Overcommit is not free capacity.

## 12. Safety Margin

Systems should reserve headroom where required for:

```text
bursts
recovery
background work
control traffic
fault handling
maintenance
```

A system operating at 100% steady-state utilization may have insufficient capacity to recover from faults.

## 13. Utilization

Utilization should be measured per resource:

```text
used / available
```

A single aggregate utilization number can hide a saturated bottleneck.

## 14. Bottlenecks

End-to-end throughput is constrained by bottleneck resources:

```text
CPU → network → storage → external API
                  ↑
               bottleneck
```

Capacity planning must identify the relevant bottleneck rather than optimizing only average utilization.

## 15. Saturation

Saturation may manifest as:

```text
queue growth
latency growth
CPU contention
memory pressure
IO wait
connection exhaustion
rate-limit responses
```

Saturation indicators should be observable through Part XIV.

## 16. Queueing

Queues convert instantaneous demand variation into waiting time.

```text
arrival rate > service rate
        ↓
queue grows
        ↓
latency grows
```

An unbounded queue converts overload into eventual resource exhaustion.

## 17. Queue Bounds

Queues should have explicit limits where unbounded accumulation is unsafe:

```text
max items
max bytes
max age
max priority class
```

Overflow semantics must be defined.

## 18. Backpressure

Backpressure communicates resource pressure upstream:

```text
consumer saturated
      ↓
backpressure
      ↓
producer slows
```

Backpressure may be:

```text
credit-based
window-based
rate-based
queue-based
explicit rejection
```

## 19. Backpressure Propagation

Backpressure can propagate through a pipeline:

```text
C saturated
 ↓
B slows
 ↓
A slows
```

The architecture must define where pressure is absorbed, propagated, or converted into rejection.

## 20. Backpressure Boundaries

Some boundaries cannot propagate pressure indefinitely:

```text
external client
hardware interrupt
human operator
remote service
```

At such boundaries NROS may need buffering, throttling, rejection, or load shedding.

## 21. Load Shedding

When demand cannot be served safely, work may be intentionally discarded or rejected.

Possible strategies:

```text
reject new work
reject low priority
expire stale work
sample telemetry
shed optional features
```

Load shedding is a controlled degradation mechanism, not an implementation failure.

## 22. Priority

Work may be classified by priority:

```text
critical
high
normal
best-effort
background
```

Priority must not create an implicit bypass around safety, authorization, or resource invariants.

## 23. Fairness

Fairness policies may be required to prevent starvation:

```text
round-robin
weighted fair sharing
hierarchical fairness
quota-based fairness
```

Fairness is a policy property and must be defined relative to scope and workload.

## 24. Starvation

Starvation occurs when eligible work cannot obtain service despite continued availability of some resources.

The architecture should define whether starvation is:

```text
forbidden
bounded
accepted for low-priority work
```

## 25. Priority Inversion

A high-priority operation can be indirectly blocked by lower-priority work holding a required resource.

Mitigation may include:

```text
priority inheritance
priority ceiling
resource partitioning
preemption
```

## 26. Resource Isolation

Resource isolation prevents one tenant or component from consuming resources needed by others.

Mechanisms may include:

```text
quotas
cgroups / OS controls
memory limits
CPU shares
network limits
separate queues
separate workers
```

The actual mechanism depends on the deployment environment.

## 27. Multi-Tenancy

For shared infrastructure, resource ownership should distinguish:

```text
tenant demand
shared capacity
reserved capacity
borrowed capacity
tenant quota
```

One tenant's burst must not silently invalidate another tenant's contract.

## 28. Resource Accounting

Resource accounting should maintain a consistent relationship:

```text
capacity
= reserved
+ allocatable
+ explicitly modeled overhead
```

The exact equation depends on the resource semantics.

## 29. Resource Leaks

A resource leak occurs when allocation persists without a corresponding valid ownership obligation.

Examples:

```text
memory
connections
file descriptors
queue entries
leases
reservations
workers
```

Part XIX invariants should define leak-prevention properties where feasible.

## 30. Admission Failure Semantics

Admission failure should produce explicit outcomes:

```text
REJECTED
DEFERRED
QUEUED
SHED
RETRYABLE
PERMANENT
```

Clients should not infer success from mere submission.

## 31. Retry Interaction

Retries increase demand:

```text
original demand
   + retries
   = amplified demand
```

Therefore retry budgets and admission control must interact.

A retry policy that ignores capacity can amplify overload into collapse.

## 32. Recovery Interaction

Recovery consumes resources:

```text
failure
 ↓
recovery work
 ↓
additional CPU / memory / network
```

Part XX recovery therefore competes with ordinary workload unless dedicated recovery capacity is reserved.

## 33. Reserved Recovery Capacity

Critical systems may reserve capacity for:

```text
supervision
failover
checkpoint restore
reconciliation
control-plane traffic
```

This prevents normal workload from consuming all capacity required to recover.

## 34. Control Plane vs Data Plane

The architecture may separate:

```text
control plane
  → supervision, policy, admission, recovery

data plane
  → application workload
```

Control-plane starvation can prevent the system from recovering from data-plane overload.

## 35. Graceful Overload

A controlled overload response can be:

```text
NORMAL
 ↓ rising demand
PRESSURED
 ↓ threshold
OVERLOADED
 ↓
SHEDDING / THROTTLING
 ↓
RECOVERY
 ↓
NORMAL
```

The transitions should be observable and policy-controlled.

## 36. Admission Thresholds

Thresholds may use:

```text
queue depth
CPU utilization
memory pressure
latency
error rate
concurrency
external rate limits
```

Thresholds should include hysteresis where necessary to prevent oscillation.

## 37. Hysteresis

Without hysteresis:

```text
threshold crossed
 ↓ throttle
threshold uncrossed
 ↓ release
threshold crossed
 ↓ throttle
```

rapid oscillation may occur.

Separate engage/release thresholds can stabilize behavior.

## 38. Capacity Planning

Capacity planning should consider:

```text
baseline demand
peak demand
burst demand
growth
fault scenarios
recovery demand
maintenance
safety margin
```

Planning against average demand alone is insufficient for resilient systems.

## 39. Saturation Planning

The system should define what happens before complete saturation:

```text
healthy
 ↓
warning
 ↓
pressure
 ↓
admission restriction
 ↓
load shedding
 ↓
protected operation
```

The thresholds are workload-specific.

## 40. Resource Economics

Every resource has a cost, even when it is not directly monetary:

```text
CPU
memory
latency
energy
storage
network
operational complexity
failure risk
```

The architecture should avoid treating unlimited resource consumption as a valid design assumption.

## 41. Cost-Aware Scheduling

Where appropriate, scheduling may optimize:

```text
priority
latency
throughput
energy
resource cost
fairness
recovery reserve
```

Part VIII remains authoritative for scheduling semantics.

## 42. Admission Policy

Admission decisions may incorporate:

```text
identity
authorization
priority
quota
resource availability
service objectives
current pressure
fault state
```

Part XVII provides the policy orchestration model.

## 43. Overload and Security

Overload can be security-relevant:

```text
request flood
retry amplification
resource exhaustion
connection exhaustion
queue exhaustion
```

Part XI security policy and Part XXI resource policy must therefore interact.

## 44. Observability

Part XIV should expose:

```text
capacity
allocation
reservation
utilization
queue depth
admission decisions
rejections
throttling
shedding
backpressure
saturation
```

Resource pressure without observability is difficult to diagnose or verify.

## 45. Formal Resource Properties

Part XIX can formalize properties such as:

```text
allocation ≤ authorized quota
```

and:

```text
allocated + available + modeled overhead = capacity
```

subject to the precise resource model.

## 46. Resilience Interaction

Part XX and Part XXI form a feedback loop:

```text
Overload
 ↓
resource pressure
 ↓
load shedding / backpressure
 ↓
reduced demand
 ↓
capacity recovery
```

But:

```text
Fault
 ↓
recovery demand
 ↓
resource pressure
 ↓
overload risk
```

Therefore resilience and capacity cannot be modeled independently.

## 47. Verification

Part XVIII should verify:

```text
quota enforcement
admission correctness
queue bounds
backpressure behavior
load shedding
fairness
starvation bounds
resource accounting
recovery reserve
saturation behavior
```

## 48. Evidence

Capacity and overload evidence may include:

```text
load-test reports
resource measurements
queue traces
admission logs
throttling events
shed-work records
latency distributions
capacity models
fault-plus-load experiments
```

Synthetic load results must identify their workload and environment.

## 49. Verification Matrix

| Property | Verification question |
|---|---|
| Capacity | Is capacity defined under explicit conditions? |
| Admission | Can unsafe or unauthorized work be prevented from entering? |
| Quotas | Are limits enforced at the intended scope? |
| Reservation | Is protected capacity actually preserved? |
| Accounting | Do allocation and capacity remain consistent? |
| Queues | Are queue bounds explicit? |
| Backpressure | Does pressure propagate as specified? |
| Shedding | Is load shedding controlled and observable? |
| Fairness | Can eligible work be starved? |
| Priority | Are priority semantics explicit? |
| Retry | Can retries amplify overload? |
| Recovery | Is recovery capacity protected? |
| Saturation | Are pre-saturation controls defined? |
| Hysteresis | Are threshold oscillations controlled? |
| Security | Can resource exhaustion become an attack path? |
| Observability | Are pressure and admission decisions observable? |
| Formal assurance | Are resource invariants stated and verified? |
| Evidence | Are capacity claims backed by reproducible measurements? |

## 50. What Part XXI Does Not Claim

This Part does not claim that the current NROS implementation already has:

- measured capacity limits;
- production-grade admission control;
- complete quota enforcement;
- universal fairness guarantees;
- bounded queues for every subsystem;
- automatic overload shedding;
- guaranteed recovery reserves;
- measured saturation thresholds;
- validated capacity-planning models.

Those require implementation and empirical evidence.

## 51. Transition to Part XXII

Part XXI defines resource pressure and sustainable operation.

Part XXII should define **security architecture at system scale: threat modeling, attack surfaces, trust boundaries, capabilities, isolation, secure failure behavior, and security assurance**, connecting Part XI with Parts XVIII–XXI.

```text
Part XX
Resilience + fault tolerance + availability
        ↓
Part XXI
Capacity + admission + overload + resource economics
        ↓
Part XXII
System security + threat model + assurance
```

## Canonical rule

> **NROS treats capacity as a bounded architectural resource: work must cross explicit admission and policy boundaries, consumption must remain accountable, overload must trigger controlled backpressure, throttling, degradation, or shedding, and recovery capacity must remain protected whenever resilience depends upon it.**
