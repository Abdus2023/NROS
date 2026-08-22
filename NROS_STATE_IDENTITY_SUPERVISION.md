# NROS State, Identity & Supervision (Part XXXI–XXXIV)

The next transition is from **execution** to **coherent system state**.

ROS gives nodes mechanisms to exchange messages and maintain parameters. NROS should go further:

> **System state should be explicit, versioned, observable, causally traceable, and recoverable.**

This is particularly important once NROS has:

- distributed components,
- resource leases,
- activations,
- scheduling,
- physical effects,
- failures,
- recovery,
- autonomous missions.

# 1. The State Fabric

The State Fabric becomes the runtime's semantic memory:

```text
                         NROS
                          │
              ┌───────────┴───────────┐
              │                       │
        Execution Fabric        Communication
              │                       │
              └───────────┬───────────┘
                          ▼
                    State Fabric
                          │
       ┌──────────────────┼──────────────────┐
       ▼                  ▼                  ▼
     State              Events            History
       │                  │                  │
       └──────────────────┼──────────────────┘
                          ▼
                    State Store
                          │
              ┌───────────┴───────────┐
              ▼                       ▼
        Checkpoints                Recovery
```

# 2. State is not just data

A value such as:

```text
battery = 73%
```

is insufficient by itself.

NROS should know:

```text
value       = 73%
revision    = 1842
source      = battery-controller
timestamp   = ...
confidence  = ...
provenance  = ...
```

So state becomes:

```text
StateValue
├── value
├── revision
├── timestamp
├── source
├── provenance
└── validity
```

# 3. State identity

Every state domain should have an identity:

```text
robot/alpha/battery
robot/alpha/navigation
robot/alpha/localization
robot/alpha/mission
robot/alpha/devices/lidar/front
```

This creates a hierarchical state namespace.

# 4. State tree

For example:

```text
robot/alpha
│
├── mission
│   ├── id
│   ├── phase
│   └── status
│
├── localization
│   ├── pose
│   ├── covariance
│   └── map_revision
│
├── battery
│   ├── percentage
│   ├── voltage
│   └── health
│
└── devices
    ├── lidar/front
    ├── camera/front
    └── motor/left
```

This gives NROS a structured system state.

# 5. State revisions

Every meaningful state mutation produces a revision:

```text
Revision 100
     ↓
Revision 101
     ↓
Revision 102
```

For example:

```text
100 → battery = 74%
101 → battery = 73%
102 → battery = 72%
```

A component can therefore say:

```text
"I computed this using state revision 101."
```

# 6. Why revisions matter

Suppose:

```text
Planner reads map revision 42
```

while another component replaces it with:

```text
map revision 43
```

The planner's result now has known provenance:

```text
Plan #81
based_on:
    map = 42
    localization = 103
```

This is much stronger than an unqualified:

```text
Plan generated.
```

# 7. State snapshot

An activation can execute against a snapshot:

```text
Snapshot #900
├── map = 42
├── pose = 103
├── battery = 77
└── mission = 18
```

Then:

```text
Activation #901
uses Snapshot #900
```

This gives deterministic context.

# 8. State consistency models

Not every state requires global consistency.

NROS should support multiple consistency levels:

```text
LOCAL
CAUSAL
VERSIONED
COORDINATED
STRONG
```

For example:

```text
temperature → LOCAL
sensor stream → CAUSAL
configuration → VERSIONED
resource ownership → COORDINATED
safety interlock → STRONG
```

This avoids forcing expensive consistency everywhere.

# 9. Events

State changes should produce events.

Example:

```text
battery.changed
mission.started
lidar.connected
motor.faulted
resource.acquired
activation.completed
```

An event describes **what happened**.

State describes **what is true now**.

# 10. Event vs state

Consider:

```text
Event:
BatteryLevelChanged(74 → 73)
```

Current state:

```text
battery = 73
```

The event belongs to history.

The state belongs to the current world model.

NROS should maintain the distinction.

# 11. Event envelope

Conceptually:

```rust
struct Event {
    id: EventId,
    kind: EventKind,

    timestamp: Timestamp,
    source: SourceId,

    state_revision: StateRevision,
    causal_parent: Option<EventId>,

    payload: Payload,
}
```

This creates a traceable event system.

# 12. Event ordering

Distributed systems rarely have one perfect global clock.

Therefore NROS should distinguish:

```text
physical timestamp
logical ordering
causal ordering
```

For example:

```text
Event A
   ↓
Event B
```

can establish:

```text
A happened-before B
```

even when their wall-clock timestamps are imperfect.

# 13. Causal chains

A complete mission might generate:

```text
MissionRequested
      ↓
MissionAdmitted
      ↓
PlannerActivated
      ↓
PathGenerated
      ↓
ControllerActivated
      ↓
MotorCommandIssued
      ↓
MotorStateChanged
```

The State Fabric can retain this causal relationship.

# 14. Event sourcing

NROS can optionally reconstruct state from events:

```text
Initial State
     +
Event 1
     +
Event 2
     +
Event 3
     ↓
Current State
```

This is the classic event-sourcing model.

But NROS should not require every deployment to persist every event forever.

# 15. Event retention policies

Possible policies:

```text
NONE
EPHEMERAL
WINDOWED
CHECKPOINTED
DURABLE
AUDIT
```

Examples:

```text
high-rate IMU:
EPHEMERAL

mission lifecycle:
DURABLE

safety events:
AUDIT
```

# 16. Checkpoints

For long-running systems, NROS can periodically materialize state:

```text
Events
  │
  ▼
Checkpoint #100
  │
Events
  │
  ▼
Checkpoint #101
```

Recovery can start from the latest valid checkpoint instead of replaying the entire history.

# 17. Checkpoint contents

A checkpoint might contain:

```text
Checkpoint
├── state revision
├── component states
├── resource leases
├── mission state
├── configuration revision
├── scheduler state
└── integrity metadata
```

# 18. Checkpoint validity

A checkpoint should not automatically be trusted.

It can have:

```text
checksum
signature
schema version
runtime version
creation timestamp
source identity
```

This supports integrity verification.

# 19. Recovery

If the runtime crashes:

```text
Runtime
   X
   │
   ▼
Restart
   │
   ▼
Load checkpoint
   │
   ▼
Validate
   │
   ▼
Replay required events
   │
   ▼
Reconstruct state
   │
   ▼
Resume / Recover
```

# 20. Recovery is not simply restart

A robotics runtime cannot blindly restore everything.

For example:

```text
Motor lease
```

may no longer be valid after restart.

Therefore recovery needs classification:

```text
RESTORE
REVALIDATE
REACQUIRE
RESET
DISCARD
```

# 21. Resource recovery

Example:

```text
Checkpoint:
motor lease = L42
```

After restart:

```text
L42
 ↓
REVALIDATE
 ↓
invalid
 ↓
REACQUIRE
```

This prevents stale ownership.

# 22. Physical state recovery

Suppose the checkpoint says:

```text
arm.position = 1.2m
```

but the actual arm is:

```text
1.0m
```

The runtime must not blindly restore the logical value.

Instead:

```text
logical state
      │
      ▼
physical observation
      │
      ▼
reconciliation
```

Physical truth takes precedence where appropriate.

# 23. State reconciliation

NROS therefore needs:

```text
Expected State
      +
Observed State
      ↓
Reconciliation
      ↓
Authoritative State
```

This is essential for robots.

# 24. State authority

Different domains may have different authorities.

Example:

```text
Battery voltage
→ hardware sensor

Motor position
→ encoder

Mission phase
→ mission manager

Resource lease
→ resource manager
```

NROS should explicitly identify the authoritative source.

# 25. Authority conflict

Suppose two components claim:

```text
motor.mode = ENABLED
```

The State Fabric needs an authority rule.

Possible result:

```text
Authority:
Motor controller

Other value:
REJECTED
```

This prevents distributed state ambiguity.

# 26. State ownership

A state entry can therefore have:

```text
owner
authority
readers
writers
version
policy
```

This naturally integrates with the capability/security model.

# 27. State mutation

A mutation should be explicit:

```text
READ
WRITE
PATCH
COMPARE_AND_SET
TRANSACTION
```

For example:

```text
compare_and_set(
    expected_revision = 103,
    new_state = ...
)
```

This prevents lost updates.

# 28. Optimistic concurrency

Two components:

```text
A reads revision 100
B reads revision 100
```

A commits:

```text
100 → 101
```

B then attempts:

```text
100 → 101
```

and receives:

```text
CONFLICT
```

B must re-evaluate against:

```text
revision 101
```

# 29. State conflict handling

Policies can be:

```text
RETRY
MERGE
REJECT
REBASE
ESCALATE
```

Different state domains can select different policies.

# 30. State machine

Many robotics domains naturally form state machines.

Example mission:

```text
IDLE
 ↓
PLANNING
 ↓
EXECUTING
 ↓
COMPLETED
```

With failure paths:

```text
EXECUTING
   ├──→ PAUSED
   ├──→ DEGRADED
   ├──→ ABORTING
   └──→ FAILED
```

NROS should represent these transitions explicitly.

# 31. State transition contract

Conceptually:

```rust
transition(
    current,
    event,
    policy
) -> next
```

A transition can be validated before becoming authoritative.

# 32. Invalid transitions

For example:

```text
COMPLETED → EXECUTING
```

may be invalid.

NROS should reject it rather than allowing arbitrary state mutation.

This makes state machines enforceable.

# 33. State invariants

A state machine can define invariants:

```text
if motor.enabled
then motor.lease.is_valid
```

Another:

```text
if mission.phase == EXECUTING
then mission.plan.is_valid
```

Another:

```text
if actuator.command_enabled
then safety_interlock == CLEAR
```

These become runtime-checkable properties.

# 34. Safety invariants

Safety invariants deserve special treatment:

```text
SAFETY INVARIANT
        │
        ▼
must hold continuously
```

Violation:

```text
INVARIANT_VIOLATED
        ↓
Safety Policy
        ↓
Safe Action
```

This connects the State Fabric directly to physical safety.

# 35. State watchers

Components can subscribe to state predicates:

```text
battery < 20%
```

rather than only raw events.

The runtime evaluates:

```text
predicate(state)
```

and activates the appropriate component when it becomes true.

# 36. Reactive state execution

Example:

```text
battery < 20%
      ↓
EnergyPolicy activation
      ↓
disable optional workloads
      ↓
reserve navigation resources
```

The State Fabric therefore becomes a trigger source for the Execution Fabric.

# 37. State → activation loop

We now have:

```text
State change
    ↓
Event
    ↓
Predicate
    ↓
Activation
    ↓
Scheduler
    ↓
Execution
    ↓
Effect
    ↓
State change
```

This is the fundamental NROS reactive loop.

# 38. Autonomous control loop

For autonomous workloads:

```text
OBSERVE
   ↓
STATE UPDATE
   ↓
INTERPRET
   ↓
PLAN
   ↓
ADMIT
   ↓
SCHEDULE
   ↓
EXECUTE
   ↓
EFFECT
   ↓
OBSERVE
```

This aligns naturally with an agentic execution model.

# 39. State and memory

The State Fabric should not be confused with long-term memory.

A useful separation:

```text
State
→ what is currently believed/known

Event History
→ what happened

Checkpoint
→ recoverable snapshot

Memory
→ retained knowledge/context
```

These have different semantics.

# 40. State and knowledge

For example:

```text
State:
battery = 63%

Event:
battery changed 64 → 63

Memory:
battery typically drops 8% per hour during mission X
```

NROS can support all three without conflating them.

# 41. State queries

Components should be able to ask:

```text
get(state_key)
```

or:

```text
get_at_revision(key, revision)
```

or:

```text
query(predicate)
```

or:

```text
watch(predicate)
```

This gives a rich state API.

# 42. State subscriptions

A component might declare:

```text
watch:
    /robot/alpha/localization/pose
```

or:

```text
watch:
    battery < 20%
```

The second is semantic rather than transport-oriented.

That distinction is important.

# 43. State snapshots for planning

A planner should ideally receive:

```text
PlanningContext
├── state_snapshot
├── resource_snapshot
├── capability_snapshot
├── policy_snapshot
└── time_context
```

It can then produce a plan against an explicit world model.

# 44. Plan validity

A plan may be tied to:

```text
state revision = 1042
resource revision = 881
capability revision = 301
```

Before execution:

```text
validate(plan)
```

If the environment has changed materially:

```text
PLAN_STALE
```

The planner must re-plan.

# 45. Stale-plan protection

This is extremely important.

Without it:

```text
Plan generated
      ↓
world changes
      ↓
old plan executes
```

NROS should support:

```text
Plan
 ↓
Validity conditions
 ↓
Runtime validation
 ↓
Execute only if still valid
```

# 46. State-aware activation

An activation can declare:

```text
requires:
    localization.revision >= 1042
    map.revision == 42
```

The scheduler/admission layer can enforce those conditions.

# 47. State Fabric + Resource Fabric

The two fabrics now interact:

```text
State
 │
 ├── battery = 15%
 │
 ▼
Resource Policy
 │
 ├── GPU → restricted
 ├── CPU → reserved
 └── navigation → priority raised
```

Resources therefore react to state.

And resource changes produce state:

```text
GPU failed
   ↓
Resource event
   ↓
State update
   ↓
Perception degraded
```

# 48. State Fabric + Execution Fabric

Likewise:

```text
Activation completed
       ↓
Execution event
       ↓
State transition
```

and:

```text
State transition
       ↓
New activation
```

The fabrics form a closed loop.

# 49. The NROS semantic cycle

At this stage:

```text
                ┌──────────────┐
                │    STATE     │
                └──────┬───────┘
                       │
                       ▼
                  ACTIVATION
                       │
                       ▼
                   ADMISSION
                       │
                       ▼
                  SCHEDULING
                       │
                       ▼
                   RESOURCE
                       │
                       ▼
                   EXECUTION
                       │
                       ▼
                    EFFECT
                       │
                       ▼
                  STATE UPDATE
                       │
                       └───────────────→
```

This is the heart of NROS.

# 50. Event log as runtime evidence

Every important transition can produce evidence:

```text
Event #9001
├── activation = A42
├── state_before = 1041
├── state_after = 1042
├── resources = [CPU2, LiDAR]
├── scheduler = EDF
├── executor = control-0
├── result = success
└── timestamp = ...
```

This can feed:

```text
observability
debugging
verification
replay
auditing
recovery
```

# 51. Deterministic replay

Given:

```text
checkpoint
+
event stream
+
configuration
```

NROS can attempt:

```text
REPLAY
```

and reconstruct:

```text
state evolution
activation sequence
resource decisions
```

This is extremely valuable for difficult robotics failures.

# 52. Simulation from the same model

The same state/event model can drive simulation:

```text
Physical Robot
      │
      ▼
 NROS Runtime
      │
      ▼
State/Event Model
      │
      ▼
 Simulation
```

Instead of building an entirely separate semantic architecture for simulation.

# 53. Digital twin direction

The State Fabric naturally enables:

```text
Robot
  ↕
Digital Twin
```

Both can share:

```text
state schema
events
capabilities
resource model
mission model
```

while differing in physical execution.

# 54. NROS observability

At this point observability is no longer an optional logging layer.

The runtime can expose:

```text
State
Events
Activations
Resources
Scheduling
Effects
Failures
Recovery
```

as a unified observability model.

# 55. Unified trace

A single trace could show:

```text
09:00:01.001
Camera frame received

09:00:01.002
Perception activation admitted

09:00:01.003
GPU reserved

09:00:01.006
Perception completed

09:00:01.007
Obstacle state updated

09:00:01.008
Planner activated

09:00:01.015
Path generated

09:00:01.016
Control activation admitted

09:00:01.017
Motor command issued
```

This is far more useful than isolated node logs.

# 56. State integrity

State should be protected against:

```text
invalid writes
stale writes
unauthorized writes
conflicting writes
corrupted checkpoints
```

The State Fabric therefore becomes another policy enforcement boundary.

# 57. State schema

State should have schemas:

```text
BatteryState
LocalizationState
MissionState
MotorState
SensorState
ResourceState
```

Schemas should be versioned:

```text
BatteryState v1
BatteryState v2
```

This matters for long-lived robots and distributed deployments.

# 58. Schema migration

When a state schema changes:

```text
v1
 ↓
migration
 ↓
v2
```

The runtime should not silently interpret incompatible data.

Migration should be explicit and verifiable.

# 59. State lifecycle

A state domain can have:

```text
UNINITIALIZED
INITIALIZING
VALID
STALE
DEGRADED
INVALID
RECOVERING
```

For example:

```text
Localization
   ↓
VALID
   ↓
sensor lost
   ↓
STALE
   ↓
recovery
   ↓
VALID
```

# 60. The State Fabric contract

The central contract becomes:

> **Every authoritative state mutation is identifiable, versioned, attributable, policy-checked, and observable.**

And for execution:

> **Every activation executes against an explicit state context and produces explicit effects and state consequences.**

# 61. NROS architecture now

```text
┌───────────────────────────────────────────────────┐
│                  NROS APPLICATION                 │
├───────────────────────────────────────────────────┤
│ Missions / Components / Capabilities / Policies   │
├───────────────────────────────────────────────────┤
│                 Execution Fabric                 │
│ Activation / Admission / Scheduler / Executor    │
├───────────────────────────────────────────────────┤
│                  Resource Fabric                 │
│ Devices / CPU / GPU / Leases / Budgets           │
├───────────────────────────────────────────────────┤
│                    State Fabric                  │
│ State / Events / Revisions / Checkpoints         │
├───────────────────────────────────────────────────┤
│               Communication Fabric              │
│ Topics / Requests / Events / Streams             │
├───────────────────────────────────────────────────┤
│                    Transport                     │
│ IPC / SHM / Network / DDS / Custom              │
├───────────────────────────────────────────────────┤
│                    Platform                      │
│ Linux / RTOS / Embedded / Hardware              │
└───────────────────────────────────────────────────┘
```

# 62. What NROS has become

We started with the ROS computation graph:

```text
Nodes
  ↓
Topics
  ↓
Services
```

NROS is becoming:

```text
Entities
   ↓
Capabilities
   ↓
State
   ↓
Activations
   ↓
Resources
   ↓
Scheduling
   ↓
Effects
   ↓
Events
   ↓
Recovery
```

That is no longer merely a middleware graph.

It is a **runtime semantic graph**.

# 63. Next architectural boundary

There is one major piece still missing:

```text
WHO is allowed to do WHAT?
```

We have already introduced:

```text
capabilities
leases
ownership
policies
authorization
safety
```

but they need to become a unified security model.

That leads to:

# **Part XXXII — NROS Capability, Identity & Policy Fabric**

The target architecture:

```text
                     POLICY FABRIC
                           │
       ┌───────────────────┼───────────────────┐
       ▼                   ▼                   ▼
    Identity          Capability           Policy
       │                   │                   │
       └───────────────────┼───────────────────┘
                           ▼
                     Authorization
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
       State            Resource         Execution
       Access             Access           Access
          │                │                │
          └────────────────┼────────────────┘
                           ▼
                         Effect
```

The central question will be:

> **How can NROS make every meaningful operation—state mutation, resource acquisition, execution, communication, and physical actuation—explicitly attributable to an identity, authorized by a capability, constrained by policy, and recorded as verifiable evidence?**

# NROS — Part XXXII: Identity, Capability & Policy Fabric

We now add the **authority layer**.

The previous parts established:

```text
Communication
State
Resources
Execution
```

But a distributed robotic runtime still needs to answer:

> **Who is requesting this operation, what are they allowed to do, against which resource, under what conditions, and why was the operation accepted?**

That becomes the **Policy Fabric**.

# 1. From permissions to capabilities

A traditional permission model says:

```text
User A
  ↓
permission = WRITE
  ↓
Motor
```

NROS should use a more contextual model:

```text
Identity
   +
Capability
   +
Resource
   +
Operation
   +
Context
   +
Policy
   ↓
Authorization Decision
```

For example:

```text
Agent A
may:
    command motor/left

only when:
    mission = authorized
    safety = clear
    lease = valid
    velocity ≤ limit
```

# 2. Identity

Every active participant should have an identity.

```text
Identity
├── id
├── type
├── issuer
├── credentials
├── attributes
└── lifecycle
```

Possible identities:

```text
Human operator
Component
Agent
Device
Robot
Fleet controller
Remote service
System supervisor
```

# 3. Component identity

A component should not simply be:

```text
process 18342
```

because process IDs are ephemeral.

Instead:

```text
component://robot-alpha/navigation/planner
```

The runtime can map this stable identity to a current process/execution instance.

# 4. Identity hierarchy

Identities can form a hierarchy:

```text
Organization
   │
   └── Robot
        │
        ├── Component
        │    ├── Planner
        │    └── Controller
        │
        └── Device
             ├── LiDAR
             └── Motor
```

This gives policy systems useful scope.

# 5. Capability

A capability represents authority to perform an operation.

Conceptually:

```text
Capability
├── subject
├── operation
├── resource
├── constraints
├── validity
└── issuer
```

Example:

```text
Subject:
planner

Operation:
read

Resource:
localization.pose
```

# 6. Capability granularity

Capabilities should be fine-grained.

Instead of:

```text
robot.admin
```

prefer:

```text
localization.read
map.read
planner.execute
motor.read
motor.command
motor.stop
```

This follows least privilege.

# 7. Capability composition

A complex operation may require multiple capabilities.

Example:

```text
MoveRobot
│
├── localization.read
├── planner.execute
├── motor.command
└── safety.status.read
```

Authorization succeeds only when the required set is satisfied.

# 8. Capability attenuation

A capability can be narrowed.

Suppose:

```text
motor.command
```

is valid.

It can be attenuated to:

```text
motor.command
velocity ≤ 0.5 m/s
```

or:

```text
motor.command
valid for 60 seconds
```

This is useful for temporary agents.

# 9. Delegation

An authorized component can delegate a restricted capability:

```text
Supervisor
   │
   └── delegates
          ↓
       Planner
```

But delegation should not automatically grant more authority than the delegator possesses.

Formally:

```text
DelegatedAuthority
⊆
DelegatorAuthority
```

# 10. Capability chain

A delegated operation can therefore have provenance:

```text
Operator
   ↓
Robot Supervisor
   ↓
Mission Manager
   ↓
Planner
   ↓
Controller
```

NROS can determine:

> **Why was this controller allowed to command the actuator?**

# 11. Policy

Capabilities alone are insufficient.

A capability might say:

```text
planner may command motor
```

But policy can say:

```text
only while:
    mission.status = EXECUTING
    safety.status = CLEAR
    battery > 10%
```

So:

```text
Capability
    +
Policy
    ↓
Authorization
```

# 12. Context-aware authorization

The authorization engine evaluates context:

```text
Identity
Operation
Resource
State
Time
Location
Mission
Safety
Lease
Network context
```

Result:

```text
ALLOW
DENY
CONDITIONAL
```

# 13. Authorization as a pure decision

Conceptually:

```rust
authorize(
    identity,
    capability,
    operation,
    resource,
    context,
) -> Decision
```

The decision should be deterministic given the same policy/context snapshot.

# 14. Decision evidence

Every meaningful authorization decision can produce:

```text
AuthorizationDecision
├── request_id
├── subject
├── operation
├── resource
├── policy_revision
├── context_revision
├── result
└── reason
```

Example:

```text
ALLOW

subject = planner
operation = motor.command
resource = motor.left

reason:
  valid lease
  valid capability
  safety clear
  velocity within policy
```

# 15. Denial should be explainable

Instead of:

```text
PERMISSION_DENIED
```

NROS should expose a structured reason:

```text
DENY

reason:
  capability = valid
  lease = valid
  safety = CLEAR
  velocity = 2.4 m/s
  policy maximum = 1.0 m/s
```

The operation was rejected because:

```text
VELOCITY_LIMIT_EXCEEDED
```

# 16. Policy revisions

Policies themselves should be versioned:

```text
Policy v41
     ↓
Policy v42
```

An authorization decision records:

```text
evaluated_against = policy v42
```

This makes historical decisions reproducible.

# 17. Policy domains

NROS can separate policy domains:

```text
Security Policy
Safety Policy
Resource Policy
Scheduling Policy
Mission Policy
Network Policy
Data Policy
```

A single operation can encounter multiple policies.

# 18. Policy composition

For a physical actuator:

```text
Security
   ↓
Safety
   ↓
Resource
   ↓
Mission
   ↓
Execution
```

All required policies must permit the operation.

# 19. Safety dominates convenience

Suppose:

```text
Mission policy:
ALLOW movement
```

but:

```text
Safety policy:
DENY movement
```

Final decision:

```text
DENY
```

Safety constraints cannot be bypassed by ordinary mission authority.

# 20. Policy precedence

A useful precedence model:

```text
Emergency/Safety
        ↓
Security
        ↓
Resource
        ↓
Mission
        ↓
Application preference
```

The exact formal model should be specified rather than assumed.

# 21. Capability + resource lease

The capability model integrates directly with the Resource Fabric.

An actuator command requires:

```text
Capability
+
Valid Lease
```

Possessing only one is insufficient.

```text
Capability = authority
Lease      = current ownership/access
```

# 22. Capability + state

Likewise:

```text
Capability:
motor.command

State:
safety = CLEAR
```

If:

```text
safety = FAULT
```

the capability may remain valid in principle, but policy prevents use.

This is contextual authorization.

# 23. Capability + execution

A scheduled activation should carry its authority context:

```text
Activation
├── identity
├── capabilities
├── policy snapshot
└── authorization decision
```

The executor can therefore verify that the planned effect is still authorized.

# 24. Time-bounded capabilities

Capabilities can expire:

```text
issued:
12:00

expires:
12:05
```

After expiry:

```text
authorization = DENY
```

This is especially useful for temporary agents.

# 25. Scope-bounded capabilities

A capability may apply only to:

```text
robot/alpha
```

rather than:

```text
all robots
```

Or:

```text
motor/*
```

rather than:

```text
*
```

# 26. Operation constraints

A capability can constrain parameters:

```text
motor.command
velocity <= 1.0
acceleration <= 0.5
```

Then the authorization decision evaluates the actual request.

```text
request.velocity = 0.8
→ ALLOW
```

versus:

```text
request.velocity = 1.5
→ DENY
```

# 27. Data capabilities

Capabilities should cover data as well as actions.

Examples:

```text
camera.raw.read
camera.metadata.read
camera.location.read
```

A component might receive:

```text
processed image
```

without receiving:

```text
raw sensor stream
```

# 28. State capabilities

Similarly:

```text
state.read
state.write
state.watch
state.admin
```

A planner might have:

```text
localization.read
```

but not:

```text
localization.write
```

# 29. Resource capabilities

Resources require their own authority model:

```text
resource.discover
resource.inspect
resource.reserve
resource.acquire
resource.release
resource.configure
```

This prevents arbitrary components from manipulating resource state.

# 30. Scheduler capabilities

Even the scheduler can be protected.

For example:

```text
scheduler.submit
scheduler.cancel
scheduler.priority.override
scheduler.domain.admin
```

A normal application should not be able to arbitrarily promote itself to:

```text
SAFETY_CRITICAL
```

# 31. Identity of physical devices

A physical device can have an identity:

```text
device://robot-alpha/motor/left
```

This allows the runtime to distinguish:

```text
same model
```

from:

```text
same physical device
```

Physical identity matters for calibration, maintenance, and provenance.

# 32. Device attestation

Where supported, a device may provide evidence that it is genuine or authorized.

Conceptually:

```text
Device
   ↓
Identity Evidence
   ↓
Attestation
   ↓
Trusted / Untrusted
```

NROS should treat attestation as optional capability, not a universal assumption.

# 33. Trust state

An identity can have:

```text
UNKNOWN
UNVERIFIED
VERIFIED
TRUSTED
REVOKED
```

Authorization can depend on this state.

# 34. Revocation

Capabilities may need immediate revocation:

```text
Capability
   ↓
REVOKED
```

Any subsequent operation should fail authorization.

Existing leases may also need to be invalidated depending on policy.

# 35. Emergency revocation

For dangerous systems:

```text
EMERGENCY
   ↓
revoke actuator capabilities
   ↓
invalidate leases
   ↓
safe-state transition
```

This becomes a cross-fabric operation.

# 36. Policy engine placement

The Policy Fabric should sit between intent and effect:

```text
Intent
  ↓
Authorization
  ↓
Admission
  ↓
Execution
  ↓
Effect
```

Not merely at network boundaries.

# 37. Defense in depth

A motor command can therefore encounter:

```text
1. API validation
2. capability validation
3. identity validation
4. lease validation
5. safety policy
6. resource policy
7. scheduler policy
8. device-side validation
```

No single layer needs to carry the entire security burden.

# 38. Policy decision point

Architecturally:

```text
             Request
                │
                ▼
       ┌──────────────────┐
       │ Policy Decision  │
       │      Point       │
       └────────┬─────────┘
                │
         ┌──────┴──────┐
         ▼             ▼
       ALLOW          DENY
         │
         ▼
      Executor
```

The policy engine itself should remain independent of individual applications.

# 39. Policy enforcement point

The executor/resource manager/device interface becomes an enforcement point:

```text
Policy Decision
       ↓
Enforcement Point
       ↓
Effect
```

Even if a bug exists upstream, critical enforcement can remain close to the effect.

# 40. Physical enforcement

For safety-critical actuators:

```text
NROS Policy
      +
Device Controller Safety
```

should both enforce constraints.

For example:

```text
NROS max velocity = 1.0 m/s

Motor controller max velocity = 0.8 m/s
```

The physical controller remains the final limiting boundary.

# 41. Policy-aware messages

Messages can carry security context:

```text
Message
├── sender
├── capability context
├── trace
├── timestamp
└── payload
```

But security metadata should not unnecessarily inflate high-rate sensor messages.

Therefore NROS should support efficient references/context propagation.

# 42. Context propagation

An activation can inherit:

```text
identity context
capability context
trace context
mission context
policy context
```

from its parent.

But inheritance must be bounded.

A child should not automatically receive unlimited authority.

# 43. Authority attenuation on spawn

When an agent creates another activation:

```text
Parent
   ↓
Spawn Child
```

the child gets:

```text
subset(parent capabilities)
```

unless an explicit policy allows elevation.

This is critical for autonomous agents.

# 44. Agent identity

This model is particularly useful for agentic NROS.

An agent becomes:

```text
Agent
├── identity
├── capabilities
├── goals
├── state
├── resource budget
└── policy constraints
```

Its autonomy is therefore bounded by explicit authority.

# 45. Agent action

An agent should not directly mutate arbitrary robot state.

Instead:

```text
Agent Intent
    ↓
Capability Check
    ↓
Policy Check
    ↓
Resource Check
    ↓
Scheduler
    ↓
Execution
```

This creates **governed autonomy**.

# 46. Mission authority

A mission can grant a bounded authority envelope:

```text
Mission:
Inspect Warehouse

Allowed:
camera.read
lidar.read
navigation.execute

Forbidden:
motor.admin
firmware.update
safety.override
```

The agent operates inside this envelope.

# 47. Policy inheritance

A mission can constrain all child activities:

```text
Mission Policy
     │
     ├── Planner
     ├── Perception
     ├── Navigation
     └── Controller
```

No child can exceed the mission's authority.

# 48. Runtime authority lattice

We can model authority as a lattice:

```text
                    SYSTEM
                       │
                    ROBOT
                       │
                    MISSION
                       │
                   COMPONENT
                       │
                    ACTION
```

But actual permissions should be capability-based rather than relying solely on hierarchy.

# 49. Policy + State + Resource

The three major runtime constraints converge:

```text
             Operation
                 │
      ┌──────────┼──────────┐
      ▼          ▼          ▼
   Identity     State     Resource
      │          │          │
      └──────────┼──────────┘
                 ▼
              Policy
                 │
                 ▼
          Authorization
```

This is one of the central NROS abstractions.

# 50. Authorization invariant

NROS can establish:

```text
ExecutableEffect(E)
    ⇔
    Authorized(E)
    ∧ ResourceValid(E)
    ∧ StateValid(E)
    ∧ PolicyValid(E)
```

For safety-critical effects, additional conditions apply.

# 51. Evidence-backed authorization

The runtime should retain enough evidence to answer:

```text
Who authorized this?
Which capability?
Which policy revision?
Which state?
Which resource lease?
Which scheduler decision?
Which activation?
```

The resulting chain:

```text
Identity
  ↓
Capability
  ↓
Policy
  ↓
Authorization
  ↓
Activation
  ↓
Resource
  ↓
Execution
  ↓
Effect
```

is an **authority provenance chain**.

# 52. Unified provenance

Now combine authority with causal state:

```text
Operator
  ↓
Mission
  ↓
Agent
  ↓
Activation
  ↓
Resource Lease
  ↓
Physical Effect
  ↓
Observed State
```

NROS can therefore reconstruct both:

**causal provenance**

and:

**authority provenance**.

# 53. Audit event

A sensitive action could produce:

```text
EffectAuthorized

subject:
    agent/navigation

operation:
    motor.command

resource:
    motor.left

policy:
    safety-policy-v18

capability:
    cap-392

lease:
    lease-77

activation:
    act-902

decision:
    ALLOW
```

This is dramatically stronger than a plain log line.

# 54. Security events

The Policy Fabric should generate events for:

```text
authorization granted
authorization denied
capability issued
capability delegated
capability revoked
lease acquired
lease revoked
identity verified
identity rejected
policy changed
```

These become part of the State/Event model.

# 55. Policy changes

Changing policy is itself a privileged operation:

```text
Policy update request
       ↓
Authorization
       ↓
Validation
       ↓
New policy revision
       ↓
Activation
```

A policy cannot simply change invisibly.

# 56. Hot policy update

For some deployments:

```text
Policy v41
    ↓
Policy v42
```

can become active without restarting the robot.

But the transition must be atomic with respect to authorization evaluation.

# 57. Policy snapshot

An activation should record:

```text
policy_revision = 42
```

Thus if policy later changes:

```text
v42 → v43
```

we still know what governed the original admission.

# 58. Policy consistency

Distributed components need a clear rule for policy versions.

For example:

```text
Activation admitted under policy v42
```

must not silently execute under an incompatible policy v43.

Possible mechanisms:

```text
pin
revalidate
abort
```

depending on policy class.

# 59. Policy-aware recovery

After restart:

```text
checkpoint
   ↓
restore identity
   ↓
restore policy revision
   ↓
revalidate capabilities
   ↓
revalidate leases
   ↓
resume
```

Stale authority must never be blindly restored.

# 60. The Policy Fabric

We can now represent it as:

```text
                      POLICY FABRIC
                            │
             ┌──────────────┼──────────────┐
             ▼              ▼              ▼
          Identity       Capability       Policy
             │              │              │
             └──────────────┼──────────────┘
                            ▼
                    Authorization Engine
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
      State              Resource           Execution
        │                   │                   │
        └───────────────────┼───────────────────┘
                            ▼
                          Effect
```

# 61. NROS — five fabrics

The architecture now has five cooperating fabrics:

```text
┌────────────────────────────────────────────┐
│              NROS RUNTIME                  │
├────────────────────────────────────────────┤
│ Communication Fabric                       │
│ State Fabric                               │
│ Resource Fabric                            │
│ Execution Fabric                           │
│ Policy Fabric                              │
└────────────────────────────────────────────┘
```

And they form a closed semantic system:

```text
                POLICY
                   │
                   ▼
STATE ───────→ EXECUTION ←────── RESOURCES
  ▲                │
  │                ▼
  └────────── COMMUNICATION
```

# 62. ROS → NROS transformation

The original ROS concepts now map approximately like this:

| ROS | NROS |
|---|---|
| Node | Component |
| Topic | Typed communication channel |
| Service | Request/response capability |
| Action | Governed activation/mission |
| Parameter Server | Versioned State Fabric |
| Master/discovery | Runtime discovery/control plane |
| Executor | Scheduler + Execution Fabric |
| Driver | Device/resource component |
| Hardware abstraction | Resource + Capability model |
| rosbag | Event/trace/replay system |
| roslaunch | Declarative deployment/runtime graph |
| tf/tf2 | Versioned spatial state |
| Package | Component/capability distribution unit |

The important point is that NROS does not merely rename ROS concepts.

It changes their **semantic responsibility**.

# 63. The emerging NROS kernel

At this point the minimal runtime kernel begins to look like:

```text
NROS Kernel
│
├── Identity Manager
├── Capability Manager
├── Policy Engine
├── State Store
├── Event Bus
├── Resource Manager
├── Scheduler
├── Executor
├── Supervisor
└── Transport
```

Everything else can be built above this foundation.

# 64. Minimal kernel principle

NROS should still avoid putting every robotics feature into the kernel.

The kernel should own only the semantics that must be globally coherent:

```text
identity
authority
state
resources
execution
events
lifecycle
supervision
```

Higher-level robotics functionality remains composable.

# 65. Next boundary: supervision

We now have enough mechanisms to execute governed work.

But robots fail.

Components crash.

Networks partition.

Sensors disappear.

Resources degrade.

Plans become stale.

Therefore the next layer is not another ordinary service.

It is:

# **Part XXXIII — NROS Supervision, Fault Domains & Recovery**

The architecture will evolve toward:

```text
                       SUPERVISOR
                           │
         ┌─────────────────┼─────────────────┐
         ▼                 ▼                 ▼
      Health             Fault             Recovery
      Monitor            Detector          Manager
         │                 │                 │
         └─────────────────┼─────────────────┘
                           ▼
                    Failure Domains
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
           Component     Resource     Network
              │            │            │
              └────────────┼────────────┘
                           ▼
                      Recovery Plan
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
            Retry        Restart      Degrade
                           │
                           ▼
                       Safe State
```

The key NROS question becomes:

> **When something fails, how does the runtime determine what actually failed, what remains trustworthy, what must be isolated, what can be recovered, and what physical state the robot must enter?**

# NROS — Part XXXIII: Supervision, Fault Domains & Recovery

The next transformation is from a runtime that can **authorize and execute** work into a runtime that can **survive failure**.

ROS traditionally gives developers mechanisms for launching nodes, monitoring processes, restarting components, and building fault-handling logic. NROS should make failure semantics a **first-class runtime concern**.

The fundamental shift is:

```text
ROS:
    process failed
        ↓
    maybe restart it

NROS:
    component failure
        ↓
    detect
        ↓
    classify
        ↓
    isolate
        ↓
    determine affected state/resources
        ↓
    select recovery policy
        ↓
    recover / degrade / stop
        ↓
    verify
        ↓
    resume or enter safe state
```

# 1. Failure is part of the runtime model

A failure should not be treated as an exceptional event outside the architecture.

Instead:

```text
Failure
```

is a first-class runtime entity.

```text
Failure
├── id
├── detected_at
├── source
├── domain
├── class
├── severity
├── evidence
├── affected resources
├── affected activations
├── recovery policy
└── status
```

# 2. Health is not simply alive/dead

A process can be alive while being unusable.

Therefore NROS needs richer health states:

```text
UNKNOWN
STARTING
HEALTHY
DEGRADED
UNRESPONSIVE
FAILED
ISOLATED
RECOVERING
STOPPED
```

A component being `RUNNING` is therefore not equivalent to being `HEALTHY`.

# 3. Component health

A component can expose a health contract:

```text
Component
│
├── liveness
├── readiness
├── responsiveness
├── resource health
├── dependency health
└── semantic health
```

For example:

```text
planner:
    process = alive
    executor = responsive
    localization = unavailable
```

Overall result:

```text
DEGRADED
```

not necessarily:

```text
FAILED
```

# 4. Liveness

Liveness asks:

> Is the component still executing?

A heartbeat may provide:

```text
heartbeat(component_id, sequence, timestamp)
```

But heartbeat alone is insufficient.

A component can continuously emit heartbeats while being logically stuck.

# 5. Readiness

Readiness asks:

> Can this component safely accept work?

For example:

```text
camera:
    process alive
    device initialized
    calibration loaded
    stream available
```

Only then:

```text
READY
```

# 6. Semantic health

NROS should support application-defined health checks.

Example:

```text
localization:
    pose updates < 500 ms
    covariance < threshold
```

If those conditions fail:

```text
semantic health = DEGRADED
```

even though the process itself remains operational.

# 7. Health as state

Health belongs in the State Fabric:

```text
ComponentState
├── lifecycle
├── health
├── capabilities
├── resources
├── dependencies
└── metrics
```

This allows other components and policies to react to health changes.

# 8. Fault detection

Fault detection can combine:

```text
heartbeat timeout
request timeout
resource failure
invalid state transition
protocol violation
health-check failure
sensor anomaly
deadline miss
memory exhaustion
CPU starvation
network partition
device fault
```

The detector should produce structured evidence.

# 9. Fault event

Example:

```text
FaultDetected
{
    fault_id: F-204,
    source: localization,
    class: deadline_miss,
    severity: critical,
    evidence:
        update_age = 2.4s,
        threshold = 0.5s
}
```

This is more useful than:

```text
"localization timeout"
```

# 10. Fault classification

NROS should distinguish at least:

```text
TRANSIENT
INTERMITTENT
PERSISTENT
DEPENDENCY
RESOURCE
CONFIGURATION
PROTOCOL
SECURITY
SAFETY
HARDWARE
UNKNOWN
```

Different classes imply different recovery strategies.

# 11. Fault severity

A simple severity scale:

```text
INFO
WARNING
ERROR
CRITICAL
FATAL
```

But severity should be policy-defined.

A camera failure may be:

```text
WARNING
```

for mapping but:

```text
CRITICAL
```

for an inspection mission requiring visual verification.

# 12. Failure domains

This is one of the most important NROS concepts.

A failure should have a **blast radius**.

For example:

```text
Robot
│
├── Compute Domain
│   ├── Planner
│   └── Perception
│
├── Control Domain
│   └── Motor Controller
│
├── Sensor Domain
│   ├── Camera
│   └── LiDAR
│
└── Network Domain
```

A failure in one domain should not automatically contaminate every other domain.

# 13. Fault containment

Suppose perception crashes:

```text
Perception
    ↓
FAILED
```

NROS should determine:

```text
Can navigation continue?
Can manual control continue?
Can safety monitoring continue?
Can logging continue?
```

Rather than blindly restarting the entire robot.

# 14. Dependency graph

Every component can declare dependencies:

```text
Planner
 ├── Localization
 ├── Map
 └── Configuration
```

If:

```text
Localization = FAILED
```

then:

```text
Planner = DEGRADED / BLOCKED
```

depending on policy.

# 15. Dependency propagation

The runtime can compute:

```text
Localization
     ↓
Planner
     ↓
Navigation
     ↓
Mission
```

A failure propagates through semantic dependencies.

But propagation should be controlled.

Not every failure should become a global failure.

# 16. Failure containment graph

NROS therefore maintains two related graphs:

```text
Communication Graph
```

and:

```text
Dependency / Failure Graph
```

They are not necessarily identical.

This distinction is crucial.

# 17. ROS graph vs NROS graph

ROS:

```text
Node A ──topic──> Node B
```

NROS:

```text
Component A
   │
   ├── communicates with B
   ├── depends on C
   ├── owns resource R
   ├── authorized by policy P
   └── belongs to fault domain D
```

The runtime graph becomes semantic rather than purely communicational.

# 18. Recovery policies

For each failure NROS can select:

```text
IGNORE
RETRY
RESTART
RECREATE
FAILOVER
DEGRADE
ROLLBACK
ABORT
SAFE_STOP
EMERGENCY_STOP
```

The correct response depends on the fault.

# 19. Retry

Transient network failure:

```text
request
  ↓
timeout
  ↓
retry
```

with bounded policy:

```text
max_attempts = 3
backoff = exponential
deadline = 2s
```

Retries must never become infinite loops.

# 20. Restart

A failed stateless component may simply restart:

```text
FAILED
  ↓
STOP
  ↓
START
  ↓
HEALTH CHECK
  ↓
READY
```

But NROS must verify that its previous authority and resources are still valid.

# 21. Rehydration

A restart may require reconstructing state:

```text
checkpoint
   ↓
component restart
   ↓
restore state
   ↓
validate state
   ↓
resume
```

Invalid or stale state must be rejected.

# 22. Resource recovery

Suppose:

```text
controller crashed
```

while holding:

```text
motor lease
```

The supervisor must determine whether the lease remains valid.

Possible policy:

```text
component failure
      ↓
lease revoked
      ↓
actuator safe-state
```

This prevents orphaned ownership.

# 23. Orphan resources

An important invariant:

> **No failed component may indefinitely retain exclusive control over a critical resource.**

Therefore leases need failure semantics:

```text
lease
├── owner
├── expiration
├── failure behavior
└── revocation policy
```

# 24. Supervisor

The Supervisor is the central coordination component.

```text
Supervisor
├── Health Monitor
├── Fault Detector
├── Dependency Analyzer
├── Recovery Planner
├── Recovery Executor
├── Safety Coordinator
└── Incident Recorder
```

The supervisor itself must be treated as highly trusted infrastructure.

# 25. Supervisor hierarchy

A single global supervisor can become a single point of failure.

NROS can therefore support:

```text
Robot Supervisor
│
├── Compute Supervisor
├── Control Supervisor
├── Sensor Supervisor
└── Network Supervisor
```

Each domain supervises locally.

A higher-level supervisor coordinates cross-domain recovery.

# 26. Hierarchical supervision

Example:

```text
                 Robot Supervisor
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
     Compute         Control        Sensors
     Supervisor      Supervisor     Supervisor
        │              │              │
      nodes          devices        drivers
```

This limits failure blast radius.

# 27. Supervision tree

The structure resembles:

```text
Supervisor
   │
   ├── Component A
   ├── Component B
   │      ├── Worker B1
   │      └── Worker B2
   └── Component C
```

If B1 fails:

```text
B1 restart
```

may be sufficient.

If B itself becomes inconsistent:

```text
B restart
```

may be required.

If the entire domain becomes unsafe:

```text
domain shutdown
```

may be necessary.

# 28. Recovery scope

NROS should explicitly define recovery scopes:

```text
TASK
COMPONENT
PROCESS
RESOURCE
DOMAIN
ROBOT
FLEET
```

Recovery should use the smallest scope that safely resolves the failure.

# 29. Minimal recovery principle

Prefer:

```text
smallest safe recovery
```

over:

```text
restart everything
```

Example:

```text
sensor parser failed
```

should ideally not cause:

```text
robot reboot
```

unless policy requires it.

# 30. Recovery state machine

A recovery operation can be modeled:

```text
DETECTED
   ↓
CLASSIFIED
   ↓
CONTAINED
   ↓
PLANNED
   ↓
EXECUTING
   ↓
VALIDATING
   ↓
RECOVERED
```

Failure during recovery:

```text
RECOVERING
    ↓
RECOVERY_FAILED
    ↓
ESCALATE
```

# 31. Escalation

Example:

```text
retry failed
   ↓
restart failed
   ↓
domain recovery failed
   ↓
robot recovery
   ↓
safe state
```

Escalation should be deterministic and policy-driven.

# 32. Recovery budgets

Recovery must have resource limits:

```text
max retries
max restart count
max recovery duration
max CPU budget
max energy budget
```

Otherwise a robot could spend all its resources repeatedly attempting recovery.

# 33. Recovery loops

NROS should detect:

```text
start
 → crash
 → restart
 → crash
 → restart
 → crash
```

This is a recovery loop.

After a threshold:

```text
FAILED_PERMANENTLY
```

or:

```text
DEGRADED
```

depending on policy.

# 34. Backoff

Repeated recovery attempts should use controlled backoff:

```text
1s
2s
4s
8s
...
```

with a maximum.

# 35. Safe state

Every safety-relevant subsystem should define a safe state.

For a mobile robot:

```text
motors = STOP
```

For an arm:

```text
actuators = HOLD / SAFE
```

For a drone:

```text
flight controller = emergency policy
```

Safe state is therefore **domain-specific**.

# 36. Safe-state hierarchy

A robot can have:

```text
Component Safe State
        ↓
Subsystem Safe State
        ↓
Robot Safe State
        ↓
Fleet Safe State
```

Recovery can escalate through this hierarchy.

# 37. Safety transition

A safety transition itself should be explicit:

```text
NORMAL
   ↓
FAULT
   ↓
SAFE_TRANSITION
   ↓
SAFE
```

Not:

```text
FAULT
   ↓
random shutdown
```

# 38. Recovery and authorization

Recovery operations are privileged.

For example:

```text
restart motor controller
```

may require:

```text
supervisor.recover
```

while:

```text
restart visualization
```

may require ordinary component-management authority.

# 39. Recovery and state

A recovery action can mutate state:

```text
fault
 ↓
state transition
 ↓
resource release
 ↓
component restart
```

Therefore recovery belongs inside the same transactional semantics as other runtime operations.

# 40. Recovery and event sourcing

A recovery sequence can be represented as events:

```text
FaultDetected
ComponentIsolated
LeaseRevoked
RecoveryStarted
ComponentRestarted
HealthCheckPassed
LeaseReacquired
RecoveryCompleted
```

This produces a complete incident timeline.

# 41. Incident identity

Each failure episode gets an incident ID:

```text
incident://robot-alpha/2026-08-21/0042
```

All related events reference it.

This allows:

```text
incident
   ↓
faults
   ↓
actions
   ↓
effects
   ↓
recovery
```

to be reconstructed.

# 42. Fault correlation

Several symptoms may actually represent one fault.

Example:

```text
camera timeout
localization degraded
planner blocked
mission paused
```

The root cause might be:

```text
camera power failure
```

NROS should support correlation rather than treating every symptom as an independent failure.

# 43. Root-cause graph

```text
Power Failure
      │
      ├── Camera FAILED
      │      ↓
      │   Localization DEGRADED
      │      ↓
      │   Planner BLOCKED
      │      ↓
      │   Mission PAUSED
      │
      └── Telemetry DEGRADED
```

This is substantially more useful than four unrelated error messages.

# 44. Fault evidence

A fault should reference evidence:

```text
Fault
├── heartbeat history
├── metrics
├── state transitions
├── resource events
├── communication failures
├── device diagnostics
└── relevant traces
```

This gives NROS an auditable incident model.

# 45. Recovery verification

Restarting a component does **not** mean recovery succeeded.

NROS must verify:

```text
process alive
+
health OK
+
dependencies ready
+
state valid
+
capabilities valid
+
resources acquired
```

Only then:

```text
RECOVERED
```

# 46. Recovery admission

Before recovery:

```text
RecoveryPlan
   ↓
Policy Check
   ↓
Resource Check
   ↓
Safety Check
   ↓
Execute
```

Recovery itself therefore passes through the Policy Fabric.

# 47. Recovery plan

A recovery plan might be:

```text
Failure:
localization unavailable

Plan:
1. stop autonomous navigation
2. revoke navigation activation
3. restart localization
4. validate sensor stream
5. reconstruct localization state
6. validate pose
7. restart planner
8. resume mission if policy permits
```

This is a **semantic recovery**, not merely a process restart.

# 48. Mission-aware recovery

The same fault can have different consequences.

```text
Mission A:
warehouse inspection
```

may tolerate:

```text
camera degraded
```

while:

```text
Mission B:
visual inspection
```

cannot.

Thus recovery policy may depend on:

```text
mission context
```

which comes directly from the Policy Fabric.

# 49. Degraded operation

A robust robot should not have only:

```text
RUNNING
FAILED
```

Instead:

```text
FULL
DEGRADED
LIMITED
SAFE
```

Example:

```text
LiDAR failed
   ↓
navigation switches to reduced mode
```

if the safety policy permits.

# 50. Graceful degradation

NROS can define capability degradation:

```text
FULL_NAVIGATION
      ↓
REDUCED_NAVIGATION
      ↓
MANUAL_CONTROL
      ↓
SAFE_STOP
```

This creates an explicit fallback ladder.

# 51. Degradation must be authorized

A component must not arbitrarily switch into a weaker safety mode.

For example:

```text
autonomous navigation unavailable
```

does not automatically mean:

```text
manual motor control enabled
```

The Policy Fabric decides whether the fallback is authorized.

# 52. Failure containment + authority

The complete relationship becomes:

```text
Fault
 ↓
Containment
 ↓
Authority Re-evaluation
 ↓
Resource Reconciliation
 ↓
Recovery
```

This prevents stale permissions surviving a fault.

# 53. Failure invalidates assumptions

After a serious fault:

```text
previous state
```

may no longer be trustworthy.

Therefore NROS should distinguish:

```text
KNOWN_VALID
KNOWN_INVALID
UNKNOWN
STALE
```

for recovered state.

# 54. Recovery trust boundary

Example:

```text
Controller crashed
```

Its memory state may be:

```text
UNKNOWN
```

Therefore the runtime should not simply restore it and assume validity.

Instead:

```text
restore
 ↓
validate
 ↓
trusted state
```

# 55. Checkpoint + recovery

The checkpoint mechanism introduced earlier now becomes part of supervision:

```text
Execution
   ↓
Checkpoint
   ↓
Failure
   ↓
Recovery
   ↓
Checkpoint validation
   ↓
Resume
```

This connects:

```text
Execution Fabric
State Fabric
Supervision Fabric
```

# 56. The sixth fabric

We can now introduce a dedicated:

# **Supervision Fabric**

The NROS architecture becomes:

```text
┌─────────────────────────────────────────────┐
│                 NROS                        │
├─────────────────────────────────────────────┤
│ Communication Fabric                        │
│ State Fabric                                │
│ Resource Fabric                             │
│ Execution Fabric                            │
│ Policy Fabric                               │
│ Supervision Fabric                          │
└─────────────────────────────────────────────┘
```

# 57. Six-fabric interaction

```text
                    POLICY
                       │
                       ▼
STATE ───────────→ EXECUTION
  ▲                    │
  │                    ▼
  │                 EFFECT
  │                    │
  │                    ▼
  └────────── SUPERVISION
                 │
                 ▼
             RECOVERY
                 │
                 ├── State
                 ├── Resources
                 └── Execution
```

Communication connects every layer.

# 58. Runtime control loop

The NROS control plane is now approaching:

```text
OBSERVE
   ↓
INTERPRET
   ↓
AUTHORIZE
   ↓
ADMIT
   ↓
EXECUTE
   ↓
OBSERVE EFFECT
   ↓
VERIFY
   ↓
RECOVER IF REQUIRED
```

This is much closer to a modern autonomous runtime than the original ROS process graph.

# 59. ROS → NROS conceptual evolution

The transformation can now be summarized:

```text
ROS
│
├── Nodes
├── Topics
├── Services
├── Actions
├── Parameters
├── Executors
└── Launch
        │
        ▼
NROS
│
├── Components
├── Typed Channels
├── Operations
├── Activations
├── Versioned State
├── Scheduler
├── Deployment
├── Identity
├── Capabilities
├── Policies
├── Resources
└── Supervision
```

# 60. The deeper transformation

ROS fundamentally asks:

> **How do robot programs communicate?**

NROS asks a larger question:

> **How does a distributed autonomous system safely transform intent into physical effect under explicit state, resource, authority, timing, and failure constraints?**

That is the architectural leap.

# 61. NROS semantic equation

The runtime can now be expressed approximately as:

```text
Physical Effect
=
Intent
∩
Authorization
∩
Valid State
∩
Resource Ownership
∩
Execution Admission
∩
Safety Constraints
∩
Healthy Dependencies
```

If any mandatory condition fails:

```text
NO EFFECT
```

or a controlled safe-state transition occurs.

# 62. The next missing layer

We now have:

```text
Communication
State
Resources
Execution
Policy
Supervision
```

But one critical concern remains:

**time**.

Robot behavior is not merely:

```text
what happened?
```

It is:

```text
when did it happen?
when must it happen?
how late is too late?
what ordering is valid?
what deadline applies?
what clock can be trusted?
```

ROS applications often handle these questions individually.

NROS should make them runtime semantics.

Therefore the next major section is:

# **Part XXXIV — NROS Time, Deadlines, Temporal Semantics & Deterministic Execution**

We will derive:

```text
Clock Domains
     ↓
Logical Time
     ↓
Monotonic Time
     ↓
Sensor Time
     ↓
Execution Time
     ↓
Deadlines
     ↓
Budgets
     ↓
Temporal Contracts
     ↓
Deterministic Scheduling
     ↓
Real-Time / Soft Real-Time / Best-Effort Domains
```

This is where NROS begins to directly address one of the fundamental historical limitations of ROS: **temporal semantics cannot remain an application-level convention if the runtime itself is expected to provide predictable autonomous execution.**

# NROS — Part XXXIV: Time, Deadlines & Temporal Semantics

The next step is to make **time a first-class runtime primitive**.

In a conventional middleware system, timestamps are often attached to messages and deadlines are handled by application code.

For NROS, that is insufficient.

A robot's behavior depends on temporal correctness just as much as logical correctness:

```text
Correct action
    +
Correct state
    +
Correct authority
    +
Correct resource
    +
Correct time
```

A command that arrives 500 ms too late can be semantically equivalent to a wrong command.

# 1. Time becomes a runtime fabric

NROS therefore gains a seventh conceptual fabric:

```text
┌──────────────────────────────────────────────┐
│                    NROS                      │
├──────────────────────────────────────────────┤
│ Communication Fabric                         │
│ State Fabric                                 │
│ Resource Fabric                              │
│ Execution Fabric                             │
│ Policy Fabric                                │
│ Supervision Fabric                           │
│ Temporal Fabric                              │
└──────────────────────────────────────────────┘
```

The Temporal Fabric governs:

```text
time
ordering
deadlines
periods
budgets
timeouts
leases
scheduling constraints
temporal validity
```

# 2. Multiple notions of time

NROS should never assume there is one universal clock.

At minimum:

```text
Physical Time
Monotonic Time
Logical Time
Sensor Time
Simulation Time
Network Time
Execution Time
```

These clocks have different semantics.

# 3. Wall-clock time

Wall-clock time answers:

> What date/time is it?

Example:

```text
2026-08-21T14:32:18Z
```

Useful for:

```text
logs
events
auditing
human interfaces
certificates
```

But wall time is not necessarily safe for measuring durations.

# 4. Monotonic time

For durations and deadlines:

```text
t2 - t1
```

NROS should prefer a monotonic clock.

A clock adjustment must not cause:

```text
deadline
```

to move backward unexpectedly.

# 5. Logical time

Distributed components may need logical ordering:

```text
event A
    ↓
event B
```

even when physical clocks cannot be perfectly synchronized.

NROS can therefore attach:

```text
logical sequence
causal relationship
```

to important events.

# 6. Sensor time

A sensor measurement has its own temporal meaning.

For example:

```text
LiDAR sample
timestamp = T_sensor
```

but the runtime receives it at:

```text
T_receive
```

and processing completes at:

```text
T_complete
```

These are not interchangeable.

# 7. Temporal provenance

A message can therefore carry:

```text
TemporalMetadata
├── source_time
├── receive_time
├── processing_start
├── processing_end
└── sequence
```

This enables latency analysis.

# 8. End-to-end latency

For a sensor-to-actuator pipeline:

```text
Sensor
  ↓
Transport
  ↓
Perception
  ↓
Planner
  ↓
Controller
  ↓
Actuator
```

NROS can calculate:

```text
T_effect - T_source
```

rather than only measuring individual node execution times.

# 9. Temporal contracts

A component can declare:

```text
TemporalContract
├── period
├── deadline
├── latency
├── jitter
├── timeout
└── execution budget
```

Example:

```text
localization:
    period = 20 ms
    deadline = 30 ms
```

# 10. Period

A periodic task may require:

```text
every 20 ms
```

approximately:

```text
20
40
60
80
100
...
```

But NROS should distinguish:

```text
requested period
actual period
jitter
```

# 11. Deadline

A deadline defines:

> The latest acceptable completion time.

Example:

```text
release = 100ms
deadline = 120ms
```

Completion at:

```text
118ms → valid
```

Completion at:

```text
123ms → deadline miss
```

# 12. Deadline miss

A deadline miss becomes a runtime event:

```text
DeadlineMissed
├── activation
├── expected_deadline
├── actual_completion
├── lateness
└── policy
```

The system can then decide:

```text
ignore
degrade
retry
cancel
escalate
```

# 13. Deadlines and safety

Not every deadline miss is equally important.

Example:

```text
visualization:
deadline miss → WARNING
```

versus:

```text
motor control:
deadline miss → CRITICAL
```

Thus temporal contracts integrate with Policy and Supervision.

# 14. Execution budget

A task may have a computation budget:

```text
budget = 2 ms
deadline = 10 ms
```

The distinction is important:

```text
budget:
    how much computation is allowed

deadline:
    when the result must be available
```

# 15. Budget exhaustion

If a component exceeds its budget:

```text
budget exceeded
```

NROS can:

```text
throttle
preempt
cancel
degrade
record violation
```

depending on execution policy.

# 16. Temporal admission

Before execution:

```text
Activation
    ↓
Temporal Check
    ↓
Resource Check
    ↓
Policy Check
    ↓
Scheduler
```

The scheduler should not admit work that cannot plausibly meet its temporal contract.

# 17. Temporal feasibility

Suppose:

```text
deadline = 5 ms
estimated execution = 8 ms
```

The runtime should recognize:

```text
INFEASIBLE
```

before blindly executing the work.

This is an important difference between:

```text
execution
```

and:

```text
temporal execution planning
```

# 18. Temporal priority

NROS can prioritize work based on:

```text
deadline
criticality
age
resource dependency
mission importance
```

Rather than simply:

```text
priority = integer
```

# 19. Criticality

Tasks can belong to temporal classes:

```text
CRITICAL
REAL_TIME
DEADLINE_SENSITIVE
INTERACTIVE
BEST_EFFORT
BACKGROUND
```

A background telemetry task should never unnecessarily delay a critical control task.

# 20. Scheduling domains

Different workloads require different scheduling semantics.

```text
Temporal Domain
├── Hard Real-Time
├── Firm Real-Time
├── Soft Real-Time
├── Interactive
└── Best-Effort
```

NROS does not have to make the entire system real-time.

It can instead create explicit domains.

# 21. Hard real-time boundary

For a hard real-time operation:

```text
deadline miss
    ↓
system correctness violation
```

Such tasks require deterministic execution paths and appropriate RTOS/hardware support.

NROS should not pretend that ordinary Linux scheduling magically provides hard real-time guarantees.

# 22. Soft real-time

For soft real-time:

```text
deadline miss
    ↓
quality degradation
```

Example:

```text
camera visualization
```

A missed frame is undesirable but not necessarily dangerous.

# 23. Firm real-time

Firm real-time:

```text
late result
    ↓
result becomes useless
```

but does not necessarily imply system failure.

Example:

```text
stale perception result
```

NROS can automatically discard expired results.

# 24. Temporal validity

A message can have:

```text
valid_from
valid_until
```

Example:

```text
Pose
valid_until = T + 100ms
```

After that:

```text
state = STALE
```

This connects time directly to the State Fabric.

# 25. Stale-state protection

Suppose:

```text
last_pose_age = 800ms
```

while policy requires:

```text
max_pose_age = 200ms
```

Then:

```text
navigation authorization = DENY
```

This is extremely important for physical systems.

# 26. Temporal capabilities

Capabilities themselves can be temporal:

```text
capability:
    motor.command

valid:
    14:00 → 14:05
```

This is already partially established by the Policy Fabric.

The Temporal Fabric makes those semantics explicit.

# 27. Lease expiration

Resource leases are also temporal contracts:

```text
Lease
├── owner
├── resource
├── acquired_at
└── expires_at
```

Expiration becomes a deterministic runtime event.

# 28. Activation lifetime

Every activation should have a temporal boundary:

```text
Activation
├── created
├── released
├── deadline
├── cancellation
└── completion
```

Thus an activation can transition:

```text
PENDING
→ RUNNING
→ COMPLETED
```

or:

```text
PENDING
→ EXPIRED
```

or:

```text
RUNNING
→ DEADLINE_MISSED
→ CANCELLED
```

# 29. Temporal state machine

A richer activation lifecycle:

```text
             ┌─────────────┐
             │   PENDING   │
             └──────┬──────┘
                    ↓
             ┌─────────────┐
             │  ADMITTED   │
             └──────┬──────┘
                    ↓
             ┌─────────────┐
             │   RUNNING   │
             └──────┬──────┘
              ┌─────┼─────┐
              ↓     ↓     ↓
          COMPLETE  MISS  CANCEL
```

# 30. Cancellation semantics

Cancellation should not simply mean:

```text
kill process
```

It should mean:

```text
request cancellation
    ↓
stop accepting new work
    ↓
quiesce
    ↓
release resources
    ↓
emit completion/cancel event
```

For critical operations, cancellation itself may require policy authorization.

# 31. Temporal causality

NROS events should preserve causality:

```text
SensorUpdate
    ↓
PlanningActivation
    ↓
ControlActivation
    ↓
MotorCommand
```

This allows the runtime to answer:

> Which sensor observation caused this physical command?

# 32. Temporal provenance chain

```text
Sensor sample
     │
     ▼
Observation event
     │
     ▼
Planning activation
     │
     ▼
Decision
     │
     ▼
Control activation
     │
     ▼
Actuator command
```

Each step carries timestamps and causal references.

# 33. Deterministic ordering

Two events may have identical timestamps.

Therefore NROS should not rely exclusively on time for ordering.

Use:

```text
timestamp
+
sequence
+
causal relation
```

For example:

```text
event_time = 100
sequence = 42
```

# 34. Distributed clocks

Robots may have:

```text
CPU clock
MCU clock
sensor clock
camera clock
network clock
```

Clock synchronization can never be assumed perfect.

NROS should therefore expose clock quality:

```text
ClockStatus
├── source
├── uncertainty
├── offset
├── drift
└── synchronization state
```

# 35. Temporal uncertainty

Instead of assuming:

```text
timestamp = exact
```

the runtime may represent:

```text
timestamp ± uncertainty
```

For safety-sensitive systems, this can influence authorization.

# 36. Clock domains

A component can declare:

```text
clock_domain = SENSOR_A
```

while another uses:

```text
clock_domain = MONOTONIC
```

Conversion requires an explicit mapping.

# 37. Simulation time

Simulation should be a first-class clock source:

```text
Clock
├── wall
├── monotonic
├── simulated
└── external
```

This enables deterministic replay.

# 38. Replay

If execution is recorded with:

```text
state
events
time
randomness
inputs
decisions
```

NROS can replay a scenario:

```text
recorded timeline
       ↓
deterministic clock
       ↓
re-execution
```

This is considerably more powerful than simple message playback.

# 39. Temporal replay

A replay engine can reproduce:

```text
T0 sensor
T1 perception
T2 planning
T3 authorization
T4 control
T5 actuator command
```

and compare:

```text
original execution
vs
replayed execution
```

# 40. Temporal observability

The runtime should expose:

```text
period
jitter
latency
deadline misses
queue delay
CPU execution time
resource wait
network delay
clock offset
```

This gives operators a temporal health picture.

# 41. Temporal budget chain

Consider:

```text
Sensor → Planner → Controller
```

with:

```text
end-to-end deadline = 20ms
```

The runtime can distribute the budget:

```text
Sensor:     3ms
Transport:  2ms
Planner:    8ms
Controller: 4ms
Margin:     3ms
```

This becomes a compositional temporal contract.

# 42. Queue delay

A component may execute quickly but wait too long in a queue.

Therefore:

```text
total latency
=
queue delay
+
execution time
+
transport delay
+
synchronization delay
```

NROS should expose these separately.

# 43. Temporal backpressure

If producers are faster than consumers:

```text
Producer
   ↓↓↓↓↓
Queue
   ↓
Consumer
```

the queue grows.

NROS can apply:

```text
drop-oldest
drop-newest
coalesce
sample
throttle
block
reject
```

according to message semantics.

# 44. Temporal semantics of sensor data

Different data types require different strategies.

For:

```text
camera frames
```

dropping an old frame may be acceptable.

For:

```text
safety stop
```

dropping the message may be unacceptable.

Thus communication policy becomes:

```text
Data Type
+
Temporal Contract
+
Safety Criticality
```

# 45. Temporal resource allocation

The Resource Fabric can expose temporal resources:

```text
CPU time
GPU time
DMA bandwidth
network bandwidth
sensor slots
actuator windows
```

The scheduler can reason about them.

# 46. Temporal admission control

An activation can be rejected before execution:

```text
requested:
    5ms CPU
deadline:
    8ms

available:
    2ms
```

Result:

```text
TEMPORALLY_INFEASIBLE
```

This is better than accepting work and discovering failure after the deadline.

# 47. Temporal policy

The Policy Fabric can express rules such as:

```text
motor.command:
    deadline ≤ 10ms
    stale_state ≤ 100ms
```

or:

```text
camera.capture:
    maximum_rate = 30Hz
```

Thus temporal constraints become enforceable policy.

# 48. Temporal supervision

The Supervision Fabric observes:

```text
deadline misses
jitter violations
clock faults
stale state
budget overruns
queue starvation
```

and can trigger recovery.

Example:

```text
Planner deadline violation
        ↓
3 consecutive misses
        ↓
degrade planner
        ↓
switch fallback strategy
```

# 49. Temporal failure domains

A timing failure can have a scope:

```text
single activation
      ↓
component
      ↓
pipeline
      ↓
control domain
      ↓
robot
```

This is analogous to ordinary fault containment.

# 50. Temporal health

Component health can therefore include:

```text
Health
├── process
├── semantic
├── resource
├── dependency
└── temporal
```

A component might be:

```text
HEALTHY
```

but:

```text
TEMPORALLY_DEGRADED
```

if its latency exceeds contract limits.

# 51. Temporal invariants

NROS can define invariants such as:

```text
Effect(E)
requires:
    now <= deadline(E)
```

or:

```text
Use(State S)
requires:
    age(S) <= max_age(S)
```

or:

```text
LeaseUse(L)
requires:
    now < expiration(L)
```

These are runtime-checkable properties.

# 52. Temporal correctness

The system can now distinguish:

```text
Logical correctness
```

from:

```text
Temporal correctness
```

A planner result may be logically correct:

```text
path = valid
```

but temporally invalid:

```text
path_age = 2 seconds
```

NROS should reject stale decisions when policy requires it.

# 53. Temporal authority

This produces an important principle:

> **Authority is not sufficient if the authority has become temporally invalid.**

For example:

```text
Capability = valid
Lease = valid
Policy = valid
```

but:

```text
state = stale
```

then:

```text
effect = DENIED
```

# 54. Unified admission predicate

The NROS admission decision becomes:

```text
ADMIT(E)
iff
    IdentityValid
 ∧  CapabilityValid
 ∧  PolicyAllows
 ∧  StateValid
 ∧  ResourceAvailable
 ∧  TemporalContractFeasible
 ∧  DependenciesHealthy
```

This is becoming the central runtime invariant.

# 55. Temporal execution pipeline

The complete pipeline is now:

```text
Intent
  ↓
Identity
  ↓
Capability
  ↓
Policy
  ↓
State
  ↓
Resource
  ↓
Temporal feasibility
  ↓
Admission
  ↓
Scheduling
  ↓
Execution
  ↓
Effect
  ↓
Observation
  ↓
Supervision
```

# 56. NROS runtime equation

We can express the emerging architecture as:

```text
Intent
    ↓
Governance
    ↓
Admission
    ↓
Temporal Scheduling
    ↓
Execution
    ↓
Physical Effect
    ↓
Observation
    ↓
State Update
    ↓
Supervision
    ↓
Recovery
```

The loop closes.

# 57. ROS → NROS: another fundamental shift

ROS largely organizes:

```text
processes + messages
```

NROS organizes:

```text
entities
+
state
+
authority
+
resources
+
time
+
execution
+
failure
```

Communication remains essential, but it is no longer the entire semantic center.

# 58. Toward a true runtime model

At this point NROS is no longer best described as:

> "ROS rewritten in Rust."

A stronger description is:

> **NROS is a governed distributed execution runtime for autonomous cyber-physical systems.**

ROS compatibility can remain a boundary/interface concern.

The internal model can be substantially richer.

# 59. NROS core semantic objects

We now have a growing canonical object vocabulary:

```text
Identity
Capability
Policy
Component
State
Event
Resource
Lease
Intent
Activation
TemporalContract
Failure
Incident
RecoveryPlan
```

These objects should become the basis of the NROS protocol and runtime APIs.

# 60. The next missing dimension

We have now modeled:

```text
WHO       → Identity
WHAT      → Intent
MAY       → Capability/Policy
STATE     → State Fabric
RESOURCE  → Resource Fabric
WHEN      → Temporal Fabric
EXECUTE   → Execution Fabric
FAILURE   → Supervision Fabric
```

There is one major question left:

> **How do autonomous components reason, coordinate, and make decisions without turning the runtime into an opaque centralized controller?**

That leads to the next layer.

# Part XXXV — NROS Agents, Intent, Planning & Decision Execution

The next transformation will introduce:

```text
Observation
    ↓
Belief / State
    ↓
Goal
    ↓
Intent
    ↓
Plan
    ↓
Plan Validation
    ↓
Capability Check
    ↓
Resource Reservation
    ↓
Temporal Admission
    ↓
Execution
    ↓
Observation
    ↓
Replanning
```

This is where the architecture moves from a **robot middleware** toward an **agent-native robotic runtime**.
