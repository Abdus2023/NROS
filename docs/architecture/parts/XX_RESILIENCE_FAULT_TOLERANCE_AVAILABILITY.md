# Part XX — Resilience, Fault Tolerance & Availability

> **Series:** NROS Architecture Series  
> **Part:** XX  
> **Role:** Fault models, failure domains, detection, containment, degradation, recovery, redundancy, recovery objectives, availability, and end-to-end resilience  
> **Status:** Architectural design document — not operational resilience evidence

## 1. Purpose

Part XIX defined formal models, invariants, and proof boundaries. Part XX defines how NROS behaves when faults occur and how the system detects, contains, degrades, recovers, and verifies recovery.

The central rule is:

> **A resilient system does not merely survive faults; it defines the fault model, detects relevant failures, limits propagation, enters a valid degraded state, performs bounded recovery where possible, and produces evidence that the recovered state satisfies its contract.**

## 2. Fundamental Distinctions

```text
Fault
  ≠
Error
  ≠
Failure
  ≠
Degradation
  ≠
Recovery
  ≠
Availability
  ≠
Resilience
```

### Fault
An underlying abnormal condition.

### Error
An incorrect internal state or observed condition caused by a fault.

### Failure
An externally visible violation of an expected service or contract.

### Degradation
A controlled reduction of service while maintaining defined guarantees.

### Recovery
Transition toward a valid operating state after failure.

### Availability
The ability to provide the specified service when required.

### Resilience
The capability to continue providing acceptable service and recover appropriately under defined disturbances.

## 3. Fault Lifecycle

```text
Fault
 ↓
Detection
 ↓
Diagnosis
 ↓
Containment
 ↓
Degradation / isolation
 ↓
Recovery
 ↓
Verification
 ↓
Restored / reconfigured state
```

Not every fault is recoverable. The contract must define terminal behavior where recovery is impossible.

## 4. Fault Model

NROS components should declare relevant fault classes:

```text
crash
hang
timeout
corruption
resource exhaustion
message loss
duplication
reordering
partition
storage failure
hardware failure
configuration error
operator error
security violation
```

A resilience claim is meaningful only relative to a stated fault model.

## 5. Fault Assumptions

The architecture should state what is outside the modeled fault set.

Examples:

```text
Byzantine behavior excluded
cryptographic primitives trusted
hardware assumed non-malicious
clock drift bounded
storage corruption detectable
```

An unmodeled fault must not be silently treated as handled.

## 6. Failure Domains

Failures should be classified by scope:

```text
operation
entity
thread/task
process
component
node
rack / zone
network segment
storage domain
cluster
external dependency
```

Containment should prevent a lower-level failure from unnecessarily becoming a higher-level failure.

## 7. Fault Containment

The desired structure is:

```text
Fault
 ↓
Local containment
 ↓
Local recovery
```

before escalation:

```text
Local failure
 ↓ if unrecoverable
Supervisor
 ↓
Replacement / failover
 ↓
Higher-level recovery
```

This connects directly to Part IX supervision.

## 8. Failure Propagation

NROS should explicitly model propagation paths:

```text
component A
   ↓ dependency
component B
   ↓ shared resource
component C
```

A resilient architecture identifies shared failure causes and avoids accidental correlated failure where required.

## 9. Dependency Failures

External dependencies can fail independently:

```text
DNS
network
storage
identity service
policy service
remote API
hardware
```

Each critical dependency should have defined timeout, fallback, degradation, or fail-stop semantics.

## 10. Failure Detection

Detection mechanisms may include:

```text
heartbeat
watchdog
timeout
health probe
supervision signal
invariant violation
resource threshold
protocol error
storage verification
```

Detection itself has failure modes and false-positive/false-negative characteristics.

## 11. Detection Latency

A detection mechanism introduces latency:

```text
fault occurs
   ↓
detection latency
   ↓
recovery begins
```

Detection bounds should be explicit when they are part of a service objective.

## 12. Diagnosis

Detection identifies that something is wrong; diagnosis attempts to identify what failed.

```text
Detected failure
      ↓
Localization
      ↓
Classification
      ↓
Recovery decision
```

Recovery must not depend on perfect diagnosis unless the contract guarantees diagnostic accuracy.

## 13. Failure Classification

Failures may be classified as:

```text
transient
intermittent
persistent
permanent
local
distributed
recoverable
non-recoverable
suspected malicious
```

Classification may change as new evidence arrives.

## 14. Timeouts

Timeouts are failure detectors, not proofs of failure.

```text
No response before T
      ⇒
assume failure / uncertainty
```

The distinction between failure and uncertainty is important in distributed systems.

## 15. Retries

Retries should be controlled by:

```text
attempt limit
backoff
jitter
time budget
idempotency
error classification
circuit state
```

Retries can amplify failures if not bounded.

## 16. Idempotency

Recovery mechanisms should identify operations safe to repeat.

```text
retry(operation)
```

must not unintentionally duplicate irreversible side effects unless duplication is explicitly part of the contract.

## 17. Backoff

Retry schedules should avoid synchronized load spikes:

```text
failure
 ↓
backoff
 ↓
retry
 ↓
backoff increase
```

Jitter may be used where synchronization risk exists.

## 18. Circuit Breaking

Repeated dependency failure may trigger:

```text
CLOSED
 ↓ failures
OPEN
 ↓ cooldown
HALF_OPEN
 ↓ successful probe
CLOSED
```

The state machine must define admission and recovery semantics.

## 19. Degraded Operation

When full service cannot be maintained, NROS may enter a declared degraded mode:

```text
FULL
 ↓ fault
DEGRADED
 ↓ recovery
FULL
```

The degraded state must have explicit guarantees and limits.

## 20. Graceful Degradation

Degradation should preserve the most important invariants first:

```text
mandatory safety
      ↓
core functionality
      ↓
optional functionality
      ↓
non-critical observability / convenience
```

The actual priority order is domain-specific and must be specified.

## 21. Fail-Open vs Fail-Closed

Different failures require different defaults:

```text
fail-open
fail-closed
fail-safe
degrade
stop
```

Security-sensitive decisions generally require explicit fail-closed semantics where mandated by Part XI.

## 22. Isolation

Isolation mechanisms can include:

```text
process boundary
resource quota
namespace
capability boundary
network isolation
storage isolation
failure domain separation
```

Isolation reduces blast radius but does not eliminate correlated dependencies.

## 23. Redundancy

Redundancy may be applied through:

```text
replication
standby
active-active
active-passive
multi-path
redundant storage
redundant network
```

Redundancy only improves resilience when the failure assumptions and independence conditions are satisfied.

## 24. Independence

Two replicas sharing the same failure domain are not fully independent:

```text
Replica A ─┐
           ├─ same failure domain
Replica B ─┘
```

Part XV placement constraints therefore contribute directly to resilience.

## 25. Failover

Failover is a transition from a failed or unavailable instance to another service instance:

```text
Primary
  ↓ failure
Detection
  ↓
Secondary activation
  ↓
Verification
  ↓
Service restored
```

The handoff must define state synchronization and ownership semantics.

## 26. Split-Brain Prevention

Distributed failover must address simultaneous ownership:

```text
A believes it is primary
B believes it is primary
```

Possible mechanisms include:

```text
leases
fencing
quorum
consensus
external arbitration
```

The selected mechanism must be appropriate to the failure model.

## 27. Fencing

When stale instances can continue producing side effects, recovery may require fencing them before activating a replacement.

```text
old owner
 ↓
fence
 ↓
new owner
```

This connects directly to Part X identity/generation safety.

## 28. Generational Recovery

Recovery should prevent stale work:

```text
generation N
   ↓ failure
recovery
   ↓
generation N+1
```

Work associated with N must not be accepted by N+1 unless explicitly permitted.

## 29. Checkpointing

Recovery may use checkpoints:

```text
running state
   ↓
checkpoint
   ↓
continued execution
```

After failure:

```text
checkpoint
   ↓
restore
   ↓
replay / reconcile
```

Checkpoint correctness is governed by Part XII persistence semantics.

## 30. Recovery Point Objective

RPO describes acceptable loss of committed or recoverable state:

```text
failure
 ←──── RPO ────→
latest recoverable state
```

The exact definition must specify which state is considered recoverable.

## 31. Recovery Time Objective

RTO describes the target time to restore the specified service level:

```text
failure
   ↓
Detection
 + diagnosis
 + recovery
 + verification
   ↓
service restored
```

RTO claims require measurable conditions.

## 32. Recovery Verification

Recovery is incomplete until the restored state is checked:

```text
Recover
  ↓
Validate state
  ↓
Validate policy
  ↓
Validate resources
  ↓
Validate dependencies
  ↓
Resume service
```

A process restart is not itself evidence of successful recovery.

## 33. Reconciliation

After failure, desired and observed state may diverge:

```text
Desired state
     ↓
Reconciliation
     ↓
Observed state
```

Part XVII configuration convergence and Part XV deployment reconciliation apply here.

## 34. Recovery Loops

A supervisor may follow:

```text
observe
 ↓
detect
 ↓
classify
 ↓
recover
 ↓
verify
 ↓
observe
```

The loop must have bounded behavior to prevent endless recovery storms.

## 35. Recovery Storms

Repeated simultaneous failures can overload recovery mechanisms:

```text
failure storm
 ↓
restarts
 ↓
resource pressure
 ↓
additional failures
```

Controls may include:

```text
restart budgets
backoff
rate limits
admission control
staggered recovery
```

## 36. Recovery Budgets

Part VII resource budgets can constrain recovery:

```text
max restart rate
max concurrent recovery
max failover attempts
max replay volume
max recovery memory
```

Recovery must not consume unlimited resources.

## 37. Observability During Failure

Part XIV should expose enough evidence to distinguish:

```text
fault detected
recovery attempted
recovery succeeded
recovery verified
service restored
```

These are separate events.

## 38. Availability

Availability should be defined relative to an agreed service contract.

Conceptually:

```text
availability
= acceptable service time
  /
  required service time
```

The definition must specify what counts as acceptable service.

## 39. Availability vs Health

A component can report healthy while the end-to-end service is unavailable.

```text
Component health
      ≠
End-to-end availability
```

Availability therefore requires service-level observation.

## 40. Resilience Envelope

Every resilience claim should identify:

```text
faults tolerated
faults not tolerated
maximum degraded duration
recovery target
state loss tolerance
availability target
assumptions
```

This defines the system's resilience envelope.

## 41. End-to-End Resilience

End-to-end resilience is limited by the weakest critical dependency:

```text
Client
 ↓
NROS runtime
 ↓
network
 ↓
storage
 ↓
external dependency
```

A locally resilient component does not imply a resilient end-to-end service.

## 42. Formal Resilience Properties

Part XIX can express properties such as:

```text
fault within modeled class
    ⇒
unsafe state remains unreachable
```

and, under liveness assumptions:

```text
recoverable fault
    ⇒
eventually restored valid state
```

The assumptions must be explicitly recorded.

## 43. Resilience Testing

Part XVIII should verify resilience through:

```text
fault injection
failover tests
restart tests
partition tests
resource exhaustion
storage recovery
corruption handling
reconciliation tests
recovery timing
```

## 44. Resilience Evidence

Evidence should include where applicable:

```text
fault injection record
detection time
recovery timeline
state before failure
state after recovery
RPO measurement
RTO measurement
availability measurement
logs/traces
configuration baseline
```

## 45. Verification Matrix

| Property | Verification question |
|---|---|
| Fault model | Are tolerated faults explicitly defined? |
| Containment | Is blast radius bounded? |
| Detection | Is detection behavior measurable? |
| Diagnosis | Is classification sufficiently reliable for recovery? |
| Degradation | Are degraded guarantees explicit? |
| Retry | Are retries bounded and idempotency-aware? |
| Isolation | Are failure domains separated where required? |
| Redundancy | Are redundant instances actually independent? |
| Failover | Is ownership transferred safely? |
| Split brain | Can simultaneous ownership occur? |
| Recovery | Is recovery state explicitly verified? |
| RPO | Is acceptable state loss defined and measured? |
| RTO | Is restoration time defined and measured? |
| Reconciliation | Can desired and observed state converge after failure? |
| Recovery storms | Are restart/recovery rates bounded? |
| Availability | Is availability measured end-to-end? |
| Formal assurance | Are resilience properties tied to explicit assumptions? |
| Evidence | Is recovery backed by attributable evidence? |

## 46. What Part XX Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- automatic fault tolerance for every component;
- zero-loss recovery;
- zero-downtime failover;
- consensus-backed failover;
- universal Byzantine fault tolerance;
- a specific RPO/RTO target;
- measured availability guarantees;
- complete end-to-end resilience.

Those require implementation, deployment, and empirical evidence.

## 47. Transition to Part XXI

Part XX defines resilience under failure.

Part XXI should define **resource economics, capacity planning, admission control, quotas, backpressure, overload behavior, and sustainable operation**, connecting Part VII's resource model with Part XIII flow control and Part XX recovery behavior.

```text
Part XIX
Formal models + invariants + proof boundaries
        ↓
Part XX
Resilience + fault tolerance + availability
        ↓
Part XXI
Capacity + admission + overload + resource economics
```

## Canonical rule

> **NROS treats resilience as a bounded contract over an explicit fault model: faults are detected, contained, isolated, degraded, recovered, and verified according to declared semantics, while recovery itself remains subject to resource, identity, persistence, security, and observability constraints.**
