# Part IX — Supervision, Recovery & Resilience

> **Series:** NROS Architecture Series  
> **Part:** IX  
> **Role:** Fault detection, supervision, recovery, isolation, escalation, and resilience  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part VIII defined scheduling and executor semantics. Part IX defines how NROS detects abnormal conditions, assigns responsibility for recovery, isolates failures, and returns entities to an explicitly valid state.

The central rule is:

> **A fault is an observation; recovery is a policy-driven state transition; successful recovery must be verified rather than assumed.**

## 2. Resilience Model

The conceptual resilience pipeline is:

```text
Normal operation
      ↓
Fault / anomaly detected
      ↓
Classify
      ↓
Contain / isolate
      ↓
Select recovery policy
      ↓
Execute recovery
      ↓
Verify
      ├── success → resume
      └── failure → escalate
```

The runtime must not collapse detection, decision, action, and verification into one opaque operation.

## 3. Fault Sources

Faults may originate from:

```text
Lifecycle
Communication
Transport
Resource
Scheduler
Executor
Device
Configuration
Dependency
Application
Platform
External environment
```

A common fault model should allow these sources to propagate without erasing their origin.

## 4. Fault Identity

A fault record should conceptually contain:

```text
Fault
├── fault_id
├── entity_id
├── generation
├── source
├── class
├── severity
├── detection time
├── evidence
├── state
└── correlation / cause
```

The exact representation is implementation-specific.

## 5. Detection vs Diagnosis

These are distinct operations.

```text
Detection
→ Something abnormal was observed.

Diagnosis
→ What condition most plausibly explains it?
```

A detector should not claim a root cause merely because a symptom was observed.

For example:

```text
Timeout detected
    ≠
Peer crashed
```

The peer may be slow, disconnected, overloaded, or otherwise unavailable.

## 6. Fault Classification

Faults can be classified by behavior:

```text
Transient
Persistent
Recoverable
NonRecoverable
Local
Dependency-induced
Cascading
Unknown
```

Classification may change as evidence accumulates.

## 7. Severity

Severity expresses impact or urgency, not necessarily cause.

A conceptual scale is:

```text
INFO
DEGRADED
WARNING
ERROR
CRITICAL
FATAL
```

The concrete severity vocabulary must be defined consistently across the runtime.

## 8. Health vs Fault

Health and fault are related but distinct.

```text
Health
→ current operational condition

Fault
→ abnormal condition/event requiring policy
```

An entity may be:

```text
RUNNING + DEGRADED
```

without being irrecoverably faulted.

## 9. Supervision

A supervisor observes one or more runtime entities and applies policy when abnormal conditions occur.

```text
Supervisor
├── observes
├── classifies
├── decides
├── invokes recovery
└── verifies outcome
```

Supervision does not necessarily mean a process hierarchy. It is a logical responsibility boundary.

## 10. Supervision Scope

A supervisor may manage:

```text
single entity
component group
resource domain
communication domain
execution domain
system subsystem
```

The scope should be explicit so that recovery authority is not ambiguous.

## 11. Failure Containment

Containment limits the impact of a failure.

Possible mechanisms include:

```text
Stop entity
Quarantine entity
Disable communication
Revoke resources
Cancel dependent work
Isolate device
Reduce operating mode
```

Containment should occur before recovery when continued operation could propagate damage or inconsistent state.

## 12. Recovery Policies

Possible policies include:

```text
Retry
Restart
Reinitialize
Reconnect
Reacquire resource
Fail over
Rollback
Restore checkpoint
Degrade functionality
Escalate
Enter safe state
```

A recovery policy should define applicability, limits, and termination conditions.

## 13. Retry

Retries require explicit bounds.

```text
Retry policy
├── maximum attempts
├── delay
├── backoff
├── retryable faults
└── terminal condition
```

Unbounded retry is not a recovery strategy; it can become a failure amplifier.

## 14. Backoff

Repeated recovery attempts may use:

```text
fixed delay
linear backoff
exponential backoff
bounded randomized backoff
```

The policy must respect system timing and resource constraints.

## 15. Restart

Restart creates a new execution generation.

```text
Entity generation 7
       ↓
restart
       ↓
Entity generation 8
```

Old operations associated with generation 7 must not silently affect generation 8.

## 16. Reinitialization

Reinitialization may restore operational state without replacing the entity identity.

Possible steps:

```text
quiesce
reset runtime state
reacquire resources
reconnect dependencies
validate configuration
resume
```

Whether persistent state survives depends on the entity contract.

## 17. Recovery Verification

Recovery is incomplete until the required postconditions are verified.

```text
Recovery action
      ↓
Expected postconditions
      ↓
Observation
      ↓
Verified
   OR
Not verified
```

For example:

```text
Restart succeeded
      ≠
Service healthy
```

The supervisor must define what evidence constitutes successful recovery.

## 18. Escalation

When recovery fails:

```text
Local recovery
      ↓ fail
Supervisor escalation
      ↓ fail
Subsystem escalation
      ↓ fail
System-level policy
      ↓
Safe state / shutdown / operator intervention
```

Escalation boundaries should be explicit.

## 19. Recovery Budget

Recovery itself consumes resources and time.

A recovery operation may therefore have:

```text
attempt budget
CPU budget
memory budget
time budget
retry budget
```

Recovery must not consume unbounded resources while the primary system is already degraded.

## 20. Dependency Failures

A component may fail because a dependency is unavailable.

```text
A
↓ requires
B
↓ requires
C
```

If C fails, the resulting state of B and A should be determined by dependency policy rather than by arbitrary local guesses.

Possible policies:

```text
propagate fault
degrade
wait
restart
substitute
isolate
```

## 21. Cascading Failures

A failure can propagate through dependencies:

```text
A → B → C → D
```

Containment should prevent one fault from producing uncontrolled secondary failures.

This may require:

```text
circuit breakers
resource limits
dependency isolation
bounded retries
backpressure
failure domains
```

## 22. Circuit Breaker

Communication or dependency access may use a circuit-breaker pattern:

```text
CLOSED
   ↓ failures exceed threshold
OPEN
   ↓ recovery interval
HALF_OPEN
   ├── success → CLOSED
   └── failure → OPEN
```

Thresholds and timing are policy, not universal constants.

## 23. Failure Domains

A failure domain identifies the boundary within which a failure may propagate.

Examples:

```text
thread
process
host
device
network segment
subsystem
```

Isolation mechanisms should align with the actual platform failure domain.

## 24. Fault Propagation

Fault propagation should preserve useful identity.

```text
Root fault
   ↓
Derived dependency fault
```

A derived fault should not be mistaken for a second independent root cause without evidence.

## 25. Recovery and Lifecycle

Recovery must use the lifecycle state machine from Part IV.

Conceptually:

```text
RUNNING
   ↓ fault
FAULTED / DEGRADED
   ↓ containment
ISOLATED
   ↓ recovery
RECOVERING
   ↓ verification
READY / RUNNING
```

The exact states may differ by entity type, but recovery must produce a defined lifecycle result.

## 26. Recovery and Resources

Recovery depends on resource availability.

```text
Fault
 ↓
Recovery request
 ↓
Resource admission
 ↓
Recovery execution
```

A system must not claim successful recovery when the resources required for recovery were never available.

## 27. Recovery and Scheduling

Recovery actions are themselves scheduled work.

They have:

```text
priority
deadline
budget
resource requirements
execution context
```

Emergency recovery may receive elevated priority, but the policy must prevent recovery work from destabilizing the rest of the system.

## 28. Recovery and Communication

Recovery may require communication with:

```text
supervisor
peer entities
resource manager
device manager
operator interface
external controller
```

Communication failures during recovery must have their own fallback policy.

## 29. Safe State

When recovery cannot establish required correctness, the system may need a safe state.

```text
Recovery exhausted
      ↓
Safety policy
      ↓
SAFE STATE
```

The safe state is domain-specific.

NROS provides the architectural mechanism for reaching and observing such a state; it does not define the physically safe behavior of every robot or device.

## 30. Operator Intervention

Some failures require human intervention.

Possible state:

```text
AWAITING_OPERATOR
```

Operator actions should be authenticated, authorized, auditable, and generation-aware where they can alter runtime state.

## 31. Fault Suppression

Repeated identical faults may be aggregated for observability, but suppression must not erase safety-relevant events.

Conceptually:

```text
1000 repeated faults
       ↓
aggregation
       ↓
summary + counters
```

The underlying evidence should remain available where required for diagnosis.

## 32. Recovery Storms

Multiple entities may recover simultaneously.

```text
A fails
B fails
C fails
 ↓
A+B+C all restart
 ↓
resource contention
 ↓
secondary failures
```

Recovery admission, budgets, jitter, and concurrency limits can prevent recovery storms.

## 33. Idempotency

Recovery operations should be idempotent where possible.

```text
reset()
reset()
```

should not produce an increasingly corrupted state merely because the operation was requested twice.

Operations that cannot be idempotent must define duplicate-request behavior.

## 34. Recovery Journal

Recovery actions should be observable through structured records:

```text
RecoveryRecord
├── recovery_id
├── fault_id
├── entity_id
├── generation
├── policy
├── attempt
├── start
├── end
├── action
├── result
└── evidence
```

This supports diagnosis and post-failure verification.

## 35. Verification Matrix

| Property | Verification question |
|---|---|
| Detection | Can abnormal conditions be detected according to policy? |
| Classification | Are fault classes assigned from observable evidence? |
| Ownership | Is recovery authority unambiguous? |
| Containment | Can failures be isolated before uncontrolled propagation? |
| Retry | Are retry counts and termination conditions bounded? |
| Restart | Does restart create a new generation? |
| Stale operations | Are old-generation commands rejected? |
| Recovery | Are required postconditions actually verified? |
| Escalation | Does failed recovery reach a defined escalation path? |
| Dependencies | Are dependency failures propagated according to policy? |
| Resources | Is recovery resource admission enforced? |
| Scheduling | Is recovery work scheduled under explicit policy? |
| Storm control | Can concurrent recovery overload be bounded? |
| Safe state | Is safe-state entry observable and policy-driven? |
| Audit | Are recovery actions reconstructable from evidence? |

## 36. What Part IX Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- automatic fault diagnosis;
- universal process supervision;
- automatic restart;
- guaranteed recovery;
- complete failure isolation;
- Byzantine fault tolerance;
- safety certification;
- guaranteed safe-state behavior for arbitrary hardware;
- autonomous operator replacement.

Those claims require implementation, platform, domain, and verification evidence.

## 37. Transition to Part X

Part IX establishes resilience and recovery.

Part X should define **configuration, identity, discovery, and dependency resolution**: how runtime entities are named, configured, located, matched, and connected without making discovery itself equivalent to readiness or health.

```text
Part VIII
Scheduling + executor
        ↓
Part IX
Supervision + recovery
        ↓
Part X
Identity + configuration + discovery
```

## Canonical rule

> **NROS treats supervision as an explicit control responsibility: detect, classify, contain, recover, verify, and escalate; successful recovery is an evidence-backed state transition, not an assumption.**
