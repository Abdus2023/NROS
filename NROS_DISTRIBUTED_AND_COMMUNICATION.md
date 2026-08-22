# NROS Distributed & Communication (Part XXV–XXVII)

The next transformation is from a **single-runtime execution model** into a **distributed robotics runtime**.

ROS was designed around networked computation graphs. NROS should preserve that strength while making **distribution, failure, identity, time, and resource locality explicit runtime concepts**.

The central principle is:

> **A distributed NROS system is one logical runtime composed of multiple execution domains, not one giant process pretending the network does not exist.**

# 1. From computation graph to runtime graph

ROS conceptually gives us:

```text
Node ── Topic ── Node
```

NROS should extend this into:

```text
                    NROS Runtime Graph

       ┌─────────────────────────────────────┐
       │             Robot A                 │
       │                                     │
       │  Component ── Component ── Agent    │
       │       │             │               │
       └───────┼─────────────┼───────────────┘
               │             │
             Network       Network
               │             │
       ┌───────┴─────────────┴───────────────┐
       │             Robot B                 │
       │                                     │
       │  Sensor ── Planner ── Controller    │
       │                                     │
       └─────────────────────────────────────┘
```

But the graph must represent more than communication.

It should also represent:

```text
identity
ownership
location
capabilities
resources
causality
time
trust
failure domains
```

# 2. Execution domains

A distributed NROS deployment should be divided into **execution domains**.

```text
ExecutionDomain
├── Runtime
├── Scheduler
├── Executor
├── Clock
├── ResourceManager
└── Transport
```

For example:

```text
Robot
 ├── MCU domain
 ├── RTOS domain
 ├── Linux domain
 └── GPU domain
```

Each domain has local guarantees.

# 3. Domain boundaries

Crossing a domain boundary is fundamentally different from a local function call.

```text
Local:

A → B

Distributed:

A
 │
 ▼
serialize
 │
 ▼
network
 │
 ▼
deserialize
 │
 ▼
B
```

Therefore NROS must expose the boundary.

# 4. Local vs remote activation

An activation may execute:

```text
LOCAL
```

or:

```text
REMOTE
```

Conceptually:

```text
Activation
   │
   ├── local executor
   │
   └── remote executor
```

The scheduling system should understand this distinction.

# 5. Remote activation

Suppose:

```text
Camera
   ↓
Vision
```

but Vision runs on another computer.

NROS can represent:

```text
Activation A
   │
   ▼
RemoteDispatch
   │
   ▼
Activation B
```

The remote activation preserves causal metadata.

# 6. Causality across machines

Example:

```text
Robot A
Camera
  │
  ▼
Detection
  │
  │ network
  ▼
Robot B
Planning
  │
  ▼
Command
```

The runtime should retain:

```text
Camera event
   ↓
Detection activation
   ↓
Planning activation
   ↓
Command effect
```

even though the work occurred on multiple machines.

# 7. Distributed trace

A distributed execution trace might look like:

```text
Robot A
t=10.000  Camera
t=10.003  Detection

Network
t=10.005  Message sent
t=10.007  Message received

Robot B
t=10.008  Planning
t=10.015  Command
```

This allows end-to-end diagnosis.

# 8. Identity hierarchy

NROS should establish explicit identities:

```text
System
 └── Domain
      └── Host
           └── Process
                └── Component
                     └── Activation
                          └── Effect
```

Each level has a distinct identity.

# 9. Why identity matters

Consider two robots running:

```text
/controller
```

The names are identical, but their identities are not.

Therefore NROS should distinguish:

```text
logical name
instance identity
deployment identity
```

For example:

```text
/controller
robot-A/controller
robot-B/controller
```

# 10. Logical names vs identities

A logical name can remain stable:

```text
navigation/planner
```

while the physical instance changes:

```text
robot-A/process-72
```

This allows service discovery and failover.

# 11. Namespaces

NROS should support hierarchical namespaces:

```text
/world
/world/robot_a
/world/robot_a/navigation
/world/robot_a/navigation/planner
```

This preserves a familiar ROS concept while making the semantics stronger.

# 12. Globally unique IDs

Names are human-oriented.

Runtime identity should use an immutable ID:

```text
ComponentId
ActivationId
EffectId
ResourceId
LeaseId
```

For example:

```text
component_id = UUID/ULID
```

The exact representation should follow NROS's existing identity design.

# 13. Discovery

A distributed system needs to answer:

> **Who exists, where are they, and what can they do?**

NROS discovery should therefore expose:

```text
Component
├── identity
├── namespace
├── location
├── capabilities
├── interfaces
├── lifecycle state
└── health
```

# 14. Capability discovery

Instead of merely discovering:

```text
planner exists
```

NROS should discover:

```text
planner
 ├── supports path planning
 ├── supports dynamic obstacles
 ├── max map size
 ├── execution profile = SOFT_RT
 └── required resources = CPU
```

This enables capability-aware orchestration.

# 15. Discovery layers

Discovery can be divided into:

```text
Local Discovery
      ↓
Domain Discovery
      ↓
Network Discovery
      ↓
Federation Discovery
```

Not every component needs global visibility.

# 16. Local discovery

Within one execution domain:

```text
Runtime
 ├── Controller
 ├── Planner
 └── Sensor
```

Discovery can be extremely cheap.

# 17. Domain discovery

Across one robot:

```text
Robot A
 ├── MCU
 ├── RTOS
 └── Linux
```

The runtime coordinates components across execution domains.

# 18. Federation

Multiple robots may form a federation:

```text
Fleet
├── Robot A
├── Robot B
├── Robot C
└── Robot D
```

But federation should not require every robot to know every internal detail.

# 19. Visibility policy

Discovery should support scopes:

```text
PRIVATE
DOMAIN
ROBOT
FLEET
GLOBAL
```

This also becomes a security boundary.

# 20. Transport abstraction

NROS should not bind the programming model to one network protocol.

Conceptually:

```rust
trait Transport {
    fn send(...);
    fn receive(...);
}
```

Possible implementations:

```text
shared memory
UDP
TCP
QUIC
serial
CAN
DDS bridge
custom embedded transport
```

The NROS semantic layer should remain independent.

# 21. Local transport

For components in one process:

```text
Component A
    │
    ▼
in-process channel
    │
    ▼
Component B
```

No serialization may be necessary.

# 22. Shared-memory transport

For high-throughput local systems:

```text
Producer
   │
   ▼
Shared Memory
   │
   ▼
Consumer
```

Useful for:

```text
camera frames
point clouds
large tensors
simulation state
```

# 23. Network transport

For distributed systems:

```text
Producer
   │
serialize
   │
network
   │
deserialize
   ▼
Consumer
```

NROS should make serialization cost visible to timing analysis.

# 24. Zero-copy

For large payloads:

```text
copy:
A → buffer1 → buffer2 → B
```

versus:

```text
zero-copy:
A ───────────────► B
        shared buffer
```

The runtime should allow transport-specific optimization without changing application semantics.

# 25. Delivery semantics

NROS should distinguish:

```text
BEST_EFFORT
AT_LEAST_ONCE
EXACTLY_ONCE*
```

The `*` is important.

Exactly-once semantics across arbitrary distributed failures are expensive and sometimes impossible to guarantee without additional assumptions.

Therefore NROS must never casually promise them.

# 26. Message durability

A message can have a lifetime:

```text
VOLATILE
SESSION
PERSISTENT
```

For example:

```text
camera frame → VOLATILE
configuration → PERSISTENT
mission state → PERSISTENT
```

# 27. Reliability vs latency

Reliable delivery often costs:

```text
acknowledgements
retries
buffers
latency
memory
```

Therefore NROS should expose the tradeoff.

A safety command may require reliability.

A high-rate sensor stream may prefer low latency.

# 28. QoS becomes a contract

Instead of treating QoS as an implementation detail:

```text
QoSContract
├── reliability
├── durability
├── deadline
├── history
├── queue depth
└── priority
```

The publisher and consumer can negotiate compatibility.

# 29. QoS compatibility

Example:

```text
Publisher:
reliable

Subscriber:
best-effort
```

Potentially compatible.

But:

```text
Publisher:
deadline = 1ms

Subscriber:
requires deadline = 100µs
```

may be incompatible.

NROS can reject an invalid connection before execution.

# 30. Backpressure across networks

Consider:

```text
Robot A
Producer
   │
   ▼
Network
   │
   ▼
Robot B
Consumer
```

If B slows down:

```text
Consumer ↓
   │
Queue ↑
   │
Network buffer ↑
```

NROS needs explicit policy.

Possible response:

```text
throttle
drop
coalesce
reject
reroute
```

# 31. Network partition

Distributed systems inevitably experience:

```text
Robot A X────X Robot B
```

The runtime must detect the partition.

But detection is not the same as knowing whether the remote system has actually failed.

Therefore:

```text
UNREACHABLE
```

should not automatically mean:

```text
DEAD
```

# 32. Failure detector

NROS can expose states:

```text
HEALTHY
DEGRADED
SUSPECTED
UNREACHABLE
RECOVERING
FAILED
```

These states should have explicit transition rules.

# 33. Leases

A powerful mechanism is a lease:

```text
Robot B grants:
Lease #92
expires at T
```

Robot A must renew it.

If renewal stops:

```text
lease expires
     ↓
ownership released
```

This is useful for:

```text
resources
leadership
locks
missions
actuator authority
```

# 34. Distributed resource ownership

Suppose two robots want:

```text
Manipulator
```

NROS can represent:

```text
Resource
   │
   ▼
Lease
   │
   ▼
Robot A
```

Robot B cannot acquire it until the lease is released or expires according to policy.

# 35. Clock synchronization

Distributed timing requires synchronized clocks.

But synchronization itself has uncertainty.

Therefore NROS should distinguish:

```text
local time
estimated remote time
synchronization uncertainty
```

# 36. Time uncertainty

Instead of pretending:

```text
Robot A time = Robot B time
```

we can model:

```text
T_B = T_A + offset ± uncertainty
```

This matters for deadline and causal reasoning.

# 37. Distributed deadline

Suppose:

```text
deadline = 10ms
```

and the work must cross:

```text
Robot A → Robot B
```

The scheduler must account for:

```text
serialization
queueing
network latency
remote scheduling
remote execution
return path
```

The deadline is therefore an end-to-end contract.

# 38. Deadline decomposition

Example:

```text
20ms total
│
├── local scheduling 2ms
├── network 3ms
├── remote scheduling 2ms
├── remote execution 8ms
└── margin 5ms
```

NROS should make these budgets observable.

# 39. Distributed scheduling

Now we have:

```text
             Global Intent
                   │
          ┌────────┴────────┐
          ▼                 ▼
      Domain A           Domain B
      Scheduler          Scheduler
          │                 │
      Executor          Executor
```

A centralized scheduler is possible, but it introduces a single coordination bottleneck.

# 40. Hierarchical scheduling

A better architecture is often:

```text
Fleet Scheduler
      │
 ┌────┼────┐
 ▼    ▼    ▼
A     B    C
│     │    │
local local local
scheduler
```

The fleet level assigns coarse work.

Each robot handles local execution.

# 41. Local autonomy

If the fleet coordinator disappears:

```text
Fleet Scheduler X
```

Robot A should ideally continue:

```text
Local Scheduler
      │
      ▼
Safe local operation
```

This is an important resilience principle.

# 42. Authority hierarchy

Distributed NROS can model:

```text
Fleet
 │
 ├── Robot
 │     ├── Domain
 │     │     └── Component
 │     │
 │     └── Resource
```

Authority should follow the hierarchy.

A remote AI should not automatically override a local safety controller.

# 43. Distributed capabilities

Capabilities should identify scope:

```text
Capability
├── subject
├── action
├── resource
├── scope
├── expiration
└── issuer
```

Example:

```text
agent-X
can-command
robot-A/arm
for 30 seconds
```

# 44. Security boundary

A distributed command path becomes:

```text
Agent
  ↓
Capability Check
  ↓
Policy
  ↓
Transport Security
  ↓
Remote Component
  ↓
Local Policy
  ↓
Actuator
```

There are multiple trust boundaries.

# 45. Defense in depth

A command should not be accepted merely because:

```text
network connection exists
```

Instead:

```text
Authenticated?
Authorized?
Capability valid?
Resource available?
Lifecycle allows it?
Safety policy allows it?
Temporal constraints valid?
```

Only then:

```text
ACCEPT
```

# 46. Distributed state

State may be:

```text
LOCAL
REPLICATED
SHARED
AUTHORITATIVE
CACHED
```

These are different semantics.

NROS should avoid pretending that all state is globally consistent.

# 47. State authority

Example:

```text
Robot A
  owns:
    /robot/A/pose
```

Robot B may have:

```text
cached copy
```

but does not own the authoritative state.

# 48. State replication

A state update can flow:

```text
Authority
   │
   ▼
Revision 42
   │
   ├── Robot B
   └── Robot C
```

Consumers know which revision they have.

# 49. Conflict handling

If two domains modify the same logical state:

```text
A → revision 43
B → revision 43
```

NROS needs an explicit consistency strategy:

```text
single authority
compare-and-swap
merge
CRDT-like structure
application-defined resolution
```

There should be no accidental conflict semantics.

# 50. Distributed transactions

NROS should avoid making every operation a distributed transaction.

Instead distinguish:

```text
local atomic operation
distributed workflow
saga/compensation
eventual consistency
```

This keeps the runtime practical.

# 51. Distributed workflow

Example:

```text
Mission Start
   │
   ├── allocate Robot A
   ├── allocate Robot B
   ├── prepare sensors
   └── activate planners
```

If Robot B fails:

```text
Mission
   │
   ▼
Compensation
   ├── release A
   ├── deactivate sensors
   └── enter safe state
```

This is workflow orchestration rather than a giant transaction.

# 52. Federation architecture

A fleet can therefore look like:

```text
                 Fleet Domain
                      │
          ┌───────────┼───────────┐
          ▼           ▼           ▼
       Robot A      Robot B      Robot C
          │           │           │
       Local         Local       Local
       Runtime       Runtime     Runtime
```

Each robot remains autonomous.

# 53. NROS federation principle

The architecture should follow:

> **Coordinate globally, execute locally, fail locally when possible.**

This reduces dependency on centralized infrastructure.

# 54. Edge/cloud integration

NROS can extend beyond robots:

```text
Robot
  │
  ▼
Edge
  │
  ▼
Cloud
```

Work can move according to:

```text
latency
compute
cost
privacy
availability
energy
```

# 55. Placement

An activation can describe placement preferences:

```text
Placement
├── locality
├── required capabilities
├── required resources
├── latency bound
├── privacy constraint
└── energy constraint
```

Then:

```text
Scheduler
   ↓
Placement Engine
   ↓
Execution Domain
```

# 56. Compute migration

An AI inference task might run:

```text
Robot GPU
```

if available, otherwise:

```text
Edge GPU
```

or:

```text
Cloud GPU
```

But only if its temporal and security contracts allow migration.

# 57. Locality-aware scheduling

Suppose:

```text
camera data = 500 MB/s
```

Moving it across the network is expensive.

The scheduler should prefer:

```text
Camera
  ↓
Local GPU
```

rather than:

```text
Camera
  ↓
Network
  ↓
Remote GPU
```

unless the remote resource provides enough benefit.

# 58. Energy-aware scheduling

For battery-powered robots:

```text
Energy
├── CPU
├── GPU
├── radio
└── sensors
```

A placement decision can trade:

```text
latency
vs
energy
```

This is especially relevant for mobile autonomous systems.

# 59. Distributed observability

NROS should provide one logical trace:

```text
Trace #A7F2
│
├── Robot A / Sensor
├── Robot A / Perception
├── Network
├── Robot B / Planner
├── Robot B / Controller
└── Robot B / Actuator
```

This is vastly more useful than isolated process logs.

# 60. Distributed runtime graph

We can now define the NROS graph as:

```text
┌─────────────────────────────────────────────────┐
│                 NROS FEDERATION                 │
│                                                 │
│  ┌───────────────┐       ┌───────────────┐      │
│  │   DOMAIN A    │       │   DOMAIN B    │      │
│  │               │       │               │      │
│  │ Components    │◄─────►│ Components    │      │
│  │ Scheduler     │       │ Scheduler     │      │
│  │ Executor      │       │ Executor      │      │
│  │ State         │       │ State         │      │
│  └───────────────┘       └───────────────┘      │
│          ▲                       ▲               │
│          │                       │               │
│       Resources               Resources          │
└─────────────────────────────────────────────────┘
```

# 61. ROS → NROS distributed transformation

The evolution is becoming clear:

```text
ROS
│
└── Computation Graph
      │
      ├── Nodes
      ├── Topics
      ├── Services
      └── Master/Discovery
```

becomes:

```text
NROS
│
└── Distributed Runtime Graph
      │
      ├── Domains
      ├── Components
      ├── Events
      ├── Activations
      ├── State
      ├── Resources
      ├── Capabilities
      ├── Schedulers
      ├── Executors
      ├── Effects
      ├── Leases
      ├── Temporal Contracts
      ├── Supervisors
      └── Federation
```

# 62. The deeper architectural shift

ROS asks:

> **Which node is connected to which node?**

NROS asks:

> **Which component owns which capability and state, where can its work execute, under what temporal/resource/security constraints, and what happens when the execution domain disappears?**

That is the distributed-runtime question.

# 63. NROS distributed invariants

We should now establish several invariants.

### Invariant 1 — Identity

```text
Every runtime entity has a stable identity.
```

### Invariant 2 — Locality

```text
Every resource and execution has an explicit locality.
```

### Invariant 3 — Authority

```text
Every authoritative state has an owner.
```

### Invariant 4 — Causality

```text
Remote execution preserves causal relationships.
```

### Invariant 5 — Failure

```text
Network failure is an explicit runtime condition.
```

### Invariant 6 — Security

```text
Remote reachability does not imply authorization.
```

### Invariant 7 — Timing

```text
Distributed timing includes communication and scheduling uncertainty.
```

# 64. The emerging NROS architecture

At this point the system can be represented as:

```text
                         NROS
                          │
             ┌────────────┴────────────┐
             │                         │
       Control Plane              Data Plane
             │                         │
       ┌─────┼─────┐             ┌─────┼─────┐
       ▼     ▼     ▼             ▼     ▼     ▼
   Discovery Policy Lifecycle   Events State Effects
       │                         │
       └──────────┬──────────────┘
                  ▼
             Activation
                  │
                  ▼
              Scheduler
                  │
                  ▼
               Executor
                  │
          ┌───────┴────────┐
          ▼                ▼
       Local             Remote
      Runtime            Runtime
```

# 65. Next: Part XXVI — NROS Communication Fabric

Now we can finally go deeper into the communication layer.

We need to define a unified semantic model for:

```text
Events
Streams
Requests
Responses
Commands
Actions
Signals
State replication
Notifications
```

and map them onto:

```text
in-process
shared memory
IPC
UDP
TCP
QUIC
CAN
serial
DDS/ROS 2 bridges
WebSocket/WebTransport
```

without allowing the transport itself to dictate the application programming model.

The key objective will be:

> **One NROS communication semantics, many transport implementations.**

That is where NROS can move beyond the traditional ROS topic/service/action split and establish a unified **typed communication fabric**.

# NROS — Part XXVI: Communication Fabric

We now arrive at one of the most important layers in the transition:

> **ROS organizes communication around topics, services, and actions. NROS should organize communication around semantic interaction patterns, then map those patterns onto different transports.**

The goal is not to eliminate publish/subscribe. It is to place it inside a more general communication model.

# 1. The NROS communication problem

ROS gives us several primitives:

```text
Topic
Service
Action
Parameter
```

NROS needs a unified model capable of expressing:

```text
Event
Stream
Request
Response
Command
Operation
State
Notification
Signal
```

without forcing every interaction into the same abstraction.

# 2. Communication taxonomy

A useful NROS model is:

```text
Communication
│
├── Event
├── Stream
├── Request/Response
├── Command
├── Operation
├── State
└── Signal
```

Each represents a different semantic contract.

# 3. Event

An event means:

> **Something happened.**

Example:

```text
ObstacleDetected
MotorFault
MissionStarted
BatteryLow
```

Model:

```text
Producer
   │
   ▼
 Event
   │
   ├── Consumer A
   ├── Consumer B
   └── Consumer C
```

The producer does not require a response.

# 4. Stream

A stream represents continuously produced data:

```text
CameraFrame
LaserScan
IMU
Odometry
Telemetry
```

Conceptually:

```text
Producer
   │
   ├── frame 1
   ├── frame 2
   ├── frame 3
   └── ...
```

Streams need explicit policies for:

```text
rate
buffering
backpressure
dropping
ordering
latency
```

# 5. Request/response

A request expects a response:

```text
Client
  │
  ▼
Request
  │
  ▼
Server
  │
  ▼
Response
```

Examples:

```text
GetMap
GetRobotState
ComputePath
QueryCapability
```

Unlike events, the interaction has a defined completion.

# 6. Command

A command means:

> **Please perform this externally meaningful operation.**

For example:

```text
MoveArm
SetVelocity
OpenGripper
StopMotor
StartMission
```

A command should carry stronger semantics than a generic request.

# 7. Command lifecycle

A command can become:

```text
CREATED
   ↓
AUTHORIZED
   ↓
ACCEPTED
   ↓
EXECUTING
   ↓
COMPLETED
```

or:

```text
REJECTED
CANCELLED
FAILED
EXPIRED
```

This aligns naturally with the NROS activation model.

# 8. Operation

An operation is a long-running interaction.

Example:

```text
NavigateTo(position)
```

which might take:

```text
seconds
minutes
hours
```

It should therefore expose:

```text
progress
state
feedback
result
cancellation
deadline
```

# 9. ROS Action → NROS Operation

ROS actions become conceptually:

```text
NROS Operation
├── request
├── acceptance
├── progress
├── feedback
├── cancellation
├── result
└── lifecycle
```

This is much closer to a general workflow primitive.

# 10. State

State is different from events.

An event says:

```text
BatteryLow
```

State says:

```text
battery.level = 18%
```

NROS should preserve that distinction.

# 11. State access patterns

State can be:

```text
READ
WRITE
WATCH
COMPARE
SUBSCRIBE
SNAPSHOT
```

For example:

```text
GET /robot/pose
```

or:

```text
WATCH /robot/pose
```

# 12. Signals

A signal is a lightweight control notification.

Examples:

```text
Pause
Resume
Wake
Shutdown
Trigger
Reset
```

Signals generally require:

```text
low latency
small payload
clear semantics
```

They need not be full data streams.

# 13. Communication as contracts

Every NROS endpoint should expose a contract:

```text
Endpoint
├── identity
├── type
├── schema
├── direction
├── QoS
├── timing
├── security
├── lifecycle
└── transport constraints
```

This makes communication discoverable.

# 14. Typed endpoints

NROS should strongly prefer typed communication:

```text
Endpoint<T>
```

where `T` has an explicit schema.

Example:

```text
Pose
├── position
├── orientation
└── timestamp
```

# 15. Schema identity

Two components should be able to determine whether their types are compatible.

For example:

```text
SchemaId
```

could represent:

```text
robotics.pose.v1
```

while the concrete encoding could vary.

# 16. Schema ≠ serialization

This distinction is critical.

Schema:

> What does the data mean?

Serialization:

> How is the data represented on the wire?

Therefore:

```text
Pose schema
   │
   ├── JSON
   ├── CDR
   ├── MessagePack
   ├── Protobuf
   └── custom binary
```

The semantic model remains the same.

# 17. Envelope

NROS should wrap payloads in a common envelope.

Conceptually:

```text
Envelope
├── message_id
├── schema_id
├── source
├── destination
├── timestamp
├── correlation_id
├── causality
├── priority
├── deadline
├── security context
└── payload
```

This creates a common metadata layer.

# 18. Message identity

Every message should have an identifier:

```text
MessageId
```

This enables:

```text
deduplication
tracing
acknowledgement
replay
correlation
```

# 19. Correlation identity

For:

```text
Request → Response
```

both should carry:

```text
CorrelationId
```

Example:

```text
Request #A12
    │
    └── correlation = C91
                      │
                      ▼
Response #B44
    └── correlation = C91
```

# 20. Causal identity

Suppose:

```text
Sensor Event
    ↓
Detection
    ↓
Planner
    ↓
Command
```

Each communication can carry:

```text
ParentActivationId
TraceId
```

allowing reconstruction of the causal chain.

# 21. Communication graph

The NROS graph can therefore show:

```text
Sensor
 │
 │ Event
 ▼
Perception
 │
 │ State
 ▼
Planner
 │
 │ Command
 ▼
Controller
 │
 │ Signal
 ▼
Actuator
```

The edges have semantics, not just names.

# 22. Topic generalization

A traditional topic can simply become:

```text
Stream<T>
```

or:

```text
EventChannel<T>
```

depending on its semantics.

This eliminates the assumption that every pub/sub channel is identical.

# 23. Subscription

A subscription becomes:

```text
Subscriber
   │
   ▼
Stream<T>
```

but the subscriber can specify:

```text
QoS
filter
deadline
priority
delivery mode
buffering
```

# 24. Content filtering

Instead of receiving everything:

```text
CameraFrame
CameraFrame
CameraFrame
...
```

a subscriber might specify:

```text
frame.timestamp > T
```

or:

```text
frame.camera == FRONT
```

or:

```text
frame.type == DEPTH
```

Filtering should ideally occur as close to the producer/transport as practical.

# 25. Event filtering

Similarly:

```text
ObstacleDetected
```

can be filtered by:

```text
distance < 1m
```

rather than delivering irrelevant events to every consumer.

# 26. Backpressure contract

Every stream should declare behavior under overload:

```text
BackpressurePolicy
├── Block
├── DropNewest
├── DropOldest
├── Sample
├── Coalesce
├── Reject
└── Escalate
```

This should be semantic, not transport-specific.

# 27. Reliability

NROS communication can expose:

```text
Reliability
├── BestEffort
└── Reliable
```

but reliability should be evaluated together with:

```text
deadline
queue depth
retries
maximum latency
```

# 28. Ordering

Ordering should also be explicit:

```text
NONE
PER_SENDER
GLOBAL
CAUSAL
```

A camera stream may need:

```text
PER_SENDER
```

while a distributed state machine might require:

```text
CAUSAL
```

# 29. Exactly-once problem

NROS should avoid making:

```text
exactly once
```

a default assumption.

A robust model is often:

```text
at-least-once delivery
+
MessageId
+
deduplication
```

This can produce effectively-once application semantics where appropriate.

# 30. Idempotent commands

For example:

```text
SetVelocity(0)
```

may be safely repeated.

But:

```text
IncrementCounter(1)
```

may not be.

NROS should allow commands to declare:

```text
idempotent = true/false
```

This affects retry policy.

# 31. Retry policy

A communication contract can define:

```text
RetryPolicy
├── max_attempts
├── backoff
├── deadline
├── retryable_errors
└── idempotency requirement
```

# 32. Request timeout

A request should carry:

```text
deadline = T
```

rather than merely:

```text
timeout = 5 seconds
```

because absolute deadlines propagate naturally through distributed systems.

# 33. Deadline propagation

Example:

```text
Client
deadline = 20ms
   │
   ▼
Service
remaining = 16ms
   │
   ▼
Worker
remaining = 11ms
```

Each downstream activation receives the remaining temporal budget.

# 34. Cancellation propagation

Similarly:

```text
Operation
   │
   ├── Request
   ├── Worker
   └── Remote Worker
```

A cancellation can propagate through the entire chain.

```text
Cancel(Operation)
       ↓
Cancel(Request)
       ↓
Cancel(Worker)
       ↓
Cancel(Remote Worker)
```

# 35. Transport-independent API

Application code should ideally express:

```rust
stream.publish(message);
```

without knowing whether the underlying transport is:

```text
shared memory
UDP
QUIC
serial
```

Transport selection belongs below the semantic API.

# 36. Transport selection

The runtime can choose based on:

```text
payload size
latency
reliability
locality
hardware
security
bandwidth
```

For example:

```text
small control command
→ UDP/QUIC

large local image
→ shared memory

MCU command
→ CAN/serial

remote management
→ secure network transport
```

# 37. Transport adapters

Architecture:

```text
             NROS API
                 │
        ┌────────┴────────┐
        ▼                 ▼
 Communication       Communication
  Semantics            Contract
        │
        ▼
 Transport Layer
        │
 ┌──────┼────────┬──────────┐
 ▼      ▼        ▼          ▼
SHM    UDP      QUIC       CAN
```

# 38. Transport capabilities

Each transport should declare capabilities:

```text
Transport
├── max payload
├── ordering
├── reliability
├── multicast
├── encryption
├── zero-copy
├── latency characteristics
└── platform support
```

The runtime can select compatible transports.

# 39. Transport negotiation

A connection could be negotiated:

```text
Endpoint A
    │
    ▼
Capability discovery
    │
    ▼
Transport selection
    │
    ▼
Connection
```

Example:

```text
same host?
 → shared memory

same machine but isolated process?
 → IPC

remote LAN?
 → QUIC/UDP

embedded link?
 → CAN
```

# 40. Multi-transport endpoints

One endpoint could support:

```text
primary:
shared memory

fallback:
network
```

This is especially useful for hybrid deployments.

# 41. Transport failure

If the primary transport fails:

```text
Shared Memory
     X
     │
     ▼
Network Transport
```

the endpoint can transition:

```text
DEGRADED
```

rather than necessarily failing entirely.

# 42. Communication lifecycle

Connections should have explicit lifecycle:

```text
DISCOVERED
   ↓
NEGOTIATING
   ↓
CONNECTED
   ↓
ACTIVE
   ↓
DEGRADED
   ↓
DISCONNECTED
```

This allows observability and recovery.

# 43. Endpoint lifecycle

Endpoints themselves can have:

```text
CREATED
REGISTERED
AVAILABLE
ACTIVE
DRAINING
UNAVAILABLE
DESTROYED
```

This is particularly useful during rolling deployments.

# 44. Zero-downtime replacement

Suppose:

```text
Planner v1
```

must be replaced.

NROS can:

```text
start Planner v2
      ↓
validate capability
      ↓
establish communication
      ↓
transfer state
      ↓
switch traffic
      ↓
drain v1
      ↓
shutdown v1
```

This becomes a runtime orchestration capability.

# 45. Communication and state

Communication should integrate with the state system:

```text
State
  │
  ├── Snapshot
  ├── Delta
  └── Watch
```

A consumer can request:

```text
current state
```

then subscribe to:

```text
future changes
```

This avoids replaying an entire history.

# 46. State synchronization

Pattern:

```text
GET Snapshot
      │
      ▼
Revision 100
      │
      ▼
WATCH
      │
      ├── revision 101
      ├── revision 102
      └── revision 103
```

This is a natural NROS synchronization primitive.

# 47. Event vs state synchronization

An event:

```text
MotorFaultOccurred
```

may be transient.

State:

```text
motor.status = FAULT
```

is persistent.

NROS applications should explicitly choose which semantic they require.

# 48. Notifications

A notification can combine:

```text
state changed
+
event metadata
```

Example:

```text
StateChanged
├── path = /robot/motor/status
├── old = READY
├── new = FAULT
└── cause = activation #881
```

This creates excellent observability.

# 49. Communication security

Every communication channel should be able to carry:

```text
Authentication
Authorization
Integrity
Confidentiality
Freshness
```

depending on its security profile.

# 50. Message freshness

For commands, old messages can be dangerous.

Therefore:

```text
Command
├── issued_at
├── expires_at
└── nonce
```

A stale command can be rejected.

# 51. Replay protection

For sensitive commands:

```text
Command #A7
```

must not be replayed maliciously.

NROS can use:

```text
MessageId
nonce
sequence number
expiration
authenticated channel
```

to enforce freshness.

# 52. Communication authorization

A component may have permission to:

```text
READ /robot/pose
```

but not:

```text
WRITE /robot/motor
```

and perhaps:

```text
COMMAND /robot/arm
```

only under a specific capability.

# 53. Communication policy

The policy layer can evaluate:

```text
Subject
   +
Action
   +
Resource
   +
Context
   ↓
Decision
```

Context may include:

```text
location
time
lifecycle
robot state
capability
network trust
```

# 54. Example command path

A remote AI requests:

```text
MoveArm(target)
```

NROS evaluates:

```text
Authenticated?
        ↓ yes
Authorized?
        ↓ yes
Capability valid?
        ↓ yes
Arm active?
        ↓ yes
Safety policy?
        ↓ yes
Deadline valid?
        ↓ yes
Resource available?
        ↓ yes
Execute
```

Only then is the actuator effect committed.

# 55. Communication observability

Every communication should produce telemetry such as:

```text
messages sent
messages received
bytes
latency
drops
retries
queue depth
deadline misses
authorization failures
transport failures
```

This should integrate directly with the distributed tracing system.

# 56. Communication trace

Example:

```text
Trace #82A1

Publisher
   │
   ├── serialize: 0.12ms
   │
   ├── network: 0.81ms
   │
   ├── deserialize: 0.09ms
   │
   ▼
Subscriber
   │
   └── schedule: 0.22ms
```

Total communication/scheduling cost:

```text
1.24ms
```

# 57. Communication graph becomes executable

The NROS graph is not merely documentation.

It can be queried:

```text
What consumes this stream?
Where is the consumer?
What transport is used?
What is the QoS?
What deadline applies?
Who authorized the connection?
What happens if the consumer disappears?
```

This is a runtime control plane.

# 58. NROS communication fabric

We can now visualize:

```text
                    NROS COMMUNICATION FABRIC

      ┌──────────────────────────────────────────┐
      │              SEMANTIC API                │
      │                                           │
      │ Event • Stream • Request • Command        │
      │ Operation • State • Signal                │
      └────────────────────┬─────────────────────┘
                           │
                           ▼
      ┌──────────────────────────────────────────┐
      │          COMMUNICATION CONTRACT           │
      │ schema • QoS • timing • security         │
      └────────────────────┬─────────────────────┘
                           │
                           ▼
      ┌──────────────────────────────────────────┐
      │            ROUTING / POLICY               │
      └────────────────────┬─────────────────────┘
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
       Local            Network          Embedded
       SHM              QUIC/UDP         CAN/Serial
```

# 59. ROS → NROS communication transformation

```text
ROS
│
├── Topic
├── Service
├── Action
└── Parameter
```

becomes:

```text
NROS
│
├── Event
├── Stream
├── Request/Response
├── Command
├── Operation
├── State
└── Signal
     │
     ▼
Communication Contract
     │
     ▼
Transport-independent routing
```

# 60. The crucial principle

NROS should not make developers ask:

> "Which transport am I using?"

They should ask:

> **"What semantic interaction does my component require?"**

The runtime then determines how that interaction is transported while respecting:

```text
semantics
timing
resources
security
locality
reliability
```

# 61. NROS is converging toward a runtime fabric

At this point the architecture is:

```text
                 ┌──────────────────┐
                 │   NROS SYSTEM    │
                 └────────┬─────────┘
                          │
       ┌──────────────────┼───────────────────┐
       ▼                  ▼                   ▼
 Communication        Execution             State
   Fabric              Runtime              Fabric
       │                  │                   │
       └──────────────┬───┴───────────────────┘
                      ▼
                 Runtime Graph
                      │
             ┌────────┴────────┐
             ▼                 ▼
         Local Domain      Remote Domain
```

The three fabrics are becoming tightly integrated:

```text
Communication
      ↕
Execution
      ↕
State
```

# 62. Next — Part XXVII: NROS State & Data Fabric

The next major layer is **state**.

We need to move beyond the ROS 1 parameter server and model:

```text
Configuration
Persistent State
Ephemeral State
Distributed State
Derived State
Snapshots
Revisions
Transactions
Subscriptions
Consistency
Conflict Resolution
Event History
Checkpoints
```

The central question becomes:

> **How does NROS represent the state of a robot, its environment, its components, and its autonomous agents as a versioned, observable, distributed state space?**

That layer will connect the communication fabric directly to **world models, robot state, configuration, memory, checkpointing, and agentic reasoning**.

# NROS — Part XXVII: State & Data Fabric

The next step is to replace the narrow ROS notion of a **parameter server** with a broader concept:

> **NROS State Fabric — a typed, versioned, observable, policy-controlled state space shared across components, execution domains, and robots.**

This is a major architectural distinction.

ROS parameters are essentially configuration-oriented shared values.

NROS needs to represent **everything whose current value influences computation or whose evolution must be observed, reproduced, synchronized, or recovered.**

# 1. From Parameter Server to State Fabric

ROS 1 gives us roughly:

```text
                    ROS Master
                       │
                Parameter Server
                 /      |      \
              Node     Node     Node
```

NROS should evolve this into:

```text
                         NROS
                          │
                    State Fabric
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
   Configuration       Runtime State     World State
        │                 │                 │
        ├── Robot        ├── Lifecycle    ├── Map
        ├── Component    ├── Health       ├── Objects
        └── Policy       ├── Resources    ├── Pose
                         └── Progress     └── Environment
```

The State Fabric becomes a first-class runtime subsystem.

# 2. State is not one thing

NROS should classify state.

```text
State
│
├── Configuration
├── Runtime
├── Operational
├── Persistent
├── Ephemeral
├── Derived
├── Distributed
├── Authoritative
├── Cached
└── Historical
```

Each category has different semantics.

# 3. Configuration state

Examples:

```text
robot.max_velocity
camera.exposure
planner.algorithm
controller.gains
network.policy
```

Configuration usually changes infrequently.

It should therefore support:

```text
validation
versioning
authorization
persistence
rollback
```

# 4. Runtime state

Runtime state changes continuously:

```text
component.state
executor.queue_depth
battery.level
motor.temperature
planner.progress
```

This state is usually ephemeral.

It may not need durable persistence.

# 5. Operational state

Operational state describes what the system is doing:

```text
mission.status = EXECUTING
navigation.status = LOCALIZING
arm.status = MOVING
```

This is especially important for autonomous systems.

# 6. Persistent state

Some state must survive process or machine restart:

```text
mission
configuration
identity
calibration
checkpoints
persistent memory
```

NROS should explicitly mark persistent state.

# 7. Ephemeral state

Other data has little value after failure:

```text
temporary buffers
executor queues
sensor frames
transient diagnostics
```

The runtime should not waste storage persisting it.

# 8. Derived state

Some state is computed from other state.

For example:

```text
velocity
```

may be derived from:

```text
position(t)
position(t-1)
timestamp
```

Likewise:

```text
battery.health
```

may be derived from multiple telemetry sources.

NROS should distinguish:

```text
source state
```

from:

```text
derived state
```

# 9. Authoritative state

Every important distributed state object should have an authority.

Example:

```text
/robot/A/pose
```

may be authoritative on:

```text
Robot A / Localization
```

Other components may hold replicas.

```text
Authority
   │
   ├── Replica B
   ├── Replica C
   └── Cache D
```

# 10. State identity

A state object needs an identity.

Conceptually:

```text
StateId
```

plus:

```text
namespace
schema
owner
version
revision
```

Example:

```text
StateObject
├── id
├── path
├── schema
├── owner
├── revision
└── value
```

# 11. Paths

Human-readable paths can organize the state space:

```text
/robot/A
/robot/A/navigation
/robot/A/navigation/pose
/robot/A/navigation/status
/robot/A/arm/joints
```

This is more powerful than an unstructured key/value store.

# 12. Typed state

A state entry should have a schema.

For example:

```text
Pose
├── position: Vector3
├── orientation: Quaternion
└── timestamp: Time
```

rather than:

```text
pose = "some arbitrary string"
```

The runtime can then validate state updates.

# 13. Schema evolution

Suppose:

```text
Pose v1
```

contains:

```text
x
y
z
```

and:

```text
Pose v2
```

adds:

```text
covariance
```

NROS should support explicit schema evolution:

```text
v1 → v2
```

rather than silently changing the meaning of the data.

# 14. State revision

Every mutation should advance a revision.

```text
revision 100
     ↓
update
     ↓
revision 101
```

Consumers can therefore detect whether they are stale.

# 15. Optimistic concurrency

Suppose:

```text
Client A reads revision 100
Client B reads revision 100
```

A updates:

```text
100 → 101
```

B attempts:

```text
100 → ?
```

NROS can reject the update:

```text
CONFLICT
expected revision = 100
actual revision = 101
```

This is much safer than silently overwriting state.

# 16. Compare-and-swap

A basic primitive can be:

```text
CAS(
    path,
    expected_revision,
    new_value
)
```

This enables safe coordination without requiring global locks.

# 17. State transactions

Some updates must happen together.

Example:

```text
robot.mode = NAVIGATION
planner.enabled = true
controller.enabled = true
```

A partial update could leave the system inconsistent.

NROS can therefore support scoped transactions:

```text
BEGIN
  update A
  update B
  update C
COMMIT
```

But transactions should remain primarily **local** unless distributed atomicity is explicitly required.

# 18. Why not global transactions?

Because distributed transactions introduce:

```text
coordination
latency
failure modes
blocking
complexity
```

NROS should prefer:

```text
local atomicity
+
distributed workflows
+
compensation
```

where possible.

# 19. State watchers

Consumers can subscribe to state changes:

```text
WATCH /robot/A/navigation
```

and receive:

```text
revision 41
revision 42
revision 43
```

This turns state into an observable runtime resource.

# 20. Snapshot

A consumer may request:

```text
SNAPSHOT /robot/A
```

and receive:

```text
revision = 1000
state = {...}
```

This provides a consistent starting point for synchronization.

# 21. Snapshot + stream

The strongest synchronization pattern is:

```text
SNAPSHOT
   │
   ▼
revision 1000
   │
   ▼
WATCH
   │
   ├── 1001
   ├── 1002
   └── 1003
```

This avoids requiring consumers to replay the entire history.

# 22. State history

For important state:

```text
State
 │
 ├── revision 1
 ├── revision 2
 ├── revision 3
 └── ...
```

The history can become an audit trail.

This is particularly valuable for:

```text
missions
configuration
safety state
agent decisions
resource ownership
```

# 23. Event-sourced state

Some NROS subsystems could reconstruct state from events:

```text
Event 1
Event 2
Event 3
   ↓
State
```

For example:

```text
MissionCreated
MissionStarted
WaypointReached
MissionPaused
MissionCompleted
```

The current mission state can be derived from the event sequence.

# 24. But not everything should be event-sourced

High-frequency sensor streams are a poor candidate for indefinite event sourcing.

For example:

```text
IMU @ 1 kHz
```

would generate enormous histories.

Therefore NROS should support:

```text
streaming
snapshots
bounded history
event sourcing
```

as separate policies.

# 25. Retention policies

State history can have:

```text
Retention
├── None
├── Last N
├── Time-based
├── Size-based
└── Permanent
```

Example:

```text
diagnostics → 24h
mission history → permanent
camera frames → 0
```

# 26. State checkpointing

This becomes especially important for NROS agents.

An agent may have:

```text
Goal
Plan
Working memory
Environment model
Current step
Pending actions
```

A checkpoint can capture:

```text
AgentCheckpoint
├── identity
├── goal
├── plan
├── state revision
├── memory references
├── pending operations
└── execution context
```

# 27. Recovery

After a crash:

```text
Agent
  X
  │
  ▼
Checkpoint
  │
  ▼
Restore
  │
  ▼
Validate
  │
  ▼
Resume
```

This is fundamentally different from simply restarting a ROS node.

# 28. Agentic state

This is where NROS begins to diverge substantially from traditional ROS.

A conventional node primarily has:

```text
input
processing
output
```

An NROS agent may have:

```text
Observe
   ↓
State
   ↓
Plan
   ↓
Execute
   ↓
Reflect
   ↓
Checkpoint
```

State therefore becomes part of the execution model itself.

# 29. World state

An autonomous robot needs more than internal state.

It needs a representation of the world:

```text
World
├── objects
├── locations
├── obstacles
├── agents
├── tasks
├── environment
└── uncertainty
```

NROS can expose this through typed state objects.

# 30. State and uncertainty

Robotics state is rarely exact.

Instead of:

```text
pose = X
```

we may need:

```text
PoseEstimate
├── mean
├── covariance
├── timestamp
└── source
```

This makes uncertainty part of the data contract.

# 31. Provenance

State should ideally identify where it came from.

For example:

```text
pose
├── value
├── source = lidar_localization
├── timestamp
├── confidence
└── revision
```

This is critical when multiple sensors or agents produce competing information.

# 32. Provenance chain

A state value could trace:

```text
Camera
   ↓
Detector
   ↓
Tracker
   ↓
World Model
```

NROS can preserve:

```text
source
parent event
activation
timestamp
```

This connects state directly to the distributed trace.

# 33. State quality

NROS can expose a quality model:

```text
StateQuality
├── freshness
├── confidence
├── provenance
├── completeness
└── consistency
```

An autonomous planner can then make better decisions.

# 34. Freshness

A state value may be valid but stale.

Example:

```text
pose timestamp = 5 seconds ago
```

The value exists, but it may no longer be suitable for control.

Therefore:

```text
valid ≠ fresh
```

This distinction should exist at runtime level.

# 35. State validity

A state object can carry:

```text
VALID
STALE
UNKNOWN
INVALID
CONFLICTED
UNAVAILABLE
```

This is much more expressive than `null`.

# 36. Distributed state synchronization

Suppose:

```text
Robot A
  authoritative map
       │
       ▼
Robot B
  replica
```

NROS can synchronize:

```text
revision
delta
checksum
timestamp
```

instead of transmitting the entire map every time.

# 37. Delta synchronization

Instead of:

```text
10 MB map
```

every update:

```text
Map v100
   ↓
Delta v101
```

The replica reconstructs:

```text
Map v101
```

This dramatically reduces network utilization.

# 38. Conflict resolution

For replicated state:

```text
A revision 42
B revision 42
```

both may change independently.

NROS must define the strategy:

```text
single authority
last-writer-wins
vector/version comparison
merge function
CRDT-like structure
application-defined resolver
```

There is no universal answer.

# 39. Authority transfer

Authority itself may migrate:

```text
Robot A
   │
   │ authority
   ▼
World Model
```

then:

```text
Robot A
   │
   ▼
handoff
   │
   ▼
Robot B
```

This should be represented as an explicit protocol.

# 40. Authority transfer protocol

Conceptually:

```text
REQUEST_HANDOFF
       ↓
VALIDATE
       ↓
FREEZE
       ↓
SNAPSHOT
       ↓
TRANSFER
       ↓
CONFIRM
       ↓
NEW_AUTHORITY
```

Only then should the old authority release ownership.

# 41. Leases and state authority

This connects directly to the previous distributed-runtime section.

```text
State Authority
       │
       ▼
Lease
       │
       ├── expires
       ├── renews
       └── transfers
```

A lease prevents two nodes from simultaneously believing they own the same mutable state.

# 42. State security

Not every state value should be visible to every component.

Example:

```text
PUBLIC
/robot/pose
```

versus:

```text
RESTRICTED
/robot/security/credentials
```

and:

```text
HIGHLY_RESTRICTED
/robot/safety/internal
```

State access should therefore pass through policy.

# 43. Capability-based state access

A capability might grant:

```text
READ /robot/A/pose
```

or:

```text
WRITE /robot/A/navigation/goal
```

or:

```text
WATCH /fleet/*
```

This integrates state with NROS's authorization model.

# 44. State API

Conceptually:

```rust
state.get(path)
state.set(path, value)
state.update(path, operation)
state.watch(path)
state.snapshot(scope)
state.compare_and_set(...)
```

The exact API can evolve independently of storage.

# 45. Storage backends

The State Fabric should not require one storage engine.

Possible backends:

```text
InMemory
SharedMemory
EmbeddedKV
SQLite/LibSQL
DistributedKV
ObjectStore
EventLog
```

The semantic layer remains stable.

# 46. Storage locality

State can exist at different scopes:

```text
Process-local
Domain-local
Robot-local
Fleet-local
Cloud
```

The runtime determines synchronization requirements.

# 47. State placement

A high-frequency value:

```text
motor.velocity
```

belongs close to the controller.

A mission record:

```text
mission.history
```

may belong in durable storage.

A fleet-wide map:

```text
fleet.map
```

may be replicated.

# 48. State tiering

This suggests:

```text
                 State Fabric
                      │
       ┌──────────────┼──────────────┐
       ▼              ▼              ▼
      HOT            WARM           COLD
       │              │              │
    RAM/SHM        Local DB       Durable Store
```

This can optimize both latency and durability.

# 49. State-plane architecture

We can now model:

```text
                    NROS STATE FABRIC

                         API
                          │
             ┌────────────┴────────────┐
             │                         │
          State API                Watch API
             │                         │
             └────────────┬────────────┘
                          ▼
                   State Coordinator
                          │
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
       Authority       Versioning      Policy
          │               │               │
          └───────────────┼───────────────┘
                          ▼
                     State Store
                          │
            ┌─────────────┼─────────────┐
            ▼             ▼             ▼
          Memory        Local DB      Remote Replica
```

# 50. The three NROS fabrics

We now have three major architectural fabrics:

```text
                 NROS
                  │
     ┌────────────┼────────────┐
     ▼            ▼            ▼
Communication  Execution      State
   Fabric       Fabric        Fabric
     │            │            │
     └────────────┼────────────┘
                  ▼
             Runtime Graph
```

And they are not independent.

# 51. Communication ↔ State

State changes produce communication:

```text
State Update
     ↓
Event
     ↓
Subscribers
```

Communication can also modify state:

```text
Command
   ↓
Activation
   ↓
State Update
```

# 52. State ↔ Execution

Execution consumes state:

```text
Scheduler
   ↓
read:
deadline
resources
priority
state
```

Execution modifies state:

```text
Activation
   ↓
state transition
```

# 53. Communication ↔ Execution

Communication triggers execution:

```text
Event
 ↓
Activation
```

Execution produces communication:

```text
Activation
 ↓
Event / Stream / Command
```

Therefore:

```text
Communication
      ↕
Execution
      ↕
State
```

forms the core NROS runtime loop.

# 54. The NROS runtime loop

The architecture is now approaching:

```text
             ┌───────────────┐
             │     State     │
             └───────┬───────┘
                     │
                     ▼
             ┌───────────────┐
             │   Observe     │
             └───────┬───────┘
                     │
                     ▼
             ┌───────────────┐
             │     Plan      │
             └───────┬───────┘
                     │
                     ▼
             ┌───────────────┐
             │    Schedule   │
             └───────┬───────┘
                     │
                     ▼
             ┌───────────────┐
             │    Execute    │
             └───────┬───────┘
                     │
                     ▼
             ┌───────────────┐
             │     Effect    │
             └───────┬───────┘
                     │
                     ▼
             ┌───────────────┐
             │    Reflect    │
             └───────┬───────┘
                     │
                     └──────────────► State
```

This is where NROS begins to become an **agentic robotics runtime**, rather than merely a redesigned ROS middleware.

# 55. ROS → NROS evolution so far

```text
ROS
│
├── Nodes
├── Topics
├── Services
├── Actions
├── Parameters
├── Master/Discovery
└── Packages
```

NROS:

```text
NROS
│
├── Components
├── Activations
├── Events
├── Streams
├── Requests
├── Commands
├── Operations
├── State
├── Resources
├── Capabilities
├── Policies
├── Schedulers
├── Executors
├── Supervisors
├── Leases
├── Distributed Domains
├── Tracing
├── Checkpoints
└── Federation
```

# 56. The fundamental abstraction

The smallest meaningful NROS unit is increasingly looking like:

```text
Component
   │
   ├── Identity
   ├── State
   ├── Capabilities
   ├── Interfaces
   ├── Resources
   ├── Lifecycle
   └── Execution
```

Rather than the ROS-centric:

```text
Node
   │
   ├── Publishers
   ├── Subscribers
   ├── Services
   └── Parameters
```

# 57. Next — Part XXVIII: NROS Component Model

The next layer should formalize the **Component** itself.

We need to answer:

```text
What is a component?
What is its lifecycle?
What resources does it own?
What state does it expose?
What capabilities does it provide?
How does it spawn activations?
How does it interact with agents?
How does it fail?
How is it supervised?
How is it upgraded?
```

The target abstraction becomes:

```text
                 NROS Component
                       │
       ┌───────────────┼────────────────┐
       ▼               ▼                ▼
    Identity         State          Capabilities
       │               │                │
       └───────────────┼────────────────┘
                       ▼
                    Runtime
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
         Activations         Resources
             │                   │
             └─────────┬─────────┘
                       ▼
                    Effects
```

That component model is the bridge between **ROS nodes** and the deeper NROS concept of **autonomous runtime entities**.
