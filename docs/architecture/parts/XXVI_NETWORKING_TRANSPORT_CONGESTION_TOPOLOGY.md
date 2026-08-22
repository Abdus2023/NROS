# Part XXVI — Networking, Transport, Congestion & Topology

> **Series:** NROS Architecture Series  
> **Part:** XXVI  
> **Role:** Network abstraction, transport semantics, connection lifecycle, delivery, ordering, reliability, flow control, backpressure, congestion, routing, topology, network faults, and recovery  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXV defined distributed coordination, consensus, membership, and authority. Part XXVI defines the communication substrate on which those distributed mechanisms operate.

The central rule is:

> **NROS must distinguish delivery, reliability, ordering, flow control, backpressure, congestion control, routing, and consistency; transport behavior must remain explicit under loss, delay, duplication, reordering, disconnection, overload, and topology change.**

## 2. Fundamental Distinctions

```text
Delivery
  ≠
Reliability
  ≠
Ordering
  ≠
Flow control
  ≠
Backpressure
  ≠
Congestion control
  ≠
Routing
  ≠
Consistency
```

A reliable transport does not automatically provide ordering appropriate to every application, and ordering does not imply successful processing.

## 3. Network Model

A network path may introduce:

```text
delay
jitter
loss
duplication
reordering
corruption
disconnection
partition
bandwidth variation
congestion
route changes
```

NROS protocols must define which failures are tolerated and how they are surfaced.

## 4. Network Boundary

The architectural boundary is:

```text
Application semantics
        ↓
NROS messaging
        ↓
Transport abstraction
        ↓
Network
        ↓
Remote transport
        ↓
Remote messaging
```

Application code should not need to infer transport guarantees from implementation accidents.

## 5. Transport Contract

A transport contract should specify:

```text
connection model
message boundaries
delivery semantics
ordering semantics
maximum message size
reliability
flow control
failure reporting
security properties
lifecycle
```

## 6. Connection Lifecycle

A connection can be modeled as:

```text
Created
  ↓
Connecting
  ↓
Established
  ↓
Active
  ↓
Draining
  ↓
Closed
```

Failure may transition the connection into recovery or terminal closure depending on policy.

## 7. Connection Identity

Connections should be distinguishable from nodes and application identities:

```text
node identity
    ≠
process generation
    ≠
connection identity
    ≠
request identity
```

This distinction prevents stale connection state from being mistaken for current authority.

## 8. Handshake

A connection handshake may negotiate:

```text
protocol version
capabilities
compression
framing
maximum message size
security parameters
flow-control parameters
```

Negotiation must produce an explicitly supported configuration.

## 9. Version Negotiation

Peers should identify compatible protocol versions before exchanging messages that depend on version-specific semantics.

```text
Supported(A)
      ∩
Supported(B)
      ↓
Negotiated version
```

An empty intersection must produce an explicit failure rather than ambiguous behavior.

## 10. Capability Negotiation

Capabilities may describe optional behavior:

```text
feature A
feature B
extension C
```

Capabilities are not authorization. Part XXII security remains authoritative for permission.

## 11. Framing

Message boundaries may use:

```text
length prefix
fixed-size records
delimiters
transport framing
stream multiplexing
```

Framing must remain unambiguous under fragmentation and concatenation.

## 12. Fragmentation

A single logical message may be transmitted through multiple transport fragments:

```text
Message M
 ↓
F1 + F2 + F3
 ↓
Message M
```

Fragmentation must not be confused with multiple logical messages.

## 13. Maximum Message Size

The transport contract should define limits for:

```text
logical message
frame
fragment
buffer
reassembly state
```

Oversized messages should fail predictably rather than causing uncontrolled allocation.

## 14. Delivery Semantics

Possible delivery contracts include:

```text
best effort
at most once
at least once
application-level deduplication
transactional delivery
```

“Exactly once” should not be claimed merely because a transport retries.

## 15. At-Most-Once

At-most-once delivery attempts to avoid duplicate delivery, potentially at the cost of lost messages.

The contract must define failure behavior around acknowledgement and retransmission.

## 16. At-Least-Once

At-least-once delivery may produce duplicates:

```text
send M
 ↓
retry
 ↓
M + M
```

Consumers therefore often require idempotence or deduplication.

## 17. Exactly-Once Semantics

Exactly-once application semantics generally require more than transport delivery.

They may require:

```text
stable operation identity
idempotent application
transactional state transition
deduplication
commit semantics
```

## 18. Request Identity

Requests should have stable identities where retries are possible:

```text
request_id
operation_id
attempt_id
```

The distinction allows a receiver to recognize retries of the same semantic operation.

## 19. Acknowledgement

Acknowledgements must define what they mean:

```text
received
parsed
validated
queued
processed
persisted
replicated
committed
```

An ACK without a defined point in the processing pipeline is ambiguous.

## 20. Ordering

Transport ordering may be:

```text
none
per-connection
per-stream
per-key
causal
global
```

The strongest ordering guarantee should not be imposed unless required.

## 21. Ordering vs Processing

Receiving messages in order does not imply processing them in order.

```text
transport order
      ≠
application execution order
```

Part XXIV execution semantics governs the latter.

## 22. Flow Control

Flow control protects a receiver from a sender that produces data faster than it can consume it.

```text
sender rate
    ↓
receiver capacity
    ↓
flow-control window
```

## 23. Backpressure

Backpressure propagates inability to accept more work toward upstream producers:

```text
consumer saturated
       ↑
producer throttled
       ↑
upstream workload reduced
```

Part XIII dataflow semantics defines the broader flow-control model.

## 24. Backpressure vs Congestion Control

```text
Backpressure:
    protects application/component capacity

Congestion control:
    protects shared network capacity
```

They may interact but solve different problems.

## 25. Congestion

Congestion occurs when offered traffic exceeds effective network capacity.

Symptoms can include:

```text
queue growth
latency increase
packet loss
retransmissions
throughput collapse
```

## 26. Congestion Feedback

Transport policy may use:

```text
queue depth
loss
latency
explicit signals
window utilization
rate measurements
```

to adapt sending behavior.

## 27. Retry Storms

Retries can amplify congestion:

```text
packet loss
 ↓
retry
 ↓
more traffic
 ↓
more congestion
 ↓
more loss
```

Retry policies should therefore define:

```text
maximum attempts
backoff
jitter
time budget
retryable errors
```

## 28. Timeout Semantics

Timeouts should identify what has timed out:

```text
connection establishment
message acknowledgement
processing
idle connection
lease renewal
request deadline
```

A generic “network timeout” is insufficient for robust recovery logic.

## 29. Deadlines

A request deadline represents an end-to-end budget:

```text
deadline
  ↓
queue + transport + processing + response
```

Each layer should avoid extending the deadline accidentally.

## 30. Cancellation

Cancellation should propagate through layers:

```text
request cancelled
      ↓
transport cancellation
      ↓
remote operation cancellation where supported
```

The contract must specify whether cancellation is best-effort or authoritative.

## 31. Connection Failure

Connection failure may imply:

```text
transport unavailable
```

but not necessarily:

```text
remote process terminated
```

Part XXV failure detection remains authoritative for distributed membership interpretation.

## 32. Reconnection

Reconnection should establish a new connection identity or generation where appropriate:

```text
Connection C1
   ↓ failure
Connection C2
```

Pending operations must define whether they are retried, failed, resumed, or deduplicated.

## 33. Session Resumption

A protocol may resume a logical session across connections using:

```text
session identity
generation
checkpoint
sequence position
security context
```

Resumption must not resurrect expired authority.

## 34. Multiplexing

Multiple logical streams may share one connection:

```text
Connection
 ├─ Stream A
 ├─ Stream B
 └─ Stream C
```

The contract must define whether failure of one stream affects the others.

## 35. Head-of-Line Blocking

A multiplexed transport should consider whether one blocked stream can delay unrelated streams.

This is both a latency and resource-management concern.

## 36. Routing

Routing selects a path between endpoints:

```text
A → R1 → R2 → B
```

Routing semantics should distinguish:

```text
path selection
service discovery
load balancing
failover
policy routing
```

## 37. Topology

NROS may operate over:

```text
point-to-point
star
mesh
hierarchical
clustered
multi-region
```

Topology affects latency, failure domains, quorum behavior, and recovery.

## 38. Failure Domains

A topology should identify correlated failure domains:

```text
process
host
rack
zone
region
network segment
provider
```

A replication strategy that places all replicas in one failure domain may provide less availability than its replica count suggests.

## 39. Service Discovery

Discovery provides endpoint information:

```text
service identity
endpoint
protocol
version
health/status
capabilities
```

Discovery data must have freshness and trust semantics.

## 40. Stale Discovery

A discovered endpoint may become invalid after discovery.

Therefore:

```text
discovery result
    ≠
current availability
```

Clients need bounded caching and failure handling.

## 41. Load Balancing

Load balancing may select endpoints using:

```text
round robin
weighted selection
least load
latency
capacity
locality
health
```

The policy should account for stale health information.

## 42. Network Security

Part XXII security applies to transport:

```text
peer authentication
confidentiality
message integrity
replay protection
credential rotation
endpoint authorization
```

Transport encryption does not automatically authorize application operations.

## 43. Transport Metadata

Messages may carry metadata such as:

```text
protocol version
request identity
trace identity
deadline
priority
schema version
compression
capability information
```

Metadata must have explicit trust and validation semantics.

## 44. Priority

Priority can affect scheduling and resource allocation.

It must therefore define:

```text
priority range
ordering semantics
starvation protection
authorization
resource limits
```

Unbounded high-priority traffic can become a denial-of-service vector.

## 45. Resource Economics

Part XXI applies directly to network resources:

```text
connections
buffers
queues
bandwidth
CPU for parsing
memory for reassembly
retry budgets
```

Network admission should therefore be resource-aware.

## 46. Observability

Part XIV should make transport state observable through stable facts such as:

```text
connection identity
peer identity
protocol version
bytes sent/received
messages sent/received
retries
reordering
queue depth
backpressure
timeouts
reconnections
```

Metrics must not be mistaken for causal explanations without supporting evidence.

## 47. Deterministic Network Testing

Part XXIV can support controlled network simulation:

```text
fixed latency
controlled jitter
packet loss
reordering
duplication
bandwidth limits
partitions
connection resets
```

This permits repeatable testing of transport invariants.

## 48. Formal Transport Properties

Part XIX may specify properties such as:

```text
accepted_message
    ⇒
message is either durably tracked,
processed, or explicitly failed
```

under the precise transport contract.

For idempotent operations:

```text
Apply(M, Apply(M,S))
    ≡
Apply(M,S)
```

where semantic idempotence is required.

## 49. Verification Matrix

| Property | Verification question |
|---|---|
| Framing | Are message boundaries unambiguous? |
| Size | Are message and buffer limits explicit? |
| Delivery | Is delivery semantics defined? |
| ACK | Is acknowledgement meaning explicit? |
| Ordering | What ordering guarantee exists? |
| Retry | Are retry limits and backoff defined? |
| Flow control | Can receivers protect capacity? |
| Backpressure | Can overload propagate upstream? |
| Congestion | Can network overload be controlled? |
| Timeout | Is each timeout semantically identified? |
| Cancellation | Is cancellation behavior defined? |
| Reconnect | Are connection generations handled safely? |
| Session | Can sessions resume without stale authority? |
| Routing | Is path selection explicit? |
| Topology | Are failure domains understood? |
| Discovery | Is discovery freshness bounded? |
| Security | Are peers authenticated and authorized? |
| Resources | Are buffers/connections/retries bounded? |
| Testing | Can network faults be reproduced? |
| Formal assurance | Are transport invariants explicit? |

## 50. What Part XXVI Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production transport stack;
- universal exactly-once delivery;
- global ordering;
- congestion control across every transport;
- complete service discovery;
- topology-aware routing;
- deterministic network behavior;
- complete network fault simulation;
- formally verified transport protocols.

Those require implementation-specific evidence.

## 51. Transition to Part XXVII

Part XXVI defines networking and transport semantics.

Part XXVII should define **storage architecture, persistence protocols, durability, replication at rest, crash consistency, recovery journals, snapshots, compaction, and durable state lifecycle**, connecting transport and distributed coordination with persistent system state.

```text
Part XXV
Distributed coordination + consensus + membership
        ↓
Part XXVI
Networking + transport + congestion + topology
        ↓
Part XXVII
Persistence + durability + crash consistency + storage lifecycle
```

## Canonical rule

> **NROS treats communication guarantees as explicit contracts: delivery, ordering, reliability, flow control, backpressure, congestion, routing, and connection lifecycle must be independently defined and remain bounded under loss, delay, duplication, reordering, overload, disconnection, and topology change.**
