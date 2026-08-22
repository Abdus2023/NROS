# Part LVIII — API, RPC, Service Contracts & Boundary Governance

> **Series:** NROS Architecture Series  
> **Part:** LVIII  
> **Role:** API semantics, RPC contracts, request/response lifecycle, validation, authorization boundaries, streaming, errors, compatibility, service composition, and boundary governance  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LVII established the networking substrate. Part LVIII defines the application-facing service boundary carried over that substrate.

The central rule is:

> **An API contract defines semantics, not merely transport syntax; an RPC response, HTTP status, or successful connection must not be treated as proof of domain completion unless the contract says so.**

## 2. Boundary Stack

```text
Network
 ↓
Protocol
 ↓
API / RPC Contract
 ↓
Validation
 ↓
Authorization
 ↓
Admission
 ↓
Execution
 ↓
State / Side Effect
 ↓
Response / Event
```

## 3. API Contract

An API contract defines at minimum:

```text
operation identity
request schema
response schema
error schema
authorization requirements
side-effect semantics
idempotency semantics
timeout / deadline behavior
compatibility policy
```

## 4. API Is Not Transport

```text
API
 ≠
HTTP
 ≠
gRPC
 ≠
TCP
 ≠
Message Queue
```

The same API semantics may be exposed through multiple transports.

## 5. Operation Identity

Each operation should have a stable semantic identity:

```text
service
operation
version
```

## 6. Request Identity

Requests may carry:

```text
request_id
correlation_id
causation_id
idempotency_key
trace context
```

These identities serve different purposes and must not be conflated.

## 7. Request Lifecycle

```text
Receive
 ↓
Decode
 ↓
Authenticate
 ↓
Authorize
 ↓
Validate
 ↓
Admit
 ↓
Execute
 ↓
Commit / Effect
 ↓
Respond / Publish
```

## 8. Decode Failure

Malformed wire data must fail before semantic processing.

```text
Invalid Encoding
    ↓
Protocol Error
```

## 9. Authentication

Authentication establishes the caller identity or authenticated channel context.

It does not establish permission.

## 10. Authorization

Authorization determines whether the authenticated principal may perform the requested operation on the target resource.

```text
Authenticated
    ≠
Authorized
```

## 11. Resource Authorization

Authorization should bind to the actual resource or capability involved rather than merely to the service endpoint.

## 12. Capability Authorization

An API may require a capability:

```text
Required Capability
 ↓
Principal Capability Set
 ↓
Allow / Deny
```

## 13. Validation

Validation should distinguish:

```text
syntactic validity
schema validity
semantic validity
policy validity
resource validity
```

## 14. Validation Ordering

Cheap deterministic validation should occur before expensive execution where possible.

## 15. Unknown Fields

Forward-compatible APIs may permit unknown fields, but semantics must be explicit.

Unknown fields must never silently alter authoritative meaning.

## 16. Defaults

Defaults are part of the API contract.

A client and server must not independently invent incompatible defaults.

## 17. Null vs Missing

The contract should distinguish:

```text
missing
null
empty
zero
false
```

when those values have different semantics.

## 18. Idempotency

Operations should explicitly declare whether they are:

```text
idempotent
non-idempotent
conditionally idempotent
```

## 19. Idempotency Key

For retryable operations with external effects, an idempotency key can bind multiple requests to one logical operation.

## 20. Idempotency Scope

The contract should define whether the key is unique within:

```text
client
principal
service
resource
namespace
time window
```

## 21. Request Replay

A replayed request must produce the behavior defined by its idempotency contract rather than accidentally creating duplicate side effects.

## 22. Read Operations

Reads should specify consistency expectations:

```text
strong
linearizable
snapshot
bounded stale
eventual
```

where relevant.

## 23. Write Operations

Writes should specify the commit semantics visible to the caller.

```text
accepted
committed
durable
published
completed
```

These are distinct states.

## 24. Accepted

`Accepted` means the service has accepted responsibility for processing according to the contract.

It does not necessarily mean completion.

## 25. Committed

`Committed` means the authoritative state mutation reached its declared commit point.

## 26. Durable

`Durable` means the operation satisfies the declared persistence failure model from Part LV.

## 27. Published

`Published` means the associated event/message reached the declared publication boundary.

## 28. Completed

`Completed` means the operation's domain-level completion condition has been satisfied.

## 29. Response Semantics

An API response should state which lifecycle state it represents.

```text
HTTP 200 / RPC OK
```

is not universally equivalent to domain completion.

## 30. Asynchronous Operations

Long-running operations may return:

```text
operation_id
status
accepted_at
tracking endpoint
```

rather than blocking until completion.

## 31. Operation State

An operation may transition through:

```text
accepted
queued
running
committing
completed
failed
cancelled
unknown
reconciling
```

## 32. Operation Identity

The operation identity should remain stable across retries and transport reconnections.

## 33. Cancellation

Cancellation is an operation request with its own authorization and lifecycle semantics.

```text
Running
 ↓ cancel requested
Cancelling
 ↓
Cancelled / Completed / Failed
```

## 34. Cancellation Race

Cancellation may race with completion.

The contract must define which outcome wins or how the final state is represented.

## 35. Deadline

A deadline bounds the caller's expectation of useful completion.

It does not necessarily cancel remote execution unless cancellation propagation is explicitly supported.

## 36. Timeout vs Deadline

```text
Timeout
 → local observation of insufficient progress

Deadline
 → explicit time boundary for an operation contract
```

## 37. Streaming API

Streaming APIs may expose:

```text
request stream
response stream
bidirectional stream
server stream
client stream
```

## 38. Stream Lifecycle

```text
Open
 ↓
Active
 ↓
Half-Closed
 ↓
Completed / Failed / Cancelled
```

## 39. Stream Backpressure

A streaming API must define what happens when the consumer cannot keep up.

Possible semantics:

```text
block
buffer
shed
sample
terminate
```

## 40. Stream Ordering

Ordering must be explicitly defined for streamed items.

## 41. Stream Resume

If stream resumption is supported, the contract should define:

```text
resume token
last acknowledged item
replay range
expiration
```

## 42. Partial Stream Failure

A stream may fail after successfully delivering earlier items.

The consumer must be able to distinguish:

```text
complete stream
partial stream
unknown completion
```

## 43. Pagination

Paginated APIs should define stable traversal semantics.

```text
page token
ordering
snapshot / consistency
expiration
```

## 44. Pagination Consistency

A page token must not silently change the dataset semantics between pages where stable traversal is required.

## 45. Filtering

Filters should have deterministic semantics and documented default behavior.

## 46. Sorting

Sorting should define tie-breakers.

```text
primary_key
secondary_key
stable identifier
```

## 47. Resource Naming

Resource identifiers should have stable semantic identity independent of physical location where mobility is supported.

## 48. Resource Versioning

APIs may expose resource versions or revisions for optimistic concurrency:

```text
Read revision R
 ↓
Write expected revision R
 ↓
Commit if unchanged
```

## 49. Concurrency Failure

A stale write should produce an explicit conflict rather than silently overwriting newer authoritative state when the contract requires optimistic concurrency.

## 50. Preconditions

Requests may carry preconditions:

```text
expected_revision
expected_etag
expected_state
required_capability
```

## 51. Error Model

Errors should be structured rather than encoded only as human-readable strings.

Possible fields:

```text
error_code
category
message
retryability
request_id
operation_id
resource
metadata
cause reference
```

## 52. Error Categories

At minimum, distinguish:

```text
invalid request
unauthenticated
unauthorized
not found
conflict
resource exhausted
unavailable
timeout
cancelled
internal
unknown outcome
```

## 53. Retryability

An error should explicitly indicate whether retry may be safe.

```text
retryable
non-retryable
retry-after
retry-condition
```

## 54. Retry-After

A service may communicate a retry delay, but the client remains subject to local budgets and policy.

## 55. Error Stability

Stable machine-readable error codes should be versioned and governed.

Human-readable messages may change without being treated as API identifiers.

## 56. Unknown Outcome

The API must support cases where the server cannot prove whether a side effect completed:

```text
Request
 ↓
Connection Lost
 ↓
Outcome Unknown
```

The client should reconcile rather than blindly duplicate the operation.

## 57. Error vs Event

An error response describes an operation outcome to a caller.

An event describes an observed fact.

They are not interchangeable.

## 58. Eventual Completion

An asynchronous API may return an operation reference while later publishing completion events.

```text
Submit
 ↓
Operation ID
 ↓
Processing
 ↓
Completion Event
```

## 59. Service Composition

A service may call other services:

```text
Client
 ↓
Service A
 ↓
Service B
 ↓
Service C
```

Each boundary retains its own authorization, timeout, and failure semantics.

## 60. Distributed Transaction

An API composition must not imply atomicity across services unless an explicit distributed transaction or compensating protocol exists.

## 61. Saga / Compensation

Long-running multi-service operations may use:

```text
Step A
 ↓
Step B
 ↓
Step C
```

with compensation where rollback is possible.

## 62. Partial Success

Composite APIs should explicitly represent partial success.

```text
A = success
B = success
C = failure
```

must not be collapsed into an ambiguous boolean.

## 63. Dependency Failure

A service should distinguish:

```text
own failure
upstream failure
downstream failure
policy rejection
resource exhaustion
unknown dependency outcome
```

## 64. Dependency Deadlines

A downstream deadline should remain within the caller's remaining deadline budget.

## 65. Dependency Cancellation

Caller cancellation should propagate to downstream work where supported and authorized.

## 66. Bulkheads

Independent dependencies may require isolated resource pools to prevent one failing dependency from exhausting the entire service.

## 67. Rate Limits

API rate limits should define:

```text
scope
window
quota
burst
response
reset semantics
```

## 68. Quotas

Quotas can apply to:

```text
principal
tenant
service
resource
operation
```

## 69. Admission

Authorization does not guarantee admission.

```text
Authorized
   ↓
Admission Check
   ↓
Accepted / Rejected
```

## 70. Resource Exhaustion

Resource exhaustion should be explicit and distinguishable from authorization failure.

## 71. Priority

Priority may influence admission and scheduling but must remain bounded by policy.

## 72. API Security

APIs should protect against:

```text
request forgery
replay
credential abuse
resource exhaustion
parser abuse
information disclosure
privilege escalation
```

## 73. Input Limits

Contracts should bound:

```text
request bytes
field sizes
list lengths
nesting depth
stream duration
concurrency
```

## 74. Output Limits

Responses should also be bounded or paginated to prevent unbounded resource consumption.

## 75. Sensitive Data

API schemas should classify sensitive fields and define:

```text
access rules
redaction
logging restrictions
retention
```

## 76. Observability

Each request should be correlatable with:

```text
request_id
operation_id
trace_id
principal
resource
outcome
latency
```

without leaking sensitive payloads.

## 77. Audit

Security-sensitive operations should produce audit evidence according to Part LI.

## 78. API Versioning

Versioning may occur at:

```text
major API version
operation version
schema version
feature negotiation
```

## 79. Compatibility

Compatibility should distinguish:

```text
source compatibility
wire compatibility
semantic compatibility
security compatibility
operational compatibility
```

## 80. Breaking Changes

A change is breaking if existing valid clients can no longer rely on previously guaranteed semantics.

Changing field names is not the only form of breaking change.

## 81. Deprecation

Deprecated operations should define:

```text
replacement
warning period
removal policy
migration guidance
```

## 82. Compatibility Windows

When rolling upgrades are required, old and new versions may need to coexist.

Compatibility windows must be explicit.

## 83. Feature Flags

Feature flags may alter behavior, but externally observable semantic changes should remain contract-governed.

## 84. Schema Migration

Schema evolution should preserve or explicitly transform existing resource semantics.

## 85. API Gateway

A gateway may provide:

```text
authentication
routing
rate limiting
protocol translation
observability
```

but should not silently redefine service semantics.

## 86. Service Mesh

A service mesh may provide transport-level concerns while application contracts remain owned by services.

## 87. Boundary Ownership

Every API should have an explicit owner responsible for:

```text
schema
semantics
security
compatibility
availability
migration
```

## 88. Contract Registry

A registry may record:

```text
service
operation
version
schema
owner
status
compatibility
security classification
```

## 89. Contract Evidence

A contract claim should be traceable to:

```text
specification
implementation
tests
CI evidence
runtime evidence
```

## 90. Generated Clients

Generated clients are derived artifacts and must not become the sole source of semantic truth.

The contract remains authoritative.

## 91. Generated Servers

Generated server skeletons similarly do not prove implementation completeness.

## 92. API Documentation

Documentation should expose:

```text
operation
request
response
errors
authorization
side effects
idempotency
limits
examples
compatibility
```

## 93. Example vs Guarantee

An example illustrates behavior.

It is not automatically a normative guarantee.

## 94. Testing

Contract tests should cover:

```text
valid requests
invalid requests
authorization
idempotency
concurrency
timeouts
errors
streaming
compatibility
```

## 95. Negative Testing

APIs should test malformed, unauthorized, oversized, stale, duplicated, and conflicting requests.

## 96. Fault Injection

Service boundaries should be tested under:

```text
timeout
connection loss
partial response
duplicate request
downstream failure
resource exhaustion
```

## 97. Formal Completion Invariant

```text
Response(OK)
    ⇏
DomainCompleted
```

unless the operation contract explicitly defines the equivalence.

## 98. Formal Authorization Invariant

```text
Authenticated(P)
 ∧
Request(O,R)
    ⇒
Allowed(P,O,R)
```

must be established before protected execution.

## 99. Formal Idempotency Invariant

```text
Retry(R)
 ∧
Idempotent(O)
    ⇒
SameLogicalEffect(R)
```

within the declared scope of idempotency.

## 100. Formal Compatibility Invariant

```text
Compatible(Client,Server)
    ⇒
RequiredSemanticContractPreserved
```

## 101. Verification Matrix

| Property | Verification question |
|---|---|
| Contract | Is operation semantics explicitly defined? |
| Identity | Are request/operation/correlation identities distinct? |
| Validation | Are syntax, schema, semantics, and policy validated? |
| Authorization | Is resource-level permission checked? |
| Idempotency | Is retry behavior explicit? |
| Lifecycle | Are accepted/committed/durable/completed states distinct? |
| Async | Can long-running operations be tracked? |
| Cancellation | Are cancellation races defined? |
| Deadlines | Are timeout and deadline semantics distinct? |
| Streaming | Are stream lifecycle and backpressure defined? |
| Errors | Are machine-readable errors stable? |
| Retry | Is retryability explicit? |
| Unknown outcome | Can ambiguous completion be represented? |
| Composition | Are partial successes represented? |
| Limits | Are request/output/resource bounds explicit? |
| Security | Are sensitive fields governed? |
| Observability | Can requests be correlated without leaking secrets? |
| Versioning | Are compatibility guarantees explicit? |
| Ownership | Does every contract have an owner? |
| Evidence | Can contract claims be verified? |

## 102. What Part LVIII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a complete production API gateway;
- universal RPC support;
- complete generated client/server tooling;
- production service composition;
- universal distributed transactions;
- complete API compatibility automation;
- universal stream resumption;
- complete contract registry enforcement;
- production-grade rate limiting across every boundary.

Those require implementation-specific evidence.

## 103. Transition to Part LIX

Part LVIII establishes application-facing service contracts.

Part LIX should define **data models, schemas, serialization, validation, evolution, canonical representations, and compatibility governance**.

```text
Part LVII
Networking + transport + sessions + discovery
        ↓
Part LVIII
APIs + RPC + service contracts + boundary governance
        ↓
Part LIX
Data models + schemas + serialization + evolution
```

## Canonical rule

> **NROS treats an API as a semantic contract spanning identity, authorization, validation, lifecycle, errors, compatibility, and side effects; transport success alone never establishes domain-level completion.**
