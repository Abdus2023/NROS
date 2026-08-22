# Part XLII — Networking, Communication, Discovery, Transport & Partitions

> **Series:** NROS Architecture Series  
> **Part:** XLII  
> **Role:** Networking, communication topology, discovery, connection lifecycle, transport, message semantics, delivery, backpressure, ordering, partitions, and distributed communication failure  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XLI established explicit identity and authority. Part XLII defines the communication substrate through which NROS components exchange information and control messages across process, node, tenant, and network boundaries.

The central rule is:

> **NROS never treats successful transmission as successful execution: network reachability, connection establishment, message delivery, acceptance, processing, and durable commitment are distinct states with explicit semantics.**

## 2. Fundamental Distinctions

```text
network
  ≠
connection
  ≠
transport
  ≠
message
  ≠
delivery
  ≠
processing
  ≠
commit
```

## 3. Communication Model

```text
Producer
  ↓
Transport
  ↓
Network
  ↓
Receiver
  ↓
Acceptance
  ↓
Processing
  ↓
Commit
```

Failure at any stage must have distinguishable semantics.

## 4. Communication Domains

NROS may communicate across:

```text
in-process
inter-process
same-node
same-cluster
cross-node
cross-region
external network
```

Each boundary can impose different latency, trust, reliability, and failure characteristics.

## 5. Topology

A deployment topology should identify:

```text
nodes
services
workers
agents
brokers
routers
gateways
external dependencies
```

Topology is runtime context, not automatically an authority grant.

## 6. Endpoint Identity

An endpoint should be identifiable by:

```text
endpoint_id
principal
node_id
instance_epoch
service identity
```

Identity and network address remain distinct.

## 7. Addressing

An address identifies where communication can be attempted.

```text
Identity
 ≠
Address
```

Addresses may change while logical identity remains stable.

## 8. Discovery

Discovery maps logical identities to reachable endpoints:

```text
Logical Service
      ↓
Discovery
      ↓
Endpoint Set
```

Discovery results require freshness semantics.

## 9. Discovery Freshness

A stale discovery result can route traffic to a retired endpoint.

```text
Endpoint epoch 7
Current epoch 8
      ↓
Reject / refresh
```

## 10. Service Registration

Registration should define:

```text
identity
endpoint
capabilities
version
health
expiry
epoch
```

Registration is not equivalent to authorization.

## 11. Service Deregistration

A retiring endpoint should become unavailable for new work before its final shutdown when graceful draining is required.

```text
Active
 ↓
Draining
 ↓
Deregistered
 ↓
Stopped
```

## 12. Connection Lifecycle

```text
Disconnected
 ↓
Connecting
 ↓
Authenticated
 ↓
Established
 ↓
Draining
 ↓
Closed
```

Invalid transitions must be rejected.

## 13. Connection Authentication

Part XLI security semantics apply to communication establishment.

A reachable endpoint is not automatically a trusted endpoint.

## 14. Connection Ownership

Connections should have explicit ownership and lifecycle responsibility.

This prevents abandoned connections and uncontrolled resource consumption.

## 15. Connection Limits

Connections consume resources:

```text
file descriptors
memory
CPU
buffers
network bandwidth
```

Part XXXVII quotas and admission controls apply.

## 16. Transport

Transport provides delivery mechanics between endpoints.

It may offer properties such as:

```text
ordered delivery
reliable delivery
streaming
datagrams
flow control
integrity
confidentiality
```

The exact guarantees must be explicit.

## 17. Message

A message should have stable identity and semantic metadata:

```text
message_id
source
recipient
schema
version
correlation_id
sequence
epoch
payload
```

## 18. Message Identity

Unique message IDs support:

```text
deduplication
correlation
replay detection
tracing
```

## 19. Delivery Semantics

NROS should explicitly distinguish:

```text
at-most-once
at-least-once
best-effort
application-confirmed
```

Exactly-once behavior requires stronger end-to-end conditions than merely using a reliable transport.

## 20. Delivery vs Processing

```text
Message delivered
    ≠
Message processed
```

A receiver may accept bytes and then fail before processing them.

## 21. Processing vs Commit

```text
Message processed
    ≠
State committed
```

The application must define when processing becomes durable or externally effective.

## 22. Acknowledgement

Acknowledgements should identify what they confirm:

```text
received
accepted
processed
committed
```

An ambiguous ACK is dangerous for retries.

## 23. Negative Acknowledgement

Where supported, a receiver may explicitly reject a message:

```text
Message
 ↓
NACK(reason)
```

Reasons should be machine-actionable where practical.

## 24. Retry Semantics

Retries require explicit classification:

```text
transient
permanent
unknown
```

Permanent failures should not create infinite retry loops.

## 25. Retry Identity

Retries should normally preserve the logical operation identity while allowing individual transmission attempts to be distinguished.

```text
operation_id
 ├─ attempt 1
 ├─ attempt 2
 └─ attempt 3
```

## 26. Idempotency

Operations retried under at-least-once delivery should be idempotent or protected by deduplication.

## 27. Deduplication

A receiver may maintain processed operation identities:

```text
operation_id
 ↓
seen?
 ├─ yes → suppress duplicate
 └─ no  → process
```

Deduplication state itself consumes resources and requires lifecycle policy.

## 28. Ordering

Ordering must identify its scope:

```text
per connection
per sender
per key
per partition
per workflow
```

Global ordering should not be assumed without an explicit mechanism.

## 29. Sequence Numbers

Sequences detect gaps and duplicates:

```text
10
11
13
```

The receiver can identify missing `12` when the protocol guarantees sequence continuity.

## 30. Reordering

Networks may deliver messages out of order.

```text
A
C
B
```

The protocol must define whether to buffer, reject, process independently, or request recovery.

## 31. Flow Control

Flow control prevents a producer from overwhelming a receiver:

```text
Producer
 ↓
credit / window
 ↓
Receiver capacity
```

## 32. Backpressure

```text
Receiver overloaded
 ↓
Backpressure
 ↓
Producer slows / queues / rejects
```

Backpressure should propagate according to explicit policy.

## 33. Queue Bounds

Queues should be bounded or have explicit admission policies.

```text
Queue
 ↓ capacity reached
Reject / shed / block / spill
```

Unbounded buffering is not a safety mechanism.

## 34. Message Size

Maximum message size should be explicit.

Oversized messages should be rejected before uncontrolled allocation.

## 35. Fragmentation

If messages can exceed transport frame sizes, fragmentation must define:

```text
message identity
fragment index
fragment count/termination
integrity
expiry
resource limits
```

## 36. Reassembly

Reassembly buffers consume resources and require:

```text
maximum size
maximum duration
maximum concurrent messages
failure cleanup
```

## 37. Compression

Compression can reduce bandwidth but consumes CPU and may create security or resource risks.

Compression policies should be bounded and negotiated explicitly.

## 38. Serialization

Wire schemas should define:

```text
encoding
schema ID
version
compatibility
limits
```

Deserialization must not trust remote input.

## 39. Schema Evolution

Compatible evolution should define:

```text
producer version
consumer version
compatibility mode
migration path
```

## 40. Unknown Fields

Protocols should explicitly define whether unknown fields are:

```text
ignored
preserved
rejected
```

## 41. Unknown Message Types

Unknown message types should fail according to protocol policy rather than being interpreted heuristically.

## 42. Transport Security

Security-sensitive links should provide appropriate:

```text
peer authentication
confidentiality
integrity
freshness
replay protection
```

Part XLI defines the authority model.

## 43. Message Authorization

A successfully authenticated connection does not imply authorization for every message.

```text
Connection authenticated
        ↓
Message authorization
        ↓
Accept / Deny
```

## 44. Tenant Routing

Routing must preserve tenant isolation.

```text
Tenant A message
    ↓
Tenant A authorized path
```

A shared transport does not imply shared authority.

## 45. Routing

Routing maps a destination to a path:

```text
Destination
 ↓
Route selection
 ↓
Next hop
```

Routes may be dynamic and therefore require freshness and validation.

## 46. Route Failure

When a route fails:

```text
Route unavailable
 ↓
Retry / alternate route / reject
```

The response depends on delivery and safety requirements.

## 47. Route Loops

Routing must prevent uncontrolled loops using explicit mechanisms such as:

```text
hop limit
visited-set
route epoch
TTL
```

## 48. Network Partition

A partition means components cannot communicate reliably even though they may continue executing locally.

```text
Cluster A  ║  Cluster B
           ║
        partition
```

Partition handling must be explicit.

## 49. Partition Semantics

During partition, NROS may choose among:

```text
continue independently
pause
degrade
quarantine
fail closed
```

The choice is operation-specific.

## 50. Split-Brain Protection

When multiple components believe they are authoritative:

```text
Authority A
     ╲
      conflict
     ╱
Authority B
```

Epochs, leases, quorum, fencing, or another explicit mechanism may be required.

## 51. Lease-Based Authority

A communication lease can provide bounded authority:

```text
Lease issued
 ↓
valid
 ↓
expiry / renewal failure
 ↓
invalid
```

Part XXXVI time semantics apply.

## 52. Quorum

Distributed decisions may require quorum:

```text
N participants
 ↓
required quorum
 ↓
decision
```

Quorum requirements must be explicit and failure-aware.

## 53. Availability vs Consistency

Partition handling can force trade-offs.

NROS should define behavior per operation rather than relying on a universal assumption.

## 54. Stale Data

A disconnected component may operate on stale state:

```text
Last known state
 ↓
partition
 ↓
current remote state unknown
```

Safety-critical operations should account for staleness explicitly.

## 55. Freshness

Messages and discovery records may carry:

```text
timestamp
sequence
epoch
lease expiry
version
```

Freshness requirements are operation-specific.

## 56. Connection Failure Detection

Failure detection is not instantaneous truth.

```text
No response
    ≠
Remote process definitely stopped
```

The architecture must represent suspicion or uncertainty where appropriate.

## 57. Failure Detector

A detector can classify a peer as:

```text
healthy
suspected
unreachable
recovered
```

rather than claiming certainty from a timeout alone.

## 58. Timeout Semantics

Timeouts should identify what expired:

```text
connect timeout
send timeout
receive timeout
processing deadline
commit deadline
```

Part XXXVI provides temporal semantics.

## 59. Deadline Propagation

A request deadline can propagate across communication hops:

```text
Request deadline
 ↓
Service A
 ↓
Service B
 ↓
Service C
```

Downstream work must not assume unlimited remaining time.

## 60. Cancellation Propagation

Cancellation may propagate through communication paths:

```text
Caller cancels
 ↓
Request cancelled
 ↓
Downstream cancellation
```

The receiver must define whether cancellation is best-effort or guaranteed.

## 61. Graceful Shutdown

Connections should drain when possible:

```text
Stop accepting new work
 ↓
Finish permitted work
 ↓
Flush required messages
 ↓
Close
```

## 62. Abrupt Failure

Crash or network loss may interrupt communication at any point.

Recovery must not assume a sender knows whether a message committed.

## 63. Ambiguous Outcome

A critical case is:

```text
Send
 ↓
connection lost
 ↓
Did receiver commit?
       ↓
     Unknown
```

The protocol must resolve this through idempotency, query, reconciliation, or explicit uncertainty handling.

## 64. Reconciliation

After reconnect:

```text
Local state
   ↕
Remote state
   ↓
Reconcile
```

Reconciliation must use version, epoch, or another conflict-resolution mechanism.

## 65. Anti-Entropy

Replicas may exchange state summaries:

```text
Digest A
 ↕
Digest B
 ↓
Missing state
```

Only explicitly authorized state should be synchronized.

## 66. Gossip

Gossip can distribute membership or health information but may provide probabilistic convergence rather than immediate consistency.

## 67. Membership

Cluster membership should distinguish:

```text
known
joining
active
suspected
leaving
removed
```

Membership changes must be observable.

## 68. Membership Epoch

A membership epoch can fence stale topology views:

```text
Membership epoch 10
 ↓
Membership epoch 11
```

Actors using stale membership may need to refresh before sensitive actions.

## 69. Discovery Failure

If discovery fails, cached endpoints may be used only within explicit freshness limits.

## 70. Communication Resource Pressure

Network pressure can affect:

```text
bandwidth
buffers
connections
queues
CPU
```

Communication admission must integrate with Part XXXVII.

## 71. Priority

Messages may have priorities:

```text
control
recovery
security
normal
telemetry
```

Priority must not become an unrestricted starvation mechanism.

## 72. Traffic Isolation

Critical control traffic may require dedicated capacity or quotas so that bulk telemetry cannot prevent recovery or security communication.

## 73. Rate Limiting

Rate limits can apply per:

```text
principal
tenant
endpoint
message class
connection
route
```

## 74. Circuit Breaking

Repeated downstream failure may trigger:

```text
Closed
 ↓ failures
Open
 ↓ cooldown
Half-open
 ↓ probe
Closed / Open
```

Circuit state must be bounded and observable.

## 75. Load Shedding

When overloaded:

```text
Preserve critical traffic
 ↓
Shed lower-value work
```

Load shedding should be policy-driven rather than arbitrary.

## 76. Communication Storms

Retries, reconnects, discovery, and health probes can amplify load:

```text
Failure
 ↓
Retries
 ↓
More load
 ↓
More failure
```

Backoff and jitter should prevent synchronized retry storms.

## 77. Retry Backoff

Backoff should be bounded:

```text
attempt 1 → short delay
attempt 2 → longer delay
...
maximum delay
```

## 78. Jitter

Randomized delay can prevent large populations from retrying simultaneously.

## 79. Retry Budget

Retries should consume an explicit budget:

```text
retry budget
 ↓
exhausted
 ↓
stop / escalate
```

## 80. Security and Retry

Retries must not bypass authorization, freshness, capability expiry, or revocation.

A retried operation remains subject to current security policy.

## 81. Observability

Communication should expose:

```text
connection state
message counts
latency
queue depth
retries
rejections
timeouts
loss
partition state
```

Part XL defines observability semantics.

## 82. Communication Evidence

Important control messages should support provenance:

```text
message_id
source
recipient
policy context
configuration epoch
transport attempt
outcome
```

## 83. Distributed Tracing

Communication spans should preserve correlation across boundaries:

```text
Trace
 ├─ caller span
 ├─ transport span
 ├─ receiver span
 └─ downstream span
```

## 84. Protocol Compatibility

A connection should negotiate or validate protocol compatibility before exchanging unsupported messages.

## 85. Version Skew

During rollout:

```text
Node A → version N
Node B → version N+1
```

Communication must follow the compatibility contract established for the transition.

## 86. Safe Protocol Migration

A protocol migration should generally support:

```text
old + new readers
old + new writers
 ↓
controlled migration
 ↓
remove old version
```

## 87. Formal Delivery Invariant

```text
Delivered(M)
    ⇒
TransportAccepted(M)
```

but:

```text
Delivered(M)
    ⇏
Committed(M)
```

## 88. Formal Authorization Invariant

```text
Accept(M)
    ⇒
Authorized(Source, Action, Destination, Context)
```

## 89. Formal Freshness Invariant

```text
SensitiveAction(M)
    ⇒
Fresh(M, RequiredWindow)
```

## 90. Formal Epoch Invariant

```text
Epoch(M) < CurrentEpoch(Resource)
    ⇒
Reject(M)
```

where epoch fencing is required.

## 91. Formal Queue Invariant

```text
QueueDepth ≤ ConfiguredBound
```

unless an explicitly bounded external spill mechanism exists.

## 92. Formal Retry Invariant

```text
Retries(Operation) ≤ RetryBudget(Operation)
```

## 93. Formal Partition Invariant

```text
PartitionDetected
    ⇒
PartitionPolicy(Operation)
```

The system must not invent unsafe behavior during communication loss.

## 94. Verification Matrix

| Property | Verification question |
|---|---|
| Identity | Are endpoints distinguishable from addresses? |
| Discovery | Are results freshness-bounded? |
| Authentication | Are communication peers authenticated where required? |
| Authorization | Is every sensitive message authorized? |
| Delivery | Is delivery semantics explicit? |
| Processing | Is processing distinguishable from receipt? |
| Commit | Is durable/external commitment distinguishable? |
| Ordering | Is ordering scope explicit? |
| Deduplication | Are retries safe under duplicate delivery? |
| Backpressure | Are queues bounded? |
| Retry | Are retries bounded and classified? |
| Partition | Is partition behavior explicit? |
| Split brain | Can stale authority be fenced? |
| Freshness | Can stale messages and discovery be rejected? |
| Resource safety | Are network resources bounded? |
| Security | Are transport and message security separated? |
| Observability | Can communication failures be reconstructed? |
| Compatibility | Can protocol version skew be handled safely? |
| Recovery | Can ambiguous outcomes be reconciled? |
| Formal assurance | Are delivery, epoch, queue, and retry invariants explicit? |

## 95. What Part XLII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- production distributed discovery;
- a universal transport protocol;
- complete partition tolerance;
- consensus or quorum implementation;
- production service mesh behavior;
- complete message deduplication;
- formally verified delivery semantics;
- universal exactly-once processing;
- complete network threat mitigation;
- production-grade distributed tracing.

Those require implementation-specific evidence.

## 96. Transition to Part XLIII

Part XLII establishes distributed communication semantics.

Part XLIII should define **storage, persistence, durability, transactional state, consistency, replication, snapshots, recovery points, data lifecycle, and crash-consistency semantics**.

```text
Part XLI
Security + identity + trust + authorization + capabilities
        ↓
Part XLII
Networking + communication + discovery + transport + partitions
        ↓
Part XLIII
Storage + persistence + durability + consistency + replication
```

## Canonical rule

> **NROS treats communication as a sequence of independently meaningful states—reachable, connected, authenticated, delivered, accepted, processed, and committed—while bounding queues, retries, stale state, partitions, and network resource consumption so distributed failure cannot silently become semantic ambiguity.**
