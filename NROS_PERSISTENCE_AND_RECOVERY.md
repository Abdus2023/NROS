# NROS Persistence & Recovery (Part LXI–LXX)

The previous layer established the **canonical object model**. We can now address one of the most important architectural differences between ROS and NROS:

> **ROS primarily discovers computational participants; NROS must discover and coordinate semantic participants.**

The result is not simply a better service-discovery mechanism. It is a **federated runtime model**.

# 1. From ROS Master to NROS Federation

A simplified ROS1 model is:

```text
             ROS MASTER
            /    |    \
           /     |     \
        Node    Node   Node
         |       |      |
       Topic   Topic   Topic
```

The Master provides registration and discovery.

NROS should instead look like:

```text
                 NROS FEDERATION
                       │
       ┌───────────────┼────────────────┐
       ↓               ↓                ↓
     Agent          Resource        Capability
       │               │                │
       └───────────────┼────────────────┘
                       ↓
                Coordination
```

There is no requirement for one universal authority to own the entire graph.

# 2. The Federation Concept

An NROS federation is:

> **A set of independently operating runtime domains that can discover, trust, coordinate with, and delegate to one another.**

For example:

```text
             Federation
                  │
      ┌───────────┼───────────┐
      ↓           ↓           ↓
   Robot A     Robot B     Control Center
      │           │           │
   sensors     actuators   planning
```

Each domain can retain local autonomy.

# 3. Runtime Domain

Introduce a first-class concept:

```text
Domain
```

A domain represents a coordination boundary.

It may correspond to:

```text
robot
vehicle
factory
warehouse
fleet
simulation
cloud
operator station
```

A domain owns some subset of:

```text
agents
resources
capabilities
state
authority
policies
```

# 4. Domain ≠ Network

A network is a communication topology.

A domain is a semantic/administrative boundary.

One domain may span:

```text
multiple networks
```

and one network may carry:

```text
multiple domains
```

Therefore:

```text
Network ≠ Domain
```

# 5. Local Autonomy

Each domain should be capable of operating independently:

```text
Domain A
    ↓
local scheduler
local authority
local resources
local execution
```

If connectivity to another domain disappears:

```text
Domain A ───── X ───── Domain B
```

A should not necessarily collapse.

This is essential for robots and industrial systems.

# 6. Federation Link

Domains communicate through an explicit federation link:

```text
Domain A
    │
    │ Federation Link
    │
Domain B
```

The link can expose:

```text
identity
capabilities
resources
events
goals
work
authority
```

subject to policy.

# 7. Discovery

Discovery should answer several different questions.

### Identity discovery

```text
Who exists?
```

### Capability discovery

```text
Who can perform X?
```

### Resource discovery

```text
Where is resource Y?
```

### Authority discovery

```text
Who may authorize X?
```

### Service discovery

```text
Where can I invoke X?
```

These should not collapse into one opaque "service registry."

# 8. Capability Discovery

Suppose an agent requires:

```text
Capability:
    3D localization
```

Discovery returns candidates:

```text
Localization Agent A
Localization Agent B
Localization Agent C
```

but NROS should also expose:

```text
quality
latency
availability
authority
cost
resource requirements
trust level
```

# 9. Capability Advertisement

A capability advertisement might conceptually contain:

```text
CapabilityAdvertisement {
    capability_id
    provider
    version
    input_schema
    output_schema
    constraints
    resource_cost
    latency_bounds
    validity
    authority_requirements
}
```

This is far richer than:

```text
/topic/foo
```

# 10. Resource Discovery

Resources should be discoverable dynamically.

Example:

```text
Resource:
    charging-station-4

State:
    AVAILABLE

Location:
    warehouse-A

Capacity:
    1 robot

Reservation:
    none
```

A scheduler can therefore reason about actual availability.

# 11. Authority Discovery

This is more subtle.

An agent might discover:

```text
Capability:
    emergency_stop
```

but that does not tell it whether it is authorized to invoke it.

Authority discovery should therefore expose:

```text
policy
scope
issuer
validity
delegation
```

without necessarily exposing sensitive policy internals.

# 12. Naming

NROS needs hierarchical names.

For example:

```text
/fleet/warehouse_a/robot_07/lidar
```

But names should not become identity.

The distinction:

```text
Name  → human/system routing
ID    → stable identity
```

is important.

# 13. Namespaces

A namespace provides contextual naming:

```text
/fleet-A/robot-7
/fleet-B/robot-7
```

Both may contain:

```text
/navigation
```

without ambiguity.

Internally:

```text
EntityId
```

remains authoritative.

# 14. Identity Resolution

The runtime may therefore maintain:

```text
Name
   ↓
EntityId
   ↓
Current Endpoint
```

This separates:

```text
logical identity
```

from:

```text
physical location
```

A robot can move between networks without changing its identity.

# 15. Discovery Is Eventually Consistent

Distributed discovery cannot assume instantaneous global truth.

Therefore:

```text
DiscoveryState
```

must include:

```text
observed_at
expires_at
source
epoch
confidence
```

A stale advertisement must be distinguishable from a current one.

# 16. Leases

A powerful mechanism is the lease.

Example:

```text
CapabilityLease {
    provider: AgentA
    capability: Navigate
    expires_at: T
}
```

When the lease expires:

```text
provider
   ↓
no longer considered currently available
```

This avoids permanent stale registrations.

# 17. Resource Leases

The same mechanism applies to resources:

```text
Robot Arm
   ↓
Lease → Work-42
```

When the lease expires:

```text
Work-42
   ↓
resource becomes reclaimable
```

This is especially important under failures.

# 18. Authority Leases

Authority itself can be temporary:

```text
Operator
   ↓
delegates
   ↓
Agent A
   ↓
Capability X
   ↓
until T
```

This prevents indefinite authority persistence.

# 19. Epochs

Leases alone are insufficient.

Suppose:

```text
Agent A
epoch = 10
```

crashes and restarts:

```text
Agent A
epoch = 11
```

An old message from epoch 10 must not be accepted as current authority.

Therefore:

> **NROS should use monotonic epochs to invalidate stale execution contexts.**

# 20. Epoch + Lease

A robust authority context becomes:

```text
Authority
    +
Lease
    +
Epoch
```

A command is valid only if:

```text
epoch == current_epoch
AND
lease is valid
AND
authority permits operation
```

# 21. Stale Command Protection

Imagine:

```text
Robot A
   ↓
Work-17
   ↓
network disconnect
```

The robot recovers and starts:

```text
Work-18
```

A delayed packet from Work-17 arrives.

NROS should reject it because:

```text
Work-17
epoch < current epoch
```

This is a critical distributed-safety property.

# 22. Federation Trust

Domains need a trust relationship.

Possible trust levels:

```text
UNKNOWN
DISCOVERED
AUTHENTICATED
AUTHORIZED
TRUSTED
RESTRICTED
REVOKED
```

Discovery alone should never imply trust.

# 23. Trust Is Not Authority

An important distinction:

```text
Trust:
    "I believe this identity is legitimate."

Authority:
    "This identity is allowed to perform X."
```

Therefore:

```text
Trusted Agent
```

does not mean:

```text
Unlimited Authority
```

# 24. Authentication vs Authorization

NROS should preserve the standard separation:

```text
Authentication
    ↓
Who are you?

Authorization
    ↓
What may you do?
```

And then:

```text
Policy
    ↓
Under which conditions?
```

# 25. Delegation

Autonomous systems frequently need delegation.

Example:

```text
Fleet Coordinator
        ↓
delegates
        ↓
Robot Manager
        ↓
delegates
        ↓
Robot Agent
```

Delegation must preserve:

```text
scope
constraints
expiration
issuer
chain
```

# 26. Authority Chain

Conceptually:

```text
Root Authority
      ↓
Fleet Authority
      ↓
Robot Authority
      ↓
Agent Authority
      ↓
Work
```

NROS should be able to answer:

> Why was this action authorized?

with a verifiable chain.

# 27. Distributed Coordination

Now we reach the harder problem.

Multiple agents may want the same resource:

```text
Agent A ─┐
         ├── Robot Arm
Agent B ─┘
```

NROS needs explicit coordination semantics.

# 28. Reservation

A resource may first be:

```text
AVAILABLE
```

then:

```text
RESERVED
```

then:

```text
ALLOCATED
```

This gives the scheduler a predictable ownership protocol.

# 29. Reservation Protocol

Conceptually:

```text
Agent
  ↓
Reserve(Resource)
  ↓
ReservationGranted
  ↓
Commit
  ↓
ResourceAllocated
```

If commitment is not completed:

```text
lease expires
     ↓
reservation released
```

# 30. Optimistic vs Pessimistic Coordination

NROS can support both.

### Pessimistic

Acquire resource before proceeding:

```text
Reserve
  ↓
Execute
```

### Optimistic

Proceed under assumptions:

```text
Plan
  ↓
Execute
  ↓
Conflict?
  ↓
Recover/Replan
```

The policy decides which model is appropriate.

# 31. Conflict

Two agents may produce incompatible intentions:

```text
Agent A:
    use elevator

Agent B:
    disable elevator
```

NROS should not hide the conflict.

Instead:

```text
Conflict
   ↓
Detect
   ↓
Classify
   ↓
Policy
   ↓
Resolve / Escalate
```

# 32. Conflict Classes

Potential categories:

```text
RESOURCE_CONFLICT
AUTHORITY_CONFLICT
TEMPORAL_CONFLICT
GOAL_CONFLICT
SAFETY_CONFLICT
POLICY_CONFLICT
STATE_CONFLICT
```

Each can have different resolution rules.

# 33. Goal Coordination

Multiple goals can coexist:

```text
Goal A:
    deliver package

Goal B:
    conserve battery

Goal C:
    maintain safety margin
```

The decision system must reason over:

```text
priority
constraints
utility
authority
risk
```

rather than treating goals as a flat queue.

# 34. Priority Is Not Enough

A naive system:

```text
priority = 100
```

cannot adequately represent:

```text
safety > mission
```

because some constraints are absolute.

Therefore NROS should distinguish:

```text
Hard Constraints
Soft Constraints
Preferences
```

# 35. Constraint Hierarchy

Example:

```text
SAFETY
  ↓
LEGAL / POLICY
  ↓
AUTHORITY
  ↓
RESOURCE
  ↓
TEMPORAL
  ↓
MISSION
  ↓
OPTIMIZATION
```

This is only a conceptual default.

Actual deployments can define their own policy hierarchy.

# 36. Federation Topology

A federation can be:

```text
              Control Domain
             /       |       \
            /        |        \
       Robot A    Robot B    Robot C
         / \         |         / \
       Agent        Agent     Agent
```

Or hierarchical:

```text
Global
  ↓
Region
  ↓
Facility
  ↓
Robot
  ↓
Component
```

Or peer-to-peer:

```text
A ─── B
│ \   │
│  \  │
C ─── D
```

NROS should not require one topology.

# 37. Federation Scope

Not every domain needs to know everything.

A robot might expose:

```text
public:
    localization capability
    battery state class

private:
    internal planner
    raw sensor data
    security configuration
```

Thus discovery itself becomes policy-controlled.

# 38. Information Boundaries

NROS should support:

```text
PUBLIC
DOMAIN
AUTHORIZED
PRIVATE
SECRET
```

or an equivalent information classification model.

This becomes important when robotics systems interact with cloud and enterprise infrastructure.

# 39. Distributed Event Propagation

Events can remain local:

```text
Robot A
  ↓
local event
```

or federated:

```text
Robot A
  ↓
event
  ↓
Fleet Domain
```

Propagation should be controlled by policy.

Not every event deserves global dissemination.

# 40. Event Scope

An event may have:

```text
scope = LOCAL
scope = DOMAIN
scope = FEDERATION
scope = PUBLIC
```

For example:

```text
motor_temperature_changed
```

may remain local.

But:

```text
robot_emergency_stop
```

may need federation-wide propagation.

# 41. Causal Propagation

Federated events should retain causal metadata:

```text
Event A
   ↓ caused
Event B
   ↓ caused
Event C
```

Across domains:

```text
Robot A Event
       ↓
Federation Event
       ↓
Robot B Action
```

This makes distributed debugging possible.

# 42. Causal Context

A distributed event may carry:

```text
origin
parent_event
causal_chain
domain
epoch
logical_time
```

The exact encoding is an implementation decision.

The semantic requirement is:

> NROS must be able to reconstruct causality across boundaries where evidence permits.

# 43. Partition Tolerance

A federation must assume:

```text
network partitions
packet loss
latency
duplication
reordering
node crashes
clock drift
```

Therefore NROS cannot depend on perfect communication.

# 44. Local-First Principle

During partition:

```text
             X
Domain A ───────── Domain B
```

each domain should determine:

```text
what can continue safely
what must stop
what can degrade
what requires remote authority
```

This is a policy decision.

# 45. Safety Under Partition

For safety-critical operations:

```text
Remote authority unavailable
        ↓
Do not assume permission
        ↓
Enter predefined safe behavior
```

For non-critical operations:

```text
Remote authority unavailable
        ↓
continue under cached/leased authority
```

provided the lease remains valid.

# 46. Federation Reconciliation

When connectivity returns:

```text
Domain A ─────── Domain B
        reconnect
```

the system must reconcile:

```text
state
events
leases
resource ownership
authority epochs
work status
faults
```

This cannot simply mean:

```text
"copy database A over B"
```

# 47. Reconciliation Strategy

A useful model:

```text
Discover divergence
       ↓
Compare epochs
       ↓
Compare causal histories
       ↓
Resolve conflicts
       ↓
Invalidate stale authority
       ↓
Re-establish leases
       ↓
Verify active Work
```

# 48. Split-Brain Protection

Suppose two domains independently believe:

```text
Robot-7 is controlled by me.
```

After reconnection:

```text
A owns Robot-7
B owns Robot-7
```

NROS must have an ownership/authority protocol that can detect and resolve this.

Possible responses:

```text
freeze
quarantine
select authority winner
require human intervention
```

depending on policy.

# 49. No Universal Consensus Requirement

NROS should not require consensus for every operation.

That would make small embedded systems unnecessarily expensive.

Instead:

```text
local operation
    → local consistency

shared resource
    → coordination protocol

global authority
    → stronger coordination
```

Consistency requirements should match semantic importance.

# 50. Federation as a Hierarchy of Trust

We can therefore model:

```text
Local Runtime
      ↓
Local Domain
      ↓
Trusted Federation
      ↓
External Federation
      ↓
Untrusted World
```

Each boundary has different:

```text
identity
authority
information
latency
failure
trust
```

# 51. NROS Discovery Plane

The resulting architecture becomes:

```text
┌──────────────────────────────────────────┐
│              APPLICATIONS                │
└──────────────────┬───────────────────────┘
                   │
┌──────────────────▼───────────────────────┐
│            SEMANTIC RUNTIME              │
│                                           │
│ Agent / Goal / Work / Resource / Policy  │
└──────────────────┬───────────────────────┘
                   │
┌──────────────────▼───────────────────────┐
│            FEDERATION PLANE              │
│                                           │
│ Discovery / Identity / Authority / Lease │
│ Delegation / Coordination / Reconciliation│
└──────────────────┬───────────────────────┘
                   │
┌──────────────────▼───────────────────────┐
│             TRANSPORT PLANE              │
│                                           │
│ DDS / Zenoh / QUIC / IPC / CAN / etc.    │
└───────────────────────────────────────────┘
```

# 52. Three Planes

This suggests a clean NROS architecture:

### 1. Semantic Plane

```text
What does the system mean?
```

### 2. Coordination Plane

```text
Who may do what, where, and when?
```

### 3. Transport Plane

```text
How does information physically move?
```

ROS historically emphasizes the third.

NROS elevates the first two.

# 53. The NROS Federation Contract

A domain joining a federation should be able to advertise:

```text
Identity
Capabilities
Resources
Policies
Authority scope
Protocol versions
Time capabilities
Execution domains
Health
```

The receiving domain decides what it accepts.

# 54. Versioning

Federated systems cannot assume synchronized software versions.

Therefore capabilities and protocols need explicit versions:

```text
Capability:
    Navigate@2
```

and:

```text
Protocol:
    NROS/1
```

Compatibility should be negotiated.

# 55. Capability Negotiation

Example:

```text
A:
    supports Navigate@1..3

B:
    requires Navigate@2

Negotiation:
    compatible → use v2
```

This is much safer than assuming that:

```text
same name = same semantics
```

# 56. Semantic Compatibility

Version compatibility must include more than wire format.

Two implementations might both support:

```text
Navigate@2
```

but differ in:

```text
timing guarantees
safety semantics
coordinate conventions
precision
```

Therefore capability contracts need semantic metadata.

# 57. NROS Federation Invariants

The federation layer should enforce principles such as:

```text
1. Discovery does not imply authority.

2. Authentication does not imply authorization.

3. Expired leases cannot authorize current work.

4. Older epochs cannot supersede newer epochs.

5. Local operation must not depend unnecessarily on remote availability.

6. Shared resources require explicit ownership/coordination.

7. Cross-domain events preserve provenance.

8. Partition recovery must reconcile authority before resuming sensitive work.

9. Capability advertisements are time-bounded.

10. Federation membership is policy-controlled.
```

# 58. From ROS Graph to NROS Federation Graph

ROS:

```text
Node ── Topic ── Node
```

NROS:

```text
             Federation
                  │
       ┌──────────┼──────────┐
       ↓          ↓          ↓
     Agent      Resource   Capability
       │          │          │
       └─────┬────┴────┬─────┘
             ↓         ↓
          Authority   Work
             │         │
             └────┬────┘
                  ↓
              Execution
                  ↓
               Evidence
```

The graph now represents **semantic relationships**, not merely communication links.

# 59. Architectural Consequence

The fundamental runtime primitive becomes:

```text
Relationship
```

rather than simply:

```text
Message
```

Examples:

```text
Agent ──has──> Capability

Agent ──owns──> Resource

Principal ──delegates──> Authority

Goal ──derived-from──> Intent

Plan ──addresses──> Goal

Work ──implements──> Commitment

Execution ──instantiates──> Work

Evidence ──supports──> Belief

Fault ──affects──> Execution
```

This is the beginning of an actual **NROS semantic graph**.

# 60. The Next Boundary: Persistence

At this point, NROS has:

```text
semantic objects
state machines
events
authority
federation
leases
coordination
```

But one critical question remains:

> **Where does all of this state live?**

We need to distinguish:

```text
volatile runtime state
durable state
event history
checkpoint
snapshot
cache
replicated state
persistent identity
```

This leads to:

# Part LXII — NROS State, Persistence, Event Log & Checkpoint Architecture

The next layer will establish:

```text
Event Log
     ↓
State Reducer
     ↓
Runtime State
     ↓
Checkpoint
     ↓
Recovery
```

and, critically, how NROS can **restart a failed autonomous runtime without losing the semantic continuity of its agents, commitments, authority, and work**.

# NROS — Part LXII: State, Persistence, Event Log & Checkpoint Architecture

We now reach a fundamental distinction between ROS and NROS.

ROS can treat much of its runtime graph as something that exists **while the system is running**.

NROS cannot do that if agents, commitments, authority, goals, and autonomous work are first-class concepts.

An NROS runtime must be able to answer:

> **What was true before the process crashed, what was in progress, what authority was still valid, and what must happen after restart?**

That requires a formal persistence architecture.

# 1. Runtime State Is Not Enough

A naive runtime looks like:

```text
Process
   ↓
Memory
   ↓
State
```

When the process dies:

```text
Process ✕
Memory ✕
State ✕
```

For autonomous systems this can be catastrophic.

Instead:

```text
                 ┌──────────────┐
                 │ Runtime State│
                 └──────┬───────┘
                        │
              ┌─────────┴─────────┐
              ↓                   ↓
        Event History        Checkpoints
              │                   │
              └─────────┬─────────┘
                        ↓
                  Recovery
```

# 2. Three Different Things

NROS should explicitly distinguish:

### State

What the runtime currently believes is true.

```text
State_t
```

### Event

What happened.

```text
Event_t
```

### Snapshot

A persisted representation of state at a particular point.

```text
Snapshot_t
```

These are related but not interchangeable.

# 3. Event Sourcing Model

A strong conceptual model is:

```text
Initial State
      +
Event 1
Event 2
Event 3
Event 4
      ↓
Current State
```

Formally:

```text
Sₙ = Reduce(...Reduce(Reduce(S₀,E₁),E₂)...,Eₙ)
```

This gives NROS a deterministic state reconstruction mechanism.

# 4. Why Events Matter

Suppose:

```text
Work-42 = FAILED
```

Current state alone does not tell us:

```text
why?
when?
which agent?
which authority?
which resource?
which execution?
which evidence?
```

The event history can.

# 5. Event Categories

A useful taxonomy:

```text
LIFECYCLE
    EntityCreated
    EntityRetired

DISCOVERY
    CapabilityDiscovered
    DomainJoined

AUTHORITY
    AuthorityGranted
    AuthorityRevoked

PLANNING
    GoalCreated
    PlanSelected
    DecisionMade

COMMITMENT
    CommitmentCreated
    CommitmentCancelled

EXECUTION
    WorkStarted
    WorkCompleted
    WorkFailed

RESOURCE
    ResourceReserved
    ResourceReleased

OBSERVATION
    ObservationReceived

FAULT
    FaultDetected
    IncidentOpened

RECOVERY
    RecoveryStarted
    RecoveryCompleted
```

# 6. Events Should Be Immutable

Once an event is committed:

```text
Event E42
```

its semantic contents should not change.

Corrections should be represented by new events:

```text
E42:
    incorrect observation

E43:
    observation corrected
```

This preserves history.

# 7. Event Envelope

An event should have a stable envelope around its domain payload.

Conceptually:

```rust
struct EventEnvelope {
    event_id: EventId,
    event_type: EventType,
    origin: EntityId,
    domain: DomainId,
    epoch: Epoch,
    sequence: Sequence,
    timestamp: Timestamp,
    causality: CausalContext,
    payload: EventPayload,
}
```

The exact Rust representation can evolve.

The semantic requirements are more important.

# 8. Sequence Numbers

Each event stream should have ordering information.

For example:

```text
Domain A

seq 100
seq 101
seq 102
seq 103
```

If the runtime receives:

```text
100
102
101
```

it can detect that delivery order differs from logical order.

# 9. Ordering Is Not Always Global

NROS should **not** assume one global sequence for the entire federation.

Instead:

```text
Domain A:
    1,2,3,4

Domain B:
    1,2,3
```

Each has its own local ordering.

Cross-domain causality is represented separately.

# 10. Causal Ordering

Suppose:

```text
A1
 ↓
B7
 ↓
C2
```

The system should preserve:

```text
B7 depends on A1
C2 depends on B7
```

without requiring a single global clock.

This is especially important in distributed robotics.

# 11. Logical Time

NROS can use logical timestamps for ordering:

```text
LogicalTime {
    domain
    counter
}
```

or a more sophisticated causal structure where necessary.

The architecture should avoid making wall-clock synchronization a correctness requirement.

# 12. Wall Clock vs Logical Clock

Wall clock answers:

```text
When did this happen in physical time?
```

Logical time answers:

```text
What happened before what?
```

NROS needs both.

For example:

```text
event_time = 08:42:10.123 UTC
causal_order = after E928
```

# 13. Event Store

The persistence subsystem can expose:

```text
append(event)
read(stream, position)
read_since(position)
snapshot()
restore()
```

Conceptually:

```text
┌────────────────────────────┐
│        Event Store         │
├────────────────────────────┤
│ E001                       │
│ E002                       │
│ E003                       │
│ ...                        │
│ E999                       │
└────────────────────────────┘
```

# 14. Append-Only Principle

The fundamental write operation should be:

```text
append
```

rather than:

```text
update event
delete event
```

This makes the history auditable.

# 15. State Projection

Not every component should replay every event forever.

A state projection can consume events:

```text
Event Store
     ↓
Projection
     ↓
Current State
```

Examples:

```text
ResourceProjection
WorkProjection
AgentProjection
AuthorityProjection
GoalProjection
FaultProjection
```

# 16. Multiple Projections

The same event history can generate multiple views.

```text
                Event Store
                /    |    \
               /     |     \
              ↓      ↓      ↓
          WorkView  ResourceView  AuditView
```

This is extremely useful for NROS.

# 17. Runtime State

The in-memory runtime can therefore be a projection:

```text
Event History
     ↓
State Reducer
     ↓
Runtime State
```

It is not necessarily the ultimate source of truth.

# 18. Checkpoint

Replaying millions of events after every restart is inefficient.

Therefore NROS periodically creates:

```text
Checkpoint
```

Example:

```text
Events:
E1 ... E1000000

Checkpoint:
after E1000000
```

Recovery starts from the checkpoint instead of `E1`.

# 19. Checkpoint Structure

Conceptually:

```rust
struct Checkpoint {
    checkpoint_id: CheckpointId,
    event_position: EventPosition,
    runtime_epoch: Epoch,
    state: RuntimeState,
    created_at: Timestamp,
    integrity: Digest,
}
```

The checkpoint itself must be verifiable.

# 20. Checkpoint Frequency

Checkpointing should be policy-controlled.

Possible triggers:

```text
every N events
every T seconds
after critical transitions
before shutdown
after major commitments
after authority changes
```

# 21. Critical-State Checkpoints

Some transitions deserve immediate persistence.

For example:

```text
AuthorityGranted
CommitmentAccepted
ResourceAllocated
SafetyModeEntered
```

The runtime should not rely solely on periodic snapshots for these.

# 22. Durability Levels

Not every event requires identical durability.

NROS can define levels such as:

```text
VOLATILE
BUFFERED
DURABLE
CRITICAL
```

For example:

```text
sensor telemetry
    → VOLATILE

diagnostic event
    → BUFFERED

goal commitment
    → DURABLE

safety authority transition
    → CRITICAL
```

This allows embedded deployments to control storage costs.

# 23. Memory-Constrained NROS

This is particularly important for small robots.

NROS should not assume:

```text
NVMe
large SSD
cloud connectivity
gigabytes of RAM
```

The persistence layer must support:

```text
tiny flash
local filesystem
memory-mapped storage
external storage
remote durable store
```

without changing semantic contracts.

# 24. Storage Backend Abstraction

The semantic API should remain independent of storage technology:

```text
NROS Persistence API
        │
 ┌──────┼─────────┐
 ↓      ↓         ↓
File   SQLite   Remote
       /RocksDB  Store
```

The backend is replaceable.

# 25. Transaction Boundary

A critical question:

> When is an event considered committed?

NROS needs a precise durability boundary.

For example:

```text
Event generated
      ↓
validated
      ↓
persisted
      ↓
acknowledged
      ↓
visible to state projection
```

The ordering must be defined.

# 26. Commit Before Effect

For critical operations, one possible policy is:

```text
Intent
 ↓
Decision
 ↓
Commitment persisted
 ↓
Work authorized
 ↓
Effect
```

This prevents the physical action from occurring without a durable semantic record.

# 27. But Physical Reality Can Still Diverge

Consider:

```text
Command persisted
      ↓
Motor activated
      ↓
Power failure
```

The event history might say:

```text
ExecutionStarted
```

but not:

```text
ExecutionCompleted
```

Therefore after restart:

```text
UNKNOWN
```

must be a legitimate state.

# 28. UNKNOWN Is a First-Class State

This is one of the most important NROS principles.

Never infer:

```text
no completion event
    =
operation failed
```

It may instead mean:

```text
operation status unknown
```

Thus:

```text
RUNNING
FAILED
COMPLETED
CANCELLED
UNKNOWN
```

are semantically distinct.

# 29. Recovery Verification

After restart:

```text
Execution = UNKNOWN
```

The runtime should attempt verification.

Example:

```text
Query actuator
      ↓
physical state
      ↓
Evidence
      ↓
reconstruct execution state
```

This is much safer than blindly replaying the command.

# 30. Never Blindly Replay Effects

Suppose:

```text
OpenValve
```

was issued before the crash.

On restart, replaying:

```text
OpenValve
```

could produce a dangerous duplicate effect.

Therefore event replay should reconstruct **state**, not automatically re-execute physical commands.

# 31. Replay Boundary

The architecture must distinguish:

```text
Replay Event
```

from:

```text
Execute Command
```

Replay:

```text
historical fact → state transition
```

Execution:

```text
current authorization → physical action
```

These are fundamentally different paths.

# 32. Restart Protocol

A canonical restart could be:

```text
Process starts
     ↓
Load identity
     ↓
Load latest checkpoint
     ↓
Replay subsequent events
     ↓
Validate epochs
     ↓
Validate leases
     ↓
Mark uncertain executions
     ↓
Verify physical state
     ↓
Reconcile resources
     ↓
Resume safe work
```

# 33. Recovery State Machine

```text
STARTING
   ↓
RESTORING
   ↓
REPLAYING
   ↓
RECONCILING
   ↓
VERIFYING
   ↓
SAFE
   ↓
ACTIVE
```

Failure during recovery:

```text
        ↓
   DEGRADED
        ↓
   QUARANTINED
```

# 34. Authority Recovery

Authority must be treated specially.

A restarted agent should not automatically assume:

```text
"I had authority before, therefore I still have authority."
```

Instead:

```text
Persisted Authority
       ↓
check epoch
       ↓
check lease
       ↓
check revocation
       ↓
revalidate
```

Only then can authority become active again.

# 35. Commitment Recovery

A commitment may have been:

```text
ACCEPTED
```

before the crash.

After restart:

```text
Commitment = ACCEPTED
```

but its associated Work may be:

```text
UNKNOWN
```

The runtime should reconcile them rather than assuming completion.

# 36. Resource Recovery

Likewise:

```text
Resource:
    Robot Arm

Last known:
    allocated to Work-42
```

After restart, the physical resource may actually be:

```text
idle
faulted
occupied
unknown
```

NROS must verify when necessary.

# 37. Persistent Identity

Identity must survive restart.

For example:

```text
AgentId = A42
```

must not become:

```text
AgentId = A93
```

simply because the process restarted.

This means identity persistence belongs below the runtime lifecycle.

# 38. Generation Numbers

However, restart should produce a new execution generation:

```text
Agent A42
generation 17
```

after restart:

```text
Agent A42
generation 18
```

Thus:

```text
Identity remains stable
Execution generation changes
```

This is another mechanism against stale messages.

# 39. Persistent Identity + Epoch

The combined model becomes:

```text
AgentId
    +
Generation
    +
Epoch
```

where:

```text
AgentId    = who
Generation = which runtime incarnation
Epoch      = which authority/state era
```

This is considerably stronger than a simple process ID.

# 40. Garbage Collection

Event histories can grow indefinitely.

NROS therefore needs retention policy.

Possible policies:

```text
retain forever
retain N days
retain until checkpoint
retain until incident closed
retain critical events forever
```

But deletion must not destroy required audit guarantees.

# 41. Compaction

Suppose:

```text
10 million events
```

have been reduced into:

```text
Checkpoint #500
```

Older non-critical events can potentially be compacted.

But the system should preserve enough metadata to prove:

```text
how checkpoint #500 was derived
```

when auditability requires it.

# 42. Merkle-Style Integrity

For strong integrity, checkpoints and event segments can carry cryptographic digests:

```text
E1 ──┐
E2 ──┤
E3 ──┤ → Segment Hash
E4 ──┤
E5 ──┘
```

Then:

```text
Checkpoint
    ↓
references segment digest
```

This makes silent history modification detectable.

# 43. Audit vs Operational Storage

NROS should distinguish:

```text
Operational State
```

from:

```text
Audit History
```

Operational storage can be aggressively optimized.

Audit history may require:

```text
immutability
long retention
cryptographic integrity
provenance
```

The two should not be conflated.

# 44. Evidence Retention

Not all sensor data needs to be retained forever.

For example:

```text
Camera frame:
    large

Observation:
    "object detected"
```

NROS may store:

```text
Observation + reference
```

rather than permanently storing the entire frame.

This leads naturally to:

```text
Evidence URI
```

or content-addressed evidence.

# 45. Content-Addressed Evidence

Conceptually:

```text
Evidence
   ↓
Digest
   ↓
Blob Store
```

The event stores:

```text
evidence_digest
```

rather than embedding massive binary payloads.

This keeps the event log compact.

# 46. Persistence Architecture

The complete layer now becomes:

```text
                 NROS RUNTIME
                      │
               ┌──────▼──────┐
               │ State Model │
               └──────┬──────┘
                      │
               ┌──────▼──────┐
               │   Reducer   │
               └──────┬──────┘
                      │
               ┌──────▼──────┐
               │ Event Store │
               └──────┬──────┘
                      │
          ┌───────────┼────────────┐
          ↓           ↓            ↓
      Checkpoint   Audit       Evidence
       Storage     History       Store
```

# 47. Recovery Architecture

Putting everything together:

```text
                 CRASH
                   │
                   ↓
              PROCESS EXIT
                   │
                   ↓
             NEW INCARNATION
                   │
                   ↓
            LOAD IDENTITY
                   │
                   ↓
          LOAD CHECKPOINT
                   │
                   ↓
             REPLAY EVENTS
                   │
                   ↓
          RECONSTRUCT STATE
                   │
                   ↓
        REVALIDATE AUTHORITY
                   │
                   ↓
          RECONCILE RESOURCES
                   │
                   ↓
          VERIFY UNKNOWN WORK
                   │
                   ↓
             SAFE RESUME
```

# 48. The Core Invariant

This leads to a very strong NROS invariant:

> **A restart must never silently transform uncertainty into success.**

If the runtime cannot establish what happened:

```text
UNKNOWN
```

must remain the state until evidence resolves it.

# 49. Another Core Invariant

Likewise:

> **Historical events may reconstruct semantic state, but must never implicitly re-execute external side effects.**

Therefore:

```text
Replay ≠ Re-execution
```

This distinction should be encoded into the API itself.

# 50. Proposed Rust Boundary

A conceptual separation might eventually resemble:

```rust
trait EventStore {
    fn append(&mut self, event: Event) -> Result<EventPosition>;
    fn read(&self, position: EventPosition) -> Result<Vec<Event>>;
}

trait StateReducer {
    fn apply(
        &mut self,
        event: &Event,
    ) -> Result<(), TransitionError>;
}

trait CheckpointStore {
    fn save(&mut self, checkpoint: Checkpoint) -> Result<()>;
    fn latest(&self) -> Result<Option<Checkpoint>>;
}

trait RecoveryEngine {
    fn recover(&mut self) -> Result<RecoveryState>;
}
```

These are **architectural boundaries**, not yet a claim about the exact NROS API.

# 51. NROS Persistence Principle

The entire persistence philosophy can be reduced to:

```text
EVENTS
   ↓
STATE
   ↓
CHECKPOINT
   ↓
RECOVERY
```

while physical reality is independently verified through:

```text
COMMAND
   ↓
EFFECT
   ↓
OBSERVATION
   ↓
EVIDENCE
```

These two paths converge only through validated evidence.

# 52. ROS → NROS Transformation

We can now extend the original transformation:

```text
ROS

Node
Topic
Service
Parameter
rosbag
```

becomes:

```text
NROS

Agent
Capability
Goal
Intent
Decision
Authority
Commitment
Work
Execution
Event
Evidence
Checkpoint
Federation
```

The architectural center has moved from:

```text
communication
```

to:

```text
state + agency + evidence + coordination
```

# 53. The Emerging NROS Stack

We now have enough pieces to define a preliminary stack:

```text
┌──────────────────────────────────────────┐
│             APPLICATION LAYER            │
├──────────────────────────────────────────┤
│      Agent / Mission / Robot Logic       │
├──────────────────────────────────────────┤
│             AGENCY LAYER                 │
│ Intent / Goal / Plan / Decision          │
├──────────────────────────────────────────┤
│            EXECUTION LAYER               │
│ Commitment / Work / Execution             │
├──────────────────────────────────────────┤
│           COORDINATION LAYER             │
│ Authority / Resource / Lease / Federation│
├──────────────────────────────────────────┤
│             STATE LAYER                  │
│ Event / Reducer / Projection / Checkpoint│
├──────────────────────────────────────────┤
│            TRANSPORT LAYER               │
│ IPC / Network / DDS / Zenoh / QUIC / ... │
├──────────────────────────────────────────┤
│             PLATFORM                     │
│ Linux / RTOS / Embedded / WASM / ...     │
└──────────────────────────────────────────┘
```

# 54. The Critical Next Question

We have now defined:

```text
WHAT exists
HOW it transitions
WHO can coordinate
WHERE state persists
HOW state recovers
```

But we have not yet defined the most fundamental runtime question:

> **How does NROS actually execute work?**

This takes us into the **execution substrate**.

The next layer therefore becomes:

# Part LXIII — NROS Execution Engine: Scheduler, Executors, Work Queues & Real-Time Boundaries

We will need to derive the replacement for the ROS concepts of:

```text
spin()
callback queue
executor
callback group
node lifecycle
```

but for NROS the central object will no longer be the callback.

It will be:

```text
                    WORK
                     │
          ┌──────────┼──────────┐
          ↓          ↓          ↓
      Immediate    Periodic   Event-driven
          │          │          │
          └──────────┼──────────┘
                     ↓
                  SCHEDULER
                     ↓
                 EXECUTOR
                     ↓
                EFFECTOR
                     ↓
                 EVIDENCE
```

And that is where NROS can begin addressing **deterministic execution, real-time constraints, priorities, preemption, resource budgets, deadlines, and agent-aware scheduling** rather than merely providing a message-passing runtime.

# NROS — Part LXIII: Execution Engine, Scheduler, Executors & Real-Time Boundaries

We now move from **what NROS knows** to **how NROS acts**.

The central change is:

> **ROS schedules callbacks. NROS schedules authorized work.**

A callback is an implementation mechanism.  
A `Work` item is a semantic runtime object.

# 1. ROS Executor → NROS Execution Engine

A simplified ROS model is:

```text
Message
   ↓
Callback
   ↓
Executor
   ↓
User Code
```

NROS should instead model:

```text
Event / Goal / Trigger
          ↓
        Work
          ↓
   Authorization
          ↓
     Scheduler
          ↓
      Executor
          ↓
      Effectors
          ↓
      Observation
          ↓
       Evidence
```

The scheduler therefore operates on **semantic work**, not merely callbacks.

# 2. Work Is the Fundamental Scheduling Unit

An NROS `Work` item should answer:

```text
WHAT must be done?
WHO owns it?
WHY does it exist?
WHEN should it run?
WHAT does it require?
WHAT authority permits it?
WHAT constraints apply?
WHAT happens if it fails?
```

Conceptually:

```rust
struct Work {
    id: WorkId,
    owner: AgentId,
    goal: GoalId,
    capability: CapabilityId,
    priority: Priority,
    deadline: Option<Deadline>,
    resources: ResourceRequirements,
    constraints: Constraints,
    authority: AuthorityRef,
}
```

The exact API can evolve later.

# 3. Work Is Not a Thread

This distinction is critical.

```text
Work
   ≠
Thread
```

One Work item may execute:

```text
on one thread
on multiple threads
through an async task
through an RT task
through a remote agent
```

The semantic Work object remains stable.

# 4. Work vs Execution

A Work item describes the obligation.

An Execution describes the current attempt.

```text
Work-42
   │
   ├── Execution-1 → FAILED
   ├── Execution-2 → FAILED
   └── Execution-3 → RUNNING
```

This enables retries without creating a new mission-level Work object.

# 5. Execution Attempt

Each attempt gets its own identity:

```text
ExecutionId
Attempt = 3
```

This provides precise provenance:

```text
Work-42
    ↓
Attempt #3
    ↓
Motor command
    ↓
Evidence
```

# 6. Scheduler

The scheduler determines:

> **Which eligible Work should execute next?**

It considers:

```text
priority
deadline
resource availability
authority
dependencies
agent policy
safety constraints
CPU budget
latency requirements
execution domain
```

Thus:

```text
Scheduler
```

is a semantic decision engine.

# 7. Eligibility

A Work item is not schedulable merely because it exists.

It must pass:

```text
Exists?
   ↓
Authorized?
   ↓
Dependencies satisfied?
   ↓
Resources available?
   ↓
Constraints satisfied?
   ↓
Deadline viable?
   ↓
Execution environment available?
```

Only then:

```text
READY
```

# 8. Work State Machine

A more complete lifecycle:

```text
CREATED
   ↓
VALIDATING
   ↓
AUTHORIZED
   ↓
READY
   ↓
QUEUED
   ↓
DISPATCHED
   ↓
RUNNING
   ↓
VERIFYING
   ↓
COMPLETED
```

Alternative paths:

```text
RUNNING → FAILED
RUNNING → CANCELLED
RUNNING → PAUSED
RUNNING → PREEMPTED
RUNNING → UNKNOWN
```

# 9. Why `VERIFYING` Matters

Execution completion does not necessarily mean effect completion.

For example:

```text
Command:
    MoveArm(X)

Execution:
    command returned success
```

This does not prove:

```text
arm actually reached X
```

Verification is therefore a first-class stage.

# 10. Scheduler Classes

NROS should support different scheduling domains rather than one universal scheduler.

For example:

```text
REAL_TIME
HARD_DEADLINE
SOFT_DEADLINE
INTERACTIVE
BEST_EFFORT
BACKGROUND
BATCH
```

A safety controller should not compete with telemetry logging under the same scheduling semantics.

# 11. Hard Real-Time

A hard real-time Work has a strict requirement:

```text
deadline missed
    →
system requirement violated
```

Example:

```text
motor control loop:
    period = 1 ms
```

NROS must not pretend that a general-purpose async executor provides this guarantee.

# 12. Soft Real-Time

A soft real-time task has:

```text
preferred deadline
```

Missing it reduces quality but does not necessarily constitute system failure.

Example:

```text
camera processing:
    target = 30 FPS
```

# 13. Best-Effort Work

Examples:

```text
logging
metrics
telemetry upload
cache maintenance
```

These can yield to higher-priority work.

# 14. Real-Time Boundary

This creates an explicit boundary:

```text
             NROS
              │
      ┌───────┴────────┐
      ↓                ↓
 Real-Time          General
 Executor           Executor
      │                │
 RT constraints     Flexible tasks
```

The runtime should not claim that everything is real-time simply because an RT executor exists.

# 15. Determinism

NROS should distinguish:

```text
deterministic semantics
```

from:

```text
deterministic timing
```

A state reducer may be deterministic even when execution timing is not.

This distinction is essential.

# 16. Deterministic Reducer

Given:

```text
State S
Event E
```

the reducer should produce:

```text
same S'
```

under the same semantic conditions.

This provides a strong foundation for replay and testing.

# 17. Executor

The executor translates scheduler decisions into actual execution.

```text
Scheduler
    ↓
Executor
    ↓
Runtime task
```

The executor is responsible for:

```text
dispatch
cancellation
preemption
resource binding
context setup
completion reporting
fault capture
```

# 18. Executor Is Not Scheduler

Scheduler:

```text
WHAT runs next?
```

Executor:

```text
HOW does it run?
```

Keeping these separate allows:

```text
same scheduling policy
+
different execution substrates
```

# 19. Executor Types

NROS could eventually provide:

```text
InlineExecutor
ThreadExecutor
AsyncExecutor
RealtimeExecutor
RemoteExecutor
ProcessExecutor
IsolatedExecutor
```

The runtime semantics remain consistent.

# 20. Remote Execution

A Work item could be dispatched to another domain:

```text
Local Scheduler
      ↓
Remote Executor
      ↓
Robot B
```

The Work identity remains stable.

Only its execution location changes.

# 21. Work Migration

This enables:

```text
Work-42
   ↓
Robot A unavailable
   ↓
Robot B capable
   ↓
Execution-2
```

The system does not need to reinterpret the mission itself.

# 22. Resource Binding

Before execution:

```text
Work
 ↓
Resource Requirements
 ↓
Resource Allocation
 ↓
Execution Context
```

Example:

```text
Navigate
requires:
    localization
    propulsion
    CPU
    battery
```

The scheduler binds those resources.

# 23. Resource Budgets

Work should be able to declare budgets:

```text
CPU ≤ 20%
memory ≤ 128 MB
energy ≤ 5 Wh
network ≤ 2 MB
duration ≤ 10 s
```

This allows resource-aware scheduling.

# 24. Energy-Aware Scheduling

For mobile robots, energy is a first-class scheduling constraint.

Example:

```text
Battery = 12%

Task A:
    energy cost ≈ 2%

Task B:
    energy cost ≈ 8%

Return-to-charge:
    required reserve = 5%
```

A naive priority queue is insufficient.

The scheduler must reason about feasibility.

# 25. Deadline Scheduling

Consider:

```text
Work A:
    deadline = 10:00
    duration = 3s

Work B:
    deadline = 10:01
    duration = 20s
```

Scheduling should account for:

```text
deadline
remaining execution time
resource conflicts
preemption
```

not just static priority.

# 26. Priority Inversion

Suppose:

```text
High-priority Work
       ↓
needs Resource R
       ↓
low-priority Work owns R
```

The scheduler can become blocked.

NROS should therefore support mechanisms such as:

```text
priority inheritance
resource ceilings
deadline inheritance
```

where appropriate.

# 27. Preemption

Work may need to be interrupted:

```text
Work A
  ↓
RUNNING
  ↓
Emergency Work B
  ↓
PREEMPT A
```

But preemption must be semantically defined.

Possible outcomes:

```text
PAUSED
CANCELLED
ABORTED
ROLLED_BACK
```

These are not equivalent.

# 28. Cancellation vs Abort

Cancellation:

```text
orderly request to stop
```

Abort:

```text
immediate termination due to safety/fault
```

For example:

```text
Cancel navigation
```

might safely stop at a controlled point.

Whereas:

```text
Emergency stop
```

may require immediate actuator action.

# 29. Safety Work

Emergency work should have a dedicated class:

```text
SAFETY_CRITICAL
```

It must bypass ordinary mission scheduling where policy permits.

Example:

```text
EmergencyStop
```

should not wait behind:

```text
LogTelemetry
```

# 30. But Safety Overrides Need Authority

A dangerous misconception would be:

```text
SAFETY_CRITICAL
=
unrestricted
```

Instead:

```text
Safety Work
   ↓
Safety Authority
   ↓
Policy
   ↓
Execution
```

The emergency path itself must be governed.

# 31. Callback Groups → Execution Domains

ROS callback groups primarily control callback concurrency.

NROS can generalize this into:

```text
ExecutionDomain
```

An execution domain specifies:

```text
concurrency
isolation
priority
resource limits
scheduler
executor
failure policy
```

# 32. Execution Domain Example

```text
Robot Runtime
│
├── RT Control Domain
│     └── 1 kHz control
│
├── Navigation Domain
│     └── planner
│
├── Perception Domain
│     └── vision
│
└── Background Domain
      └── telemetry/logging
```

This is more expressive than callback groups.

# 33. Isolation

Execution domains can optionally provide:

```text
thread isolation
process isolation
memory isolation
CPU quotas
network quotas
fault containment
```

A failed perception component should not necessarily kill the control subsystem.

# 34. Fault Containment

Suppose:

```text
Vision Executor
      ↓
segmentation fault
```

NROS should be able to contain the fault:

```text
Vision Domain
    ↓
FAILED
    ↓
Recovery
```

while:

```text
Control Domain
    ↓
continues safely
```

subject to system policy.

# 35. Executor Health

Executors themselves become observable:

```text
ExecutorHealth {
    queue_depth
    active_work
    deadline_misses
    utilization
    faults
}
```

The scheduler can use this information.

# 36. Backpressure

A high-rate source can overwhelm an executor.

For example:

```text
Camera
 ↓ 1000 events/s
Queue
 ↓ 100 events/s processing
```

The queue grows without bound unless policy intervenes.

NROS needs explicit backpressure semantics.

# 37. Queue Policies

Possible policies:

```text
BLOCK
DROP_OLDEST
DROP_NEWEST
COALESCE
SAMPLE
PRIORITIZE
SHED
```

Different workloads require different policies.

# 38. Event Coalescing

For state-like data:

```text
Battery:
80%
79%
78%
77%
76%
```

the runtime may not need every intermediate value.

It can coalesce to:

```text
latest Battery = 76%
```

while preserving critical events separately.

# 39. Streaming vs Work

NROS should distinguish:

```text
streaming data
```

from:

```text
semantic Work
```

A lidar stream can continuously produce observations without every packet becoming a scheduled Work item.

This prevents semantic overload.

# 40. Event-Driven Scheduling

Events can trigger Work:

```text
Event:
    obstacle_detected

        ↓

Trigger Policy

        ↓

Work:
    stop_robot
```

This creates:

```text
Event → Policy → Work
```

rather than:

```text
Event → arbitrary callback
```

# 41. Temporal Triggers

Work can also be triggered by time:

```text
every 10 ms
at deadline T
after 5 seconds
when condition becomes true
```

The scheduler should treat these uniformly as trigger conditions.

# 42. Conditional Work

Example:

```text
IF battery < 15%
THEN schedule ReturnToCharge
```

The condition should be evaluated against the runtime's state model.

Thus:

```text
State
 ↓
Policy
 ↓
Work creation
```

# 43. Agent Scheduling

NROS introduces another dimension absent from traditional callback scheduling:

> **Which agent's work should receive runtime attention?**

Suppose:

```text
Agent A:
    mission critical

Agent B:
    optimization

Agent C:
    telemetry
```

The scheduler can reason at the agent level.

# 44. Agent Budgets

An agent may have:

```text
CPU budget
memory budget
energy budget
network budget
concurrency budget
```

This prevents one autonomous agent from monopolizing the runtime.

# 45. Fairness

A fleet coordinator might have:

```text
Agent A → 40%
Agent B → 40%
Agent C → 20%
```

But safety constraints can override fairness.

Therefore NROS scheduling is:

```text
constraints first
then policy
then optimization
```

# 46. Hierarchical Scheduling

A powerful model:

```text
Global Scheduler
       ↓
Agent Scheduler
       ↓
Execution-Domain Scheduler
       ↓
Thread/Task Scheduler
```

Each level solves a different problem.

# 47. Example

```text
Fleet Scheduler
    ↓
Robot A allocated 30% compute
    ↓
Robot Scheduler
    ↓
Navigation gets 15%
Control gets 10%
Perception gets 5%
    ↓
RT Executor
```

This enables large heterogeneous systems.

# 48. Scheduler Policy Is Pluggable

NROS should not hard-code one universal scheduling algorithm.

Potential policies:

```text
FIFO
Priority
EDF
Rate Monotonic
Deadline + Priority
Fair Share
Energy Aware
Utility Maximization
Safety Dominant
Custom Policy
```

The runtime provides the substrate.

# 49. Scheduler Determinism

A deterministic scheduler should have reproducible inputs:

```text
ready Work
resource state
policy version
time
authority
```

Then its choice can be recorded:

```text
SchedulerDecision
```

This enables post-mission analysis.

# 50. Scheduling Evidence

An execution record can include:

```text
Why was Work-42 selected?

Policy:
    EDF-v3

Candidates:
    W40
    W41
    W42

Selected:
    W42

Reason:
    earliest feasible deadline
```

This makes the scheduler explainable.

# 51. Scheduler as a Policy Engine

This suggests:

```text
Scheduler
    =
Policy
    +
Runtime State
    +
Constraints
    +
Optimization
```

rather than merely:

```text
queue.pop()
```

# 52. Execution Feedback Loop

Execution should continuously feed the runtime:

```text
Scheduler
    ↓
Work
    ↓
Executor
    ↓
Execution
    ↓
Observation
    ↓
Evidence
    ↓
State Update
    ↓
Scheduler
```

This creates a closed-loop autonomous runtime.

# 53. ROS Callback Loop → NROS Agency Loop

ROS:

```text
spin()
  ↓
callback
  ↓
spin()
```

NROS:

```text
observe
   ↓
update state
   ↓
evaluate policy
   ↓
schedule work
   ↓
execute
   ↓
verify
   ↓
learn/update belief
   ↓
repeat
```

This is a fundamental conceptual evolution.

# 54. The NROS Execution Loop

A canonical runtime cycle is therefore:

```text
┌───────────────┐
│    OBSERVE    │
└───────┬───────┘
        ↓
┌───────────────┐
│ UPDATE STATE  │
└───────┬───────┘
        ↓
┌───────────────┐
│ EVALUATE      │
│ GOALS/POLICY  │
└───────┬───────┘
        ↓
┌───────────────┐
│    SCHEDULE   │
└───────┬───────┘
        ↓
┌───────────────┐
│    EXECUTE    │
└───────┬───────┘
        ↓
┌───────────────┐
│    VERIFY     │
└───────┬───────┘
        ↓
┌───────────────┐
│ RECORD EVENT  │
└───────┬───────┘
        │
        └──────────────→ OBSERVE
```

This is the **NROS execution cycle**.

# 55. The Safety Loop

For safety-critical work, we can add:

```text
PLAN
 ↓
AUTHORIZE
 ↓
EXECUTE
 ↓
VERIFY
 ↓
SAFE?
 ├── YES → COMPLETE
 └── NO  → ABORT / RECOVER
```

Verification therefore sits directly in the execution contract.

# 56. NROS vs ROS — Execution Comparison

| ROS | NROS |
|---|---|
| Callback | Work |
| Callback queue | Work queue |
| Executor | Execution engine |
| Callback group | Execution domain |
| `spin()` | Agency/execution loop |
| Parameter | State/configuration |
| Service call | Authorized operation |
| Topic message | Event/observation |
| rosbag | Event/evidence history |
| Node lifecycle | Agent/work lifecycle |
| ROS master/discovery | Federation/discovery |

The transformation is semantic rather than merely syntactic.

# 57. Critical Architectural Rule

NROS should **not** force every computation through the semantic Work system.

Otherwise:

```text
1 million sensor samples/sec
```

could become:

```text
1 million Work objects/sec
```

which would be absurd.

Instead:

```text
raw data
   ↓
transport/stream layer
   ↓
aggregation/filter
   ↓
Observation
   ↓
semantic runtime
```

Only meaningful actions enter the Work scheduler.

# 58. The Resulting Layer Boundary

We can now establish:

```text
RAW TRANSPORT
      ↓
STREAM PROCESSING
      ↓
OBSERVATION
      ↓
STATE
      ↓
AGENCY
      ↓
WORK
      ↓
SCHEDULER
      ↓
EXECUTOR
      ↓
EFFECTOR
      ↓
OBSERVATION
```

This separation prevents NROS from becoming either:

- merely another message bus, or
- an unnecessarily heavyweight workflow engine.

# 59. NROS Execution Invariants

The execution layer should enforce at least:

```text
1. Unauthorized Work cannot execute.

2. Work cannot execute without satisfying mandatory constraints.

3. Replay cannot trigger physical side effects.

4. Completion is not assumed without evidence where verification is required.

5. Preemption semantics are explicit.

6. Resource ownership is explicit.

7. Deadline misses are observable.

8. Safety-critical Work has an explicit execution class.

9. Runtime failure cannot silently convert active Work into success.

10. Execution attempts are individually identifiable.
```

# 60. Where We Are Now

The NROS architecture has now evolved through four major dimensions:

```text
                NROS
                 │
     ┌───────────┼────────────┐
     ↓           ↓            ↓
  SEMANTICS   FEDERATION   PERSISTENCE
     │           │            │
 Agent        Domain        Events
 Goal         Authority     State
 Plan         Lease         Checkpoint
 Work         Resource      Recovery
     │           │            │
     └───────────┼────────────┘
                 ↓
             EXECUTION
                 │
          Scheduler
          Executor
          Runtime
```

# 61. The Next Major Problem

We have deliberately postponed one foundational concern:

> **How does NROS communicate efficiently across processes, machines, robots, and heterogeneous hardware?**

ROS was historically built around communication primitives.

NROS cannot eliminate them.

Instead, we need to redesign the communication layer beneath the semantic model.

That takes us to:

# Part LXIV — NROS Communication Fabric: IPC, Transport, Serialization, Zero-Copy & Dataflow

The next layer will derive the relationship between:

```text
Event
Observation
Command
Query
Stream
Work
```

and transport mechanisms such as:

```text
shared memory
Unix sockets
TCP/QUIC
DDS
Zenoh
CAN
serial
UDP
embedded transports
```

while preserving the central NROS principle:

```text
SEMANTICS
   ≠
TRANSPORT
```

The goal is to make the same NROS semantic object capable of moving from:

```text
single thread
    ↓
same process
    ↓
same machine
    ↓
robot network
    ↓
fleet federation
    ↓
cloud
```

without forcing the application model to change at every boundary.

# NROS — Part LXIV: Communication Fabric

We now descend one layer below the execution engine.

The central principle is:

> **NROS semantics must remain independent of the transport used to carry them.**

A Work, Event, Observation, Command, or Query should not care whether it crosses:

```text
thread
→ process
→ machine
→ robot
→ fleet
```

## 1. ROS Communication Model

A simplified ROS communication path is:

```text
Publisher
   ↓
Topic
   ↓
DDS
   ↓
Subscriber
   ↓
Callback
```

NROS needs a richer model:

```text
Producer
   ↓
Semantic Object
   ↓
Communication Fabric
   ↓
Transport Selection
   ↓
Consumer
   ↓
State / Work / Execution
```

The semantic object is independent of the transport.

# 2. Separate Four Layers

NROS should distinguish:

```text
Semantic Layer
      ↓
Messaging Layer
      ↓
Transport Layer
      ↓
Physical Link
```

For example:

```text
Observation
   ↓
NROS Message
   ↓
QUIC
   ↓
Wi-Fi
```

or:

```text
Observation
   ↓
NROS Message
   ↓
Shared Memory
   ↓
RAM
```

# 3. Semantic Object

At the top:

```text
Event
Observation
Command
Query
Work
Evidence
```

These have meaning.

Transport should not redefine that meaning.

# 4. Message Envelope

A common envelope can carry metadata:

```rust
struct MessageEnvelope {
    id: MessageId,
    kind: MessageKind,
    source: EntityId,
    destination: Destination,
    epoch: Epoch,
    timestamp: Timestamp,
    correlation: Option<CorrelationId>,
    causality: CausalContext,
    priority: Priority,
    payload: Payload,
}
```

Again, this is an architectural model rather than a frozen API.

# 5. Message Types

NROS should not reduce everything to one generic "message".

At minimum:

```text
EVENT
OBSERVATION
COMMAND
QUERY
RESPONSE
WORK
CANCEL
CONTROL
HEARTBEAT
```

Their delivery semantics differ.

# 6. Event

An Event states:

> Something happened.

Example:

```text
ObstacleDetected
```

Events are generally historical facts.

# 7. Command

A Command states:

> Perform an operation.

Example:

```text
StopMotor
```

Commands therefore have side effects.

They require stronger authorization semantics.

# 8. Query

A Query states:

> Tell me something.

Example:

```text
GetBatteryState
```

Queries normally have no physical side effect.

# 9. Response

A Response corresponds to a Query or Command interaction:

```text
Request
   ↓
Response
```

Correlation is explicit:

```text
request_id = Q42
response.request_id = Q42
```

# 10. Observation

An Observation is different from an arbitrary Event.

It represents:

```text
agent/system perception
```

For example:

```text
CameraObservation
LocalizationObservation
BatteryObservation
TemperatureObservation
```

It may carry confidence and provenance.

# 11. Confidence

An observation can include:

```text
confidence = 0.93
```

but confidence must not be confused with truth.

The runtime should preserve:

```text
observed value
confidence
source
timestamp
method
```

# 12. Provenance

Every meaningful observation should ideally answer:

```text
Who produced it?
What sensor/model produced it?
When?
Under which execution?
Using which configuration?
```

This becomes:

```text
Provenance
```

and connects directly to the evidence system.

# 13. Data Plane vs Control Plane

NROS should explicitly separate:

```text
CONTROL PLANE
```

from:

```text
DATA PLANE
```

Control plane:

```text
authority
discovery
work
commands
leases
lifecycle
```

Data plane:

```text
images
point clouds
audio
lidar
telemetry
large sensor streams
```

# 14. Why This Matters

Consider a camera:

```text
1920 × 1080
30 FPS
```

Trying to treat every frame as a durable semantic event would destroy the runtime.

Instead:

```text
Camera
  ↓
Data Plane
  ↓
Perception
  ↓
Observation
  ↓
NROS State
```

Only the meaningful result crosses into the semantic layer.

# 15. Communication Classes

NROS can classify communication into:

```text
CONTROL
EVENT
REQUEST/RESPONSE
STREAM
BULK DATA
STATE REPLICATION
```

Each class can select different transport behavior.

# 16. Transport Independence

For example:

| Semantic traffic | Possible transport |
|---|---|
| control | Unix socket / QUIC |
| local event | in-process |
| large image | shared memory |
| robot telemetry | UDP/QUIC |
| industrial bus | CAN |
| fleet coordination | QUIC/Zenoh/DDS |
| persistent event | local durable store |

The application should not have to rewrite its semantic logic for each case.

# 17. Transport Adapter

Conceptually:

```rust
trait Transport {
    fn send(&self, message: Message) -> Result<()>;
    fn receive(&self) -> Result<Message>;
}
```

Then:

```text
Transport
   ├── InProcess
   ├── SharedMemory
   ├── Unix
   ├── TCP
   ├── QUIC
   ├── UDP
   ├── CAN
   └── Custom
```

The actual trait will need more precise async/concurrency semantics later.

# 18. Transport Selection

NROS should not require developers to manually select transports everywhere.

Instead:

```text
Semantic Requirement
        ↓
Transport Policy
        ↓
Best Available Transport
```

Example:

```text
Payload = 5 MB
same host
low latency
zero-copy preferred
```

→ Shared memory.

# 19. Small Control Message

Example:

```text
EmergencyStop
```

Requirements:

```text
tiny
low latency
high reliability
high priority
```

A completely different transport policy may be selected.

# 20. Large Data

Example:

```text
PointCloud
```

Requirements:

```text
large payload
high throughput
possibly lossy
short retention
```

The data plane may use shared memory or streaming transport.

# 21. Zero-Copy

Large robotics data makes copying expensive.

Traditional:

```text
Sensor
 ↓ copy
Middleware
 ↓ copy
Application
 ↓ copy
Algorithm
```

NROS should support:

```text
Sensor
   ↓
Shared Buffer
   ↓
Consumers
```

with ownership controlled explicitly.

# 22. Buffer Ownership

Zero-copy creates a new problem:

> Who owns the memory?

Possible model:

```text
Producer
   ↓
Lease Buffer
   ↓
Consumers
   ↓
Release
```

The buffer lifetime must be explicit.

# 23. Reference Counting

A shared payload may use:

```text
Arc<Buffer>
```

or an equivalent ownership mechanism.

But NROS should not assume heap-based reference counting is suitable for every real-time path.

# 24. Real-Time Zero-Copy

For hard real-time domains:

```text
allocation
deallocation
locking
page faults
```

may be unacceptable.

Therefore NROS needs the concept of:

```text
preallocated buffer pools
```

Example:

```text
Pool:
  Buffer 0
  Buffer 1
  Buffer 2
  ...
```

# 25. Memory Pools

A real-time publisher can obtain:

```text
buffer = pool.acquire()
```

fill it:

```text
write(buffer)
```

publish:

```text
publish(buffer)
```

and later:

```text
pool.release(buffer)
```

without runtime heap allocation.

# 26. Backpressure + Zero-Copy

The two concepts interact.

Suppose all buffers are occupied:

```text
Pool:
[busy]
[busy]
[busy]
[busy]
```

A producer cannot acquire another buffer.

NROS must then apply a policy:

```text
BLOCK
DROP
OVERWRITE
DEGRADE
SHED
```

This must be explicit.

# 27. Reliability Classes

Not every message requires reliable delivery.

NROS could define:

```text
BEST_EFFORT
RELIABLE
DURABLE
CRITICAL
```

For example:

```text
camera frame:
    BEST_EFFORT

configuration:
    RELIABLE

authority revocation:
    CRITICAL
```

# 28. Ordering Classes

Likewise:

```text
UNORDERED
PER_STREAM
CAUSAL
TOTAL
```

should be separate concepts.

Most traffic does not require global total ordering.

# 29. Delivery Semantics

NROS should explicitly define:

```text
at-most-once
at-least-once
effectively-once
```

But **"exactly once"** requires special care.

# 30. Exactly-Once Is Not a Magic Transport Property

Suppose:

```text
Command
 ↓
Robot
 ↓
Motor
```

The network cannot know whether the physical effect occurred if the acknowledgement is lost.

Therefore:

```text
exactly-once delivery
```

does not necessarily mean:

```text
exactly-once physical effect
```

This is why NROS needs execution IDs and idempotency.

# 31. Idempotency

A command can carry:

```text
ExecutionId = E42
```

If the receiver sees:

```text
E42
E42
E42
```

it can recognize duplicates.

This transforms unreliable transport into safer command semantics.

# 32. Command Deduplication

Conceptually:

```text
Command(E42)
     ↓
Execution Registry
     ↓
Already seen?
 ┌───┴────┐
YES      NO
 ↓        ↓
return   execute
result
```

This is particularly important after retries and reconnects.

# 33. Correlation

For asynchronous operations:

```text
Command C42
     ↓
accepted
     ↓
Execution E91
     ↓
Observation
     ↓
Completion Event
```

All should be correlated.

```text
correlation_id
```

connects the chain.

# 34. Causality

Correlation tells us:

```text
these messages belong together
```

Causality tells us:

```text
this happened because of that
```

NROS needs both.

# 35. Communication Graph

Instead of only a topic graph:

```text
A → Topic → B
```

NROS can represent:

```text
Agent A
   ↓
Observation O1
   ↓
Policy P7
   ↓
Work W42
   ↓
Command C8
   ↓
Execution E3
   ↓
Evidence V9
```

This is a **causal execution graph**.

# 36. Distributed Causality

Across robots:

```text
Robot A
  O1
   ↓
  W42
   ↓
  C8
   ↓
──────────── network ────────────
   ↓
Robot B
   E7
   ↓
  O9
```

The runtime should preserve the causal chain even though physical execution is distributed.

# 37. Discovery

Communication requires discovery.

ROS traditionally exposes nodes/topics/services.

NROS discovery needs to expose richer objects:

```text
Agent
Capability
Endpoint
Execution Domain
Resource
Authority
Transport
```

# 38. Capability Discovery

Example:

```text
Agent B advertises:

Capability:
    navigation

Requirements:
    localization
    map

Constraints:
    max payload = ...
```

Another agent can discover this without knowing implementation details.

# 39. Endpoint Discovery

A capability can expose:

```text
input
output
command
query
event
stream
```

The communication fabric maps those endpoints to actual transports.

# 40. Dynamic Transport Selection

Suppose:

```text
Robot A ↔ Robot B
```

Initially:

```text
Wi-Fi
```

Later:

```text
Ethernet
```

The semantic endpoint remains:

```text
Agent B / navigation
```

while the transport changes underneath.

# 41. Transport Failure

If a connection fails:

```text
Transport DOWN
```

NROS should distinguish:

```text
endpoint unavailable
```

from:

```text
agent unavailable
```

These are not necessarily the same.

# 42. Reconnection

A transport can recover:

```text
DOWN
 ↓
RECONNECTING
 ↓
CONNECTED
 ↓
RESYNCING
 ↓
READY
```

The semantic state may continue to exist throughout.

# 43. Resynchronization

After reconnection:

```text
Robot A
     ↓
"What events did I miss?"
     ↓
Robot B
     ↓
event/state synchronization
```

This is another reason event logs and checkpoints are valuable.

# 44. State Replication

NROS may replicate selected state:

```text
Agent A
   ↓
State subset
   ↓
Agent B
```

But not necessarily the entire runtime state.

Replication should be policy-driven.

# 45. State Ownership

Every replicated state item needs an owner.

Example:

```text
BatteryState
owner = Robot-A
```

Robot B can observe it but should not silently mutate it.

This prevents distributed state conflicts.

# 46. Conflict Resolution

If multiple authorities can modify shared state:

```text
A says X
B says Y
```

NROS needs explicit conflict semantics:

```text
single owner
priority
lease
version
consensus
merge
manual resolution
```

The runtime should never silently pick one.

# 47. Versioned State

State can carry:

```text
version = 1042
```

An update can specify:

```text
expected_version = 1042
```

and fail if the state has already advanced.

This gives NROS optimistic concurrency control.

# 48. Transport Security

Communication security belongs below semantic authorization but must support it.

We need:

```text
authentication
confidentiality
integrity
replay protection
identity binding
```

# 49. Identity Binding

A secure transport should establish:

```text
cryptographic identity
        ↓
NROS AgentId
```

The runtime should not trust a claimed `AgentId` merely because it appears in a message.

# 50. Replay Protection

An attacker or faulty network must not be able to replay:

```text
AuthorityGranted
```

or:

```text
OpenDoor
```

from an old session.

Therefore messages need:

```text
epoch
nonce/sequence
freshness
```

appropriate to the protocol.

# 51. Authorization Remains Semantic

Transport authentication answers:

> Who sent this?

NROS authorization answers:

> Is this agent allowed to do this?

These must remain separate.

```text
Authentication
      ↓
Identity
      ↓
Authorization
      ↓
Execution
```

# 52. Communication and Persistence

Critical messages can bridge the two:

```text
Command
   ↓
Persist intent
   ↓
Transmit
   ↓
Execute
   ↓
Persist result
```

This provides stronger recovery semantics.

# 53. But Not Everything Should Be Persisted

For example:

```text
camera frame
```

does not necessarily need:

```text
durable event log
```

while:

```text
authority revoked
```

probably does.

The semantic class determines persistence policy.

# 54. Communication Policy Matrix

A useful architecture artifact is:

| Message | Reliability | Ordering | Persistence | Priority |
|---|---|---|---|---|
| Sensor stream | best effort | stream | no | normal |
| Observation | reliable | causal | optional | normal |
| Query | reliable | request/response | no | normal |
| Command | reliable | causal | often | high |
| Authority revoke | critical | ordered | yes | critical |
| Emergency stop | critical | immediate | yes | critical |
| Telemetry | best effort | unordered | optional | low |

This matrix should eventually become part of the NROS specification.

# 55. Communication Fabric Architecture

The emerging structure is:

```text
                 NROS SEMANTICS
                       │
        ┌──────────────┼──────────────┐
        ↓              ↓              ↓
      Events       Commands       Streams
        │              │              │
        └──────────────┼──────────────┘
                       ↓
              Message Envelope
                       ↓
              Communication API
                       ↓
              Transport Selector
                       ↓
       ┌───────────────┼───────────────┐
       ↓               ↓               ↓
   In-Process      Shared Memory     Network
       │               │               │
       ↓               ↓               ↓
     Memory          Buffers       QUIC/UDP/etc.
```

# 56. NROS Communication Principle

The strongest abstraction is therefore:

> **NROS defines semantic communication contracts; transports provide mechanisms for carrying those contracts.**

This prevents NROS from becoming permanently coupled to DDS, Zenoh, ROS 2 middleware, or any single networking technology.

# 57. ROS → NROS Evolution

The transformation is becoming clearer:

```text
ROS:
Topic
   ↓
Message
   ↓
DDS
   ↓
Callback
```

NROS:

```text
Semantic Object
      ↓
Message Contract
      ↓
Communication Policy
      ↓
Transport Selection
      ↓
Execution / State
      ↓
Evidence
```

# 58. The Complete NROS Runtime Loop

We can now connect the layers developed so far:

```text
                 ┌──────────────┐
                 │   OBSERVE    │
                 └──────┬───────┘
                        ↓
                 ┌──────────────┐
                 │     STATE    │
                 └──────┬───────┘
                        ↓
                 ┌──────────────┐
                 │   AGENCY     │
                 │ Goal/Plan    │
                 └──────┬───────┘
                        ↓
                 ┌──────────────┐
                 │    WORK      │
                 └──────┬───────┘
                        ↓
                 ┌──────────────┐
                 │  SCHEDULER   │
                 └──────┬───────┘
                        ↓
                 ┌──────────────┐
                 │   EXECUTOR   │
                 └──────┬───────┘
                        ↓
                 ┌──────────────┐
                 │   COMMAND    │
                 └──────┬───────┘
                        ↓
                 ┌──────────────┐
                 │   TRANSPORT  │
                 └──────┬───────┘
                        ↓
                 ┌──────────────┐
                 │   EFFECTOR   │
                 └──────┬───────┘
                        ↓
                 ┌──────────────┐
                 │  VERIFICATION│
                 └──────┬───────┘
                        ↓
                 ┌──────────────┐
                 │   EVIDENCE   │
                 └──────┬───────┘
                        ↓
                 ┌──────────────┐
                 │     EVENT    │
                 └──────┬───────┘
                        │
                        └──────────→ STATE
```

This is increasingly the defining architecture of NROS.

# 59. The Next Layer: Time

One major dimension is still missing.

All of the following depend on time:

```text
deadlines
periods
leases
timeouts
event ordering
sensor timestamps
recovery
scheduling
safety
distributed coordination
```

ROS largely exposes time as a utility.

NROS needs to make **time semantics** a foundational runtime abstraction.

# Part LXV — NROS Time Model

The next step will therefore derive:

```text
Physical Time
Logical Time
Monotonic Time
Simulation Time
Mission Time
Distributed Time
Epoch Time
Deadlines
Leases
Timers
Temporal Constraints
```

and answer a critical question:

> **How can a distributed autonomous runtime reason correctly about time when clocks disagree, networks delay messages, processes restart, and simulation time can move independently of physical time?**

That is the foundation required before we can safely define **leases, deadlines, temporal authority, deterministic replay, and distributed scheduling**.

# NROS — Part LXV: Time Model

Time must become a **first-class runtime primitive** in NROS.

The key principle:

> **NROS must never assume that one notion of time is sufficient for every semantic operation.**

A robot has physical time.  
A scheduler needs monotonic time.  
A distributed system needs logical ordering.  
A simulator may control time entirely.  
A mission may have its own temporal frame.

These must not be conflated.

# 1. The Time Problem

Consider:

```text
Robot A
    ↓
timestamp = 10:00:01.100

Network
    ↓ 80 ms

Robot B
    ↓
receives at 10:00:01.180
```

Which time is relevant?

```text
A's observation time?
B's receive time?
Network transit time?
Mission time?
```

The answer depends on the operation.

Therefore NROS needs explicit time domains.

# 2. Time Domains

At minimum:

```text
PHYSICAL_TIME
MONOTONIC_TIME
LOGICAL_TIME
SIMULATION_TIME
MISSION_TIME
```

Potentially:

```text
NETWORK_TIME
EPOCH_TIME
TRACE_TIME
```

Each has a distinct purpose.

# 3. Physical / Wall Time

Wall-clock time represents a human/calendar reference:

```text
2026-08-21T06:30:00Z
```

Useful for:

```text
logs
auditing
human interfaces
scheduled missions
certificates
cross-system records
```

But it should **not** normally control low-level timing.

# 4. Monotonic Time

A monotonic clock should only move forward:

```text
100
101
102
103
...
```

It is appropriate for:

```text
timeouts
durations
deadlines
scheduling
watchdogs
latency measurement
```

If wall time changes:

```text
10:00
→
09:55
```

monotonic time remains:

```text
1000
→
1001
```

# 5. Logical Time

Logical time represents ordering rather than physical duration.

For example:

```text
Event A
   ↓
Event B
```

can establish:

```text
A < B
```

even if their physical timestamps are uncertain.

This is essential for distributed causal reasoning.

# 6. Lamport-Style Ordering

Conceptually:

```text
A sends E1
    ↓
B receives E1
    ↓
B emits E2
```

Then:

```text
E1 → E2
```

regardless of imperfect wall clocks.

NROS should preserve this causal relationship.

# 7. Vector / Causal Time

For more sophisticated distributed systems, NROS may eventually need causal metadata such as:

```text
Agent A: 10
Agent B: 7
Agent C: 3
```

This can distinguish:

```text
causally ordered
```

from:

```text
concurrent
```

events.

But NROS should avoid requiring vector clocks everywhere.

They can be an optional advanced mechanism.

# 8. Simulation Time

Simulation introduces a radically different clock:

```text
real time:
10:00:00 → 10:00:01

simulation:
100.0 → 101.0
```

or:

```text
simulation:
100.0 → 100.1 → 100.2
```

while real execution may take:

```text
2 seconds
```

to simulate:

```text
0.2 seconds
```

# 9. Simulation Must Not Leak Into Production Semantics

A component should explicitly declare:

```text
time source = simulation
```

rather than silently replacing the system clock.

Otherwise a production safety mechanism could accidentally depend on simulated time.

# 10. Mission Time

A mission may define:

```text
T0 = mission start
```

Then:

```text
MissionTime = MonotonicTime - T0
```

Example:

```text
T+0s    launch
T+15s   navigation
T+40s   inspection
T+90s   return
```

Mission time is extremely useful for autonomous plans.

# 11. Time Identity

A timestamp should therefore carry its clock domain.

Instead of:

```rust
timestamp: u64
```

conceptually:

```rust
Time {
    domain: TimeDomain,
    value: TimeValue,
}
```

This prevents accidental comparisons between unrelated clocks.

# 12. Invalid Comparison

This should be considered invalid:

```text
SimulationTime(100)
<
WallTime(2026-08-21)
```

Likewise:

```text
RobotA.Monotonic(500)
<
RobotB.Monotonic(501)
```

does not imply that A's event happened before B's event.

Monotonic clocks are local unless synchronized semantics are explicitly established.

# 13. Timestamp Structure

A useful event timestamp could contain:

```text
timestamp:
    domain
    value
    uncertainty
    source
```

For distributed observations:

```text
ObservationTime
ReceiveTime
```

should be distinct.

# 14. Event Time vs Processing Time

This distinction is crucial.

Suppose:

```text
sensor event:
    t = 100.0

processed:
    t = 100.15
```

Then:

```text
event_time = 100.0
processing_time = 100.15
```

If the system records only processing time, latency and causality become difficult to reconstruct.

# 15. Transport Time

The communication layer can additionally observe:

```text
send_time
receive_time
```

giving:

```text
event
 ↓
send
 ↓
network
 ↓
receive
 ↓
process
```

This enables latency diagnostics.

# 16. Clock Uncertainty

Distributed timestamps are never perfectly exact.

A clock could report:

```text
T = 100.000 ± 2 ms
```

NROS should have a way to represent uncertainty when it matters.

# 17. Synchronization

Robots may synchronize through:

```text
NTP
PTP
GNSS
hardware clocks
custom synchronization
```

But synchronization quality should be observable.

For example:

```text
clock offset = +320 µs
uncertainty = ±50 µs
```

# 18. Temporal Trust

A timestamp from another agent should not automatically be trusted.

NROS can distinguish:

```text
trusted synchronized time
estimated time
unsynchronized time
unknown time
```

This matters for safety and distributed coordination.

# 19. Deadline

A deadline expresses:

> Work must complete by this temporal boundary.

For example:

```text
deadline = MissionTime 30s
```

The scheduler uses this to determine feasibility.

# 20. Timeout

A timeout expresses:

> Stop waiting after this duration.

Example:

```text
timeout = 500 ms
```

A timeout is a duration-based constraint, not necessarily an absolute timestamp.

# 21. Deadline vs Timeout

They are related but different.

```text
Deadline:
    absolute temporal boundary

Timeout:
    relative waiting limit
```

Example:

```text
Work deadline = T+30s
Network timeout = 500ms
```

Both can apply simultaneously.

# 22. Period

A periodic Work may specify:

```text
period = 10 ms
```

This does not necessarily mean:

```text

```

It should also specify:

```text
jitter tolerance
deadline
miss policy
```

# 23. Periodic Execution

For control:

```text
release
 ↓
execute
 ↓
deadline
 ↓
release
 ↓
execute
```

NROS can model this explicitly.

# 24. Jitter

Suppose a task should execute at:

```text
10.000 ms
20.000 ms
30.000 ms
```

Actual execution:

```text
10.2
20.1
30.7
```

The difference is jitter.

For real-time Work, jitter becomes a first-class metric.

# 25. Temporal Constraints

A Work item can have:

```text
release_time
deadline
period
jitter_bound
execution_budget
timeout
```

This gives the scheduler enough information to reason about temporal feasibility.

# 26. Temporal Feasibility

Before dispatching:

```text
Current Time
      ↓
Remaining Budget
      ↓
Deadline
      ↓
Resources
      ↓
Feasible?
```

If not:

```text
DO NOT EXECUTE
```

or apply an explicitly defined degradation policy.

# 27. Temporal Authority

Now time interacts with authority.

Suppose an agent receives:

```text
Authorization:
    operate actuator
```

valid only until:

```text
T + 30 seconds
```

This is a **lease**.

# 28. Lease

A lease means:

> Authority remains valid only during a defined temporal interval.

Conceptually:

```text
Lease {
    authority
    holder
    issued_at
    expires_at
}
```

# 29. Lease Expiration

When:

```text
now >= expires_at
```

the authority becomes invalid.

This must be enforced by the runtime, not merely documented.

# 30. Why Leases Matter

Distributed systems fail.

An agent may disappear:

```text
Agent A
   ↓
network failure
```

Without leases:

```text
   ↓
could remain active indefinitely
```

With leases:

```text
   ↓
expires automatically
```

This provides a natural failure boundary.

# 31. Lease Renewal

An active agent can renew:

```text
Lease
 ↓
Renew
 ↓
Lease'
```

But renewal itself must be authorized.

An expired lease cannot simply renew itself without policy permitting that behavior.

# 32. Clock Failure and Leases

This is dangerous:

```text
Agent A clock
    ↓
incorrect
```

Therefore distributed leases require careful clock semantics.

Possible approaches:

```text
monotonic local lease
trusted synchronized clocks
central lease authority
epoch-based expiration
```

The architecture must choose explicitly.

# 33. Temporal Epochs

NROS can use epochs to separate runtime eras:

```text
Epoch 1
   ↓ restart
Epoch 2
```

An execution from Epoch 1 must not accidentally be accepted as a fresh execution in Epoch 2.

This is especially useful for replay protection.

# 34. Restart Semantics

Suppose:

```text
Work-42
Execution-7
```

was running when the process crashed.

After restart:

```text
Should it resume?
Retry?
Abort?
Mark UNKNOWN?
```

The answer depends on:

```text
idempotency
checkpoint
authority
lease
physical effect
```

# 35. Temporal Checkpointing

A checkpoint can record:

```text
Work
Execution
State
Temporal position
Authority
Resources
```

After restart:

```text
checkpoint
   ↓
reconstruct
   ↓
validate temporal assumptions
   ↓
resume or recover
```

# 36. Deterministic Replay

Replay requires more than event ordering.

It needs:

```text
event sequence
logical time
relevant physical timestamps
scheduler decisions
random seeds
configuration
external inputs
```

Otherwise replay may diverge.

# 37. Replay Clock

During replay:

```text
Real Clock
    ↓
IGNORED / ABSTRACTED

Replay Clock
    ↓
controls runtime
```

The system can therefore reproduce temporal behavior deterministically.

# 38. Replay Must Not Execute Effects

This creates an important invariant:

```text
REPLAY
  ↓
simulate execution
  ↓
record hypothetical effects
```

not:

```text
REPLAY
  ↓
real motor
```

Side-effecting transports must be disabled or replaced with simulators.

# 39. Temporal Ordering

NROS should distinguish:

```text
physical order
causal order
logical order
arrival order
processing order
```

They are often different.

Example:

```text
Event A occurred first
Event B arrived first
```

The communication fabric must preserve enough metadata to explain this.

# 40. Out-of-Order Events

Distributed systems naturally produce:

```text
E1
E3
E2
```

NROS should not blindly interpret arrival order as causal order.

Possible strategies:

```text
buffering
watermarks
causal metadata
sequence numbers
application-specific ordering
```

# 41. Watermarks

A stream processor may declare:

```text
"I believe no event earlier than T remains outstanding."
```

This is a watermark.

It enables temporal aggregation without requiring perfect global clocks.

# 42. Temporal Windows

Observations can be grouped:

```text
[T, T+1s)
```

For example:

```text
      ↓
1-second window
      ↓
aggregate
```

This belongs primarily to the data/streaming layer but depends on the NROS time model.

# 43. Time and Safety

A safety rule might be:

```text
If heartbeat absent for > 100 ms:
    transition actuator to safe state
```

This is not simply networking.

It combines:

```text
+
monotonic time
+
authority
+
safety policy
+
execution
```

Therefore time must be runtime-native.

# 44. Time and Scheduling

The scheduler now becomes:

```text
Scheduler
   │
   ├── Priority
   ├── Resources
   ├── Authority
   ├── Constraints
   └── Time
```

Time is no longer an external utility.

# 45. Time-Aware Work

A complete conceptual Work object now looks more like:

```rust
struct Work {
    id: WorkId,
    owner: AgentId,
    goal: GoalId,

    priority: Priority,

    timing: TimingConstraints,

    resources: ResourceRequirements,

    authority: AuthorityRef,

    retry_policy: RetryPolicy,

    execution_policy: ExecutionPolicy,
}
```

# 46. Timing Constraints

Conceptually:

```rust
struct TimingConstraints {
    release: Option<Time>,
    deadline: Option<Time>,
    timeout: Option<Duration>,
    period: Option<Duration>,
    jitter: Option<Duration>,
}
```

The final API may differ, but the semantic distinction should remain.

# 47. Temporal State Machine

A Work lifecycle can now include temporal transitions:

```text
CREATED
   ↓
AUTHORIZED
   ↓
WAITING_FOR_RELEASE
   ↓
READY
   ↓
RUNNING
   ↓
VERIFYING
   ↓
COMPLETED
```

Temporal failure:

```text
deadline missed
     ↓
DEADLINE_MISSED
```

# 48. Deadline Miss Policy

A deadline miss must not have one universal behavior.

Possible policies:

```text
IGNORE
WARN
DROP
RETRY
DEGRADE
ABORT
ESCALATE
```

The Work's execution policy determines which applies.

# 49. Temporal Degradation

Suppose:

```text
Perception target = 30 FPS
```

If CPU pressure increases:

```text
30 FPS
 ↓
20 FPS
 ↓
10 FPS
```

A policy may permit graceful degradation rather than total failure.

# 50. Temporal Contracts

We can now define a powerful concept:

> **A temporal contract specifies the timing guarantees and acceptable deviations of an operation.**

For example:

```text
ControlLoop:
    period = 1 ms
    deadline = 1 ms
    jitter ≤ 50 µs
    execution budget = 200 µs
```

This is far more expressive than a timer callback.

# 51. Time Model Architecture

The resulting time subsystem becomes:

```text
                 NROS TIME
                     │
       ┌─────────────┼──────────────┐
       ↓             ↓              ↓
 Physical        Monotonic       Logical
       │             │              │
       ↓             ↓              ↓
 Calendar       Scheduling       Causality
       │             │              │
       └─────────────┼──────────────┘
                     ↓
               Mission Time
                     │
                     ↓
              Simulation Time
```

# 52. Unified Runtime

We now have:

```text
SEMANTICS
   ↓
STATE
   ↓
AGENCY
   ↓
WORK
   ↓
SCHEDULER
   ↓
EXECUTOR
   ↓
COMMUNICATION
   ↓
TIME
   ↓
EVIDENCE
```

But time is actually **cross-cutting** rather than simply another layer.

It touches nearly every subsystem:

```text
              TIME
        ┌───────┼────────┐
        ↓       ↓        ↓
   Scheduler  Leases  Events
        ↓       ↓        ↓
   Execution Authority Replay
```

# 53. Architectural Invariants

The time model should establish these invariants:

```text
1. Time domains are explicit.

2. Wall time is never silently used for duration measurement.

3. Local monotonic time is not assumed globally comparable.

4. Event time and receive time remain distinguishable.

5. Causality is not inferred solely from arrival order.

6. Simulation time cannot silently control production safety logic.

7. Lease expiration is enforced, not advisory.

8. Deadline misses are observable.

9. Replay uses an explicit replay clock.

10. Replay cannot produce uncontrolled physical side effects.
```

# 54. Major Consequence

At this point NROS is no longer simply:

```text
"ROS with agents"
```

The architecture is becoming:

```text
                 NROS
                  │
       ┌──────────┴──────────┐
       │                     │
   Semantic Runtime      Infrastructure
       │                     │
 Agent / Goal / Work     Time
 Plan / Policy           Transport
 Evidence                Storage
       │                     │
       └──────────┬──────────┘
                  ↓
             Execution
                  ↓
             Real World
```

The semantic runtime and infrastructure are deliberately separated.

# 55. Next Critical Layer: State

We have discussed observations and events, but not yet established the most important distinction:

```text
EVENT
    ≠
STATE
```

An event says:

```text
"Something happened."
```

State says:

```text
"What is believed to be true now?"
```

This distinction becomes fundamental once NROS agents reason autonomously.

# Part LXVI — NROS State Model

The next stage will derive:

```text
Event
Observation
Fact
Belief
State
World Model
Agent State
Shared State
Derived State
State Ownership
Versioning
Consistency
Snapshots
Checkpoints
Recovery
```

and establish the pipeline:

```text
EVENT
  ↓
OBSERVATION
  ↓
INTERPRETATION
  ↓
FACT / BELIEF
  ↓
STATE
  ↓
WORLD MODEL
  ↓
PLANNING
  ↓
WORK
```

That is the layer where NROS begins to acquire a genuine **persistent world model**, rather than merely processing messages and scheduling tasks.

# NROS — Part LXVI: State Model

We now reach one of the most important architectural boundaries:

> **NROS must distinguish what happened, what was observed, what is believed, and what the runtime currently considers state.**

Without this distinction, an autonomous runtime quickly becomes a collection of callbacks manipulating mutable variables.

# 1. Event ≠ State

An Event is historical:

```text
ObstacleDetected
```

State is current:

```text
obstacle_present = true
```

The event may disappear from active memory while the state remains.

```text
EVENT
  ↓
STATE TRANSITION
  ↓
CURRENT STATE
```

# 2. Observation ≠ Fact

Suppose a camera reports:

```text
ObjectDetected
confidence = 0.81
```

That is an observation.

It does not automatically mean:

```text
object definitely exists
```

Therefore:

```text
Observation
     ↓
Interpretation
     ↓
Belief / Fact
```

must remain explicit.

# 3. Four Levels of Knowledge

NROS can model:

```text
1. Event
2. Observation
3. Belief
4. State
```

Example:

```text
Event:
    CameraFrameReceived

Observation:
    object at (10, 20)

Belief:
    object probably exists

State:
    tracked_object_42 = active
```

Each level has different confidence and provenance semantics.

# 4. Fact vs Belief

A **Fact** is treated as established within a defined authority/context.

A **Belief** is an interpretation with uncertainty.

For example:

```text
Fact:
    battery_voltage = 11.7V
```

versus:

```text
Belief:
    battery will reach critical level in 5 minutes
```

The second is predictive.

# 5. State Is Contextual

There is rarely one universal state.

Consider:

```text
Robot A
```

It may simultaneously have:

```text
Physical State
Operational State
Mission State
Communication State
Safety State
Energy State
Navigation State
```

NROS should therefore model state as structured domains rather than one giant mutable object.

# 6. State Domains

Conceptually:

```text
RobotState
│
├── Identity
├── Lifecycle
├── Safety
├── Energy
├── Localization
├── Navigation
├── Actuation
├── Communication
└── Mission
```

Each domain can have independent ownership and update rules.

# 7. State Ownership

Every important state value should answer:

> Who is authoritative for this value?

Example:

```text
BatteryVoltage
owner = BatteryController
```

Another agent may observe it, cache it, or derive values from it.

It should not silently become the authority.

# 8. Local vs Shared State

NROS should distinguish:

```text
LOCAL STATE
```

from:

```text
SHARED STATE
```

Example:

```text
Local:
    scheduler queue

Shared:
    robot position
```

Local implementation state should not automatically become distributed state.

# 9. State Visibility

Possible visibility classes:

```text
PRIVATE
DOMAIN
AGENT
SYSTEM
FLEET
PUBLIC
```

This also integrates naturally with authorization.

# 10. State Lifecycle

A state value can move through:

```text
UNKNOWN
   ↓
OBSERVED
   ↓
VALIDATED
   ↓
CURRENT
   ↓
STALE
   ↓
EXPIRED
   ↓
UNKNOWN
```

This is more realistic than treating every variable as permanently valid.

# 11. Staleness

Suppose:

```text
position = (10, 20)
timestamp = T
```

If no update arrives for 30 seconds, the value may still exist but no longer be trustworthy.

Thus:

```text
value = (10,20)
freshness = STALE
```

rather than simply deleting it.

# 12. Freshness Policy

Each state field can have:

```text
max_age
```

For example:

```text
Battery:
    max_age = 5s

Localization:
    max_age = 100ms

Static map:
    max_age = 24h
```

Different state domains have radically different freshness requirements.

# 13. State Version

State should be versioned.

For example:

```text
Position
version = 184
```

After update:

```text
version = 185
```

This allows consumers to detect changes and synchronization gaps.

# 14. State Revision

A larger state snapshot may have:

```text
StateRevision = 1042
```

with many fields:

```text
revision 1042
 ├── position
 ├── battery
 ├── mission
 └── navigation
```

This is useful for checkpoints and deterministic replay.

# 15. Immutable State Transitions

A strong architecture is:

```text
Old State
    +
Event
    ↓
Reducer
    ↓
New State
```

rather than arbitrary mutation:

```text
anything
 ↓
state.field = x
```

The first approach creates much stronger auditability.

# 16. Reducer

Conceptually:

```rust
fn reduce(
    state: State,
    event: Event,
) -> State
```

The reducer should be deterministic wherever possible.

Same semantic input:

```text
S + E
```

should produce:

```text
S'
```

# 17. Event-Sourced State

This gives us:

```text
Event Log
    ↓
Reducer
    ↓
State
```

The state can therefore be reconstructed.

```text
E1
E2
E3
E4
 ↓
Replay
 ↓
State₄
```

This is extremely valuable for autonomous systems.

# 18. But Event Sourcing Is Not Enough

A real robot may generate enormous event volumes.

Therefore NROS should support:

```text
events
+
snapshots
```

For example:

```text
Events:
E1000 ... E5000

Snapshot:
S5000
```

Recovery can start from the snapshot rather than replaying the entire history.

# 19. Checkpoint

A checkpoint captures enough state to resume or analyze execution.

Conceptually:

```text
Checkpoint {
    epoch,
    state_revision,
    active_work,
    authority,
    resources,
    scheduler_state,
    temporal_context,
}
```

Not every implementation detail needs persistence.

# 20. Snapshot vs Checkpoint

A **snapshot** primarily describes state.

A **checkpoint** describes a recoverable runtime position.

Thus:

```text
Snapshot:
    "What was the state?"

Checkpoint:
    "Where can execution safely resume?"
```

# 21. Derived State

Not all state needs storage.

Example:

```text
battery_voltage = 11.5V
battery_current = 4A
```

can derive:

```text
power ≈ 46W
```

Derived state should ideally carry:

```text
source fields
derivation rule
revision
timestamp
```

so it can be recomputed.

# 22. World Model

An autonomous agent needs more than its own state.

It needs a model of its environment:

```text
WorldModel
│
├── Objects
├── Locations
├── Obstacles
├── Agents
├── Resources
├── Infrastructure
└── Environmental Conditions
```

# 23. World Model Is Not Reality

This distinction is essential.

```text
REAL WORLD
     ↓ sensors
OBSERVATIONS
     ↓ interpretation
WORLD MODEL
```

The model is an approximation.

Therefore:

```text
WorldModel ≠ Truth
```

It represents the system's current knowledge.

# 24. Belief State

For uncertain environments:

```text
WorldModel
```

may contain:

```text
Object 42
position = (10,20)
confidence = 0.72
last_seen = T
```

The confidence and freshness are part of the knowledge representation.

# 25. Multiple Sources

Suppose:

```text
Camera:
    object at X
    confidence 0.8

Lidar:
    object at X'
    confidence 0.9
```

NROS should not blindly overwrite one with the other.

Instead:

```text
Observation Fusion
       ↓
Belief Update
       ↓
World Model
```

# 26. Provenance Chain

The resulting chain can be:

```text
Camera Frame
   ↓
Detection
   ↓
Observation O17
   ↓
Fusion
   ↓
Belief B8
   ↓
World State W203
   ↓
Plan P4
   ↓
Work W71
```

This gives us complete reasoning provenance.

# 27. State and Evidence

Evidence should explain why state exists.

For example:

```text
State:
    obstacle_present = true
```

Evidence:

```text
Observation O17
Observation O19
Fusion Rule F2
```

This is dramatically stronger than:

```text
obstacle_present = true
```

with no explanation.

# 28. State Confidence

A state entry can include:

```text
confidence
freshness
provenance
authority
revision
```

For example:

```text
Localization:
    position = X
    confidence = 0.94
    age = 20 ms
    authority = LocalizationAgent
    revision = 1827
```

# 29. State Conflict

Distributed agents can disagree:

```text
Robot A:
    position = X

Robot B:
    position = Y
```

NROS needs explicit conflict policies.

Possible strategies:

```text
authority wins
newest wins
highest confidence
sensor priority
fusion
consensus
manual arbitration
```

No silent overwrite.

# 30. State Consistency Models

Not every state requires strong consistency.

NROS can support different classes:

```text
STRONG
CAUSAL
EVENTUAL
LOCAL
```

Example:

```text
EmergencyStop:
    strong / authoritative

Telemetry:
    eventual

Camera metadata:
    local/eventual
```

# 31. Strong Consistency Has a Cost

Trying to make every piece of robot state globally consistent creates:

```text
latency
network dependence
availability problems
complexity
```

Therefore consistency should be selected according to semantic requirements.

# 32. State Replication

A state domain can declare:

```text
replication = none
replication = local
replication = selected peers
replication = fleet
```

This allows resource-aware distributed systems.

# 33. State Synchronization

Synchronization can use:

```text
full snapshot
delta
event replay
version negotiation
checkpoint transfer
```

A reconnecting agent can choose the cheapest correct mechanism.

# 34. Delta State

Instead of:

```text
Full State:
    10 MB
```

send:

```text
Delta:
    position changed
    battery changed
```

This is especially valuable over constrained networks.

# 35. State Queries

NROS should support semantic queries:

```text
Query:
    current localization
```

rather than requiring direct access to another component's memory.

The response includes:

```text
value
revision
timestamp
freshness
authority
```

# 36. State Subscriptions

Consumers can subscribe to state changes:

```text
subscribe:
    NavigationState
```

rather than receiving every underlying sensor message.

This reduces coupling.

# 37. State Triggers

State transitions can trigger Work:

```text
battery < 15%
       ↓
Policy
       ↓
Work:
    ReturnToCharge
```

This integrates state directly with the agency layer.

# 38. State Guards

A Work can specify:

```text
precondition:
    battery > 20%
```

Before execution:

```text
State
 ↓
evaluate guard
 ↓
true → execute
false → defer/reject
```

This is much safer than assuming state remains unchanged.

# 39. TOCTOU Problem

There is a subtle race:

```text
Check:
    battery > 20%

time passes

Execute:
    battery = 10%
```

The state changed between validation and execution.

NROS therefore needs stronger mechanisms for critical operations.

# 40. State Preconditions + Revalidation

A critical Work may require:

```text
expected_revision = 1204
```

If state becomes:

```text
revision = 1205
```

the Work must revalidate.

This is optimistic concurrency control.

# 41. Transactional State Transition

For tightly coupled operations:

```text
Read State
   ↓
Validate
   ↓
Reserve Resources
   ↓
Execute
   ↓
Commit Result
```

This provides stronger semantics where necessary.

# 42. State Machine Domains

Some state should explicitly be modeled as a finite-state machine.

Example:

```text
RobotLifecycle:

UNCONFIGURED
    ↓
INACTIVE
    ↓
ACTIVE
    ↓
DEACTIVATING
    ↓
INACTIVE
```

Illegal transitions must be rejected.

# 43. Safety State Machine

Example:

```text
NORMAL
  ↓ fault
DEGRADED
  ↓ severe fault
SAFE
  ↓ recovery
RECOVERING
  ↓
NORMAL
```

This should not be represented as an arbitrary string field.

It is a governed state machine.

# 44. Agent State

An autonomous agent can have:

```text
AgentState
│
├── Identity
├── Lifecycle
├── Goal
├── Beliefs
├── Current Plan
├── Active Work
├── Resources
├── Authority
└── Health
```

This is the runtime representation of an agent's current situation.

# 45. State and Planning

Planning consumes state:

```text
WorldModel
     +
Goals
     +
Constraints
     ↓
Planner
     ↓
Plan
```

The planner should not directly manipulate execution state.

It produces intent.

# 46. Plan vs State

A Plan describes:

```text
what the agent intends to do
```

State describes:

```text
what the runtime currently believes is true
```

Execution bridges the two:

```text
Plan
 ↓
Work
 ↓
Execution
 ↓
State Update
```

# 47. State Divergence

Suppose:

```text
Plan:
    Robot at waypoint B
```

but state says:

```text
Robot at waypoint A
```

NROS must detect:

```text
plan-state divergence
```

and trigger replanning or recovery.

# 48. State as Runtime Truth

A useful rule:

> **Execution must be based on current validated state, not stale planning assumptions.**

Therefore:

```text
Plan
```

is not authoritative over:

```text
State
```

The plan must adapt when reality changes.

# 49. State Recovery

After restart:

```text
Persistent Snapshot
      ↓
Event Replay
      ↓
State Reconstruction
      ↓
Validation
      ↓
Runtime Ready
```

If state cannot be reconstructed reliably:

```text
UNKNOWN
```

is preferable to fabricated certainty.

# 50. UNKNOWN Is a Valid State

This is a critical NROS principle.

Instead of:

```text
battery = 0
```

when the sensor is unavailable, represent:

```text
battery = UNKNOWN
```

Likewise:

```text
position = UNKNOWN
authority = UNKNOWN
connection = UNKNOWN
```

Unknown must not be silently converted into a plausible default.

# 51. Safety Implication

For safety-critical decisions:

```text
UNKNOWN
```

should trigger an explicit policy.

For example:

```text
Localization UNKNOWN
       ↓
navigation prohibited
```

rather than:

```text
Localization UNKNOWN
       ↓
assume position = (0,0)
```

# 52. State Confidence and Safety

Confidence thresholds can be policy-controlled:

```text
navigation:
    confidence ≥ 0.90

inspection:
    confidence ≥ 0.70

telemetry:
    confidence irrelevant
```

The same state can therefore be usable for one Work and unacceptable for another.

# 53. State Contracts

We can define a semantic contract:

```text
StateContract {
    owner
    visibility
    freshness
    consistency
    confidence
    persistence
    replication
}
```

This makes state behavior explicit.

# 54. State Architecture

The complete model now becomes:

```text
                 REAL WORLD
                     │
                  Sensors
                     ↓
               OBSERVATIONS
                     ↓
                 FUSION
                     ↓
             FACTS / BELIEFS
                     ↓
                WORLD STATE
                     ↓
          ┌──────────┴──────────┐
          ↓                     ↓
       PLANNER              MONITOR
          ↓                     ↓
        PLAN                 POLICY
          ↓                     ↓
        WORK  ←───────────────┘
          ↓
      EXECUTION
          ↓
       EFFECT
          ↓
     OBSERVATION
          ↓
       STATE UPDATE
```

This closes the autonomous loop.

# 55. Event-Sourced State Architecture

A more implementation-oriented representation:

```text
                   EVENT LOG
                       │
             ┌─────────┴─────────┐
             ↓                   ↓
         Reducer A           Reducer B
             ↓                   ↓
       Agent State         World State
             │                   │
             └─────────┬─────────┘
                       ↓
                  Runtime State
                       │
          ┌────────────┼────────────┐
          ↓            ↓            ↓
       Planner      Scheduler    Monitor
```

Different state projections can be derived from the same event history.

# 56. Why This Is Powerful

The same historical event stream can reconstruct:

```text
agent state
world model
mission state
execution history
safety state
diagnostic views
```

without requiring every consumer to maintain an independent authoritative database.

# 57. But Avoid a Giant Global Event Store

The architecture should remain partitionable:

```text
Robot-local event store
       +
Mission event store
       +
Fleet event store
```

rather than forcing every robot to replicate everything.

# 58. State Partitioning

A natural partition:

```text
LOCAL
  ↓
ROBOT
  ↓
MISSION
  ↓
FLEET
```

Each level owns different information.

This aligns with the hierarchical scheduling model introduced earlier.

# 59. State Authority Hierarchy

For example:

```text
Sensor
  ↓
Subsystem
  ↓
Robot
  ↓
Mission Coordinator
  ↓
Fleet Coordinator
```

But higher-level authority should not automatically override lower-level physical facts.

A fleet coordinator can issue:

```text
mission = RETURN
```

but cannot truthfully declare:

```text
battery = 90%
```

if the robot's battery subsystem says otherwise.

# 60. NROS State Invariants

The state subsystem should enforce:

```text
1. Events are historical; state is current knowledge.

2. Observations are not automatically facts.

3. Unknown is a valid state.

4. State ownership is explicit.

5. State freshness is explicit.

6. State revisions are monotonic within an authority domain.

7. Derived state identifies its source.

8. Conflicting authorities cannot silently overwrite one another.

9. Critical Work can require state revalidation.

10. State reconstruction must preserve provenance.

11. Simulation state cannot silently become production state.

12. Plans never override verified runtime state.
```

# 61. The NROS Knowledge Pipeline

We can now formalize the central knowledge pipeline:

```text
EVENT
  │
  ↓
OBSERVATION
  │
  ↓
INTERPRETATION
  │
  ├──→ FACT
  │
  └──→ BELIEF
          │
          ↓
       WORLD MODEL
          │
          ↓
        STATE
          │
          ↓
       PLANNING
          │
          ↓
         WORK
          │
          ↓
      EXECUTION
          │
          ↓
      NEW EVENT
```

This is the core **Observe → Understand → Act → Verify** cycle.

# 62. What Is Still Missing?

We now have:

```text
Agent
Goal
Plan
Work
Scheduler
Executor
Communication
Time
State
World Model
Evidence
```

But autonomous planning requires one more major abstraction:

> **How does NROS represent capabilities, actions, preconditions, effects, resources, and plans in a machine-executable way?**

Without this, an agent can observe state and schedule Work, but cannot systematically reason about **what actions are possible**.

That takes us to the next major layer.

# Part LXVII — NROS Capability & Action Model

We will derive:

```text
Capability
Action
Operation
Precondition
Effect
Resource Requirement
Constraint
Skill
Tool
Plan Step
Action Graph
Dependency
Fallback
Recovery
```

and establish the transformation:

```text
WORLD STATE
     ↓
AVAILABLE CAPABILITIES
     ↓
POSSIBLE ACTIONS
     ↓
PRECONDITION CHECK
     ↓
PLAN
     ↓
WORK
     ↓
EXECUTION
     ↓
EFFECT
     ↓
STATE UPDATE
```

This is where NROS moves from an **execution runtime** toward a genuine **agent-native robotics runtime**.

# NROS — Part LXVII: Capability & Action Model

We now need to define the bridge between **knowledge** and **action**.

The runtime knows:

```text
Current State
World Model
Goals
Resources
Authority
Time
```

But that does not yet answer:

> **What can the system actually do?**

That question belongs to the **Capability & Action Model**.

# 1. Capability

A capability describes an ability available to an agent or subsystem.

Examples:

```text
Navigate
MoveArm
ReadTemperature
CaptureImage
OpenValve
ChargeBattery
InspectComponent
```

A capability is not necessarily an execution itself.

It is a declaration of possibility.

```text
Agent
  ↓
Capability
  ↓
Action
  ↓
Execution
```

# 2. Capability vs Action

This distinction is fundamental.

### Capability

```text
"I can navigate."
```

### Action

```text
"Navigate to waypoint B."
```

### Execution

```text
"Navigation execution #742 is currently running."
```

Therefore:

```text
Capability
    ≠
Action
    ≠
Execution
```

# 3. Capability Ownership

Every capability should have an owner:

```text
Capability:
    navigation

Owner:
    NavigationSubsystem
```

or:

```text
Capability:
    arm.motion

Owner:
    ArmController
```

This establishes authority boundaries.

# 4. Capability Identity

Capabilities should have stable identifiers:

```text
navigation
navigation.goto
navigation.stop
arm.move
arm.grasp
camera.capture
battery.charge
```

Namespaces prevent collisions.

# 5. Capability Metadata

A capability can expose:

```text
Capability {
    id
    owner
    version
    inputs
    outputs
    preconditions
    effects
    resources
    timing
    authority
    reliability
}
```

This turns capability discovery into something machine-readable.

# 6. Capability Discovery

An agent can ask:

```text
"What capabilities are available?"
```

and receive:

```text
navigation.goto
navigation.stop
camera.capture
arm.move
```

This is more powerful than discovering only topics.

# 7. Capability Availability

A capability may exist but currently be unavailable.

For example:

```text
Capability:
    arm.move

Status:
    AVAILABLE
```

Later:

```text
arm.move
    ↓
BUSY
```

or:

```text
arm.move
    ↓
FAULTED
```

Thus capability identity and capability availability must be separate.

# 8. Capability State

Possible lifecycle:

```text
UNKNOWN
DISCOVERED
AVAILABLE
RESERVED
BUSY
DEGRADED
UNAVAILABLE
FAULTED
```

This state can itself be observable.

# 9. Action

An Action is an invocation of a capability with concrete parameters.

Example:

```text
Capability:
    navigation.goto
```

Action:

```text
navigation.goto(
    destination = B
)
```

The action is an intent to perform something.

# 10. Action Is Not Yet Execution

Important distinction:

```text
Action
   ↓
accepted
   ↓
Work
   ↓
Execution
```

The action may be rejected before execution.

For example:

```text
navigation.goto(B)
```

could fail because:

```text
battery too low
```

without ever becoming an execution.

# 11. Action Schema

Conceptually:

```rust
struct Action {
    id: ActionId,
    capability: CapabilityId,
    arguments: Arguments,
    requested_by: AgentId,
    authority: AuthorityRef,
    constraints: ActionConstraints,
}
```

# 12. Preconditions

An Action should describe what must already be true.

Example:

```text
navigation.goto(B)

preconditions:
    localization.valid
    battery > 20%
    navigation.available
```

The runtime evaluates these before dispatch.

# 13. Preconditions Are Not Just Boolean

A condition may have:

```text
value
confidence
freshness
authority
revision
```

So:

```text
battery > 20%
```

is only valid if the battery state is sufficiently fresh and trusted.

# 14. State Guard

A precondition can therefore become:

```text
Guard {
    expression,
    minimum_confidence,
    maximum_age,
    required_revision,
}
```

This ties the action model directly to the State Model.

# 15. Effects

An Action also describes expected consequences.

For example:

```text
Action:
    open_valve(V1)

Expected effect:
    valve.V1 = OPEN
```

But:

> An expected effect is not automatically a verified effect.

# 16. Expected vs Observed Effect

This distinction is critical.

```text
Action
  ↓
Expected Effect
```

versus:

```text
Physical World
  ↓
Observation
  ↓
Verified Effect
```

Therefore:

```text
expected_effect ≠ actual_effect
```

# 17. Action Lifecycle

A useful lifecycle:

```text
PROPOSED
   ↓
VALIDATING
   ↓
AUTHORIZED
   ↓
ADMITTED
   ↓
DISPATCHED
   ↓
RUNNING
   ↓
VERIFYING
   ↓
SUCCEEDED
```

Failure paths:

```text
REJECTED
CANCELLED
FAILED
TIMED_OUT
ABORTED
UNKNOWN
```

# 18. Why UNKNOWN Matters

Suppose:

```text
OpenValve
```

was sent.

The controller reports:

```text
accepted
```

Then communication disappears.

We cannot safely conclude:

```text
valve = OPEN
```

The correct result may be:

```text
execution = UNKNOWN
```

until independently verified.

# 19. Action Verification

The runtime should therefore support:

```text
Action
  ↓
Execution
  ↓
Expected Effect
  ↓
Observation
  ↓
Verification
```

This produces:

```text
VERIFIED_SUCCESS
```

rather than merely:

```text
COMMAND_SENT
```

# 20. Capability vs Skill

For autonomous systems, a useful distinction is:

### Capability

Primitive ability:

```text
arm.move
```

### Skill

Higher-level reusable behavior:

```text
pick_object
```

A skill can be implemented using several capabilities:

```text
pick_object
   ├── camera.detect
   ├── arm.move
   ├── gripper.close
   └── force_sensor.read
```

# 21. Skill Composition

Skills become reusable building blocks:

```text
inspect_component
    ↓
navigate_to
    ↓
capture_image
    ↓
analyze_image
    ↓
record_result
```

This creates a hierarchy:

```text
Capability
    ↓
Action
    ↓
Skill
    ↓
Plan
    ↓
Mission
```

# 22. Tool

An agent may also use external tools:

```text
Tool:
    camera
    database
    shell
    network
    PLC
    robotic arm
```

A tool exposes capabilities.

Therefore:

```text
Tool
  ↓
Capabilities
  ↓
Actions
```

This is particularly relevant to agentic systems.

# 23. Resources

Actions consume resources.

Examples:

```text
battery
CPU
memory
motor
arm
network
operator attention
physical workspace
```

A capability should declare resource requirements where relevant.

# 24. Resource Requirement

For example:

```text
navigation.goto

requires:
    localization
    drive_system
    battery > threshold
```

or:

```text
arm.move

requires:
    arm_controller
    joint_access
    workspace_lock
```

# 25. Resource Reservation

Before execution:

```text
Action
   ↓
Resource Check
   ↓
Reserve
   ↓
Execute
   ↓
Release
```

This integrates directly with the scheduler.

# 26. Resource Conflict

Suppose two actions request:

```text
ArmController
```

simultaneously:

```text
Action A → arm
Action B → arm
```

The runtime must resolve:

```text
priority
ordering
reservation
preemption
```

rather than allowing uncontrolled concurrent access.

# 27. Capability Constraints

A capability can have hard limits:

```text
arm.move:
    max_velocity
    max_acceleration
    workspace
    payload_limit
```

An action outside these constraints should never reach physical execution.

# 28. Capability Contracts

Conceptually:

```text
CapabilityContract {
    inputs
    outputs

    preconditions
    effects

    resources
    constraints

    timing
    authority

    failure_modes
    verification
}
```

This becomes a machine-readable description of what the capability promises.

# 29. Action Planning

Now the planner can reason:

```text
Goal:
    inspect machine

Available capabilities:
    navigate
    camera.capture
    image.analyze
```

It can construct:

```text
navigate(machine)
      ↓
capture_image()
      ↓
analyze_image()
      ↓
record_result()
```

# 30. Action Graph

A Plan can therefore be represented as a graph:

```text
          ┌──────────────┐
          │ Navigate     │
          └──────┬───────┘
                 ↓
          ┌──────────────┐
          │ Capture      │
          └──────┬───────┘
                 ↓
          ┌──────────────┐
          │ Analyze      │
          └──────┬───────┘
                 ↓
          ┌──────────────┐
          │ Report       │
          └──────────────┘
```

But plans need not always be linear.

# 31. Parallel Actions

Example:

```text
           ┌── Camera ────┐
Navigate ──┤              ├── Analyze
           └── Lidar ─────┘
```

Both sensing operations can execute concurrently.

The planner should express dependency rather than forcing sequential execution.

# 32. Dependency Types

At minimum:

```text
SEQUENTIAL
PARALLEL
CONDITIONAL
OPTIONAL
MUTUALLY_EXCLUSIVE
RETRY
FALLBACK
```

# 33. Conditional Branch

Example:

```text
DetectObject
      ↓
 ┌────┴────┐
 ↓         ↓
YES        NO
 ↓         ↓
Inspect    Search
```

This is an Action Graph, not simply a list.

# 34. Fallback

Suppose:

```text
Primary:
    GPS navigation
```

fails.

Fallback:

```text
Visual navigation
```

The plan can encode:

```text
GPS
 ↓ failure
Visual
 ↓ failure
SafeStop
```

# 35. Recovery

Recovery should be explicit rather than hidden inside arbitrary error handlers.

Example:

```text
MoveArm
   ↓
FAILED
   ↓
RetractArm
   ↓
ResetController
   ↓
Retry MoveArm
```

This makes recovery itself executable and auditable.

# 36. Retry Policy

An action may specify:

```text
max_attempts
backoff
retryable_errors
deadline
```

Example:

```text
max_attempts = 3
backoff = exponential
retryable = NETWORK_TIMEOUT
```

A physical actuator fault should probably not automatically receive the same retry policy as a network timeout.

# 37. Compensation

Some actions require compensating actions.

Example:

```text
ReserveResource
   ↓
PerformOperation
   ↓
failure
   ↓
ReleaseResource
```

Or:

```text
OpenValve
   ↓
later failure
   ↓
CloseValve
```

This resembles transactional compensation but applies to physical actions.

# 38. Physical Actions Are Not Transactions

A key invariant:

> **A physical effect cannot necessarily be rolled back.**

For example:

```text
MoveRobotForward
```

cannot simply be "undone" by:

```text
MoveRobotBackward
```

because the environment may have changed.

Therefore compensation is a new action, not a magical rollback.

# 39. Action Preconditions + Effects

This begins to resemble classical planning:

```text
Action:
    open_door

Precondition:
    door_closed
    access_authorized

Effect:
    door_open
```

But NROS needs richer semantics:

```text
confidence
freshness
authority
resources
time
verification
uncertainty
```

# 40. Temporal Actions

An Action may have:

```text
start condition
duration
deadline
end condition
```

Example:

```text
ChargeBattery

start:
    battery < 80%

duration:
    variable

end:
    battery >= 80%
```

The runtime monitors the temporal contract.

# 41. Continuous Actions

Some actions are not instantaneous:

```text
Navigate
Charge
Inspect
Track
MaintainTemperature
```

Their execution is a process:

```text
START
  ↓
RUNNING
  ↓
progress
  ↓
completion
```

# 42. Progress

A long-running action can expose:

```text
progress = 0.0 ... 1.0
```

but progress should be semantically meaningful.

For navigation:

```text
distance_remaining
```

may be more useful than arbitrary percentages.

# 43. Cancellation

Cancellation should be explicit:

```text
RUNNING
   ↓ cancel
CANCELLING
   ↓
CANCELLED
```

But physical systems may require safe termination:

```text
RUNNING
   ↓ cancel
SAFE_STOP
   ↓
CANCELLED
```

# 44. Preemption

A higher-priority action may preempt another:

```text
Patrol
   ↓
EmergencyStop
```

The runtime must define:

```text
can_preempt?
how?
what cleanup?
resume later?
discard?
```

# 45. Preemption Classes

Possible:

```text
NON_PREEMPTIBLE
COOPERATIVE
IMMEDIATE
SAFE_POINT
```

A motor control action may require a safe stopping point rather than arbitrary interruption.

# 46. Authority and Actions

Before dispatch:

```text
Action
   ↓
Identity
   ↓
Authority
   ↓
Capability Permission
   ↓
Resource Admission
   ↓
Execution
```

Authentication alone is insufficient.

# 47. Capability Delegation

An agent may delegate:

```text
Agent A
   ↓
delegates capability
   ↓
Agent B
```

The delegation should define:

```text
scope
duration
constraints
revocation
```

This naturally integrates with the lease model.

# 48. Capability Revocation

If authority is revoked:

```text
Capability
   ↓
REVOKED
```

then new actions must be rejected.

For active execution:

```text
running action
   ↓
revocation policy
   ↓
continue / safe-stop / abort
```

This must be explicitly defined.

# 49. Capability Versioning

Capabilities evolve.

Example:

```text
navigation.goto@1
navigation.goto@2
```

The runtime should support compatibility negotiation.

This prevents an agent from assuming that two similarly named capabilities have identical semantics.

# 50. Capability Negotiation

Two agents may negotiate:

```text
Required:
    navigation.goto
    version >= 2

Provider:
    version = 3
```

Then:

```text
compatible → proceed
```

Otherwise:

```text
fallback / reject
```

# 51. Capability Composition

Capabilities can themselves be composed.

Example:

```text
InspectMachine
    =
Navigate
+
Capture
+
Analyze
+
Report
```

This means higher-level capabilities can be represented as executable graphs rather than monolithic code.

# 52. Hierarchical Capability Model

The resulting hierarchy:

```text
Mission
   ↓
Skill
   ↓
Composite Capability
   ↓
Primitive Capability
   ↓
Hardware / Software Interface
```

Example:

```text
Inspect Elevator
   ↓
Inspect Door
   ↓
Move Door
   ↓
Motor Command
   ↓
Hardware Register
```

This is particularly powerful for industrial robotics.

# 53. Capability Registry

NROS can maintain a registry:

```text
Capability Registry
│
├── identity
├── provider
├── version
├── schema
├── state
├── authority
├── resources
└── contracts
```

Discovery becomes queryable.

# 54. Capability Registry ≠ Service Registry

A traditional service registry answers:

```text
"Where is service X?"
```

A capability registry should answer:

```text
"Who can perform X?"
"Under what conditions?"
"With what constraints?"
"Using which resources?"
"With what guarantees?"
```

That is a much richer abstraction.

# 55. Capability Selection

If multiple agents provide:

```text
navigation.goto
```

the planner can select based on:

```text
distance
latency
confidence
energy
authority
availability
cost
reliability
```

This creates **capability-aware planning**.

# 56. Capability Cost

An action may expose a cost function:

```text
cost =
    energy
  + latency
  + risk
  + resource_usage
```

The planner can choose between alternatives.

# 57. Risk

For physical systems, minimizing latency alone is insufficient.

An action may have:

```text
risk = 0.02
```

while another has:

```text
risk = 0.001
```

A mission policy can prefer the safer option even if it is slower.

# 58. Action Feasibility

Before creating Work:

```text
Goal
 ↓
Candidate Actions
 ↓
Preconditions
 ↓
Capabilities
 ↓
Resources
 ↓
Authority
 ↓
Time
 ↓
Risk
 ↓
Feasible Actions
```

Only then should planning commit to execution.

# 59. Planning Boundary

This suggests an important architectural boundary:

```text
Planner
   ↓
Action Intent
```

then:

```text
Runtime
   ↓
Validation
   ↓
Admission
   ↓
Work
```

The planner proposes.

The runtime enforces.

This prevents an AI planner from bypassing safety/runtime constraints.

# 60. AI Planner Integration

An LLM or other planner can generate:

```text
ActionPlan
```

but NROS should treat that output as **untrusted intent** until validated.

```text
AI Planner
    ↓
Candidate Plan
    ↓
Schema Validation
    ↓
Capability Validation
    ↓
Authority Validation
    ↓
State Validation
    ↓
Resource Validation
    ↓
Temporal Validation
    ↓
Executable Plan
```

This is a major safety boundary.

# 61. No Direct AI → Actuator Path

The architecture should explicitly prohibit:

```text
LLM
 ↓
motor command
```

Instead:

```text
LLM
 ↓
intent
 ↓
planner
 ↓
NROS validation
 ↓
capability
 ↓
authority
 ↓
work
 ↓
executor
 ↓
actuator
```

The runtime remains the enforcement layer.

# 62. Capability Model

The complete conceptual object is now:

```text
Capability
│
├── Identity
├── Provider
├── Version
├── Inputs
├── Outputs
├── Preconditions
├── Effects
├── Resources
├── Constraints
├── Timing
├── Authority
├── Failure Modes
├── Verification
└── Cost / Risk
```

# 63. Action Model

And:

```text
Action
│
├── Capability
├── Arguments
├── Preconditions
├── Expected Effects
├── Resources
├── Timing
├── Authority
├── Priority
├── Retry Policy
├── Cancellation Policy
└── Verification Policy
```

# 64. Complete Action Pipeline

We can now connect everything:

```text
              GOAL
                ↓
           WORLD MODEL
                ↓
       CAPABILITY DISCOVERY
                ↓
         ACTION GENERATION
                ↓
       PRECONDITION CHECK
                ↓
       RESOURCE ANALYSIS
                ↓
       AUTHORITY CHECK
                ↓
       TEMPORAL CHECK
                ↓
          RISK ANALYSIS
                ↓
              PLAN
                ↓
              WORK
                ↓
           SCHEDULER
                ↓
            EXECUTOR
                ↓
          COMMUNICATION
                ↓
          PHYSICAL EFFECT
                ↓
           OBSERVATION
                ↓
          VERIFICATION
                ↓
             STATE
```

This is becoming the central NROS execution architecture.

# 65. Architectural Invariants

The capability/action subsystem should enforce:

```text
1. Capability ≠ Action ≠ Execution.

2. Capability availability is explicit.

3. Actions require validated preconditions.

4. Expected effects are not treated as verified effects.

5. Physical actions are not assumed reversible.

6. Resources must be admitted before execution.

7. Authority is checked before dispatch.

8. Capability versions are explicit.

9. Planner output is untrusted intent.

10. Runtime validation cannot be bypassed by the planner.

11. Unknown execution outcomes remain UNKNOWN until resolved.

12. Long-running actions expose explicit lifecycle state.

13. Cancellation and preemption have defined safety semantics.

14. Recovery and compensation are explicit actions.
```

# 66. NROS Is Now a Capability-Oriented Runtime

The architecture has evolved significantly:

```text
ROS:
    nodes
    topics
    services
    actions

NROS:
    agents
    goals
    beliefs
    capabilities
    actions
    plans
    work
    resources
    authority
    execution
    state
    evidence
    time
    communication
```

The difference is not merely additional APIs.

The **semantic center has moved from message transport to autonomous execution**.

# 67. The Next Missing Piece: Resources

We have repeatedly referenced:

```text
CPU
memory
motors
battery
network
locks
workspace
operator attention
```

but we have not yet defined them formally.

This is dangerous because scheduling and planning cannot reason correctly about capabilities without a resource model.

The next layer therefore becomes:

# Part LXVIII — NROS Resource & Reservation Model

We will derive:

```text
Resource
Resource Type
Capacity
Unit
Ownership
Availability
Reservation
Allocation
Lease
Contention
Priority
Preemption
Resource Pools
Composite Resources
Exclusive Resources
Shared Resources
Consumable Resources
Renewable Resources
Physical Resources
Virtual Resources
```

and establish:

```text
CAPABILITY
     ↓
RESOURCE REQUIREMENTS
     ↓
RESOURCE DISCOVERY
     ↓
RESERVATION
     ↓
ADMISSION
     ↓
WORK
     ↓
EXECUTION
     ↓
RELEASE
```

This is the layer that will connect **planning, scheduling, physical hardware, concurrency, and real-time execution** into one coherent resource-management model.

# NROS — Part LXVIII: Resource & Reservation Model

We now have capabilities, actions, work, scheduling, time, authority, communication, and state.

But every real execution eventually encounters the same question:

> **What does this operation require, and who gets access to it?**

That is the purpose of the **NROS Resource Model**.

# 1. Resource

A Resource is something required, consumed, occupied, or constrained by execution.

Examples:

```text
CPU
Memory
GPU
Battery
Motor
Arm
Sensor
Network Link
Storage
Workspace
Radio Channel
Operator Attention
```

Resources can be physical or virtual.

# 2. Resource ≠ Capability

A capability describes:

```text
what can be done
```

A resource describes:

```text
what is required to do it
```

Example:

```text
Capability:
    arm.move

Resources:
    arm_controller
    joint_group
    workspace
    power
```

# 3. Resource Types

NROS should distinguish several resource classes.

```text
EXCLUSIVE
SHARED
CONSUMABLE
RENEWABLE
CAPACITY
POOL
COMPOSITE
VIRTUAL
PHYSICAL
```

These have different allocation semantics.

# 4. Exclusive Resource

Only one Work may use it at a time.

Example:

```text
ArmController
```

If:

```text
Work A → ArmController
```

then:

```text
Work B → ArmController
```

must wait, unless explicit preemption is permitted.

# 5. Shared Resource

Multiple consumers may use it concurrently.

Example:

```text
CPU
```

or:

```text
Read-only map
```

The important property is that simultaneous access is permitted under defined limits.

# 6. Capacity Resource

A resource can have finite capacity.

Example:

```text
CPU capacity = 100%
```

Work A:

```text
requires = 30%
```

Work B:

```text
requires = 40%
```

Together:

```text
70%
```

may be acceptable.

# 7. Capacity Is Not Binary

This is important.

A resource should not always be:

```text
FREE / BUSY
```

It may instead be:

```text
capacity = 100
allocated = 65
available = 35
```

This supports realistic scheduling.

# 8. Consumable Resource

Some resources are consumed permanently or semi-permanently.

Examples:

```text
battery energy
fuel
material
storage capacity
chemical reagent
```

If Work consumes:

```text
10 Wh
```

that resource does not simply become "released".

# 9. Renewable Resource

Some resources recover over time.

Example:

```text
Battery
```

It is partly consumable, but can be replenished.

NROS therefore needs a distinction between:

```text
capacity
current level
replenishment rate
```

# 10. Resource Pool

Several interchangeable resources can form a pool.

Example:

```text
CPU Pool
├── Core 0
├── Core 1
├── Core 2
└── Core 3
```

A Work may request:

```text
2 CPU cores
```

without caring which specific cores are allocated.

# 11. Resource Selection

For interchangeable resources:

```text
Requirement
   ↓
Pool
   ↓
eligible resources
   ↓
selection policy
```

Selection may consider:

```text
load
locality
temperature
energy
priority
affinity
availability
```

# 12. Physical Resource

A physical actuator is a resource:

```text
Motor M1
```

Unlike CPU time, it has physical constraints.

For example:

```text
max_velocity
max_current
thermal_limit
position_range
```

Resource admission must respect these.

# 13. Virtual Resource

Some resources exist only logically:

```text
NavigationLock
MissionSlot
SafetyAuthority
DatabaseTransaction
```

Virtual resources are extremely useful for coordination.

# 14. Resource Ownership

Every resource should have an owner:

```text
Motor M1
owner = MotorController
```

Ownership does not necessarily mean exclusive use.

It identifies the authoritative manager.

# 15. Resource Manager

NROS can expose a Resource Manager:

```text
Resource Manager
│
├── discover
├── inspect
├── reserve
├── allocate
├── release
├── revoke
└── monitor
```

This becomes the bridge between planning and physical execution.

# 16. Resource Requirement

An Action declares requirements.

Example:

```text
arm.move

requires:
    arm_controller = 1
    workspace = ZoneA
    power >= 50W
```

The scheduler can then determine feasibility.

# 17. Hard vs Soft Requirements

Some requirements are mandatory:

```text
arm_controller = REQUIRED
```

Others are preferences:

```text
preferred CPU core = 2
```

NROS should distinguish:

```text
HARD CONSTRAINT
```

from:

```text
SOFT PREFERENCE
```

# 18. Resource Reservation

Reservation means:

> This resource has been committed to a future execution window.

Example:

```text
Work W42
   ↓
reserve ArmController
   ↓
T = 10s → 20s
```

The reservation may exist before actual execution begins.

# 19. Reservation vs Allocation

These are distinct.

### Reservation

```text
"I intend to use this resource."
```

### Allocation

```text
"I am currently using this resource."
```

So:

```text
RESERVED
   ↓
ALLOCATED
   ↓
RELEASED
```

# 20. Why Reservation Matters

Suppose two plans exist:

```text
Plan A:
    arm at T+10

Plan B:
    arm at T+12
```

Without reservation, both plans may appear feasible.

Reservation allows the scheduler to detect the conflict before execution.

# 21. Resource Admission

Before Work becomes runnable:

```text
Work
 ↓
requirements
 ↓
resource feasibility
 ↓
authority
 ↓
time
 ↓
ADMIT
```

If resources cannot be guaranteed:

```text
WAIT / REPLAN / REJECT
```

# 22. Resource Lease

A reservation can expire.

```text
Reservation:
    resource = Arm
    expires = T+30s
```

If the owner does not activate or renew it:

```text
reservation expires
```

and the resource becomes available again.

This connects directly to the time model.

# 23. Resource Contention

Suppose:

```text
Work A → Motor
Work B → Motor
```

The scheduler needs an arbitration policy.

Possible:

```text
priority
deadline
fairness
mission criticality
safety
first-come
authority
```

# 24. Safety Dominates Ordinary Priority

An emergency operation:

```text
EmergencyStop
```

must normally outrank:

```text
RoutineInspection
```

Therefore priority should be multidimensional.

# 25. Resource Priority

A useful conceptual ordering:

```text
SAFETY
   ↓
CRITICAL
   ↓
MISSION
   ↓
NORMAL
   ↓
BACKGROUND
```

But safety semantics should not be reduced to an integer alone.

# 26. Deadlock

Multiple resources create deadlock risk.

Example:

```text
Work A:
    locks Arm
    waits for Camera

Work B:
    locks Camera
    waits for Arm
```

Both wait forever.

NROS must explicitly address resource acquisition ordering.

# 27. Global Resource Ordering

One solution:

```text
Resource IDs have ordering:

Camera < Arm < Drive
```

Work must acquire resources in ascending order.

This reduces deadlock risk.

# 28. Try-Lock / Abort

Another strategy:

```text
Acquire A
 ↓
try B
 ↓
failure
 ↓
release A
 ↓
retry
```

This avoids indefinite waiting.

# 29. Resource Bundles

A Work may require multiple resources atomically:

```text
{
    Arm,
    Camera,
    WorkspaceA
}
```

The scheduler should ideally avoid partially admitting:

```text
Arm = acquired
Camera = acquired
Workspace = unavailable
```

unless partial allocation is explicitly allowed.

# 30. Composite Resources

Some resources are composed:

```text
RobotArm
├── Controller
├── JointGroup
├── Gripper
└── Power
```

A capability may require the composite resource rather than manually listing every internal component.

# 31. Resource Hierarchy

Resources can therefore form:

```text
Robot
 ├── Arm
 │    ├── Joint1
 │    ├── Joint2
 │    └── Gripper
 ├── Camera
 └── Battery
```

This supports hierarchical allocation.

# 32. Resource Affinity

Some Work prefers local resources.

Example:

```text
AI inference
```

prefer:

```text
GPU physically attached to sensor host
```

rather than sending data across the network.

Affinity becomes a scheduling hint.

# 33. Resource Locality

Distributed execution makes locality important:

```text
Sensor
   ↓
GPU
```

is preferable to:

```text
Sensor
   ↓ network
GPU
```

when bandwidth and latency matter.

The resource model therefore needs topology awareness.

# 34. Resource Cost

A resource allocation may have a cost:

```text
CPU cost
Energy cost
Network cost
Thermal cost
Latency cost
```

Planning can optimize across these dimensions.

# 35. Energy as a Resource

Battery deserves special treatment.

Represent:

```text
energy:
    capacity
    current
    reserve
    discharge_rate
    recharge_rate
```

Then actions can declare:

```text
estimated_energy = 20 Wh
```

# 36. Energy Reservation

The planner could reserve energy:

```text
Mission:
    requires ≥ 100 Wh reserve
```

An action consuming too much energy can be rejected before execution.

# 37. Resource Prediction

Some resources change continuously.

Example:

```text
Battery:
    60%
```

estimated after action:

```text
48%
```

The planner can reason over projected state rather than only current state.

# 38. Resource Constraints and Planning

Now planning becomes:

```text
Goal
 ↓
Candidate Plan
 ↓
Temporal constraints
 ↓
Resource requirements
 ↓
Resource availability
 ↓
Projected consumption
 ↓
Feasible Plan
```

This is much closer to real robotic planning.

# 39. Resource Reservation Timeline

For example:

```text
Time ─────────────────────────────>

Arm:
      [Inspect]───────
                       [Repair]────

Camera:
      [Inspect]───────

Battery:
      consumption ────────────────>
```

A scheduler can reason about resource overlap.

# 40. Resource Scheduling

The scheduler now has a richer input:

```text
Work
 ├── Priority
 ├── Deadline
 ├── Resources
 ├── Authority
 ├── Dependencies
 └── Execution Cost
```

Scheduling becomes constrained optimization rather than simply:

```text
pop next queue item
```

# 41. Preemption

If:

```text
low-priority Work
```

holds:

```text
critical resource
```

and emergency Work arrives, the scheduler may request preemption.

But physical resources require safe release.

# 42. Resource-Safe Preemption

Instead of:

```text
kill task
```

the sequence can be:

```text
PREEMPT_REQUEST
      ↓
SAFE_POINT
      ↓
RESOURCE_RELEASE
      ↓
PREEMPTED
```

This is especially important for robotics.

# 43. Resource Revocation

A resource can also be forcibly revoked:

```text
Resource
 ↓
REVOKE
 ↓
current owner notified
 ↓
safe termination
```

This should be reserved for authority or safety mechanisms.

# 44. Resource Failure

Resources can fail:

```text
Camera
 ↓
FAULT
```

Any Work requiring that resource should be reevaluated.

Possible transitions:

```text
RUNNING
 ↓
resource fault
 ↓
PAUSE
 / \
REPLAN ABORT
```

# 45. Dynamic Resource Availability

A resource may change state:

```text
AVAILABLE
   ↓
DEGRADED
   ↓
UNAVAILABLE
   ↓
RECOVERING
   ↓
AVAILABLE
```

The scheduler must respond to these transitions.

# 46. Resource Health

A resource should expose health metadata:

```text
temperature
fault_count
utilization
latency
capacity
quality
```

This enables intelligent allocation.

# 47. Resource Quality

Two nominally identical resources may differ.

For example:

```text
Camera A:
    quality = 0.95

Camera B:
    quality = 0.70
```

A vision task can request:

```text
minimum quality >= 0.9
```

This is richer than simple availability.

# 48. Resource Reservation and Authority

Reservation itself must be authorized.

An untrusted agent must not be able to reserve:

```text
all motors
```

and prevent legitimate safety operations.

Therefore:

```text
Identity
 ↓
Authority
 ↓
Reservation Policy
 ↓
Resource Manager
```

# 49. Resource Quotas

Agents can have quotas:

```text
Agent A:
    CPU ≤ 20%
    Memory ≤ 512MB
    Network ≤ 10Mbps
```

This prevents one agent from monopolizing the system.

# 50. Resource Budget

Mission-level planning can impose budgets:

```text
Mission Budget:
    Energy ≤ 500 Wh
    Network ≤ 2 GB
    Time ≤ 30 min
```

Every Work consumes part of the budget.

# 51. Hierarchical Budgets

Budgets can be nested:

```text
Mission
 ├── Navigation Budget
 │    ├── Energy
 │    └── Time
 │
 └── Inspection Budget
      ├── Energy
      └── Compute
```

This maps naturally onto hierarchical agent systems.

# 52. Resource Accounting

After execution:

```text
estimated:
    20 Wh

actual:
    23 Wh
```

The difference becomes evidence.

Over time, NROS can improve estimates.

# 53. Resource Prediction

Historical evidence can inform future planning:

```text
Past executions:
    average = 22.4 Wh
    variance = ...
```

The planner can then use:

```text
expected consumption
```

instead of optimistic theoretical values.

# 54. Resource Accounting + Evidence

This creates another traceability chain:

```text
Plan
 ↓
Resource Estimate
 ↓
Reservation
 ↓
Execution
 ↓
Actual Usage
 ↓
Evidence
 ↓
Model Update
```

The runtime becomes self-observing.

# 55. Resource Semantics

A useful abstract representation:

```rust
struct Resource {
    id: ResourceId,
    kind: ResourceKind,
    owner: EntityId,
    capacity: Capacity,
    availability: Availability,
    health: Health,
    policy: ResourcePolicy,
}
```

Again, this is conceptual rather than a frozen API.

# 56. Requirement Semantics

```rust
struct ResourceRequirement {
    resource_kind: ResourceKind,
    quantity: Quantity,
    mode: AccessMode,
    duration: Option<Duration>,
    constraints: Constraints,
}
```

Potential access modes:

```text
EXCLUSIVE
SHARED
READ
WRITE
CONSUME
RESERVE
```

# 57. Reservation Semantics

```rust
struct Reservation {
    id: ReservationId,
    resource: ResourceId,
    owner: WorkId,
    start: Time,
    end: Time,
    quantity: Quantity,
    lease: Lease,
}
```

The final implementation can evolve, but the semantic fields should remain traceable.

# 58. Resource Lifecycle

A resource may move through:

```text
DISCOVERED
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
```

Failure:

```text
FAULTED
```

Recovery:

```text
RECOVERING
```

# 59. Resource Graph

Resources can be represented as a graph:

```text
Robot
 ├── Power
 │    └── Battery
 │
 ├── Compute
 │    ├── CPU
 │    └── GPU
 │
 └── Motion
      ├── Arm
      └── Gripper
```

Capabilities reference this graph.

# 60. Resource-Aware Capability

A capability therefore becomes:

```text
arm.grasp
│
├── Preconditions
│
├── Resources
│   ├── Arm
│   ├── Gripper
│   ├── Power
│   └── Workspace
│
├── Timing
│
├── Authority
│
└── Verification
```

Now capability discovery can include actual feasibility.

# 61. Resource-Aware Planning

The complete planning pipeline becomes:

```text
GOAL
 ↓
WORLD MODEL
 ↓
CAPABILITY DISCOVERY
 ↓
CANDIDATE ACTIONS
 ↓
PRECONDITIONS
 ↓
RESOURCE REQUIREMENTS
 ↓
RESOURCE AVAILABILITY
 ↓
AUTHORITY
 ↓
TIME
 ↓
RISK
 ↓
PLAN
```

Only then:

```text
PLAN
 ↓
WORK
```

# 62. Resource-Aware Execution

And execution:

```text
WORK
 ↓
ADMISSION
 ↓
RESERVE
 ↓
ALLOCATE
 ↓
EXECUTE
 ↓
MONITOR
 ↓
VERIFY
 ↓
RELEASE
 ↓
ACCOUNT
```

This is the complete resource lifecycle.

# 63. Architectural Invariants

The resource subsystem should enforce:

```text
1. Resource ≠ Capability.

2. Resource ownership is explicit.

3. Capacity is not assumed to be binary.

4. Consumable resources are not "released".

5. Reservations and allocations are distinct.

6. Resource requirements can be hard or soft.

7. Resource acquisition must avoid uncontrolled deadlocks.

8. Critical resources require authority checks.

9. Resource failure invalidates affected execution assumptions.

10. Resource usage should be observable and auditable.

11. Physical resources require safe preemption semantics.

12. Resource estimates should be distinguishable from actual usage.

13. Resource budgets can exist at multiple hierarchy levels.

14. Unknown resource state must not be treated as available.
```

# 64. NROS Resource Architecture

The runtime now looks like:

```text
                       NROS
                         │
          ┌──────────────┼──────────────┐
          ↓              ↓              ↓
       SEMANTICS       TIME          STATE
          │              │              │
          └──────────────┼──────────────┘
                         ↓
                    CAPABILITIES
                         ↓
                      ACTIONS
                         ↓
                       WORK
                         ↓
                  RESOURCE MANAGER
                         ↓
                    RESERVATION
                         ↓
                     SCHEDULER
                         ↓
                     EXECUTOR
                         ↓
                 PHYSICAL SYSTEM
                         ↓
                    OBSERVATION
                         ↓
                      EVIDENCE
```

# 65. A Major Architectural Insight

At this point, NROS is converging around **three fundamental runtime dimensions**:

```text
              WHAT?
          Capabilities
             Actions
                │
                ↓
              WHEN?
               Time
                │
                ↓
              WITH WHAT?
             Resources
```

And all three are constrained by:

```text
Authority
State
Safety
```

This produces:

```text
WHAT
 ↓
WHEN
 ↓
WITH WHAT
 ↓
UNDER WHICH AUTHORITY
 ↓
GIVEN WHICH STATE
 ↓
EXECUTE
```

That is a strong candidate for the core semantic contract of NROS.

# 66. Next Layer — Policy & Authority

Resources answer:

> **What can be allocated?**

Capabilities answer:

> **What can be done?**

State answers:

> **What is believed to be true?**

Time answers:

> **When is it valid?**

But one critical question remains:

> **Who is allowed to decide or perform it?**

That brings us to:

# Part LXIX — NROS Authority, Policy & Governance Model

We will derive:

```text
Identity
Principal
Agent
Role
Capability Permission
Authority
Delegation
Lease
Policy
Constraint
Scope
Revocation
Trust
Safety Policy
Admission Policy
Execution Policy
Governance
```

and establish the security/authority chain:

```text
IDENTITY
   ↓
PRINCIPAL
   ↓
AUTHORITY
   ↓
POLICY
   ↓
CAPABILITY
   ↓
ACTION
   ↓
RESOURCE
   ↓
WORK
   ↓
EXECUTION
```

The critical architectural rule will be:

> **An agent may propose an action, but only the NROS authority/policy layer can make that action admissible for execution.**

# NROS — Part LXIX: Authority, Policy & Governance Model

We now have:

```text
Goal
 ↓
Plan
 ↓
Capability
 ↓
Action
 ↓
Resource
 ↓
Work
 ↓
Execution
```

But the runtime still needs a hard answer to:

> **Who is permitted to cause this action, under which conditions, and with whose authority?**

That is the purpose of the **Authority & Policy Model**.

# 1. Identity

Identity answers:

> **Who is this entity?**

Examples:

```text
agent:planner-01
agent:maintenance-02
operator:alice
service:motion-controller
system:safety-controller
```

Identity should be stable enough to support accountability.

# 2. Principal

A Principal is an identity that can participate in authorization.

Conceptually:

```text
Identity
   ↓
Principal
```

Not every observable entity must necessarily be a principal.

For example:

```text
TemperatureSensor
```

may have an identity without possessing authority to issue commands.

# 3. Agent

An Agent is an active decision-making entity.

It may be:

```text
Human
LLM
Autonomous Controller
Planner
Robot
Service
Supervisor
```

An Agent can act through capabilities available to it.

# 4. Authority

Authority answers:

> **What is this principal permitted to cause?**

For example:

```text
agent:inspection
```

may have:

```text
camera.capture
navigation.goto
```

but not:

```text
arm.move
```

Authority therefore constrains capability access.

# 5. Permission

A permission is a specific authorization.

Conceptually:

```text
Principal
    ↓
Permission
    ↓
Capability
```

Example:

```text
inspection-agent
    ALLOW
camera.capture
```

# 6. Permission Is Not Capability

This distinction is fundamental:

```text
Capability:
    arm.move
```

means:

> The system can perform arm movement.

Permission:

```text
inspection-agent
    DENY arm.move
```

means:

> This principal cannot invoke it.

So:

```text
Capability ≠ Permission
```

# 7. Authorization

Authorization evaluates:

```text
Can principal P perform action A
using capability C
on resource R
under state S
at time T?
```

This is much richer than:

```text
user == admin
```

# 8. Authorization Context

A policy decision may depend on:

```text
Principal
Capability
Action
Resource
State
Time
Location
Mission
Risk
Trust
Emergency status
```

Conceptually:

```text
AuthorizationContext {
    principal
    action
    capability
    resources
    state
    time
    mission
    risk
}
```

# 9. Policy

A Policy defines rules governing behavior.

Example:

```text
Policy:
    Only maintenance agents may move the elevator
    when maintenance mode is active.
```

This becomes a machine-evaluable constraint.

# 10. Policy Is Broader Than Permission

Permission:

```text
ALLOW arm.move
```

Policy:

```text
ALLOW arm.move
IF
    principal.role == maintenance
    AND maintenance_mode == true
    AND emergency_stop == false
```

Policy adds context.

# 11. Policy Layers

NROS should support multiple policy layers:

```text
System Policy
    ↓
Safety Policy
    ↓
Mission Policy
    ↓
Agent Policy
    ↓
Action Policy
```

Higher-level policies should constrain lower-level decisions.

# 12. Policy Precedence

Suppose:

```text
Mission Policy:
    ALLOW arm.move
```

but:

```text
Safety Policy:
    DENY arm.move
```

The result must be:

```text
DENY
```

Safety constraints cannot be overridden by ordinary mission intent.

# 13. Deny-by-Default

A strong baseline:

```text
No explicit authority
        ↓
      DENY
```

This prevents accidental capability escalation.

# 14. Explicit Allow

An action becomes admissible only when required conditions are satisfied:

```text
Identity
 +
Capability
 +
Policy
 +
Context
 =
Authorized
```

Authorization should not be inferred from capability existence.

# 15. Scope

Authority should have scope.

Example:

```text
ALLOW:
    navigation.goto
```

is broader than:

```text
ALLOW:
    navigation.goto
    WITHIN ZoneA
```

Scope may include:

```text
Capability
Resource
Geographic Area
Mission
Time
Quantity
Risk
```

# 16. Temporal Authority

Authority may expire.

Example:

```text
maintenance-agent
```

has permission:

```text
arm.move
```

only during:

```text
08:00 → 18:00
```

After expiration:

```text
DENY
```

This connects authority directly to the Time Model.

# 17. Quantity Limits

Authority can constrain quantities.

Example:

```text
ALLOW:
    charge_battery
```

but:

```text
maximum:
    10 kWh
```

This prevents unlimited resource consumption.

# 18. Rate Limits

Authority may also specify:

```text
max_actions_per_minute
```

For example:

```text
camera.capture
≤ 100 / minute
```

This protects resources and prevents runaway agents.

# 19. Delegation

A principal may delegate authority.

Example:

```text
Supervisor
    ↓
delegates
    ↓
Maintenance Agent
```

But delegation should preserve constraints.

If:

```text
Supervisor:
    arm.move
```

delegates to:

```text
Agent B
```

it does not necessarily follow that Agent B can delegate unlimited authority onward.

# 20. Delegation Chain

Represent:

```text
Root Authority
      ↓
Supervisor
      ↓
Agent A
      ↓
Agent B
```

The runtime should be able to trace:

```text
"Why is Agent B authorized?"
```

Answer:

```text
B ← A ← Supervisor ← Root Authority
```

# 21. Delegation Attenuation

Delegation should normally be able to **reduce** authority, not expand it.

Example:

```text
Supervisor:
    arm.move
    ZoneA + ZoneB
```

delegates:

```text
Agent:
    arm.move
    ZoneA only
```

Agent cannot transform this into:

```text
arm.move
ZoneA + ZoneB + ZoneC
```

# 22. Delegation Constraints

A delegation may specify:

```text
scope
expiration
maximum uses
resource limits
risk ceiling
subdelegation
```

Example:

```text
delegate:
    camera.capture

scope:
    Machine42

expires:
    12:00

max_calls:
    20

subdelegate:
    false
```

# 23. Revocation

Authority must be revocable.

```text
AUTHORIZED
   ↓
REVOKED
```

Revocation can occur because:

```text
operator action
policy change
agent compromise
lease expiration
safety event
mission termination
```

# 24. Revocation vs Active Execution

A difficult question:

> What happens to an action already running when authority disappears?

Possible policies:

```text
CONTINUE
SAFE_STOP
ABORT
COMPLETE_CURRENT_ATOMIC_STEP
```

This cannot be left implicit.

# 25. Safety-Critical Revocation

For safety-critical operations:

```text
Authority revoked
       ↓
Safe termination
```

may be mandatory.

For example:

```text
robot arm
   ↓
authority revoked
   ↓
controlled stop
```

rather than uncontrolled process termination.

# 26. Trust

Authority is not necessarily equivalent to trust.

An agent may be:

```text
authenticated
```

but still:

```text
untrusted for physical execution
```

Therefore:

```text
Identity
 ≠
Trust
 ≠
Authority
```

These are separate concepts.

# 27. Trust Level

An entity may have a trust classification:

```text
UNTRUSTED
OBSERVED
VERIFIED
TRUSTED
CRITICAL
```

But trust should not silently grant permissions.

Instead:

```text
Trust
 ↓
Policy Evaluation
 ↓
Authorization
```

# 28. Evidence-Based Trust

Trust can be supported by evidence:

```text
Agent
 ↓
execution history
 ↓
verification results
 ↓
audit evidence
 ↓
trust assessment
```

This connects authority with NROS's evidence model.

# 29. Policy Conditions

Policies can inspect state:

```text
ALLOW arm.move
IF:
    emergency_stop == false
```

or:

```text
ALLOW elevator.motion
IF:
    maintenance_mode == true
```

This means policy evaluation depends on the State Model.

# 30. Stale State Must Not Authorize Dangerous Actions

Suppose:

```text
emergency_stop == false
```

but that observation is 30 seconds old.

For a safety-critical action, it may be invalid.

Therefore:

```text
Policy
 +
State Freshness
 +
State Confidence
```

must be considered together.

# 31. Policy Guard

A policy condition can therefore include:

```text
condition
required_confidence
maximum_age
required_source
```

Example:

```text
emergency_stop == false

max_age:
    100 ms

source:
    safety-controller
```

This creates a strong safety boundary.

# 32. Policy Categories

NROS can classify policies:

```text
SAFETY
SECURITY
RESOURCE
MISSION
OPERATIONAL
PRIVACY
COMPLIANCE
GOVERNANCE
```

Each category can have different precedence.

# 33. Safety Policy

Safety policy governs:

```text
physical hazards
emergency stops
operating envelopes
interlocks
unsafe states
```

Example:

```text
DENY motor.move
IF
    guard_door.open == true
```

# 34. Security Policy

Security policy governs:

```text
identity
authentication
authorization
delegation
secrets
network access
```

Example:

```text
DENY shell.execute
IF
    principal != trusted-admin
```

# 35. Resource Policy

Resource policy controls:

```text
quotas
allocation
reservation
priority
preemption
```

Example:

```text
Agent A
≤ 30% CPU
```

# 36. Mission Policy

Mission policy describes operational intent:

```text
Mission:
    inspect elevator

Allowed:
    navigation
    camera
    diagnostics

Forbidden:
    configuration mutation
```

This constrains autonomous behavior.

# 37. Governance Policy

Governance answers:

```text
Who may change policies?
Who may delegate authority?
Who may approve new capabilities?
Who may modify safety constraints?
```

This is the control plane for the control plane.

# 38. Policy Administration

Not every agent should be able to modify policies.

A hierarchy might be:

```text
Operator
   ↓
Governance Authority
   ↓
Policy Manager
   ↓
Runtime
```

The runtime consumes policies but does not arbitrarily rewrite them.

# 39. Policy Versioning

Policies should be immutable versions:

```text
policy-v17
policy-v18
policy-v19
```

A decision should record:

```text
policy_version = v18
```

This makes authorization decisions reproducible.

# 40. Policy Decision Record

Every significant authorization decision should produce evidence:

```text
Decision {
    principal
    action
    capability
    resources
    result
    policy_version
    state_revision
    timestamp
    reason
}
```

Example:

```text
DENY

reason:
    emergency_stop_active

policy:
    safety-v42
```

# 41. Explainability

An agent should be able to ask:

> Why was my action denied?

The runtime should answer with structured information:

```text
DENIED
 ├── capability: arm.move
 ├── policy: safety-v42
 ├── condition: emergency_stop == false
 ├── observed: true
 └── state_revision: 8172
```

This is much better than:

```text
Permission denied.
```

# 42. Admission Decision

Authorization should produce a structured decision:

```text
ALLOW
DENY
ALLOW_WITH_CONSTRAINTS
DEFER
REQUIRE_APPROVAL
```

Not every action needs a binary result.

# 43. Require Approval

For sensitive actions:

```text
AI Agent
   ↓
Action Proposal
   ↓
Policy
   ↓
Human Approval Required
   ↓
Approved
   ↓
Execute
```

This supports human-in-the-loop operation.

# 44. Approval as Authority

An approval should itself be represented as a signed/traceable authority artifact:

```text
Approval {
    approver
    action
    scope
    expiration
    constraints
    timestamp
}
```

Then the runtime can verify it.

# 45. Four-Eyes Principle

Some actions may require two independent authorities:

```text
Operator A
     +
Operator B
     ↓
Authorized
```

This is appropriate for highly consequential operations.

# 46. Separation of Duties

NROS can enforce:

```text
Requester ≠ Approver
```

For example:

```text
Agent A requests
Operator B approves
```

rather than allowing the same principal to authorize itself.

# 47. Policy Composition

Policies should compose rather than overwrite each other blindly.

Conceptually:

```text
System Policy
    ∩
Safety Policy
    ∩
Mission Policy
    ∩
Agent Policy
```

The effective authority is the intersection of constraints.

# 48. Policy Monotonicity

A useful invariant:

> A lower-trust layer must not broaden authority granted by a higher-trust layer.

Therefore:

```text
EffectiveAuthority
    ⊆
ParentAuthority
```

This prevents accidental privilege escalation.

# 49. Capability Escalation

An agent must not be able to transform:

```text
camera.capture
```

into:

```text
shell.execute
```

merely by constructing a different Action.

Capability boundaries must be enforced at runtime.

# 50. Action Construction Is Untrusted

An AI planner may produce:

```json
{
  "capability": "arm.move",
  "target": "unsafe-zone"
}
```

NROS should treat this as:

```text
UNTRUSTED PROPOSAL
```

Then:

```text
schema validation
 ↓
capability lookup
 ↓
policy evaluation
 ↓
resource admission
 ↓
safety validation
```

Only afterward can it become executable Work.

# 51. No Authority Through Argument Injection

An action's arguments must never be able to modify authorization.

For example:

```text
action:
    arm.move

arguments:
    role = admin
```

must not affect:

```text
principal.role
```

Identity and authority are external runtime facts, not action parameters.

# 52. Capability Tokens

NROS may represent delegated authority using capability-like tokens:

```text
Token {
    principal
    capability
    scope
    constraints
    expiration
}
```

But tokens must remain bounded by the root policy.

# 53. Token Lifetime

Short-lived authority is preferable for dynamic agents:

```text
token
 ↓
valid for 60 seconds
 ↓
expires
```

This limits the damage of compromised credentials.

# 54. Lease + Authority

Authority and resource leases can interact:

```text
Authority Lease
      +
Resource Lease
      ↓
Executable Window
```

If either expires:

```text
execution must be reevaluated
```

# 55. Authority + Resource + State

An action is executable only if all three hold:

```text
Authority
   AND
Resource Availability
   AND
State Preconditions
```

For time-sensitive execution:

```text
Authority
 AND Resource
 AND State
 AND Time
```

This gives us the core admissibility predicate.

# 56. NROS Admissibility

Conceptually:

```text
Admissible(Action) =
    Authorized(Action)
    ∧ PreconditionsSatisfied(Action)
    ∧ ResourcesAvailable(Action)
    ∧ TemporalConstraintsSatisfied(Action)
    ∧ SafetyConstraintsSatisfied(Action)
```

Only if:

```text
Admissible(Action) == true
```

may Work transition into an executable state.

# 57. Critical Invariant

This should become a formal NROS invariant:

> **No action may enter physical execution solely because an agent requested it.**

Instead:

```text
REQUEST
 ↓
VALIDATE
 ↓
AUTHORIZE
 ↓
ADMIT
 ↓
EXECUTE
```

# 58. Governance Boundary

We now have two fundamentally different planes.

### Control Plane

```text
Identity
Policy
Authority
Governance
Capability Registry
Resource Registry
```

### Execution Plane

```text
Action
Work
Scheduler
Resource Allocation
Executor
Physical Interfaces
```

The execution plane should consume decisions produced by the control plane.

# 59. Why This Separation Matters

An autonomous agent can operate inside:

```text
execution environment
```

without being able to modify:

```text
authority policy
```

This is essential for safe autonomy.

# 60. Governance Graph

The authority system can be represented as:

```text
ROOT AUTHORITY
      │
      ├── Governance
      │      │
      │      └── Policies
      │
      ├── Supervisors
      │      │
      │      └── Delegations
      │
      └── Agents
             │
             └── Action Requests
```

Every execution should be traceable back through this graph.

# 61. Complete Authority Chain

The final chain becomes:

```text
Identity
   ↓
Principal
   ↓
Trust Context
   ↓
Authority
   ↓
Delegation
   ↓
Policy
   ↓
Capability Permission
   ↓
Action
   ↓
Resource Permission
   ↓
Admission
   ↓
Execution
```

# 62. Auditability

For every consequential action, NROS should be able to answer:

```text
WHO?
WHAT?
WHY?
UNDER WHICH POLICY?
WITH WHICH AUTHORITY?
USING WHICH RESOURCES?
BASED ON WHICH STATE?
AT WHAT TIME?
WHAT HAPPENED?
WAS IT VERIFIED?
```

This is the beginning of a complete execution provenance model.

# 63. Provenance Record

Conceptually:

```text
ExecutionProvenance {
    principal
    delegation_chain
    policy_versions
    capability
    action
    resource_allocations
    state_revision
    timestamps
    execution_result
    verification_result
}
```

This connects:

```text
Authority
+
State
+
Resources
+
Execution
+
Evidence
```

into one trace.

# 64. Architectural Invariants

The authority subsystem should enforce:

```text
1. Identity ≠ Trust ≠ Authority.

2. Capability existence does not imply permission.

3. Authorization is contextual.

4. Authority is scoped.

5. Delegation cannot silently expand authority.

6. Authority can expire and be revoked.

7. Safety policies dominate ordinary mission policies.

8. Unknown or stale state cannot automatically authorize critical actions.

9. Planner output is untrusted.

10. Agents cannot grant themselves authority through action arguments.

11. Policy changes are versioned.

12. Significant authorization decisions are auditable.

13. The requester need not be the approver.

14. Execution requires successful admission, not merely authorization.
```

# 65. NROS Admissibility Architecture

We can now formulate the central gate:

```text
                    ACTION
                       │
                       ↓
              ┌────────────────┐
              │ Schema Valid?  │
              └───────┬────────┘
                      YES
                       ↓
              ┌────────────────┐
              │ Capability?    │
              └───────┬────────┘
                      YES
                       ↓
              ┌────────────────┐
              │ Authorized?    │
              └───────┬────────┘
                      YES
                       ↓
              ┌────────────────┐
              │ Preconditions? │
              └───────┬────────┘
                      YES
                       ↓
              ┌────────────────┐
              │ Resources?     │
              └───────┬────────┘
                      YES
                       ↓
              ┌────────────────┐
              │ Time valid?    │
              └───────┬────────┘
                      YES
                       ↓
              ┌────────────────┐
              │ Safety valid?  │
              └───────┬────────┘
                      YES
                       ↓
                   ADMITTED
                       ↓
                     WORK
```

Any failed gate must prevent ordinary execution.

# 66. The Emerging NROS Core

At this point the architecture can be summarized as:

```text
                    ┌───────────┐
                    │   GOALS   │
                    └─────┬─────┘
                          ↓
                    ┌───────────┐
                    │   PLANS   │
                    └─────┬─────┘
                          ↓
                    ┌───────────┐
                    │ ACTIONS   │
                    └─────┬─────┘
                          ↓
              ┌───────────┴───────────┐
              ↓                       ↓
        CAPABILITY                AUTHORITY
              │                       │
              └───────────┬───────────┘
                          ↓
                    ┌───────────┐
                    │ ADMISSION │
                    └─────┬─────┘
                          ↓
                    ┌───────────┐
                    │ RESOURCES │
                    └─────┬─────┘
                          ↓
                    ┌───────────┐
                    │   WORK    │
                    └─────┬─────┘
                          ↓
                    ┌───────────┐
                    │ EXECUTION │
                    └─────┬─────┘
                          ↓
                    ┌───────────┐
                    │ OBSERVE   │
                    └─────┬─────┘
                          ↓
                    ┌───────────┐
                    │ VERIFY    │
                    └─────┬─────┘
                          ↓
                    ┌───────────┐
                    │  STATE    │
                    └───────────┘
```

# 67. Next Missing Layer — Observation & Evidence

We now have a complete path for **causing** change.

But autonomous systems cannot safely operate only by issuing actions.

They need to establish:

> **What actually happened?**

That requires the next major subsystem:

# Part LXX — NROS Observation, Evidence & Verification Model

We will define:

```text
Observation
Sensor
Event
Evidence
Source
Timestamp
Freshness
Confidence
Provenance
Measurement
Inference
Fact
Belief
Verification
Contradiction
Staleness
Evidence Chain
Execution Receipt
Outcome
```

and build the loop:

```text
OBSERVE
   ↓
INGEST
   ↓
NORMALIZE
   ↓
ATTRIBUTE
   ↓
CORRELATE
   ↓
UPDATE STATE
   ↓
VERIFY EFFECT
   ↓
GENERATE EVIDENCE
   ↓
REFLECT
   ↓
PLAN AGAIN
```

This is where NROS's **Observe → Plan → Execute → Reflect → Checkpoint** architecture becomes a closed-loop runtime rather than merely an action executor.

# NROS — Part LXX: Observation, Evidence & Verification Model

The previous layers established how NROS decides **what may happen**.

Now we need the complementary question:

> **How does NROS know what actually happened?**

This distinction is critical.

An agent saying:

```text
"motor.move succeeded"
```

is not itself evidence that the motor moved.

NROS therefore needs a first-class **Observation → Evidence → Verification** subsystem.

# 1. Observation

An Observation is an externally or internally produced representation of something detected.

Examples:

```text
temperature = 72°C
position = 14.2°
motor.current = 3.4A
door = CLOSED
command_completed = true
network_peer = reachable
```

An Observation says:

> **Something was observed.**

It does not automatically say:

> **The underlying fact is permanently true.**

# 2. Observation Source

Every observation should identify its source.

Examples:

```text
sensor:temperature-01
controller:motion-02
agent:planner-01
operator:console
kernel:runtime
external:api
```

This gives us:

```text
Observation
    ↓
Source
```

# 3. Source Trust

Different sources have different semantics.

For example:

```text
SafetyController
```

may be authoritative for:

```text
emergency_stop
```

while:

```text
VisionAgent
```

may only provide an inference.

Therefore:

```text
source identity
≠
source authority
```

The State Model must preserve this distinction.

# 4. Observation Timestamp

An observation needs temporal metadata.

At minimum:

```text
observed_at
received_at
```

These are not necessarily identical.

Example:

```text
sensor observed:
10:00:00.100

runtime received:
10:00:00.350
```

Network latency is therefore measurable.

# 5. Event Time vs Processing Time

NROS should distinguish:

```text
EVENT TIME
```

from:

```text
PROCESSING TIME
```

This becomes important in distributed systems.

```text
Sensor
  │
  │ event @ T1
  ↓
Network
  │
  │ delay
  ↓
NROS
  │
  │ processed @ T2
```

Where:

```text
T1 != T2
```

# 6. Observation Identity

Every observation should have an identifier:

```text
ObservationId
```

This allows later evidence to refer precisely to the observation.

Example:

```text
obs-8f31...
```

rather than relying on timestamps alone.

# 7. Observation Payload

Conceptually:

```rust
struct Observation {
    id: ObservationId,
    source: SourceId,
    observed_at: Time,
    received_at: Time,
    subject: EntityId,
    predicate: Predicate,
    value: Value,
    confidence: Confidence,
}
```

The exact Rust API can evolve later.

# 8. Observation ≠ Fact

This distinction is fundamental.

Observation:

```text
Camera A observed:
"door appears closed"
```

Fact:

```text
Door is closed.
```

The first is evidence.

The second is a state assertion.

NROS should not collapse the two.

# 9. Observation → State

The State Engine can transform observations into state updates:

```text
Observation
    ↓
Interpretation
    ↓
State Candidate
    ↓
Validation
    ↓
State Update
```

This prevents raw sensor data from becoming trusted state automatically.

# 10. Confidence

Observations may have confidence.

Example:

```text
camera:
door_closed = true
confidence = 0.82
```

while:

```text
door_sensor:
door_closed = true
confidence = 0.99
```

The State Engine can combine them.

# 11. Confidence Is Not Truth

A high confidence observation can still be wrong.

Therefore:

```text
confidence ≠ certainty
```

Confidence is metadata about belief strength.

# 12. Evidence

Evidence is information that supports or contradicts a proposition.

Examples:

```text
sensor reading
execution receipt
log record
test result
operator confirmation
cryptographic attestation
measurement
external observation
```

Evidence can therefore come from many sources.

# 13. Evidence Object

Conceptually:

```text
Evidence {
    id
    source
    timestamp
    subject
    claim
    observation
    provenance
    confidence
}
```

The important field is:

```text
claim
```

What proposition does this evidence support?

# 14. Claim

A Claim represents a proposition.

Example:

```text
Claim:
    elevator.door == CLOSED
```

Evidence may support:

```text
Claim
 ↑
 ├── DoorSensor
 ├── Camera
 └── ControllerState
```

# 15. Supporting Evidence

Multiple independent observations can support one claim.

```text
                 CLAIM
          door == CLOSED
             /    |    \
            /     |     \
       Sensor   Camera  Controller
```

This enables evidence aggregation.

# 16. Contradictory Evidence

Suppose:

```text
DoorSensor:
    CLOSED
```

but:

```text
Camera:
    OPEN
```

NROS must not silently choose one.

Instead:

```text
CLAIM CONFLICT
```

should become explicit state.

# 17. Contradiction

Contradiction is itself meaningful information.

```text
Observation A
      +
Observation B
      ↓
  CONTRADICTION
```

This may trigger:

```text
re-observation
diagnostics
safe mode
human escalation
replanning
```

depending on policy.

# 18. Evidence Freshness

Evidence becomes stale.

Example:

```text
door_closed = true
observed 30 seconds ago
```

may be acceptable for a log but unacceptable for:

```text
unlock_door
```

Therefore evidence should have:

```text
observed_at
valid_until
max_age
```

or equivalent freshness semantics.

# 19. Freshness Is Contextual

There is no universal freshness threshold.

For:

```text
ambient_temperature
```

one minute may be fine.

For:

```text
motor_position
```

one minute may be useless.

For:

```text
emergency_stop
```

milliseconds may matter.

Freshness therefore belongs partly to policy.

# 20. Provenance

Evidence must answer:

> **Where did this information come from?**

Example:

```text
Observation
 ↓
Sensor
 ↓
Gateway
 ↓
NROS
 ↓
State Engine
```

Provenance records this chain.

# 21. Provenance Graph

Instead of a flat log:

```text
event → event → event
```

NROS can maintain:

```text
                Observation
                     │
                     ↓
                 Inference
                     │
                     ↓
                   Claim
                     │
                     ↓
                State Update
                     │
                     ↓
               Policy Decision
```

This makes reasoning traceable.

# 22. Raw Evidence Must Be Preserved

A critical principle:

> **Do not preserve only the interpreted state. Preserve enough raw evidence to reconstruct the interpretation.**

For example:

```text
State:
    door = CLOSED
```

should ideally be traceable to:

```text
sensor reading
timestamp
source
decoder/version
```

# 23. Decoder Version

Interpretation logic changes.

Suppose:

```text
sensor payload:
0x01
```

was interpreted differently under:

```text
decoder-v1
decoder-v2
```

Therefore evidence should record:

```text
decoder_version
```

where relevant.

# 24. Observation Normalization

Different sources may represent the same concept differently:

```text
"closed"
1
true
CLOSED
0x01
```

NROS can normalize these into a canonical semantic representation.

```text
Raw Observation
       ↓
Normalization
       ↓
Canonical Observation
```

# 25. Normalization Must Be Traceable

Normalization should not destroy the original representation.

Store:

```text
raw_value
normalized_value
normalizer_version
```

This enables reconstruction.

# 26. Measurement

Measurements require units.

Example:

```text
temperature = 72
```

is incomplete.

Prefer:

```text
temperature = 72 °C
```

NROS should therefore support:

```text
quantity
unit
precision
uncertainty
```

# 27. Measurement Uncertainty

A sensor might produce:

```text
72.0°C ± 0.5°C
```

This is richer than:

```text
72°C
```

Uncertainty can affect policy decisions.

# 28. Sensor Quality

Observations can include quality metadata:

```text
signal_strength
calibration_state
error_margin
resolution
health
```

A degraded sensor should not necessarily produce equally trusted evidence.

# 29. Derived Observation

Not all observations come directly from sensors.

Example:

```text
Camera
 ↓
Object Detector
 ↓
"person_detected = true"
```

This is a **derived observation**.

Its provenance should include:

```text
source observation
model
model version
inference timestamp
```

# 30. Inference

NROS should distinguish:

```text
OBSERVED
```

from:

```text
INFERRED
```

Example:

```text
Observed:
    wheel_speed = 0

Inferred:
    robot_is_stationary = true
```

The inference may be reasonable, but it is not directly observed.

# 31. Belief

A higher-level reasoning system may maintain:

```text
Belief:
    obstacle probably exists
```

This is even further removed from raw observation.

A useful semantic hierarchy is:

```text
Raw Observation
       ↓
Derived Observation
       ↓
Fact Candidate
       ↓
Belief
       ↓
Plan Assumption
```

Each transition should preserve provenance.

# 32. Verification

Verification answers:

> **Did the expected effect actually occur?**

Suppose an action was:

```text
arm.move(target=30°)
```

Command acceptance is not verification.

Verification might require:

```text
position_sensor == 30°
```

# 33. Command Receipt ≠ Effect Verification

This distinction is essential.

```text
Command sent
     ↓
Controller accepted
     ↓
Controller executed
     ↓
Physical state changed
     ↓
Observed
     ↓
Verified
```

These are separate events.

# 34. Execution Receipt

The executor can produce:

```text
ExecutionReceipt {
    work_id
    accepted_at
    started_at
    completed_at
    result
}
```

This proves runtime processing.

It does **not necessarily prove physical success**.

# 35. Effect Evidence

Physical verification produces:

```text
EffectEvidence
```

Example:

```text
Expected:
    door = CLOSED

Observed:
    door_sensor = CLOSED

timestamp:
    T+200ms
```

Now the runtime can classify the action as verified.

# 36. Verification Levels

A useful hierarchy:

```text
LEVEL 0
Command Created

LEVEL 1
Command Accepted

LEVEL 2
Execution Completed

LEVEL 3
Expected Effect Observed

LEVEL 4
Effect Independently Verified

LEVEL 5
Postcondition Confirmed
```

This avoids treating all "success" values equally.

# 37. Independent Verification

For critical actions, the same subsystem that performed the action should not necessarily be the sole verifier.

Example:

```text
MotorController
    performs movement
```

while:

```text
PositionSensor
    verifies movement
```

This provides stronger assurance.

# 38. Verification Policy

Different actions can require different verification levels.

Routine:

```text
LEVEL 2
```

Safety-critical:

```text
LEVEL 4+
```

Mission-critical:

```text
LEVEL 5
```

The requirement should be declared by policy/capability semantics.

# 39. Postconditions

An Action can declare:

```text
preconditions
effects
postconditions
```

Example:

```text
Action:
    door.close

Postcondition:
    door == CLOSED
```

Verification then becomes:

```text
Action
 ↓
Expected Postcondition
 ↓
Observation
 ↓
Verification
```

# 40. Verification Failure

If:

```text
Expected:
    door == CLOSED
```

but:

```text
Observed:
    door == OPEN
```

then:

```text
VERIFICATION_FAILED
```

The Work should not be labeled simply:

```text
SUCCESS
```

# 41. Partial Success

Some operations can produce partial results.

Example:

```text
Target:
    100 units

Actual:
    70 units
```

NROS should support:

```text
SUCCESS
PARTIAL_SUCCESS
FAILED
UNKNOWN
```

rather than binary success/failure only.

# 42. Unknown Outcome

Suppose communication is lost immediately after command execution.

NROS cannot conclude:

```text
FAILED
```

nor:

```text
SUCCESS
```

Correct state may be:

```text
OUTCOME_UNKNOWN
```

This is extremely important for distributed systems.

# 43. Unknown Must Not Become Success

A strong invariant:

```text
UNKNOWN ≠ SUCCESS
```

And for safety:

```text
UNKNOWN
```

should often be treated conservatively.

# 44. Reconciliation

When an outcome is unknown:

```text
UNKNOWN
   ↓
RECONCILIATION
   ↓
observe system
   ↓
compare expected state
   ↓
RESOLVED
```

This prevents unnecessary duplicate actions.

# 45. Duplicate Execution Hazard

Suppose:

```text
Command: open_valve
```

is sent.

Network fails.

The agent does not know whether it succeeded.

It retries:

```text
open_valve
```

The valve may already be open.

Therefore actions need:

```text
idempotency
```

or:

```text
deduplication
```

semantics where appropriate.

# 46. Work Identity

Every execution attempt should have a stable identity:

```text
WorkId
```

and possibly:

```text
AttemptId
```

Then duplicate messages can be recognized.

```text
Work W42
 ├── Attempt 1
 └── Attempt 2
```

# 47. Event Log

Observation and execution evidence naturally form an event stream:

```text
Event
 ├── Observation
 ├── ActionRequested
 ├── ActionAdmitted
 ├── ResourceReserved
 ├── WorkStarted
 ├── WorkCompleted
 ├── Verification
 └── StateChanged
```

This gives NROS a coherent temporal history.

# 48. Event Immutability

Historical events should generally be append-only.

Instead of:

```text
modify old event
```

use:

```text
new corrective event
```

Example:

```text
Observation:
    door=CLOSED

Correction:
    previous_observation_invalidated
```

This preserves history.

# 49. Event Ordering

Distributed systems may receive events out of order.

Therefore:

```text
received_at
```

cannot always determine causal order.

NROS may need:

```text
sequence_number
logical_clock
causal_parent
source_sequence
```

depending on subsystem requirements.

# 50. Causality

A useful relation:

```text
Action
  ↓ causes
Observation
  ↓ updates
State
  ↓ triggers
Policy
  ↓ creates
Action
```

This creates the closed control loop:

```text
ACT
 ↓
OBSERVE
 ↓
UPDATE
 ↓
DECIDE
 ↓
ACT
```

# 51. Observation-Action Correlation

An Observation should be correlatable with Work when possible.

Example:

```text
Work W42
    ↓
arm.move
    ↓
Observation O91
    ↓
position=30°
```

This enables direct verification.

# 52. Correlation IDs

Use:

```text
work_id
action_id
attempt_id
correlation_id
```

to connect distributed events.

Example:

```text
Request
  correlation_id = C123

Controller event
  correlation_id = C123

Sensor observation
  correlation_id = C123
```

# 53. Evidence Chain

A verified execution can then produce:

```text
Goal
 ↓
Plan
 ↓
Action
 ↓
Work
 ↓
Execution
 ↓
Observation
 ↓
Postcondition
 ↓
Verification
```

This is the central provenance chain of NROS.

# 54. Checkpoint

NROS's checkpoint mechanism should capture enough information to resume reasoning:

```text
Checkpoint {
    state_revision
    active_work
    resource_allocations
    authority_context
    pending_verifications
    observations
    evidence
}
```

A checkpoint is therefore more than:

```text
save variables
```

It is a semantic execution snapshot.

# 55. Recovery

After restart:

```text
Checkpoint
    ↓
Restore State
    ↓
Reconcile Resources
    ↓
Revalidate Authority
    ↓
Revalidate Time
    ↓
Resolve Unknown Work
    ↓
Resume / Replan
```

This is critical for autonomous runtimes.

# 56. Never Blindly Resume

A restored checkpoint may contain stale information.

Therefore:

```text
checkpoint says:
    motor=STOPPED
```

does not automatically mean:

```text
motor is currently stopped
```

The runtime may need fresh observation.

# 57. Evidence Freshness After Recovery

On restart:

```text
old observation
```

should be classified according to its freshness policy.

Potentially:

```text
VALID
STALE
INVALID
UNKNOWN
```

This prevents stale checkpoint state from becoming false certainty.

# 58. Evidence Store

NROS can expose a logical Evidence Store:

```text
EvidenceStore
├── observations
├── events
├── claims
├── verifications
├── provenance
├── contradictions
└── checkpoints
```

The storage implementation can vary.

# 59. Evidence Query

Agents and policies may need queries such as:

```text
"Show evidence that door is closed."
```

Result:

```text
Claim:
    door == CLOSED

Evidence:
    DoorSensor @ T1
    Controller @ T2

Verification:
    confirmed
```

This makes evidence a usable runtime primitive.

# 60. Evidence-Based Planning

Planning can now consider evidence quality:

```text
Goal
 ↓
State
 ↓
Evidence confidence
 ↓
Action selection
```

If state confidence is low:

```text
OBSERVE FIRST
```

rather than immediately acting.

# 61. Information-Gathering Actions

Observation itself can be an Action.

Example:

```text
camera.inspect
sensor.read
diagnostic.run
```

These actions consume resources but improve state knowledge.

Therefore planning becomes:

```text
ACT
```

or:

```text
OBSERVE
```

depending on uncertainty.

# 62. Value of Information

A planner may ask:

> Is it worth spending 2 seconds and 5 Wh to reduce uncertainty?

This introduces:

```text
information cost
```

and:

```text
information value
```

into planning.

That is a powerful direction for agentic runtime design.

# 63. Epistemic State

NROS can distinguish:

```text
KNOWN_TRUE
KNOWN_FALSE
PROBABLE
CONTRADICTED
STALE
UNKNOWN
```

This is more expressive than ordinary boolean state.

# 64. State Transition

The state machine can therefore become:

```text
UNKNOWN
   ↓ observation
KNOWN
   ↓ contradiction
CONFLICTED
   ↓ resolution
KNOWN
```

or:

```text
KNOWN
   ↓ time
STALE
   ↓ refresh
KNOWN
```

# 65. Evidence Policy

Evidence itself can be governed by policy.

For example:

```text
Critical Action:
    requires 2 independent observations
```

while:

```text
Routine Action:
    requires 1 observation
```

This links evidence directly to the Authority/Safety layer.

# 66. Verification as a Runtime Gate

The architecture now supports:

```text
Action
 ↓
Admission
 ↓
Execution
 ↓
Expected Effect
 ↓
Observation
 ↓
Verification
 ↓
Completion
```

So:

> **Execution completion and Work completion are not necessarily the same event.**

A Work may remain:

```text
EXECUTED_AWAITING_VERIFICATION
```

until its postconditions are established.

# 67. Strong Work Lifecycle

The lifecycle can now become:

```text
PROPOSED
   ↓
VALIDATED
   ↓
AUTHORIZED
   ↓
ADMITTED
   ↓
RESERVED
   ↓
READY
   ↓
RUNNING
   ↓
EXECUTED
   ↓
VERIFYING
   ↓
VERIFIED
   ↓
COMPLETED
```

Alternative paths:

```text
REJECTED
BLOCKED
CANCELLED
FAILED
PARTIAL
UNKNOWN
```

# 68. Reflect

Now the "Reflect" stage becomes meaningful.

Reflection consumes:

```text
Execution Result
+
Observations
+
Evidence
+
Verification
+
Resource Usage
+
Policy Decisions
```

and produces:

```text
Assessment
```

Example:

```text
Expected:
    position=30°

Observed:
    position=29.7°

Assessment:
    within tolerance
```

# 69. Learning Without Losing Determinism

NROS can improve estimates from evidence while preserving deterministic execution semantics.

For example:

```text
Historical Evidence
 ↓
Updated Estimate
 ↓
Planner
```

but the actual runtime decision remains governed by explicit:

```text
policy
constraints
state
resources
authority
```

This separates adaptive intelligence from safety-critical execution semantics.

# 70. Complete Closed Loop

We can now finally express the core loop:

```text
┌───────────────┐
│    OBSERVE    │
└───────┬───────┘
        ↓
┌───────────────┐
│  UPDATE STATE │
└───────┬───────┘
        ↓
┌───────────────┐
│     PLAN      │
└───────┬───────┘
        ↓
┌───────────────┐
│  AUTHORIZE    │
└───────┬───────┘
        ↓
┌───────────────┐
│    ADMIT      │
└───────┬───────┘
        ↓
┌───────────────┐
│    EXECUTE    │
└───────┬───────┘
        ↓
┌───────────────┐
│   VERIFY      │
└───────┬───────┘
        ↓
┌───────────────┐
│    REFLECT    │
└───────┬───────┘
        ↓
┌───────────────┐
│  CHECKPOINT   │
└───────┬───────┘
        │
        └──────────────→ OBSERVE
```

This is now a genuine **closed-loop autonomous runtime model**.

# 71. Core NROS Semantic Equation

The architecture can be summarized by:

```text
Executable Work
=
Intent
∧ Capability
∧ Authority
∧ Valid State
∧ Resource Availability
∧ Temporal Validity
∧ Safety
```

And completed Work additionally requires:

```text
Verified Outcome
```

Therefore:

```text
Execution
≠
Successful Completion
```

until verification succeeds.

# 72. Fundamental Invariants

The Observation/Evidence subsystem should enforce:

```text
1. Observation ≠ Fact.

2. Evidence must have provenance.

3. Event time ≠ processing time.

4. Confidence ≠ certainty.

5. Derived observations retain their source provenance.

6. Contradictory evidence must remain visible.

7. Unknown outcome must not become success.

8. Execution receipt does not prove physical effect.

9. Critical actions may require independent verification.

10. Evidence freshness is policy-dependent.

11. Historical evidence should be append-only/auditable.

12. State updates must remain traceable to evidence.

13. Checkpoints must not silently convert stale state into current truth.

14. Verification is a distinct runtime stage.

15. Reflection consumes evidence, not merely executor status.
```

# 73. NROS Architecture — Current Complete Form

We can now assemble the major semantic layers:

```text
                         NROS
                          │
       ┌──────────────────┼──────────────────┐
       ↓                  ↓                  ↓
    IDENTITY           GOVERNANCE         WORLD
       │                  │                  │
       ↓                  ↓                  ↓
   AUTHORITY            POLICY             STATE
       │                  │                  │
       └──────────────┬───┴──────────────────┘
                      ↓
                 CAPABILITIES
                      ↓
                    ACTIONS
                      ↓
                  RESOURCES
                      ↓
                    TIME
                      ↓
                  ADMISSION
                      ↓
                     WORK
                      ↓
                  SCHEDULER
                      ↓
                  EXECUTOR
                      ↓
                 OBSERVATION
                      ↓
                    EVIDENCE
                      ↓
                  VERIFICATION
                      ↓
                   REFLECTION
                      ↓
                  CHECKPOINT
                      │
                      └──────────→ STATE
```

# 74. What Is Still Missing?

The semantic core is becoming coherent, but several major runtime concerns remain.

The next major layer should be:

# **Part LXXI — NROS Failure, Fault, Recovery & Resilience Model**

We need to formally distinguish:

```text
Error
Failure
Fault
Degradation
Timeout
Cancellation
Crash
Partition
Lost Message
Duplicate Message
Corrupted State
Resource Failure
Authority Loss
Verification Failure
Recovery
Retry
Compensation
Rollback
Reconciliation
Safe Mode
Degraded Mode
```

Most importantly, we need to answer:

> **What does NROS do when reality stops matching the plan?**

The target architecture will be:

```text
EXPECTED
   ↓
OBSERVED
   ↓
MISMATCH
   ↓
CLASSIFY
   ↓
CONTAIN
   ↓
RECOVER / COMPENSATE / REPLAN
   ↓
VERIFY
   ↓
RESUME or TERMINATE
```

That will complete the **failure semantics** needed before we can safely turn the entire conceptual NROS model into a concrete repository architecture.
