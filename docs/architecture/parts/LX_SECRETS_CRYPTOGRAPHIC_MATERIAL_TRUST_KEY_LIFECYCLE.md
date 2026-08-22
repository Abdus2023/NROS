# Part LX — Secrets, Cryptographic Material, Trust & Key Lifecycle

> **Series:** NROS Architecture Series  
> **Part:** LX  
> **Role:** Secrets, credentials, cryptographic keys, trust stores, secure persistence, rotation, revocation, cryptographic boundaries, and confidential-data handling  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LIX established data contracts. Part LX defines how NROS protects secret and cryptographic material throughout its lifecycle.

The central rule is:

> **A secret is protected data with a lifecycle and authority model; storing or encrypting a secret is not by itself evidence that the secret is safely managed.**

## 2. Secret Material Classes

NROS may distinguish:

```text
password
API credential
token
private key
symmetric key
certificate
certificate chain
recovery secret
seed material
session credential
```

## 3. Secret vs Identifier

```text
Secret Value
    ≠
Secret Identifier
    ≠
Credential Metadata
```

Logs and telemetry should normally carry identifiers or fingerprints rather than secret values.

## 4. Credential

A credential binds a principal to an authentication mechanism.

Authentication proves possession or control according to the mechanism; authorization remains a separate decision.

## 5. Key Material

Cryptographic keys should have explicit:

```text
key_id
algorithm
purpose
owner
status
creation time
activation time
expiry
rotation policy
```

## 6. Key Purpose

A key must not silently be reused across incompatible purposes.

Examples:

```text
signing
encryption
key wrapping
authentication
session establishment
```

## 7. Algorithm Governance

Approved algorithms and parameter sets should be governed centrally where security policy requires it.

Deprecated or disallowed algorithms must not become active through accidental configuration.

## 8. Randomness

Cryptographic material must originate from an approved cryptographically secure randomness source.

## 9. Entropy Failure

Failure to obtain adequate randomness must fail closed for operations requiring cryptographic randomness.

## 10. Secret Generation

Generated secrets should satisfy declared entropy and format requirements.

Human-readable identifiers must not be mistaken for cryptographic secrets.

## 11. Secret Storage

Secret storage should define:

```text
at-rest protection
access control
key hierarchy
backup policy
retention
rotation
recovery
```

## 12. Plaintext Exposure

Plaintext secret material should exist only for the minimum necessary lifetime and scope.

## 13. Memory Handling

Where platform capabilities permit, sensitive memory may require:

```text
restricted access
explicit lifetime
zeroization
locked pages
non-dumping controls
```

These are implementation-specific guarantees and must be verified on the target platform.

## 14. Zeroization

When a secret is no longer required, implementations should clear sensitive buffers where practical.

Zeroization must not be claimed merely because a variable went out of scope.

## 15. Secret Lifetime

A secret may have:

```text
created
active
suspended
expired
revoked
destroyed
```

states.

## 16. Secret Scope

Access should be minimized by:

```text
principal
process
service
workload
resource
operation
```

## 17. Least Privilege

A component should receive only the secret material required for its declared operation.

## 18. Secret Injection

Preferred secret delivery may use:

```text
secure runtime injection
protected file descriptor
OS credential store
hardware-backed store
short-lived token
```

The mechanism depends on platform capabilities.

## 19. Environment Variables

Environment variables may expose secrets to child processes, diagnostics, crash reports, or process inspection.

They should not automatically be considered a secure secret store.

## 20. Configuration

Secret references should be separated from ordinary configuration where possible.

```text
Configuration
   ↓
Secret Reference
   ↓
Secret Provider
   ↓
Secret Value
```

## 21. Secret Provider

A provider may supply secrets from:

```text
OS key store
HSM
TPM
secret manager
secure enclave
protected local store
```

## 22. Provider Failure

Secret retrieval failure must be explicit.

Applications should not silently substitute insecure defaults for unavailable secrets.

## 23. Secret Caching

Caching secret material should define:

```text
TTL
scope
memory protection
invalidation
revocation response
```

## 24. Revocation

Revocation invalidates previously issued authority according to the credential contract.

## 25. Revocation Freshness

Systems must define how quickly revocation becomes effective across caches, sessions, replicas, and disconnected nodes.

## 26. Rotation

Rotation changes active cryptographic or credential material without necessarily changing the logical identity.

```text
Key K1
 ↓ rotation
Key K2
```

## 27. Overlapping Keys

Safe rotation may require:

```text
K1 = verify-only
K2 = sign + verify
```

for a controlled overlap period.

## 28. Key Activation

A generated key should not become authoritative merely because it exists.

Activation is a distinct lifecycle transition.

## 29. Key Expiry

Expiry should prevent new protected operations according to policy while preserving controlled verification of historical artifacts when required.

## 30. Historical Verification

Expired signing keys may remain available for verification of historical evidence without being authorized for new signatures.

## 31. Key Revocation

Revocation may require stronger treatment than expiry because previously valid material may become actively untrusted.

## 32. Key Destruction

Destruction should be explicit and evidence-backed where the security policy requires proof of destruction.

## 33. Key Backup

Backups of key material are themselves sensitive secrets and require equivalent or stronger protection.

## 34. Key Recovery

Recovery procedures should define:

```text
authority
required approvals
backup source
integrity validation
re-activation rules
rotation after recovery
```

## 35. Key Escrow

Escrow must be explicitly authorized by policy and must not become an implicit universal recovery mechanism.

## 36. Trust Store

A trust store defines which authorities or keys are trusted for a particular purpose.

```text
Trust Store
    ≠
Identity Store
    ≠
Credential Store
```

## 37. Trust Anchor

A trust anchor is authoritative input to a verification chain.

Its installation and replacement require explicit governance.

## 38. Certificate Lifecycle

Certificates may transition through:

```text
issued
active
renewed
expired
revoked
retired
```

## 39. Certificate Validation

Validation may include:

```text
chain validation
hostname / identity binding
validity period
revocation state
key usage
algorithm policy
```

## 40. Certificate Pinning

Pinning can strengthen trust in constrained environments but increases rotation and recovery complexity.

## 41. Trust Rotation

Trust-anchor rotation should support controlled overlap where required:

```text
Old Anchor
   +
New Anchor
   ↓
Migration Window
   ↓
Old Anchor Removed
```

## 42. Mutual Authentication

Mutual authentication establishes both peer identities before privileged communication.

## 43. Authentication Context

The authenticated identity should be bound to the session and protected from confused-deputy substitution.

## 44. Authorization Binding

Credentials prove identity or possession; authorization determines permitted actions.

```text
Credential
 ↓
Principal
 ↓
Policy
 ↓
Permission
```

## 45. Delegation

Delegated credentials should explicitly identify:

```text
issuer
subject
scope
audience
expiry
constraints
```

## 46. Short-Lived Credentials

Short-lived credentials reduce exposure windows but increase dependency on reliable renewal.

## 47. Credential Renewal

Renewal should occur before expiry and must not silently downgrade trust.

## 48. Session Credentials

Session credentials should be scoped to a session and invalidated when the relevant authority or session epoch changes.

## 49. Replay Protection

Security-sensitive tokens should include freshness or uniqueness mechanisms where replay would be harmful.

## 50. Nonces

Nonces must be unique within their declared scope.

## 51. Anti-Replay State

If replay prevention depends on stored state, that state becomes part of the security boundary and requires durable lifecycle semantics.

## 52. Secret Logging

Secret values must not appear in:

```text
logs
traces
metrics
exceptions
CI output
crash reports
repository artifacts
```

unless explicitly authorized and protected.

## 53. Redaction

Redaction should occur before data reaches persistent observability sinks whenever possible.

## 54. Structured Logging

Secret-bearing objects should have safe serialization behavior that excludes sensitive fields by default.

## 55. Debugging

Debug modes must not silently disable secret redaction or cryptographic verification.

## 56. Error Messages

Errors should identify failures without exposing:

```text
private keys
passwords
tokens
session credentials
secret configuration
```

## 57. Crash Dumps

Crash dumps may contain sensitive memory and should be treated as protected artifacts.

## 58. Core Dumps

Systems handling high-value secrets should explicitly govern whether core dumps are permitted.

## 59. Swap / Paging

Sensitive memory may be exposed through swap or paging depending on the platform.

Protection is platform-specific and must not be assumed.

## 60. Secure Persistence

Persistent secret stores should define:

```text
access control
integrity
confidentiality
atomicity
backup
recovery
rotation
```

## 61. Atomic Secret Update

Secret rotation should avoid states where a reader can observe a partially written credential.

## 62. Crash Consistency

Secret-store updates should remain valid across process and system crashes.

## 63. Integrity Protection

Encrypted storage should also provide integrity/authenticity protection.

```text
Confidentiality
    ≠
Integrity
```

## 64. Encryption at Rest

Encryption at rest protects stored representations according to the key hierarchy and threat model.

It does not protect secrets after legitimate plaintext access has been granted.

## 65. Envelope Encryption

A data-encryption key may be protected by a key-encryption key:

```text
Data
 ↓ DEK
Ciphertext
 ↓ KEK
Protected DEK
```

## 66. Key Hierarchy

Key hierarchies should separate:

```text
root / trust anchor
 ↓
key-encryption key
 ↓
data / session key
```

according to policy.

## 67. Cryptographic Domain Separation

Keys and derivation contexts should be separated by purpose to prevent unintended cross-use.

## 68. Key Derivation

Derived keys should use approved KDFs and explicit context labels where required.

## 69. Hashing vs Encryption

```text
Hashing
 ≠
Encryption
```

Password verification, integrity, content identity, and confidentiality have different requirements.

## 70. Password Storage

Passwords should use an approved password hashing scheme rather than reversible encryption or ordinary fast hashes.

## 71. Password Reset

Reset credentials should be short-lived, scoped, single-use where possible, and invalidated after use.

## 72. Secret Rotation During Incident

Incident response may require emergency rotation:

```text
Detect Compromise
 ↓
Revoke
 ↓
Rotate
 ↓
Invalidate Sessions
 ↓
Re-authenticate
 ↓
Audit
```

## 73. Compromised Key

A compromised key should not remain trusted merely because its nominal expiry date has not passed.

## 74. Trust Recovery

After compromise, recovery may require replacement of both credentials and trust anchors depending on the threat model.

## 75. Multi-Party Control

High-impact key operations may require multiple authorized actors or independent approvals.

## 76. Hardware-Backed Security

Where available, HSM, TPM, secure enclave, or hardware-backed keystore facilities may protect high-value keys.

Hardware-backed protection is an implementation capability, not an architectural assumption.

## 77. Non-Exportable Keys

Some keys should never leave the protected cryptographic boundary.

## 78. Remote Signing

A signing service may expose signing without exposing private key material:

```text
Caller
 ↓ authenticated request
Signer
 ↓
Signature
```

The signing service remains responsible for authorization and audit.

## 79. Cryptographic API Boundary

Applications should interact with cryptographic capabilities through narrow APIs rather than manipulating raw key material unnecessarily.

## 80. Algorithm Agility

Cryptographic protocols should permit controlled migration to newer algorithms without requiring uncontrolled rewrites of application semantics.

## 81. Post-Compromise Recovery

Systems should define how trust is re-established after suspected credential compromise.

## 82. Secure Bootstrap

Initial trust establishment must define how the first trusted credential or trust anchor is installed.

Bootstrap trust is a distinct security boundary.

## 83. Trust-on-First-Use

TOFU may be appropriate in constrained environments only when explicitly accepted by policy and accompanied by change detection.

## 84. Secret Distribution

Secret distribution should minimize fan-out.

```text
Central Secret
 ↓
Only Authorized Consumers
```

## 85. Secret Replication

Replicated secrets increase exposure surface and require explicit consistency and revocation semantics.

## 86. Secret Synchronization

Synchronization must protect against stale credentials becoming authoritative after rotation or revocation.

## 87. Offline Operation

Offline nodes may need cached credentials, but offline validity must have bounded lifetime and explicit risk acceptance.

## 88. Clock Dependence

Expiry and certificate validation depend on time correctness.

Security-sensitive systems should account for clock skew and time-source failure.

## 89. Secret Access Audit

Access to high-value secrets should produce auditable evidence without recording the secret itself.

## 90. Access Justification

Privileged secret access may require an explicit reason, ticket, workflow, or approval according to governance.

## 91. Secret Rotation Evidence

Rotation evidence should identify:

```text
old key identity
new key identity
actor / authority
time
scope
result
```

without exposing key values.

## 92. Cryptographic Evidence

Verification evidence should record:

```text
algorithm
key_id
artifact identity
verification result
policy version
```

## 93. Secret Destruction Evidence

Where required, destruction records should prove the lifecycle transition without reconstructing the secret.

## 94. Secure Export

Exporting secret material should be treated as a privileged operation and should be disabled by default where possible.

## 95. Import Validation

Imported key or credential material must be validated before activation.

## 96. Secret Boundary Invariant

```text
SecretValue
    ⇏
Log / Metric / Trace / Evidence
```

unless explicitly authorized by policy.

## 97. Key Lifecycle Invariant

```text
Created
  ⇏
Active
```

A key becomes authoritative only through an explicit activation transition.

## 98. Revocation Invariant

```text
Revoked(C)
    ⇒
NewAuthorizedUse(C) = false
```

subject to explicitly defined historical-verification exceptions.

## 99. Rotation Invariant

```text
Activate(K2)
 ∧
Retire(K1)
    ⇒
NoUnauthorizedUse(K1)
```

## 100. Trust Invariant

```text
Authenticated(P)
 ∧
TrustedFor(P, Purpose)
    ⇒
EligibleForAuthorization(P)
```

Authentication alone is insufficient.

## 101. Verification Matrix

| Property | Verification question |
|---|---|
| Secret classification | Are secret types explicitly identified? |
| Key purpose | Is every key bound to an intended purpose? |
| Generation | Is cryptographic randomness guaranteed? |
| Storage | Are confidentiality and integrity protected? |
| Memory | Is plaintext exposure minimized? |
| Zeroization | Is sensitive memory handled deliberately? |
| Access | Is least privilege enforced? |
| Injection | Are secrets delivered through controlled mechanisms? |
| Rotation | Can credentials rotate safely? |
| Revocation | Can stale authority be invalidated? |
| Trust | Are trust anchors explicitly governed? |
| Certificates | Are lifecycle and validation rules defined? |
| Replay | Is replay prevented where required? |
| Logging | Are secret values excluded from observability? |
| Persistence | Are updates crash-consistent? |
| Backup | Are secret backups equally protected? |
| Recovery | Is recovery authorized and auditable? |
| Hardware | Are hardware-backed claims platform-verified? |
| Incident | Can compromised credentials be rapidly replaced? |
| Evidence | Can security claims be verified without exposing secrets? |

## 102. What Part LX Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a production secret manager;
- universal HSM/TPM integration;
- hardware-backed non-exportable keys on every platform;
- complete automated certificate rotation;
- universal memory zeroization guarantees;
- complete compromise recovery automation;
- universal multi-party key approval;
- complete secret-access audit enforcement.

Those require implementation-specific evidence.

## 103. Transition to Part LXI

Part LX establishes the cryptographic and secret-material lifecycle.

Part LXI should define **isolation, sandboxing, privilege boundaries, process/container security, capability confinement, and execution trust domains**.

```text
Part LIX
Data models + schemas + serialization + evolution
        ↓
Part LX
Secrets + cryptographic material + trust + key lifecycle
        ↓
Part LXI
Isolation + sandboxing + privilege boundaries + trust domains
```

## Canonical rule

> **NROS treats cryptographic and secret material as lifecycle-governed authority, not ordinary configuration; creation, activation, use, rotation, revocation, recovery, and destruction are distinct security transitions that require explicit policy and evidence.**
