# Part LVI — Messaging, Eventing, Delivery, Ordering & Replay

> **Series:** NROS Architecture Series  
> **Part:** LVI  
> **Role:** Commands, messages, queues, events, routing, delivery semantics, acknowledgement, ordering, correlation, replay, backpressure, dead letters, and cross-domain communication  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LV established durable state. Part LVI defines how NROS communicates commands, observations, events, and work across process, node, and distributed-system boundaries.

The central rule is:

> **A message is transportable information; an event is a record of an observed fact; a command is an instruction; delivery is a separate property from processing and completion.**

## 2. Fundamental Distinctions

```text
Command
  ≠
Message
  ≠
Event
  ≠
Acknowledgement
  ≠
Processing Result
```

## 3. Command

A command requests an action.

```text
Command
 → receiver
 → authorization
 → execution
```

A command does not prove that the requested action succeeded.

## 4. Event

An event represents an observed or committed fact:

```text
Something Happened
```

Events should not be used to imply an action merely because it was requested.

## 5. Message Envelope

A message envelope may contain:

```text
message_id
message_type
schema_version
producer
recipient / routing key
correlation_id
causation_id
sequence
created_at
payload
integrity metadata
```

## 6. Message Identity

Each message should have a stable unique identity within its delivery domain.

## 7. Correlation

Correlation links related operations:

```text
Request
 ↓ correlation_id
Command
 ↓ correlation_id
Result / Event
```

## 8. Causation

Causation identifies the message or event that directly caused another message or event.

```text
Event B
caused_by
Event A
```

## 9. Correlation vs Causation

```text
correlation
 → same logical operation

causation
 → direct causal predecessor
```

They must not be conflated.

## 10. Routing

Routing may use:

```text
recipient
subject
queue
partition key
topic
capability
policy
```

## 11. Routing Authority

Routing decisions must respect authorization and communication policy.

## 12. Queue

A queue provides a bounded or policy-controlled holding area between production and consumption.

```text
Producer
 ↓
Queue
 ↓
Consumer
```

## 13. Queue Capacity

Queue capacity may be constrained by:

```text
message count
bytes
age
priority
resource budget
```

## 14. Backpressure

When consumers cannot keep up:

```text
Demand
 ↓
Queue Pressure
 ↓
Backpressure Policy
```

Possible actions:

```text
slow producers
reject
shed
buffer
prioritize
scale consumers
```

## 15. Unbounded Queues

Unbounded queues must not be treated as harmless buffering; they convert message pressure into unbounded state consumption.

## 16. Delivery Semantics

NROS should explicitly declare one of:

```text
at-most-once
at-least-once
exactly-once where end-to-end evidence exists
```

## 17. At-Most-Once

A message is delivered zero or one time.

Loss is possible.

## 18. At-Least-Once

A message may be delivered multiple times.

Consumers therefore require duplicate-safe processing where duplication is possible.

## 19. Exactly-Once

Exactly-once delivery or effect must not be inferred from a transport acknowledgement alone.

It requires an end-to-end contract covering:

```text
identity
transport
processing
state mutation
side effects
acknowledgement
recovery
```

## 20. Acknowledgement

An acknowledgement means only what its protocol defines.

```text
received
 ≠
processed
 ≠
committed
 ≠
effect completed
```

## 21. Processing Acknowledgement

A consumer may acknowledge after successful processing, after durable commit, or at another explicitly defined point.

## 22. Negative Acknowledgement

A consumer may indicate that a message should be retried, delayed, or dead-lettered.

## 23. Retry

Message retry requires bounded policy:

```text
attempt limit
backoff
jitter
failure classification
retry budget
```

## 24. Duplicate Delivery

Consumers should use:

```text
message_id
idempotency key
transaction identity
processed-message state
```

where duplicate processing could be harmful.

## 25. Deduplication

Deduplication state is itself durable state and inherits the guarantees of Part LV.

## 26. Ordering

Ordering must be scoped.

Possible scopes:

```text
per message key
per partition
per queue
per producer
per workload
```

## 27. Global Ordering

Global ordering is expensive and should not be implied unless explicitly provided.

## 28. Sequence Numbers

A stream may use sequence numbers:

```text
100
101
102
```

Gaps can indicate loss, filtering, or partition changes and require explicit interpretation.

## 29. Ordering vs Causality

```text
sequence order
 ≠
causal order
```

A message sequence can be ordered without representing all distributed causality.

## 30. Causal Metadata

Distributed workflows may use:

```text
causation_id
correlation_id
logical clocks
vector metadata
```

according to required guarantees.

## 31. Partitioning

Partitions distribute message processing while preserving ordering within the declared partition scope.

## 32. Partition Key

Partition keys should be selected according to the entity whose ordering or locality matters.

Examples:

```text
workload_id
resource_id
tenant_id
stream_id
```

## 33. Consumer Groups

A consumer group distributes work among consumers.

The group must define ownership and reassignment semantics.

## 34. Consumer Lease

A consumer may hold a lease for a partition or queue assignment.

Lease expiry must prevent stale consumers from continuing authoritative processing.

## 35. Consumer Fencing

```text
Consumer epoch 3
Current epoch 4
      ↓
Reject protected acknowledgement / mutation
```

## 36. Visibility Timeout

Queue systems may temporarily hide a message while it is being processed.

Expiration can cause redelivery.

## 37. Poison Messages

A message that repeatedly fails must not create an infinite retry loop.

## 38. Dead-Letter Queue

Repeatedly failing messages may move to a dead-letter destination:

```text
Queue
 ↓ repeated failure
Dead Letter
 ↓
Inspection / Repair / Replay
```

## 39. Dead-Letter Semantics

Dead-lettering is not equivalent to successful processing.

## 40. Delayed Delivery

Retry and scheduling may use delayed delivery.

Delay policy should be observable and bounded.

## 41. Priority

Priority queues should define starvation-prevention semantics where required.

Priority does not override authorization or resource limits.

## 42. Message Expiration

Messages may have a TTL or deadline.

Expired messages require explicit disposition:

```text
reject
archive
dead-letter
drop
```

## 43. Message Size

Message size should be bounded.

Large payloads may use external artifacts with references in the message.

## 44. Payload vs Reference

```text
Message
 ├── metadata
 └── payload reference
        ↓
     Artifact Store
```

The reference must have explicit integrity and access semantics.

## 45. Serialization

Message schemas must be versioned.

```text
schema_id
schema_version
encoding
```

## 46. Schema Evolution

Consumers should explicitly define compatibility expectations:

```text
backward compatible
forward compatible
both
neither
```

## 47. Unknown Fields

Where forward compatibility is required, consumers should safely handle unknown fields rather than silently corrupting state.

## 48. Invalid Messages

Malformed or incompatible messages must not be processed as valid domain operations.

## 49. Authentication

Message origin should be authenticated where required by the trust model.

## 50. Authorization

Authentication does not grant permission to publish, route, consume, or execute commands.

## 51. Confidentiality

Sensitive message payloads may require encryption in transit and at rest.

## 52. Integrity

Messages may require integrity protection to detect modification or corruption.

## 53. Replay Protection

Security-sensitive commands may require protection against malicious or accidental replay.

Mechanisms can include:

```text
nonce
sequence
expiry
idempotency key
state revision
```

## 54. Event Persistence

Events may be transient or durable.

A durable event requires the persistence guarantees defined by Part LV.

## 55. Event Log

An event log may provide:

```text
append-only sequence
replay
historical audit
consumer recovery
```

## 56. Event Sourcing

Where event sourcing is used, the event stream becomes the authoritative state reconstruction source.

This is a deliberate architectural choice, not a default property of event-driven systems.

## 57. State + Events

An alternative is:

```text
Authoritative State
      +
Change Events
```

The architecture must define which is authoritative.

## 58. Transactional Publication

When a state mutation and event publication must correspond, NROS should use an explicit mechanism such as:

```text
transactional outbox
integrated transaction log
atomic event/state store
```

## 59. Outbox

The outbox pattern can provide:

```text
State Transaction
 ↓
Outbox Record
 ↓
Publisher
 ↓
Message Broker
```

The outbox itself is durable state.

## 60. Inbox

An inbox records received messages when durable duplicate suppression or processing recovery is required.

## 61. Inbox + Outbox

Together:

```text
Inbox
 ↓
Process
 ↓
State + Outbox Transaction
 ↓
Publish
```

can establish robust at-least-once processing patterns.

## 62. Replay

Replay reprocesses historical messages or events.

Replay must define:

```text
range
consumer version
side-effect policy
ordering
idempotency
```

## 63. Replay Is Not Time Travel

Replaying an event does not automatically recreate the exact historical external environment.

## 64. Replay Safety

A replay may require:

```text
read-only mode
new idempotency namespace
side-effect suppression
sandbox
compensation policy
```

## 65. Reprocessing

Reprocessing should identify the consumer/software revision used to interpret the message.

## 66. Consumer Versioning

Different consumer versions may interpret the same event differently.

Compatibility must therefore be explicit.

## 67. Event Retention

Retention policy should define:

```text
minimum retention
maximum retention
storage budget
compliance requirements
replay requirements
```

## 68. Compaction

Compacted streams may retain the latest state per key while discarding intermediate events.

Compaction is safe only when the consumer contract permits it.

## 69. Eventual Delivery

Asynchronous publication does not guarantee immediate observation.

Consumers must distinguish:

```text
not observed yet
not published
published but delayed
lost
```

where the protocol allows these states to be distinguished.

## 70. Delivery Failure

Failures may occur at:

```text
routing
transport
queue
consumer
processing
commit
acknowledgement
```

## 71. Acknowledgement Loss

If processing succeeded but the acknowledgement was lost, redelivery may occur.

This is a primary reason for idempotent consumers.

## 72. Transaction Boundary

A message processing transaction should define whether it covers:

```text
message acknowledgement
state mutation
outbox publication
external side effect
```

Global atomicity must not be implied accidentally.

## 73. External Side Effects

Message processing that calls external systems must account for uncertain outcomes.

```text
Send Request
 ↓
Response Lost
 ↓
Unknown External Outcome
```

## 74. Idempotent External Commands

Where possible, external commands should carry an idempotency identity derived from the logical operation.

## 75. Ordering Across Retries

Redelivery must not accidentally violate ordering guarantees that the consumer depends upon.

## 76. Concurrent Consumers

Parallel consumers require explicit concurrency semantics.

Two messages for the same ordering key must not execute concurrently when the contract forbids it.

## 77. Workload Integration

Messages can create or control workloads:

```text
Command Message
 ↓
Admission
 ↓
Workload
```

The message itself does not bypass workload policy.

## 78. Resource Integration

Message consumers consume CPU, memory, storage, and network resources and therefore participate in Part LIII resource policy.

## 79. Backpressure Integration

Backpressure may propagate:

```text
Consumer Pressure
 ↓
Queue Pressure
 ↓
Producer Throttling
 ↓
Admission Pressure
```

## 80. Priority Integration

Message priority must remain compatible with workload and resource priority rules.

## 81. Event Integration With Observability

Lifecycle events can feed the observability plane:

```text
Execution Event
 ↓
Event Stream
 ↓
Metrics / Audit / Trace
```

An event used for observability does not automatically become an authoritative state mutation.

## 82. Trace Context

Messages may carry distributed trace context:

```text
trace_id
span_id
parent context
```

Trace metadata should not be confused with security identity.

## 83. Audit Events

Security-sensitive actions may generate audit events with stronger retention and integrity requirements than ordinary operational events.

## 84. Event Ordering During Recovery

Consumers recovering from failure should resume from a known revision, offset, or checkpoint rather than assuming current position.

## 85. Offset Commit

Consumer offsets are state and should follow explicit durability semantics.

```text
Processed Message N
 ↓
Commit Offset N
```

The relationship between processing and offset commit determines redelivery behavior.

## 86. Offset + State Atomicity

When required, state mutation and offset advancement should share a transaction or equivalent recovery protocol.

## 87. Lost Consumer State

If consumer progress is lost, replay should be safe under the declared processing semantics.

## 88. Queue Recovery

After broker or queue failure, recovery must preserve the declared guarantees for:

```text
message durability
ordering
acknowledgement
redelivery
expiration
```

## 89. Queue Ownership

A queue should have explicit ownership and authorization boundaries.

## 90. Cross-Domain Messaging

Communication across trust or authority domains requires explicit:

```text
identity
authorization
schema
routing
failure semantics
```

## 91. Bridge

A bridge translating between messaging systems must preserve or explicitly weaken guarantees.

It must not silently claim stronger semantics than either side supports.

## 92. Gateway

A gateway may transform:

```text
protocol
schema
identity
routing
```

Each transformation should be observable where it affects semantics.

## 93. Message Translation

Translation must preserve semantic meaning, not merely field names.

## 94. Ordering Across Bridges

A bridge may break ordering guarantees unless it preserves the required ordering key and sequence semantics.

## 95. Delivery Contract

Every production messaging interface should document:

```text
transport
schema
ordering
delivery
acknowledgement
retry
retention
security
```

## 96. Formal Delivery Invariant

```text
AtLeastOnce(M)
    ⇒
MayDeliverMultipleTimes(M)
```

## 97. Formal Ack Invariant

```text
Ack(M)
    ⇒
MeaningDefinedByProtocol
```

and not automatically:

```text
Ack(M) ⇒ SideEffectCompleted
```

## 98. Formal Ordering Invariant

```text
Ordered(K)
    ⇒
OrderingScopeExplicit(K)
```

## 99. Formal Replay Invariant

```text
Replay(M)
    ⇒
ReplayPolicyAllows(M)
 ∧
SideEffectPolicyDefined(M)
```

## 100. Formal Consumer Invariant

```text
Process(M)
    ∧
PossibleRedelivery(M)
    ⇒
DuplicateSafetyDefined(M)
```

## 101. Verification Matrix

| Property | Verification question |
|---|---|
| Identity | Does every message have stable identity? |
| Semantics | Are command, event, acknowledgement, and result distinct? |
| Routing | Is routing policy explicit? |
| Delivery | Is at-most/at-least/exactly-once behavior declared? |
| Ack | Is acknowledgement meaning unambiguous? |
| Ordering | Is ordering scope explicit? |
| Causality | Can causal relationships be reconstructed where required? |
| Retry | Are retries bounded? |
| Deduplication | Can duplicate delivery be handled safely? |
| Backpressure | Is queue pressure bounded? |
| Poison messages | Can infinite retry loops be prevented? |
| Dead letters | Is failed-message disposition explicit? |
| Schema | Are message schemas versioned? |
| Security | Are origin and authorization protected? |
| Replay | Is replay behavior safe and defined? |
| Offsets | Is consumer progress durable enough for the contract? |
| State | Can processing and state mutation be reconciled? |
| External effects | Are ambiguous external outcomes represented? |
| Bridges | Are weakened guarantees explicit? |
| Retention | Are event/message retention rules defined? |
| Recovery | Can consumers resume without silent loss? |
| Evidence | Can delivery and processing claims be verified? |

## 102. What Part LVI Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production message broker;
- universal exactly-once delivery;
- universal exactly-once side effects;
- complete durable event sourcing;
- production-grade cross-domain messaging bridges;
- universal global ordering;
- complete replay-safe implementations for every consumer;
- complete broker disaster recovery;
- automatic semantic schema translation.

Those require implementation-specific evidence.

## 103. Transition to Part LVII

Part LVI establishes the communication and eventing plane.

Part LVII should define **networking, transport, sessions, connections, addressing, discovery, protocol negotiation, reliability, and network failure semantics**.

```text
Part LV
State + storage + durability + recovery
        ↓
Part LVI
Messaging + events + delivery + replay
        ↓
Part LVII
Networking + transport + sessions + discovery
```

## Canonical rule

> **NROS treats commands, messages, events, acknowledgements, and processing results as distinct semantic objects; delivery, ordering, processing, durability, replay, and side effects require explicit contracts rather than assumptions derived from transport behavior.**
