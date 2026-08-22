# Part LXVI — Messaging, Delivery, Ordering, Acknowledgement, Retry & Deduplication

> **Series:** NROS Architecture Series  
> **Part:** LXVI  
> **Role:** Message identity, transport, queues, delivery guarantees, acknowledgement, retry, ordering, deduplication, replay, backpressure, and distributed messaging authority  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LXV established scheduling and dispatch. Part LXVI defines how work and state cross NROS boundaries through messages and how the system reasons about delivery, acknowledgement, ordering, retry, duplication, replay, and failure.

The central rule is:

> **NROS must treat message delivery as an explicit protocol contract; transport success, message acceptance, processing completion, and durable outcome are distinct states and must never be silently conflated.**

## 2. Message Model

```text
Producer
 ↓
Message
 ↓
Transport
 ↓
Delivery
 ↓
Acceptance
 ↓
Processing
 ↓
Completion
```

## 3. Message Identity

Every message requiring deduplication, acknowledgement, replay, or reconciliation should have an explicit identity.

```text
message_id
producer_id
stream_id
sequence
attempt
```

## 4. Message Identity vs Payload

```text
Message Identity
    ≠
Payload Equality
```

Two identical payloads may represent distinct messages.

## 5. Message Envelope

A message envelope may carry:

```text
message_id
producer
source
schema/version
sequence
causality
priority
deadline
trace context
payload
```

## 6. Transport

Transport moves message bytes or structured frames between endpoints.

Transport success does not imply application acceptance.

## 7. Delivery

Delivery means the receiving boundary obtained the message according to the transport contract.

## 8. Acceptance

Acceptance means the receiving subsystem accepted responsibility for the message under its application contract.

```text
Delivered
    ≠
Accepted
```

## 9. Processing

Processing represents application work performed because of an accepted message.

## 10. Completion

Completion represents the declared processing outcome.

```text
Accepted
 ↓
Processing
 ↓
Completed
```

## 11. Acknowledgement

An acknowledgement is an explicit protocol signal describing a message state.

Its meaning must be defined by the protocol.

## 12. Ack Semantics

Possible acknowledgement meanings include:

```text
received
accepted
persisted
processed
completed
rejected
```

An `ack` without semantic definition is insufficient.

## 13. Ack Durability

A receiver should distinguish an acknowledgement based on volatile state from one based on durable state when failure recovery depends on that distinction.

## 14. At-Most-Once

At-most-once delivery attempts to avoid duplicate delivery at the cost of possible message loss.

```text
DeliveryCount ≤ 1
```

within the declared protocol scope.

## 15. At-Least-Once

At-least-once delivery permits duplicates in exchange for reducing loss after uncertain failures.

```text
DeliveryCount ≥ 1
```

when delivery is eventually successful under the contract.

## 16. Exactly-Once

Exactly-once is not a universal transport property.

A system may achieve effectively-once observable effects only when message identity, durable state transitions, idempotency, and transactional boundaries provide the required guarantees.

## 17. Exactly-Once Boundary

Claims of exactly-once must identify the exact scope:

```text
transport
queue
consumer
state transition
external side effect
```

## 18. Duplicate Delivery

Duplicate delivery is expected under at-least-once protocols and should be treated as a protocol condition rather than an exceptional surprise.

## 19. Deduplication

Deduplication suppresses repeated processing of a message identity.

```text
Seen(message_id)
    ⇒
Duplicate
```

subject to the retention scope.

## 20. Deduplication Store

A deduplication record must have an explicit lifetime and durability model.

## 21. Deduplication Window

A bounded deduplication window cannot guarantee suppression of duplicates arriving after expiration.

The protocol must account for this limitation.

## 22. Idempotency

Idempotency means repeated execution produces an equivalent declared outcome.

```text
Apply(M)
≈
Apply(M) + Apply(M)
```

under the operation's idempotency contract.

## 23. Deduplication vs Idempotency

```text
Deduplication
    ≠
Idempotency
```

Deduplication prevents repeated processing; idempotency makes repetition safe.

## 24. Idempotency Key

External side-effecting operations may require an explicit idempotency key linked to message identity.

## 25. Retry

Retry reattempts delivery or processing after a failure or uncertain outcome.

## 26. Retry Classification

Retry policy should distinguish:

```text
transport failure
transient rejection
permanent rejection
processing failure
unknown outcome
expired deadline
```

## 27. Retry After Unknown Outcome

If processing may have completed remotely, retrying a non-idempotent operation can create duplicate side effects.

Reconciliation or idempotency is required.

## 28. Retry Budget

Retries should have explicit limits:

```text
attempt limit
elapsed-time budget
backoff budget
resource budget
```

## 29. Backoff

Retry backoff should be bounded and policy-controlled.

## 30. Jitter

Jitter can reduce synchronized retry waves against shared dependencies.

## 31. Retry Storm

A retry storm occurs when many failures trigger correlated retries that amplify load.

Schedulers and transports should expose mechanisms to prevent it.

## 32. Deadline-Aware Retry

Retries must respect the originating deadline.

```text
DeadlineExpired
    ⇒
NoRetry
```

unless explicit re-admission establishes a new contract.

## 33. Ordering

Ordering defines the sequence in which messages become observable or processed within a declared scope.

## 34. Ordering Scope

Ordering may be:

```text
per-message-key
per-stream
per-partition
per-producer
per-queue
global
```

Global ordering must not be assumed when only local ordering is guaranteed.

## 35. Sequence Numbers

Sequence numbers provide explicit order within a declared producer or stream scope.

## 36. Gaps

A missing sequence number may indicate delay, loss, partitioning, or invalid producer behavior.

The consumer policy must define how gaps are handled.

## 37. Out-of-Order Delivery

Consumers should distinguish out-of-order delivery from duplication.

## 38. Reordering Buffer

A consumer may buffer messages to restore declared ordering, subject to bounded memory and deadline constraints.

## 39. Ordering vs Timestamp

```text
Timestamp
    ≠
Sequence Order
```

unless the protocol explicitly establishes that relationship.

## 40. Causality

Causality should be represented through explicit protocol relationships such as:

```text
parent_message_id
sequence
correlation_id
epoch
logical clock
```

## 41. Correlation

Correlation identifies messages participating in the same logical operation without implying ordering.

## 42. Request / Response

A response should identify the request or operation it answers.

## 43. Response Duplication

Consumers should tolerate duplicate responses where transport or retry semantics permit them.

## 44. Cancellation

Cancellation messages require explicit semantics regarding already-delivered or already-processing work.

## 45. Cancellation Race

```text
Execute
   ×
Cancel
```

must resolve under an explicit protocol rule.

## 46. Poison Message

A poison message repeatedly fails processing and should not cause infinite retry loops.

## 47. Poison Handling

Possible policies include:

```text
quarantine
dead-letter
reject
alert
manual recovery
```

## 48. Dead-Letter Queue

Dead-letter storage should preserve enough metadata to diagnose and safely replay or discard the message.

## 49. Replay

Replay reintroduces previously delivered messages into processing.

Replay must be explicit because it can intentionally bypass ordinary deduplication assumptions.

## 50. Replay Identity

A replayed message should retain original identity while also carrying replay context where required.

## 51. Replay Safety

Replay should require idempotency, deduplication policy, or a transaction boundary appropriate to the operation.

## 52. Consumer Offset

Stream consumers may track a durable position representing the highest safely acknowledged sequence.

## 53. Offset Semantics

The meaning of an offset must be explicit:

```text
received
accepted
persisted
processed
committed
```

## 54. Commit Position

A committed consumer position should not advance beyond the state whose recovery depends on it.

## 55. Redelivery

When acknowledgement is absent or invalidated, a message may be redelivered according to retry policy.

## 56. Visibility Timeout

Queue systems may hide a message temporarily while it is being processed.

Expiration should return the message to an eligible state unless completion was durably recorded.

## 57. Lease-Based Consumption

Consumer ownership of a message can use a lease and fencing token to prevent stale workers from committing obsolete results.

## 58. Consumer Fencing

```text
ConsumerEpoch(C) ≠ CurrentEpoch(M)
    ⇒
Commit(M) = Forbidden
```

where stale consumer commits could corrupt state.

## 59. Backpressure

Consumers should expose bounded processing capacity to producers or transport layers.

## 60. Queue Capacity

Message queues should define capacity or explicit overflow behavior.

## 61. Overflow

Possible policies include:

```text
block
reject
spill
shed
coalesce
sample
```

## 62. Coalescing

Coalescing may replace multiple messages with an equivalent aggregate when the protocol explicitly permits it.

## 63. Lossy Messaging

Lossy delivery is valid only when message loss is part of the declared contract.

## 64. Critical Messaging

Critical messages should use delivery and durability semantics sufficient for their consequence level.

## 65. Priority Messaging

Priority may influence delivery order but must not bypass authorization or queue isolation.

## 66. Fair Delivery

Shared messaging systems should define fairness expectations where one producer could otherwise monopolize capacity.

## 67. Producer Flow Control

Producers should receive explicit capacity signals where sustained production can exceed consumer capacity.

## 68. Consumer Flow Control

Consumers may limit concurrent processing to preserve resource and deadline guarantees.

## 69. Message Size

Message size should be bounded or governed by explicit resource policy.

## 70. Large Payloads

Large data should generally use a referenced-object model when transporting the entire payload would create unacceptable memory or latency pressure.

## 71. Payload Reference

A message may carry:

```text
object_id
location
integrity metadata
access policy
```

instead of embedding large data.

## 72. Reference Lifetime

Referenced payloads must remain available for the declared message lifetime or provide explicit expiration semantics.

## 73. Integrity

Messages requiring integrity should carry or inherit an integrity mechanism appropriate to the transport and trust model.

## 74. Authentication

Message producers and consumers should be authenticated according to the trust boundary.

## 75. Authorization

A valid message is not automatically an authorized command.

The receiver must evaluate authority for the requested operation.

## 76. Schema Validation

Consumers should validate message schema and version before processing.

## 77. Versioning

Schema evolution must define compatibility semantics for producers and consumers operating across versions.

## 78. Unknown Fields

Consumers should define whether unknown fields are ignored, rejected, or preserved.

## 79. Unknown Message Type

Unknown message types should produce explicit rejection or quarantine behavior rather than undefined execution.

## 80. Message Expiry

Messages may have temporal validity boundaries.

Expired messages should not be processed as current commands unless explicitly re-admitted.

## 81. Freshness

Freshness should use a defined temporal or logical mechanism from Part LXIV.

## 82. Message Cancellation on Expiry

When a message's deadline expires, downstream work should be cancelled or marked expired according to policy.

## 83. Transport Partition

Network partition may produce uncertainty about whether a message was delivered or processed.

The protocol must model unknown outcomes explicitly.

## 84. Unknown Delivery State

```text
Sent
 ↓
Connection Lost
 ↓
Unknown
```

The sender must not infer loss solely from missing acknowledgement.

## 85. Reconciliation

Unknown delivery state may require querying the receiver, durable journal, idempotency store, or operation status.

## 86. Durable Inbox

A receiver may persist accepted message identities before processing to support crash recovery and deduplication.

## 87. Durable Outbox

A producer may persist outgoing messages with associated state transitions to prevent message loss between local commit and transport publication.

## 88. Outbox / Inbox Pattern

Combining durable outbox and inbox state can provide stronger message-to-state consistency when transactions span a single persistence boundary.

## 89. Transaction Boundary

Claims of atomic message processing must identify the transaction boundary.

## 90. External Side Effects

Database transaction atomicity does not automatically make external effects exactly once.

External effects require idempotency, transactional integration, or reconciliation.

## 91. Exactly-Once Effect

An effectively-once effect requires:

```text
stable identity
idempotent or deduplicated effect
recoverable state
bounded replay ambiguity
```

## 92. Ack Loss

An acknowledgement can be lost after processing succeeds.

This is a principal reason at-least-once systems produce duplicates.

## 93. Processing Success + Ack Loss

```text
Process(M) = Success
Ack(M) = Lost
        ↓
Redelivery(M)
```

The consumer must remain safe under this condition.

## 94. Ack Before Processing

Acknowledging before durable acceptance can create message loss after receiver failure.

The protocol must explicitly permit that tradeoff if chosen.

## 95. Ack After Durable Acceptance

Acknowledging after durable acceptance provides stronger crash recovery semantics than acknowledgement based solely on volatile receipt.

## 96. Completion Ack

A completion acknowledgement should only claim completion when the declared completion conditions are satisfied.

## 97. Error Semantics

Message errors should distinguish:

```text
invalid
unauthorized
transient
permanent
expired
duplicate
unknown
```

## 98. Observability

Message traces should allow reconstruction of:

```text
created
sent
delivered
accepted
processed
acknowledged
retried
redelivered
completed
rejected
quarantined
```

## 99. Formal Delivery Invariant

```text
AckProcessed(M)
    ⇒
ProcessingCompletion(M)
```

when the acknowledgement contract means processing completion.

## 100. Formal Deduplication Invariant

```text
Duplicate(M)
    ⇒
NoDuplicateSideEffect(M)
```

when the consumer contract guarantees deduplication or idempotency.

## 101. Verification Matrix

| Property | Verification question |
|---|---|
| Identity | Is every relevant message uniquely identifiable? |
| Delivery | Is delivery distinct from acceptance? |
| Ack | Is acknowledgement meaning explicit? |
| Durability | Is ack durability defined? |
| Ordering | Is ordering scope explicit? |
| Retry | Are retry causes classified? |
| Idempotency | Are repeated effects safe? |
| Deduplication | Is the dedup window sufficient? |
| Replay | Is replay explicit and safe? |
| Offsets | Does committed position match durable state? |
| Backpressure | Is consumer capacity bounded? |
| Poison | Can infinite retry loops be prevented? |
| Security | Are messages authenticated and authorized? |
| Schema | Are versions validated? |
| Expiry | Are expired messages rejected or re-admitted? |
| Partition | Are unknown delivery outcomes modeled? |
| Inbox | Can accepted messages survive receiver restart? |
| Outbox | Can committed messages survive transport failure? |
| External effects | Are side effects idempotent or reconcilable? |
| Evidence | Can message lifecycle be reconstructed? |

## 102. What Part LXVI Does Not Claim

This Part does not claim that the current NROS implementation already has:

- universal exactly-once transport;
- complete durable inbox/outbox infrastructure;
- production-grade distributed messaging;
- universal global ordering;
- complete deduplication across every boundary;
- transactional integration with arbitrary external side effects;
- universal replay safety;
- complete dead-letter management;
- lossless delivery for every message class.

Those require implementation-specific evidence.

## 103. Transition to Part LXVII

Part LXVI establishes message delivery semantics.

Part LXVII should define **distributed state, replication, consistency models, journals, logs, snapshots, consensus boundaries, recovery, and state convergence** using the identity, ordering, temporal, resource, and lifecycle contracts established by the preceding Parts.

```text
Part LXV
Scheduling + queues + priorities + fairness + dispatch
        ↓
Part LXVI
Messaging + delivery + ordering + acknowledgement + deduplication
        ↓
Part LXVII
Distributed state + replication + consistency + convergence
```

## Canonical rule

> **NROS treats message delivery as a stateful protocol: transport, delivery, acceptance, processing, acknowledgement, completion, retry, duplication, replay, and external effects have distinct semantics, and any stronger guarantee must be bounded by an explicit identity, durability, ordering, idempotency, and reconciliation contract.**
