# Part XXVIII — Resource Lifecycle, Ownership, Allocation & Reclamation

> **Series:** NROS Architecture Series  
> **Part:** XXVIII  
> **Role:** Resource ownership, allocation, reservation, accounting, quotas, isolation, reclamation, garbage collection, leak prevention, lifecycle termination, and resource governance  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXVII defined persistence and durable state. Part XXVIII defines how NROS acquires, owns, accounts for, shares, limits, transfers, and releases finite resources.

The central rule is:

> **NROS must make resource ownership and lifecycle explicit: every managed resource should have an identity, owner, allocation state, accounting boundary, release rule, and failure-safe reclamation path appropriate to its resource class.**

## 2. Fundamental Distinctions

```text
ownership
  ≠
allocation
  ≠
reservation
  ≠
usage
  ≠
accounting
  ≠
quota
  ≠
capacity
  ≠
reclamation
```

These concepts may be implemented together but have different semantics.

## 3. Resource Model

A resource can be modeled as:

```text
Resource
  ├─ identity
  ├─ class
  ├─ owner
  ├─ state
  ├─ capacity
  ├─ allocation
  ├─ usage
  ├─ limits
  └─ lifecycle
```

Resource classes may include:

```text
CPU
memory
storage
network bandwidth
file descriptors
connections
threads
queues
handles
devices
locks
leases
execution slots
```

## 4. Resource Identity

Managed resources should have stable identities where lifecycle tracking requires them:

```text
resource_id
resource_generation
resource_class
```

Generation prevents an old handle from being confused with a newly created resource using the same identifier.

## 5. Ownership

Ownership answers:

> Which principal or subsystem is responsible for the resource lifecycle?

Ownership may belong to:

```text
process
agent
session
tenant
workflow
node
runtime subsystem
system authority
```

Ownership does not automatically grant unrestricted access.

## 6. Authorization vs Ownership

Part XXII security remains authoritative:

```text
owner
  ≠
authorized user
```

An owner may delegate permitted capabilities, while an unauthorized principal must remain unable to manipulate the resource.

## 7. Allocation

Allocation assigns resource capacity to a consumer:

```text
Available
   ↓
Allocated
   ↓
In use
```

Allocation policy should define whether capacity is exclusive, shared, elastic, or overcommitted.

## 8. Reservation

Reservation holds capacity for future use:

```text
capacity
   ↓
reserved
   ↓
allocated
   ↓
used
```

A reservation may expire or be revoked according to policy.

## 9. Usage

Usage measures actual consumption rather than merely allocated capacity.

```text
allocated = 100
used      = 60
```

Accounting must define units, sampling, precision, and aggregation boundaries.

## 10. Capacity

Capacity is the resource available under a defined boundary:

```text
physical capacity
logical capacity
configured capacity
admitted capacity
available capacity
```

These values may differ.

## 11. Quotas

A quota limits consumption by a principal or resource scope:

```text
tenant quota
agent quota
workflow quota
node quota
system quota
```

Quota semantics should define whether the limit applies to:

```text
allocation
usage
reservation
rate
concurrency
cumulative consumption
```

## 12. Hard vs Soft Limits

A hard limit rejects or prevents operations beyond the boundary.

A soft limit may trigger:

```text
warning
throttling
priority reduction
reclamation
admission changes
```

The enforcement point must be explicit.

## 13. Admission Control

Before work starts, NROS may evaluate:

```text
identity
authorization
quota
capacity
priority
deadline
resource requirements
```

Admission control connects Part XXI economics with runtime execution.

## 14. Resource Reservation and Deadlines

Reservations should have explicit lifetime semantics:

```text
created
 ↓
active
 ↓
consumed
 ↓
released
```

or:

```text
created
 ↓
expired
```

Expired reservations must not silently consume capacity forever.

## 15. Resource State Machine

A generic lifecycle is:

```text
Discovered
   ↓
Available
   ↓
Reserved
   ↓
Allocated
   ↓
Active
   ↓
Draining
   ↓
Released
   ↓
Reclaimed
```

Failure may transition a resource directly to recovery or reclamation depending on its class.

## 16. Ownership Transfer

Ownership transfer should be explicit:

```text
Owner A
   ↓ transfer
Owner B
```

The transfer should define:

```text
authorization
atomicity
generation change
pending operations
old-owner invalidation
new-owner activation
```

## 17. Stale Handles

Handles may outlive their resources.

A handle should therefore carry enough identity/generation information to reject stale operations:

```text
resource generation = 8
handle generation   = 7
        ↓
reject stale handle
```

## 18. Leases for Resources

Some resources may be leased rather than permanently owned:

```text
Acquire
 ↓
Lease active
 ↓
Renew
 ↓
Release / Expire
```

Part XXV lease and fencing semantics apply where distributed authority is involved.

## 19. Resource Isolation

Resource isolation may operate at:

```text
process
agent
session
tenant
workflow
node
failure domain
```

Isolation should define which resources can affect one another.

## 20. Noisy Neighbor Control

One consumer should not be able to exhaust shared resources and starve unrelated consumers.

Controls may include:

```text
quotas
rate limits
concurrency limits
priority classes
weighted scheduling
memory limits
queue limits
```

## 21. Fairness

Fairness is distinct from resource equality.

Policies may allocate based on:

```text
priority
weight
reservation
service class
usage history
deadline
```

Fairness guarantees should be explicit rather than assumed from scheduler behavior.

## 22. Resource Accounting

Accounting should identify:

```text
principal
resource
allocation
usage
start time
end time
cost class
limits
```

Accounting data may feed admission, billing, observability, or policy.

## 23. Accounting Consistency

Resource accounting should define whether values are:

```text
best effort
sampled
transactional
monotonic
eventually consistent
strongly consistent
```

An approximate metric should not be used as an authoritative quota counter without an explicit contract.

## 24. Resource Hierarchies

Resources may be nested:

```text
Node
 ├─ CPU
 ├─ Memory
 ├─ Storage
 └─ Network
     ├─ Connection A
     └─ Connection B
```

Parent and child accounting must define whether usage is additive, shared, or overlapping.

## 25. Composite Resources

Some operations require multiple resources simultaneously:

```text
CPU + memory + network + storage
```

Allocation must avoid partial acquisition that creates unrecoverable resource holds.

## 26. Deadlock Risk

Multiple-resource allocation can create circular waits:

```text
A holds R1 → waits R2
B holds R2 → waits R1
```

Policies may use:

```text
global acquisition order
try-acquire
timeouts
preemption
reservation
rollback
```

## 27. Atomic Resource Allocation

Where required, composite allocation should behave as:

```text
all resources acquired
       OR
none acquired
```

The exact atomicity scope must be defined because cross-node atomic allocation is substantially stronger.

## 28. Preemption

Preemption can reclaim resources from active work.

It must define:

```text
eligibility
priority
warning period
checkpoint behavior
cleanup
restart policy
```

Preemption should not silently corrupt persistent state.

## 29. Graceful Draining

Before termination, a resource or consumer may enter:

```text
Active
  ↓
Draining
  ↓
Released
```

Draining should stop new work while allowing selected work to finish or migrate.

## 30. Reclamation

Reclamation returns resources to an available pool:

```text
resource no longer needed
        ↓
cleanup
        ↓
validation
        ↓
reclaimed
```

Reclamation must ensure that stale users cannot continue accessing the resource.

## 31. Forced Reclamation

If graceful cleanup fails, the system may require forced reclamation.

Possible measures:

```text
terminate process
close connection
revoke handle
invalidate generation
unmount resource
quarantine state
```

Forced reclamation must be failure-safe for persistent state.

## 32. Garbage Collection

Automatic garbage collection may reclaim resources whose ownership is no longer reachable.

A collector must distinguish:

```text
unreachable
inactive
expired
orphaned
still referenced
```

Unreachable does not always mean safe to delete.

## 33. Orphan Detection

Orphans can arise from crashes:

```text
owner dies
 ↓
resource remains
 ↓
orphan
```

Recovery must determine whether to reclaim, transfer, quarantine, or preserve the resource.

## 34. Leak Prevention

Resource leaks occur when resources remain allocated without a valid lifecycle owner.

NROS should track:

```text
allocation
owner
release event
release reason
reclamation status
```

## 35. Leak Detection

Evidence may include:

```text
allocation age
unreleased count
orphan count
owner state
resource generation
last activity
```

Detection does not automatically authorize reclamation.

## 36. Resource Failure

A resource may fail independently of its owner:

```text
hardware failure
storage failure
network failure
memory pressure
runtime fault
```

The lifecycle must support:

```text
failed
quarantined
recovered
replaced
retired
```

## 37. Resource Replacement

Replacement should not silently preserve stale identity:

```text
Resource R generation 3
        ↓ failure
Replacement R generation 4
```

Consumers should explicitly discover or receive the replacement.

## 38. Persistence Interaction

Part XXVII governs persistent resource metadata:

```text
ownership
reservations
quotas
leases
allocation records
recovery state
```

A resource manager should recover its authoritative lifecycle state before reopening protected resources.

## 39. Distributed Interaction

Part XXV governs resources shared across nodes.

Distributed resource ownership may require:

```text
leader/authority
lease
term
quorum
fencing token
```

A local process must not assume exclusive ownership of a distributed resource merely because it acquired a local handle.

## 40. Network Interaction

Part XXVI governs resource costs associated with communication:

```text
connections
buffers
streams
bandwidth
retries
reassembly memory
```

Network resource limits should participate in admission control.

## 41. Execution Interaction

Part XXIV execution semantics should make resource acquisition observable when it affects deterministic behavior:

```text
resource available
      ↓
execution branch
```

If resource availability changes execution outcomes, it becomes part of the relevant execution environment.

## 42. Priority and Starvation

Priority-based allocation can cause starvation.

A resource policy may require:

```text
aging
minimum service
weighted fairness
reserved capacity
bounded waiting
```

The guarantee should be measurable.

## 43. Resource Accounting Under Failure

A crash may occur between allocation and accounting:

```text
allocate
 ↓
CRASH
 ↓
accounting update missing
```

Recovery must reconcile authoritative resource state with accounting state.

## 44. Resource Events

A lifecycle event stream may contain:

```text
created
reserved
allocated
started
used
throttled
preempted
draining
released
reclaimed
failed
replaced
```

Events should have stable identity and ordering semantics.

## 45. Observability

Part XIV should expose facts such as:

```text
resource identity
owner
state
allocation
usage
quota
reservation expiry
queue depth
reclamation progress
orphan count
```

Metrics must distinguish measured usage from policy interpretation.

## 46. Security

Part XXII applies to resource control:

```text
ownership authorization
allocation authorization
resource isolation
handle protection
secret-bearing resources
administrative operations
```

Resource exhaustion can also become a security event.

## 47. Formal Resource Model

A conceptual lifecycle transition is:

```text
Rₙ + Operation
      ↓
Policy + Capacity + Authority
      ↓
Rₙ₊₁
```

A safety property may state:

```text
Allocated(R, P)
    ⇒
P is authorized for R
and allocation is within policy bounds.
```

## 48. Reclamation Safety Invariant

A useful invariant is:

```text
Reclaimed(R, generation=g)
    ⇒
No valid operation from generation g
can mutate or consume R afterward.
```

This connects lifecycle management with stale-handle prevention and fencing.

## 49. Verification Matrix

| Property | Verification question |
|---|---|
| Identity | Does every managed resource have an appropriate identity/generation? |
| Ownership | Is lifecycle ownership explicit? |
| Authorization | Can only authorized principals allocate/use resources? |
| Allocation | Is allocation state distinguishable from usage? |
| Reservation | Can reservations expire safely? |
| Quota | Are limits enforced at the correct boundary? |
| Accounting | Is authoritative usage distinguishable from estimates? |
| Isolation | Can one consumer exhaust shared capacity? |
| Composite allocation | Can partial acquisition leak resources? |
| Preemption | Is forced interruption safe? |
| Reclamation | Can resources be safely returned? |
| Orphans | Are crash-created orphans detectable? |
| Leaks | Can unreleased resources be identified? |
| Replacement | Are resource generations invalidated safely? |
| Persistence | Can lifecycle state survive restart? |
| Distribution | Is shared ownership protected by distributed authority? |
| Networking | Are network resource costs bounded? |
| Security | Is resource control authorized and isolated? |
| Observability | Can lifecycle state be reconstructed? |
| Formal assurance | Are lifecycle invariants explicit? |

## 50. What Part XXVIII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a unified resource manager;
- universal resource ownership tracking;
- complete quota enforcement;
- cross-node atomic allocation;
- production-grade garbage collection;
- complete leak detection;
- formally verified reclamation;
- universal preemption;
- complete resource accounting.

Those require implementation-specific evidence.

## 51. Transition to Part XXIX

Part XXVIII defines resource lifecycle and ownership.

Part XXIX should define **isolation, sandboxing, capabilities, trust boundaries, execution domains, privilege transitions, and containment**, connecting resource ownership with the security architecture and agent execution model.

```text
Part XXVII
Persistence + durability + crash consistency
        ↓
Part XXVIII
Resource lifecycle + ownership + allocation + reclamation
        ↓
Part XXIX
Isolation + sandboxing + capabilities + containment
```

## Canonical rule

> **NROS treats every managed resource as a lifecycle-governed object: ownership, authorization, allocation, reservation, usage, accounting, isolation, transfer, failure, and reclamation must be explicit, generation-safe, and recoverable without allowing stale or orphaned authority to consume or mutate resources indefinitely.**
