# Part XV — Deployment, Composition & Isolation

> **Series:** NROS Architecture Series  
> **Part:** XV  
> **Role:** Deployment topology, composition, placement, isolation, multi-node execution, and deployment lifecycle  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part XIV defined observability and evidence. Part XV maps the logical NROS runtime into concrete deployment environments: processes, hosts, containers, devices, networks, and isolation domains.

The central rule is:

> **Logical topology, deployment topology, physical topology, and runtime health are distinct models and must not be conflated.**

## 2. Topology Layers

NROS distinguishes:

```text
Logical topology
      ↓ realization
Deployment topology
      ↓ placement
Physical topology
      ↓ observation
Runtime state
```

A logical dependency does not imply a particular process or host placement.

## 3. Logical Entity

A logical entity is defined by its NROS identity and contract.

```text
Entity
├── identity
├── type
├── capabilities
├── dependencies
└── lifecycle contract
```

It may execute locally or remotely depending on deployment policy.

## 4. Component

A component is a deployable composition unit.

```text
Component
├── entities
├── configuration
├── resources
├── dependencies
└── deployment constraints
```

Composition allows multiple logical entities to share a deployment boundary where appropriate.

## 5. Process Boundary

A process can contain one or more runtime entities.

```text
Process P1
├── Entity A
├── Entity B
└── Entity C
```

Process boundaries can provide failure and resource isolation, but they do not automatically provide security isolation strong enough for every threat model.

## 6. Host / Node

A host or node provides physical or virtual execution resources:

```text
Node
├── CPU
├── memory
├── storage
├── network
└── devices
```

Multiple processes and components may execute on one node.

## 7. Container / Sandbox

Containers or sandboxes may provide additional isolation:

```text
Host
 ├── Sandbox A
 │    └── Process
 └── Sandbox B
      └── Process
```

The exact isolation guarantees are platform-specific and must not be assumed from the label alone.

## 8. Device Boundary

Some entities interact with physical devices:

```text
NROS entity
    ↓
Driver / interface
    ↓
Device
```

Device access should be explicitly authorized and resource-controlled.

## 9. Deployment Descriptor

A deployment description may specify:

```text
entities
components
configuration
placement
resource requirements
security domains
network requirements
restart policy
version constraints
```

Deployment descriptors describe intended topology, not proof of actual runtime state.

## 10. Placement

Placement maps logical components to execution locations:

```text
Entity A → Node 1
Entity B → Node 1
Entity C → Node 2
```

Placement may consider:

```text
CPU
memory
accelerators
latency
bandwidth
power
fault domain
security domain
device locality
```

## 11. Affinity

Affinity expresses a preference or requirement:

```text
prefer same node
require same host
prefer same NUMA domain
require device locality
avoid same failure domain
```

Affinity must be distinguished from a hard placement constraint.

## 12. Anti-Affinity

Anti-affinity prevents or discourages co-location:

```text
Replica A ──X── Replica B
```

This may improve fault tolerance when replicas would otherwise share a failure domain.

## 13. Failure Domains

Deployment should model possible correlated failures:

```text
process
host
rack
power domain
network segment
region
```

Replicas placed inside the same failure domain do not provide independence against failures of that domain.

## 14. Resource Placement

Part VII resource semantics continue into deployment.

A component may declare:

```text
CPU requirement
memory requirement
storage requirement
network requirement
device requirement
latency requirement
```

The deployment system must distinguish requested, allocated, and actually available resources.

## 15. Isolation Domains

Isolation may exist at multiple layers:

```text
security domain
resource domain
failure domain
network domain
lifecycle domain
```

One boundary does not automatically imply all others.

## 16. Communication Placement

Part V transport choices interact with deployment:

```text
same process
   ↓
in-process mechanism

same host
   ↓
IPC / shared memory

remote host
   ↓
network transport
```

These are deployment realizations, not interchangeable semantic guarantees.

## 17. Local vs Distributed Transparency

NROS may preserve a logical communication contract across placement changes:

```text
Entity A → Entity B
```

while the physical path changes from:

```text
local IPC
```

to:

```text
network transport
```

Any transport-dependent semantics must remain explicit.

## 18. Startup

Deployment startup may follow:

```text
Deployment accepted
      ↓
Resources admitted
      ↓
Components created
      ↓
Entities started
      ↓
Dependencies resolved
      ↓
Readiness verified
      ↓
Deployment active
```

Global ordering should not be assumed where dependency-driven startup is sufficient.

## 19. Shutdown

A deployment may shut down through:

```text
quiesce
stop new work
complete / cancel work
persist required state
release resources
terminate entities
```

The exact sequence depends on lifecycle and safety requirements.

## 20. Rolling Replacement

A component can be replaced incrementally:

```text
Version A
   ↓
Start compatible Version B
   ↓
Verify B
   ↓
Redirect / transition traffic
   ↓
Retire A
```

Replacement must not violate identity, compatibility, or state-transition rules.

## 21. Blue/Green and Parallel Deployment

Two deployment versions may coexist:

```text
Blue  → current
Green → candidate
```

Traffic or dependencies can transition only after candidate readiness and required verification.

## 22. Canary Deployment

A new version may receive limited traffic:

```text
100% old
   ↓
95% old / 5% new
   ↓
50% / 50%
   ↓
100% new
```

Promotion should depend on explicit health and verification criteria.

## 23. Version Compatibility

Deployment must consider:

```text
entity contract
message schema
protocol version
configuration schema
state schema
security policy
transport capability
```

A component being individually executable does not establish deployment compatibility.

## 24. State During Replacement

Replacement interacts with Part XII:

```text
Old instance
   ↓ checkpoint / transfer
New instance
   ↓ restore
verify
```

State transfer must respect schema, ownership, generation, integrity, and authorization rules.

## 25. Identity During Replacement

Replacement may preserve logical identity while changing generation:

```text
Entity X / G7
      ↓ replacement
Entity X / G8
```

Consumers must not accidentally route work to the retired generation.

## 26. Discovery During Deployment

Part X discovery records may change during deployment:

```text
register new
validate
switch
expire / deregister old
```

Discovery state must not be treated as instantaneous proof of deployment convergence.

## 27. Deployment Health

Deployment health is an aggregate assessment.

```text
Deployment health
    ↓
component health
    ↓
entity health
    ↓
dependency health
```

A healthy deployment does not require every noncritical entity to be perfect if the deployment contract explicitly permits degraded operation.

## 28. Observability Integration

Part XIV should expose deployment events:

```text
DeploymentAccepted
PlacementAssigned
ResourceAllocated
ComponentStarted
EntityReady
DeploymentDegraded
ComponentReplaced
PlacementChanged
DeploymentFailed
DeploymentCompleted
```

These records should include topology and generation information where relevant.

## 29. Security Integration

Part XI applies to deployment operations:

```text
create deployment
modify placement
start component
stop component
attach device
change security domain
export topology
```

Deployment authority must be explicit.

## 30. Resource Isolation

Resource isolation may include:

```text
CPU quotas
memory limits
storage quotas
network limits
device access controls
priority classes
```

A resource request is not proof that the resource was actually granted.

## 31. Network Partition

Distributed deployments must tolerate possible partitions:

```text
Node A  X  Node B
```

The deployment contract must specify whether entities:

```text
block
retry
operate degraded
fail over
enter safe state
```

## 32. Placement Failure

A requested placement may fail because of:

```text
insufficient resources
missing device
security policy
compatibility failure
node unavailable
fault-domain constraint
```

Deployment should report the reason rather than silently selecting an incompatible placement.

## 33. Reconciliation

Actual deployment may diverge from intended deployment:

```text
Desired topology
       ↓ reconcile
Observed topology
```

Reconciliation should identify:

```text
missing
unexpected
misplaced
stale
unhealthy
incompatible
```

resources or entities.

## 34. Desired vs Observed State

The distinction is fundamental:

```text
Desired state
    ≠
Observed state
```

A deployment controller should not declare convergence merely because the desired configuration has been accepted.

## 35. Convergence

Conceptually:

```text
Desired
  ↓
Plan
  ↓
Apply
  ↓
Observe
  ↓
Compare
  ↓
Converged / Diverged
```

Convergence requires explicit observation criteria.

## 36. Disaster Recovery

Deployment recovery may involve:

```text
replacement node
restored state
recreated components
re-established dependencies
revalidated security
```

Recovery does not automatically restore the same physical topology.

## 37. Multi-Node Coordination

Operations spanning nodes require explicit coordination semantics.

```text
Node A
   ↘
    coordination
   ↗
Node B
```

NROS must not imply distributed atomicity unless the deployment protocol actually provides it.

## 38. Verification Matrix

| Property | Verification question |
|---|---|
| Topology | Are logical, deployment, and physical topology distinct? |
| Placement | Are placement decisions explicit and observable? |
| Resources | Are requested and allocated resources distinguishable? |
| Isolation | Are security, resource, and failure boundaries explicit? |
| Affinity | Are hard constraints separated from preferences? |
| Failure domains | Are correlated failures modeled? |
| Compatibility | Are version and schema constraints enforced? |
| Startup | Is readiness explicitly verified? |
| Replacement | Can instances be replaced without stale routing? |
| State transfer | Is restored state validated? |
| Discovery | Is deployment convergence distinguished from registration? |
| Security | Are deployment operations authorized? |
| Reconciliation | Can desired and observed topology be compared? |
| Network failure | Are partition semantics explicit? |
| Observability | Are deployment transitions evidenced? |

## 39. What Part XV Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- a universal deployment orchestrator;
- container-native execution;
- automatic placement optimization;
- zero-downtime upgrades;
- distributed consensus;
- automatic disaster recovery;
- hardware-level isolation on every platform;
- topology convergence under arbitrary failures.

Those properties require implementation and verification evidence.

## 40. Transition to Part XVI

Part XV defines deployment and physical realization.

Part XVI should define **APIs, interfaces, ABI boundaries, protocol evolution, compatibility, and version negotiation**, connecting logical contracts to stable implementation boundaries.

```text
Part XIV
Observability + evidence
        ↓
Part XV
Deployment + composition + isolation
        ↓
Part XVI
Interfaces + ABI + protocol evolution
```

## Canonical rule

> **NROS separates desired, logical, deployed, physical, and observed topology; deployment is a controlled realization process whose placement, isolation, compatibility, convergence, and lifecycle transitions require explicit policy and evidence.**
