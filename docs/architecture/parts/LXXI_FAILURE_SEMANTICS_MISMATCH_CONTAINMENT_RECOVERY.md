# Part LXXI — Failure Semantics, Mismatch, Fault Containment & Recovery

> **Series:** NROS Architecture Series  
> **Part:** LXXI  
> **Canonical source:** `NROS_SECURITY_AND_POLICY.md` (Parts LXXI–LXXX)  
> **Role:** Failure semantics, expected-vs-observed mismatch, fault classification, containment, recovery decisions, degraded execution, reconciliation, and safe resumption  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LXXI establishes the semantic boundary between normal execution, mismatch, fault, failure, recovery, and degraded operation.

The central rule is:

> **A failure is not merely an error value: NROS must preserve the distinction between what was expected, what was observed, what is known, what is unknown, what can be contained, and what recovery is permitted to do.**

## 2. Failure Model

```text
Expected
   ↓
Observed
   ↓
Compare
   ↓
Match / Mismatch
   ↓
Classify
   ↓
Contain
   ↓
Recover / Replan / Compensate / Abort
   ↓
Verify
   ↓
Resume / Safe Stop
```

## 3. Error

An error is an explicitly represented condition indicating that an operation did not produce its ordinary result or encountered an exceptional condition.

An error does not by itself establish the scope or consequence of a failure.

## 4. Fault

A fault is a condition capable of causing incorrect, unsafe, unavailable, or otherwise contract-violating behavior.

## 5. Failure

A failure occurs when an expected contract is not satisfied or when a fault produces a prohibited outcome.

```text
Fault
   ≠
Failure
```

A latent fault may exist without having produced a failure.

## 6. Expected State

Expected state represents what the current contract, plan, protocol, policy, or invariant predicts should occur.

## 7. Observed State

Observed state represents what the runtime can establish from available evidence.

## 8. Expected vs Observed

```text
Expected
   ↕
Observed
```

The comparison must not silently substitute one for the other.

## 9. Mismatch

A mismatch exists when observed behavior or state cannot be reconciled with the expected contract under the available evidence.

## 10. Mismatch Is Not Always Failure

Some mismatches are transient, informational, recoverable, or caused by incomplete observation.

```text
Mismatch
   ≠ automatically
Failure
```

## 11. Unknown Outcome

An operation may terminate without enough evidence to establish whether its externally visible effect occurred.

```text
Known Success
Known Failure
Outcome Unknown
```

These states must remain distinct.

## 12. Timeout Semantics

A timeout does not automatically imply that the operation failed.

```text
TIMEOUT
   ↓
OUTCOME_UNKNOWN
   ↓
RECONCILE
```

## 13. Failure Classification

NROS should classify failures sufficiently to determine safe handling.

Possible classes include:

```text
validation
authorization
resource
temporal
communication
execution
verification
state
dependency
safety
recovery
```

## 14. Validation Failure

Validation failure means the requested operation does not satisfy a required precondition or input contract.

## 15. Authorization Failure

Authorization failure means required authority cannot be established.

It must not be silently transformed into an ordinary resource failure.

## 16. Resource Failure

Resource failure includes unavailable, exhausted, corrupted, conflicting, or otherwise unusable resources.

## 17. Temporal Failure

Temporal failure includes missed deadlines, expired leases, invalid temporal assumptions, and clock-dependent contract violations.

## 18. Communication Failure

Communication failure includes transport loss, protocol violation, delivery uncertainty, and unavailable peers.

## 19. Execution Failure

Execution failure occurs when the runtime cannot complete the requested execution according to its contract.

## 20. Verification Failure

Verification failure occurs when the resulting state cannot satisfy the required verification criteria.

## 21. State Failure

State failure occurs when runtime state violates an invariant or cannot be established consistently enough to continue safely.

## 22. Dependency Failure

Dependency failure occurs when a required external or internal dependency is unavailable, incompatible, or outside its declared contract.

## 23. Safety Failure

Safety failure indicates that continuing ordinary execution could violate a safety invariant.

Safety failures require stronger containment semantics than ordinary operational failures.

## 24. Recovery Failure

Recovery failure occurs when the selected recovery procedure cannot restore the required contract or establish a safe degraded state.

## 25. Fault Containment

Fault containment limits the propagation of a fault's consequences.

```text
FAULT
 ↓
CONTAIN
 ↓
ASSESS IMPACT
 ↓
PROPAGATE ONLY NECESSARY CONSEQUENCES
```

## 26. Containment Boundary

Containment may occur at:

```text
operation
work item
queue
resource
component
runtime
node
domain
tenant
```

The narrowest safe boundary should normally be preferred.

## 27. Blast Radius

A recovery decision should consider the blast radius of continuing, retrying, isolating, or terminating the affected operation.

## 28. Isolation

When a component cannot establish safe behavior, it may need to be isolated from further work until its state is understood.

## 29. Quarantine

Quarantine is an explicit state in which an entity remains present but is prevented from participating in selected operations.

## 30. Safe Stop

Safe stop is a deliberate transition that prevents further unsafe progress while preserving enough state for diagnosis or recovery.

## 31. Abort

Abort terminates the current operation or work item without claiming successful completion.

## 32. Cancel vs Abort

Cancellation is a requested termination.

Abort is a termination caused by an execution or safety decision.

```text
Cancel
   ≠
Abort
```

## 33. Retry

Retry is a new attempt to achieve the same logical operation after an unsuccessful or uncertain attempt.

## 34. Retry Eligibility

Not every failure is retryable.

Retry eligibility depends on:

```text
failure class
idempotency
side effects
resource state
policy
attempt budget
backoff
safety
```

## 35. Retry and Unknown Outcomes

Unknown outcomes require reconciliation before retry when duplicate external effects could be harmful.

## 36. Retry Budget

Retries must be bounded.

```text
attempts ≤ configured_budget
```

unless an explicit policy permits otherwise.

## 37. Backoff

Repeated failures should normally use controlled backoff to prevent retry storms.

## 38. Retry Storm Prevention

A failed dependency must not cause unbounded synchronized retries across a large workload.

## 39. Replan

Replanning changes the execution strategy while preserving the logical objective where possible.

## 40. Compensate

Compensation performs an explicit corrective operation when an earlier operation cannot simply be undone.

## 41. Recovery

Recovery is the process of restoring sufficient state and guarantees to continue safely or to reach a defined terminal condition.

## 42. Recovery Is Not Resume

```text
Recovery
   ≠
Resume
```

Recovery establishes conditions; resume is a subsequent authorization to continue.

## 43. Recovery Ordering

A recovery sequence may require:

```text
1. Restore runtime
2. Detect external state
3. Reconcile resources
4. Reconcile authority
5. Reconcile active Work
6. Resolve unknown outcomes
7. Verify safety
8. Resume or replan
```

The exact ordering is contract-dependent, but unsafe resume must not precede required reconciliation.

## 44. External State

After restart or disconnection, locally persisted state may differ from externally observable state.

External state must therefore be detected rather than assumed.

## 45. Resource Reconciliation

Recovery must reconcile resource ownership, leases, allocations, reservations, and availability before continuing operations that depend on them.

## 46. Authority Reconciliation

Recovery must not assume that authority held before a fault remains valid.

Capabilities, leases, credentials, epochs, and policy state may require revalidation.

## 47. Work Reconciliation

Active work must be classified after recovery:

```text
not started
in progress
completed
failed
outcome unknown
cancelled
aborted
```

## 48. Duplicate Work Prevention

Recovery should prevent duplicate execution where the operation's side effects are not safely repeatable.

## 49. Idempotency

Idempotent operations may be retried when the contract guarantees equivalent externally visible effects.

Idempotency must not be inferred merely because an operation appears harmless.

## 50. Deduplication

Logical operation identity should support duplicate detection where required.

## 51. Epochs

Epochs can fence stale execution after restart, leadership change, or authority transition.

```text
OperationEpoch == CurrentEpoch
    ⇒
eligible for execution
```

## 52. Stale Authority

Authority from an obsolete epoch must not authorize protected mutations.

## 53. Degraded Mode

If full recovery is impossible, NROS may enter an explicitly defined degraded state.

## 54. Degraded State

A degraded state must identify:

```text
what remains available
what is unavailable
what guarantees are weakened
what operations are prohibited
what recovery conditions remain
```

## 55. No Silent Degradation

A system must not silently weaken safety, authorization, consistency, or durability guarantees merely to preserve availability.

## 56. Capability Reduction

Degraded execution may require reducing available capabilities.

```text
NormalAuthority
   ↓
DegradedAuthority ⊆ NormalAuthority
```

## 57. Failure Propagation

Only consequences required by the failure contract should propagate to unrelated work.

## 58. Dependency Propagation

A dependency outage may transition dependent work to:

```text
blocked
waiting
retrying
replanned
failed
```

rather than indiscriminately failing the entire runtime.

## 59. Circuit Breaking

Repeated dependency failures may trigger a circuit breaker to prevent continued unsafe or wasteful calls.

## 60. Recovery Preconditions

Recovery must define explicit preconditions before attempting resume.

## 61. Recovery Verification

A recovery operation is not successful merely because it completed without reporting an error.

Required invariants must be verified.

## 62. Verification Before Resume

```text
Recovered
   ↓
Verify
   ↓
Resume
```

not:

```text
Recovered
   ↓
Resume
   ↓
Hope
```

## 63. Safety Dominance

Where availability and safety conflict, the declared safety contract must dominate ordinary progress.

## 64. Recovery Policy

Recovery policy should determine whether to:

```text
retry
reconcile
replan
compensate
quarantine
abort
safe-stop
```

## 65. Recovery Determinism

Equivalent failure conditions should produce deterministic recovery decisions when the policy requires reproducibility.

## 66. Recovery Provenance

Recovery actions should be attributable to the triggering failure, policy, authority, and execution context where auditability is required.

## 67. Failure Correlation

Failure records should correlate with:

```text
request_id
operation_id
work_id
trace_id
resource_id
principal_id
policy_version
epoch
```

where applicable.

## 68. Failure Evidence

A failure claim should identify the evidence supporting it.

```text
Claimed Failure
    ↓
Observed Evidence
    ↓
Classification
    ↓
Verification
```

## 69. Unknown Must Remain Unknown

NROS must not convert insufficient evidence into false certainty.

```text
Unknown
    ≠
Failed
    ≠
Succeeded
```

## 70. Recovery Evidence

Recovery evidence should establish:

```text
trigger
initial state
recovery action
result
verification
final state
```

## 71. Failure State Machine

```text
NORMAL
  ↓
ANOMALY
  ↓
CLASSIFY
  ├── transient → RETRY
  ├── recoverable → RECOVER
  ├── uncertain → RECONCILE
  ├── unsafe → CONTAIN / SAFE STOP
  └── terminal → ABORT / FAIL
```

## 72. Recovery State Machine

```text
RECOVERY_REQUIRED
      ↓
CONTAINED
      ↓
STATE_RESTORED
      ↓
EXTERNAL_STATE_RECONCILED
      ↓
AUTHORITY_REVALIDATED
      ↓
WORK_RECONCILED
      ↓
SAFETY_VERIFIED
      ↓
RESUME | REPLAN | SAFE_STOP
```

## 73. Failure Aggregation

Multiple related failures may share a root cause and should not necessarily be treated as independent failures.

## 74. Root Cause vs Symptom

```text
Root Cause
    ≠
Observed Symptom
```

Diagnostics may associate symptoms with a suspected cause without claiming that the cause is verified.

## 75. Cascading Failure

NROS should detect and contain cascading failures where possible.

## 76. Recovery Storm Prevention

Recovery itself can overload a recovering system.

Recovery actions therefore require bounded concurrency, backoff, and resource budgets where appropriate.

## 77. Partial Recovery

Some components may recover while others remain unavailable.

Partial recovery must be represented explicitly.

## 78. Split-Brain Risk

Recovery and coordination mechanisms must prevent independently active authorities from making conflicting protected mutations.

## 79. Persistence Interaction

Recovery state inherits the persistence and durability semantics established by Part LXVIII.

## 80. Security Interaction

Recovery cannot bypass the authorization semantics established by Part LXX merely because the system is degraded.

## 81. Observability Interaction

Failure and recovery transitions should integrate with the observability and evidence semantics established by Part LXIX.

## 82. Temporal Interaction

Timeouts, leases, deadlines, and expiration during recovery must use the temporal semantics established by Part LXIV.

## 83. Messaging Interaction

Unknown delivery outcomes must use the delivery and deduplication semantics established by Part LXVI.

## 84. Distributed State Interaction

Recovery must reconcile distributed state according to the consistency and convergence semantics established by Part LXVII.

## 85. Formal Mismatch Invariant

```text
Mismatch(E,O)
    ⇒
E ≠ O under the declared comparison relation
```

## 86. Formal Unknown Outcome Invariant

```text
OutcomeUnknown(O)
    ⇒
¬ProvenSuccess(O) ∧ ¬ProvenFailure(O)
```

## 87. Formal Recovery Invariant

```text
Resume(W)
    ⇒
RecoveryPrerequisitesSatisfied(W)
 ∧
RequiredVerificationPassed(W)
```

## 88. Formal Containment Invariant

```text
Unsafe(W)
    ⇒
FurtherUnsafeProgress(W) is prevented
```

within the declared detection and enforcement bounds.

## 89. Formal Retry Invariant

```text
Retry(W)
    ⇒
RetryPolicyAllows(W)
 ∧
AttemptBudgetAvailable(W)
 ∧
SideEffectRiskAcceptable(W)
```

## 90. Formal Recovery Authority Invariant

```text
Recover(W)
    ⇒
RecoveryAuthorityValid(W)
```

## 91. Formal Degradation Invariant

```text
DegradedCapability
    ⊆
PreviouslyAuthorizedCapability
```

unless explicit re-authorization occurs.

## 92. Verification Matrix

| Property | Verification question |
|---|---|
| Error semantics | Are error, fault, and failure distinct? |
| Mismatch | Can expected and observed state be compared explicitly? |
| Unknown | Can unknown outcomes remain unknown? |
| Classification | Are failure classes actionable? |
| Containment | Can blast radius be limited? |
| Retry | Are retries policy-controlled and bounded? |
| Idempotency | Is repeatability established rather than assumed? |
| Reconciliation | Can external state be rediscovered? |
| Authority | Is authority revalidated after recovery? |
| Work | Can active work be reconciled? |
| Epoch | Can stale execution be fenced? |
| Degradation | Are weakened guarantees explicit? |
| Safety | Can unsafe progress be prevented? |
| Resume | Is verification required before resume? |
| Recovery | Are recovery actions attributable? |
| Cascades | Can cascading failures be contained? |
| Recovery storms | Is recovery itself bounded? |
| Persistence | Is recovery state durable as required? |
| Messaging | Are uncertain deliveries reconciled? |
| Security | Does recovery preserve authorization? |
| Observability | Are failure/recovery transitions evidenced? |

## 93. What Part LXXI Does Not Claim

This Part does not claim that the current NROS implementation already provides complete:

- failure classification;
- automatic fault containment;
- distributed outcome reconciliation;
- recovery orchestration;
- universal idempotency guarantees;
- automatic stale-authority fencing;
- safe degraded execution;
- recovery verification;
- cascading-failure containment;
- production-grade failure evidence.

Those require implementation-specific evidence.

## 94. Transition to Part LXXII

Part LXXI establishes the semantics of failure and recovery decisions.

Part LXXII should refine **failure policy, retry control, idempotency, compensation, backoff, circuit breaking, and recovery budgets**, without duplicating the foundational failure semantics established here.

```text
LXX
Security Governance / Authorization
        ↓
LXXI
Failure Semantics / Mismatch / Containment / Recovery
        ↓
LXXII
Failure Policy / Retry / Idempotency / Compensation
```

## Canonical rule

> **NROS must never confuse an error with a failure, a mismatch with a proven fault, a timeout with a failed operation, or recovery completion with permission to resume; uncertainty must remain explicit until sufficient evidence and verification establish a safe next state.**
