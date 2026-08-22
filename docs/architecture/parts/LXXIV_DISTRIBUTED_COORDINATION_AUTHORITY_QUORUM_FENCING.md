# Part LXXIV — Distributed Coordination, Authority, Quorum & Fencing

> **Series:** NROS Architecture Series  
> **Part:** LXXIV  
> **Role:** Coordination authority, quorum, epochs, leases, fencing, takeover, and split-brain prevention  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LXXIV establishes the authority boundary required when multiple actors may observe, coordinate, recover, or mutate the same distributed runtime domain.

The preceding Parts establish a deliberate chain:

```text
LXXI  Failure semantics
   ↓
LXXII Failure-response policy
   ↓
LXXIII State/resource reconciliation
   ↓
LXXIV Coordination authority and fencing
```

The question changes from **what happened**, to **what recovery is permitted**, to **what state exists**, and finally to:

> **Who is currently authorized to establish or mutate that state?**

## 2. Scope

This Part addresses:

- authority ownership;
- coordination domains;
- epochs and generations;
- leases and renewal;
- quorum;
- leader/controller selection;
- fencing;
- takeover and relinquishment;
- split-brain prevention;
- stale-authority rejection;
- coordination failure;
- safe authority re-establishment.

It does not redefine distributed state replication, failure semantics, or recovery policy already established elsewhere.

## 3. Prerequisite Parts

Part LXXIV builds on six architectural boundaries:

| Part | Contribution |
|---|---|
| XXV | Distributed coordination, membership, quorum, consensus, leases, fencing |
| XLVII | Authoritative coordination and decision authority |
| LXVII | Distributed state, replication, consistency, convergence |
| LXXI | Failure semantics and uncertainty |
| LXXII | Failure-response policy and bounded recovery |
| LXXIII | State/resource reconciliation and convergence |

These Parts are referenced rather than reproduced. LXXIV provides the missing authority-focused synthesis needed to make concurrent mutation safe.

## 4. Fundamental Distinctions

NROS must not collapse the following concepts:

```text
Observation
    ≠
Capability
    ≠
Ownership
    ≠
Coordination
    ≠
Decision
    ≠
Authority
    ≠
Fenced authority
```

A node may observe a resource without owning it. An actor may possess the capability to request an operation without having authority to perform that operation for the current epoch.

## 5. Coordination Domain

Authority is scoped to a coordination domain.

```text
CoordinationDomain
 ├── membership
 ├── authority state
 ├── epoch
 ├── quorum policy
 ├── lease state
 ├── fencing state
 └── protected resources
```

Authority in one domain must not silently imply authority in another domain.

## 6. Authority State Machine

A simplified authority lifecycle is:

```text
Unknown
  ↓
Candidate
  ↓
Quorum-Eligible
  ↓
Authorized
  ↓
Fenced
  ↓
Active Authority
  ↓
Renew / Extend
  ↓
Relinquish / Expire / Revoke
  ↓
Fenced / Invalid
```

Transitions must be explicit and observable.

## 7. Epochs and Generations

An epoch identifies the authority generation under which mutations are valid.

```text
Epoch N
   ↓
Authority A
   ↓
Lease / fencing token
   ↓
Mutations

Epoch N+1
   ↓
Authority B
   ↓
Old authority becomes stale
```

The critical invariant is:

```text
Mutation(epoch = N)
    is rejected
when
CurrentEpoch > N
```

An old controller must not regain authority merely because it reconnects or continues executing previously issued work.

## 8. Fencing

Fencing is the mechanism that prevents stale authority from producing accepted side effects.

```text
Old authority
    ↓
stale epoch/token
    ↓
mutation attempt
    ↓
FENCE CHECK
    ↓
REJECT
```

The important property is not merely that NROS elects a new authority. The system must ensure that the previous authority cannot continue mutating the protected domain successfully.

## 9. Quorum

Quorum establishes the minimum coordination evidence required for an authority decision.

Let `Q` be the required quorum and `V` the currently eligible voting membership:

```text
|votes_for_authority| ≥ Q
```

is necessary for authorization when the configured coordination model requires quorum.

Quorum does not automatically imply Byzantine fault tolerance, consensus, or safety under every failure model. Those properties require explicit assumptions.

## 10. Leases

A lease provides bounded authority over a defined interval.

```text
Acquire
  ↓
Active until expiry
  ↓
Renew
  ├── success → continue
  └── failure → authority expires
```

Lease validity must account for:

- monotonic time;
- renewal deadlines;
- clock assumptions;
- network uncertainty;
- authority epoch;
- fencing state.

A lease must never be interpreted as indefinite ownership.

## 11. Takeover

A takeover must not simply mean that another actor declares itself leader.

```text
Failure / expiry
      ↓
Observe authority state
      ↓
Establish quorum
      ↓
Advance epoch
      ↓
Fence previous authority
      ↓
Acquire new authority
      ↓
Reconcile state
      ↓
Resume permitted work
```

The ordering is essential: **fencing must precede acceptance of conflicting mutations**.

## 12. Relinquishment

An authority may relinquish voluntarily or become invalid because of lease expiry, revocation, fencing, membership change, or coordination failure.

```text
Active
  ↓
Stop accepting new mutations
  ↓
Publish relinquishment / expiry evidence
  ↓
Invalidate local authority
  ↓
Fence locally held work where required
  ↓
Become non-authoritative
```

Relinquishment is therefore a state transition, not merely a process exit.

## 13. Split-Brain Prevention

The architecture must prevent two actors from simultaneously possessing accepted unfenced authority over the same protected domain.

Central invariant:

```text
For a protected domain D and epoch E:

At most one authority may produce
accepted mutations for (D, E).
```

If two actors believe they are authoritative, the system must prefer safety over ambiguous concurrent mutation.

## 14. Coordination Failure

Loss of coordination does not automatically imply resource release.

```text
CoordinationLost
      ↓
Authority validity check
      ↓
Lease / epoch / fencing evaluation
      ↓
 ┌───────────────┐
 │ still valid   │ → bounded continuation
 │ invalid       │ → stop / fence
 │ uncertain     │ → safe degraded mode
 └───────────────┘
```

This preserves the LXXI distinction between known failure and unknown state.

## 15. Reconciliation and Authority

LXXIII establishes what state and resources exist. LXXIV establishes whether an actor is authorized to act on that reconciled state.

```text
Observed State
      ↓
Reconciliation
      ↓
Authoritative State
      ↓
Authority Validation
      ↓
Fenced Mutation
```

Therefore:

```text
Reconciled state
    ≠
Permission to mutate reconciled state
```

## 16. Authority and Capability

Capability answers:

> **Can this actor perform this class of operation?**

Authority answers:

> **Is this actor currently permitted to exercise that capability over this coordination domain?**

Both constraints apply:

```text
AcceptedMutation
    requires
Capability
 ∧
CurrentAuthority
 ∧
ValidEpoch
 ∧
ValidFence
 ∧
PolicyAllows
```

## 17. Safety Invariants

### A1 — No stale authority

```text
CurrentEpoch > ActorEpoch
    ⇒ Actor is non-authoritative
```

### A2 — No unfenced takeover

```text
NewAuthority
    ⇒
OldAuthority fenced or demonstrably incapable of accepted mutation
```

### A3 — Scoped authority

Authority is valid only within its declared coordination domain.

### A4 — Bounded authority

Leases and epochs must have explicit validity conditions.

### A5 — Quorum-aware transition

Authority changes requiring quorum must not bypass the configured quorum rule.

### A6 — Reconnection does not restore authority

A disconnected actor must re-establish authority rather than resume it implicitly.

### A7 — Safe uncertainty

When authority cannot be established safely, NROS must not assume authority merely because an actor is locally available.

## 18. Failure Matrix

| Condition | Authority response |
|---|---|
| Lease valid | Continue within bounds |
| Lease expired | Relinquish / fence |
| Epoch stale | Reject mutation |
| Quorum unavailable | No new quorum-dependent authority |
| Previous authority uncertain | Fence before takeover |
| Membership changed | Re-evaluate authority |
| Reconnection after partition | Reconcile and re-authorize |
| Conflicting authorities | Safety-first fencing / rejection |
| Unknown external mutation outcome | Reconcile before retry |

## 19. Relationship to Earlier Coordination Parts

LXXIV does not replace XXV or XLVII.

```text
XXV
Coordination primitives
      ↓
XLVII
Authoritative decision model
      ↓
LXVII
Distributed state semantics
      ↓
LXXI–LXXIII
Failure → policy → reconciliation
      ↓
LXXIV
Authority enforcement / fencing synthesis
```

Its novel contribution is the explicit connection between **reconciled state** and **fenced authority to mutate that state**.

## 20. Verification Boundary

The architecture alone cannot establish that fencing is actually effective.

Verification should eventually demonstrate, under defined conditions:

- stale epoch rejection;
- lease expiry behavior;
- takeover ordering;
- quorum loss;
- partition/rejoin behavior;
- conflicting authority attempts;
- stale-controller mutation rejection;
- reconciliation before resumed mutation;
- persistence of authority/fence state where required.

Until such evidence exists, these remain architectural requirements.

## 21. Architectural Rule

> **Coordination selects authority; epochs bound authority; leases bound time; quorum bounds collective authorization; fencing makes stale authority harmless; reconciliation establishes state before authority resumes mutation.**

## 22. Transition to Part LXXV

Part LXXIV establishes the authority boundary.

The next Part should therefore examine the **durable protocol/state machinery required to record, propagate, and verify authority transitions**, without repeating the coordination model already defined here.
