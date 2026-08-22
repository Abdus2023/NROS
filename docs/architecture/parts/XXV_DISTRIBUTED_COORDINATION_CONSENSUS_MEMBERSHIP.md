# Part XXV — Distributed Coordination, Consensus & Membership

> **Series:** NROS Architecture Series  
> **Part:** XXV  
> **Role:** Distributed coordination, membership, failure detection, leader election, leases, quorum, consensus, distributed clocks, fencing, partitions, split-brain prevention, and coordinated recovery  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXIV defined determinism, reproducibility, replay, and execution semantics. Part XXV extends those semantics across multiple nodes.

The central rule is:

> **NROS must distinguish coordination, membership, failure detection, leader election, leases, quorum, and consensus; distributed correctness must remain explicit under delay, message loss, node failure, duplication, reordering, restart, and network partition.**

## 2. Fundamental Distinctions

```text
Coordination
  ≠
Membership
  ≠
Failure detection
  ≠
Leader election
  ≠
Lease
  ≠
Quorum
  ≠
Consensus
  ≠
Distributed clock
```

Each mechanism solves a different problem.

## 3. Distributed System Model

A distributed NROS deployment may contain:

```text
Node A
Node B
Node C
...
```

with communication subject to:

```text
delay
loss
duplication
reordering
partition
reconnection
partial failure
```

The architecture must not assume that failure of communication proves failure of the remote node.

## 4. Partial Failure

Unlike a single-process failure, distributed failures may affect only part of the system:

```text
A ↔ B   ✓
A ↔ C   ✗
B ↔ C   ?
```

A node may therefore be:

```text
alive
unreachable
slow
partitioned
restarted
stale
```

at the same time from different observers' perspectives.

## 5. Membership

Membership defines which nodes are considered participants in a distributed domain.

Membership should specify:

```text
node identity
configuration epoch
generation
status
roles
eligibility
join semantics
leave semantics
revocation
```

Membership is a control-plane state, not merely a list of network addresses.

## 6. Node Identity

Node identity must remain distinguishable across restarts.

A useful conceptual identity is:

```text
node_id + generation
```

This prevents an old process instance from being confused with a new instance using the same node identity.

## 7. Membership Epoch

Membership changes should be associated with an epoch or configuration generation:

```text
Membership M1
    ↓
change
    ↓
Membership M2
```

Operations tied to M1 must not silently become valid under incompatible M2 assumptions.

## 8. Join

A node joining a distributed domain should establish:

```text
identity
membership configuration
protocol version
capabilities
security credentials
initial state
```

Joining must not automatically grant every operational authority.

## 9. Leave

A node may leave because of:

```text
planned shutdown
maintenance
administrative removal
failure
membership reconfiguration
```

The system must distinguish graceful departure from suspected failure.

## 10. Failure Detection

Failure detection produces suspicion, not omniscient knowledge.

```text
No response
   ↓
Suspect node
```

does not necessarily imply:

```text
Node is definitely dead
```

The detector's guarantees and false-positive behavior should be explicit.

## 11. Failure Detector Outputs

Possible states include:

```text
healthy
suspected
unreachable
failed
recovered
stale
```

The semantics of each state must be defined.

## 12. Timeouts

Timeouts are evidence of missing communication within a bound, not proof of physical failure.

Timeout policy should account for:

```text
network delay
processing delay
load
clock assumptions
retry policy
```

## 13. Leader Election

Leader election selects a coordinator under a defined membership configuration.

```text
Candidates
 ↓
Election protocol
 ↓
Leader identity + term
```

A leader must not rely solely on local belief that it remains leader.

## 14. Leadership Terms

Leadership should be associated with a monotonically advancing term or epoch:

```text
Leader A, term 7
        ↓
Leader B, term 8
```

An operation from an older term must not override a newer authoritative state.

## 15. Stale Leaders

Network partitions can create stale leadership:

```text
Old leader
   │ partition
   ├─────────────┐
   │             │
followers      new leader
```

The architecture must prevent stale leaders from performing operations that require current leadership.

## 16. Fencing

Fencing prevents an old authority holder from continuing to act after authority has moved elsewhere.

Conceptually:

```text
Old token: 41
New token: 42

resource accepts only current token
```

Fencing is especially important for external resources and side effects.

## 17. Leases

A lease grants authority for a bounded interval under defined renewal semantics.

```text
Acquire
 ↓
Lease valid
 ↓
Renew
 ↓
Expire / revoke
```

A lease must define behavior under uncertain communication and clock conditions.

## 18. Lease Expiration

A client must not assume:

```text
“I cannot contact the coordinator”
    ⇒
“My lease is still valid”
```

Lease validity requires the protocol's authoritative conditions.

## 19. Clock Assumptions

Lease safety depends on time semantics.

The architecture must distinguish:

```text
wall-clock time
monotonic time
logical time
coordinator time
local time
```

Unsafe clock assumptions can invalidate lease guarantees.

## 20. Quorum

A quorum is a subset of participants sufficient for a defined decision.

For N participants, a majority quorum is commonly:

```text
floor(N / 2) + 1
```

but NROS must not assume majority semantics where another quorum model is required.

## 21. Quorum Intersection

Safety often relies on quorum intersection:

```text
Q1 ∩ Q2 ≠ ∅
```

The intersection property prevents incompatible decisions from being independently finalized when the protocol's assumptions hold.

## 22. Consensus

Consensus establishes agreement on a value or ordered log under a defined failure model.

A consensus contract should specify:

```text
safety
liveness assumptions
membership
quorum
leader behavior
log/state semantics
failure model
recovery
```

## 23. Consensus vs Coordination

Coordination can help nodes organize work without requiring agreement on one authoritative value.

Consensus is stronger:

```text
coordination
    ≠
agreement protocol
```

NROS should use the weakest mechanism sufficient for the contract.

## 24. Safety and Liveness

Distributed protocols should distinguish:

```text
Safety:
    nothing forbidden happens

Liveness:
    something required eventually happens
```

A protocol may preserve safety by sacrificing progress during a partition.

## 25. Partition Behavior

During network partition:

```text
Cluster A  |  Cluster B
```

NROS must define whether each side:

```text
continues
stops
serves reads
serves writes
becomes read-only
requires quorum
```

The behavior must be explicit.

## 26. Split Brain

Split brain occurs when incompatible authorities simultaneously act as though they control the same protected resource.

Prevention may require:

```text
quorum
terms
leases
fencing
external witness
resource arbitration
```

## 27. Split-Brain Safety Invariant

A useful invariant is:

```text
At most one current authority
may perform a protected operation
for a given fenced resource and epoch.
```

The exact invariant must be adapted to the resource contract.

## 28. Replication

Replicated state may use:

```text
primary/replica
leader/follower
multi-leader
quorum replication
log replication
state-machine replication
```

The selected model determines consistency and failure semantics.

## 29. Replication Lag

A replica may be:

```text
current
slightly behind
stale
unavailable
```

Read policies must define whether stale data is acceptable.

## 30. Read Consistency

Possible read semantics include:

```text
eventual
read-your-writes
monotonic reads
causal
linearizable
snapshot
```

NROS must not label a read “consistent” without specifying the consistency model.

## 31. Write Consistency

Writes should define:

```text
acknowledgement point
replication requirement
quorum requirement
commit point
visibility point
failure behavior
```

An acknowledgement does not automatically mean durable global commitment.

## 32. Commit Semantics

The architecture should distinguish:

```text
accepted
replicated
quorum-confirmed
committed
applied
visible
```

These states may occur at different times.

## 33. Distributed Ordering

Events may be ordered using:

```text
sequence numbers
logical clocks
Lamport clocks
vector clocks
consensus log positions
causal metadata
```

A total ordering mechanism should not be introduced where causal ordering is sufficient.

## 34. Logical Clocks

Logical clocks provide ordering information without claiming physical-time synchronization.

```text
A: 5
 ↓ message
B: max(B,5)+1
```

Logical ordering and elapsed time remain separate concepts.

## 35. Causal Consistency

If:

```text
A → B
```

then observers using causal semantics should not observe B as preceding A.

This builds on Part XXIV execution causality.

## 36. Distributed Transactions

If transactions span nodes, NROS must define:

```text
prepare
commit
abort
failure recovery
participant identity
transaction timeout
idempotence
```

Distributed transactions should not be assumed atomic merely because local transactions are atomic.

## 37. Idempotent Coordination

Retries are common in distributed systems.

Control operations should preferably support stable operation identities:

```text
operation_id
    ↓
retry
    ↓
same semantic operation
```

This reduces duplicate side effects.

## 38. Duplicate Messages

Receivers should define behavior for duplicate messages:

```text
ignore
replay result
reapply safely
reject
```

Exactly-once network delivery should not be assumed.

## 39. Message Reordering

Protocols must define whether messages may arrive out of order and how receivers handle them.

Possible mechanisms:

```text
sequence numbers
buffering
causal metadata
version checks
idempotent application
```

## 40. Recovery After Restart

A restarted node should recover:

```text
identity generation
membership state
term/epoch
persistent state
pending operations
protocol position
security credentials
```

A restarted process must not accidentally resume stale authority.

## 41. Recovery and Fencing

After restart:

```text
old authority
      ↓ invalidated
new process generation
      ↓
new authority acquisition
```

This connects Part XX recovery with distributed safety.

## 42. Security Interaction

Part XXII security applies to distributed coordination:

```text
node identity
authentication
membership authorization
leader authorization
credential rotation
message integrity
replay protection
```

A correct consensus algorithm running over unauthenticated identities may still fail its system security contract.

## 43. Resource Interaction

Part XXI resource economics applies to coordination traffic:

```text
heartbeats
membership updates
replication
retries
elections
recovery traffic
```

A failure storm can become a resource-exhaustion event.

## 44. Failure Storms

When many nodes detect failure simultaneously:

```text
failure
 ↓
retries
 ↓
more traffic
 ↓
more delay
 ↓
more suspected failures
 ↓
retry amplification
```

Backoff, rate limits, jitter, and admission policy may be required.

## 45. Coordination Storm Control

Coordination mechanisms should define bounds for:

```text
heartbeat frequency
retry rate
election frequency
membership churn
state transfer
recovery concurrency
```

## 46. Deterministic Testing

Part XXIV enables controlled distributed tests using:

```text
virtual time
controlled message delivery
recorded schedules
fault injection
network partitions
node restart
message duplication
message reordering
```

This permits repeatable testing of distributed invariants without claiming production networks are deterministic.

## 47. Formal Distributed Properties

Part XIX may express invariants such as:

```text
Committed(value, term=t)
    ⇒
No conflicting value can become committed
under the protocol assumptions.
```

And fencing:

```text
epoch(old) < epoch(current)
    ⇒
old authority cannot perform protected operation.
```

These statements require explicit protocol assumptions and are not implementation evidence by themselves.

## 48. Verification Matrix

| Property | Verification question |
|---|---|
| Membership | Are node identities and generations explicit? |
| Failure | Is suspicion distinguished from certainty? |
| Election | Are leadership terms/epochs defined? |
| Stale leader | Can an old leader be fenced? |
| Lease | Are expiration semantics safe under clock uncertainty? |
| Quorum | Is the quorum rule explicit? |
| Consensus | Are safety and liveness assumptions defined? |
| Partition | Is behavior during network split explicit? |
| Split brain | Is simultaneous authority prevented? |
| Replication | Is commit/visibility semantics defined? |
| Ordering | Is event ordering explicit? |
| Restart | Can stale authority survive restart? |
| Retry | Are operations idempotent or deduplicated? |
| Security | Are nodes and control messages authenticated? |
| Resources | Are retry/recovery storms bounded? |
| Recovery | Is distributed state safely reconstructed? |
| Testing | Can partitions and faults be reproduced? |
| Formal assurance | Are distributed invariants stated with assumptions? |

## 49. What Part XXV Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production consensus protocol;
- a distributed membership service;
- formally verified leader election;
- production-grade fencing;
- linearizable storage;
- Byzantine fault tolerance;
- globally synchronized clocks;
- complete split-brain prevention;
- end-to-end distributed failure testing.

Those require implementation-specific evidence.

## 50. Transition to Part XXVI

Part XXV defines distributed coordination and agreement.

Part XXVI should define **networking, transport reliability, backpressure, congestion, connection lifecycle, routing, topology, and network fault semantics**, connecting distributed coordination to the communication substrate.

```text
Part XXIV
Determinism + reproducibility + execution semantics
        ↓
Part XXV
Distributed coordination + consensus + membership
        ↓
Part XXVI
Networking + transport + congestion + topology
```

## Canonical rule

> **NROS treats distributed authority as a protocol-governed state, not a local belief: membership, leadership, leases, quorum, consensus, ordering, and fencing must remain correct under delay, loss, duplication, reordering, restart, and partition, with every safety and liveness claim bounded by explicit failure and timing assumptions.**
