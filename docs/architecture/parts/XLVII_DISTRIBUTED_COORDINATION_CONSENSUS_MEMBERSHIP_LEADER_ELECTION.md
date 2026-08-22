# Part XLVII — Distributed Coordination, Consensus, Membership & Authoritative State

> **Series:** NROS Architecture Series  
> **Part:** XLVII  
> **Role:** Distributed coordination, membership, leader election, quorum, consensus, leases, epochs, authoritative state transitions, split-brain prevention, reconfiguration, and coordination safety  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Parts XLIII–XLVI established persistence, scheduling, execution, and resilience. Part XLVII defines how multiple NROS nodes coordinate when more than one node could otherwise act concurrently on shared authoritative state.

The central rule is:

> **NROS must distinguish the ability to execute from the authority to decide: distributed coordination establishes which actor is authoritative for a decision, under which epoch, membership, quorum, and consensus rules.**

## 2. Execution vs Authority

```text
Can execute
    ≠
Can decide
    ≠
Is authoritative
```

A worker may execute a task without being authorized to establish cluster-wide state.

## 3. Coordination Plane

```text
Membership
 ↓
Leader Election
 ↓
Quorum
 ↓
Consensus
 ↓
Authoritative State
 ↓
Committed Decision
```

## 4. Node Identity

Each participating node should have stable identity:

```text
node_id
instance_id
epoch
capabilities
fault domain
```

Ephemeral process identity must not silently replace logical node identity where durable coordination depends on it.

## 5. Membership

Membership defines which nodes are eligible to participate in coordination.

```text
Members = {N1, N2, N3}
```

Membership changes are themselves authoritative state transitions.

## 6. Membership States

Nodes may be:

```text
joining
active
suspected
leaving
removed
```

The transition rules must be explicit.

## 7. Membership Authority

A node must not unilaterally declare another authoritative member removed unless the membership protocol grants it that authority.

## 8. Membership Epoch

Membership changes can advance an epoch:

```text
Epoch 12
 ↓ membership change
Epoch 13
```

Old decisions can then be fenced.

## 9. Leader

A leader is an authority role, not merely the most responsive node.

```text
Leader
 ↓
Propose
 ↓
Consensus
 ↓
Commit
```

## 10. Leader Election

Election establishes a single authoritative leader for a defined scope and epoch when the protocol requires one.

```text
Candidates
 ↓
Election
 ↓
Leader(epoch E)
```

## 11. Election Safety

At most one leader should be authoritative for the same scope and epoch under the protocol's failure assumptions.

## 12. Election Liveness

When the system assumptions hold, an eligible leader should eventually be elected.

Safety and liveness are separate properties.

## 13. Leader Lease

A lease can bound leader authority:

```text
Lease acquired
 ↓
Leader active
 ↓
Lease expires
 ↓
Authority ends
```

Lease expiry must prevent stale decisions.

## 14. Clock Assumptions

Lease safety depends on explicit timing assumptions.

Where clock uncertainty matters, the protocol must account for bounded clock error or use logical mechanisms instead.

## 15. Quorum

A quorum is a set of members sufficient for a defined decision.

Quorum rules depend on membership and protocol semantics.

## 16. Majority Quorum

For a fixed odd-sized membership:

```text
N = 3
quorum = 2
```

But quorum size alone does not define a complete consensus protocol.

## 17. Quorum Intersection

Safety requires relevant quorums to intersect according to the protocol's assumptions.

```text
Q1 ∩ Q2 ≠ ∅
```

where the intersection contains sufficient authority to prevent conflicting commits.

## 18. Read vs Write Quorum

Systems may distinguish:

```text
read quorum
write quorum
```

Their intersection and consistency guarantees must be explicit.

## 19. Consensus

Consensus provides agreement on authoritative decisions under the protocol's failure model.

```text
Proposal
 ↓
Agreement
 ↓
Commit
```

## 20. Consensus Properties

A consensus protocol should define:

```text
agreement
validity
termination / liveness
```

under explicit assumptions.

## 21. Agreement

Correct participants must not commit conflicting decisions for the same consensus instance.

## 22. Validity

A committed value must satisfy the protocol's validity rules.

## 23. Termination

When the required system assumptions hold, a consensus instance should eventually reach a decision.

## 24. Consensus vs Replication

```text
replication
 ≠
consensus
```

Replication copies state; consensus establishes authoritative agreement about state transitions.

## 25. Log-Based Coordination

A replicated log can represent ordered authoritative transitions:

```text
Entry 1
Entry 2
Entry 3
...
```

The log must have explicit ordering and commit semantics.

## 26. Commit Index

A replicated entry should distinguish:

```text
appended
replicated
committed
applied
```

This parallels the execution distinction in Part XLV.

## 27. Apply vs Commit

```text
Committed
    ≠
Applied
```

A node may receive a committed decision before applying it locally.

## 28. State Machine Replication

A deterministic state machine can apply the same committed sequence:

```text
same initial state
 +
same committed log
 ↓
equivalent state
```

## 29. Determinism Requirement

State-machine replication requires deterministic application semantics or explicit handling of nondeterministic operations.

## 30. External Effects

Consensus does not automatically make external side effects exactly once.

```text
Consensus commit
    ≠
External effect commit
```

External effects require idempotency, transactional coordination, or reconciliation.

## 31. Proposal Identity

Every proposal should have stable identity:

```text
proposal_id
origin
term/epoch
sequence
```

Duplicate proposals should converge safely.

## 32. Proposal Validation

Before accepting a proposal, participants should validate:

```text
authority
membership
epoch
schema
policy
```

## 33. Stale Proposal

```text
Proposal epoch < current epoch
        ↓
      reject
```

## 34. Epoch / Term

An epoch or term distinguishes successive authority periods:

```text
term 7
 ↓
term 8
```

Stale terms lose authority.

## 35. Fencing

Fencing converts logical authority changes into enforceable rejection of stale actions.

```text
Old epoch
 ↓
Storage / scheduler / worker
 ↓
Reject
```

## 36. Split Brain

Split brain occurs when independent partitions both believe they can make authoritative decisions.

```text
Partition A → leader
Partition B → leader
```

## 37. Split-Brain Prevention

Controls may include:

```text
quorum
consensus
leases
epochs
fencing
```

No single mechanism is universally sufficient.

## 38. Minority Partition

A minority partition should lose authority for operations requiring quorum.

It may continue local non-authoritative work only if policy explicitly permits it.

## 39. Network Partition

```text
Network partition
 ↓
Cannot communicate
```

The system must distinguish inability to communicate from evidence that the peer has failed.

## 40. Failure Detector

Failure detectors provide suspicion, not necessarily truth.

```text
Suspect(N)
 ≠
Prove(N failed)
```

Coordination actions must account for this uncertainty.

## 41. Membership Reconfiguration

Membership changes should follow a safe transition protocol:

```text
Current configuration
 ↓
Proposed configuration
 ↓
Validated transition
 ↓
New configuration
```

## 42. Joint Configuration

For protocols that require it, transition may temporarily involve both old and new configurations:

```text
C_old + C_new
 ↓
commit transition
 ↓
C_new
```

## 43. Removing a Failed Leader

Leader removal must be based on protocol authority, not merely local timeout.

## 44. Leader Handoff

Graceful handoff can reduce disruption:

```text
Leader A
 ↓ transfer
Leader B
```

The handoff must preserve ordering and authority.

## 45. Leadership Revocation

Leader authority ends when:

```text
term changes
lease expires
membership removes leader
protocol revokes authority
```

## 46. Old Leader Behavior

A demoted leader may remain alive but must stop authoritative actions:

```text
alive
 ≠
authoritative
```

## 47. Coordination Backpressure

Consensus traffic competes for resources.

Control-plane capacity should be protected from ordinary workload saturation.

## 48. Priority of Coordination

Emergency control operations may require reserved capacity for:

```text
leader election
membership change
fencing
recovery
operator control
```

## 49. Coordination Timeouts

Timeouts should trigger protocol transitions rather than directly asserting facts.

```text
timeout
 ↓
suspect / retry / election
```

## 50. Retry Safety

Consensus and coordination retries must be idempotent with respect to proposal identity.

## 51. Duplicate Messages

Duplicate coordination messages should not create duplicate authoritative transitions.

## 52. Out-of-Order Messages

Messages should be checked against:

```text
term
sequence
membership
causal context
```

## 53. Message Authentication

Coordination messages require authenticated node identity and authorization.

Part XLI security semantics apply.

## 54. Authorization of Coordination

Not every authenticated node should necessarily be authorized to:

```text
propose
vote
commit
change membership
revoke authority
```

## 55. Configuration Authority

Membership and consensus configuration should have a clearly defined administrative authority.

## 56. Persistent Coordination State

Critical coordination state should survive node restart where required:

```text
term
voted-for / election state
membership
committed position
```

Part XLIII durability semantics apply.

## 57. Crash Recovery

After restart, a node must reconstruct coordination state before making authoritative decisions.

```text
Persistent state
 ↓
Recover
 ↓
Validate epoch
 ↓
Rejoin
```

## 58. Rejoining Node

A recovered node should not immediately resume authority using stale state.

It must synchronize according to membership protocol.

## 59. Catch-Up

A lagging replica may catch up using:

```text
log replication
snapshot
state transfer
```

## 60. Snapshot Installation

Snapshot installation must validate:

```text
snapshot identity
configuration
integrity
epoch
log position
```

## 61. Snapshot + Log

Recovery may combine:

```text
snapshot
 +
log suffix
```

to reconstruct current state.

## 62. Compaction

Committed historical log entries may be compacted once recovery guarantees permit it.

Compaction must not destroy required audit or recovery evidence.

## 63. Applied Position

Nodes should distinguish:

```text
last received
last replicated
last committed
last applied
```

## 64. Linearizability Boundary

If NROS exposes linearizable operations, it must define the exact coordination boundary that establishes linearization.

## 65. Sequential Consistency

If weaker ordering is used, the contract must state it explicitly rather than implying linearizability.

## 66. Consistency Model

Possible models include:

```text
eventual
causal
sequential
linearizable
serializable
```

The selected model is workload-specific.

## 67. Coordination Scope

Not every operation needs global consensus.

Coordination scope may be:

```text
task
workflow
tenant
service
node
cluster
region
```

Avoiding unnecessary global coordination improves scalability.

## 68. Sharded Authority

Authority can be partitioned:

```text
Shard A → Leader A
Shard B → Leader B
Shard C → Leader C
```

Each shard has its own epoch and coordination state where appropriate.

## 69. Cross-Shard Operations

Cross-shard operations require explicit atomicity or compensation semantics.

```text
Shard A
   ↘
    transaction / coordination
   ↗
Shard B
```

## 70. Tenant Isolation

A tenant should not gain authority over another tenant's coordination scope merely because both share infrastructure.

## 71. Multi-Tenant Quorum

Quorum resources should be protected from one tenant exhausting coordination capacity.

## 72. Authority Delegation

A leader may delegate execution authority without delegating final consensus authority.

```text
Leader
 ↓ delegate
Worker
```

Worker actions remain bounded by the delegated scope.

## 73. Capability-Based Coordination

Delegated authority may be represented as scoped capabilities:

```text
capability
 =
principal + scope + operation + expiry/epoch
```

## 74. Lease Renewal

Lease renewal should require continued eligibility:

```text
valid member
 ∧
current epoch
 ∧
protocol conditions
```

## 75. Lease Expiration

After expiration:

```text
Leader
 ↓
No longer authoritative
```

unless safely renewed before the expiration boundary.

## 76. Clock Skew

Clock uncertainty can make naive time-based leases unsafe.

NROS should use explicit clock assumptions and safety margins.

## 77. Logical Time

Logical clocks can support ordering without claiming wall-clock synchronization.

Relevant semantics originate in Part XXXVI.

## 78. Causality

Coordination events should preserve causal relationships where required:

```text
Membership change
 ↓
Leader election
 ↓
Proposal
 ↓
Commit
```

## 79. Coordination Evidence

Every authoritative transition should be attributable:

```text
who
what
when/logical time
term/epoch
membership
proposal
quorum evidence
result
```

## 80. Auditability

Coordination history should support reconstruction of:

```text
Why was this leader elected?
Why was this proposal accepted?
Which membership was active?
Which quorum authorized it?
```

## 81. Byzantine Considerations

If NROS assumes Byzantine faults, the coordination protocol must explicitly state the required assumptions and quorum thresholds.

If it assumes crash faults only, Byzantine tolerance must not be implied.

## 82. Crash Fault Model

A crash-fault model assumes nodes may stop, restart, or become unreachable without intentionally producing arbitrary malicious protocol messages.

## 83. Byzantine Fault Model

Byzantine tolerance additionally considers arbitrary or adversarial behavior.

The security and consensus design must be substantially stronger.

## 84. Trust Boundary

Consensus participants require an explicit trust model:

```text
trusted
authenticated
authorized
correct
```

These properties are not interchangeable.

## 85. Reconfiguration Safety

Changing membership must not create a period in which conflicting configurations can both authorize conflicting decisions.

## 86. Configuration Commit

A new membership configuration becomes authoritative only at its defined commit point.

## 87. Configuration Rollback

Rollback of membership requires an explicit protocol; silently reverting configuration can resurrect stale authority.

## 88. Recovery + Consensus

After a major recovery event:

```text
Recover state
 ↓
Re-establish membership
 ↓
Validate quorum
 ↓
Elect authority if required
 ↓
Resume decisions
```

## 89. Consensus + Scheduling

Schedulers requiring cluster-wide authority should submit authoritative scheduling decisions through the coordination boundary.

Local scheduling may remain independent where policy permits.

## 90. Consensus + Execution

Execution workers must validate that delegated authority remains current before performing protected operations.

## 91. Consensus + Persistence

Consensus commit and durable persistence must be aligned:

```text
Consensus decision
 ↔
Durable authoritative state
```

The exact ordering is protocol-specific but must be explicit.

## 92. Coordination + Recovery

Recovery must not promote a stale node simply because it possesses a locally complete snapshot.

## 93. Formal Single-Leader Invariant

```text
AuthoritativeLeader(Scope, Epoch)
    ≤ 1
```

under the protocol's stated assumptions.

## 94. Formal Stale-Term Invariant

```text
Term(Action) < CurrentTerm
    ⇒
Reject(Action)
```

## 95. Formal Quorum Invariant

```text
Commit(Decision)
    ⇒
RequiredQuorumEvidence(Decision)
```

## 96. Formal Membership Invariant

```text
Authorize(Node, Scope)
    ⇒
Node ∈ ActiveMembership(Scope)
```

## 97. Formal Recovery Invariant

```text
Rejoin(Node)
    ⇒
StateSynchronized(Node)
 ∧
CurrentEpoch(Node)
```

## 98. Verification Matrix

| Property | Verification question |
|---|---|
| Membership | Are membership states and transitions explicit? |
| Leadership | Can more than one node become authoritative? |
| Election | Are safety and liveness separately defined? |
| Quorum | Are quorum intersection requirements explicit? |
| Consensus | Are agreement, validity, and termination defined? |
| Epochs | Can stale terms be rejected? |
| Fencing | Can stale actors be physically/logically prevented? |
| Split brain | Can minority partitions lose authority? |
| Reconfiguration | Are old/new configurations transitioned safely? |
| Persistence | Does coordination state survive restart? |
| Recovery | Can stale nodes rejoin safely? |
| Replication | Are appended, committed, and applied states distinct? |
| Determinism | Can replicated state machines converge? |
| Security | Are coordination messages authenticated and authorized? |
| Delegation | Is execution authority bounded by scope? |
| Leases | Are clock assumptions explicit? |
| Cross-shard | Are multi-shard atomicity semantics explicit? |
| Evidence | Can authoritative decisions be reconstructed? |
| Fault model | Is crash vs Byzantine behavior explicit? |
| Scalability | Is global coordination avoided where unnecessary? |

## 99. What Part XLVII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production consensus protocol;
- production leader election;
- formally verified quorum safety;
- Byzantine fault tolerance;
- production multi-node membership management;
- universal linearizable storage;
- complete cross-shard transactions;
- production split-brain prevention across every deployment topology.

Those require implementation-specific evidence.

## 100. Transition to Part XLVIII

Part XLVII establishes distributed authority and coordination.

Part XLVIII should define **the NROS data/control plane boundary, protocol architecture, message envelopes, command/query/event semantics, version negotiation, compatibility, and wire-level contracts**.

```text
Part XLVI
Supervision + resilience + recovery
        ↓
Part XLVII
Distributed coordination + consensus + membership + authority
        ↓
Part XLVIII
Protocols + message model + compatibility + wire contracts
```

## Canonical rule

> **NROS does not infer distributed authority from reachability: authoritative decisions require explicit membership, epoch, quorum, consensus, fencing, and persistence semantics, with stale actors prevented from producing conflicting state.**
