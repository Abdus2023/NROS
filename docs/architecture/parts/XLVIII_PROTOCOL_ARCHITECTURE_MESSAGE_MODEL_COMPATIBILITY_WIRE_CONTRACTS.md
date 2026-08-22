# Part XLVIII — Protocol Architecture, Message Model, Compatibility & Wire Contracts

> **Series:** NROS Architecture Series  
> **Part:** XLVIII  
> **Role:** Protocol architecture, control/data-plane boundaries, message envelopes, command/query/event semantics, versioning, negotiation, compatibility, serialization, validation, and wire contracts  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XLVII established distributed authority. Part XLVIII defines how NROS communicates decisions, requests, observations, state transitions, and failures across process, node, service, and cluster boundaries.

The central rule is:

> **NROS treats every cross-boundary interaction as a protocol contract: identity, authority, intent, schema, version, correlation, delivery semantics, validation, and failure behavior must be explicit.**

## 2. Protocol vs Transport

```text
Protocol
  ≠
Transport
```

A protocol defines meaning and state transitions; a transport provides a mechanism for carrying protocol messages.

## 3. Protocol Stack

```text
Application Semantics
 ↓
NROS Protocol
 ↓
Message Envelope
 ↓
Serialization
 ↓
Transport
 ↓
Network
```

## 4. Control Plane vs Data Plane

```text
Control Plane
 → authority, topology, scheduling, membership, lifecycle

Data Plane
 → workload payloads, streams, application data
```

The planes may share transport infrastructure while retaining distinct semantics.

## 5. Message Classes

NROS messages should distinguish:

```text
Command
Query
Response
Event
Notification
Stream item
Error
Acknowledgement
```

## 6. Command

A command requests a state-changing action:

```text
Command
 ↓
Validate
 ↓
Authorize
 ↓
Execute / Commit
```

## 7. Query

A query requests information without intentionally changing authoritative state.

Queries still require identity, authorization, consistency, and freshness semantics.

## 8. Event

An event represents an observed or committed fact:

```text
Event = something that happened
```

An event should not be confused with a command requesting that something happen.

## 9. Response

Responses correlate with commands or queries and should explicitly identify success, failure, partial completion, or pending state.

## 10. Error

Protocol errors are structured data rather than arbitrary strings.

```text
code
category
message
retryability
metadata
```

Sensitive data must not leak through error metadata.

## 11. Acknowledgement

An acknowledgement confirms a defined protocol milestone.

```text
received
accepted
committed
applied
```

These are distinct states.

## 12. Canonical Envelope

A generic NROS envelope may contain:

```text
protocol_version
message_type
message_id
correlation_id
causation_id
sender
recipient
scope
epoch
capability/reference
timestamp/logical_time
schema_id
payload
integrity metadata
```

The exact wire representation is implementation-specific.

## 13. Message Identity

Every message requiring deduplication or traceability should have stable identity:

```text
message_id
```

## 14. Correlation

Related request/response flows use:

```text
correlation_id
```

## 15. Causation

Causal chains may use:

```text
causation_id
```

allowing reconstruction of:

```text
Command A
 ↓
Event B
 ↓
Command C
```

## 16. Scope

Messages should declare their authority/semantic scope where relevant:

```text
task
workflow
agent
service
node
cluster
```

## 17. Sender Identity

Sender identity must be authenticated independently from payload claims.

```text
claimed_sender
    ≠
verified_sender
```

## 18. Recipient

A recipient can identify:

```text
node
service
worker
partition
logical endpoint
```

Routing semantics should remain separate from application meaning.

## 19. Epoch

Authority-sensitive messages should carry or be checked against an epoch/term when required.

```text
message epoch < current epoch
        ↓
       reject
```

## 20. Capability Reference

Delegated authority can be represented by a capability or capability reference:

```text
principal
 + scope
 + operation
 + constraints
 + epoch/expiry
```

## 21. Schema Identity

Every structured payload should have explicit schema identity:

```text
schema_id
schema_version
```

## 22. Serialization

Serialization transforms structured protocol data into bytes.

```text
Object
 ↓
Serialize
 ↓
Bytes
 ↓
Deserialize
 ↓
Object
```

Serialization is not itself the protocol.

## 23. Canonical Encoding

Where signatures, hashes, or deterministic comparison depend on serialized bytes, canonical encoding rules must be defined.

## 24. Binary vs Text

Protocol implementations may choose binary or textual encodings according to constraints.

The semantic contract must remain independent of representation where practical.

## 25. Endianness / Numeric Representation

Binary protocols must explicitly define numeric representation, widths, signedness, and byte ordering.

## 26. Strings

String encoding must be explicit, normally with a defined Unicode encoding and length semantics.

## 27. Length Prefixes

Variable-length fields require unambiguous length or framing rules.

## 28. Framing

Streaming transports require message boundaries:

```text
Frame
 ├─ header
 ├─ length
 └─ payload
```

## 29. Maximum Message Size

Every protocol should define maximum accepted message size or an equivalent resource bound.

Oversized messages should be rejected before uncontrolled allocation.

## 30. Resource Limits

Protocol parsing must enforce limits for:

```text
message size
field count
nesting depth
string length
collection length
compression ratio
processing time
```

## 31. Parser Safety

Untrusted input must never be assumed valid merely because it successfully deserializes.

Parsing and semantic validation are separate stages.

## 32. Validation Pipeline

```text
Bytes
 ↓
Framing validation
 ↓
Decoding
 ↓
Schema validation
 ↓
Identity validation
 ↓
Authority validation
 ↓
Semantic validation
 ↓
Execution
```

## 33. Authentication

Authentication establishes who sent a message.

## 34. Authorization

Authorization establishes whether that sender may perform the requested operation.

```text
Authenticated
    ≠
Authorized
```

## 35. Integrity

Integrity protection detects unauthorized modification according to the chosen transport/protocol security model.

## 36. Confidentiality

Sensitive payloads require confidentiality according to the applicable security boundary.

## 37. Replay Protection

Messages that can cause side effects should define replay resistance:

```text
message_id
nonce
sequence
epoch
expiry
```

as appropriate.

## 38. Idempotency

Commands may expose an idempotency key:

```text
idempotency_key
```

Repeated delivery should converge to one logical effect where the operation contract promises idempotency.

## 39. Exactly Once

```text
exactly-once delivery
    ≠
exactly-once effect
```

External side effects require additional transactional or reconciliation mechanisms.

## 40. Delivery Semantics

A protocol should explicitly declare whether an interaction provides:

```text
at-most-once
at-least-once
best-effort
ordered
replayable
```

## 41. Ordering

Ordering may be:

```text
none
per connection
per stream
per key
per partition
global
```

Global ordering should not be implied accidentally.

## 42. Delivery State

A message may progress through:

```text
created
sent
received
accepted
processed
committed
acknowledged
```

## 43. Backpressure

Producers must be able to observe receiver capacity where the protocol requires bounded flow.

## 44. Flow Control

Flow-control semantics should define:

```text
window
credit
queue bound
pause
resume
```

## 45. Cancellation

Long-running requests should support explicit cancellation where appropriate:

```text
request
 ↓
cancel
 ↓
acknowledge cancellation
```

Cancellation is not guaranteed to erase already-committed effects.

## 46. Deadlines

Requests may carry deadlines or timeout budgets.

A receiver must distinguish expired requests from requests with no deadline.

## 47. Timeout Semantics

A timeout does not prove that the remote operation failed.

```text
timeout
 ≠
remote failure
```

The operation may still commit remotely.

## 48. Retries

Retries must use stable request identity and respect idempotency semantics.

## 49. Retryability

Errors should identify whether retry is:

```text
safe
unsafe
conditional
not applicable
```

## 50. Error Taxonomy

Protocol errors can include:

```text
malformed
unsupported_version
unauthorized
forbidden
not_found
conflict
stale_epoch
rate_limited
resource_exhausted
timeout
unavailable
internal
```

## 51. Error Stability

Machine-readable error codes should remain stable across compatible versions.

Human-readable messages may evolve.

## 52. Compatibility

Compatibility has multiple dimensions:

```text
wire compatibility
schema compatibility
semantic compatibility
behavioral compatibility
security compatibility
```

## 53. Versioning

Protocol versioning should distinguish:

```text
major
minor
patch
```

or another explicitly defined scheme.

## 54. Major Changes

Major changes may introduce incompatible semantics and require explicit negotiation or migration.

## 55. Minor Changes

Minor versions may add compatible capabilities where the contract permits.

## 56. Patch Changes

Patch versions should preserve wire and semantic compatibility under the declared versioning policy.

## 57. Schema Evolution

Schemas should support explicit evolution rules:

```text
add field
remove field
rename field
change type
change semantics
```

These operations are not equally compatible.

## 58. Unknown Fields

Receivers may ignore unknown fields only when the schema contract explicitly permits it.

## 59. Required vs Optional Fields

Every field should be classified:

```text
required
optional
defaulted
conditionally required
```

## 60. Default Values

Defaults are semantic behavior and must be versioned like any other behavior.

## 61. Null vs Missing

Protocols must distinguish null, absent, empty, and default where these have different meanings.

## 62. Enum Evolution

Adding enum values can break strict receivers.

Compatibility policy must define unknown-value behavior.

## 63. Capability Negotiation

Peers can negotiate supported features:

```text
Peer A capabilities
 ↕
Peer B capabilities
 ↓
Intersection
```

## 64. Feature Flags

Feature flags should not silently change protocol semantics for peers that have not negotiated the feature.

## 65. Negotiation Handshake

A handshake may establish:

```text
protocol version
schema versions
capabilities
limits
security parameters
compression
```

## 66. Negotiation Failure

If no mutually compatible protocol exists:

```text
Negotiation
 ↓
No compatible version
 ↓
Explicit failure
```

## 67. Downgrade Protection

Negotiation must prevent an attacker or faulty intermediary from forcing an unsafe weaker protocol version.

## 68. Capability vs Version

```text
version
 ≠
capability
```

Two peers on the same protocol version may support different optional features.

## 69. Transport Independence

Protocol semantics should not depend unnecessarily on one transport.

Possible transports include:

```text
Unix domain socket
TCP
QUIC
IPC
in-process channel
```

The concrete implementation determines which are supported.

## 70. Connection Lifecycle

```text
Disconnected
 ↓
Connecting
 ↓
Negotiating
 ↓
Ready
 ↓
Draining
 ↓
Closed
```

## 71. Connection Identity

Connection identity should not replace logical message identity.

Connections can reconnect while preserving higher-level request semantics.

## 72. Multiplexing

Multiple logical streams may share one transport connection.

Stream identity must be explicit.

## 73. Stream Lifecycle

```text
Created
 ↓
Open
 ↓
Half-closed
 ↓
Closed
```

## 74. Heartbeats

Heartbeats are liveness signals, not authoritative proof of application health.

## 75. Keepalive

Transport keepalive may detect dead connections but does not replace protocol-level health semantics.

## 76. Compression

Compression should be negotiated and bounded.

Decompression must enforce resource limits to prevent expansion attacks.

## 77. Checksums

Checksums can detect accidental corruption; cryptographic integrity mechanisms are required for adversarial tampering.

## 78. Encryption

Confidential transport must define:

```text
algorithm
key establishment
identity verification
rotation
failure behavior
```

## 79. Protocol State Machine

Each stateful protocol should define legal transitions:

```text
Idle
 ↓ request
Accepted
 ↓ processing
Completed
```

Illegal transitions must produce structured errors.

## 80. State Machine Versioning

Changing protocol state transitions can be a semantic breaking change even when the wire schema remains compatible.

## 81. Command Semantics

Commands should define:

```text
preconditions
authority
side effects
commit point
idempotency
failure behavior
```

## 82. Query Semantics

Queries should define:

```text
consistency level
freshness
authorization
pagination
resource limits
```

## 83. Event Semantics

Events should define:

```text
fact identity
producer
ordering
retention
replay behavior
schema
```

## 84. Event Replay

Consumers should know whether events are:

```text
replayable
ephemeral
compacted
loss-tolerant
```

## 85. Event Ordering

Ordering guarantees should be scoped explicitly.

## 86. Event Deduplication

Replayable streams require consumer strategies for duplicate events when delivery is at-least-once.

## 87. Command/Event Separation

A command expresses intent:

```text
Do X
```

An event expresses fact:

```text
X happened
```

Mixing these semantics creates ambiguous APIs.

## 88. Response Semantics

Responses should distinguish:

```text
success
accepted-but-pending
partial
conflict
failure
```

## 89. Pagination

Large query results require bounded pagination:

```text
page_size
cursor
next_cursor
```

Cursors should be stable under the declared consistency model.

## 90. Streaming

Streaming APIs should define:

```text
stream identity
ordering
backpressure
termination
reconnect
resume
```

## 91. Resume Tokens

Reconnectable streams may expose a resume token representing a valid replay position.

## 92. Protocol Observability

Every protocol operation should support traceability through:

```text
message_id
correlation_id
causation_id
trace/span context
```

## 93. Protocol Metrics

Useful metrics include:

```text
message rate
latency
retries
rejections
queue depth
bytes
error categories
protocol version usage
```

## 94. Audit Evidence

Authority-sensitive commands should produce sufficient evidence to reconstruct:

```text
who requested
what was requested
under which epoch
which policy applied
what result occurred
```

## 95. Redaction

Logs and protocol traces must prevent sensitive payloads from becoming accidental observability data.

## 96. Schema Registry

If a registry is used, schema identity and compatibility rules must be authoritative and versioned.

A registry is an implementation mechanism, not a substitute for protocol governance.

## 97. Contract Testing

Protocol implementations should test:

```text
valid messages
invalid messages
boundary sizes
unknown fields
version negotiation
replay
ordering
timeouts
cancellation
compatibility
```

## 98. Formal Wire Contract Invariant

```text
Encode(Decode(bytes))
    ≡
Canonical(bytes)
```

where canonical equivalence is defined by the serialization contract.

## 99. Formal Authority Invariant

```text
Accept(Command)
    ⇒
Authenticated(Sender)
 ∧
Authorized(Sender, Command)
 ∧
CurrentEpoch(Command)
```

## 100. Formal Compatibility Invariant

```text
Compatible(PeerA, PeerB)
    ⇒
AgreedVersion
 ∧
AgreedCapabilities
 ∧
CompatibleSchemas
 ∧
CompatibleSemantics
```

## 101. Verification Matrix

| Property | Verification question |
|---|---|
| Envelope | Are identity, correlation, scope, and version explicit? |
| Authentication | Is sender identity independently verified? |
| Authorization | Is authority checked separately? |
| Schema | Is schema identity explicit? |
| Framing | Are message boundaries unambiguous? |
| Limits | Are parsing/resource limits enforced? |
| Replay | Are side-effecting messages replay-safe? |
| Idempotency | Can retries converge safely? |
| Ordering | Is ordering scope explicit? |
| Delivery | Are delivery guarantees explicit? |
| Backpressure | Is producer/consumer capacity bounded? |
| Cancellation | Are cancellation semantics defined? |
| Timeout | Is timeout distinguished from remote failure? |
| Errors | Are machine-readable error categories stable? |
| Versioning | Are incompatible changes explicit? |
| Negotiation | Can peers establish common capabilities? |
| Downgrade | Is unsafe downgrade prevented? |
| Serialization | Are encoding rules deterministic where required? |
| Security | Are integrity/confidentiality boundaries explicit? |
| Observability | Can distributed operations be reconstructed? |
| Compatibility | Can old/new peers interoperate safely? |

## 102. What Part XLVIII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a finalized universal wire format;
- production protocol negotiation;
- complete backward/forward compatibility;
- production schema registry infrastructure;
- universal exactly-once effects;
- transport-independent implementations;
- formally verified serialization;
- complete Byzantine-secure messaging.

Those require implementation-specific evidence.

## 103. Transition to Part XLIX

Part XLVIII establishes the protocol and wire-contract plane.

Part XLIX should define **API and service boundary architecture: resource models, RPC semantics, endpoint contracts, streaming APIs, lifecycle APIs, error surfaces, pagination, idempotency, and external-facing NROS interfaces**.

```text
Part XLVII
Distributed authority + consensus + membership
        ↓
Part XLVIII
Protocol + messages + compatibility + wire contracts
        ↓
Part XLIX
API + service boundaries + external interfaces
```

## Canonical rule

> **NROS does not treat serialization as a protocol: every cross-boundary operation must carry explicit semantic identity, authority, schema, version, correlation, delivery, validation, compatibility, and failure contracts.**
