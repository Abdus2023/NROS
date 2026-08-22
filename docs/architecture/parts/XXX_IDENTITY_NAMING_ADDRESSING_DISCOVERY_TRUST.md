# Part XXX — Identity, Naming, Addressing, Discovery & Trust

> **Series:** NROS Architecture Series  
> **Part:** XXX  
> **Role:** Identity, names, addresses, endpoints, discovery, trust establishment, key binding, rotation, revocation, and cross-domain identity  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXIX defined isolation, capabilities, and containment. Part XXX defines how NROS identifies actors and resources, names them, locates them, discovers endpoints, establishes trust, and manages identity over time.

The central rule is:

> **NROS must not confuse identity with naming or location: identity answers “who/what is this?”, naming provides a stable reference, addressing describes where communication can occur, discovery resolves current reachability, and trust establishes which identity claims are acceptable.**

## 2. Fundamental Distinctions

```text
identity
  ≠
name
  ≠
address
  ≠
endpoint
  ≠
location
  ≠
credential
  ≠
trust
```

These concepts may be represented by related identifiers but have different lifecycles and failure modes.

## 3. Identity

Identity represents the continuity of an actor or resource across its declared lifecycle:

```text
Identity
 ├─ stable identifier
 ├─ type
 ├─ issuer / authority
 ├─ generation
 ├─ credentials / key bindings
 └─ lifecycle state
```

An identity should not silently change merely because its network address changes.

## 4. Identity Types

NROS may identify:

```text
node
process
agent
session
workflow
service
resource
tenant
operator
credential subject
```

Each identity class should define its persistence and reuse rules.

## 5. Name

A name is a reference used by humans or software:

```text
nros://cluster/worker/a
```

Names may be stable while their resolved addresses change.

## 6. Name Uniqueness

A naming authority must define the uniqueness scope:

```text
local
cluster
organization
global
```

Global uniqueness should never be assumed merely because an identifier looks globally unique.

## 7. Address

An address identifies a communication location or route:

```text
transport://host:port
```

Addresses may change without changing identity.

```text
Identity X
  ↓ relocation
Address A → Address B
```

## 8. Endpoint

An endpoint combines sufficient information to initiate or receive communication under a protocol contract:

```text
Endpoint
 ├─ identity binding
 ├─ transport
 ├─ address
 ├─ protocol
 └─ policy/context
```

An address alone is not necessarily an authenticated endpoint identity.

## 9. Location

Location describes where an entity currently resides or is reachable.

Location may be:

```text
physical
logical
network
failure-domain
region
availability-zone
```

Location is mutable and should not be used as a permanent identity unless explicitly defined as such.

## 10. Credential

A credential proves or supports an identity claim:

```text
credential
   ↓
binding
   ↓
identity
```

Credentials have their own lifecycle:

```text
issued
 ↓
active
 ↓
rotated
 ↓
revoked / expired
```

## 11. Trust

Trust answers whether a presented identity claim is acceptable under a defined policy.

```text
Identity claim
      ↓
Verification
      ↓
Trust policy
      ↓
Accept / Reject
```

Successful cryptographic verification does not automatically imply authorization.

## 12. Identity Authority

An identity authority may issue or attest identities:

```text
Authority
   ↓ attest
Identity
   ↓ credential
Subject
```

The trust model must define which authorities are accepted and for what scopes.

## 13. Root of Trust

A trust domain should identify its root or roots of trust:

```text
Trust Root
   ↓
Intermediate Authority
   ↓
Identity / Credential
```

Trust roots require lifecycle and compromise-response procedures.

## 14. Key Binding

Cryptographic keys may be bound to identities:

```text
Identity X
    ↕
Key K
```

The binding must specify validity period, purpose, algorithm constraints, and revocation behavior.

## 15. Key Rotation

Identity continuity must survive credential rotation:

```text
Identity X
 ├─ Key K1
 └─ Key K2
```

Rotation should not create an unintended new identity unless explicitly designed to do so.

## 16. Identity Generation

Generation identifiers distinguish incarnations:

```text
Node X generation 4
       ↓ replacement
Node X generation 5
```

This connects identity lifecycle with Part XXVIII resource generations and Part XXIX stale-authority prevention.

## 17. Reincarnation

A restarted or replaced process may reuse a human-readable name while representing a new incarnation.

Therefore:

```text
name continuity
   ≠
process incarnation continuity
```

The protocol should expose generation/incarnation when it matters for correctness.

## 18. Self-Identification

A node or agent should be able to present:

```text
identity
incarnation/generation
supported protocols
capabilities
policy context
```

Claims must be verifiable to the degree required by the trust model.

## 19. Identity Discovery

Discovery can return identity metadata and endpoints:

```text
Query(name/service)
      ↓
Discovery
      ↓
Identity + endpoint candidates
```

Discovery output is information, not automatically proof of trust.

## 20. Discovery Freshness

Discovery results may become stale:

```text
Discovery result
      ↓ time
Current reality
```

Records should carry appropriate freshness, expiry, generation, or version information.

## 21. Stale Discovery

A client must be prepared for:

```text
known identity
but unavailable endpoint
```

or:

```text
known endpoint
but changed identity generation
```

The latter must trigger identity validation rather than silent acceptance.

## 22. Service Discovery

A service name may resolve to multiple endpoints:

```text
Service S
 ├─ Endpoint A
 ├─ Endpoint B
 └─ Endpoint C
```

Selection policy may use:

```text
health
latency
capacity
locality
failure domain
priority
load
```

## 23. Discovery vs Health

Discovery answers where something may be found.

Health answers whether it is currently functioning under a defined health contract.

```text
discovered
  ≠
healthy
```

## 24. Discovery vs Authorization

Finding an endpoint does not grant permission to use it:

```text
Discovery
   ↓
Endpoint candidate
   ↓
Authentication
   ↓
Authorization
```

## 25. Trust Establishment

A connection can establish trust through a defined exchange:

```text
Connect
 ↓
Present identity
 ↓
Verify credential
 ↓
Validate trust chain
 ↓
Check policy
 ↓
Establish session authority
```

The exact protocol is implementation-specific, but the semantic stages should be explicit.

## 26. Mutual Authentication

For mutually trusted communication:

```text
A authenticates B
B authenticates A
```

Both directions should have explicit identity and trust semantics.

## 27. Channel Binding

Where appropriate, identity claims should bind to the communication channel to prevent credential reuse in an unrelated channel.

Conceptually:

```text
Identity
 + Channel context
 + Credential
      ↓
Authenticated session
```

## 28. Session Identity

A communication session may have its own identity:

```text
Node identity
      ↓
Session identity
```

Session identity must not be confused with the long-lived node identity.

## 29. Session Lifecycle

A session may transition through:

```text
Created
 ↓
Authenticating
 ↓
Established
 ↓
Active
 ↓
Rekeying / refreshing
 ↓
Draining
 ↓
Closed
```

## 30. Trust Expiration

Trust may expire because of:

```text
credential expiry
lease expiry
policy change
identity revocation
key compromise
session termination
```

Expiration should prevent future protected operations according to policy.

## 31. Revocation

Identity revocation may use:

```text
revocation lists
status service
short-lived credentials
generation changes
key rotation
authority withdrawal
```

The architecture must define freshness requirements for revocation state.

## 32. Offline Verification

Offline-verifiable credentials can improve availability but introduce revocation freshness tradeoffs.

```text
offline verification
   ↔
revocation freshness
```

The accepted staleness window must be explicit.

## 33. Identity Compromise

If an identity or key is compromised:

```text
Detect
 ↓
Revoke
 ↓
Rotate
 ↓
Invalidate stale sessions/authority
 ↓
Re-establish trust
```

Recovery must prevent the compromised incarnation from reclaiming authority silently.

## 34. Cross-Domain Identity

Two trust domains may represent the same subject differently:

```text
Domain A identity A:X
        ↕ federation
Domain B identity B:Y
```

The mapping must be explicit and scoped.

## 35. Federation

Federation requires:

```text
trust relationship
mapping rules
credential acceptance rules
scope
revocation semantics
failure behavior
```

Federation does not imply unrestricted cross-domain authority.

## 36. Identity Mapping

Mappings should preserve provenance:

```text
External Identity
      ↓ mapping rule
NROS Principal
```

The source authority should remain identifiable for audit and policy decisions.

## 37. Identity and Capabilities

Part XXIX capabilities should bind to identity and generation where necessary:

```text
Identity X
   + generation 5
   ↓
Capability C
```

A capability issued to generation 5 must not silently authorize generation 6 unless explicitly designed to do so.

## 38. Identity and Resources

Part XXVIII resource ownership can reference identity:

```text
Resource R
   ↓ owner
Identity X / generation 5
```

Ownership records should survive address changes but should not survive identity revocation unless policy explicitly permits transfer.

## 39. Identity and Persistence

Part XXVII defines durable identity metadata:

```text
identity
incarnation
key binding
trust metadata
revocation state
```

Recovery must prevent duplicate active incarnations where exclusivity is required.

## 40. Identity and Distributed Coordination

Part XXV requires identity to participate in membership and authority decisions:

```text
Identity
 ↓
Membership
 ↓
Authority / quorum
```

A stale node must not regain distributed authority merely by reconnecting with an old address.

## 41. Identity and Networking

Part XXVI supplies addresses and transport. Part XXX supplies identity binding:

```text
Address
  ↓ connect
Endpoint
  ↓ authenticate
Identity
```

The network location is therefore not the security principal.

## 42. Naming Lifecycle

Names may transition through:

```text
Reserved
 ↓
Bound
 ↓
Published
 ↓
Reassigned / Retired
```

Reassignment should not cause stale clients to confuse an old subject with a new one.

## 43. Name Reuse

If a name can be reused, clients need an identity/generation check:

```text
Name = worker-a
Generation = 12
```

This prevents name reuse from becoming identity confusion.

## 44. Address Migration

A subject may move:

```text
Identity X
 A1 → A2 → A3
```

Clients should follow identity rather than treating address persistence as identity persistence.

## 45. Multi-Address Identity

An identity may advertise multiple addresses:

```text
Identity X
 ├─ A1
 ├─ A2
 └─ A3
```

Each endpoint must remain subject to authentication and authorization policy.

## 46. Observability

Part XIV should expose:

```text
identity
name
incarnation
address
endpoint
credential generation
trust state
policy version
discovery freshness
session state
revocation state
```

Sensitive credentials and secret material must never be emitted as ordinary observability data.

## 47. Formal Identity Binding

A conceptual invariant is:

```text
AuthenticatedEndpoint(E)
    ⇒
Identity(E) is bound to the verified credential
under the active trust policy.
```

## 48. Discovery Safety Invariant

A useful safety property is:

```text
Discovered(E)
    ⇏
Authorized(E)
```

Discovery may produce a candidate; authorization must still be established.

## 49. Incarnation Safety Invariant

```text
Identity(X, generation=g_old)
    ≠
Identity(X, generation=g_new)
```

when the architecture defines generations as distinct authority incarnations.

## 50. Verification Matrix

| Property | Verification question |
|---|---|
| Identity | Is every principal unambiguous within its trust scope? |
| Naming | Is name uniqueness/reuse explicit? |
| Addressing | Can addresses change without changing identity? |
| Endpoint | Is identity bound to communication endpoints? |
| Discovery | Are results freshness-bounded? |
| Health | Is discovery distinguished from health? |
| Authentication | Can presented identities be verified? |
| Trust | Are accepted authorities explicit? |
| Keys | Are key bindings and rotation defined? |
| Revocation | Can compromised identities lose authority? |
| Incarnation | Can stale generations be rejected? |
| Federation | Are cross-domain mappings scoped? |
| Capabilities | Are capabilities bound to the intended identity/generation? |
| Persistence | Can recovery avoid duplicate authority? |
| Distribution | Can stale nodes regain authority? |
| Networking | Is address separated from identity? |
| Observability | Can identity state be reconstructed without exposing secrets? |
| Formal assurance | Are identity-binding invariants explicit? |

## 51. What Part XXX Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a universal identity provider;
- production service discovery;
- complete federation;
- formally verified identity binding;
- universal revocation infrastructure;
- automatic key rotation;
- complete cross-node incarnation fencing;
- production-grade trust establishment.

Those require implementation-specific evidence.

## 52. Transition to Part XXXI

Part XXX defines identity, naming, addressing, discovery, and trust.

Part XXXI should define **protocol sessions, negotiation, version compatibility, feature capability exchange, handshake state, and graceful protocol evolution**, connecting identity establishment to interoperable communication.

```text
Part XXIX
Isolation + capabilities + containment
        ↓
Part XXX
Identity + naming + addressing + discovery + trust
        ↓
Part XXXI
Sessions + negotiation + compatibility + protocol evolution
```

## Canonical rule

> **NROS treats identity as distinct from location: names may remain stable while addresses move, discovery may become stale, credentials may rotate, and incarnations may change; every protected communication path must therefore establish an explicit identity-to-endpoint binding under a defined trust policy, with freshness, revocation, and generation semantics sufficient to reject stale authority.**
