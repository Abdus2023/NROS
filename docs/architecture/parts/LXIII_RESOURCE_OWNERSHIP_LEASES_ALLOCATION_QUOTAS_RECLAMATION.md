# Part LXIII — Resource Ownership, Leases, Allocation, Quotas & Reclamation

> **Series:** NROS Architecture Series  
> **Part:** LXIII  
> **Role:** Resource ownership, allocation, quotas, leases, locks, renewal, transfer, expiration, reclamation, and lifetime management across runtime boundaries  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LXII defines runtime lifecycle and supervision. Part LXIII defines how runtime resources are owned, allocated, bounded, transferred, expired, and reclaimed across those lifecycle transitions.

The central rule is:

> **NROS treats every scarce or authority-bearing runtime resource as having an explicit ownership and lifetime contract; allocation without ownership, bounded lifetime, or reclamation semantics is incomplete resource management.**

## 2. Resource Model

A resource may be:

```text
CPU
memory
storage
file descriptor
socket / port
device
lock
queue
buffer
shared-memory region
connection
credential handle
execution slot
```

## 3. Resource Identity

```text
Resource
    ≠
Resource Handle
    ≠
Resource Owner
```

A handle identifies a means of accessing a resource; it does not necessarily define ownership.

## 4. Ownership

Ownership defines which principal or workload is authoritative for a resource's lifecycle.

```text
Owner(R) = P
```

## 5. Ownership vs Access

```text
Ownership
    ≠
Permission
```

A component may access a resource without owning its lifecycle.

## 6. Resource Authority

Operations should be checked against both:

```text
resource identity
requested operation
```

and the authority of the caller.

## 7. Allocation

Allocation grants a resource or resource capacity to a consumer under an explicit contract.

```text
Available
 ↓ allocation
Allocated
```

## 8. Admission vs Allocation

```text
Workload Admitted
    ≠
All Resources Allocated
```

A workload may remain pending until required resources become available.

## 9. Reservation

Reservation represents an intended future allocation.

```text
Reservation
    ≠
Allocation
```

## 10. Reservation Expiry

Reservations should have bounded lifetimes to prevent abandoned capacity from remaining unavailable indefinitely.

## 11. Quota

A quota limits how much resource capacity a principal may consume.

```text
Usage(P) ≤ Quota(P)
```

## 12. Quota Scope

Quotas may apply by:

```text
principal
workload
service
namespace
tenant
resource class
fault domain
```

## 13. Hard vs Soft Quota

A hard quota prevents allocation beyond the limit.

A soft quota may permit temporary excess under explicit policy.

## 14. Quota Accounting

Accounting should define whether reserved, allocated, or actively consumed capacity counts toward the quota.

## 15. Hierarchical Quotas

Parent and child quotas should preserve aggregate bounds:

```text
ChildUsage
    ≤
ChildQuota
    ≤
ParentQuota
```

## 16. Resource Classes

Resources may be grouped into classes with common allocation semantics.

Examples:

```text
CPU
memory
persistent storage
ephemeral storage
network bandwidth
device access
```

## 17. Capacity

Capacity represents the amount of allocatable resource available within a declared scope.

## 18. Capacity Reservation

Reserved capacity should remain distinguishable from currently free capacity.

## 19. Fragmentation

Allocation algorithms should account for fragmentation where resources cannot be arbitrarily subdivided.

## 20. Allocation Failure

Allocation failure should be explicit and should not silently downgrade requested resource guarantees.

## 21. Degraded Allocation

If reduced resources are acceptable, degradation must be part of the workload contract rather than an implicit allocator decision.

## 22. Priority

When resources are scarce, allocation may consider explicit priority.

Priority must not bypass security or hard quota boundaries.

## 23. Fairness

Fair allocation should prevent indefinite starvation among eligible consumers according to policy.

## 24. Preemption

Preemption may reclaim resources from lower-priority consumers when policy permits.

## 25. Preemption Safety

A preempted workload should receive explicit lifecycle semantics rather than being treated as a crash.

## 26. Graceful Reclamation

Where possible, preemption should allow controlled release before forced reclamation.

## 27. Forced Reclamation

Forced reclamation may terminate or invalidate resource use when deadlines expire.

## 28. Lease

A lease grants temporary authority or ownership until expiry or explicit release.

```text
Lease
 ↓
Resource Authority
 ↓
Expiry
```

## 29. Lease vs Lock

```text
Lease
    ≠
Lock
```

A lease is time-bounded authority; a lock is a synchronization mechanism. Some systems combine them, but their semantics remain distinct.

## 30. Lease Identity

A lease should have an explicit identity and owner.

## 31. Lease Expiry

After expiry, the former holder must no longer be considered authoritative.

## 32. Renewal

Lease renewal must occur before expiry and must be authorized by the current owner or governing authority.

## 33. Renewal Failure

Renewal failure should lead to explicit expiration semantics rather than indefinite implicit ownership.

## 34. Lease Clock

Lease semantics depend on a declared time model and must account for clock uncertainty in distributed systems.

## 35. Lease Duration

Lease duration should balance failure detection latency against renewal overhead and temporary authority exposure.

## 36. Fencing Token

A fencing token or epoch can prevent stale lease holders from continuing to modify a resource after lease loss.

```text
Lease 41
 ↓
Fencing Epoch 9
```

## 37. Fencing Invariant

```text
Operation(O, Epoch=E)
    ⇒
E = CurrentEpoch(Resource)
```

for resources requiring fencing.

## 38. Lock Ownership

Locks should identify their owner instance or lifecycle epoch where stale ownership is possible.

## 39. Lock Expiration

A lock that can survive owner failure needs explicit recovery or lease semantics.

## 40. Deadlock

Resource allocation must consider deadlock risk when multiple resources are acquired in dependent order.

## 41. Lock Ordering

A global or declared lock ordering can reduce deadlock risk.

## 42. Try-Lock

Non-blocking acquisition may be preferable at lifecycle-sensitive boundaries where waiting indefinitely is unsafe.

## 43. Resource Dependencies

Resource requirements may form a graph:

```text
Workload
 ↓
CPU + Memory
 ↓
Network
 ↓
Device
```

Allocation should respect dependency semantics.

## 44. Atomic Allocation

Where partial allocation could create unsafe states, resource acquisition should be atomic or compensatable.

## 45. Partial Allocation

If atomic allocation is impossible, partially allocated resources must have deterministic rollback or reclamation semantics.

## 46. Resource Handle Lifetime

Handles should not outlive the authority or lifecycle contract that created them.

## 47. Stale Handle

```text
OldHandle
    ⇏
CurrentAuthority
```

## 48. Handle Revocation

Security-sensitive handles may require explicit revocation when ownership or authority changes.

## 49. Transfer

Resource ownership may be transferred only through an explicit transition.

```text
Owner A
 ↓ authorized transfer
Owner B
```

## 50. Transfer Atomicity

Transfer should avoid states where neither party is authoritative or both believe they are authoritative.

## 51. Transfer Epoch

An ownership epoch can make stale ownership attempts rejectable.

## 52. Shared Ownership

Shared resources should distinguish:

```text
shared access
shared ownership
exclusive ownership
```

These have different lifecycle semantics.

## 53. Reference Counting

Reference counting may manage object lifetime but does not automatically provide distributed ownership or security authorization.

## 54. Borrowing

Borrowed access should have a bounded lifetime or explicit invalidation relationship to its owner.

## 55. Resource Lifetime

Every allocation should define when the resource is released:

```text
explicit release
scope exit
lease expiry
workload termination
policy revocation
```

## 56. Reclamation

Reclamation returns resources to an allocatable state after ownership or authority ends.

## 57. Reclamation Safety

A resource must not be reclaimed while a still-authoritative consumer can legally use it.

## 58. Reclamation Completeness

Reclamation should include secondary resources such as:

```text
handles
locks
ports
temporary storage
IPC endpoints
credential references
```

## 59. Crash Reclamation

The system must define how resources are recovered after owner failure.

## 60. Supervisor-Assisted Reclamation

Part LXII supervision may initiate reclamation after lifecycle termination, subject to ownership and fencing rules.

## 61. Orphan Resources

An orphan resource has no valid current owner but remains allocated or reachable.

Orphans should be detectable and reclaimable according to policy.

## 62. Leak Detection

Resource leaks should be observable through accounting, lifecycle evidence, or periodic reconciliation.

## 63. Reconciliation

Resource managers should reconcile:

```text
Recorded Ownership
        ×
Observed Resource State
```

## 64. Accounting Invariant

```text
TotalAllocated
    ≤
TotalCapacity
```

within the declared accounting scope.

## 65. Ownership Invariant

```text
Resource R
    ⇒
AtMostOneExclusiveOwner(R)
```

for resources declared exclusive.

## 66. Lease Invariant

```text
LeaseExpired(L)
    ⇒
Authority(L) = false
```

## 67. Quota Invariant

```text
Usage(P)
    ≤
EffectiveQuota(P)
```

unless an explicitly authorized soft-quota exception exists.

## 68. Reclamation Invariant

```text
OwnerTerminated(R)
 ∧
NoValidTransfer(R)
    ⇒
Reclaimable(R)
```

## 69. Resource Safety

Resource availability must never be treated as proof of authorization.

```text
Available
    ≠
Authorized
```

## 70. Resource Security

A resource manager should validate both capacity and authority before allocation.

## 71. Resource Reservation Fairness

Reservations should not allow one principal to indefinitely monopolize future capacity without use.

## 72. Reservation Cancellation

Cancelled or expired reservations must release reserved capacity promptly.

## 73. Quota Recalculation

Policy changes may alter effective quotas; existing allocations require explicit transition semantics.

## 74. Quota Reduction

Reducing a quota below current usage should not create an undefined state.

Possible policies include:

```text
freeze growth
preempt
request release
mark non-compliant
```

## 75. Quota Increase

Increasing quota does not automatically allocate additional resources.

## 76. Resource Admission

Admission decisions should account for:

```text
capacity
quota
priority
dependencies
isolation
lifecycle state
policy
```

## 77. Resource Backpressure

When capacity is unavailable, the system should apply explicit backpressure rather than creating unbounded queues or hidden overcommit.

## 78. Overcommit

Overcommit is valid only when its semantics and failure behavior are explicitly defined.

## 79. Memory Overcommit

Memory overcommit must account for eviction, reclaim, swap, and failure behavior according to the platform.

## 80. CPU Overcommit

CPU overcommit may be acceptable when scheduling guarantees remain explicit.

## 81. Storage Overcommit

Storage overcommit can produce delayed failure and should be tightly governed.

## 82. Network Overcommit

Bandwidth allocation should distinguish reserved guarantees from best-effort capacity.

## 83. Device Ownership

Exclusive devices require explicit ownership and transfer semantics.

## 84. Device Revocation

Device access should be revoked before a workload is considered fully terminated when device state could remain active.

## 85. Port Ownership

Network ports should have explicit ownership and conflict detection.

## 86. Persistent Storage Ownership

Persistent storage must distinguish data ownership from temporary mount or handle ownership.

## 87. Ephemeral Storage

Ephemeral storage should be reclaimed with the associated lifecycle unless explicitly promoted to durable state.

## 88. Shared Memory

Shared-memory regions require explicit producer/consumer ownership and lifecycle rules.

## 89. Buffer Ownership

Message or buffer ownership should define when responsibility transfers between producer, transport, and consumer.

## 90. Queue Capacity

Queues should have bounded capacity or explicit backpressure semantics.

## 91. Resource Accounting Evidence

Accounting evidence should identify:

```text
resource
owner
allocation
release
current usage
epoch
policy
```

without exposing unrelated sensitive data.

## 92. Resource Events

A resource lifecycle event may record:

```text
resource_id
resource_class
owner
previous_state
new_state
amount
epoch
actor
time
cause
```

## 93. Failure During Allocation

If allocation fails midway, the system must either rollback or enter a recoverable reconciliation state.

## 94. Failure During Release

Release failures must not silently mark a resource free while the underlying resource remains active.

## 95. Unknown Ownership

Unknown ownership is a security and consistency condition requiring reconciliation, not an implicit grant.

## 96. Recovery

After resource-manager restart, ownership and allocation state should be reconstructed or reconciled before unsafe allocations resume.

## 97. Resource Manager Leadership

Distributed resource managers require leadership, lease, or fencing semantics when multiple authorities can allocate the same resource.

## 98. Resource Lifecycle and Runtime Lifecycle

Runtime termination should trigger resource cleanup, but resource reclamation must remain independently verifiable.

```text
Runtime Stopped
    ⇒
Cleanup Required
    ⇏
Cleanup Complete
```

## 99. Verification Matrix

| Property | Verification question |
|---|---|
| Ownership | Is every exclusive resource owner explicit? |
| Allocation | Is allocation authorized and capacity-bounded? |
| Quota | Are hard/soft quota semantics explicit? |
| Reservation | Can abandoned reservations expire? |
| Lease | Is authority time-bounded where required? |
| Renewal | Can stale owners continue after renewal failure? |
| Fencing | Are stale distributed owners rejected? |
| Locking | Are deadlock and stale-lock cases handled? |
| Transfer | Is ownership transfer atomic or compensatable? |
| Handles | Can stale handles be rejected or revoked? |
| Reclamation | Are resources fully reclaimed? |
| Crash | Are resources recoverable after owner failure? |
| Orphans | Are orphan resources detectable? |
| Accounting | Does allocated capacity remain within bounds? |
| Devices | Is device ownership explicit? |
| Storage | Are persistent and ephemeral lifetimes distinct? |
| Queues | Is capacity bounded or backpressured? |
| Recovery | Is resource state reconciled before unsafe allocation? |
| Evidence | Can allocation and reclamation be reconstructed? |

## 100. What Part LXIII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production distributed resource manager;
- universal lease/fencing infrastructure;
- complete hierarchical quotas;
- complete device ownership management;
- automatic orphan reclamation;
- universal resource accounting;
- complete preemption;
- full crash-consistent allocation transactions;
- hardware-independent resource isolation.

Those require implementation-specific and platform-specific evidence.

## 101. Transition to Part LXIV

Part LXIII establishes resource ownership and lifetime semantics.

Part LXIV should define **time, clocks, deadlines, timers, scheduling time, temporal authority, clock uncertainty, and temporal consistency across distributed NROS nodes**.

```text
Part LXII
Runtime lifecycle + supervision + recovery
        ↓
Part LXIII
Resource ownership + leases + allocation + reclamation
        ↓
Part LXIV
Time + clocks + deadlines + temporal consistency
```

## Canonical rule

> **NROS treats resource allocation as a governed ownership transition: capacity, authority, quota, lifetime, lease validity, transfer, and reclamation must remain explicit, bounded, and reconcilable across normal execution, failure, restart, and distributed authority changes.**
