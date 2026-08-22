# NROS Lifecycle & Packaging (Part XXXV–XL)

The next step is the most important conceptual transition:

> **ROS coordinates software components. NROS should coordinate autonomous decision-making while keeping every physical effect governed by explicit runtime contracts.**

The objective is **not** to turn every ROS node into an LLM agent.

Instead, NROS should provide a deterministic substrate on which autonomous agents can operate safely.

# 1. From computation graph to decision graph

Traditional ROS:

```text
Sensor
  ↓
Node
  ↓
Topic
  ↓
Node
  ↓
Actuator
```

NROS:

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
Authorization
    ↓
Resource Reservation
    ↓
Temporal Admission
    ↓
Execution
    ↓
Effect
    ↓
Observation
```

The graph is therefore no longer only a **communication graph**.

It becomes a **decision-execution graph**.

# 2. Agent is not a process

This distinction is fundamental.

A process is:

```text
runtime entity
```

An agent is:

```text
decision-making entity
```

One process may host:

```text
Agent A
Agent B
Agent C
```

and one agent may span:

```text
multiple processes
multiple components
multiple execution contexts
```

Therefore:

```text
Agent ≠ Process
```

# 3. Agent identity

An agent needs an identity:

```text
AgentIdentity
├── agent_id
├── owner
├── authority domain
├── capabilities
├── policy context
├── lifecycle
└── provenance
```

Example:

```text
agent://robot/navigation
```

or:

```text
agent://fleet/mission-planner
```

# 4. Agent lifecycle

Agents should have explicit lifecycle states:

```text
CREATED
 ↓
INITIALIZING
 ↓
READY
 ↓
ACTIVE
 ↓
PAUSED
 ↓
DEGRADED
 ↓
RECOVERING
 ↓
STOPPED
```

This reuses the lifecycle and supervision semantics already established.

# 5. Agent perception

An agent does not necessarily consume raw sensor data directly.

Instead:

```text
Sensors
   ↓
Drivers
   ↓
Perception
   ↓
Observations
   ↓
Agent
```

This separates:

```text
physical sensing
```

from:

```text
decision reasoning
```

# 6. Observation

An observation is structured evidence:

```text
Observation
├── observation_id
├── source
├── timestamp
├── validity
├── confidence
├── provenance
├── payload
└── semantic type
```

For example:

```text
ObstacleDetected
{
    location: ...
    confidence: 0.94
    source: lidar
}
```

# 7. Observation is not truth

This is critical for autonomous systems.

An observation represents:

```text
what was measured
```

not necessarily:

```text
what is true
```

Therefore:

```text
Observation
    ↓
Interpretation
    ↓
Belief
```

# 8. Belief state

An agent may maintain:

```text
BeliefState
├── world model
├── robot state
├── mission state
├── uncertainty
├── temporal validity
└── provenance
```

For example:

```text
RobotPose:
    x = ...
    y = ...
    confidence = 0.92
    age = 43ms
```

# 9. Belief revision

New observations update beliefs:

```text
B₀
 ↓
Observation O₁
 ↓
B₁
 ↓
Observation O₂
 ↓
B₂
```

The runtime should preserve the transition history.

This allows reasoning about:

> Why does the agent currently believe this?

# 10. Goals

A goal describes a desired state:

```text
Goal
├── goal_id
├── objective
├── priority
├── constraints
├── deadline
├── authority
└── termination condition
```

Example:

```text
Goal:
Reach waypoint W
before 15:30
while maintaining safety constraints.
```

# 11. Goal ≠ command

A command says:

```text
drive forward
```

A goal says:

```text
reach destination
```

This distinction gives the runtime room for planning and adaptation.

# 12. Intent

Intent is the agent's committed interpretation of what it wants to achieve.

```text
Goal
  ↓
Intent
```

Example:

```text
Goal:
Inspect warehouse sector B

Intent:
Navigate to sector B,
scan shelves,
record anomalies,
return to staging area.
```

Intent becomes the bridge between:

```text
high-level objective
```

and:

```text
executable plan
```

# 13. Intent lifecycle

```text
PROPOSED
 ↓
VALIDATING
 ↓
AUTHORIZED
 ↓
ADMITTED
 ↓
ACTIVE
 ↓
COMPLETED
```

Alternative:

```text
PROPOSED
 ↓
REJECTED
```

or:

```text
ACTIVE
 ↓
REVOKED
```

# 14. Intent is not authority

An agent saying:

```text
"I want to move the robot."
```

does not grant permission.

The runtime separately evaluates:

```text
Intent
+
Identity
+
Capability
+
Policy
+
State
+
Resources
+
Time
```

Only then can an effect occur.

# 15. Planning

The planner transforms:

```text
Intent
```

into:

```text
Plan
```

A plan consists of executable steps:

```text
Plan
├── step A
├── step B
├── step C
└── completion criteria
```

# 16. Plan example

```text
Goal:
Inspect room B

Plan:
1. localize
2. navigate to room B
3. stop
4. activate camera
5. scan area
6. classify objects
7. record observations
8. return
```

The important point:

**The plan itself is not yet execution authority.**

# 17. Plan validation

Before execution:

```text
Plan
 ↓
Structural validation
 ↓
Capability validation
 ↓
Policy validation
 ↓
State validation
 ↓
Resource validation
 ↓
Temporal validation
 ↓
Safety validation
 ↓
ADMITTED
```

This unifies the previous fabrics.

# 18. Plan as a graph

Plans should not necessarily be linear lists.

They can be graphs:

```text
        Localize
           ↓
       Navigate
        /     \
       /       \
   Camera    LiDAR
       \       /
        \     /
         Analyze
            ↓
          Return
```

This enables parallelism.

# 19. Plan dependencies

Each step can specify:

```text
requires:
    localization.ready
```

and:

```text
produces:
    pose.valid
```

The planner therefore operates over semantic dependencies.

# 20. Plan resources

A step can require resources:

```text
Navigate:
    requires:
        motor_control
        localization
        map
```

The Resource Fabric evaluates availability.

# 21. Plan temporal contracts

A plan can specify:

```text
overall deadline = 5 min
```

with step constraints:

```text
localize:
    deadline = 500ms

navigate:
    deadline = 3min

scan:
    deadline = 30s
```

The Temporal Fabric determines whether the plan is feasible.

# 22. Plan authority

A plan may require multiple capabilities:

```text
navigate
camera.capture
object.inspect
record.observation
```

The agent may possess some but not all.

The runtime must reject plans containing unauthorized effects.

# 23. Capability-aware planning

This creates a powerful feedback loop:

```text
Goal
 ↓
Planner
 ↓
Available capabilities
 ↓
Available resources
 ↓
Policies
 ↓
Feasible plan
```

The planner should ideally avoid generating plans that cannot be executed.

# 24. Resource-aware planning

Suppose:

```text
battery = 12%
```

The agent should not plan:

```text
20-minute mission
```

if policy requires:

```text
minimum return reserve = 15%
```

The Resource Fabric therefore becomes an input to planning.

# 25. Temporal-aware planning

Likewise:

```text
deadline = 30 seconds
```

and:

```text
estimated plan = 47 seconds
```

should produce:

```text
PLAN_INFEASIBLE
```

rather than discovering the problem halfway through execution.

# 26. Policy-aware planning

A plan may be technically executable but prohibited.

Example:

```text
Goal:
enter restricted area
```

Planner:

```text
path exists
```

Policy:

```text
area access denied
```

Result:

```text
PLAN_REJECTED
```

# 27. Plan alternatives

An autonomous runtime should support alternatives:

```text
Plan A
Plan B
Plan C
```

with ranking:

```text
Plan A:
optimal
requires LiDAR

Plan B:
slower
camera-only

Plan C:
manual fallback
```

If LiDAR fails:

```text
A → invalid
B → selected
```

# 28. Planning under uncertainty

A plan should carry uncertainty:

```text
Plan
├── expected cost
├── confidence
├── assumptions
├── risks
└── fallback
```

For example:

```text
assumption:
localization confidence > 0.8
```

If this becomes false:

```text
plan invalidated
```

# 29. Plan assumptions

This is especially important.

A plan can explicitly declare:

```text
Assumptions
────────────
battery > 20%
localization healthy
motor controller healthy
map version = X
environment unchanged
```

NROS can continuously monitor these assumptions.

# 30. Plan validity

Therefore:

```text
PlanValid =
AssumptionsValid
∧
AuthorizationValid
∧
ResourcesValid
∧
TemporalConstraintsValid
∧
DependenciesHealthy
```

If false:

```text
pause
revalidate
replan
or abort
```

# 31. Plan execution

Execution is activation-driven:

```text
Plan Step
   ↓
Activation
   ↓
Admission
   ↓
Scheduler
   ↓
Executor
   ↓
Effect
```

This preserves the separation between:

```text
planning
```

and:

```text
execution
```

# 32. No direct planner-to-actuator path

A crucial invariant:

```text
Planner
   X
   │
   └──────────────→ Motor
```

should not bypass runtime governance.

Instead:

```text
Planner
   ↓
Intent / Plan
   ↓
Runtime Admission
   ↓
Control Component
   ↓
Actuator
```

# 33. Agentic execution

An agent therefore operates as:

```text
OBSERVE
   ↓
UPDATE BELIEF
   ↓
EVALUATE GOAL
   ↓
FORM INTENT
   ↓
PLAN
   ↓
VALIDATE
   ↓
EXECUTE
   ↓
OBSERVE RESULT
   ↓
REFLECT
   ↓
CONTINUE / REPLAN
```

This is the core agent loop.

# 34. Reflection

Reflection should not mean unrestricted self-modification.

It should mean:

```text
evaluate outcome
```

against:

```text
expected outcome
```

Example:

```text
Expected:
arrive at waypoint

Observed:
position error = 2.4m
```

The agent decides:

```text
replan
```

# 35. Runtime versus agent reasoning

NROS should keep a strict boundary:

```text
Agent
    proposes
       ↓
Runtime
    validates
       ↓
Runtime
    executes
```

The agent does not become the ultimate authority.

This is essential for safety.

# 36. AI planner integration

An AI/LLM planner can produce:

```text
PlanProposal
```

but not directly:

```text
physical effect
```

Architecture:

```text
LLM / Agent
     ↓
Plan Proposal
     ↓
Schema Validation
     ↓
Policy Engine
     ↓
Capability Check
     ↓
Resource Check
     ↓
Temporal Check
     ↓
Safety Check
     ↓
Execution
```

# 37. Untrusted reasoning

This allows NROS to treat model-generated reasoning as potentially untrusted.

For example:

```text
LLM says:
"Move arm to position X."
```

The runtime asks:

```text
Is the operation authorized?
Is X within limits?
Is the arm available?
Is the state current?
Is the trajectory valid?
Is the timing acceptable?
```

Only after validation does the action become executable.

# 38. Agent sandbox

Agents can receive a restricted execution environment:

```text
Agent
├── observation access
├── planning APIs
├── simulation
├── memory
├── tool capabilities
└── restricted effect APIs
```

They do not automatically receive:

```text
raw hardware authority
```

# 39. Tool invocation

An agent may invoke tools:

```text
Agent
 ├── map.query
 ├── localization.query
 ├── camera.capture
 ├── planner.solve
 └── actuator.request
```

Each tool is a governed capability.

# 40. Tool ≠ capability

A tool is an interface.

A capability is authority.

Therefore:

```text
Tool:
camera.capture()
```

does not imply:

```text
Capability:
camera.capture
```

The runtime still checks authorization.

# 41. Agent memory

Agents may maintain:

```text
short-term state
episodic memory
semantic knowledge
mission context
```

But memory must have provenance.

Example:

```text
Memory:
"door B is closed"

source:
observation O-421

timestamp:
T

confidence:
0.91
```

# 42. Memory freshness

Memory can expire:

```text
door_state:
valid_for = 30s
```

After expiration:

```text
STALE
```

The agent must re-observe before relying on it.

This connects Agent Memory directly to the Temporal Fabric.

# 43. Multi-agent NROS

Once agents become first-class entities, multiple agents can cooperate:

```text
Mission Agent
      │
 ┌────┴────┐
 ▼         ▼
Nav Agent  Perception Agent
      │
      ▼
Control Agent
```

But all still operate through the same runtime governance.

# 44. Agent negotiation

Agents may negotiate:

```text
Agent A:
"I need camera access."

Agent B:
"I currently own camera."

Runtime:
"Agent B lease expires in 4 seconds."

Scheduler:
"Agent A can wait or use alternative sensor."
```

This is resource-aware multi-agent coordination.

# 45. Agent-to-agent communication

Communication should carry semantic metadata:

```text
AgentMessage
├── sender
├── receiver
├── intent
├── correlation
├── priority
├── deadline
├── authorization context
└── payload
```

Not merely:

```text
bytes
```

# 46. Multi-agent conflict

Suppose:

```text
Agent A:
move robot to warehouse

Agent B:
move robot to charging station
```

The runtime must resolve:

```text
conflict
```

using:

```text
priority
authority
mission policy
resource ownership
safety constraints
```

rather than whichever message arrived first.

# 47. Mission hierarchy

NROS can model:

```text
Mission
 ↓
Objective
 ↓
Intent
 ↓
Plan
 ↓
Activation
 ↓
Effect
```

This gives traceability from:

```text
high-level mission
```

to:

```text
physical action
```

# 48. End-to-end provenance

A physical command should be traceable:

```text
MotorCommand
    ↑
ControlActivation
    ↑
PlanStep
    ↑
Plan
    ↑
Intent
    ↑
Goal
    ↑
Mission
```

And also:

```text
MotorCommand
    ↓
PhysicalEffect
    ↓
Observation
```

This creates a complete causal loop.

# 49. The NROS decision ledger

We can therefore introduce:

```text
DecisionRecord
```

containing:

```text
DecisionRecord
├── decision_id
├── agent
├── goal
├── intent
├── plan
├── assumptions
├── evidence
├── policies
├── capabilities
├── resources
├── temporal constraints
├── decision
├── execution result
└── provenance
```

This becomes extremely valuable for debugging and auditing.

# 50. Decision ≠ execution

An agent can decide:

```text
"Navigate to B."
```

while execution can still fail:

```text
motor unavailable
```

Therefore:

```text
Decision
```

and:

```text
Effect
```

must remain separate events.

# 51. Decision outcome

Every decision should eventually resolve as:

```text
EXECUTED
PARTIALLY_EXECUTED
REJECTED
CANCELLED
EXPIRED
FAILED
SUPERSEDED
```

This creates precise lifecycle semantics.

# 52. Replanning

When reality diverges from the plan:

```text
Plan
 ↓
Execution
 ↓
Unexpected observation
 ↓
Plan invalid
 ↓
Replanning
```

The old plan should not simply continue invisibly.

It becomes:

```text
SUPERSEDED
```

and a new plan obtains its own identity.

# 53. Plan versioning

Example:

```text
plan-42.v1
```

becomes:

```text
plan-42.v2
```

after replanning.

The runtime can then answer:

> Which plan actually produced this actuator command?

# 54. Plan lineage

```text
Plan v1
   │
   └── invalidated by Observation O7
          ↓
       Plan v2
          │
          └── completed
```

This provides decision lineage.

# 55. Agent supervision

Agents themselves are supervised.

A malfunctioning agent can:

```text
loop endlessly
generate invalid plans
consume excessive resources
issue repeated rejected requests
```

NROS should detect these patterns.

# 56. Agent behavioral health

Agent health may include:

```text
planning latency
plan rejection rate
resource consumption
replanning frequency
failure rate
policy violation rate
decision confidence
```

An agent may therefore become:

```text
DEGRADED
```

without its process crashing.

# 57. Agent quarantine

A problematic agent can be isolated:

```text
Agent
 ↓
behavior anomaly
 ↓
QUARANTINED
```

It may retain:

```text
observation access
```

while losing:

```text
effect capabilities
```

This is much safer than merely killing the process.

# 58. Agent trust levels

NROS can distinguish:

```text
TRUSTED
VERIFIED
RESTRICTED
UNTRUSTED
QUARANTINED
```

Trust should never automatically imply unlimited authority.

# 59. The complete autonomous loop

We now have:

```text
┌──────────────────────────────────────────────┐
│                  NROS                        │
│                                              │
│  Observe                                     │
│     ↓                                        │
│  State / Belief                              │
│     ↓                                        │
│  Goal                                        │
│     ↓                                        │
│  Intent                                      │
│     ↓                                        │
│  Plan                                        │
│     ↓                                        │
│  Governance                                  │
│     ↓                                        │
│  Admission                                   │
│     ↓                                        │
│  Temporal Scheduling                         │
│     ↓                                        │
│  Execution                                   │
│     ↓                                        │
│  Physical Effect                             │
│     ↓                                        │
│  Observe                                     │
│     ↓                                        │
│  Supervise                                   │
│     ↓                                        │
│  Recover / Replan                            │
│     └────────────────────────────────────────┘
```

This is the defining NROS execution cycle.

# 60. The architectural boundary

The most important rule is:

> **Agents may propose; the runtime decides whether effects are admissible.**

Therefore:

```text
Agent intelligence
        ≠
Runtime authority
```

This permits sophisticated AI while preserving deterministic governance.

# 61. NROS versus ROS

The transformation is now clearer:

| ROS | NROS |
|---|---|
| Node | Component / Agent |
| Topic | Typed Channel |
| Service | Operation |
| Action | Activation |
| Parameter | Versioned State |
| Launch | Deployment |
| Master/Discovery | Runtime Graph |
| Executor | Scheduler |
| Callback | Activation |
| Package | Module |
| Driver | Device Capability |
| rosbag | Event/Trace Archive |
| Lifecycle | Governed Lifecycle |
| — | Identity |
| — | Capabilities |
| — | Policy |
| — | Resources |
| — | Temporal Contracts |
| — | Supervision |
| — | Intent |
| — | Planning |
| — | Decision Provenance |

The final rows are where NROS moves beyond conventional robotics middleware.

# 62. The NROS stack

The conceptual stack now becomes:

```text
┌──────────────────────────────────────────┐
│ Mission / Multi-Agent Layer              │
├──────────────────────────────────────────┤
│ Agent / Intent / Planning Layer          │
├──────────────────────────────────────────┤
│ Governance / Policy Layer                │
├──────────────────────────────────────────┤
│ Temporal / Scheduling Layer               │
├──────────────────────────────────────────┤
│ Execution / Activation Layer             │
├──────────────────────────────────────────┤
│ Resource / Lease Layer                   │
├──────────────────────────────────────────┤
│ State / Event Layer                      │
├──────────────────────────────────────────┤
│ Communication / Transport Layer          │
├──────────────────────────────────────────┤
│ Device / Hardware Layer                  │
└──────────────────────────────────────────┘
```

And surrounding everything:

```text
        Supervision
            │
            ▼
       ┌─────────┐
       │   NROS  │
       └─────────┘
            ▲
            │
       Observability
```

# 63. The next problem: physical resources

Planning and agents expose the next fundamental issue.

An autonomous system does not execute against abstract software alone.

It acts upon scarce physical resources:

```text
motors
cameras
LiDAR
GPS
CPU
GPU
memory
battery
network
storage
manipulator joints
radio channels
```

Therefore NROS needs a much deeper resource model.

The next section should establish:

# **Part XXXVI — NROS Resource Model: Ownership, Leases, Reservations, Arbitration & Physical Effects**

We will derive:

```text
Resource
    ↓
Capability
    ↓
Ownership
    ↓
Lease
    ↓
Reservation
    ↓
Arbitration
    ↓
Activation
    ↓
Physical Effect
    ↓
Release / Reconciliation
```

and distinguish the crucial concepts:

```text
"Can use"
vs
"Owns"
vs
"Reserved"
vs
"Currently controlling"
vs
"Physically affecting"
```

That distinction will become the foundation for safe multi-agent and multi-component control.

# NROS — Part XXXVI: Resource Model, Ownership, Leases & Physical Effects

The previous section established **agents and decision execution**.

Now we need to answer a harder question:

> **When multiple components or agents want to affect the same physical system, who gets to use the resource, under what conditions, for how long, and with what guarantees?**

This is where NROS moves from merely coordinating computation to **governing physical effects**.

# 1. Resource is a first-class object

In ROS, resources are usually implicit.

A node opens:

```text
/dev/ttyUSB0
```

or publishes:

```text
/cmd_vel
```

and the application is responsible for coordination.

NROS should make the resource explicit:

```text
Resource
├── identity
├── type
├── owner
├── capabilities
├── state
├── availability
├── constraints
├── lease
└── safety policy
```

# 2. Resource taxonomy

NROS resources can be divided into:

```text
Physical
├── actuator
├── sensor
├── manipulator
└── power source

Computational
├── CPU
├── GPU
├── memory
└── accelerator

Communication
├── network
├── radio
├── channel
└── bandwidth

Logical
├── map
├── localization
├── database
└── mission state
```

# 3. Resource identity

Every resource should have a stable identity:

```text
resource://robot/base/motor-left
resource://robot/base/motor-right
resource://robot/sensors/lidar
resource://robot/compute/gpu
```

This allows capabilities and policies to reference resources without depending on process IDs.

# 4. Resource capability

A resource exposes capabilities.

For example:

```text
Motor
├── velocity_control
├── position_control
├── stop
└── emergency_stop
```

A camera:

```text
Camera
├── capture
├── stream
├── configure
└── diagnostics
```

Thus:

```text
Resource
    ↓
Capabilities
```

# 5. Capability is not ownership

An agent may have:

```text
capability:
    camera.capture
```

without owning the camera.

Therefore:

```text
Capability ≠ Ownership
```

This distinction is foundational.

# 6. Capability is authorization

Capability answers:

> **May this entity perform this operation?**

Ownership answers:

> **Who currently controls the resource?**

These are different questions.

# 7. Resource states

A resource can have an explicit lifecycle:

```text
AVAILABLE
RESERVED
LEASED
ACTIVE
DEGRADED
FAULTED
QUARANTINED
OFFLINE
```

Transitions must be governed.

# 8. Ownership

Ownership represents control authority over a resource.

Example:

```text
motor-controller-agent
    owns
base-motion-resource
```

But ownership should not necessarily be permanent.

That leads to leases.

# 9. Lease

A lease is:

> **Time-bounded control authority over a resource.**

Example:

```text
Lease
├── resource
├── holder
├── acquired_at
├── expires_at
├── capabilities
├── priority
└── revocation policy
```

# 10. Why leases matter

Suppose an agent crashes while controlling a motor.

Without leases:

```text
Agent crashes
     ↓
motor remains logically owned
```

Potentially dangerous.

With a lease:

```text
Agent crashes
     ↓
lease expires
     ↓
runtime detects expiration
     ↓
resource enters safe state
```

# 11. Lease expiration

Expiration should be explicit:

```text
LeaseActive(t)
iff:

acquired_at ≤ t < expires_at
```

At:

```text
t >= expires_at
```

the lease is invalid.

No new effect should be authorized under that lease.

# 12. Lease renewal

A lease can be renewed:

```text
ACTIVE
   ↓
RENEW
   ↓
ACTIVE
```

But renewal itself requires authority.

An untrusted component must not be able to extend its own authority indefinitely.

# 13. Lease heartbeat

Critical resources can require:

```text
heartbeat
```

Example:

```text
lease TTL = 500 ms
heartbeat = every 100 ms
```

If heartbeats stop:

```text
lease → expired
```

# 14. Grace periods

Some resources may require controlled shutdown:

```text
lease expires
    ↓
grace period
    ↓
safe transition
    ↓
release
```

For a motor this might mean:

```text
stop command
→ deceleration
→ zero velocity
→ release
```

rather than abruptly removing control.

# 15. Reservation

Reservation is different from ownership.

Reservation means:

> **This resource is promised for a future operation.**

Example:

```text
Agent A
reserves
GPU
for
T + 10s → T + 30s
```

The GPU may still be available now.

# 16. Reservation versus lease

```text
Reservation:
    future right

Lease:
    current bounded authority

Ownership:
    current control relationship
```

These should not be conflated.

# 17. Resource scheduling

A resource scheduler can therefore maintain:

```text
Timeline
─────────────────────────────→

Agent A   [reservation]
Agent B             [lease]
Agent C                       [reservation]
```

This allows planned resource allocation.

# 18. Resource contention

Suppose:

```text
Agent A → wants camera
Agent B → wants camera
```

NROS must arbitrate.

Inputs include:

```text
priority
criticality
mission
deadline
authority
current lease
resource policy
```

# 19. Arbitration

Arbitration produces:

```text
GRANT
WAIT
PREEMPT
DENY
DEGRADE
```

For example:

```text
Agent A:
inspection

Agent B:
safety verification

Safety verification
    > inspection

Result:
B = GRANT
A = WAIT
```

# 20. Priority is not enough

A naïve system says:

```text
priority = 10
```

wins over:

```text
priority = 5
```

NROS should use richer semantics:

```text
ArbitrationScore =
authority
+
criticality
+
deadline
+
mission priority
+
safety
+
resource policy
```

The exact algorithm can vary by deployment.

# 21. Preemption

Sometimes the current holder must be interrupted.

Example:

```text
teleoperation
```

is controlling a robot when:

```text
emergency-stop
```

arrives.

The runtime should support:

```text
current lease
     ↓
preempt
     ↓
safe transition
     ↓
emergency authority
```

# 22. Preemption authority

Not every component may preempt every other component.

Example:

```text
debug agent
    X
preempt
safety controller
```

while:

```text
emergency controller
    ✓
preempt
motion planner
```

Preemption itself is therefore a capability.

# 23. Resource hierarchy

Resources can contain subresources.

Example:

```text
robot
└── arm
    ├── joint-1
    ├── joint-2
    ├── joint-3
    └── gripper
```

A lease on:

```text
arm
```

may imply control over:

```text
joint-1..3
gripper
```

depending on policy.

# 24. Composite resources

Some operations require multiple resources simultaneously.

Example:

```text
Manipulation
requires:
    arm
    gripper
    camera
```

The runtime must avoid partial acquisition.

Otherwise:

```text
arm = acquired
camera = acquired
gripper = unavailable
```

can create deadlock or inconsistent state.

# 25. Atomic resource acquisition

NROS should support:

```text
AcquireSet(
    arm,
    gripper,
    camera
)
```

with semantics:

```text
ALL
```

or:

```text
NONE
```

when atomic acquisition is required.

# 26. Resource deadlock

Consider:

```text
Agent A:
holds arm
waits for camera

Agent B:
holds camera
waits for arm
```

Classic deadlock.

NROS can prevent this through:

```text
resource ordering
transactional acquisition
timeouts
deadlock detection
preemption
```

# 27. Resource transactions

A multi-resource operation can be modeled as:

```text
BEGIN
 ↓
RESERVE
 ↓
ACQUIRE
 ↓
EXECUTE
 ↓
COMMIT
 ↓
RELEASE
```

Failure:

```text
BEGIN
 ↓
RESERVE
 ↓
FAIL
 ↓
ROLLBACK
```

This is especially useful for complex physical operations.

# 28. Physical resources are not databases

However, physical effects cannot always be rolled back.

For example:

```text
motor moves 20 cm
```

cannot necessarily be undone atomically.

Therefore NROS must distinguish:

```text
logical transaction
```

from:

```text
physical transaction
```

# 29. Physical effect

An effect is an externally observable change:

```text
Effect
├── effect_id
├── initiator
├── resource
├── operation
├── requested state
├── actual state
├── timestamp
└── provenance
```

Example:

```text
Effect:
motor velocity = 0.5 m/s
```

# 30. Requested versus actual state

This is crucial.

Agent requests:

```text
velocity = 1.0 m/s
```

but actual robot state becomes:

```text
velocity = 0.82 m/s
```

because of:

```text
load
friction
controller limits
battery
```

Therefore:

```text
RequestedEffect
```

must remain distinct from:

```text
ObservedEffect
```

# 31. Effect lifecycle

```text
REQUESTED
   ↓
AUTHORIZED
   ↓
ADMITTED
   ↓
DISPATCHED
   ↓
APPLIED
   ↓
OBSERVED
```

Failure paths:

```text
REJECTED
CANCELLED
EXPIRED
FAILED
PARTIAL
```

# 32. Physical acknowledgement

A successful API call does not necessarily mean a physical effect occurred.

For example:

```text
motor.set_velocity(1.0)
```

returning:

```text
OK
```

only proves:

```text
request accepted
```

It does not prove:

```text
motor actually reached 1.0
```

NROS should distinguish these.

# 33. Effect confirmation

For critical operations:

```text
Request
 ↓
Controller acknowledgement
 ↓
Sensor confirmation
```

Only then can the runtime establish:

```text
EFFECT_CONFIRMED
```

# 34. Closed-loop control

This leads naturally to:

```text
Command
   ↓
Actuator
   ↓
Physical System
   ↓
Sensor
   ↓
Observation
   ↓
State
   ↓
Controller
   ↓
Command
```

NROS therefore treats control as a closed-loop process rather than a message pipeline.

# 35. Safety envelope

Each physical resource can expose limits:

```text
Motor
├── max_velocity
├── max_acceleration
├── max_current
└── thermal_limit
```

An effect request must satisfy:

```text
requested_value ∈ safety_envelope
```

# 36. Runtime enforcement

If an agent requests:

```text
velocity = 20 m/s
```

while:

```text
max_velocity = 5 m/s
```

the runtime should not rely on the agent to behave correctly.

It should enforce:

```text
REJECT
```

or an explicitly configured:

```text
CLAMP
```

policy.

# 37. Safety envelope hierarchy

Constraints can exist at multiple layers:

```text
Hardware limit
    ↓
Device driver limit
    ↓
Controller limit
    ↓
Runtime policy
    ↓
Mission constraint
    ↓
Agent request
```

The effective envelope is the intersection:

```text
E_effective =
E_hardware
∩ E_driver
∩ E_runtime
∩ E_mission
```

# 38. Resource state machine

A physical resource can follow:

```text
              ┌───────────┐
              │ AVAILABLE │
              └─────┬─────┘
                    ↓
                RESERVED
                    ↓
                  LEASED
                    ↓
                  ACTIVE
                 /     \
                /       \
               ↓         ↓
           DEGRADED    FAULTED
               │          │
               ↓          ↓
            RECOVERY    QUARANTINE
               │
               ↓
           AVAILABLE
```

# 39. Faulted resources

If a sensor reports:

```text
hardware fault
```

the runtime should transition:

```text
ACTIVE
 ↓
FAULTED
```

and invalidate dependent operations.

# 40. Dependency propagation

Suppose:

```text
LiDAR
 ↓
Localization
 ↓
Navigation
 ↓
Motion
```

If LiDAR fails:

```text
LiDAR = FAULTED
```

then NROS evaluates:

```text
Localization = DEGRADED
Navigation = DEGRADED
Motion = ?
```

rather than allowing stale assumptions to persist.

# 41. Resource health graph

This becomes another graph:

```text
LiDAR
  ↓
Localization
  ↓
Planner
  ↓
Controller
  ↓
Motor
```

NROS can propagate resource health through dependency relationships.

# 42. Resource substitution

A degraded resource may have alternatives.

Example:

```text
Primary:
LiDAR

Fallback:
Stereo Camera
```

The planner can therefore select:

```text
perception strategy B
```

instead of stopping the entire mission.

# 43. Resource abstraction

Applications should not necessarily depend on:

```text
/dev/ttyUSB0
```

Instead:

```text
resource://robot/sensors/lidar
```

The runtime resolves the implementation.

This restores the hardware abstraction principle in a stronger form.

# 44. Resource drivers

Drivers become resource providers:

```text
Hardware
   ↓
Driver
   ↓
Resource Interface
   ↓
NROS
```

The driver translates:

```text
hardware protocol
```

into:

```text
resource semantics
```

# 45. Device capability discovery

At runtime:

```text
discover(robot)
```

could return:

```text
Resources:
    lidar
    camera
    imu
    motors
    gpu

Capabilities:
    scan
    capture
    orientation
    velocity_control
    inference
```

This supports dynamic robot configurations.

# 46. Resource contracts

A resource can declare:

```text
ResourceContract
├── capabilities
├── limits
├── timing
├── ownership
├── safety
├── dependencies
└── failure behavior
```

Thus the runtime understands what the resource guarantees.

# 47. Resource QoS

Different operations require different quality levels.

For example:

```text
Camera:
30 FPS
latency < 50ms
```

versus:

```text
Camera:
5 FPS
latency < 500ms
```

The resource scheduler can negotiate an appropriate operating mode.

# 48. Resource negotiation

An agent can request:

```text
Camera
resolution = 1920x1080
rate = 30Hz
latency < 50ms
```

The resource manager responds:

```text
GRANTED
```

or:

```text
DEGRADED:
1280x720
20Hz
```

or:

```text
DENIED
```

This makes resource allocation explicit.

# 49. Power as a resource

Battery should be modeled explicitly:

```text
BatteryResource
├── charge
├── voltage
├── current
├── temperature
├── estimated_runtime
└── reserve
```

Now planning can reason about energy.

# 50. Energy-aware execution

A plan may have:

```text
estimated_energy = 12%
```

while:

```text
battery = 20%
return_reserve = 10%
```

Available mission energy:

```text
20 - 10 = 10%
```

Therefore:

```text
PLAN_INFEASIBLE
```

before execution.

# 51. Compute as resource

AI workloads can consume:

```text
CPU
GPU
NPU
RAM
VRAM
```

These should be schedulable resources.

Example:

```text
Vision Agent:
GPU reservation = 30%
```

while:

```text
Planner:
GPU reservation = 20%
```

# 52. Network as resource

Distributed robots also compete for:

```text
bandwidth
latency
radio channels
packet budgets
```

NROS can represent these explicitly.

# 53. Resource budgets

An agent may have:

```text
CPU budget
GPU budget
network budget
energy budget
storage budget
```

This prevents an autonomous workload from consuming the entire robot.

# 54. Resource accounting

NROS can track:

```text
Agent A
CPU: 12%
GPU: 30%
Network: 2 MB/s
Energy: 4 W

Agent B
CPU: 8%
GPU: 10%
Network: 500 KB/s
Energy: 1 W
```

This enables policy enforcement.

# 55. Resource-aware supervision

Supervision can detect:

```text
resource starvation
resource leaks
lease leaks
overconsumption
deadlocks
priority inversion
```

and initiate recovery.

# 56. Priority inversion

Example:

```text
Low-priority Agent A
holds motor lease

High-priority Agent B
needs motor

Medium-priority Agent C
consumes CPU
```

B may be indirectly blocked by A.

NROS scheduling needs mechanisms such as:

```text
priority inheritance
lease priority escalation
bounded preemption
```

where appropriate.

# 57. Resource provenance

Every physical effect should be traceable to:

```text
resource
lease
agent
intent
plan
policy
activation
```

Example:

```text
Motor effect M-9382

resource:
motor-left

lease:
L-192

agent:
navigation-agent

plan:
P-42.v3

intent:
I-77

policy:
motion-policy-v4

activation:
A-818
```

This is the beginning of **physical-effect provenance**.

# 58. Why this matters

If a robot unexpectedly moves, NROS should be able to answer:

```text
WHO?
WHAT?
WHY?
WHEN?
UNDER WHICH POLICY?
WITH WHICH CAPABILITY?
USING WHICH RESOURCE?
UNDER WHICH LEASE?
WHAT ACTUALLY HAPPENED?
```

That is far beyond ordinary middleware logging.

# 59. The NROS physical-effect invariant

We can now establish a major invariant:

> **No governed physical effect occurs without an attributable, authorized, temporally valid execution context.**

Formally:

```text
Effect(e)
⇒
Identity(e)
∧ Capability(e)
∧ Policy(e)
∧ Resource(e)
∧ Lease(e)
∧ TemporalValidity(e)
∧ Activation(e)
```

For safety-critical effects, we can strengthen it:

```text
EffectCritical(e)
⇒
SafetyValidation(e)
∧ StateValidity(e)
∧ ResourceHealth(e)
```

# 60. The complete NROS control chain

The architecture now becomes:

```text
                    ┌───────────────┐
                    │    Mission    │
                    └───────┬───────┘
                            ↓
                         Goal
                            ↓
                         Intent
                            ↓
                          Plan
                            ↓
                  ┌───────────────────┐
                  │ Runtime Governance│
                  └─────────┬─────────┘
                            ↓
                Capability / Policy
                            ↓
                    Resource Check
                            ↓
                       Lease/Reserve
                            ↓
                    Temporal Admission
                            ↓
                       Activation
                            ↓
                        Scheduler
                            ↓
                       Controller
                            ↓
                       Actuator
                            ↓
                     Physical Effect
                            ↓
                         Sensors
                            ↓
                       Observation
                            ↓
                          State
                            ↓
                    Supervision/Replan
                            └───────────→
```

This is now a coherent **agent-to-physics execution architecture**.

# 61. The next abstraction: events

We now have many entities:

```text
Observation
Intent
Plan
Activation
Lease
Reservation
Effect
State
Failure
Recovery
```

They need a common connective tissue.

That connective tissue should not merely be messages.

It should be an **event model**.

# Part XXXVII — NROS Event Fabric & Causal Execution Ledger

The next layer will define:

```text
Event
├── identity
├── type
├── source
├── subject
├── timestamp
├── sequence
├── causality
├── correlation
├── provenance
├── payload
└── integrity
```

Then we can construct:

```text
Observation
    ↓ event
Decision
    ↓ event
Authorization
    ↓ event
Resource Lease
    ↓ event
Activation
    ↓ event
Physical Effect
    ↓ event
Observation
```

This gives NROS a **causal execution ledger** capable of reconstructing not merely *what messages moved*, but **why the robot did what it did**.

# NROS — Part XXXVII: Event Fabric, Causality & the Execution Ledger

We now have the major semantic objects:

```text
Identity
Capability
Policy
State
Resource
Lease
Reservation
Goal
Intent
Plan
Activation
Effect
Observation
Failure
Recovery
```

The missing question is:

> **How does NROS connect all of these into one causally traceable runtime history?**

The answer is the **Event Fabric**.

# 1. Event becomes the universal runtime primitive

A ROS message primarily communicates information:

```text
Publisher → Topic → Subscriber
```

An NROS event represents a **state transition, decision, observation, authorization, or effect in the runtime**.

Conceptually:

```text
Event
 ├── something happened
 ├── to something
 ├── at some time
 ├── because of something
 └── with some consequence
```

# 2. Event versus message

These concepts should remain distinct.

```text
Message:
    communication payload

Event:
    occurrence in the runtime
```

A message can cause an event:

```text
SensorMessage
      ↓
ObservationCreated
```

but they are not identical.

# 3. Canonical event structure

An NROS event can be modeled as:

```text
Event
├── event_id
├── event_type
├── source
├── subject
├── timestamp
├── sequence
├── causality
├── correlation
├── actor
├── authority_context
├── payload
├── provenance
└── integrity
```

# 4. Event identity

Every event needs a unique identifier:

```text
event://robot-01/01HXYZ...
```

The exact encoding can evolve.

The important property is:

```text
event_id ≠ reusable
```

An event identity should remain stable throughout its lifetime.

# 5. Event type

Examples:

```text
ComponentStarted
ObservationCreated
GoalCreated
IntentProposed
IntentAuthorized
PlanCreated
PlanRejected
ResourceReserved
LeaseGranted
ActivationStarted
ActivationCompleted
EffectRequested
EffectApplied
EffectConfirmed
DeadlineMissed
ResourceFaulted
AgentQuarantined
RecoveryStarted
RecoveryCompleted
```

This provides a common vocabulary.

# 6. Subject

Every event concerns some subject.

For example:

```text
ObservationCreated
subject = lidar-observation-42
```

or:

```text
LeaseGranted
subject = motor-resource
```

or:

```text
EffectApplied
subject = motor-command-883
```

# 7. Actor

The event should also identify who caused it.

For example:

```text
actor = navigation-agent
```

or:

```text
actor = runtime.scheduler
```

or:

```text
actor = hardware.driver
```

This separates:

```text
who was affected
```

from:

```text
who caused the event
```

# 8. Causality

This is one of the most important NROS additions.

Suppose:

```text
ObstacleDetected
```

causes:

```text
EmergencyStopRequested
```

which causes:

```text
MotorStopActivation
```

which causes:

```text
MotorStopped
```

The event graph becomes:

```text
O1
 ↓
E1
 ↓
A1
 ↓
E2
```

NROS can preserve these causal links explicitly.

# 9. Causal parent

A simple event can contain:

```text
caused_by = event_id
```

Example:

```text
Event E2
caused_by = E1
```

This creates a causal chain.

# 10. Multiple causes

Some decisions depend on multiple observations:

```text
LiDAR observation
       \
        \
Camera observation → Planning decision
        /
IMU observation
```

Therefore an event may contain:

```text
caused_by = [
    O1,
    O2,
    O3
]
```

# 11. Correlation

Causality and correlation are different.

Causality:

```text
A caused B
```

Correlation:

```text
A and B belong to the same operation
```

Example:

```text
Mission M42
```

may correlate:

```text
Goal G1
Intent I1
Plan P1
Activations A1-A8
Effects E1-E20
```

without every event directly causing every other event.

# 12. Execution trace

A mission can therefore produce:

```text
Mission M42
 │
 ├── Goal G1
 │
 ├── Intent I1
 │
 ├── Plan P1
 │    ├── Activation A1
 │    ├── Activation A2
 │    └── Activation A3
 │
 └── Effects
      ├── E1
      ├── E2
      └── E3
```

This is an **execution trace**.

# 13. Event graph

The runtime can represent events as a directed graph:

```text
        O1
       /  \
      /    \
     O2     O3
      \     /
       \   /
         D1
         ↓
         P1
       /    \
      A1     A2
       \    /
        \  /
         E1
```

This is much richer than a simple chronological log.

# 14. Sequence numbers

Timestamps alone are insufficient.

Events should also have sequence numbers:

```text
seq = 1001
seq = 1002
seq = 1003
```

This gives deterministic local ordering.

# 15. Distributed ordering

Across machines:

```text
Robot CPU
MCU
GPU
Remote computer
```

sequence numbers from different domains cannot automatically be compared.

Therefore NROS should preserve:

```text
local sequence
+
clock domain
+
causal dependencies
```

# 16. Logical ordering

For distributed execution:

```text
A → B
```

is more meaningful than:

```text
timestamp(A) < timestamp(B)
```

when clocks differ.

Thus causal metadata is authoritative for causality.

# 17. Event envelope

A transport-independent envelope could conceptually be:

```text
EventEnvelope {
    id
    type
    source
    subject
    actor
    timestamp
    clock_domain
    sequence
    parent_events
    correlation_id
    authority_context
    payload
    integrity
}
```

The actual wire representation can be defined later.

# 18. Event payload

The envelope contains common metadata.

The payload contains event-specific information.

For example:

```text
EffectRequested {
    resource
    operation
    requested_state
    constraints
}
```

This keeps the event system extensible.

# 19. Event immutability

Once an event is committed:

```text
Event E42
```

its semantic content should not be modified.

Corrections should be represented by new events:

```text
E42
 ↓
E43 Correction
```

rather than rewriting history.

# 20. Append-only execution history

NROS should conceptually maintain:

```text
Event
Event
Event
Event
Event
...
```

as an append-oriented history.

This enables:

```text
audit
debugging
replay
forensics
verification
analytics
```

# 21. Event sourcing

This naturally enables event-sourced runtime state.

Instead of storing only:

```text
robot_state = ACTIVE
```

the runtime can derive it from:

```text
RobotCreated
LeaseGranted
ActivationStarted
EffectApplied
ActivationCompleted
...
```

The current state becomes a projection of history.

# 22. But NROS should not require pure event sourcing

This distinction is important.

The runtime can use:

```text
current state
+
event history
```

rather than requiring every component to reconstruct everything from events.

Therefore:

```text
State = operational representation
Event Log = historical representation
```

Both are useful.

# 23. State projection

For example:

```text
Events
───────
LeaseGranted
LeaseRenewed
LeaseExpired
```

project to:

```text
ResourceState
─────────────
status = AVAILABLE
holder = none
```

The projection can be rebuilt when necessary.

# 24. Event replay

Given:

```text
E1
E2
E3
...
E1000
```

NROS can replay them against a state projection.

This enables:

```text
historical reconstruction
```

without interacting with hardware.

# 25. Deterministic simulation

Replay can use:

```text
recorded time
recorded inputs
recorded events
deterministic scheduler
```

to reconstruct a scenario.

This is especially useful for:

```text
robot failures
navigation bugs
planning anomalies
race conditions
```

# 26. Replay must distinguish observation from effect

A replay system should not accidentally command hardware.

Therefore:

```text
Replay Mode
    ↓
Simulated resources
    ↓
No physical effect
```

This is a runtime safety boundary.

# 27. Event classes

A useful taxonomy:

```text
Lifecycle Events
State Events
Observation Events
Decision Events
Authorization Events
Resource Events
Execution Events
Effect Events
Failure Events
Recovery Events
Security Events
```

# 28. Lifecycle events

Examples:

```text
ComponentCreated
ComponentStarted
ComponentPaused
ComponentStopped
ComponentFailed
```

# 29. State events

Examples:

```text
StateCreated
StateUpdated
StateInvalidated
StateExpired
StateRecovered
```

# 30. Observation events

Examples:

```text
SensorObservation
ObjectDetected
PoseObserved
BatteryObserved
TemperatureObserved
```

# 31. Decision events

Examples:

```text
GoalCreated
IntentProposed
PlanGenerated
PlanSelected
PlanRejected
ReplanTriggered
```

# 32. Authorization events

Examples:

```text
CapabilityGranted
CapabilityDenied
PolicyEvaluated
PolicyViolation
AuthorizationRevoked
```

# 33. Resource events

Examples:

```text
ResourceDiscovered
ResourceReserved
ResourceLeaseGranted
ResourceLeaseRenewed
ResourceLeaseExpired
ResourceReleased
ResourceFaulted
```

# 34. Execution events

Examples:

```text
ActivationCreated
ActivationAdmitted
ActivationStarted
ActivationSuspended
ActivationCancelled
ActivationCompleted
DeadlineMissed
BudgetExceeded
```

# 35. Effect events

Examples:

```text
EffectRequested
EffectDispatched
EffectApplied
EffectObserved
EffectConfirmed
EffectFailed
```

# 36. Failure events

Examples:

```text
ComponentFault
ResourceFault
DependencyFailure
Timeout
PolicyViolation
TemporalViolation
CommunicationFailure
```

# 37. Recovery events

Examples:

```text
RecoveryStarted
FallbackSelected
ResourceReinitialized
PlanRecomputed
AgentRestarted
RecoveryCompleted
```

# 38. Security events

Examples:

```text
IdentityAuthenticated
CapabilityChecked
UnauthorizedRequest
LeaseRevoked
AgentQuarantined
IntegrityFailure
```

These become part of the same causal history.

# 39. Event severity

Events may carry severity:

```text
TRACE
DEBUG
INFO
NOTICE
WARNING
ERROR
CRITICAL
EMERGENCY
```

But severity is metadata.

It should not determine semantics by itself.

# 40. Event priority

Priority is separate:

```text
severity ≠ execution priority
```

An informational event may trigger an important state transition.

Conversely, a high-volume warning may have little immediate execution significance.

# 41. Event retention

Not every event needs infinite storage.

Policies can specify:

```text
retain:
    critical events → forever

normal events:
    30 days

high-frequency telemetry:
    1 hour
```

But critical provenance should not be silently discarded.

# 42. Event durability classes

For example:

```text
VOLATILE
BUFFERED
DURABLE
AUDIT
```

A safety event may require:

```text
AUDIT
```

while high-rate telemetry can remain:

```text
VOLATILE
```

# 43. Event streams

Events can be streamed:

```text
Event Store
    ├── /events/robot
    ├── /events/mission
    ├── /events/security
    ├── /events/resources
    └── /events/execution
```

Consumers subscribe according to needs.

# 44. Event filtering

Consumers may specify:

```text
type = Effect*
severity >= ERROR
resource = motor/*
mission = M42
agent = navigation-agent
```

This avoids forcing every consumer to process the entire event stream.

# 45. Event indexes

The runtime can index:

```text
event_id
timestamp
actor
subject
resource
mission
agent
correlation_id
causal_parent
event_type
```

This enables rapid forensic queries.

# 46. Query example

An operator asks:

> Why did motor-left stop?

NROS could trace:

```text
MotorStopped
   ↑
EmergencyStopActivation
   ↑
ObstacleDetected
   ↑
LiDARObservation
```

and return:

```text
source:
lidar-01

confidence:
0.97

policy:
safety-stop-v3

agent:
safety-agent

activation:
A-821

timestamp:
T
```

# 47. Decision explanation

The event graph allows:

> Why was the plan rejected?

Trace:

```text
PlanRejected
   ↑
TemporalValidationFailed
   ↑
EstimatedExecution = 42s
RequiredDeadline = 30s
```

This is an explanation based on runtime evidence, not an opaque AI answer.

# 48. Agent explanation

Likewise:

> Why did the agent choose Plan B?

The ledger can expose:

```text
Plan A
  rejected:
      LiDAR unavailable

Plan B
  selected:
      camera available
      deadline feasible
      policy compliant
      battery sufficient
```

This is structured decision provenance.

# 49. Event integrity

For high-assurance deployments, events may require integrity protection.

Conceptually:

```text
Event
  ↓
hash
  ↓
previous-event hash
  ↓
chain
```

This creates tamper-evident history.

# 50. Hash-linked event chains

For example:

```text
E1
 │ hash(E1)
 ▼
E2
 │ hash(E2 + hash(E1))
 ▼
E3
```

Tampering with an earlier event breaks subsequent integrity verification.

# 51. This is not necessarily blockchain

NROS does not need a blockchain.

A local append-only hash chain can provide useful tamper evidence without introducing unnecessary distributed consensus.

# 52. Signed events

For distributed systems, selected events can carry signatures:

```text
Event
├── actor
├── payload
├── signature
└── verification metadata
```

This helps establish:

```text
who emitted the event
```

rather than merely:

```text
which machine forwarded it
```

# 53. Event trust

The runtime can distinguish:

```text
VERIFIED
AUTHENTICATED
UNVERIFIED
INVALID
```

Events from untrusted sources should not automatically affect safety-critical state.

# 54. Event causality and security

Consider:

```text
UnknownSource
    ↓
FakeObservation
    ↓
BadPlan
```

If the observation fails authentication:

```text
FakeObservation
    ↓
UNTRUSTED
```

then downstream decisions can reject it.

This connects the Event Fabric to the Security Fabric.

# 55. Causal cut

Sometimes an event should not propagate beyond a boundary.

Example:

```text
UntrustedAgent
    ↓
Observation
```

The event may be recorded for diagnostics but blocked from:

```text
SafetyDecision
```

This is a **causal trust boundary**.

# 56. Event-driven supervision

Supervision can subscribe to event patterns:

```text
3 × DeadlineMissed
within 5 seconds
```

and trigger:

```text
AgentDegraded
```

Another pattern:

```text
5 × UnauthorizedRequest
within 1 minute
```

could trigger:

```text
AgentQuarantine
```

# 57. Event patterns

NROS can eventually support temporal patterns:

```text
A then B
A within 500ms of B
A repeated N times
A unless B
A and B
A before C
```

This creates a powerful supervision language.

# 58. Event-driven recovery

Example:

```text
ResourceFaulted
       ↓
DependencyDegraded
       ↓
PlanInvalidated
       ↓
FallbackPlanSelected
       ↓
ActivationStarted
```

The entire recovery process becomes traceable.

# 59. Event correlation across robots

A fleet system may produce:

```text
Robot A → Event E1
Robot B → Event E2
Robot C → Event E3
```

all correlated to:

```text
Mission M42
```

The fleet controller can reconstruct the distributed mission.

# 60. Distributed execution ledger

This produces:

```text
                 Mission M42
                      │
       ┌──────────────┼──────────────┐
       ↓              ↓              ↓
    Robot A         Robot B        Robot C
       │              │              │
      E...            E...           E...
       └──────────────┼──────────────┘
                      ↓
               Mission Outcome
```

This is the foundation for multi-robot traceability.

# 61. Event bus versus event ledger

NROS should distinguish:

```text
Event Bus
    → real-time delivery

Event Ledger
    → durable history
```

The bus answers:

> Who needs this event now?

The ledger answers:

> What happened?

# 62. Event bus architecture

```text
Producer
   ↓
Event Router
   ├── Runtime subscribers
   ├── Supervision
   ├── Observability
   └── Event Ledger
```

# 63. Backpressure

Event infrastructure must itself have temporal/resource contracts.

A telemetry flood must not block:

```text
EmergencyStop
```

Therefore event classes require priorities and isolation.

# 64. Critical event lanes

Conceptually:

```text
Critical Lane
    ↓
Safety / control events

Normal Lane
    ↓
Runtime events

Telemetry Lane
    ↓
High-volume diagnostics
```

This prevents observability traffic from interfering with control.

# 65. Event ordering guarantees

Different streams may provide different guarantees:

```text
NONE
LOCAL_ORDERED
CAUSALLY_ORDERED
TOTAL_ORDERED
```

NROS should not impose expensive total ordering when causal ordering is sufficient.

# 66. Event delivery semantics

Possible semantics:

```text
AT_MOST_ONCE
AT_LEAST_ONCE
EXACTLY_ONCE_EFFECT
```

The last one is particularly important.

NROS should be careful:

> **Exactly-once message delivery is not equivalent to exactly-once physical effect.**

A motor command can be duplicated physically even if the message transport is deduplicated.

# 67. Effect idempotency

Critical effects should therefore have identifiers:

```text
effect_id = E-884
```

A controller can detect:

```text
E-884 already applied
```

and avoid duplicate application where possible.

# 68. Effect sequence

For actuators requiring ordered commands:

```text
E1
E2
E3
```

the runtime can enforce:

```text
sequence(E2) > sequence(E1)
```

and reject stale commands.

# 69. Anti-replay protection

An old command:

```text
Effect E1
timestamp = yesterday
```

must not become valid merely because it was replayed.

NROS can validate:

```text
freshness
lease
sequence
authorization
temporal validity
```

before effect application.

# 70. Event ledger as the robot's causal memory

At this point we can define a powerful concept:

> **The NROS Event Ledger is the causal memory of the runtime.**

It records:

```text
what happened
who caused it
what it affected
when it happened
why it happened
what authority permitted it
what happened afterward
```

# 71. ROS bag versus NROS ledger

The distinction becomes:

```text
ROS bag
    records messages

NROS ledger
    records execution semantics
```

A bag may tell us:

```text
/cmd_vel = 0.5
```

The ledger can tell us:

```text
Agent:
navigation-agent

Intent:
reach waypoint W

Plan:
P-42.v3

Policy:
motion-policy-v4

Lease:
L-93

Activation:
A-771

Effect:
E-882

Observed result:
velocity = 0.48 m/s
```

That is a radically richer representation.

# 72. Causal debugging

Debugging becomes:

```text
Symptom
  ↓
Effect
  ↓
Activation
  ↓
Plan
  ↓
Intent
  ↓
Goal
  ↓
Observations
  ↓
State
```

Instead of searching through thousands of unrelated log lines.

# 73. Causal replay

Given the event graph:

```text
Observation
 → Decision
 → Plan
 → Activation
 → Effect
```

NROS can replay only the relevant causal subgraph.

This can dramatically reduce debugging complexity.

# 74. Causal slicing

For an incident:

```text
Motor overspeed
```

the runtime can compute a causal slice:

```text
MotorOverspeed
 ↑
MotorCommand
 ↑
ControllerDecision
 ↑
PlannerOutput
 ↑
ObstacleObservation
```

Unrelated events are excluded.

# 75. Incident reconstruction

A safety incident can therefore produce:

```text
Incident I-17

Timeline:
T0 obstacle observed
T1 belief updated
T2 emergency intent created
T3 policy evaluated
T4 motor stop authorized
T5 activation started
T6 motor stop applied
T7 physical stop confirmed
```

This becomes an auditable incident record.

# 76. Formal runtime property

We can now state:

> **Every safety-relevant effect must have a reconstructible causal path from an admissible decision to the observed physical outcome.**

Formally:

```text
CriticalEffect(e)
⇒
∃ path:
Observation/Goal
→ Decision
→ Authorization
→ Activation
→ Effect
→ Observation
```

If such a path does not exist:

```text
PROVENANCE_FAILURE
```

should itself be a runtime fault.

# 77. NROS architecture after the Event Fabric

The architecture is now:

```text
┌──────────────────────────────────────────────┐
│              Mission / Agents                │
├──────────────────────────────────────────────┤
│          Goal / Intent / Planning            │
├──────────────────────────────────────────────┤
│       Identity / Capability / Policy         │
├──────────────────────────────────────────────┤
│       Resource / Lease / Reservation         │
├──────────────────────────────────────────────┤
│        Temporal / Scheduling / QoS           │
├──────────────────────────────────────────────┤
│          Activation / Execution              │
├──────────────────────────────────────────────┤
│        State / Observation / Memory          │
├──────────────────────────────────────────────┤
│          Communication / Transport           │
├──────────────────────────────────────────────┤
│             Device / Hardware                │
└──────────────────────────────────────────────┘
                 ▲
                 │
          ┌──────────────┐
          │ Event Fabric │
          └──────────────┘
                 │
        ┌────────┴────────┐
        │                 │
   Supervision       Event Ledger
```

The Event Fabric cuts across the entire stack.

# 78. NROS semantic loop

The complete system can now be summarized:

```text
        ┌──────────────┐
        │  OBSERVATION │
        └──────┬───────┘
               ↓
        ┌──────────────┐
        │    STATE     │
        └──────┬───────┘
               ↓
        ┌──────────────┐
        │     GOAL     │
        └──────┬───────┘
               ↓
        ┌──────────────┐
        │    INTENT    │
        └──────┬───────┘
               ↓
        ┌──────────────┐
        │     PLAN     │
        └──────┬───────┘
               ↓
        ┌──────────────┐
        │  GOVERNANCE  │
        └──────┬───────┘
               ↓
        ┌──────────────┐
        │  RESOURCES   │
        └──────┬───────┘
               ↓
        ┌──────────────┐
        │    TIME      │
        └──────┬───────┘
               ↓
        ┌──────────────┐
        │  ACTIVATION  │
        └──────┬───────┘
               ↓
        ┌──────────────┐
        │    EFFECT    │
        └──────┬───────┘
               ↓
        ┌──────────────┐
        │  OBSERVATION │
        └──────┬───────┘
               │
               └──────────────→ REPLAN
```

And every transition emits events into the causal ledger.

# 79. The emerging NROS invariant

We can now formulate the strongest architectural rule so far:

> **No autonomous decision becomes a physical effect directly.**

Instead:

```text
Decision
   ↓
Governance
   ↓
Resource
   ↓
Temporal
   ↓
Activation
   ↓
Effect
```

And every stage is observable:

```text
Event
Event
Event
Event
Event
```

# 80. What remains

The runtime now has:

```text
✓ Identity
✓ Capabilities
✓ Policy
✓ State
✓ Agents
✓ Intent
✓ Planning
✓ Resources
✓ Ownership
✓ Leases
✓ Reservations
✓ Scheduling
✓ Temporal contracts
✓ Activation
✓ Physical effects
✓ Events
✓ Causality
✓ Provenance
✓ Supervision
```

The next major boundary is **communication itself**.

ROS's biggest historical contribution was the communication graph.

NROS should therefore now redefine that layer rather than simply copying ROS topics.

# Part XXXVIII — NROS Communication Fabric: Topics, Streams, RPC, Actions, Events & Dataflow

We will derive a unified communication model:

```text
Data Stream
Event Stream
Request / Response
Command
Action
State Synchronization
Discovery
```

and determine what should replace the old conceptual separation between:

```text
ROS Topic
ROS Service
ROS Action
ROS Parameter
ROS Master
```

with a coherent NROS protocol model based on:

```text
Identity
Schema
Semantics
QoS
Authority
Temporal Contract
Delivery
Causality
Backpressure
```

That will give us the foundation needed to map the conceptual architecture into the actual **Rust crates, protocol types, runtime APIs, executors, transports, and compatibility layers of NROS**.

# NROS — Part XXXVIII: Communication Fabric

ROS made communication the center of its architecture:

```text
Node
 ├── Topic
 ├── Service
 ├── Action
 └── Parameter Server
```

NROS should preserve the underlying insight—**distributed components need structured communication**—but replace the ROS-era communication model with one that understands:

```text
identity
authority
schema
time
resources
execution
causality
reliability
backpressure
```

The result is the **NROS Communication Fabric**.

# 1. Communication is not one primitive

A common mistake would be to create one universal transport primitive and force everything through it.

NROS should instead recognize several semantic communication patterns:

```text
                 Communication
                       │
       ┌───────────────┼────────────────┐
       ↓               ↓                ↓
    Streaming       Interaction      Synchronization
       │               │                │
   Data/Event      Request/Reply     State/Lease
       │               │
       ├── Topic      ├── RPC
       ├── Stream     ├── Command
       └── Event      └── Action
```

The transport can be shared.

The **semantics must not be**.

# 2. The fundamental NROS communication object

Instead of thinking:

```text
Topic = named pipe
```

NROS should think:

```text
Communication Contract
```

A communication contract defines:

```text
CommunicationContract
├── identity
├── source
├── destination
├── schema
├── semantics
├── QoS
├── temporal constraints
├── authorization
├── lifecycle
└── observability
```

Communication therefore becomes a governed runtime resource.

# 3. Channel identity

Every communication endpoint should have an explicit identity.

Conceptually:

```text
channel://robot-01/navigation/pose
```

But the name alone is insufficient.

The runtime also needs:

```text
channel_id
schema_id
version
owner
authority
```

# 4. Namespaces

ROS already introduced namespaces.

NROS should retain the concept but make it part of a broader identity system:

```text
/nros
  /robot
    /robot-01
      /sensors
      /control
      /navigation
      /diagnostics
```

Names are for discovery and human reasoning.

Cryptographic identity is for security.

These should not be conflated.

# 5. Endpoint identity

A communication endpoint can be modeled as:

```text
Endpoint
├── endpoint_id
├── identity
├── namespace
├── capability
├── schema
├── direction
└── lifecycle
```

For example:

```text
Endpoint:
    robot-01/lidar/front

Direction:
    publish

Schema:
    LaserScan.v2
```

# 6. Topics become streams

ROS topics are essentially asynchronous publish/subscribe channels.

NROS can generalize them as:

```text
Stream<T>
```

where:

```text
T = typed payload
```

A stream may contain:

```text
Sensor samples
State updates
Commands
Events
Telemetry
Observations
```

# 7. Data stream

Example:

```text
Stream<Pose>
```

Producer:

```text
localization-agent
```

Consumers:

```text
navigation-agent
mapping-agent
telemetry-agent
```

The producer does not need to know its consumers.

This preserves ROS's useful decoupling.

# 8. Event stream

Events use the same underlying streaming mechanism but different semantics:

```text
Stream<Event>
```

For example:

```text
/events/execution
```

contains:

```text
ActivationStarted
ActivationCompleted
EffectApplied
EffectFailed
```

Unlike ordinary telemetry, events have causal and provenance semantics.

# 9. Commands are different

A command should not simply be treated as another topic message.

Consider:

```text
MoveArm
```

A command represents an **authorized request to cause an effect**.

Therefore:

```text
Command
├── issuer
├── target
├── capability
├── authority
├── deadline
├── correlation_id
├── command_id
└── payload
```

# 10. Command lifecycle

A command can move through:

```text
Created
   ↓
Authenticated
   ↓
Authorized
   ↓
Admitted
   ↓
Dispatched
   ↓
Applied
   ↓
Confirmed
```

Failures can occur at every stage.

# 11. Request/response

ROS services provide request/response semantics.

NROS should retain this pattern:

```text
Request<T>
    ↓
Service
    ↓
Response<R>
```

But make its temporal and authorization properties explicit.

For example:

```text
GetRobotState
```

is fundamentally different from:

```text
SetMotorSpeed
```

The latter can produce a physical effect and therefore requires stronger governance.

# 12. RPC

NROS RPC can therefore represent:

```text
Client
  ↓
Request
  ↓
Authorized Service
  ↓
Response
```

The runtime should attach:

```text
request_id
deadline
caller_identity
authority_context
schema_version
```

# 13. RPC is not necessarily physical execution

This distinction matters.

```text
GetTemperature()
```

may simply read state.

But:

```text
OpenValve()
```

may trigger:

```text
Resource authorization
Activation
Physical effect
Confirmation
```

Therefore NROS should not give every RPC the same trust level.

# 14. Commands versus RPC

A useful distinction:

```text
RPC:
    "perform this interaction"

Command:
    "cause this controlled effect"
```

Commands should therefore integrate directly with the execution model.

# 15. Actions

ROS actions exist because some operations are:

```text
long-running
preemptable
feedback-producing
goal-oriented
```

NROS should preserve this concept.

An NROS action can be modeled:

```text
Action
├── Goal
├── Execution
├── Feedback
├── Cancellation
└── Result
```

# 16. Action as execution contract

Unlike a simple ROS action interface, NROS should connect the action to:

```text
Goal
 ↓
Intent
 ↓
Plan
 ↓
Activation
 ↓
Effects
 ↓
Result
```

Thus an action is not merely an asynchronous RPC.

It is a **managed execution contract**.

# 17. Action lifecycle

```text
GoalSubmitted
      ↓
GoalAccepted
      ↓
IntentCreated
      ↓
PlanSelected
      ↓
ExecutionStarted
      ↓
Feedback*
      ↓
ExecutionCompleted
      ↓
Result
```

Possible alternatives:

```text
Cancelled
Rejected
Preempted
Failed
Expired
```

# 18. Feedback

Feedback is an asynchronous stream associated with an action:

```text
Action A42
 ├── Goal
 ├── Feedback ──────┐
 ├── Feedback ──────┤
 ├── Feedback ──────┤
 └── Result          │
                     ↓
                  Consumer
```

Feedback should carry:

```text
action_id
sequence
timestamp
progress
state
```

# 19. Cancellation

Cancellation is not merely:

```text
send cancel message
```

NROS must determine whether cancellation is:

```text
requested
authorized
accepted
executed
confirmed
```

because a physical action may already be irreversible.

# 20. Preemption

For example:

```text
MoveForward
```

is running.

A new goal arrives:

```text
EmergencyStop
```

The runtime must resolve:

```text
priority
authority
resource ownership
preemption policy
safety policy
```

before replacing the existing activation.

# 21. Dataflow

NROS should also support explicit dataflow:

```text
Sensor
  ↓
Filter
  ↓
Estimator
  ↓
Planner
  ↓
Controller
  ↓
Actuator
```

Each edge is a typed communication contract.

# 22. Dataflow versus event flow

These should remain distinct:

```text
Dataflow:
    values move through computation

Event flow:
    occurrences move through the runtime
```

For example:

```text
Pose(x,y,z)
```

is data.

```text
LocalizationLost
```

is an event.

# 23. State synchronization

ROS's parameter server was designed for shared configuration.

NROS should replace the broad "shared database" concept with explicit state synchronization.

For example:

```text
State<RobotConfiguration>
```

Consumers can subscribe to changes.

# 24. State ownership

Shared mutable state creates ambiguity.

NROS should therefore define:

```text
State
├── owner
├── authority
├── version
├── timestamp
├── validity
└── synchronization policy
```

A state value should have an authoritative source.

# 25. State versions

Example:

```text
RobotConfig v41
```

followed by:

```text
RobotConfig v42
```

Consumers can detect:

```text
gap:
v41 → v42
```

and request synchronization if necessary.

# 26. State versus event

This gives us another important distinction:

```text
State:
    "what is true now"

Event:
    "what happened"
```

Example:

```text
State:
    motor.status = STOPPED

Event:
    EmergencyStopApplied
```

Both are necessary.

# 27. Discovery

ROS 1 used the Master for discovery.

NROS should separate:

```text
Discovery
```

from:

```text
Data transport
```

Discovery answers:

> What exists?

Transport answers:

> How do I communicate with it?

# 28. Discovery information

A discovered service may expose:

```text
Endpoint
├── identity
├── capabilities
├── schemas
├── transports
├── QoS
├── authority requirements
├── lifecycle state
└── health
```

# 29. No mandatory central master

NROS should not require a single global master.

A deployment may use:

```text
Local discovery
Distributed discovery
Static configuration
Registry service
Multicast
Broker
```

depending on environment.

# 30. Embedded deployment

A tiny embedded robot may use:

```text
Static endpoint table
```

rather than running a discovery service.

A large fleet may use:

```text
Distributed registry
```

Both should implement the same discovery semantics.

# 31. Discovery is not authorization

Finding an endpoint does not imply permission to use it.

Therefore:

```text
Discovery
   ↓
"endpoint exists"

Authorization
   ↓
"you may interact with it"
```

This is a critical NROS boundary.

# 32. Schema

Every meaningful communication contract needs a schema.

Conceptually:

```text
Schema
├── schema_id
├── version
├── fields
├── constraints
├── units
├── semantics
└── compatibility
```

# 33. ROS messages versus NROS schemas

ROS messages primarily define structure.

NROS schemas should eventually be capable of expressing more:

```text
type
units
range
optionality
semantic meaning
version
constraints
```

For example:

```text
Velocity
    value: f32
    unit: m/s
    range: [-10, 10]
```

rather than simply:

```text
float32 velocity
```

# 34. Schema evolution

A distributed robot cannot upgrade every component simultaneously.

Therefore:

```text
Pose.v1
Pose.v2
Pose.v3
```

must coexist where necessary.

The communication fabric should support compatibility rules.

# 35. Compatibility

Potential compatibility classes:

```text
EXACT
BACKWARD_COMPATIBLE
FORWARD_COMPATIBLE
CONVERTIBLE
INCOMPATIBLE
```

A subscriber can declare what it accepts.

# 36. Serialization

The transport layer should not dictate one serialization format.

Possible implementations:

```text
CDR
JSON
CBOR
MessagePack
Protobuf
FlatBuffers
custom binary
zero-copy shared memory
```

The semantic layer remains independent.

# 37. Transport independence

A communication contract might select:

```text
Transport
├── UDP
├── TCP
├── QUIC
├── shared memory
├── Unix socket
├── serial
├── CAN
└── custom embedded transport
```

The application should not need to rewrite its semantics for each transport.

# 38. Transport capability

A transport advertises properties:

```text
TransportCapabilities
├── latency
├── reliability
├── ordering
├── MTU
├── bandwidth
├── multicast
├── encryption
├── zero_copy
└── availability
```

The runtime can choose an appropriate transport.

# 39. QoS becomes semantic

ROS 2 introduced QoS policies.

NROS should take this further.

A communication contract may specify:

```text
Reliability
Durability
Ordering
History
Deadline
Lifespan
Priority
Backpressure
Bandwidth
Latency
```

# 40. Temporal QoS

For control:

```text
deadline = 5ms
```

means something fundamentally different from:

```text
deadline = 5s
```

The scheduler and communication fabric should understand this contract.

# 41. Deadline violation

If:

```text
deadline = 5ms
```

and the message arrives after:

```text
7ms
```

NROS should generate:

```text
DeadlineMissed
```

rather than silently treating it as normal traffic.

# 42. Latency budget

A distributed computation can have:

```text
Sensor
  1ms
   ↓
Transport
  2ms
   ↓
Estimator
  1ms
   ↓
Planner
  3ms
   ↓
Controller
  1ms
```

Total:

```text
8ms
```

The communication fabric can participate in enforcing the global temporal budget.

# 43. Backpressure

High-rate producers can overwhelm consumers.

NROS must make overflow behavior explicit:

```text
DROP_OLDEST
DROP_NEWEST
BLOCK
BUFFER
SAMPLE
COALESCE
FAIL
```

The correct policy depends on semantics.

# 44. Sensor example

For a camera preview:

```text
DROP_OLDEST
```

may be appropriate.

For a safety event:

```text
FAIL
```

or durable delivery may be required.

# 45. Control commands

For actuator commands:

```text
DROP_OLDEST
```

could be dangerous.

The runtime may instead require:

```text
sequence validation
deadline validation
lease validation
```

before accepting a command.

# 46. Priority

Communication should support semantic priority:

```text
EMERGENCY
SAFETY
CONTROL
MISSION
TELEMETRY
DEBUG
```

Priority should influence queueing and resource allocation.

# 47. Isolation

A debug stream must never starve:

```text
SafetyCommand
```

Therefore communication resources need isolation.

Conceptually:

```text
Safety Queue
Control Queue
Mission Queue
Telemetry Queue
Debug Queue
```

# 48. Zero-copy

For large sensor data:

```text
CameraFrame
PointCloud
DepthImage
```

copying can dominate execution cost.

NROS should support:

```text
Producer
   ↓
Shared Buffer
   ↓
Consumers
```

with ownership/lease semantics around the buffer.

# 49. Zero-copy and ownership

A zero-copy buffer cannot simply be globally mutable.

NROS can model:

```text
BufferLease
├── owner
├── readers
├── lifetime
└── release condition
```

This connects communication directly to the resource model.

# 50. Loaned data

Conceptually:

```text
loan()
   ↓
BufferLease
   ↓
write/read
   ↓
publish
   ↓
release
```

The runtime can enforce safe lifetime management.

Rust becomes particularly useful here because ownership can be reflected in APIs.

# 51. Shared-memory transport

On one machine:

```text
Node A
   ↓
Shared Memory
   ↓
Node B
```

can avoid network serialization.

Across machines:

```text
Node A
   ↓
Network transport
   ↓
Node B
```

The semantic contract remains unchanged.

# 52. Communication security

Every communication operation should eventually answer:

```text
Who?
What?
To whom?
Why?
With what authority?
For how long?
```

This is where the earlier identity/capability model becomes operational.

# 53. Capability-gated communication

A component may discover:

```text
motor-controller
```

but lack:

```text
Capability::ControlMotor
```

Therefore:

```text
Discovery → allowed
Communication → denied
```

This is preferable to hiding the endpoint completely.

# 54. Channel authorization

A channel can require:

```text
Capability::ReadSensor
```

while another requires:

```text
Capability::CommandActuator
```

The same transport can carry both.

# 55. Encryption

Security policy can require:

```text
plaintext
authenticated
encrypted
mutually_authenticated
```

depending on deployment.

Embedded local communication may use different requirements from fleet networking.

# 56. Communication lifecycle

Endpoints themselves should have lifecycle states:

```text
DISCOVERING
AVAILABLE
DEGRADED
PAUSED
DRAINING
CLOSED
FAILED
```

This prevents consumers from treating a disappearing endpoint as an unexplained failure.

# 57. Draining

Suppose a node is shutting down.

Instead of:

```text
kill process
```

NROS can:

```text
ACTIVE
  ↓
DRAINING
  ↓
finish accepted work
  ↓
close communication
  ↓
STOPPED
```

This is especially valuable for controlled robots.

# 58. Communication failure

A disconnected endpoint should generate structured information:

```text
CommunicationFailure
├── endpoint
├── peer
├── transport
├── reason
├── duration
└── affected activations
```

Supervision can then decide what to do.

# 59. Communication failure is not necessarily execution failure

For example:

```text
Camera disconnected
```

may affect:

```text
Perception
```

but not:

```text
Wheel control
```

The runtime should propagate dependency impact rather than declaring the entire robot failed.

# 60. Dependency graph

Communication contracts create an explicit dependency graph:

```text
Camera
  ↓
Perception
  ↓
Localization
  ↓
Navigation
  ↓
Motion
```

When Camera fails, NROS can calculate affected capabilities.

# 61. Dependency-aware degradation

Instead of:

```text
Camera failure
    ↓
Robot failure
```

NROS can produce:

```text
Camera unavailable
       ↓
Vision degraded
       ↓
Localization fallback
       ↓
Navigation continues
       ↓
Reduced capability
```

This is a major shift from process-centric middleware toward capability-centric runtime behavior.

# 62. Communication graph versus capability graph

ROS primarily exposes:

```text
Node Graph
```

NROS should expose at least three graphs:

```text
Communication Graph
Capability Graph
Execution/Causality Graph
```

# 63. Communication graph

```text
A ──stream──> B
B ──stream──> C
C ──command─> D
```

Answers:

> Who communicates with whom?

# 64. Capability graph

```text
Agent
  ↓
Capability
  ↓
Resource
  ↓
Physical Effect
```

Answers:

> What can this actor actually do?

# 65. Causality graph

```text
Observation
   ↓
Decision
   ↓
Activation
   ↓
Effect
```

Answers:

> Why did this happen?

# 66. The three graphs together

```text
             ┌─────────────────┐
             │ Communication   │
             │     Graph       │
             └────────┬────────┘
                      │
                      ▼
             ┌─────────────────┐
             │ Capability      │
             │     Graph       │
             └────────┬────────┘
                      │
                      ▼
             ┌─────────────────┐
             │ Causality /     │
             │ Execution Graph │
             └─────────────────┘
```

This becomes one of the defining architectural differences between ROS and NROS.

# 67. NROS communication equation

We can summarize an interaction as:

```text
Communication
=
Identity
+
Schema
+
Authority
+
Temporal Contract
+
QoS
+
Transport
+
Causality
```

A raw message is only the payload.

# 68. From ROS primitives to NROS primitives

The conceptual migration becomes:

| ROS | NROS |
|---|---|
| Topic | Typed Stream |
| Message | Typed Data |
| Service | Governed RPC |
| Action | Managed Execution Contract |
| Parameter Server | Owned State |
| Master | Discovery Fabric |
| roslaunch | Runtime Composition |
| rosbag | Event/Data Ledger |
| node | Component / Agent |
| callback | Event/Work Handler |
| nodelet | In-process Component |
| ROS graph | Multi-graph Runtime Model |

The mapping is deliberately **semantic**, not merely syntactic.

# 69. The NROS communication stack

The implementation architecture can now be decomposed:

```text
┌────────────────────────────────────────────┐
│          Application / Agent API           │
├────────────────────────────────────────────┤
│       Actions / Commands / RPC / Stream    │
├────────────────────────────────────────────┤
│          Communication Contracts            │
├────────────────────────────────────────────┤
│       Schema / Serialization Layer         │
├────────────────────────────────────────────┤
│          QoS / Temporal Layer              │
├────────────────────────────────────────────┤
│       Authorization / Identity Layer       │
├────────────────────────────────────────────┤
│             Routing / Discovery            │
├────────────────────────────────────────────┤
│               Transport                    │
├────────────────────────────────────────────┤
│          OS / Embedded / Hardware          │
└────────────────────────────────────────────┘
```

# 70. Rust implications

For NROS, this naturally suggests Rust-level abstractions such as:

```text
Stream<T>
Publisher<T>
Subscriber<T>
Request<T>
Response<T>
Command<T>
Action<G, F, R>
Event
Endpoint
Channel
Schema
QoS
Deadline
Capability
Lease
```

The important point is that these should not become disconnected APIs.

They must share common runtime semantics.

# 71. One common envelope

A strong implementation direction is to give interactions a common envelope:

```text
Envelope<T>
├── id
├── source
├── destination
├── schema
├── timestamp
├── deadline
├── correlation
├── authority
├── causality
└── payload: T
```

Different communication primitives can specialize the semantics.

# 72. Communication as a runtime capability

An agent should not automatically receive unrestricted communication.

Its capabilities might say:

```text
Read:
    /sensors/imu

Publish:
    /navigation/intent

Request:
    localization/get_state

Command:
    navigation/execute

Forbidden:
    motor/raw
```

This provides fine-grained authority.

# 73. The key NROS transformation

ROS asks:

> **How do processes communicate?**

NROS asks:

> **How do authorized actors exchange typed information and coordinate governed execution under temporal, resource, and safety constraints?**

That is the conceptual leap.

# 74. NROS communication invariant

A communication operation should be representable as:

```text
Actor
  ↓
Identity
  ↓
Capability
  ↓
Communication Contract
  ↓
Schema Validation
  ↓
Temporal/QoS Admission
  ↓
Transport
  ↓
Receiver
  ↓
Event
```

For physical commands:

```text
Receiver
  ↓
Activation
  ↓
Effect
  ↓
Confirmation
```

# 75. Final architecture

We now have a much more complete NROS model:

```text
                       NROS
                        │
       ┌────────────────┼────────────────┐
       │                │                │
       ▼                ▼                ▼
   GOVERNANCE       COMMUNICATION     EXECUTION
       │                │                │
 Identity             Streams          Goals
 Capability           Events           Intents
 Policy               RPC              Plans
 Authority             Commands         Activations
       │               Actions          Effects
       │                │                │
       └────────────────┼────────────────┘
                        ▼
                  RESOURCE FABRIC
                        │
                  Temporal Fabric
                        │
                  State Fabric
                        │
                  Event Fabric
                        │
                  Transport Fabric
                        │
                    Hardware
```

The architecture is no longer merely:

```text
Node → Message → Node
```

It becomes:

```text
Actor
  ↓
Intent
  ↓
Governance
  ↓
Communication
  ↓
Resource Admission
  ↓
Temporal Admission
  ↓
Activation
  ↓
Physical Effect
  ↓
Observation
  ↓
Event Ledger
  ↓
State Update
  ↓
Next Decision
```

And that gives us the central NROS execution cycle:

```text
OBSERVE
   ↓
UNDERSTAND
   ↓
DECIDE
   ↓
AUTHORIZE
   ↓
ADMIT
   ↓
EXECUTE
   ↓
OBSERVE EFFECT
   ↓
RECORD
   ↓
REPLAN
```

**Next: Part XXXIX — NROS Runtime Composition & Lifecycle**, where we replace the ROS notion of *nodes + roslaunch* with a runtime model for **components, agents, supervisors, profiles, manifests, dependency graphs, startup ordering, health, restart, hot replacement, and deterministic shutdown**.

# NROS — Part XXXIX: Runtime Composition & Lifecycle

ROS gave us a powerful process model:

```text
Package
   ↓
Node
   ↓
roslaunch
   ↓
Running computation graph
```

NROS should preserve the useful composition idea while changing the unit of composition.

The fundamental unit becomes:

> **A managed runtime component with identity, capabilities, resources, lifecycle, dependencies, and communication contracts.**

## 1. From `roslaunch` to Runtime Composition

ROS 1 commonly expresses deployment through launch files:

```text
roslaunch
 ├── node A
 ├── node B
 ├── node C
 ├── parameters
 └── remappings
```

NROS should evolve this into:

```text
Runtime Profile
 ├── Components
 ├── Agents
 ├── Resources
 ├── Communication Contracts
 ├── Policies
 ├── Dependencies
 ├── Security
 └── Lifecycle
```

The launch description becomes a **deployment specification**, rather than merely a process-startup script.

# 2. Component

A component is the basic executable unit.

```text
Component
├── identity
├── implementation
├── version
├── capabilities
├── inputs
├── outputs
├── resources
├── dependencies
└── lifecycle
```

Example:

```text
localization
├── subscribes: /sensors/imu
├── subscribes: /sensors/lidar
├── publishes: /state/pose
└── provides: localization/get_state
```

# 3. Component ≠ Process

This distinction is important.

One process may contain:

```text
Process
 ├── Component A
 ├── Component B
 └── Component C
```

Alternatively:

```text
Process A → Component A
Process B → Component B
```

The semantic component model should remain independent of process placement.

# 4. Execution Unit

The runtime therefore needs another abstraction:

```text
ExecutionUnit
```

An execution unit determines **where and how** a component actually runs.

Possible forms:

```text
Thread
Task
Process
Container
Remote process
Embedded task
Hardware accelerator
```

Thus:

```text
Component
    ↓
ExecutionUnit
    ↓
Runtime
```

# 5. Component Placement

A deployment profile can specify:

```text
localization → CPU core 2
planner      → CPU core 3
camera       → GPU
telemetry    → background
```

But placement should be declarative.

The scheduler may later choose the actual execution resource.

# 6. Agent

NROS introduces a higher-level concept:

```text
Agent
```

A component performs a defined computation.

An agent can possess:

```text
identity
memory
goals
capabilities
policies
state
planning
execution
```

Conceptually:

```text
Agent
 ├── Observe
 ├── Reason
 ├── Plan
 ├── Execute
 ├── Reflect
 └── Checkpoint
```

This aligns NROS with an agent-native architecture.

# 7. Agent and Component Relationship

An agent can contain or coordinate components:

```text
NavigationAgent
 ├── perception
 ├── localization
 ├── planner
 ├── controller
 └── recovery
```

But the runtime should not require every component to be intelligent.

A simple motor driver can remain deterministic:

```text
MotorDriver
```

while an agent orchestrates it.

# 8. Supervisor

Every serious NROS deployment needs supervision.

```text
Supervisor
 ├── observes health
 ├── manages lifecycle
 ├── enforces policy
 ├── handles failures
 ├── restarts components
 └── coordinates shutdown
```

This replaces the simplistic assumption that a process either exists or does not.

# 9. Lifecycle

A component should have explicit states.

A useful baseline:

```text
DISCOVERED
    ↓
REGISTERED
    ↓
CONFIGURED
    ↓
READY
    ↓
ACTIVE
    ↓
DRAINING
    ↓
INACTIVE
    ↓
STOPPED
```

Failures can transition to:

```text
FAILED
QUARANTINED
```

# 10. Why Lifecycle Matters

Consider a controller:

```text
MotorController
```

Starting the process does not necessarily mean:

```text
motors are enabled
```

We need a distinction between:

```text
process alive
component configured
component ready
component active
physical capability enabled
```

This is essential for robotics.

# 11. Lifecycle Invariants

NROS should establish rules such as:

```text
ACTIVE ⇒ dependencies satisfied
ACTIVE ⇒ required capabilities available
ACTIVE ⇒ communication contracts established
ACTIVE ⇒ resource leases valid
```

Therefore a component cannot simply declare itself active.

The runtime must validate the transition.

# 12. Configuration

Configuration should be explicit and typed.

Instead of arbitrary parameters:

```text
speed: 10
```

NROS should know:

```text
Configuration
├── schema
├── version
├── owner
├── defaults
├── constraints
└── source
```

For example:

```text
max_velocity:
    value: 2.0
    unit: m/s
    range: [0, 3]
```

# 13. Configuration Sources

A value can originate from:

```text
compiled default
deployment profile
device configuration
environment
secure store
runtime update
```

The source should be observable.

# 14. Immutable Configuration

Some configuration must not change while active.

For example:

```text
motor.protocol
hardware.device_id
safety.limit
```

NROS can classify configuration as:

```text
STATIC
RESTART_REQUIRED
DYNAMIC
```

# 15. Dynamic Configuration

Other values may safely change:

```text
logging level
telemetry rate
planner preference
diagnostic verbosity
```

A dynamic update should still be transactional:

```text
validate
   ↓
prepare
   ↓
apply
   ↓
confirm
```

# 16. Dependency Graph

A runtime profile defines dependencies:

```text
Camera
   ↓
Perception
   ↓
Localization
   ↓
Planner
   ↓
Controller
   ↓
MotorDriver
```

The runtime can derive startup ordering automatically.

# 17. Startup

Instead of:

```text
start everything
sleep 5
hope everything works
```

NROS should perform dependency-aware startup:

```text
Discover
   ↓
Validate
   ↓
Allocate resources
   ↓
Configure
   ↓
Establish communication
   ↓
Check dependencies
   ↓
Activate
```

# 18. Readiness

A component reports:

```text
READY
```

only after its prerequisites are satisfied.

For example:

```text
Localization
```

requires:

```text
IMU available
LiDAR available
clock synchronized
configuration valid
output channel established
```

Only then:

```text
READY
```

# 19. Activation

Readiness and activation remain separate.

```text
READY
```

means:

> I can operate.

```text
ACTIVE
```

means:

> I am currently authorized and expected to operate.

This distinction enables safe staged startup.

# 20. Shutdown

Shutdown should be equally structured:

```text
ACTIVE
   ↓
STOP ACCEPTING NEW WORK
   ↓
DRAIN
   ↓
RELEASE EFFECTS
   ↓
RELEASE LEASES
   ↓
CLOSE COMMUNICATION
   ↓
RELEASE RESOURCES
   ↓
STOP
```

# 21. Deterministic Shutdown

A robot must not depend on:

```text
SIGKILL
```

as its normal shutdown mechanism.

The runtime should provide:

```text
graceful shutdown
deadline
forced shutdown
post-shutdown verification
```

# 22. Failure Handling

Suppose:

```text
Localization
```

crashes.

The supervisor evaluates:

```text
What depends on localization?
What resources remain valid?
Can localization restart?
Can another implementation replace it?
Should navigation degrade?
Should the robot stop?
```

This is substantially richer than simply restarting a process.

# 23. Restart Policy

Components can declare:

```text
restart:
    NEVER
    ON_FAILURE
    ALWAYS
    BACKOFF
    SUPERVISED
```

But restart must also respect safety.

A failed motor controller should not automatically restart into an uncontrolled physical state.

# 24. Restart as a State Transition

Instead of:

```text
kill → exec
```

NROS:

```text
FAILED
  ↓
ISOLATE
  ↓
REVOKE EFFECT CAPABILITIES
  ↓
CLEANUP
  ↓
REINITIALIZE
  ↓
VERIFY
  ↓
READY
  ↓
ACTIVE
```

# 25. Quarantine

Repeated failures indicate something deeper.

For example:

```text
Controller
 ↓
crash
 ↓
restart
 ↓
crash
 ↓
restart
 ↓
crash
```

NROS should eventually produce:

```text
QUARANTINED
```

rather than an infinite restart loop.

# 26. Recovery Policies

A deployment can specify:

```text
on localization failure:
    switch to odometry

on camera failure:
    disable vision navigation

on controller failure:
    emergency stop

on telemetry failure:
    continue mission
```

This becomes a declarative recovery graph.

# 27. Health Model

"Process alive" is insufficient.

NROS health should include:

```text
Health
├── liveness
├── readiness
├── latency
├── deadline compliance
├── resource usage
├── dependency health
├── communication health
└── semantic health
```

# 28. Semantic Health

A component can be alive but wrong.

Example:

```text
Localization process:
    RUNNING
```

but:

```text
position covariance:
    unacceptable
```

Therefore:

```text
process health ≠ functional health
```

NROS should expose both.

# 29. Heartbeats

Components may expose:

```text
Heartbeat
```

containing:

```text
component_id
timestamp
state
health
sequence
```

But heartbeats should not become the only health mechanism.

# 30. Watchdogs

Critical components can have watchdogs:

```text
Supervisor
    ↓
Watchdog
    ↓
Controller
```

A watchdog can monitor:

```text
deadline misses
heartbeats
state transitions
resource consumption
output validity
```

# 31. Resource Admission

Before activation, NROS should determine whether resources exist.

For example:

```text
Planner
requires:
    CPU ≥ 20%
    memory ≥ 128MB
    GPU = optional
```

The runtime performs:

```text
resource admission
```

before activating it.

# 32. Resource Ownership

Resources can include:

```text
CPU
memory
GPU
camera
motor
CAN bus
network port
shared memory
filesystem
device handle
```

A component should not implicitly own them.

# 33. Leases

A powerful mechanism is:

```text
ResourceLease
```

Example:

```text
MotorController
    ↓
Lease(motor.left)
```

The lease can expire if the component disappears.

This prevents stale ownership.

# 34. Safety through Lease Expiration

Consider:

```text
Controller crashes
```

Without leases:

```text
motor remains commanded
```

With leases:

```text
controller crash
   ↓
lease expires
   ↓
motor authority revoked
   ↓
safe state
```

This is a major runtime primitive.

# 35. Runtime Profile

A complete NROS deployment can be represented conceptually as:

```text
Profile
├── identity
├── components
├── agents
├── resources
├── communication
├── policies
├── security
├── lifecycle
├── recovery
└── observability
```

# 36. Example

Conceptually:

```text
profile: mobile_robot

components:
    imu
    lidar
    localization
    planner
    controller
    motor_driver

agents:
    navigation

resources:
    cpu
    memory
    imu
    lidar
    motors

policies:
    safety
    restart
    scheduling
```

The profile describes the system.

The runtime realizes it.

# 37. Deployment Targets

The same profile should ideally target:

```text
single process
single machine
multi-process
multi-machine
embedded board
robot fleet
simulation
```

without changing application semantics.

# 38. Simulation

This becomes extremely useful.

The same:

```text
NavigationAgent
```

can operate against:

```text
SimulationMotor
```

instead of:

```text
PhysicalMotor
```

because both satisfy the same capability contract.

# 39. Hardware Abstraction

Instead of:

```text
ROS node → driver
```

NROS becomes:

```text
Capability
     ↑
 ┌───┴────┐
 │        │
Physical  Simulated
Device    Device
```

The application depends on capability semantics rather than hardware implementation.

# 40. Runtime Composition

NROS therefore enables:

```text
                Profile
                   │
        ┌──────────┼──────────┐
        ↓          ↓          ↓
    Components   Agents    Resources
        │          │          │
        └──────────┼──────────┘
                   ↓
             Dependency Graph
                   ↓
             Resource Admission
                   ↓
              Lifecycle Engine
                   ↓
               Execution
```

# 41. The Runtime Kernel

At the center of this architecture sits the:

```text
NROS Runtime
```

Its responsibilities become:

```text
identity
discovery
communication
scheduling
lifecycle
resource management
authorization
supervision
observability
```

Not application logic.

# 42. Kernel Boundary

This suggests a clean boundary:

```text
┌──────────────────────────────┐
│       NROS Applications      │
│ Agents / Components / Nodes  │
└───────────────┬──────────────┘
                │
        Stable Runtime API
                │
┌───────────────▼──────────────┐
│          NROS Core            │
│                              │
│ Lifecycle                    │
│ Communication               │
│ Scheduler                   │
│ Resources                   │
│ Identity                    │
│ Policy                      │
│ Supervision                 │
└───────────────┬──────────────┘
                │
┌───────────────▼──────────────┐
│ OS / RTOS / Embedded / HW    │
└──────────────────────────────┘
```

# 43. NROS versus ROS

The transformation is now clearer:

| ROS concept | NROS evolution |
|---|---|
| Node | Component |
| Process | Execution Unit |
| roslaunch | Runtime Profile |
| Master | Discovery Fabric |
| Parameter Server | Owned State |
| Topic | Typed Stream |
| Service | Governed RPC |
| Action | Execution Contract |
| Driver | Capability Provider |
| rosbag | Event/Data Ledger |
| Node lifecycle | Managed Runtime Lifecycle |
| Process restart | Supervised Recovery |
| Parameter | Typed Configuration |
| Namespace | Identity Namespace |

# 44. The deeper transformation

ROS primarily organizes:

```text
software processes
```

NROS organizes:

```text
actors
+
capabilities
+
resources
+
communication
+
execution
```

This changes the abstraction level.

# 45. NROS Runtime Equation

A useful conceptual equation is:

```text
NROS Runtime
=
Composition
+
Lifecycle
+
Communication
+
Scheduling
+
Resources
+
Governance
+
Supervision
```

And application execution becomes:

```text
Observe
   ↓
Decide
   ↓
Request
   ↓
Authorize
   ↓
Admit
   ↓
Activate
   ↓
Execute
   ↓
Observe
```

# 46. What this enables

With these primitives, NROS can support:

```text
✓ deterministic startup
✓ dependency-aware activation
✓ controlled shutdown
✓ supervised recovery
✓ capability-based authorization
✓ resource leasing
✓ temporal contracts
✓ simulation/physical substitution
✓ distributed execution
✓ agent-oriented orchestration
✓ fault containment
✓ graceful degradation
```

without requiring every application to reinvent these mechanisms.

# 47. The next architectural layer

We now have:

```text
Part XXXVIII
Communication Fabric
        ↓
Part XXXIX
Runtime Composition & Lifecycle
```

The next logical layer is **Scheduling**.

ROS generally treats scheduling as an OS/runtime concern.

NROS should make scheduling a first-class semantic layer because an NROS runtime may need to coordinate:

```text
periodic control loops
event-driven work
deadlines
priorities
agents
actions
resource contention
preemption
CPU affinity
real-time tasks
background cognition
```

That leads to:

# **Part XL — NROS Scheduler: From ROS Callbacks to Intent-Aware Execution**

The central question becomes:

> **What should execute next, on which resource, with what priority, under what deadline, and with what authority?**

# NROS — Part XL: Scheduler & Execution Model

The next major transformation is from the ROS callback/process execution model to an **intent-aware execution model**.

ROS applications often ultimately depend on:

```text
OS scheduler
    ↓
thread
    ↓
callback
    ↓
user code
```

That is sufficient for conventional robotics software.

NROS should make the execution decision itself a first-class runtime concern:

```text
Observation
    ↓
Work / Intent
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
```

## 1. Scheduling is not merely CPU allocation

A conventional scheduler answers:

> Which thread runs next?

NROS needs to answer a larger question:

> Which **work item** should execute next, given deadlines, priorities, dependencies, resources, authority, and system state?

Therefore:

```text
NROS Scheduler
=
CPU Scheduling
+
Work Scheduling
+
Resource Scheduling
+
Temporal Scheduling
+
Priority Management
```

# 2. The NROS Work Unit

Instead of making the callback the fundamental execution object, NROS introduces:

```text
WorkItem
```

Conceptually:

```text
WorkItem
├── work_id
├── source
├── kind
├── priority
├── deadline
├── budget
├── dependencies
├── required_resources
├── authority
├── cancellation
└── execution_state
```

A work item could represent:

```text
sensor processing
control update
RPC
command
action step
agent reasoning
recovery operation
maintenance task
```

# 3. Work Types

NROS can distinguish:

```text
PERIODIC
EVENT_DRIVEN
DEADLINE_DRIVEN
BEST_EFFORT
BACKGROUND
EMERGENCY
INTERACTIVE
```

Example:

```text
motor_control      → PERIODIC
lidar_processing    → EVENT_DRIVEN
emergency_stop      → EMERGENCY
telemetry_upload    → BEST_EFFORT
AI_planning         → BACKGROUND
```

The scheduler can therefore make semantic decisions.

# 4. Periodic Work

A control loop may specify:

```text
period = 1ms
deadline = 1ms
budget = 200µs
```

Conceptually:

```text
|----1ms----|----1ms----|----1ms----|
    ctrl         ctrl         ctrl
```

The scheduler must preserve the temporal contract.

# 5. Event-Driven Work

Sensor events may instead generate work dynamically:

```text
LidarFrame
    ↓
WorkItem
    ↓
Perception
```

No fixed periodic schedule is necessary.

# 6. Deadline-Driven Work

Suppose:

```text
ObstacleDetected
```

creates:

```text
AvoidanceCommand
```

with:

```text
deadline = now + 10ms
```

The scheduler must prioritize work according to the remaining temporal window.

# 7. Earliest Deadline First

A possible scheduling policy is:

```text
EDF
```

where the earliest deadline receives priority.

For example:

```text
Work A → deadline +30ms
Work B → deadline +5ms
Work C → deadline +20ms
```

Order:

```text
B → C → A
```

But EDF should not be the universal policy.

# 8. Priority + Deadline

NROS may combine:

```text
priority
deadline
criticality
```

Example:

```text
Emergency Stop
priority = CRITICAL
deadline = 2ms

Navigation Update
priority = HIGH
deadline = 20ms

Telemetry
priority = LOW
deadline = 2s
```

# 9. Criticality

Not all missed deadlines are equivalent.

A missed:

```text
telemetry deadline
```

is usually less serious than:

```text
motor control deadline
```

Therefore NROS should model:

```text
Criticality
```

perhaps as:

```text
SAFETY
CONTROL
MISSION
PERCEPTION
TELEMETRY
BACKGROUND
```

# 10. Scheduling Tuple

A useful conceptual scheduling key:

```text
ScheduleKey =
(
    criticality,
    priority,
    deadline,
    resource_availability,
    dependency_state
)
```

This gives the scheduler more information than a conventional thread priority.

# 11. Admission Control

Before executing work, NROS should ask:

```text
Can this work actually be admitted?
```

Check:

```text
authority
dependencies
resources
deadline
memory
execution budget
safety policy
```

If not:

```text
REJECT
DEFER
DEGRADE
CANCEL
```

rather than blindly enqueueing it.

# 12. Resource-Aware Scheduling

Suppose:

```text
Planner
requires GPU
```

but:

```text
GPU = occupied
```

The scheduler should understand the dependency:

```text
Planner
  ↓
GPU unavailable
  ↓
WAIT
```

while unrelated CPU work continues.

# 13. Resource Reservation

Critical work may reserve resources:

```text
ControlLoop
   ↓
CPU core 2
   ↓
reserved
```

This can provide deterministic execution.

# 14. CPU Affinity

For real-time workloads:

```text
Control
 → CPU 2

Perception
 → CPU 3

Background
 → CPU 0/1
```

NROS can expose this declaratively.

# 15. Execution Budget

Each work item may declare:

```text
budget = 200µs
```

The runtime measures actual consumption:

```text
start
 ↓
100µs
 ↓
180µs
 ↓
200µs
```

If it exceeds the budget:

```text
BudgetExceeded
```

can be emitted.

# 16. Budget ≠ Deadline

This distinction is important.

```text
Budget:
    how much execution time is expected

Deadline:
    when the work must finish
```

For example:

```text
budget = 200µs
deadline = 1ms
```

A work item can consume only 200µs but still miss its deadline because it waited too long.

# 17. Queueing Delay

NROS should therefore observe:

```text
latency =
queue_delay
+
execution_time
+
communication_delay
```

This enables proper performance diagnostics.

# 18. Callback Replacement

ROS-style:

```text
message
   ↓
callback()
```

NROS:

```text
message
   ↓
event
   ↓
work admission
   ↓
scheduler
   ↓
execution
```

This extra layer is intentional.

It lets the runtime reason about work before executing it.

# 19. Execution Context

Each work item receives an execution context:

```text
ExecutionContext
├── identity
├── authority
├── deadline
├── cancellation token
├── resource leases
├── tracing context
└── parent activation
```

Thus execution remains traceable.

# 20. Parent/Child Work

An action may create several work items:

```text
Action A42
 ├── perception
 ├── planning
 ├── trajectory generation
 └── control
```

The scheduler knows that these belong to the same execution.

# 21. Structured Concurrency

This naturally suggests structured concurrency.

```text
Activation A
 ├── Work A1
 ├── Work A2
 └── Work A3
```

When the activation is cancelled:

```text
cancel A
 ↓
cancel A1
cancel A2
cancel A3
```

No orphaned work should remain.

# 22. Cancellation

Cancellation becomes a first-class runtime operation:

```text
RUNNING
   ↓
CANCEL_REQUESTED
   ↓
DRAINING
   ↓
CANCELLED
```

Critical work may be non-cancellable.

For example:

```text
atomic hardware shutdown
```

may need to finish before cancellation is honored.

# 23. Preemption

NROS should distinguish:

```text
cooperative cancellation
preemption
forced termination
```

They have different safety implications.

# 24. Cooperative

A work item reaches a safe cancellation point:

```text
checkpoint
   ↓
observe cancellation
   ↓
cleanup
   ↓
exit
```

Preferred for most application work.

# 25. Preemptive

The runtime can interrupt execution when required:

```text
Background AI
       ↓
Emergency Control
       ↓
preempt
```

This requires careful isolation and is especially relevant to real-time deployments.

# 26. Emergency Scheduling

Emergency work should bypass ordinary queues where possible.

Conceptually:

```text
BACKGROUND
MISSION
CONTROL
SAFETY
   ↓
EMERGENCY
```

An emergency stop must not wait behind:

```text
telemetry
logging
AI planning
```

# 27. Safety Scheduler

This suggests a dedicated high-criticality path:

```text
                 Scheduler
                    │
        ┌───────────┴───────────┐
        ↓                       ↓
 Safety/Critical            General
        │                       │
 Emergency                   Agents
 Control                     Planning
 Watchdogs                   Telemetry
```

The exact implementation may differ, but the semantic isolation is important.

# 28. Agent Scheduling

Now we reach a key NROS difference.

An agent is not simply:

```text
while true:
    think()
```

Instead:

```text
Observation
    ↓
Reasoning Work
    ↓
Decision
    ↓
Execution Work
```

The scheduler treats each stage as work with explicit resource and temporal requirements.

# 29. Agentic Scheduling

For example:

```text
NavigationAgent
```

receives:

```text
ObstacleDetected
```

The runtime generates:

```text
WorkItem:
    reason_about_obstacle
```

with:

```text
priority = HIGH
deadline = 100ms
resources = CPU
```

The result can produce:

```text
Command:
    replan_navigation
```

which generates another work item.

# 30. Cognition Should Not Block Control

Suppose AI planning takes:

```text
300ms
```

while motor control requires:

```text
1ms cycle
```

The scheduler must guarantee:

```text
AI planning
    ↓
cannot starve
    ↓
motor control
```

This leads to explicit execution classes.

# 31. Execution Classes

A useful model:

```text
REALTIME
CONTROL
INTERACTIVE
MISSION
COGNITIVE
BACKGROUND
```

Each class receives different scheduling guarantees.

# 32. Cognitive Work

LLM inference may be:

```text
slow
variable
resource-intensive
interruptible
non-deterministic
```

Therefore it should generally run under:

```text
COGNITIVE
```

rather than pretending to be deterministic control work.

# 33. Deterministic Boundary

This produces an important architecture:

```text
             Agent / AI
                 │
        nondeterministic
          reasoning
                 │
                 ▼
          Intent / Plan
                 │
        deterministic gate
                 │
                 ▼
       Control / Execution
                 │
                 ▼
             Hardware
```

AI can propose.

The runtime decides whether and how that proposal executes.

# 34. Intent Admission

Suppose an agent proposes:

```text
MoveArm(position=X)
```

The scheduler/runtime checks:

```text
authority
safety
resource ownership
workspace constraints
deadline
current state
```

Only then does the work enter the execution scheduler.

# 35. This creates two schedulers

Conceptually:

```text
Decision Scheduling
        ↓
Execution Scheduling
```

The first determines:

> What should happen?

The second determines:

> When and where should it happen?

They should interact, but remain architecturally distinct.

# 36. Work Graph

The runtime can maintain a dependency graph:

```text
A
↓
B
↓
C

D ─────────→ C
```

If:

```text
A
```

has not completed, then:

```text
B
```

cannot execute.

The scheduler should derive runnable work dynamically.

# 37. Runnable Set

At every scheduling point:

```text
All Work
   ↓
Dependencies satisfied?
   ↓
Resource available?
   ↓
Authorized?
   ↓
Deadline viable?
   ↓
Runnable Set
```

The scheduler selects from the runnable set.

# 38. Deadline Viability

Suppose:

```text
deadline = +5ms
estimated execution = 10ms
```

The scheduler should detect:

```text
deadline impossible
```

instead of pretending it can succeed.

Possible response:

```text
DEGRADE
FALLBACK
CANCEL
ESCALATE
```

# 39. Graceful Degradation

For perception:

```text
High-resolution inference
```

may be replaced by:

```text
Low-resolution inference
```

if resources are constrained.

Thus scheduling can select among alternative execution strategies.

# 40. Scheduling Policy

NROS should make policy pluggable.

Possible policies:

```text
FIFO
Priority
EDF
Rate Monotonic
Criticality-aware
Fair
Resource-aware
Hybrid
```

The runtime should expose stable scheduling semantics without forcing one algorithm.

# 41. Hybrid Scheduler

A practical robot might use:

```text
Safety:
    fixed priority

Control:
    rate-monotonic

Mission:
    EDF

Cognitive:
    fair/background

Telemetry:
    best-effort
```

All can coexist.

# 42. Work Stealing

For non-critical computation:

```text
Worker 1
Worker 2
Worker 3
Worker 4
```

can use work stealing.

This is particularly useful for:

```text
perception
planning
simulation
AI preprocessing
```

but should generally be isolated from hard real-time work.

# 43. Executor

The scheduler selects work.

The executor runs it.

```text
Scheduler
    ↓
Execution Plan
    ↓
Executor
    ↓
CPU / GPU / Device
```

This distinction is useful.

# 44. Executor Types

NROS can provide:

```text
SingleThreadExecutor
MultiThreadExecutor
RealtimeExecutor
AsyncExecutor
RemoteExecutor
EmbeddedExecutor
```

All implement a common semantic interface.

# 45. Realtime Executor

A real-time executor should minimize:

```text
dynamic allocation
unbounded locks
unpredictable syscalls
unbounded queues
runtime discovery
```

during critical execution.

# 46. Non-Realtime Executor

The general executor can support richer workloads:

```text
async IO
networking
dynamic tasks
AI inference
logging
filesystem
```

The two execution domains should not be confused.

# 47. Execution Isolation

A high-performance NROS deployment might look like:

```text
┌──────────────────────────────┐
│ Safety / RT Domain           │
│                              │
│ Motor Control                │
│ Watchdog                     │
│ Emergency Handling           │
└──────────────┬───────────────┘
               │
          controlled boundary
               │
┌──────────────▼───────────────┐
│ General Domain               │
│                              │
│ Perception                   │
│ Navigation                   │
│ Planning                     │
└──────────────┬───────────────┘
               │
          controlled boundary
               │
┌──────────────▼───────────────┐
│ Cognitive / Background       │
│                              │
│ Agents                       │
│ LLM inference                │
│ Analytics                    │
└──────────────────────────────┘
```

# 48. Scheduler Observability

Every scheduling decision should be observable.

For example:

```text
WorkScheduled
WorkStarted
WorkPreempted
WorkCompleted
DeadlineMissed
BudgetExceeded
WorkCancelled
WorkRejected
```

This produces a scheduling trace.

# 49. Scheduling Trace

Example:

```text
12:00:00.001  Work A queued
12:00:00.002  Work A admitted
12:00:00.002  Work A started
12:00:00.003  Work A completed

12:00:00.004  Work B queued
12:00:00.006  Work B started
12:00:00.009  DeadlineMissed
```

This can later be correlated with communication and execution traces.

# 50. Causality

Now the three graphs introduced earlier become connected:

```text
Communication
     ↓
Work creation
     ↓
Scheduler
     ↓
Execution
     ↓
Effect
     ↓
Observation
```

NROS can therefore answer:

> Why was this task executed?

# 51. End-to-End Example

Consider an autonomous mobile robot.

```text
LiDAR
  ↓
ObstacleDetected
  ↓
NavigationAgent
  ↓
Reasoning Work
  ↓
New Plan
  ↓
Command
  ↓
Authorization
  ↓
Execution Work
  ↓
Controller
  ↓
Motor
```

Every transition is observable.

# 52. Temporal Contract

Suppose:

```text
Obstacle detection:
    deadline = 20ms

planning:
    deadline = 100ms

control:
    period = 1ms
```

The scheduler must coordinate these independently.

The planning system cannot consume the CPU needed by control.

# 53. Failure Scenario

Suppose AI planning becomes expensive.

```text
AI inference
    ↓
CPU saturation
```

NROS detects:

```text
control deadline risk
```

and can:

```text
throttle AI
pause background tasks
reduce perception rate
switch planner
```

before the control loop fails.

# 54. Scheduling as Feedback Control

This leads to an interesting property:

```text
Runtime state
     ↓
Scheduler decision
     ↓
Execution
     ↓
Measured performance
     ↓
Scheduler adaptation
```

The scheduler itself becomes a feedback system.

# 55. Adaptive Scheduling

The runtime can observe:

```text
CPU pressure
memory pressure
queue depth
deadline misses
thermal state
battery state
network congestion
```

and adjust non-critical workloads.

For example:

```text
Battery low
   ↓
reduce perception rate
   ↓
reduce AI frequency
   ↓
retain control
```

# 56. Energy-Aware Scheduling

For mobile robots:

```text
WorkItem
├── CPU cost
├── GPU cost
├── estimated energy
└── priority
```

The scheduler can optimize:

```text
mission progress
versus
energy consumption
```

while respecting safety constraints.

# 57. Thermal Awareness

If:

```text
GPU temperature ↑
```

the runtime could:

```text
reduce inference frequency
switch model
move workload
defer background work
```

Again, critical control remains protected.

# 58. Scheduler and Resource Fabric

We can now connect:

```text
Scheduler
      │
      ├── CPU
      ├── Memory
      ├── GPU
      ├── Network
      ├── Device
      └── Energy
```

Scheduling becomes resource-aware rather than CPU-only.

# 59. NROS Execution Model

The complete execution path is now:

```text
Event / Intent
      ↓
Work Creation
      ↓
Dependency Resolution
      ↓
Authorization
      ↓
Resource Admission
      ↓
Temporal Admission
      ↓
Scheduling
      ↓
Execution
      ↓
Effect
      ↓
Observation
      ↓
Trace
```

# 60. The Core Difference from ROS

ROS tends toward:

```text
Message
 ↓
Callback
 ↓
Thread/Executor
```

NROS evolves this into:

```text
Message / Intent
       ↓
Semantic Work
       ↓
Governance
       ↓
Resource + Temporal Admission
       ↓
Scheduler
       ↓
Executor
       ↓
Effect
```

The scheduler is therefore no longer an invisible implementation detail.

It becomes part of the runtime's **semantic control plane**.

# 61. NROS Scheduler Architecture

```text
                 ┌─────────────────────┐
                 │    Work Sources      │
                 │ Events / Commands    │
                 │ Timers / Agents      │
                 └──────────┬──────────┘
                            ↓
                 ┌─────────────────────┐
                 │   Work Manager      │
                 └──────────┬──────────┘
                            ↓
                 ┌─────────────────────┐
                 │ Admission Controller│
                 │ Auth / Resources    │
                 │ Deadlines / Policy  │
                 └──────────┬──────────┘
                            ↓
                 ┌─────────────────────┐
                 │     Scheduler       │
                 │ Priority / EDF / RT │
                 └──────────┬──────────┘
                            ↓
                 ┌─────────────────────┐
                 │      Executor       │
                 └──────────┬──────────┘
                            ↓
              ┌─────────────┼─────────────┐
              ↓             ↓             ↓
             CPU           GPU          Device
```

# 62. Rust Architecture Direction

For the NROS Rust implementation, the conceptual modules now naturally become:

```text
nros-core
nros-identity
nros-schema
nros-communication
nros-discovery
nros-resource
nros-policy
nros-lifecycle
nros-scheduler
nros-executor
nros-supervisor
nros-tracing
```

The exact crate boundaries should be validated against the existing repository rather than imposed prematurely.

# 63. Scheduler Invariants

A strong NROS implementation should eventually enforce invariants such as:

```text
ACTIVE work
    ⇒ authorized

RUNNING work
    ⇒ admitted

PHYSICAL effect
    ⇒ valid capability

CRITICAL work
    ⇒ protected execution resources

EXPIRED work
    ⇒ never silently executed

CANCELLED work
    ⇒ no new child work

REVOKED capability
    ⇒ no new dependent execution
```

These invariants are much more valuable than simply reproducing ROS APIs.

# 64. The NROS Runtime Loop

We can now formulate the complete runtime loop:

```text
┌──────────────────────────────┐
│           OBSERVE            │
└──────────────┬───────────────┘
               ↓
┌──────────────────────────────┐
│       CREATE WORK / INTENT   │
└──────────────┬───────────────┘
               ↓
┌──────────────────────────────┐
│         GOVERNANCE           │
│ identity / capability /      │
│ policy / authority           │
└──────────────┬───────────────┘
               ↓
┌──────────────────────────────┐
│        ADMISSION             │
│ resources / deadline / deps  │
└──────────────┬───────────────┘
               ↓
┌──────────────────────────────┐
│         SCHEDULING           │
└──────────────┬───────────────┘
               ↓
┌──────────────────────────────┐
│          EXECUTE             │
└──────────────┬───────────────┘
               ↓
┌──────────────────────────────┐
│       EFFECT / STATE         │
└──────────────┬───────────────┘
               ↓
┌──────────────────────────────┐
│       TRACE / REFLECT        │
└──────────────┬───────────────┘
               │
               └──────────────→ OBSERVE
```

This is the fundamental **NROS execution cycle**.

# 65. Where We Are Now

The architecture has progressed through four major transformations:

```text
ROS
 │
 ├── Communication
 │       ↓
 │   NROS Communication Fabric
 │
 ├── roslaunch / processes
 │       ↓
 │   NROS Runtime Composition
 │
 ├── callbacks / executors
 │       ↓
 │   NROS Work + Scheduler
 │
 └── process supervision
         ↓
     NROS Lifecycle + Recovery
```

The next missing layer is **state and time**.

A distributed robotic runtime cannot reason correctly about execution without a coherent temporal model:

```text
timestamps
monotonic clocks
deadlines
periods
logical time
causal ordering
clock synchronization
simulation time
event ordering
time validity
```

So the next architectural step is:

# **Part XLI — NROS Temporal Fabric: Time, Clocks, Causality & Deterministic Ordering**

The central question becomes:

> **When did something happen, what does "now" mean across distributed components, and how can NROS distinguish temporal order from mere message arrival order?**
