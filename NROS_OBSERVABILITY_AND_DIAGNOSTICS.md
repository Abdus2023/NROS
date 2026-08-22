# NROS Observability & Diagnostics (Part XCI–C)

The communication layer established how NROS components exchange information.

The security layer now answers the harder question:

> **Who is allowed to cause which state transition, against which resource, under which conditions, and with what evidence?**

The fundamental model is:

```text
Identity
   ↓
Authentication
   ↓
Authorization
   ↓
Capability
   ↓
Policy Evaluation
   ↓
State Transition
   ↓
Evidence
```

# 1. Security Is Not One Mechanism

NROS security should not be represented by a single `authenticated = true` flag.

Instead:

```text
Identity
+
Authentication
+
Authorization
+
Capability
+
Policy
+
Resource State
+
Context
```

collectively determine whether an operation is permitted.

# 2. Security Principal

A Principal is an entity that can participate in security decisions.

Examples:

```text
Human
Agent
Runtime
Service
Device
Gateway
Plugin
Tenant
System Component
```

Conceptually:

```text
Principal {
    principal_id
    principal_type
    trust_domain
    status
}
```

# 3. Identity

Identity answers:

> Which principal is this?

Example:

```text
agent:nros.worker.17
```

Identity should be stable enough to support:

```text
authorization
audit
ownership
accountability
revocation
```

# 4. Authentication

Authentication establishes evidence that a principal controls an identity credential.

Possible mechanisms:

```text
public-key credentials
certificates
signed tokens
hardware-backed credentials
operator credentials
```

The mechanism should be explicit.

# 5. Authentication ≠ Authorization

This remains a fundamental invariant:

```text
Authenticated
     ↓
Known identity

Authorized
     ↓
Permitted operation
```

Therefore:

```text
authenticated = true
```

must never imply:

```text
authorized_for_everything = true
```

# 6. Trust Domain

NROS deployments may contain different trust domains:

```text
Control Plane
Data Plane
Agent Sandbox
External Network
Operator Domain
Device Domain
```

A principal trusted in one domain does not automatically become trusted everywhere.

# 7. Trust Boundary

A Trust Boundary is where assumptions about identity, integrity, or authority change.

Example:

```text
┌──────────────────────┐
│ Trusted NROS Runtime │
└──────────┬───────────┘
           │
       TRUST BOUNDARY
           │
┌──────────▼───────────┐
│ External Agent       │
└──────────────────────┘
```

Every boundary needs explicit validation.

# 8. Security Boundary

Security boundaries may exist between:

```text
processes
containers
VMs
hosts
tenants
networks
plugins
agents
operators
external systems
```

# 9. Root of Trust

A deployment needs an initial trust anchor.

Conceptually:

```text
Trust Root
    ↓
Identity
    ↓
Credential
    ↓
Authenticated Principal
```

The root must be protected more strongly than ordinary credentials.

# 10. Cryptographic Identity

A principal can be represented by a public/private key pair:

```text
Private Key
    │
    └── proves control

Public Key
    │
    └── identifies principal
```

The private key must never be transmitted as ordinary application data.

# 11. Credential

A credential proves possession of an identity or capability.

Examples:

```text
certificate
signed assertion
capability token
session credential
```

Credentials should have:

```text
issuer
subject
scope
expiration
status
```

where appropriate.

# 12. Credential Expiration

Credentials should not live indefinitely by default.

```text
VALID
  ↓
EXPIRING
  ↓
EXPIRED
```

Expiration limits the damage caused by credential compromise.

# 13. Credential Revocation

A credential can become invalid before expiration:

```text
VALID
  ↓
REVOKED
```

Reasons might include:

```text
compromise
decommissioning
policy violation
identity retirement
security incident
```

# 14. Key Rotation

Keys should be replaceable without destroying identity continuity.

```text
Key K1
  ↓ rotation
Key K2
```

The system must define:

```text
overlap period
validation rules
revocation
migration
```

# 15. Secrets

Secrets include:

```text
private keys
API credentials
tokens
passwords
encryption keys
connection credentials
```

Secrets must not be treated as ordinary configuration.

# 16. Secret Exposure

The following are dangerous:

```text
logs
event payloads
debug output
error messages
tracebacks
metrics labels
Git history
source code
```

Secrets must be deliberately excluded or redacted.

# 17. Secret Store

A secure architecture should separate:

```text
Application
    ↓
Secret interface
    ↓
Secret provider
```

rather than embedding secrets directly into source code.

# 18. Secret Lifecycle

```text
GENERATE
   ↓
STORE
   ↓
RETRIEVE
   ↓
USE
   ↓
ROTATE
   ↓
REVOKE
   ↓
DESTROY
```

Each transition should have explicit semantics.

# 19. Capability Security

NROS should strongly consider capability-oriented authorization.

Instead of merely saying:

```text
Agent A is an administrator.
```

provide narrowly scoped authority:

```text
Capability:
    resource = R1
    operation = read
    expires = T
```

# 20. Capability

A capability can conceptually contain:

```text
Capability {
    capability_id
    issuer
    subject
    resource
    operations
    constraints
    issued_at
    expires_at
}
```

# 21. Least Privilege

The Agent should receive only what it needs.

Bad:

```text
Agent A
    → ALL_RESOURCES
    → ALL_OPERATIONS
```

Better:

```text
Agent A
    → resource:R17
    → operation:read
    → duration:10m
```

# 22. Scope

Capabilities should be scoped by:

```text
resource
operation
time
environment
tenant
network
data classification
```

as appropriate.

# 23. Capability Delegation

A capability may sometimes be delegated:

```text
Authority
    ↓
Agent A
    ↓
Agent B
```

But delegation must not increase authority.

# 24. Non-Amplification

If:

```text
A has X
```

then A may delegate:

```text
X
```

or a subset of X.

It must not create:

```text
X + Y
```

without explicit authority to do so.

# 25. Delegation Depth

Deep delegation chains can become difficult to reason about:

```text
A → B → C → D → E
```

Therefore NROS may impose:

```text
max_delegation_depth
```

or require stronger authorization for delegation.

# 26. Capability Revocation

Capability-based systems must support revocation when necessary.

Possible strategies:

```text
short expiration
revocation registry
epoch invalidation
resource-side validation
key rotation
```

# 27. Epoch-Based Revocation

Suppose:

```text
Capability epoch = 20
```

Then the resource advances to:

```text
epoch = 21
```

All capabilities bound to epoch 20 become invalid.

This can efficiently invalidate groups of credentials.

# 28. Policy

Authorization should ultimately be evaluated through explicit policy.

Conceptually:

```text
authorize(
    principal,
    action,
    resource,
    context
)
```

returns:

```text
ALLOW
DENY
```

or a richer decision.

# 29. Policy Context

The decision may depend on:

```text
identity
resource
operation
time
location
network
work_id
tenant
risk
credential
system state
```

Only attributes actually required by policy should be collected.

# 30. Policy Evaluation

Example:

```text
Agent A
   ↓
WRITE Resource R1
   ↓
Policy
   ├─ identity valid
   ├─ capability valid
   ├─ resource scope matches
   ├─ lease active
   └─ system state permits
        ↓
      ALLOW
```

# 31. Deny by Default

When authorization information is incomplete:

```text
UNKNOWN
```

should normally not become:

```text
ALLOW
```

For privileged operations:

```text
UNKNOWN → DENY
```

unless an explicit safe-degradation policy exists.

# 32. Policy Version

Every security-sensitive decision should be attributable to a policy version:

```text
policy_version = P42
```

This enables later reconstruction.

# 33. Policy Activation

A new policy should not become active merely because a file changed.

A robust lifecycle is:

```text
PROPOSED
   ↓
VALIDATED
   ↓
AUTHORIZED
   ↓
ACTIVATED
   ↓
EFFECTIVE
```

# 34. Policy Rollback

If policy P43 introduces an error:

```text
P43
 ↓
invalid behavior
 ↓
rollback
 ↓
P42
```

Rollback itself should be governed and auditable.

# 35. Security State

Security should have explicit state:

```text
NORMAL
DEGRADED
SUSPECTED
QUARANTINED
LOCKDOWN
RECOVERY
```

# 36. Quarantine

A suspicious principal can be isolated:

```text
Agent A
   ↓
SUSPICIOUS
   ↓
QUARANTINED
```

Quarantine may disable:

```text
new assignments
privileged commands
resource access
delegation
external communication
```

while preserving evidence collection.

# 37. Lockdown

Lockdown is stronger than quarantine.

Possible behavior:

```text
ALLOW:
    emergency safety operations

DENY:
    ordinary mutations
```

This should be explicit rather than improvised.

# 38. Privilege Separation

A component should not possess every privilege needed by the entire system.

Example:

```text
Scheduler
    → scheduling authority

Storage
    → persistence authority

Network Gateway
    → network authority

Agent Sandbox
    → execution authority
```

This limits compromise impact.

# 39. Process Isolation

Potential boundaries:

```text
Agent
 ↓
Sandbox
 ↓
Runtime
 ↓
Host
```

An Agent should not automatically inherit host privileges.

# 40. Tool Isolation

Agent tools should be separately authorized:

```text
Agent
  ↓
Tool Request
  ↓
Tool Policy
  ↓
ALLOW / DENY
```

A tool should not inherit unrestricted Agent authority.

# 41. Tool Capability

Example:

```text
Agent A
    Capability:
        filesystem.read
        /workspace/project-x/*
```

This is safer than:

```text
filesystem.read/*
```

# 42. Resource Isolation

Isolation may apply to:

```text
CPU
memory
filesystem
network
devices
IPC
environment variables
credentials
```

# 43. Tenant Isolation

If NROS supports multiple tenants:

```text
Tenant A
    ├── Agents
    ├── Work
    └── Resources

Tenant B
    ├── Agents
    ├── Work
    └── Resources
```

cross-tenant access must be explicitly authorized.

# 44. Cross-Tenant Operation

A cross-tenant operation should require explicit policy:

```text
Tenant A → Resource owned by Tenant B
```

Default:

```text
DENY
```

unless an approved delegation exists.

# 45. Network Security

Network policy should constrain:

```text
source
destination
port
protocol
identity
direction
```

# 46. Egress Control

Agent execution environments should not automatically have unrestricted outbound access.

Possible policy:

```text
Agent Sandbox
    ↓
allowed:
    api.example
    artifact.example

denied:
    everything else
```

# 47. Ingress Control

Likewise, inbound traffic should be explicitly exposed.

An internal Agent should not become publicly reachable merely because a process opened a socket.

# 48. Secure Boot / Integrity

Where hardware/platform support exists, NROS may establish a chain:

```text
Boot Root
   ↓
Verified Runtime
   ↓
Verified Components
   ↓
Trusted Execution
```

# 49. Attestation

Attestation can provide evidence that a runtime is executing an expected software configuration.

Conceptually:

```text
Runtime
   ↓
Attestation Evidence
   ↓
Verifier
   ↓
TRUST / REJECT
```

# 50. Attestation ≠ Authorization

A verified runtime is not automatically authorized for every operation.

Instead:

```text
Attestation
    +
Identity
    +
Policy
    ↓
Authorization decision
```

# 51. Artifact Integrity

Executable artifacts should be identifiable by:

```text
digest
signature
version
build provenance
```

# 52. Supply Chain

NROS itself depends on:

```text
source repositories
dependencies
build tools
compilers
containers
artifacts
plugins
```

Each can introduce supply-chain risk.

# 53. Dependency Integrity

Dependency resolution should be reproducible where possible:

```text
dependency
    ↓
exact version
    ↓
verified checksum
    ↓
reproducible build
```

# 54. Artifact Signing

Release artifacts can be signed:

```text
Artifact
   ↓
Digest
   ↓
Signature
   ↓
Verification
```

A signature proves authenticity relative to its trust root, not that the artifact is inherently safe.

# 55. Build Provenance

A release should ideally record:

```text
source revision
builder
toolchain
dependencies
build configuration
artifact digest
```

# 56. Security Audit Event

Security-relevant actions should generate events such as:

```text
AuthenticationSucceeded
AuthenticationFailed
AuthorizationDenied
CapabilityIssued
CapabilityRevoked
PolicyActivated
CredentialRotated
AgentQuarantined
ResourceAccessDenied
```

# 57. Audit Event

An audit event should answer:

```text
WHO?
WHAT?
WHICH RESOURCE?
WHEN?
UNDER WHICH POLICY?
WITH WHICH CREDENTIAL?
WHAT WAS THE RESULT?
```

# 58. Security Evidence

Evidence should preserve enough information to reconstruct the decision without unnecessarily storing secrets.

For example:

```text
principal_id
resource_id
action
policy_version
decision
capability_id
timestamp
correlation_id
```

but not:

```text
private_key
raw_password
secret_token
```

# 59. Security Logging

Security logs themselves become sensitive resources.

Therefore:

```text
Log access
    ↓
Authorization
```

must also be enforced.

# 60. Tamper Evidence

Security evidence should preferably be tamper-evident.

Possible mechanisms:

```text
hash chaining
signed records
append-only storage
immutable snapshots
external anchoring
```

# 61. Hash Chain

Example:

```text
E1 → hash H1
E2 → hash(H1 + E2)
E3 → hash(H2 + E3)
```

Modification of an earlier event breaks the chain.

# 62. Incident State

Security incidents should have lifecycle:

```text
DETECTED
   ↓
TRIAGED
   ↓
CONTAINED
   ↓
INVESTIGATED
   ↓
RECOVERED
   ↓
CLOSED
```

# 63. Security Detection

Detection may originate from:

```text
authentication anomalies
policy violations
unexpected network activity
resource abuse
integrity failures
repeated authorization failures
agent behavior anomalies
```

# 64. Automated Response

A policy may automatically:

```text
revoke capability
terminate session
quarantine Agent
fence resource
disable endpoint
increase logging
```

Automated response itself must be governed.

# 65. Security Recovery

Recovery should not simply restore the previous state.

It must establish:

```text
trusted identity
trusted software
trusted credentials
trusted policy
trusted resource ownership
```

before returning to normal operation.

# 66. Compromised Agent Recovery

Example:

```text
Agent A
   ↓
compromise suspected
   ↓
quarantine
   ↓
revoke capabilities
   ↓
terminate sessions
   ↓
fence resources
   ↓
collect evidence
   ↓
rebuild / re-attest
   ↓
new identity or re-enrollment
```

# 67. Security and Distributed Ownership

Security interacts directly with the previous coordination layer.

Suppose:

```text
Agent A owns Resource R
```

and A becomes compromised.

Security must trigger:

```text
capability revocation
+
lease revocation
+
fencing
```

Otherwise a compromised Agent may continue acting after logical removal.

# 68. Security and Communication

Likewise:

```text
Connection
   ↓
Authentication
   ↓
Authorization
   ↓
Message Validation
   ↓
Protocol Handling
```

Security therefore sits across the entire communication path.

# 69. Security and Scheduling

A quarantined Agent must not remain eligible merely because the scheduler has stale state.

Correct sequence:

```text
Security State
      ↓
Eligibility
      ↓
Scheduling
```

not:

```text
Scheduling
      ↓
Security check afterward
```

for operations requiring pre-authorization.

# 70. Security Invariants

```text
1. Authentication does not imply authorization.

2. Presence does not imply trust.

3. Trust in one domain does not imply trust in another.

4. Privileges are explicit.

5. Least privilege is the default design principle.

6. Capabilities are scoped.

7. Delegation cannot amplify authority.

8. Sensitive credentials expire or rotate according to policy.

9. Revocation can invalidate authority before expiration.

10. Secrets are never treated as ordinary telemetry.

11. Private credentials never enter ordinary logs.

12. Authorization decisions are attributable to a policy version.

13. Denied operations remain observable where appropriate.

14. Quarantined Agents cannot retain unrestricted authority.

15. Revoking an Agent must also address existing leases.

16. Fencing prevents stale authority from affecting protected resources.

17. Tool access is independently authorized.

18. Tenant boundaries are explicit.

19. Network access is explicitly governed.

20. External inputs cross a validation boundary.

21. Artifact integrity is independently verifiable.

22. Build provenance is distinct from artifact authenticity.

23. Attestation does not automatically grant authorization.

24. Security logs are themselves protected resources.

25. Security evidence is tamper-evident where required.

26. Incident response is stateful and auditable.

27. Recovery establishes trust before restoring authority.

28. Unknown security state does not silently become trusted state.

29. Security mechanisms never create hidden alternative authorities.

30. Every privileged state transition has a verifiable security basis.
```

# 71. Unified Security Flow

```text
                     PRINCIPAL
                         │
                         ↓
                  ┌─────────────┐
                  │ IDENTITY    │
                  └──────┬──────┘
                         ↓
                  AUTHENTICATION
                         │
                         ↓
                  CREDENTIAL CHECK
                         │
                         ↓
                   CAPABILITY
                         │
                         ↓
                 POLICY EVALUATION
                         │
                 ┌───────┴───────┐
                 ↓               ↓
               DENY             ALLOW
                 │               │
                 ↓               ↓
               AUDIT        RESOURCE CHECK
                                 │
                                 ↓
                              LEASE
                                 │
                                 ↓
                              FENCE
                                 │
                                 ↓
                           STATE CHANGE
                                 │
                                 ↓
                              EVIDENCE
```

# 72. The Security Principle

The central NROS rule becomes:

> **Authority is a stateful, scoped, verifiable property—not an implicit consequence of identity, connectivity, process ownership, or Agent status.**

This distinction prevents a large class of distributed-system failures.

# Part XCII — Observability, Telemetry, Tracing, Metrics, Diagnostics & Explainability

The next subsystem follows naturally.

NROS now has:

```text
Governance
Evidence
State
Recovery
Coordination
Communication
Security
```

But a system of this complexity must answer:

> **What is happening right now, why is it happening, and how can we prove what happened afterward?**

The next layer will formalize:

```text
Logs
Metrics
Traces
Events
Spans
Correlation
Causation
State snapshots
Health
Liveness
Readiness
Diagnostics
Profiling
Resource telemetry
Agent telemetry
Scheduler telemetry
Security telemetry
Protocol telemetry
Queue telemetry
Latency
Throughput
Error rates
Saturation
SLOs
SLIs
Alerts
Anomaly detection
Explainability
Decision provenance
Replay
Forensics
Operational dashboards
Debug mode
Diagnostic bundles
```

with the key principle:

> **Observability must expose the system's behavior without becoming an uncontrolled second source of truth.**

# NROS — Part XCII: Observability, Telemetry, Tracing, Diagnostics & Explainability

The previous layer established **identity, authorization, capabilities, isolation, secrets, revocation, quarantine, and security evidence**.

Now we need to make NROS **observable**.

The fundamental distinction is:

```text
State
   ↓
what the system IS

Events
   ↓
what happened

Metrics
   ↓
how much / how often

Logs
   ↓
what was reported

Traces
   ↓
how an operation propagated

Evidence
   ↓
what can be independently established
```

Observability describes the system.

It must not secretly become another state machine.

# 1. Observability Model

A useful NROS observability model is:

```text
                    NROS
                     │
       ┌─────────────┼─────────────┐
       ↓             ↓             ↓
     Events        Metrics        Logs
       │             │             │
       └─────────────┼─────────────┘
                     ↓
                  Traces
                     │
                     ↓
                Diagnostics
                     │
                     ↓
              Evidence / Audit
```

# 2. Observability vs Evidence

These are related but different.

Observability asks:

> What appears to be happening?

Evidence asks:

> What can we establish actually happened?

For example:

```text
Metric:
    scheduler.success_rate = 98%

Evidence:
    Assignment A-102 committed
    Attempt T-901 completed
```

A metric is an aggregate.

Evidence is attributable state/history.

# 3. Event

An Event represents an occurrence:

```text
WorkCreated
AssignmentCreated
LeaseGranted
ExecutionStarted
ExecutionCompleted
PolicyDenied
AgentQuarantined
```

Events should have stable identities.

# 4. Event Envelope

Conceptually:

```text
Event {
    event_id
    event_type

    timestamp
    sequence

    subject
    actor

    correlation_id
    causation_id

    state_version
    payload

    schema_version
}
```

# 5. Event Time

Distributed systems contain multiple relevant times:

```text
created_at
observed_at
received_at
committed_at
processed_at
```

These must not be silently conflated.

# 6. Event Ordering

An event can have:

```text
local sequence
partition sequence
causal predecessor
```

but this does not necessarily imply global ordering.

# 7. Event Source

Every event should identify its source:

```text
scheduler
runtime
agent
security subsystem
storage subsystem
gateway
external integration
```

# 8. Actor vs Subject

These should be distinct.

Example:

```text
Actor:
    Agent A

Subject:
    Resource R

Action:
    RELEASE
```

This allows the system to answer:

> Who changed what?

# 9. Metrics

Metrics represent measurements over time.

Examples:

```text
scheduler_queue_depth
active_agents
lease_count
execution_latency
error_rate
memory_usage
cpu_usage
```

# 10. Counter

A Counter increases monotonically:

```text
commands_total
failures_total
authorization_denials_total
```

Counters should not normally be manually decremented.

# 11. Gauge

A Gauge represents current state:

```text
active_agents
queue_depth
memory_usage
current_leases
```

It can increase or decrease.

# 12. Histogram

A Histogram represents a distribution:

```text
execution_duration
request_latency
queue_wait_time
```

This is more useful than merely recording an average.

# 13. Percentiles

Operational latency often needs:

```text
p50
p90
p95
p99
```

rather than only:

```text
average
```

because tail latency can reveal serious degradation.

# 14. Metric Dimensions

Metrics can be labeled by:

```text
agent
work_type
resource
tenant
operation
status
```

But excessive dimensions create cardinality explosions.

# 15. Cardinality

Avoid metrics such as:

```text
request_latency{message_id="every-unique-ID"}
```

because the number of time series can become enormous.

Prefer bounded dimensions:

```text
request_latency{operation="execute",status="success"}
```

# 16. High-Cardinality Data

Unique IDs belong more naturally in:

```text
logs
traces
events
evidence
```

rather than general-purpose metrics.

# 17. Log

A Log is a diagnostic record.

Example:

```text
2026-08-21T...
scheduler:
    assignment A-42 selected agent-7
```

Logs are primarily for humans and diagnostics.

# 18. Structured Logs

NROS should prefer structured records:

```text
{
    level,
    timestamp,
    component,
    event,
    correlation_id,
    fields
}
```

rather than unstructured strings alone.

# 19. Log Levels

Possible levels:

```text
TRACE
DEBUG
INFO
WARN
ERROR
FATAL
```

The exact taxonomy should be standardized.

# 20. Logs Are Not State

A log saying:

```text
"lease granted"
```

does not itself establish authoritative ownership.

Authoritative state remains in:

```text
lease state machine
```

The log merely reports it.

# 21. Trace

A Trace represents an end-to-end operation.

Example:

```text
Client Request
      ↓
Gateway
      ↓
Authorization
      ↓
Scheduler
      ↓
Lease
      ↓
Agent
      ↓
Tool
      ↓
Resource
```

One operation can therefore span many components.

# 22. Span

Each stage can become a Span:

```text
Trace T1
 ├─ Gateway
 ├─ Auth
 ├─ Scheduler
 ├─ Lease
 ├─ Agent
 └─ Tool
```

# 23. Span Metadata

A span can include:

```text
trace_id
span_id
parent_span_id
start_time
end_time
status
component
attributes
```

# 24. Trace Context Propagation

When a message crosses components:

```text
A
 ↓
B
 ↓
C
```

the trace context should propagate.

Thus:

```text
Trace T1
    ├─ Span A
    ├─ Span B
    └─ Span C
```

rather than producing unrelated traces.

# 25. Correlation

Correlation connects:

```text
request
event
log
span
attempt
resource
```

through shared identifiers.

Example:

```text
correlation_id = C-700
```

# 26. Causation

Causation answers:

> Why did this event happen?

Example:

```text
Command C1
    ↓
Transition T1
    ↓
Event E1
    ↓
Assignment A1
```

This forms a causal graph.

# 27. Decision Trace

Security and scheduling decisions should be explainable:

```text
Work W
 ↓
Candidate Agents
 ↓
Eligibility filters
 ↓
Policy filters
 ↓
Resource matching
 ↓
Agent A selected
```

This is much more useful than:

```text
Agent A selected.
```

# 28. Explainability

NROS should be able to answer:

```text
Why was this Agent selected?
Why was this operation denied?
Why was this Work delayed?
Why was this resource unavailable?
Why was this Agent quarantined?
Why was this Work retried?
```

# 29. Explainability Record

Conceptually:

```text
DecisionExplanation {
    decision_id
    decision_type

    inputs
    rules_evaluated
    constraints
    selected_action
    rejected_alternatives

    policy_version
    scheduler_version
}
```

# 30. Rejected Alternatives

For important decisions, preserving rejected candidates is valuable.

Example:

```text
Agent A → selected

Agent B → rejected: insufficient capability
Agent C → rejected: resource unavailable
Agent D → rejected: policy denied
```

This makes scheduling behavior auditable.

# 31. Health

Health answers:

> Is this component functioning sufficiently?

Health should be multi-dimensional.

Possible state:

```text
HEALTHY
DEGRADED
UNHEALTHY
UNKNOWN
```

# 32. Liveness

Liveness asks:

> Is the component alive/responding?

A component can be:

```text
alive
but unhealthy
```

# 33. Readiness

Readiness asks:

> Can the component currently accept work?

Therefore:

```text
Liveness ≠ Readiness
```

# 34. Example

```text
Runtime:
    alive = true
    ready = false
```

because it is:

```text
replaying state
recovering storage
loading policy
```

# 35. Startup State

NROS startup should be observable:

```text
INITIALIZING
 ↓
LOADING_STATE
 ↓
VALIDATING
 ↓
RECOVERING
 ↓
READY
```

# 36. Recovery Observability

Recovery must expose:

```text
recovery_started
snapshot_loaded
journal_replayed
inconsistencies_detected
reconciliation_started
reconciliation_completed
```

# 37. Scheduler Telemetry

Scheduler metrics can include:

```text
queue_depth
queue_wait_time
assignment_rate
assignment_failure_rate
starvation_count
preemption_count
reassignment_count
```

# 38. Resource Telemetry

Resources can expose:

```text
capacity
allocated
reserved
available
utilization
saturation
```

# 39. Agent Telemetry

Agent-level telemetry may include:

```text
state
active_work
success_count
failure_count
latency
resource_consumption
heartbeat_age
```

# 40. Lease Telemetry

Track:

```text
active_leases
lease_acquisition_latency
renewal_failures
expired_leases
revocations
fencing_events
```

# 41. Security Telemetry

Security metrics include:

```text
authentication_failures
authorization_denials
credential_expirations
revocations
quarantines
policy_evaluations
integrity_failures
```

# 42. Communication Telemetry

Track:

```text
messages_sent
messages_received
message_errors
retries
timeouts
connection_failures
bytes_in
bytes_out
stream_count
```

# 43. Queue Telemetry

Important signals:

```text
queue_depth
oldest_item_age
enqueue_rate
dequeue_rate
rejection_rate
dead_letter_rate
```

The most useful signal may be:

```text
oldest_item_age
```

because queue depth alone does not reveal starvation.

# 44. Saturation

The classic operational question is:

> Which resource is preventing additional work?

Possible saturation points:

```text
CPU
memory
storage
network
scheduler
queue
database
external API
GPU
```

NROS should expose the bottleneck.

# 45. SLI

A Service Level Indicator measures an operational property.

Examples:

```text
successful execution ratio
p99 scheduling latency
availability
queue wait time
```

# 46. SLO

An SLO establishes a target.

Example:

```text
99.9% of eligible Work
should receive assignment
within defined latency.
```

The exact value should be policy/configuration rather than hard-coded architecture.

# 47. Error Budget

If an SLO is defined, the remaining tolerance can be represented as:

```text
error_budget
```

This can guide operational decisions.

# 48. Alert

An alert is an actionable condition.

Bad:

```text
CPU = 70%
```

Better:

```text
Scheduler queue oldest item
has exceeded operational threshold.
```

# 49. Alert Severity

Possible levels:

```text
INFO
WARNING
CRITICAL
EMERGENCY
```

Severity should reflect operational consequences, not merely numeric magnitude.

# 50. Alert Deduplication

A persistent failure should not produce:

```text
100,000 identical alerts
```

NROS should group repeated alerts.

# 51. Alert Correlation

Multiple symptoms may represent one root cause:

```text
network failure
   ↓
agent disconnect
   ↓
lease expiration
   ↓
work reassignment
   ↓
queue growth
```

Observability should help correlate these into one incident.

# 52. Anomaly Detection

Anomaly detection can identify deviations from expected behavior:

```text
normal:
    20 ms queue latency

observed:
    5000 ms
```

But anomaly detection should produce:

```text
SUSPECTED_ANOMALY
```

not automatically:

```text
CERTAIN_FAILURE
```

# 53. Diagnostic Bundle

When an incident occurs, NROS can create a bounded diagnostic bundle containing:

```text
configuration snapshot
relevant logs
metrics window
trace references
state snapshot
security events
coordination state
software versions
policy versions
```

Sensitive fields must be redacted.

# 54. Redaction

Diagnostics must remove:

```text
private keys
passwords
tokens
session secrets
unnecessary personal data
```

before export.

# 55. Diagnostic Snapshot

A snapshot can capture:

```text
runtime state
active agents
leases
queues
policies
connections
resource utilization
```

at a specific point.

# 56. Snapshot ≠ Evidence

A diagnostic snapshot is observational.

It may help explain what was happening but does not automatically establish a committed historical fact.

# 57. Historical Reconstruction

For serious incidents:

```text
Events
+
State snapshots
+
Traces
+
Security audit
+
Logs
```

can reconstruct the sequence of events.

# 58. Replay

If deterministic state transitions are available, NROS may replay historical input:

```text
Initial State
   ↓
Event 1
   ↓
Event 2
   ↓
Event 3
   ↓
Reconstructed State
```

This is valuable for debugging.

# 59. Deterministic Replay

Replay becomes substantially more reliable if the runtime records:

```text
input
ordering
random seeds
time assumptions
external responses
policy versions
software versions
```

# 60. External Effects

Pure replay cannot reproduce arbitrary external effects.

Therefore NROS should distinguish:

```text
deterministic internal state
```

from:

```text
external side effects
```

# 61. Effect Recording

For replayable workflows, external responses can be recorded as evidence:

```text
External Call
    ↓
Response
    ↓
Recorded Artifact
```

Replay then uses the recorded response instead of contacting the external system.

# 62. Forensics

Forensics requires preserving:

```text
who
what
when
where
why
sequence
evidence
integrity
```

without altering original evidence.

# 63. Evidence Retention

Retention should be policy-driven:

```text
short-lived telemetry
longer-lived audit
critical security evidence
```

Different classes should have different retention requirements.

# 64. Observability Cost

Observability consumes:

```text
CPU
memory
storage
network bandwidth
```

Therefore:

```text
maximum diagnostic detail
```

cannot always be enabled permanently.

# 65. Adaptive Diagnostics

NROS may increase diagnostics during incidents:

```text
NORMAL
 ↓ anomaly
ENHANCED
 ↓ incident
FORENSIC
 ↓ recovery
NORMAL
```

This should itself be observable.

# 66. Sampling

High-volume traces may require sampling:

```text
100% critical operations
10% normal operations
1% low-value telemetry
```

Critical events should not be sampled away.

# 67. Tail Sampling

The system can retain traces selectively based on outcome:

```text
successful request → maybe sample
failed request → retain
slow request → retain
security event → retain
```

# 68. Privacy

Observability data can expose sensitive information.

Therefore:

```text
Observability
    ↓
Data classification
    ↓
Redaction
    ↓
Access control
```

must be part of the architecture.

# 69. Telemetry Isolation

Telemetry should not be able to mutate authoritative runtime state merely because it can observe it.

For example:

```text
Metrics Collector
    ✕
cannot directly modify
scheduler state
```

# 70. Observability Feedback

Automated controllers may consume telemetry:

```text
Metric
  ↓
Controller
  ↓
Decision
```

When this happens, the feedback loop must be explicitly modeled.

Otherwise observability becomes hidden control logic.

# 71. Control Loop

Example:

```text
CPU saturation
      ↓
Telemetry
      ↓
Autoscaling policy
      ↓
New Agent
      ↓
Scheduler
```

This is no longer passive observability.

It is an autonomous control loop and must be governed accordingly.

# 72. Diagnostic Commands

NROS may expose commands such as:

```text
status
agents
work
leases
queues
health
metrics
traces
policy
security
recovery
```

Each command remains subject to authorization.

# 73. Explainable Status

Instead of:

```text
Agent A: unavailable
```

NROS should ideally report:

```text
Agent A:
    state = SUSPECTED
    last_heartbeat = ...
    active_lease = L42
    lease_expiry = ...
    reason = heartbeat_timeout
    fencing_state = pending
```

# 74. System Status Model

A high-level status could be:

```text
SystemStatus {
    lifecycle
    health
    readiness
    security_state
    coordination_state
    storage_state
    scheduler_state
    active_incidents
}
```

# 75. Health Aggregation

Overall health should not hide subsystem failures.

For example:

```text
Overall: DEGRADED

Scheduler: HEALTHY
Storage: HEALTHY
Security: HEALTHY
Network: DEGRADED
Agent Pool: DEGRADED
```

# 76. Unknown State

If health cannot be determined:

```text
UNKNOWN
```

should remain distinguishable from:

```text
HEALTHY
```

This prevents false confidence.

# 77. Observability Invariants

```text
1. Observability is not authoritative state.

2. Logs do not replace durable state.

3. Metrics do not prove historical facts.

4. Diagnostic snapshots do not automatically establish truth.

5. Evidence and telemetry have distinct semantics.

6. Every important operation can be correlated across components.

7. Causation is preserved where required.

8. Event identity is stable.

9. Metric cardinality is bounded.

10. High-cardinality identifiers belong in appropriate diagnostic channels.

11. Security-sensitive events are retained according to policy.

12. Secrets never appear in ordinary telemetry.

13. Diagnostic exports are redacted.

14. Health, liveness, and readiness are distinct.

15. UNKNOWN is not silently converted to HEALTHY.

16. Scheduler decisions are explainable.

17. Security decisions are attributable to policy versions.

18. Queue starvation is observable.

19. Resource saturation is observable.

20. Distributed traces preserve cross-component context.

21. Timeouts remain distinguishable from confirmed failures.

22. Replay requires explicit handling of external effects.

23. Sampling cannot silently discard mandatory security evidence.

24. Observability feedback loops are explicitly governed.

25. Telemetry systems cannot become hidden state authorities.

26. Diagnostic collection itself is authorized.

27. Incident reconstruction preserves evidence integrity.

28. Retention is policy-driven.

29. Observability overhead is bounded.

30. Every critical state transition has sufficient diagnostic context to explain it.
```

# 78. Unified NROS Observability Architecture

```text
                         NROS RUNTIME
                              │
        ┌─────────────────────┼──────────────────────┐
        ↓                     ↓                      ↓
      EVENTS               METRICS                 LOGS
        │                     │                      │
        └─────────────────────┼──────────────────────┘
                              ↓
                           TRACES
                              │
                     ┌────────┴────────┐
                     ↓                 ↓
                DIAGNOSTICS        ALERTING
                     │                 │
                     ↓                 ↓
                 FORENSICS         INCIDENT
                     │
                     ↓
                  EVIDENCE
                     │
                     ↓
              HISTORICAL REPLAY
```

# 79. The Observability Principle

The key rule is:

> **NROS must make authoritative decisions explainable without allowing observability artifacts to become authoritative state themselves.**

This keeps the architecture clean:

```text
State       → authoritative
Evidence    → historical proof
Telemetry   → measurement
Logs        → diagnostic narrative
Traces      → causal execution view
```

# Part XCIII — Configuration, Runtime Parameters, Feature Flags, Dynamic Policy & System Reconfiguration

The next major layer is **configuration**.

NROS has now accumulated many configurable dimensions:

```text
scheduler policy
resource limits
security policy
lease duration
timeouts
retry policy
queue limits
observability levels
network policy
agent capabilities
feature availability
```

The dangerous question is:

> **How can configuration change without creating an uncontrolled state transition?**

The next layer will formalize:

```text
Configuration
Configuration Schema
Defaults
Overrides
Profiles
Environment
Static Configuration
Dynamic Configuration
Runtime Configuration
Feature Flags
Policy References
Versioning
Validation
Compatibility
Atomic Activation
Staged Rollout
Canary Configuration
Rollback
Configuration Snapshots
Configuration Provenance
Secret References
Immutable Configuration
Mutable Configuration
Configuration Drift
Desired State
Observed State
Reconciliation
Configuration Locking
Change Authorization
Configuration Audit
Failure Semantics
Safe Defaults
Emergency Configuration
```

with the central principle:

> **Configuration is executable intent only after it has been validated, authorized, versioned, activated, and reconciled against the runtime state.**

# NROS — Part XCIII: Configuration, Runtime Parameters, Feature Flags & Reconfiguration

The previous layer established observability.

We now need to formalize **configuration as controlled intent**.

The critical distinction is:

```text
Configuration
    ↓
declares desired behavior

Runtime State
    ↓
represents actual behavior

Reconciliation
    ↓
brings actual state toward authorized desired state
```

Configuration must therefore **never silently become an uncontrolled mutation channel**.

# 1. Configuration Model

A configuration can be represented as:

```text
Configuration {
    config_id
    schema_version
    revision
    scope
    values
    source
    provenance
    policy
}
```

# 2. Configuration Sources

NROS may receive configuration from:

```text
built-in defaults
configuration files
environment variables
CLI arguments
deployment manifests
control plane
remote configuration service
operator commands
policy engine
```

These sources must have deterministic precedence.

# 3. Configuration Precedence

For example:

```text
Defaults
   ↓
System Config
   ↓
Deployment Config
   ↓
Environment
   ↓
Runtime Override
```

The exact hierarchy should be explicitly specified.

Never leave precedence to accidental implementation order.

# 4. Configuration Schema

Every configuration surface should have a schema defining:

```text
field name
type
required/optional
default
constraints
allowed values
security classification
mutability
```

Example:

```text
scheduler.max_concurrency:
    type = integer
    minimum = 1
    maximum = 1024
    mutable = runtime
```

# 5. Configuration Validation

Validation should occur before activation:

```text
Candidate Configuration
        ↓
Schema Validation
        ↓
Semantic Validation
        ↓
Security Validation
        ↓
Dependency Validation
        ↓
ACTIVATABLE / REJECTED
```

# 6. Syntax vs Semantics

These are different.

Syntactic validation:

```text
max_workers = "100"
```

might fail because the type is incorrect.

Semantic validation:

```text
max_workers = 100
```

may still fail because:

```text
max_workers > available_resource_limit
```

# 7. Cross-Field Validation

Configuration fields can depend on each other.

Example:

```text
min_workers <= max_workers
```

or:

```text
lease_timeout > heartbeat_interval
```

These constraints belong to semantic validation.

# 8. Configuration Dependency Graph

A configuration may affect multiple subsystems:

```text
config.scheduler.max_concurrency
          │
          ├── Scheduler
          ├── Queue
          ├── Agent Pool
          └── Resource Accounting
```

Therefore changing one field can have distributed consequences.

# 9. Immutable Configuration

Some configuration should be immutable after startup.

Examples may include:

```text
identity root
cryptographic trust anchors
storage format
fundamental protocol version
```

Changing these may require a restart or explicit migration.

# 10. Mutable Configuration

Other configuration can safely change during runtime:

```text
log level
sampling rate
queue thresholds
non-critical scheduling parameters
diagnostic verbosity
```

Each field should explicitly declare its mutability.

# 11. Configuration Mutability Classes

A useful classification:

```text
STATIC
RESTART_REQUIRED
DYNAMIC
HOT_RELOADABLE
EMERGENCY_ONLY
```

# 12. Dynamic Configuration

A dynamic change should follow:

```text
REQUEST
   ↓
AUTHENTICATE
   ↓
AUTHORIZE
   ↓
VALIDATE
   ↓
PREPARE
   ↓
ACTIVATE
   ↓
VERIFY
   ↓
COMMIT
```

# 13. Configuration Transaction

Configuration activation should ideally be atomic from the system's perspective.

Avoid:

```text
Field A updated
Field B updated
Field C failed
```

leaving an incoherent partial configuration.

Prefer:

```text
Old Config
    ↓
Candidate Config
    ↓
Validation
    ↓
Atomic Activation
```

# 14. Configuration Revision

Every activation should create a revision:

```text
Config R41
   ↓
Config R42
```

The revision should be immutable once committed.

# 15. Configuration History

NROS should retain enough history to answer:

```text
What configuration was active?
When?
Who changed it?
Why?
From which revision?
To which revision?
Was activation successful?
```

# 16. Configuration Provenance

A revision should identify its origin:

```text
operator
deployment
automation
policy engine
recovery
bootstrap
```

# 17. Configuration Author

The actor changing configuration must be identifiable.

Example:

```text
actor = operator:admin
```

or:

```text
actor = controller:auto-scaler
```

# 18. Change Reason

Important configuration changes should include a reason:

```text
reason = "reduce scheduler saturation"
```

This improves operational explainability.

# 19. Configuration Diff

Every change should have a deterministic diff:

```text
scheduler.max_concurrency:
    32 → 64

scheduler.queue_limit:
    1000 → 2000
```

# 20. Configuration Audit

Configuration changes should emit auditable events:

```text
ConfigurationProposed
ConfigurationValidated
ConfigurationRejected
ConfigurationActivated
ConfigurationRolledBack
```

# 21. Configuration Rollback

Rollback should reference a known revision:

```text
R42
 ↓ failure
R41
```

not reconstruct configuration manually from memory.

# 22. Rollback Safety

Rollback itself can be unsafe.

For example:

```text
R41
 ↓
resource migration
 ↓
R42
```

If R42 changed the storage format, simply reverting to R41 may not be valid.

Therefore rollback requires compatibility validation.

# 23. Configuration Compatibility

For each revision:

```text
compatible_with_runtime = true/false
```

should be determined explicitly.

# 24. Staged Activation

Large changes should support stages:

```text
PROPOSED
   ↓
VALIDATED
   ↓
STAGED
   ↓
CANARY
   ↓
ROLLED_OUT
   ↓
ACTIVE
```

# 25. Canary Configuration

A new configuration can initially apply to:

```text
one Agent
one scheduler partition
one tenant
one resource group
```

before global activation.

# 26. Canary Verification

After canary activation:

```text
metrics
errors
latency
resource utilization
security events
```

can determine whether rollout continues.

# 27. Automatic Rollback

A rollout controller may detect:

```text
error_rate > threshold
```

and execute:

```text
ROLLBACK
```

But automatic rollback must itself be governed by explicit policy.

# 28. Configuration Lock

Some configuration changes should be temporarily prevented:

```text
configuration_lock = active
```

Possible reasons:

```text
incident
migration
upgrade
recovery
maintenance
```

# 29. Emergency Configuration

Emergency configuration exists for incident response.

Example:

```text
disable_external_egress = true
```

But emergency configuration should be:

```text
authorized
audited
time-bounded
reversible
```

# 30. Time-Bounded Overrides

An override can include:

```text
expires_at
```

Example:

```text
log_level = DEBUG
expires = 30 minutes
```

This prevents temporary operational changes from becoming permanent configuration drift.

# 31. Feature Flags

Feature flags control optional behavior:

```text
feature_x = enabled
```

But feature flags should not become an alternative policy system.

They need:

```text
owner
scope
version
expiration
authorization
```

# 32. Feature Flag Scope

A flag may target:

```text
global
tenant
agent
work type
resource group
deployment
```

Scope must be explicit.

# 33. Feature Flag Lifecycle

```text
DEFINED
 ↓
DISABLED
 ↓
CANARY
 ↓
ENABLED
 ↓
DEPRECATED
 ↓
REMOVED
```

A flag should not remain indefinitely.

# 34. Flag Debt

Long-lived flags create hidden complexity.

Example:

```text
if feature_a
    ...
else
    ...
```

after years of historical compatibility.

Therefore feature flags require ownership and retirement criteria.

# 35. Configuration vs Policy

These are different.

Configuration:

```text
scheduler.max_concurrency = 32
```

Policy:

```text
Agents without capability X
must not execute operation Y.
```

Configuration defines system parameters.

Policy defines allowed behavior.

# 36. Configuration vs Secrets

Secrets should not be embedded directly:

```text
api_key = "..."
```

Prefer:

```text
api_key = secret_ref("provider/key")
```

The secret value remains outside ordinary configuration state.

# 37. Secret References

A configuration may contain:

```text
SecretReference {
    provider
    secret_id
    version
}
```

instead of the secret itself.

# 38. Environment Variables

Environment variables are convenient but dangerous for sensitive configuration because they can leak through:

```text
process inspection
debugging
crash reports
child processes
diagnostic bundles
```

Sensitive values require explicit handling.

# 39. Configuration Encryption

Encryption at rest protects stored configuration, but does not solve authorization.

The complete model is:

```text
encrypted
+
integrity protected
+
access controlled
+
audited
```

# 40. Configuration Integrity

Configuration should be protected against unauthorized modification.

Possible mechanism:

```text
Config
  ↓
Digest
  ↓
Signature / integrity metadata
```

# 41. Desired State

NROS can distinguish:

```text
Desired State
```

from:

```text
Observed State
```

Example:

```text
desired_agents = 10
observed_agents = 7
```

# 42. Reconciliation

A controller can reconcile:

```text
Desired State
      ↓
Compare
      ↓
Observed State
      ↓
Required Actions
      ↓
Apply
      ↓
Verify
```

# 43. Configuration Drift

Drift occurs when:

```text
desired configuration
        ≠
actual runtime configuration
```

Example:

```text
desired max_workers = 32
runtime max_workers = 16
```

# 44. Drift Detection

The system should detect and report:

```text
CONFIGURATION_DRIFT
```

rather than silently assuming the runtime matches configuration.

# 45. Drift Resolution

Possible strategies:

```text
reconcile automatically
reject runtime mutation
alert operator
freeze affected component
```

The strategy must be policy-defined.

# 46. Configuration Ownership

Every configuration domain should have an owner:

```text
scheduler config → Scheduler authority
security config → Security authority
storage config → Storage authority
```

Avoid a universal configuration writer.

# 47. Configuration Authority

A configuration change must be authorized against the specific domain.

For example:

```text
Operator A
    scheduler: ALLOW
    security: DENY
```

# 48. Configuration Isolation

Subsystems should receive only configuration relevant to them.

Avoid:

```text
every component
    ↓
entire global configuration object
```

Prefer:

```text
Scheduler
    ↓
SchedulerConfig

Storage
    ↓
StorageConfig
```

# 49. Configuration Distribution

In distributed NROS deployments:

```text
Control Plane
      ↓
Config Revision R42
      ↓
Node A
Node B
Node C
```

Nodes may receive the same revision independently.

# 50. Configuration Acknowledgement

Each node should report:

```text
received
validated
activated
rejected
```

for the revision.

# 51. Partial Rollout

A distributed rollout may temporarily look like:

```text
Node A → R42
Node B → R42
Node C → R41
```

This must be an explicit rollout state, not an invisible inconsistency.

# 52. Configuration Convergence

The system should define whether it requires:

```text
eventual convergence
```

or:

```text
synchronous global activation
```

for a given configuration class.

# 53. Safety-Critical Configuration

Some changes may require stronger guarantees.

Example:

```text
security policy
resource isolation
protocol compatibility
```

These may require coordinated activation.

# 54. Configuration Barriers

A barrier can prevent incompatible revisions from coexisting:

```text
R41
 ↓
prepare
 ↓
all required nodes acknowledge
 ↓
activate R42
```

# 55. Version Skew

During upgrades:

```text
Node A → version 10
Node B → version 11
```

configuration schema compatibility must be explicit.

# 56. Schema Versioning

Configuration should identify its schema:

```text
schema_version = 4
```

Migration:

```text
Schema 3
   ↓
migration
   ↓
Schema 4
```

# 57. Migration

Migration should be deterministic and testable.

It should not silently reinterpret old configuration.

# 58. Unknown Fields

NROS should define behavior for unknown configuration fields:

```text
REJECT
WARN
IGNORE
```

For security-sensitive settings, rejection is generally safer.

# 59. Defaults

Defaults must be explicit and documented.

A missing value should not produce an accidental runtime behavior.

# 60. Safe Defaults

Where possible, defaults should minimize damage:

```text
network access → restricted
privilege → minimal
diagnostic exposure → bounded
resource consumption → bounded
```

# 61. Configuration Failure

If configuration activation fails:

```text
Candidate R42
    ↓
validation failure
    ↓
R41 remains active
```

The system should not enter an undefined intermediate configuration.

# 62. Fail-Safe vs Fail-Open

Different configuration domains require different semantics.

Security:

```text
unknown authorization policy
→ DENY
```

Observability:

```text
metrics exporter unavailable
→ runtime may continue
```

The failure policy must therefore be domain-specific.

# 63. Configuration Transaction State

A useful state machine:

```text
DRAFT
  ↓
PROPOSED
  ↓
VALIDATING
  ↓
VALIDATED
  ↓
AUTHORIZED
  ↓
STAGED
  ↓
ACTIVATING
  ↓
ACTIVE
```

Failure paths:

```text
VALIDATING → REJECTED
AUTHORIZED → CANCELLED
ACTIVATING → FAILED
ACTIVE → ROLLED_BACK
```

# 64. Configuration Explainability

NROS should answer:

```text
Why is this value active?
```

For example:

```text
scheduler.max_concurrency = 32

source:
    deployment profile

revision:
    R42

activated:
    2026-08-21T...

actor:
    deployment-controller

previous:
    16
```

# 65. Configuration Security Invariants

```text
1. Configuration is not automatically authoritative merely because it exists.

2. Configuration must be schema-valid before activation.

3. Semantic validation is distinct from syntax validation.

4. Security-sensitive configuration requires authorization.

5. Every committed revision is identifiable.

6. Configuration history is auditable.

7. Activation is atomic where required.

8. Partial activation is explicitly represented.

9. Rollback is validated for compatibility.

10. Temporary overrides are time-bounded where appropriate.

11. Secrets are referenced rather than exposed in ordinary configuration.

12. Configuration and policy remain semantically distinct.

13. Feature flags have lifecycle ownership.

14. Feature flags cannot silently replace authorization policy.

15. Desired state and observed state remain distinct.

16. Configuration drift is observable.

17. Configuration ownership is explicit.

18. Components receive only configuration relevant to their authority.

19. Unknown configuration fields have deterministic semantics.

20. Defaults are explicit.

21. Safe defaults are used where practical.

22. Configuration migration is deterministic.

23. Version skew is explicitly handled.

24. Emergency configuration is audited.

25. Automatic rollback is policy-controlled.

26. Configuration changes are explainable.

27. Failed configuration activation preserves the last valid state.

28. Configuration distribution is observable.

29. Configuration state cannot bypass security authorization.

30. Configuration cannot become a hidden second control plane.
```

# 66. Unified Configuration Architecture

```text
                    CONFIG SOURCE
                         │
                         ↓
                  CONFIG SCHEMA
                         │
                         ↓
                    VALIDATION
                  ┌──────┴──────┐
                  ↓             ↓
                REJECT       VALIDATE
                                │
                                ↓
                         AUTHORIZATION
                                │
                                ↓
                           REVISION Rn
                                │
                         ┌──────┴──────┐
                         ↓             ↓
                       STAGE         CANARY
                         │             │
                         └──────┬──────┘
                                ↓
                            ACTIVATE
                                │
                                ↓
                         OBSERVED STATE
                                │
                                ↓
                           RECONCILE
                                │
                    ┌───────────┴───────────┐
                    ↓                       ↓
                 CONVERGED               DRIFT
                    │                       │
                    ↓                       ↓
                  ACTIVE                REMEDIATE
```

# 67. Core Configuration Principle

The governing rule is:

> **Configuration represents authorized desired state; activation converts that intent into runtime state through validation, controlled transition, and observable reconciliation.**

This gives NROS a clean separation:

```text
Configuration → Desired State
Runtime       → Observed State
Controller    → Reconciliation
Policy        → Authority
Evidence      → Proof
Telemetry     → Visibility
```

# Part XCIV — Resource Model, Capacity, Allocation, Reservations, Quotas & Accounting

The next layer is the **resource model**.

NROS cannot schedule Agents or Work correctly without formally defining what a resource actually is.

We therefore need to model:

```text
Resource identity
Resource type
Capacity
Availability
Ownership
Allocation
Reservation
Quota
Usage
Utilization
Saturation
Overcommitment
Admission control
Accounting
Resource lifecycle
Resource health
Resource locality
Resource affinity
Resource constraints
Resource fragmentation
Resource reclamation
Resource fencing
Resource delegation
Resource pools
Resource hierarchies
```

The central question becomes:

> **What exactly does it mean for an Agent to have access to a resource, and how is that authority represented, measured, reserved, consumed, released, and verified?**

The next section should establish the formal resource state machine before moving into advanced scheduling algorithms.

# NROS — Part XCIV: Resource Model, Capacity, Allocation, Reservations, Quotas & Accounting

The configuration layer established **desired state**.

The resource layer establishes the physical/logical substrate against which NROS makes scheduling decisions.

The central distinction is:

```text
Resource
   ↓
what exists

Capacity
   ↓
how much can be provided

Availability
   ↓
how much can currently be used

Reservation
   ↓
what has been promised

Allocation
   ↓
what has been assigned

Usage
   ↓
what is actually being consumed
```

These concepts must not be collapsed into one number.

# 1. Resource

A Resource is anything whose availability constrains execution.

Examples:

```text
CPU
Memory
GPU
Storage
Network bandwidth
Device
Port
Database connection
API quota
License
Human operator
Agent slot
Execution environment
```

# 2. Resource Identity

Every managed resource needs a stable identity:

```text
Resource {
    resource_id
    resource_type
    owner
    provider
    location
    lifecycle_state
}
```

Example:

```text
resource:node-07/cpu
resource:node-07/memory
resource:gpu-02
```

# 3. Resource Type

The type determines the semantics of capacity and consumption.

Examples:

```text
cpu
memory
gpu
storage
network
device
slot
quota
external-service
```

Different resource types require different accounting rules.

# 4. Resource Lifecycle

A resource should have explicit lifecycle states:

```text
DISCOVERING
   ↓
REGISTERED
   ↓
AVAILABLE
   ↓
ALLOCATED
   ↓
DEGRADED
   ↓
UNAVAILABLE
   ↓
DRAINING
   ↓
RETIRED
```

# 5. Resource Discovery

NROS may discover resources from:

```text
local host
container runtime
cluster scheduler
hardware inventory
cloud provider
external service
operator declaration
```

Discovery produces **observed information**, not automatic authority.

# 6. Registration

A discovered resource becomes managed only after registration:

```text
Discovery
   ↓
Validation
   ↓
Registration
   ↓
Resource ID
```

# 7. Capacity

Capacity describes the maximum usable quantity under defined conditions.

Examples:

```text
CPU = 8 cores
Memory = 16 GiB
Storage = 500 GiB
GPU = 1
```

# 8. Capacity Is Not Availability

Suppose:

```text
CPU capacity = 8
```

and:

```text
CPU allocated = 6
```

then nominal availability is:

```text
2
```

But if 1 core is reserved for system overhead:

```text
effective availability = 1
```

# 9. Effective Capacity

A useful model:

```text
effective_capacity
=
physical_capacity
-
system_reserve
-
unavailable_capacity
```

# 10. Allocation

Allocation represents an actual assignment:

```text
Resource R
    ↓
Allocation A
    ↓
Agent X
```

Allocation creates authority to consume the resource.

# 11. Reservation

A reservation is a promise to preserve capacity for future use.

```text
Resource
   ↓
Reserved
   ↓
Later Allocated
```

Reservation therefore differs from current consumption.

# 12. Reservation Example

```text
GPU capacity = 4

allocated = 2
reserved = 1

available for new reservations = 1
```

# 13. Double Counting Prevention

The resource accounting model must ensure:

```text
allocated + reserved
≤
effective capacity
```

unless explicit overcommitment is enabled.

# 14. Usage

Usage measures actual consumption.

Example:

```text
allocation = 4 CPU
actual usage = 2.1 CPU
```

Allocation and usage therefore differ.

# 15. Utilization

Utilization can be represented as:

```text
usage / effective_capacity
```

but the precise formula depends on resource type.

For CPU:

```text
CPU utilization
```

is meaningful.

For a discrete device:

```text
GPU utilization
```

may need a different interpretation.

# 16. Discrete Resources

Some resources are indivisible:

```text
GPU
USB device
network interface
hardware accelerator
license seat
```

Allocation is generally:

```text
0 or 1
```

for a single instance.

# 17. Quantitative Resources

Other resources are divisible:

```text
CPU
memory
storage
bandwidth
```

These support quantities:

```text
CPU = 2.5 cores
Memory = 4 GiB
Bandwidth = 100 Mbps
```

where supported.

# 18. Scalar Resources

A scalar resource can be represented as:

```text
ResourceQuantity {
    capacity
    allocated
    reserved
    available
}
```

# 19. Vector Resources

An Agent may require multiple resources simultaneously:

```text
CPU = 2
Memory = 4 GiB
GPU = 1
Storage = 20 GiB
```

This becomes a resource vector:

```text
R = (CPU, Memory, GPU, Storage)
```

# 20. Multi-Resource Admission

A Work item is admissible only if all required dimensions can be satisfied.

```text
requested:
    CPU  = 2
    RAM  = 4 GiB
    GPU  = 1

available:
    CPU  = 4
    RAM  = 8 GiB
    GPU  = 1

→ admissible
```

# 21. Fragmentation

A resource pool may have sufficient total capacity but insufficient contiguous or compatible capacity.

Example:

```text
Node A:
    free CPU = 1
    free RAM = 1 GiB

Node B:
    free CPU = 1
    free RAM = 1 GiB
```

Total:

```text
CPU = 2
RAM = 2 GiB
```

but a Work requiring both on the same node may not fit.

# 22. Locality

Resources have locations:

```text
host
rack
zone
region
device
process
container
```

Location can influence scheduling.

# 23. Resource Affinity

A Work may prefer:

```text
same host
same zone
same GPU
same storage
```

Affinity expresses preference.

# 24. Resource Anti-Affinity

A Work may require separation:

```text
Agent A ≠ same host as Agent B
```

This improves fault isolation.

# 25. Hard vs Soft Constraints

Hard constraint:

```text
must have GPU
```

Soft preference:

```text
prefer GPU in same zone
```

The scheduler must distinguish them.

# 26. Resource Ownership

A resource may have an owner:

```text
owner = tenant:A
```

Ownership does not necessarily mean exclusive usage.

It defines authority boundaries.

# 27. Resource Delegation

An owner can delegate resource usage:

```text
Owner
   ↓
Capability
   ↓
Agent
```

The delegation must remain within the owner's authority.

# 28. Quota

A quota limits how much a principal may consume.

Example:

```text
Tenant A:
    CPU quota = 32
    GPU quota = 4
    Memory quota = 128 GiB
```

# 29. Quota vs Capacity

These are distinct:

```text
Capacity
    ↓
what infrastructure provides

Quota
    ↓
what a principal may consume
```

A system may have:

```text
capacity = 100 CPU
quota = 20 CPU
```

for one tenant.

# 30. Quota Hierarchy

Quotas can be hierarchical:

```text
Organization
   ↓
Tenant
   ↓
Project
   ↓
Agent
   ↓
Work
```

# 31. Quota Inheritance

If a child inherits quota:

```text
Tenant quota = 100 CPU
Project A = 40 CPU
Project B = 60 CPU
```

then:

```text
A + B ≤ 100
```

unless explicit overcommitment exists.

# 32. Quota Enforcement

Quota should be checked before allocation:

```text
Request
 ↓
Resource Availability
 ↓
Quota Check
 ↓
Policy Check
 ↓
Allocation
```

# 33. Reservation vs Quota

A reservation says:

> capacity is promised.

A quota says:

> maximum authority to consume.

Therefore:

```text
Reservation ≠ Quota
```

# 34. Admission Control

Admission control decides whether a new Work may enter the execution system.

Inputs may include:

```text
resource availability
quota
priority
security
policy
health
capacity
```

# 35. Admission Pipeline

```text
Work Request
     ↓
Authentication
     ↓
Authorization
     ↓
Quota
     ↓
Resource Constraints
     ↓
Capacity
     ↓
Scheduler
     ↓
Admitted / Rejected
```

# 36. Overcommitment

Some resources can intentionally be overcommitted.

Example:

```text
CPU capacity = 8
allocated logical CPU = 12
```

This can work when workloads are not simultaneously saturated.

But overcommitment must be explicit.

# 37. Overcommit Ratio

Example:

```text
logical allocation / physical capacity
```

For:

```text
12 / 8 = 1.5
```

the overcommit ratio is:

```text
1.5×
```

# 38. Overcommit Risk

Overcommitment can cause:

```text
contention
latency spikes
starvation
thrashing
OOM
deadline failures
```

Therefore it must be observable.

# 39. Guaranteed Resources

Some Work may require guaranteed capacity:

```text
CPU guarantee = 2
Memory guarantee = 4 GiB
```

The scheduler must not allocate that capacity elsewhere if the guarantee is active.

# 40. Best-Effort Resources

Other Work may accept opportunistic capacity:

```text
priority = low
resource_class = best_effort
```

These workloads can be displaced.

# 41. Resource Classes

A useful classification:

```text
GUARANTEED
BURSTABLE
BEST_EFFORT
EXCLUSIVE
SHARED
```

# 42. Exclusive Allocation

For a device:

```text
GPU-1
   ↓
Agent A
```

Agent B cannot simultaneously claim it unless the device explicitly supports sharing.

# 43. Shared Allocation

For a network link:

```text
Bandwidth = 1 Gbps

Agent A → 300 Mbps
Agent B → 200 Mbps
Agent C → 100 Mbps
```

Remaining:

```text
400 Mbps
```

# 44. Resource Accounting

NROS should maintain accounting records:

```text
Resource R
 ├─ Capacity
 ├─ Reserved
 ├─ Allocated
 ├─ Used
 └─ Available
```

# 45. Accounting Invariant

For non-overcommitted resources:

```text
allocated + reserved
≤
effective_capacity
```

and:

```text
usage
≤
allocation
```

unless burst semantics explicitly permit otherwise.

# 46. Usage Violations

If:

```text
usage > allocation
```

the system should distinguish:

```text
BURST_ALLOWED
```

from:

```text
RESOURCE_VIOLATION
```

# 47. Resource Metering

Metering records consumption over time:

```text
CPU-seconds
GB-hours
network bytes
GPU-seconds
storage GB-days
```

This supports billing, quotas, optimization, and governance.

# 48. Resource Cost

A resource may have an operational cost:

```text
CPU = cost/unit
GPU = high cost/unit
external API = monetary cost
```

The scheduler may optimize for cost when policy permits.

# 49. Resource Priority

Resources may have priority classes:

```text
critical
high
normal
low
```

But priority should not bypass security authorization.

# 50. Preemption

If higher-priority Work arrives:

```text
Low Priority Work
       ↓
Preempted
       ↓
High Priority Work
```

Preemption must preserve resource accounting.

# 51. Graceful Preemption

Prefer:

```text
NOTIFY
 ↓
DRAIN
 ↓
CHECKPOINT
 ↓
RELEASE
```

before:

```text
FORCE TERMINATE
```

where supported.

# 52. Forced Reclamation

If a workload refuses to release a resource:

```text
lease expiry
    ↓
fencing
    ↓
forced reclamation
```

This is particularly important for distributed resources.

# 53. Resource Fencing

Fencing prevents stale holders from continuing to use a resource.

```text
Old Agent
    ↓
lease expired
    ↓
fenced
    ↓
New Agent
```

Without fencing:

```text
old + new
   ↓
concurrent authority
```

can corrupt state.

# 54. Resource Lease

Resource authority should often be time-bounded:

```text
Lease {
    resource_id
    holder
    issued_at
    expires_at
    generation
}
```

# 55. Generation Number

A generation protects against stale messages:

```text
Lease generation = 42
```

A new allocation:

```text
generation = 43
```

Messages carrying generation 42 can be rejected.

# 56. Resource State

A resource state machine:

```text
DISCOVERING
    ↓
REGISTERED
    ↓
AVAILABLE
    ↓
RESERVED
    ↓
ALLOCATED
    ↓
IN_USE
    ↓
RELEASED
    ↓
AVAILABLE
```

Failure path:

```text
IN_USE
   ↓
DEGRADED
   ↓
FENCED
   ↓
RECOVERING
   ↓
AVAILABLE
```

# 57. Resource Drain

Before maintenance:

```text
AVAILABLE
   ↓
DRAINING
```

New allocations are blocked.

Existing allocations are allowed to finish or migrate.

# 58. Resource Retirement

After draining:

```text
DRAINING
   ↓
RETIRED
```

Retired resources cannot be newly allocated.

# 59. Resource Failure

A failure should not immediately become:

```text
RETIRED
```

Instead:

```text
HEALTHY
   ↓
SUSPECTED
   ↓
DEGRADED
   ↓
FAILED
```

depending on evidence.

# 60. Resource Health

Resource health can be:

```text
HEALTHY
DEGRADED
SUSPECTED
FAILED
UNKNOWN
```

This integrates directly with the observability model.

# 61. Resource Availability

Availability should be calculated from multiple dimensions:

```text
capacity
health
reservation
allocation
quota
policy
connectivity
```

Not simply:

```text
capacity > 0
```

# 62. Resource Eligibility

A resource may physically have capacity but still be ineligible.

Example:

```text
GPU available
but
security policy forbids tenant A
```

Therefore:

```text
physical availability ≠ schedulable availability
```

# 63. Resource Pool

Resources can be grouped:

```text
GPU Pool
 ├─ GPU-1
 ├─ GPU-2
 └─ GPU-3
```

The scheduler can select from the pool.

# 64. Resource Hierarchy

Resources can form a tree:

```text
Cluster
 ├─ Node A
 │   ├─ CPU
 │   ├─ Memory
 │   └─ GPU
 │
 └─ Node B
     ├─ CPU
     ├─ Memory
     └─ GPU
```

# 65. Hierarchical Accounting

Consumption can propagate:

```text
Agent
 ↓
Node
 ↓
Cluster
 ↓
Tenant
```

This enables both local and global quota enforcement.

# 66. Resource Reservation Conflict

Suppose:

```text
Capacity = 10
Reservation A = 6
Reservation B = 6
```

The second reservation must be rejected or explicitly overcommitted.

Never silently exceed capacity.

# 67. Reservation Expiration

Reservations should generally have:

```text
expires_at
```

Otherwise stale reservations can permanently consume scheduling capacity.

# 68. Reservation Renewal

Renewal should be explicit:

```text
Reservation R1
   ↓
renew
   ↓
new expiration
```

Repeated renewal may itself be subject to quota and policy.

# 69. Reservation Cancellation

A reservation can be:

```text
CANCELLED
EXPIRED
FULFILLED
RELEASED
```

These states should remain distinguishable.

# 70. Resource Transfer

Some systems may support transferring an allocation:

```text
Agent A
   ↓
Resource R
   ↓ transfer
Agent B
```

This should not happen implicitly.

It requires:

```text
authorization
ownership validation
atomicity
accounting update
```

# 71. Resource Reconciliation

If accounting becomes inconsistent:

```text
reported allocation = 4
observed usage = 0
```

NROS should detect the mismatch.

```text
Accounting
   ↓
Observation
   ↓
Reconciliation
```

# 72. Resource Drift

Possible drift:

```text
Desired:
    allocation = 4

Observed:
    allocation = 2
```

This should generate:

```text
RESOURCE_DRIFT
```

# 73. Resource Reclamation

When Work ends:

```text
ExecutionCompleted
       ↓
Release
       ↓
Verify
       ↓
Available
```

The release must be confirmed rather than assumed.

# 74. Release Failure

If release fails:

```text
Work completed
    ↓
resource release failed
    ↓
resource remains SUSPECTED
```

The scheduler should not immediately allocate it elsewhere.

# 75. Resource Accounting Ledger

For critical resources, an append-only accounting stream can record:

```text
Reserve
Allocate
Consume
Release
Reclaim
Fence
Transfer
```

This creates a historical resource ledger.

# 76. Resource Ledger Example

```text
R1:
    +100 capacity
    -20 reservation
    -30 allocation
    +10 release
    -5 allocation
```

The resulting state can be reconstructed from authoritative transitions.

# 77. Accounting Authority

Only designated components may modify resource accounting.

Example:

```text
Resource Manager
    → allocation authority

Telemetry
    → observation only
```

This prevents telemetry from corrupting resource state.

# 78. Resource Security

Resource access must still pass through the security model:

```text
Principal
   ↓
Capability
   ↓
Resource Policy
   ↓
Allocation
```

Possessing a scheduler slot does not automatically grant arbitrary resource access.

# 79. Resource + Scheduling

The scheduler should receive a resource view:

```text
Resource State
+
Work Requirements
+
Agent Capabilities
+
Policy
```

and produce:

```text
Candidate Assignment
```

# 80. Resource + Agent

An Agent may advertise:

```text
capabilities
resource capacity
current allocations
health
location
constraints
```

But the Agent's self-report should not automatically be authoritative.

# 81. Resource Claims

Claims can be verified:

```text
Agent reports:
    GPU = available

Resource manager:
    GPU = allocated
```

The authoritative manager wins.

# 82. Resource Reservation Protocol

A robust allocation flow:

```text
REQUEST
   ↓
CHECK
   ↓
RESERVE
   ↓
VERIFY
   ↓
ALLOCATE
   ↓
START
   ↓
USE
   ↓
RELEASE
   ↓
VERIFY RELEASE
```

# 83. Failed Allocation

If startup fails after reservation:

```text
RESERVED
   ↓
START_FAILED
   ↓
RELEASE
```

The reservation must not leak.

# 84. Atomic Allocation

Where possible:

```text
Reserve all required resources
```

before execution starts.

For:

```text
CPU + Memory + GPU
```

partial acquisition should not leave a permanently inconsistent state.

# 85. Two-Phase Resource Acquisition

For distributed resources:

```text
PREPARE
   ↓
COMMIT
```

may be required.

If preparation fails:

```text
ABORT
```

# 86. Resource Deadlock

Naive sequential acquisition can produce:

```text
Agent A:
    holds CPU
    waits GPU

Agent B:
    holds GPU
    waits CPU
```

This is a resource deadlock.

# 87. Deadlock Prevention

Possible strategies:

```text
global resource ordering
atomic multi-resource reservation
timeout
deadlock detection
preemption
```

# 88. Resource Ordering

A deterministic acquisition order can reduce deadlocks:

```text
CPU
 ↓
Memory
 ↓
GPU
 ↓
Storage
```

All participants follow the same order.

# 89. Resource Starvation

A resource can repeatedly be allocated to high-priority workloads:

```text
low-priority Work
     ↓
never executes
```

Scheduler fairness mechanisms are therefore required.

# 90. Fairness

Fair allocation may consider:

```text
tenant share
priority
historical usage
wait time
quota
resource cost
```

# 91. Fairness vs Utilization

Maximum utilization and fairness can conflict.

Example:

```text
Tenant A:
    highly efficient workload

Tenant B:
    long waiting workload
```

A pure utilization optimizer may continually favor A.

NROS must therefore define the intended scheduling objective.

# 92. Resource Accounting Invariants

```text
1. Every managed resource has a stable identity.

2. Capacity is distinct from availability.

3. Allocation is distinct from usage.

4. Reservation is distinct from allocation.

5. Quota is distinct from capacity.

6. Physical availability is distinct from schedulable availability.

7. Resource ownership is explicit.

8. Delegated authority cannot exceed ownership authority.

9. Resource accounting is authoritative only in designated components.

10. Telemetry cannot directly mutate authoritative accounting.

11. Allocation cannot silently exceed capacity unless overcommitment is explicit.

12. Resource usage violations are detectable.

13. Reservations expire or are explicitly released.

14. Failed allocations cannot leak reservations.

15. Release is verified.

16. Stale resource authority is fenced.

17. Generation numbers prevent stale holders from acting.

18. Resource health is distinct from resource capacity.

19. Unknown health is not equivalent to healthy.

20. Draining blocks inappropriate new allocations.

21. Retired resources cannot be newly allocated.

22. Multi-resource acquisition must avoid inconsistent partial allocation.

23. Deadlock behavior is explicitly addressed.

24. Resource fragmentation is observable.

25. Resource locality is explicit.

26. Affinity and anti-affinity are distinct.

27. Hard constraints cannot be violated for soft preferences.

28. Quotas are enforced before unauthorized allocation.

29. Resource accounting is reconstructable for critical resources.

30. Every resource state transition has an attributable cause.
```

# 93. Unified Resource Architecture

```text
                       RESOURCE
                           │
                    ┌──────┴──────┐
                    ↓             ↓
                 CAPACITY       HEALTH
                    │             │
                    └──────┬──────┘
                           ↓
                      AVAILABILITY
                           │
              ┌────────────┼────────────┐
              ↓            ↓            ↓
         RESERVATION     QUOTA        POLICY
              │            │            │
              └────────────┼────────────┘
                           ↓
                       ALLOCATION
                           │
                           ↓
                          LEASE
                           │
                           ↓
                         USAGE
                           │
                           ↓
                       ACCOUNTING
                           │
                           ↓
                         RELEASE
                           │
                           ↓
                       VERIFY
                           │
                           ↓
                       AVAILABLE
```

# 94. Core Resource Principle

The central NROS rule becomes:

> **A resource is not merely something that exists; it is a governed capacity whose ownership, availability, reservation, allocation, consumption, and release are independently represented and verifiable.**

This gives us the foundation required for the next layer:

```text
Resource Model
      ↓
Scheduling Model
      ↓
Placement
      ↓
Priorities
      ↓
Fairness
      ↓
Preemption
      ↓
Backpressure
      ↓
Admission Control
```

# Part XCV — Scheduling Theory, Placement, Priorities, Fairness & Preemption

The next layer will formalize the **scheduler itself**:

```text
Work selection
Agent selection
Resource placement
Hard constraints
Soft constraints
Priority
Deadline
Fairness
Starvation prevention
Affinity
Anti-affinity
Bin packing
Load balancing
Gang scheduling
Preemption
Backoff
Retry interaction
Queue discipline
Backpressure
Admission control
Scheduling epochs
Determinism
Scheduling explanations
```

The key principle will be:

> **A scheduler does not merely choose what runs next; it produces an authorized, resource-valid, explainable assignment under explicit constraints and fairness policy.**

# NROS — Part XCV: Scheduling Theory, Placement, Priorities, Fairness & Preemption

The resource model established **what can be consumed**.

The scheduler establishes **who gets what, when, where, and why**.

The scheduler therefore sits at the intersection of:

```text
                    WORK
                      │
                      ↓
                  SCHEDULER
                ↙     ↓      ↘
            POLICY   RESOURCES  AGENTS
               \       │       /
                \      │      /
                 ↓     ↓     ↓
                 ASSIGNMENT
                      │
                      ↓
                  EXECUTION
```

The scheduler must never be treated as an opaque function such as:

```text
schedule(work) → node
```

It is a governed decision system.

# 1. Scheduling Decision

A scheduling decision should contain enough information to explain itself:

```text
SchedulingDecision {
    decision_id
    work_id
    agent_id
    resource_assignments
    scheduler_revision
    policy_revision
    reason
    constraints_checked
    timestamp
}
```

# 2. Scheduler Inputs

The scheduler consumes:

```text
Work
Agent state
Resource state
Capabilities
Constraints
Policies
Priorities
Quotas
Deadlines
Affinity
Health
Topology
Historical scheduling state
```

# 3. Scheduler Output

The fundamental output is:

```text
CandidateAssignment
```

not necessarily immediate execution.

```text
Work
 ↓
Scheduling Decision
 ↓
Admission
 ↓
Reservation
 ↓
Execution
```

This separation is important.

# 4. Scheduling vs Admission

Scheduling asks:

> Where should this Work run?

Admission asks:

> Is this Work allowed to run now?

A candidate can therefore be:

```text
SCHEDULABLE
```

but:

```text
NOT ADMISSIBLE
```

because of quota, authorization, or policy.

# 5. Scheduling Pipeline

A robust scheduler can operate as:

```text
QUEUE
  ↓
ELIGIBILITY
  ↓
FILTERING
  ↓
SCORING
  ↓
SELECTION
  ↓
RESERVATION
  ↓
COMMIT
  ↓
DISPATCH
```

# 6. Queue

Pending Work enters a scheduler queue.

A queue entry should include:

```text
work_id
arrival_time
priority
deadline
tenant
requirements
constraints
retry_state
```

# 7. Queue Ordering

Possible ordering policies include:

```text
FIFO
Priority
Deadline
Weighted Fair Queue
Shortest Expected Runtime
Aging
Tenant Fairness
```

The policy must be explicit.

# 8. FIFO

First-In-First-Out:

```text
W1 → W2 → W3
```

is simple and predictable.

But FIFO can cause priority inversion when urgent Work arrives behind long-running Work.

# 9. Priority Queue

Higher priority can execute first:

```text
P10 → P7 → P3 → P1
```

But pure priority scheduling can starve low-priority Work.

# 10. Aging

Aging increases effective priority as Work waits.

Conceptually:

```text
effective_priority
=
base_priority
+
waiting_bonus
```

This provides starvation resistance.

# 11. Deadline Scheduling

Work may have:

```text
deadline = T
```

The scheduler can prioritize Work with the earliest deadline.

But deadline scheduling should not automatically override security or hard resource constraints.

# 12. Deadline Feasibility

A Work item should ideally be evaluated against:

```text
remaining_time
estimated_execution_time
queue_delay
resource availability
```

If completion before deadline is impossible, the scheduler can identify it early.

# 13. Scheduling Classes

NROS can define scheduling classes:

```text
REALTIME
LATENCY_SENSITIVE
NORMAL
BATCH
BEST_EFFORT
BACKGROUND
```

Each class may have different policies.

# 14. Eligibility

Before scoring candidates, the scheduler should eliminate impossible targets.

For each candidate:

```text
Agent
 ↓
Capability Check
 ↓
Health Check
 ↓
Resource Check
 ↓
Policy Check
 ↓
Constraint Check
```

# 15. Hard Constraints

Hard constraints produce binary eligibility:

```text
GPU required
→ GPU exists?

YES → eligible
NO  → reject
```

Hard constraints should never be converted into a mere score.

# 16. Soft Constraints

Soft constraints influence ranking:

```text
prefer same zone
prefer cached data
prefer lower utilization
```

Failure to satisfy them does not necessarily make a candidate invalid.

# 17. Filtering

Example:

```text
100 Agents
   ↓ capability filter
60
   ↓ health filter
48
   ↓ resource filter
17
   ↓ policy filter
12
```

Only these 12 proceed to scoring.

# 18. Scoring

A candidate score can combine:

```text
resource fit
locality
load
cost
latency
fairness
cache affinity
energy
priority
```

For example:

```text
Score =
    locality_weight
  + resource_fit_weight
  - load_penalty
  - cost_penalty
```

The exact formula should be versioned.

# 19. Determinism

Given identical:

```text
inputs
policy revision
scheduler revision
random seed
```

the scheduler should ideally produce the same result.

This is extremely valuable for debugging and reproducibility.

# 20. Tie Breaking

Two candidates may have identical scores.

NROS should define deterministic tie-breaking:

```text
score
 ↓
priority
 ↓
resource fit
 ↓
agent ID
```

Never rely on hash-map iteration order.

# 21. Scheduler Revision

Scheduling logic itself should be versioned:

```text
scheduler_policy = v17
```

A scheduling decision can therefore record:

```text
policy=v17
```

for later explanation.

# 22. Placement

Placement determines where Work executes.

Dimensions may include:

```text
host
zone
region
device
container
process
runtime
```

# 23. Bin Packing

Resource placement often resembles bin packing.

Example:

```text
Node A: 8 CPU
Node B: 8 CPU

Work:
W1 = 4
W2 = 3
W3 = 2
```

Possible placement:

```text
A → W1 + W3 = 6
B → W2 = 3
```

# 24. Best Fit

Best-fit placement chooses the smallest suitable remaining capacity.

This can reduce fragmentation.

# 25. Worst Fit

Worst-fit places Work on the least utilized suitable resource.

This can distribute load more evenly.

# 26. First Fit

First-fit chooses the first eligible candidate.

It is simple and computationally cheap.

# 27. Algorithm Selection

NROS should not mandate one algorithm globally.

Different workloads may benefit from:

```text
bin packing
load balancing
deadline scheduling
fair sharing
locality-aware placement
```

# 28. Scheduling Objective

A scheduler should explicitly declare its objective.

Examples:

```text
maximize throughput
minimize latency
maximize fairness
minimize cost
minimize fragmentation
maximize availability
```

# 29. Multi-Objective Scheduling

Real systems often need:

```text
latency
+
fairness
+
cost
+
resource efficiency
```

These objectives must have explicit priority or weighting.

Otherwise behavior becomes impossible to reason about.

# 30. Fairness

Fairness means preventing one principal from monopolizing shared resources.

Example:

```text
Tenant A → 90%
Tenant B → 10%
```

may be acceptable if A has 9× the entitlement.

Fairness therefore requires defined entitlements.

# 31. Weighted Fairness

Example:

```text
Tenant A weight = 2
Tenant B weight = 1
```

Long-run allocation can target:

```text
A ≈ 66.7%
B ≈ 33.3%
```

subject to capacity and workload availability.

# 32. Fairness Scope

Fairness may apply at:

```text
tenant
project
agent
workload
user
resource pool
```

The scope must be explicit.

# 33. Fairness vs Priority

Priority is not the same as fairness.

A high-priority Work may run sooner.

Fairness determines whether the same principal can continually dominate future scheduling.

# 34. Starvation

Starvation occurs when:

```text
Work remains eligible
but
never receives resources.
```

This must be detectable.

# 35. Starvation Detection

Useful indicators:

```text
queue_age
number_of_deferrals
last_scheduled_at
resource_wait_duration
priority_adjustment
```

# 36. Starvation Prevention

Possible mechanisms:

```text
aging
fair-share scheduling
maximum wait thresholds
priority boosting
reservation
quota-aware scheduling
```

# 37. Backpressure

When demand exceeds capacity:

```text
incoming Work
      ↓
     queue
      ↓
capacity limit
```

The system should apply backpressure rather than allowing unbounded growth.

# 38. Queue Limits

Example:

```text
max_pending_work = 10000
```

When reached:

```text
REJECT
DEFER
SHED
```

depending on policy.

# 39. Load Shedding

Under severe overload, NROS may reject low-priority Work:

```text
critical → admitted
normal   → limited
batch    → deferred
background → rejected
```

# 40. Admission Under Pressure

A useful hierarchy:

```text
1. Safety-critical
2. Security-critical
3. Deadline-sensitive
4. Normal
5. Best-effort
```

But the exact order is a policy decision.

# 41. Preemption

Preemption allows a newly admitted Work item to reclaim resources.

```text
Low Priority W1
       ↓
preempt
       ↓
High Priority W2
```

# 42. Preemption Eligibility

Not every Work item should be preemptible.

Possible classes:

```text
PREEMPTIBLE
NON_PREEMPTIBLE
CHECKPOINTABLE
TERMINATION_ONLY
```

# 43. Graceful Preemption

Preferred sequence:

```text
REQUEST_STOP
   ↓
CHECKPOINT
   ↓
RELEASE
   ↓
CONFIRM
```

# 44. Forced Preemption

If graceful shutdown exceeds its deadline:

```text
GRACE_PERIOD_EXPIRED
        ↓
FORCE_STOP
        ↓
FENCE
        ↓
RECLAIM
```

# 45. Preemption Cost

Preemption is not free.

Costs may include:

```text
checkpoint overhead
restart latency
cache loss
state reconstruction
network traffic
lost computation
```

The scheduler should account for this.

# 46. Preemption Decision

Instead of:

```text
priority(A) > priority(B)
```

consider:

```text
benefit_of_preemption
>
cost_of_preemption
```

when policy allows optimization.

# 47. Checkpointing

Checkpointable Work can preserve execution state:

```text
Work
 ↓
Checkpoint
 ↓
Release Resources
 ↓
Resume Later
```

This is especially useful for batch workloads.

# 48. Gang Scheduling

Some Work requires multiple resources simultaneously.

Example:

```text
distributed job:
    4 Agents
    4 GPUs
```

Starting only 2 may produce no useful progress.

Gang scheduling requires:

```text
all required allocations
    ↓
commit together
```

# 49. Co-Scheduling

Related Work may need coordinated execution:

```text
Agent A
Agent B
Agent C
```

must start within a bounded interval.

# 50. Affinity

Affinity expresses a preference or requirement to place Work near another workload/resource.

Examples:

```text
same node
same zone
same cache
same storage
```

# 51. Anti-Affinity

Anti-affinity intentionally separates workloads:

```text
Replica A → Node 1
Replica B → Node 2
Replica C → Node 3
```

This improves resilience.

# 52. Failure-Domain Awareness

Placement should understand failure domains:

```text
process
host
rack
zone
region
```

Replicas should not accidentally share the same failure domain when redundancy is required.

# 53. Topology-Aware Scheduling

The scheduler should model:

```text
distance
bandwidth
latency
failure domain
```

rather than treating all resources as equivalent.

# 54. Data Locality

If Work requires data:

```text
Dataset D
```

placement near D may reduce:

```text
network traffic
latency
cost
```

Data locality can therefore become a soft or hard constraint.

# 55. Cache Affinity

An Agent may already contain useful state:

```text
model cache
dataset cache
compiled artifact
connection pool
```

Reusing it may improve efficiency.

# 56. Scheduling Hints

Work can provide hints:

```text
preferred_zone
preferred_agent
preferred_resource_type
locality
```

Hints are not necessarily guarantees.

# 57. Scheduling Constraints

A clean representation:

```text
Constraint {
    kind
    target
    operator
    value
    hardness
}
```

where:

```text
hardness = HARD | SOFT
```

# 58. Scheduling Explainability

Every rejected candidate should ideally have a reason.

Example:

```text
Agent A:
    REJECTED
    reason = insufficient_memory

Agent B:
    REJECTED
    reason = capability_missing:gpu

Agent C:
    ACCEPTED
    score = 87.2
```

This makes scheduling debuggable.

# 59. Decision Trace

A scheduling trace can record:

```text
candidate_count
filtered_count
rejection_reasons
scores
selected_candidate
policy_revision
```

This becomes valuable operational evidence.

# 60. Scheduling Dry Run

NROS should support:

```text
schedule --dry-run
```

conceptually:

```text
Work
 ↓
evaluate
 ↓
show candidates
 ↓
show decision
```

without mutating resource state.

# 61. Scheduling Simulation

A simulation mode can test:

```text
new policy
new workload
new topology
new quotas
```

before activation.

# 62. Scheduling Epoch

A scheduler may operate in epochs:

```text
Epoch 100
 ↓
observe
 ↓
make decisions
 ↓
commit
 ↓
Epoch 101
```

This can improve deterministic behavior.

# 63. Event-Driven Scheduling

Alternatively:

```text
ResourceAvailable
WorkArrived
WorkCompleted
AgentFailed
QuotaChanged
```

can trigger scheduling decisions.

# 64. Hybrid Scheduling

A practical architecture may combine:

```text
event-driven wakeups
+
periodic reconciliation
```

The periodic loop repairs missed events or stale state.

# 65. Scheduler State

Scheduler state may include:

```text
queues
fairness counters
reservations
backoff timers
preemption state
placement state
policy revision
```

# 66. Scheduler Recovery

After restart, scheduler state must be reconstructable.

Sources may include:

```text
persistent Work state
resource ledger
active leases
configuration revision
event log
```

# 67. Scheduler Crash

A crash must not automatically cause duplicate execution.

After restart:

```text
recover
 ↓
observe
 ↓
reconcile
 ↓
recover leases
 ↓
resume scheduling
```

# 68. Duplicate Scheduling

Two schedulers must not both believe they own the same decision authority.

Possible mechanisms:

```text
leader election
epoch fencing
lease ownership
partitioned scheduler authority
```

# 69. Scheduler Leadership

If one scheduler is active:

```text
Leader
   ↓
scheduling authority
```

Followers observe and prepare.

# 70. Scheduler Epoch

Every scheduler leadership generation can receive:

```text
scheduler_epoch = 57
```

Decisions from older epochs become invalid.

# 71. Stale Scheduler Protection

Execution should reject:

```text
decision.epoch < current_epoch
```

This prevents stale controllers from assigning resources.

# 72. Scheduler Concurrency

Multiple Work items may be schedulable simultaneously.

The scheduler must avoid:

```text
W1 sees GPU free
W2 sees GPU free
W1 allocates GPU
W2 allocates same GPU
```

# 73. Reservation as Concurrency Control

Therefore:

```text
check
 ↓
reserve atomically
 ↓
commit
```

is safer than:

```text
check
 ↓
later allocate
```

# 74. Scheduling Race

A candidate can disappear between scoring and allocation:

```text
score Agent A
      ↓
Agent A fails
      ↓
commit
```

The commit phase must revalidate.

# 75. Final Validation

Before dispatch:

```text
candidate
 ↓
resource revalidation
 ↓
lease validation
 ↓
policy validation
 ↓
commit
```

# 76. Scheduling Failure

If final validation fails:

```text
Decision
   ↓
COMMIT_FAILED
   ↓
release reservation
   ↓
retry scheduling
```

The failure must not create an orphaned reservation.

# 77. Retry

Retries interact with scheduling.

A retry should not automatically receive unlimited priority.

Otherwise:

```text
failing Work
   ↓
retry
   ↓
retry
   ↓
retry
```

can starve healthy Work.

# 78. Retry Backoff

A common strategy:

```text
delay = base × 2^attempt
```

with:

```text
maximum_delay
jitter
attempt_limit
```

# 79. Retry Classification

Not all failures deserve retry.

Examples:

```text
transient resource failure → retry
authorization failure → reject
invalid Work → reject
capacity shortage → defer
system overload → backoff
```

# 80. Scheduler + Configuration

Scheduling behavior comes from versioned configuration:

```text
scheduler_policy_revision
resource_policy_revision
fairness_policy_revision
```

This allows decisions to be reconstructed.

# 81. Scheduler + Policy

The scheduler cannot override policy.

For example:

```text
scheduler:
    Agent A has best score

policy:
    Tenant X cannot use Agent A
```

Result:

```text
Agent A = INELIGIBLE
```

# 82. Scheduler + Security

Authorization occurs before execution authority is granted.

```text
Principal
 ↓
Work
 ↓
Authorization
 ↓
Scheduling
 ↓
Allocation
 ↓
Execution
```

# 83. Scheduler + Observability

Every meaningful decision should produce telemetry:

```text
schedule_requested
candidate_filtered
assignment_selected
reservation_created
dispatch_started
dispatch_failed
preemption_started
preemption_completed
```

# 84. Scheduling Metrics

Important metrics include:

```text
queue_depth
queue_wait_time
scheduling_latency
placement_success_rate
placement_failure_rate
preemption_count
starvation_count
resource_fragmentation
fairness_index
deadline_miss_rate
```

# 85. Scheduling SLOs

Examples:

```text
99% of eligible Work
receives a scheduling decision
within X ms.
```

or:

```text
deadline-sensitive Work
has <Y% deadline misses.
```

# 86. Scheduling Invariants

```text
1. Scheduling cannot bypass authorization.

2. Hard constraints cannot be violated.

3. Soft constraints influence ranking rather than eligibility.

4. Resource state is revalidated before final commit.

5. Allocation must be protected against scheduling races.

6. Stale scheduler decisions cannot acquire current authority.

7. Scheduler leadership changes invalidate stale epochs.

8. Reservations prevent double allocation.

9. Failed scheduling releases temporary reservations.

10. Queue growth is bounded or governed by backpressure.

11. Retry behavior cannot create unlimited starvation.

12. Low-priority Work cannot be permanently starved without explicit policy.

13. Scheduling decisions are explainable.

14. Scheduling policy is versioned.

15. Tie-breaking is deterministic where deterministic scheduling is required.

16. Candidate rejection reasons are observable.

17. Preemption is explicitly authorized.

18. Forced preemption is distinguishable from graceful preemption.

19. Gang-scheduled Work receives atomic resource treatment.

20. Affinity and anti-affinity semantics are explicit.

21. Failure domains are represented where resilience requires them.

22. Resource locality is explicit.

23. Scheduler state is recoverable.

24. Scheduler crashes cannot silently create duplicate authority.

25. Scheduling state can be reconciled against authoritative resource state.

26. Admission and scheduling remain distinct decisions.

27. Fairness policy is explicit.

28. Scheduling objectives are explicit.

29. Scheduling metrics are observable.

30. Scheduling decisions can be reconstructed from evidence.
```

# 87. Unified Scheduling Architecture

```text
                         WORK
                           │
                           ↓
                        QUEUE
                           │
                           ↓
                    ADMISSION CONTROL
                           │
                           ↓
                       ELIGIBILITY
                           │
                ┌──────────┼──────────┐
                ↓          ↓          ↓
             POLICY     CAPACITY   CAPABILITY
                │          │          │
                └──────────┼──────────┘
                           ↓
                       FILTERING
                           │
                           ↓
                        SCORING
                           │
                    ┌──────┴──────┐
                    ↓             ↓
                 AFFINITY      FAIRNESS
                    │             │
                    └──────┬──────┘
                           ↓
                       SELECTION
                           │
                           ↓
                       RESERVATION
                           │
                           ↓
                    FINAL VALIDATION
                           │
                           ↓
                         COMMIT
                           │
                           ↓
                        DISPATCH
                           │
                           ↓
                       EXECUTION
                           │
              ┌────────────┼────────────┐
              ↓            ↓            ↓
          COMPLETE      PREEMPT       FAIL
              │            │            │
              └────────────┼────────────┘
                           ↓
                       RELEASE
                           │
                           ↓
                      RECONCILIATION
```

# 88. Core Scheduling Principle

The governing rule is:

> **A scheduling decision is an authorized placement decision over currently valid resources, constrained by policy, fairness, topology, and workload requirements, and committed only after final state validation.**

This completes the conceptual chain:

```text
Configuration
     ↓
Desired State
     ↓
Resource Model
     ↓
Capacity
     ↓
Scheduling
     ↓
Placement
     ↓
Reservation
     ↓
Allocation
     ↓
Execution
     ↓
Observation
     ↓
Reconciliation
```

# Part XCVI — Execution Admission, Dispatch, Leases & Work Lifecycle

The next layer should move from **"the scheduler selected a placement"** to **"the system safely turns that placement into an executing Work instance."**

That requires formalizing:

```text
Work identity
Execution identity
Attempt identity
Admission
Dispatch
Command issuance
Lease acquisition
Lease renewal
Start protocol
Execution states
Cancellation
Timeouts
Heartbeat
Lost execution
Orphan detection
Reattachment
Checkpoint
Recovery
Completion
Failure
Retry
Idempotency
Exactly-once vs at-least-once semantics
Result publication
```

The key principle for the next layer:

> **Scheduling creates intent; dispatch creates execution authority; a lease makes that authority time-bounded; execution evidence determines whether the intended Work actually became real.**

# NROS — Part XCVI: Execution Admission, Dispatch, Leases & Work Lifecycle

The scheduling layer answered:

> **Where should Work run?**

The execution layer answers:

> **How does that decision become a real, observable, bounded, recoverable execution?**

The critical distinction is:

```text
Scheduling
    ↓
intent

Admission
    ↓
authorization to proceed

Dispatch
    ↓
delivery of execution command

Lease
    ↓
time-bounded authority

Execution
    ↓
actual Work

Observation
    ↓
evidence of reality
```

# 1. Work Identity

A Work item must have a stable identity independent of individual execution attempts.

```text
Work {
    work_id
    tenant_id
    specification
    priority
    lifecycle
}
```

Example:

```text
work-7f21
```

# 2. Execution Identity

A Work can execute multiple times.

Therefore:

```text
Work
 ├── Execution #1
 ├── Execution #2
 └── Execution #3
```

Each execution receives its own identity:

```text
execution_id
```

# 3. Attempt Identity

An execution may itself contain retry attempts:

```text
work_id
   ↓
execution_id
   ↓
attempt_1
attempt_2
attempt_3
```

This distinction prevents retry history from being confused with Work identity.

# 4. Identity Hierarchy

```text
Tenant
  ↓
Work
  ↓
Execution
  ↓
Attempt
  ↓
Process / Agent Instance
```

Every level has different lifecycle semantics.

# 5. Execution Record

A minimal execution record:

```text
Execution {
    execution_id
    work_id
    attempt
    placement
    scheduler_epoch
    policy_revision
    state
    created_at
    started_at
    finished_at
}
```

# 6. Admission

Scheduling does not automatically mean execution.

The execution admission phase verifies:

```text
authorization
resource reservation
policy
quota
lease availability
agent health
execution prerequisites
```

# 7. Admission Pipeline

```text
Scheduled
   ↓
Admission Check
   ↓
Admitted
   ↓
Dispatch
```

Failure:

```text
Scheduled
   ↓
Admission Rejected
   ↓
Reschedule / Fail
```

# 8. Admission Token

The system may issue an explicit admission artifact:

```text
AdmissionToken {
    execution_id
    resource_set
    agent_id
    scheduler_epoch
    expires_at
}
```

This binds the execution to the scheduling decision.

# 9. Dispatch

Dispatch is the act of delivering an execution request to the selected execution target.

```text
Scheduler
    ↓
Execution Manager
    ↓
Agent
```

# 10. Dispatch Is Not Execution

A successful message transmission does not prove execution started.

```text
DISPATCH_SENT
```

is different from:

```text
EXECUTION_STARTED
```

This distinction is fundamental.

# 11. Dispatch States

```text
PENDING
   ↓
SENT
   ↓
ACKNOWLEDGED
   ↓
ACCEPTED
   ↓
STARTING
   ↓
RUNNING
```

Failure paths:

```text
SENT → TIMEOUT
ACKNOWLEDGED → START_FAILED
STARTING → CRASHED
```

# 12. Dispatch Acknowledgement

The target should acknowledge receipt:

```text
Dispatch
   ↓
ACK
```

But acknowledgement only proves:

> the target received the request.

It does not prove:

> the Work is running.

# 13. Start Confirmation

Execution should produce an independent start event:

```text
execution_started
```

containing:

```text
execution_id
attempt
agent_id
process_id / runtime_id
timestamp
```

# 14. Execution Authority

The agent must verify:

```text
execution_id
scheduler_epoch
lease
resource assignment
authorization
```

before starting.

# 15. Stale Dispatch

Suppose:

```text
scheduler epoch = 51
```

creates a dispatch.

Later:

```text
scheduler epoch = 52
```

becomes authoritative.

A delayed epoch-51 message must be rejected.

```text
epoch 51 < epoch 52
→ STALE
```

# 16. Execution Lease

Execution authority should normally be bounded:

```text
Lease {
    execution_id
    holder
    generation
    issued_at
    expires_at
}
```

# 17. Why Leases?

Without leases:

```text
scheduler crashes
     ↓
agent continues forever
```

The system may no longer know whether the execution remains valid.

A lease establishes:

```text
authority until T
```

# 18. Lease Renewal

A running execution periodically renews:

```text
heartbeat
   ↓
lease renewal
   ↓
new expiration
```

# 19. Heartbeat

Heartbeat provides liveness evidence:

```text
Heartbeat {
    execution_id
    generation
    timestamp
    state
    resource_usage
}
```

# 20. Heartbeat Is Evidence

A heartbeat is not merely a networking mechanism.

It provides evidence that:

```text
execution still exists
```

at a particular point in time.

# 21. Heartbeat Timeout

If:

```text
now > lease_expiration
```

without renewal:

```text
LEASE_EXPIRED
```

The execution becomes suspect.

# 22. Suspicion Before Reclamation

A robust system should distinguish:

```text
RUNNING
 ↓
HEARTBEAT_MISSED
 ↓
SUSPECTED
 ↓
LEASE_EXPIRED
 ↓
FENCED
 ↓
RECLAIMED
```

This avoids immediately killing workloads because of a transient network interruption.

# 23. Network Partition

Consider:

```text
Scheduler
   X
Agent
```

The scheduler cannot reach the agent.

The agent cannot reach the scheduler.

Both sides may believe the other has disappeared.

Leases and fencing prevent both sides from acquiring contradictory authority.

# 24. Split-Brain Prevention

Authority must be associated with:

```text
generation
epoch
lease
fencing token
```

rather than merely:

```text
agent_id
```

# 25. Fencing Token

Example:

```text
fencing_token = 88342
```

Every resource operation must carry the current token.

An old execution using:

```text
88341
```

is rejected.

# 26. Execution State Machine

A formal lifecycle:

```text
CREATED
   ↓
ADMITTED
   ↓
DISPATCHING
   ↓
DISPATCHED
   ↓
STARTING
   ↓
RUNNING
   ↓
┌───────────────┬──────────────┐
↓               ↓              ↓
COMPLETED      FAILED       CANCELLED
```

Exceptional path:

```text
RUNNING
   ↓
SUSPECTED
   ↓
LOST
   ↓
RECOVERING
```

# 27. State Ownership

Only authoritative components should mutate execution state.

For example:

```text
Execution Manager
    → lifecycle authority

Agent
    → execution observations

Telemetry
    → observation only
```

# 28. State Transition Rule

Every transition should have:

```text
previous_state
event
new_state
actor
timestamp
reason
```

Example:

```text
RUNNING
 + heartbeat_timeout
 → SUSPECTED
```

# 29. Illegal Transitions

Examples:

```text
COMPLETED → RUNNING
CANCELLED → STARTING
FAILED → RUNNING
```

should be rejected unless an explicitly defined recovery transition exists.

# 30. Start Idempotency

Network retries can duplicate dispatch messages.

Therefore:

```text
start(execution_id)
```

must be idempotent.

If the execution is already running:

```text
same request
→ existing execution
```

not:

```text
spawn second process
```

# 31. Idempotency Key

A dispatch should carry:

```text
idempotency_key
```

or use:

```text
execution_id + attempt
```

as a unique execution key.

# 32. Duplicate Dispatch

Scenario:

```text
Dispatch #1
    ↓
Agent starts
    ↓
ACK lost

Scheduler retries
    ↓
Dispatch #2
```

The agent must recognize:

```text
same execution
```

and avoid duplicate execution.

# 33. Exactly-Once Execution

Exactly-once execution is difficult in distributed systems.

NROS should not claim:

```text
exactly once
```

unless the entire execution and side-effect model actually guarantees it.

# 34. At-Least-Once Dispatch

A more realistic transport semantic:

```text
dispatch = at-least-once
```

combined with:

```text
idempotent start
```

can produce safe behavior.

# 35. At-Most-Once Dispatch

Alternatively:

```text
dispatch = at-most-once
```

reduces duplicates but risks lost Work.

The protocol must state which semantics apply.

# 36. Execution Side Effects

Even if start is idempotent, Work itself may not be.

Example:

```text
charge_account()
```

could execute twice.

Therefore idempotency must extend to externally visible effects where required.

# 37. Result Identity

Results should be associated with:

```text
work_id
execution_id
attempt
```

not merely:

```text
work_id
```

Otherwise stale attempts can overwrite newer results.

# 38. Result Publication

A completed execution produces:

```text
ExecutionResult {
    execution_id
    status
    outputs
    metrics
    artifacts
    completion_time
}
```

# 39. Result Commit

Completion should follow:

```text
RUNNING
 ↓
RESULT_PREPARED
 ↓
RESULT_COMMITTED
 ↓
COMPLETED
```

This separates:

```text
computation finished
```

from:

```text
result durably accepted
```

# 40. Partial Results

Long-running Work may publish intermediate results:

```text
checkpoint_1
checkpoint_2
checkpoint_3
```

Each should carry:

```text
execution_id
sequence
generation
```

# 41. Monotonic Sequence

For execution events:

```text
seq = 1
seq = 2
seq = 3
```

The receiver can reject stale or duplicated events.

# 42. Event Ordering

Distributed systems may deliver:

```text
event 3
event 1
event 2
```

Therefore consumers should not blindly assume transport order equals execution order.

# 43. Execution Event

A canonical event:

```text
ExecutionEvent {
    execution_id
    sequence
    generation
    type
    timestamp
    payload
}
```

# 44. Completion

Normal completion:

```text
RUNNING
   ↓
RESULT_PREPARED
   ↓
RESULT_COMMITTED
   ↓
COMPLETED
   ↓
RESOURCE_RELEASE
```

# 45. Failure

Failure should preserve diagnostic information:

```text
FAILED {
    error_code
    reason
    source
    retryable
    timestamp
}
```

# 46. Failure Classification

At minimum:

```text
USER_ERROR
RESOURCE_ERROR
AGENT_ERROR
NETWORK_ERROR
POLICY_ERROR
TIMEOUT
SYSTEM_ERROR
UNKNOWN
```

# 47. Retryability

A failure should explicitly indicate:

```text
retryable = true
```

or:

```text
retryable = false
```

rather than relying solely on error strings.

# 48. Cancellation

Cancellation is different from failure.

```text
RUNNING
   ↓
CANCELLATION_REQUESTED
   ↓
CANCELLED
```

The reason may be:

```text
user
policy
timeout
preemption
system shutdown
```

# 49. Cancellation Timeout

If graceful cancellation does not finish:

```text
CANCELLATION_REQUESTED
       ↓
grace period
       ↓
FORCED_TERMINATION
```

# 50. Timeout

Timeouts should be explicit:

```text
queue_timeout
admission_timeout
dispatch_timeout
startup_timeout
execution_timeout
lease_timeout
result_timeout
```

These are not interchangeable.

# 51. Startup Timeout

Example:

```text
DISPATCHED
   ↓
STARTING
   ↓
startup deadline exceeded
```

Result:

```text
START_TIMEOUT
```

# 52. Execution Timeout

Once running:

```text
RUNNING
   ↓
execution deadline exceeded
```

Result:

```text
EXECUTION_TIMEOUT
```

# 53. Lease Timeout

Lease timeout is different:

```text
RUNNING
   ↓
no valid renewal
```

Result:

```text
LEASE_EXPIRED
```

The Work may or may not actually have stopped.

# 54. Lost Execution

The scheduler may lose contact with an Agent while the process continues.

Therefore:

```text
LOST
```

means:

> authoritative control/evidence was lost,

not necessarily:

> the process definitely terminated.

# 55. Reattachment

If an Agent reconnects and proves continuity:

```text
LOST
 ↓
REATTACHING
 ↓
RUNNING
```

This requires strong identity and generation validation.

# 56. Reattachment Conditions

Possible requirements:

```text
same execution_id
same attempt
valid generation
valid fencing token
resource still assigned
agent identity verified
```

# 57. Orphan Execution

An orphan exists when:

```text
process exists
```

but:

```text
no valid execution authority exists
```

This is dangerous.

# 58. Orphan Handling

Possible sequence:

```text
ORPHAN_DETECTED
   ↓
VERIFY
   ↓
FENCE
   ↓
TERMINATE
   ↓
RECLAIM
```

unless policy permits recovery.

# 59. Resource Release

Execution completion must trigger:

```text
resource release
```

but resource release must also be independently verified.

# 60. Execution Finalization

A complete lifecycle:

```text
CREATE
 ↓
ADMIT
 ↓
DISPATCH
 ↓
ACK
 ↓
START
 ↓
LEASE
 ↓
RUN
 ↓
RESULT
 ↓
COMMIT
 ↓
RELEASE
 ↓
VERIFY
 ↓
FINALIZE
```

# 61. Finalization Record

The final execution record should contain:

```text
execution_id
work_id
attempt
final_state
start_time
finish_time
duration
agent
resources
result_reference
failure_reference
policy_revision
scheduler_revision
```

# 62. Execution Accounting

Execution should connect resource accounting:

```text
Execution
   ↓
Allocation
   ↓
Usage
   ↓
Release
```

This creates traceability between:

```text
what was scheduled
```

and:

```text
what was actually consumed.
```

# 63. Execution Traceability

A single Work should be traceable through:

```text
Work ID
 ↓
Scheduling Decision
 ↓
Reservation
 ↓
Allocation
 ↓
Dispatch
 ↓
Lease
 ↓
Execution
 ↓
Usage
 ↓
Result
 ↓
Release
```

This is the execution equivalent of a provenance chain.

# 64. Execution Evidence

Evidence levels can distinguish:

```text
REQUESTED
DISPATCHED
ACKNOWLEDGED
START_CONFIRMED
RUNNING_OBSERVED
COMPLETED_OBSERVED
RESULT_COMMITTED
RELEASE_CONFIRMED
```

# 65. Important Distinction

Never equate:

```text
DISPATCHED
```

with:

```text
RUNNING
```

and never equate:

```text
RUNNING
```

with:

```text
COMPLETED
```

Each requires independent evidence.

# 66. Execution Observability

Recommended metrics:

```text
dispatch_latency
startup_latency
execution_duration
heartbeat_gap
lease_renewal_latency
completion_latency
failure_rate
retry_rate
preemption_rate
orphan_rate
```

# 67. Execution SLOs

Examples:

```text
dispatch success ≥ target
startup latency ≤ target
lease renewal reliability ≥ target
orphan recovery ≤ target
result commit latency ≤ target
```

# 68. Recovery

On controller restart:

```text
Recover persisted state
        ↓
Observe agents
        ↓
Observe resources
        ↓
Validate leases
        ↓
Reconcile executions
        ↓
Recover / terminate / reschedule
```

# 69. Recovery Principle

Never infer execution reality solely from stale controller memory.

Instead:

```text
persisted intent
+
current observation
+
lease authority
=
recovered state
```

# 70. Execution Reconciliation

Example:

```text
Controller:
    RUNNING

Agent:
    process absent

Resource manager:
    allocation released
```

Result:

```text
execution = FAILED / LOST
```

depending on evidence and policy.

# 71. Contradictory Evidence

Example:

```text
Controller → RUNNING
Agent → RUNNING
Resource Manager → allocation missing
```

This is a reconciliation conflict.

It should become explicit:

```text
STATE_CONFLICT
```

rather than silently selecting one value.

# 72. Reconciliation Authority

The system should define authority per fact:

```text
execution lifecycle → Execution Manager
resource allocation → Resource Manager
process liveness → Agent
result durability → Result Store
```

No single subsystem should pretend to own every fact.

# 73. Execution Governance

Execution transitions should be authorized.

For example:

```text
RUNNING → CANCELLED
```

requires a principal with cancellation authority.

# 74. Execution Capability

The execution capability can be scoped:

```text
Capability {
    execution_id
    actions
    expires_at
    generation
}
```

Possible actions:

```text
START
STOP
RENEW
READ
CHECKPOINT
PUBLISH_RESULT
```

# 75. Least Authority

An Agent should receive only the capabilities necessary for its execution.

Example:

```text
Agent:
    START = yes
    STOP = yes
    MODIFY_POLICY = no
    ALLOCATE_OTHER_AGENT_RESOURCES = no
```

# 76. Execution Isolation

Execution environments may require isolation:

```text
process
container
sandbox
VM
microVM
dedicated host
```

The scheduler can select the isolation class based on policy.

# 77. Execution Environment

The execution specification may include:

```text
runtime
image
command
environment
mounts
network policy
resource limits
security profile
```

# 78. Immutable Execution Specification

Once admitted, critical execution fields should become immutable:

```text
execution_id
resource assignment
security identity
scheduler epoch
```

Changes should create a new revision or execution.

# 79. Mutable Runtime State

Other fields can change:

```text
heartbeat
usage
progress
logs
checkpoint
```

These belong to runtime state, not scheduling intent.

# 80. Execution Attempt Semantics

A retry should normally create:

```text
attempt + 1
```

rather than mutate the historical attempt.

Example:

```text
attempt 1 → FAILED
attempt 2 → RUNNING
```

Historical evidence remains intact.

# 81. Retry Tree

Complex Work can produce:

```text
Execution
 ├─ Attempt 1 → FAILED
 ├─ Attempt 2 → FAILED
 └─ Attempt 3 → COMPLETED
```

This should remain queryable.

# 82. Result Selection

If multiple attempts exist:

```text
attempt 1 → failure
attempt 2 → success
```

the canonical result is normally derived from the successful authoritative attempt.

But previous attempts remain immutable historical evidence.

# 83. Cancellation + Retry

A cancellation should normally not automatically become a retryable failure.

```text
CANCELLED
```

and:

```text
FAILED
```

must remain semantically distinct.

# 84. Preemption + Retry

Preemption may result in:

```text
PREEMPTED
```

which can be retried according to workload policy.

# 85. Checkpoint + Preemption

Preferred path for checkpointable Work:

```text
RUNNING
 ↓
CHECKPOINT
 ↓
PREEMPTED
 ↓
QUEUED
 ↓
RESCHEDULE
 ↓
RESUME
```

# 86. Execution Lifecycle Invariants

```text
1. Work identity is stable across executions.

2. Execution identity is unique.

3. Attempt identity is unique within an execution.

4. Scheduling intent is distinct from execution reality.

5. Dispatch is not equivalent to start.

6. ACK is not equivalent to running.

7. Running requires independent evidence.

8. Execution authority is time-bounded where leases are used.

9. Lease generation prevents stale holders from acting.

10. Scheduler epochs prevent stale scheduling decisions.

11. Dispatch must be idempotent.

12. Duplicate dispatch must not create duplicate execution.

13. Exactly-once semantics must never be claimed without proof.

14. Execution results are tied to execution identity.

15. Stale attempts cannot overwrite newer authoritative results.

16. Cancellation is distinct from failure.

17. Timeout types are explicitly distinguished.

18. Lease expiration is not proof of process termination.

19. Lost execution is distinct from confirmed termination.

20. Orphan executions must be detectable.

21. Orphan resources must be reclaimable.

22. Resource release must be verified.

23. Execution state transitions are explicit.

24. Illegal state transitions are rejected.

25. Execution state has an authoritative owner.

26. Observations do not automatically mutate authoritative state.

27. Recovery uses current observations rather than stale memory alone.

28. Contradictory observations become explicit reconciliation conflicts.

29. Retry attempts preserve historical evidence.

30. Execution lifecycle is fully traceable from Work to result and resource release.
```

# 87. Unified Execution Architecture

```text
                         WORK
                           │
                           ↓
                      SCHEDULING
                           │
                           ↓
                       ADMISSION
                           │
                           ↓
                      RESERVATION
                           │
                           ↓
                        DISPATCH
                           │
                           ↓
                          ACK
                           │
                           ↓
                        START
                           │
                           ↓
                         LEASE
                           │
                    ┌──────┴──────┐
                    ↓             ↓
                 HEARTBEAT      USAGE
                    │             │
                    └──────┬──────┘
                           ↓
                        RUNNING
                           │
          ┌────────────────┼────────────────┐
          ↓                ↓                ↓
       COMPLETE          FAIL           CANCEL
          │                │                │
          ↓                ↓                ↓
        RESULT           RETRY          TERMINATE
          │                │                │
          └────────────────┼────────────────┘
                           ↓
                        RELEASE
                           │
                           ↓
                       RECONCILE
                           │
                           ↓
                        FINALIZE
```

# 88. Core Execution Principle

> **Execution is not established by intent or message delivery; it is established through authoritative lifecycle transitions backed by leases, identity, liveness evidence, resource accounting, and verifiable completion.**

The complete NROS control path is now:

```text
                    ┌──────────────┐
                    │ Configuration│
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │     Work     │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │  Admission   │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │  Scheduling  │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │  Placement   │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │ Reservation  │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │   Dispatch   │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │   Execution  │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │ Observation  │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │Reconciliation│
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │   Finality   │
                    └──────────────┘
```

# Part XCVII — Execution Semantics, Concurrency, Cancellation & Failure Recovery

The next layer should formalize the **semantic guarantees** of NROS execution itself:

```text
Concurrency model
Atomicity
Isolation
Ordering
Visibility
Cancellation semantics
Failure semantics
Retry semantics
Recovery semantics
Idempotency
Deduplication
Commit points
Transactional boundaries
Partial failure
Compensation
Exactly-once result publication
At-least-once event delivery
At-most-once control operations
```

The central question becomes:

> **When NROS says that a Work item "executed," what exactly does that statement guarantee—and what evidence is required to prove it?**

# NROS — Part XCVII: Execution Semantics, Concurrency, Cancellation & Failure Recovery

The previous layer established **how execution becomes real**.

This layer establishes **what execution means** under concurrency, failure, retries, cancellation, partial completion, and recovery.

The fundamental distinction is:

```text
Execution intent
      ↓
Execution attempt
      ↓
Side effects
      ↓
Commit
      ↓
Observable result
```

NROS must never confuse:

```text
"the command was sent"
```

with:

```text
"the operation happened"
```

or:

```text
"the operation happened"
```

with:

```text
"its effects became authoritative."
```

# 1. Execution Semantics

NROS needs an explicit execution contract.

At minimum:

```text
ExecutionSemantics {
    delivery
    execution
    result
    side_effect
    cancellation
    retry
    recovery
}
```

Each dimension must be independently specified.

# 2. Delivery Semantics

Control messages can use:

```text
AT_MOST_ONCE
AT_LEAST_ONCE
DEDUPLICATED
```

For distributed execution, a practical model is often:

```text
control delivery = at-least-once
execution start = idempotent
```

# 3. Execution Semantics

Possible guarantees:

```text
MAY_RUN
AT_MOST_ONCE
AT_LEAST_ONCE
EFFECTIVELY_ONCE
EXACTLY_ONCE
```

The strongest labels should only be used when their conditions are actually enforceable.

# 4. Exactly-Once Is Not a Transport Property

A message arriving exactly once does not guarantee exactly-once execution.

Conversely:

```text
at-least-once delivery
+
deduplicated execution
+
idempotent effects
```

can provide effectively-once behavior.

# 5. Result Semantics

Result publication is another dimension:

```text
RESULT_MAY_BE_LOST
RESULT_AT_LEAST_ONCE
RESULT_DEDUPLICATED
RESULT_DURABLE
```

Execution and result durability must not be conflated.

# 6. Side-Effect Semantics

A Work item may have external effects:

```text
database write
file creation
API request
device command
message publication
financial operation
```

These effects require their own idempotency strategy.

# 7. Execution Boundary

A useful conceptual boundary:

```text
┌──────────────────────────────┐
│       NROS Execution         │
│                              │
│  prepare → execute → result  │
└───────────────┬──────────────┘
                │
                ↓
        External Side Effects
```

NROS cannot automatically make arbitrary external systems transactional.

# 8. Commit Point

Every execution should have a semantic commit point.

Before commit:

```text
result = provisional
```

After commit:

```text
result = authoritative
```

# 9. Commit Example

```text
RUNNING
   ↓
RESULT_GENERATED
   ↓
RESULT_VALIDATED
   ↓
RESULT_COMMITTED
```

Only after:

```text
RESULT_COMMITTED
```

should the execution be considered successfully finalized.

# 10. Atomicity

Atomicity asks:

> Does an operation happen completely or not at all?

For example:

```text
allocate CPU
allocate memory
allocate GPU
```

may need atomic treatment.

# 11. Partial Allocation

Bad state:

```text
CPU → allocated
RAM → allocated
GPU → failed
```

If the execution cannot proceed, the earlier allocations must be released or intentionally retained under a valid reservation.

# 12. Atomic Resource Reservation

Preferred:

```text
prepare all
    ↓
commit all
```

Failure:

```text
abort all
```

# 13. Isolation

Isolation determines whether concurrent executions can interfere.

Examples:

```text
Execution A
     X
Execution B
```

Potential isolation domains:

```text
process
container
VM
filesystem
network
memory
device
database
```

# 14. Execution Isolation Policy

A Work specification can declare:

```text
isolation = process
```

or:

```text
isolation = container
```

or:

```text
isolation = dedicated_device
```

# 15. Concurrency

NROS may execute many Work items simultaneously:

```text
W1 ────────────────>
W2 ────────>
W3 ─────────────>
W4 ───>
```

Concurrency must be constrained by:

```text
resources
dependencies
locks
quotas
policy
```

# 16. Dependency Ordering

Some Work has dependencies:

```text
W1
 ↓
W2
 ↓
W3
```

W2 cannot begin until W1 reaches its required completion state.

# 17. Dependency Types

Dependencies can mean different things:

```text
completion dependency
resource dependency
data dependency
ordering dependency
success dependency
```

These should not be represented by a single generic flag.

# 18. Success Dependency

Example:

```text
W2 starts only if W1 = COMPLETED_SUCCESSFULLY
```

If:

```text
W1 = FAILED
```

then W2 becomes:

```text
BLOCKED
```

# 19. Completion Dependency

Another Work may only require:

```text
W1 = TERMINAL
```

regardless of whether it succeeded.

This is different from success dependency.

# 20. Data Dependency

```text
W1
 ↓ output
W2
```

W2 must not consume the output until it is durably available.

# 21. Dependency Graph

A workflow can be represented as:

```text
       W1
      /  \
     ↓    ↓
    W2    W3
     \    /
      ↓  ↓
       W4
```

This forms a DAG when cycles are forbidden.

# 22. Dependency Cycle

Example:

```text
W1 → W2
W2 → W3
W3 → W1
```

This cannot make progress.

The scheduler should detect cycles before admission.

# 23. Dependency Failure Propagation

If:

```text
W1 = FAILED
```

then dependent Work may become:

```text
BLOCKED
```

rather than:

```text
FAILED
```

These states should remain distinct.

# 24. Blocked vs Failed

```text
BLOCKED
```

means:

> execution cannot currently proceed because a prerequisite is unresolved.

```text
FAILED
```

means:

> execution itself encountered a terminal failure.

# 25. Cancellation Semantics

Cancellation should have explicit meaning:

```text
CANCEL_REQUESTED
```

does not mean:

```text
CANCELLED
```

until execution termination is confirmed.

# 26. Cancellation Propagation

If:

```text
Parent Work
   ├── Child A
   └── Child B
```

is cancelled, policy must define whether children are:

```text
cancelled
detached
allowed to finish
```

# 27. Cancellation Reasons

Canonical reasons:

```text
USER_REQUEST
TIMEOUT
PREEMPTION
DEPENDENCY_FAILURE
POLICY
SYSTEM_SHUTDOWN
RESOURCE_FAILURE
```

# 28. Cooperative Cancellation

Preferred mechanism:

```text
cancel request
    ↓
application handles
    ↓
cleanup
    ↓
exit
```

This minimizes corrupted state.

# 29. Forced Cancellation

When cooperation fails:

```text
grace period
    ↓
forced termination
```

This is a different semantic event.

# 30. Cancellation Idempotency

Repeated cancellation:

```text
cancel(W)
cancel(W)
cancel(W)
```

must not produce:

```text
multiple termination operations
```

with inconsistent effects.

Instead:

```text
already_cancelled
```

or:

```text
cancellation_in_progress
```

# 31. Timeout Semantics

A timeout should identify what timed out.

```text
QUEUE_TIMEOUT
ADMISSION_TIMEOUT
DISPATCH_TIMEOUT
START_TIMEOUT
EXECUTION_TIMEOUT
RESULT_TIMEOUT
LEASE_TIMEOUT
```

# 32. Deadline vs Timeout

A deadline is an absolute point:

```text
2026-08-21T12:00:00Z
```

A timeout is usually relative:

```text
30 seconds
```

The internal model should support both.

# 33. Deadline Propagation

Parent Work:

```text
deadline = T
```

should constrain children:

```text
child_deadline ≤ T
```

unless policy explicitly permits a different boundary.

# 34. Failure Semantics

Failure should identify:

```text
source
category
cause
retryability
attempt
evidence
```

Example:

```text
Failure {
    category: RESOURCE
    code: GPU_LOST
    retryable: true
}
```

# 35. Failure Domains

Failure can originate from:

```text
workload
agent
resource
network
scheduler
storage
policy
external dependency
```

Correct classification improves recovery.

# 36. Transient Failure

Examples:

```text
temporary network outage
resource temporarily unavailable
agent restart
```

These may justify retry.

# 37. Permanent Failure

Examples:

```text
invalid configuration
unauthorized operation
malformed input
unsupported capability
```

Retrying may be pointless.

# 38. Failure Escalation

Repeated transient failures may become:

```text
attempt 1 → transient
attempt 2 → transient
attempt 3 → transient
```

Eventually:

```text
RETRY_EXHAUSTED
```

# 39. Retry Budget

Retries should be bounded:

```text
max_attempts
max_retry_duration
max_retry_cost
```

# 40. Retry Budget Example

```text
max_attempts = 5
```

means:

```text
attempt 1
attempt 2
attempt 3
attempt 4
attempt 5
→ terminal
```

# 41. Retry Backoff

A scheduler may use:

```text
backoff(attempt)
```

with:

```text
base
maximum
jitter
```

to avoid synchronized retry storms.

# 42. Retry Storm

If 10,000 Work items fail simultaneously and retry immediately:

```text
failure
 ↓
retry
 ↓
overload
 ↓
failure
 ↓
retry
```

This can destabilize the system.

Backoff is therefore a resilience mechanism.

# 43. Jitter

Without jitter:

```text
all retries → t = 10s
```

With jitter:

```text
9.2s
10.7s
11.3s
9.8s
```

This distributes load.

# 44. Circuit Breaking

Repeated external failures can activate:

```text
CLOSED
 ↓
OPEN
 ↓
HALF_OPEN
 ↓
CLOSED
```

This prevents continuously scheduling Work against an unhealthy dependency.

# 45. Bulkhead Isolation

Independent workload groups can be isolated:

```text
Pool A
Pool B
Pool C
```

Failure in A should not automatically exhaust B and C.

# 46. Concurrency Limits

A workload may define:

```text
max_concurrent_executions = 4
```

If 10 Work items exist:

```text
4 → running
6 → queued
```

# 47. Global Concurrency

The scheduler may also enforce:

```text
max_running_work = 1000
```

The effective limit becomes the minimum of applicable constraints.

# 48. Hierarchical Concurrency

Example:

```text
Global = 100

Tenant A = 40
Project A1 = 10
Agent A1 = 4
```

An execution must satisfy all relevant limits.

# 49. Locking

Some Work requires exclusive access:

```text
lock(resource-X)
```

The lock must have:

```text
owner
generation
lease
expiration
```

# 50. Lock vs Allocation

A lock controls logical exclusion.

An allocation controls resource authority.

They may coexist:

```text
resource allocation
+
logical lock
```

but should not be conflated.

# 51. Distributed Locks

Distributed locks require fencing.

A simple:

```text
"lock = true"
```

is insufficient.

Instead:

```text
lock generation = 42
```

must accompany operations.

# 52. Mutual Exclusion

For an exclusive resource:

```text
A holds resource
B waits
```

B must not enter the critical section until A's authority is safely released or fenced.

# 53. Ordering Guarantees

NROS may need:

```text
FIFO
causal ordering
per-work ordering
per-resource ordering
```

These should be explicit.

# 54. Per-Work Ordering

Events for one execution should have monotonic sequence:

```text
1 START
2 HEARTBEAT
3 CHECKPOINT
4 COMPLETE
```

# 55. Cross-Work Ordering

NROS should not assume:

```text
W1 event 5
```

must precede:

```text
W2 event 1
```

unless a dependency or explicit ordering constraint exists.

# 56. Causality

If:

```text
W1 → W2
```

then:

```text
completion(W1)
```

must causally precede:

```text
start(W2)
```

This can be represented through dependency evidence.

# 57. Visibility

An event becoming visible to one component does not necessarily mean it is durable.

Distinguish:

```text
observed
persisted
committed
published
```

# 58. Durable Completion

A strong completion guarantee requires:

```text
execution finished
+
result persisted
+
result committed
```

Only then:

```text
COMPLETED
```

may be authoritative.

# 59. Crash Between Execution and Commit

Suppose:

```text
execution finishes
 ↓
process crashes
 ↓
result not committed
```

The system must classify the execution as:

```text
UNKNOWN / RECOVERABLE
```

until reconciliation determines whether the result exists.

# 60. Unknown Is a Real State

NROS must support:

```text
UNKNOWN
```

rather than forcing:

```text
SUCCESS
```

or:

```text
FAILURE
```

without evidence.

# 61. Unknown Execution

Example:

```text
agent disconnected
lease expired
process state unknown
```

Correct state:

```text
EXECUTION_UNKNOWN
```

until evidence resolves it.

# 62. Recovery Decision

For unknown Work:

```text
recover existing
```

or:

```text
terminate old
and retry
```

must be policy-driven.

# 63. Duplicate Execution Risk

If an unknown execution might still be running, blindly retrying creates:

```text
old execution
+
new execution
```

This is dangerous for side-effecting Work.

# 64. Fencing Before Retry

Preferred sequence:

```text
UNKNOWN
 ↓
FENCE OLD AUTHORITY
 ↓
VERIFY
 ↓
RECLAIM
 ↓
NEW ATTEMPT
```

when the resource supports fencing.

# 65. Recovery Classes

Execution can be:

```text
RECOVERABLE
NON_RECOVERABLE
UNKNOWN
RETRYABLE
NON_RETRYABLE
```

These dimensions should not be collapsed into one boolean.

# 66. Compensation

Some operations cannot be rolled back.

Example:

```text
send_payment()
```

If execution succeeds but acknowledgement is lost, the system cannot simply execute it again.

Instead it may require:

```text
query status
```

or:

```text
compensating action
```

# 67. Saga-Like Semantics

A multi-step operation:

```text
A
 ↓
B
 ↓
C
```

may require compensations:

```text
C failed
 ↓
compensate B
 ↓
compensate A
```

NROS should model compensation separately from retry.

# 68. Retry vs Compensation

Retry means:

> attempt the same operation again.

Compensation means:

> perform another operation to neutralize an already-applied effect.

They are fundamentally different.

# 69. Side-Effect Journal

For important operations, record:

```text
operation_id
effect_type
target
status
idempotency_key
timestamp
```

This allows recovery after uncertain outcomes.

# 70. Idempotency Registry

External side-effect operations may use:

```text
idempotency_key = execution_id + operation_sequence
```

Repeated requests with the same key should return the existing outcome where the external system supports it.

# 71. Transaction Boundary

NROS should identify:

```text
transaction_start
transaction_commit
transaction_abort
```

but should not pretend arbitrary distributed operations form one atomic transaction.

# 72. Partial Success

A Work may contain several outputs:

```text
output A → committed
output B → failed
output C → committed
```

The execution model must support partial results when the Work contract permits them.

# 73. Work-Level Success

Work-level success should be defined by its specification:

```text
all outputs required
```

or:

```text
minimum successful outputs
```

or:

```text
best effort
```

# 74. Success Predicate

A Work can define:

```text
success_condition
```

rather than relying on:

```text
process_exit_code == 0
```

alone.

# 75. Process Exit Code

Exit code is useful evidence:

```text
0 → normal process termination
non-zero → process-level failure
```

but it does not universally determine Work success.

# 76. External Verification

For critical Work:

```text
process exited 0
```

may still require:

```text
output validation
```

before success.

# 77. Result Validation

Example:

```text
Execution
 ↓
output generated
 ↓
schema validation
 ↓
integrity check
 ↓
commit
```

Invalid output should not be treated as successful completion.

# 78. Integrity

Results may carry:

```text
checksum
content hash
signature
version
producer identity
```

This protects against corruption and stale publication.

# 79. Stale Result

Attempt 1:

```text
execution = old
```

publishes late after Attempt 2 has succeeded.

The system must reject the stale result using:

```text
attempt
generation
execution identity
```

# 80. Result Finality

Once:

```text
RESULT_COMMITTED
```

the result should become immutable unless an explicit correction protocol exists.

# 81. Correction

If correction is required:

```text
Result v1
 ↓
Correction
 ↓
Result v2
```

rather than silently modifying v1.

# 82. Auditability

Every important semantic event should be attributable:

```text
who
what
when
why
which execution
which generation
which policy
```

# 83. Execution Evidence Chain

A strong execution record looks like:

```text
Work
 ↓
Admission
 ↓
Scheduling Decision
 ↓
Reservation
 ↓
Dispatch
 ↓
ACK
 ↓
Start Evidence
 ↓
Lease
 ↓
Heartbeat
 ↓
Result
 ↓
Commit
 ↓
Release
 ↓
Final State
```

# 84. Semantic State Machine

```text
                 ┌───────────────┐
                 │    CREATED    │
                 └───────┬───────┘
                         ↓
                 ┌───────────────┐
                 │    ADMITTED   │
                 └───────┬───────┘
                         ↓
                 ┌───────────────┐
                 │  DISPATCHING  │
                 └───────┬───────┘
                         ↓
                 ┌───────────────┐
                 │   STARTING    │
                 └───────┬───────┘
                         ↓
                 ┌───────────────┐
                 │    RUNNING    │
                 └───────┬───────┘
                         │
             ┌───────────┼───────────┐
             ↓           ↓           ↓
        CHECKPOINT    CANCEL       FAILURE
             │           │           │
             ↓           ↓           ↓
         RUNNING      STOPPING     RETRY
             │           │           │
             └──────┬────┴──────┬────┘
                    ↓           ↓
               COMPLETION    RECOVERY
                    │           │
                    └─────┬─────┘
                          ↓
                      FINALIZE
```

# 85. Semantic Invariants

```text
1. Execution identity is immutable.

2. Attempt identity is immutable.

3. A dispatch request is not proof of execution.

4. An acknowledgement is not proof of execution.

5. Running state requires execution evidence.

6. Unknown state must remain representable.

7. Lease expiry does not itself prove process termination.

8. Stale execution authority must be fenceable.

9. Duplicate dispatch must be safe.

10. Retry must preserve historical attempts.

11. Retry must not blindly duplicate uncertain side effects.

12. Cancellation is distinct from failure.

13. Timeout categories remain distinct.

14. Dependency failure is distinct from execution failure.

15. Partial success is explicit when supported.

16. Result publication has its own commit semantics.

17. Stale results cannot overwrite newer authoritative results.

18. Result finality is explicit.

19. External side effects require idempotency or compensation where necessary.

20. Unknown outcomes require reconciliation before unsafe retry.

21. Concurrency limits are enforced hierarchically.

22. Exclusive resources require safe fencing.

23. Dependency cycles are rejected or explicitly handled.

24. Execution ordering is explicit.

25. Event ordering is not assumed across independent Work.

26. Recovery uses evidence rather than controller memory alone.

27. Forced termination is distinguishable from cooperative cancellation.

28. Retry is distinguishable from compensation.

29. Execution success is defined by the Work contract, not merely process exit code.

30. Every terminal state is supported by sufficient evidence.
```

# 86. NROS Execution Semantic Model

```text
                    WORK
                      │
                      ↓
                 EXECUTION
                      │
          ┌───────────┼───────────┐
          ↓           ↓           ↓
      CONTROL       STATE       EFFECTS
          │           │           │
          ↓           ↓           ↓
      dispatch     running      side-effect
      cancel       failed       journal
      retry        unknown      idempotency
      recover      complete     compensation
          │           │           │
          └───────────┼───────────┘
                      ↓
                  COMMIT MODEL
                      │
          ┌───────────┼───────────┐
          ↓           ↓           ↓
       success      failure      unknown
          │           │           │
          └───────────┼───────────┘
                      ↓
                  RECONCILIATION
                      │
                      ↓
                    FINALITY
```

# 87. Core Semantic Principle

> **NROS must distinguish intent, execution, observation, side effect, and finality; no stronger execution guarantee may be inferred merely from a weaker piece of evidence.**

This gives us the complete semantic progression:

```text
Resource
   ↓
Scheduling
   ↓
Admission
   ↓
Dispatch
   ↓
Execution
   ↓
Concurrency
   ↓
Failure
   ↓
Recovery
   ↓
Commit
   ↓
Finality
```

# Part XCVIII — Event Model, Evidence, Causality, Provenance & Deterministic Reconstruction

The next layer should formalize the **event and evidence substrate** underneath all of NROS.

That includes:

```text
Events
Event identity
Sequence numbers
Generations
Logical clocks
Causality
Correlation IDs
Trace IDs
Event sourcing
Snapshots
Durability
Ordering
Deduplication
Replay
Provenance
Evidence levels
State reconstruction
Audit trails
Tamper evidence
Reconciliation
Deterministic replay
```

The central question becomes:

> **Given the state of NROS at time T, can we reconstruct what happened, why it happened, which authority made the decision, and what evidence justified the resulting state?**

# NROS — Part XCVIII: Event Model, Evidence, Causality, Provenance & Deterministic Reconstruction

The execution model defines **what happens**.

The event model defines **how NROS knows that it happened**.

This is the foundation for:

```text
observability
auditability
recovery
debugging
replay
forensics
state reconstruction
deterministic verification
```

The core rule is:

> **State is authoritative only to the extent that NROS can explain the evidence and causal history that produced it.**

# 1. Event as a First-Class Object

An NROS event should not merely be a log line.

It is a structured protocol object:

```text
Event {
    event_id
    event_type
    entity_id
    entity_kind
    sequence
    generation
    timestamp
    actor
    correlation_id
    causation_id
    payload
    schema_version
}
```

# 2. Event Identity

Every event requires a unique:

```text
event_id
```

Example:

```text
evt_01JX...
```

This prevents ambiguity when identical payloads appear multiple times.

# 3. Event Identity ≠ Execution Identity

These are different:

```text
execution_id = exec_123
event_id     = evt_456
```

One execution produces many events:

```text
exec_123
 ├── evt_001 START
 ├── evt_002 HEARTBEAT
 ├── evt_003 CHECKPOINT
 ├── evt_004 RESULT
 └── evt_005 COMPLETE
```

# 4. Entity Identity

Events may refer to:

```text
Work
Execution
Attempt
Agent
Resource
Lease
Reservation
Scheduler
Policy
Result
```

Therefore:

```text
entity_kind
entity_id
```

should be explicit.

# 5. Event Type

Use canonical event types rather than free-form messages.

Examples:

```text
WORK_CREATED
WORK_ADMITTED
SCHEDULE_DECIDED
RESERVATION_CREATED
DISPATCH_SENT
DISPATCH_ACKED
EXECUTION_STARTED
LEASE_ISSUED
LEASE_RENEWED
HEARTBEAT_RECEIVED
CHECKPOINT_CREATED
RESULT_PREPARED
RESULT_COMMITTED
EXECUTION_COMPLETED
EXECUTION_FAILED
EXECUTION_CANCELLED
LEASE_EXPIRED
EXECUTION_LOST
EXECUTION_RECOVERED
```

# 6. Event Schema Version

Event schemas evolve.

Therefore:

```text
schema_version
```

must be part of the event envelope.

Example:

```text
event_type = EXECUTION_STARTED
schema_version = 2
```

# 7. Immutable Event Principle

Once committed, an event should be immutable.

Corrections should create new events:

```text
EVENT_A
   ↓
CORRECTION_FOR_EVENT_A
```

rather than modifying historical evidence.

# 8. Event Log

A conceptual event stream:

```text
t1  WORK_CREATED
t2  WORK_ADMITTED
t3  SCHEDULE_DECIDED
t4  RESERVATION_CREATED
t5  DISPATCH_SENT
t6  DISPATCH_ACKED
t7  EXECUTION_STARTED
t8  HEARTBEAT_RECEIVED
t9  RESULT_PREPARED
t10 RESULT_COMMITTED
t11 EXECUTION_COMPLETED
t12 RESOURCE_RELEASED
```

# 9. State as a Projection

Current state can be derived from events:

```text
Events
  ↓
Reducer
  ↓
State
```

For example:

```text
EXECUTION_STARTED
+
LEASE_ISSUED
+
HEARTBEAT_RECEIVED
```

projects to:

```text
RUNNING
```

# 10. Event Sourcing

NROS may use event sourcing for selected authoritative domains:

```text
event log
    ↓
projection
    ↓
current state
```

This provides historical reconstruction.

But event sourcing should not automatically be imposed on every subsystem.

# 11. Snapshotting

Long event streams become expensive to replay.

Use:

```text
Snapshot
+
Events after snapshot
```

Example:

```text
Snapshot @ seq=10
     +
events 11..25
     ↓
current state
```

# 12. Snapshot Integrity

A snapshot should identify:

```text
entity_id
snapshot_sequence
schema_version
state_hash
created_at
```

This allows verification that replay starts from a trustworthy state.

# 13. Replay

Given:

```text
snapshot
events
reducer
```

NROS should reconstruct:

```text
state(T)
```

deterministically.

# 14. Deterministic Reducer

For an authoritative state machine:

```text
state(n+1) = reduce(state(n), event(n))
```

The reducer should avoid hidden external dependencies.

Bad:

```text
reduce(state, event)
    → read current wall clock
```

Better:

```text
reduce(state, event.timestamp)
```

# 15. Deterministic Time

Events should carry their relevant timestamps.

Avoid reconstructing historical state using:

```text
now()
```

because replay at a later date would produce a different result.

# 16. Logical Time

NROS can use:

```text
sequence numbers
logical clocks
epochs
generations
```

to establish ordering independently of wall-clock precision.

# 17. Wall Clock

Wall-clock time remains useful for:

```text
diagnostics
latency
SLOs
expiration
human-readable audit
```

but should not be the sole mechanism for causal ordering.

# 18. Sequence Number

Per-stream sequence:

```text
1
2
3
4
5
```

allows consumers to detect:

```text
missing events
duplicates
reordering
```

# 19. Sequence Gap

If consumer receives:

```text
1
2
4
```

it knows:

```text
3
```

is missing.

The consumer should not silently assume the stream is complete.

# 20. Event Gap Handling

Possible response:

```text
GAP_DETECTED
      ↓
request replay
      ↓
receive event 3
      ↓
continue
```

# 21. Event Deduplication

If:

```text
evt_42
```

arrives twice:

```text
evt_42
evt_42
```

the second copy must be recognized as a duplicate.

# 22. Idempotent Consumer

Consumers should preferably process:

```text
event_id
```

idempotently.

This is essential under at-least-once event delivery.

# 23. Correlation ID

A correlation ID groups related operations:

```text
correlation_id = corr_abc
```

Example:

```text
API request
 ↓
Work creation
 ↓
Scheduling
 ↓
Execution
 ↓
Result
```

All can share the correlation ID.

# 24. Causation ID

The causation ID answers:

> Which event caused this event?

Example:

```text
EXECUTION_STARTED
```

may have:

```text
causation_id = DISPATCH_ACKED.event_id
```

# 25. Correlation vs Causation

Correlation:

```text
same broader operation
```

Causation:

```text
direct causal predecessor
```

These must not be conflated.

# 26. Causal Chain

Example:

```text
WORK_CREATED
      ↓
SCHEDULE_DECIDED
      ↓
DISPATCH_SENT
      ↓
EXECUTION_STARTED
      ↓
RESULT_COMMITTED
```

Every event can point toward its causal predecessor.

# 27. Causal Graph

Instead of only a linear log:

```text
A → B → C
```

distributed execution may produce:

```text
       A
      / \
     B   C
      \ /
       D
```

NROS therefore has a causal graph rather than necessarily one global ordering.

# 28. No Global Total Order Assumption

Independent events:

```text
W1_HEARTBEAT
W2_HEARTBEAT
```

do not necessarily require:

```text
W1 < W2
```

or:

```text
W2 < W1
```

They may be concurrent.

# 29. Happens-Before

A useful relation:

```text
A → B
```

means:

> A happened-before B.

If neither:

```text
A → B
```

nor:

```text
B → A
```

is established, they may be concurrent.

# 30. Concurrency Detection

This distinction matters for:

```text
conflict resolution
replay
distributed scheduling
debugging
```

because concurrent operations must not be arbitrarily interpreted as causally ordered.

# 31. Lamport-Style Logical Ordering

A logical clock can provide a deterministic partial ordering:

```text
logical_time
```

but it does not prove physical simultaneity or causality by itself.

# 32. Generation

Generations identify authority revisions:

```text
generation = 1
generation = 2
generation = 3
```

A higher generation supersedes lower generations where the protocol says so.

# 33. Epoch

An epoch generally identifies a broader authority period:

```text
scheduler_epoch = 51
```

A scheduler restart may produce:

```text
scheduler_epoch = 52
```

# 34. Generation vs Epoch

Conceptually:

```text
epoch
  → authority regime

generation
  → revision within an authority domain
```

The exact ownership should be specified by the NROS protocol rather than left ambiguous.

# 35. Evidence

An event is not necessarily equivalent to truth.

For example:

```text
AGENT_REPORTS_RUNNING
```

is evidence from the Agent.

It is not necessarily proof that:

```text
resource manager
```

still considers the execution valid.

# 36. Evidence Source

Each evidence item should identify:

```text
source
source_type
observation_time
received_time
confidence
```

# 37. Evidence Types

Possible sources:

```text
scheduler
agent
resource_manager
result_store
network
external_system
operator
```

# 38. Evidence Strength

A useful evidence ladder:

```text
UNKNOWN
   ↓
REQUESTED
   ↓
OBSERVED
   ↓
ACKNOWLEDGED
   ↓
VALIDATED
   ↓
COMMITTED
   ↓
FINAL
```

Not every domain needs every level.

# 39. Observation vs Authority

Suppose Agent reports:

```text
RUNNING
```

but the lease is expired.

Then:

```text
observation = process may exist
authority    = no longer valid
```

Both facts can coexist.

# 40. Evidence Bundle

A state transition can reference evidence:

```text
EvidenceBundle {
    evidence_ids
    transition
    evaluator
    evaluation_time
}
```

Example:

```text
RUNNING → COMPLETED
```

supported by:

```text
process_exit
+
result_hash
+
result_commit
```

# 41. Provenance

Provenance answers:

```text
Where did this state come from?
```

For example:

```text
COMPLETED
  ← RESULT_COMMITTED
      ← RESULT_VALIDATED
          ← EXECUTION_FINISHED
```

# 42. Provenance Graph

```text
Input
 ↓
Work
 ↓
Execution
 ↓
Intermediate Artifact
 ↓
Result
 ↓
Published Artifact
```

Each edge should be traceable.

# 43. Artifact Identity

Artifacts require stable identity:

```text
artifact_id
content_hash
producer_execution_id
schema_version
```

# 44. Artifact Lineage

Example:

```text
artifact-A
   ↓ transform
artifact-B
   ↓ aggregate
artifact-C
```

NROS should be able to answer:

> Which executions produced artifact C?

and:

> Which inputs contributed to artifact C?

# 45. Data Provenance

For data-producing Work:

```text
input references
processing execution
code/version
configuration
output hash
```

should be recorded.

# 46. Configuration Provenance

Execution should reference the exact configuration revision:

```text
config_revision
```

not merely:

```text
config_name = production
```

Otherwise historical replay becomes ambiguous.

# 47. Code Provenance

Likewise:

```text
code_revision
runtime_version
dependency_set
```

should be associated with execution where reproducibility matters.

# 48. Environment Provenance

Record relevant environment:

```text
agent_version
OS/runtime
capabilities
resource class
security profile
```

The exact level depends on the reproducibility requirements.

# 49. Policy Provenance

An execution should identify:

```text
policy_revision
```

because policy may change while Work is running.

# 50. Scheduling Provenance

Likewise:

```text
scheduler_revision
placement_decision_id
```

connect the execution to the scheduling decision that produced it.

# 51. Decision Record

A scheduling decision can contain:

```text
Decision {
    decision_id
    work_id
    candidates
    selected_target
    constraints
    policy_revision
    scheduler_epoch
    reason
}
```

This makes placement explainable.

# 52. Why Was This Agent Selected?

NROS should eventually answer:

```text
Why agent A?
```

with evidence such as:

```text
capability match
resource availability
affinity
anti-affinity
priority
policy
cost
load
```

rather than:

```text
because scheduler chose it.
```

# 53. Explainability

The scheduler should expose a machine-readable decision explanation:

```text
DecisionExplanation {
    satisfied_constraints
    rejected_candidates
    scoring_factors
}
```

# 54. Rejected Candidate

Example:

```text
Agent A → rejected: insufficient memory
Agent B → rejected: capability missing
Agent C → selected
```

This is valuable for debugging scheduling behavior.

# 55. Event Store

The event store should support:

```text
append
read_by_stream
read_by_sequence
read_by_time
read_by_correlation
read_by_execution
```

# 56. Append-Only Semantics

The canonical event stream should be append-only:

```text
E1
E2
E3
E4
```

Never:

```text
edit E2
```

Instead:

```text
E2
E5 = correction(E2)
```

# 57. Event Integrity

Events may include:

```text
payload_hash
previous_event_hash
```

creating a tamper-evident chain:

```text
E1.hash
   ↓
E2.previous_hash
   ↓
E3.previous_hash
```

# 58. Hash Chain

Conceptually:

```text
H1 = hash(E1)

H2 = hash(E2 || H1)

H3 = hash(E3 || H2)
```

Tampering with E2 invalidates subsequent hashes.

# 59. Hash Chain ≠ Security Boundary

A hash chain provides tamper evidence.

It does not by itself provide:

```text
authentication
authorization
confidentiality
```

Those require separate mechanisms.

# 60. Event Authentication

Where required, events may carry:

```text
signature
signer_id
key_id
```

This allows verification of event origin.

# 61. Trusted Event Producers

NROS should maintain explicit producer identities:

```text
scheduler-01
agent-17
resource-manager-02
result-store-01
```

rather than trusting arbitrary event claims.

# 62. Event Admission

Incoming events should be validated:

```text
identity
schema
authorization
generation
sequence
signature
```

before entering the authoritative stream.

# 63. Invalid Event

Invalid event should become:

```text
EVENT_REJECTED
```

with a reason.

It should not silently disappear.

# 64. Event Rejection Evidence

Record:

```text
rejected_event_id
producer
reason
validator
timestamp
```

This is important for audit and debugging.

# 65. Event Ordering Per Stream

A stream can guarantee:

```text
monotonic sequence
```

without requiring global ordering across the entire cluster.

This scales better.

# 66. Partitioned Streams

Example:

```text
stream: execution/exec-001
stream: execution/exec-002
stream: execution/exec-003
```

Each stream can maintain local ordering.

# 67. Global Coordination

Only operations requiring global ordering should introduce global coordination.

Avoid creating:

```text
one global event lock
```

for every event.

# 68. Event Backpressure

If consumers are slower than producers:

```text
producer rate > consumer rate
```

the system needs:

```text
buffering
backpressure
consumer lag metrics
retention policy
```

# 69. Consumer Lag

Track:

```text
latest_event
consumer_position
lag
```

Example:

```text
latest = 10000
consumer = 9975
lag = 25
```

# 70. Retention

Events may require:

```text
hot retention
cold archival
compaction
snapshotting
```

Retention policy must not destroy evidence required for compliance or recovery.

# 71. Compaction

If snapshots are authoritative, old events may eventually be compacted.

But compaction must preserve enough provenance to reconstruct the state within the declared retention guarantees.

# 72. Replay Modes

NROS may support:

```text
FULL_REPLAY
SNAPSHOT_REPLAY
TIME_RANGE_REPLAY
EXECUTION_REPLAY
CAUSAL_REPLAY
```

# 73. Execution Replay

To reconstruct one execution:

```text
filter by execution_id
        ↓
sort by causal/stream order
        ↓
apply reducer
        ↓
reconstruct lifecycle
```

# 74. Causal Replay

For debugging a failure:

```text
failure event
    ↓
causation chain
    ↓
upstream events
```

rather than replaying the entire cluster.

# 75. Deterministic Reconstruction

Given the same:

```text
snapshot
event sequence
schema versions
reducer version
```

NROS should produce:

```text
same resulting state
```

# 76. Reducer Version

State interpretation may evolve.

Therefore snapshots/replay should identify:

```text
reducer_version
```

or equivalent migration semantics.

# 77. Schema Migration

Old event:

```text
schema v1
```

may be upgraded to:

```text
canonical v3
```

during replay through explicit migration:

```text
v1 → v2 → v3
```

# 78. Never Silent Migration

A migration should be explicit and testable.

Bad:

```text
if missing_field:
    guess()
```

Better:

```text
migrate_v1_to_v2(event)
```

with deterministic rules.

# 79. State Reconstruction Test

A strong invariant:

```text
persisted_state
        ==
replay(event_log)
```

If they differ:

```text
STATE_RECONSTRUCTION_MISMATCH
```

must be surfaced.

# 80. Snapshot Verification

Likewise:

```text
snapshot_hash
```

must validate before replay.

If invalid:

```text
SNAPSHOT_CORRUPTED
```

# 81. Audit Trail

The audit trail should answer:

```text
WHO
WHAT
WHEN
WHY
ON WHICH ENTITY
UNDER WHICH AUTHORITY
BASED ON WHICH EVIDENCE
```

# 82. Audit Event

Example:

```text
AuditEvent {
    actor
    action
    target
    reason
    authority
    evidence
    timestamp
}
```

# 83. Human Actions

Operator actions must be events too:

```text
OPERATOR_CANCELLED
OPERATOR_REQUEUED
OPERATOR_FENCED
OPERATOR_OVERRIDE
```

This prevents manual actions from becoming invisible state mutations.

# 84. Automated Actions

Likewise:

```text
AUTOMATED_RETRY
AUTOMATED_RECOVERY
AUTOMATED_REBALANCE
AUTOMATED_RECLAMATION
```

should identify the responsible controller/policy.

# 85. Policy-Driven Action

An automated transition should reference:

```text
policy_id
policy_revision
trigger_event
decision
```

# 86. Why Did NROS Retry?

The system should be able to produce:

```text
retry_reason:
    failure = NETWORK_TIMEOUT
    retryable = true
    attempt = 2
    policy = retry-policy-v4
    budget_remaining = 3
```

This makes automated behavior explainable.

# 87. Evidence Graph

At the highest level:

```text
                  ┌──────────────┐
                  │    POLICY    │
                  └──────┬───────┘
                         │
                         ↓
                  ┌──────────────┐
                  │   DECISION   │
                  └──────┬───────┘
                         │
                         ↓
Work ───────────────→ Execution
                         │
               ┌─────────┼─────────┐
               ↓         ↓         ↓
          Observation  Result    Resource
               │         │         │
               └─────────┼─────────┘
                         ↓
                      Finality
```

Every important state should be traceable through this graph.

# 88. Reconstruction Query

NROS should support a conceptual query:

```text
reconstruct(execution_id, at_time)
```

Result:

```text
state
authority
evidence
causal_chain
resources
policy
decision
```

# 89. "What Happened?"

Query:

```text
explain(execution_id)
```

should return:

```text
creation
admission
placement
dispatch
start
runtime
failure/recovery
result
finalization
```

# 90. "Why Is It Running?"

The answer should include:

```text
current state
lease
agent
scheduling decision
policy
latest heartbeat
resource allocation
```

# 91. "Why Did It Fail?"

The answer should include:

```text
terminal event
failure classification
causal chain
relevant observations
retry policy
previous attempts
```

# 92. "Can We Trust This Result?"

Evidence should include:

```text
producer
execution identity
attempt
result hash
commit event
schema version
policy revision
```

# 93. Evidence Completeness

NROS can define a completeness predicate:

```text
complete(execution) =
    identity_valid
    ∧ start_evidence
    ∧ terminal_evidence
    ∧ result_evidence
    ∧ authority_valid
```

The exact predicate depends on the execution class.

# 94. Evidence Monotonicity

Evidence should generally move toward stronger states:

```text
UNKNOWN
 ↓
OBSERVED
 ↓
VALIDATED
 ↓
COMMITTED
 ↓
FINAL
```

A later weaker observation should not silently downgrade authoritative finality.

# 95. Contradictory Evidence

Example:

```text
ResultStore → committed
Agent → process failed
```

This does not necessarily mean the result is invalid.

The system must understand the semantics:

```text
process failure after result commit
```

may be a normal post-commit termination.

# 96. Evidence Resolution

A reconciliation engine evaluates:

```text
observations
+
authority
+
causality
+
generation
+
policy
```

and produces:

```text
resolved state
```

# 97. Resolution Event

The resolution itself should be recorded:

```text
STATE_RECONCILED {
    previous_observations
    selected_state
    rule
    policy_revision
}
```

# 98. Deterministic Reconciliation

Given identical evidence:

```text
E
```

the reconciliation function should produce the same:

```text
state
```

This is crucial for reproducibility.

# 99. Nondeterministic Inputs

If reconciliation depends on:

```text
current wall clock
random number
unordered iteration
```

then replay may diverge.

Such inputs must be captured explicitly.

# 100. Randomness

If randomness affects an authoritative decision:

```text
random_seed
```

or the resulting random choice must be persisted.

# 101. External Queries

If a historical decision depended on an external system:

```text
resource availability
policy service
capability registry
```

the relevant response should be captured as evidence.

Otherwise deterministic reconstruction may become impossible.

# 102. Decision Evidence

A decision should include:

```text
inputs
constraints
candidate set
selected candidate
algorithm version
random seed if relevant
policy revision
```

# 103. Algorithm Version

Scheduling and reconciliation algorithms evolve.

Record:

```text
algorithm_version
```

with decisions where reproducibility matters.

# 104. Replay Safety

Replay must never accidentally perform real-world side effects.

Therefore:

```text
REPLAY
```

must be semantically different from:

```text
LIVE_EXECUTION
```

# 105. Side-Effect Firewall

During replay:

```text
event
 ↓
reducer
 ↓
state
```

not:

```text
event
 ↓
real API call
```

# 106. Simulation

NROS can support:

```text
simulation mode
```

where decisions are computed but not committed to real resources.

This is useful for:

```text
testing
capacity planning
policy validation
```

# 107. Event-Based Testing

A state machine can be tested through event sequences:

```text
START
HEARTBEAT
RESULT
COMMIT
```

Expected:

```text
COMPLETED
```

# 108. Invalid Event Sequence

Example:

```text
COMPLETE
START
```

should produce:

```text
INVALID_TRANSITION
```

not silently reconstruct:

```text
RUNNING
```

# 109. Property Testing

Important properties include:

```text
replay determinism
idempotent event handling
monotonic sequences
no impossible transitions
no stale authority acceptance
```

# 110. Fault Injection

Test:

```text
message loss
message duplication
message reordering
agent crash
scheduler crash
storage failure
network partition
lease expiration
clock skew
```

# 111. Reconstruction Under Fault

Example:

```text
DISPATCH_SENT
ACK_LOST
AGENT_STARTED
SCHEDULER_RESTART
```

The recovered state should be derivable from surviving evidence.

# 112. Eventual Reconciliation

Temporary disagreement is allowed:

```text
controller ≠ agent
```

during a partition.

The system should converge after connectivity returns, provided authority and fencing rules are correct.

# 113. Convergence Invariant

After successful reconciliation:

```text
all authoritative projections
```

should converge to:

```text
one valid state
```

or explicitly remain:

```text
CONFLICT
```

if resolution is impossible.

# 114. No Silent Conflict

Never do:

```text
if disagreement:
    choose_local_state()
```

without recording why.

Instead:

```text
CONFLICT_DETECTED
 ↓
RESOLUTION
 ↓
STATE_RECONCILED
```

# 115. Event Model Invariants

```text
1. Every authoritative event has a unique identity.

2. Events are immutable after commitment.

3. Corrections are represented by new events.

4. Event schemas are versioned.

5. Event streams provide explicit ordering semantics.

6. Consumers tolerate duplicate delivery.

7. Missing events are detectable.

8. Correlation and causation are distinct.

9. Independent events are not assumed to have global ordering.

10. State reconstruction is deterministic.

11. Snapshots are verifiable.

12. Replay never performs unintended real-world side effects.

13. Authoritative decisions retain sufficient input evidence.

14. External dependencies relevant to historical decisions are captured.

15. State cannot be claimed stronger than its supporting evidence.

16. Unknown remains a valid state.

17. Conflicting observations are explicit.

18. Reconciliation is deterministic for identical inputs.

19. Operator actions are auditable.

20. Automated actions identify their policy and trigger.

21. Results have provenance.

22. Artifacts have lineage.

23. Final state can be traced to causal evidence.

24. Event integrity is independently verifiable.

25. Event producer identity is authenticated where required.

26. Historical execution can be reconstructed.

27. Scheduling decisions can be explained.

28. Retry decisions can be explained.

29. Resource decisions can be explained.

30. The system can answer "what happened?" without relying on ephemeral logs alone.
```

# 116. Unified NROS Evidence Architecture

```text
                         EVENT
                           │
            ┌──────────────┼──────────────┐
            ↓              ↓              ↓
        Identity        Causality      Evidence
            │              │              │
            └──────────────┼──────────────┘
                           ↓
                       EVENT LOG
                           │
                    ┌──────┴──────┐
                    ↓             ↓
                SNAPSHOT        REPLAY
                    │             │
                    └──────┬──────┘
                           ↓
                        REDUCER
                           │
                           ↓
                         STATE
                           │
                    ┌──────┴──────┐
                    ↓             ↓
               RECONCILIATION   AUDIT
                    │
                    ↓
                 FINALITY
```

# 117. Complete NROS Provenance Chain

We can now connect the layers developed so far:

```text
Policy
  ↓
Work Specification
  ↓
Admission
  ↓
Scheduling Decision
  ↓
Placement
  ↓
Reservation
  ↓
Dispatch
  ↓
Execution
  ↓
Lease
  ↓
Observation
  ↓
Side Effects
  ↓
Result
  ↓
Commit
  ↓
Resource Release
  ↓
Reconciliation
  ↓
Final State
```

Every arrow can produce evidence.

Every state can therefore have:

```text
identity
authority
causality
provenance
evidence
```

# 118. The NROS Evidence Principle

> **NROS should never require operators, agents, or future recovery logic to infer historical truth from logs when that truth can be represented as structured, immutable, causally linked evidence.**

This turns the system from:

```text
distributed runtime + logs
```

into:

```text
distributed runtime + verifiable state history
```

# Part XCIX — Control Plane / Data Plane / Evidence Plane

The next architectural boundary should separate three concerns:

```text
CONTROL PLANE
    decisions
    policy
    scheduling
    lifecycle authority

DATA PLANE
    execution
    resources
    workloads
    artifacts

EVIDENCE PLANE
    events
    provenance
    audit
    reconstruction
    verification
```

The key question becomes:

> **How can these three planes evolve, fail, restart, and scale independently without allowing evidence, execution, and authority to become inconsistent?**

# NROS — Part XCIX: Control Plane, Data Plane & Evidence Plane

We now separate NROS into three fundamental planes:

```text
┌──────────────────────────────────────────────┐
│                 CONTROL PLANE                │
│                                              │
│ policy • admission • scheduling • authority  │
└──────────────────────┬───────────────────────┘
                       │
                 commands / leases
                       │
                       ↓
┌──────────────────────────────────────────────┐
│                  DATA PLANE                  │
│                                              │
│ agents • execution • resources • artifacts  │
└──────────────────────┬───────────────────────┘
                       │
                 observations / results
                       │
                       ↓
┌──────────────────────────────────────────────┐
│                EVIDENCE PLANE                │
│                                              │
│ events • provenance • audit • replay        │
└──────────────────────────────────────────────┘
```

The planes interact, but **none should be mistaken for another**.

# 1. Control Plane

The control plane answers:

> What should happen?

Responsibilities include:

```text
admission
scheduling
placement
resource policy
authorization
lease management
retry policy
recovery policy
cancellation
reconciliation
```

It owns **intent and authority**.

# 2. Data Plane

The data plane answers:

> What is actually executing?

It contains:

```text
agents
workers
processes
containers
devices
network paths
storage
artifacts
runtime environments
```

The data plane performs the actual work.

# 3. Evidence Plane

The evidence plane answers:

> What can we prove happened?

It contains:

```text
events
observations
execution records
audit records
provenance
snapshots
checksums
decision records
reconciliation records
```

# 4. Critical Separation

A controller saying:

```text
RUNNING
```

belongs to the control plane.

A process actually running belongs to:

```text
DATA PLANE
```

Evidence that connects the two belongs to:

```text
EVIDENCE PLANE
```

This separation prevents a common distributed-systems mistake:

```text
desired state == observed state
```

# 5. Desired State

The control plane may declare:

```text
desired_state = RUNNING
```

This means:

> NROS wants this Work executing.

It does **not** mean execution is already happening.

# 6. Observed State

An Agent may report:

```text
observed_state = RUNNING
```

This means:

> The Agent currently observes an execution.

Still, the observation must be validated against authority.

# 7. Effective State

The reconciler combines:

```text
desired state
+
observed state
+
lease
+
evidence
+
generation
```

to derive:

```text
effective state
```

# 8. State Triad

This gives:

```text
Desired
   │
   ├──────────────┐
   ↓              ↓
Observed      Authorized
   │              │
   └───────┬──────┘
           ↓
      Reconciled
         State
```

This should become a central NROS abstraction.

# 9. Control Plane Does Not Execute

The controller should generally issue:

```text
START
STOP
PAUSE
RESUME
CANCEL
```

rather than directly becoming the execution process.

This keeps execution failures isolated from control logic.

# 10. Data Plane Does Not Define Policy

An Agent should not independently decide:

```text
"this Work has unlimited resources"
```

unless the protocol explicitly grants that authority.

The Agent executes within delegated authority.

# 11. Evidence Plane Does Not Become Authority

An event such as:

```text
AGENT_REPORTED_RUNNING
```

is evidence.

It should not automatically mutate authoritative control state without validation.

# 12. Command Flow

A normal start operation becomes:

```text
Control
  │
  │ START
  ↓
Agent
  │
  │ execution
  ↓
Data Plane
  │
  │ evidence
  ↓
Evidence Plane
  │
  │ observation
  ↓
Control
```

This is a feedback loop.

# 13. Closed-Loop Architecture

NROS therefore becomes:

```text
       ┌───────────────┐
       │    CONTROL    │
       └───────┬───────┘
               │ intent
               ↓
       ┌───────────────┐
       │     DATA      │
       └───────┬───────┘
               │ observation
               ↓
       ┌───────────────┐
       │   EVIDENCE    │
       └───────┬───────┘
               │ reconciliation
               └──────────────→ CONTROL
```

# 14. Command vs Event

This distinction is fundamental.

A command says:

```text
DO X
```

An event says:

```text
X HAPPENED
```

Examples:

```text
START_EXECUTION
```

versus:

```text
EXECUTION_STARTED
```

# 15. Commands Are Imperative

Commands represent requested actions:

```text
START
CANCEL
RENEW_LEASE
RELEASE_RESOURCE
```

They may fail.

# 16. Events Are Historical

Events represent observations or committed transitions:

```text
EXECUTION_STARTED
LEASE_RENEWED
RESOURCE_RELEASED
```

They should not be treated as requests.

# 17. Command Lifecycle

```text
Command Created
      ↓
Authorized
      ↓
Dispatched
      ↓
Accepted
      ↓
Executed
      ↓
Evidence Produced
      ↓
Result Event
```

# 18. Command Identity

Every command requires:

```text
command_id
```

This permits deduplication.

# 19. Command Correlation

A command should reference:

```text
work_id
execution_id
attempt_id
generation
issuer
policy
```

where applicable.

# 20. Stale Command

Suppose:

```text
generation = 7
```

is replaced by:

```text
generation = 8
```

A delayed command from generation 7 must not mutate generation 8.

This is an essential fencing rule.

# 21. Generation Fence

Agent receives:

```text
START(exec-A, generation=7)
```

but currently owns:

```text
generation=8
```

Therefore:

```text
REJECT_STALE_COMMAND
```

# 22. Lease as Authority Token

A lease can carry:

```text
lease_id
subject
generation
issued_by
issued_at
expires_at
capabilities
```

The Agent acts under that authority.

# 23. Lease Expiration

Expiration means:

```text
authority expired
```

not necessarily:

```text
process terminated
```

The distinction established earlier remains critical.

# 24. Data Plane After Lease Expiration

If the Agent continues executing after lease expiry:

```text
process exists
```

but:

```text
authority = invalid
```

The control plane may need fencing.

# 25. Fencing Token

A monotonically increasing token:

```text
token = 100
```

then:

```text
token = 101
```

allows external resources to reject operations from stale owners.

# 26. Fencing Example

Agent A:

```text
token = 10
```

Agent B later receives:

```text
token = 11
```

A's subsequent write:

```text
WRITE(token=10)
```

must be rejected by a fencing-aware resource.

# 27. Control Plane Failure

Suppose the controller crashes:

```text
CONTROL DOWN
```

The data plane may continue temporarily under existing leases.

Therefore NROS must define:

```text
lease grace
renewal behavior
execution behavior
```

during control-plane outages.

# 28. Agent Failure

If an Agent disappears:

```text
DATA PLANE FAILURE
```

the control plane should not immediately assume:

```text
execution = failed
```

It should enter:

```text
UNKNOWN
```

and reconcile.

# 29. Evidence Store Failure

If the evidence plane becomes unavailable:

```text
EVIDENCE UNAVAILABLE
```

NROS must decide whether execution can continue.

This is a major architectural policy.

# 30. Evidence Dependency Modes

Possible modes:

```text
STRICT
DEGRADED
BUFFERED
BEST_EFFORT
```

# 31. Strict Evidence Mode

Critical operations may require:

```text
event persisted
```

before continuing.

Example:

```text
resource allocation
     ↓
persist evidence
     ↓
execution
```

# 32. Buffered Evidence Mode

The data plane can temporarily buffer evidence:

```text
execution
   ↓
local durable journal
   ↓
evidence plane
```

This improves availability but introduces recovery complexity.

# 33. Volatile Evidence

Least reliable:

```text
execution
 ↓
memory
 ↓
event plane unavailable
 ↓
crash
```

Evidence is lost.

This should be explicitly classified as a weaker guarantee.

# 34. Plane Availability Matrix

| Plane | Failure | Immediate concern |
|---|---|---|
| Control | controller unavailable | authority/reconciliation |
| Data | agent unavailable | execution uncertainty |
| Evidence | event store unavailable | audit/recovery uncertainty |
| Control + Evidence | both unavailable | autonomous execution policy |
| Data + Evidence | agent gone | unknown outcome |
| All | total outage | persistent recovery |

# 35. Independent Scaling

Control plane workload:

```text
scheduling decisions
```

Data plane workload:

```text
CPU / memory / I/O
```

Evidence workload:

```text
events / writes / queries
```

These have different scaling characteristics.

# 36. Control Plane Scaling

Possible architecture:

```text
Scheduler
   ├── shard A
   ├── shard B
   └── shard C
```

Work ownership should be deterministic.

# 37. Data Plane Scaling

Agents can scale independently:

```text
Agent 1
Agent 2
...
Agent N
```

without requiring proportional controller replication.

# 38. Evidence Plane Scaling

Event streams can be partitioned by:

```text
tenant
work
execution
agent
resource
```

depending on query patterns.

# 39. Partition Ownership

A control-plane shard should have explicit authority over its partition:

```text
partition = P7
owner = scheduler-03
epoch = 19
```

# 40. Ownership Transfer

When ownership changes:

```text
Scheduler A
   ↓
handoff
   ↓
Scheduler B
```

the new owner must obtain a higher authority generation.

# 41. Split Brain

Dangerous state:

```text
Scheduler A → owns P7
Scheduler B → also believes it owns P7
```

Both may issue conflicting commands.

NROS needs an ownership/fencing mechanism.

# 42. Authority Epoch

Example:

```text
epoch 10 → Scheduler A
epoch 11 → Scheduler B
```

All actions from epoch 10 become stale after epoch 11 is established.

# 43. Control-Plane Leader Election

If using leader-based coordination:

```text
Candidate
   ↓
Leader
   ↓
Follower
```

leadership must be tied to an explicit authority generation.

# 44. Leadership Is Not Ownership

A scheduler can be cluster leader without necessarily owning every Work partition.

Keep:

```text
cluster leadership
```

distinct from:

```text
partition ownership
```

# 45. Evidence of Leadership

Leadership changes should generate:

```text
LEADERSHIP_ACQUIRED
LEADERSHIP_LOST
OWNERSHIP_TRANSFERRED
```

with:

```text
epoch
leader
partition
```

# 46. Data Plane Registration

An Agent should register capabilities:

```text
agent_id
capabilities
resources
runtime_version
protocol_version
```

and obtain an identity/authority context.

# 47. Capability Advertisement

Example:

```text
Agent A:
    cpu = 8
    memory = 32GiB
    gpu = none
    features = [...]
```

This is an observation.

The scheduler may cache it, but must account for staleness.

# 48. Capability Freshness

Every capability observation should have:

```text
observed_at
expires_at
generation
```

or equivalent freshness semantics.

# 49. Stale Capability

Scheduler receives:

```text
GPU = available
```

but the observation is 10 minutes old.

It should not necessarily treat this as current truth.

# 50. Resource State

Resources have at least:

```text
capacity
allocated
available
reserved
unavailable
```

These are different dimensions.

# 51. Resource Reservation

Reservation belongs primarily to control:

```text
reserve resource
```

Actual consumption belongs to data plane:

```text
consume resource
```

Evidence records both.

# 52. Resource Reconciliation

Example:

```text
Control:
allocated = 4 CPUs

Agent:
consuming = 6 CPUs
```

This is a discrepancy.

NROS must detect:

```text
RESOURCE_DRIFT
```

# 53. Resource Drift

Possible causes:

```text
external workload
agent bug
stale accounting
race
unauthorized process
```

The reconciler should classify rather than silently overwrite.

# 54. Desired vs Actual Resource State

```text
Desired:
CPU = 4

Actual:
CPU = 6

Effective:
DRIFT
```

The system may then:

```text
correct
quarantine
alert
reconcile
```

depending on policy.

# 55. Evidence Plane as Black Box

A particularly useful property:

The evidence plane should be able to record:

```text
control event
data observation
resource evidence
```

without becoming responsible for executing them.

This reduces coupling.

# 56. Event Ingestion

```text
Control ──────┐
              │
Data ─────────┼──→ Event Gateway → Evidence Store
              │
External ─────┘
```

The gateway validates:

```text
identity
schema
authorization
sequence
generation
```

# 57. Event Publication

Events may be published:

```text
synchronously
asynchronously
batched
streamed
```

The chosen mode must be reflected in durability guarantees.

# 58. Synchronous Evidence

```text
action
 ↓
event commit
 ↓
continue
```

Stronger evidence guarantees, higher latency.

# 59. Asynchronous Evidence

```text
action
 ↓
continue
 ↓
event eventually committed
```

Higher availability, weaker immediate durability.

# 60. Command/Event Duality

A complete operation therefore becomes:

```text
CONTROL
   │
   │ command
   ↓
DATA
   │
   │ observation
   ↓
EVIDENCE
   │
   │ event
   ↓
CONTROL
```

This is NROS's fundamental control loop.

# 61. Plane Boundary Invariants

```text
1. Control expresses authority and intent.

2. Data performs actual work.

3. Evidence records observable history.

4. Commands are not events.

5. Events are not commands.

6. Desired state is not actual state.

7. Observation is not automatically authority.

8. Lease authority is explicit.

9. Stale commands are rejected.

10. Stale ownership is fenceable.

11. Control failure does not automatically prove data failure.

12. Data failure does not automatically prove Work failure.

13. Evidence failure does not silently erase history.

14. Resource reservation is distinct from resource consumption.

15. Capability advertisements have freshness semantics.

16. Control-plane ownership has explicit generations.

17. Split-brain ownership is prohibited or fenced.

18. Operator actions are represented as auditable events.

19. Automated decisions carry policy provenance.

20. Every plane can independently report degraded operation.
```

# 62. NROS Three-Plane Reference Model

```text
                           ┌─────────────────────┐
                           │    CONTROL PLANE    │
                           │                     │
                           │ Policy              │
                           │ Admission           │
                           │ Scheduling          │
                           │ Authority           │
                           │ Reconciliation      │
                           └─────────┬───────────┘
                                     │
                            commands │ leases
                                     ↓
        ┌─────────────────────────────────────────────┐
        │                  DATA PLANE                 │
        │                                             │
        │ Agents ─ Workers ─ Processes ─ Resources   │
        │                                             │
        └──────────────────────┬──────────────────────┘
                               │
                       observations
                         results
                         effects
                               ↓
        ┌─────────────────────────────────────────────┐
        │                EVIDENCE PLANE               │
        │                                             │
        │ Events ─ Audit ─ Provenance ─ Snapshots    │
        │ Replay ─ Reconstruction ─ Verification     │
        │                                             │
        └──────────────────────┬──────────────────────┘
                               │
                         reconciliation
                               │
                               └──────────────→ CONTROL
```

# 63. Failure Model Across the Three Planes

Consider:

```text
Control:  healthy
Data:     healthy
Evidence: unavailable
```

The system is **not simply healthy**.

It is:

```text
EXECUTION_HEALTHY
EVIDENCE_DEGRADED
```

This distinction should appear in NROS health state.

# 64. Composite Health

NROS should expose:

```text
control_health
data_health
evidence_health
```

and derive:

```text
overall_operational_state
```

without hiding the individual dimensions.

# 65. Example

```text
CONTROL = HEALTHY
DATA = DEGRADED
EVIDENCE = HEALTHY
```

Overall:

```text
DEGRADED
```

Reason:

```text
agent availability reduced
```

# 66. Degraded Modes

NROS should explicitly define:

```text
NORMAL
DEGRADED_CONTROL
DEGRADED_DATA
DEGRADED_EVIDENCE
PARTITIONED
RECOVERING
READ_ONLY
EMERGENCY
```

# 67. Read-Only Mode

When authoritative mutation is unsafe:

```text
queries → allowed
commands → rejected
```

This can protect state during recovery.

# 68. Emergency Mode

For critical failures:

```text
new Work admission → disabled
existing execution → policy-defined
resource reclamation → restricted
evidence → mandatory
```

The exact semantics must be specified per deployment profile.

# 69. Recovery Mode

During restart:

```text
load snapshots
 ↓
replay events
 ↓
verify authority
 ↓
reconcile agents
 ↓
reconcile resources
 ↓
resume scheduling
```

This should be deterministic.

# 70. Startup Gate

A controller should not immediately schedule Work after process startup.

Preferred:

```text
START
 ↓
LOAD STATE
 ↓
VERIFY STATE
 ↓
ESTABLISH AUTHORITY
 ↓
RECONCILE DATA PLANE
 ↓
RECONCILE RESOURCES
 ↓
ENABLE SCHEDULING
```

# 71. No Premature Scheduling

Before reconciliation:

```text
scheduler = NOT_READY
```

Only after required evidence is established:

```text
scheduler = READY
```

# 72. Recovery Barrier

Introduce:

```text
RECOVERY_BARRIER
```

No new authoritative scheduling decisions cross the barrier until mandatory recovery checks pass.

# 73. Recovery Evidence

The barrier should produce:

```text
RECOVERY_STARTED
STATE_LOADED
STATE_VERIFIED
AGENTS_RECONCILED
RESOURCES_RECONCILED
RECOVERY_COMPLETED
```

# 74. Final Architectural Principle

NROS is therefore not simply:

```text
scheduler + workers
```

It becomes:

```text
        INTENT
          ↓
      AUTHORITY
          ↓
       CONTROL
          ↓
      EXECUTION
          ↓
         DATA
          ↓
      OBSERVATION
          ↓
       EVIDENCE
          ↓
    RECONCILIATION
          ↓
        STATE
          ↓
      NEW INTENT
```

This is the **closed-loop distributed runtime model**.

# Part C — State Authority, Reconciliation & Recovery Protocol

The next layer should formalize the mechanism that closes this loop:

```text
Authority
Ownership
Leases
Generations
Desired state
Observed state
Actual state
Drift detection
Conflict resolution
Reconciliation loops
Recovery barriers
Failover
Split-brain prevention
State convergence
Controller restarts
Agent restarts
Resource reclamation
```

The central question becomes:

> **When control-plane intent, data-plane reality, and evidence-plane history disagree, which state wins, under what authority, and how does NROS safely converge without duplicating or losing side effects?**

# NROS — Part C: State Authority, Reconciliation & Recovery Protocol

We now reach one of the most important parts of the architecture:

> **How does NROS safely converge when desired state, observed state, and historical evidence disagree?**

The answer cannot simply be:

```text
latest_state_wins
```

because distributed systems routinely contain:

```text
delayed messages
duplicate messages
stale observations
expired leases
restarted controllers
restarted agents
partitions
partial commits
```

NROS therefore needs an explicit **authority and reconciliation protocol**.

# 1. The Four-State Model

For an execution, distinguish four concepts:

```text
DESIRED
OBSERVED
AUTHORIZED
EFFECTIVE
```

### Desired

What the control plane wants:

```text
desired = RUNNING
```

### Observed

What the data plane reports:

```text
observed = RUNNING
```

### Authorized

What the current authority permits:

```text
authorized = RUNNING
```

### Effective

What NROS currently considers the authoritative state:

```text
effective = RUNNING
```

These are related, but not interchangeable.

# 2. Why Four States?

Consider:

```text
desired     = RUNNING
observed    = RUNNING
authorized  = EXPIRED
```

Then:

```text
effective ≠ RUNNING
```

The process may still physically exist, but it no longer possesses valid execution authority.

# 3. State Tuple

A useful conceptual representation:

```text
State {
    desired
    observed
    authorized
    effective

    generation
    epoch
    lease
    evidence
}
```

This becomes the basis of reconciliation.

# 4. Authority Hierarchy

Not all information has equal authority.

A possible hierarchy:

```text
                ┌────────────────────┐
                │ Protocol Authority │
                └─────────┬──────────┘
                          ↓
                ┌────────────────────┐
                │ Valid Lease/Token  │
                └─────────┬──────────┘
                          ↓
                ┌────────────────────┐
                │ Committed Evidence │
                └─────────┬──────────┘
                          ↓
                ┌────────────────────┐
                │ Agent Observation  │
                └─────────┬──────────┘
                          ↓
                ┌────────────────────┐
                │ Cached Observation │
                └────────────────────┘
```

The exact ordering must be protocol-defined.

# 5. Authority Is Scoped

Authority should never be globally implied.

An authority may apply to:

```text
tenant
work
execution
resource
partition
agent
artifact
```

Therefore:

```text
authority_scope
```

must be explicit.

# 6. Authority Token

Conceptually:

```text
AuthorityToken {
    token_id
    subject
    scope
    epoch
    generation
    issuer
    issued_at
    expires_at
    capabilities
}
```

# 7. Authority ≠ Identity

An Agent may have identity:

```text
agent_id = A
```

while currently holding:

```text
lease = L17
generation = 42
```

Identity says:

> Who are you?

Authority says:

> What are you currently allowed to do?

# 8. Authority ≠ Ownership

Ownership means:

> Which controller is responsible for this state?

Authority means:

> Which actor is permitted to perform a specific operation?

These concepts may overlap but should remain separate.

# 9. Ownership Record

Example:

```text
Ownership {
    partition
    owner
    epoch
    acquired_at
    expires_at
}
```

# 10. Ownership Transfer

```text
Owner A
   │
   │ release
   ↓
TRANSFER_PENDING
   │
   │ acquire
   ↓
Owner B
```

The transfer itself must be represented as evidence.

# 11. Safe Handoff

The safe sequence is:

```text
A owns P
 ↓
A stops issuing new mutations
 ↓
A persists handoff state
 ↓
ownership epoch increments
 ↓
B acquires P
 ↓
B verifies state
 ↓
B becomes ACTIVE
```

# 12. No Concurrent Owners

The fundamental invariant:

```text
∀ partition P:
    active_authority(P) ≤ 1
```

unless the protocol explicitly supports multiple independent authorities.

# 13. Split Brain

A split-brain situation:

```text
Controller A → epoch 20
Controller B → epoch 20
```

Both believe they own the same partition.

NROS must make this state:

```text
INVALID
```

or make one authority unable to produce valid side effects.

# 14. Fencing Solves Stale Ownership

If B receives:

```text
epoch = 21
```

then A's:

```text
epoch = 20
```

is stale.

External resources must reject operations carrying epoch 20 where fencing is required.

# 15. Generation

Within an execution:

```text
generation = 1
```

may become:

```text
generation = 2
```

after reassignment or recovery.

This prevents old agents from continuing to act as current owners.

# 16. Generation Transition

```text
Execution
   generation 1
        ↓
lease lost
        ↓
recovery
        ↓
generation 2
```

The new attempt is not merely another message.

It is a new authority generation.

# 17. Attempt vs Generation

An execution can contain multiple attempts:

```text
Execution E
 ├── Attempt 1
 ├── Attempt 2
 └── Attempt 3
```

A generation can identify authority across those transitions.

Do not automatically equate:

```text
attempt == generation
```

unless the protocol explicitly defines that relationship.

# 18. Reconciliation Loop

The fundamental control loop:

```text
OBSERVE
   ↓
COMPARE
   ↓
CLASSIFY
   ↓
DECIDE
   ↓
ACT
   ↓
VERIFY
   ↓
OBSERVE
```

This should be deterministic wherever possible.

# 19. Observe

Collect:

```text
desired state
agent observations
resource state
lease state
event history
```

# 20. Compare

Determine:

```text
desired vs observed
desired vs authorized
authorized vs observed
observed vs evidence
```

# 21. Classify

Possible classifications:

```text
CONVERGED
DRIFT
STALE
UNKNOWN
CONFLICT
UNAUTHORIZED
LOST
RECOVERABLE
FATAL
```

# 22. Converged

Example:

```text
desired    = RUNNING
observed   = RUNNING
authorized = RUNNING
```

Result:

```text
CONVERGED
```

No corrective action is required.

# 23. Drift

Example:

```text
desired    = RUNNING
observed   = STOPPED
authorized = RUNNING
```

Result:

```text
DRIFT
```

Controller may issue:

```text
START
```

subject to policy.

# 24. Unauthorized

Example:

```text
desired    = STOPPED
observed   = RUNNING
authorized = NONE
```

The process exists without current authority.

Classification:

```text
UNAUTHORIZED_EXECUTION
```

# 25. Unknown

Example:

```text
desired    = RUNNING
observed   = UNKNOWN
authorized = RUNNING
```

Possible causes:

```text
network partition
agent crash
telemetry loss
controller restart
```

NROS must avoid prematurely declaring failure.

# 26. Lost

A stronger state may require:

```text
lease expired
+
timeout exceeded
+
recovery evidence
```

Then:

```text
EXECUTION_LOST
```

becomes justified.

# 27. Conflict

Example:

```text
Agent A → RUNNING
Agent B → RUNNING
Resource manager → owner = B
Control plane → generation = 8
Agent A → generation = 7
```

The conflict is resolved through generation/authority rules.

# 28. Reconciliation Is Not Guessing

Bad:

```text
if states differ:
    pick newest message
```

Better:

```text
if states differ:
    evaluate authority
    evaluate generation
    evaluate causality
    evaluate evidence
    evaluate freshness
    apply protocol rule
```

# 29. Freshness

Every observation should have a freshness concept.

For example:

```text
observed_at
received_at
valid_until
```

These must not be confused.

# 30. Observation Time vs Receive Time

Suppose:

```text
Agent observation:
10:00:00

Controller receives:
10:00:08
```

The event is eight seconds old when received.

This matters for scheduling and failure detection.

# 31. Clock Skew

Distributed nodes may have different clocks.

Therefore NROS should not depend on exact wall-clock comparison for correctness.

Use:

```text
generation
epoch
sequence
causation
lease semantics
```

for authoritative ordering.

# 32. Lease Validity

A lease may be represented as:

```text
LEASE(
    subject,
    generation,
    issued_at,
    expires_at
)
```

But correctness should not depend solely on local clock agreement.

For strict environments, lease renewal must have an explicit protocol.

# 33. Renewal

```text
LEASE_ISSUED
     ↓
LEASE_RENEWED
     ↓
LEASE_RENEWED
     ↓
LEASE_RENEWED
```

Each renewal can carry:

```text
generation
sequence
expiry
```

# 34. Renewal Failure

If renewal fails:

```text
Agent
  ↓
renew request
  X
Control
```

the Agent enters a protocol-defined state such as:

```text
LEASE_AT_RISK
```

before:

```text
LEASE_EXPIRED
```

# 35. Grace Period

A grace period may exist:

```text
VALID
  ↓
AT_RISK
  ↓
EXPIRED
```

This avoids treating a transient network delay as immediate execution loss.

# 36. Fencing After Expiry

Once the protocol declares:

```text
EXPIRED
```

the Agent must no longer perform protected side effects.

For externally fenced resources:

```text
resource
  ↑
token validation
```

must reject stale authority.

# 37. Recovery

Recovery begins when normal control assumptions no longer hold.

Examples:

```text
controller restart
agent restart
network partition
resource manager restart
evidence-store restart
```

# 38. Recovery State Machine

```text
NORMAL
  ↓
FAILURE_DETECTED
  ↓
RECOVERY_PENDING
  ↓
RECONCILING
  ↓
RECOVERED
```

or:

```text
RECONCILING
  ↓
UNRECOVERABLE
```

# 39. Recovery Must Be Evidence-Driven

Do not recover merely because:

```text
process restarted
```

Recovery should establish:

```text
identity
authority
generation
resource ownership
execution state
result state
```

# 40. Controller Restart

Controller starts:

```text
BOOTING
```

Then:

```text
LOAD_STATE
VERIFY_STATE
ESTABLISH_AUTHORITY
RECONCILE
READY
```

# 41. Controller Must Not Immediately Schedule

A restarted scheduler should not blindly execute:

```text
schedule()
```

before discovering the current world.

Otherwise it may duplicate Work.

# 42. Duplicate Execution Hazard

Suppose previous state was:

```text
RUNNING
```

Controller restarts and state is temporarily unknown.

It schedules another execution:

```text
Attempt 1 → still running
Attempt 2 → newly started
```

Now two executions exist.

This is precisely the class of failure reconciliation must prevent.

# 43. Recovery Barrier

Therefore:

```text
controller restart
       ↓
recovery barrier
       ↓
discover existing executions
       ↓
reconcile
       ↓
enable scheduling
```

# 44. Agent Restart

Agent restarts and reports:

```text
agent_id = A
```

The identity may be the same, but execution ownership must be revalidated.

# 45. Agent Re-registration

Agent sends:

```text
REGISTER {
    agent_id
    incarnation
    capabilities
    runtime_version
}
```

The:

```text
incarnation
```

distinguishes a new process instance from an older one.

# 46. Incarnation Number

Example:

```text
agent A
 incarnation 41
```

restarts:

```text
agent A
 incarnation 42
```

Messages from incarnation 41 become stale.

# 47. Agent Identity Tuple

A robust identity may be:

```text
(agent_id, incarnation)
```

rather than merely:

```text
agent_id
```

# 48. Execution Identity

Likewise:

```text
execution_id
attempt_id
generation
```

provide multiple layers of identity.

# 49. Why Multiple IDs?

Because these questions differ:

```text
Which logical execution?
Which attempt?
Which authority generation?
Which process instance?
Which event?
```

A single identifier cannot answer all of them cleanly.

# 50. Resource Reconciliation

After controller restart:

```text
desired allocations
        ↓
resource manager observations
        ↓
actual allocations
        ↓
reconciliation
```

# 51. Resource Leak

If control says:

```text
allocated = 0
```

but resource manager says:

```text
allocated = 4 CPU
```

NROS detects:

```text
RESOURCE_LEAK
```

and must determine whether:

```text
valid execution
stale allocation
orphaned allocation
```

# 52. Orphan

An orphan is a data-plane object without a valid controlling relationship.

Examples:

```text
orphan process
orphan reservation
orphan artifact
orphan lease
```

# 53. Orphan Handling

Never automatically delete every orphan.

Possible states:

```text
ORPHAN_DETECTED
 ↓
INSPECTION
 ↓
RECLAIM
```

or:

```text
ORPHAN_DETECTED
 ↓
ADOPT
```

# 54. Adoption

A controller may adopt an existing execution if:

```text
identity matches
generation valid
provenance valid
resource ownership valid
policy permits adoption
```

# 55. Adoption Event

Record:

```text
EXECUTION_ADOPTED {
    execution_id
    previous_owner
    new_owner
    generation
    evidence
}
```

# 56. Reclamation

If adoption is unsafe:

```text
RECLAIM_REQUESTED
```

followed by:

```text
RECLAIMED
```

with complete evidence.

# 57. Safe Reclamation

The reclamation path should be:

```text
detect orphan
 ↓
fence old authority
 ↓
verify no valid owner
 ↓
terminate/release
 ↓
verify release
 ↓
record final evidence
```

# 58. Side-Effect Safety

Every recovery operation must consider:

> Could this action duplicate a side effect?

For example:

```text
CHARGE_PAYMENT
SEND_EMAIL
WRITE_DATABASE
DEPLOY_ARTIFACT
```

may not be safely repeatable.

# 59. Idempotency Key

External side effects should use:

```text
idempotency_key
```

where possible.

Example:

```text
side_effect_key =
execution_id + operation_id
```

# 60. Exactly-Once Illusion

NROS should avoid claiming:

```text
exactly once
```

unless it can genuinely guarantee it across the relevant failure boundaries.

Prefer explicit semantics:

```text
at-most-once
at-least-once
effectively-once
idempotent retry
```

# 61. Effectively-Once

A useful pattern:

```text
at-least-once delivery
+
idempotency
+
durable deduplication
=
effectively-once effect
```

# 62. Recovery Decision Table

| Desired | Observed | Authority | Classification |
|---|---|---|---|
| RUNNING | RUNNING | valid | CONVERGED |
| RUNNING | STOPPED | valid | DRIFT |
| STOPPED | RUNNING | valid | DRIFT |
| RUNNING | UNKNOWN | valid | UNKNOWN |
| STOPPED | RUNNING | expired | UNAUTHORIZED |
| RUNNING | RUNNING | expired | UNAUTHORIZED |
| RUNNING | RUNNING | stale generation | STALE |
| RUNNING | conflicting owners | ambiguous | CONFLICT |

# 63. Reconciliation Decision Object

A reconciliation result can be modeled as:

```text
ReconciliationDecision {
    entity
    observed_state
    desired_state
    authority_state
    classification
    action
    reason
    policy_revision
    evidence
}
```

# 64. Example

```text
classification = DRIFT
action         = START
reason         = desired_running_but_execution_absent
policy         = recovery-v3
```

This is far better than:

```text
action = START
```

with no explanation.

# 65. Reconciliation Events

Recommended events:

```text
RECONCILIATION_STARTED
DRIFT_DETECTED
CONFLICT_DETECTED
STALE_STATE_DETECTED
ORPHAN_DETECTED
ADOPTION_REQUESTED
EXECUTION_ADOPTED
RECLAMATION_REQUESTED
RESOURCE_RECONCILED
RECONCILIATION_COMPLETED
```

# 66. Reconciliation Idempotence

Running reconciliation twice with no world change should produce:

```text
same state
```

and should not create duplicated side effects.

# 67. Idempotent Reconciler

Conceptually:

```text
reconcile(S, O)
```

should satisfy:

```text
reconcile(reconcile(S, O), O)
=
reconcile(S, O)
```

for stable conditions.

This is a powerful architectural invariant.

# 68. Reconciliation Convergence

For a stable environment:

```text
desired
   ↓
reconcile
   ↓
action
   ↓
observe
   ↓
reconcile
   ↓
converged
```

The system should eventually reach:

```text
CONVERGED
```

provided failures cease and required dependencies recover.

# 69. Non-Convergence

If reconciliation repeatedly produces:

```text
action
→ opposite observation
→ same action
→ opposite observation
```

NROS has a reconciliation loop.

Example:

```text
START
↓
Agent stops
↓
START
↓
Agent stops
```

# 70. Loop Detection

Track:

```text
reconciliation_count
state_transition_frequency
action_repeat_count
```

and detect unstable behavior.

# 71. Circuit Breaker

After repeated failure:

```text
START
START
START
START
```

controller may transition to:

```text
RECOVERY_BLOCKED
```

rather than creating infinite churn.

# 72. Recovery Budget

Policies may specify:

```text
max_attempts
max_recovery_time
max_resource_cost
```

# 73. Recovery Policy

Example:

```text
RecoveryPolicy {
    retry_limit
    retry_backoff
    adoption_allowed
    reclamation_allowed
    fencing_required
    timeout
}
```

# 74. Recovery Escalation

```text
retry
 ↓
retry
 ↓
recovery
 ↓
quarantine
 ↓
operator intervention
```

This prevents indefinite automated action.

# 75. Quarantine

A problematic execution can enter:

```text
QUARANTINED
```

meaning:

```text
normal scheduling/recovery actions disabled
```

while evidence remains available for investigation.

# 76. Quarantine Is Not Deletion

Critical distinction:

```text
QUARANTINE
≠
DELETE
```

The execution and evidence remain preserved.

# 77. Recovery Finality

Recovery should end in a well-defined state:

```text
RECOVERED
FAILED
CANCELLED
QUARANTINED
LOST
UNKNOWN
```

Avoid:

```text
probably recovered
```

as an implicit state.

# 78. Unknown Is a First-Class State

This is particularly important.

If NROS cannot prove:

```text
RUNNING
```

or:

```text
FAILED
```

then:

```text
UNKNOWN
```

is more correct than inventing certainty.

# 79. Unknown → Known

Evidence may later resolve:

```text
UNKNOWN
 ↓
RUNNING
```

or:

```text
UNKNOWN
 ↓
FAILED
```

or:

```text
UNKNOWN
 ↓
LOST
```

# 80. No False Finality

NROS must not mark:

```text
FAILED
```

merely because:

```text
heartbeat timeout
```

unless the protocol establishes that timeout as sufficient evidence.

# 81. Failure Classification

Failures should be structured:

```text
FAILURE {
    class
    code
    source
    retryable
    terminal
    evidence
}
```

# 82. Failure Classes

Examples:

```text
USER_ERROR
POLICY_REJECTION
RESOURCE_EXHAUSTION
AGENT_FAILURE
NETWORK_FAILURE
DEPENDENCY_FAILURE
PROTOCOL_FAILURE
AUTHORITY_FAILURE
DATA_CORRUPTION
SYSTEM_FAILURE
UNKNOWN_FAILURE
```

# 83. Retryability

Do not infer retryability from textual error messages.

Instead:

```text
retryable = true
```

should be determined by explicit policy/classification.

# 84. Recovery vs Retry

Retry means:

```text
same logical operation attempted again
```

Recovery may mean:

```text
restore authority
adopt execution
reconcile resources
repair state
```

These are different mechanisms.

# 85. Final State Transition

A terminal transition should require sufficient evidence.

Example:

```text
RUNNING
 ↓
RESULT_COMMITTED
 ↓
RESOURCE_RELEASED
 ↓
COMPLETED
```

The exact sequence may differ by execution type.

# 86. Terminal State Immutability

Once an execution reaches a protocol-defined terminal state:

```text
COMPLETED
FAILED
CANCELLED
```

it should not silently become:

```text
RUNNING
```

A new attempt must have a new identity/generation.

# 87. Reopening Work

If a completed logical Work needs execution again:

```text
Work W
 ├── Execution E1 → COMPLETED
 └── Execution E2 → RUNNING
```

rather than mutating E1 back to RUNNING.

# 88. Historical Integrity

This preserves:

```text
what actually happened
```

instead of rewriting history to represent the new operation.

# 89. Recovery State Machine

A robust execution lifecycle can therefore be:

```text
                    ┌──────────────┐
                    │   CREATED    │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │   ADMITTED   │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │  SCHEDULED   │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │   STARTING   │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │   RUNNING    │
                    └──────┬───────┘
                           │
               ┌───────────┼────────────┐
               ↓           ↓            ↓
          COMPLETING    RECOVERING   CANCELLING
               │           │            │
               ↓           │            ↓
          COMPLETED        │        CANCELLED
                           │
                ┌──────────┴──────────┐
                ↓                     ↓
            RECOVERED               LOST
```

# 90. Recovery Is a State Machine

Do not implement recovery as an uncontrolled collection of timers.

Instead:

```text
recovery_state
+
evidence
+
policy
→
next_transition
```

# 91. Timer Semantics

Timers should trigger observations:

```text
timeout elapsed
```

not directly assert:

```text
execution failed
```

The reconciler interprets the timeout.

# 92. Timer as Evidence

A timeout is evidence:

```text
HEARTBEAT_TIMEOUT
```

not necessarily final truth.

# 93. Control Plane Recovery Invariant

After restart:

```text
No scheduling before reconciliation barrier.
```

# 94. Agent Recovery Invariant

After restart:

```text
No protected side effects before authority revalidation.
```

# 95. Resource Recovery Invariant

After restart:

```text
No resource reclamation before ownership verification.
```

# 96. Evidence Recovery Invariant

After evidence-store restart:

```text
No claim of historical completeness before journal/snapshot verification.
```

# 97. Unified Recovery Rule

> **Never convert absence of observation into proof of absence without an explicit protocol rule establishing that inference.**

This is one of the strongest NROS safety invariants.

# 98. Authority + Evidence + Observation

The final reconciliation equation becomes conceptually:

```text
EffectiveState =
    Resolve(
        DesiredState,
        ObservedState,
        Authority,
        Generation,
        Evidence,
        Policy
    )
```

Not:

```text
EffectiveState = LatestMessage
```

# 99. Reconciliation Architecture

```text
             ┌─────────────────────┐
             │   DESIRED STATE     │
             └──────────┬──────────┘
                        │
                        ↓
┌──────────────┐   ┌───────────────┐   ┌──────────────┐
│ OBSERVATIONS │──→│ RECONCILER    │←──│   EVIDENCE   │
└──────────────┘   └───────┬───────┘   └──────────────┘
                            │
                     authority check
                            │
                            ↓
                    ┌───────────────┐
                    │ CLASSIFICATION│
                    └───────┬───────┘
                            ↓
                    ┌───────────────┐
                    │    ACTION     │
                    └───────┬───────┘
                            ↓
                    ┌───────────────┐
                    │   DATA PLANE  │
                    └───────┬───────┘
                            │
                            ↓
                       OBSERVATION
```

# 100. Core Reconciliation Invariants

```text
1. Desired state is intent, not proof.

2. Observed state is evidence, not authority.

3. Authority is explicit and scoped.

4. Ownership is explicitly represented.

5. Generations prevent stale actors from mutating current state.

6. Epochs prevent stale controllers from retaining ownership.

7. Leases represent time-bounded authority.

8. Lease expiration does not automatically prove process termination.

9. Unknown is preferable to unsupported certainty.

10. Reconciliation is deterministic.

11. Reconciliation is idempotent.

12. Reconciliation should converge under stable conditions.

13. Reconciliation actions require explicit policy.

14. Recovery requires evidence.

15. Controller restart requires a recovery barrier.

16. Agent restart requires identity/incarnation handling.

17. Resource recovery requires ownership verification.

18. Orphans are detected before being reclaimed.

19. Adoption is explicit.

20. Reclamation is explicit.

21. Split-brain ownership is prohibited or fenced.

22. Stale commands are rejected.

23. Stale generations cannot perform protected side effects.

24. Terminal executions are immutable.

25. New attempts use new execution/attempt identities.

26. Recovery must not duplicate irreversible side effects.

27. External side effects should use idempotency mechanisms.

28. Retry and recovery are distinct.

29. Reconciliation loops must be detectable.

30. Repeated failure can trigger quarantine.

31. Evidence is preserved during quarantine.

32. Finality requires sufficient evidence.

33. Absence of observation is not automatically failure.

34. Recovery transitions are auditable.

35. Every important reconciliation decision has provenance.
```

# Part CI — Failure Semantics & Exactly-Once Effects

The next layer should go deeper into the hardest operational boundary:

```text
network partition
process crash
partial command delivery
partial result commit
duplicate execution
lost acknowledgement
resource reservation races
external side effects
```

The central question becomes:

> **How does NROS guarantee safe behavior when an operation may have happened, but the caller cannot determine whether it happened?**
