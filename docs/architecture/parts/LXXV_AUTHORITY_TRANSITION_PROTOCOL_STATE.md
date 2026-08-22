# Part LXXV — Authority Transition Protocol & Durable Authority State

> **Series:** NROS Architecture Series  
> **Part:** LXXV  
> **Role:** Durable recording, propagation, ordering, and verification of authority transitions  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LXXIV established **who may be authoritative**. Part LXXV establishes the protocol/state machinery by which an authority transition becomes a durable, ordered, observable, and independently checkable system fact.

```text
LXXIII  Reconciliation
   ↓
LXXIV   Authority / quorum / epoch / fencing
   ↓
LXXV    Transition protocol + durable authority state
```

The central question is:

> **How does NROS make an authority transition persistently knowable and reject stale or conflicting transitions?**

## 2. Scope

This Part covers:

- authority-transition records;
- monotonic transition sequence;
- epoch/generation persistence;
- durable fencing state;
- transition intent and commitment;
- acknowledgement and observation;
- replay/recovery;
- idempotent transition processing;
- conflicting transition detection;
- durable authority snapshots;
- propagation and convergence;
- verification boundaries.

It does not redefine quorum election, lease policy, failure semantics, or reconciliation policy.

## 3. Transition Record

An authority transition should be representable as a durable record containing at least:

```text
TransitionId
DomainId
PreviousEpoch
NewEpoch
PreviousAuthority
NewAuthority
TransitionReason
QuorumEvidence
FenceToken
Sequence
Timestamp / logical time
Policy / protocol version
```

The exact serialization remains implementation-specific.

## 4. Transition Lifecycle

```text
Proposed
   ↓
Quorum-checked
   ↓
Epoch-assigned
   ↓
Fence-established
   ↓
Committed
   ↓
Propagated
   ↓
Observed
   ↓
Reconciled
```

A proposal is not equivalent to a committed authority transition.

## 5. Ordering

For a protected coordination domain, transition ordering must be monotonic.

```text
T1(epoch 4)
   ↓
T2(epoch 5)
   ↓
T3(epoch 6)
```

An observed transition with an older generation must not replace a newer durable generation.

```text
IncomingEpoch < DurableEpoch
    ⇒ reject / ignore as stale
```

Equal epochs require identity/evidence consistency checks.

## 6. Durable Authority State

A restart must not silently erase authority history.

```text
Runtime
  ↓
Durable authority state
  ├── current epoch
  ├── current authority
  ├── fence state
  ├── transition sequence
  └── committed transition evidence
```

After restart, the runtime must recover this state before accepting authority-sensitive mutations.

## 7. Idempotency

Processing the same transition more than once must not create a second logical transition.

```text
TransitionId already committed
        ↓
return existing result
        ↓
no duplicate authority change
```

This is especially important during retries, replay, reconnect, and crash recovery.

## 8. Conflict Detection

Conflicting transitions must be detected rather than silently merged.

Examples include:

```text
same domain + same epoch + different authority
same sequence + different transition
new epoch without valid predecessor
fence token inconsistent with committed epoch
```

The safe response is rejection and escalation to reconciliation/coordination policy.

## 9. Propagation

A committed transition may need to propagate across runtime participants.

```text
Committed authority transition
          ↓
      propagation
      /    |    \
     A     B     C
      \    |    /
       observed state
```

Propagation does not mean every participant immediately becomes authoritative. It means participants obtain enough durable information to reject stale state and interpret the current authority generation correctly.

## 10. Snapshot + Log Model

NROS may represent authority state using a snapshot plus ordered transition history:

```text
AuthoritySnapshot(E)
       +
TransitionLog(E+1 ... N)
```

Recovery can reconstruct the current authority state by loading a trusted snapshot and replaying valid subsequent transitions.

The architecture does not mandate a particular storage technology.

## 11. Recovery

On restart or crash:

```text
Load durable authority state
        ↓
Validate integrity / version
        ↓
Recover latest committed transition
        ↓
Recover current epoch/fence
        ↓
Reject stale local authority
        ↓
Reconcile external state if necessary
        ↓
Resume only after authority validation
```

A process restart must not implicitly restore pre-crash authority.

## 12. Rejoin

A participant returning after partition must first synchronize authority state.

```text
Disconnected participant
        ↓
Read current durable authority
        ↓
Compare epoch
        ↓
Discard stale local state
        ↓
Reconcile
        ↓
Re-authorize if policy permits
```

Reconnection is therefore a state synchronization event, not automatic authority restoration.

## 13. Verification Invariants

### P1 — Monotonic epoch

```text
CommittedEpoch(next) > CommittedEpoch(current)
```

for every new authority generation.

### P2 — Durable ordering

Committed transitions have a deterministic order within their domain.

### P3 — Stale rejection

```text
IncomingEpoch < DurableEpoch
    ⇒ no authority mutation
```

### P4 — Idempotent replay

Replaying an already committed transition does not alter logical authority state.

### P5 — Conflict rejection

Conflicting transition evidence is never silently accepted.

### P6 — Restart safety

Restart does not restore authority without validated durable state.

### P7 — Fence continuity

The effective fence state corresponds to the committed authority generation.

## 14. Relationship to Part LXXIV

```text
LXXIV
Can this actor be authoritative?
        ↓
LXXV
How is that authority transition recorded,
ordered, propagated, recovered, and verified?
```

LXXIV defines the authority mechanism.

LXXV defines the **authority-state protocol boundary**.

## 15. Relationship to Earlier Parts

```text
XXV
Distributed coordination primitives
        ↓
XLVII
Authoritative decision model
        ↓
LXVII
Distributed state / consistency
        ↓
LXXI–LXXIII
Failure → policy → reconciliation
        ↓
LXXIV
Authority / fencing
        ↓
LXXV
Durable authority transition protocol
```

LXXV therefore does not replace earlier coordination or state Parts; it connects their concepts through an explicit durable transition model.

## 16. Verification Boundary

The following must eventually be demonstrated by implementation-level evidence:

- transition durability across restart;
- monotonic epoch enforcement;
- stale-transition rejection;
- duplicate-transition idempotency;
- conflicting-transition detection;
- fence-state recovery;
- partition/rejoin behavior;
- transition replay correctness;
- snapshot/log reconstruction;
- protocol-version compatibility.

Until executed and recorded, these remain architectural requirements.

## 17. Architectural Rule

> **An authority transition becomes system truth only when its generation, ordering, fencing evidence, and commitment state are durably recorded and recoverable.**

## 18. Transition to Part LXXVI

The next Part should address the **distributed observation and dissemination plane** required to make authority-state transitions visible across participants, while preserving ordering, freshness, provenance, and stale-data rejection.
