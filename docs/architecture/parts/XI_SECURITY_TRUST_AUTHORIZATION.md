# Part XI — Security, Trust & Authorization

> **Series:** NROS Architecture Series  
> **Part:** XI  
> **Role:** Identity, authentication, authorization, capabilities, trust boundaries, delegation, revocation, and audit  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part X defined identity, configuration, discovery, and dependency resolution. Part XI defines how NROS establishes trust and controls what identified actors are permitted to do.

The central rule is:

> **Identity, authentication, authorization, capability, and trust are separate security properties and require separate evidence.**

## 2. Security Model

The conceptual security path is:

```text
Identity
   ↓
Authentication
   ↓
Authenticated principal
   ↓
Authorization policy
   ↓
Permission / capability
   ↓
Action
   ↓
Audit / evidence
```

Failure at any stage must not be silently interpreted as success at another stage.

## 3. Identity vs Authentication

Identity answers:

```text
Who does this principal claim to be?
```

Authentication answers:

```text
What evidence supports that claim?
```

Therefore:

```text
Identity
   ≠
Authenticated identity
```

A name or identifier received over a channel is not automatically authenticated.

## 4. Principal

A principal is an actor to which security policy can be applied.

Possible principals include:

```text
Human operator
NROS entity
Process
Device
Service
Host
Fleet controller
External system
```

Principal identity should be stable enough for authorization and audit within its security scope.

## 5. Credentials

Authentication may use credentials such as:

```text
Cryptographic key
Certificate
Token
Platform credential
Hardware-backed identity
Shared secret
Attestation evidence
```

The credential mechanism is deployment-specific.

Credentials should not be confused with permissions.

```text
Credential
   ≠
Permission
```

## 6. Authentication

Authentication establishes evidence for a principal identity.

Conceptually:

```text
Claim
 ↓
Credential / evidence
 ↓
Verification
 ↓
Authenticated principal
```

Authentication should include freshness or replay protection where the threat model requires it.

## 7. Authorization

Authorization determines whether an authenticated principal may perform an operation on a resource or entity.

```text
Principal
   ↓
Policy evaluation
   ↓
Allow / Deny
```

Authorization should consider relevant context such as:

```text
identity
operation
resource
scope
capability
runtime state
time
security domain
policy
```

## 8. Permission

A permission represents an allowed operation under a policy.

```text
Permission
├── principal / subject
├── action
├── resource
├── scope
└── constraints
```

Permission is policy state, not merely a string attached to an identity.

## 9. Capability

A capability is an explicit authority to perform a defined operation on a defined object or class of objects.

Conceptually:

```text
Capability
├── authority
├── target
├── scope
├── constraints
├── issuer
└── validity
```

Possession of a capability can be treated as an authorization mechanism when the platform supports it.

## 10. Capability vs Advertisement

Part X defined advertised capabilities such as:

```text
camera.capture
camera.trigger
```

These are functional capabilities, not automatically security capabilities.

```text
Advertised capability
    ≠
Authorization capability
```

A service advertising `camera.capture` does not imply that every caller is authorized to invoke it.

## 11. Trust

Trust expresses the degree to which a principal, credential, platform, or assertion is accepted for a specific purpose.

Trust is contextual:

```text
Trusted for discovery
   ≠
Trusted for control
```

Likewise:

```text
Authenticated
   ≠
Authorized
```

## 12. Trust Domains

A trust domain defines a boundary in which security assumptions are shared.

Examples:

```text
Process
Host
Robot
Fleet
Deployment
Administrative domain
```

Cross-domain interaction should establish an explicit trust relationship rather than inheriting trust accidentally.

## 13. Trust Boundaries

A trust boundary exists where data or authority crosses from one security domain to another.

```text
Domain A
   │
   │ authenticated channel
   ▼
Domain B
```

Inputs crossing a trust boundary must be treated according to the receiving domain's validation and authorization policy.

## 14. Least Authority

NROS should follow the principle of least authority:

> Give an actor only the authority required for the intended operation and scope.

For example:

```text
Camera controller
   ↓
control camera X
```

is preferable to:

```text
Camera controller
   ↓
control every device
```

unless broader authority is explicitly required.

## 15. Scope

Authorization should have explicit scope.

Possible scopes include:

```text
single resource
entity
namespace
subsystem
host
fleet
operation class
```

Scope should not silently expand because an entity moves between environments.

## 16. Delegation

A principal may delegate limited authority.

```text
Principal A
   ↓ delegates
Principal B
   ↓
Capability / permission
```

Delegation should preserve:

```text
scope
constraints
expiration
issuer
chain / provenance
```

A delegate should not automatically gain authority beyond the delegator's own authority.

## 17. Credential Lifecycle

Credentials require lifecycle management:

```text
Provisioned
   ↓
Active
   ↓
Rotated
   ↓
Expired / Revoked
```

Credential rotation must not accidentally grant new authority.

## 18. Revocation

Authority may need to be revoked:

```text
Valid
  ↓
Revoked
  ↓
Denied
```

Revocation semantics should specify:

```text
scope
propagation delay
cache behavior
existing sessions
existing leases
existing capabilities
```

## 19. Session Security

Authenticated sessions should have explicit lifecycle semantics:

```text
Created
   ↓
Authenticated
   ↓
Authorized
   ↓
Active
   ↓
Expired / Revoked / Closed
```

A session remaining connected does not prove that its authority remains valid.

## 20. Secure Communication

Security can be applied to Part V communication mechanisms through properties such as:

```text
Confidentiality
Integrity
Authentication
Replay protection
Authorization
```

Transport security does not automatically establish application-level authorization.

## 21. Message Authorization

A message should be authorized according to the intended operation, not merely according to the fact that it arrived over an authenticated channel.

```text
Authenticated channel
       ≠
Every message authorized
```

This is especially important for control-plane operations.

## 22. Resource Authorization

Part VII resources require security policy.

Examples:

```text
allocate CPU domain
open device
reserve memory
control actuator
access storage
configure network
```

Resource ownership and security authority remain distinct concepts.

```text
Resource owner
   ≠
Security administrator
```

## 23. Configuration Authorization

Part X configuration operations should be authorization-controlled.

```text
Read configuration
Modify configuration
Apply configuration
Rollback configuration
```

Different principals may receive different authority for each operation.

## 24. Discovery Authorization

Discovery may itself require authorization.

Policies can control:

```text
who may discover
what may be discovered
which metadata is visible
who may register
who may resolve
```

Topology information can be sensitive even when service invocation is separately protected.

## 25. Secret Handling

Secrets should be represented through references where possible.

```text
Configuration
   ↓
Secret reference
   ↓
Secret provider
   ↓
Runtime use
```

Secrets should not unnecessarily appear in:

```text
logs
discovery records
topology exports
diagnostics
fault records
source configuration
```

## 26. Secret Zeroization

Where supported by the platform, sensitive material should have controlled lifetime and zeroization semantics.

The architecture must not claim perfect memory erasure where the platform cannot provide such guarantees.

## 27. Secure Boot / Platform Trust

Some deployments may establish a chain of trust from platform initialization.

Conceptually:

```text
Boot trust
   ↓
Platform identity
   ↓
Runtime identity
   ↓
Application identity
```

Platform trust can strengthen authentication evidence but does not automatically authorize every application operation.

## 28. Attestation

An entity may provide evidence about its execution environment.

```text
Identity
 +
Platform evidence
 +
Software measurement
   ↓
Attestation result
```

Attestation should be evaluated according to explicit policy and threat model.

## 29. Authorization Policy

Policies should be explicit and versioned where reproducibility matters.

Conceptually:

```text
Policy
├── subject
├── action
├── resource
├── conditions
├── effect
└── version
```

Policy evaluation should produce an auditable decision where required.

## 30. Deny by Default

For security-sensitive operations, absence of an applicable authorization should result in denial unless an explicit policy says otherwise.

```text
No authority
   ↓
Deny
```

Default behavior should be defined per security domain and operation class.

## 31. Fail-Secure vs Availability

Security failures may conflict with availability requirements.

Possible policy outcomes include:

```text
Fail closed
Fail degraded
Use cached authority
Require reauthentication
Enter safe state
```

The selected policy must be explicit for each critical operation.

## 32. Authorization and Lifecycle

Lifecycle transitions may require authority.

Examples:

```text
start entity
stop entity
restart entity
pause entity
modify configuration
enter maintenance mode
```

The lifecycle state machine from Part IV should therefore consume authorization decisions rather than bypass them.

## 33. Authorization and Recovery

Part IX recovery actions are security-sensitive operations.

```text
Supervisor
   ↓ authorized recovery action
Restart / isolate / revoke / failover
```

A supervisor must possess sufficient authority for the recovery operation it performs.

## 34. Audit

Security-relevant actions should generate structured audit evidence.

```text
AuditRecord
├── event_id
├── timestamp
├── principal
├── authenticated identity
├── action
├── target
├── authorization decision
├── policy version
├── result
└── correlation / generation
```

Audit records should support reconstruction without exposing unnecessary secrets.

## 35. Non-Repudiation Boundary

Audit evidence may support accountability, but NROS should not claim legal non-repudiation merely because an event was logged.

Cryptographic signing, trusted time, key custody, and organizational controls may be required for stronger claims.

## 36. Security Events

Relevant events include:

```text
AuthenticationSucceeded
AuthenticationFailed
AuthorizationGranted
AuthorizationDenied
CredentialRotated
CredentialExpired
CredentialRevoked
CapabilityIssued
CapabilityRevoked
SessionCreated
SessionExpired
PolicyChanged
SecretAccessed
SecurityViolation
```

Security events should integrate with the general observability model rather than creating an unrelated event system.

## 37. Threat Model Boundary

Security guarantees depend on assumptions.

A security claim should identify relevant assumptions such as:

```text
trusted boot chain
credential protection
network threat model
physical access
operator trust
cryptographic primitives
platform isolation
```

A secure protocol design cannot compensate for an explicitly compromised trust anchor without a recovery mechanism.

## 38. Verification Matrix

| Property | Verification question |
|---|---|
| Identity | Is the principal identity unambiguous within scope? |
| Authentication | Is identity backed by valid evidence? |
| Freshness | Is replay prevented where required? |
| Authorization | Is each protected operation policy-checked? |
| Least authority | Is granted authority limited to required scope? |
| Delegation | Is delegated authority bounded by the issuer's authority? |
| Revocation | Does revoked authority become unusable within the defined bound? |
| Session | Are expired/revoked sessions rejected? |
| Secrets | Are secret values excluded from inappropriate observability paths? |
| Discovery | Are sensitive topology records access-controlled? |
| Configuration | Are configuration mutations authorized? |
| Recovery | Are recovery actions authorized? |
| Audit | Can security decisions be reconstructed? |
| Policy | Are policy versions and changes attributable? |
| Trust | Are cross-domain trust assumptions explicit? |

## 39. What Part XI Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- complete authentication infrastructure;
- universal hardware-backed identity;
- cryptographic attestation;
- complete capability-based authorization;
- automatic credential rotation;
- instantaneous distributed revocation;
- tamper-proof audit logs;
- secure boot;
- protection against arbitrary physical compromise;
- security certification.

Those properties require implementation, platform, cryptographic, operational, and verification evidence.

## 40. Transition to Part XII

Part XI defines security, trust, and authorization.

Part XII should define **persistence, state durability, checkpointing, recovery state, and consistency**, connecting runtime state with Part IX recovery while clearly distinguishing volatile execution state from durable state.

```text
Part X
Identity + configuration + discovery
        ↓
Part XI
Security + trust + authorization
        ↓
Part XII
Persistence + state durability
```

## Canonical rule

> **NROS separates identity, authentication, authorization, capability, and trust; authority must be explicit, scoped, revocable, auditable, and evaluated at the boundary where the protected operation occurs.**
