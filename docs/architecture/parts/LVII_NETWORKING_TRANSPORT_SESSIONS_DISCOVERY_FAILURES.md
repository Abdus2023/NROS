# Part LVII — Networking, Transport, Sessions, Discovery & Failure Semantics

> **Series:** NROS Architecture Series  
> **Part:** LVII  
> **Role:** Addressing, discovery, connections, sessions, transport, flow control, reliability, failure detection, recovery, and network security boundaries  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LVI established messaging and eventing. Part LVII defines the network substrate over which those semantics may operate.

The central rule is:

> **NROS separates addressing, discovery, connection, session, transport, delivery, and application semantics; success at one layer must not be mistaken for success at another.**

## 2. Layer Distinctions

```text
Address
  ≠
Discovery
  ≠
Connection
  ≠
Session
  ≠
Transport
  ≠
Message Delivery
  ≠
Application Completion
```

## 3. Addressing

An address identifies a reachable communication endpoint according to a declared namespace.

Possible components include:

```text
scheme
network domain
host / node
port / endpoint
service identity
resource path
```

## 4. Logical vs Physical Address

```text
Logical Service Identity
        ≠
Current Network Location
```

Services may move while retaining logical identity.

## 5. Address Ownership

Address allocation and authority should be explicit where collisions or spoofing could affect correctness.

## 6. Endpoint Identity

An endpoint may be identified by:

```text
node identity
service identity
instance identity
network address
certificate / credential identity
```

These identities must not be conflated.

## 7. Discovery

Discovery maps logical service requirements to reachable endpoints.

```text
Service Requirement
 ↓
Discovery
 ↓
Candidate Endpoints
```

## 8. Discovery Authority

Discovery data should identify its source and validity period where stale information could be dangerous.

## 9. Discovery Freshness

A discovered endpoint may become unavailable immediately after discovery.

```text
Discovered
   ↓
Attempt Connect
   ↓
Unavailable
```

Discovery is therefore a hint or authoritative registry result according to its declared contract, not proof of reachability.

## 10. Service Registration

A service registration may contain:

```text
service_id
instance_id
capabilities
endpoint
version
health
expiry
metadata
```

## 11. Registration Lease

Registrations may use leases or TTLs.

Expired registrations must not remain authoritative indefinitely.

## 12. Capability Discovery

Discovery may expose capabilities separately from location:

```text
Capability
 ↓
Eligible Endpoints
```

Capability claims should be evidence-backed when used for safety-critical placement.

## 13. Connection

A connection represents an established transport relationship between endpoints.

```text
Discover
 ↓
Connect
 ↓
Transport Established
```

## 14. Connection Identity

Connections may carry:

```text
connection_id
local endpoint
remote endpoint
transport parameters
security context
creation time
```

## 15. Connection vs Session

A connection is a transport relationship; a session represents higher-level communication continuity.

```text
Connection
   ≠
Session
```

A session may survive connection replacement if the protocol supports resumption.

## 16. Session

A session may carry:

```text
session_id
principal
protocol version
capabilities
state revision
sequence state
lease / expiry
```

## 17. Session Resumption

If supported:

```text
Connection A
 ↓ failure
Session preserved
 ↓
Connection B
 ↓
Resume Session
```

Resumption must protect against stale or unauthorized clients.

## 18. Session Fencing

Session epochs can prevent old connections from continuing protected operations:

```text
Session epoch 8
Current epoch 9
      ↓
Reject protected action
```

## 19. Transport

Transport provides a defined set of properties such as:

```text
byte delivery
ordering
reliability
flow control
congestion handling
connection semantics
```

The application must not assume properties not provided by the selected transport.

## 20. Reliable Transport

Reliable byte delivery means bytes arrive according to transport guarantees.

It does not imply successful application processing.

```text
Reliable Transport
      ≠
Reliable Application Operation
```

## 21. Datagram Transport

Datagram-based communication may expose:

```text
loss
duplication
reordering
variable latency
```

Application protocols must explicitly compensate when needed.

## 22. Framing

Stream transports require message framing:

```text
bytes
 ↓
frame
 ↓
message
```

Framing errors must not silently become application payloads.

## 23. Maximum Frame Size

Frame size should be bounded to prevent memory exhaustion and parser abuse.

## 24. Flow Control

Flow control protects receivers from producers that operate faster than they can consume.

```text
Producer
 ↓
Window / Credit
 ↓
Receiver
```

## 25. Credit-Based Flow Control

A receiver may grant explicit credits:

```text
Credit = N
```

The sender must remain within the granted window.

## 26. Congestion

Network congestion is distinct from application backpressure but may interact with it.

```text
Network Pressure
      +
Application Pressure
```

must not be collapsed into one unexplained failure state.

## 27. Timeouts

Network operations should distinguish:

```text
connect timeout
handshake timeout
read timeout
write timeout
idle timeout
session timeout
deadline
```

## 28. Timeout Semantics

A timeout indicates that the expected progress condition was not observed within the declared interval.

It does not always prove that the remote operation failed.

## 29. Unknown Remote Outcome

```text
Request Sent
 ↓
Connection Lost
 ↓
Remote Outcome Unknown
```

Retry requires idempotency, reconciliation, or another safety mechanism.

## 30. Keepalive

Keepalive mechanisms can detect broken communication paths but do not prove application health.

```text
Keepalive
 ≠
Application Progress
```

## 31. Failure Detection

Failure detectors operate under imperfect information.

A suspected failure should be represented as suspicion rather than absolute fact when the model requires that distinction.

## 32. Failure States

Possible network states include:

```text
healthy
slow
degraded
suspected unreachable
unreachable
recovering
```

## 33. Partial Failure

Distributed systems must assume components can fail independently:

```text
client alive
server alive
network partitioned
```

## 34. Network Partition

A partition may isolate otherwise healthy components.

Partition handling must respect the authority model from Part XLVII.

## 35. Split Brain

Network connectivity alone must never determine authoritative ownership where consensus or fencing is required.

## 36. Reconnect

Reconnect policy may use:

```text
bounded retries
exponential backoff
jitter
circuit breaking
```

## 37. Reconnect Storms

Large populations reconnecting simultaneously can amplify failures.

Jitter and admission control should prevent synchronized retry storms.

## 38. Circuit Breaker

A circuit breaker can transition:

```text
Closed
 ↓ repeated failure
Open
 ↓ recovery window
Half-Open
 ↓ success
Closed
```

Exact thresholds are policy-defined.

## 39. Connection Pooling

Connection pools should bound:

```text
connections
idle lifetime
pending requests
memory
```

## 40. Multiplexing

Multiple logical streams may share one connection.

A single connection failure may therefore affect multiple logical operations.

## 41. Stream Isolation

Multiplexed protocols should prevent one stream from consuming all shared resources where fairness matters.

## 42. Head-of-Line Blocking

Transport or application design should account for head-of-line blocking where it can affect latency or availability.

## 43. Network Backpressure

Backpressure may propagate through layers:

```text
Receiver
 ↓
Transport Window
 ↓
Connection
 ↓
Message Queue
 ↓
Producer
```

## 44. MTU / Fragmentation

Network payload sizing should account for transport and network limits.

Large application messages may require application-level chunking or artifact references.

## 45. Chunking

Chunked transfer requires:

```text
message identity
chunk index
chunk count or completion marker
integrity
reassembly timeout
```

## 46. Reassembly

Incomplete or conflicting chunks must not be interpreted as a complete message.

## 47. Network Security

Communication may require:

```text
authentication
authorization
confidentiality
integrity
replay protection
peer verification
```

## 48. Mutual Authentication

For trusted peer communication, both sides may authenticate each other.

Authentication establishes identity; authorization establishes permission.

## 49. Certificate / Credential Rotation

Credential rotation must support overlapping validity or controlled session migration where required.

## 50. Trust Establishment

A connection should establish the trust context before privileged operations are accepted.

## 51. Secure Channel

A secure channel protects data according to its declared threat model.

It does not automatically validate application-level authorization.

## 52. Encryption Boundaries

Encryption may terminate at:

```text
endpoint
proxy
gateway
service mesh
```

Each termination changes the trust boundary and must be explicit.

## 53. Proxy

A proxy may forward traffic without becoming the application authority.

If it terminates security or changes identity, that transformation must be explicit.

## 54. Gateway

A gateway may translate:

```text
protocol
address
identity
schema
policy
```

Semantic changes must be observable where they affect correctness.

## 55. NAT / Address Translation

Network location may change without logical identity changing.

Therefore logical identity should not depend solely on source address.

## 56. Service Mobility

A service may move between nodes while retaining service identity.

Discovery and connection establishment must tolerate this when mobility is supported.

## 57. Connection Draining

Before planned shutdown:

```text
Stop New Requests
 ↓
Drain Existing Streams
 ↓
Close Connections
```

## 58. Graceful Shutdown

Graceful network shutdown should distinguish:

```text
stop accepting
finish existing
cancel existing
force close
```

## 59. Abrupt Disconnect

Remote disconnect should trigger bounded recovery behavior rather than unbounded retries.

## 60. Session Expiry

Sessions may expire due to:

```text
TTL
idle timeout
credential expiry
authority change
explicit revocation
```

## 61. Revocation

Revoked credentials or sessions must not regain privileged access merely by reconnecting with stale state.

## 62. Network Identity vs Workload Identity

```text
Network Connection Identity
      ≠
Workload Identity
```

A single connection may carry many workloads; one workload may use multiple connections.

## 63. Network Identity vs Security Principal

A network endpoint is not necessarily a security principal.

Identity binding must be explicit.

## 64. Network Identity vs Resource Identity

An address does not prove ownership of a resource.

Resource authorization belongs to the resource policy plane.

## 65. Discovery + Authorization

Discovery should not expose sensitive endpoints to unauthorized principals where information disclosure matters.

## 66. Capability-Based Routing

Routing may be constrained by capability requirements:

```text
Required Capability
 ↓
Eligible Endpoint Set
 ↓
Policy Filter
 ↓
Connection
```

## 67. Version Negotiation

Peers may negotiate:

```text
protocol version
features
compression
security options
message limits
```

## 68. Feature Negotiation

Negotiated features must have deterministic fallback behavior.

Unknown mandatory features should cause explicit incompatibility rather than silent degradation.

## 69. Protocol Compatibility

Compatibility should distinguish:

```text
wire compatibility
semantic compatibility
security compatibility
operational compatibility
```

## 70. Protocol Upgrade

A protocol upgrade should define whether existing sessions:

```text
continue
migrate
restart
expire
```

## 71. Health Checking

Health checks may test:

```text
reachability
transport readiness
protocol readiness
application readiness
```

These are distinct signals.

## 72. Readiness

A service can be reachable but not ready to accept work.

```text
Reachable
   ≠
Ready
```

## 73. Liveness

A service can be alive but unable to make useful progress.

```text
Alive
   ≠
Healthy
```

## 74. Load Balancing

Load balancing selects among eligible endpoints.

Eligibility should account for:

```text
health
capacity
policy
capability
locality
```

## 75. Load Balancer State

Load-balancing decisions may depend on stale observations and therefore require bounded staleness handling.

## 76. Locality

Routing may prefer:

```text
same process
same node
same zone
same region
```

according to latency, cost, or failure-domain policy.

## 77. Failure-Domain Awareness

Network routing should avoid concentrating critical dependencies in one failure domain where policy requires resilience.

## 78. Retry Routing

Retries may select a different endpoint when the failure model indicates that doing so improves recovery.

Retry must not bypass affinity or security constraints without policy authorization.

## 79. Request Hedging

Hedged requests can reduce tail latency but may duplicate work.

They require explicit idempotency and resource-budget controls.

## 80. Cancellation Propagation

Cancellation should propagate through network boundaries when protocol support exists:

```text
Caller Cancels
 ↓
Request Cancellation
 ↓
Remote Processing Cancellation
```

Remote cancellation remains subject to authorization and implementation semantics.

## 81. Deadline Propagation

A request deadline may propagate across service boundaries.

Each hop must avoid extending the deadline beyond the original contract without explicit policy.

## 82. Priority Propagation

Priority may propagate across network calls, but it must remain bounded by receiving-side policy.

## 83. Trace Propagation

Trace context may propagate through network calls:

```text
Trace
 ↓
Service A
 ↓
Service B
 ↓
Service C
```

Trace metadata is observational, not authoritative state.

## 84. Message Size / Resource Limits

Network interfaces should bound:

```text
frame size
request size
response size
concurrent streams
buffer memory
```

## 85. Parser Safety

Network parsers must reject malformed input safely and within bounded resource consumption.

## 86. Slowloris / Slow Sender

Long-lived partial input should be bounded by:

```text
read deadlines
minimum progress rates
connection limits
resource quotas
```

## 87. Connection Exhaustion

Connection creation must be subject to resource limits and admission policy.

## 88. Port / Endpoint Exhaustion

The architecture should account for finite endpoint resources and avoid unbounded connection churn.

## 89. Network Recovery

Recovery should follow:

```text
Detect
 ↓
Classify
 ↓
Backoff
 ↓
Reconnect
 ↓
Re-authenticate
 ↓
Re-establish Session
 ↓
Resume / Replay / Fail
```

## 90. Resume Semantics

Session resume must identify the last safely processed message or state revision where required.

## 91. Duplicate Requests After Reconnect

Reconnect can produce ambiguous retry situations.

Idempotency and durable request identity should determine whether retry is safe.

## 92. Network Partition Recovery

After partition healing:

```text
Re-authenticate
 ↓
Validate Epoch
 ↓
Reconcile State
 ↓
Resume Communication
```

## 93. Authority After Partition

Connectivity restoration does not automatically restore authority.

The authority model remains the source of truth.

## 94. Transport Observability

Network telemetry should include, where appropriate:

```text
connection count
connection failures
RTT
retransmissions
timeouts
bytes
frames
queue depth
stream count
reconnect rate
```

## 95. Network Evidence

Evidence should distinguish:

```text
endpoint discovered
connection established
session authenticated
message delivered
message processed
application completed
```

## 96. Formal Reachability Invariant

```text
Discovered(E)
    ⇏
Reachable(E)
```

Discovery does not prove current reachability.

## 97. Formal Transport Invariant

```text
TransportDelivered(M)
    ⇏
ApplicationProcessed(M)
```

## 98. Formal Session Invariant

```text
SessionValid(S)
    ⇒
CurrentAuthority(S)
 ∧
SecurityContextValid(S)
```

where those properties are part of the session contract.

## 99. Formal Retry Invariant

```text
ConnectionLost(R)
 ∧
RemoteOutcomeUnknown(R)
    ⇒
RetryOnlyIfSafeOrReconciled(R)
```

## 100. Formal Fencing Invariant

```text
OldSessionEpoch < CurrentEpoch
    ⇒
Reject(ProtectedAction)
```

## 101. Verification Matrix

| Property | Verification question |
|---|---|
| Addressing | Are logical and physical identities distinct? |
| Discovery | Is freshness and authority explicit? |
| Connection | Is connection identity defined? |
| Session | Can sessions resume safely? |
| Transport | Are transport guarantees explicit? |
| Framing | Are malformed frames safely rejected? |
| Flow control | Are sender/receiver limits bounded? |
| Congestion | Is network pressure distinguished from application pressure? |
| Timeouts | Are timeout types and meanings explicit? |
| Failure | Can partial failures be represented? |
| Retry | Are ambiguous remote outcomes handled safely? |
| Security | Are authentication and authorization separate? |
| Credentials | Can stale credentials regain access? |
| Versioning | Are protocol compatibility rules explicit? |
| Health | Are reachability, readiness, and health distinct? |
| Routing | Are capacity and locality considered? |
| Multiplexing | Can one connection failure affect many streams? |
| Shutdown | Can connections drain safely? |
| Recovery | Can sessions reconnect without duplication hazards? |
| Evidence | Can network claims be independently verified? |

## 102. What Part LVII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production service-discovery system;
- universal session resumption;
- complete connection pooling;
- production-grade circuit breaking;
- universal mutual authentication;
- universal end-to-end encryption;
- complete network partition recovery;
- universal exactly-once request semantics;
- production-grade global load balancing;
- automatic semantic protocol migration.

Those require implementation-specific evidence.

## 103. Transition to Part LVIII

Part LVII establishes the network substrate.

Part LVIII should define **API semantics, RPC contracts, service composition, request/response lifecycle, streaming APIs, compatibility, errors, and boundary governance**.

```text
Part LVI
Messaging + events + delivery + replay
        ↓
Part LVII
Networking + transport + sessions + discovery
        ↓
Part LVIII
APIs + RPC + service contracts + boundary governance
```

## Canonical rule

> **NROS treats discovery, connection, session, transport, delivery, and application completion as separate claims; network reliability or reachability never becomes evidence of application success without an explicit end-to-end contract.**
