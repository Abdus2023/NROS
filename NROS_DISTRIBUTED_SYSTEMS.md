# NROS Distributed Systems (Part XI–XVI)

The next layer is where the architecture meets the physical reality of robotics.

A component might execute:

```text
same function
same process
```

or:

```text
different process
```

or:

```text
different CPU
```

or:

```text
different robot
```

NROS should preserve the programming model while allowing the deployment topology to change.

## 1. The fundamental abstraction

The application should see:

```text
Component A
    │
    ▼
Channel<T>
    │
    ▼
Component B
```

It should **not** need to care whether the underlying path is:

```text
A ──► B
```

inside one process, or:

```text
A
│
├── shared memory ──► B
│
├── local IPC ──────► B
│
└── network ────────► B
```

This gives NROS a transport-independent communication model.

# 2. Transport hierarchy

A useful NROS architecture is:

```text
Transport
│
├── InProcess
│
├── SharedMemory
│
├── IPC
│
├── UDP
│
├── TCP
│
├── QUIC
│
└── Custom
```

But the application API remains:

```text
Channel<T>
```

rather than:

```text
UdpChannel<T>
```

unless the application explicitly needs transport-specific behavior.

# 3. The local-first principle

NROS should optimize the common case:

```text
Component A
     │
     ▼
Component B
```

when both live in the same process.

The ideal path is:

```text
publish
   │
   ▼
typed queue
   │
   ▼
consumer
```

without unnecessary:

```text
serialization
copy
kernel transition
network stack
```

# 4. Zero-copy communication

For large sensor data:

```text
Camera
  │
  ▼
ImageFrame
```

copying repeatedly is expensive:

```text
Camera
  ↓ copy
Transport
  ↓ copy
Vision
  ↓ copy
Recorder
```

NROS should support:

```text
Camera
  │
  ▼
Shared Buffer
  │
  ├── Vision
  ├── Recorder
  └── Diagnostics
```

with ownership and lifetime governed by Rust-safe abstractions.

# 5. Ownership matters

Zero-copy is dangerous if ownership is vague.

NROS should distinguish:

```text
Owned<T>
Borrowed<T>
Shared<T>
Immutable<T>
Mutable<T>
```

For example:

```text
Sensor → Shared<Image>
```

allows multiple consumers to observe the same immutable frame.

A controller requiring mutation should instead receive an appropriate owned or mutable representation.

# 6. Serialization boundary

When communication crosses a machine boundary:

```text
Component A
    │
    ▼
Serialize<T>
    │
    ▼
Transport
    │
    ▼
Deserialize<T>
    │
    ▼
Component B
```

NROS should isolate serialization from the application-level channel abstraction.

# 7. Wire representation

A message should have more than just payload bytes.

Conceptually:

```text
Frame
│
├── protocol version
├── message type
├── schema/version
├── source
├── destination
├── sequence
├── timestamp
├── flags
└── payload
```

This provides the metadata necessary for robust distributed operation.

# 8. Type identity

A receiver must know what it received.

Instead of relying exclusively on:

```text
string topic name
```

NROS should identify types explicitly:

```text
TypeId
│
├── namespace
├── name
├── version
└── schema hash
```

Example:

```text
sensor.lidar.PointCloud
version = 2
schema = HASH(...)
```

This helps detect incompatible participants.

# 9. Schema evolution

Robotics systems can remain deployed for years.

Therefore:

```text
PointCloud v1
```

may need to communicate with:

```text
PointCloud v2
```

NROS should define compatibility rules:

```text
SchemaCompatibility
│
├── Exact
├── BackwardCompatible
├── ForwardCompatible
└── Incompatible
```

Compatibility should be explicit rather than accidental.

# 10. Discovery

ROS 1 relied heavily on the Master.

NROS should avoid requiring one centralized coordinator for every deployment.

The conceptual model becomes:

```text
Participant
   │
   ▼
Discovery
   │
   ├── who exists?
   ├── what types?
   ├── what channels?
   ├── what capabilities?
   └── what endpoints?
```

# 11. Local discovery

Inside one process:

```text
Runtime Registry
```

may be enough.

Across one machine:

```text
Local Discovery Service
```

could maintain participants.

Across a network:

```text
Distributed Discovery
```

is required.

The discovery abstraction should remain consistent.

# 12. Discovery is not data transport

This distinction is critical.

Discovery answers:

> **Who can communicate with whom?**

Transport answers:

> **How do the actual messages move?**

Therefore:

```text
Discovery
    │
    ▼
Endpoint Information
    │
    ▼
Transport
```

Data should not unnecessarily pass through the discovery mechanism.

# 13. Endpoint model

A channel endpoint might expose:

```text
Endpoint
│
├── participant_id
├── component_id
├── channel_id
├── type_id
├── transport
├── address
├── QoS
└── capabilities
```

A publisher can then discover compatible subscribers.

# 14. Distributed channel

The resulting architecture:

```text
Publisher
    │
    ▼
Channel<T>
    │
    ▼
Discovery
    │
    ▼
Endpoint Selection
    │
    ▼
Transport
    │
    ▼
Subscriber
```

The application remains unaware of the discovery protocol unless it explicitly asks for that information.

# 15. QoS

ROS 2 made Quality of Service a major part of its architecture.

NROS should retain the useful concept but make the semantics explicit.

Possible dimensions:

```text
QoS
│
├── Reliability
├── Durability
├── History
├── Depth
├── Deadline
├── Lifespan
├── Priority
└── Delivery mode
```

# 16. Reliability

A channel may be:

```text
BestEffort
```

or:

```text
Reliable
```

For camera frames:

```text
/camera/image
    → BestEffort
```

may be perfectly reasonable.

For configuration commands:

```text
/robot/config
    → Reliable
```

is more appropriate.

# 17. History

NROS could support:

```text
History
│
├── KeepLast(N)
└── KeepAll
```

But `KeepAll` should not be interpreted as infinite memory.

Resource limits must remain enforceable.

# 18. Durability

A late subscriber may need the latest configuration.

Example:

```text
Configuration Publisher
       │
       ▼
Current configuration
       │
       ▼
Late subscriber
```

NROS can support explicit durability policies.

Again, this must be bounded and resource-aware.

# 19. Deadline QoS

Transport-level deadline and execution-level deadline are related but distinct.

```text
Transport deadline:
    message delivery constraint

Execution deadline:
    computation completion constraint
```

For example:

```text
Sensor
  │
  │ delivery ≤ 5ms
  ▼
Controller
  │
  │ execution ≤ 500µs
  ▼
Motor
```

NROS should preserve this distinction.

# 20. Backpressure

A critical distributed problem is:

```text
Producer
  │
  │ 1000 msg/s
  ▼
Queue
  │
  │ consumer handles 100 msg/s
  ▼
Consumer
```

Without policy, the queue grows indefinitely.

NROS should define:

```text
BackpressurePolicy
│
├── DropOldest
├── DropNewest
├── Block
├── Reject
└── Sample
```

The appropriate policy depends on the channel semantics.

# 21. Backpressure is a safety issue

For control traffic:

```text
Queue grows
    │
    ▼
stale commands
    │
    ▼
unsafe behavior
```

Therefore bounded queues and freshness policies can become safety mechanisms.

For example:

```text
MotorCommand
lifespan = 20 ms
```

A command arriving after its lifespan can simply be rejected.

# 22. Network partitions

Distributed robotics must assume connectivity failures.

```text
Robot A
   X
Robot B
```

NROS should explicitly model:

```text
Connectivity
│
├── Connected
├── Degraded
├── Partitioned
└── Reconnecting
```

Applications can then react to meaningful state rather than raw socket errors.

# 23. Partition policy

For a disconnected component:

```text
Partition detected
      │
      ├── continue locally
      ├── degrade
      ├── pause
      ├── retry
      └── safe-stop
```

The correct behavior depends on the component's declared policy.

# 24. Local autonomy

This leads to a powerful property:

```text
Network
   │
   X
   │
Robot
   │
   ├── safety controller → continues
   ├── local state → continues
   └── cloud planner → unavailable
```

NROS should allow the deployment to define which functions are:

```text
local-critical
```

versus:

```text
remote-optional
```

# 25. Transport selection

A channel could have:

```text
TransportPolicy
│
├── preferred = SharedMemory
├── fallback = IPC
└── remote = UDP
```

The runtime selects an appropriate implementation based on topology.

Thus:

```text
same process
   → InProcess

same host
   → SharedMemory

remote host
   → Network
```

without changing application code.

# 26. Discovery + topology

NROS can represent deployment topology:

```text
Machine A
│
├── CPU0
│   ├── Safety
│   └── Controller
│
└── CPU1
    ├── Vision
    └── Planner

Machine B
│
└── AI Accelerator
    └── Inference
```

The deployment layer maps components to physical resources.

# 27. Placement

Component placement becomes a policy:

```text
Component
   │
   ▼
PlacementPolicy
   │
   ├── CPU affinity
   ├── NUMA node
   ├── device
   ├── host
   └── isolation domain
```

For example:

```text
SafetyController
    → CPU 0
    → isolated
    → local
```

while:

```text
VisionModel
    → GPU
```

# 28. Same API, different deployment

This is the key NROS promise:

```text
                Application
                     │
                     ▼
                  NROS API
                     │
          ┌──────────┼──────────┐
          ▼          ▼          ▼
      in-process    local      remote
        runtime      IPC       network
```

The component's logical contract stays stable.

Only deployment changes.

# 29. ROS Master → NROS Discovery

The conceptual transformation:

```text
ROS 1 Master
      │
      ▼
Central registration
      │
      ▼
Peer communication
```

becomes:

```text
NROS Discovery
      │
      ├── local registry
      ├── peer discovery
      ├── static discovery
      └── distributed discovery
              │
              ▼
        endpoint selection
              │
              ▼
          peer transport
```

No single discovery mechanism has to dominate every deployment.

# 30. ROS 2 DDS → NROS transport abstraction

The goal should **not** be to blindly reimplement DDS.

Instead:

```text
NROS
 │
 └── Transport Abstraction
       │
       ├── native NROS transport
       ├── DDS bridge
       ├── UDP transport
       ├── shared memory
       └── custom embedded transport
```

This preserves interoperability without coupling the NROS execution model to one middleware implementation.

# 31. ROS interoperability

Migration becomes:

```text
ROS 1
 │
 ▼
ROS Bridge
 │
 ▼
NROS Channel
```

or:

```text
ROS 2 / DDS
       │
       ▼
NROS Transport Adapter
       │
       ▼
NROS Component
```

This is important because replacing an entire ROS ecosystem at once is unrealistic.

# 32. Embedded transport

NROS should also work without a conventional IP stack.

Example:

```text
MCU
 │
 ├── CAN
 ├── SPI
 ├── UART
 └── custom bus
```

The same logical abstraction can remain:

```text
Channel<MotorCommand>
```

with a specialized transport implementation.

# 33. The transport boundary

This suggests a very important NROS API boundary:

```text
              NROS Core
                  │
             Channel<T>
                  │
                  ▼
          Transport Interface
                  │
      ┌───────────┼───────────┐
      ▼           ▼           ▼
  InProcess    SharedMem    Network
                              │
                    ┌─────────┼─────────┐
                    ▼         ▼         ▼
                   UDP       QUIC      DDS
```

Core should not know the details of UDP, DDS, or shared memory.

# 34. Transport security

Distributed communication also introduces security concerns:

```text
Authentication
Authorization
Integrity
Confidentiality
Replay protection
```

But again:

```text
Security policy
      ≠
transport implementation
```

The transport provides mechanisms; the NROS security layer defines policy.

# 35. Identity

Distributed NROS participants need stable identities:

```text
ParticipantId
ComponentId
ChannelId
GoalId
ActivationId
EffectId
```

These should be globally distinguishable enough for tracing and diagnostics.

A complete causal chain can then be reconstructed:

```text
Participant
  ↓
Component
  ↓
Activation
  ↓
Effect
  ↓
Observation
  ↓
Goal
```

# 36. Distributed tracing

Now combine everything:

```text
Machine A
  Sensor
    │
    ▼
  A100
    │
    │ network
    ▼
Machine B
  Localization
    │
    ▼
  A101
    │
    ▼
  Planner
    │
    ▼
  A102
```

NROS can record:

```text
causal ID
timestamps
transport latency
queue latency
execution latency
deadline status
```

This makes distributed performance diagnosable.

# 37. End-to-end latency

Instead of measuring only:

```text
callback runtime = 200µs
```

NROS can measure:

```text
Sensor acquisition
      +
serialization
      +
network
      +
queue
      +
scheduler
      +
execution
      +
actuator command
```

For robotics, the **end-to-end path** is often more important than any individual callback.

# 38. NROS distributed model

We can now express the complete distributed architecture:

```text
┌──────────────────────────────────────────────────┐
│                  NROS APPLICATION                │
├──────────────────────────────────────────────────┤
│ Components │ Goals │ Channels │ Services │ Events │
├──────────────────────────────────────────────────┤
│             EXECUTION MODEL                     │
│ Activation │ Scheduler │ Executor │ Effects      │
├──────────────────────────────────────────────────┤
│              RESOURCE MODEL                      │
│ Capabilities │ Ownership │ Budgets │ Isolation    │
├──────────────────────────────────────────────────┤
│              DISTRIBUTION                        │
│ Discovery │ Endpoint │ QoS │ Transport           │
├──────────────────────────────────────────────────┤
│              OBSERVABILITY                       │
│ Trace │ Metrics │ Replay │ Diagnostics            │
├──────────────────────────────────────────────────┤
│       OS / RTOS / MCU / NETWORK / HARDWARE      │
└──────────────────────────────────────────────────┘
```

# 39. The deeper ROS → NROS transformation

We can now summarize the evolution:

```text
ROS
│
├── Node
├── Topic
├── Service
├── Action
├── Master / DDS
├── Callback
├── Parameter
└── rosbag
        │
        ▼
NROS
│
├── Component
├── Channel
├── Service
├── Goal
├── Discovery
├── Activation
├── Configuration
├── Execution Trace
│
├── Scheduler
├── Resources
├── Capabilities
├── Fault Domains
├── Safety Policies
├── Effects
├── Replay
└── Deployment
```

The transformation is therefore **semantic**, not simply linguistic.

# 40. The next layer: persistence & state

Distributed robotics eventually encounters another problem:

> **What survives process termination, machine failure, restart, or network partition?**

ROS parameters and rosbag provide pieces of this problem, but NROS can make state management explicit:

```text
Runtime State
│
├── Ephemeral state
├── Persistent configuration
├── Checkpoints
├── Goal state
├── Component state
├── Execution history
└── Recovery state
```

That leads to the next major NROS layer:

# **NROS State, Persistence, Checkpointing & Recovery**

The objective is to make:

```text
failure
   ↓
restart
   ↓
restore state
   ↓
resume safely
```

a defined runtime capability rather than an application-specific collection of recovery scripts.

# NROS — Part XII: State, Persistence, Checkpointing & Recovery

We now move from **communication** to **continuity**.

A distributed robot is not merely a collection of processes exchanging messages. It is a system that must continue operating when:

- a node crashes,
- a machine reboots,
- a network disappears,
- a sensor temporarily fails,
- an actuator becomes unavailable,
- a component is upgraded,
- or an autonomous task must resume after interruption.

ROS historically leaves much of this responsibility to application packages.

NROS should make **state and recovery explicit runtime concepts**.

## 1. The central distinction: data vs state

Not every piece of information deserves persistence.

Consider:

```text
CameraFrame
```

This is usually ephemeral:

```text
frame N
   ↓
processed
   ↓
discarded
```

But:

```text
RobotConfiguration
```

may need to survive restart.

And:

```text
NavigationGoal
```

may need to survive a temporary process failure.

Therefore NROS needs explicit state classes.

```text
State
│
├── Ephemeral
├── Runtime
├── Persistent
├── Checkpointed
└── Recoverable
```

# 2. Ephemeral state

Ephemeral state exists only while execution is active.

Examples:

```text
temporary buffer
current callback context
network packet
intermediate calculation
sensor frame
```

Its lifetime is:

```text
CREATE
  ↓
USE
  ↓
DISCARD
```

NROS should not accidentally persist this state.

# 3. Runtime state

Runtime state survives individual activations but normally disappears when the component terminates.

Example:

```text
LocalizationComponent
│
├── current_pose
├── covariance
├── tracking_status
└── active_map_region
```

Conceptually:

```text
Component
   │
   ▼
RuntimeState
```

# 4. Persistent state

Persistent state explicitly survives restart.

Examples:

```text
robot identity
calibration
configuration
mission metadata
learned parameters
persistent safety configuration
```

The runtime should distinguish:

```text
persistent ≠ merely serialized
```

Persistence implies a defined durability contract.

# 5. State ownership

Every persistent state item should have an owner.

```text
StateItem
│
├── state_id
├── owner
├── schema
├── version
├── durability
├── consistency
└── recovery_policy
```

Example:

```text
state_id = localization.map
owner    = Localization
version  = 3
```

This prevents the system from degenerating into a global mutable database.

# 6. State should have schemas

Just like messages:

```text
Message
   → TypeId
```

state should have:

```text
State
   → StateType
```

For example:

```text
nros.localization.PoseState
version = 2
schema_hash = ...
```

This becomes important during upgrades.

# 7. Checkpointing

A checkpoint captures a consistent recovery point.

Conceptually:

```text
Component
    │
    ▼
Checkpoint
    │
    ├── state
    ├── active goals
    ├── sequence numbers
    ├── configuration version
    └── recovery metadata
```

Then:

```text
crash
  ↓
restart
  ↓
restore checkpoint
  ↓
resume
```

# 8. Checkpoint ≠ snapshot of everything

A naive implementation might attempt:

```text
serialize entire process memory
```

That is usually the wrong abstraction.

Instead NROS should define **recoverable state** explicitly.

```text
Process Memory
│
├── recoverable state      ← checkpoint
├── derived state          ← recompute
├── external handles       ← reacquire
└── transient state        ← discard
```

This makes recovery deterministic.

# 9. Recovery contract

A component should declare what it can recover.

Conceptually:

```text
RecoveryPolicy
│
├── Restart
├── Restore
├── Replay
├── Recompute
├── Reconnect
└── SafeStop
```

For example:

```text
Planner
  → Restore state
  → Recompute plan
```

while:

```text
MotorController
  → SafeStop
  → Reinitialize
```

# 10. Recovery state machine

A component can move through:

```text
RUNNING
   │
   ▼
FAILURE_DETECTED
   │
   ▼
ISOLATED
   │
   ▼
RECOVERING
   │
   ├──────────────┐
   ▼              ▼
RESTORED       RECOVERY_FAILED
   │              │
   ▼              ▼
RUNNING        SAFE_STATE
```

This is much stronger than simply restarting a process.

# 11. Failure domains

NROS should explicitly model failure boundaries.

```text
Robot
│
├── Safety domain
├── Control domain
├── Perception domain
└── AI domain
```

If the AI domain crashes:

```text
AI ✕
```

the safety domain should remain operational.

# 12. Isolation

This leads to:

```text
Failure Domain
      │
      ▼
Isolation Boundary
      │
      ├── process
      ├── thread
      ├── CPU
      ├── container
      └── machine
```

The deployment layer decides how strong the isolation must be.

# 13. Goal persistence

Long-running goals deserve special treatment.

Suppose:

```text
Goal:
"Navigate to Dock B"
```

The planner crashes halfway through.

Without persistent goal state:

```text
Goal
  ↓
process crash
  ↓
lost
```

With NROS:

```text
Goal
 │
 ▼
GoalState
 │
 ├── goal_id
 ├── objective
 ├── progress
 ├── constraints
 └── recovery_policy
```

After restart:

```text
GoalState
   ↓
Recovery
   ↓
Resume / Replan / Cancel
```

# 14. Goal vs activation

This distinction becomes important.

A **goal** represents a long-lived intention.

An **activation** represents one execution opportunity.

Therefore:

```text
Goal
 │
 ├── Activation 1
 ├── Activation 2
 ├── Activation 3
 └── Activation 4
```

Example:

```text
Goal:
navigate to waypoint

Activation 1:
compute route

Activation 2:
replan

Activation 3:
avoid obstacle

Activation 4:
resume trajectory
```

The goal persists while individual activations come and go.

# 15. Agentic execution fits naturally

This is particularly valuable for autonomous agents.

```text
Goal
 │
 ▼
Observe
 │
 ▼
Plan
 │
 ▼
Execute
 │
 ▼
Reflect
 │
 ▼
Checkpoint
 │
 └──────────────► next activation
```

The checkpoint becomes a formal boundary in the agent loop.

# 16. Checkpoint consistency

A distributed component may have:

```text
state
+
messages
+
goals
+
external effects
```

A checkpoint must define what consistency means.

For example:

```text
Checkpoint C42
│
├── state version = 19
├── processed message = 834
├── active goal = G7
└── effects committed ≤ E921
```

This prevents replaying an effect accidentally.

# 17. Exactly-once is dangerous to assume

Distributed systems often encounter:

```text
Did the command execute?
```

The sender may not know.

```text
Sender
  │
  │ command
  ▼
Actuator
  │
  X response lost
```

The sender cannot distinguish:

```text
command failed
```

from:

```text
command succeeded but acknowledgment was lost
```

Therefore NROS should model effect semantics explicitly.

# 18. Effect identity

Every externally meaningful effect should have an identity:

```text
EffectId = E921
```

The actuator or effect adapter can record:

```text
E921 → applied
```

If the same effect arrives again:

```text
E921
```

the receiver can recognize the duplicate.

This enables idempotent recovery strategies.

# 19. Effect journal

For critical operations:

```text
Effect Journal
│
├── E918 → committed
├── E919 → committed
├── E920 → committed
├── E921 → pending
└── E922 → rejected
```

Recovery can then determine what needs to be retried.

# 20. State + effect journal

The deeper recovery architecture becomes:

```text
             Checkpoint
                 │
                 ▼
        ┌──────────────────┐
        │ Recoverable State │
        └──────────────────┘
                 │
                 +
        ┌──────────────────┐
        │   Effect Journal │
        └──────────────────┘
                 │
                 ▼
             Recovery
```

This is considerably stronger than simply restoring variables.

# 21. Event sourcing perspective

NROS does not necessarily need full event sourcing, but the idea is useful.

Instead of storing only:

```text
current_state
```

the runtime can retain:

```text
events
```

such as:

```text
GoalCreated
ActivationStarted
EffectCommitted
ObservationReceived
GoalUpdated
CheckpointCreated
```

Then state can conceptually be reconstructed:

```text
Events
  ↓
State Projection
```

# 22. Execution log

A unified NROS execution record might look conceptually like:

```text
Event
│
├── event_id
├── timestamp
├── logical_clock
├── participant
├── component
├── activation
├── cause
├── payload/reference
└── parent_causality
```

This creates a common substrate for:

- diagnostics
- replay
- debugging
- auditing
- recovery
- performance analysis

# 23. Logical time

Wall-clock time alone is insufficient in distributed systems.

NROS should distinguish:

```text
WallClock
MonotonicClock
LogicalClock
SimulationClock
```

For example:

```text
timestamp:
2026-08-21T12:00:00Z
```

is useful for humans.

But causal ordering may require:

```text
logical_time = 8421
```

# 24. Simulation time

Robotics heavily relies on simulation.

A component should not need to know whether:

```text
Clock
```

comes from:

```text
real hardware
simulation
replay
test harness
```

Therefore:

```text
ClockProvider
│
├── SystemClock
├── SimClock
├── ReplayClock
└── DeterministicClock
```

# 25. Replay

Now combine:

```text
Event Log
+
Checkpoint
+
Clock
+
Input Data
```

and NROS can replay execution:

```text
Recorded run
     │
     ▼
Replay Clock
     │
     ▼
Recorded Events
     │
     ▼
NROS Runtime
```

This is much more powerful than simply replaying sensor messages.

# 26. Deterministic replay

For debugging:

```text
Run #1
   ↓
failure at Activation A817
```

NROS should ideally allow:

```text
Replay
   ↓
A817
   ↓
same inputs
same scheduling constraints
same state
   ↓
reproduce failure
```

Absolute bit-for-bit determinism may not always be achievable, but the runtime should define what determinism guarantees are available.

# 27. Recovery and replay are connected

A crashed component can potentially recover by:

```text
Checkpoint
   +
Events since checkpoint
```

Conceptually:

```text
Checkpoint C10
      │
      ▼
E101
E102
E103
E104
      │
      ▼
Recovered State
```

This avoids checkpointing every tiny state transition.

# 28. Compaction

Long-running robots cannot keep infinite history.

Therefore:

```text
Events
  ↓
Checkpoint
  ↓
Compaction
```

Old events can be archived or removed once their state is safely represented by a checkpoint.

# 29. Persistent storage abstraction

NROS should not hard-code one database.

A storage abstraction might expose:

```text
StateStore
│
├── MemoryStore
├── FileStore
├── SQLiteStore
├── EmbeddedStore
└── RemoteStore
```

The runtime depends on the abstraction.

Deployment chooses the implementation.

# 30. Embedded constraint

A microcontroller might have:

```text
RAM: 256 KB
Flash: 2 MB
```

It cannot run the same persistence system as a workstation.

Therefore NROS should define capability tiers:

```text
Persistence
│
├── None
├── Volatile
├── Local durable
├── Transactional
└── Distributed durable
```

A small embedded deployment can use the minimum required level.

# 31. Configuration is state too

ROS parameters should evolve conceptually into:

```text
Configuration
│
├── schema
├── version
├── scope
├── ownership
├── mutability
└── persistence
```

For example:

```text
robot.motion.max_velocity
```

could be:

```text
scope       = robot
mutable     = controlled
persistent  = yes
safety      = protected
```

# 32. Configuration changes

A configuration mutation should itself be observable:

```text
ConfigChanged
    │
    ├── old version
    ├── new version
    ├── actor
    └── reason
```

This makes runtime behavior auditable.

# 33. Safe configuration

Not every configuration change should be immediately accepted.

For safety-sensitive values:

```text
Requested Config
      │
      ▼
Validation
      │
      ▼
Safety Policy
      │
      ├── reject
      └── commit
```

This follows the same effect-policy architecture introduced earlier.

# 34. Recovery policy as a first-class object

A deployment could declare:

```text
RecoveryPolicy
│
├── max_restarts = 3
├── restart_window = 60s
├── checkpoint = required
├── preserve_goals = true
├── replay_inputs = true
└── failure_action = isolate
```

The runtime then has something concrete to enforce.

# 35. Health supervision

A supervisor can monitor:

```text
Component
│
├── heartbeat
├── deadline misses
├── memory
├── CPU
├── queue pressure
├── fault state
└── recovery state
```

This is not merely monitoring.

It becomes part of runtime lifecycle management.

# 36. NROS Supervisor

Conceptually:

```text
                    Supervisor
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
      Component A   Component B   Component C
          │             │             │
          ▼             ▼             ▼
        health        health        health
```

The supervisor can transition components:

```text
RUNNING
  ↓
DEGRADED
  ↓
FAILED
  ↓
RECOVERING
  ↓
RUNNING
```

# 37. But avoid a giant supervisor

The supervisor should not become another ROS Master-like central bottleneck.

Instead NROS should allow:

```text
Local Supervisor
      +
Domain Supervisor
      +
Robot Supervisor
```

with clearly defined scopes.

# 38. Recovery hierarchy

For example:

```text
Component failure
      │
      ▼
Local restart
      │
      X
      ▼
Component isolation
      │
      ▼
Domain recovery
      │
      X
      ▼
Robot-level safe state
```

This provides graduated response.

# 39. The resulting NROS state architecture

```text
                    NROS STATE
                        │
        ┌───────────────┼────────────────┐
        ▼               ▼                ▼
     Runtime         Persistent       Checkpoint
       State            State             │
        │                │                ▼
        │                │           Recovery State
        └────────────────┼────────────────┘
                         ▼
                    State Store
                         │
                         ▼
                   Event / Journal
                         │
                         ▼
                      Replay
```

# 40. The complete NROS runtime is now emerging

After the previous layers, the architecture is becoming:

```text
┌───────────────────────────────────────────────┐
│                 NROS APPLICATION              │
├───────────────────────────────────────────────┤
│ Components │ Goals │ Channels │ Services      │
├───────────────────────────────────────────────┤
│               EXECUTION                       │
│ Activations │ Scheduler │ Executor │ Effects   │
├───────────────────────────────────────────────┤
│               RESOURCES                       │
│ Capabilities │ Budgets │ Ownership │ Isolation │
├───────────────────────────────────────────────┤
│             DISTRIBUTION                      │
│ Discovery │ QoS │ Transport │ Security        │
├───────────────────────────────────────────────┤
│                STATE                          │
│ Runtime │ Persistent │ Checkpoint │ Recovery   │
├───────────────────────────────────────────────┤
│             OBSERVABILITY                     │
│ Events │ Traces │ Metrics │ Replay             │
├───────────────────────────────────────────────┤
│         OS / RTOS / MCU / HARDWARE            │
└───────────────────────────────────────────────┘
```

The important architectural consequence is that **recovery is no longer an application feature**.

It becomes a property of the runtime.

# 41. ROS → NROS: the conceptual endpoint

We can now describe the transformation much more precisely:

```text
ROS
 │
 ├── communication graph
 ├── nodes
 ├── callbacks
 ├── topics
 ├── services
 ├── actions
 └── packages
        │
        ▼
NROS
 │
 ├── execution graph
 ├── components
 ├── activations
 ├── channels
 ├── services
 ├── goals
 ├── resources
 ├── capabilities
 ├── effects
 ├── state
 ├── checkpoints
 ├── recovery
 ├── causal tracing
 └── deployment
```

That is the fundamental architectural direction.

# 42. Next: the real-time boundary

The next layer is especially important for NROS.

We need to answer:

> **What happens when an NROS component must satisfy a hard 100 µs, 1 ms, or 10 ms control deadline?**

This requires separating:

```text
NROS general runtime
```

from:

```text
NROS real-time execution
```

without splitting the programming model.

The next section should therefore define the **NROS Real-Time Execution Model**:

```text
NROS
│
├── General Execution
│
├── Soft Real-Time
│
├── Hard Real-Time Boundary
│
├── RT Executor
│
├── RT Scheduler
│
├── Memory Guarantees
│
├── Lock-Free Communication
│
├── Priority / Deadline Scheduling
│
├── Watchdogs
│
└── RT ↔ Non-RT Isolation
```

That boundary is what determines whether NROS becomes merely a modern ROS alternative—or a **serious Rust-native robotics runtime capable of spanning AI workloads, embedded control, and safety-critical execution**.

# NROS — Part XIII: Real-Time Execution Model

This is the point where NROS must draw a hard architectural boundary.

A robotics runtime can tolerate a missed deadline in:

```text
mapping
visualization
logging
AI inference
```

but potentially **cannot** tolerate one in:

```text
motor control
safety monitoring
emergency stop
servo loops
```

Therefore NROS should not treat "real-time" as a single boolean property.

# 1. Real-time is a spectrum

NROS should define execution classes:

```text
Execution Class
│
├── Best Effort
├── Soft Real-Time
├── Firm Real-Time
├── Hard Real-Time
└── Safety-Critical
```

### Best Effort

Missing a deadline is acceptable.

```text
Telemetry
Logging
Visualization
```

### Soft Real-Time

Deadline misses degrade quality.

```text
Perception
Planning
AI inference
```

### Firm Real-Time

Late results have little or no value.

```text
Trajectory update
Sensor fusion
```

### Hard Real-Time

A deadline must be guaranteed.

```text
Servo control
Safety interlock
```

### Safety-Critical

Timing guarantees are combined with formal safety constraints.

```text
Emergency stop
Safety controller
Protective motion boundary
```

# 2. The crucial rule

NROS should **never claim hard real-time merely because a task has high priority**.

This:

```text
priority = 255
```

does not prove:

```text
deadline = guaranteed
```

Real-time guarantees require control over the entire execution path.

# 3. End-to-end real-time path

Consider:

```text
Encoder
   │
   ▼
Driver
   │
   ▼
NROS RT Input
   │
   ▼
Controller
   │
   ▼
Motor Command
   │
   ▼
Driver
   │
   ▼
Motor
```

A 1 ms deadline applies to the **whole chain**, not merely:

```text
Controller.execute() = 100 µs
```

Therefore NROS needs end-to-end timing semantics.

# 4. Deadline model

An activation can carry:

```text
Deadline
│
├── absolute deadline
├── relative deadline
├── period
├── budget
└── criticality
```

Example:

```text
period   = 1 ms
deadline = 1 ms
budget   = 150 µs
```

The scheduler can reason about the activation explicitly.

# 5. Periodic execution

A control loop:

```text
0ms    1ms    2ms    3ms    4ms
│      │      │      │      │
▼      ▼      ▼      ▼      ▼
A1     A2     A3     A4     A5
```

should not depend on:

```text
sleep(1ms)
```

because ordinary sleep is not a real-time guarantee.

Instead the RT scheduler owns the periodic activation.

# 6. Jitter

Suppose the target is:

```text
period = 1ms
```

but execution occurs:

```text
1.00ms
1.03ms
0.98ms
1.07ms
1.01ms
```

The variation is jitter.

NROS should measure:

```text
release jitter
start-time jitter
completion jitter
deadline jitter
```

rather than simply reporting average latency.

# 7. Scheduler

NROS can support multiple scheduling policies:

```text
Scheduler
│
├── FIFO
├── Priority
├── EDF
├── Rate Monotonic
├── Deadline Monotonic
└── Custom
```

But the runtime should not pretend all policies provide identical guarantees.

# 8. Earliest Deadline First

For EDF:

```text
A deadline = 10ms
B deadline =  5ms
C deadline = 20ms
```

the scheduler chooses:

```text
B
```

first.

This makes deadline a scheduling primitive rather than merely diagnostic metadata.

# 9. Rate Monotonic Scheduling

For periodic tasks:

```text
Task A = 1ms
Task B = 5ms
Task C = 20ms
```

the higher-frequency task generally receives higher priority.

This is particularly relevant to control systems.

# 10. Priority inversion

Suppose:

```text
High priority task
       │
       ▼
     Mutex
       ▲
       │
Low priority task
```

The high-priority task waits for the low-priority task.

A medium-priority task can then prevent the low-priority task from running.

NROS must therefore carefully constrain synchronization in RT contexts.

# 11. RT synchronization

Preferred primitives include:

```text
SPSC queues
MPSC queues
atomic state
lock-free rings
preallocated channels
```

Blocking mutexes should be avoided in the strict RT path unless their timing characteristics are explicitly controlled.

# 12. Allocation

Dynamic allocation can introduce unpredictable latency.

Therefore:

```text
RT component
    │
    ├── no uncontrolled allocation
    ├── bounded queues
    ├── preallocated buffers
    └── deterministic memory strategy
```

A useful rule is:

> **Allocate before entering the hard real-time region.**

# 13. Rust helps—but does not magically make code real-time

Rust provides:

```text
ownership
borrowing
type safety
Send
Sync
```

but this does **not** imply:

```text
hard real-time
```

A Rust program can still contain:

```text
allocation
unbounded loops
blocking I/O
page faults
contention
OS scheduling uncertainty
```

Therefore NROS must define RT constraints independently.

# 14. RT-safe API subset

A useful architecture is:

```text
nros-core
     │
     ├── general API
     │
     └── rt API
```

The RT API should intentionally expose fewer operations.

For example:

```text
RT Component
│
├── bounded channel
├── preallocated memory
├── atomic state
├── deterministic clock
├── deadline
└── RT-safe logging
```

while preventing accidental access to:

```text
filesystem
network configuration
dynamic plugin loading
unbounded allocation
blocking operations
```

# 15. RT / non-RT separation

This is one of the most important NROS architectural patterns.

```text
┌───────────────────────┐
│      NON-RT DOMAIN    │
│                       │
│ AI / Planning / Logs  │
│ Networking / Storage  │
└───────────┬───────────┘
            │
       bounded bridge
            │
┌───────────▼───────────┐
│       RT DOMAIN       │
│                       │
│ Servo / Safety / I/O  │
└───────────────────────┘
```

The bridge must have explicit bounded semantics.

# 16. No accidental RT contamination

A common failure is:

```text
RT Controller
    │
    ▼
Logger
    │
    ▼
Disk
```

Now a hard real-time component depends indirectly on filesystem latency.

NROS should prevent this architectural pattern.

Instead:

```text
RT Controller
    │
    ▼
Bounded Event Buffer
    │
    ▼
NON-RT Logger
    │
    ▼
Disk
```

# 17. RT-safe observability

Logging from RT code is particularly dangerous.

Instead:

```text
RT code
  │
  ▼
fixed-size event
  │
  ▼
lock-free buffer
  │
  ▼
non-RT exporter
```

The RT path records minimal information.

The non-RT side performs expensive formatting.

# 18. Watchdogs

A hard real-time component needs supervision.

```text
Controller
    │
    ├── heartbeat
    ├── deadline status
    └── health state
          │
          ▼
       Watchdog
```

If the controller stops responding:

```text
Watchdog
   │
   ▼
Safe transition
```

# 19. Watchdog must itself be reliable

The watchdog cannot depend on the component it supervises.

Bad:

```text
Controller
  └── supervises itself
```

Better:

```text
Independent watchdog
        │
        ▼
Controller
```

For high-criticality systems, this may need hardware support.

# 20. Hardware boundary

NROS should be able to integrate:

```text
NROS
  │
  ▼
RTOS
  │
  ▼
MCU
  │
  ▼
Motor driver
```

or:

```text
NROS
  │
  ▼
Linux PREEMPT_RT
  │
  ▼
Industrial I/O
```

or:

```text
NROS
  │
  ▼
standard Linux
  │
  ▼
simulation
```

The logical component model should remain compatible.

# 21. RT executor

A dedicated executor can enforce stronger rules:

```text
RTExecutor
│
├── bounded work queue
├── fixed worker set
├── priority control
├── deadline tracking
├── CPU affinity
├── memory constraints
└── watchdog integration
```

This should be separate from a general-purpose executor.

# 22. General executor

Meanwhile:

```text
Executor
│
├── async tasks
├── blocking operations
├── dynamic workloads
├── I/O
└── flexible scheduling
```

The two executors can coexist.

# 23. Mixed-criticality runtime

A single robot may contain:

```text
Safety       → Hard RT
Controller   → Hard RT
Sensor fusion→ Firm RT
Perception   → Soft RT
Planner      → Soft RT
LLM agent    → Best Effort
Logging      → Best Effort
```

NROS should allow all of these within one deployment.

# 24. Criticality-aware scheduling

Conceptually:

```text
Criticality
│
├── C0 BestEffort
├── C1 SoftRT
├── C2 FirmRT
├── C3 HardRT
└── C4 Safety
```

Higher-criticality work gets stronger guarantees and isolation.

# 25. Graceful degradation

If CPU pressure increases:

```text
Normal
 │
 ▼
Resource pressure
 │
 ├── reduce visualization
 ├── reduce AI frequency
 ├── reduce perception quality
 └── preserve controller
```

The runtime should preserve high-criticality functions first.

# 26. Resource budgets

An activation can therefore carry:

```text
Budget
│
├── CPU time
├── memory
├── bandwidth
├── queue capacity
└── energy
```

Example:

```text
Vision:
CPU ≤ 20%
Bandwidth ≤ 200 MB/s
```

This turns resource management into a runtime concern.

# 27. Deadline miss policy

A missed deadline should generate a defined event:

```text
DeadlineMiss
│
├── activation
├── expected deadline
├── actual completion
├── lateness
└── criticality
```

Then policy determines what happens:

```text
DeadlineMiss
    │
    ├── record
    ├── retry
    ├── skip
    ├── degrade
    ├── isolate
    └── safe-stop
```

# 28. Freshness

For robotics, a result can be technically correct but useless because it is old.

Therefore NROS should distinguish:

```text
correctness
```

from:

```text
freshness
```

Example:

```text
velocity command
age = 250 ms
```

Even if valid mathematically, it may be unsafe operationally.

# 29. Temporal validity

Messages and effects can carry:

```text
valid_from
valid_until
```

Then:

```text
now > valid_until
```

means the data is expired.

This provides a first-class temporal safety mechanism.

# 30. Real-time channels

An RT channel should declare:

```text
RTChannel<T>
│
├── capacity
├── latency bound
├── reliability
├── freshness
├── priority
└── allocation policy
```

The runtime can then reject an invalid configuration instead of silently degrading.

# 31. RT capability declaration

A component could declare:

```text
ComponentRequirements
│
├── execution_class = HardRT
├── period = 1ms
├── deadline = 1ms
├── budget = 100µs
├── memory = preallocated
└── CPU = isolated
```

Deployment must satisfy these requirements.

# 32. Admission control

Before starting the system:

```text
Deployment
   │
   ▼
Admission Control
   │
   ├── CPU capacity
   ├── memory
   ├── deadlines
   ├── bandwidth
   └── isolation
```

If requirements cannot be satisfied:

```text
DEPLOYMENT REJECTED
```

rather than discovering the problem during operation.

# 33. This is a major difference from ordinary ROS

The traditional application model often says:

> launch everything and observe what happens.

NROS can instead say:

> **prove the deployment is admissible before entering the operational state.**

This fits the strict gate-oriented philosophy we've been developing.

# 34. Runtime state machine

The entire NROS runtime can therefore have:

```text
BOOT
 │
 ▼
DISCOVER
 │
 ▼
VALIDATE
 │
 ▼
ADMIT
 │
 ▼
INITIALIZE
 │
 ▼
RUN
 │
 ├── DEGRADED
 │
 ├── RECOVERING
 │
 └── FAULT
 │
 ▼
SAFE
 │
 ▼
SHUTDOWN
```

Critically:

```text
DISCOVER → VALIDATE → ADMIT
```

occurs before operational execution.

# 35. No state transition without evidence

This gives NROS a strong invariant:

```text
NO OBSERVED PREREQUISITE
        ↓
NO STATE TRANSITION
```

For example:

```text
RT controller requested
        │
        ▼
CPU isolation verified?
        │
        ├── NO → BLOCK
        │
        └── YES
              │
              ▼
        memory constraints verified?
```

This is precisely the kind of runtime discipline required for serious robotics.

# 36. NROS real-time architecture

We can now place the RT layer into the overall system:

```text
┌────────────────────────────────────────────┐
│              APPLICATION                  │
├────────────────────────────────────────────┤
│ Components │ Goals │ Channels │ Services   │
├────────────────────────────────────────────┤
│              EXECUTION                    │
│ Activations │ Scheduler │ Executor         │
├────────────────────────────────────────────┤
│          REAL-TIME EXECUTION              │
│ RT Scheduler │ RT Executor │ Watchdog      │
├────────────────────────────────────────────┤
│              RESOURCES                    │
│ CPU │ Memory │ Devices │ Bandwidth         │
├────────────────────────────────────────────┤
│             DISTRIBUTION                  │
│ Discovery │ QoS │ Transport │ Security     │
├────────────────────────────────────────────┤
│                STATE                      │
│ Checkpoint │ Recovery │ Journal            │
├────────────────────────────────────────────┤
│          OS / RTOS / HARDWARE             │
└────────────────────────────────────────────┘
```

# 37. The resulting NROS philosophy

At this stage, NROS is no longer simply:

```text
ROS rewritten in Rust
```

It is becoming:

> **A typed, resource-aware, causally traceable, distributed execution runtime for robotics and autonomous systems.**

ROS's fundamental insight was:

```text
robot software = distributed computation graph
```

NROS extends that into:

```text
robot software =
    distributed execution
  + communication
  + resources
  + time
  + effects
  + state
  + recovery
  + safety
```

# 38. Next architectural frontier: hardware abstraction

The next question is now unavoidable:

> **How does NROS cross the boundary from software runtime into physical hardware?**

We need a hardware model capable of representing:

```text
Sensors
Actuators
GPIO
CAN
SPI
I²C
UART
Ethernet
PCIe
USB
GPUs
FPGAs
MCUs
PLCs
industrial fieldbuses
```

without turning `nros-core` into a giant hardware-driver framework.

The next layer should therefore define:

# **NROS Hardware & Device Model**

with the architecture:

```text
NROS Component
      │
      ▼
Device Capability
      │
      ▼
Hardware Resource
      │
      ▼
Driver Interface
      │
      ▼
HAL
      │
      ▼
OS / RTOS / MCU / FPGA
      │
      ▼
Physical Device
```

The key design challenge will be preserving the same **Component → Activation → Effect → Observation** semantics all the way down to an actual motor, sensor, PLC, or FPGA.

# NROS — Part XIV: Hardware & Device Model

We now cross the most important boundary in a robotics runtime:

```text
software world
       ↓
physical world
```

ROS historically provides many hardware drivers and abstractions through its ecosystem. NROS should go one step further by making **device capabilities, ownership, timing, lifecycle, and physical effects** explicit runtime concepts.

# 1. The hardware abstraction problem

A robot may contain:

```text
Sensors
├── IMU
├── camera
├── LiDAR
├── encoder
├── force sensor
└── temperature sensor

Actuators
├── motor
├── servo
├── gripper
├── valve
└── relay

Compute
├── CPU
├── GPU
├── FPGA
└── MCU

Buses
├── CAN
├── EtherCAT
├── SPI
├── I²C
├── UART
├── USB
└── Ethernet
```

NROS should not require every component to understand the underlying hardware.

Instead:

```text
Application
     │
     ▼
NROS Device API
     │
     ▼
Device Capability
     │
     ▼
Driver
     │
     ▼
Hardware
```

# 2. Device is not merely a driver

A driver is software.

A device is a runtime resource.

Therefore:

```text
Driver
  ≠
Device
```

A device should have identity:

```text
Device
│
├── device_id
├── type
├── capabilities
├── lifecycle
├── ownership
├── health
├── timing
└── safety state
```

For example:

```text
device_id = motor.left.front
type      = BLDC
```

# 3. Device capability

Applications should depend on capabilities rather than vendor-specific implementations.

For example:

```text
VelocityActuator
PositionActuator
TorqueActuator
TemperatureSensor
ImuSensor
Camera
RangeSensor
```

Then:

```text
Controller
   │
   ▼
VelocityActuator
   │
   ├── Motor A
   ├── Motor B
   └── Motor C
```

The controller does not need to know which vendor supplied the motor.

# 4. Capability is a contract

A capability describes what can be guaranteed.

For example:

```text
VelocityActuator
│
├── command type
├── units
├── range
├── update rate
├── latency
├── safety limits
└── failure behavior
```

This turns hardware access into a typed contract.

# 5. Hardware identity

Physical identity should be stable where possible.

A device could expose:

```text
DeviceIdentity
│
├── logical_id
├── serial
├── vendor
├── model
├── firmware
└── hardware_revision
```

Example:

```text
logical_id = arm.joint.3
vendor     = ...
model      = ...
firmware   = ...
```

The logical identity allows deployment-independent software.

# 6. Logical vs physical device

This distinction is essential.

```text
Logical Device
      │
      ▼
Binding
      │
      ▼
Physical Device
```

For example:

```text
camera.front
      │
      ▼
/dev/video2
```

or:

```text
camera.front
      │
      ▼
Ethernet camera @ network endpoint
```

The application remains unchanged.

# 7. Device graph

NROS can represent hardware as a graph:

```text
Robot
│
├── Compute
│   ├── CPU
│   ├── GPU
│   └── FPGA
│
├── Sensors
│   ├── IMU
│   └── Camera
│
├── Actuators
│   ├── Motor.Left
│   └── Motor.Right
│
└── Buses
    ├── CAN
    └── Ethernet
```

This graph becomes part of deployment metadata.

# 8. Device lifecycle

Hardware is not always available.

A device should therefore have a lifecycle:

```text
UNKNOWN
   ↓
DISCOVERED
   ↓
PROBED
   ↓
INITIALIZED
   ↓
READY
   ↓
ACTIVE
   ↓
DEGRADED
   ↓
FAULT
   ↓
RECOVERING
   ↓
READY
```

This parallels the NROS component lifecycle.

# 9. Driver lifecycle

The driver itself should have lifecycle state:

```text
Driver
│
├── load
├── bind
├── initialize
├── activate
├── deactivate
├── reset
└── unload
```

The runtime should not confuse:

```text
driver loaded
```

with:

```text
device operational
```

# 10. Hardware discovery

NROS can support several discovery strategies:

```text
Discovery
│
├── static configuration
├── OS enumeration
├── bus discovery
├── network discovery
├── hardware manifest
└── explicit registration
```

For safety-critical systems, static declarations may be preferable.

For development robots, dynamic discovery is convenient.

# 11. Static hardware manifests

A robot could provide:

```text
robot.hardware.toml
```

conceptually:

```text
robot
 ├── imu.main
 ├── camera.front
 ├── motor.left
 └── motor.right
```

The runtime verifies that the actual machine matches the declared topology.

# 12. Hardware mismatch

Suppose:

```text
Expected:
motor.left = model X
```

but:

```text
Detected:
motor.left = model Y
```

NROS should not silently continue.

Possible policy:

```text
Mismatch
   │
   ├── compatible → allow
   ├── degraded  → warn/isolate
   └── incompatible → reject
```

Again:

> **No prerequisite → no state transition.**

# 13. Sensor abstraction

A sensor should expose observations.

Conceptually:

```text
Sensor<T>
   │
   ▼
Observation<T>
```

An observation should carry metadata:

```text
Observation
│
├── timestamp
├── sequence
├── frame
├── quality
├── validity
├── calibration_version
└── source_device
```

# 14. Sensor timestamps

A sensor reading isn't just:

```text
temperature = 42°C
```

It is:

```text
temperature = 42°C
timestamp   = T
source      = sensor-7
sequence    = 18392
quality     = valid
```

Timing metadata is fundamental to sensor fusion.

# 15. Hardware time

Different devices may have different clocks.

```text
Camera clock
IMU clock
CPU clock
FPGA clock
PLC clock
```

NROS should therefore support clock synchronization metadata.

```text
DeviceClock
     │
     ▼
Timestamp Mapping
     │
     ▼
NROS Clock Domain
```

# 16. Calibration

Calibration is part of device state.

For example:

```text
Camera
│
├── intrinsic calibration
├── distortion model
├── extrinsic transform
└── calibration version
```

A calibration update should be versioned.

```text
Calibration v7
      ↓
Calibration v8
```

Observations should identify which calibration generated them.

# 17. Coordinate frames

The ROS `tf` / `tf2` idea remains essential.

But NROS can model transforms as typed state:

```text
Frame
│
├── frame_id
├── parent
├── transform
├── timestamp
└── validity
```

Example:

```text
base_link
   │
   ├── imu_link
   ├── camera_link
   └── lidar_link
```

# 18. Transform authority

Multiple components should not unknowingly publish conflicting transforms.

NROS should therefore track:

```text
TransformAuthority
│
├── owner
├── source
├── validity
└── priority
```

Conflicting authorities can become a deployment error.

# 19. Actuator abstraction

Actuators are more dangerous than sensors.

A sensor produces information.

An actuator produces a physical effect.

Therefore:

```text
Sensor
    → Observation

Actuator
    → Effect
```

This connects directly to the effect model established earlier.

# 20. Actuator command

A command should carry:

```text
ActuatorCommand
│
├── effect_id
├── target
├── value
├── units
├── deadline
├── validity
├── safety constraints
└── origin
```

For example:

```text
effect_id = E1842
target    = motor.left
velocity  = 1.2 rad/s
deadline  = 1 ms
```

# 21. Actuator state

The actuator should expose feedback:

```text
ActuatorState
│
├── commanded
├── measured
├── enabled
├── fault
├── current
├── temperature
└── mode
```

This enables closed-loop control.

# 22. Command vs effect

A crucial distinction:

```text
Command
  = request

Effect
  = physical result
```

For example:

```text
Command:
"Set motor to 10 rad/s"

Effect:
motor actually reached 9.8 rad/s
```

NROS should not report the command as proof that the physical effect occurred.

# 23. Effect acknowledgement

A device can report:

```text
EffectStatus
│
├── accepted
├── executing
├── applied
├── rejected
├── expired
└── failed
```

This provides much stronger semantics than simple publish/subscribe.

# 24. Safety envelope

Actuator capabilities should expose safety limits:

```text
Motor
│
├── max_velocity
├── max_acceleration
├── max_torque
├── thermal_limit
├── current_limit
└── emergency_behavior
```

The runtime can validate commands before they reach hardware.

# 25. Safety should exist at multiple layers

```text
Application
    │
    ▼
NROS policy
    │
    ▼
Driver validation
    │
    ▼
Hardware controller
    │
    ▼
Physical safety mechanism
```

No single software layer should be assumed to be the only protection for critical motion.

# 26. Device ownership

Two components should not simultaneously control the same actuator unless explicitly supported.

Therefore:

```text
DeviceOwnership
│
├── unowned
├── reserved
├── exclusive
└── shared
```

Example:

```text
motor.left
    │
    ▼
Controller-A
```

If Controller-B requests it:

```text
REJECT
```

unless shared ownership is part of the capability contract.

# 27. Resource leasing

Ownership can be implemented as a lease:

```text
Lease
│
├── resource
├── owner
├── expiration
└── renewal
```

If a controller crashes:

```text
lease expires
      ↓
device released
      ↓
safety policy
```

This prevents abandoned control resources.

# 28. Bus abstraction

Hardware devices often share a bus.

```text
CAN
│
├── motor1
├── motor2
├── encoder1
└── BMS
```

NROS should model bus capacity:

```text
BusResource
│
├── bandwidth
├── latency
├── arbitration
├── reliability
└── failure domain
```

This allows deployment admission to include communications constraints.

# 29. CAN example

A controller might require:

```text
CAN bandwidth:
30%
```

while another component requires:

```text
50%
```

and diagnostics:

```text
30%
```

Total:

```text
110%
```

Deployment should fail admission.

This is much better than discovering saturation during operation.

# 30. FPGA as a device

NROS should not assume devices are simple peripherals.

An FPGA can expose capabilities:

```text
FPGA
│
├── encoder processing
├── PWM generation
├── filtering
├── vision pipeline
└── deterministic control
```

Applications consume capabilities rather than FPGA-specific implementation details.

# 31. MCU as a device

Likewise:

```text
Main NROS host
      │
      ▼
MCU capability
      │
      ├── motor control
      ├── safety monitor
      └── sensor acquisition
```

Communication may use:

```text
CAN
UART
Ethernet
SPI
custom transport
```

but the logical contract remains NROS-native.

# 32. PLC integration

This is especially interesting for industrial robotics.

NROS could treat a PLC as:

```text
IndustrialControlDevice
```

with capabilities such as:

```text
DigitalInput
DigitalOutput
AnalogInput
AnalogOutput
MotionControl
SafetySignal
```

Then:

```text
NROS
  │
  ▼
PLC Capability
  │
  ▼
Industrial Protocol
```

The application need not directly encode every protocol.

# 33. Industrial fieldbus

A future NROS hardware layer can accommodate:

```text
EtherCAT
PROFINET
CANopen
Modbus
OPC UA
EtherNet/IP
```

without embedding those protocols into the NROS core.

The correct architecture is:

```text
nros-core
   │
nros-device
   │
nros-driver-api
   │
protocol adapter
   │
hardware
```

# 34. Driver boundary

The driver API should be intentionally narrow.

Conceptually:

```text
Driver
│
├── probe()
├── capabilities()
├── configure()
├── start()
├── stop()
├── reset()
└── health()
```

Device-specific functionality belongs above or below this boundary as appropriate.

# 35. Driver safety

Drivers should not be trusted simply because they are part of the runtime.

A driver can fail.

Therefore:

```text
Driver
   │
   ▼
Capability boundary
   │
   ▼
Policy validation
   │
   ▼
Hardware
```

The runtime retains authority over lifecycle and permissions.

# 36. Hardware faults

A device can report:

```text
OVERHEAT
OVERCURRENT
COMMUNICATION_LOSS
POSITION_ERROR
ENCODER_FAILURE
VOLTAGE_FAULT
```

NROS should normalize these into structured fault events:

```text
DeviceFault
│
├── device
├── fault_code
├── severity
├── timestamp
├── source
└── recovery_policy
```

# 37. Fault severity

For example:

```text
INFO
WARNING
DEGRADED
FAULT
CRITICAL
EMERGENCY
```

Policy can then determine response.

```text
WARNING
   → continue

FAULT
   → isolate

CRITICAL
   → safe-state

EMERGENCY
   → immediate safety action
```

# 38. Device health

Health should not be a simple boolean.

Instead:

```text
Health
│
├── availability
├── communication
├── timing
├── thermal
├── electrical
├── functional
└── confidence
```

A device may be:

```text
available = true
functional = degraded
```

which is much more informative than:

```text
healthy = false
```

# 39. Hardware resource graph

We can now combine devices and resources:

```text
                         ROBOT
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
       COMPUTE           BUSES            DEVICES
          │                │                │
       CPU/GPU          CAN/Ethernet      Sensors
          │                │              Actuators
          │                │              PLC/FPGA
          └────────────────┼────────────────┘
                           │
                           ▼
                    NROS CAPABILITIES
```

This graph can participate in:

- scheduling
- admission
- lifecycle
- ownership
- recovery
- safety
- observability.

# 40. The complete physical execution chain

NROS can now express:

```text
Goal
 │
 ▼
Activation
 │
 ▼
Controller
 │
 ▼
Capability
 │
 ▼
Device Lease
 │
 ▼
Actuator Command
 │
 ▼
Driver
 │
 ▼
Bus
 │
 ▼
Hardware
 │
 ▼
Physical Effect
 │
 ▼
Sensor Observation
 │
 ▼
NROS Event
 │
 ▼
Controller
```

This is the real robotics feedback loop.

# 41. ROS → NROS transformation

The classical ROS model:

```text
Node
  ↓
Topic
  ↓
Node
  ↓
Driver
```

becomes:

```text
Component
  ↓
Activation
  ↓
Typed Channel
  ↓
Capability
  ↓
Device
  ↓
Effect
  ↓
Observation
  ↓
Event
```

The second model exposes significantly more semantics.

# 42. NROS hardware crate architecture

A plausible Rust workspace decomposition is:

```text
rust/
└── nros/
    ├── nros-core
    ├── nros-runtime
    ├── nros-executor
    ├── nros-rt
    ├── nros-transport
    ├── nros-device
    ├── nros-driver
    ├── nros-hardware
    ├── nros-safety
    ├── nros-state
    ├── nros-observability
    └── nros-cli
```

The important principle:

```text
nros-core
```

must remain small.

Hardware protocols should not leak into it.

# 43. Hardware capability trait

Conceptually:

```rust
pub trait Device {
    type Capability;

    fn identity(&self) -> DeviceIdentity;
    fn capabilities(&self) -> Self::Capability;
    fn health(&self) -> DeviceHealth;
}
```

The exact API should be designed later, but the architectural boundary is already clear.

# 44. Capability-oriented design

Instead of:

```rust
let motor = SomeVendorMotor::new(...);
```

NROS application code should conceptually ask for:

```rust
let motor = runtime
    .capability::<VelocityActuator>("arm.joint.3")?;
```

This gives deployment flexibility.

# 45. Hardware becomes discoverable infrastructure

A major consequence:

```text
Hardware
   ↓
Capability registry
   ↓
Runtime
```

Applications can discover:

```text
"Which devices provide PositionSensor?"
```

or:

```text
"Which actuator supports TorqueControl?"
```

This is analogous to service discovery—but for physical capabilities.

# 46. Physical capability discovery

The runtime can answer:

```text
CapabilityQuery
│
├── type
├── constraints
├── timing
├── location
├── safety class
└── availability
```

For example:

```text
Find:
TorqueActuator
range >= 10 Nm
deadline <= 1ms
```

Only compatible hardware should be returned.

# 47. This creates a powerful deployment model

The application declares:

```text
Requirements
```

The robot provides:

```text
Capabilities
```

The deployment system performs:

```text
Requirements
      +
Capabilities
      ↓
Binding
```

This is much closer to a **resource-oriented runtime** than a traditional middleware launch system.

# 48. The emerging NROS architecture

At this point:

```text
                     NROS
                      │
 ┌────────────────────┼────────────────────┐
 ▼                    ▼                    ▼
Execution         Communication          State
 │                    │                    │
 ▼                    ▼                    ▼
Activation          Channel            Checkpoint
Scheduler           Transport          Recovery
RT Executor         Discovery           Journal
 │                    │                    │
 └────────────────────┼────────────────────┘
                      ▼
                  Resources
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
       CPU           Memory        Devices
                                      │
                          ┌───────────┼───────────┐
                          ▼           ▼           ▼
                       Sensors    Actuators      PLC
```

The next problem is therefore no longer merely hardware.

It is **how all these resources communicate safely across process, machine, network, and trust boundaries**.

# Next — Part XV: NROS Communication & Transport Architecture

We should next replace the simplistic:

```text
ROS Topic → Middleware
```

with a layered NROS transport model:

```text
Application
     │
     ▼
Typed Channel
     │
     ▼
QoS / Temporal Contract
     │
     ▼
Discovery
     │
     ▼
Session
     │
     ▼
Transport
     │
 ┌───┼───────────────┐
 ▼   ▼               ▼
SHM  QUIC/TCP        UDP
 │
 ▼
CAN / EtherCAT / MCU links
```

The key question will be:

> **How can the same NROS communication model work for an in-process lock-free queue, shared memory between processes, Ethernet between machines, and deterministic industrial/embedded links—without pretending that these transports have identical semantics?**

# NROS — Part XV: Communication & Transport Architecture

We now reach the layer closest to what made ROS recognizable: **communication**.

But NROS should avoid treating communication as simply:

```text
Publisher → Middleware → Subscriber
```

That model hides too much.

For NROS, communication should be understood as a **typed, temporal, resource-aware contract** that can execute over very different physical transports.

# 1. The NROS communication stack

The proposed hierarchy is:

```text
┌───────────────────────────────────────────┐
│             Application                  │
├───────────────────────────────────────────┤
│          Typed Communication             │
│   Channel │ Request │ Goal │ Event       │
├───────────────────────────────────────────┤
│        Communication Contract            │
│ QoS │ Deadline │ Reliability │ Freshness  │
├───────────────────────────────────────────┤
│             Session Layer                │
│ Identity │ Security │ Flow Control       │
├───────────────────────────────────────────┤
│             Discovery                   │
│ Names │ Capabilities │ Endpoints         │
├───────────────────────────────────────────┤
│             Transport                   │
│ SHM │ QUIC │ TCP │ UDP │ CAN │ Custom    │
├───────────────────────────────────────────┤
│               Link                      │
│ Ethernet │ PCIe │ CAN │ SPI │ UART       │
├───────────────────────────────────────────┤
│              Hardware                   │
└───────────────────────────────────────────┘
```

The most important principle is:

> **Transport is an implementation detail; communication semantics are not.**

# 2. Channel replaces the simplistic topic abstraction

ROS topics are powerful, but NROS should make the communication contract richer.

Conceptually:

```text
Channel<T>
│
├── type
├── schema
├── producer
├── consumers
├── QoS
├── deadline
├── freshness
├── capacity
├── reliability
└── security
```

For example:

```text
Channel<ImuObservation>
```

is not merely:

```text
/imu/data
```

It is a typed communication resource.

# 3. Topic naming remains useful

NROS does not need to throw away familiar concepts.

A channel may still have:

```text
robot/sensors/imu
```

but the name should be treated as **identity**, not as the complete contract.

```text
name
+
type
+
schema
+
QoS
+
security
+
temporal contract
```

together define the communication resource.

# 4. Publish/subscribe

The basic pattern remains:

```text
Producer
   │
   ▼
Channel<T>
   │
 ┌─┴──────────┐
 ▼            ▼
Consumer A   Consumer B
```

This is appropriate for:

```text
sensor data
telemetry
state
events
detections
```

But NROS should support more than pub/sub.

# 5. Request/response

For operations requiring a result:

```text
Client
  │
  │ Request
  ▼
Service
  │
  │ Response
  ▼
Client
```

Examples:

```text
GetCalibration
ResetDevice
QueryState
ValidatePlan
```

The service contract should include:

```text
request type
response type
deadline
authorization
failure semantics
```

# 6. Goals

Long-running operations should use the goal abstraction developed earlier:

```text
Goal
│
├── accepted
├── executing
├── feedback
├── cancellation
├── completed
└── failed
```

This replaces the need to abuse services for long-running work.

# 7. Events

NROS should also support event streams:

```text
EventChannel<Event>
```

Examples:

```text
DeviceConnected
DeviceFault
GoalCreated
DeadlineMiss
CheckpointCreated
ComponentRestarted
```

Events are particularly valuable for observability and recovery.

# 8. Messages vs events

These should not be conflated.

A message generally means:

> "Here is current data."

An event means:

> "Something happened."

For example:

```text
PoseObservation
```

versus:

```text
LocalizationLost
```

The distinction makes runtime semantics clearer.

# 9. Typed messages

Every message should have a stable type identity.

Conceptually:

```text
TypeId
│
├── namespace
├── name
├── version
└── schema_hash
```

Example:

```text
nros.sensor.ImuObservation
version = 2
```

# 10. Schema evolution

Robots live for years.

A robot may receive:

```text
software version 1
```

and later:

```text
software version 2
```

Therefore communication must support schema evolution.

```text
Schema V1
    │
    ▼
Compatibility Layer
    │
    ▼
Schema V2
```

# 11. Compatibility

NROS should distinguish:

```text
exactly compatible
backward compatible
forward compatible
convertible
incompatible
```

A deployment system can reject incompatible bindings before execution.

# 12. Serialization should not be mandatory

For in-process communication:

```text
Component A
   │
   ▼
memory
   │
   ▼
Component B
```

serializing to bytes is wasteful.

NROS should therefore support:

```text
In-process
Zero-copy
Shared-memory
Serialized
```

depending on deployment.

# 13. Data representation layers

Conceptually:

```text
Typed Object
     │
     ├── InProcess representation
     │
     ├── SharedMemory representation
     │
     └── Wire representation
```

The semantic type remains the same.

# 14. Zero-copy

Large robotics data is expensive to copy.

Consider:

```text
4K camera frame
≈ several MB
```

A pipeline:

```text
Camera
 ↓ copy
Preprocessor
 ↓ copy
Detector
 ↓ copy
Planner
```

can consume enormous memory bandwidth.

NROS should support:

```text
Camera
   │
   ▼
Shared Buffer
   │
 ┌─┴────────┐
 ▼          ▼
Detector   Recorder
```

with reference-counted ownership.

# 15. Ownership becomes critical

Zero-copy does not mean:

```text
everyone can mutate memory
```

Instead:

```text
Producer
   │
   ▼
Immutable Buffer
   │
 ├── Consumer A
 ├── Consumer B
 └── Consumer C
```

Mutation requires explicit ownership transfer or a new buffer.

This maps naturally onto Rust ownership semantics.

# 16. Loaned messages

A useful NROS abstraction:

```text
Loan<T>
```

The runtime provides memory:

```text
loan()
  ↓
fill
  ↓
publish
```

Consumers receive a read-only view.

This can eliminate unnecessary allocation and copying.

# 17. Shared memory

For processes on one machine:

```text
Process A
   │
   ▼
Shared Memory Segment
   │
   ▼
Process B
```

NROS can use shared-memory transport without exposing OS-specific details to the application.

# 18. Intra-process optimization

If two components happen to run inside one process:

```text
Component A
      │
      ▼
direct queue
      │
      ▼
Component B
```

The communication semantics remain identical.

The transport implementation simply becomes cheaper.

This is an important principle:

> **Deployment topology should optimize communication, not change application semantics.**

# 19. Inter-process transport

When components are separate:

```text
Process A
   │
   ▼
IPC
   │
   ▼
Process B
```

Possible mechanisms:

```text
Unix domain sockets
shared memory
pipes
memfd
eventfd
```

NROS chooses according to contract.

# 20. Inter-machine transport

Now:

```text
Machine A
    │
 Ethernet
    │
Machine B
```

Potential transports:

```text
UDP
TCP
QUIC
DDS-compatible transport
custom deterministic transport
```

Again, the channel contract stays above the transport.

# 21. Transport selection

A deployment could express:

```text
Channel:
camera/front

Requirements:
├── bandwidth ≥ 500 Mbps
├── latency ≤ 10 ms
├── reliability = best effort
└── freshness ≤ 50 ms
```

The runtime chooses an appropriate transport.

# 22. QoS becomes a contract

Instead of vaguely configuring QoS, NROS can model:

```text
QoS
│
├── reliability
├── durability
├── history
├── capacity
├── deadline
├── lifespan
├── priority
└── ordering
```

These parameters should have explicit semantics.

# 23. Reliability

Possible modes:

```text
BEST_EFFORT
RELIABLE
```

But NROS should also distinguish:

```text
delivery reliability
```

from:

```text
application correctness
```

A reliable transport cannot guarantee that a consumer processed the message correctly.

# 24. Ordering

Possible contracts:

```text
unordered
per-producer ordering
global ordering
causal ordering
```

Global ordering is expensive and should never be assumed by default.

# 25. Causality

NROS already has an event/activation model.

Communication should therefore carry causal metadata.

Conceptually:

```text
Message
│
├── event_id
├── parent_event
├── activation_id
├── producer
└── logical_time
```

Then:

```text
Sensor
  ↓
Detection
  ↓
Plan
  ↓
Command
```

can be reconstructed as a causal chain.

# 26. Distributed tracing

This gives NROS a powerful observability model:

```text
Sensor Event
    │
    ▼
Perception Activation
    │
    ▼
Planning Activation
    │
    ▼
Control Activation
    │
    ▼
Actuator Effect
```

Every stage can share a trace identity.

# 27. End-to-end latency

Instead of measuring:

```text
callback duration = 200 µs
```

NROS can measure:

```text
sensor timestamp
      ↓
transport
      ↓
processing
      ↓
planning
      ↓
control
      ↓
actuator effect
```

and report:

```text
sensor → physical effect = 2.3 ms
```

That is far more meaningful for robotics.

# 28. Deadline propagation

A goal may have:

```text
deadline = 20 ms
```

The runtime can propagate timing constraints:

```text
Goal
 │
 ▼
Perception
 deadline = 5ms
 │
 ▼
Planning
 deadline = 10ms
 │
 ▼
Control
 deadline = 5ms
```

This creates an end-to-end temporal budget.

# 29. Backpressure

Suppose:

```text
Producer = 1000 msg/s
Consumer = 100 msg/s
```

An unlimited queue is dangerous.

NROS should support:

```text
drop-oldest
drop-newest
block
reject
sample
coalesce
degrade
```

depending on the channel contract.

# 30. Robotics-specific freshness policy

For sensor streams, the correct policy is often:

```text
latest value wins
```

rather than:

```text
process every historical value
```

For example:

```text
camera:
30 fps

controller:
10 Hz
```

The controller may only need the freshest valid frame.

NROS can explicitly model:

```text
history = latest
```

rather than requiring every message to be delivered.

# 31. Queue semantics

A channel can therefore define:

```text
capacity = 1
history = latest
freshness = 50ms
```

This creates a bounded real-time sensor channel.

# 32. Network partitions

Distributed robots may lose connectivity.

NROS should treat this as a normal state:

```text
CONNECTED
    ↓
DEGRADED
    ↓
DISCONNECTED
    ↓
RECONNECTING
    ↓
CONNECTED
```

Applications should not have to reinvent this lifecycle.

# 33. Partition policy

Different channels need different behavior.

For example:

```text
Telemetry
 → drop

Camera
 → drop stale frames

Mission Goal
 → persist

Safety command
 → local fallback
```

Therefore partition behavior belongs to the communication contract.

# 34. Locality

NROS should expose locality:

```text
LOCAL_THREAD
LOCAL_PROCESS
LOCAL_MACHINE
LOCAL_NETWORK
REMOTE_NETWORK
```

This allows policy and performance optimization.

# 35. Local-first communication

A deployment may look like:

```text
Robot
│
├── RT control
├── perception
├── planning
└── AI
```

Most communication should remain local.

Remote communication should occur only when necessary.

This reduces latency and failure surface.

# 36. Security boundary

Communication must also establish:

```text
WHO
    │
    ▼
CAN communicate with WHAT
    │
    ▼
UNDER WHICH CONDITIONS
```

Therefore each communication endpoint should have an identity.

```text
Identity
   +
Capability
   +
Authorization
```

# 37. Channel authorization

Example:

```text
motor.left.command
```

might allow:

```text
MotionController → WRITE
SafetyController → WRITE
Planner           → READ
Telemetry         → READ
AI Agent          → DENY
```

This is substantially safer than unrestricted topic access.

# 38. Encryption

For remote transport:

```text
NROS
 │
 ▼
Secure Session
 │
 ▼
Encrypted Transport
```

Possible mechanisms include:

```text
TLS
QUIC/TLS
DTLS
OS-native secure channels
```

The exact mechanism depends on deployment.

# 39. Authentication

A robot should know:

```text
who connected?
```

not merely:

```text
what IP address connected?
```

Identity can be based on:

```text
device credentials
certificates
cryptographic keys
local process identity
hardware-backed identity
```

# 40. Authorization

Authentication answers:

> Who are you?

Authorization answers:

> What may you do?

NROS should make the distinction explicit.

# 41. Transport adapters

The transport layer can be modular:

```text
nros-transport
│
├── inproc
├── shm
├── unix
├── tcp
├── udp
├── quic
├── can
├── serial
└── custom
```

Applications should depend on:

```text
Channel<T>
```

not:

```text
TcpChannel<T>
```

unless explicitly required.

# 42. Deterministic transport

For hard RT:

```text
general network stack
```

may be insufficient.

NROS should allow specialized deterministic transports:

```text
RT
 │
 ▼
Deterministic Transport
 │
 ├── EtherCAT
 ├── TSN
 ├── dedicated shared memory
 └── MCU link
```

The runtime contract remains the same while implementation changes.

# 43. Transport selection matrix

Conceptually:

| Environment | Preferred transport |
|---|---|
| Same thread | direct |
| Same process | lock-free queue |
| Same host | shared memory |
| Local control network | UDP / deterministic transport |
| General network | QUIC/TCP |
| Embedded | CAN / UART / custom |
| Industrial RT | EtherCAT / TSN-class transport |

The runtime should choose based on requirements rather than preference.

# 44. Transport independence

This gives the NROS application an important property:

```text
same component
      │
      ├── simulation → in-process
      ├── workstation → shared memory
      ├── robot LAN → Ethernet
      ├── MCU → CAN
      └── industrial cell → deterministic fieldbus
```

The component's logical contract does not change.

# 45. The communication pipeline

A complete message path becomes:

```text
Typed Data
   │
   ▼
Channel Contract
   │
   ▼
Authorization
   │
   ▼
Discovery
   │
   ▼
Session
   │
   ▼
QoS
   │
   ▼
Transport Selection
   │
   ▼
Serialization / Zero-Copy
   │
   ▼
Network / IPC
   │
   ▼
Receiver
   │
   ▼
Validation
   │
   ▼
Activation
```

This is significantly richer than a simple topic bus.

# 46. Communication failure is observable

Every communication failure should become structured state.

For example:

```text
TransportFailure
│
├── endpoint
├── channel
├── transport
├── cause
├── duration
└── recovery
```

Examples:

```text
TIMEOUT
CONNECTION_RESET
QUEUE_FULL
STALE_DATA
AUTH_FAILURE
SCHEMA_MISMATCH
BANDWIDTH_EXCEEDED
```

# 47. Communication and recovery connect

If a network disappears:

```text
Channel
   ↓
Disconnected
   ↓
Policy
   ├── reconnect
   ├── buffer
   ├── drop
   ├── switch transport
   └── safe state
```

This directly integrates with the recovery architecture from Part XII.

# 48. Communication and real-time connect

For an RT channel:

```text
deadline = 1ms
```

the runtime can reject:

```text
transport latency bound = 10ms
```

before deployment.

Thus:

```text
Communication
      +
Real-Time
      +
Admission Control
```

become one coherent system.

# 49. Communication and hardware connect

The complete chain is now:

```text
Component
    │
    ▼
Channel
    │
    ▼
Transport
    │
    ▼
Driver
    │
    ▼
Device
    │
    ▼
Physical Effect
```

This unifies software and hardware execution.

# 50. Communication and agent execution connect

An autonomous agent can now operate through exactly the same primitives:

```text
Agent
 │
 ├── observe → channels
 ├── plan → activations
 ├── execute → effects
 ├── monitor → events
 └── checkpoint → state
```

This is where NROS begins to become particularly different from conventional robotics middleware.

# 51. NROS communication architecture

The consolidated model:

```text
                     NROS COMMUNICATION
                             │
          ┌──────────────────┼──────────────────┐
          ▼                  ▼                  ▼
       DATA              CONTROL              EVENTS
          │                  │                  │
       Channel            Service              Event
          │                  │                  │
          └──────────────────┼──────────────────┘
                             ▼
                     COMMUNICATION CONTRACT
                             │
       ┌─────────────┬───────┼────────┬────────────┐
       ▼             ▼       ▼        ▼            ▼
      QoS         Deadline  Freshness Security   Ordering
                             │
                             ▼
                         SESSION
                             │
                             ▼
                        TRANSPORT
                             │
       ┌────────────┬────────┼─────────┬───────────┐
       ▼            ▼        ▼         ▼           ▼
     InProc        SHM      TCP       UDP        CAN
```

# 52. The NROS principle

The architectural rule should be:

> **One communication contract, many transport implementations.**

Not:

> One transport pretending to work for every robotics workload.

That distinction is critical.

# 53. ROS → NROS communication transformation

Traditional:

```text
Node
 │
 └── Topic
       │
       ▼
    Middleware
       │
       ▼
     Node
```

NROS:

```text
Component
 │
 ▼
Activation
 │
 ▼
Typed Channel
 │
 ├── temporal contract
 ├── QoS contract
 ├── security contract
 └── resource contract
 │
 ▼
Session
 │
 ▼
Transport
 │
 ▼
Endpoint
 │
 ▼
Activation
```

The channel becomes a **first-class typed runtime resource**.

# 54. NROS architecture after Part XV

```text
┌──────────────────────────────────────────────┐
│                  NROS                        │
├──────────────────────────────────────────────┤
│ Applications / Agents / Robotics Algorithms  │
├──────────────────────────────────────────────┤
│ Components │ Goals │ Activations │ Effects   │
├──────────────────────────────────────────────┤
│ Channels │ Services │ Events │ Discovery     │
├──────────────────────────────────────────────┤
│ QoS │ Deadlines │ Freshness │ Security       │
├──────────────────────────────────────────────┤
│ Sessions │ Flow Control │ Causality          │
├──────────────────────────────────────────────┤
│ InProc │ SHM │ IPC │ TCP │ UDP │ QUIC │ CAN │
├──────────────────────────────────────────────┤
│ Drivers │ Devices │ PLC │ MCU │ FPGA        │
├──────────────────────────────────────────────┤
│ RTOS / Linux / Embedded / Hardware           │
└──────────────────────────────────────────────┘
```

# 55. Next: Discovery & Naming

There is now a fundamental missing piece.

Before:

```text
Component A
```

can communicate with:

```text
Component B
```

NROS needs to answer:

```text
Where is B?
What does B provide?
Which version?
Which transport?
Which security policy?
Which QoS?
Is it currently available?
```

ROS1 relied heavily on the Master.

NROS should instead investigate a more sophisticated model:

```text
                    NROS Discovery
                         │
        ┌────────────────┼────────────────┐
        ▼                ▼                ▼
   Identity          Capability        Endpoint
        │                │                │
        └────────────────┼────────────────┘
                         ▼
                    Binding
                         │
             ┌───────────┼───────────┐
             ▼           ▼           ▼
          InProc        SHM        Network
```

The next part should therefore define **NROS Discovery, Naming, Identity, Service Registration, Capability Advertisement, and decentralized discovery**, including how NROS avoids recreating the ROS1 Master as a new centralized bottleneck.

# NROS — Part XVI: Discovery, Naming & Identity

The communication layer from Part XV assumes that endpoints can be found and authenticated. We now define that missing layer.

The central architectural shift is:

> **ROS1 discovery answers "where is this node?"; NROS discovery should answer "which capability exists, who owns it, where is it reachable, under which contract, and is it currently admissible?"**

## 1. Why discovery must become a first-class subsystem

In a traditional ROS-style system:

```text
Node A
   │
   ├── register
   ▼
Master
   │
   ├── locate Node B
   ▼
Node B
```

This works, but the Master becomes an important dependency.

NROS should instead separate:

```text
Identity
Discovery
Capability
Endpoint
Binding
Session
```

into explicit concepts.

# 2. NROS discovery architecture

```text
                         DISCOVERY
                            │
        ┌───────────────────┼──────────────────┐
        ▼                   ▼                  ▼
     Identity           Capability          Endpoint
        │                   │                  │
        ▼                   ▼                  ▼
      WHO?                WHAT?              WHERE?
        │                   │                  │
        └───────────────────┼──────────────────┘
                            ▼
                         Binding
                            │
                            ▼
                         Session
```

Discovery should therefore be a **control-plane operation**, not part of the data path.

# 3. Control plane vs data plane

This distinction is extremely important.

```text
CONTROL PLANE

Discovery
Identity
Authorization
Negotiation
Health
Topology
       │
       ▼
DATA PLANE

Messages
Streams
Commands
Observations
Effects
```

A temporary discovery failure should not necessarily interrupt an already-established data session.

# 4. Discovery must not carry application data

Suppose:

```text
Camera → Detector
```

Discovery establishes:

```text
Camera endpoint = X
Detector endpoint = Y
compatible contract = Z
```

Then:

```text
Camera ───────────────► Detector
```

The data should bypass the discovery service.

This preserves the decentralized ROS principle while making discovery more sophisticated.

# 5. The NROS identity model

Every runtime participant should have an identity.

Conceptually:

```text
NrosIdentity
│
├── identity_id
├── public_key
├── kind
├── issuer
├── validity
└── metadata
```

Possible identity kinds:

```text
PROCESS
DEVICE
ROBOT
MACHINE
SERVICE
AGENT
OPERATOR
```

# 6. Component identity

A component could have:

```text
component_id = robot1.navigation.planner
```

But its identity should not depend exclusively on its name.

Therefore:

```text
logical_name
    +
cryptographic_identity
```

should be separate.

# 7. Logical names

Names are for humans and configuration.

For example:

```text
/robot1/sensors/front_camera
```

Names may change between deployments.

The underlying identity should remain independently addressable.

This gives:

```text
Name
  ↓
Identity
  ↓
Endpoint
```

rather than:

```text
Name == Identity == Network Address
```

# 8. Namespaces

NROS can retain hierarchical namespaces:

```text
/robot1
/robot1/sensors
/robot1/control
/robot1/navigation
```

This preserves a useful property of ROS.

But namespaces should be treated as **logical organization**, not security boundaries.

# 9. Names are not security

This distinction must be explicit:

```text
/robot1/motor_controller
```

does not mean:

```text
trusted
```

A malicious component could claim the same name unless identity and authorization prevent it.

Therefore:

```text
Name ≠ Authority
```

# 10. Capability advertisement

A component advertises what it can provide.

For example:

```text
NavigationController
│
├── capability:
│     NavigateToPose
│
├── capability:
│     CancelNavigation
│
└── capability:
      NavigationState
```

A camera:

```text
Camera
│
├── ImageStream
├── CameraInfo
└── ExposureControl
```

# 11. Capability descriptors

A capability descriptor might contain:

```text
CapabilityDescriptor
│
├── capability_id
├── type
├── schema
├── version
├── provider
├── endpoint
├── QoS
├── timing
├── security requirements
└── lifecycle state
```

This becomes the basis of runtime binding.

# 12. Capability discovery

An application should be able to ask:

```text
Find:
ImageStream
resolution >= 1920×1080
fps >= 30
latency <= 20ms
```

Discovery returns candidates:

```text
Camera.front
Camera.rear
Camera.arm
```

The runtime then evaluates compatibility.

# 13. Requirements → capabilities

This creates a powerful matching model:

```text
Consumer Requirements
          │
          ▼
       Discovery
          │
          ▼
 Provider Capabilities
          │
          ▼
    Compatibility
          │
          ▼
        Binding
```

This can become one of NROS's most important architectural features.

# 14. Endpoint descriptor

A discovered endpoint might expose:

```text
Endpoint
│
├── identity
├── address
├── transport
├── protocol
├── capabilities
├── security
├── QoS
└── lifecycle
```

Example:

```text
transport = shm
address   = segment:abc123
```

or:

```text
transport = quic
address   = 10.0.0.42:7443
```

The application does not need to know these details.

# 15. Discovery records

A registry record could conceptually look like:

```text
DiscoveryRecord
{
    identity,
    name,
    capabilities,
    endpoints,
    contracts,
    health,
    leases,
    metadata
}
```

The record is a **description**, not a live data channel.

# 16. Registration

A participant enters the discovery system:

```text
START
  ↓
IDENTIFY
  ↓
REGISTER
  ↓
ADVERTISE
  ↓
READY
```

If registration fails:

```text
READY
```

must not be entered.

This follows NROS's strict prerequisite principle.

# 17. Leased registration

Discovery information becomes stale.

Therefore registrations should have leases:

```text
RegistrationLease
│
├── owner
├── expiration
├── renewal
└── generation
```

If a participant disappears:

```text
lease expires
     ↓
record invalidated
```

This prevents zombie endpoints.

# 18. Heartbeats

A participant can periodically renew:

```text
REGISTER
   ↓
HEARTBEAT
   ↓
HEARTBEAT
   ↓
HEARTBEAT
```

Missing heartbeats eventually produce:

```text
SUSPECTED
```

then:

```text
EXPIRED
```

rather than immediately assuming failure.

# 19. Discovery state machine

```text
UNKNOWN
   ↓
DISCOVERING
   ↓
DISCOVERED
   ↓
VERIFIED
   ↓
BOUND
   ↓
CONNECTED
   ↓
STALE
   ↓
REVALIDATING
```

A security failure should produce:

```text
REJECTED
```

rather than:

```text
CONNECTED
```

# 20. Discovery should be decentralized

NROS should avoid:

```text
                 Master
              /    |    \
             /     |     \
           A       B      C
```

as the only possible architecture.

Instead:

```text
A ◄────────► B
│            │
│            │
▼            ▼
C ◄────────► D
```

with discovery information distributed according to deployment requirements.

# 21. Discovery modes

NROS can support multiple discovery modes.

### Static

```text
configuration → endpoints
```

Best for:

- embedded
- safety-critical systems
- deterministic deployments.

### Local

```text
machine-local discovery
```

Best for:

- development
- single-host robots.

### Distributed

```text
peer ↔ peer
```

Best for:

- robot fleets
- dynamic systems.

### Managed

```text
discovery service
```

Best for:

- large deployments
- cloud/edge infrastructure.

# 22. Static discovery

A deployment manifest might specify:

```text
navigation.planner
    requires:
        /robot/sensors/laser
```

The runtime resolves this during startup.

No network discovery is required.

This is useful when determinism matters more than flexibility.

# 23. Peer-to-peer discovery

For a dynamic robot:

```text
Node A
  ↕
Node B
  ↕
Node C
```

Nodes can exchange:

```text
identity
capabilities
endpoints
health
```

without depending on a central server.

# 24. Managed discovery

A large robot fleet may use:

```text
Robot A ─┐
Robot B ─┼──► Discovery Service
Robot C ─┘
```

The service becomes a convenience and scaling mechanism rather than a fundamental dependency of the data plane.

# 25. Discovery federation

Multiple robots can have separate local discovery domains:

```text
Robot A
 └── Local Discovery

Robot B
 └── Local Discovery
```

A fleet layer can federate them:

```text
Local A ──┐
Local B ──┼──► Fleet Discovery
Local C ──┘
```

This avoids requiring every robot to know the entire world.

# 26. Discovery domains

A domain can isolate systems:

```text
Domain: factory-01
Domain: factory-02
Domain: simulation
Domain: development
```

A component normally discovers only within its authorized domain.

# 27. Discovery filtering

Queries can include:

```text
domain
namespace
capability
version
location
hardware class
QoS
security level
latency
```

For example:

```text
Find:
TorqueActuator
domain = robot-7
latency <= 1ms
safety_class >= SIL2
```

This is much stronger than simple name lookup.

# 28. Binding

Discovery produces candidates.

Binding chooses one.

```text
Requirements
    │
    ▼
Discovery
    │
    ▼
Candidates
    │
    ▼
Constraint Evaluation
    │
    ▼
Selected Provider
    │
    ▼
Session
```

Binding should therefore be explicit.

# 29. Binding should be reproducible

If multiple candidates exist:

```text
camera.front
camera.back
camera.arm
```

selection should follow deterministic rules.

For example:

```text
1. exact capability
2. security compatibility
3. timing compatibility
4. locality
5. priority
6. stable identity
```

Avoid arbitrary selection.

# 30. Binding and deployment

A deployment plan can pin bindings:

```text
planner
  → lidar.front
```

while another environment may allow:

```text
planner
  → any LaserScanProvider
```

This gives both deterministic and dynamic deployments.

# 31. Identity verification

Before binding:

```text
Discovery Record
       │
       ▼
Cryptographic verification
       │
       ├── valid → continue
       └── invalid → reject
```

A component should never trust an unverified capability advertisement.

# 32. Capability authenticity

Suppose an attacker advertises:

```text
motor.control
```

NROS must verify:

```text
Who owns this capability?
```

The capability descriptor should therefore be signed or bound to an authenticated identity.

Conceptually:

```text
Identity
   +
Capability
   +
Signature
```

# 33. Authorization after discovery

Even a genuine provider may not be authorized for a particular consumer.

Thus:

```text
Discovery
   ↓
Identity verification
   ↓
Authorization
   ↓
Binding
   ↓
Session
```

All five steps are distinct.

# 34. Endpoint migration

A device may move from:

```text
Ethernet
```

to:

```text
Shared Memory
```

because the deployment changed.

The logical capability remains:

```text
Camera.front
```

while the endpoint changes.

This is another reason not to expose transport addresses as application identities.

# 35. Rebinding

If an endpoint disappears:

```text
CONNECTED
   ↓
DISCONNECTED
   ↓
DISCOVER
   ↓
VERIFY
   ↓
REBIND
   ↓
RECONNECT
```

This can happen automatically when policy permits.

# 36. Safe rebinding

Not every resource can be rebound automatically.

For sensors:

```text
camera.front
→ camera.front.backup
```

might be acceptable.

For a motor:

```text
motor.left
→ unknown motor
```

may be completely unsafe.

Therefore capabilities should declare:

```text
rebind_policy
```

such as:

```text
NEVER
MANUAL
COMPATIBLE_ONLY
AUTOMATIC
```

# 37. Discovery and physical identity

The hardware model from Part XIV now becomes useful.

We can distinguish:

```text
Logical Capability
       │
       ▼
Device Identity
       │
       ▼
Physical Endpoint
```

Example:

```text
/robot/arm/joint3
      │
      ▼
device-id: actuator-8291
      │
      ▼
CAN node 17
```

# 38. Discovery and agents

An autonomous NROS agent can ask:

```text
What sensors are available?
```

then:

```text
Which satisfy my perception requirements?
```

then:

```text
Which are currently healthy?
```

then:

```text
Which can I access?
```

then bind dynamically.

This makes hardware capability discovery directly useful to agentic execution.

# 39. Capability graph

The runtime can maintain:

```text
                Capability Graph

Camera ─────► ImageStream
                  │
                  ▼
             Perception
                  │
                  ▼
              PoseEstimate
                  │
                  ▼
             Navigation
                  │
                  ▼
             MotorControl
                  │
                  ▼
               Motor
```

The graph describes not merely connectivity but **what the system can do**.

# 40. From computation graph to capability graph

ROS traditionally emphasizes:

```text
Node Graph
```

NROS should maintain several related graphs:

```text
1. Execution Graph
2. Communication Graph
3. Resource Graph
4. Capability Graph
5. Causal Graph
6. Trust Graph
```

This is a major conceptual expansion.

# 41. Execution graph

Answers:

> What is running?

```text
Planner
Controller
Perception
```

# 42. Communication graph

Answers:

> Who communicates with whom?

```text
Camera → Detector → Planner
```

# 43. Resource graph

Answers:

> What resources are consumed?

```text
Planner
 ├── CPU
 ├── Memory
 └── GPU
```

# 44. Capability graph

Answers:

> What can the system do?

```text
ImageStream
   ↓
ObjectDetection
   ↓
Navigation
   ↓
MotionControl
```

# 45. Causal graph

Answers:

> Why did this effect happen?

```text
SensorEvent
   ↓
Inference
   ↓
Decision
   ↓
Command
   ↓
PhysicalEffect
```

# 46. Trust graph

Answers:

> Who is allowed to cause what?

```text
Agent
  ↓
Planner
  ↓
Controller
  ↓
Motor
```

with authorization edges.

This is particularly important when autonomous agents are introduced.

# 47. Unified NROS graph model

The long-term architecture can therefore represent:

```text
                      NROS GRAPH
                           │
       ┌───────────┬───────┼────────┬──────────┐
       ▼           ▼       ▼        ▼          ▼
   Execution   Communication Resource Capability Trust
       │           │       │        │          │
       └───────────┴───────┼────────┴──────────┘
                           ▼
                         Causal
```

Rather than having one monolithic graph.

# 48. ROS1 Master → NROS Discovery evolution

The transformation can be summarized:

```text
ROS1

Master
 │
 ├── node registration
 ├── topic registration
 ├── service registration
 └── parameter coordination
```

versus:

```text
NROS

Identity
   │
Discovery
   │
Capability Registry
   │
Endpoint Resolution
   │
Authorization
   │
Binding
   │
Session
```

The latter separates responsibilities and avoids making one component responsible for everything.

# 49. What belongs in `nros-core`?

Very little.

The core should understand abstractions such as:

```text
Identity
Name
CapabilityId
TypeId
EndpointId
ChannelId
DomainId
```

But not:

```text
UDP
CAN
DDS
QUIC
EtherCAT
Linux sockets
```

Those belong in lower layers.

# 50. Proposed crate decomposition

```text
nros-core
│
├── identity
├── names
├── ids
├── capabilities
├── schemas
└── contracts

nros-discovery
│
├── registry
├── leases
├── queries
├── advertisements
├── binding
└── federation

nros-security
│
├── authentication
├── authorization
├── credentials
└── policy

nros-transport
│
├── inproc
├── shm
├── ipc
├── network
└── embedded
```

This keeps the architecture modular.

# 51. Discovery API concept

Application code should conceptually look like:

```rust
let camera = runtime
    .discover::<ImageStream>()
    .require("front")
    .min_resolution(1920, 1080)
    .max_latency(Duration::from_millis(20))
    .bind()
    .await?;
```

The exact API is not yet frozen.

The architectural point is:

```text
query
→ constraints
→ verified candidate
→ binding
```

rather than:

```text
find string
→ trust result
```

# 52. Discovery should be observable

The runtime should expose:

```text
DiscoveryMetrics
│
├── registrations
├── expirations
├── failed_auth
├── binding_failures
├── rebindings
├── stale_records
└── discovery_latency
```

This is essential for diagnosing distributed robots.

# 53. Discovery failure modes

NROS must explicitly model:

```text
NO_PROVIDER
AMBIGUOUS_PROVIDER
SCHEMA_MISMATCH
QOS_MISMATCH
SECURITY_DENIED
TIMING_INCOMPATIBLE
STALE_ENDPOINT
DOMAIN_MISMATCH
CAPABILITY_UNAVAILABLE
```

These should be structured errors rather than generic connection failures.

# 54. Admission control

Discovery and deployment now converge.

Before activation:

```text
Requirement
   ↓
Discovery
   ↓
Capability
   ↓
Security
   ↓
Resource
   ↓
Timing
   ↓
Binding
   ↓
Admission
   ↓
Activation
```

Therefore NROS can reject an impossible deployment **before physical execution begins**.

# 55. This is the critical NROS difference

ROS asks primarily:

> "Can these nodes communicate?"

NROS asks:

> "Can these participants safely, securely, temporally, and semantically cooperate under the current resource and hardware constraints?"

That is a substantially stronger runtime question.

# 56. Consolidated architecture after Part XVI

```text
                         NROS
                          │
 ┌────────────────────────┼────────────────────────┐
 ▼                        ▼                        ▼
Execution              Discovery                Resources
 │                        │                        │
 ▼                        ▼                        ▼
Agents                 Identity                  CPU
Components             Names                     Memory
Activations            Capabilities              GPU
Goals                  Endpoints                 Devices
Effects                Binding                   Buses
 │                        │                        │
 └────────────────────────┼────────────────────────┘
                          ▼
                     Communication
                          │
              ┌───────────┼───────────┐
              ▼           ▼           ▼
           Channel      Service      Event
              │
              ▼
          QoS / Timing
              │
              ▼
           Session
              │
              ▼
          Transport
              │
              ▼
          Hardware
```

# 57. Next — Part XVII: Parameters, Configuration & State

ROS's **parameter server** is another historical concept that needs to be reconsidered.

The old model:

```text
Nodes
  │
  ▼
Parameter Server
  │
  ├── robot_mass
  ├── wheel_radius
  ├── controller_gain
  └── camera_exposure
```

mixes several fundamentally different things:

```text
configuration
runtime state
calibration
secrets
parameters
persistent data
```

NROS should separate these.

The next architecture should define:

```text
                 NROS STATE MODEL

Configuration ───────┐
Calibration ─────────┤
Secrets ─────────────┤
Runtime Parameters ──┼──► Typed State
Persistent State ────┤
Checkpoints ─────────┤
Ephemeral State ─────┘
                         │
                         ▼
                  Versioned Store
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
           Local       Shared      Persistent
```

The key question will be:

> **What should replace the ROS1 Parameter Server so that configuration, mutable runtime state, calibration, secrets, and durable checkpoints have different semantics instead of being forced into one global key/value database?**
