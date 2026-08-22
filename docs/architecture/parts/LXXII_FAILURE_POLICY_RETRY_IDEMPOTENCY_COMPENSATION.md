# Part LXXII — Failure Policy, Retry, Idempotency, Compensation & Recovery Control

> **Series:** NROS Architecture Series  
> **Part:** LXXII  
> **Role:** Failure-response policy, retry control, idempotency, backoff, budgets, compensation, circuit breaking, and controlled recovery attempts  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LXXI established failure semantics: expected versus observed behavior, mismatch classification, uncertainty, containment, recovery, safe stop, and verification. Part LXXII defines what NROS should do **after a failure or uncertain outcome has been classified**.

The central rule is:

> **Failure handling must be policy-controlled: retries are attempts, not guarantees; idempotency must be established before repetition; compensation must be explicit; and recovery must remain bounded by authority, time, resources, and safety policy.**

## 2. Failure-Response Model

```text
Failure / Unknown
 ↓
Classify
 ↓
Policy Lookup
 ↓
Choose Response
 ├─ Retry
 ├─ Reconcile
 ├─ Compensate
 ├─ Replan
 ├─ Degrade
 ├─ Quarantine
 └─ Abort / Safe Stop
```

## 3. Failure Policy

A failure policy maps a classified condition to an allowed response set.

```text
FailureClass + Context + PolicyVersion
    →
ResponsePolicy
```

## 4. Policy Explicitness

NROS must not infer unlimited retry or recovery behavior from the mere existence of a failure.

## 5. Retry Is an Attempt

```text
RetryRequested
    ≠
RetrySucceeded
```

Each attempt requires its own outcome.

## 6. Retry Identity

A logical operation and its attempts should remain distinguishable.

```text
operation_id
attempt_id
```

## 7. Retry Budget

Retries require explicit bounds.

Possible limits include:

```text
max_attempts
max_elapsed_time
max_cost
max_resource_consumption
max_consecutive_failures
```

## 8. Retry Exhaustion

When a retry budget is exhausted, NROS must transition to the policy-defined next action rather than retry indefinitely.

## 9. Backoff

Repeated failures should normally use controlled backoff when immediate repetition would increase load or collision probability.

## 10. Exponential Backoff

An exponential strategy may be used where appropriate:

```text
Delay(n) = min(Cap, Base × Factor^n)
```

The exact strategy is policy-dependent.

## 11. Jitter

Distributed retry should use jitter where synchronized retries could produce a thundering-herd effect.

## 12. Retry Storm Prevention

Retry mechanisms must account for aggregate load rather than evaluating every operation independently.

## 13. Admission Control

Retries may be denied when system capacity is insufficient even if the original operation remains logically permitted.

```text
Authorized
    ≠
AdmissibleForRetry
```

## 14. Retry Classification

Retryability should depend on failure class.

Typical categories:

```text
retryable
non-retryable
conditionally-retryable
unknown
```

## 15. Permanent Failure

Permanent failures should not be retried merely because an attempt budget remains.

## 16. Unknown Outcome

An unknown outcome must not automatically trigger a duplicate execution.

```text
Timeout
 ↓
OutcomeUnknown
 ↓
Reconcile / Query / Verify
```

## 17. Idempotency

An operation is idempotent when repeating the operation does not produce an unintended additional effect under its declared semantics.

## 18. Idempotency Is Not Automatic

```text
Retryable
    ≠
Idempotent
```

A retryable operation may still require an idempotency mechanism.

## 19. Idempotency Key

Where supported, a logical operation should carry a stable idempotency key across attempts.

```text
operation_id → idempotency_key
```

## 20. Deduplication

A receiver may use the idempotency key to recognize previously accepted operations.

## 21. Deduplication State

Deduplication state requires lifecycle and persistence semantics consistent with Part LXVIII.

## 22. Deduplication Window

A deduplication key cannot be assumed valid forever unless explicitly specified.

```text
valid_from
valid_until
```

## 23. Replay

A stale request that is technically valid syntactically may still be invalid semantically.

Freshness, epoch, nonce, or sequence requirements should be used where replay is dangerous.

## 24. At-Most-Once

At-most-once execution reduces duplicate effects but may lose work when delivery fails.

## 25. At-Least-Once

At-least-once delivery may produce duplicate attempts and therefore requires idempotency or deduplication where effects are not naturally repeat-safe.

## 26. Exactly-Once Claims

NROS must not claim exactly-once execution merely because a protocol uses identifiers.

Exactly-once semantics require an explicit end-to-end proof of the relevant effect boundary.

## 27. Compensation

Compensation is an explicit action intended to counteract an already-applied effect.

```text
ForwardEffect
 ↓
Failure
 ↓
Compensation
```

## 28. Compensation Is Not Rollback

```text
Rollback
    ≠
Compensation
```

Rollback restores state within a transactional boundary; compensation performs a new operation intended to offset an earlier effect.

## 29. Compensation Safety

Compensation itself may fail, be delayed, or produce new effects.

Therefore compensation requires its own policy and verification.

## 30. Compensation Chain

Complex workflows may require:

```text
A
 ↓
B
 ↓
C
 ↓
Failure
 ↓
Compensate C
 ↓
Compensate B
 ↓
Compensate A
```

The order must be explicitly defined.

## 31. Non-Compensable Effects

Some external effects cannot be safely compensated.

Such operations require stronger confirmation, fencing, or human/system approval before execution where appropriate.

## 32. Side Effects

Retry policy must classify side effects independently from computational success.

## 33. External Effects

Network calls, physical actuators, payments, resource allocation, and administrative actions may have irreversible external effects.

## 34. Confirmation Before Retry

If an operation may have succeeded but the result is unknown, NROS should query or reconcile before repeating it where duplicate effects are dangerous.

## 35. Recovery Budget

Recovery itself requires bounded resources:

```text
recovery_attempts
recovery_time
recovery_memory
recovery_io
recovery_network
```

## 36. Recovery Recursion

Recovery must not recursively consume unbounded recovery resources while attempting to recover its own recovery machinery.

## 37. Circuit Breaking

Repeated failures against a dependency may trigger a circuit state:

```text
Closed
 ↓ failure threshold
Open
 ↓ cooldown
HalfOpen
 ↓ probe
Closed / Open
```

## 38. Circuit State

Circuit state must be explicit, observable, and concurrency-safe.

## 39. Probe Operations

Half-open probes must be bounded and must not allow uncontrolled concurrent recovery attempts.

## 40. Dependency Failure

A dependency failure should not necessarily terminate unrelated work if isolation and degraded behavior permit continued operation.

## 41. Bulkhead Isolation

Independent workloads should be protected from shared failure amplification through resource and concurrency isolation.

## 42. Failure Domains

Retry and recovery policy should respect failure domains:

```text
operation
component
process
node
network
region/domain
external system
```

## 43. Escalation

Repeated or severe failures may escalate through policy-defined levels.

```text
retry
 ↓
backoff
 ↓
reconcile
 ↓
degrade
 ↓
quarantine
 ↓
abort
```

## 44. Quarantine

A component, resource, or workload may be quarantined when continued operation could propagate unsafe behavior.

## 45. Quarantine Exit

Quarantine release requires explicit prerequisites and verification.

## 46. Degraded Mode

Degraded execution should be explicitly represented rather than silently changing system guarantees.

## 47. Guarantee Reduction

A degraded state must identify which guarantees remain valid and which are suspended.

## 48. Safety Priority

When availability and safety conflict, the declared safety policy must determine the response.

## 49. Security Boundary

Failure recovery must not bypass authorization merely because the system is degraded.

## 50. Recovery Authority

Recovery actions require authority just like ordinary actions.

```text
Failure
    ≠
Authority Bypass
```

## 51. Recovery Idempotency

Recovery operations themselves should be idempotent where repeated recovery attempts are possible.

## 52. Recovery Ordering

Recovery actions should preserve declared dependencies and avoid restoring a downstream component before its prerequisites.

## 53. Dependency Recovery

A workload depending on an unavailable service should not repeatedly retry while the dependency is known to be outside its recovery window.

## 54. Temporal Bounds

Retry and recovery must use the temporal semantics established by Part LXIV.

## 55. Deadline Propagation

A retry must not silently reset the original operation deadline unless policy explicitly permits a new logical operation.

## 56. Deadline vs Attempt Timeout

```text
Logical Deadline
    ≠
Per-Attempt Timeout
```

## 57. Cancellation

Cancellation must stop future attempts and define what happens to an attempt already in progress.

## 58. Cancellation Race

Cancellation and completion may race.

The resulting state must remain explicitly represented rather than inferred from message arrival order alone.

## 59. Retry Race

A scheduler must prevent multiple independent retry controllers from unintentionally executing the same logical retry budget concurrently.

## 60. Single Retry Authority

Each logical operation should have a clearly defined retry authority.

## 61. Retry Coordination

Distributed retry coordination should use the state and lease semantics established by earlier parts where multiple workers can act on the same operation.

## 62. Lease Expiration

A worker must not continue recovery authority after its lease or fencing epoch expires.

## 63. Stale Recovery

A stale recovery attempt must not overwrite newer state merely because it started earlier.

## 64. Monotonic Recovery State

Recovery state should progress through an explicit lifecycle rather than oscillating invisibly.

Possible states:

```text
Pending
Attempting
Waiting
Reconciling
Compensating
Degraded
Quarantined
Recovered
Aborted
```

## 65. Recovery State Persistence

Recovery state that must survive restart requires durable persistence.

## 66. Recovery Evidence

Important recovery transitions should emit evidence through the observability mechanisms of Part LXIX.

## 67. Recovery Audit

Security-sensitive recovery actions should be auditable under Part LXX.

## 68. Recovery Verification

Recovery is not complete merely because a process restarted.

```text
Restarted
    ≠
Recovered
```

## 69. Post-Recovery Verification

Verification should establish the required invariants before returning to normal operation.

## 70. Partial Recovery

NROS should support explicit partial recovery when only a subset of components can safely return to service.

## 71. Recovery Isolation

A failed workload should not automatically force unrelated workloads through the same recovery path.

## 72. Failure Aggregation

Multiple correlated failures should be distinguishable from multiple independent failures to prevent inappropriate retry amplification.

## 73. Root-Cause Correlation

Failure correlation may use:

```text
causation_id
trace_id
component
failure_domain
policy_version
```

## 74. Retry Observability

Telemetry should expose enough information to distinguish:

```text
initial attempt
retry attempt
reconciliation
compensation
recovery probe
```

## 75. Retry Metrics

Useful metrics include:

```text
attempt_count
retry_count
retry_success_rate
retry_exhaustion_rate
backoff_duration
recovery_duration
compensation_rate
circuit_open_count
```

## 76. Cardinality Control

Operation-specific identifiers should not automatically become unbounded metric labels.

## 77. Failure Policy Version

Recovery decisions should identify the policy version when policy evolution affects reproducibility.

## 78. Policy Change During Recovery

A policy update during an active recovery sequence must define whether the sequence continues under the old policy, migrates, or is restarted.

## 79. Recovery Migration

Changing recovery policy mid-flight must not accidentally expand authority or reset safety budgets.

## 80. Recovery Budget Monotonicity

Used recovery budget must not be restored by process restart unless explicitly defined as a new logical recovery operation.

## 81. Restart Semantics

A restart must not implicitly reset attempt counts, idempotency state, circuit state, or safety restrictions when those states belong to the logical operation rather than the process.

## 82. Durable Retry State

Retry state requiring cross-process continuity must be persisted in an appropriate durable state store.

## 83. Crash During Retry

If a worker crashes after issuing an external effect but before recording the result, the resulting uncertainty must be handled as an unknown outcome.

## 84. Crash During Compensation

A crash during compensation must not cause an automatic assumption that compensation succeeded.

## 85. Recovery After Unknown Outcome

Unknown outcomes require reconciliation before unsafe repetition whenever possible.

## 86. Human Escalation

Some failures require human or higher-level supervisory intervention rather than further automatic retries.

## 87. Automatic vs Manual Recovery

The policy must distinguish:

```text
automatic
supervised
manual
forbidden
```

## 88. Recovery Safety Gate

A recovery action may proceed only when its prerequisites are satisfied.

```text
RecoveryAllowed
    ⇒
Authority
 ∧
Budget
 ∧
TemporalValidity
 ∧
SafetyPrerequisites
```

## 89. Formal Retry Invariant

```text
Retry(A)
    ⇒
Retryable(A)
 ∧
BudgetAvailable(A)
 ∧
AuthorityValid(A)
```

## 90. Formal Idempotency Invariant

```text
DuplicateAttempt(A)
    ⇒
NoUnintendedAdditionalEffect(A)
```

where the operation's declared semantics require retry safety.

## 91. Formal Compensation Invariant

```text
Compensate(E)
    ⇒
CompensationPolicy(E)
 ∧
CompensationAuthority(E)
 ∧
CompensationOutcomeRecorded(E)
```

## 92. Formal Recovery Invariant

```text
Recovered(W)
    ⇒
RequiredRecoveryChecksPassed(W)
```

## 93. Formal Budget Invariant

```text
UsedBudget + RemainingBudget = DeclaredBudget
```

subject to explicitly defined policy transitions.

## 94. Verification Matrix

| Property | Verification question |
|---|---|
| Retry policy | Is every retry governed by an explicit policy? |
| Bounds | Are attempts, time, and resources bounded? |
| Backoff | Can retry storms be prevented? |
| Idempotency | Can duplicate attempts create unintended effects? |
| Unknown outcome | Is reconciliation attempted before unsafe repetition? |
| Deduplication | Is deduplication state durable when required? |
| Compensation | Are compensating effects explicit and verifiable? |
| Circuit breaking | Are dependency failures isolated? |
| Bulkheads | Can one failure domain exhaust unrelated resources? |
| Authority | Can recovery bypass security policy? |
| Leases | Can stale workers continue recovery? |
| Deadlines | Can retries silently extend logical deadlines? |
| Cancellation | Are cancellation races explicitly handled? |
| Persistence | Does restart preserve logical recovery state? |
| Verification | Is restart distinguished from successful recovery? |
| Escalation | Are repeated failures escalated appropriately? |
| Degradation | Are reduced guarantees explicit? |
| Audit | Are security-sensitive recovery actions auditable? |
| Observability | Can retry/recovery behavior be reconstructed? |
| Policy changes | Is active recovery behavior deterministic under policy updates? |

## 95. What Part LXXII Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- universal idempotency;
- exactly-once effects;
- distributed circuit breakers;
- automatic compensation for arbitrary external effects;
- globally coordinated retry budgets;
- immediate recovery;
- complete crash-safe retry persistence;
- automatic root-cause determination;
- human escalation infrastructure.

Those require implementation-specific evidence.

## 96. Transition to Part LXXIII

Part LXXII defines the policy and control mechanics for responding to failures.

Part LXXIII should focus on **state/resource reconciliation after failure**, including snapshots, epochs, leases, conflict detection, stale state, resource ownership, convergence, and verification of recovered state.

```text
LXXI
Failure semantics + mismatch + containment + recovery states
        ↓
LXXII
Retry + idempotency + compensation + budgets + recovery control
        ↓
LXXIII
State + resource reconciliation + convergence
```

## Canonical rule

> **NROS must never treat repetition as recovery by default: every retry is policy-governed, every potentially duplicated effect requires explicit idempotency or reconciliation semantics, every compensation is itself an operation, and recovery remains bounded, authorized, observable, persistent where required, and verified before normal execution resumes.**
