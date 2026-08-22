# Part VII — Resources & Budgets

> **Series:** NROS Architecture Series  
> **Part:** VII  
> **Role:** Resource identity, ownership, allocation, admission, accounting, budgets, quotas, and enforcement  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part VI defined temporal semantics. Part VII defines how NROS represents and governs resources consumed by components and activations.

The central rule is:

> **A resource claim is meaningful only when the resource, owner, scope, accounting model, admission policy, and enforcement mechanism are explicit.**

NROS must not collapse these distinct concepts into a single field such as `resource_limit`.

## 2. Resource Model

A resource is a bounded or governable capability required by an entity or activation.

```text
Resource
├── identity
├── class
├── capacity
├── unit
├── owner
├── scope
├── allocation state
├── accounting state
├── policy
└── enforcement mechanism
```

Examples include:

```text
CPU
Memory
Storage
Network bandwidth
Network queues
Device access
GPU / accelerator time
DMA / shared-memory regions
File descriptors
Energy budget
Execution slots
```

A resource does not have to be physical. A scheduler slot or bounded activation budget can also be an architectural resource.

## 3. Resource Identity

Resources should have stable identities within their declared scope.

Conceptually:

```text
ResourceId
├── class
├── namespace
├── instance
└── generation
```

Generation-aware resource identity prevents stale ownership or allocation records from being applied to a reincarnated resource instance.

```text
resource = gpu-0 / generation 4
              ↓ restart
resource = gpu-0 / generation 5
```

An allocation referring to generation 4 must not silently authorize access to generation 5.

## 4. Resource Classes

NROS should distinguish at least four broad classes.

### Consumable resources

Use reduces an available quantity.

Examples:

```text
CPU time
network bandwidth
energy
storage capacity
```

### Exclusive resources

Only one owner may hold the resource or protected region at a time.

Examples:

```text
device controller
exclusive peripheral
hardware execution channel
```

### Shareable resources

Multiple consumers may use the resource subject to policy.

Examples:

```text
memory
network link
GPU
shared-memory region
```

### Capability-like resources

Possession represents permission to access or invoke something.

Examples:

```text
device handle
IPC endpoint
privileged operation
protected namespace
```

Capability possession must remain distinct from successful execution.

## 5. Capacity

A resource may expose a capacity:

```text
capacity = quantity + unit + scope
```

Examples:

```text
4 CPU cores
512 MiB memory
100 MiB/s bandwidth
1 device instance
20 execution slots
```

Capacity is a model of what is available under a defined scope. It is not automatically a guarantee of continuously available performance.

For example:

```text
CPU capacity = 1 core
```

does not by itself establish:

```text
1 core of uninterrupted execution
```

## 6. Ownership

Ownership identifies the entity responsible for a resource allocation.

Conceptually:

```text
Resource
   ↓
Owner
   ↓
Allocation
   ↓
Usage
```

Ownership may be assigned to:

- component;
- activation;
- process/runtime domain;
- subsystem;
- deployment unit;
- system authority.

Ownership must have explicit lifetime semantics.

## 7. Allocation

Allocation establishes that a resource has been assigned to an owner.

```text
Request
  ↓
Admission
  ↓
Allocation
  ↓
Use
  ↓
Release
```

Allocation may be:

```text
exclusive
shared
reserved
lease-based
on-demand
```

An allocation record should identify at least:

```text
resource_id
owner_id
amount
scope
allocation_generation
policy
lifetime
```

## 8. Reservation

Reservation establishes an intended future claim on a resource.

```text
Reservation
    ≠
Allocation
    ≠
Actual usage
```

A reservation can support planning and admission without implying that the resource is currently being consumed.

## 9. Admission Control

Admission determines whether a requested operation or entity may enter an execution regime given resource constraints.

Conceptually:

```text
Request
  ↓
Resource requirements
  ↓
Policy evaluation
  ↓
Capacity / reservation check
  ↓
ADMIT or REJECT
```

Admission should occur before a state transition that depends on the resource guarantee.

This follows the series-wide rule:

> **No observed prerequisite → no valid state transition.**

## 10. Accounting

Accounting records resource usage.

Examples:

```text
CPU time consumed
bytes allocated
bytes transmitted
storage occupied
GPU time consumed
energy measured
```

Accounting answers:

> **What was observed to be used?**

It does not by itself answer:

> **Was usage prevented from exceeding a limit?**

Therefore:

```text
Accounting ≠ Enforcement
```

## 11. Enforcement

Enforcement actively constrains resource use according to a policy.

Examples include:

```text
reject allocation
terminate activation
throttle execution
block device access
limit queue depth
apply memory protection
shape network traffic
```

A resource limit should not be called enforced unless the implementation contains a mechanism capable of preventing or controlling the prohibited behavior.

## 12. The Four-Way Separation

NROS explicitly separates:

```text
Resource accounting
        ≠
Resource admission
        ≠
Resource enforcement
        ≠
Resource guarantee
```

These represent progressively stronger claims.

### Accounting

Records observed behavior.

### Admission

Controls entry based on known policy and available information.

### Enforcement

Actively constrains behavior.

### Guarantee

Requires evidence that the specified bound is maintained under the stated assumptions.

## 13. Budgets

A budget is a bounded allowance assigned to an owner or activation.

Conceptually:

```text
Budget
├── resource
├── amount
├── unit
├── interval
├── owner
├── policy
└── exhaustion behavior
```

Examples:

```text
CPU: 2 ms / activation
Memory: 64 MiB / component
Bandwidth: 10 MiB/s / channel
Storage: 1 GiB / deployment
Energy: 5 J / operation
```

A budget is meaningful only when the measurement interval and accounting source are defined.

## 14. Budget Exhaustion

When a budget is exhausted, the architecture must specify the resulting policy.

Possible policies include:

```text
Reject
Throttle
Pause
Cancel
Degrade
Escalate
Recover
Continue with overrun accounting
```

The policy must not be inferred from the existence of the budget field.

```text
Budget configured
      ≠
Budget observed exhausted
      ≠
Budget enforced
      ≠
Operation stopped
```

## 15. Temporal Budgets

Part VI introduced temporal budgets.

Part VII treats them as resource constraints over execution time.

```text
Activation
├── release
├── deadline
├── execution budget
└── consumed CPU time
```

The following must remain distinct:

```text
wall-clock duration
CPU time
scheduler delay
execution budget
deadline
```

An activation running for 10 ms of wall time does not necessarily consume 10 ms of CPU time.

## 16. Memory

Memory resources should distinguish at least:

```text
virtual address space
resident memory
allocated memory
shared memory
DMA-capable memory
pinned memory
persistent storage
```

A configured allocation limit does not automatically establish a hard physical-memory guarantee.

Memory policy may include:

```text
maximum allocation
reservation
pool ownership
allocation failure
reclamation
isolation
```

## 17. CPU

CPU resources can be represented using different policies:

```text
core affinity
execution slots
CPU-time budgets
priority
shares
reservations
quotas
```

These policies are not interchangeable.

For example:

```text
CPU quota ≠ CPU reservation
CPU reservation ≠ CPU affinity
CPU affinity ≠ real-time guarantee
```

Any real-time claim must identify the scheduler, platform, workload assumptions, interference model, and measurement or proof evidence.

## 18. Devices

Device resources require explicit ownership and access semantics.

Conceptually:

```text
Device
├── identity
├── capabilities
├── ownership
├── access mode
├── lifecycle
└── fault state
```

Access modes may include:

```text
exclusive
shared
read-only
write-only
control
observe
```

A device capability does not imply that a command will succeed.

```text
Capability granted
      ≠
Command accepted
      ≠
Command executed
      ≠
Device effect achieved
```

## 19. Network Resources

Network resources include more than link bandwidth.

Possible resources include:

```text
bandwidth
queue capacity
socket / endpoint count
packet rate
buffer memory
connection slots
```

Network accounting must define where measurement occurs.

```text
application
   ↓
transport
   ↓
OS/network stack
   ↓
interface
   ↓
physical link
```

A measurement at one boundary must not automatically be represented as an end-to-end network guarantee.

## 20. Storage

Storage resources should distinguish:

```text
capacity
throughput
IOPS
latency
queue depth
persistent allocation
```

For persistent resources, lifecycle and recovery semantics become important.

A storage quota does not by itself establish write-latency or durability guarantees.

## 21. GPU and Accelerators

Accelerator resources may be represented by:

```text
device
execution context
memory
queue
compute slots
transfer bandwidth
```

Resource ownership should distinguish device access from successful kernel execution and from completion within a requested deadline.

## 22. Resource Hierarchy

Resources may form a hierarchy.

```text
System
 ├── CPU pool
 │    ├── core 0
 │    ├── core 1
 │    └── core 2
 ├── Memory pool
 ├── Network
 │    ├── interface
 │    └── queues
 └── Devices
      ├── camera-0
      └── actuator-0
```

A child allocation must remain compatible with parent capacity and policy.

Hierarchical accounting can then answer:

```text
system → deployment → component → activation
```

## 23. Resource Sharing

Sharing requires an explicit policy.

Examples:

```text
fair sharing
weighted sharing
priority sharing
time slicing
quota-based sharing
reservation-based sharing
```

The existence of multiple owners does not establish fairness.

A fairness claim requires:

- defined fairness metric;
- observation interval;
- workload assumptions;
- scheduler/resource policy;
- evidence.

## 24. Resource Leases

A lease associates ownership or access with a validity interval.

```text
Lease
├── resource
├── owner
├── generation
├── expiration
└── renewal policy
```

Expiration should invalidate the authority associated with the lease.

Generation and lease identity together prevent stale holders from continuing to act after resource reassignment.

## 25. Resource Faults

Resource failures should be observable separately from component failures.

Examples:

```text
ResourceUnavailable
AllocationFailed
BudgetExceeded
QuotaExceeded
DeviceFault
MemoryExhausted
StorageFull
NetworkCapacityExceeded
LeaseExpired
```

The runtime may then apply a policy such as:

```text
retry
backoff
degrade
isolate
recover
restart
stop
```

## 26. Resource Accounting Record

A resource observation should preserve sufficient context for verification.

Conceptually:

```text
ResourceRecord
├── resource_id
├── owner_id
├── activation_id
├── timestamp
├── clock_domain
├── amount
├── unit
├── measurement_boundary
├── policy
├── result
└── platform_context
```

Derived totals should remain traceable to the underlying measurements where stronger claims depend on them.

## 27. Resource and Lifecycle Interaction

Resource prerequisites can guard lifecycle transitions.

```text
CONFIGURED
    ↓
resource requirements evaluated
    ↓
ADMITTED
    ↓
resources allocated
    ↓
READY
    ↓
RUNNING
```

If a required resource disappears:

```text
RUNNING
   ↓
ResourceFault
   ├── Recover
   ├── Degrade
   ├── Isolate
   └── Stop / Safe State
```

The lifecycle state must reflect the actual policy outcome rather than merely the requested transition.

## 28. Resource and Scheduling Interaction

The scheduler may consume resource metadata when selecting work.

```text
Activation
├── priority
├── deadline
├── budget
├── resource requirements
└── eligibility
```

The architecture separates:

```text
Resource eligibility
        ≠
Scheduling selection
        ≠
Physical execution capacity
```

A scheduler selecting an activation does not prove that the requested resource capacity will be available for its complete execution.

## 29. Zero-Copy and Resource Semantics

NROS may use zero-copy mechanisms to reduce copying and allocation overhead.

However:

```text
zero-copy API
      ≠
zero allocation
      ≠
zero memory ownership cost
      ≠
zero synchronization cost
      ≠
zero-copy end-to-end
```

The current `nros-core` implementation contains a type-state SPSC ring buffer with explicit reservation, initialization, commit, and read-guard ownership semantics. This is implementation evidence for that mechanism only; it does not establish that all NROS communication is zero-copy end-to-end. fileciteturn2file0

## 30. Current Repository Reality

The workspace currently contains dedicated crates including:

```text
nros-types
nros-core
nros-node
nros-hal
nros-transport
nros-distributed
nros-cli
nros-sim
nros-studio
nros-macros
nros
nros-audit
```

This establishes an actual modular workspace boundary, but crate presence does not prove that the complete resource model described by this Part is implemented. fileciteturn1file0

The older `NROS_COMPONENT_AND_RESOURCE.md` already treats resources as part of the component contract and gives conceptual CPU and memory requirements. Part VII formalizes the resource semantics needed to distinguish declaration, allocation, accounting, enforcement, and guarantee. fileciteturn5file0

## 31. Verification Matrix

| Property | Verification question |
|---|---|
| Resource identity | Are resource instances uniquely identified within their scope? |
| Generation | Are stale allocations rejected after resource reincarnation? |
| Capacity | Is capacity defined with an explicit unit and scope? |
| Ownership | Is ownership unambiguous and lifetime-bounded? |
| Allocation | Does successful allocation correspond to an actual resource assignment? |
| Admission | Are prerequisite resource checks performed before dependent state transitions? |
| Accounting | Are usage measurements tied to a defined measurement boundary? |
| Enforcement | Is there an actual mechanism constraining prohibited usage? |
| Budget | Are amount, unit, interval, owner, and exhaustion policy explicit? |
| CPU | Are quota, reservation, affinity, and timing guarantees distinguished? |
| Memory | Are virtual, resident, shared, and persistent resources distinguished where required? |
| Device | Is capability distinct from successful device effect? |
| Network | Is the measurement boundary explicit? |
| Storage | Are capacity and performance/durability claims separated? |
| Sharing | Is any fairness claim supported by a defined metric and evidence? |
| Lease | Does expiration invalidate stale authority? |
| Fault | Are resource failures observable and policy-driven? |
| Evidence | Can strong resource claims be traced to raw measurements or proofs? |

## 32. What Part VII Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- complete CPU quotas or reservations;
- hard memory limits;
- deterministic resource allocation;
- guaranteed network bandwidth;
- guaranteed storage latency;
- GPU scheduling guarantees;
- energy guarantees;
- end-to-end zero-copy behavior;
- universal resource isolation;
- hard real-time resource guarantees.

Those claims require implementation evidence and, where applicable, measurement, formal analysis, or physical validation.

## 33. Architectural Invariants

### R1 — Resource identity is explicit

Every governed resource has an identifiable scope and lifecycle.

### R2 — Ownership is explicit

Resource authority must identify its owner and lifetime.

### R3 — Admission precedes dependent state claims

A state transition requiring a resource must not be represented as valid before the relevant prerequisite has been established.

### R4 — Accounting does not imply enforcement

Observed usage is not evidence that usage was constrained.

### R5 — Enforcement does not automatically imply guarantee

A mechanism can enforce a policy under defined conditions without proving a universal bound.

### R6 — Budgets have explicit exhaustion semantics

A budget without a defined exhaustion policy is incomplete architecture.

### R7 — Resource failures are observable

Failure of a required resource must be represented distinctly enough for recovery, degradation, or isolation policy.

### R8 — Strong claims require scoped evidence

Resource guarantees must identify platform, workload, measurement boundary, assumptions, and evidence level.

## 34. Canonical Rule

> **NROS treats resources as governed runtime objects: identity, ownership, allocation, admission, accounting, enforcement, and guarantee are distinct architectural layers, and no resource guarantee is claimed without evidence under an explicit scope.**

## 35. Transition to Part VIII

Part VII defines resource semantics.

Part VIII should establish **execution and scheduling semantics** over the runtime primitives introduced so far.

```text
Part V
Communication + transport
        ↓
Part VI
Time + temporal semantics
        ↓
Part VII
Resources + budgets
        ↓
Part VIII
Execution + scheduling
```

The next Part should reconcile the architectural scheduler/executor separation with the actual NROS execution-related implementation and distinguish scheduling policy, eligibility, dispatch, execution, preemption/cooperative behavior, budgets, deadlines, and observed timing.
