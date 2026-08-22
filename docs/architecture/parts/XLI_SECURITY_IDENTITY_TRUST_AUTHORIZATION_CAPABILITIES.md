# Part XLI — Security, Identity, Trust, Authorization & Capabilities

> **Series:** NROS Architecture Series  
> **Part:** XLI  
> **Role:** Identity, authentication, authorization, capabilities, trust boundaries, secrets, secure communication, threat modeling, and security invariants  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XL established observability and evidence. Part XLI defines the security and trust plane that determines who or what may act, under which authority, within which scope, and with which guarantees.

The central rule is:

> **NROS never equates knowing an identity with having authority: authentication establishes identity, authorization determines permission, capabilities carry bounded authority, and every security-sensitive action remains constrained by scope, policy, trust boundaries, resource limits, and auditability.**

## 2. Fundamental Distinctions

```text
identity
  ≠
authentication
  ≠
authorization
  ≠
capability
  ≠
trust
  ≠
permission
  ≠
authority
```

## 3. Identity

Identity answers:

```text
Who or what is this principal?
```

A principal may be:

```text
human
agent
worker
service
node
controller
device
external system
```

## 4. Authentication

Authentication establishes evidence supporting an identity claim.

```text
Principal
 ↓
Credential / proof
 ↓
Authentication
 ↓
Authenticated identity
```

Authentication does not itself grant authorization.

## 5. Credentials

Credentials can include:

```text
password
key
certificate
token
hardware-backed identity
attestation
```

Credentials should have explicit lifecycle and scope.

## 6. Authorization

Authorization evaluates whether an authenticated principal may perform an operation:

```text
Principal
 + Action
 + Resource
 + Context
 ↓
Authorization Decision
```

Possible decisions include:

```text
Allow
Deny
Require additional authority
```

## 7. Permission

A permission is a policy-level allowance.

```text
Permission
 = allowed operation within declared scope
```

Permissions should not implicitly expand to unrelated resources.

## 8. Authority

Authority is the effective ability to cause an authorized state transition.

```text
Permission
 + Valid credential/capability
 + Valid context
 = Effective authority
```

## 9. Capability

A capability is a bounded authority-bearing reference or token:

```text
Capability
 ├─ subject
 ├─ action
 ├─ resource
 ├─ scope
 ├─ constraints
 └─ expiry / epoch
```

Capabilities should grant only the minimum required authority.

## 10. Least Authority

NROS should prefer:

```text
minimum authority
for
minimum scope
for
minimum duration
```

This extends the resource and isolation principles from Parts XXXVII and XXXVIII.

## 11. Authority Attenuation

A principal may derive a narrower capability:

```text
Capability A
 ↓ attenuate
Capability B
```

B must not exceed A's authority.

## 12. Capability Delegation

Delegation should be explicit:

```text
A
 ↓ delegate
B
```

Delegated authority should preserve or reduce the original constraints.

## 13. Non-Transferability

Where required, a capability may be bound to:

```text
principal
node
session
epoch
channel
```

A stolen or replayed capability should fail when its binding requirements are not satisfied.

## 14. Scope

Authorization scope may include:

```text
task
workflow
agent
worker
node
tenant
cluster
resource class
```

Broad scope requires stronger justification.

## 15. Resource Authorization

Security authorization and resource admission are complementary:

```text
Authorized
   ≠
Admitted
```

A principal may be authorized but still denied because resource limits or system pressure prohibit the requested operation.

## 16. Time-Bounded Authority

Authority may expire:

```text
Capability
 ↓
valid_until
 ↓
Expired
```

Part XXXVI defines temporal semantics.

## 17. Epoch-Bounded Authority

Authority can be fenced by an epoch:

```text
Capability epoch 7
Current epoch 8
      ↓
Reject
```

This prevents stale actors from exercising authority after replacement or recovery.

## 18. Session Identity

Sessions should have distinct identities from long-lived principals:

```text
Principal
 ↓
Session
 ↓
Actions
```

Session compromise should not automatically imply indefinite principal authority.

## 19. Worker Identity

A restarted worker should receive a new instance identity or epoch where stale execution must be distinguishable.

This connects directly to Part XXXVIII fencing.

## 20. Node Identity

Nodes need stable identities appropriate to their trust model.

A node identity must not automatically imply unrestricted authority over every workload hosted by that node.

## 21. Tenant Identity

Tenant identity must remain distinct from infrastructure identity:

```text
Tenant
 ≠
Node owner
 ≠
Worker identity
```

## 22. Trust Boundary

A trust boundary separates domains with different assumptions:

```text
Trusted Domain
 ║
 ║ boundary
 ║
Less-Trusted Domain
```

Crossing a boundary requires explicit validation.

## 23. Trust Is Not Transitive

```text
A trusts B
B trusts C
```

does not automatically imply:

```text
A trusts C
```

Trust relationships must be explicit.

## 24. Zero-Trust Principle

NROS should avoid implicit trust based solely on network location, process adjacency, or deployment proximity.

Every sensitive operation should be evaluated according to its declared security context.

## 25. Secure Channel

Communication across a security boundary should provide appropriate:

```text
peer authentication
confidentiality
integrity
replay protection
freshness
```

The exact mechanisms depend on deployment requirements.

## 26. Message Integrity

Sensitive control messages should be protected against undetected modification.

```text
Message
 ↓
Integrity verification
 ↓
Accept / Reject
```

## 27. Replay Protection

An old valid message must not automatically remain valid forever.

Possible controls:

```text
nonce
sequence
expiry
epoch
unique command ID
```

## 28. Command Identity

Security-sensitive commands should have stable identifiers:

```text
command_id
principal
scope
epoch
```

Duplicate processing can then be detected where required.

## 29. Authentication Freshness

High-risk operations may require recent authentication or reauthentication.

```text
Authenticated
 ↓
Freshness requirement
 ↓
Sensitive action
```

## 30. Authorization Context

Authorization may depend on:

```text
principal
resource
action
tenant
workflow state
configuration
policy version
risk context
```

## 31. Policy Version

Authorization decisions should be attributable to a policy version when reproducibility matters:

```text
Decision
 ↓
Policy v17
```

Part XXXIX establishes controlled policy activation.

## 32. Deny by Default

When authorization is absent or ambiguous:

```text
Unknown authority
 ↓
Deny
```

The system must not infer permission from missing data.

## 33. Explicit Allow

Sensitive actions should require explicit policy allowance rather than broad implicit inheritance.

## 34. Separation of Duties

Critical operations may require multiple authorities:

```text
Requester
 + Approver
 ↓
Sensitive Change
```

This reduces the risk of one compromised principal exercising unrestricted control.

## 35. Administrative Authority

Administrative control should itself be scoped:

```text
admin
 ≠
unlimited authority
```

Administrative privileges should be constrained by operation, resource, tenant, and time where practical.

## 36. Emergency Authority

Emergency controls may require elevated authority, but emergency mode should remain:

```text
authenticated
scoped
time-bounded
audited
```

## 37. Break-Glass Access

A break-glass mechanism may exist for exceptional recovery.

It should produce a high-integrity audit trail and require post-event review where appropriate.

## 38. Secret Management

Secrets include:

```text
API keys
private keys
passwords
tokens
session credentials
```

They should be managed separately from ordinary telemetry and configuration visibility.

## 39. Secret Exposure Rule

```text
Observable(Configuration)
    ≠
Observable(Secrets)
```

Secret-bearing structures require explicit redaction and access controls.

## 40. Secret Rotation

Rotation should support overlapping validity where necessary:

```text
Old credential
      +
New credential
      ↓
Consumers migrate
      ↓
Old revoked
```

Part XXXIX defines controlled change and activation.

## 41. Key Epochs

Cryptographic material may use epochs:

```text
Key epoch 4
 ↓
Key epoch 5
```

Stale keys can then be rejected according to policy.

## 42. Credential Revocation

Revocation must define propagation and freshness semantics.

A revoked credential should not remain effectively valid indefinitely because of stale caches.

## 43. Authorization Cache

Cached decisions require:

```text
TTL
policy version
revocation behavior
scope
invalidation rules
```

A cache must not silently extend authority beyond policy intent.

## 44. Capability Revocation

Capabilities may require explicit revocation mechanisms:

```text
Capability issued
 ↓
Revoked
 ↓
Reject
```

If immediate revocation is impossible, maximum validity must be bounded.

## 45. Identity Lifecycle

Identity should have explicit states:

```text
Created
 ↓
Active
 ↓
Suspended
 ↓
Revoked
 ↓
Retired
```

Illegal lifecycle transitions must be rejected.

## 46. Enrollment

New nodes, agents, or services should enter a controlled enrollment process:

```text
Untrusted
 ↓
Identity verification
 ↓
Policy assignment
 ↓
Enrolled
```

Enrollment is not merely registration.

## 47. Attestation

Where supported, a component may provide evidence about its execution environment:

```text
Identity
 + Environment evidence
 ↓
Attestation
```

Attestation claims must be limited to what the evidence can establish.

## 48. Trust Score

If a deployment uses risk or trust scores, they must not silently replace explicit authorization semantics.

```text
Trust signal
 ≠
Permission
```

## 49. Security Context

Each sensitive operation should have an explicit security context containing the relevant principal, scope, policy, and authority information.

## 50. Ambient Authority

NROS should minimize authority available merely because code runs in a particular process, node, or environment.

```text
Process location
    ≠
Permission
```

## 51. Privilege Separation

Components should receive only the authority required for their function:

```text
Parser
 → parse authority

Scheduler
 → scheduling authority

Storage
 → storage authority
```

## 52. Confused Deputy Protection

A privileged component must not unknowingly use its authority on behalf of an unauthorized caller.

Capabilities should preserve the original authorization context where necessary.

## 53. Delegation Chain

Delegated operations may carry provenance:

```text
A
 ↓ delegates
B
 ↓ invokes
C
```

C should be able to determine the relevant authority chain when required.

## 54. Authority Provenance

Security-sensitive decisions should record:

```text
principal
capability
policy version
resource
action
decision
```

This integrates with Part XL observability.

## 55. Security Events

Important events include:

```text
authentication success/failure
authorization denial
credential issuance
credential revocation
capability delegation
privilege change
security-policy activation
emergency access
```

## 56. Security Telemetry

Security telemetry must preserve confidentiality and integrity while remaining diagnostically useful.

Part XL observability constraints apply.

## 57. Audit Boundary

Security audit records should be distinguishable from ordinary diagnostic logs.

```text
Security Audit
 ≠
Debug Log
```

## 58. Audit Integrity

High-value security events may require append-only or tamper-evident storage.

## 59. Tenant Security Boundary

Tenant-scoped authorization must prevent cross-tenant access even when infrastructure resources are shared.

## 60. Resource Security Boundary

Authorization should include the actual target resource, not merely its hosting process or node.

## 61. Workflow Security

Workflow transitions may require authorization:

```text
Running
 ↓ authorized transition
Suspended
```

A valid workflow request is not automatically allowed to change arbitrary workflow state.

## 62. Agent Tool Security

Agent tool access should be capability-scoped:

```text
Agent
 ↓
Capability
 ↓
Tool
 ↓
Specific operation/resource
```

Tool availability does not imply unrestricted tool authority.

## 63. External Effects

External effects require explicit authority:

```text
Agent
 ↓
Authorization
 ↓
External API
```

The system should distinguish planning from authorized execution.

## 64. Approval Gates

High-risk actions may require explicit approval:

```text
Proposed
 ↓
Risk evaluation
 ↓
Approval
 ↓
Execute
```

## 65. Policy Composition

Multiple policies may apply:

```text
Global security
 + Tenant policy
 + Workflow policy
 + Resource policy
```

Composition rules must be deterministic.

## 66. Deny Precedence

For security-critical policy composition, an explicit deny should normally dominate a conflicting allow unless a stronger, explicitly defined rule says otherwise.

## 67. Policy Conflict

Conflicting policies must produce an explicit outcome:

```text
Allow + Deny
 ↓
Policy conflict resolution
```

Silently choosing the last-loaded policy is unsafe unless explicitly defined.

## 68. Security and Configuration

Part XXXIX configuration controls must not bypass security policy:

```text
Configuration change
 ↓
Authorization
 ↓
Validation
 ↓
Activation
```

## 69. Security and Supervision

Supervisors need authority to contain faults, but supervision authority must itself be scoped.

A supervisor should not automatically gain unrelated application privileges.

## 70. Security and Recovery

Recovery may require privileged operations:

```text
Quarantine
Restart
Restore
Reconcile
```

Each action should have explicit authority.

## 71. Security and Resource Control

Security authorization does not bypass quotas or admission:

```text
Authorized
 ↓
Resource admission
 ↓
Execute
```

## 72. Security and Time

Security-sensitive operations may depend on:

```text
credential expiry
time-limited capability
time-based policy
replay window
```

Part XXXVI defines temporal guarantees.

## 73. Security and Observability

Security decisions should be observable at the appropriate evidence level:

```text
Request
 ↓
Authorization decision
 ↓
Policy version
 ↓
Action
```

Sensitive credentials must remain protected.

## 74. Threat Modeling

NROS threat modeling should identify:

```text
assets
actors
trust boundaries
entry points
attack paths
security controls
residual risk
```

## 75. Threat Categories

Relevant categories include:

```text
credential theft
replay
impersonation
privilege escalation
confused deputy
cross-tenant access
message tampering
resource exhaustion
stale authority
supply-chain compromise
```

## 76. Attack Surface

Attack surface includes:

```text
control plane
worker APIs
agent tools
network interfaces
configuration
plugins
storage
telemetry ingestion
administrative interfaces
```

## 77. Security Failure Mode

Security failures should transition explicitly:

```text
Security violation
 ↓
Deny
 ↓
Contain
 ↓
Record
 ↓
Alert / Escalate
```

## 78. Fail-Secure

When authorization state is unavailable or ambiguous for a security-critical operation:

```text
Unknown authority
 ↓
Do not execute
```

Exceptions require explicit architecture and risk justification.

## 79. Fail-Open Exceptions

If a system intentionally permits limited operation during identity or policy infrastructure failure, the exception must define:

```text
allowed actions
scope
duration
risk boundary
recovery behavior
```

## 80. Security Recovery

Recovery from security incidents may require:

```text
credential revocation
capability invalidation
session termination
node quarantine
policy rollback
forensic evidence preservation
```

## 81. Secure Defaults

New principals, services, tools, and resources should begin with minimum authority:

```text
New
 ↓
No implicit privileges
 ↓
Explicit grant
```

## 82. Security Invariant — Authorization

```text
Execute(Action)
    ⇒
Authorized(Principal, Action, Resource, Context)
```

unless an explicitly modeled system exception applies.

## 83. Security Invariant — Capability Attenuation

```text
Authority(DerivedCapability)
    ⊆
Authority(ParentCapability)
```

## 84. Security Invariant — Stale Authority

```text
Epoch(Authority) < Epoch(Current)
    ⇒
Reject(Authority)
```

where epoch fencing is required.

## 85. Security Invariant — Tenant Isolation

```text
Tenant(A) ≠ Tenant(B)
    ⇒
Access(A, Resource(B)) = Deny
```

unless an explicitly authorized cross-tenant operation exists.

## 86. Security Invariant — Least Authority

```text
GrantedAuthority
    ⊆
RequiredAuthority
```

## 87. Security Invariant — Auditability

Security-critical decisions should produce the required audit/evidence record:

```text
SecurityDecision
    ⇒
RequiredAuditEvidence
```

## 88. Verification Matrix

| Property | Verification question |
|---|---|
| Identity | Can every sensitive principal be identified? |
| Authentication | Is identity backed by an appropriate proof? |
| Authorization | Is permission evaluated for the actual action/resource? |
| Capabilities | Is authority bounded and attenuable? |
| Least privilege | Are unnecessary privileges absent? |
| Revocation | Can authority be invalidated? |
| Epoch fencing | Are stale actors rejected? |
| Replay | Can old commands be rejected? |
| Trust | Are trust boundaries explicit? |
| Tenant isolation | Can cross-tenant access be prevented? |
| Secrets | Are credentials protected and rotatable? |
| Policy | Are decisions versioned and attributable? |
| Recovery | Are privileged recovery operations controlled? |
| Agents | Are tool actions capability-scoped? |
| Audit | Are security-critical decisions recorded? |
| Telemetry | Is security observability protected? |
| Failure | Does ambiguous authority fail securely? |
| Threat model | Are attack surfaces and residual risks documented? |
| Formal assurance | Are authorization and isolation invariants explicit? |

## 89. What Part XLI Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production identity provider;
- complete mutual authentication;
- a universal capability-token implementation;
- hardware-backed attestation;
- complete secret management;
- distributed revocation with immediate propagation;
- formally verified authorization;
- complete tenant isolation enforcement;
- complete threat-model coverage;
- production-grade security audit storage.

Those require implementation-specific evidence.

## 90. Transition to Part XLII

Part XLI establishes the security and trust plane.

Part XLII should define **networking, communication topology, transport semantics, message delivery, discovery, routing, backpressure across links, connection lifecycle, partitions, and distributed communication failure modes**.

```text
Part XL
Observability + telemetry + evidence + diagnostics
        ↓
Part XLI
Security + identity + trust + authorization + capabilities
        ↓
Part XLII
Networking + communication + discovery + transport + partitions
```

## Canonical rule

> **NROS treats authority as explicit, bounded, attributable, and revocable: identity establishes who acts, authentication establishes confidence in that identity, authorization decides what may happen, capabilities carry constrained authority, and security boundaries remain enforced even during overload, recovery, reconfiguration, and distributed failure.**
