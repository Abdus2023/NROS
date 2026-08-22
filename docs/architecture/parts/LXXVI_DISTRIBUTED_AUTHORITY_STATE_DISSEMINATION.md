# Part LXXVI — Distributed Authority-State Dissemination, Freshness & Provenance

> **Series:** NROS Architecture Series  
> **Part:** LXXVI  
> **Role:** Dissemination and observation of durable authority state across runtime participants  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LXXV defines how authority transitions become durable system facts. Part LXXVI defines how those facts become **distributedly observable** without allowing stale, forged, reordered, or ambiguous observations to regain authority.

```text
LXXIV
Authority / fencing
   ↓
LXXV
Durable transition state
   ↓
LXXVI
Dissemination / observation / freshness / provenance
```

The central question is:

> **How does a participant know which authority state it may safely trust right now?**

## 2. Scope

This Part covers:

- authority-state announcements;
- observation records;
- dissemination channels;
- freshness;
- provenance;
- sequence and epoch validation;
- acknowledgement;
- replay and duplicate suppression;
- stale-state rejection;
- partition and rejoin observation;
- convergence of authority views;
- verification boundaries.

It does not redefine election, quorum, fencing, durable transition commitment, or reconciliation policy.

## 3. Authority Observation

A participant may observe authority state without becoming authoritative.

```text
ObservedAuthority
    ≠
LocalAuthority
```

An observation is evidence about authority state; it is not itself an authorization grant.

## 4. Dissemination Record

A disseminated authority-state record should carry sufficient metadata to evaluate trust and freshness:

```text
DomainId
Epoch
TransitionSequence
AuthorityId
FenceToken / generation
SourceId
SourceIdentity
ProtocolVersion
ObservedAt / logical time
Integrity / authenticity evidence
```

The concrete wire representation remains implementation-specific.

## 5. Freshness

Freshness must be evaluated using explicit ordering semantics rather than arrival time alone.

```text
IncomingSequence < CurrentSequence
    ⇒ stale
```

Likewise:

```text
IncomingEpoch < CurrentEpoch
    ⇒ obsolete authority state
```

A late packet must not become authoritative merely because it arrived last.

## 6. Provenance

Participants must be able to distinguish:

```text
Who produced this observation?
Which authority generation does it describe?
What transition does it refer to?
Under which protocol version?
```

Provenance is therefore part of the authority-state observation contract.

## 7. Authenticity

Authority-state dissemination must not be treated as trusted solely because it arrived through a known transport.

Where required, observations must be authenticated and integrity-protected according to the applicable security/policy architecture.

## 8. Sequence Validation

A participant maintains its highest accepted authority sequence:

```text
CurrentSequence = N
```

An incoming record is classified as:

```text
N+1 → candidate next state
N   → duplicate / consistency check
<N  → stale
>N+1 → gap / incomplete knowledge
```

A gap must not automatically be interpreted as a valid transition.

## 9. Epoch Validation

Epoch and sequence provide different information.

```text
Epoch   = authority generation
Sequence = ordered transition position
```

Both may be required to establish whether an observation is applicable.

## 10. Acknowledgement

Dissemination may require acknowledgement that a participant has received and validated an authority-state record.

```text
Publish
  ↓
Receive
  ↓
Authenticate
  ↓
Validate epoch/sequence
  ↓
Acknowledge
```

Acknowledgement means receipt/validation, not authority transfer.

## 11. Duplicate Suppression

Repeated dissemination of the same transition must be idempotent.

```text
TransitionId already observed
    ↓
no duplicate state transition
```

This permits retransmission without changing logical state.

## 12. Reordering

Messages may arrive out of order.

```text
T5 arrives
T3 arrives
T4 arrives
T6 arrives
```

The participant must retain enough ordering information to avoid applying T3/T4 as newer authority state after T5 has already been accepted.

## 13. Gaps

A missing transition creates incomplete knowledge.

```text
Current = T5
Incoming = T7
```

The safe state is not automatically `T7 accepted`.

The participant may need to:

- request missing transitions;
- obtain a trusted snapshot;
- enter a bounded stale/unknown state;
- defer authority-sensitive decisions.

## 14. Stale Observation Rule

> **No stale authority observation may increase local authority.**

An old observation can be useful as diagnostic evidence, but it cannot restore obsolete authority.

## 15. Partition

During a partition, authority views may diverge:

```text
A → Epoch 7
B → Epoch 6
```

The participant with Epoch 6 must not infer that its local view remains authoritative merely because it has not received Epoch 7.

Authority validity remains governed by LXXIV/LXXV semantics.

## 16. Rejoin Dissemination

A returning participant should synchronize authority state before resuming authority-sensitive operations.

```text
Reconnect
   ↓
Authenticate
   ↓
Exchange authority summary
   ↓
Compare epoch / sequence
   ↓
Fetch missing state
   ↓
Validate provenance
   ↓
Reconcile
   ↓
Resume permitted activity
```

## 17. Snapshot Dissemination

When transition history is too large or gaps exist, a trusted authority snapshot may be disseminated.

A snapshot should identify:

```text
epoch
sequence
authority
fence state
snapshot version
source
integrity evidence
```

## 18. Snapshot vs Transition Stream

```text
Snapshot
  ↓
bounded baseline

Transition stream
  ↓
incremental evolution
```

The two mechanisms complement each other.

## 19. Freshness Windows

Some authority observations may be valid only within an explicit freshness bound.

```text
now - observed_at ≤ freshness_window
```

However, wall-clock freshness must not replace epoch/sequence correctness where ordering is safety-critical.

## 20. Unknown Authority State

If the participant cannot establish sufficiently current authority information:

```text
Authority = UNKNOWN
```

must remain distinct from:

```text
Authority = VALID
```

Unknown authority should cause bounded restriction of authority-sensitive operations according to policy.

## 21. Local Cache

Participants may cache authority state for performance, provided the cache has explicit validity semantics.

A cache must not silently outlive the epoch/freshness conditions under which it was trusted.

## 22. Dissemination Failure

Failure to disseminate an update does not necessarily invalidate the authority itself.

It creates an **observation gap** whose consequences depend on the participant's policy and authority guarantees.

## 23. Authority vs Knowledge

A participant may have:

```text
valid authority + incomplete knowledge
```

or:

```text
complete knowledge + no authority
```

These states must not be conflated.

## 24. Safety Invariants

### D1 — No authority from observation alone

An observation cannot grant authority.

### D2 — Monotonic accepted state

Accepted authority state must not regress to an older generation.

### D3 — Provenance preservation

Authority observations retain enough origin information for trust and audit decisions.

### D4 — Gap awareness

Missing transition information is represented as incomplete knowledge, not silently fabricated state.

### D5 — Duplicate idempotency

Replaying an already accepted dissemination record does not alter logical state.

### D6 — Rejoin safety

A rejoining participant cannot resume stale authority without current validation.

### D7 — Freshness is bounded

Cached authority information has explicit validity conditions.

## 25. Verification Boundary

Implementation evidence should eventually demonstrate:

- stale observation rejection;
- out-of-order delivery handling;
- duplicate suppression;
- transition-gap detection;
- snapshot recovery;
- provenance validation;
- authentication/integrity validation;
- partition/rejoin behavior;
- freshness expiry;
- safe handling of unknown authority state;
- convergence after dissemination resumes.

Until such tests or evidence exist, these remain architectural requirements.

## 26. Relationship to Earlier Parts

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
Authority / quorum / fencing
        ↓
LXXV
Durable authority transition state
        ↓
LXXVI
Distributed observation and dissemination
```

The novel boundary is **knowledge of authority**, not authority itself.

## 27. Architectural Rule

> **Authority state must be disseminated as ordered, provenance-bearing evidence; observation can inform authority decisions but must never manufacture authority.**

## 28. Transition to Part LXXVII

The next Part should address **authority-state convergence and conflict arbitration across heterogeneous participants**, building on dissemination without redefining the dissemination protocol itself.
