# Part XLIX — API Architecture, Service Boundaries & External Interfaces

> **Series:** NROS Architecture Series  
> **Part:** XLIX  
> **Role:** API architecture, resource models, service boundaries, RPC semantics, lifecycle APIs, streaming, pagination, idempotency, error surfaces, and external interfaces  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XLVIII established the protocol and wire-contract plane. Part XLIX defines the externally consumable service boundary: how clients discover resources, invoke operations, observe state, stream data, handle errors, and safely evolve against NROS.

The central rule is:

> **NROS APIs expose stable semantic contracts rather than internal implementation details: every externally visible operation has explicit resource identity, authorization, consistency, lifecycle, failure, idempotency, and compatibility semantics.**

## 2. API vs Implementation

```text
API
 ≠
internal module
 ≠
transport
 ≠
database schema
```

An API contract may be implemented by many internal components.

## 3. Service Boundary

```text
External Client
      ↓
API Boundary
      ↓
Authorization
      ↓
Protocol
      ↓
Service
      ↓
Runtime / Storage / Scheduler
```

The boundary is a semantic and security boundary, not merely a network endpoint.

## 4. Resource Model

NROS resources should have stable logical identity:

```text
resource_type
resource_id
scope
version
```

Examples may include:

```text
node
agent
workflow
work
execution
queue
schedule
checkpoint
stream
```

Concrete implementations may expose a different subset.

## 5. Resource Identity

Resource IDs must not accidentally encode mutable topology assumptions.

```text
logical_id
 ≠
ephemeral process id
```

## 6. Resource Scope

Resources can be scoped to:

```text
tenant
project
workflow
service
node
cluster
```

Scope is part of authorization and visibility semantics.

## 7. Resource Version

Mutable resources may expose a version or revision:

```text
resource version 41
 ↓ update
resource version 42
```

Clients can use revisions for optimistic concurrency.

## 8. Optimistic Concurrency

A mutation may require:

```text
expected_version = 41
```

If the resource is already version 42:

```text
Conflict
```

This prevents silent lost updates.

## 9. API Operations

Common operations include:

```text
Create
Get
List
Update
Patch
Delete
Execute
Cancel
Pause
Resume
Watch
Stream
```

Each operation requires explicit semantics.

## 10. Create

Create defines:

```text
identity generation
validation
authorization
defaults
initial state
idempotency
commit point
```

## 11. Get

Get defines:

```text
consistency
freshness
authorization
not-found semantics
representation version
```

## 12. List

List operations must be bounded and paginated where collections may grow.

```text
page_size
cursor
next_cursor
```

## 13. Update

Update must define whether it is:

```text
replace
merge
patch
command-like mutation
```

Ambiguous update semantics create compatibility failures.

## 14. Patch

Patch operations should define field-level semantics:

```text
add
replace
remove
append
merge
```

## 15. Delete

Delete must define:

```text
authorization
preconditions
cascade behavior
retention
idempotency
final state
```

## 16. Soft Delete

If resources are retained after deletion:

```text
Active
 ↓
Deleting / Deleted
 ↓
Retained
```

Retention semantics must be explicit.

## 17. Execute

Execution APIs should distinguish:

```text
accepted
running
completed
failed
cancelled
```

An accepted request is not necessarily completed.

## 18. Lifecycle API

Long-running resources should expose observable lifecycle state rather than forcing clients to infer state from transport behavior.

## 19. State Transitions

APIs should define legal transitions:

```text
Created
 ↓
Ready
 ↓
Running
 ↓
Completed
```

Illegal transitions should return structured conflicts/errors.

## 20. Cancellation API

Cancellation should identify the logical operation:

```text
cancel(work_id)
```

and optionally the execution attempt where required.

## 21. Cancellation Semantics

Cancellation should state whether it is:

```text
best-effort
cooperative
preemptive
strong
```

The API must not promise stronger semantics than the runtime provides.

## 22. Pause / Resume

Pause/resume is distinct from cancellation:

```text
Running
 ↓
Paused
 ↓
Running
```

Persistence and resource ownership semantics must be defined.

## 23. RPC

RPC APIs should define:

```text
request
response
errors
timeouts
cancellation
idempotency
authorization
```

## 24. Synchronous RPC

Synchronous operations should have bounded execution expectations.

Long-running work should generally return an operation/resource handle rather than block indefinitely.

## 25. Asynchronous RPC

```text
Request
 ↓
Accepted
 ↓
Operation resource
 ↓
Poll / Watch
 ↓
Completed
```

## 26. Operation Resource

An operation resource may contain:

```text
operation_id
status
progress
result reference
error
created_at
updated_at
```

## 27. Idempotent Create

Create APIs that may be retried should support an idempotency key where duplicate creation is possible.

```text
same idempotency key
 → same logical operation
```

## 28. Idempotency Scope

Idempotency keys must have explicit scope and retention:

```text
client
operation
resource
expiration
```

## 29. Idempotency Conflicts

Reusing an idempotency key with materially different request parameters should produce an explicit conflict rather than a new operation.

## 30. Request Deduplication

Servers may persist request identity sufficiently to survive retry windows where the contract requires it.

## 31. Error Surface

API errors should expose stable machine-readable information:

```text
code
category
message
retryability
request_id
metadata
```

## 32. Error Categories

Possible categories include:

```text
invalid_argument
unauthenticated
permission_denied
not_found
already_exists
conflict
failed_precondition
resource_exhausted
unavailable
deadline_exceeded
cancelled
internal
```

## 33. Error Privacy

Errors must not expose:

```text
secrets
credentials
private payloads
internal stack traces
cross-tenant data
```

unless explicitly authorized.

## 34. Request Identity

Every external request should have a request identity:

```text
request_id
```

This should propagate into internal traces where appropriate.

## 35. Correlation

Distributed API flows should preserve:

```text
request_id
correlation_id
causation_id
trace context
```

## 36. Authorization

API authorization should evaluate:

```text
principal
resource
operation
scope
policy
current authority
```

## 37. Authentication vs Authorization

```text
Authenticated client
    ≠
Authorized operation
```

## 38. Tenant Isolation

A client authorized for tenant A must not infer or enumerate tenant B resources through API behavior.

## 39. Resource Enumeration

List/Get operations must avoid accidental cross-scope disclosure through:

```text
IDs
errors
counts
pagination
sorting
timing
```

## 40. Consistency Contract

Every read API should state its consistency expectation where relevant:

```text
strong
linearizable
eventual
snapshot
stale-allowed
```

## 41. Freshness

If stale reads are permitted, clients should be able to understand freshness bounds where the system can provide them.

## 42. Read-After-Write

APIs should define whether a successful mutation guarantees that subsequent reads observe the new state.

## 43. Conditional Requests

Mutations can use:

```text
If-Match
expected_version
revision
etag
```

where appropriate.

## 44. ETags / Revisions

Representation validators can reduce unnecessary transfers and prevent stale overwrites.

## 45. Pagination

Pagination should be stable under the declared consistency model.

```text
cursor
 ≠
offset
```

Cursor pagination is generally preferable for mutable large collections when supported.

## 46. Cursor Semantics

A cursor should encode enough state to continue from a defined logical position without exposing sensitive implementation details.

## 47. Page Limits

Servers should enforce maximum page sizes.

Client requests larger than the allowed limit should be clamped or rejected according to contract.

## 48. Sorting

List sorting must define:

```text
field
ascending/descending
tie-breaker
null ordering
```

## 49. Filtering

Filtering should use explicit supported fields and operators rather than exposing arbitrary internal query languages by default.

## 50. Search

Search semantics should define whether results are:

```text
exact
prefix
full-text
fuzzy
eventually indexed
```

## 51. Streaming API

Streaming exposes a sequence rather than one response:

```text
Subscribe
 ↓
Event 1
Event 2
Event 3
...
 ↓
Close
```

## 52. Watch API

Watch APIs should define:

```text
initial state
change events
ordering
resume
reconnect
termination
```

## 53. Resume

A reconnecting client may supply a resume token:

```text
resume_token
 ↓
continue from valid position
```

The server must reject expired or invalid positions explicitly.

## 54. Stream Backpressure

Streaming must prevent a slow consumer from causing unbounded memory growth.

```text
producer
 ↓ bounded buffer
consumer
```

## 55. Stream Overflow

Overflow policy should be explicit:

```text
block
shed
terminate
coalesce
checkpoint
```

## 56. Event Loss

If a watch stream is lossy, clients must be able to detect loss and resynchronize.

## 57. Stream Ordering

Ordering guarantees must be scoped:

```text
per resource
per partition
per tenant
global
```

## 58. Batch APIs

Batch operations should define whether execution is:

```text
all-or-nothing
best-effort
partially successful
ordered
parallel
```

## 59. Partial Failure

Batch responses must identify per-item results when partial completion is possible.

## 60. Bulk Operations

Bulk APIs require explicit resource and rate limits to avoid accidental denial of service.

## 61. Rate Limiting

Rate limits may be scoped by:

```text
principal
tenant
endpoint
resource
cluster
```

## 62. Quotas

Quotas define durable or policy-level resource limits:

```text
CPU
memory
storage
work count
stream count
API requests
```

## 63. Retry Guidance

Clients should be able to distinguish:

```text
safe retry
retry after
retry only after reconciliation
never retry
```

## 64. Retry-After

Transient overload or throttling may expose a retry delay or equivalent scheduling hint.

## 65. Deadlines

API requests may carry deadlines propagated internally.

A server must not silently extend a client deadline and claim compliance.

## 66. Timeout vs Cancellation

```text
deadline exceeded
 ≠
explicit cancellation
```

Both should remain distinguishable in the error model.

## 67. Long-Running Work

Long-running execution should use an operation/resource model rather than indefinite request blocking.

## 68. Progress

Progress reporting must define whether values are:

```text
estimated
monotonic
exact
best-effort
```

## 69. Result References

Large results should be returned through references rather than embedding unbounded payloads in control responses.

## 70. Resource Handles

Handles should be opaque and stable across internal implementation changes.

## 71. API Lifecycle

An API itself has lifecycle:

```text
experimental
preview
stable
deprecated
retired
```

## 72. Deprecation

Deprecation should specify:

```text
replacement
announcement
compatibility window
removal criteria
migration guidance
```

## 73. Versioning

API versions should reflect semantic compatibility rather than implementation release numbers alone.

## 74. Version Negotiation

Clients and servers may negotiate supported API versions or explicitly select one.

## 75. Version Pinning

Critical clients may pin versions while migration is validated.

## 76. Forward Compatibility

Servers should tolerate future client behavior only where explicitly permitted by the contract.

## 77. Backward Compatibility

New servers should continue supporting declared older clients during the compatibility window.

## 78. Capability Discovery

Clients may discover optional API capabilities before invoking them.

```text
capabilities
 ↓
client adapts
```

## 79. Feature Gating

Optional features should be explicitly gated rather than inferred from undocumented behavior.

## 80. Documentation Contract

Every public API should document:

```text
purpose
inputs
outputs
errors
authorization
consistency
idempotency
lifecycle
limits
versioning
```

## 81. SDK Boundary

SDKs should remain clients of the API contract rather than becoming the source of truth for semantics.

## 82. CLI Boundary

A CLI should map explicitly onto API operations and preserve structured errors rather than parsing human-readable messages as protocol truth.

## 83. Automation Boundary

Automation clients require stable machine-readable interfaces.

Human-oriented output should not be the canonical automation contract.

## 84. Web / UI Boundary

Interactive interfaces may transform API resources for presentation but should not silently invent authoritative state.

## 85. Webhooks / Push

If NROS provides push callbacks, they require:

```text
authentication
signature/integrity
replay protection
retry policy
ordering
idempotency
```

## 86. Webhook Delivery

Webhook delivery should define:

```text
at-least-once
retry schedule
failure handling
endpoint disablement
```

## 87. External Side Effects

API success must not imply an external side effect succeeded unless the contract explicitly guarantees that relationship.

## 88. Transaction Boundaries

API operations should identify their transaction boundary:

```text
request accepted
state committed
side effect dispatched
side effect confirmed
```

## 89. Saga / Compensation

Distributed API workflows may require compensation rather than rollback:

```text
Step A
 ↓
Step B
 ↓ failure
Compensate B/A as defined
```

## 90. Auditability

Sensitive mutations should produce audit evidence sufficient to reconstruct:

```text
principal
request
resource
policy
result
```

## 91. Observability

API metrics should include:

```text
request count
latency
error rate
retry rate
rate-limit events
stream count
active operations
```

## 92. Health Endpoints

Health APIs should distinguish:

```text
liveness
readiness
dependency health
service health
```

A process being alive does not imply readiness.

## 93. Administrative APIs

Administrative operations require stronger authorization and should be isolated from ordinary workload APIs where appropriate.

## 94. Emergency Control

Emergency APIs may need reserved capacity for:

```text
cancel
fence
isolate
recover
shutdown
```

## 95. Safe Shutdown API

Shutdown should define:

```text
drain
stop admission
finish/cancel work
persist state
release authority
terminate
```

## 96. API Testing

Contract tests should cover:

```text
valid requests
invalid requests
authorization failures
concurrency conflicts
retries
timeouts
cancellation
pagination
stream resume
version negotiation
resource limits
```

## 97. Formal API Authorization Invariant

```text
Accept(Operation)
    ⇒
Authenticated(Principal)
 ∧
Authorized(Principal, Resource, Operation)
```

## 98. Formal Idempotency Invariant

```text
Retry(Request, SameIdempotencyKey)
    ⇒
SameLogicalOperation
```

where the API declares idempotency for that operation.

## 99. Formal Lifecycle Invariant

```text
APIStateTransition
    ⇒
ValidRuntimeTransition
```

The API must not expose states the underlying lifecycle cannot actually represent.

## 100. Formal Consistency Invariant

```text
SuccessfulMutation
    ⇒
DeclaredConsistencyContractSatisfied
```

## 101. Verification Matrix

| Property | Verification question |
|---|---|
| Resource model | Are resource identities stable and scoped? |
| Lifecycle | Are state transitions explicit? |
| Authorization | Is every operation policy-checked? |
| Consistency | Is read/write consistency documented? |
| Concurrency | Are stale updates rejected safely? |
| Idempotency | Can retries converge? |
| Errors | Are machine-readable error semantics stable? |
| Pagination | Are large collections bounded? |
| Streaming | Are ordering, loss, resume, and backpressure explicit? |
| Limits | Are resource and request limits enforced? |
| Rate limits | Can clients understand throttling? |
| Deadlines | Are timeout and cancellation distinct? |
| Async work | Are long-running operations observable? |
| Versioning | Are compatibility windows explicit? |
| Capabilities | Can clients discover optional features? |
| Security | Are sensitive errors and resources isolated? |
| Audit | Can mutations be reconstructed? |
| Shutdown | Can service drain and release authority safely? |
| External effects | Are transaction boundaries explicit? |
| Testing | Are contract and failure cases exercised? |

## 102. What Part XLIX Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a finalized public API;
- stable production resource schemas;
- production SDKs;
- universal REST/gRPC compatibility;
- production webhook infrastructure;
- complete API version negotiation;
- universal exactly-once external effects;
- complete tenant-isolation enforcement across every interface.

Those require implementation-specific evidence.

## 103. Transition to Part L

Part XLIX establishes the external service/API boundary.

Part L should define **security architecture at the API/runtime boundary: identity, authentication, authorization, capabilities, policy evaluation, secret handling, isolation, trust zones, and security invariants**.

```text
Part XLVIII
Protocol + messages + compatibility
        ↓
Part XLIX
API + service boundaries + external interfaces
        ↓
Part L
Identity + authentication + authorization + policy + isolation
```

## Canonical rule

> **NROS APIs expose semantic contracts, not implementation accidents: resource identity, lifecycle, authority, consistency, idempotency, streaming, limits, errors, versioning, and external side effects must remain explicit at every service boundary.**
