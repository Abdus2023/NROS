# Part VII — Resources & Budgets

> **Series:** NROS Architecture Series  
> **Part:** VII  
> **Role:** Resource model, ownership, budgets, admission, accounting, and enforcement  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part VI defined temporal semantics. Part VII defines how NROS represents finite resources and controls the relationship between workload demand and available capacity.

The central rule is:

> **Resource accounting, admission, enforcement, and guarantees are distinct properties and must never be conflated.**

## 2. Resource Model

A resource is a finite capability required, consumed, shared, or controlled by runtime work.

```text
Resource
├── CPU
├── Memory
├── Device
├── GPU
├── Network
├── Storage
├── Energy
└── Platform service
```

Resources may be physical, virtual, logical, or externally managed.

## 3. Resource Identity

A resource should have an identity within its management scope.

```text
ResourceId
├── namespace
├── local_id
└── generation / incarnation (when required)
```

Examples:

```text
cpu/worker-0
memory/domain-A
camera/front
network/interface-1
```

Identity does not itself imply ownership or exclusive access.

## 4. Resource Classes

Resources can be classified according to their sharing behavior.

```text
Exclusive
Shared
Partitionable
Consumable
Replenishable
Persistent
Ephemeral
```

Examples:

```text
Motor actuator      → typically exclusive
CPU                 → shared / partitionable
Memory              → allocatable
Network bandwidth   → shareable / budgetable
Battery energy      → consumable
```

The class determines which allocation semantics are meaningful.

## 5. Ownership

Resource ownership must be explicit.

```text
Resource
   ↓
Owner
   ↓
Lease / allocation
   ↓
Use
   ↓
Release
```

Ownership can be:

```text
Exclusive
Shared
Delegated
Borrowed
Leased
Managed by platform
```

Ownership does not necessarily mean the owning component can physically prevent all other use; enforcement depends on the platform.

## 6. Resource Requirement

A workload may declare resource requirements.

Conceptually:

```text
Requirement
├── resource type
├── quantity
├── minimum
├── maximum
├── duration
├── exclusivity
├── affinity
└── policy
```

Example:

```text
CameraProcessor
    CPU: 1 core
    Memory: 64 MiB
    Camera: exclusive
    Network: ≤ 5 MB/s
```

Requirements are requests or constraints until the runtime/platform verifies them.

## 7. Allocation

Allocation assigns resource capacity to a workload.

```text
Request
   ↓
Availability
   ↓
Policy
   ↓
Allocation
   ↓
Lease / ownership record
```

Allocation may be static or dynamic.

## 8. Admission

Resource admission answers:

> **Can this workload enter the active execution domain under the declared resource constraints?**

```text
Workload
   ↓
Requirements
   ↓
Available capacity
   ↓
Policy
   ↓
Admission decision
```

Admission should happen before execution when failure to obtain resources would violate correctness or safety requirements.

## 9. Accounting

Accounting measures or records resource usage.

Examples:

```text
CPU time
Allocated memory
Peak memory
Network bytes
Storage operations
GPU time
Energy estimate
Device usage time
```

Accounting can answer:

```text
Who consumed what?
When?
How much?
Under which activation?
```

But accounting alone does not control consumption.

## 10. Enforcement

Enforcement prevents or limits resource use according to policy.

Examples:

```text
CPU quota
Memory limit
Bandwidth shaping
Device access control
Storage quota
GPU partition
```

The platform may provide enforcement mechanisms.

NROS should expose the semantic contract without assuming that every deployment supports every enforcement mechanism.

## 11. The Four-Way Distinction

NROS explicitly separates:

```text
Accounting
   ↓
What happened?

Admission
   ↓
May this work start?

Enforcement
   ↓
Can consumption exceed policy?

Guarantee
   ↓
Can a bound be demonstrated under defined assumptions?
```

For example:

```text
CPU usage measured
        ≠
CPU quota enforced
        ≠
CPU budget always respected
        ≠
Worst-case CPU demand proven
```

## 12. Budgets

A budget limits resource consumption over a defined scope.

Conceptually:

```text
Budget
├── resource
├── amount
├── interval / lifetime
├── accounting basis
├── enforcement policy
└── exhaustion policy
```

Example:

```text
Activation
CPU budget = 2 ms
period     = 10 ms
```

This is a constraint declaration unless an enforcement mechanism actually exists.

## 13. Budget Exhaustion

When a budget is exhausted, the runtime needs an explicit policy.

Possible policies:

```text
Throttle
Suspend
Cancel
Defer
Reject
Escalate
Continue but record violation
```

The appropriate response depends on execution class and safety requirements.

## 14. Quotas

A quota limits aggregate resource use over a scope.

Examples:

```text
Component memory quota
Tenant CPU quota
Network bandwidth quota
Storage quota
```

Budget and quota are related but not identical.

```text
Budget
→ constraint for a workload/interval

Quota
→ allocation limit for a scope
```

## 15. Reservations

A reservation pre-allocates or protects capacity for future work.

```text
Reservation
├── resource
├── capacity
├── time window
├── owner
└── policy
```

Reservations can support predictable scheduling but require platform support to become meaningful guarantees.

## 16. Leases

A lease grants resource use for a bounded validity period.

```text
Lease
├── resource
├── holder
├── generation
├── expiration
└── renewal policy
```

Expiration should prevent stale holders from retaining logical ownership indefinitely.

## 17. Resource Pools

Resources may be grouped into pools:

```text
CPU Pool
├── core-0
├── core-1
└── core-2
```

or:

```text
GPU Pool
├── device-A
└── device-B
```

A pool can expose aggregate capacity and allocation policy.

## 18. Resource Affinity

Some workloads require specific resources or locality.

Examples:

```text
CPU affinity
NUMA locality
GPU locality
Device locality
Network interface affinity
```

Affinity is a scheduling/allocation constraint, not necessarily an exclusive ownership claim.

## 19. Memory

Memory requires particular care because several distinct quantities may matter:

```text
Requested
Allocated
Resident
Peak
Committed
Available
```

A memory allocation succeeding does not imply that the workload has a guaranteed future memory budget.

## 20. CPU

CPU resource semantics may include:

```text
cores
threads
CPU time
quota
priority
affinity
execution budget
```

A CPU-time budget should specify the accounting domain and enforcement mechanism.

## 21. Devices

Devices are often exclusive or capability-controlled resources.

```text
Device
   ↓
Capability
   ↓
Lease / ownership
   ↓
Operation
```

Examples:

```text
Camera
LiDAR
Motor
GPIO
Serial port
CAN interface
```

Device ownership must not be inferred merely because a component opened a handle.

## 22. Network Resources

Network capacity may be represented as:

```text
Bandwidth
Packets/sec
Connections
Queues
Buffer capacity
```

A bandwidth budget requires a defined measurement point and enforcement mechanism before it can become a hard bound.

## 23. Storage

Storage resources include:

```text
Capacity
IOPS
Bandwidth
File descriptors
Write budget
Persistence lifetime
```

Storage exhaustion is a resource fault and should be observable through the runtime fault model.

## 24. Energy

For mobile or embedded robotics, energy may be treated as a resource.

Possible measurements include:

```text
Battery state
Power
Energy estimate
Thermal budget
```

Energy estimates must identify measurement uncertainty and sampling assumptions.

## 25. Resource Revocation

Resources may need to be revoked.

```text
ALLOCATED
   ↓
REVOCATION REQUEST
   ↓
QUIESCE
   ↓
RELEASE
```

Revocation must define what happens to active work:

```text
cancel
migrate
checkpoint
continue degraded
fail
```

Forced revocation is platform-dependent and must not be assumed safe for arbitrary workloads.

## 26. Resource Failure

A resource may become unavailable after admission.

```text
RUNNING
   ↓
RESOURCE FAILURE
   ↓
DEGRADED / FAULTED
   ├── recover
   ├── substitute
   ├── migrate
   └── stop / safe state
```

The resource manager should expose enough information for the lifecycle and supervision layers to apply policy.

## 27. Resource Contention

Multiple workloads may compete for one resource.

```text
A ─┐
B ─┼──→ Resource R
C ─┘
```

The runtime must define whether contention is resolved by:

```text
priority
fairness
reservation
quota
deadline
first-come
explicit arbitration
```

The policy should be deterministic where the system requires deterministic arbitration.

## 28. Priority Inversion

Shared resources can create priority inversion.

Conceptually:

```text
High-priority task
       ↓ waits
Low-priority task
       ↑ blocked by
Medium-priority work
```

Possible mitigation mechanisms include:

```text
Priority inheritance
Priority ceiling
Resource partitioning
Lock-free design
Scheduling isolation
```

The presence of a mechanism does not by itself prove bounded blocking.

## 29. Resource Transactions

Complex operations may require multiple resources atomically or quasi-atomically.

```text
Request
 ↓
CPU + Memory + Device
 ↓
Admission
 ↓
Commit
```

If partial allocation is possible, rollback semantics must be explicit.

## 30. Resource Observability

Resource events should be observable where useful:

```text
ResourceRequested
ResourceAdmitted
ResourceAllocated
ResourceReleased
BudgetExhausted
QuotaExceeded
ResourceRevoked
ResourceUnavailable
ResourceRecovered
```

Records should identify the relevant entity and activation when possible.

## 31. Verification Matrix

| Property | Verification question |
|---|---|
| Identity | Are resources uniquely identifiable within scope? |
| Ownership | Can unauthorized ownership/use be detected? |
| Admission | Are unmet requirements rejected before execution where required? |
| Accounting | Is consumption attributed correctly? |
| Enforcement | Does the platform actually enforce declared limits? |
| Budget | Is exhaustion detected at the specified boundary? |
| Quota | Is aggregate consumption constrained correctly? |
| Reservation | Is reserved capacity actually protected? |
| Lease | Are expired leases invalidated? |
| Revocation | Does revocation produce a defined terminal outcome? |
| Contention | Is arbitration consistent with policy? |
| Blocking | Are resource-induced delays measurable? |
| Memory | Are requested/allocated/peak values distinguished? |
| Device | Are device capabilities enforced? |
| Failure | Are resource failures observable and propagated? |

## 32. What Part VII Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- universal resource isolation;
- hard CPU or memory quotas;
- deterministic allocation;
- complete device ownership enforcement;
- guaranteed bandwidth;
- energy-aware scheduling;
- bounded priority inversion;
- complete resource migration;
- hardware-level resource guarantees.

Those require implementation and verification evidence.

## 33. Transition to Part VIII

Part VII defines resource semantics.

Part VIII should define **execution scheduling and executor semantics**: how admitted activations compete for execution, how priorities/deadlines/budgets interact, and how scheduling policy remains separate from platform execution mechanisms.

```text
Part VI
Time + temporal semantics
        ↓
Part VII
Resources + budgets
        ↓
Part VIII
Scheduling + executor semantics
```

## Canonical rule

> **NROS treats resources as explicit, bounded runtime objects whose requirements, allocation, accounting, admission, enforcement, and guarantees are separate semantics requiring separate evidence.**
