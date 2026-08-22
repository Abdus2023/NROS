# Part V — Communication & Transport Contract

> **Series:** NROS Architecture Series  
> **Part:** V  
> **Role:** Communication semantics and transport abstraction  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part IV defined lifecycle and entity state. Part V defines how NROS entities exchange information and requests while keeping the logical communication contract independent from the mechanism used to transport data.

The central rule is:

> **Communication semantics belong to the contract; transport mechanisms are replaceable implementations beneath that contract.**

## 2. Communication Model

NROS communication can be modeled as four logical classes:

```text
Topic / Stream
    → continuous typed data

Service
    → bounded request / response

Action
    → long-running goal / feedback / result

Event
    → occurrence / state transition notification
```

These mechanisms may share infrastructure, but their semantics must remain distinguishable.

## 3. Message Contract

A message is a typed value exchanged between runtime entities.

Conceptually:

```text
Message<T>
├── Type identity
├── Schema/version
├── Payload
├── Source
├── Timestamp
├── Correlation metadata
└── Optional sequence metadata
```

Not every deployment requires every metadata field, but any field required for correctness must be explicitly defined.

## 4. Channel

A channel is the logical contract through which compatible entities communicate.

```text
Channel<T>
├── Type
├── Capacity
├── Ordering
├── Delivery semantics
├── Reliability
├── Ownership
├── Backpressure
├── QoS
└── Transport binding
```

A channel is not itself a queue, socket, shared-memory region, or DDS topic.

Those are possible implementations.

## 5. Publisher / Subscriber

The basic stream pattern is:

```text
Publisher<T>
      │
      ▼
 Channel<T>
      │
      ▼
Subscriber<T>
```

Multiple publishers and subscribers may participate if the channel contract permits them.

The architecture must define whether ordering is:

```text
per publisher
per subscriber
per channel
or unspecified
```

An unspecified ordering must never be interpreted as deterministic ordering.

## 6. Ownership

Communication must define what happens to message ownership.

Possible models include:

```text
Move
Clone
Shared ownership
Borrowed view
Serialized representation
Shared-memory reference
```

The logical contract should not claim zero-copy merely because a particular local implementation avoids one copy.

```text
Local zero-copy
      ≠
End-to-end zero-copy
```

## 7. Serialization

Serialization is a transport concern unless the message contract explicitly requires a canonical wire representation.

```text
Typed message
     │
     ├── in-process → native representation
     │
     ├── IPC → platform representation
     │
     └── network → wire representation
```

The architecture should define when serialization is required and what compatibility guarantees the wire representation provides.

## 8. Schema and Versioning

Messages need stable type identity and controlled evolution.

Conceptually:

```text
TypeId
SchemaVersion
CompatibilityPolicy
```

Possible compatibility policies:

```text
Exact
BackwardCompatible
ForwardCompatible
BidirectionallyCompatible
Incompatible
```

Schema compatibility must be tested rather than inferred from field names alone.

## 9. Delivery Semantics

NROS must distinguish delivery guarantees.

Possible semantics include:

```text
BestEffort
AtMostOnce
AtLeastOnce
ExactlyOnce
```

These terms require precise definitions.

For example, `AtLeastOnce` can imply duplicate delivery and therefore requires consumers to tolerate or deduplicate duplicates where necessary.

`ExactlyOnce` is a particularly strong distributed-systems claim and must not be used merely because an API returns one result per request.

## 10. Ordering

Ordering is an independent dimension from reliability.

Possible ordering scopes include:

```text
None
PerPublisher
PerChannel
PerKey
Global
```

A system can provide reliable delivery without providing global ordering.

Likewise, ordering metadata does not establish that messages are delivered without loss.

## 11. Backpressure

A producer can outpace a consumer.

NROS therefore requires an explicit policy:

```text
Block
DropNewest
DropOldest
LatestOnly
Reject
Buffer
Throttle
```

The policy should be selected according to the workload.

For sensor streams, `LatestOnly` may be appropriate. For command or transactional data, dropping messages may be unacceptable.

## 12. Capacity

Channel capacity is part of the runtime contract when bounded behavior matters.

```text
capacity = N
```

The architecture should define what occurs at capacity:

```text
producer blocks
producer fails
oldest item removed
newest item removed
consumer notified
```

Unbounded buffering must not be treated as a default real-time solution.

## 13. QoS

Quality-of-service is the collection of communication constraints negotiated or configured for a channel.

Conceptually:

```text
QoS
├── Reliability
├── Durability
├── Ordering
├── History
├── Deadline
├── Lifespan
├── Liveliness
└── Resource limits
```

Not every transport can implement every QoS property.

Therefore the runtime should expose capability negotiation or rejection rather than silently weakening a requested guarantee.

## 14. Local Transport

For components in the same execution domain, communication may use:

```text
Direct call
Queue
Ring buffer
Shared memory
Lock-free structure
```

The transport must preserve the logical channel contract.

A faster local mechanism is not automatically semantically equivalent if it changes ordering, ownership, delivery, or backpressure behavior.

## 15. Inter-Process Transport

Across process boundaries, NROS may use:

```text
Unix domain sockets
Shared memory
OS IPC
Other platform IPC
```

The core communication contract should remain independent of which mechanism is selected.

## 16. Network Transport

Across machines, transport may use:

```text
UDP
TCP
QUIC
DDS/RTPS
Custom protocol
```

NROS should not make the architectural communication model identical to any one transport protocol.

The adapter is responsible for translating the contract into the chosen wire mechanism.

## 17. Transport Binding

A channel can conceptually bind to a transport:

```text
Channel<T>
     │
     ▼
TransportBinding
     │
 ┌───┼───────────┐
 ▼   ▼           ▼
IPC SHM        Network
```

The binding may depend on:

- deployment topology;
- latency requirements;
- payload size;
- reliability requirements;
- security policy;
- hardware capabilities;
- available platform services.

## 18. Transport Capability

A transport should advertise what it can actually provide.

```text
TransportCapabilities
├── max_message_size
├── ordering
├── reliability
├── durability
├── zero_copy
├── encryption
├── multicast
└── bounded_latency
```

A requested channel contract should be rejected or adapted explicitly if the transport cannot satisfy it.

```text
Requested guarantee
        ↓
Transport capability check
        ↓
Satisfied / Rejected / Explicitly degraded
```

Silent degradation is prohibited for guarantees that affect correctness or safety.

## 19. Connection Lifecycle

Transport connections have their own state and must not be confused with entity lifecycle.

Conceptually:

```text
DISCONNECTED
    ↓
CONNECTING
    ↓
CONNECTED
    ↓
DEGRADED
    ↓
DISCONNECTING
    ↓
DISCONNECTED
```

Connection failure may coexist with a healthy component:

```text
Component = RUNNING
Transport  = DEGRADED
```

This is another reason state dimensions must remain separate.

## 20. Reconnection

A transport may reconnect after failure.

Reconnection must define:

```text
session identity
sequence behavior
duplicate handling
queued data
ordering reset
schema renegotiation
security reauthentication
```

Reconnection does not automatically imply that previously queued data remains valid.

## 21. Request / Response

Services use a correlated request/response model:

```text
Request
├── request_id
├── source
├── deadline
├── timestamp
└── payload

Response
├── request_id
├── status
├── timestamp
└── payload / error
```

Timeouts and cancellation must have explicit semantics.

```text
Request timeout
      ≠
Server stopped
```

A client timeout only establishes that the client stopped waiting according to its policy unless the protocol provides stronger evidence.

## 22. Actions

Actions represent long-running work:

```text
Goal
  ↓
Admission
  ↓
Execution
  ├── Feedback*
  ├── Cancellation
  └── Checkpoint*
  ↓
Result
```

The action protocol must distinguish:

```text
accepted
executing
cancel requested
cancelled
succeeded
failed
```

## 23. Events

Events communicate occurrences without necessarily establishing a request/response relationship.

Examples:

```text
LifecycleChanged
ResourceAvailable
FaultDetected
ConfigurationChanged
GoalCompleted
```

Events should carry enough identity and ordering information for their intended use.

## 24. Security Boundary

Communication security may include:

```text
Authentication
Authorization
Confidentiality
Integrity
Replay protection
Peer identity
```

Security properties belong to the communication contract where they affect correctness or authority.

Encryption alone does not establish authorization.

```text
Encrypted
      ≠
Authenticated
      ≠
Authorized
```

## 25. Failure Semantics

Communication failures should be classified explicitly:

```text
SerializationFailure
ConnectionFailure
Timeout
BackpressureFailure
SchemaMismatch
AuthenticationFailure
AuthorizationFailure
TransportFailure
PeerFailure
```

The runtime should expose enough information for policy and recovery layers to choose appropriate actions.

## 26. Observability

Communication should expose observations such as:

```text
message_sent
message_received
message_dropped
queue_full
delivery_failed
schema_rejected
connection_changed
request_timeout
```

Instrumentation must preserve the distinction between observed events and inferred guarantees.

## 27. Verification Matrix

| Property | Verification question |
|---|---|
| Type safety | Are incompatible message types rejected? |
| Schema | Are incompatible versions detected? |
| Ordering | Does the implementation obey its declared ordering scope? |
| Delivery | Are duplicate/loss semantics consistent with the contract? |
| Capacity | Is bounded capacity actually enforced? |
| Backpressure | Does the configured policy occur under saturation? |
| Deadline | Is deadline behavior observable and correct? |
| Reconnection | Are stale/duplicate messages handled correctly? |
| Ownership | Are ownership/lifetime rules preserved? |
| Transport binding | Are unsupported guarantees rejected rather than silently weakened? |
| Security | Are unauthorized communications rejected? |
| Observability | Can failures and delivery outcomes be identified? |

## 28. What Part V Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- exactly-once distributed delivery;
- global message ordering;
- universal zero-copy transport;
- bounded network latency;
- complete QoS negotiation;
- transparent transport failover;
- production-grade authenticated networking;
- deterministic distributed communication.

Those properties require implementation-specific evidence.

## 29. Transition to Part VI

Part V defines the communication contract.

Part VI should address the next major runtime dimension:

> **How does NROS represent and control time, deadlines, clocks, temporal ordering, and deterministic/replay-oriented execution?**

```text
Part IV
Lifecycle + entity state
        ↓
Part V
Communication + transport
        ↓
Part VI
Time + temporal semantics
```

## Canonical rule

> **NROS communication is defined by typed, explicit delivery, ordering, ownership, capacity, and failure semantics; transport mechanisms are replaceable implementations and must never silently weaken required guarantees.**
