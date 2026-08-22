# Part X — Identity, Configuration & Discovery

> **Series:** NROS Architecture Series  
> **Part:** X  
> **Role:** Identity, naming, configuration, discovery, dependency resolution, and topology  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part IX defined supervision and recovery. Part X defines how NROS identifies runtime entities, represents configuration, discovers peers and capabilities, resolves dependencies, and constructs runtime topology.

The central rule is:

> **Discovery establishes that something can be found; configuration establishes intended parameters; resolution establishes a usable match; readiness and health remain separate runtime properties.**

## 2. Identity Model

Every runtime entity that participates in cross-component interaction should have a stable logical identity within its scope.

```text
EntityIdentity
├── namespace
├── name
├── type
├── instance
└── generation / incarnation
```

Identity answers:

```text
Who is this entity?
```

It does not by itself answer:

```text
Where is it?
Is it configured?
Is it reachable?
Is it ready?
Is it healthy?
```

## 3. Name vs Identity

A human-readable name is not necessarily a globally unique identity.

```text
Name
  ↓ resolution
Identity
```

Aliases may exist:

```text
canonical name
alias
symbolic reference
```

Aliases should have explicit lifetime and collision rules.

## 4. Namespace

Namespaces scope names and prevent accidental collisions.

```text
/system/camera/front
/system/navigation
/system/control/motor
```

Namespace boundaries may correspond to:

```text
process
host
robot
fleet
tenant
application
```

The mapping is deployment-specific.

## 5. Generation / Incarnation

An entity identity can outlive one runtime instance.

```text
Entity A / generation 7
        ↓ restart
Entity A / generation 8
```

Generation prevents stale references from being interpreted as references to the new incarnation.

This connects identity directly to Part IV lifecycle semantics and Part IX recovery semantics.

## 6. Type Identity

An entity should expose a type identity describing its contract.

Conceptually:

```text
TypeIdentity
├── package / namespace
├── name
├── version
└── interface set
```

Type identity should be stronger than an arbitrary display string where compatibility matters.

## 7. Capability Identity

Entities may advertise capabilities independently of their type.

```text
Capability
├── identifier
├── version
├── parameters
└── constraints
```

For example:

```text
camera.capture
camera.exposure-control
camera.trigger
```

A capability claim should be treated as an advertised contract until verified or otherwise trusted according to policy.

## 8. Configuration

Configuration represents intended runtime parameters.

```text
Configuration
├── scope
├── schema/version
├── values
├── source
├── precedence
└── validation state
```

Configuration is not equivalent to runtime state.

```text
Configured
   ≠
Applied
   ≠
Active
```

## 9. Configuration Sources

Possible sources include:

```text
Built-in defaults
Static file
Environment
Command line
Deployment manifest
Parameter service
Operator input
Remote management system
```

Precedence rules must be deterministic and documented.

## 10. Configuration Validation

Configuration should pass through explicit validation:

```text
Loaded
  ↓
Parsed
  ↓
Schema validated
  ↓
Semantic validation
  ↓
Accepted / Rejected
  ↓
Applied
```

Successful parsing does not imply semantic validity.

## 11. Configuration Lifecycle

Configuration changes should be modeled as state transitions rather than arbitrary mutation.

```text
Current configuration
        ↓
Proposed configuration
        ↓
Validate
        ↓
Apply
        ↓
Verify
        ↓
Active configuration
```

If application fails, rollback or partial-application semantics must be explicit.

## 12. Dynamic Configuration

Runtime configuration changes may be:

```text
Hot-reloadable
Restart-required
Quiesce-required
Immutable
```

The entity contract must specify which parameters belong to each category.

## 13. Discovery

Discovery answers:

> **Which entities, endpoints, capabilities, or resources are currently observable within a discovery scope?**

Conceptually:

```text
Discovery
├── identity
├── type
├── endpoint
├── capabilities
├── version
├── availability metadata
└── expiration / lease
```

Discovery is observational and may become stale.

## 14. Registration

An entity may register itself with a discovery mechanism.

```text
Entity
   ↓ register
Discovery service
   ↓ publish
Registry
```

Registration should have a lease or heartbeat policy when stale entries are possible.

## 15. Discovery Lease

A discovery record may expire:

```text
Registration
   ↓ heartbeat / renewal
Valid
   ↓ timeout
Expired
```

An expired record should not be treated as proof that the entity has stopped; it establishes that the discovery record is no longer valid according to the lease policy.

## 16. Discovery vs Reachability

These properties are distinct:

```text
Discovered
    ≠
Reachable
```

A registry can advertise an endpoint that is temporarily unreachable.

Likewise:

```text
Reachable
    ≠
Ready
```

## 17. Resolution

Resolution maps a requested contract to an available provider.

```text
Request
  ↓
Discovery results
  ↓
Filter by type/version/capability/policy
  ↓
Select candidate
  ↓
Resolved endpoint
```

Resolution may be deterministic or policy-driven.

## 18. Matching

Matching criteria may include:

```text
type
version
capability
namespace
security identity
resource requirements
latency constraints
location
transport compatibility
```

A candidate that fails required constraints must not be selected merely because it is discoverable.

## 19. Version Compatibility

Providers and consumers may expose versions:

```text
API version
schema version
capability version
protocol version
```

Compatibility should be evaluated according to explicit rules rather than numeric proximity.

```text
Version 2
   ≠ automatically compatible with
Version 1
```

## 20. Dependency Model

A dependency is a runtime relationship requiring another entity, capability, resource, or service.

```text
A
 ↓ requires
B
 ↓ requires
C
```

Dependency metadata should identify:

```text
required capability
minimum contract
optional/required
startup policy
failure policy
```

## 21. Dependency States

A dependency may progress through:

```text
UNKNOWN
   ↓
DISCOVERED
   ↓
RESOLVED
   ↓
REACHABLE
   ↓
READY
   ↓
HEALTHY
```

These are deliberately separate states.

## 22. Dependency Policies

When a dependency is unavailable, policy may specify:

```text
block startup
start degraded
retry
substitute
operate independently
fail
```

The policy should be declared by the dependent entity or its supervising deployment layer.

## 23. Topology

Runtime topology can be represented as a graph:

```text
Entities = vertices
Dependencies / channels = edges
```

Example:

```text
Sensor
  ↓
Perception
  ↓
Planning
  ↓
Control
  ↓
Actuator
```

Topology is a model of relationships, not a guarantee that every edge is currently healthy.

## 24. Dynamic Topology

Entities may appear, disappear, restart, or move.

```text
ADD
REMOVE
REPLACE
RESTART
RELOCATE
```

Consumers should therefore avoid treating discovery results as immutable configuration unless explicitly guaranteed.

## 25. Endpoint Identity

An endpoint should distinguish logical identity from physical location.

```text
Logical endpoint
        ↓ binding
Physical endpoint
```

A service may move from one process, host, or transport address while retaining its logical identity.

## 26. Transport Selection

Discovery and resolution may select transport according to capability:

```text
Provider
├── local IPC
├── shared memory
└── network
```

The selected transport must satisfy the communication contract from Part V.

Discovery must not silently downgrade required transport guarantees.

## 27. Security

Discovery itself may expose sensitive topology information.

Security policies may control:

```text
who may discover
who may register
who may resolve
who may connect
who may modify configuration
```

```text
Discoverable
   ≠
Authorized to use
```

## 28. Configuration and Secrets

Configuration may contain sensitive values.

The architecture should distinguish:

```text
configuration metadata
secret reference
secret value
```

Secrets should not be unnecessarily embedded in discovery records, logs, topology exports, or diagnostics.

## 29. Discovery Consistency

Distributed discovery may be eventually consistent.

Therefore:

```text
Registry says provider exists
        ≠
Provider is currently reachable
```

Consumers requiring stronger guarantees must perform an explicit validation step.

## 30. Caching

Discovery results may be cached.

A cache should define:

```text
TTL
invalidation policy
staleness tolerance
refresh policy
failure behavior
```

A stale cache entry must not be silently treated as current state.

## 31. Startup Ordering

Discovery and dependencies interact with lifecycle startup.

A component may use:

```text
STARTING
   ↓ discover dependencies
   ↓ resolve
   ↓ validate
   ↓ establish communication
   ↓ verify readiness
READY
```

The architecture should avoid global startup-order assumptions where dependency-driven readiness is sufficient.

## 32. Shutdown and Deregistration

A clean shutdown may publish:

```text
Deregister
Release leases
Close endpoints
Release resources
```

But abrupt failure may prevent deregistration.

Lease expiration therefore remains necessary where stale registrations are possible.

## 33. Identity and Recovery

After restart:

```text
same logical identity
new generation
possibly new endpoint
```

Consumers should resolve the current generation rather than assuming the previous physical endpoint remains valid.

## 34. Configuration and Recovery

Recovery may restore configuration from:

```text
known-good configuration
checkpoint
deployment manifest
persistent configuration store
operator-approved configuration
```

Restoring configuration does not establish that the entity successfully applied it.

Post-recovery verification remains mandatory under Part IX.

## 35. Observability

Identity and discovery events should be observable:

```text
EntityRegistered
EntityDeregistered
DiscoveryLeaseExpired
EndpointResolved
ResolutionFailed
CapabilityChanged
ConfigurationLoaded
ConfigurationRejected
ConfigurationApplied
ConfigurationRollback
DependencyAvailable
DependencyUnavailable
```

Records should preserve identity and generation where relevant.

## 36. Verification Matrix

| Property | Verification question |
|---|---|
| Identity | Is logical identity stable and unambiguous within scope? |
| Generation | Are stale incarnations rejected? |
| Namespace | Are name collisions controlled? |
| Configuration | Is configuration schema and semantics validated? |
| Precedence | Are configuration sources resolved deterministically? |
| Discovery | Are stale records detectable? |
| Registration | Do leases expire correctly? |
| Resolution | Are candidates filtered against required constraints? |
| Compatibility | Are version rules actually enforced? |
| Dependency | Are dependency states distinguishable? |
| Reachability | Is reachability verified independently of discovery? |
| Readiness | Is readiness established by explicit postconditions? |
| Topology | Can topology changes be observed? |
| Security | Are discovery and resolution authorization-controlled? |
| Caching | Is stale information bounded by policy? |
| Recovery | Does restart invalidate stale physical bindings? |

## 37. What Part X Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- globally unique identity across arbitrary deployments;
- strongly consistent distributed discovery;
- automatic dependency resolution;
- universal dynamic configuration;
- zero-downtime reconfiguration;
- secure discovery by default;
- guaranteed topology convergence;
- automatic endpoint failover.

Those properties require implementation and verification evidence.

## 38. Transition to Part XI

Part X defines identity, configuration, discovery, and dependency resolution.

Part XI should define **security, trust, authorization, and capability control** across the NROS runtime.

```text
Part IX
Supervision + recovery
        ↓
Part X
Identity + configuration + discovery
        ↓
Part XI
Security + trust + authorization
```

## Canonical rule

> **NROS separates identity, configuration, discovery, resolution, reachability, readiness, and health; each is an independently observable runtime property with explicit validity and failure semantics.**
