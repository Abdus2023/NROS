# Part LXII — Runtime Lifecycle, Supervision, Failure Containment & Recovery

> **Series:** NROS Architecture Series  
> **Part:** LXII  
> **Role:** Runtime lifecycle, process supervision, health state, failure containment, restart policy, graceful shutdown, crash handling, recovery orchestration, quarantine, and terminal-state governance  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LXI established execution isolation and trust domains. Part LXII defines how NROS creates, supervises, stops, restarts, recovers, and ultimately retires execution instances.

The central rule is:

> **NROS treats runtime lifecycle as an explicit state machine governed by observable health, policy, authority, and evidence; process existence alone does not constitute workload health or successful execution.**

## 2. Lifecycle Model

```text
Admitted
 ↓
Spawning
 ↓
Initializing
 ↓
Ready
 ↓
Running
 ↓
Degraded / Recovering
 ↓
Stopping
 ↓
Stopped
 ↓
Restarting / Quarantined / Terminal
```

## 3. Lifecycle Identity

A workload identity is distinct from an individual process instance.

```text
Workload
    ≠
Process Instance
```

A restart may create a new process while preserving logical workload identity.

## 4. Instance Identity

Every execution instance should have an explicit instance identity where lifecycle reconciliation depends on it.

```text
workload_id
instance_id
revision
```

## 5. Admission

Admission means that policy has authorized the workload to enter execution.

```text
Admitted
    ≠
Running
```

## 6. Spawn

Spawning creates the execution instance according to:

```text
identity
isolation policy
capabilities
resource limits
configuration
runtime revision
```

## 7. Initialization

Initialization establishes required runtime state before the instance becomes ready.

## 8. Ready

`Ready` means required startup conditions have been satisfied.

It does not necessarily mean the workload is currently processing work.

## 9. Running

`Running` means the instance is actively admitted and participating in execution according to its lifecycle contract.

## 10. Health

Health should be evaluated independently from process existence.

```text
Process Exists
    ≠
Healthy
```

## 11. Liveness

Liveness indicates that the execution mechanism is making progress or responding according to the liveness contract.

## 12. Readiness

Readiness indicates that the workload can safely receive its intended work.

## 13. Startup Health

Startup checks may include:

```text
configuration validation
resource availability
dependency availability
security policy establishment
schema compatibility
internal initialization
```

## 14. Dependency Health

A workload may be alive while a required dependency is unavailable.

Dependency failures should therefore be represented explicitly rather than hidden inside generic health status.

## 15. Health States

NROS may distinguish:

```text
healthy
starting
degraded
unhealthy
unknown
```

## 16. Unknown Health

Failure to obtain health evidence should not automatically be interpreted as healthy.

## 17. Health Evidence

Health assertions should identify:

```text
observer
subject
check
revision
time
result
```

## 18. Supervision

A supervisor observes runtime state and applies lifecycle policy.

```text
Supervisor
 ↓ observe
Runtime
 ↓ report
Supervisor
 ↓ decide
Lifecycle Action
```

## 19. Supervisor Authority

A supervisor should possess only the lifecycle authority required to perform its role.

## 20. Supervisor Hierarchy

Complex systems may use hierarchical supervision:

```text
Root Supervisor
 ↓
Service Supervisor
 ↓
Workload Supervisor
 ↓
Process Instance
```

## 21. Supervision Domain

Each supervisor should have an explicit set of workloads and lifecycle operations under its authority.

## 22. Failure Detection

Failure detection may use:

```text
exit status
health probes
heartbeats
timeouts
resource violations
protocol failures
security events
```

## 23. Exit Status

Process exit is evidence about an instance, not necessarily about the logical workload.

## 24. Crash

A crash is an abnormal termination or runtime failure according to the execution contract.

## 25. Crash Classification

Failures should be classified where useful:

```text
expected
transient
configuration
dependency
resource
protocol
security
internal
unknown
```

## 26. Failure Containment

A failure in one workload should not automatically propagate to unrelated workloads.

```text
Failure(W1)
    ⇏
Failure(W2)
```

unless an explicit dependency or shared-resource failure exists.

## 27. Fault Domain

Workloads should be grouped into fault domains where correlated failure behavior is expected.

## 28. Dependency Graph

Supervision should account for dependencies:

```text
A → B → C
```

A may become degraded when B fails even if A's process remains alive.

## 29. Failure Propagation

Propagation should follow declared dependency semantics rather than arbitrary process topology.

## 30. Restart Policy

Restart policy should define:

```text
whether restart is allowed
maximum attempts
backoff
reset conditions
failure classification
quarantine threshold
```

## 31. Restart Is Not Recovery

```text
Restart
    ≠
Recovery
```

Restart creates a new execution attempt; recovery establishes the required logical state and authority after failure.

## 32. Restart Backoff

Repeated failures should use bounded backoff to prevent restart storms.

## 33. Restart Jitter

Distributed workloads may require randomized or policy-defined jitter to avoid synchronized restart waves.

## 34. Restart Budget

A workload may have a bounded restart budget within a defined time window.

## 35. Restart Storm

A supervisor should detect repeated rapid failures and transition the workload toward quarantine or terminal handling rather than restarting indefinitely.

## 36. Quarantine

Quarantine isolates an unhealthy or suspicious workload from normal execution while preserving controlled investigation or recovery paths.

## 37. Quarantine Entry

Quarantine may be triggered by:

```text
repeated crash
security violation
corrupt state
integrity failure
resource abuse
unknown persistent failure
```

## 38. Quarantine Exit

Leaving quarantine requires explicit recovery criteria rather than simply waiting for time to pass.

## 39. Recovery

Recovery may include:

```text
restore state
rebuild runtime
revalidate configuration
re-establish credentials
reconcile dependencies
replay durable work
```

## 40. Recovery Ordering

Recovery operations should define ordering when dependencies exist.

```text
Foundation
 ↓
Dependencies
 ↓
State
 ↓
Service
 ↓
Workload
```

## 41. State Restoration

Restored state must pass schema, integrity, authorization, and lifecycle validation before becoming authoritative.

## 42. Checkpoint Recovery

Checkpoint restoration should validate checkpoint identity and compatibility before execution resumes.

## 43. Replay Recovery

Replay must obey idempotency and ordering rules established by the relevant workload and event contracts.

## 44. Duplicate Recovery

Recovery must avoid applying a durable side effect twice when the original outcome is uncertain.

## 45. Unknown Outcome

```text
Execute
 ↓
Connection / Process Failure
 ↓
Outcome Unknown
```

The supervisor should reconcile state before retrying non-idempotent work.

## 46. Graceful Shutdown

Shutdown should provide a controlled transition:

```text
Running
 ↓
Draining
 ↓
Stopping
 ↓
Stopped
```

## 47. Drain

Draining prevents new work while allowing eligible in-flight work to complete.

## 48. Drain Deadline

Graceful shutdown should have a bounded deadline.

## 49. Forced Termination

If graceful shutdown exceeds its deadline, policy may require forced termination.

Forced termination is a distinct lifecycle event.

## 50. Shutdown Ordering

Dependencies should define shutdown order so consumers stop before resources they require disappear.

## 51. Signal Semantics

Platform signals or control messages should map to explicit lifecycle transitions rather than being treated as arbitrary implementation details.

## 52. Cancellation

Cancellation should distinguish:

```text
requested
accepted
in-progress
completed
forced
```

## 53. Cancellation vs Failure

```text
Cancelled
    ≠
Crashed
```

The distinction affects retries, metrics, and user-visible semantics.

## 54. Terminal State

A terminal state means no further automatic lifecycle transitions are expected without explicit re-admission or operator action.

## 55. Terminal Causes

Possible terminal causes include:

```text
completed
permanent failure
policy rejection
security quarantine
resource exhaustion
manual retirement
```

## 56. Retirement

Retirement removes a workload from active lifecycle management while preserving required historical evidence.

## 57. Supervisor Restart

A supervisor may itself fail.

Supervision therefore requires a higher-level recovery strategy or durable lifecycle state.

## 58. Supervisor State

Critical supervisor state should not exist only in volatile process memory when loss would prevent safe reconciliation.

## 59. Split-Brain Supervision

Multiple supervisors must not independently exercise conflicting authority over the same workload without an explicit coordination mechanism.

## 60. Leadership

If supervision is distributed, leadership or lease semantics should establish which supervisor currently owns lifecycle authority.

## 61. Lease Expiry

A supervisor whose lease expires must stop performing operations requiring active authority.

## 62. Fencing

Lifecycle control may require fencing tokens or epochs to prevent stale supervisors from acting after leadership changes.

## 63. Stale Supervisor

```text
Old Supervisor
    ⇏
Current Lifecycle Authority
```

## 64. Lifecycle Epoch

A workload may carry a lifecycle epoch that changes after authoritative transitions.

## 65. Reconciliation

Supervisors should periodically reconcile desired state against observed state.

```text
Desired State
      ×
Observed State
      ↓
Reconciliation
```

## 66. Desired State

Desired state describes what should be running, with what configuration, revision, and policy.

## 67. Observed State

Observed state describes what the runtime currently reports or what the supervisor can independently verify.

## 68. Drift

```text
Desired State
    ≠
Observed State
```

is lifecycle drift and should trigger reconciliation.

## 69. Convergence

A supervisor should converge toward desired state subject to safety, authorization, resource, and dependency constraints.

## 70. Reconciliation Safety

Reconciliation must not blindly recreate workloads when doing so could duplicate side effects or violate singleton constraints.

## 71. Singleton Workloads

Singleton semantics require explicit identity and fencing when multiple execution attempts could exist simultaneously.

## 72. Duplicate Instances

Duplicate instances may be acceptable for stateless workloads but dangerous for workloads owning exclusive side effects.

## 73. Startup Idempotency

Initialization should be idempotent or explicitly detect previous initialization where repeated startup is possible.

## 74. Shutdown Idempotency

Repeated shutdown requests should converge toward the same terminal lifecycle state.

## 75. Recovery Idempotency

Recovery steps should be safely repeatable where possible.

## 76. Resource Cleanup

Failed or terminated instances must release or revoke their resources according to policy.

## 77. Credential Cleanup

Credentials associated with an instance may require revocation when the instance is terminated or compromised.

## 78. Temporary State

Temporary files, sockets, locks, IPC handles, and runtime metadata should not survive termination in ways that create stale authority or resource leaks.

## 79. Lock Ownership

Locks should identify their owner instance or lifecycle epoch so stale instances cannot retain logical ownership.

## 80. Health Probe Safety

Health probes must not themselves become uncontrolled attack surfaces or grant privileged capabilities.

## 81. Probe Failure

A failed health probe should be interpreted according to probe semantics rather than automatically treated as process failure.

## 82. Watchdogs

Watchdogs may detect unresponsive workloads, but watchdog actions require explicit authority and bounded behavior.

## 83. Timeout Semantics

Every lifecycle timeout should identify:

```text
clock
start condition
deadline
action
failure interpretation
```

## 84. Time Uncertainty

Distributed lifecycle decisions should account for clock skew and delayed observations.

## 85. Evidence Ordering

Lifecycle events should preserve sufficient ordering information to reconstruct transitions.

## 86. Lifecycle Event

A lifecycle event may include:

```text
workload_id
instance_id
epoch
previous_state
new_state
cause
actor
time
evidence
```

## 87. Auditability

Security-sensitive lifecycle actions should be auditable without exposing sensitive runtime data.

## 88. Observability

Metrics should distinguish:

```text
startup failures
runtime failures
restart count
recovery count
quarantine count
shutdown duration
health state
```

## 89. Alerting

Alerts should identify actionable lifecycle conditions rather than merely reporting process exits.

## 90. Backpressure

Supervisors should avoid creating unbounded lifecycle work during widespread failure.

## 91. Failure Amplification

A supervisor must prevent a local failure from causing uncontrolled restart or recovery amplification.

## 92. Dependency Recovery

Recovery order should respect dependency graphs and avoid retrying dependents while required foundations remain unavailable.

## 93. Recovery Budget

Recovery should have bounded resource and time budgets.

## 94. Human Intervention

Some failures should transition to a state requiring operator or policy-level intervention rather than endless automation.

## 95. Safety Gate

A workload must not return to `Running` until required recovery conditions are satisfied.

## 96. Formal Lifecycle Invariant

```text
StateTransition(S1 → S2)
    ⇒
GuardSatisfied
 ∧
AuthorityValid
 ∧
RequiredEvidencePresent
```

## 97. Formal Health Invariant

```text
Running(W)
    ⇒
RequiredReadiness(W)
```

but:

```text
ProcessExists(W)
    ⇏
Running(W)
```

## 98. Formal Restart Invariant

```text
Restart(W)
    ⇒
NewInstance(I2)
 ∧
OldInstance(I1) NotAuthoritative
```

## 99. Formal Reconciliation Invariant

```text
Desired(W) ≠ Observed(W)
    ⇒
Reconcile(W)
```

subject to policy and safety guards.

## 100. Formal Recovery Invariant

```text
Recover(W)
    ⇒
StateValid
 ∧
PolicyValid
 ∧
DependenciesReady
 ∧
CapabilitiesValid
```

## 101. Verification Matrix

| Property | Verification question |
|---|---|
| Lifecycle | Are states and transitions explicit? |
| Identity | Are workload and process identities distinct? |
| Admission | Is execution authorization separate from process creation? |
| Health | Is health independent of process existence? |
| Supervision | Is lifecycle authority explicit? |
| Failure | Are failures classified? |
| Restart | Are retries bounded and backoff-controlled? |
| Quarantine | Can repeated or unsafe failures be contained? |
| Recovery | Are state and dependencies revalidated? |
| Shutdown | Is graceful shutdown bounded? |
| Cancellation | Are cancellation and failure distinguished? |
| Distributed supervision | Are stale supervisors fenced? |
| Reconciliation | Is desired state compared with observed state? |
| Singleton | Are duplicate execution attempts prevented where required? |
| Cleanup | Are resources and credentials reclaimed? |
| Timeouts | Are lifecycle deadlines explicit? |
| Evidence | Can lifecycle transitions be reconstructed? |
| Amplification | Are restart/recovery storms bounded? |
| Human intervention | Can automation safely stop and escalate? |
| Recovery gate | Is `Running` blocked until recovery conditions hold? |

## 102. What Part LXII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production supervisor hierarchy;
- universal process health probes;
- complete restart-budget enforcement;
- automatic quarantine;
- distributed supervisor fencing;
- complete lifecycle reconciliation;
- universal checkpoint recovery;
- automatic dependency-aware recovery;
- complete crash containment;
- full lifecycle evidence reconstruction.

Those require implementation-specific evidence.

## 103. Transition to Part LXIII

Part LXII establishes runtime lifecycle and supervision semantics.

Part LXIII should define **resource ownership, leases, locks, quotas, allocation, reclamation, and lifetime management across runtime boundaries**.

```text
Part LXI
Isolation + sandboxing + privilege + capabilities
        ↓
Part LXII
Runtime lifecycle + supervision + recovery
        ↓
Part LXIII
Resource ownership + leases + allocation + reclamation
```

## Canonical rule

> **NROS treats execution as a governed lifecycle rather than a process existence check: admission, readiness, health, failure, restart, recovery, shutdown, quarantine, and retirement are explicit states with bounded transitions, authoritative supervision, and independently verifiable evidence.**
