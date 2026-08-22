# Part XXXI — Protocol Sessions, Negotiation, Compatibility & Evolution

> **Series:** NROS Architecture Series  
> **Part:** XXXI  
> **Role:** Handshakes, sessions, version negotiation, feature exchange, compatibility, downgrade resistance, rekeying, graceful shutdown, and protocol evolution  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXX established identity, naming, addressing, discovery, and trust. Part XXXI defines how trusted parties establish protocol sessions, negotiate mutually supported behavior, maintain compatibility, evolve protocol contracts, and terminate communication safely.

The central rule is:

> **NROS must negotiate protocol behavior explicitly: protocol version, feature set, capabilities, session state, and identity are distinct concepts, and successful connection establishment must never imply compatibility or authority that was not explicitly established.**

## 2. Fundamental Distinctions

```text
protocol version
  ≠
feature set
  ≠
capability
  ≠
identity
  ≠
session state
  ≠
transport state
  ≠
application state
```

## 3. Protocol Stack

A conceptual NROS communication path is:

```text
Identity / Trust
       ↓
Transport
       ↓
Handshake
       ↓
Protocol Negotiation
       ↓
Session
       ↓
Application Messages
```

Each layer has its own lifecycle and failure semantics.

## 4. Handshake

The handshake establishes the minimum information required before normal protocol traffic:

```text
Connect
 ↓
Peer identification
 ↓
Credential verification
 ↓
Protocol negotiation
 ↓
Feature negotiation
 ↓
Session establishment
```

The handshake should be bounded and failure-safe.

## 5. Handshake State Machine

```text
Idle
 ↓
Connecting
 ↓
Handshaking
 ├─ Reject
 └─ Negotiate
       ↓
   Established
       ↓
   Active
       ↓
   Closing
       ↓
   Closed
```

Timeout, authentication failure, protocol mismatch, or policy rejection may terminate the handshake.

## 6. Version Negotiation

Peers may advertise supported protocol versions:

```text
Peer A: V1, V2, V3
Peer B: V2, V3
          ↓
Selected: V3
```

Selection must be deterministic under the same advertised inputs and policy.

## 7. Compatibility

Compatibility should distinguish:

```text
wire compatibility
semantic compatibility
behavioral compatibility
security compatibility
operational compatibility
```

Two peers can parse the same message while still being semantically incompatible.

## 8. Feature Negotiation

Features should be negotiated independently where possible:

```text
Features A: F1 F2 F3 F5
Features B: F2 F3 F4
             ↓
Common:       F2 F3
```

Feature support does not automatically grant permission to use a feature.

## 9. Capability vs Feature

A feature describes protocol functionality.

A capability describes authority or permitted operation.

```text
Feature: streaming supported
Capability: principal may stream resource R
```

Both may be required before an operation is valid.

## 10. Negotiated Session Contract

After negotiation, the session should have an explicit contract:

```text
Session
 ├─ peer identity
 ├─ protocol version
 ├─ negotiated features
 ├─ security context
 ├─ limits
 ├─ framing rules
 └─ lifecycle state
```

## 11. Limits Negotiation

Peers may negotiate bounded values such as:

```text
maximum frame size
maximum message size
stream count
window size
concurrency
idle timeout
keepalive interval
```

Negotiated values must remain within local policy limits.

## 12. Local Policy Dominance

A peer must not obtain a capability merely by requesting it during negotiation:

```text
Peer request
     ↓
Local policy
     ↓
Allowed subset
```

Negotiation selects from what both sides support and what policy permits.

## 13. Downgrade Resistance

An attacker should not be able to force a weaker protocol or security mode merely by manipulating negotiation.

The protocol should authenticate or integrity-protect the negotiation transcript where required.

## 14. Minimum Security Version

A trust domain may define:

```text
minimum accepted protocol version
minimum cryptographic strength
minimum authentication mechanism
```

Older peers must fail explicitly rather than silently entering an unsafe compatibility mode.

## 15. Negotiation Transcript

Where security requires it, the session can bind its resulting state to a transcript:

```text
Identity
 + Versions
 + Features
 + Parameters
 + Handshake messages
       ↓
Session binding
```

This prevents peers from authenticating one set of parameters while using another.

## 16. Session Identity

A session should have a unique identity within its authority scope:

```text
Session ID
 + peer identity
 + incarnation
```

The session identity should not be confused with the long-lived identity of the node or agent.

## 17. Session Establishment

A session becomes established only after all mandatory conditions are satisfied:

```text
identity verified
AND
protocol compatible
AND
features compatible
AND
policy satisfied
AND
limits accepted
```

## 18. Session State

The session state machine may include:

```text
Negotiating
Established
Active
Idle
Rekeying
Draining
Closing
Closed
Failed
```

Invalid transitions should be rejected.

## 19. Framing

A protocol must define message boundaries independently of the underlying transport where necessary:

```text
frame header
payload
integrity metadata
```

Frame parsing must enforce size and structural limits before expensive processing.

## 20. Message Ordering

Each protocol should explicitly define whether messages are:

```text
ordered
unordered
partially ordered
per-stream ordered
```

Transport ordering must not automatically be mistaken for application ordering.

## 21. Request Identity

Requests should have stable identifiers when retries, deduplication, or tracing require them:

```text
request_id
session_id
principal
operation
```

A request identifier should not be reused across incompatible scopes.

## 22. Idempotency

Protocol operations should declare whether they are:

```text
idempotent
non-idempotent
conditionally idempotent
```

Retry behavior must respect the operation's semantics.

## 23. Retry Semantics

A retry may occur because of:

```text
transport failure
timeout
session reset
server overload
leader change
```

The protocol must distinguish “operation not received” from “operation received but response lost” where that distinction matters.

## 24. Duplicate Suppression

At-least-once delivery may produce duplicate requests.

A receiver may require:

```text
request_id
operation identity
idempotency key
replay window
```

for safe duplicate handling.

## 25. Flow Control

Session-level flow control can bound outstanding data:

```text
sender
 ↓
window / credit
 ↓
receiver
```

Flow-control semantics should be separate from congestion control defined in Part XXVI.

## 26. Backpressure

Application-level backpressure may propagate through the session:

```text
consumer slow
   ↓
application queue
   ↓
protocol window
   ↓
sender throttled
```

The protocol should avoid unbounded buffering.

## 27. Multiplexing

A session may contain multiple logical streams:

```text
Session
 ├─ Stream A
 ├─ Stream B
 └─ Stream C
```

Stream isolation, ordering, cancellation, and resource limits must be explicit.

## 28. Stream Lifecycle

```text
Created
 ↓
Opening
 ↓
Active
 ↓
Half-closed
 ↓
Closed
```

Stream failure should not necessarily terminate the entire session unless required by the protocol contract.

## 29. Cancellation

Cancellation should identify the target precisely:

```text
session cancellation
stream cancellation
request cancellation
operation cancellation
```

Cancellation semantics must define whether already-committed effects remain valid.

## 30. Deadlines and Timeouts

Operations may carry:

```text
deadline
absolute timeout
idle timeout
handshake timeout
```

A timeout is not automatically proof that the remote operation failed.

## 31. Rekeying

Long-lived secure sessions may rotate cryptographic material:

```text
Key K1
 ↓ rekey
Key K2
```

Rekeying should preserve session identity while invalidating obsolete key material according to policy.

## 32. Renegotiation

If protocol parameters can change after establishment, renegotiation must be authenticated and state-aware.

Unexpected parameter changes should be rejected.

## 33. Capability Refresh

Session authority may be refreshed:

```text
Capability generation N
       ↓ refresh
Capability generation N+1
```

A refresh must not silently expand authority.

## 34. Session Resumption

A session may be resumed after temporary transport loss if the security model permits it.

Resumption should verify:

```text
peer identity
session context
credential validity
protocol compatibility
session generation
```

## 35. Session Migration

A logical session may move between transport endpoints:

```text
Session S
 A1 → A2
```

Migration must preserve or explicitly re-establish identity, authorization, ordering, and security guarantees.

## 36. Graceful Shutdown

Normal termination should provide an explicit lifecycle:

```text
Active
 ↓
Draining
 ↓
Close signal
 ↓
Acknowledgement
 ↓
Closed
```

The protocol should define what happens to outstanding requests.

## 37. Abrupt Termination

Peers must tolerate:

```text
connection reset
process crash
node failure
network partition
credential revocation
```

Recovery behavior should distinguish transport failure from confirmed application failure.

## 38. Protocol Errors

Errors should distinguish classes such as:

```text
malformed message
unsupported version
unsupported feature
unauthorized operation
resource exhaustion
policy rejection
state violation
internal failure
```

Clients should not infer detailed security-sensitive information from error text.

## 39. Error Stability

Machine-readable error classes/codes should remain stable enough for clients to make safe decisions.

Human-readable messages may evolve independently.

## 40. Version Evolution

A protocol can evolve through:

```text
V1
 ↓ additive extension
V2
 ↓ semantic expansion
V3
```

Every change should classify compatibility impact.

## 41. Additive Extensions

New optional fields or features can often preserve wire compatibility when unknown data can be safely ignored.

Ignoring an unknown field is only safe when its omission cannot change required security or correctness semantics.

## 42. Breaking Changes

A breaking change may alter:

```text
message meaning
required fields
state transitions
security assumptions
ordering
error semantics
```

Such changes require explicit versioning or equivalent negotiation boundaries.

## 43. Unknown Fields

The protocol should define whether unknown fields are:

```text
ignored
preserved
rejected
forwarded
```

Security-sensitive fields should not be silently ignored if doing so weakens policy.

## 44. Extension Registry

Extensible protocol identifiers should have a governed namespace:

```text
feature_id
extension_id
message_type
error_code
```

Collision and ownership rules must be explicit.

## 45. Compatibility Matrix

A compatibility matrix can model peer behavior:

| Local | Remote | Result |
|---|---|---|
| supported | supported | negotiate |
| supported | unsupported optional feature | disable feature |
| required | unsupported | reject |
| newer | older compatible | select compatible mode |
| newer | older incompatible | reject |
| security requirement unmet | any | reject |

## 46. Security and Negotiation

Negotiation must be subordinate to the trust and authorization model:

```text
Identity
 ↓
Trust
 ↓
Policy
 ↓
Negotiation
 ↓
Session authority
```

Negotiated protocol support must never override security policy.

## 47. Persistence Interaction

Part XXVII may persist:

```text
session recovery metadata
protocol compatibility state
sequence checkpoints
revocation state
```

Persistent session state must not resurrect invalid authority after restart.

## 48. Resource Interaction

Part XXVIII governs resources consumed by sessions:

```text
connections
buffers
streams
queues
memory
CPU
```

Session negotiation must respect local resource quotas.

## 49. Isolation Interaction

Part XXIX determines whether a session may access a resource:

```text
Session identity
 + capability
 + isolation policy
      ↓
Authorized operation
```

Protocol correctness does not bypass sandbox boundaries.

## 50. Identity Interaction

Part XXX provides:

```text
identity
incarnation
credential
trust
endpoint binding
```

Part XXXI consumes these facts to construct a trusted session.

## 51. Deterministic Negotiation

Where the same inputs and policies apply:

```text
Negotiation(A, B, policy)
        ↓
same result
```

unless nondeterministic inputs are explicitly part of the contract.

## 52. Formal Session Safety

A conceptual invariant is:

```text
Established(S)
    ⇒
IdentityVerified(S)
 ∧ ProtocolCompatible(S)
 ∧ PolicySatisfied(S)
 ∧ LimitsValid(S)
```

## 53. Downgrade Safety Invariant

```text
NegotiatedVersion(V)
    ⇒
V satisfies the minimum security/version policy
and is supported by both peers.
```

## 54. Session Authority Invariant

```text
AuthorizedOperation(S, O)
    ⇒
SessionAuthority(S)
 ∧ CapabilityAllows(O)
 ∧ PolicyAllows(O)
```

A negotiated feature alone is never sufficient authority.

## 55. Verification Matrix

| Property | Verification question |
|---|---|
| Handshake | Are mandatory establishment steps explicit? |
| Identity | Is the peer identity verified? |
| Version | Is version selection deterministic and policy-bounded? |
| Features | Are features explicitly negotiated? |
| Capability | Is authority distinct from feature support? |
| Downgrade | Can an attacker force an unsafe mode? |
| Limits | Are negotiated limits locally bounded? |
| Framing | Are message boundaries and size limits enforced? |
| Ordering | Are ordering guarantees explicit? |
| Retry | Are retry semantics safe for each operation? |
| Duplicates | Can duplicate requests be handled safely? |
| Flow control | Is buffering bounded? |
| Multiplexing | Are stream limits and isolation defined? |
| Cancellation | Are cancellation effects explicit? |
| Deadlines | Are timeout semantics unambiguous? |
| Rekey | Can long-lived sessions rotate keys safely? |
| Resumption | Is resumed authority revalidated? |
| Shutdown | Is graceful termination deterministic? |
| Evolution | Are compatibility impacts classified? |
| Persistence | Can restart resurrect invalid sessions? |
| Formal assurance | Are session invariants explicit? |

## 56. What Part XXXI Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a universal protocol handshake;
- complete version negotiation;
- feature negotiation across all transports;
- formally verified downgrade resistance;
- production session resumption;
- universal rekeying;
- complete multiplexed streams;
- fully specified compatibility matrices;
- formally verified session-state machines.

Those require implementation-specific evidence.

## 57. Transition to Part XXXII

Part XXXI defines trusted protocol sessions and their evolution.

Part XXXII should define **serialization, schemas, message contracts, canonical encoding, compatibility rules, validation, framing, and data evolution**, establishing how NROS turns protocol semantics into stable machine-readable representations.

```text
Part XXX
Identity + naming + addressing + discovery + trust
        ↓
Part XXXI
Sessions + negotiation + compatibility + evolution
        ↓
Part XXXII
Serialization + schemas + encoding + validation + data evolution
```

## Canonical rule

> **NROS establishes communication through an explicit, authenticated, policy-bounded session contract: peers negotiate only mutually supported behavior, negotiated parameters remain subordinate to local security and resource policy, retries and cancellation preserve operation semantics, and protocol evolution must remain explicit, compatibility-aware, and resistant to unsafe downgrade or stale authority.**
