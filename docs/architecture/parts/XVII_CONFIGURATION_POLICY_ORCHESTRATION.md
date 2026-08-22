# Part XVII — Configuration & Policy Orchestration

> **Series:** NROS Architecture Series  
> **Part:** XVII  
> **Role:** Declarative configuration, policy, scope, precedence, validation, dynamic reconfiguration, rollout, rollback, and effective-state orchestration  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part XVI defined stable interfaces and evolution. Part XVII defines how NROS describes, validates, resolves, applies, observes, and safely evolves configuration and policy across a running system.

The central rule is:

> **Configuration, policy, runtime state, applied configuration, and effective policy are distinct artifacts; accepting a configuration does not imply that it has been safely applied or that the resulting runtime state conforms to it.**

## 2. Core Distinctions

```text
Configuration
    ≠
Policy
    ≠
Runtime state
    ≠
Applied configuration
    ≠
Effective policy
```

These distinctions prevent desired intent from being confused with observed reality.

## 3. Configuration

Configuration describes values controlling runtime behavior.

```text
identity
endpoints
limits
timeouts
resources
feature settings
storage
transport
logging
```

Configuration should have an explicit schema and version.

## 4. Policy

Policy expresses rules or constraints governing decisions.

Examples:

```text
authorization
resource admission
scheduling
placement
retention
retry
failure handling
security
```

Policy is generally more semantic than raw configuration.

## 5. Declarative Model

NROS favors declarative descriptions for system-level intent:

```text
Desired configuration / policy
          ↓
Validation
          ↓
Planning
          ↓
Application
          ↓
Observation
```

Declarative intent must remain distinguishable from execution results.

## 6. Scope

Configuration and policy may be scoped to:

```text
global
cluster
node
process
component
entity
interface
operation
resource
request
```

The scope hierarchy must be explicit.

## 7. Precedence

When multiple sources apply, precedence determines the effective value.

A conceptual hierarchy might be:

```text
default
   ↓
system
   ↓
deployment
   ↓
component
   ↓
entity
   ↓
request
```

The actual NROS precedence order must be defined by the relevant contract rather than assumed from this example.

## 8. Inheritance

A scoped configuration may inherit values from a parent scope.

```text
Global
  ↓ inherit
Node
  ↓ inherit
Component
```

Inheritance must specify whether values are replaced, merged, appended, or reset.

## 9. Defaults

Defaults should be explicit and versioned.

```text
unset
  ↓
default resolution
  ↓
effective value
```

A default is part of the behavioral contract and can therefore become compatibility-sensitive.

## 10. Overrides

Overrides intentionally replace inherited or default values.

```text
base = X
override = Y
effective = Y
```

Overrides should be observable and attributable to a source and version.

## 11. Merge Semantics

Different configuration types require different merge rules:

```text
replace
merge
append
union
subtract
reset
```

No generic merge behavior should be assumed for every field.

## 12. Validation

Validation occurs before application where possible.

```text
Input
 ↓
Syntax validation
 ↓
Schema validation
 ↓
Semantic validation
 ↓
Policy validation
 ↓
Compatibility validation
```

A syntactically valid configuration can still be semantically invalid.

## 13. Validation Classes

Validation may include:

```text
syntax
schema
type/range
cross-field constraints
dependency constraints
resource availability
security authorization
version compatibility
runtime feasibility
```

## 14. Planning

A valid desired state may require a transition plan:

```text
Current state
      ↓
Diff
      ↓
Plan
      ↓
Apply
```

Planning can identify ordering, dependencies, resource changes, and restart requirements before mutation begins.

## 15. Atomicity

Configuration changes should define their atomicity scope.

Possible scopes:

```text
field
object
entity
component
node
deployment
cluster
```

NROS must not imply global atomicity unless the implementation provides it.

## 16. Transactions

A configuration transaction may follow:

```text
prepare
  ↓
validate
  ↓
commit
  ↓
observe
```

Failure semantics must specify whether changes are rolled back, partially applied, or left in a documented intermediate state.

## 17. Two-Phase Application

For complex changes:

```text
Prepare
  ↓
Validate
  ↓
Stage
  ↓
Commit
  ↓
Verify
```

This is a conceptual model; it does not imply distributed two-phase commit.

## 18. Dynamic Reconfiguration

Some configuration can change while entities remain active.

```text
Running
  ↓
Prepare new config
  ↓
Apply
  ↓
Verify
  ↓
Running with new config
```

Changes that cannot safely occur live must declare restart, replacement, or quiescence requirements.

## 19. Quiescence

A reconfiguration may require temporary quiescence:

```text
stop admission
   ↓
drain / checkpoint
   ↓
apply configuration
   ↓
resume
```

The required quiescence scope must be explicit.

## 20. Rollback

A failed change should have defined rollback behavior where rollback is possible.

```text
Version N
   ↓
Apply N+1
   ↓ failure
Rollback N
```

Rollback itself is a state transition and requires verification.

## 21. Roll-forward

Some changes cannot safely be reversed.

```text
N → N+1 → corrective N+2
```

The configuration contract should identify when roll-forward is required instead of rollback.

## 22. Configuration Versioning

Configuration should carry identity:

```text
configuration_id
version
source
created_at
schema_version
```

Runtime instances should expose the configuration version they actually applied.

## 23. Policy Versioning

Policies also require version identity:

```text
policy_id
policy_version
scope
source
status
```

A security or scheduling decision should be attributable to the policy version that produced it where required.

## 24. Effective Configuration

The effective configuration is the resolved result after defaults, inheritance, and overrides:

```text
Sources
  ↓
Resolution
  ↓
Effective configuration
```

The effective result is not necessarily identical to any single source document.

## 25. Effective Policy

Likewise:

```text
Policy sources
     ↓
Resolution
     ↓
Effective policy
     ↓
Decision
```

This is particularly important for Part XI authorization and Part VII resource admission.

## 26. Conflict Resolution

Conflicts may arise when multiple policies apply.

Possible strategies:

```text
priority
specificity
explicit deny
explicit allow
merge
reject ambiguity
```

The selected strategy must be defined rather than inferred.

## 27. Security Policy

Part XI remains authoritative for security decisions.

Configuration can supply policy inputs, but a configuration write must not itself grant permissions unless explicitly authorized.

```text
Configuration authority
      ≠
Runtime authorization authority
```

## 28. Resource Policy

Part VII resource semantics may be configured through policy:

```text
CPU limits
memory limits
quotas
priority
admission
rate limits
storage limits
```

The resulting policy must still obey runtime and platform constraints.

## 29. Scheduling Policy

Part VIII can consume policy for:

```text
priority
fairness
deadlines
affinity
preemption
queue selection
```

Changing scheduling policy can alter observable timing behavior and therefore requires appropriate verification.

## 30. Deployment Policy

Part XV can consume policy for:

```text
placement
affinity
anti-affinity
replication
failure domains
upgrade strategy
resource constraints
```

A desired placement policy is not proof of actual placement.

## 31. Policy Precedence and Safety

A higher-precedence policy must not silently weaken a mandatory lower-level safety constraint.

Conceptually:

```text
User preference
      ↓
System policy
      ↓
Safety invariant
```

Safety invariants remain authoritative where explicitly defined.

## 32. Configuration Sources

Sources may include:

```text
built-in defaults
static files
environment
deployment descriptor
control plane
operator command
API
remote policy service
```

Each source should have provenance and authority semantics.

## 33. Provenance

Configuration records should identify:

```text
source
actor
scope
version
timestamp
change reason
parent version
```

Sensitive actor information must follow Part XI privacy and authorization requirements.

## 34. Audit Events

Configuration and policy changes should produce structured evidence:

```text
ConfigurationLoaded
ConfigurationValidated
ConfigurationRejected
ConfigurationStaged
ConfigurationApplied
ConfigurationRolledBack
PolicyActivated
PolicyDeactivated
PolicyConflictDetected
```

These integrate with Part XIV observability.

## 35. Desired vs Effective vs Observed

Three states must remain distinct:

```text
Desired
  ↓ resolution/application
Effective
  ↓ runtime observation
Observed
```

They may temporarily diverge during rollout or failure.

## 36. Convergence

A configuration rollout is converged only when the required observation criteria are satisfied.

```text
Desired
  ↓
Apply
  ↓
Observe
  ↓
Compare
  ↓
Converged / Diverged
```

Acceptance of an update is not convergence evidence.

## 37. Partial Application

Distributed systems may temporarily contain mixed versions:

```text
Node A → v2
Node B → v1
Node C → v2
```

The system must define whether this state is supported, degraded, or invalid.

## 38. Compatibility Window

Part XVI protocol compatibility determines whether mixed configuration/policy generations can coexist safely.

```text
v1 ↔ v2
```

A rollout plan should respect the supported compatibility window.

## 39. Safe Defaults

When configuration is missing, malformed, unavailable, or rejected, the fallback must be explicit:

```text
retain previous
use safe default
fail closed
fail open
enter degraded mode
stop
```

Safety-sensitive policies should not rely on accidental fallback behavior.

## 40. Configuration Availability Failure

If a remote configuration source becomes unavailable, NROS should define whether the runtime:

```text
continue with last known valid config
use local fallback
freeze changes
enter degraded mode
stop
```

The choice depends on the contract and safety domain.

## 41. Rate of Change

Configuration churn can itself become a resource problem.

Controls may include:

```text
debounce
rate limit
batching
coalescing
cooldown
change budget
```

This connects configuration management to Part VII resource semantics and Part XIII flow control.

## 42. Verification Matrix

| Property | Verification question |
|---|---|
| Distinction | Are desired, effective, applied, and observed states separate? |
| Scope | Is configuration/policy scope explicit? |
| Precedence | Is conflict resolution deterministic? |
| Defaults | Are defaults versioned and documented? |
| Validation | Are syntax and semantic validation separated? |
| Planning | Can changes be previewed before application? |
| Atomicity | Is the atomicity boundary explicit? |
| Transactions | Are partial-application semantics defined? |
| Rollback | Is rollback supported and verified where required? |
| Versioning | Are configuration and policy versions identifiable? |
| Provenance | Can changes be attributed to an authorized source? |
| Security | Can configuration bypass authorization? |
| Convergence | Is effective state verified after rollout? |
| Compatibility | Are mixed versions within a supported window? |
| Failure | Is configuration-source failure behavior explicit? |
| Observability | Are changes recorded as evidence? |

## 43. What Part XVII Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- a universal control plane;
- distributed transactional configuration;
- automatic global policy convergence;
- zero-downtime dynamic reconfiguration;
- automatic rollback for every change;
- conflict-free policy composition;
- remote configuration availability guarantees.

Those properties require implementation and verification evidence.

## 44. Transition to Part XVIII

Part XVII defines configuration and policy orchestration.

Part XVIII should define **testing, conformance, verification, and certification**, providing the systematic mechanism for turning NROS architectural claims into reproducible evidence.

```text
Part XVI
Interfaces + ABI + protocol evolution
        ↓
Part XVII
Configuration + policy orchestration
        ↓
Part XVIII
Testing + conformance + verification
```

## Canonical rule

> **NROS separates desired intent, resolved policy, applied configuration, and observed runtime state; every mutation requires explicit scope, precedence, validation, authority, compatibility, failure semantics, and evidence of convergence appropriate to its contract.**
