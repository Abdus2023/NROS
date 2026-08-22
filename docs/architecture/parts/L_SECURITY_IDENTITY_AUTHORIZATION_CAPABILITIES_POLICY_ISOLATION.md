# Part L — Security Architecture, Identity, Authorization, Capabilities, Policy & Isolation

> **Series:** NROS Architecture Series  
> **Part:** L  
> **Role:** identity, authentication, authorization, capabilities, policy evaluation, secrets, trust zones, isolation, security invariants, and security enforcement boundaries  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XLIX established the API/service boundary. Part L defines the security architecture that protects every boundary from external API entry through protocol, scheduling, execution, persistence, and distributed authority.

The central rule is:

> **Security is an end-to-end property: identity establishes who acts, authorization establishes what may be done, capabilities constrain delegated authority, policy determines admissibility, and isolation limits the consequences of compromise.**

## 2. Security Is Not One Layer

```text
Identity
 ↓
Authentication
 ↓
Authorization
 ↓
Policy
 ↓
Capability
 ↓
Isolation
 ↓
Audit
```

No single mechanism substitutes for the others.

## 3. Principal Model

A principal may represent:

```text
human
service
agent
node
workload
external client
automation identity
```

A process is not automatically a trusted principal merely because it runs inside NROS.

## 4. Identity

Identity should have stable logical representation:

```text
principal_id
issuer
credential/reference
scope
status
```

## 5. Authentication

Authentication establishes control of an identity.

```text
credential
 ↓
verification
 ↓
authenticated principal
```

## 6. Authentication vs Identity

```text
claimed identity
    ≠
verified identity
```

All security-sensitive decisions must use verified identity.

## 7. Credential Types

Concrete deployments may use:

```text
keys
certificates
tokens
hardware-backed credentials
workload identity
federated identity
```

The architecture does not require one universal credential mechanism.

## 8. Credential Lifecycle

```text
issued
 ↓
active
 ↓
rotated
 ↓
revoked
 ↓
expired
```

Credential lifecycle is security-critical state.

## 9. Key Rotation

Rotation should allow overlap where required:

```text
old key + new key
       ↓
transition
       ↓
new key
```

Rotation must not silently invalidate required recovery paths.

## 10. Revocation

Revocation must have explicit propagation and freshness semantics.

A revoked identity must not remain authorized indefinitely because one component has stale policy state.

## 11. Authorization

Authorization evaluates whether a principal may perform an operation:

```text
Principal
 + Resource
 + Operation
 + Context
 + Policy
 → Decision
```

## 12. Authorization Decision

Canonical results:

```text
allow
 deny
 require additional proof
```

Default behavior for unknown policy should be deny unless a specific fail-open contract exists.

## 13. Authentication Does Not Grant Authority

```text
Authenticated
    ≠
Authorized
```

## 14. Policy

Policy defines admissible behavior independently from individual implementation paths.

Policy inputs may include:

```text
principal
resource
operation
scope
environment
risk
epoch
capabilities
resource state
```

## 15. Policy Evaluation

```text
Request
 ↓
Normalize context
 ↓
Load applicable policy
 ↓
Evaluate
 ↓
Decision
 ↓
Enforce
```

## 16. Policy Version

Security-sensitive decisions should be traceable to the policy version that produced them.

```text
policy_version
```

## 17. Policy Precedence

When multiple policies apply, precedence must be deterministic.

```text
specific deny/allow
 ↓
resource policy
 ↓
scope policy
 ↓
default policy
```

The concrete precedence hierarchy must be explicitly defined by implementation.

## 18. Default Deny

Unknown, malformed, expired, or unverifiable authority should not silently become permission.

```text
unknown authority
    ↓
    deny
```

## 19. Capabilities

A capability represents constrained authority:

```text
principal
 + operation
 + resource/scope
 + constraints
 + expiry/epoch
```

## 20. Capability vs Role

```text
role
 → broad policy classification

capability
 → delegated concrete authority
```

They solve different problems.

## 21. Least Authority

Components should receive only the authority required for their task.

```text
required authority
      ↓
minimal grant
```

## 22. Capability Attenuation

Delegated authority may be narrowed:

```text
Parent capability
 ↓ constrain scope
Child capability
```

A child capability must never expand authority beyond its parent.

## 23. Capability Expiration

Time-bound authority should expire automatically where appropriate.

## 24. Epoch-Bound Authority

Distributed authority can be bound to the current epoch:

```text
capability(epoch=7)
current epoch=8
      ↓
reject
```

This connects security to Part XLVII consensus and fencing semantics.

## 25. Fencing

When authority is revoked, stale holders must be prevented from continuing protected operations.

```text
old authority
 ↓
fence
 ↓
protected operation rejected
```

## 26. Trust Zones

NROS should model distinct trust zones rather than assuming one homogeneous trust domain.

Example:

```text
Zone A — external/untrusted
Zone B — authenticated clients
Zone C — service boundary
Zone D — privileged runtime
Zone E — host/platform
```

Concrete deployments may define different zones.

## 27. Trust Boundaries

Every crossing between zones requires explicit security controls.

```text
Zone A
  ↓ authentication + validation
Zone B
  ↓ authorization
Zone C
  ↓ capability
Zone D
```

## 28. Zero Trust Principle

Network location, process ancestry, or membership alone must not be treated as sufficient authorization evidence.

## 29. Service-to-Service Security

Internal services must authenticate and authorize each other when crossing security boundaries.

```text
internal
 ≠
implicitly trusted
```

## 30. Agent Identity

Autonomous agents require explicit identity separate from the human or service that launched them.

```text
operator
 ↓ delegates
agent identity
```

The delegation relationship must be auditable.

## 31. Agent Authority

Agent authority should be scoped by:

```text
task
resources
operations
time
budget
policy
```

## 32. Agent Delegation

```text
Principal A
 ↓ constrained delegation
Agent B
 ↓ constrained action
Resource C
```

Delegation must not create privilege escalation.

## 33. Impersonation

If an actor operates on behalf of another principal, the protocol should distinguish:

```text
actor
subject
delegator
```

## 34. Audit Identity

Security evidence must preserve the chain of responsibility:

```text
who acted
on whose behalf
using which authority
against which resource
```

## 35. Secret Management

Secrets include:

```text
private keys
tokens
passwords
API credentials
encryption keys
```

They must not be treated as ordinary configuration values.

## 36. Secret Exposure

Secrets must not appear unintentionally in:

```text
logs
errors
metrics
traces
crash dumps
API responses
```

## 37. Secret Storage

Secret storage should use the strongest available platform mechanism and minimize plaintext exposure.

## 38. Secret Injection

Prefer controlled runtime injection over embedding secrets in source, images, or static artifacts.

## 39. Secret Rotation

Consumers must tolerate secret rotation without requiring unsafe plaintext export.

## 40. Encryption at Rest

Sensitive persistent state should have explicitly defined encryption requirements and key ownership.

## 41. Encryption in Transit

Cross-boundary sensitive communication requires authenticated integrity and confidentiality according to the deployment threat model.

## 42. Data Classification

Data should be classified according to sensitivity, for example:

```text
public
internal
sensitive
secret
high-impact
```

The exact taxonomy is deployment-specific.

## 43. Data Minimization

Components should receive only the data required for their operation.

## 44. Tenant Isolation

Multi-tenant deployments require explicit isolation of:

```text
identity
resources
metadata
logs
streams
quotas
secrets
```

## 45. Cross-Tenant Access

Cross-tenant access must be explicit, authorized, and auditable.

## 46. Namespace Isolation

Resource namespaces should prevent accidental collisions and unauthorized discovery.

## 47. Process Isolation

Where threat models require it, workloads may be isolated through:

```text
process boundaries
containers
sandboxes
VMs
OS policies
```

The architecture does not assume any single mechanism.

## 48. Filesystem Isolation

Workloads should receive only required filesystem visibility and write access.

## 49. Network Isolation

Workloads should receive only required network connectivity.

```text
allowlist
 ↓
required destinations
```

## 50. Resource Isolation

Security and availability both require limits on:

```text
CPU
memory
storage
processes
threads
connections
streams
```

## 51. Privilege Separation

Privileged operations should be isolated behind narrow interfaces.

```text
unprivileged worker
      ↓
minimal privileged service
      ↓
protected operation
```

## 52. Privileged Runtime

The runtime should not expose broad host authority to ordinary workload code.

## 53. Host Boundary

Host-level operations require an explicit trust transition.

```text
workload
 ↓ policy
host operation
```

## 54. Dangerous Operations

Operations affecting the host or cluster should be explicitly classified as privileged:

```text
shutdown
reboot
mount
network reconfiguration
credential changes
cluster membership changes
authority transfer
```

## 55. Policy Enforcement Points

Security controls should identify where decisions are enforced:

```text
API gateway
protocol handler
service boundary
scheduler
executor
storage
host adapter
```

Defense in depth should not mean contradictory policies.

## 56. Policy Decision Point

Policy evaluation may be centralized or distributed, but the authoritative decision source must be identifiable.

## 57. Policy Enforcement Point

The component performing the protected action must enforce the decision or verify a valid authorization artifact.

## 58. Decision Freshness

Security decisions may have a freshness requirement:

```text
policy snapshot
 ↓
valid until
 ↓
refresh
```

## 59. Fail-Closed vs Fail-Open

Each security control must define failure behavior.

For high-impact operations:

```text
policy unavailable
      ↓
      deny
```

## 60. Emergency Access

Emergency access must be:

```text
explicit
scoped
time-limited
audited
revocable
```

Emergency access must not become permanent administrative authority.

## 61. Break-Glass

Break-glass access should require stronger evidence and produce high-value audit records.

## 62. Rate Limiting

Authentication and authorization surfaces require abuse controls against:

```text
credential guessing
request flooding
policy evaluation exhaustion
resource enumeration
```

## 63. Security Resource Limits

Untrusted inputs must be bounded before expensive security operations.

## 64. Replay Security

Authentication artifacts and authorization decisions requiring freshness must include appropriate replay resistance.

## 65. Time Security

Security decisions involving expiration require trustworthy time semantics.

Where wall-clock trust is insufficient, epochs, monotonic counters, or other mechanisms should supplement it.

## 66. Clock Skew

Distributed expiration must define acceptable clock skew and failure behavior.

## 67. Supply-Chain Identity

Executable components should have verifiable provenance where the deployment threat model requires it.

## 68. Component Integrity

NROS should distinguish:

```text
known component
trusted component
verified artifact
running identity
```

These are not automatically equivalent.

## 69. Secure Startup

Startup should validate required security state before exposing protected APIs.

```text
load identity
 ↓
validate configuration
 ↓
load policy
 ↓
initialize enforcement
 ↓
open protected service
```

## 70. Secure Shutdown

Shutdown must preserve security state and revoke/release authority as required.

## 71. Compromise Containment

A compromised component should have bounded authority and bounded blast radius.

```text
compromise
 ↓
limited capability
 ↓
limited resources
 ↓
limited scope
```

## 72. Blast Radius

Security architecture should minimize:

```text
resource scope
credential scope
network scope
administrative scope
```

## 73. Revocation Propagation

When authority is revoked, dependent components must receive or detect revocation within the declared security window.

## 74. Stale Authorization

A previously valid decision must not remain valid beyond its declared lifetime, epoch, or revocation semantics.

## 75. Audit Trail

Security-sensitive decisions should record:

```text
principal
resource
operation
decision
policy version
capability/reference
epoch
timestamp/logical time
request/correlation id
```

## 76. Audit Integrity

Audit evidence should be protected against unauthorized alteration according to the threat model.

## 77. Audit Privacy

Audit records must avoid becoming an uncontrolled repository of secrets or sensitive payloads.

## 78. Security Telemetry

Useful signals include:

```text
authentication failures
authorization denials
credential rotation
revocation
policy failures
capability misuse
isolation violations
privileged operations
```

## 79. Detection vs Prevention

```text
prevention
 ≠
detection
 ≠
response
```

NROS should support all three where required by deployment.

## 80. Security Incident State

A security incident may require:

```text
detect
 ↓
contain
 ↓
revoke
 ↓
fence
 ↓
recover
 ↓
audit
```

This connects Part L to Part XLVI resilience/recovery.

## 81. Security and Consensus

Membership and consensus authority must be security-controlled.

```text
membership change
    ⇒
authorized operation
```

## 82. Security and Scheduling

Schedulers must not assign work in ways that violate capability or isolation constraints.

## 83. Security and Execution

Executors must revalidate security-sensitive authority at protected execution boundaries where required.

## 84. Security and Persistence

Persisted authorization state must not outlive its declared validity.

## 85. Security and Protocol

Protocol messages carry claims; enforcement verifies those claims against trusted identity and policy.

```text
message claim
 ↓
verification
 ↓
policy
 ↓
enforcement
```

## 86. Security and API

Every API operation is subject to authentication, authorization, resource scope, and policy constraints.

## 87. Security Invariant — Identity

```text
Accept(Principal)
    ⇒
VerifiedIdentity(Principal)
```

## 88. Security Invariant — Authorization

```text
ProtectedAction(P)
    ⇒
Authorized(P)
```

## 89. Security Invariant — Capability Attenuation

```text
ChildCapability
    ⊆
ParentCapability
```

## 90. Security Invariant — Epoch

```text
AuthorityEpoch < CurrentEpoch
    ⇒
Reject(ProtectedAction)
```

## 91. Security Invariant — Revocation

```text
Revoked(Credential)
    ⇒
NoNewAuthorizedUse(Credential)
```

subject to the declared revocation propagation window.

## 92. Security Invariant — Isolation

```text
WorkloadA
    cannot access
ResourceB
```

unless an explicit policy grants that access.

## 93. Security Invariant — Least Authority

```text
GrantedAuthority
    ⊆
RequiredAuthority
```

## 94. Security Invariant — Audit

```text
PrivilegedAction
    ⇒
AuditableEvidence
```

## 95. Security Invariant — Fail Closed

```text
RequiredSecurityDecisionUnavailable
    ⇒
ProtectedActionRejected
```

unless a specifically documented emergency policy applies.

## 96. Verification Matrix

| Property | Verification question |
|---|---|
| Identity | Is every security principal verifiable? |
| Authentication | Are credentials independently validated? |
| Authorization | Is authority checked for every protected operation? |
| Policy | Is policy evaluation deterministic and versioned? |
| Capabilities | Can authority be narrowly delegated? |
| Attenuation | Can delegated authority ever expand? |
| Epochs | Are stale authorities fenced? |
| Revocation | Does revocation propagate within its declared window? |
| Secrets | Are secrets isolated from ordinary telemetry? |
| Isolation | Are filesystem/network/process/resource boundaries enforced? |
| Least privilege | Is authority minimized? |
| Tenancy | Is cross-tenant access prevented by default? |
| Fail closed | What happens when policy infrastructure fails? |
| Emergency | Is break-glass access scoped and audited? |
| Audit | Can privileged actions be reconstructed? |
| Incident response | Can identities/capabilities be revoked and workloads fenced? |
| Startup | Are security controls active before protected APIs open? |
| Shutdown | Is authority safely released? |
| Supply chain | Is component provenance verifiable where required? |
| Testing | Are security boundaries and failure modes continuously tested? |

## 97. What Part L Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- production identity federation;
- hardware-backed key storage;
- complete zero-trust enforcement;
- universal tenant isolation;
- production capability tokens;
- formally verified policy evaluation;
- complete secret-management infrastructure;
- Byzantine-secure identity consensus;
- complete host sandboxing;
- universal compromise containment.

Those require implementation-specific evidence.

## 98. Transition to Part LI

Part L establishes the security plane.

Part LI should define **observability, telemetry, tracing, metrics, logging, audit evidence, diagnostics, and forensic reconstruction**, connecting security evidence to the protocol, API, execution, persistence, and recovery planes.

```text
Part XLIX
API + service boundaries
        ↓
Part L
Identity + authorization + capabilities + isolation
        ↓
Part LI
Observability + telemetry + tracing + audit + forensics
```

## Canonical rule

> **NROS never equates identity with authority: every protected action must cross explicit authentication, authorization, policy, capability, epoch, and isolation boundaries, with sufficient evidence to reconstruct the decision.**
