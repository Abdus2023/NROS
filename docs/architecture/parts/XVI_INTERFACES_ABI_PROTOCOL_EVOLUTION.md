# Part XVI — Interfaces, ABI & Protocol Evolution

> **Series:** NROS Architecture Series  
> **Part:** XVI  
> **Role:** APIs, interfaces, ABI boundaries, schemas, protocols, compatibility, negotiation, versioning, and migration  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part XV defined deployment and isolation. Part XVI defines the stable contract boundaries through which NROS components interact and evolve without silently breaking existing consumers.

The central rule is:

> **NROS treats API, ABI, protocol, schema, and behavioral compatibility as distinct contracts; compatibility must be defined at the boundary where it is required.**

## 2. Contract Layers

```text
API
 ↓
Interface
 ↓
ABI
 ↓
Protocol
 ↓
Schema
 ↓
Behavior
```

These layers overlap but are not interchangeable.

## 3. API

An API describes an invocation-facing contract:

```text
operation
arguments
results
errors
lifecycle
```

An API can remain source-compatible while its binary or behavioral contract changes.

## 4. Interface

An interface defines the capabilities exposed by an entity or component.

```text
Interface
├── identity
├── operations
├── events
├── data types
├── errors
└── version
```

Interfaces should expose stable semantics rather than implementation details.

## 5. ABI

An ABI governs binary-level interaction:

```text
calling convention
layout
alignment
symbol names
linkage
representation
error boundary
```

ABI compatibility is platform/toolchain dependent.

```text
API compatible
    ≠
ABI compatible
```

## 6. Protocol

A protocol defines communication behavior between participants.

```text
request
response
acknowledgement
error
state transitions
ordering
timeouts
```

Protocol compatibility includes more than message syntax.

## 7. Schema

A schema defines the structure and constraints of exchanged data.

```text
field
name
 type
required/optional
constraints
version
```

Schema compatibility must consider producers and consumers in both directions where applicable.

## 8. Behavioral Contract

Two implementations can share identical schemas while behaving incompatibly.

Examples:

```text
timeout semantics
ordering
idempotency
retry behavior
side effects
resource usage
error classification
```

Therefore:

```text
Schema compatible
    ≠
Behaviorally compatible
```

## 9. Version Identity

Interfaces and protocols should have explicit version identity.

```text
major.minor
```

The exact numbering scheme is implementation policy, but compatibility rules must be machine- and human-understandable.

## 10. Compatibility Dimensions

NROS distinguishes:

```text
source compatibility
binary compatibility
schema compatibility
wire compatibility
protocol compatibility
behavioral compatibility
configuration compatibility
state compatibility
security compatibility
```

A release can be compatible in one dimension and incompatible in another.

## 11. Backward Compatibility

Backward compatibility asks:

> Can a newer implementation continue serving an older consumer contract?

```text
New server
   ↑
Old client
```

The direction must be stated explicitly.

## 12. Forward Compatibility

Forward compatibility asks whether an older implementation can tolerate newer inputs or extensions.

```text
Old consumer
   ↑
New producer
```

Forward compatibility often requires unknown-field or extension handling.

## 13. Bidirectional Compatibility

Some systems require both:

```text
old ↔ new
```

Others only require compatibility during controlled migration windows.

The required compatibility direction must be part of the release contract.

## 14. Additive Evolution

A generally safer evolution pattern is additive change:

```text
v1
 ↓
add optional field
 ↓
v1-compatible extension
```

However, even additive fields can break consumers that incorrectly reject unknown data.

## 15. Breaking Changes

Potential breaking changes include:

```text
remove operation
rename field
change type
change requiredness
change error semantics
change ordering
change timeout
change side effects
change security requirements
```

A syntactically small change may be behaviorally breaking.

## 16. Negotiation

Participants may negotiate capabilities:

```text
A capabilities
      ↕
B capabilities
      ↓
common contract
```

Negotiation may cover:

```text
protocol version
features
encodings
compression
transport options
security mechanisms
extensions
```

## 17. Capability Discovery

Capability advertisement does not imply authorization.

```text
Supported feature
    ≠
Authorized feature
```

Part XI authorization remains authoritative.

## 18. Feature Flags

Features may be introduced independently of protocol versions:

```text
feature X = supported / unsupported
```

Feature flags require explicit lifecycle and default semantics.

## 19. Optionality

Optional fields and operations require defined behavior when absent.

```text
present → use value
absent  → defined default / omission behavior
```

"Optional" must never mean "undefined behavior."

## 20. Unknown Fields

Consumers may encounter fields introduced by newer producers.

Policy may be:

```text
ignore
preserve
reject
route to extension handler
```

The selected behavior is part of compatibility semantics.

## 21. Extension Points

Protocols should provide controlled extension mechanisms where future evolution is expected.

```text
core contract
   +
extension namespace
```

Extensions must not silently redefine core semantics.

## 22. Deprecation

Deprecation should be explicit:

```text
active
  ↓
deprecated
  ↓
removal scheduled
  ↓
removed
```

A deprecated feature remains governed by its existing contract until removal.

## 23. Migration

Migration may require:

```text
old client
   ↓ adapter
new interface
```

or:

```text
old state
   ↓ migration
new state
```

Migration should identify rollback and failure behavior.

## 24. Adapters

Adapters can preserve compatibility between versions:

```text
Old contract
     ↓
 Adapter
     ↓
New contract
```

Adapters should be observable because they can hide semantic translation and performance costs.

## 25. Protocol State Machines

Protocol behavior should be modeled as explicit state transitions where sequencing matters.

```text
INIT
 ↓ handshake
NEGOTIATED
 ↓ operation
ACTIVE
 ↓ close
CLOSED
```

Messages invalid for the current state should be rejected according to protocol policy.

## 26. Error Contracts

Errors are part of the interface contract.

```text
error code
classification
retryability
context
correlation
```

Changing an error from retryable to terminal can be a behavioral breaking change even when its name is unchanged.

## 27. Idempotency

Operations that may be retried should define idempotency semantics.

```text
request X
request X again
```

must have a defined effect.

Part XIII duplicate-delivery semantics therefore depend on Part XVI operation contracts.

## 28. Timeouts and Retries

Timeout and retry behavior are part of protocol behavior:

```text
request
 ↓ timeout
retry?
 ↓
response
```

Changing retry behavior can change load, side effects, and failure semantics.

## 29. Security Evolution

Protocol evolution must preserve security contracts.

Changes may include:

```text
authentication mechanism
authorization requirements
credential format
cryptographic algorithm
trust domain
permission scope
```

A wire-compatible change that weakens authorization is not acceptable compatibility.

## 30. State Compatibility

Part XII persistence introduces another compatibility boundary:

```text
State schema v1
       ↓
 migration
       ↓
State schema v2
```

A new runtime must explicitly determine whether old state can be read, migrated, rejected, or recovered through another path.

## 31. Deployment Compatibility

Part XV deployment requires compatibility across:

```text
component
runtime
protocol
schema
configuration
state
security policy
transport
```

"Starts successfully" is not sufficient evidence of deployment compatibility.

## 32. ABI Boundaries

ABI boundaries should minimize accidental exposure of unstable implementation details.

Where possible, stable opaque handles or explicit serialization boundaries can reduce coupling.

ABI contracts must define ownership and lifetime rules for memory and resources.

## 33. Memory Ownership

Cross-boundary data ownership must be explicit:

```text
borrowed
owned
shared
transferred
```

Undefined ownership creates correctness and security failures.

## 34. Serialization

Serialization defines representation across a boundary.

The contract should define:

```text
encoding
endianness where relevant
numeric representation
string rules
null/absence semantics
maximum sizes
canonicalization
```

Serialization format alone does not define protocol behavior.

## 35. Size Limits

Interfaces should define bounds where unbounded input could create resource or security risk:

```text
message size
field size
queue depth
batch size
recursion depth
```

Limits should integrate with Part VII resource policies.

## 36. Compatibility Testing

Compatibility should be tested explicitly:

```text
old ↔ new
new ↔ old
old state → new runtime
new state → supported runtime
```

Test matrices should cover supported compatibility windows rather than assuming universal interoperability.

## 37. Contract Testing

Contract tests verify boundary behavior independently of internal implementation.

```text
consumer expectation
        ↓
contract test
        ↓
provider behavior
```

These tests complement unit and integration tests.

## 38. Conformance

A component can claim conformance only against a defined contract version and acceptance criteria.

```text
Implementation
    ↓
Conformance suite
    ↓
Evidence
    ↓
Conformant for contract X
```

Conformance to one version does not imply conformance to all versions.

## 39. Observability Integration

Part XIV should expose interface evolution events:

```text
HandshakeStarted
VersionNegotiated
FeatureNegotiated
CompatibilityRejected
DeprecatedFeatureUsed
SchemaRejected
ProtocolViolation
AdapterActivated
```

These records help explain interoperability failures.

## 40. Verification Matrix

| Property | Verification question |
|---|---|
| API | Are operations and errors explicitly defined? |
| ABI | Are binary layout and ownership rules defined where required? |
| Protocol | Is the state machine explicit? |
| Schema | Are field constraints and versions defined? |
| Behavior | Are timing, ordering, retries, and side effects specified? |
| Compatibility | Is the direction of compatibility explicit? |
| Negotiation | Can participants determine a common supported contract? |
| Extensions | Are extension semantics bounded? |
| Deprecation | Is removal predictable? |
| Migration | Are migration and rollback paths defined? |
| Security | Does evolution preserve authorization and trust requirements? |
| State | Are persisted-state versions compatible or migratable? |
| Testing | Are compatibility matrices executable? |
| Conformance | Is a claim tied to a contract version and evidence? |

## 41. What Part XVI Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- a frozen public ABI;
- universal wire compatibility;
- automatic protocol negotiation;
- automatic schema migration;
- unlimited backward/forward compatibility;
- zero-cost adapters;
- complete conformance suites for every interface.

Those properties require implementation and verification evidence.

## 42. Transition to Part XVII

Part XVI defines contract stability and evolution.

Part XVII should define **configuration and policy orchestration at system scale**, including declarative policy, validation, rollout, dynamic reconfiguration, policy precedence, conflict resolution, and safe application of changes.

```text
Part XV
Deployment + composition + isolation
        ↓
Part XVI
Interfaces + ABI + protocol evolution
        ↓
Part XVII
Configuration + policy orchestration
```

## Canonical rule

> **NROS evolves interfaces through explicit contracts, compatibility dimensions, negotiation, versioning, migration, and conformance evidence; no single notion of “compatibility” is assumed to cover API, ABI, schema, protocol, state, security, and behavior simultaneously.**
