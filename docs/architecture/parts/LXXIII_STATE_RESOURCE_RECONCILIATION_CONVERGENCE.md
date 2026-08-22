# Part LXXIII — State & Resource Reconciliation, Conflict Resolution & Convergence

> **Series:** NROS Architecture Series  
> **Part:** LXXIII  
> **Role:** Re-establishing authoritative state after failure, partition, restart, timeout, retry, stale observation, or partial execution  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LXXI defines failure semantics. Part LXXII defines the policies governing retry, idempotency, compensation, and recovery attempts. Part LXXIII defines how NROS determines the state that actually exists after uncertainty and moves distributed state and resources toward a safe, authoritative, convergent condition.

The central rule is:

> **Recovery is incomplete until NROS has reconciled uncertain state and resources against an authoritative source or an explicitly bounded consistency model.**

## 2. Reconciliation Model

```text
Uncertainty
 ↓
Observe
 ↓
Collect State
 ↓
Compare
 ↓
Classify Conflict
 ↓
Select Authority
 ↓
Resolve
 ↓
Persist Result
 ↓
Verify
 ↓
Converge
```

## 3. State

State is the information required to determine the current condition of a runtime, resource, workload, operation, or coordination domain.

## 4. Desired State

Desired state expresses what NROS intends to hold true.

```text
Desired
    ≠
Observed
```

until reconciliation establishes their relationship.

## 5. Observed State

Observed state is information obtained from an actual runtime, node, resource, peer, or durable record.

Observation may be stale, incomplete, contradictory, or unavailable.

## 6. Authoritative State

An authoritative state source is defined by explicit architecture and policy rather than assumed from whichever observation arrived last.

## 7. Authority Selection

When multiple state sources disagree, NROS must apply a deterministic authority rule.

Possible authority dimensions include:

```text
epoch
version
lease
quorum
trusted source
transaction outcome
resource owner
policy
```

## 8. State Version

Mutable state should carry a version, generation, revision, epoch, or equivalent ordering mechanism when stale updates are possible.

## 9. Epoch

An epoch identifies an authority or generation boundary.

```text
OldEpoch
    ≠
CurrentEpoch
```

## 10. Stale State

State from an older epoch or version must not silently overwrite newer authoritative state.

## 11. Fencing

Where stale actors can mutate shared resources, fencing must prevent operations from obsolete authority generations.

## 12. Lease

A lease grants temporary authority subject to expiry and renewal semantics.

Expired leases cannot authorize new mutations.

## 13. Lease + Epoch

Combining lease and epoch information can prevent a renewed actor from being confused with an obsolete actor holding an older lease.

## 14. Resource State

Resource reconciliation should distinguish:

```text
free
reserved
allocated
active
released
lost
unknown
quarantined
```

## 15. Resource Ownership

Resource ownership must be reconciled independently from mere observation of resource activity.

## 16. Resource Reservation

Reservations should identify:

```text
resource_id
owner
reservation_id
epoch
expiry
purpose
```

where required.

## 17. Orphaned Resources

A resource may remain externally active after the controlling runtime disappears.

Therefore:

```text
ControllerAbsent
    ≠
ResourceFree
```

## 18. Orphan Recovery

Orphaned resources require an explicit recovery policy before reuse.

Possible outcomes:

```text
reclaim
quarantine
adopt
release
manual intervention
unknown
```

## 19. Unknown Resource State

Unknown state must not be treated as safe-to-use state for operations whose safety depends on exclusivity or ownership.

## 20. Conflict

A conflict exists when credible state observations cannot simultaneously satisfy the declared consistency model.

## 21. Conflict Classes

Conflicts may include:

```text
version conflict
ownership conflict
lease conflict
epoch conflict
resource allocation conflict
configuration conflict
operation outcome conflict
replica divergence
```

## 22. Conflict Resolution

Conflict resolution must identify:

```text
conflicting observations
authority rule
selected state
rejected state
reason
verification
```

## 23. Last-Writer-Wins

Last-writer-wins must not be used as a universal conflict-resolution mechanism where causal order, ownership, or safety semantics are stronger requirements.

## 24. Causal Ordering

Causal relationships should be preferred over wall-clock ordering where they determine correctness.

## 25. Quorum

A quorum may establish authority where the architecture requires agreement among replicas.

The quorum definition must be explicit.

## 26. Partition

During a network partition, replicas may observe different states.

```text
Replica A → S1
Replica B → S2
```

The system must define whether both may continue operating and under what authority constraints.

## 27. Partition Safety

When simultaneous authority would create unsafe effects, NROS must preserve a single valid authority or fence conflicting actors.

## 28. Local-First Recovery

A disconnected node may continue under explicitly bounded local authority if the security, resource, and consistency policies permit it.

## 29. Rejoin

A node rejoining a coordination domain must reconcile its state before resuming authority that could conflict with current state.

## 30. Rejoin Protocol

```text
Disconnected
 ↓
Reconnect
 ↓
Authenticate
 ↓
Establish Current Epoch
 ↓
Exchange State Summary
 ↓
Detect Divergence
 ↓
Reconcile
 ↓
Persist
 ↓
Verify
 ↓
Resume Authority
```

## 31. State Summary

A compact state summary may contain:

```text
epoch
revision
hash
lease state
resource ownership
active operations
configuration version
```

## 32. State Hash

Hashes can detect divergence but do not themselves determine which state is authoritative.

```text
HashEqual
    ⇒
Potentially Same State
```

but:

```text
HashDifferent
    ⇒
Divergence Detected
```

not necessarily a resolution.

## 33. Snapshot

A snapshot provides a bounded representation of state at a declared point or consistency boundary.

## 34. Snapshot Validity

Snapshots must identify their version, epoch, source, and consistency guarantees where those properties matter.

## 35. Journal + Snapshot

Recovery may combine snapshots with subsequent event/journal records.

```text
Snapshot(S)
 ↓
Events(E₁...Eₙ)
 ↓
Reconstructed State
```

## 36. Durable Reconciliation Record

Important reconciliation decisions should be persisted so a crash does not erase the resolution history.

## 37. Reconciliation Idempotency

Repeating reconciliation should converge to the same result when the underlying authoritative state has not changed.

```text
Reconcile(Reconcile(S)) = Reconcile(S)
```

for stable conditions.

## 38. Monotonic Reconciliation

A reconciliation process should not reintroduce authority or state that was explicitly invalidated by a newer authoritative decision.

## 39. Recovery After Unknown Outcome

For an operation whose external outcome is unknown:

```text
Query External State
 ↓
Match Operation Identity
 ↓
Determine Outcome
 ↓
Record Outcome
```

Only if the external state cannot determine the outcome should a policy-defined compensation or retry path be selected.

## 40. Operation Identity

Operations requiring reconciliation should carry stable identity sufficient to distinguish repeated attempts.

## 41. Attempt vs Operation

```text
operation_id
    ≠
attempt_id
```

A retry may create a new attempt while referring to the same logical operation.

## 42. Duplicate Detection

Duplicate operation detection should use explicit identity rather than heuristic similarity whenever correctness depends on it.

## 43. Resource Reconciliation Ordering

Where resource state affects safety, reconciliation should establish:

```text
authority
 ↓
ownership
 ↓
reservation
 ↓
allocation
 ↓
activity
```

rather than inferring ownership solely from activity.

## 44. Configuration Reconciliation

Configuration should be reconciled against its declared source of authority and version.

## 45. Policy Reconciliation

Security policy must be reconciled before authority-dependent operations resume.

## 46. Time Reconciliation

Time-dependent leases, deadlines, and expiry conditions must be evaluated using the temporal semantics established by Part LXIV.

## 47. Messaging Reconciliation

Undelivered, duplicated, or uncertain messages must be reconciled using message identity, delivery state, and the messaging contract established by Part LXVI.

## 48. Persistence Reconciliation

Durable state must be compared against recovered runtime state according to the persistence guarantees established by Part LXVIII.

## 49. Observability Integration

Reconciliation actions should emit structured evidence identifying:

```text
observed state
selected authority
resolution
reason
result
verification status
```

## 50. Security Integration

Reconciliation must not grant authority merely because an actor claims a previous role.

Current identity, policy, capability, lease, and epoch must be evaluated.

## 51. Authority Resurrection

Recovery must prevent revoked or expired authority from being resurrected from stale state.

## 52. Resource Resurrection

A resource marked released or quarantined by newer authority must not be reintroduced as active merely because an old snapshot still contains it.

## 53. Tombstones

Tombstones may preserve knowledge that an entity, resource, lease, or authority was removed so stale replicas cannot recreate it accidentally.

## 54. Tombstone Retention

Tombstones must remain available long enough to cover the maximum stale-replica or message-replay window defined by the system.

## 55. Garbage Collection

State garbage collection must not remove information still required to prevent stale resurrection or replay.

## 56. Convergence

Convergence means that, under stable conditions and a valid consistency model, replicas eventually reach compatible authoritative state.

## 57. Convergence Preconditions

Convergence requires defined assumptions about:

```text
connectivity
message delivery
authority
conflict resolution
state persistence
retry
clock/epoch semantics
```

## 58. Eventual Consistency

Eventual consistency does not mean temporary disagreement is harmless for every operation.

Safety-sensitive operations may require stronger consistency.

## 59. Strong Consistency

Strong consistency should be used only where the architecture can actually establish its required guarantees.

## 60. Convergence Failure

If convergence cannot be established within declared bounds, NROS must expose degraded or blocked state rather than claiming normal operation.

## 61. Manual Resolution

Some conflicts cannot be safely resolved automatically.

Manual resolution must be explicit and auditable.

## 62. Quarantine

Unresolvable or unsafe state may be quarantined until authority and correctness are established.

## 63. Safe Resume

A recovered workload may resume only after all resume prerequisites are satisfied.

```text
Resume(W)
    ⇒
AuthorityValid
∧
ResourceStateValid
∧
PolicyCurrent
∧
TemporalConstraintsValid
∧
UnknownOutcomesResolved
∧
RequiredVerificationPassed
```

## 64. Safe Stop

If required reconciliation cannot establish safe state, the system should enter the declared safe-stop or blocked state.

## 65. Reconciliation Budget

Reconciliation itself requires bounded resources and time.

Possible limits:

```text
max duration
max retries
max state size
max conflict set
max external queries
```

## 66. Reconciliation Storms

Large-scale recovery can create synchronized retries and state exchanges.

NROS should use backoff, batching, prioritization, and admission control to avoid recovery storms.

## 67. Priority

Safety-critical reconciliation should take precedence over optional diagnostic or background convergence work where resources are constrained.

## 68. Dependency Ordering

If state A determines the validity of state B, reconciliation must establish A before accepting B.

```text
Authority
   ↓
Resource Ownership
   ↓
Work Assignment
   ↓
Execution State
```

## 69. Circular Dependencies

Reconciliation graphs containing cycles require explicit fixed-point or staged resolution semantics.

## 70. Fixed Point

A reconciliation cycle has converged when another reconciliation pass produces no material state change under stable inputs.

## 71. Verification of Convergence

Convergence should be tested through explicit state comparison rather than inferred from absence of errors.

## 72. Reconciliation Evidence

A reconciliation record should support reconstruction of:

```text
before
observations
authority decision
resolution
after
verification
```

## 73. Conflict Provenance

Conflicts should preserve the origin of competing observations where that information is required for diagnosis or audit.

## 74. Stale Message Protection

A message carrying obsolete state must not override a newer state solely because it arrives later.

## 75. Epoch Validation

```text
MessageEpoch == CurrentEpoch
    ⇒
Potentially Acceptable
```

Otherwise it must be rejected, quarantined, or reconciled according to policy.

## 76. Lease Validation

```text
LeaseValid(now)
    ⇒
Authority may remain active
```

subject to all other policy constraints.

## 77. Clock Uncertainty

Expiry decisions must account for the clock assumptions and uncertainty model defined by the temporal architecture.

## 78. Recovery Race

Two recovery actors may attempt to reconcile the same state concurrently.

Coordination must prevent conflicting reconciliation decisions from both becoming authoritative.

## 79. Reconciliation Locking

Locks, epochs, compare-and-swap, transactions, or equivalent mechanisms may serialize conflicting state transitions.

## 80. Compare-and-Swap

State transitions may require:

```text
ExpectedVersion
    ==
CurrentVersion
```

before applying a mutation.

## 81. Atomic Resolution

Where partial application could produce unsafe state, reconciliation and its authoritative state update should be atomic within the declared boundary.

## 82. Recovery Transaction

A recovery transaction may encompass:

```text
state observation
conflict resolution
resource ownership
operation outcome
new state
```

## 83. External Systems

NROS cannot assume transactional control over external systems it does not own.

External effects therefore require explicit uncertainty and compensation semantics.

## 84. Compensation

Compensation is a new operation intended to restore an acceptable state after an already-executed effect.

Compensation does not necessarily restore the original state exactly.

## 85. Irreversible Effects

Irreversible effects require stronger preconditions and evidence because rollback may be impossible.

## 86. Human Intervention

When automated reconciliation cannot safely determine the outcome, escalation to an authorized human or supervisory agent may be required.

## 87. Human Resolution Authority

Manual resolution must itself be authenticated, authorized, scoped, and auditable.

## 88. Recovery Completion

Recovery is complete only when:

```text
StateReconciled
∧
ResourcesReconciled
∧
AuthorityValid
∧
UnknownOutcomesResolved
∧
RequiredEvidenceRecorded
∧
ResumeCriteriaSatisfied
```

## 89. Formal Authority Invariant

```text
AcceptedStateMutation(M)
    ⇒
Authority(M) = CurrentAuthority
```

within the declared consistency boundary.

## 90. Formal Staleness Invariant

```text
Version(M) < CurrentVersion
    ⇒
M cannot overwrite CurrentState
```

unless an explicit conflict-resolution rule permits it.

## 91. Formal Reconciliation Idempotency

```text
R(R(S)) = R(S)
```

for unchanged authoritative inputs.

## 92. Formal Safe Resume Invariant

```text
Resume(W)
    ⇒
ReconciliationComplete(W)
```

## 93. Formal Resurrection Invariant

```text
RevokedOrReleased(X)
    ⇒
StaleStateCannotReactivate(X)
```

## 94. Formal Convergence Invariant

```text
StableInputs ∧ ValidConnectivity
    ⇒
Eventually CompatibleAuthoritativeState
```

only when the declared consistency model provides such a guarantee.

## 95. Verification Matrix

| Property | Verification question |
|---|---|
| Authority | Is the authoritative state source explicit? |
| Versions | Can stale state be rejected? |
| Epochs | Can obsolete authority be fenced? |
| Leases | Can expired authority mutate state? |
| Resources | Can orphaned resources be safely identified? |
| Unknown outcomes | Can uncertain operations be reconciled? |
| Conflicts | Are conflict classes explicit? |
| Resolution | Is the authority rule deterministic? |
| Partition | Are partition safety semantics explicit? |
| Rejoin | Is state reconciled before authority resumes? |
| Tombstones | Can stale replicas resurrect removed state? |
| Recovery | Is reconciliation durable? |
| Idempotency | Does repeated reconciliation converge? |
| Concurrency | Can recovery races create conflicting authority? |
| External effects | Are non-transactional effects treated as uncertain? |
| Compensation | Are irreversible effects explicitly handled? |
| Convergence | Are convergence assumptions declared? |
| Degradation | Is failed convergence observable? |
| Evidence | Can reconciliation decisions be reconstructed? |
| Resume | Are resume prerequisites verified? |

## 96. What Part LXXIII Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- universal distributed consensus;
- automatic conflict resolution for every resource;
- exactly-once external effects;
- immediate global convergence;
- universal transactional control of external systems;
- complete orphan-resource recovery;
- production-grade reconciliation orchestration.

Those require implementation-specific evidence.

## 97. Transition to Part LXXIV

Part LXXIII establishes how NROS reconciles state and resources after uncertainty.

Part LXXIV should define the **distributed coordination and authority mechanisms that make reconciliation safe under concurrent actors, partitions, leases, epochs, quorum decisions, and competing controllers**.

```text
LXXI
Failure semantics
        ↓
LXXII
Failure-response policy
        ↓
LXXIII
State/resource reconciliation
        ↓
LXXIV
Distributed coordination + authority + quorum + fencing
```

## Canonical rule

> **NROS must never infer authoritative state merely from the latest observation: recovery establishes authority, reconciles state and resources, prevents stale resurrection, records the resolution, verifies the resulting condition, and only then permits safe continuation.**
