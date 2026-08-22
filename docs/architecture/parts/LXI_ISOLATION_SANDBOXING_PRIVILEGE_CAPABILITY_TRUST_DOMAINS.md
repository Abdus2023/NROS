# Part LXI — Isolation, Sandboxing, Privilege, Capabilities & Trust Domains

> **Series:** NROS Architecture Series  
> **Part:** LXI  
> **Role:** Process isolation, privilege boundaries, capability confinement, sandboxing, namespaces, resource isolation, IPC boundaries, execution trust domains, escape resistance, and recovery  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LX defines how secrets and cryptographic authority are protected. Part LXI defines the execution boundaries in which that authority and other resources are allowed to operate.

The central rule is:

> **NROS treats isolation as an explicit security boundary: possessing code, an identity, or a credential does not automatically grant access to another execution domain, resource, namespace, or privilege level.**

## 2. Isolation Stack

```text
Host
 ↓
Trust Domain
 ↓
Process / Runtime
 ↓
Sandbox
 ↓
Namespace
 ↓
Capability
 ↓
Resource
```

## 3. Process Boundary

A process boundary separates execution state and authority according to the operating-system model.

A process boundary is useful only to the extent that the underlying platform enforces it.

## 4. Trust Domain

A trust domain is a set of execution components that share an explicitly defined security authority model.

```text
Same Trust Domain
    ≠
Same Process
```

## 5. Trust Domain Separation

Components with incompatible trust levels should not share unnecessary authority.

## 6. Privilege

Privilege is authority granted by the underlying platform or NROS policy.

Examples include:

```text
filesystem access
network access
device access
process control
memory operations
credential access
administrative operations
```

## 7. Privilege Boundary

A privilege boundary exists when execution transitions between authority levels.

```text
Unprivileged
   ↓ controlled transition
Privileged
```

## 8. Least Privilege

Every workload should receive the minimum authority required for its declared function.

## 9. Privilege Drop

Components that require elevated startup authority should drop unnecessary privileges before processing untrusted workload data where possible.

## 10. Privilege Retention

Retaining elevated privileges after initialization must be explicitly justified and governed.

## 11. Capability Model

A capability represents authority to access a specific resource or perform a specific operation.

```text
Capability
 ↓
Resource / Operation
```

## 12. Ambient Authority

Ambient authority is access available without explicitly presenting a capability.

NROS should minimize ambient authority at security-sensitive boundaries.

## 13. Capability Delegation

Capabilities may be delegated only within an explicit scope:

```text
issuer
subject
resource
operation
constraints
expiry
```

## 14. Capability Revocation

Delegated capabilities require revocation or expiry semantics where authority must be withdrawable.

## 15. Capability Attenuation

A delegated capability should be reducible to a narrower authority set.

```text
Parent Capability
      ↓ attenuation
Restricted Capability
```

## 16. Sandbox

A sandbox constrains what an execution unit may access or perform.

Possible controls include:

```text
filesystem
network
syscalls
IPC
devices
processes
resources
```

## 17. Sandbox Policy

Sandbox policy should define both allowed and denied behavior.

A sandbox that is merely documented but not enforced is not a security boundary.

## 18. Fail-Closed

When a mandatory isolation policy cannot be established, the protected workload should fail closed rather than silently running with broader authority.

## 19. Namespace Isolation

Namespaces may isolate:

```text
process IDs
filesystem views
network identities
users/groups
mounts
IPC resources
```

The exact mechanisms are platform-specific.

## 20. Filesystem Isolation

A workload may receive a restricted filesystem view rather than unrestricted host access.

```text
Workload
 ↓
Sandbox Root
 ↓
Allowed Paths
```

## 21. Path Traversal

Filesystem APIs must prevent traversal outside the declared authority boundary.

## 22. Device Isolation

Device access should be explicitly granted.

```text
Device
    ⇏
Automatic Workload Access
```

## 23. Network Isolation

Network authority may be restricted by:

```text
interface
address
port
protocol
destination
traffic class
```

## 24. Egress Policy

Outbound network access should be governed independently from inbound exposure.

## 25. Ingress Policy

Inbound connections should be accepted only through explicitly authorized listeners or channels.

## 26. IPC Isolation

Inter-process communication should be treated as a security boundary when communicating across trust domains.

## 27. IPC Authorization

An IPC endpoint should authenticate or otherwise establish the identity of its peer where required.

## 28. IPC Capability Passing

Passing a handle or capability through IPC transfers authority and must therefore be governed as a security-sensitive operation.

## 29. Handle Inheritance

Child processes should not inherit unnecessary file descriptors, sockets, devices, or privileged handles.

## 30. Process Creation

Process creation should define:

```text
parent authority
child identity
environment
inherited handles
filesystem view
network view
resource limits
```

## 31. Child Process Trust

A parent must not assume that child code has the same trust level merely because it was launched by the parent.

## 32. Environment Isolation

Sensitive environment state should be minimized before launching less-trusted workloads.

## 33. Resource Isolation

Isolation may cover:

```text
CPU
memory
storage
file descriptors
threads
process count
network bandwidth
IPC queues
```

## 34. Resource Quotas

Resource limits should prevent one workload from consuming all shared capacity.

## 35. CPU Limits

CPU scheduling and quotas may prevent starvation or denial of service between workloads.

## 36. Memory Limits

Memory ceilings should be explicit where untrusted workloads can otherwise exhaust the host.

## 37. File Descriptor Limits

Descriptor limits reduce exhaustion risk from unbounded connection or file creation.

## 38. Process Limits

Process and thread limits protect the host against process-fork or thread-exhaustion attacks.

## 39. Storage Limits

Writable storage should be bounded where workload-generated data could exhaust shared storage.

## 40. Time Limits

Sandboxed operations may require CPU or wall-clock deadlines.

## 41. Scheduler Interaction

Isolation policy should remain compatible with NROS resource admission and scheduling semantics from Part LIII.

## 42. Workload Identity

Each isolated workload should have a stable execution identity or instance identity where policy decisions depend on it.

## 43. Instance Identity

```text
Workload Identity
    ≠
Process ID
```

Process IDs are platform-level implementation identifiers and may be reused.

## 44. Execution Attestation

Where supported, a trust domain may require evidence that code is running under an expected environment or configuration.

## 45. Attestation Boundary

Attestation claims must identify:

```text
measured component
measurement
platform context
policy
verification result
```

## 46. Measurement

A measurement may identify executable or configuration state using a cryptographic digest or platform-defined measurement mechanism.

## 47. Measurement vs Identity

```text
Code Hash
    ≠
Runtime Identity
```

A trusted workload may require both.

## 48. Secure Boot Relationship

Where platform secure boot exists, NROS may rely on platform-established trust roots, but the exact chain must be verified on the target system.

## 49. Container Boundary

Containers can provide useful isolation but should not automatically be treated as equivalent to a virtual machine or hardware security boundary.

## 50. Virtual Machine Boundary

Virtualization may provide stronger isolation than process or container boundaries, depending on the threat model and hypervisor implementation.

## 51. Hardware Boundary

Hardware-enforced isolation can strengthen trust separation but remains platform-specific.

## 52. Boundary Strength

NROS should classify isolation boundaries according to actual enforcement strength rather than naming alone.

Possible levels:

```text
logical
process
container
virtual machine
hardware
```

## 53. Boundary Crossing

Every transition across trust domains should identify:

```text
source
 destination
identity
capability
protocol
validation
policy
```

## 54. Privileged Helper

A small privileged helper can reduce the amount of privileged code exposed to untrusted workloads.

```text
Untrusted Workload
 ↓ narrow IPC
Privileged Helper
 ↓
Protected Resource
```

## 55. Helper API

Privileged helpers should expose narrow, typed, auditable operations rather than generic arbitrary-command interfaces.

## 56. Confused Deputy

A privileged component must not use its authority on behalf of an untrusted caller without validating the caller's authorization for the requested resource.

## 57. Ambient Credential Exposure

Privileged processes must not unintentionally expose credentials to less-trusted children or plugins.

## 58. Plugin Isolation

Plugins should be treated as separate trust domains unless explicitly trusted.

## 59. Extension Boundary

Extensions should receive only the APIs and capabilities explicitly required by their declared function.

## 60. Dynamic Code

Dynamically loaded code should be subject to integrity, provenance, compatibility, and authorization checks where the threat model requires them.

## 61. Loading Policy

Code loading should define:

```text
source
integrity
signature / provenance
version
allowed capabilities
```

## 62. Dependency Trust

Third-party dependencies inherit neither unlimited trust nor unlimited authority merely because they are dependencies.

## 63. Supply-Chain Boundary

Build and runtime trust should distinguish:

```text
source provenance
build provenance
artifact integrity
runtime authorization
```

## 64. Artifact Verification

Executable artifacts may require signature or digest verification before activation.

## 65. Runtime Verification

Runtime activation should fail when a mandatory integrity requirement cannot be established.

## 66. Sandbox Escape

An escape occurs when execution obtains authority outside its declared isolation boundary.

## 67. Escape Detection

Detection may use:

```text
policy violations
unexpected syscalls
unexpected filesystem access
unexpected network access
integrity events
resource anomalies
```

## 68. Escape Response

Possible responses include:

```text
terminate workload
revoke capabilities
isolate instance
quarantine artifact
rotate credentials
emit security evidence
```

## 69. Quarantine

A suspected compromised workload may be isolated from further privileged resources while evidence is preserved.

## 70. Kill Semantics

Termination should define whether it means:

```text
request termination
force termination
process tree termination
resource revocation
network isolation
```

## 71. Cleanup

Termination should reclaim resources without accidentally exposing or reusing sensitive state.

## 72. Secret Revocation on Isolation Failure

If a workload crosses a critical boundary, associated credentials may require immediate revocation according to security policy.

## 73. Checkpoint Interaction

Checkpointing an isolated workload must preserve security context and must not allow restoration into a broader authority domain without explicit validation.

## 74. Restore Boundary

Restoration should verify:

```text
workload identity
checkpoint integrity
schema/version
security policy
capabilities
resource limits
```

## 75. Migration Across Hosts

Moving a workload to another host may require re-establishing trust and capabilities rather than blindly copying authority.

## 76. Host Identity

A workload's trust decision may depend on host identity or platform attestation.

## 77. Cross-Host Capability

Capabilities valid on one host should not automatically become valid on another host.

## 78. Isolation and Scheduling

Scheduler decisions should not violate isolation policy.

```text
Schedulable
    ≠
Authorized
```

## 79. Isolation and Storage

Storage mounts and handles crossing an isolation boundary should be explicitly authorized and tracked.

## 80. Isolation and Networking

Network namespaces or policy boundaries should be reconciled with NROS service-discovery and transport semantics.

## 81. Isolation and Observability

Monitoring systems should receive sufficient evidence to detect policy violations without unnecessarily exposing sandbox contents.

## 82. Isolation and Audit

Security-relevant boundary crossings should produce auditable records where required.

## 83. Policy Versioning

Isolation policy should be versioned so that an execution instance can be evaluated against the policy that authorized it.

## 84. Policy Update

Policy changes should define whether existing workloads:

```text
continue under old policy
re-evaluate
restart
terminate
```

## 85. Policy Revocation

Revoked isolation policy should not remain silently authoritative for newly admitted workloads.

## 86. Fail-Safe Defaults

Absent explicit permission, access should default to denied at protected boundaries.

## 87. Deny vs Error

A policy denial should be distinguishable from a platform failure to enforce the policy.

## 88. Enforcement Evidence

A documented policy is not evidence of enforcement.

Evidence may require:

```text
runtime tests
system policy inspection
negative tests
CI checks
platform verification
```

## 89. Negative Testing

Isolation testing should attempt prohibited actions:

```text
forbidden file access
forbidden network access
forbidden device access
forbidden IPC
forbidden privilege escalation
```

## 90. Escape Testing

Where appropriate, security testing should verify that attempts to cross the boundary are blocked or detected.

## 91. Fuzzing Boundary Interfaces

IPC parsers, privileged helpers, sandbox APIs, and policy evaluators should be fuzz-tested where practical.

## 92. Resource Exhaustion Testing

Isolation tests should include:

```text
CPU exhaustion
memory exhaustion
FD exhaustion
process exhaustion
storage exhaustion
connection exhaustion
```

## 93. Recovery Testing

Tests should verify that compromised or terminated workloads do not retain access to revoked capabilities.

## 94. Formal Isolation Invariant

```text
Workload W
 ∧
CapabilitySet(W) = C
    ⇒
Access(W) ⊆ Resources(C)
```

## 95. Formal Privilege Invariant

```text
RequiredPrivilege(O) ⊆ GrantedPrivilege(W)
```

and no additional privilege should be assumed implicitly.

## 96. Formal Boundary Invariant

```text
Cross(T1 → T2)
    ⇒
Authenticated
 ∧
Authorized
 ∧
Validated
 ∧
PolicySatisfied
```

where the transition crosses trust domains.

## 97. Formal Fail-Closed Invariant

```text
MandatoryIsolationUnavailable
    ⇒
ProtectedWorkloadNotAdmitted
```

## 98. Formal Capability Invariant

```text
DelegatedCapability
    ⊆
IssuerCapability
```

Delegation cannot increase authority.

## 99. Formal Restore Invariant

```text
Restore(CP)
    ⇒
Integrity(CP)
 ∧
PolicyCompatible(CP)
 ∧
CapabilitiesRevalidated
```

## 100. Verification Matrix

| Property | Verification question |
|---|---|
| Trust domain | Are trust domains explicitly defined? |
| Privilege | Is least privilege enforced? |
| Capability | Are permissions explicitly scoped? |
| Delegation | Can authority only be attenuated? |
| Sandbox | Is the boundary actually enforced? |
| Filesystem | Are paths constrained? |
| Network | Are ingress and egress governed? |
| IPC | Are cross-domain peers authenticated/authorized? |
| Handles | Is inheritance minimized? |
| Resources | Are CPU/memory/storage limits enforced? |
| Dynamic code | Is code provenance checked? |
| Supply chain | Are artifacts verified? |
| Attestation | Are claims tied to measurable evidence? |
| Escape | Can prohibited access be detected/blocked? |
| Quarantine | Can compromised workloads be isolated? |
| Revocation | Are capabilities invalidated after compromise? |
| Checkpoint | Is security context preserved during restore? |
| Migration | Is authority re-established on another host? |
| Policy | Are policy versions and updates governed? |
| Evidence | Can enforcement be independently verified? |

## 101. What Part LXI Does Not Claim

This Part does not claim that the current NROS implementation already has:

- universal OS-level sandboxing;
- complete container isolation;
- VM or hardware isolation;
- universal capability-based security;
- production seccomp/AppArmor/SELinux integration;
- complete device isolation;
- complete workload attestation;
- automatic sandbox escape detection;
- universal privileged-helper architecture;
- complete cross-host trust re-establishment.

Those require implementation-specific and platform-specific evidence.

## 102. Transition to Part LXII

Part LXI establishes execution isolation and trust boundaries.

Part LXII should define **runtime lifecycle, process supervision, failure containment, restart policy, health state, graceful shutdown, and service recovery orchestration**.

```text
Part LX
Secrets + cryptographic material + trust
        ↓
Part LXI
Isolation + sandboxing + privilege + capabilities
        ↓
Part LXII
Runtime lifecycle + supervision + recovery orchestration
```

## Canonical rule

> **NROS treats isolation as enforceable authority confinement: every trust-domain crossing, capability delegation, process launch, restore, and privileged operation must remain within explicitly authorized boundaries, and mandatory isolation failure prevents protected workload admission.**
