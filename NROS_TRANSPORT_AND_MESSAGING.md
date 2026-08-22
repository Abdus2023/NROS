# NROS Transport & Messaging (Part XLI–L)

ROS has timestamps, timers, simulated time, and message headers, but time is often treated as supporting metadata.

NROS should elevate time into a **first-class runtime primitive**.

The runtime needs to reason about:

```text
time
├── physical time
├── monotonic time
├── logical time
├── simulation time
├── deadlines
├── periods
├── timeouts
├── causality
└── ordering
```

The key principle is:

> **Execution semantics must never depend accidentally on wall-clock time or message-arrival order.**

## 1. Why Time Becomes a Runtime Primitive

Consider two observations:

```text
A: obstacle detected at t=10ms
B: wheel encoder update at t=9ms
```

But due to network delay:

```text
B arrives first
A arrives second
```

Arrival order is:

```text
B → A
```

Physical/event time is:

```text
A → B
```

A distributed robotics runtime must be able to distinguish these.

# 2. Multiple Clocks

NROS should explicitly distinguish clock domains.

```text
Clock
├── WallClock
├── MonotonicClock
├── RuntimeClock
├── SimulationClock
└── LogicalClock
```

They must not be interchangeable.

# 3. Wall Clock

Wall time answers:

> What date/time is it?

Example:

```text
2026-08-21T15:30:00
```

Useful for:

```text
logs
human interfaces
calendar operations
timestamps for external systems
```

But wall time may jump because of:

```text
NTP
PTP
manual adjustment
time synchronization
```

Therefore it should not normally drive safety-critical timing.

# 4. Monotonic Clock

A monotonic clock answers:

> How much time has elapsed?

Conceptually:

```text
t0 = 1000
t1 = 1005
Δt = 5
```

It must not go backward.

This is the preferred clock for:

```text
timeouts
deadlines
durations
watchdogs
scheduling
latency measurement
```

# 5. Runtime Clock

NROS can expose an abstract:

```text
RuntimeClock
```

so applications don't need to know whether execution is occurring on:

```text
physical robot
simulation
replay
test harness
```

The same application can therefore operate against different temporal sources.

# 6. Simulation Time

Simulation requires:

```text
sim_time
```

The simulator may execute:

```text
1 second simulation time
```

in:

```text
10 milliseconds real time
```

or:

```text
10 seconds real time
```

The application must follow simulation time rather than wall time when configured to do so.

# 7. Replay Time

Recorded execution introduces another useful mode:

```text
ReplayClock
```

For example:

```text
recorded:
10.000
10.005
10.020
10.021
```

The runtime can replay those events at:

```text
1×
2×
10×
```

or step through them manually.

This is critical for deterministic debugging.

# 8. Time Source Identity

A timestamp should not merely contain:

```text
timestamp = 123456
```

NROS should conceptually know:

```text
timestamp
clock_domain
clock_epoch
precision
quality
```

because:

```text
123456 on clock A
```

may not be comparable to:

```text
123456 on clock B
```

# 9. Clock Domains

A distributed robot may contain:

```text
MCU clock
CPU clock
GPU clock
camera clock
LiDAR clock
network clock
simulation clock
```

NROS needs explicit clock-domain semantics.

# 10. Clock Synchronization

For distributed systems, NROS can support synchronization mechanisms such as:

```text
PTP
NTP
hardware timestamping
shared clock
simulation clock
```

But the runtime should distinguish:

```text
synchronized
approximately synchronized
unsynchronized
unknown
```

rather than assuming clocks are equal.

# 11. Time Quality

A temporal reading could conceptually contain:

```text
TimeQuality
├── accuracy
├── uncertainty
├── synchronization_state
└── source
```

Example:

```text
camera timestamp
uncertainty = ±100µs
```

This is much more useful than pretending the timestamp is exact.

# 12. Timestamp Semantics

A timestamp must have a defined meaning.

For sensor data:

```text
capture_time
```

may differ from:

```text
arrival_time
processing_time
publication_time
```

NROS should preserve these distinctions.

# 13. Event Time

A message can therefore conceptually contain:

```text
EventMetadata
├── event_id
├── source
├── event_time
├── arrival_time
├── sequence
└── causal_parent
```

Now the runtime can reason about temporal provenance.

# 14. Sequence Numbers

Time alone is insufficient.

Two events may share the same timestamp.

Therefore streams should also support:

```text
sequence_number
```

Example:

```text
frame 100
frame 101
frame 102
```

This provides local ordering.

# 15. Ordering

NROS should distinguish several ordering concepts:

```text
arrival order
source order
timestamp order
causal order
total order
```

They are not equivalent.

# 16. Source Order

A sensor can establish:

```text
frame 1 → frame 2 → frame 3
```

even when network delivery is:

```text
frame 1 → frame 3 → frame 2
```

The transport should not silently erase this information.

# 17. Causal Ordering

Suppose:

```text
A = obstacle detected
B = planner generated avoidance
C = controller executed avoidance
```

Then:

```text
A → B → C
```

is a causal chain.

NROS should be able to represent this relationship.

# 18. Causal Context

A work item can carry:

```text
causal_parent
```

so that:

```text
Observation #42
    ↓
Planning #91
    ↓
Command #117
    ↓
Motor effect #122
```

can be reconstructed.

# 19. Distributed Causality

Consider:

```text
Robot A
   │
   └── event X
          ↓
       network
          ↓
Robot B
          │
          └── event Y
```

Even if clocks are imperfect:

```text
X → Y
```

is still known causally.

This distinction is extremely valuable for debugging.

# 20. Logical Time

NROS can optionally use logical clocks.

Conceptually:

```text
event A
logical time = 10

event B caused by A
logical time = 11
```

Logical time does not represent physical duration.

It represents ordering.

# 21. Vector Clocks

For highly distributed systems, NROS could support vector-clock-like causal metadata.

For example:

```text
A = [3, 1, 0]
B = [3, 2, 0]
```

This can identify whether events are:

```text
causally ordered
```

or:

```text
concurrent
```

This is particularly useful for distributed agents.

# 22. Don't Overuse Logical Clocks

Vector clocks have overhead.

Therefore NROS should not force them on every message.

A layered approach is better:

```text
Basic:
    timestamp + sequence

Advanced:
    causal context

Distributed:
    logical/vector clock
```

# 23. Deadline Semantics

The scheduler introduced deadlines.

The temporal fabric defines what those deadlines mean.

For example:

```text
deadline = now + 10ms
```

must use a monotonic/runtime clock.

Not:

```text
wall clock
```

# 24. Deadline Propagation

Suppose:

```text
Perception
deadline = 50ms
```

creates:

```text
Planning
```

which creates:

```text
Control
```

The remaining temporal budget should propagate.

Conceptually:

```text
50ms total
 ├── perception: 10ms
 ├── planning:   25ms
 └── control:    10ms
```

with margin.

# 25. Temporal Budget

This produces:

```text
TemporalBudget
```

similar to the execution budget discussed earlier.

A work item can carry:

```text
start_time
deadline
remaining_budget
```

The scheduler can then make better decisions.

# 26. Timeout

Timeout means:

> Stop waiting after this duration.

Deadline means:

> Complete before this temporal point.

They are related but distinct.

```text
timeout → waiting constraint

deadline → completion constraint
```

NROS should preserve this distinction.

# 27. Periodic Scheduling

Periodic work can specify:

```text
period
phase
deadline
jitter_tolerance
```

For example:

```text
controller:
    period = 1ms
    deadline = 800µs
    phase = 0
```

# 28. Jitter

A control loop intended to run every:

```text
1ms
```

might actually run:

```text
1.01ms
0.98ms
1.03ms
0.99ms
```

The difference is jitter.

NROS should measure:

```text
period jitter
release jitter
completion jitter
```

rather than treating execution as perfectly periodic.

# 29. Temporal Contract

This gives us a general abstraction:

```text
TemporalContract
├── clock
├── period
├── phase
├── deadline
├── timeout
├── jitter_bound
└── tolerance
```

A component can declare its temporal requirements.

# 30. Temporal Admission

Before activation:

```text
Controller
```

declares:

```text
period = 1ms
deadline = 800µs
```

The runtime asks:

> Can the current system actually provide this contract?

If not:

```text
activation rejected
```

or:

```text
degraded mode
```

This prevents false guarantees.

# 31. Time-Aware Lifecycle

Lifecycle and temporal state now interact.

```text
CONFIGURED
   ↓
TEMPORALLY VALIDATED
   ↓
READY
   ↓
ACTIVE
```

A component whose timing contract can no longer be satisfied may enter:

```text
DEGRADED
```

rather than remaining falsely healthy.

# 32. Temporal Health

Health can now include:

```text
deadline misses
jitter
clock quality
queue latency
execution latency
synchronization uncertainty
```

Example:

```text
Controller
Liveness: healthy
Readiness: healthy
Temporal health: degraded
```

This is much more informative.

# 33. Event-Time Processing

Distributed perception often benefits from event-time processing.

Instead of:

```text
process when packet arrives
```

the system can reason:

```text
process according to sensor event time
```

This becomes important for:

```text
sensor fusion
SLAM
multi-camera systems
distributed perception
simulation
replay
```

# 34. Watermarks

For streams with delayed events, NROS could eventually support a watermark concept:

```text
watermark = "we believe events before T are complete"
```

Then a fusion component can safely finalize:

```text
events ≤ T
```

without waiting indefinitely.

This is an advanced capability, not necessarily a core primitive.

# 35. Late Events

If an event arrives after the watermark:

```text
event_time < watermark
```

NROS must define behavior:

```text
DROP
RETRACT
REPROCESS
SIDE_CHANNEL
```

This is especially useful for distributed sensor processing.

# 36. Time in Replay

Suppose a bag contains:

```text
camera
lidar
imu
commands
```

NROS replay can reconstruct:

```text
event time
causal ordering
scheduler decisions
runtime state
```

rather than merely replaying messages as quickly as possible.

# 37. Deterministic Replay

The ideal replay model is:

```text
Recorded Inputs
      ↓
Same Runtime Semantics
      ↓
Same Work Graph
      ↓
Same Scheduling Constraints
      ↓
Same Outputs
```

This becomes a powerful verification mechanism.

# 38. Determinism Levels

NROS should not promise absolute determinism everywhere.

Instead:

```text
LEVEL 0
Best effort

LEVEL 1
Temporal constraints

LEVEL 2
Deterministic event ordering

LEVEL 3
Deterministic execution

LEVEL 4
Reproducible distributed execution
```

Different deployments can choose the level they require.

# 39. Temporal Isolation

A slow clock source should not block the runtime.

Likewise:

```text
wall-clock synchronization
```

must not stall:

```text
real-time control
```

Therefore clock infrastructure itself must be isolated.

# 40. Temporal Authority

A particularly important concept:

> Who is allowed to define time?

For physical robots:

```text
hardware/runtime clock
```

may be authoritative.

For simulation:

```text
simulation engine
```

may be authoritative.

For replay:

```text
recorded timeline
```

may be authoritative.

NROS should make this explicit.

# 41. Time Authority Graph

Conceptually:

```text
Time Authority
      ↓
Runtime Clock
      ↓
Components
      ↓
Events
      ↓
Work
      ↓
Deadlines
```

Changing time authority changes runtime semantics.

Therefore it should be a controlled configuration.

# 42. Temporal Security

An attacker who can manipulate time can potentially manipulate:

```text
timeouts
leases
deadlines
authentication windows
replay protection
```

Therefore temporal metadata may become security-sensitive.

For example:

```text
expired lease
```

must not be revived simply because a compromised clock moved backward.

# 43. Monotonic Safety Rule

A fundamental NROS invariant:

```text
Safety-critical elapsed-time calculations
    MUST NOT depend on wall-clock rollback.
```

This should be enforced at the API boundary.

# 44. Time API

Conceptually:

```text
Clock::now()
Clock::monotonic_now()
Clock::deadline()
Clock::sleep_until()
Clock::sleep_for()
Clock::resolution()
Clock::quality()
```

Applications should not directly manipulate raw platform clocks when runtime semantics matter.

# 45. Temporal Event API

An event can expose:

```text
event.id
event.timestamp
event.clock
event.sequence
event.cause
event.source
```

This provides enough metadata for:

```text
ordering
tracing
replay
debugging
fusion
```

# 46. Temporal Trace

Combining temporal metadata with the previous tracing model:

```text
Observation
   │
   ├── timestamp
   ├── sequence
   └── cause
        ↓
Work
   │
   ├── admitted_at
   ├── started_at
   ├── completed_at
   └── deadline
        ↓
Effect
   │
   └── applied_at
```

Now we can calculate:

```text
sensor → decision latency
decision → execution latency
execution → effect latency
end-to-end latency
```

# 47. End-to-End Temporal Budget

For a control chain:

```text
Sensor
  2ms
   ↓
Perception
  5ms
   ↓
Planning
  10ms
   ↓
Controller
  1ms
   ↓
Actuator
  2ms
```

Total:

```text
20ms
```

If the mission requires:

```text
≤ 25ms
```

the runtime has:

```text
5ms margin
```

This can be monitored continuously.

# 48. Temporal Debt

If workloads repeatedly consume the available margin:

```text
25ms budget
 ↓
23ms
 ↓
24ms
 ↓
24.5ms
```

the system is approaching a temporal failure even before a deadline is formally missed.

NROS observability can expose this as:

```text
temporal margin
```

# 49. Temporal Degradation

When temporal margin collapses:

```text
NORMAL
   ↓
PRESSURED
   ↓
DEGRADED
   ↓
CRITICAL
```

The runtime can trigger policies:

```text
reduce optional workloads
switch algorithms
reduce sensor rates
pause cognition
increase control isolation
```

# 50. Temporal Fabric

We can now define the NROS temporal layer:

```text
┌────────────────────────────────┐
│       NROS Temporal Fabric     │
├────────────────────────────────┤
│ Clock Domains                  │
│ Time Sources                   │
│ Synchronization                │
│ Timestamps                     │
│ Sequence                       │
│ Causality                      │
│ Deadlines                      │
│ Periods                        │
│ Timeouts                       │
│ Temporal Contracts             │
│ Replay / Simulation Time       │
└────────────────────────────────┘
```

It serves every other runtime subsystem.

# 51. Integration with Scheduler

```text
Temporal Fabric
       │
       ├── deadline
       ├── period
       ├── jitter
       └── budget
              ↓
         Scheduler
```

# 52. Integration with Communication

```text
Communication
      │
      ├── timestamp
      ├── sequence
      └── causal context
             ↓
      Temporal Fabric
```

# 53. Integration with Lifecycle

```text
Lifecycle
    │
    └── temporal validation
             ↓
           READY
```

# 54. Integration with Supervision

```text
Supervisor
    ↓
deadline miss
    ↓
temporal degradation
    ↓
recovery policy
```

# 55. Integration with Agents

Agents receive:

```text
observation
timestamp
causal context
deadline
```

and produce:

```text
intent
validity window
temporal constraints
```

This is essential for agentic robotics.

# 56. NROS Temporal Model

The emerging architecture can now be summarized as:

```text
                TIME
                 │
        ┌────────┼────────┐
        ↓        ↓        ↓
 Communication Scheduler Lifecycle
        │        │        │
        └────────┼────────┘
                 ↓
             Execution
                 ↓
               Effect
                 ↓
             Observation
```

Time is no longer metadata attached to the system.

**Time participates in the semantics of the system.**

# 57. ROS → NROS Evolution

The conceptual progression is becoming:

```text
ROS
 │
 ├── Node
 ├── Topic
 ├── Service
 ├── Parameter
 ├── Process
 └── Callback
```

becomes:

```text
NROS
 │
 ├── Component
 ├── Stream
 ├── RPC / Action
 ├── Owned State
 ├── Execution Unit
 ├── Work Item
 ├── Agent
 ├── Capability
 ├── Resource
 ├── Lifecycle
 ├── Scheduler
 ├── Temporal Contract
 └── Causal Trace
```

That is no longer simply "ROS rewritten in Rust."

It is a different runtime abstraction.

# 58. The Next Missing Primitive: State

We now have:

```text
Part XXXVIII
Communication Fabric

Part XXXIX
Runtime Composition & Lifecycle

Part XL
Scheduler & Execution

Part XLI
Temporal Fabric
```

The next question is:

> **What is the authoritative state of an NROS system, who owns it, how does it change, and how can agents safely remember and recover from it?**

That leads to:

# **Part XLII — NROS State Fabric: Ownership, Memory, Events, Snapshots & Recovery**

The progression becomes:

```text
Communication
      ↓
Composition
      ↓
Execution
      ↓
Time
      ↓
STATE
```

And this is where NROS can begin moving beyond a middleware architecture toward a **stateful, recoverable robotics runtime**.

# NROS — Part XLII: State Fabric

If the Temporal Fabric answers **"when?"**, the State Fabric answers:

> **"What is true now, who owns that truth, how did it become true, and how can the system recover it?"**

This is a major architectural boundary between traditional ROS-style middleware and NROS.

## 1. ROS State Is Usually Distributed Informally

A typical ROS system may have:

```text
Node A
 ├── internal variables
 ├── parameters
 └── published state

Node B
 ├── internal variables
 └── cached messages

Node C
 └── parameter server
```

The system's actual state is therefore spread across processes.

NROS should make state **explicitly modeled**.

```text
NROS State
├── ownership
├── identity
├── version
├── validity
├── provenance
├── lifecycle
└── persistence
```

# 2. State Is Not the Same as Messages

A message says:

```text
"the robot is at position X"
```

State says:

```text
"the authoritative localization state currently believes
the robot is at X, version 184, based on observations A/B/C."
```

This distinction matters.

```text
Event
    ↓
State transition
    ↓
New state version
```

# 3. State Categories

NROS should distinguish several classes.

```text
State
├── Ephemeral
├── Operational
├── Persistent
├── Derived
├── Cached
├── Configuration
├── Mission
└── Safety
```

Each class has different lifetime and recovery semantics.

# 4. Ephemeral State

Examples:

```text
current callback context
temporary buffers
in-flight computation
network connection state
```

Usually:

```text
not persisted
```

and can simply disappear during restart.

# 5. Operational State

Examples:

```text
current velocity
active navigation goal
current controller mode
battery state
localization estimate
```

This state may need reconstruction after restart.

# 6. Persistent State

Examples:

```text
robot identity
calibration
configuration
mission checkpoints
learned maps
persistent policies
```

This state survives process/runtime restarts.

# 7. Derived State

Some state can be reconstructed:

```text
sensor observations
       ↓
localization
       ↓
pose
```

The pose may be:

```text
derived state
```

rather than the ultimate source of truth.

This distinction prevents unnecessary persistence.

# 8. State Ownership

Every authoritative state should have an owner.

```text
Localization
    owns:
        robot_pose

Battery
    owns:
        battery_state

Navigation
    owns:
        active_goal
```

Other components may:

```text
read
subscribe
derive
cache
```

but should not silently mutate another component's authoritative state.

# 9. Single Writer Principle

For critical state:

```text
one authoritative writer
many readers
```

is often safer than:

```text
many writers
```

because concurrent mutation creates ambiguity.

# 10. State Authority

Consider:

```text
robot_pose
```

Possible sources:

```text
GPS
SLAM
odometry
visual localization
```

But these are observations.

The NROS localization component can own:

```text
AuthoritativePose
```

and fuse those observations into it.

# 11. State vs Observation

This creates a clean architecture:

```text
Sensor
   ↓
Observation
   ↓
Fusion
   ↓
State
```

Rather than:

```text
Sensor
   ↓
everyone modifies shared state
```

# 12. State Identity

A state object should have stable identity:

```text
StateId
```

For example:

```text
robot.pose
navigation.goal
battery.status
arm.mode
```

This enables generic runtime tooling.

# 13. State Versioning

Every meaningful state mutation can produce:

```text
version
```

Example:

```text
pose v101
pose v102
pose v103
```

Readers can detect stale information.

# 14. Optimistic Concurrency

A state update may specify:

```text
expected_version = 103
```

If the current state is:

```text
104
```

the update fails rather than overwriting newer state.

Conceptually:

```text
compare-and-update
```

This is valuable for distributed control and mission state.

# 15. State Transition

Instead of directly saying:

```text
state.x = value
```

NROS can model:

```text
StateTransition
```

as:

```text
old_state
    ↓
transition
    ↓
new_state
```

The transition becomes observable and traceable.

# 16. State Event

A state transition can emit:

```text
StateChanged
```

containing:

```text
state_id
previous_version
new_version
cause
timestamp
actor
```

Now the system can reconstruct **why** state changed.

# 17. Causal State

Combining the previous Temporal Fabric:

```text
Observation #42
      ↓
Work #51
      ↓
Decision #57
      ↓
StateTransition #62
```

The state system preserves causality.

# 18. State Graph

Instead of thinking only in terms of variables:

```text
x
y
z
```

NROS can model relationships:

```text
Robot
 ├── Pose
 ├── Battery
 ├── Navigation
 │    └── Goal
 ├── Sensors
 └── Actuators
```

This becomes a **state graph**.

# 19. State Namespaces

State should be namespaced:

```text
robot/
    pose
    battery
    mode

navigation/
    goal
    route

arm/
    joint_state
    controller_mode
```

This avoids the ambiguity common to globally named shared state.

# 20. State Schema

A state entry needs a schema.

Conceptually:

```text
StateSchema
├── type
├── version
├── constraints
├── ownership
├── mutability
└── persistence
```

For example:

```text
battery/percentage
type = f32
range = 0..100
owner = battery_manager
mutability = owner_only
```

# 21. State Constraints

State should be validated.

For example:

```text
battery_percentage ∈ [0,100]
```

or:

```text
joint_angle ∈ allowed_workspace
```

Invalid state transitions should be rejected before becoming authoritative.

# 22. State Transactions

Some changes must be atomic.

Suppose:

```text
navigation.goal
navigation.mode
controller.mode
```

must change together.

A transaction provides:

```text
BEGIN
 ↓
validate
 ↓
apply
 ↓
COMMIT
```

or:

```text
ROLLBACK
```

# 23. Why Transactions Matter

Without atomicity:

```text
goal = new_goal
mode = OLD_MODE
```

may temporarily create an inconsistent robot state.

NROS should provide controlled atomic transitions where necessary.

# 24. State Consistency Levels

Not every state needs strong consistency.

NROS can distinguish:

```text
STRONG
CAUSAL
EVENTUAL
BEST_EFFORT
```

For example:

```text
emergency state → STRONG

telemetry cache → EVENTUAL
```

# 25. State Replication

Distributed robots may replicate state:

```text
Robot CPU
   ↕
Companion computer
   ↕
Edge computer
```

Replication should preserve:

```text
version
ownership
causality
authority
```

rather than simply copying bytes.

# 26. Primary and Replica

For authoritative state:

```text
Primary
  ↓
Replica
```

The replica is not automatically authoritative.

This prevents split-brain behavior.

# 27. Split Brain

Suppose two nodes believe they own:

```text
navigation/goal
```

Then:

```text
Node A → Goal A
Node B → Goal B
```

is dangerous.

NROS needs explicit ownership/lease semantics.

# 28. State Lease

Ownership may be represented as:

```text
StateLease
├── owner
├── expiration
├── authority
└── renewal
```

If the owner disappears:

```text
lease expires
   ↓
recovery policy
```

can transfer or invalidate ownership.

# 29. Capability + State

Ownership should interact with the Capability Fabric.

A component needs:

```text
capability:
    write navigation.goal
```

not merely network access.

Thus:

```text
Network access ≠ state authority
```

# 30. Read vs Write Capabilities

Capabilities can be granular:

```text
read(robot.pose)

write(navigation.goal)

transition(controller.mode)

admin(robot.configuration)
```

This provides much stronger governance than process identity alone.

# 31. State Access Path

The complete path becomes:

```text
Requester
   ↓
Identity
   ↓
Capability
   ↓
State Policy
   ↓
Validation
   ↓
State Transition
   ↓
Version
   ↓
Event
```

# 32. State Watches

Consumers may subscribe to state changes:

```text
watch(robot.pose)
```

They receive:

```text
StateChanged
```

rather than repeatedly polling.

# 33. Snapshot

NROS should support:

```text
StateSnapshot
```

A snapshot represents a consistent view of selected state.

Example:

```text
Snapshot #821
├── pose
├── battery
├── navigation goal
├── controller mode
└── active mission
```

# 34. Snapshot Consistency

A snapshot should specify its temporal boundary:

```text
snapshot_time
snapshot_version
clock_domain
```

so consumers know what they are looking at.

# 35. Checkpoint

A checkpoint is more than a snapshot.

```text
Snapshot
=
state at a point in time
```

while:

```text
Checkpoint
=
state + recovery metadata
```

A checkpoint may include:

```text
state
active work
mission progress
resource ownership
runtime configuration
```

# 36. Recovery

Suppose the runtime crashes.

NROS can:

```text
restart
   ↓
load checkpoint
   ↓
validate state
   ↓
reconstruct ownership
   ↓
reconcile external world
   ↓
resume
```

This is fundamentally different from simply restarting processes.

# 37. Recovery Must Not Assume Reality

A robot may have physically changed while software was offline.

Therefore:

```text
checkpointed pose
```

must not automatically be treated as:

```text
current physical truth
```

After restart:

```text
checkpoint
   ↓
re-observation
   ↓
reconciliation
   ↓
new authoritative state
```

# 38. State Reconciliation

This becomes a central recovery primitive:

```text
Persisted State
       +
Fresh Observations
       ↓
Reconciliation
       ↓
Current State
```

For example:

```text
stored battery = 72%
fresh battery = 68%
```

The runtime must use the fresh observation.

# 39. External State

Some state exists outside NROS:

```text
motor controller
PLC
camera
battery management system
industrial actuator
```

NROS therefore needs:

```text
external state adapters
```

and should distinguish:

```text
internal desired state
```

from:

```text
external observed state
```

# 40. Desired vs Actual

This is especially important in robotics.

```text
Desired velocity
      ≠
Actual velocity
```

Likewise:

```text
Desired joint position
      ≠
Measured joint position
```

NROS should explicitly model both.

# 41. Command-State Feedback Loop

The complete pattern becomes:

```text
Desired State
      ↓
Command
      ↓
Hardware
      ↓
Observed State
      ↓
Reconciliation
      ↓
Authoritative Runtime State
```

This is a much safer model than assuming commands succeed.

# 42. State Machines

NROS components often need explicit state machines:

```text
IDLE
 ↓
ARMING
 ↓
ACTIVE
 ↓
STOPPING
 ↓
IDLE
```

Transitions should be governed by:

```text
preconditions
authority
events
timing
safety policy
```

# 43. Illegal Transitions

For example:

```text
IDLE → ACTIVE
```

may require:

```text
hardware_ready
safety_ok
controller_initialized
```

If those conditions are absent:

```text
transition rejected
```

# 44. State Machine + Lifecycle

The lifecycle state:

```text
UNCONFIGURED
INACTIVE
ACTIVE
FINALIZED
```

is one state machine.

Application state:

```text
IDLE
NAVIGATING
DOCKING
EMERGENCY
```

is another.

They should not be conflated.

# 45. State Domains

NROS can distinguish:

```text
Runtime State
Component State
Mission State
World State
Hardware State
Safety State
```

Each domain has different authorities.

# 46. World State

A robot may maintain:

```text
WorldModel
├── obstacles
├── objects
├── map
├── landmarks
└── semantic entities
```

This is often dynamic and derived.

Therefore its persistence strategy differs from configuration state.

# 47. Memory

This naturally leads to a broader concept:

```text
State
   +
History
   +
Knowledge
```

NROS needs to distinguish them.

```text
State:
    what is believed now

History:
    what happened

Memory:
    what should be retained for future reasoning
```

# 48. Event Log

An append-only event stream can preserve history:

```text
Event 1
Event 2
Event 3
...
Event N
```

State can then be reconstructed by replaying events.

# 49. Event Sourcing

Conceptually:

```text
Events
  ↓
Reducer
  ↓
State
```

For selected state domains, this can provide excellent auditability.

But it should not be forced onto every high-frequency sensor stream.

# 50. Hybrid Persistence

A practical NROS architecture may use:

```text
hot state
   +
snapshots
   +
selected event log
```

instead of storing every byte of every sensor sample indefinitely.

# 51. State Retention Policy

Each state/history domain can declare:

```text
retention:
    ephemeral
    session
    checkpoint
    persistent
    archival
```

This allows the runtime to manage storage intelligently.

# 52. Memory for Agents

Now consider an NROS agent.

It may need:

```text
current world state
recent observations
mission history
learned information
```

These should not all live in one giant "memory."

Instead:

```text
Agent Memory
├── working state
├── episodic history
├── semantic knowledge
└── persistent preferences/policy
```

# 53. Agent Working State

Short-lived:

```text
current plan
current hypothesis
active subtask
```

This can be discarded after the task completes.

# 54. Episodic Memory

Historical events:

```text
mission started
obstacle encountered
plan changed
goal completed
```

Useful for:

```text
debugging
learning
reflection
```

# 55. Semantic Memory

Stable knowledge:

```text
room A is restricted
charger is at location X
tool Y requires capability Z
```

This can survive many missions.

# 56. Memory Governance

An agent should not automatically have unrestricted access to every memory domain.

Capabilities can define:

```text
read memory
write memory
create memory
delete memory
```

This prevents uncontrolled knowledge mutation.

# 57. Memory Provenance

Every persistent fact should ideally answer:

```text
Where did this come from?
When was it created?
Who created it?
How confident is it?
When was it last validated?
```

For example:

```text
Fact:
    charger_location = X

Source:
    mapping run #42

Confidence:
    0.97

Last validated:
    T
```

# 58. Confidence

For derived world state:

```text
confidence
```

can be useful.

But confidence must not replace safety validation.

An agent saying:

```text
confidence = 99%
```

does not authorize a dangerous actuator operation.

# 59. State Confidence vs Authority

These are separate:

```text
confidence → epistemic quality

authority → permission to act
```

NROS should never conflate them.

# 60. State Fabric Architecture

We can now define the layer:

```text
┌──────────────────────────────────┐
│          NROS State Fabric       │
├──────────────────────────────────┤
│ State Identity                   │
│ Ownership                        │
│ Versioning                       │
│ Validation                       │
│ Transactions                     │
│ State Machines                   │
│ Snapshots                        │
│ Checkpoints                      │
│ Replication                      │
│ Reconciliation                   │
│ Event History                    │
│ Memory                           │
│ Provenance                       │
└──────────────────────────────────┘
```

# 61. Complete Runtime Stack

We now have four major fabrics:

```text
┌────────────────────────────────────┐
│           Agent / Application      │
├────────────────────────────────────┤
│          State Fabric              │
├────────────────────────────────────┤
│        Temporal Fabric             │
├────────────────────────────────────┤
│      Execution / Scheduler         │
├────────────────────────────────────┤
│      Communication Fabric          │
├────────────────────────────────────┤
│       Transport / OS / HW          │
└────────────────────────────────────┘
```

But something is still missing.

We have:

```text
who owns state
```

and:

```text
who can execute work
```

but we have not yet fully defined:

> **Who is allowed to do what?**

That is the **Capability & Authority Fabric**.

# 62. The Next Step

The next architectural transformation is therefore:

# **Part XLIII — NROS Capability & Authority Fabric**

It will define:

```text
Identity
   ↓
Principal
   ↓
Capability
   ↓
Authority
   ↓
Policy
   ↓
Action
```

and answer questions such as:

- Can this node publish?
- Can this agent command the robot?
- Can this component modify navigation state?
- Can this process access a device?
- Can an agent delegate authority?
- What happens when authority expires?
- How are safety-critical operations separated from ordinary application permissions?
- How can a distributed NROS system prove **why an action was permitted**?

That layer is what turns the previous runtime architecture into a **governed robotics execution system**, rather than merely a faster ROS-like middleware.

# NROS — Part XLIII: Capability & Authority Fabric

The previous layers established:

```text
Communication → Execution → Time → State
```

The missing question is:

> **Who is permitted to cause a state transition or physical effect?**

In ROS, authorization is generally left to the surrounding operating system, network, application, or deployment.

NROS should make authority explicit.

The fundamental pipeline becomes:

```text
Identity
   ↓
Capability
   ↓
Policy
   ↓
Authorization
   ↓
Work
   ↓
State Transition
   ↓
Physical Effect
```

## 1. Identity Is Not Authority

This distinction is foundational.

A component may identify itself as:

```text
navigation-agent
```

but that does **not** automatically mean it can:

```text
move_robot
```

Likewise:

```text
process = controller
```

does not imply unlimited authority.

Therefore:

```text
Identity ≠ Capability ≠ Authority
```

# 2. Principal

NROS can introduce a generic:

```text
Principal
```

A principal may represent:

```text
human
robot
component
agent
service
device
runtime
operator
automation
```

Every security-sensitive operation can be attributed to a principal.

# 3. Principal Identity

Conceptually:

```text
Principal
├── principal_id
├── identity
├── credentials
├── trust domain
└── attributes
```

The identity mechanism itself can vary by deployment.

NROS should not hard-code one authentication technology into its semantic model.

# 4. Capability

A capability represents an authority that can be exercised.

Examples:

```text
read(robot.pose)

write(navigation.goal)

command(motor.velocity)

operate(camera)

configure(controller)

admin(runtime)
```

Capabilities should be explicit.

# 5. Capability ≠ API Endpoint

A topic or RPC endpoint describes **how** something is accessed.

A capability describes:

> **Whether the principal is allowed to access it, and under what constraints.**

Therefore:

```text
API:
    /navigation/goal

Capability:
    submit_navigation_goal
```

These should remain conceptually separate.

# 6. Capability Scope

Capabilities should be scoped.

Instead of:

```text
move_robot
```

prefer:

```text
move_robot(
    robot = robot-01,
    region = workspace-A,
    max_velocity = 0.5m/s
)
```

Authority can therefore be constrained.

# 7. Least Authority

The default principle should be:

> **Give a component only the authority required for its role.**

For example:

```text
Telemetry
    read pose
    read battery
    ✗ write motor commands
```

Navigation:

```text
read perception
write navigation goal
request motion
✗ modify safety configuration
```

Safety controller:

```text
read system state
command emergency stop
```

# 8. Authority Hierarchy

A useful conceptual hierarchy:

```text
SYSTEM
  ↓
SAFETY
  ↓
MISSION
  ↓
CONTROL
  ↓
APPLICATION
  ↓
OBSERVATION
```

But this is not necessarily a simple universal hierarchy.

Authority should be governed by explicit policy.

# 9. Safety Dominance

For certain operations:

```text
Safety Stop
```

must dominate:

```text
Navigation Command
```

Therefore:

```text
navigation → request stop
```

cannot override:

```text
safety → stop
```

This is an authority relation.

# 10. Policy

Capabilities answer:

> What can this principal potentially do?

Policy answers:

> **Under what conditions may it do it now?**

Example:

```text
Agent has:
    command(robot)

Policy:
    only when robot.mode == AUTONOMOUS
    max_speed <= 0.5m/s
    region != restricted
```

# 11. Policy Evaluation

The authorization pipeline becomes:

```text
Request
   ↓
Identify Principal
   ↓
Resolve Capabilities
   ↓
Evaluate Policy
   ↓
Check State
   ↓
Check Temporal Constraints
   ↓
ALLOW / DENY / DEFER
```

Notice how the previous fabrics participate.

# 12. Authorization Is Contextual

Authorization should not depend only on identity.

A request may depend on:

```text
principal
action
resource
current state
location
time
mission
safety state
resource availability
```

Therefore:

```text
Authorization =
f(identity, capability, state, time, context)
```

# 13. Example

An agent requests:

```text
MoveArm(position=X)
```

NROS evaluates:

```text
identity:
    manipulation-agent

capability:
    arm.motion

state:
    robot operational

safety:
    enabled

workspace:
    X permitted

temporal:
    deadline achievable
```

Only then:

```text
ALLOW
```

# 14. Authorization Before Scheduling

This is an important architectural rule.

Do not:

```text
schedule
   ↓
execute
   ↓
discover unauthorized
```

Instead:

```text
authorize
   ↓
admit
   ↓
schedule
   ↓
execute
```

This prevents unauthorized work from entering critical execution paths.

# 15. Capability Revocation

Authority must be revocable.

Suppose:

```text
Agent A
```

has:

```text
move_robot
```

Then safety state changes.

NROS can:

```text
REVOKE capability
```

and all future work requiring it becomes invalid.

# 16. Revocation and In-Flight Work

The difficult question is:

> What happens to work that already started?

Possible policies:

```text
ALLOW_TO_FINISH
CANCEL
PREEMPT
ESCALATE
```

Different capabilities require different semantics.

For example:

```text
telemetry upload
```

may finish.

But:

```text
dangerous actuator operation
```

may require immediate interruption.

# 17. Capability Lease

Temporary authority can be represented as:

```text
CapabilityLease
├── principal
├── capability
├── scope
├── issued_at
├── expires_at
└── revocation_state
```

This is useful for:

```text
temporary agents
remote operators
mission-specific permissions
delegated authority
```

# 18. Delegation

An agent may need to delegate a limited capability.

Example:

```text
MissionAgent
     ↓
delegates
     ↓
NavigationAgent
```

But delegation should never automatically grant more authority than the delegator possesses.

This creates a capability constraint:

```text
delegated_authority
    ⊆
delegator_authority
```

# 19. Delegation Chain

A complete chain could be:

```text
Operator
   ↓
Mission Supervisor
   ↓
Navigation Agent
   ↓
Motion Controller
```

NROS should be able to reconstruct this chain.

# 20. Authority Provenance

Every privileged action should answer:

```text
Who requested it?
Who authorized it?
Under which policy?
Using which capability?
What state was evaluated?
When?
```

For example:

```text
Action #821

Principal:
    navigation-agent

Capability:
    motion.command

Policy:
    autonomous-navigation-v3

State:
    safety=normal

Decision:
    ALLOW

Authority source:
    mission-supervisor
```

This is extremely valuable for auditability.

# 21. Authorization Trace

Combining this with the temporal and state fabrics:

```text
Observation
    ↓
State
    ↓
Agent Decision
    ↓
Authorization
    ↓
Scheduling
    ↓
Execution
    ↓
Effect
```

Every step can have provenance.

# 22. Policy Is Not Just Security

Policies can govern more than access.

Examples:

```text
resource limits
energy consumption
maximum speed
operating region
mission phase
temporal restrictions
safety modes
```

Thus NROS policy becomes a general runtime governance mechanism.

# 23. Safety Policy

For example:

```text
IF
    safety_mode == EMERGENCY
THEN
    deny all motion commands
    except safety-authorized commands
```

This policy should be evaluated inside the runtime boundary.

# 24. Spatial Policy

A robot may be prohibited from entering:

```text
restricted_zone
```

The policy can evaluate:

```text
requested trajectory
+
current position
+
authorized region
```

and reject the action.

# 25. Temporal Policy

A capability might only be valid:

```text
08:00 → 18:00
```

or:

```text
mission phase = DOCKING
```

Again, authorization depends on runtime context.

# 26. Resource Policy

An agent may have:

```text
GPU capability
```

but only:

```text
2GB memory
```

or:

```text
30% GPU budget
```

This ties authority to the Resource Fabric.

# 27. Capability Composition

A complex operation may require multiple capabilities.

Example:

```text
ExecuteDocking
```

requires:

```text
read(localization)
read(battery)
command(motion)
operate(docking_interface)
```

Authorization can therefore become:

```text
capability_set ⊇ required_capabilities
```

# 28. Atomic Authorization

For critical operations, authorization should be evaluated as one decision.

Avoid:

```text
check A
   ↓
state changes
   ↓
check B
   ↓
execute
```

because the context may become invalid between checks.

Prefer:

```text
capture context
   ↓
evaluate policy
   ↓
admit operation
```

with a defined validity interval.

# 29. Authority + Temporal Contract

An authorization decision may include:

```text
valid_until
```

For example:

```text
ALLOW
for:
    100ms
```

This avoids using stale authorization indefinitely.

# 30. Authority + State Version

An authorization decision can also be bound to state:

```text
authorized_if:
    safety_state.version == 812
```

If the safety state changes:

```text
version 813
```

the authorization may become invalid.

This is powerful for race prevention.

# 31. TOCTOU Protection

Traditional systems can suffer from:

```text
check
 ↓
state changes
 ↓
use
```

NROS can reduce this with:

```text
authorization context
+
state version
+
temporal validity
```

Conceptually:

```text
Authorize(state=v812)
      ↓
Execute only if
state still compatible
```

# 32. Authority Boundary

We can now establish a critical architectural boundary:

```text
┌─────────────────────────────┐
│       NROS Control Plane    │
│                             │
│ Identity                    │
│ Capability                  │
│ Policy                      │
│ State                       │
│ Time                        │
│ Scheduler                   │
└──────────────┬──────────────┘
               │
        authorized effect
               ↓
┌─────────────────────────────┐
│       Effect Plane          │
│                             │
│ Drivers                     │
│ Actuators                   │
│ Devices                     │
│ External Systems            │
└─────────────────────────────┘
```

The effect plane should never bypass the control plane accidentally.

# 33. Device Capabilities

Hardware can expose capabilities:

```text
camera.capture
lidar.read
motor.command
gpio.write
plc.read
plc.write
```

But the device capability itself does not mean every application gets it.

The runtime maps:

```text
principal
→ allowed capability
→ device operation
```

# 34. Capability-Based Device Access

This is especially relevant to NROS's potential embedded/industrial direction.

Instead of:

```text
process → /dev/device
```

conceptually:

```text
component
    ↓
capability
    ↓
device service
    ↓
hardware
```

This provides a much clearer security boundary.

# 35. Agent Authority

This becomes particularly important for autonomous agents.

An agent should not receive:

```text
root-like robot authority
```

just because it is intelligent.

Instead:

```text
Agent
  ↓
bounded capability set
```

Example:

```text
NavigationAgent:
    ✓ read map
    ✓ read localization
    ✓ submit navigation goals
    ✗ modify safety policy
    ✗ flash firmware
    ✗ disable watchdog
```

# 36. Agent Sandbox

A cognitive agent can therefore execute inside:

```text
Agent Sandbox
```

with:

```text
memory limits
CPU limits
network limits
capabilities
time limits
effect permissions
```

This combines:

```text
Scheduler
+
Resource Fabric
+
Capability Fabric
```

# 37. Tool Authority

An agent may have tools:

```text
navigate()
inspect_camera()
query_map()
execute_command()
```

Each tool should map to a capability.

For example:

```text
navigate()
    ↓
capability: navigation.request
```

rather than granting the agent unrestricted internal access.

# 38. Tool Calls Become Governed Work

The pipeline becomes:

```text
Agent
  ↓
Tool Request
  ↓
Capability Check
  ↓
Policy
  ↓
State Check
  ↓
Temporal Check
  ↓
Work Item
  ↓
Scheduler
  ↓
Execution
```

This is the core pattern for **agent-native robotics**.

# 39. Human Authority

Human operators should also be modeled as principals.

For example:

```text
Operator
    ↓
manual_override
```

could temporarily supersede:

```text
autonomous navigation
```

but only under explicit policy.

# 40. Manual Override

A safety-critical override might produce:

```text
AUTONOMOUS
    ↓
MANUAL_OVERRIDE
```

with:

```text
autonomous motion commands
    → denied
```

This should be represented as state + authority, not merely a UI flag.

# 41. Emergency Authority

Emergency authority may have special semantics:

```text
EmergencyStop
```

should remain available even when ordinary mission authority is revoked.

But:

```text
Emergency authority
```

should not imply:

```text
arbitrary system administration
```

Capabilities remain granular.

# 42. Policy Conflict

What if:

```text
Mission Policy:
    move forward

Safety Policy:
    do not move
```

NROS needs deterministic conflict resolution.

A possible principle:

```text
safety constraints
    dominate
mission intent
```

But this should be encoded as explicit policy precedence rather than assumed everywhere.

# 43. Policy Engine

Conceptually:

```text
PolicyEngine
├── identity rules
├── capability rules
├── state predicates
├── temporal predicates
├── resource predicates
├── safety constraints
└── precedence
```

The engine returns:

```text
ALLOW
DENY
DEFER
ALLOW_WITH_CONSTRAINTS
```

# 44. Allow With Constraints

This is particularly useful.

Instead of:

```text
DENY MoveArm
```

the policy can return:

```text
ALLOW
max_velocity = 0.1m/s
workspace = zone-A
deadline = 50ms
```

The runtime then constrains the resulting work.

# 45. Policy Compilation

For real-time paths, dynamic policy evaluation may be too expensive.

NROS can therefore distinguish:

```text
control-plane policy evaluation
```

from:

```text
compiled execution constraints
```

Example:

```text
Policy
  ↓
validated
  ↓
compiled constraint
  ↓
fast runtime check
```

This is important for deterministic control.

# 46. Capability Cache

Similarly, frequently used capabilities can be represented as short-lived validated tokens.

But:

```text
cached authority
```

must have:

```text
expiration
revocation semantics
scope
state assumptions
```

Otherwise stale permissions become dangerous.

# 47. Capability Attenuation

A powerful concept is:

```text
full capability
      ↓
attenuate
      ↓
restricted capability
```

For example:

```text
motion.command
```

becomes:

```text
motion.command(
    max_speed = 0.2m/s,
    region = zone-A
)
```

This is ideal for agent delegation.

# 48. Authority as a Data Object

NROS can model:

```text
AuthorityGrant
├── issuer
├── subject
├── capability
├── scope
├── constraints
├── issued_at
├── expires_at
└── provenance
```

This makes authority inspectable and auditable.

# 49. Runtime Decision Record

Every privileged execution can produce:

```text
AuthorizationDecision
├── request_id
├── principal
├── capability
├── policy
├── state_version
├── temporal_context
├── decision
└── constraints
```

This integrates directly with the Trace Fabric.

# 50. Full NROS Governance Pipeline

We can now connect all the architecture layers:

```text
              Observation
                   ↓
             State Update
                   ↓
                Agent
                   ↓
              Intent
                   ↓
              Identity
                   ↓
             Capability
                   ↓
               Policy
                   ↓
        State + Time Validation
                   ↓
               Admission
                   ↓
              Scheduler
                   ↓
               Executor
                   ↓
                Device
                   ↓
                Effect
                   ↓
             Observation
```

This is becoming the central NROS model.

# 51. The Five-Fabric Architecture

At this point NROS can be viewed as five cooperating fabrics:

```text
┌──────────────────────────────────────┐
│          Capability Fabric           │
│ Identity • Authority • Policy        │
├──────────────────────────────────────┤
│             State Fabric             │
│ Ownership • Version • Memory         │
├──────────────────────────────────────┤
│           Temporal Fabric             │
│ Clocks • Deadlines • Causality       │
├──────────────────────────────────────┤
│          Execution Fabric             │
│ Work • Scheduler • Executor          │
├──────────────────────────────────────┤
│       Communication Fabric            │
│ Events • Streams • RPC • Transport   │
└──────────────────────────────────────┘
```

The fabrics are not independent.

They form a dependency chain.

# 52. Dependency Graph

```text
Communication
      ↓
Observation
      ↓
State
      ↓
Intent
      ↓
Capability / Policy
      ↓
Temporal Admission
      ↓
Execution
      ↓
Effect
      ↓
Observation
```

And the feedback path continuously updates:

```text
state
time
authority
scheduling
```

# 53. What NROS Is Becoming

At the beginning:

```text
ROS
=
robotics middleware
```

The NROS architecture is now closer to:

```text
NROS
=
distributed robotic runtime
+
state fabric
+
temporal runtime
+
execution scheduler
+
capability system
+
agent execution environment
```

The distinction is substantial.

# 54. NROS Core Principle

A concise formulation is emerging:

> **Every meaningful physical effect in NROS should be attributable to an authorized work item executing under a valid temporal, state, resource, and capability context.**

That single principle ties together nearly everything built so far.

# 55. The Remaining Major Problem

We can now authorize and execute actions.

But distributed robots fail.

Networks disconnect.

Processes crash.

Machines reboot.

Sensors disappear.

Agents become unavailable.

Capabilities expire.

State becomes stale.

Therefore NROS needs a first-class mechanism for:

```text
failure detection
failure classification
containment
recovery
reconciliation
restart
failover
degraded operation
```

That takes us to the next architectural layer:

# **Part XLIV — NROS Supervision & Recovery Fabric**

The core transformation will be:

```text
ROS:
    process died
        ↓
    restart process

NROS:
    execution failed
        ↓
    classify failure
        ↓
    determine affected state
        ↓
    revoke/contain authority
        ↓
    recover or degrade
        ↓
    reconcile state
        ↓
    restore valid execution
```

This is where **fault tolerance becomes part of the runtime semantics rather than an external launch-script concern**.

# NROS — Part XLIV: Supervision & Recovery Fabric

The previous architecture established **authority over execution**.

Now we address the unavoidable reality of robotics:

> **Components fail, networks partition, hardware becomes unavailable, state becomes stale, and autonomous work can become invalid while the process itself is still alive.**

Traditional ROS commonly treats these problems through node monitoring, launch systems, watchdogs, lifecycle mechanisms, and application-level recovery.

NROS should elevate recovery into a **first-class runtime fabric**.

# 1. Failure Is a First-Class Event

NROS should not model failure simply as:

```text
process exited
```

A process can remain alive while being functionally broken.

Therefore:

```text
Failure
├── Process failure
├── Thread failure
├── Communication failure
├── Timing failure
├── State failure
├── Resource failure
├── Capability failure
├── Device failure
├── Sensor failure
├── Safety failure
└── Semantic failure
```

# 2. Liveness ≠ Health

This distinction is fundamental.

A node may respond to:

```text
ping()
```

while its controller is producing invalid commands.

Therefore:

```text
liveness
≠
health
```

NROS should expose both.

# 3. Health Model

A component can have:

```text
Health
├── Alive
├── Responsive
├── Functional
├── Degraded
├── Failed
└── Unknown
```

For example:

```text
Controller
    Alive       ✓
    Responsive  ✓
    Functional  ✗
```

The supervisor should detect the semantic failure.

# 4. Health Signals

Health can be established through:

```text
heartbeat
watchdog
progress signal
deadline compliance
invariant checks
state validation
resource metrics
application health reports
```

No single mechanism is sufficient for every component.

# 5. Progress Monitoring

A particularly useful primitive is:

```text
Progress
```

Suppose a planner is alive but has produced no meaningful progress for:

```text
10 seconds
```

NROS may classify:

```text
progress_timeout
```

even though the process has not crashed.

# 6. Failure Detection

The Supervision Fabric can continuously evaluate:

```text
component
   ↓
health signals
   ↓
failure detector
   ↓
failure classification
```

The result becomes a structured event:

```text
FailureDetected
```

# 7. Failure Object

Conceptually:

```text
Failure
├── failure_id
├── subject
├── category
├── severity
├── detected_at
├── evidence
├── affected_resources
├── affected_state
└── recovery_hint
```

This gives failures identity and provenance.

# 8. Failure Severity

For example:

```text
INFO
WARNING
DEGRADED
ERROR
CRITICAL
FATAL
```

But severity should be contextual.

A failed camera may be:

```text
CRITICAL
```

for visual navigation,

but:

```text
WARNING
```

for a robot currently performing a task that does not require vision.

# 9. Dependency Graph

NROS needs an explicit dependency model.

Example:

```text
Navigation
 ├── Localization
 ├── Map
 ├── Planner
 └── Motion Controller
```

If:

```text
Localization → FAILED
```

then Navigation may become:

```text
DEGRADED
```

rather than continuing blindly.

# 10. Failure Propagation

Failures should propagate through dependencies according to policy.

```text
Sensor failure
      ↓
Perception degraded
      ↓
Planner degraded
      ↓
Navigation suspended
```

But propagation should be selective.

# 11. Failure Containment

A failure should not automatically bring down the whole runtime.

For example:

```text
Telemetry
   ✗
```

should not necessarily cause:

```text
Motion Controller
   ✗
```

Therefore NROS requires **fault containment boundaries**.

# 12. Fault Domains

A runtime can be divided into:

```text
Fault Domain A
├── perception
└── visualization

Fault Domain B
├── control
└── safety

Fault Domain C
└── mission planning
```

A failure inside one domain should remain contained where possible.

# 13. Isolation

Isolation can occur at several levels:

```text
process
thread
task
memory
capability
device
network
runtime instance
```

The strongest isolation mechanisms should be reserved for components whose failure could affect safety.

# 14. Supervision Tree

NROS can borrow a useful structural concept from actor-style runtimes:

```text
NROS Runtime
│
├── Safety Supervisor
│   ├── Controller
│   └── Actuator Gateway
│
├── Navigation Supervisor
│   ├── Localization
│   └── Planner
│
└── Perception Supervisor
    ├── Camera
    └── Object Detector
```

A supervisor owns recovery policy for its subtree.

# 15. Supervisor Responsibilities

A supervisor should be able to:

```text
observe
classify
contain
restart
replace
degrade
escalate
recover
```

It should not itself become an uncontrolled source of authority.

# 16. Restart Is Not Recovery

This is one of the most important NROS distinctions.

Traditional approach:

```text
crash
 ↓
restart process
```

NROS:

```text
failure
 ↓
contain
 ↓
determine state impact
 ↓
recover state
 ↓
restore capabilities
 ↓
reconcile hardware
 ↓
resume valid work
```

A restart is only one recovery mechanism.

# 17. Recovery Strategies

NROS can define standard strategies:

```text
RETRY
RESTART
RESET
REINITIALIZE
FAILOVER
ROLLBACK
RECONCILE
DEGRADE
ESCALATE
ABORT
```

# 18. Retry

Useful for transient failures:

```text
network request failed
```

Policy:

```text
retry
backoff
retry
```

But retry must be bounded.

# 19. Exponential Backoff

A retry policy can use:

```text
100ms
200ms
400ms
800ms
...
```

with:

```text
max_attempts
max_delay
deadline
```

This prevents recovery loops from consuming all resources.

# 20. Restart

For isolated components:

```text
failed process
    ↓
terminate
    ↓
recreate
    ↓
initialize
```

But the supervisor must determine whether the component is safe to restart.

# 21. Reinitialization

Some hardware cannot simply be restarted.

Example:

```text
camera
```

may require:

```text
disconnect
reset
reopen
reconfigure
calibrate
```

Thus:

```text
restart process
```

and:

```text
reinitialize resource
```

are different operations.

# 22. Reset

A controller may support:

```text
reset()
```

without destroying the process.

This is useful when internal state is corrupted but the process remains healthy.

# 23. Failover

Suppose:

```text
Planner A
```

fails.

A standby:

```text
Planner B
```

may take ownership.

The sequence becomes:

```text
A fails
 ↓
authority revoked
 ↓
state checkpoint recovered
 ↓
B acquires lease
 ↓
B resumes
```

This connects directly to the State and Capability Fabrics.

# 24. Ownership Transfer

Failover must not allow both instances to act simultaneously.

Therefore:

```text
A:
    lease revoked

B:
    lease acquired
```

must happen under an explicit ownership protocol.

# 25. Fencing

If a failed component might still be alive but unreachable, NROS needs **fencing**.

Example:

```text
Planner A
```

becomes partitioned.

The supervisor starts:

```text
Planner B
```

Without fencing:

```text
A → commands
B → commands
```

could happen simultaneously.

Fencing prevents stale authority from producing effects.

# 26. Fencing Is More Important Than Restart

For physical systems:

```text
restart safety
```

is less important than:

```text
preventing stale components from retaining authority
```

Therefore the sequence should often be:

```text
detect failure
 ↓
fence authority
 ↓
recover
 ↓
restart/replace
```

not:

```text
restart
 ↓
hope old instance is gone
```

# 27. Network Partition

Distributed robots may experience:

```text
network partition
```

but each machine may continue operating.

NROS must distinguish:

```text
local liveness
```

from:

```text
distributed connectivity
```

# 28. Partition Policy

For each component:

```text
ON_PARTITION
```

may specify:

```text
continue
degrade
pause
stop
safe-state
```

Example:

```text
Telemetry
    → continue locally

Mission Planner
    → pause

Remote Actuator Authority
    → revoke
```

# 29. Graceful Degradation

A robot should not always choose:

```text
everything works
```

or:

```text
everything stops
```

Instead:

```text
FULL
 ↓
DEGRADED
 ↓
MINIMAL
 ↓
SAFE
```

# 30. Degraded Mode

Example:

```text
GPS unavailable
```

Navigation may switch:

```text
GPS + SLAM
```

to:

```text
SLAM only
```

if the policy allows it.

This is a recovery transition.

# 31. Capability Reduction During Degradation

A degraded system should often lose authority.

Example:

```text
NORMAL:
    max velocity = 1.0 m/s

DEGRADED:
    max velocity = 0.3 m/s

SAFE:
    motion prohibited
```

Thus:

```text
health ↓
   ⇒
authority ↓
```

where appropriate.

# 32. Recovery State Machine

A component can have:

```text
HEALTHY
   ↓
DEGRADED
   ↓
FAILED
   ↓
CONTAINED
   ↓
RECOVERING
   ↓
VALIDATING
   ↓
HEALTHY
```

Or:

```text
FAILED
   ↓
UNRECOVERABLE
   ↓
ESCALATED
```

# 33. Recovery Validation

Restarting a process does not prove recovery.

NROS should validate:

```text
process alive
state valid
dependencies available
capabilities restored
timing valid
hardware synchronized
health restored
```

Only then:

```text
RECOVERY_COMPLETE
```

# 34. State Reconciliation After Recovery

This connects directly to Part XLII.

```text
checkpoint
    +
fresh observations
    +
external device state
    ↓
reconciliation
```

Only the reconciled state becomes authoritative.

# 35. Work Recovery

What happens to an interrupted work item?

NROS should classify it:

```text
UNKNOWN
ABORTED
RETRYABLE
RESUMABLE
COMPLETED
FAILED
```

This prevents duplicate physical effects.

# 36. Idempotency

Suppose:

```text
Dock()
```

was sent immediately before a crash.

After restart, blindly retrying:

```text
Dock()
```

may be unsafe.

NROS should support:

```text
operation_id
```

and idempotency semantics.

# 37. Exactly-Once Is Not Always Possible

Distributed physical systems cannot simply assume:

```text
exactly once
```

for every action.

Instead NROS should explicitly support:

```text
at-most-once
at-least-once
idempotent retry
transactional effect
uncertain outcome
```

The correct semantic depends on the operation.

# 38. Uncertain Outcome

A particularly important state:

```text
UNKNOWN_OUTCOME
```

Example:

```text
motor command sent
 ↓
network lost
 ↓
did motor receive it?
```

NROS must not automatically assume:

```text
failed
```

or:

```text
completed
```

It may need an observation/reconciliation step.

# 39. Recovery Journal

Critical work can be recorded:

```text
WorkJournal
├── submitted
├── admitted
├── started
├── effect_requested
├── effect_observed
└── completed
```

After crash:

```text
journal
 ↓
determine last known stage
 ↓
reconcile
```

This creates deterministic recovery behavior.

# 40. Recovery Checkpoints

Long-running work can periodically checkpoint:

```text
Mission
 ├── phase 1 ✓
 ├── phase 2 ✓
 ├── phase 3 ← checkpoint
 └── phase 4
```

After failure:

```text
resume from phase 3
```

rather than restarting from zero.

# 41. Recovery Policy

Each work type should declare recovery semantics.

For example:

```text
NavigationGoal:
    resumable = true

FirmwareUpdate:
    resumable = policy-dependent

EmergencyStop:
    never retry automatically

TelemetryUpload:
    retryable = true
```

This avoids generic recovery assumptions.

# 42. Recovery Authority

Recovery actions themselves require authorization.

A supervisor may have:

```text
restart(component)
```

but not necessarily:

```text
disable_safety()
```

Thus:

```text
Recovery
    ⊂
Governed Authority
```

# 43. Recovery Escalation

If local recovery fails:

```text
local supervisor
    ↓
parent supervisor
    ↓
runtime supervisor
    ↓
system supervisor
    ↓
human operator
```

Escalation becomes structured rather than ad hoc.

# 44. Human Escalation

For unresolved failures:

```text
REQUIRES_OPERATOR
```

can become a first-class state.

The runtime can expose:

```text
failure
evidence
attempted recovery
current state
recommended action
```

instead of merely:

```text
node crashed
```

# 45. Recovery Evidence

Every recovery attempt should produce:

```text
RecoveryRecord
├── failure_id
├── strategy
├── attempt
├── start_time
├── end_time
├── result
└── evidence
```

This integrates with NROS observability.

# 46. Recovery Storms

A badly designed supervisor can create:

```text
failure
 ↓
restart
 ↓
failure
 ↓
restart
 ↓
...
```

NROS should detect:

```text
recovery_loop
```

and transition to:

```text
ESCALATE
```

or:

```text
SAFE
```

# 47. Circuit Breaker

For unstable dependencies:

```text
CLOSED
 ↓ failure threshold
OPEN
 ↓ cooldown
HALF_OPEN
 ↓ success
CLOSED
```

This is useful for network and service dependencies.

# 48. Watchdog

A watchdog can monitor deadlines:

```text
expected heartbeat:
    every 10ms

observed:
    35ms
```

Then:

```text
deadline violation
```

becomes a temporal failure.

Notice again how fabrics converge:

```text
Temporal Fabric
       ↓
Supervision Fabric
```

# 49. Runtime Invariants

NROS can continuously evaluate invariants:

```text
Invariant:
    exactly one controller owns motor authority
```

If violated:

```text
FAULT
```

This is stronger than merely checking process health.

# 50. Safety Invariants

Examples:

```text
never:
    actuator enabled
    AND safety_state == EMERGENCY

never:
    two controllers own same actuator

always:
    command velocity <= configured limit
```

These invariants become runtime guardrails.

# 51. Supervisor as Runtime Guardian

The Supervision Fabric therefore becomes:

```text
             ┌────────────────────┐
             │    Supervisor      │
             ├────────────────────┤
             │ Observe            │
             │ Detect             │
             │ Classify           │
             │ Contain            │
             │ Recover            │
             │ Validate           │
             │ Escalate           │
             └─────────┬──────────┘
                       │
          ┌────────────┼────────────┐
          ↓            ↓            ↓
       State        Authority      Work
```

# 52. The NROS Failure Loop

The complete lifecycle becomes:

```text
Healthy
  ↓
Observation
  ↓
Anomaly
  ↓
Failure Detection
  ↓
Classification
  ↓
Containment
  ↓
Authority Revocation
  ↓
Recovery
  ↓
State Reconciliation
  ↓
Validation
  ↓
Authority Restoration
  ↓
Resume
```

This is considerably more powerful than a process supervisor.

# 53. NROS Six-Fabric Architecture

We can now extend the architecture:

```text
┌──────────────────────────────────────┐
│       Capability & Authority         │
├──────────────────────────────────────┤
│             State                    │
├──────────────────────────────────────┤
│           Supervision                │
├──────────────────────────────────────┤
│            Temporal                  │
├──────────────────────────────────────┤
│            Execution                │
├──────────────────────────────────────┤
│          Communication              │
└──────────────────────────────────────┘
```

But there is another missing dimension.

The runtime now knows:

```text
what happened
what state exists
who is authorized
when things happened
what is executing
what failed
```

Yet we need to answer:

> **How do we observe, explain, trace, diagnose, and prove all of this?**

That requires a dedicated **Observability & Evidence Fabric**.

# Part XLV — NROS Observability & Evidence Fabric

The next layer will unify:

```text
Logs
Metrics
Traces
Events
State History
Authorization Decisions
Failure Records
Recovery Records
Execution History
Causal Graphs
```

into one coherent evidence model.

The central idea will be:

```text
NROS should not merely execute a robot.

It should be able to explain, after the fact,
what happened, why it happened,
who caused it, under which authority,
from which state,
at what time,
and what the runtime did when reality diverged.
```

That is the point where NROS begins moving from **robot middleware** toward a **verifiable robotic runtime**.

# NROS — Part XLV: Observability & Evidence Fabric

The previous layers gave NROS the ability to **communicate, schedule, govern, maintain state, and recover**.

But a complex autonomous runtime needs another property:

> **It must be possible to reconstruct what happened and why.**

Not merely:

```text
"the robot stopped."
```

but:

```text
At T:
    agent A observed X
    state changed from V81 → V82
    policy P authorized action W
    scheduler admitted W
    controller executed W
    hardware reported Y
    invariant Z failed
    supervisor revoked authority
    recovery R began
    state was reconciled
```

That is the purpose of the **Observability & Evidence Fabric**.

# 1. Observability Is More Than Logging

Traditional systems often begin with:

```text
logs/
metrics/
traces/
```

NROS needs a broader model:

```text
Evidence
├── Events
├── Logs
├── Metrics
├── Traces
├── State transitions
├── Decisions
├── Authorization
├── Failures
├── Recovery
├── Resource usage
└── Physical observations
```

The important distinction is:

```text
Telemetry = information about execution

Evidence = information that can establish what happened
```

# 2. The Evidence Object

Every important runtime occurrence should be representable as an evidence record.

Conceptually:

```text
Evidence
├── evidence_id
├── event_type
├── subject
├── timestamp
├── causal_context
├── state_context
├── authority_context
├── execution_context
├── payload
└── provenance
```

This creates a common foundation for the entire runtime.

# 3. Evidence Is Append-Oriented

Runtime state changes.

Evidence should generally be append-oriented:

```text
E1
E2
E3
E4
...
```

rather than repeatedly overwriting historical information.

This gives NROS an auditable history.

# 4. Event Identity

Each significant event should have an identity:

```text
EventId
```

For example:

```text
evt_8f71...
```

This allows different systems to refer to exactly the same occurrence.

# 5. Correlation Identity

A single operation may produce many events.

Example:

```text
NavigationGoal
```

can generate:

```text
submitted
authorized
scheduled
started
progress
completed
```

A:

```text
CorrelationId
```

can bind them together.

```text
Goal #42
 ├── authorization
 ├── scheduling
 ├── execution
 └── completion
```

# 6. Causality

Correlation says:

> these events belong to the same operation.

Causality says:

> this event caused or depended upon that event.

NROS therefore needs causal relationships.

```text
Observation
   ↓
Decision
   ↓
Command
   ↓
Effect
```

# 7. Causal Graph

The runtime can maintain a graph:

```text
       Observation
            │
            ▼
      State Transition
            │
            ▼
          Intent
            │
            ▼
       Authorization
            │
            ▼
        Work Item
            │
            ▼
         Execution
            │
            ▼
        Physical Effect
```

This becomes the foundation for deep diagnosis.

# 8. Why Causality Matters

Suppose the robot entered the wrong room.

A simple log might say:

```text
Navigation failed.
```

A causal trace can reveal:

```text
camera observation
     ↓
object recognition
     ↓
incorrect semantic state
     ↓
planner decision
     ↓
authorized trajectory
     ↓
motion command
```

Now the actual failure can be identified.

# 9. Temporal Context

Every evidence item should carry temporal information appropriate to its clock domain.

For example:

```text
timestamp
logical_time
monotonic_time
clock_domain
```

This is especially important across distributed machines.

# 10. Distributed Time

Suppose:

```text
Robot CPU:
    T = 102.4

Edge computer:
    T = 99.8
```

Raw timestamps cannot automatically establish causal ordering.

NROS should therefore support:

```text
logical ordering
causal ordering
clock synchronization metadata
```

# 11. State Context

An event should be able to reference the state in which it occurred:

```text
state_version = 821
```

Example:

```text
AuthorizationDecision
    state_version = safety:821
```

Later investigators can ask:

> What did the runtime believe when it authorized the operation?

# 12. Authority Context

Likewise:

```text
principal
capability
policy
decision
```

should be traceable.

A privileged action can therefore be represented as:

```text
Action
 ├── principal
 ├── capability
 ├── policy
 ├── state
 ├── time
 └── result
```

# 13. Execution Context

Execution evidence should identify:

```text
runtime
worker
scheduler
work_id
attempt
priority
deadline
```

This connects the execution system to observability.

# 14. Work Lifecycle

A work item might generate:

```text
WORK_CREATED
WORK_AUTHORIZED
WORK_ADMITTED
WORK_QUEUED
WORK_STARTED
WORK_PROGRESS
WORK_COMPLETED
```

or:

```text
WORK_FAILED
WORK_CANCELLED
WORK_PREEMPTED
WORK_TIMED_OUT
```

This gives every operation a lifecycle.

# 15. Event Taxonomy

NROS should standardize event classes.

```text
Runtime
├── Started
├── Stopped
└── ConfigurationChanged

State
├── Created
├── Updated
├── Invalidated
└── Reconciled

Work
├── Submitted
├── Started
├── Completed
└── Failed

Authority
├── Granted
├── Used
├── Revoked
└── Expired

Supervision
├── FailureDetected
├── RecoveryStarted
└── RecoveryCompleted
```

# 16. Logs Become Structured Evidence

Instead of:

```text
"planner failed"
```

NROS should prefer structured information:

```text
{
    event: PlannerFailure,
    planner: planner-01,
    reason: deadline_missed,
    deadline: 50ms,
    observed: 73ms,
    work_id: W821
}
```

The exact serialization can vary.

The semantic model should not.

# 17. Metrics

Metrics provide aggregate behavior:

```text
control_loop_latency
planner_latency
message_rate
queue_depth
CPU_usage
memory_usage
deadline_misses
recovery_count
```

Metrics are valuable, but they are not enough to explain individual events.

# 18. Metrics + Events

For example:

```text
Metric:
    planner_latency_p99 = 74ms
```

tells us:

```text
performance is degrading
```

An event tells us:

```text
Work W821 missed its 50ms deadline.
```

The two should be correlated.

# 19. Traces

A trace follows one operation across components.

Example:

```text
Trace #42

Agent
 ↓
Planner
 ↓
Controller
 ↓
Transport
 ↓
Motor Gateway
```

Each span can include:

```text
start
end
status
attributes
causal links
```

# 20. Robotics Trace

A robotics trace can be richer than a conventional distributed-system trace:

```text
Trace
├── observation
├── perception
├── planning
├── authorization
├── scheduling
├── control
├── hardware effect
└── feedback
```

This is extremely useful for autonomous behavior.

# 21. Observation-to-Effect Trace

NROS can answer:

> Which observation ultimately caused this motor command?

Example:

```text
Camera Frame 981
      ↓
Detection 412
      ↓
World State v183
      ↓
Agent Decision 77
      ↓
Navigation Goal 91
      ↓
Trajectory 52
      ↓
Motor Command 992
```

That is **causal observability**.

# 22. Effect-to-Cause Analysis

The reverse query is equally useful:

> Why did this actuator move?

NROS traverses:

```text
Motor Effect
   ↑
Command
   ↑
Controller
   ↑
Work
   ↑
Authorization
   ↑
Agent Decision
   ↑
State
   ↑
Observation
```

This can dramatically simplify debugging.

# 23. Evidence Levels

Not every event deserves the same durability.

NROS can classify evidence:

```text
EPHEMERAL
LOCAL
BUFFERED
DURABLE
CRITICAL
ARCHIVAL
```

For example:

```text
high-frequency IMU sample
    → ephemeral/buffered

emergency stop
    → durable/critical
```

# 24. Evidence Policy

Every evidence stream can define:

```text
retention
sampling
priority
durability
privacy
redaction
```

This prevents observability from overwhelming the robot.

# 25. High-Frequency Data

Robotics can generate enormous volumes:

```text
camera
lidar
IMU
point clouds
joint states
```

NROS should not blindly persist all of it.

Instead:

```text
raw stream
   ↓
sampling/filtering
   ↓
evidence references
```

Critical events can preserve pointers to the relevant raw data.

# 26. Evidence Windows

When something fails, the runtime can retain:

```text
before failure
+
failure
+
after failure
```

For example:

```text
T-5s ───── failure ───── T+5s
```

This creates a diagnostic evidence window without permanently storing everything.

# 27. Flight Recorder

This naturally leads to a robotics **flight recorder**.

A circular buffer continuously retains recent evidence:

```text
┌───────────────────────────────┐
│ E991 E992 E993 ... E1200      │
└───────────────────────────────┘
                  ↑
                newest
```

On critical failure:

```text
freeze buffer
persist evidence
```

This is extremely useful for physical-system debugging.

# 28. Incident

A major failure can create:

```text
Incident
```

containing:

```text
Incident
├── trigger
├── affected components
├── timeline
├── causal graph
├── state snapshots
├── authority decisions
├── recovery attempts
└── final outcome
```

# 29. Incident Lifecycle

```text
NORMAL
  ↓
ANOMALY
  ↓
INCIDENT_DETECTED
  ↓
CONTAINMENT
  ↓
RECOVERY
  ↓
VALIDATION
  ↓
RESOLVED
```

The evidence remains available after resolution.

# 30. Evidence Bundles

An incident can be exported as:

```text
NROS Evidence Bundle
```

containing:

```text
manifest
timeline
events
state snapshots
traces
logs
metrics
authorization records
failure records
recovery records
```

This becomes a portable diagnostic artifact.

# 31. Evidence Integrity

For important evidence, NROS should support integrity mechanisms.

Conceptually:

```text
E1
 ↓ hash
E2 + hash(E1)
 ↓
E3 + hash(E2)
```

creating a tamper-evident chain.

This is especially valuable for safety or regulated deployments.

# 32. Provenance

Evidence should identify:

```text
source
producer
runtime
schema
version
timestamp
```

This avoids ambiguous diagnostic records.

# 33. Schema Versioning

Evidence schemas evolve.

Therefore:

```text
EventSchema
```

needs:

```text
schema_id
schema_version
```

An old evidence bundle must remain interpretable after NROS upgrades.

# 34. Deterministic Replay

One of the strongest capabilities NROS can provide is:

```text
Evidence
   ↓
Replay
   ↓
Reconstruct execution
```

But replay must distinguish:

```text
simulation
```

from:

```text
historical reconstruction
```

They are not the same.

# 35. Replay Modes

Possible modes:

```text
OBSERVATIONAL_REPLAY
```

Reconstruct what happened.

```text
DETERMINISTIC_REPLAY
```

Re-run runtime decisions against recorded inputs.

```text
SIMULATION_REPLAY
```

Use recorded inputs in a simulated environment.

```text
COUNTERFACTUAL_REPLAY
```

Ask:

> What would have happened under a different policy?

The last one becomes particularly interesting for agent evaluation.

# 36. Determinism Boundary

Exact replay is difficult when external reality is involved.

Therefore NROS should identify:

```text
deterministic inputs
nondeterministic inputs
external effects
unknown outcomes
```

A replay report can say:

```text
replay fidelity:
    deterministic = 98%
    external effects = unavailable
```

rather than falsely claiming perfect reproduction.

# 37. Causal Debugging

With all previous fabrics combined:

```text
Evidence
   ↓
Causal Graph
   ↓
State History
   ↓
Authority History
   ↓
Execution History
   ↓
Recovery History
```

the runtime can support sophisticated root-cause analysis.

# 38. Root Cause vs Symptom

Example:

```text
Symptom:
    robot stopped
```

Immediate cause:

```text
controller timeout
```

Underlying cause:

```text
CPU starvation
```

Root cause:

```text
unbounded perception workload
```

NROS evidence should preserve enough context to distinguish these levels.

# 39. Observability of Autonomous Agents

Agents introduce another challenge.

An agent may internally reason:

```text
observe
plan
choose
act
reflect
```

NROS does not necessarily need to expose private reasoning.

Instead, it should expose **operational provenance**:

```text
observation references
decision identifier
selected action
policy constraints
execution result
```

This provides useful auditability without requiring unrestricted internal reasoning traces.

# 40. Agent Decision Record

A useful structure:

```text
DecisionRecord
├── decision_id
├── agent
├── input_state
├── input_references
├── selected_intent
├── alternatives_metadata
├── policy_context
├── confidence/score
└── resulting_work
```

The runtime can therefore explain:

```text
what the agent decided
```

without pretending that an opaque internal model is fully inspectable.

# 41. Confidence Provenance

If an agent supplies:

```text
confidence = 0.91
```

NROS should preserve:

```text
confidence source
model/version
input context
```

But, again:

```text
confidence ≠ authority
```

# 42. Evidence and Safety

Safety events deserve elevated evidence guarantees.

For example:

```text
EmergencyStop
SafetyViolation
AuthorityRevoked
ActuatorFault
```

should typically be:

```text
durable
ordered
attributable
tamper-evident
```

according to deployment requirements.

# 43. Evidence Priorities

NROS can define:

```text
P0 — safety-critical
P1 — control-critical
P2 — operational
P3 — diagnostic
P4 — informational
```

This helps control storage and transport under pressure.

# 44. Observability Backpressure

The observability system itself must not destabilize the robot.

Bad architecture:

```text
robot control
   ↓
huge logging workload
   ↓
CPU starvation
   ↓
control failure
```

Therefore evidence collection must have bounded overhead.

# 45. Separate Critical Path

A possible architecture:

```text
Control Path
     │
     ├── minimal evidence emission
     │
     ▼
Evidence Buffer
     │
     ▼
Observability Workers
     │
     ▼
Persistent Store
```

The control loop should not wait on slow storage.

# 46. Evidence Sampling

For noncritical telemetry:

```text
100,000 events/s
```

may become:

```text
sampled metrics
aggregated statistics
selected traces
```

while critical events remain lossless.

# 47. Evidence Query Model

NROS should provide a unified query concept.

Examples:

```text
find all failures affecting navigation
```

```text
show why actuator A moved
```

```text
show all authority grants to agent X
```

```text
reconstruct state at T
```

```text
show recovery attempts for incident I
```

These should not require manually searching unrelated logs.

# 48. Evidence Graph API

Conceptually:

```text
query(event)
    ↓
related_state()
related_work()
related_authority()
related_failures()
related_recovery()
causes()
effects()
```

This is far more powerful than plain log search.

# 49. NROS Observatory

This naturally suggests a runtime observatory:

```text
┌────────────────────────────────────┐
│             NROS Observatory       │
├────────────────────────────────────┤
│ Runtime Health                     │
│ State Graph                        │
│ Work Graph                         │
│ Capability Graph                   │
│ Causal Timeline                    │
│ Failure Map                        │
│ Recovery Status                    │
│ Resource Usage                     │
└────────────────────────────────────┘
```

This would be the operational interface into the runtime.

# 50. Graph of Graphs

At this point NROS can expose several related graphs:

```text
Communication Graph
Execution Graph
State Graph
Capability Graph
Dependency Graph
Causal Graph
```

These should be connected rather than isolated.

# 51. Unified Runtime Graph

Conceptually:

```text
                   ┌───────────────┐
                   │   Principal   │
                   └───────┬───────┘
                           │
                       authority
                           │
                           ▼
Observation ───────→ State ───────→ Intent
    │                   │              │
    │                   │              ▼
    │                   │         Authorization
    │                   │              │
    │                   │              ▼
    └──────────────→ Evidence ←──── Execution
                           │
                           ▼
                        Effect
                           │
                           ▼
                      Observation
```

This is the deeper semantic model of NROS.

# 52. NROS Seven-Fabric Architecture

We can now add Observability:

```text
┌──────────────────────────────────────┐
│       Observability & Evidence       │
├──────────────────────────────────────┤
│       Capability & Authority         │
├──────────────────────────────────────┤
│               State                  │
├──────────────────────────────────────┤
│            Supervision               │
├──────────────────────────────────────┤
│              Temporal                │
├──────────────────────────────────────┤
│              Execution               │
├──────────────────────────────────────┤
│           Communication              │
└──────────────────────────────────────┘
```

But there is still a fundamental issue.

All these fabrics assume that NROS can represent:

```text
messages
events
state
work
capabilities
failures
evidence
```

in a common, versioned, interoperable semantic form.

That requires a **Protocol & Type System** underneath them.

# Part XLVI — NROS Protocol & Type Fabric

The next step is to design the semantic substrate:

```text
Message
Event
Command
Query
Response
State
StateTransition
Work
Capability
Policy
Failure
Recovery
Evidence
```

with:

```text
identity
schema
version
encoding
compatibility
validation
serialization
```

The critical transformation will be from:

```text
ROS message types
```

toward:

```text
NROS typed runtime objects
```

where the protocol itself understands **events, state, work, authority, causality, and lifecycle**, rather than treating everything as merely a message moving between nodes.

# NROS — Part XLVI: Protocol & Type Fabric

We now reach the layer that turns the previous seven fabrics into a **coherent runtime system**.

ROS historically revolves around:

```text
Node
Topic
Message
Service
Action
```

NROS needs a richer semantic vocabulary.

The fundamental unit should no longer be merely:

```text
message
```

but:

> **a typed, identifiable, versioned runtime object with explicit semantics.**

# 1. From Messages to Runtime Objects

The NROS protocol space should distinguish:

```text
┌──────────────────────────────┐
│ Runtime Object               │
├──────────────────────────────┤
│ Message                      │
│ Event                        │
│ Command                      │
│ Query                        │
│ Response                     │
│ State                        │
│ State Transition             │
│ Work                         │
│ Intent                       │
│ Capability                   │
│ Policy                       │
│ Failure                      │
│ Recovery                     │
│ Evidence                     │
└──────────────────────────────┘
```

This distinction matters because these objects have different semantics.

# 2. Message

A message represents transported information:

```text
SensorFrame
JointState
Pose
Velocity
Temperature
```

Its basic semantics are:

```text
producer → consumer
```

A message does not necessarily imply:

```text
authority
execution
state transition
```

# 3. Event

An event represents something that happened:

```text
NodeStarted
StateChanged
WorkCompleted
AuthorityRevoked
FailureDetected
```

Its semantics are:

```text
something happened
```

Events should generally be immutable.

# 4. Command

A command requests an operation:

```text
MoveArm
SetVelocity
StartMission
ResetController
```

A command is fundamentally different from an event.

```text
Command:
    "do X"

Event:
    "X happened"
```

This distinction prevents a huge class of architectural ambiguities.

# 5. Query

A query requests information:

```text
GetRobotState
GetCapabilities
GetHealth
GetTaskStatus
```

Conceptually:

```text
Query
   ↓
Response
```

Queries should not implicitly mutate state.

# 6. Response

A response is the result of a query or request.

It should carry:

```text
request_id
status
result
error
metadata
```

This enables deterministic correlation.

# 7. Intent

Autonomous systems need another level:

```text
Intent
```

An intent represents:

> what an autonomous component wants to accomplish.

Example:

```text
Intent:
    reach waypoint W
```

It does not yet mean:

```text
motor command
```

The runtime may transform:

```text
Intent
 ↓
Plan
 ↓
Work
 ↓
Commands
```

# 8. Work

Work represents something admitted for execution.

```text
Intent
   ↓
Authorization
   ↓
Work
```

Work has lifecycle semantics:

```text
CREATED
READY
ADMITTED
RUNNING
PAUSED
COMPLETED
FAILED
CANCELLED
```

This connects directly to the Execution Fabric.

# 9. Capability

A capability represents authority.

Example:

```text
Capability:
    control/arm/write
```

Capabilities should be typed and scoped.

```text
Capability
├── subject
├── resource
├── operation
├── constraints
├── validity
└── provenance
```

# 10. Policy

A policy determines whether an operation is permitted.

```text
Request
   +
Context
   ↓
Policy
   ↓
Allow / Deny / Conditional
```

The policy itself should be representable as a versioned object.

# 11. State

State represents authoritative runtime knowledge.

For example:

```text
RobotState
├── pose
├── velocity
├── battery
├── mode
└── safety_state
```

But NROS should distinguish:

```text
observed state
```

from:

```text
authoritative state
```

# 12. State Transition

A transition records:

```text
state_before
   ↓
operation
   ↓
state_after
```

This is more powerful than simply publishing the new state.

It enables:

```text
history
audit
replay
causality
validation
```

# 13. Failure

Failure becomes a typed object:

```text
Failure
├── category
├── severity
├── subject
├── evidence
├── state_context
└── recovery_policy
```

The failure itself can then participate in the protocol.

# 14. Recovery

Recovery should also be represented explicitly:

```text
Recovery
├── failure_id
├── strategy
├── attempt
├── actor
├── state
├── result
└── evidence
```

Thus recovery becomes observable and auditable.

# 15. Evidence

Evidence is the connective tissue:

```text
Evidence
├── event
├── state
├── authority
├── work
├── failure
└── recovery
```

It establishes provenance.

# 16. Universal Envelope

All NROS protocol objects can share a common envelope.

Conceptually:

```text
NrosEnvelope
├── version
├── object_type
├── object_id
├── timestamp
├── source
├── correlation_id
├── causation_id
├── schema_id
└── payload
```

This does not mean every object has identical semantics.

It means the transport layer can reason consistently about identity and provenance.

# 17. Identity

Every significant object needs stable identity.

For example:

```text
message_id
event_id
work_id
decision_id
failure_id
evidence_id
```

Identity enables cross-fabric references.

# 18. Correlation

Example:

```text
work_id = W42
```

could correlate:

```text
W42
├── Intent I7
├── Authorization A12
├── Plan P8
├── Command C71
├── Events E901...E945
└── Evidence bundle B3
```

This gives NROS a native relationship model.

# 19. Causation

Correlation is insufficient.

Suppose:

```text
E1
E2
E3
```

all belong to Work `W42`.

We additionally need:

```text
E2 caused by E1
E3 caused by E2
```

Hence:

```text
causation_id
```

becomes a protocol-level primitive.

# 20. Parent/Child Relationships

Runtime objects may form hierarchies.

Example:

```text
Mission
 ├── Navigation Goal
 │    ├── Planner Work
 │    └── Controller Work
 │
 └── Docking Goal
      └── Controller Work
```

The protocol should support:

```text
parent_id
```

or equivalent structured relationships.

# 21. Schema Identity

Every payload needs a schema:

```text
schema_id
```

For example:

```text
nros.robot.pose
nros.motion.command
nros.work.status
```

Schema identity should be globally meaningful within an NROS deployment.

# 22. Schema Version

Schemas evolve:

```text
nros.robot.pose@1
nros.robot.pose@2
```

Compatibility must be explicit.

Never rely on:

```text
"the receiver probably understands it."
```

# 23. Compatibility

NROS should define compatibility categories:

```text
BACKWARD_COMPATIBLE
FORWARD_COMPATIBLE
WIRE_COMPATIBLE
SEMANTICALLY_COMPATIBLE
INCOMPATIBLE
```

This is especially important for distributed robots where components may upgrade independently.

# 24. Unknown Fields

A robust protocol should permit controlled evolution.

For example:

```text
Pose@2
```

may add:

```text
covariance_source
```

while older consumers ignore unknown fields.

But unknown **semantics** should never silently become trusted semantics.

# 25. Semantic Versioning Is Not Enough

Protocol compatibility is more subtle than:

```text
1.2.3
```

NROS should distinguish:

```text
wire schema
semantic contract
capability contract
runtime protocol
```

Each can evolve independently.

# 26. Type System

The NROS type system should support primitives:

```text
bool
integer
float
string
bytes
timestamp
duration
identifier
```

and structured types:

```text
struct
enum
list
map
optional
union
```

# 27. Robotics Types

Then provide standardized robotics primitives:

```text
Pose
Quaternion
Transform
Twist
Wrench
JointState
Trajectory
Covariance
FrameId
Timestamp
```

These should have precisely defined semantics.

# 28. Units

A dangerous ROS-era problem can be:

```text
velocity = 1.0
```

What does that mean?

NROS types should make units explicit where appropriate:

```text
velocity:
    value = 1.0
    unit = m/s
```

or encode units in the schema contract.

# 29. Coordinate Frames

Robotics semantics require frame identity.

A pose should not merely be:

```text
x
y
z
```

but something conceptually like:

```text
Pose
├── frame
├── position
├── orientation
└── timestamp
```

This allows the State and Temporal fabrics to interpret it correctly.

# 30. Serialization

The protocol should separate:

```text
semantic schema
```

from:

```text
wire encoding
```

Possible encodings include:

```text
binary
JSON
CBOR
MessagePack
custom compact encoding
```

The runtime should not hard-code the semantic model to one serialization format.

# 31. Binary Is the Default for Critical Paths

For high-rate control:

```text
binary encoding
```

is generally preferable because of:

```text
predictable size
lower overhead
faster parsing
less allocation
```

Human-readable formats remain useful for:

```text
debugging
configuration
inspection
bridges
```

# 32. Zero-Copy Potential

The NROS Rust architecture should allow:

```text
receive
 ↓
validate
 ↓
dispatch
```

without unnecessary copying.

This matters for:

```text
camera frames
point clouds
large maps
trajectories
```

# 33. Borrowed vs Owned Data

Rust provides an important opportunity.

Protocol APIs can distinguish:

```text
borrowed view
```

from:

```text
owned message
```

This can significantly reduce allocation pressure in high-rate pipelines.

# 34. Validation

Incoming objects should be validated before becoming trusted runtime state.

Conceptually:

```text
wire data
   ↓
decode
   ↓
schema validation
   ↓
semantic validation
   ↓
policy validation
   ↓
runtime object
```

# 35. Validation Layers

There are at least four distinct validations:

```text
1. Encoding valid?
2. Schema valid?
3. Semantics valid?
4. Authorized for this context?
```

Passing #1 does not imply passing #4.

# 36. Invalid Messages

Invalid protocol objects should become explicit events:

```text
ProtocolViolation
```

with:

```text
reason
source
schema
payload metadata
timestamp
```

rather than causing undefined behavior.

# 37. Error Model

NROS should standardize structured errors.

Conceptually:

```text
NrosError
├── code
├── category
├── message
├── retryability
├── causal_context
└── details
```

Examples:

```text
INVALID_SCHEMA
UNAUTHORIZED
RESOURCE_BUSY
DEADLINE_EXCEEDED
STATE_CONFLICT
CAPABILITY_EXPIRED
DEPENDENCY_FAILED
UNKNOWN_OUTCOME
```

# 38. Error Semantics

The key field is often:

```text
retryability
```

For example:

```text
INVALID_SCHEMA
    → never retry unchanged

TIMEOUT
    → possibly retry

CAPABILITY_EXPIRED
    → refresh/re-authorize

STATE_CONFLICT
    → reconcile first
```

This allows intelligent recovery.

# 39. Command Semantics

Commands should declare execution semantics where applicable:

```text
at_most_once
at_least_once
idempotent
transactional
uncertain_on_timeout
```

This directly connects Protocol to Supervision.

# 40. Deadline Semantics

A command may carry:

```text
deadline
```

rather than merely:

```text
timeout
```

This means:

> the operation is no longer useful after this temporal point.

The Temporal Fabric can then enforce it.

# 41. Priority

Work and commands can carry:

```text
priority
```

but priority must never override safety or authority.

The correct conceptual ordering is:

```text
Safety
   ↓
Authority
   ↓
Validity
   ↓
Deadline
   ↓
Priority
```

not:

```text
priority > everything
```

# 42. Cancellation

Cancellation should be a first-class protocol operation:

```text
Cancel(work_id)
```

with explicit semantics.

Cancellation may result in:

```text
CANCELLED
CANCELLATION_PENDING
CANCELLATION_FAILED
UNKNOWN_OUTCOME
```

This is especially important for physical work.

# 43. Preemption

Cancellation and preemption are different.

```text
Cancellation:
    stop this work

Preemption:
    temporarily suspend this work
    because another work has precedence
```

NROS should represent both explicitly.

# 44. Pause and Resume

Long-running work can support:

```text
pause
resume
```

but only if the work type declares those semantics.

A motor trajectory may be resumable.

A one-shot actuator pulse may not be.

# 45. Lifecycle Protocol

NROS components can use a standardized lifecycle:

```text
CREATED
 ↓
CONFIGURED
 ↓
READY
 ↓
ACTIVE
 ↓
DEGRADED
 ↓
STOPPING
 ↓
STOPPED
```

with failure transitions:

```text
FAILED
RECOVERING
```

This becomes a runtime protocol rather than an application convention.

# 46. Node Evolution

We can now reconsider the traditional ROS node.

Instead of:

```text
Node
```

NROS can define:

```text
RuntimeComponent
```

with:

```text
identity
lifecycle
capabilities
ports
state
health
supervision
work
evidence
```

The process becomes an implementation detail.

# 47. Ports

A component can expose typed ports:

```text
Input Port
Output Port
Command Port
Event Port
Query Port
```

This is more expressive than treating every interaction as simply a topic.

# 48. Example Component

Conceptually:

```text
LocalizationComponent

Inputs:
    SensorFrame

Outputs:
    PoseEstimate

Queries:
    GetLocalizationState

Commands:
    ResetLocalization

Events:
    LocalizationLost
    LocalizationRecovered

Capabilities:
    localization.read
    localization.reset
```

Now the interface itself describes the component's semantics.

# 49. ROS Compatibility

This does **not** require abandoning ROS.

NROS can provide a compatibility boundary:

```text
ROS 1 / ROS 2
      │
      ▼
NROS Bridge
      │
      ▼
NROS Protocol
```

ROS messages can therefore remain useful at the edge.

# 50. Bridge Principle

The bridge should translate semantics where possible:

```text
ROS Topic
   ↓
NROS Message/Event
```

rather than pretending:

```text
everything is just bytes
```

For example:

```text
ROS service call
```

can become:

```text
NROS Request + Response
```

while preserving correlation and provenance.

# 51. Actions

ROS actions map naturally into:

```text
Intent
 ↓
Work
 ↓
Progress Events
 ↓
Completion
```

This gives NROS a more general model than a dedicated action mechanism.

# 52. Topics

ROS topics map primarily to:

```text
streaming messages
```

But NROS can additionally identify:

```text
event stream
state stream
telemetry stream
```

so consumers understand the semantic contract.

# 53. Services

ROS services map to:

```text
Query
Request
Command
Response
```

depending on semantics.

This avoids treating all RPC-like interactions as equivalent.

# 54. Parameter Server

The ROS parameter server can evolve into:

```text
Configuration State
```

with:

```text
version
authority
provenance
validation
change events
```

A parameter change then becomes:

```text
ConfigurationChanged
```

rather than an invisible mutation.

# 55. Protocol as a Contract

The ultimate objective is:

```text
Component A
     │
     │ typed contract
     ▼
NROS Protocol
     │
     ▼
Component B
```

The protocol specifies not merely:

```text
bytes
```

but:

```text
meaning
lifecycle
authority
timing
failure semantics
compatibility
```

# 56. The NROS Semantic Stack

We can now express the complete progression:

```text
ROS
│
├── Nodes
├── Topics
├── Services
├── Actions
└── Messages

                ↓

NROS
│
├── Components
├── Streams
├── Requests
├── Commands
├── Events
├── Queries
├── Intents
├── Work
├── State
├── Capabilities
├── Policies
├── Failures
├── Recovery
└── Evidence
```

This is the fundamental conceptual migration.

# 57. NROS Eight-Fabric Architecture

We now have:

```text
┌──────────────────────────────────────┐
│       Protocol & Type Fabric         │
├──────────────────────────────────────┤
│       Observability & Evidence       │
├──────────────────────────────────────┤
│       Capability & Authority         │
├──────────────────────────────────────┤
│               State                  │
├──────────────────────────────────────┤
│            Supervision               │
├──────────────────────────────────────┤
│              Temporal                │
├──────────────────────────────────────┤
│              Execution               │
├──────────────────────────────────────┤
│           Communication              │
└──────────────────────────────────────┘
```

This is beginning to look less like a conventional middleware library and more like a **runtime architecture**.

# 58. But One Major Problem Remains

A typed protocol tells us:

```text
what an object means
```

but not:

```text
where it is allowed to exist
```

A robot may have:

```text
safety-critical MCU
real-time controller
Linux computer
GPU
edge server
cloud service
mobile device
simulator
```

These environments have radically different guarantees.

NROS therefore needs to model **execution domains**.

# Part XLVII — NROS Domain & Deployment Fabric

The next layer will answer:

```text
Where does a component execute?

What guarantees does that domain provide?

What may cross domain boundaries?

Which operations require deterministic execution?

How does NROS bridge:

MCU ↔ RTOS ↔ Linux ↔ GPU ↔ Edge ↔ Cloud?
```

This is where NROS can address one of ROS's deepest architectural realities:

> **Robotic systems are heterogeneous distributed computers, not merely collections of processes.**

And the NROS runtime must make that heterogeneity explicit rather than hiding it behind the abstraction of a node.

# NROS — Part XLVII: Domain & Deployment Fabric

The previous layer defined **what moves through NROS**.

Now we define **where those things execute**.

This is critical because a robot is rarely one homogeneous computer.

A realistic system may look like:

```text
┌───────────────────────────────────────────────────────┐
│                    Robot System                       │
│                                                       │
│  MCU ── RTOS ── Linux ── GPU ── Edge ── Cloud       │
│   │       │        │       │       │        │        │
│   │       │        │       │       │        │        │
│  motor  control   ROS    vision   fleet    storage   │
└───────────────────────────────────────────────────────┘
```

NROS should make these boundaries **explicit, typed, observable, and enforceable**.

# 1. From Nodes to Execution Domains

ROS tends to present:

```text
Node
```

as the primary execution abstraction.

NROS should instead distinguish:

```text
Component
```

from:

```text
Execution Domain
```

A component describes **what** something is.

A domain describes **where and under what guarantees** it runs.

# 2. Execution Domain

Conceptually:

```text
ExecutionDomain
├── identity
├── host
├── runtime
├── scheduler
├── clock
├── memory model
├── isolation
├── capabilities
├── timing guarantees
└── connectivity
```

Example:

```text
control-mcu
```

might provide:

```text
hard deadlines
static memory
isolated execution
hardware access
```

while:

```text
cloud-agent
```

might provide:

```text
elastic compute
high latency
unreliable connectivity
large memory
```

# 3. Domain Classes

NROS could classify domains:

```text
BARE_METAL
RTOS
REALTIME_OS
GENERAL_OS
CONTAINER
VM
GPU
EDGE
CLOUD
SIMULATION
```

The classification is descriptive, not necessarily tied to one operating system.

# 4. Why This Matters

Consider:

```text
MotorController
```

It may require:

```text
deadline < 1 ms
jitter < 50 µs
```

Placing it in:

```text
cloud
```

would violate its execution contract.

NROS should be able to express this **before deployment**.

# 5. Execution Contract

A component can declare:

```text
ExecutionRequirements
├── latency
├── jitter
├── deadline
├── throughput
├── memory
├── CPU
├── accelerator
├── persistence
└── availability
```

The deployment system can then determine whether a domain is suitable.

# 6. Placement

Instead of:

```text
run component X
```

NROS can reason:

```text
Component X
    requires:
        realtime
        100µs deadline
        motor capability

Domain A
    provides:
        realtime
        50µs jitter
        motor capability

→ placement valid
```

# 7. Placement Constraints

A component may specify:

```text
requires:
    gpu

requires:
    realtime

requires:
    device("motor0")
```

or:

```text
forbids:
    cloud
```

This becomes part of deployment semantics.

# 8. Hard vs Soft Constraints

Not every requirement has equal importance.

```text
HARD:
    must be satisfied

SOFT:
    preferred
```

Example:

```text
Vision:
    GPU = preferred
    8GB RAM = minimum
```

while:

```text
Motor control:
    realtime = mandatory
```

# 9. Domain Capabilities

Domains themselves advertise capabilities:

```text
DomainCapabilities
├── realtime
├── gpu
├── network
├── persistent_storage
├── hardware_access
├── secure_execution
└── high_precision_clock
```

This allows matching:

```text
requirements ↔ capabilities
```

# 10. Deployment Graph

A complete robot can then be represented as:

```text
Deployment
│
├── Domain: MCU
│   ├── SafetyController
│   └── MotorGateway
│
├── Domain: RT Linux
│   ├── MotionController
│   └── StateEstimator
│
├── Domain: Linux
│   ├── Planner
│   └── MissionAgent
│
├── Domain: GPU
│   └── Vision
│
└── Domain: Cloud
    └── FleetAnalytics
```

# 11. Domain Boundaries

Every domain boundary is a potential source of:

```text
latency
failure
serialization
security risk
clock skew
resource contention
```

Therefore NROS should model boundaries explicitly.

# 12. Local Execution

Within a domain:

```text
Component A
      │
      ▼
Component B
```

communication may use:

```text
shared memory
channels
queues
direct calls
lock-free structures
```

depending on requirements.

# 13. Remote Execution

Across domains:

```text
Domain A
    │
 transport
    │
Domain B
```

requires:

```text
serialization
authentication
flow control
failure handling
clock metadata
```

# 14. Transport Selection

NROS should separate:

```text
semantic communication
```

from:

```text
transport mechanism
```

Possible transports:

```text
shared memory
Unix sockets
TCP
UDP
QUIC
CAN
Ethernet
serial
DDS bridge
ROS bridge
```

The application should not necessarily depend directly on one transport.

# 15. Transport Policy

A message may declare:

```text
transport requirements:
    low_latency
    reliable
    ordered
```

The runtime can select an appropriate transport.

For example:

```text
high-rate sensor:
    shared memory

control command:
    reliable low-latency transport

cloud telemetry:
    buffered reliable transport
```

# 16. Data Plane vs Control Plane

This distinction becomes important.

### Data plane

Carries:

```text
sensor data
images
point clouds
trajectories
commands
```

### Control plane

Carries:

```text
registration
capability negotiation
health
configuration
lifecycle
deployment
```

NROS should keep these concepts separate.

# 17. Management Plane

A third plane can manage the runtime:

```text
Management Plane
├── deployment
├── upgrades
├── diagnostics
├── observability
├── recovery
└── policy
```

Therefore:

```text
NROS
├── Data Plane
├── Control Plane
└── Management Plane
```

# 18. Domain Registration

When a runtime domain starts, it registers:

```text
DomainDescriptor
```

containing:

```text
identity
version
capabilities
resources
clock
network
security posture
```

This becomes discoverable.

# 19. Resource Model

A domain should advertise resources:

```text
CPU
Memory
GPU
Storage
Network
Devices
```

Example:

```text
CPU:
    4 cores

GPU:
    CUDA

Memory:
    8 GB

Devices:
    /dev/camera0
    /dev/motor0
```

# 20. Resource Reservations

Critical work can reserve resources.

For example:

```text
MotionController
    CPU:
        core 2
    deadline:
        1ms
```

This connects Domain to the Execution and Temporal fabrics.

# 21. Resource Admission

Before admitting work:

```text
Work
 ↓
Requirements
 ↓
Resource availability
 ↓
Admission decision
```

Possible result:

```text
ADMITTED
```

or:

```text
RESOURCE_UNAVAILABLE
```

# 22. Real-Time Domain

A real-time domain should advertise stronger guarantees:

```text
deterministic scheduler
bounded interrupt latency
priority control
memory locking
bounded allocation
clock guarantees
```

NROS should not simply label a system "real-time" because it happens to run on Linux.

The guarantee must be explicit.

# 23. Real-Time Contract

Example:

```text
ControlLoop:
    period = 1ms
    deadline = 1ms
    jitter <= 20µs
```

The runtime can observe whether the contract is actually being met.

This links:

```text
Domain
+
Temporal Fabric
+
Observability
```

# 24. Soft Real-Time

Some workloads tolerate occasional misses:

```text
Vision:
    30 FPS
    occasional frame loss acceptable
```

This differs from:

```text
Motor control:
    deadline miss may be unsafe
```

NROS should distinguish:

```text
hard
firm
soft
best-effort
```

timing contracts.

# 25. Safety Domains

A particularly important domain is:

```text
Safety Domain
```

It can contain components that enforce:

```text
speed limits
collision constraints
emergency stop
actuator interlocks
```

Other domains must not be able to bypass it.

# 26. Safety Boundary

A command path could become:

```text
Agent
 ↓
Planner
 ↓
Controller
 ↓
Safety Gate
 ↓
Actuator
```

The safety gate is a domain boundary.

Even if the agent is compromised:

```text
Agent → unsafe command
```

the safety domain can reject it.

# 27. Hardware Domains

Hardware itself can be modeled as a domain or resource provider:

```text
Motor0
Camera0
Lidar0
IMU0
```

Each can expose:

```text
identity
capabilities
state
health
authority
```

This allows the same runtime model to encompass physical resources.

# 28. Device Ownership

A device may have:

```text
owner = controller-A
```

Only the owner can exercise specific capabilities.

This connects:

```text
Device
+
Capability
+
Authority
```

into a coherent model.

# 29. Device Lease

For exclusive resources:

```text
DeviceLease
```

can define:

```text
owner
issued_at
expires_at
scope
renewal
```

If the owner fails:

```text
lease expires
 ↓
authority revoked
 ↓
device enters safe state
```

# 30. Domain Failure

Domains themselves can fail.

For example:

```text
GPU Domain
    ✗
```

Then:

```text
Vision
```

may become:

```text
DEGRADED
```

while:

```text
Motion Control
```

continues.

The Supervision Fabric operates at domain level as well as component level.

# 31. Domain Migration

If supported:

```text
Vision
Domain A
   ↓
Domain B
```

NROS can migrate a component.

But migration requires:

```text
state transfer
authority transfer
resource acquisition
identity preservation
```

# 32. Stateful Migration

A running component cannot simply be copied.

NROS needs:

```text
checkpoint
 ↓
freeze
 ↓
transfer state
 ↓
restore
 ↓
validate
 ↓
transfer authority
```

This is closely related to the recovery model.

# 33. Stateless Migration

Stateless components are easier:

```text
destroy A
start B
```

because state is externalized.

This reinforces a valuable architectural principle:

> **Important state should not be trapped inside an opaque process.**

# 34. Edge/Cloud Boundary

Cloud services should be treated as inherently different domains.

```text
Robot
 ↓
Edge
 ↓
Cloud
```

The cloud may have:

```text
high capacity
high latency
intermittent connectivity
```

Therefore cloud operations should normally not hold direct hard-real-time authority.

# 35. Cloud Authority

For safety-critical systems:

```text
Cloud
    ✗ direct motor authority
```

Instead:

```text
Cloud
 ↓
high-level intent
 ↓
Robot policy
 ↓
local authorization
 ↓
local execution
```

This preserves local safety.

# 36. Edge Authority

An edge computer may be allowed:

```text
mission planning
vision
fleet coordination
```

but not:

```text
bypass local safety gate
```

Authority is therefore domain-sensitive.

# 37. Offline Operation

A robot should define:

```text
OFFLINE_POLICY
```

for each dependency.

Example:

```text
Cloud unavailable:
    mission planner → continue cached mission
    telemetry → buffer
    remote commands → reject
```

This is a deployment-level version of graceful degradation.

# 38. Network Zones

A deployment may contain:

```text
Safety Zone
Control Zone
Application Zone
External Zone
Cloud Zone
```

Communication policies can then specify permitted paths:

```text
Safety → Actuator
Application → Safety
Cloud → Application
Cloud ✗ Actuator
```

# 39. Zero-Trust Domain Communication

A domain should not automatically trust another domain merely because they are on the same robot.

Instead:

```text
identity
+
authentication
+
authorization
+
policy
```

should govern cross-domain interaction.

# 40. Domain Identity

Each domain receives an identity:

```text
domain_id
```

and potentially a cryptographic identity:

```text
domain_identity
```

Components inherit or separately establish identity according to deployment policy.

# 41. Attestation

High-assurance systems may require:

```text
runtime attestation
```

before granting authority.

Conceptually:

```text
Domain
 ↓
attestation
 ↓
verified software/configuration
 ↓
capability grant
```

This is especially relevant for safety-critical or remotely managed robots.

# 42. Deployment Manifest

NROS deployments can be declarative.

Conceptually:

```text
deployment:
    domains:
        - control
        - perception
        - cloud

    components:
        controller:
            domain: control

        vision:
            domain: perception

        analytics:
            domain: cloud
```

The exact file format can evolve later.

The semantic model comes first.

# 43. Deployment Validation

Before startup:

```text
deployment
 ↓
validate
 ↓
requirements
 ↓
domain capabilities
 ↓
authority policies
 ↓
resource constraints
```

Invalid deployments should fail before dangerous components become active.

# 44. Deployment as a Graph

The deployment itself becomes evidence:

```text
Deployment D17
├── Domain D1
│   ├── Component C1
│   └── Device M1
├── Domain D2
│   └── Component C2
└── Domain D3
    └── Component C3
```

The Observability Fabric can then answer:

> Where was component C1 running when failure F occurred?

# 45. Versioned Deployment

Every runtime instance should know:

```text
deployment_version
```

because the same software can behave differently under different topology and policies.

Example:

```text
Robot A:
    deployment v12

Robot B:
    deployment v13
```

Evidence can therefore distinguish them.

# 46. Rolling Upgrade

NROS can eventually support:

```text
old component
      ↓
new component
```

with controlled transition.

But the protocol requires:

```text
compatibility
state transfer
authority transfer
health validation
rollback
```

before treating the upgrade as complete.

# 47. Blue/Green Deployment

For larger systems:

```text
BLUE
    current

GREEN
    candidate
```

The candidate can be validated before receiving authority.

Then:

```text
BLUE
 ↓
authority transfer
 ↓
GREEN
```

This is another application of the Capability and Supervision fabrics.

# 48. Simulation Domain

Simulation should be a first-class domain:

```text
SIM_DOMAIN
```

A simulated robot can expose the same protocol objects:

```text
state
commands
events
work
evidence
```

This enables software to operate against:

```text
real hardware
```

or:

```text
simulation
```

through the same semantic interfaces.

# 49. Hardware-in-the-Loop

A particularly valuable topology:

```text
Simulator
   │
   ├── simulated environment
   │
   └── real controller
            │
          hardware
```

NROS domains make such arrangements explicit.

# 50. Digital Twin

The same model can support:

```text
Physical Robot
      ↕
Digital Twin
```

Both expose compatible state and evidence models.

This enables:

```text
replay
simulation
testing
diagnostics
predictive maintenance
```

# 51. Domain-Aware Scheduling

The scheduler can now reason about:

```text
what
when
where
with which resources
under which guarantees
```

So scheduling evolves from:

```text
queue → worker
```

into:

```text
Work
 ↓
Requirements
 ↓
Authority
 ↓
Temporal constraints
 ↓
Domain selection
 ↓
Resource admission
 ↓
Execution
```

This is a major architectural step.

# 52. NROS Nine-Fabric Architecture

We now have:

```text
┌──────────────────────────────────────┐
│        Domain & Deployment           │
├──────────────────────────────────────┤
│       Protocol & Type                │
├──────────────────────────────────────┤
│       Observability & Evidence       │
├──────────────────────────────────────┤
│       Capability & Authority         │
├──────────────────────────────────────┤
│               State                  │
├──────────────────────────────────────┤
│            Supervision               │
├──────────────────────────────────────┤
│              Temporal                │
├──────────────────────────────────────┤
│              Execution               │
├──────────────────────────────────────┤
│           Communication              │
└──────────────────────────────────────┘
```

At this point, the architecture has crossed an important boundary.

NROS is no longer simply:

```text
ROS + Rust
```

or:

```text
ROS + faster middleware
```

It is becoming a **distributed, typed, authority-aware, stateful execution environment for robotics**.

# 53. The Next Missing Primitive: Resources

We have discussed resources throughout this layer:

```text
CPU
memory
GPU
devices
network
storage
```

but we have not yet made them a first-class runtime abstraction.

That is a serious gap.

A robot does not merely execute software.

It controls and competes for **resources**.

Examples:

```text
motor
camera
battery
GPU
CPU core
memory
network bandwidth
power budget
physical workspace
charging station
```

Therefore the next architectural layer should introduce:

# Part XLVIII — NROS Resource & Allocation Fabric

The central question becomes:

> **How does NROS represent, allocate, lease, reserve, constrain, and safely share every resource required by autonomous execution?**

This will connect:

```text
Resource
   ↓
Capability
   ↓
Authority
   ↓
Work
   ↓
Scheduler
   ↓
Execution
   ↓
State
   ↓
Evidence
```

and ultimately give NROS a complete **resource-aware execution model** rather than merely a process-aware one.

# NROS — Part XLVIII: Resource & Allocation Fabric

The Domain & Deployment Fabric answered:

> **Where can a component execute?**

The Resource & Allocation Fabric answers:

> **What can it consume, control, reserve, share, or own while executing?**

This distinction is fundamental.

A process is not the unit of reality in a robot.

The robot contains **resources**:

```text
CPU
Memory
GPU
Sensors
Actuators
Battery
Network
Storage
Physical workspace
Communication channels
Energy budget
Time
```

NROS should represent these explicitly.

# 1. Resource as a First-Class Object

Define:

```text
Resource
├── resource_id
├── kind
├── owner
├── state
├── capacity
├── availability
├── constraints
├── capabilities
└── provenance
```

Examples:

```text
resource: motor.left
resource: camera.front
resource: gpu.0
resource: cpu.core.2
resource: battery.main
```

The important change is:

```text
resource ≠ file
resource ≠ process
resource ≠ topic
```

It is an independently managed runtime object.

# 2. Resource Taxonomy

NROS should distinguish several resource classes.

```text
Resource
│
├── Compute
│   ├── CPU
│   ├── GPU
│   ├── NPU
│   └── accelerator
│
├── Memory
│   ├── RAM
│   ├── shared memory
│   └── persistent memory
│
├── Device
│   ├── sensor
│   └── actuator
│
├── Network
│   ├── interface
│   ├── bandwidth
│   └── channel
│
├── Storage
│   ├── filesystem
│   └── database
│
├── Energy
│   ├── battery
│   └── power budget
│
└── Physical
    ├── workspace
    ├── manipulator
    └── docking station
```

# 3. Resource Capacity

A resource is not necessarily binary.

For example:

```text
CPU
capacity = 4 cores
```

or:

```text
Network
capacity = 100 Mbit/s
```

or:

```text
Battery
capacity = 120 Wh
```

Therefore:

```text
capacity
```

must be a first-class concept.

# 4. Allocation

Allocation answers:

> Who currently has the right to consume this resource?

Example:

```text
GPU0
   ↓
VisionService
```

The allocation can be represented as:

```text
Allocation
├── resource_id
├── subject_id
├── quantity
├── scope
├── constraints
├── issued_at
└── expires_at
```

# 5. Allocation ≠ Authority

This distinction is extremely important.

A component may have:

```text
authority to use a motor
```

without currently having:

```text
exclusive allocation of the motor
```

Conversely, a resource may be allocated but still constrained by safety policy.

Therefore:

```text
Capability
    ≠
Allocation
```

They cooperate but represent different concepts.

# 6. Capability + Allocation

A complete operation may require:

```text
Capability
+
Allocation
+
Policy
```

Example:

```text
MotionController
    capability:
        motor.write

    allocation:
        motor.left

    policy:
        max_velocity = 1.5 m/s
```

Only then can execution proceed.

# 7. Resource Ownership

Resources may have owners:

```text
owner = safety-controller
```

or:

```text
owner = robot-runtime
```

Ownership does not necessarily mean exclusive physical use.

It may instead mean:

> the entity responsible for lifecycle and policy of the resource.

# 8. Exclusive Resources

Some resources cannot safely be shared.

Example:

```text
EmergencyStopRelay
```

or:

```text
MotorController
```

NROS can mark:

```text
sharing = EXCLUSIVE
```

Then:

```text
A owns resource
```

means:

```text
B cannot simultaneously acquire it.
```

# 9. Shared Resources

Other resources can safely be shared.

Example:

```text
Camera
```

may support:

```text
VisionA → frames
VisionB → frames
Recorder → frames
```

The resource policy might specify:

```text
sharing = SHARED
```

# 10. Partitionable Resources

Some resources can be divided.

For example:

```text
GPU
```

could expose:

```text
GPU capacity = 100 units
```

with:

```text
Vision = 60
Planner = 20
Analytics = 20
```

This requires quantitative allocation.

# 11. Consumable Resources

Some resources disappear when consumed.

Examples:

```text
battery energy
fuel
storage capacity
network quota
```

These should be modeled as:

```text
ConsumableResource
```

rather than ordinary shared resources.

# 12. Replenishable Resources

A consumable resource can be replenished:

```text
battery
```

through:

```text
charging
```

So resource state can evolve:

```text
80%
 ↓
50%
 ↓
20%
 ↓
charging
 ↓
70%
```

State changes should generate protocol events.

# 13. Resource State

A resource should have explicit lifecycle/state.

Example:

```text
Motor
├── AVAILABLE
├── ALLOCATED
├── ACTIVE
├── DEGRADED
├── FAULTED
├── SAFE
└── OFFLINE
```

This state is distinct from component state.

# 14. Resource Health

Resource state should include health information:

```text
Health
├── status
├── confidence
├── diagnostics
├── last_seen
└── failure_count
```

Example:

```text
Camera:
    state = ACTIVE
    health = DEGRADED
    reason = thermal
```

# 15. Resource Discovery

A runtime domain should be able to discover:

```text
available resources
```

Example:

```text
discover()
```

returns:

```text
GPU0
Camera0
Camera1
Motor0
Motor1
IMU0
```

with their capabilities and states.

# 16. Resource Registration

Physical devices can register into NROS:

```text
Device
   ↓
Resource Registry
   ↓
Runtime
```

This avoids scattering hardware identity across arbitrary nodes.

# 17. Resource Registry

The registry becomes:

```text
ResourceRegistry
├── lookup
├── discovery
├── allocation
├── reservation
├── leases
├── health
└── events
```

This is a major system primitive.

# 18. Reservation

Allocation answers:

> Who has it now?

Reservation answers:

> Who is promised access later?

Example:

```text
Mission A
    reserves:
        charging station
        18:00–18:30
```

Another mission should then see:

```text
resource unavailable during reservation window
```

# 19. Temporal Resource Reservation

Reservations can therefore include:

```text
start
end
```

and optionally:

```text
deadline
priority
owner
conditions
```

This connects Resource directly to the Temporal Fabric.

# 20. Resource Lease

A lease is temporary authority to hold a resource.

```text
Lease
├── lease_id
├── resource
├── holder
├── issued_at
├── expires_at
└── renewal_policy
```

This is safer than indefinite ownership.

# 21. Lease Expiration

Suppose:

```text
Controller A
```

crashes.

Without a lease:

```text
motor remains locked
```

With a lease:

```text
Controller A crashes
        ↓
lease expires
        ↓
allocation revoked
        ↓
safe-state policy
```

This is essential for autonomous systems.

# 22. Fencing

Lease expiration alone may not be sufficient.

A crashed component could still send commands.

Therefore NROS may require:

```text
fencing token
```

Example:

```text
lease token = 42
```

Only commands carrying the current token are accepted.

After takeover:

```text
lease token = 43
```

Commands from token `42` become invalid.

This prevents stale controllers from continuing to act.

# 23. The Stale Controller Problem

Without fencing:

```text
Controller A
     ↓
Motor
```

A fails.

```text
Controller B
     ↓
Motor
```

takes over.

But A recovers and sends:

```text
MOVE
```

Now:

```text
A → motor
B → motor
```

could conflict.

With fencing:

```text
A token = 10
B token = 11
```

the motor accepts only:

```text
token = 11
```

# 24. Resource Arbitration

Multiple workloads may request the same resource:

```text
Work A ─┐
Work B ─┼──> Motor
Work C ─┘
```

NROS needs an arbiter.

Conceptually:

```text
ResourceArbiter
├── requests
├── policy
├── priority
├── safety
├── fairness
└── decision
```

# 25. Arbitration Order

The decision should not simply be:

```text
highest priority wins
```

Instead:

```text
Safety
 ↓
Authority
 ↓
Validity
 ↓
Resource constraints
 ↓
Deadline
 ↓
Priority
 ↓
Fairness
```

This prevents priority from becoming an unsafe escape hatch.

# 26. Priority Inversion

Real-time systems can encounter:

```text
high-priority task
       ↓
needs resource
       ↓
low-priority task holds resource
```

NROS should support established mitigation mechanisms such as:

```text
priority inheritance
priority ceiling
resource partitioning
```

where appropriate.

# 27. Resource Groups

Some operations require multiple resources simultaneously.

Example:

```text
PickObject
```

requires:

```text
arm
gripper
camera
workspace
```

This is a:

```text
ResourceSet
```

rather than a single resource.

# 28. Atomic Acquisition

The runtime should avoid:

```text
acquire arm
 ↓
acquire gripper
 ↓
gripper unavailable
```

leaving the arm locked indefinitely.

Instead:

```text
request:
    {arm, gripper, workspace}
```

and either:

```text
ALL_GRANTED
```

or:

```text
NONE_GRANTED
```

where atomicity is required.

# 29. Deadlock

Multiple resource acquisition creates classic deadlocks:

```text
Work A:
    holds A
    waits for B

Work B:
    holds B
    waits for A
```

NROS should provide mechanisms for:

```text
ordering
timeouts
deadlock detection
rollback
lease expiration
```

# 30. Resource Dependencies

Resources can depend on one another.

Example:

```text
Camera
   ↓
GPU
   ↓
CPU
   ↓
Power
```

If power becomes unavailable:

```text
GPU
 ↓
Vision
```

may become unavailable as well.

Dependency graphs make this explicit.

# 31. Resource Graph

The resource graph can look like:

```text
Battery
 ├── CPU
 │    └── Vision
 ├── GPU
 │    └── Perception
 └── Motor
      └── Navigation
```

This enables impact analysis:

> If Battery capacity drops below X, what work becomes infeasible?

# 32. Resource Contracts

A workload can declare:

```text
ResourceRequirements
├── required
├── preferred
├── minimum
├── maximum
└── forbidden
```

Example:

```text
Vision:
    GPU:
        minimum = 30%
        preferred = 60%
```

# 33. Elastic Work

Not every workload requires fixed resources.

For example:

```text
analytics
```

can operate with:

```text
CPU:
    1–4 cores
```

The scheduler can dynamically scale it.

This allows:

```text
resource elasticity
```

without changing application semantics.

# 34. Admission Control

Before work enters execution:

```text
Work
 ↓
Resource requirements
 ↓
Current allocations
 ↓
Policy
 ↓
Admission
```

Possible results:

```text
ADMIT
WAIT
DEGRADE
REJECT
```

# 35. Graceful Degradation

Suppose a vision workload requests:

```text
60 FPS
```

but resources only support:

```text
20 FPS
```

Instead of immediate failure:

```text
60 → 30 → 20 FPS
```

could be negotiated if the workload declares degradability.

# 36. Resource Quality

Resources can have quality levels.

For example:

```text
Camera
├── resolution
├── FPS
├── exposure
└── latency
```

A workload might request:

```text
minimum:
    720p / 20 FPS
preferred:
    1080p / 30 FPS
```

The allocator chooses a valid operating point.

# 37. Resource Profiles

A resource can expose profiles:

```text
CameraProfile
├── LOW_POWER
├── BALANCED
└── HIGH_QUALITY
```

Changing profile becomes an explicit state transition.

# 38. Power as a Resource

Robotics makes energy particularly important.

NROS should model:

```text
PowerBudget
```

with:

```text
available_energy
current_draw
predicted_draw
reserved_energy
minimum_safe_energy
```

# 39. Energy-Aware Scheduling

A mission can require:

```text
estimated_energy = 15 Wh
```

The scheduler checks:

```text
battery
+
future workload
+
safety reserve
```

before admitting it.

This transforms:

```text
battery monitoring
```

into:

```text
resource-aware planning
```

# 40. Physical Workspace

A less obvious resource is:

```text
physical space
```

Two manipulators cannot necessarily occupy the same volume.

Therefore:

```text
workspace
```

can become a reservable resource.

Example:

```text
Arm A
    reserves:
        workspace region R1
```

This connects resource allocation with physical safety.

# 41. Resource Constraints and Safety

A safety policy may say:

```text
Motor velocity
    ≤ 1.0 m/s
```

Resource allocation must not be allowed to bypass this.

Therefore:

```text
Allocation
   ↓
Policy
   ↓
Safe operating envelope
```

must be enforced.

# 42. Resource Observability

Every important resource operation should emit events:

```text
ResourceDiscovered
ResourceAllocated
ResourceReleased
ResourceReserved
LeaseIssued
LeaseExpired
ResourceDegraded
ResourceFaulted
```

These become part of the Evidence Fabric.

# 43. Resource Provenance

For a physical action we should eventually be able to answer:

```text
Which component?
Which work item?
Which capability?
Which allocation?
Which resource?
Which policy?
Which deployment?
Which domain?
```

This is the beginning of complete causal traceability.

# 44. Resource Audit Chain

Example:

```text
Command C91
    ↓
Work W17
    ↓
Capability Cap8
    ↓
Allocation A12
    ↓
Resource Motor0
    ↓
Domain Control
    ↓
Deployment D42
```

The Observability system can reconstruct the chain.

# 45. Resource Failure

Suppose:

```text
Motor0
```

fails.

The Resource Fabric emits:

```text
ResourceFaulted(Motor0)
```

The scheduler identifies affected work:

```text
W17
W19
W22
```

Supervision then determines:

```text
restart
migrate
degrade
cancel
safe-stop
```

This is a concrete example of the fabrics working together.

# 46. Resource Recovery

A recovered resource should not automatically become available.

Instead:

```text
FAULTED
 ↓
RECOVERING
 ↓
SELF_TEST
 ↓
VALIDATED
 ↓
AVAILABLE
```

This prevents unsafe resurrection.

# 47. Resource Quarantine

A suspicious resource can be quarantined:

```text
QUARANTINED
```

Meaning:

```text
discovery = yes
use = no
diagnostics = yes
```

This is useful for fault isolation.

# 48. Resource Revocation

An allocation may be revoked because:

```text
policy changed
safety violation
lease expired
resource fault
higher authority
emergency stop
```

Revocation should produce a structured event.

# 49. Emergency Resource Revocation

Safety systems may require:

```text
EmergencyStop
```

to immediately invalidate actuator allocations.

Conceptually:

```text
EmergencyStop
 ↓
revoke actuator authority
 ↓
fence active controllers
 ↓
safe-state transition
```

This is significantly stronger than merely publishing:

```text
stop = true
```

on a topic.

# 50. Resource Fabric API

At the conceptual level:

```text
ResourceRegistry
    discover()
    inspect()
    allocate()
    reserve()
    release()
    renew()
    revoke()
    quarantine()

ResourceArbiter
    request()
    evaluate()
    admit()
    deny()

ResourceLease
    renew()
    expire()
    fence()
```

The exact Rust API should come later.

First we freeze semantics.

# 51. Rust Representation

Eventually this could map to strongly typed structures:

```text
ResourceId
AllocationId
ReservationId
LeaseId
CapabilityId
DomainId
WorkId
```

rather than generic strings everywhere.

This is especially valuable for the NROS Rust core.

# 52. No Stringly-Typed Runtime

Avoid:

```text
resource = "motor.left"
```

as the only identity mechanism.

Prefer:

```text
ResourceId
```

with explicit resolution through the registry.

Names can remain human-readable aliases.

# 53. Deterministic Allocation

For safety-critical systems, allocation decisions should ideally be reproducible.

Given:

```text
same state
same requests
same policy
same ordering
```

the result should be:

```text
same decision
```

where the runtime contract requires determinism.

# 54. Allocation Decision Record

Every nontrivial allocation decision can produce:

```text
AllocationDecision
├── request
├── candidates
├── selected
├── policy
├── reason
├── timestamp
└── evidence_id
```

This is extremely useful for debugging autonomous behavior.

# 55. The Resource Lifecycle

We can now define:

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
ACTIVE
    ↓
RELEASED
    ↓
AVAILABLE
```

Failure path:

```text
ACTIVE
 ↓
FAULTED
 ↓
QUARANTINED
 ↓
RECOVERING
 ↓
VALIDATED
 ↓
AVAILABLE
```

# 56. The Complete Work Path

We can now combine almost every NROS fabric:

```text
Intent
  ↓
Work
  ↓
Authorization
  ↓
Temporal validation
  ↓
Resource admission
  ↓
Domain placement
  ↓
Allocation
  ↓
Execution
  ↓
State transition
  ↓
Event
  ↓
Evidence
```

And on failure:

```text
Failure
  ↓
Supervision
  ↓
Recovery
  ↓
Resource reconciliation
  ↓
State validation
  ↓
Evidence
```

This is becoming the central NROS execution model.

# 57. NROS Ten-Fabric Architecture

The architecture now becomes:

```text
┌────────────────────────────────────────┐
│       Domain & Deployment              │
├────────────────────────────────────────┤
│       Resource & Allocation            │
├────────────────────────────────────────┤
│       Protocol & Type                  │
├────────────────────────────────────────┤
│       Observability & Evidence         │
├────────────────────────────────────────┤
│       Capability & Authority           │
├────────────────────────────────────────┤
│       State                            │
├────────────────────────────────────────┤
│       Supervision                      │
├────────────────────────────────────────┤
│       Temporal                         │
├────────────────────────────────────────┤
│       Execution                        │
├────────────────────────────────────────┤
│       Communication                    │
└────────────────────────────────────────┘
```

This gives us something substantially more expressive than the classic ROS computation graph.

# 58. ROS → NROS Transformation

The progression can now be summarized:

```text
ROS
│
├── Node
├── Topic
├── Service
├── Action
├── Parameter
└── Message

                ↓

NROS
│
├── Component
├── Domain
├── Resource
├── Allocation
├── Capability
├── Policy
├── Stream
├── Event
├── Query
├── Command
├── Intent
├── Work
├── State
├── Transition
├── Failure
├── Recovery
├── Evidence
└── Deployment
```

The crucial transformation is:

```text
communication-centric
        ↓
execution-centric
        ↓
resource-aware
        ↓
authority-aware
        ↓
state-aware
        ↓
evidence-aware
```

# 59. The Next Architectural Question

We can now control:

```text
what
where
when
with what
under whose authority
```

But there is still a missing dimension:

> **How does NROS know what is true?**

A robot continuously receives contradictory, delayed, partial, noisy, and independently generated observations.

Examples:

```text
Camera says:
    obstacle at X

Lidar says:
    obstacle at X + 20cm

Planner believes:
    path is clear

Controller reports:
    motor stalled
```

NROS therefore needs a first-class mechanism for **knowledge, observation, consistency, and truth provenance**.

That leads to the next layer:

# Part XLIX — NROS Knowledge & State Fabric

The central transition will be:

```text
ROS:
    publish data

NROS:
    observe
      ↓
    validate
      ↓
    correlate
      ↓
    derive
      ↓
    establish state
      ↓
    track confidence
      ↓
    make decisions
```

This is where NROS begins moving from a **robot middleware** toward a genuine **robotic runtime substrate**.

# NROS — Part XLIX: Knowledge & State Fabric

The Resource Fabric answered:

> **What can the system use?**

The next question is:

> **What does the system believe is happening?**

This is where NROS must move beyond the traditional ROS model of simply transporting messages.

A sensor message is **data**.

A robot's operational understanding is **state**.

And a decision based on multiple observations is **knowledge**.

# 1. Data → Observation → State → Knowledge

NROS should distinguish four levels:

```text
Sensor/Data
    ↓
Observation
    ↓
State
    ↓
Knowledge
    ↓
Decision
```

For example:

```text
Camera frame
    ↓
Observation:
    "object detected at X"
    ↓
State:
    "object O is probably at X"
    ↓
Knowledge:
    "O blocks planned route"
    ↓
Decision:
    "replan"
```

This distinction is foundational.

# 2. Raw Data Is Not Truth

Suppose:

```text
Lidar:
    obstacle = (4.1, 2.0)

Camera:
    obstacle = (4.3, 2.1)

Radar:
    obstacle = (4.0, 1.9)
```

NROS should not simply overwrite one value with another.

Instead:

```text
Observation A
Observation B
Observation C
        ↓
   Correlation
        ↓
    Fusion
        ↓
    State Estimate
```

# 3. Observation

An observation is an assertion derived from some source.

Conceptually:

```text
Observation
├── observation_id
├── source
├── subject
├── predicate
├── value
├── timestamp
├── validity
├── confidence
└── provenance
```

Example:

```text
source = lidar.front
subject = obstacle.17
predicate = position
value = (4.1, 2.0)
confidence = 0.92
```

# 4. Observation Provenance

Every important observation should answer:

```text
Who produced it?
What produced it?
When?
Using which resource?
Under which deployment?
From which domain?
```

Example:

```text
Observation O71
    source:
        lidar.front

    resource:
        lidar0

    component:
        perception.lidar

    domain:
        perception

    timestamp:
        T

    evidence:
        E902
```

This makes sensor information auditable.

# 5. Observation Validity

Observations should not simply be:

```text
true / false
```

Instead:

```text
VALID
INVALID
EXPIRED
SUPERSEDED
RETRACTED
UNKNOWN
```

An observation can become stale without ever having been wrong.

# 6. Temporal Validity

Consider:

```text
speed = 2.0 m/s
```

at:

```text
12:00:00
```

At:

```text
12:00:10
```

that value may no longer represent reality.

Therefore observations need:

```text
observed_at
valid_from
valid_until
```

where appropriate.

# 7. Confidence

NROS should support confidence independently from validity.

For example:

```text
valid = true
confidence = 0.73
```

This means:

> The observation is structurally valid, but the system is only moderately confident in the estimate.

This distinction is essential for sensor fusion.

# 8. Confidence Is Not Truth Probability

We should avoid prematurely defining:

```text
confidence = probability of truth
```

because different algorithms interpret confidence differently.

Instead, NROS should define a semantic contract such as:

```text
confidence:
    bounded quantitative indication of source certainty
```

with algorithm-specific interpretation.

# 9. State

State is a structured representation of the current known condition of an entity.

Example:

```text
RobotState
├── pose
├── velocity
├── battery
├── mode
├── health
└── active_mission
```

State should not merely be the latest message received.

It should be a **derived, versioned representation**.

# 10. State Ownership

Every authoritative state should have an owner.

For example:

```text
MotionController
    owns:
        robot.velocity
```

while:

```text
Localization
    owns:
        robot.pose
```

This prevents arbitrary components from silently mutating shared truth.

# 11. State Authorities

For each state field:

```text
state field
    ↓
authoritative producer
```

Other components may provide:

```text
observations
```

but only the designated authority commits the canonical state.

# 12. State vs Observation

Example:

```text
Lidar:
    "obstacle detected"
```

is an:

```text
Observation
```

while:

```text
WorldModel:
    "obstacle O17 occupies region R"
```

is:

```text
State
```

The latter may combine:

```text
lidar
camera
radar
history
motion model
```

# 13. State Versioning

Every state should have a version:

```text
StateVersion = 1842
```

A transition becomes:

```text
v1842
   ↓
event
   ↓
v1843
```

This gives NROS deterministic state history.

# 14. State Snapshot

The runtime should support snapshots:

```text
snapshot()
```

producing:

```text
StateSnapshot
├── version
├── timestamp
├── entities
├── resources
├── domains
└── active work
```

This is useful for:

```text
recovery
debugging
simulation
replay
migration
```

# 15. State Delta

Transmitting an entire world state repeatedly can be expensive.

NROS can represent:

```text
StateDelta
```

such as:

```text
robot.pose changed
battery changed
mission.status changed
```

Then:

```text
Snapshot
+
Delta stream
=
Current state
```

# 16. State Machine

Every important entity can have a formal state machine:

```text
IDLE
 ↓
STARTING
 ↓
RUNNING
 ↓
DEGRADED
 ↓
STOPPING
 ↓
STOPPED
```

Transitions should be explicit.

# 17. State Transition Authority

A component should not simply write:

```text
state = RUNNING
```

Instead:

```text
request transition
       ↓
policy validation
       ↓
preconditions
       ↓
transition
       ↓
event
```

This prevents invalid states.

# 18. Preconditions

A transition can require:

```text
RUNNING
```

only if:

```text
resource available
capability granted
dependencies healthy
configuration valid
```

For example:

```text
START_MOTOR
```

requires:

```text
safety_gate = READY
motor = AVAILABLE
controller = HEALTHY
```

# 19. Invariants

NROS should support explicit invariants.

Example:

```text
Motor.velocity ≤ SafetyLimit
```

or:

```text
battery.remaining ≥ minimum_safe_energy
```

An invariant is stronger than a recommendation.

If violated:

```text
InvariantViolation
```

must become a first-class runtime event.

# 20. Knowledge

Knowledge is derived information that can support decisions.

Example:

```text
State:
    obstacle.position = X

Knowledge:
    planned_path intersects obstacle
```

Knowledge therefore has:

```text
sources
derivation
confidence
validity
```

# 21. Knowledge Provenance

Suppose the planner decides:

```text
REPLAN
```

NROS should eventually be able to answer:

```text
Why?
```

For example:

```text
REPLAN
  because:
      obstacle O17
  derived from:
      lidar observation O71
      camera observation O83
  fused by:
      perception.fusion
  evaluated against:
      route R42
```

This is far beyond a conventional message trace.

# 22. Knowledge Graph

NROS can conceptually represent:

```text
Robot
 ├── hasResource → Motor0
 ├── hasState → Moving
 ├── observes → Obstacle17
 ├── executes → Mission42
 └── locatedAt → Zone3

Obstacle17
 ├── position → P
 ├── confidence → 0.91
 └── intersects → Path7
```

This creates a semantic graph over runtime state.

# 23. Entities

The Knowledge Fabric needs stable entity identities:

```text
EntityId
```

Examples:

```text
robot.01
obstacle.17
mission.42
person.3
warehouse.zone.7
motor.left
```

Identity must survive individual messages.

# 24. Entity Lifecycle

Entities can be:

```text
DISCOVERED
TRACKED
ACTIVE
LOST
EXPIRED
REMOVED
```

For example:

```text
Obstacle17
    ↓
TRACKED
    ↓
not observed
    ↓
STALE
    ↓
LOST
```

# 25. Entity Re-identification

This is important for perception.

A camera may see:

```text
object A
```

and later:

```text
object B
```

NROS should allow perception systems to establish:

```text
B ≈ A
```

with confidence and provenance rather than forcing every observation to create a new identity.

# 26. World Model

The accumulated state can form a:

```text
WorldModel
```

containing:

```text
entities
relations
locations
resources
hazards
missions
environment
```

This becomes a major NROS subsystem.

# 27. Local vs Global Knowledge

Not all knowledge should be globally visible.

For example:

```text
local obstacle map
```

may belong to:

```text
robot.01
```

while:

```text
fleet map
```

belongs to:

```text
fleet
```

Therefore NROS needs scopes:

```text
LOCAL
DOMAIN
ROBOT
FLEET
EDGE
CLOUD
```

# 28. Knowledge Authority

Different components may produce competing estimates.

Example:

```text
Localization A
    pose = X

Localization B
    pose = Y
```

NROS needs a policy determining:

```text
which estimate is authoritative
```

or whether both remain available as observations.

# 29. Multiple Truth Domains

A useful model is:

```text
Observation
    ↓
Estimator
    ↓
State Authority
```

The state authority chooses the canonical estimate.

Alternative estimates remain available for:

```text
diagnostics
comparison
fallback
research
```

# 30. Fusion

Fusion can be represented explicitly:

```text
FusionJob
├── inputs
├── algorithm
├── output
├── timestamp
└── provenance
```

Example:

```text
Lidar
Camera
IMU
  ↓
Fusion
  ↓
PoseEstimate
```

# 31. Temporal Fusion

Sensors operate at different frequencies:

```text
IMU       400 Hz
Camera     30 Hz
Lidar      10 Hz
GPS         1 Hz
```

The Knowledge Fabric must preserve timestamps rather than treating arrival order as truth.

This connects directly to the Temporal Fabric.

# 32. Late Data

Suppose:

```text
t=10.0
state S10

t=9.8
late observation arrives
```

NROS must define whether the system:

```text
ignores it
updates history
recomputes state
marks it for diagnostics
```

This is a semantic policy, not merely a transport decision.

# 33. Stale Data

Every stateful consumer should be able to ask:

```text
Is this state fresh enough?
```

Example:

```text
pose age < 100 ms
```

If not:

```text
PoseStale
```

becomes an explicit condition.

# 34. Contradictions

The Knowledge Fabric must explicitly represent contradictions.

Example:

```text
Source A:
    door = OPEN

Source B:
    door = CLOSED
```

Rather than silently selecting one:

```text
Contradiction
├── subject
├── propositions
├── sources
├── timestamps
└── resolution
```

can be recorded.

# 35. Contradiction Resolution

Possible strategies:

```text
priority
recency
confidence
sensor trust
majority
fusion
human confirmation
```

The policy determines the appropriate strategy.

# 36. Unknown State

One of the most important principles:

```text
UNKNOWN ≠ FALSE
```

If a sensor stops reporting:

```text
door = UNKNOWN
```

is often safer than:

```text
door = CLOSED
```

NROS should make this distinction explicit.

# 37. Epistemic State

A component can distinguish:

```text
KNOWN_TRUE
KNOWN_FALSE
UNKNOWN
CONFLICTED
STALE
UNTRUSTED
```

This provides a much richer semantic model than boolean flags.

# 38. Decision Preconditions

A decision can require:

```text
pose = KNOWN
battery > 20%
obstacle_map = FRESH
```

If:

```text
obstacle_map = UNKNOWN
```

the decision may become:

```text
WAIT
```

rather than:

```text
EXECUTE
```

# 39. Knowledge Queries

NROS should eventually support queries such as:

```text
get_state(robot)
```

or:

```text
query:
    all obstacles
    within region R
    confidence > 0.8
    observed within 500ms
```

This moves beyond topic subscription toward semantic access.

# 40. State Query vs Event Subscription

Both are required.

### Query

```text
What is true now?
```

### Event

```text
What changed?
```

NROS therefore needs:

```text
Query
+
Event
```

as complementary primitives.

# 41. Historical Queries

Because state is versioned:

```text
What was the robot state at T?
```

should eventually be answerable.

For example:

```text
state(robot, t=12:04:31)
```

This is extremely valuable for incident reconstruction.

# 42. Time-Travel Debugging

Given:

```text
StateSnapshot S100
```

and:

```text
Events E101 ... E150
```

the runtime can reconstruct:

```text
S150
```

This enables:

```text
replay
debugging
simulation
postmortem
```

# 43. Event Sourcing

This naturally suggests:

```text
State
    =
initial snapshot
+
ordered transitions/events
```

NROS does not have to mandate pure event sourcing everywhere.

But its protocol should make event-derived reconstruction possible where required.

# 44. Knowledge Expiration

Knowledge can have a TTL:

```text
ObstacleKnowledge
    expires_after = 2s
```

After expiration:

```text
VALID
 ↓
STALE
 ↓
UNKNOWN
```

This is particularly useful for dynamic environments.

# 45. Confidence Decay

Some knowledge should lose confidence over time.

Conceptually:

```text
confidence(t)
```

decreases according to a domain-specific policy.

For example:

```text
object position:
    confidence decreases as time since observation increases
```

The runtime need not impose the mathematical model; it should expose the mechanism.

# 46. State Consistency

Different components may hold different views:

```text
Planner:
    robot = MOVING

Controller:
    robot = STOPPED

Hardware:
    motor = FAULT
```

NROS should not pretend these are automatically consistent.

Instead, consistency itself becomes observable:

```text
ConsistencyStatus
    = CONSISTENT
    | DIVERGENT
    | UNKNOWN
```

# 47. State Reconciliation

A reconciliation process can determine:

```text
expected state
        vs
observed state
```

Example:

```text
Controller says:
    motor ACTIVE

Hardware says:
    motor FAULTED
```

Result:

```text
hardware state wins
controller state invalidated
```

according to authority policy.

# 48. Knowledge Safety

Knowledge used for safety decisions should carry stronger requirements:

```text
SafetyCriticalKnowledge
```

may require:

```text
freshness
minimum confidence
trusted source
validated derivation
known clock quality
```

This prevents an arbitrary stale observation from influencing a critical actuator.

# 49. Knowledge → Work

Now the autonomous loop becomes:

```text
Observe
  ↓
Interpret
  ↓
Update State
  ↓
Derive Knowledge
  ↓
Generate Intent
  ↓
Create Work
  ↓
Allocate Resources
  ↓
Execute
  ↓
Observe Result
```

This is the foundation for the NROS agentic execution model.

# 50. ROS Computation Graph vs NROS Knowledge Graph

Traditional ROS emphasizes:

```text
Node
  ↕
Topic
  ↕
Node
```

NROS expands this into:

```text
Entities
   ↓
Observations
   ↓
State
   ↓
Knowledge
   ↓
Intent
   ↓
Work
   ↓
Resources
   ↓
Execution
   ↓
Evidence
```

The graph is no longer only a **communication graph**.

It becomes a **causal execution graph**.

# 51. NROS Eleven-Fabric Architecture

We now have:

```text
┌──────────────────────────────────────────┐
│ Domain & Deployment                      │
├──────────────────────────────────────────┤
│ Resource & Allocation                    │
├──────────────────────────────────────────┤
│ Knowledge & State                        │
├──────────────────────────────────────────┤
│ Protocol & Type                          │
├──────────────────────────────────────────┤
│ Observability & Evidence                 │
├──────────────────────────────────────────┤
│ Capability & Authority                   │
├──────────────────────────────────────────┤
│ Supervision                              │
├──────────────────────────────────────────┤
│ Temporal                                 │
├──────────────────────────────────────────┤
│ Execution                                │
├──────────────────────────────────────────┤
│ Communication                            │
└──────────────────────────────────────────┘
```

# 52. The Critical Difference From ROS

ROS essentially asks:

> How do independent robot processes communicate?

NROS asks a much larger question:

> How does a distributed autonomous system establish knowledge, obtain authority, acquire resources, execute work, maintain state, recover from failure, and produce evidence?

That is the conceptual leap.

# 53. The Autonomous Control Loop

We can now formalize the NROS loop:

```text
┌──────────────┐
│   OBSERVE    │
└──────┬───────┘
       ↓
┌──────────────┐
│   INTERPRET  │
└──────┬───────┘
       ↓
┌──────────────┐
│ UPDATE STATE │
└──────┬───────┘
       ↓
┌──────────────┐
│     PLAN     │
└──────┬───────┘
       ↓
┌──────────────┐
│   AUTHORIZE  │
└──────┬───────┘
       ↓
┌──────────────┐
│   ALLOCATE   │
└──────┬───────┘
       ↓
┌──────────────┐
│   EXECUTE    │
└──────┬───────┘
       ↓
┌──────────────┐
│   VERIFY     │
└──────┬───────┘
       ↓
     OBSERVE
```

This loop should be native to NROS rather than implemented independently by every application.

# 54. The Next Missing Layer: Intent

We now have enough primitives to represent:

```text
observations
state
knowledge
authority
resources
domains
work
execution
```

But something still sits between **knowledge** and **work**.

That is:

# **Intent**

An autonomous system does not directly turn every observation into an executable command.

It first establishes:

> **What is it trying to accomplish?**

For example:

```text
Knowledge:
    battery = 15%

        ↓

Intent:
    preserve operational capability

        ↓

Plan:
    navigate to charging station

        ↓

Work:
    navigate(...)
```

Therefore the next layer should define:

# Part L — NROS Intent & Planning Fabric

The key progression will be:

```text
Observation
     ↓
Knowledge
     ↓
Intent
     ↓
Goal
     ↓
Plan
     ↓
Work
     ↓
Resource Allocation
     ↓
Execution
     ↓
Verification
```

This is the point where NROS starts becoming not merely **robot middleware**, but an **autonomous execution substrate**.

# NROS — Part L: Intent & Planning Fabric

The Knowledge & State Fabric established **what the system believes to be true**.

The Resource Fabric established **what the system can use**.

Now we need the layer that connects understanding to purposeful behavior:

> **What does the robot intend to achieve, and how does that intention become executable work?**

This is the **Intent & Planning Fabric**.

# 1. The Fundamental Chain

NROS should distinguish:

```text
Observation
    ↓
Knowledge
    ↓
Intent
    ↓
Goal
    ↓
Plan
    ↓
Work
    ↓
Execution
```

These are not interchangeable concepts.

A ROS node publishing:

```text
/cmd_vel
```

is already operating near the end of this chain.

NROS wants to preserve the reasoning structure that produced that command.

# 2. Intent

An intent represents a desired direction of behavior.

Example:

```text
Intent:
    recharge_robot
```

It does not yet specify exactly how to do it.

Another example:

```text
Intent:
    inspect_zone
```

The system may later choose:

```text
route A
```

or:

```text
route B
```

depending on state and resources.

# 3. Intent Is Not a Command

This distinction is essential.

```text
Intent:
    "Reach charging station"
```

is fundamentally different from:

```text
Command:
    "Set wheel velocity to 0.4 m/s"
```

The first expresses **purpose**.

The second expresses **mechanism**.

NROS should preserve both.

# 4. Intent Structure

Conceptually:

```text
Intent
├── intent_id
├── issuer
├── objective
├── priority
├── constraints
├── context
├── validity
├── authority
└── lifecycle
```

Example:

```text
intent = recharge
issuer = mission-manager
priority = HIGH
constraint = battery > minimum-safe-level
```

# 5. Intent Sources

Intent may originate from:

```text
Human
Mission
Planner
Agent
Safety System
Scheduler
External System
Recovery System
```

Therefore:

```text
IntentSource
```

should be explicit.

# 6. Human Intent

A human may say:

```text
"Inspect the warehouse."
```

NROS should not immediately translate this into motor commands.

Instead:

```text
Human request
    ↓
Intent
    ↓
Interpretation
    ↓
Goal
    ↓
Plan
```

This preserves the semantic boundary.

# 7. Machine-Generated Intent

An autonomous agent may derive:

```text
battery < threshold
```

and create:

```text
Intent:
    seek_energy
```

The origin remains:

```text
source = autonomous-planner
```

with provenance.

# 8. Safety-Generated Intent

A safety subsystem may generate:

```text
Intent:
    stop_motion
```

This intent should have special authority.

It should not compete with ordinary navigation intents as though both had equal priority.

# 9. Intent Authority

Every intent should carry an authority context.

Example:

```text
Navigation Agent
    intent:
        move_to(A)
```

versus:

```text
Safety Controller
    intent:
        stop
```

The system must be able to establish:

```text
Safety > Navigation
```

through explicit policy.

# 10. Goal

An intent becomes a goal when the desired outcome is sufficiently specified.

Example:

```text
Intent:
    recharge

Goal:
    battery >= 80%
```

Another:

```text
Intent:
    inspect_zone

Goal:
    every required area observed
```

A goal should be machine-evaluable.

# 11. Goal Structure

```text
Goal
├── goal_id
├── objective
├── success_conditions
├── failure_conditions
├── constraints
├── deadline
├── priority
└── provenance
```

# 12. Goal Completion

A goal should never be considered complete merely because a command finished.

Instead:

```text
Command completed
       ↓
Result observed
       ↓
State updated
       ↓
Success condition evaluated
       ↓
GOAL_ACHIEVED
```

This is a crucial difference.

# 13. Example

Goal:

```text
Reach room B
```

Plan:

```text
navigate(A → B)
```

Navigation reports:

```text
SUCCESS
```

But localization says:

```text
robot is actually in room A
```

NROS should conclude:

```text
goal = NOT_ACHIEVED
```

The execution result alone is insufficient.

# 14. Goal Preconditions

Goals can require:

```text
battery > 20%
localization = valid
route = available
```

If these are false:

```text
goal = BLOCKED
```

rather than:

```text
goal = FAILED
```

This distinction matters.

# 15. Goal Lifecycle

A useful lifecycle is:

```text
PROPOSED
   ↓
ACCEPTED
   ↓
READY
   ↓
PLANNING
   ↓
EXECUTING
   ↓
VERIFYING
   ↓
ACHIEVED
```

Alternative paths:

```text
READY → BLOCKED
EXECUTING → FAILED
EXECUTING → CANCELLED
EXECUTING → PREEMPTED
```

# 16. Planning

Planning transforms:

```text
Goal
+
Current State
+
Constraints
+
Available Resources
```

into:

```text
Plan
```

Conceptually:

```text
Plan = f(State, Goal, Constraints, Resources, Policy)
```

The exact planning algorithm is not part of the core protocol.

# 17. Plan Is Not Execution

A plan is a proposal.

For example:

```text
Plan P42

1. Navigate to A
2. Inspect object
3. Navigate to B
4. Dock
```

The runtime still has to determine:

```text
Can this plan execute now?
```

# 18. Plan Validation

Before execution:

```text
Plan
 ↓
Resource requirements
 ↓
Capability requirements
 ↓
Temporal constraints
 ↓
Safety constraints
 ↓
Authority
 ↓
Validation
```

Only then:

```text
Plan → Work
```

# 19. Plan Alternatives

A planner should be able to provide alternatives:

```text
Plan A
Plan B
Plan C
```

with metadata:

```text
cost
risk
duration
energy
confidence
resource usage
```

The scheduler/policy layer can select one.

# 20. Plan Cost

NROS should not mandate one universal definition of cost.

A domain may define:

```text
Cost
├── time
├── energy
├── risk
├── distance
├── resource consumption
└── uncertainty
```

A plan can expose multiple dimensions.

# 21. Multi-Objective Planning

Example:

```text
Plan A:
    10 min
    20 Wh
    low risk

Plan B:
    6 min
    35 Wh
    medium risk
```

The runtime can use policy:

```text
minimize energy
```

or:

```text
minimize time
```

without changing the underlying plan representation.

# 22. Plan Validity

A plan is not permanently valid.

If:

```text
obstacle appears
```

then:

```text
Plan P42
```

may become:

```text
INVALIDATED
```

The planner must be able to replan.

# 23. Plan Versioning

Plans should be immutable/versioned:

```text
Plan P42 v1
Plan P42 v2
Plan P42 v3
```

This makes it possible to answer:

> Which plan was actually executed?

# 24. Plan Provenance

Every plan should record:

```text
generated_by
generated_at
input_state_version
input_knowledge
constraints
planner_version
```

For example:

```text
Plan P42
    generated_by:
        planner-v7

    state:
        S1842

    knowledge:
        K93

    planner:
        version 7.2
```

This becomes extremely valuable for reproducibility.

# 25. Planning Context

Planning should operate inside an explicit context:

```text
PlanningContext
├── world_state
├── knowledge
├── goals
├── resources
├── capabilities
├── policies
├── temporal_constraints
└── environmental assumptions
```

This prevents hidden dependencies.

# 26. Assumptions

A planner may assume:

```text
battery remains above 20%
```

or:

```text
door remains open
```

These assumptions should be represented explicitly.

# 27. Assumption Monitoring

If an assumption becomes false:

```text
assumption violated
       ↓
plan invalidation
       ↓
replanning
```

This is much safer than blindly continuing.

# 28. Plan Steps

A plan consists of semantic steps:

```text
Plan
├── Step 1
├── Step 2
├── Step 3
└── Step 4
```

Each step may contain:

```text
action
preconditions
effects
resources
timeout
failure policy
```

# 29. Action

An action represents an operation that can be executed.

Example:

```text
NavigateTo
```

with:

```text
target = room_B
```

Another:

```text
CaptureImage
```

with:

```text
camera = front
```

# 30. Action vs Work

An action describes **what should happen**.

Work describes an **actual runtime execution instance**.

For example:

```text
Action:
    NavigateTo(room_B)
```

becomes:

```text
Work:
    W193
    action = NavigateTo(room_B)
    status = RUNNING
```

This distinction gives us traceability.

# 31. Work Identity

Every executable operation gets:

```text
WorkId
```

Example:

```text
W193
```

Everything associated with that operation can reference it:

```text
allocation
events
logs
state transitions
failures
evidence
```

# 32. Work Tree

Complex actions can decompose:

```text
Work W1
├── W1.1 Navigate
├── W1.2 Align
├── W1.3 Inspect
└── W1.4 Report
```

This gives NROS a hierarchical execution model.

# 33. Work Dependencies

Work items can depend on one another:

```text
W2 depends_on W1
```

or:

```text
W3 = W1 AND W2
```

or:

```text
W4 = W1 OR W2
```

This can support richer planning structures.

# 34. Sequential Execution

Simple plan:

```text
W1
 ↓
W2
 ↓
W3
```

Straightforward.

# 35. Parallel Execution

Some work can run concurrently:

```text
       ┌── W2 ──┐
W1 ────┤        ├── W4
       └── W3 ──┘
```

For example:

```text
W2 = map environment
W3 = monitor battery
```

while both precede:

```text
W4 = execute mission
```

# 36. Conditional Execution

A plan may contain:

```text
if battery < 20%:
    recharge
else:
    continue
```

This should be represented semantically rather than embedded as arbitrary shell scripting.

# 37. Loops

Autonomous behavior often requires:

```text
while goal_not_satisfied:
    observe
    plan
    execute
```

NROS should support bounded or policy-controlled loops.

For safety:

```text
maximum_iterations
maximum_duration
resource_budget
```

can constrain loops.

# 38. Replanning

The runtime should support:

```text
execute
 ↓
observe
 ↓
plan invalid
 ↓
replan
```

This means the plan is not necessarily a static script.

It can be a **living execution hypothesis**.

# 39. Partial Plan Commitment

A planner might produce:

```text
A → B → C → D → E
```

but the runtime commits only:

```text
A → B → C
```

because the environment beyond C is uncertain.

This reduces risk.

# 40. Rolling Horizon Planning

NROS can support:

```text
plan horizon = H
```

Then repeatedly:

```text
plan
 ↓
execute partial horizon
 ↓
observe
 ↓
replan
```

This is especially suitable for dynamic robotics.

# 41. Plan Checkpoints

A plan can define checkpoints:

```text
Checkpoint 1:
    robot at A

Checkpoint 2:
    object inspected

Checkpoint 3:
    robot docked
```

After a failure, recovery can resume from the latest valid checkpoint.

# 42. Plan Failure

A failed action does not necessarily mean the goal failed.

Example:

```text
Plan:
    Route A
```

fails because:

```text
obstacle
```

The system can:

```text
replan Route B
```

while preserving the same goal.

Therefore:

```text
Action failure
    ≠
Goal failure
```

# 43. Failure Classes

NROS should distinguish:

```text
BLOCKED
TIMEOUT
RESOURCE_UNAVAILABLE
AUTHORIZATION_DENIED
PRECONDITION_FAILED
EXECUTION_ERROR
SAFETY_VIOLATION
ENVIRONMENT_CHANGED
UNKNOWN
```

This enables intelligent recovery.

# 44. Recovery Policy

Every plan/action can specify:

```text
on_failure:
    retry
    replan
    fallback
    compensate
    abort
    safe_stop
```

The runtime chooses according to policy and authority.

# 45. Compensation

Some actions have reversible effects.

Example:

```text
lock_door
```

could have:

```text
compensation:
    unlock_door
```

This supports transactional-style workflows.

# 46. Irreversible Actions

Other actions cannot be safely undone:

```text
release_payload
activate_hazardous_device
```

These require stronger authorization.

A plan can mark:

```text
reversibility = IRREVERSIBLE
```

and trigger additional policy checks.

# 47. Human Approval

Some plans may require human approval:

```text
Plan
 ↓
risk assessment
 ↓
approval required
 ↓
HUMAN_APPROVAL
 ↓
execution
```

The approval itself becomes an auditable event.

# 48. Agentic Planning

Now the NROS agent model becomes clear.

An agent can operate:

```text
Observe
 ↓
Interpret
 ↓
Establish State
 ↓
Form Intent
 ↓
Generate Goal
 ↓
Plan
 ↓
Request Authority
 ↓
Acquire Resources
 ↓
Execute
 ↓
Verify
 ↓
Reflect
 ↓
Replan
```

This is much closer to an **agent-native runtime** than a conventional robotics middleware.

# 49. Agent ≠ Node

A ROS node is fundamentally:

```text
process
```

An NROS agent is better modeled as:

```text
Agent
├── identity
├── goals
├── intents
├── policy
├── memory
├── planning
├── work
├── resources
└── lifecycle
```

An agent may use processes, threads, containers, GPUs, or remote services internally.

# 50. Agent Lifecycle

Potential lifecycle:

```text
CREATED
   ↓
INITIALIZING
   ↓
READY
   ↓
ACTIVE
   ↓
DEGRADED
   ↓
SUSPENDED
   ↓
RECOVERING
   ↓
STOPPED
```

# 51. Agent Memory

Planning requires memory.

The Knowledge Fabric provides state and knowledge, but agents may also require:

```text
episodic memory
semantic memory
working memory
procedural memory
```

This should not necessarily be collapsed into one database.

The protocol should distinguish memory semantics.

# 52. Working Memory

During planning:

```text
Current goal
Candidate plans
Assumptions
Constraints
Recent observations
```

form working memory.

This is transient.

# 53. Episodic Memory

Past events:

```text
mission started
obstacle encountered
plan changed
goal completed
```

can become episodic memory.

This is useful for future planning.

# 54. Semantic Memory

Stable knowledge:

```text
room A is restricted
charger is at location X
tool Y requires capability Z
```

can become semantic memory.

# 55. Procedural Memory

Reusable procedures:

```text
dock_robot()
inspect_panel()
recover_motor()
```

can become procedural knowledge.

# 56. Planning and Memory

The planner can therefore consume:

```text
Current State
+
Current Knowledge
+
Memory
+
Goal
+
Constraints
```

to generate a plan.

This gives NROS a path toward long-running autonomous systems.

# 57. Verification

The plan must not be considered successful merely because execution returned success.

NROS should perform:

```text
Expected Effect
        ↓
Observed Effect
        ↓
Verification
```

Example:

```text
Action:
    open_door

Expected:
    door = OPEN

Observed:
    door = CLOSED

Result:
    VERIFICATION_FAILED
```

# 58. Postconditions

Every action can define:

```text
preconditions
effects
```

For example:

```text
NavigateTo(B):

precondition:
    localization.valid

postcondition:
    robot.location == B
```

This makes execution semantically checkable.

# 59. Plan Evidence

A completed plan should produce:

```text
PlanExecutionEvidence
├── plan_version
├── state_before
├── work_items
├── resources
├── state_after
├── verification
└── outcome
```

Now autonomous behavior becomes reconstructable.

# 60. Complete NROS Autonomy Pipeline

We can now express the entire system:

```text
┌──────────────┐
│ OBSERVATIONS │
└──────┬───────┘
       ↓
┌──────────────┐
│   KNOWLEDGE  │
└──────┬───────┘
       ↓
┌──────────────┐
│    INTENT    │
└──────┬───────┘
       ↓
┌──────────────┐
│     GOAL     │
└──────┬───────┘
       ↓
┌──────────────┐
│     PLAN     │
└──────┬───────┘
       ↓
┌──────────────┐
│ AUTHORIZATION│
└──────┬───────┘
       ↓
┌──────────────┐
│   RESOURCE   │
│   ALLOCATION  │
└──────┬───────┘
       ↓
┌──────────────┐
│     WORK     │
└──────┬───────┘
       ↓
┌──────────────┐
│   EXECUTION  │
└──────┬───────┘
       ↓
┌──────────────┐
│ VERIFICATION │
└──────┬───────┘
       ↓
┌──────────────┐
│ STATE UPDATE │
└──────┬───────┘
       │
       └──────────→ OBSERVATION
```

This is the **NROS autonomous execution loop**.

# 61. Twelve-Fabric NROS Architecture

The architecture now becomes:

```text
┌───────────────────────────────────────────┐
│ Domain & Deployment                       │
├───────────────────────────────────────────┤
│ Resource & Allocation                     │
├───────────────────────────────────────────┤
│ Intent & Planning                         │
├───────────────────────────────────────────┤
│ Knowledge & State                         │
├───────────────────────────────────────────┤
│ Protocol & Type                           │
├───────────────────────────────────────────┤
│ Observability & Evidence                  │
├───────────────────────────────────────────┤
│ Capability & Authority                    │
├───────────────────────────────────────────┤
│ Supervision                               │
├───────────────────────────────────────────┤
│ Temporal                                  │
├───────────────────────────────────────────┤
│ Execution                                 │
├───────────────────────────────────────────┤
│ Communication                             │
└───────────────────────────────────────────┘
```

The next missing primitive is now becoming very clear.

An autonomous runtime can generate an intent, produce a plan, allocate resources, execute work, and update state.

But it also needs a **persistent, structured memory of everything that happened**—not just logs.

That leads naturally to:

# Part LI — NROS Memory, Event & Evidence Fabric

The next transformation is:

```text
Event
  ↓
Memory
  ↓
History
  ↓
Evidence
  ↓
Replay
  ↓
Learning / Diagnosis / Audit
```

The objective will be to make **history a first-class runtime primitive**, rather than something reconstructed afterward from scattered logs.
