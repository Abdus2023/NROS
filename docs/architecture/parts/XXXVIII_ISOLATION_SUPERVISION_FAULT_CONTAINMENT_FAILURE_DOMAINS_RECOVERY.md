# Part XXXVIII — Isolation, Supervision, Fault Containment, Failure Domains & Recovery

> **Series:** NROS Architecture Series  
> **Part:** XXXVIII  
> **Role:** Execution isolation, fault/error semantics, supervision, failure domains, containment, health, restart, escalation, and recovery  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXXVII established finite-resource and overload semantics. Part XXXVIII defines how NROS contains failures and supervises execution so that faults do not propagate without bounds.

The central rule is:

> **NROS treats failure as a scoped state transition: faults are detected, classified, contained, supervised, and recovered according to explicit failure domains, authority boundaries, restart policies, and health contracts. A restart is not itself recovery, and a timeout is not proof of failure.**

## 2. Fundamental Distinctions

```text
failure
  ≠
fault
  ≠
error
  ≠
unhealthy
  ≠
crash
  ≠
timeout
  ≠
restart
  ≠
recovery
```

## 3. Fault

A fault is an abnormal condition capable of violating an execution invariant:

```text
Fault
 ↓
Detection
 ↓
Classification
 ↓
Containment / Recovery
```

A fault may exist before a visible failure occurs.

## 4. Error

An error is an observed condition reported by an operation or component.

An error does not automatically imply component failure.

```text
recoverable error
 ≠
component fault
```

## 5. Failure

Failure means the component or operation can no longer satisfy its declared contract.

The scope must be explicit:

```text
task failure
worker failure
agent failure
node failure
service failure
control-plane failure
```

## 6. Crash

A crash is an abrupt termination or loss of execution continuity.

Crash detection and state recovery are separate concerns.

## 7. Timeout

A timeout means an expected observation did not occur within a defined temporal boundary.

```text
Timeout
  ≠
Proof of crash
```

Part XXXVI governs the temporal semantics.

## 8. Health

Health is a contract about whether a component can perform its declared role:

```text
Healthy
Degraded
Unhealthy
Unknown
```

Health must be defined operationally rather than reduced to process existence.

## 9. Liveness

Liveness concerns whether execution continues to make progress.

A process can be alive while making no useful progress.

```text
Alive
  ≠
Progressing
```

## 10. Readiness

Readiness means the component is currently able to accept work.

```text
Healthy
  ≠
Ready
```

A healthy component may intentionally be unavailable for admission during recovery or reconfiguration.

## 11. Failure Domain

A failure domain is the boundary within which a fault may propagate:

```text
Task
 ↓
Worker
 ↓
Process
 ↓
Node
 ↓
Zone
 ↓
Cluster
```

The architecture should minimize unnecessary propagation across domains.

## 12. Isolation Boundary

Isolation limits what one execution unit can affect:

```text
Execution A | Boundary | Execution B
```

Isolation may be implemented through processes, containers, sandboxes, capabilities, namespaces, resource controls, or hardware boundaries.

## 13. Isolation Dimensions

Isolation can cover:

```text
memory
CPU
filesystem
network
identity
credentials
IPC
resources
control authority
```

Part XXIX governs the broader capability and containment model.

## 14. Fault Containment

```text
Fault in A
    ↓
Detect
    ↓
Contain
    ↓
Protect B
```

Containment should occur before uncontrolled propagation whenever possible.

## 15. Containment Levels

Possible levels include:

```text
operation
attempt
task
workflow
agent
worker
process
node
cluster
```

The smallest safe containment scope should normally be preferred.

## 16. Supervisor

A supervisor owns responsibility for observing and managing subordinate execution:

```text
Supervisor
 ├─ Worker A
 ├─ Worker B
 └─ Worker C
```

Supervision includes health observation, restart policy, escalation, and terminal-state handling.

## 17. Supervision Tree

```text
Root Supervisor
 ├─ Control Supervisor
 │   ├─ Worker A
 │   └─ Worker B
 └─ Agent Supervisor
     ├─ Agent A
     └─ Agent B
```

The hierarchy should align with failure domains and ownership.

## 18. Supervisor Authority

A supervisor must have explicit authority to:

```text
observe
stop
restart
quarantine
escalate
reassign
```

Authority must remain constrained by authorization and isolation policy.

## 19. Restart

Restart replaces or reinitializes failed execution:

```text
Failed
 ↓
Restart
 ↓
Initializing
 ↓
Ready / Failed
```

Restart does not prove that durable state has been recovered.

## 20. Restart Policy

A policy may define:

```text
max restarts
restart window
backoff
jitter
failure classes
escalation threshold
```

Unlimited restart loops are prohibited as a safe default.

## 21. Restart Storm

```text
Failure
 ↓
Restart
 ↓
Immediate failure
 ↓
Restart
 ↓
Resource exhaustion
```

Supervision must bound restart frequency and cooperate with Part XXXVII's resource controls.

## 22. Restart Backoff

Restart delay may use:

```text
exponential backoff
+ jitter
+ maximum delay
```

The policy must also respect workflow and recovery deadlines.

## 23. Crash Loop

A component repeatedly failing startup enters a crash-loop condition:

```text
Start
 ↓
Crash
 ↓
Restart
 ↓
Crash
```

The supervisor should eventually stop restarting and escalate or quarantine the component.

## 24. Escalation

```text
Task failure
 ↓
Worker supervisor
 ↓
Agent supervisor
 ↓
Node supervisor
 ↓
Control plane
```

Escalation should occur only when the local supervisor cannot safely resolve the fault.

## 25. Failure Classification

Failures should be classified where useful:

```text
transient
permanent
configuration
resource
dependency
authorization
protocol
integrity
unknown
```

Classification determines recovery policy.

## 26. Transient Failure

A transient failure may be retried:

```text
Transient
 ↓
bounded retry
 ↓
recover
```

Retry must obey Part XXXV and Part XXXVII constraints.

## 27. Permanent Failure

A permanent failure should not trigger unlimited retries:

```text
Permanent
 ↓
Fail
 ↓
Escalate / Compensate
```

## 28. Dependency Failure

```text
Service A
   ↓
Dependency B unavailable
```

A should not repeatedly overload B while B is unhealthy.

Circuit-breaking and admission controls may be used.

## 29. Circuit Breaker

```text
Closed
 ↓ failure threshold
Open
 ↓ cooldown
Half-Open
 ↓ successful probe
Closed
```

The state machine must define probe concurrency and recovery semantics.

## 30. Bulkhead Isolation

Independent workloads may be assigned separate resource pools:

```text
Pool A | Pool B | Pool C
```

Failure in one pool should not automatically exhaust another.

## 31. Failure Amplification

A small fault can become systemic through:

```text
retry
queue growth
resource exhaustion
cascading dependency failure
```

NROS must treat these as coupled failure mechanisms.

## 32. Fault Containment and Backpressure

```text
Dependency degraded
 ↓
Circuit / backpressure
 ↓
Reduced demand
 ↓
Recovery opportunity
```

## 33. Health Checks

Health checks may test:

```text
process liveness
internal progress
resource viability
dependency availability
protocol readiness
state consistency
```

A single heartbeat is insufficient for every health contract.

## 34. Health Check Cost

Health checking consumes resources.

Checks must be bounded to avoid turning monitoring into another overload source.

## 35. Failure Detection Delay

Detection is not instantaneous:

```text
Fault occurs
 ↓
Observation delay
 ↓
Detection
 ↓
Containment
```

Temporal assumptions must be explicit.

## 36. False Positive

A healthy component may be classified as failed because of:

```text
network partition
clock issues
overload
slow scheduling
observer failure
```

Recovery policies must account for false positives.

## 37. False Negative

A component may appear healthy while violating its functional contract.

Functional health signals should therefore complement process-level signals where required.

## 38. Observer Failure

The supervisor itself can fail:

```text
Supervisor
 ↓
Crash
```

Supervision must therefore have a higher-level recovery mechanism or a defined limitation.

## 39. Supervisor Hierarchy Invariant

```text
Supervisor failure
    ⇒
Higher-level supervisor or external authority can recover it
```

where the architecture claims hierarchical supervision.

## 40. Orphaned Work

A failed worker may leave work without an active owner:

```text
Worker fails
 ↓
Task orphaned
 ↓
Ownership reconciliation
 ↓
Resume / Retry / Cancel / Compensate
```

Part XXXIV state reconciliation and Part XXXV workflow semantics govern the resulting transitions.

## 41. In-Flight Effects

A worker crash does not prove that its external effect did not occur:

```text
Request sent
 ↓
Worker crashes
```

The external operation may have committed.

Recovery must therefore use idempotency, reconciliation, or durable effect records where required.

## 42. Crash Recovery

```text
Crash
 ↓
Recover durable state
 ↓
Validate
 ↓
Reconcile in-flight work
 ↓
Resume / Fail / Compensate
```

## 43. Recovery Is Not Restart

```text
Restart = recreate execution
Recovery = restore valid system state
```

A restart may be one step inside recovery.

## 44. Recovery Point

A component may recover from a checkpoint:

```text
Checkpoint N
 ↓
Failure
 ↓
Restore N
```

Checkpoint semantics belong to the persistence and workflow contracts.

## 45. Recovery Epoch

Recovery may establish a new generation:

```text
Worker epoch 7
 ↓ failure
Worker epoch 8
```

Stale messages from earlier epochs must be rejected when they could cause unsafe effects.

## 46. Fencing During Recovery

```text
Old worker token
      ↓
REJECT

New worker token
      ↓
ACCEPT
```

This connects supervision with Part XXXVI lease and fencing semantics.

## 47. Quarantine

A repeatedly unsafe component may enter quarantine:

```text
Failed
 ↓
Quarantined
 ↓
Diagnostic / Repair
 ↓
Revalidated
 ↓
Admitted
```

Quarantine prevents automatic resurrection of known-bad execution.

## 48. Quarantine Conditions

Possible triggers:

```text
crash-loop
integrity violation
resource abuse
protocol violation
repeated recovery failure
unknown unsafe state
```

## 49. Dependency Health

Dependency state should be represented explicitly:

```text
Available
Degraded
Unavailable
Unknown
```

Consumers should not infer availability solely from stale cached observations.

## 50. Graceful Shutdown

```text
Running
 ↓
Stop admission
 ↓
Drain
 ↓
Cancel remaining work
 ↓
Release resources
 ↓
Stopped
```

Shutdown is distinct from crash recovery.

## 51. Drain Semantics

A draining component should define whether it:

```text
finishes accepted work
rejects new work
allows new internal work
cancels long-running work
```

## 52. Termination Authority

Only authorized supervisors or owners should be able to force termination.

Termination authority must respect isolation and capability boundaries.

## 53. Failure Domain Mapping

Every critical component should map to a failure domain:

```text
Component
 ↓
Domain
 ↓
Containment policy
 ↓
Recovery authority
```

## 54. Correlated Failures

Multiple failures may share a common cause:

```text
Node failure
 ↓
Worker A fails
Worker B fails
Worker C fails
```

The supervisor must avoid treating every symptom as an independent fault.

## 55. Failure Correlation

Correlation may use:

```text
node
zone
resource
dependency
time window
failure signature
```

## 56. Blast Radius

A fault's blast radius should be measurable:

```text
single task
 → workflow
 → tenant
 → node
 → cluster
```

Architecture should minimize unnecessary expansion.

## 57. Recovery Ordering

Recovery may require dependencies first:

```text
Storage
 ↓
Control plane
 ↓
Workers
 ↓
Agents
 ↓
User workloads
```

The exact ordering is deployment-specific but must be explicit where required.

## 58. Recovery Admission

Recovered components should not immediately receive unrestricted traffic:

```text
Recovered
 ↓
Health validation
 ↓
Limited admission
 ↓
Observe
 ↓
Full admission
```

## 59. Recovery Storm

Many components recovering simultaneously can overload dependencies.

Recovery therefore participates in resource admission and scheduling.

## 60. Recovery Jitter

Staggered restart and recovery times reduce synchronized load spikes.

## 61. Recovery Deadline

Recovery itself may have a deadline:

```text
Recovery Start
 ↓
Recovery Deadline
```

If recovery cannot complete, escalation or safe degradation may be required.

## 62. Safety During Degradation

Degraded operation must preserve:

```text
authorization
isolation
integrity
resource bounds
state invariants
```

## 63. Unknown State

If the system cannot determine whether a component or effect is safe:

```text
Unknown
 ↓
Do not assume success
 ↓
Reconcile / Quarantine / Escalate
```

Unknown must not silently become healthy.

## 64. Error Budget

Operational systems may define bounded tolerated failure rates:

```text
Error Budget
 ↓
Normal operation
 ↓
Budget exhausted
 ↓
Reduce risky changes / increase recovery focus
```

The exact policy is deployment-specific.

## 65. Fault Injection

Verification may intentionally introduce:

```text
crash
latency
packet loss
resource exhaustion
dependency failure
corrupted state
```

to test containment and recovery claims.

## 66. Recovery Testing

A recovery claim should be backed by evidence such as:

```text
fault injected
 ↓
observed detection
 ↓
observed containment
 ↓
observed recovery
 ↓
post-recovery invariant check
```

## 67. Observability

Failure diagnostics should expose:

```text
failure ID
component ID
failure domain
classification
supervisor
restart count
epoch/fencing token
health state
recovery state
```

Sensitive information remains subject to Part XXIX controls.

## 68. Auditability

Critical supervisory actions should produce durable records when required:

```text
fault detected
restart requested
restart executed
quarantined
recovered
escalated
```

## 69. Formal Containment Invariant

```text
Fault(A)
    ⇒
EffectsOutsideDeclaredDomain(A)
    are prohibited
```

except through explicitly authorized propagation paths.

## 70. Formal Restart Invariant

```text
RestartCount(Component, Window)
    ≤
ConfiguredRestartLimit
```

## 71. Formal Recovery Invariant

```text
Recovered(Component)
    ⇒
DeclaredStateInvariants(Component)
```

must hold before unrestricted admission.

## 72. Formal Stale-Actor Invariant

```text
Epoch(old) < Epoch(current)
    ⇒
AuthoritativeEffect(old) = Reject
```

where fencing is required.

## 73. Formal Health Invariant

```text
Ready(Component)
    ⇒
CanAcceptDeclaredWork(Component)
```

## 74. Verification Matrix

| Property | Verification question |
|---|---|
| Fault model | Are fault, error, failure, and timeout distinguished? |
| Isolation | Is the containment boundary explicit? |
| Failure domain | Is blast radius defined? |
| Supervision | Is supervisory authority explicit? |
| Health | Are liveness and readiness distinct? |
| Restart | Are restart loops bounded? |
| Backoff | Is restart pressure controlled? |
| Escalation | Is unresolved failure propagated safely? |
| Recovery | Is durable state validated before admission? |
| Fencing | Are stale actors rejected? |
| Quarantine | Can unsafe components be prevented from re-entering? |
| Dependencies | Are dependency failures contained? |
| Shutdown | Are drain and termination semantics explicit? |
| Recovery storm | Is simultaneous recovery bounded? |
| Unknown state | Is uncertainty handled safely? |
| Observability | Are supervisory decisions diagnosable? |
| Fault injection | Can recovery claims be tested? |
| Formal assurance | Are containment and recovery invariants explicit? |

## 75. What Part XXXVIII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a universal supervision tree;
- production-grade crash recovery;
- complete failure-domain isolation;
- automatic fault classification;
- guaranteed zero-downtime recovery;
- universal circuit breakers;
- complete fencing enforcement;
- formally verified containment;
- comprehensive fault-injection coverage.

Those require implementation-specific evidence.

## 76. Transition to Part XXXIX

Part XXXVIII establishes fault containment and supervision.

Part XXXIX should define **configuration, control, dynamic reconfiguration, rollout, version activation, feature flags, policy distribution, and safe change management**, connecting stable runtime behavior with controlled evolution.

```text
Part XXXVII
Resources + quotas + admission + pressure + overload
        ↓
Part XXXVIII
Isolation + supervision + fault containment + recovery
        ↓
Part XXXIX
Configuration + control + reconfiguration + rollout + safe change
```

## Canonical rule

> **NROS contains faults within explicit failure domains, supervises execution through bounded policies, rejects stale authorities, distinguishes restart from recovery, and admits recovered components only after their declared invariants and safety conditions have been re-established.**
