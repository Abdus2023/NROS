# Part XXIX — Isolation, Sandboxing, Capabilities & Containment

> **Series:** NROS Architecture Series  
> **Part:** XXIX  
> **Role:** Trust boundaries, execution domains, capabilities, privilege, sandboxing, isolation, containment, revocation, escape resistance, and security lifecycle  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXVIII defined resource ownership and lifecycle. Part XXIX defines the boundaries within which NROS actors may execute and interact with resources.

The central rule is:

> **NROS must treat authority as explicit and bounded: authentication, authorization, capability possession, privilege, isolation, sandboxing, and containment are distinct properties, and every execution domain must have a defined trust boundary and revocation path.**

## 2. Fundamental Distinctions

```text
authentication
  ≠
authorization
  ≠
capability
  ≠
privilege
  ≠
isolation
  ≠
sandboxing
  ≠
containment
```

A successful identity check must not implicitly grant unrestricted authority.

## 3. Trust Boundary

A trust boundary identifies where assumptions about an actor or component change:

```text
Trusted Domain
      │
      │ boundary
      ▼
Less-Trusted Domain
```

Crossing a boundary requires an explicit policy and validation point.

## 4. Execution Domain

An execution domain groups code, state, identity, and resources under a defined security policy:

```text
Execution Domain
 ├─ identity
 ├─ authority
 ├─ resources
 ├─ policy
 ├─ isolation boundary
 └─ lifecycle
```

Examples may include an agent, workflow, session, plugin, worker, or host process.

## 5. Authentication

Authentication establishes an identity claim:

```text
Principal
   ↓ authenticate
Identity established
```

Authentication alone does not determine what that identity may do.

## 6. Authorization

Authorization evaluates whether an identity may perform an operation:

```text
Identity + Operation + Resource + Context
                    ↓
                 Policy
                    ↓
             Allow / Deny
```

The decision should be attributable to an explicit policy.

## 7. Capabilities

A capability represents authority to perform a defined class of operations on a defined resource or scope.

```text
Capability
 ├─ resource/scope
 ├─ permitted operations
 ├─ constraints
 ├─ issuer
 ├─ generation
 └─ expiration/revocation semantics
```

Possession of a capability should be sufficient only for the authority it explicitly encodes.

## 8. Capability Attenuation

A broader capability may be reduced before delegation:

```text
Capability A
   ↓ attenuate
Capability B
```

The derived capability must not gain authority beyond its parent.

## 9. Least Authority

Execution should receive only the authority required for its declared task:

```text
required authority
      ↓
minimum capability set
      ↓
execution
```

Unused privileges increase the potential impact of faults and compromise.

## 10. Privilege

Privilege is authority granted by the execution environment, operating system, runtime, or NROS control plane.

Privilege should be explicitly mapped to capabilities where possible.

```text
privilege
   ↓
capability boundary
   ↓
operation
```

## 11. Privilege Separation

Sensitive operations should be isolated into narrower domains:

```text
Unprivileged Worker
        ↓ request
Privileged Broker
        ↓ validate
Sensitive Operation
```

The broker must not become an unrestricted privilege tunnel.

## 12. Sandboxing

A sandbox restricts what code can observe or manipulate:

```text
Sandbox
 ├─ filesystem policy
 ├─ network policy
 ├─ process policy
 ├─ device policy
 ├─ memory/resource limits
 └─ syscall/runtime policy
```

Sandboxing is a mechanism; containment is the broader security property.

## 13. Isolation Dimensions

Isolation may apply to:

```text
memory
filesystem
network
processes
IPC
devices
CPU
storage
credentials
secrets
logs
persistent state
```

A domain isolated in one dimension may remain coupled in another.

## 14. Process Isolation

Separate processes can provide a stronger boundary than language-level separation, depending on the host environment.

The architecture must identify the actual enforcement layer rather than assuming process separation alone is sufficient.

## 15. Memory Isolation

Memory isolation should prevent one execution domain from directly reading or modifying another domain's protected memory.

Shared memory, if permitted, requires an explicit contract:

```text
shared region
access mode
ownership
lifetime
synchronization
revocation
```

## 16. Filesystem Isolation

Filesystem access should be scoped:

```text
read scope
write scope
execute scope
path restrictions
mounts
sensitive paths
```

Path-based restrictions must account for links, traversal, mount changes, and alternate access mechanisms.

## 17. Network Isolation

Network authority should specify:

```text
allowed destinations
allowed ports/protocols
inbound listeners
DNS access
proxy requirements
bandwidth limits
connection limits
```

Network connectivity is a capability, not merely a runtime default.

## 18. Device Isolation

Access to physical or virtual devices should be explicitly granted:

```text
device identity
operation scope
exclusive/shared mode
lifetime
revocation
```

Device access can cross otherwise strong software isolation boundaries.

## 19. Credential Isolation

Secrets should not become ambient authority.

Instead:

```text
Execution Domain
      ↓ scoped request
Credential Broker
      ↓ policy
Specific credential/action
```

Credentials should have bounded scope and lifetime where practical.

## 20. Secret Lifetime

Secret-bearing resources should follow Part XXVIII lifecycle semantics:

```text
issued
 ↓
active
 ↓
rotated / renewed
 ↓
revoked
 ↓
expired
```

Revocation must invalidate future use according to the credential contract.

## 21. Containment

Containment limits the consequences of a faulty or compromised component:

```text
Fault / Compromise
       ↓
Containment Boundary
       ↓
Limited Impact
```

Containment should be analyzed in terms of reachable resources and authorities.

## 22. Blast Radius

A security design should identify the maximum authority reachable after compromise:

```text
compromised domain
      ↓
reachable capabilities
      ↓
reachable resources
      ↓
potential impact
```

Least authority and isolation reduce this blast radius.

## 23. Escape Resistance

A sandbox should define which paths could cross its boundary:

```text
IPC
filesystem
network
shared memory
device access
privileged APIs
runtime bugs
host interfaces
```

An implementation should not claim complete sandboxing without identifying its enforcement boundary.

## 24. Brokered Operations

Sensitive operations may pass through a broker:

```text
Agent
 ↓ capability
Broker
 ↓ policy check
Resource
```

The broker should validate both authority and request constraints.

## 25. Confused Deputy Prevention

A privileged broker must not use its authority on behalf of an untrusted caller without validating the caller's delegated authority.

Requests should preserve principal and capability context where required.

## 26. Ambient Authority

Ambient authority is authority available without explicit capability possession.

NROS should minimize ambient authority because it makes dependency and security reasoning harder.

## 27. Capability Revocation

Revocation can use:

```text
expiration
revocation list
capability generation
lease termination
key rotation
execution-domain termination
```

Revocation semantics must define how quickly existing authority becomes unusable.

## 28. Generation-Based Revocation

A resource or capability generation can invalidate stale authority:

```text
Generation 7
   ↓ revoke
Generation 8
```

Requests carrying generation 7 should fail once generation 8 becomes authoritative.

## 29. Lease-Based Authority

Time-bounded authority can be represented as a lease:

```text
Acquire
 ↓
Active until deadline
 ↓
Renew or expire
```

Distributed leases require Part XXV's authority and fencing semantics.

## 30. Isolation and Resource Lifecycle

Part XXVIII provides the resource boundary:

```text
Capability
    ↓
Resource allocation
    ↓
Use
    ↓
Release / revoke
```

Reclamation must also revoke stale access paths.

## 31. Isolation and Persistence

Part XXVII applies to security state that must survive restart:

```text
policy
capability metadata
revocation state
resource ownership
credential generations
```

A restart must not resurrect revoked authority.

## 32. Isolation and Networking

Part XXVI defines transport semantics, while Part XXIX determines whether an execution domain is permitted to use them.

```text
Transport capability
       ↓
Network policy
       ↓
Connection
```

Successful connection establishment is not proof of authorization for every subsequent operation.

## 33. Isolation and Execution

Part XXIV execution semantics should identify security-sensitive environment dependencies:

```text
capability availability
resource availability
policy state
sandbox configuration
```

If these alter execution behavior, they belong to the relevant execution environment.

## 34. Isolation Failure Modes

Important failure modes include:

```text
privilege escalation
capability forgery
capability leakage
sandbox escape
confused deputy
stale authority
credential reuse
resource boundary bypass
cross-tenant data access
```

Each should have an explicit detection and response strategy.

## 35. Fail-Closed Behavior

When an authorization or isolation decision cannot be established safely, sensitive operations should fail closed where the security contract requires it:

```text
Unknown authority
      ↓
Deny
```

Availability tradeoffs must be explicit.

## 36. Fail-Open Exceptions

If any subsystem intentionally permits degraded operation, the exception must specify:

```text
scope
maximum duration
allowed operations
risk boundary
recovery condition
observability
```

“Temporary” must not become an indefinite bypass.

## 37. Policy Evaluation

Security policy should have deterministic inputs where practical:

```text
principal
operation
resource
capability
context
state
```

Policy decisions should be auditable without exposing sensitive secrets.

## 38. Policy Versioning

Policy changes should carry explicit versions:

```text
Policy V1
   ↓ migration / transition
Policy V2
```

Mixed-version behavior must be defined for distributed systems.

## 39. Security Events

Relevant lifecycle events include:

```text
authenticated
authorized
denied
capability issued
capability attenuated
capability revoked
privilege changed
sandbox created
sandbox terminated
isolation violation
escape detected
credential rotated
```

Events should have stable identities and appropriate integrity protection.

## 40. Observability

Part XIV should expose security-relevant facts such as:

```text
execution-domain identity
policy version
capability identity/generation
authorization result
sandbox state
resource scope
revocation state
security violations
```

Sensitive values should be redacted according to security policy.

## 41. Formal Capability Safety

A conceptual invariant is:

```text
CanExecute(P, Op, R)
    ⇒
∃ capability C
such that
C authorizes Op on R
and C is valid for P's context.
```

The exact model may differ by implementation, but authority should remain explicit.

## 42. Containment Invariant

A useful safety property is:

```text
Compromise(D)
    ⇒
Impact(D)
⊆
ReachableResources(D, policy)
```

The purpose of containment is to keep the reachable set bounded.

## 43. Revocation Invariant

For generation-based authority:

```text
Revoked(C, generation=g)
    ⇒
operations using generation g
cannot obtain new protected effects.
```

Existing effects must be handled according to the operation's commit semantics.

## 44. Verification Matrix

| Property | Verification question |
|---|---|
| Identity | Is the execution principal unambiguous? |
| Authorization | Is every protected operation policy-checked? |
| Capability | Is authority explicit and scoped? |
| Attenuation | Can delegated authority only decrease? |
| Privilege | Are privileged operations isolated? |
| Sandboxing | What boundary actually enforces restrictions? |
| Memory | Can domains access protected memory? |
| Filesystem | Are filesystem scopes enforced safely? |
| Network | Are destinations and protocols bounded? |
| Devices | Is device access explicitly controlled? |
| Secrets | Are credentials scoped and isolated? |
| Revocation | Can authority be reliably revoked? |
| Containment | Is blast radius bounded? |
| Escape | Are boundary-crossing paths identified and tested? |
| Persistence | Can restart resurrect revoked authority? |
| Distribution | Are authority changes protected across nodes? |
| Fail closed | What happens when policy state is unavailable? |
| Observability | Can security decisions be reconstructed? |
| Formal assurance | Are authority and containment invariants explicit? |

## 45. What Part XXIX Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a universal capability system;
- a formally verified sandbox;
- complete process isolation;
- complete filesystem/network/device isolation;
- universal privilege separation;
- formally proven containment;
- automatic capability revocation;
- complete escape detection;
- production-grade secret brokering.

Those require implementation-specific evidence.

## 46. Transition to Part XXX

Part XXIX defines isolation, authority, and containment.

Part XXX should define **identity, naming, addressing, discovery, trust establishment, and identity lifecycle**, connecting execution domains and capabilities with distributed communication and durable identity.

```text
Part XXVIII
Resource lifecycle + ownership + allocation + reclamation
        ↓
Part XXIX
Isolation + sandboxing + capabilities + containment
        ↓
Part XXX
Identity + naming + addressing + discovery + trust establishment
```

## Canonical rule

> **NROS treats authority as bounded state: every protected operation must resolve through an explicit identity, capability or policy decision, execution boundary, and resource scope, while isolation and containment must limit blast radius and revocation must prevent stale authority from surviving beyond its declared lifecycle.**
