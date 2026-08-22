# NROS Core Concepts (Part II–X)

NROS can make the computational unit richer:

```text
                    Component
                       │
       ┌───────────────┼────────────────┐
       │               │                │
      Input          State            Output
       │               │                │
       ▼               ▼                ▼
    Channel         State Store       Channel
       │                                │
       └──────────────┐    ┌────────────┘
                      ▼    ▼
                    Logic
                      │
              ┌───────┴───────┐
              ▼               ▼
           Request          Action
```

A component therefore has an explicit **contract**.

For example:

```text
Component: lidar_processor

Inputs:
    /lidar/raw : LaserScan

Outputs:
    /lidar/obstacles : ObstacleSet

State:
    calibration
    filtering
    diagnostics

Execution:
    periodic / event-driven

Constraints:
    period = 10 ms
    deadline = 5 ms
    memory = bounded
```

That last part is where NROS can substantially diverge from traditional ROS.

# 3. Execution becomes a first-class concept

Instead of:

```text
message arrives
       ↓
callback executes
```

NROS can model:

```text
Event
  ↓
Activation
  ↓
Scheduling
  ↓
Execution
  ↓
Completion
  ↓
Observation
```

For example:

```text
Sensor Event
     │
     ▼
Activation
     │
     ├── priority
     ├── deadline
     ├── budget
     ├── affinity
     └── execution class
     │
     ▼
Scheduler
     │
     ▼
Component
     │
     ▼
Result
```

This gives NROS a much more explicit execution semantics.

# 4. Execution classes

One possible NROS model is to classify work:

```text
ExecutionClass
│
├── RealTime
│
├── Periodic
│
├── EventDriven
│
├── Background
│
└── BestEffort
```

For example:

```text
MotorController
    → RealTime

SensorFusion
    → Periodic

ObstacleDetector
    → EventDriven

MapPersistence
    → Background

Diagnostics
    → BestEffort
```

The runtime can then reason about these workloads differently.

# 5. Channel semantics

A ROS topic is primarily a named communication mechanism.

NROS can make the **channel contract** explicit.

```text
Channel<T>
│
├── Type<T>
├── Capacity
├── Ordering
├── Reliability
├── Delivery
├── Ownership
├── Backpressure
└── QoS
```

For example:

```text
Channel<JointCommand>
    capacity = 32
    ordering = FIFO
    reliability = reliable
    delivery = latest
```

Or:

```text
Channel<CameraFrame>
    capacity = 2
    ordering = FIFO
    reliability = best_effort
    delivery = drop_old
```

This is especially valuable for high-rate sensor data.

# 6. Backpressure

This is a major systems-level difference.

Imagine:

```text
Camera
  │
  │ 60 FPS
  ▼
Vision Processor
  │
  │ only 20 FPS
  ▼
Planner
```

If the producer generates data faster than consumers process it, NROS needs an explicit policy.

Possible policies:

```text
Backpressure
│
├── Block
├── DropNewest
├── DropOldest
├── LatestOnly
├── Buffer
└── Reject
```

That policy should be part of the communication contract.

# 7. Ownership

Rust introduces another important dimension.

Instead of treating every message as an abstract serialized object:

```text
Publisher
    │
    ▼
serialize
    │
    ▼
transport
    │
    ▼
deserialize
    │
    ▼
Subscriber
```

NROS can support multiple transport modes:

```text
Message<T>
    │
    ├── Copy
    ├── Move
    ├── Shared
    ├── Borrowed
    ├── ZeroCopy
    └── Serialized
```

This becomes particularly important for:

- cameras
- LiDAR
- point clouds
- GPU buffers
- shared-memory systems
- embedded systems

# 8. Local vs distributed communication

NROS should not necessarily force every communication path through networking.

The runtime can select the appropriate transport:

```text
                    NROS Channel
                         │
             ┌───────────┼───────────┐
             │           │           │
             ▼           ▼           ▼
          InProc      SharedMem     Network
```

For example:

```text
Component A ──┐
              │
              ▼
         same process
              │
          In-process
              │
              ▼
Component B
```

versus:

```text
Robot A                         Robot B

Component ── Network Channel ── Component
```

The programming model remains the same.

That is a powerful abstraction.

# 9. Runtime graph

The ROS graph primarily describes communication relationships.

NROS can make the runtime graph richer:

```text
                      Runtime Graph
                           │
       ┌───────────────────┼───────────────────┐
       │                   │                   │
   Components           Channels           Resources
       │                   │                   │
       ├── lifecycle       ├── QoS             ├── CPU
       ├── state           ├── capacity        ├── memory
       ├── execution       ├── transport       ├── device
       └── health          └── type            └── network
```

This transforms the graph from a visualization concept into an **executable runtime model**.

# 10. Lifecycle

ROS 2 introduced managed lifecycle nodes.

NROS can push the concept further:

```text
                 CREATED
                    │
                    ▼
                 CONFIGURED
                    │
                    ▼
                  READY
                    │
                    ▼
                 RUNNING
                  /    \
                 /      \
                ▼        ▼
             PAUSED    ERROR
                │        │
                └────┬───┘
                     ▼
                  STOPPED
```

But the state machine should be associated with the **component contract**, not merely process management.

A component might have:

```text
Configuration
    ↓
Resource acquisition
    ↓
Validation
    ↓
Activation
    ↓
Execution
    ↓
Deactivation
    ↓
Resource release
```

That is much closer to industrial runtime semantics.

# 11. Fault model

Another important opportunity is making failure explicit.

Instead of:

```text
Node crashed
```

NROS can distinguish:

```text
Failure
│
├── ComponentFailure
├── ChannelFailure
├── TransportFailure
├── DeadlineMiss
├── ResourceExhaustion
├── InvalidMessage
├── HardwareFailure
└── RuntimeFailure
```

Then the runtime can define policies:

```text
Failure
   │
   ├── Retry
   ├── Restart
   ├── Isolate
   ├── Degrade
   ├── FailSafe
   └── Shutdown
```

This becomes critical when NROS moves from research robots toward real autonomous systems.

# 12. The emerging NROS architecture

Putting the pieces together:

```text
┌─────────────────────────────────────────────────────┐
│                  NROS APPLICATION                   │
├─────────────────────────────────────────────────────┤
│ Components │ State │ Actions │ Services │ Policies │
├─────────────────────────────────────────────────────┤
│              NROS EXECUTION MODEL                   │
│ Activation │ Scheduling │ Deadlines │ Priorities   │
├─────────────────────────────────────────────────────┤
│              NROS COMMUNICATION                     │
│ Channels │ QoS │ Backpressure │ Ownership │ Types  │
├─────────────────────────────────────────────────────┤
│                NROS RUNTIME                         │
│ Lifecycle │ Discovery │ Resources │ Faults │ Graph │
├─────────────────────────────────────────────────────┤
│               TRANSPORT ABSTRACTION                 │
│ InProc │ Shared Memory │ UDP │ DDS │ Custom         │
├─────────────────────────────────────────────────────┤
│                  RUST CORE                           │
│ Ownership │ Type Safety │ Concurrency │ no_std      │
├─────────────────────────────────────────────────────┤
│                 HOST / RTOS                          │
├─────────────────────────────────────────────────────┤
│                 HARDWARE                             │
└─────────────────────────────────────────────────────┘
```

This is where **NROS starts becoming something more ambitious than a ROS implementation**.

The next architectural question is the most fundamental one:

> **What should NROS preserve from ROS's graph/message model, and what should it replace with a Rust-native execution model?**

That leads directly into **NROS Core: the kernel/runtime boundary, crate architecture, and the minimum primitives that must exist before higher-level robotics functionality can be built.**

# NROS — Part III: The Core Runtime Boundary

The next layer is the **NROS Core**.

If ROS is viewed as a large ecosystem, NROS should avoid making the entire ecosystem part of its kernel. The core should be **small, explicit, strongly typed, and replaceable**.

A useful principle is:

> **NROS Core defines semantics; transports, applications, tooling, and integrations implement those semantics.**

## 1. The NROS Core boundary

```text
┌──────────────────────────────────────────────────────────┐
│                    NROS ECOSYSTEM                         │
│                                                          │
│  Navigation │ Perception │ Planning │ Simulation │ AI   │
└──────────────────────────┬───────────────────────────────┘
                           │
                    NROS public API
                           │
┌──────────────────────────▼───────────────────────────────┐
│                     NROS CORE                             │
│                                                          │
│  Types │ Channels │ Events │ Components │ Lifecycle      │
│  Runtime │ Scheduling │ Services │ Actions │ Errors      │
└──────────────────────────┬───────────────────────────────┘
                           │
                     Runtime traits
                           │
┌──────────────────────────▼───────────────────────────────┐
│                   NROS PLATFORM                           │
│                                                          │
│  Threads │ Tasks │ Timers │ IPC │ Shared Memory │ I/O   │
└──────────────────────────┬───────────────────────────────┘
                           │
┌──────────────────────────▼───────────────────────────────┐
│                    TRANSPORT                              │
│                                                          │
│  In-process │ Shared Memory │ UDP │ DDS │ CAN │ Custom   │
└──────────────────────────────────────────────────────────┘
```

The critical boundary is between **semantics** and **mechanism**.

# 2. Minimum NROS primitives

The core should ideally be explainable through a small vocabulary.

### Fundamental objects

```text
NROS
│
├── Type
├── Message
├── Channel
├── Event
├── Component
├── Service
├── Action
├── State
├── Resource
├── Runtime
└── Scheduler
```

Everything else should build on these.

For example:

```text
Camera
   │
   ▼
Component
   │
   ▼
Channel<Image>
   │
   ▼
Component
   │
   ▼
Service<Request, Response>
```

# 3. Type system

Rust gives NROS a major opportunity here.

Instead of treating message definitions primarily as runtime metadata, NROS can make the type system central.

Conceptually:

```text
Message<T>
```

where `T` defines the payload.

For example:

```text
Message<ImageFrame>
Message<LaserScan>
Message<Pose>
Message<JointState>
Message<Trajectory>
```

But the message itself may also carry runtime metadata:

```text
Message<T>
├── payload: T
├── timestamp
├── sequence
├── source
├── correlation
└── metadata
```

This separates:

**payload semantics**

from

**transport/runtime metadata**.

# 4. Channel as the central communication primitive

Rather than making `Topic` the universal abstraction, NROS could use:

```text
Channel<T>
```

A channel defines:

```text
Channel<T>
│
├── producer(s)
├── consumer(s)
├── capacity
├── ordering
├── delivery semantics
├── reliability
├── ownership
├── transport
└── QoS
```

This lets the runtime choose implementation details without changing the application API.

For example:

```text
Channel<ImageFrame>
```

could internally become:

```text
InProcessChannel<ImageFrame>
```

or:

```text
SharedMemoryChannel<ImageFrame>
```

or:

```text
NetworkChannel<ImageFrame>
```

while the component still sees:

```text
Channel<ImageFrame>
```

# 5. Event model

A channel carries data.

An **event** describes something happening in the runtime.

```text
Event
│
├── MessageReceived
├── TimerElapsed
├── ComponentStarted
├── ComponentStopped
├── ServiceRequested
├── ActionRequested
├── ResourceChanged
├── DeadlineMissed
└── FaultDetected
```

This gives NROS a unified event-driven foundation.

```text
                 Event
                   │
                   ▼
               Scheduler
                   │
                   ▼
              Component
                   │
             ┌─────┴─────┐
             ▼           ▼
          State        Output
```

# 6. Component contract

A component should describe more than executable code.

Conceptually:

```text
ComponentDescriptor
│
├── Identity
├── Inputs
├── Outputs
├── Services
├── Actions
├── State
├── Resources
├── Lifecycle
├── Execution constraints
└── Dependencies
```

This enables the runtime to inspect a component before executing it.

For example:

```text
Component: motor_controller

Inputs:
    JointCommand

Outputs:
    JointState

Execution:
    periodic

Period:
    1 ms

Deadline:
    500 µs

Memory:
    bounded

Required:
    motor_bus
```

The runtime can then determine whether the component can actually be admitted.

# 7. Runtime admission

This suggests an important NROS concept:

```text
Component
     │
     ▼
Validation
     │
     ├── dependencies available?
     ├── resources available?
     ├── configuration valid?
     ├── timing constraints valid?
     └── communication graph valid?
     │
     ▼
Admission
     │
     ▼
Execution
```

This is fundamentally different from simply launching processes.

NROS becomes capable of reasoning about the system **before execution**.

# 8. Scheduler

The scheduler becomes one of the most important core abstractions.

Rather than embedding one scheduling strategy into the framework:

```text
Scheduler
│
├── FIFO
├── Priority
├── Deadline
├── Periodic
├── RealTime
└── Custom
```

the runtime should ideally expose a scheduling interface.

The application describes constraints:

```text
Task:
    period = 1ms
    deadline = 700µs
    priority = high
```

The runtime decides how that work is actually scheduled.

# 9. Runtime traits

A Rust architecture naturally leads toward traits.

Conceptually:

```text
Runtime
├── spawn()
├── schedule()
├── register()
├── shutdown()
├── clock()
└── resources()
```

Then platform implementations can differ:

```text
NROS Runtime
     │
     ├── LinuxRuntime
     ├── EmbeddedRuntime
     ├── RTOSRuntime
     └── SimulationRuntime
```

The core should not need to know which operating system is underneath.

# 10. Clock abstraction

Time is extremely important in robotics.

NROS should not casually depend on wall-clock time.

A conceptual clock interface:

```text
Clock
│
├── monotonic()
├── realtime()
├── simulation()
└── logical()
```

This enables:

```text
Production
    → hardware clock

Simulation
    → simulated clock

Replay
    → recorded timeline

Testing
    → deterministic virtual clock
```

That becomes extremely useful for reproducibility.

# 11. Deterministic replay

ROS bags established the value of recording computation-graph data.

NROS can make replay more fundamental:

```text
Live execution
      │
      ▼
Event stream
      │
      ├── messages
      ├── timing
      ├── state transitions
      ├── faults
      └── runtime events
      │
      ▼
Trace
      │
      ▼
Deterministic replay
```

Instead of recording only messages, NROS can eventually record **execution history**.

That enables:

```text
production failure
       ↓
record
       ↓
replay
       ↓
debug
       ↓
verify
       ↓
regression test
```

# 12. State

ROS traditionally separates communication from state/configuration.

NROS can explicitly model state:

```text
ComponentState<T>
│
├── current
├── version
├── timestamp
├── provenance
└── validity
```

This gives a component a coherent state model:

```text
Inputs
   ↓
State transition
   ↓
New state
   ↓
Outputs / events
```

That is much closer to a formal state-machine interpretation.

# 13. Services

Services remain useful.

But instead of viewing them merely as:

```text
request → response
```

NROS can make correlation and execution semantics explicit:

```text
Request
│
├── request_id
├── source
├── timestamp
├── deadline
└── payload
        │
        ▼
     Service
        │
        ▼
Response
│
├── request_id
├── status
├── timestamp
└── payload
```

This is important for distributed systems.

# 14. Actions

ROS `actionlib` solved a real problem: some operations are neither simple messages nor instantaneous request/response calls.

NROS can make this a first-class abstraction:

```text
Action
│
├── Goal
├── Feedback
├── Result
├── Cancellation
└── Status
```

For example:

```text
NavigateTo(goal)
       │
       ├── feedback → current position
       │
       ├── feedback → remaining distance
       │
       └── result → success/failure
```

Actions therefore become **long-running distributed operations**.

# 15. Resource model

This is another area where NROS can move beyond traditional ROS.

Resources might include:

```text
Resource
│
├── CPU
├── Memory
├── GPU
├── Camera
├── LiDAR
├── Motor
├── Network
├── File
└── Device
```

Components declare requirements:

```text
PerceptionComponent
    requires:
        camera
        GPU
        256 MB memory
```

The runtime can then manage ownership and conflicts.

# 16. The resulting NROS Core

The architecture now becomes:

```text
                     NROS CORE
                         │
        ┌────────────────┼────────────────┐
        │                │                │
      DATA           EXECUTION          STATE
        │                │                │
     Message          Event            State
     Channel          Scheduler        Lifecycle
        │                │                │
        └────────────────┼────────────────┘
                         │
                  DISTRIBUTION
                         │
             ┌───────────┼───────────┐
             │           │           │
          Service      Action      Discovery
             │           │           │
             └───────────┼───────────┘
                         │
                      RUNTIME
                         │
             ┌───────────┼───────────┐
             │           │           │
          Platform     Transport   Resources
```

This gives us a much cleaner foundation for analyzing the actual NROS repository.

The next level should therefore be **the crate/workspace architecture**:

```text
nros-core
nros-types
nros-channel
nros-runtime
nros-executor
nros-transport
nros-discovery
nros-service
nros-action
nros-lifecycle
nros-time
nros-tracing
nros-cli
```

The crucial question for the repository is then:

> **Which of these responsibilities already exist in NROS, which are only scaffolded, and which architectural boundaries need to be introduced or corrected?**

# NROS — Part IV: From Core Concepts to a Rust Workspace

Now we can translate the conceptual architecture into a **Rust workspace architecture**.

The key rule should be:

> **Dependency direction must flow from fundamental semantics toward platform-specific mechanisms—not the other way around.**

That prevents the NROS core from becoming coupled to Linux, DDS, networking, Tokio, or a particular executor.

## 1. Proposed workspace topology

A mature NROS workspace can be organized approximately like this:

```text
nros/
│
├── Cargo.toml
├── crates/
│
│   ├── nros-core/
│   ├── nros-types/
│   ├── nros-channel/
│   ├── nros-event/
│   ├── nros-runtime/
│   ├── nros-executor/
│   ├── nros-scheduler/
│   ├── nros-time/
│   ├── nros-lifecycle/
│   ├── nros-service/
│   ├── nros-action/
│   ├── nros-discovery/
│   ├── nros-transport/
│   ├── nros-ipc/
│   ├── nros-shm/
│   ├── nros-dds/
│   ├── nros-tracing/
│   ├── nros-record/
│   ├── nros-replay/
│   └── nros-cli/
│
├── examples/
├── tests/
├── benches/
└── docs/
```

This is not necessarily the exact final structure. The important thing is the **separation of responsibilities**.

# 2. `nros-core`

This should be extremely small.

Its purpose is to contain primitives that almost everything else needs.

```text
nros-core
│
├── identifiers
├── errors
├── metadata
├── versioning
├── timestamps
├── handles
├── result/status types
└── fundamental traits
```

It should ideally have very few dependencies.

Conceptually:

```text
nros-core
    ↑
    │
almost everything
```

Not:

```text
nros-core
    ↓
tokio
    ↓
network
    ↓
DDS
```

That would invert the architecture.

# 3. `nros-types`

This layer defines typed communication contracts.

```text
nros-types
│
├── Message
├── TypeId
├── TypeDescriptor
├── Schema
├── Serialization
└── MessageMetadata
```

The dependency direction should look like:

```text
nros-types
     │
     └── nros-core
```

Rather than coupling the type layer to a specific transport.

# 4. `nros-channel`

This is potentially one of the most important crates.

```text
nros-channel
│
├── Channel<T>
├── Publisher<T>
├── Subscriber<T>
├── Queue
├── Capacity
├── Backpressure
├── DeliveryPolicy
└── QoS
```

Its job is to answer:

> **How do NROS components exchange typed data?**

Not:

> How does DDS work?

DDS belongs below this abstraction.

# 5. `nros-event`

Events provide the bridge between communication and execution.

```text
nros-event
│
├── Event
├── EventId
├── EventKind
├── EventSource
├── EventTimestamp
└── EventMetadata
```

Potential event categories:

```text
Message
Timer
Lifecycle
Service
Action
Resource
Fault
Runtime
```

This allows the runtime to operate on a unified event model.

# 6. `nros-runtime`

This is where the system starts becoming a runtime rather than merely a collection of communication libraries.

```text
nros-runtime
│
├── Runtime
├── RuntimeBuilder
├── ComponentRegistry
├── ResourceRegistry
├── Graph
├── RuntimeContext
└── RuntimeState
```

Conceptually:

```text
Application
     │
     ▼
RuntimeBuilder
     │
     ▼
Runtime
 ┌───┼───────────┐
 │   │           │
Graph Components Resources
```

# 7. `nros-executor`

The executor should not necessarily own scheduling policy.

Instead:

```text
Executor
    │
    ▼
Scheduler
```

The executor answers:

> **How is runnable work executed?**

The scheduler answers:

> **Which work should execute, when, and under what constraints?**

That distinction is valuable.

# 8. `nros-scheduler`

The scheduler can eventually support:

```text
Scheduler
│
├── FIFO
├── Priority
├── Deadline
├── Periodic
├── EDF
├── RealTime
└── Custom
```

The first implementation should probably remain conservative.

A simple deterministic scheduler is more valuable than prematurely implementing a sophisticated real-time scheduler that cannot yet be verified.

# 9. `nros-time`

Time deserves its own abstraction.

```text
nros-time
│
├── Duration
├── Instant
├── Clock
├── MonotonicClock
├── WallClock
├── SimClock
└── VirtualClock
```

This allows tests to replace real time with deterministic time.

For example:

```text
Production → SystemClock
Simulation → SimClock
Replay     → ReplayClock
Testing    → VirtualClock
```

# 10. `nros-lifecycle`

Lifecycle management becomes independent from process management.

```text
nros-lifecycle
│
├── LifecycleState
├── Transition
├── TransitionError
├── LifecycleManager
└── LifecyclePolicy
```

The state machine might be:

```text
Unconfigured
      │
      ▼
Configured
      │
      ▼
Inactive
      │
      ▼
Active
      │
      ├──────► Error
      │
      ▼
Finalized
```

The exact states can evolve, but the important thing is that transitions become explicit and testable.

# 11. `nros-service`

Services should remain independent from transport.

```text
nros-service
│
├── Service
├── Request
├── Response
├── ServiceClient
├── ServiceServer
└── CorrelationId
```

Then:

```text
Service
   │
   ├── InProcess transport
   ├── Shared-memory transport
   └── Network transport
```

The service abstraction does not care which one is used.

# 12. `nros-action`

Actions build on services/events/channels.

Conceptually:

```text
Action
│
├── Goal
├── Feedback
├── Result
├── Cancel
└── Status
```

Architecture:

```text
                  Action
                    │
       ┌────────────┼────────────┐
       ▼            ▼            ▼
      Goal       Feedback      Result
       │            │            │
       └────────────┼────────────┘
                    ▼
                Component
```

Actions should therefore not become an isolated subsystem.

# 13. `nros-discovery`

This is where the distributed graph enters.

```text
nros-discovery
│
├── Participant
├── ComponentDiscovery
├── ChannelDiscovery
├── ServiceDiscovery
├── ActionDiscovery
└── GraphState
```

The core should define **what discovery means**.

The implementation determines **how discovery happens**.

For example:

```text
Discovery
│
├── Local
├── UDP
├── DDS
├── Static
└── Embedded
```

# 14. `nros-transport`

This is the abstraction between communication semantics and actual data movement.

```text
nros-transport
│
├── Transport
├── Endpoint
├── Connection
├── Sender
├── Receiver
└── TransportCapabilities
```

Possible implementations:

```text
                    Transport
                       │
       ┌───────────────┼────────────────┐
       ▼               ▼                ▼
    InProcess       SharedMemory      Network
                                       │
                                  ┌────┴────┐
                                  ▼         ▼
                                 UDP       DDS
```

This prevents DDS from becoming synonymous with NROS.

# 15. `nros-ipc` and `nros-shm`

These should be platform mechanisms rather than core semantics.

```text
nros-ipc
    │
    ├── Unix IPC
    ├── pipes
    └── local sockets

nros-shm
    │
    ├── shared memory
    ├── memory pools
    └── zero-copy buffers
```

They can depend on the transport abstraction.

# 16. `nros-dds`

DDS should be an **adapter**, not the identity of NROS.

```text
NROS Channel
      │
      ▼
Transport
      │
      ▼
DDS Adapter
      │
      ▼
DDS implementation
```

This is an important architectural distinction from ROS 2.

NROS could use DDS where it makes sense without requiring every NROS deployment to use DDS.

# 17. `nros-tracing`

Observability should be a first-class subsystem.

```text
nros-tracing
│
├── Span
├── Event
├── TraceId
├── ExecutionTrace
├── Metrics
└── Diagnostics
```

Then:

```text
Component
    │
    ├── execution
    ├── message
    ├── scheduling
    ├── lifecycle
    └── fault
          │
          ▼
       Trace
```

This becomes extremely valuable for deterministic debugging.

# 18. `nros-record`

Instead of merely recording messages:

```text
nros-record
│
├── messages
├── events
├── lifecycle
├── timing
├── faults
└── metadata
```

A recording becomes an **execution artifact**.

That makes it possible to reproduce system behavior.

# 19. `nros-replay`

Replay then consumes that artifact:

```text
Recording
    │
    ▼
Replay Engine
    │
    ├── virtual clock
    ├── recorded messages
    ├── recorded events
    └── deterministic scheduling
    │
    ▼
NROS Runtime
```

This creates a very powerful development loop:

```text
RUN
 ↓
RECORD
 ↓
REPRODUCE
 ↓
DEBUG
 ↓
FIX
 ↓
REPLAY
 ↓
VERIFY
```

# 20. `nros-cli`

The CLI should be an observer/controller of the runtime rather than embedding runtime logic.

Potential interface:

```text
nros graph
nros node
nros channel
nros service
nros action
nros lifecycle
nros resource
nros trace
nros record
nros replay
nros doctor
```

For example:

```text
nros graph
```

could inspect:

```text
Component A
   │
   ├── Channel<X>
   ▼
Component B
   │
   └── Service<Y>
```

# 21. Dependency graph

The workspace should approximately converge toward:

```text
                         nros-cli
                            │
                     ┌──────┴──────┐
                     ▼             ▼
                nros-record   nros-replay
                     │             │
                     └──────┬──────┘
                            ▼
                       nros-runtime
                            │
              ┌─────────────┼─────────────┐
              ▼             ▼             ▼
        nros-executor  nros-lifecycle  nros-discovery
              │
              ▼
        nros-scheduler
              │
              ▼
          nros-event
              │
              ▼
         nros-channel
              │
              ▼
          nros-types
              │
              ▼
          nros-core
```

Transport remains underneath:

```text
nros-channel
     │
     ▼
nros-transport
     │
 ┌───┼───────────────┐
 ▼   ▼               ▼
IPC SHM             DDS
```

The crucial property is **acyclic architectural dependency**.

# 22. The NROS design test

Every crate should be challenged with five questions:

### A. Is this a semantic primitive?

If yes, it probably belongs near `nros-core`.

### B. Is this execution policy?

It probably belongs in runtime/executor/scheduler.

### C. Is this communication mechanism?

It belongs below the communication abstraction.

### D. Is this platform-specific?

It should not leak into the core.

### E. Is this tooling?

It should consume runtime APIs rather than define them.

# 23. The most important architectural constraint

NROS should avoid this failure mode:

```text
              nros-core
                  │
          ┌───────┴────────┐
          ▼                ▼
        Tokio             DDS
          │                │
          └───────┬────────┘
                  ▼
             Linux APIs
```

That would make NROS effectively a Linux-specific application framework.

Instead:

```text
                     NROS SEMANTICS
                           │
                  ┌────────┴────────┐
                  ▼                 ▼
              Runtime API       Transport API
                  │                 │
        ┌─────────┼──────┐    ┌────┼──────────┐
        ▼         ▼      ▼    ▼    ▼          ▼
      Linux     RTOS   Embedded IPC  SHM      DDS
```

The **core defines the contract; implementations provide the mechanism**.

## 24. What this means for the actual NROS repository

At this point, the ROS → NROS transformation can be evaluated systematically:

```text
ROS Concept              NROS Question
──────────────────────   ─────────────────────────────
Node                     Component?
Topic                    Channel?
Message                  Typed Message?
ROS Master               Discovery/Runtime?
Callback                 Event/Activation?
Executor                 Executor + Scheduler?
Parameter Server         State/Configuration?
Service                  Service?
actionlib                Action?
rosbag                   Record/Replay?
roslaunch                Runtime orchestration?
catkin                   Cargo?
roscpp/rospy             Rust APIs?
DDS                      Transport implementation?
rosnode/rostopic         NROS CLI?
```

The critical task is **not to implement all of these immediately**.

First establish the invariant:

```text
                 NROS CORE
                     │
          ┌──────────┴──────────┐
          │                     │
       Semantics             Contracts
          │                     │
          └──────────┬──────────┘
                     ▼
                  Runtime
                     │
          ┌──────────┼──────────┐
          ▼          ▼          ▼
       Execute    Transport   Observe
```

Once that foundation is stable, robotics-specific functionality can grow above it without contaminating the runtime core.

**The next layer is therefore NROS's actual execution lifecycle: from process startup → runtime initialization → component registration → graph formation → scheduling → communication → shutdown, including the exact boundaries where determinism and safety guarantees can be enforced.**

# NROS — Part V: The Runtime Lifecycle

Now we move from **static architecture** to **runtime behavior**.

A strong NROS design should define exactly what happens from the moment an application starts until the runtime shuts down.

The lifecycle should be explicit:

```text
BOOT
  ↓
INITIALIZE
  ↓
DISCOVER
  ↓
CONFIGURE
  ↓
ADMIT
  ↓
ACTIVATE
  ↓
RUN
  ↓
DEACTIVATE
  ↓
SHUTDOWN
```

This is more than process management. It defines the **operational semantics of NROS**.

# 1. Runtime boot

An NROS application begins with a runtime:

```text
main()
  │
  ▼
NrosRuntime::builder()
  │
  ▼
Runtime configuration
  │
  ▼
Runtime initialization
```

At this point NROS should establish:

- runtime identity
- clock
- logging
- configuration
- resource manager
- scheduler
- transport registry
- component registry
- tracing

Conceptually:

```text
Runtime
├── Identity
├── Clock
├── Scheduler
├── Resources
├── Graph
├── Transport
├── Components
└── Diagnostics
```

# 2. Runtime initialization

Initialization should occur in deterministic stages.

```text
INIT
 │
 ├── Core
 │
 ├── Clock
 │
 ├── Configuration
 │
 ├── Transport
 │
 ├── Discovery
 │
 ├── Scheduler
 │
 └── Observability
 │
 ▼
READY
```

A failure at this stage should prevent the runtime from entering `RUNNING`.

This is important:

> **NROS should never claim a runtime state that has not actually been established.**

# 3. Component registration

Components are then registered.

```text
Runtime
   │
   ▼
ComponentRegistry
   │
   ├── Sensor
   ├── Perception
   ├── Planner
   ├── Controller
   └── Diagnostics
```

Registration should produce a descriptor:

```text
ComponentDescriptor
│
├── identity
├── inputs
├── outputs
├── services
├── actions
├── resources
├── execution constraints
└── lifecycle
```

This gives NROS enough information to inspect the system before executing it.

# 4. Discovery

For distributed deployments, components may exist on different machines.

```text
Robot A
┌──────────────┐
│ Sensor       │
│ Perception   │
└──────┬───────┘
       │
       │ discovery
       ▼
┌──────────────┐
│ NROS Graph   │
└──────┬───────┘
       │
       ▼
Robot B
┌──────────────┐
│ Planner      │
│ Controller   │
└──────────────┘
```

But discovery should not be mandatory for every deployment.

NROS should support:

```text
DiscoveryMode
│
├── Static
├── Local
├── Dynamic
├── Network
└── Disabled
```

This is particularly useful for constrained embedded systems.

# 5. Graph formation

Once components and communication endpoints are known:

```text
Component A
    │
    │ Channel<X>
    ▼
Component B
    │
    │ Service<Y>
    ▼
Component C
```

NROS can construct the runtime graph.

But the graph should contain more than topology:

```text
Graph
│
├── Components
├── Channels
├── Services
├── Actions
├── Resources
├── Dependencies
└── Execution constraints
```

The graph therefore becomes a **system model**.

# 6. Admission control

This is one of the strongest potential differences from conventional ROS.

Before activation:

```text
             SYSTEM GRAPH
                   │
                   ▼
             Validation
                   │
      ┌────────────┼────────────┐
      ▼            ▼            ▼
  Type safety   Resources    Scheduling
      │            │            │
      └────────────┼────────────┘
                   ▼
                ADMIT?
                /    \
              YES     NO
               │       │
               ▼       ▼
            Activate  Reject
```

Validation can include:

### Type compatibility

```text
Publisher<T>
      │
      ▼
Subscriber<T>
```

must agree on the communication contract.

### Resource availability

```text
Component requires:
    GPU
    Camera
    128 MB
```

The runtime must determine whether those resources exist.

### Timing feasibility

```text
period = 1 ms
deadline = 500 µs
```

should not be blindly accepted if the runtime cannot provide the requested guarantees.

# 7. Activation

After admission:

```text
REGISTERED
    │
    ▼
CONFIGURED
    │
    ▼
ADMITTED
    │
    ▼
ACTIVATING
    │
    ▼
ACTIVE
```

Activation can acquire:

- devices
- memory pools
- channels
- timers
- scheduler slots
- transport endpoints

Only after successful acquisition should the component become active.

# 8. Running

The runtime now enters its main execution phase.

```text
                     NROS Runtime
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
          Sensor       Planner     Controller
             │            │            │
             ▼            ▼            ▼
          Events       Events       Timers
             │            │            │
             └────────────┼────────────┘
                          ▼
                      Scheduler
                          │
                          ▼
                       Executor
```

The distinction between **event**, **scheduling**, and **execution** becomes important here.

# 9. Event → activation

Suppose a LiDAR frame arrives:

```text
LiDAR
  │
  ▼
Channel<LaserScan>
  │
  ▼
MessageReceived
  │
  ▼
Activation
  │
  ▼
Scheduler
  │
  ▼
Perception Component
```

The message arrival itself does not necessarily execute the callback immediately.

Instead:

**event → runnable work → scheduling decision → execution**

This gives NROS much better control over execution semantics.

# 10. Periodic execution

Some components should not depend on message arrival.

For example:

```text
Motor controller
       │
       ▼
Periodic activation
       │
   every 1 ms
       │
       ▼
Controller execution
```

The scheduler therefore needs timer-driven activations.

```text
Clock
 │
 ├── 1 ms → Controller
 ├── 10 ms → State estimator
 ├── 50 ms → Planner
 └── 1 s → Diagnostics
```

# 11. Execution budget

NROS can associate an execution budget with work:

```text
Task
├── period
├── deadline
├── priority
├── budget
└── affinity
```

Example:

```text
Controller
period   = 1 ms
deadline = 800 µs
budget   = 500 µs
priority = high
```

This does **not automatically prove real-time correctness**.

It gives the runtime explicit information with which such properties can eventually be analyzed or enforced.

That distinction is important for engineering rigor.

# 12. Communication during execution

A typical pipeline becomes:

```text
Sensor
  │
  ▼
Channel<SensorData>
  │
  ▼
Perception
  │
  ▼
Channel<WorldState>
  │
  ▼
Planner
  │
  ▼
Channel<Trajectory>
  │
  ▼
Controller
  │
  ▼
Actuator
```

Each channel can independently specify:

```text
capacity
ordering
delivery
reliability
ownership
transport
QoS
```

Therefore the same computational graph can operate over different physical communication mechanisms.

# 13. Local optimization

Consider:

```text
Camera
   │
   ▼
Image Processing
```

If both components are in the same process:

```text
Component A
     │
     ▼
In-process Channel
     │
     ▼
Component B
```

NROS may avoid serialization entirely.

If they occupy separate processes:

```text
Process A
    │
    ▼
Shared Memory
    │
    ▼
Process B
```

If they're on different machines:

```text
Robot A
   │
   ▼
Network Transport
   │
   ▼
Robot B
```

The application-level contract remains stable.

# 14. Fault handling

Suppose a component misses its deadline:

```text
Deadline Miss
     │
     ▼
Runtime Event
     │
     ▼
Fault Policy
```

Possible policies:

```text
FaultPolicy
│
├── Ignore
├── Warn
├── Retry
├── Restart
├── Isolate
├── Degrade
└── FailSafe
```

The correct choice depends on the component.

A diagnostic component may tolerate missed deadlines.

A safety-critical actuator controller may not.

# 15. Degraded operation

This allows NROS to model graceful degradation.

```text
NORMAL
  │
  │ perception failure
  ▼
DEGRADED
  │
  ├── reduce sensor fusion
  ├── reduce planning complexity
  └── restrict actuator commands
```

Eventually:

```text
DEGRADED
    │
    │ unrecoverable failure
    ▼
SAFE STATE
```

This is much more appropriate for autonomous machines than simply restarting a crashed process.

# 16. Shutdown

Shutdown should be the reverse of activation.

```text
ACTIVE
  │
  ▼
DEACTIVATING
  │
  ├── stop scheduling
  ├── drain/cancel work
  ├── stop communication
  ├── release devices
  └── release resources
  │
  ▼
INACTIVE
  │
  ▼
FINALIZED
```

The order matters.

For example, a motor controller should not release its motor interface while work can still be scheduled against it.

# 17. Complete NROS lifecycle

Putting everything together:

```text
                    ┌──────────────┐
                    │     BOOT     │
                    └──────┬───────┘
                           ▼
                    ┌──────────────┐
                    │ INITIALIZE   │
                    └──────┬───────┘
                           ▼
                    ┌──────────────┐
                    │   REGISTER   │
                    └──────┬───────┘
                           ▼
                    ┌──────────────┐
                    │   DISCOVER   │
                    └──────┬───────┘
                           ▼
                    ┌──────────────┐
                    │  VALIDATE    │
                    └──────┬───────┘
                           ▼
                    ┌──────────────┐
                    │    ADMIT     │
                    └──────┬───────┘
                           ▼
                    ┌──────────────┐
                    │   ACTIVATE   │
                    └──────┬───────┘
                           ▼
                 ┌─────────────────────┐
                 │       RUNNING       │
                 │                     │
                 │ Events              │
                 │ Scheduling          │
                 │ Communication       │
                 │ State               │
                 │ Monitoring          │
                 └──────────┬──────────┘
                            │
                  ┌─────────┴─────────┐
                  ▼                   ▼
             DEACTIVATE             FAULT
                  │                   │
                  │            ┌──────┴──────┐
                  │            ▼             ▼
                  │         RECOVER      SAFE STATE
                  │            │
                  │            └──────┐
                  │                   ▼
                  └──────────────► SHUTDOWN
```

# 18. The critical NROS invariant

The entire lifecycle can be governed by one strong rule:

> **No state transition without observed evidence that its prerequisites have been satisfied.**

For example:

```text
REGISTERED
    │
    ├── descriptor valid?
    └── dependencies resolved?
         │
         ▼
CONFIGURED
```

Then:

```text
CONFIGURED
    │
    ├── resources available?
    ├── channels valid?
    ├── scheduler admits workload?
    └── transport ready?
         │
         ▼
ADMITTED
```

And:

```text
ADMITTED
    │
    ├── activation succeeded?
    └── runtime state consistent?
         │
         ▼
ACTIVE
```

This makes NROS state transitions **evidence-driven rather than aspirational**.

That principle is particularly valuable for the repository's verification and CI model.

# 19. From lifecycle to verification

Every transition can produce an observable record:

```text
Transition
│
├── previous_state
├── requested_state
├── prerequisites
├── evidence
├── timestamp
├── actor
└── result
```

So NROS can eventually produce a trace like:

```text
00:00 BOOT
00:01 INITIALIZED
00:02 COMPONENTS_REGISTERED
00:03 GRAPH_VALIDATED
00:04 RESOURCES_ADMITTED
00:05 COMPONENTS_ACTIVATED
00:06 RUNNING
```

If something fails:

```text
00:04 RESOURCE_ADMISSION_FAILED
     │
     ├── component = motor_controller
     ├── resource = motor_bus
     └── reason = unavailable
```

That gives the runtime a verifiable operational history.

## 20. Where this leaves the ROS → NROS transformation

We now have four increasingly concrete layers:

```text
                 ROS
                  │
                  ▼
        Architectural Concepts
                  │
                  ▼
             NROS Model
                  │
                  ▼
            NROS Core
                  │
                  ▼
          NROS Workspace
                  │
                  ▼
        NROS Runtime Lifecycle
```

The next major boundary is **transport and distributed communication**.

That is where we need to answer a particularly important question:

> **Can NROS provide one communication model that works identically for in-process, inter-process, embedded, and networked execution without forcing DDS to become the core architecture?**

That leads to the next layer: **NROS Transport — channels, serialization, zero-copy, shared memory, discovery, QoS, and network boundaries.**

# NROS — Part VI: Transport, Communication & Distribution

This is where NROS meets the physical world.

A robotics runtime may execute:

- two components in the same function call,
- components in different threads,
- components in different processes,
- components on different CPU cores,
- components on different machines,
- or components across a robot network.

The programming model should ideally remain stable across all of these.

The central abstraction is therefore:

> **A Channel is a logical communication contract; Transport is the mechanism used to realize that contract.**

## 1. Separate communication semantics from transport

A clean architecture is:

```text
                 Component A
                      │
                      ▼
                 Channel<T>
                      │
                Transport API
                      │
       ┌──────────────┼──────────────┐
       ▼              ▼              ▼
   In-Process      Shared Memory   Network
       │              │              │
       ▼              ▼              ▼
     Local          Local IPC      Remote
```

The application should not need to know whether a channel is implemented through:

```text
VecDeque
```

or:

```text
shared memory
```

or:

```text
UDP
```

or:

```text
DDS
```

# 2. Logical channel

A channel can be modeled as:

```text
Channel<T>
│
├── Identity
├── Type
├── Publisher(s)
├── Subscriber(s)
├── Capacity
├── QoS
├── Delivery policy
├── Ordering
├── Reliability
└── Transport binding
```

Example:

```text
Channel<LaserScan>
    name       = /lidar/front
    capacity   = 8
    ordering   = FIFO
    reliability = best_effort
    delivery   = drop_oldest
```

Notice that nothing here says **DDS**.

# 3. Transport abstraction

The transport layer realizes the channel.

```text
Transport
│
├── bind()
├── connect()
├── send()
├── receive()
├── close()
└── capabilities()
```

The runtime can select:

```text
TransportSelector
        │
        ├── InProcess
        ├── SharedMemory
        ├── IPC
        ├── UDP
        ├── TCP
        ├── DDS
        └── Custom
```

This makes the transport **pluggable**.

# 4. In-process transport

The simplest implementation:

```text
Component A
    │
    ▼
Channel<T>
    │
    ▼
Component B
```

No serialization is required.

Potentially:

```text
T
│
└── move / ownership transfer
```

This is ideal for:

- tests
- tightly coupled components
- simulation
- high-performance pipelines

# 5. Inter-process transport

Now components live in different processes:

```text
Process A
┌─────────────────┐
│ Component A     │
└────────┬────────┘
         │
         ▼
     IPC / SHM
         │
         ▼
┌────────┴────────┐
│ Component B     │
└─────────────────┘
Process B
```

Possible mechanisms:

```text
IPC
├── Unix sockets
├── shared memory
├── eventfd
├── pipes
└── memory-mapped queues
```

The logical API remains:

```text
Channel<T>
```

# 6. Shared-memory transport

For large robotic data:

```text
Camera
   │
   ▼
Shared Memory Buffer
   │
   ├───────────────┐
   ▼               ▼
Perception      Recorder
```

This can avoid expensive copies.

A useful abstraction is:

```text
Buffer<T>
│
├── ownership
├── lifetime
├── reference
├── generation
└── synchronization
```

Rust's ownership model becomes especially valuable here.

# 7. Zero-copy

For high-bandwidth data:

```text
Camera Frame
     │
     ▼
Buffer<T>
     │
     ├──────────────┐
     │              │
     ▼              ▼
Vision           Recorder
```

Instead of:

```text
Camera
 ↓
copy
 ↓
transport
 ↓
copy
 ↓
processor
```

NROS can aim for:

```text
Camera
 ↓
shared buffer
 ↓
processor
```

But zero-copy should be treated as an **optimization with explicit capability requirements**, not something the core assumes is always possible.

# 8. Serialization

Remote communication eventually requires serialization.

Conceptually:

```text
Message<T>
    │
    ▼
Serializer
    │
    ▼
Bytes
    │
    ▼
Transport
    │
    ▼
Bytes
    │
    ▼
Deserializer
    │
    ▼
Message<T>
```

NROS should therefore separate:

```text
Type
Serialization
Transport
```

A type definition should not inherently determine the network protocol.

# 9. Serialization strategies

Potential implementations include:

```text
Serialization
│
├── Binary
├── CDR
├── JSON
├── Custom
└── Zero-copy
```

Different deployments can choose different strategies.

For example:

```text
Embedded
    → compact binary

DDS interoperability
    → CDR-compatible representation

Debugging/API gateway
    → JSON
```

# 10. QoS

QoS should belong to the communication contract.

Potential dimensions:

```text
QoS
│
├── Reliability
├── Durability
├── History
├── Depth
├── Deadline
├── Lifespan
├── Liveliness
└── Delivery policy
```

This is one area where ROS 2's DDS integration provides a useful conceptual foundation, but NROS can expose QoS independently of DDS.

# 11. Reliability

Different robotic data has different requirements.

For example:

```text
Camera Frame
    → best effort

Motor Command
    → reliable

Emergency Stop
    → highly reliable
```

The runtime should not assume one policy is correct for every channel.

# 12. History

A channel might retain:

```text
History
│
├── KeepLast(1)
├── KeepLast(N)
└── KeepAll
```

For a camera:

```text
KeepLast(1)
```

may make sense.

For a command stream:

```text
KeepLast(N)
```

may be appropriate.

For some state/configuration channels:

```text
Durable latest state
```

may be more appropriate.

# 13. Deadline

A communication contract can include:

```text
Deadline = 10 ms
```

If the expected update doesn't arrive:

```text
Deadline missed
      │
      ▼
Runtime Event
      │
      ▼
Fault / diagnostic policy
```

This ties transport directly into runtime observability.

# 14. Backpressure

Consider:

```text
Producer: 1000 messages/s
Consumer: 100 messages/s
```

NROS must have an explicit policy.

```text
Backpressure
│
├── Block
├── DropNewest
├── DropOldest
├── LatestOnly
├── Reject
└── Expand
```

For real-time systems, unbounded expansion is generally dangerous.

A bounded queue is therefore a much safer primitive.

# 15. Ownership and lifetime

Rust allows a more explicit model:

```text
Message<T>
     │
     ├── Owned
     ├── Shared
     ├── Borrowed
     └── SharedBuffer
```

This becomes particularly important when using:

- shared memory
- DMA
- GPU buffers
- camera frames
- LiDAR point clouds

The lifetime of the underlying memory must be explicit.

# 16. Discovery

Communication requires discovering endpoints.

For example:

```text
Component A
    │
    │ "I publish LaserScan"
    ▼
Discovery
    │
    ▼
Component B
    │
    │ "I subscribe LaserScan"
    ▼
Match
```

Discovery should produce a logical relationship:

```text
Publisher<T>
      │
      ▼
Compatible Channel<T>
      │
      ▼
Subscriber<T>
```

Compatibility can include:

```text
Type
QoS
Transport
Security
Version
```

# 17. Static discovery

Embedded deployments may not want dynamic discovery.

A configuration can describe:

```text
Component A
    publishes → channel_1

Component B
    subscribes → channel_1
```

Then startup becomes deterministic.

This is useful for:

- microcontrollers
- safety-oriented deployments
- fixed industrial machines
- offline systems

# 18. Dynamic discovery

More flexible systems can dynamically discover:

```text
Robot enters network
       │
       ▼
Discovery
       │
       ▼
Capabilities announced
       │
       ▼
Graph updated
       │
       ▼
Channels matched
```

This is useful for multi-robot systems.

# 19. Security boundary

Distributed robotics eventually requires authentication and authorization.

A channel can therefore have:

```text
SecurityPolicy
│
├── Authentication
├── Authorization
├── Encryption
├── Integrity
└── Identity
```

But security should not be hard-coded into every message.

Instead:

```text
Application
   │
   ▼
NROS Channel
   │
   ▼
Security Policy
   │
   ▼
Transport
```

This keeps security composable.

# 20. Network topology

NROS can eventually support:

```text
                 NROS Network
                     │
          ┌──────────┼──────────┐
          ▼          ▼          ▼
        Robot A    Robot B    Base Station
          │          │          │
       sensors    sensors     planning
       control    control     compute
```

The same logical channel model can span machines.

# 21. DDS becomes an adapter

This gives a very clean relationship with ROS 2:

```text
                   NROS
                    │
               Transport API
                    │
             ┌──────┴──────┐
             ▼             ▼
          Native         DDS
         Transport      Adapter
             │             │
             ▼             ▼
         NROS system    ROS 2 ecosystem
```

That means NROS could potentially communicate with existing ROS 2 systems **without making DDS the internal architectural center**.

That is an important strategic distinction.

# 22. ROS interoperability

A possible bridge:

```text
NROS Component
      │
      ▼
NROS Channel<T>
      │
      ▼
DDS Adapter
      │
      ▼
ROS 2 Topic
      │
      ▼
ROS 2 Node
```

This makes migration more realistic.

A robot doesn't necessarily need to migrate from ROS 2 to NROS in one step.

Instead:

```text
ROS 2 ──────┐
            │
            ▼
        Interop Layer
            ▲
            │
NROS ───────┘
```

# 23. Transport capability negotiation

Not every transport supports every feature.

For example:

```text
TransportCapabilities
│
├── zero_copy
├── reliability
├── multicast
├── ordering
├── encryption
├── bounded_latency
└── discovery
```

Then the runtime can select an appropriate transport.

Example:

```text
Channel requires:
    zero_copy = true
    reliability = true

Transport A:
    zero_copy = false
    → reject

Transport B:
    zero_copy = true
    reliability = true
    → admit
```

This connects transport directly to **admission control**.

# 24. The complete communication path

We can now model the full path:

```text
                    APPLICATION
                         │
                         ▼
                    Component
                         │
                         ▼
                    Channel<T>
                         │
                 ┌───────┴───────┐
                 ▼               ▼
              QoS/Policy      Ownership
                 │               │
                 └───────┬───────┘
                         ▼
                      Transport
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
         InProc         SHM         Network
                                      │
                             ┌────────┼────────┐
                             ▼        ▼        ▼
                            UDP      DDS      Custom
```

This is the point where NROS becomes a genuinely **distributed runtime architecture** rather than simply a Rust robotics library.

# 25. The key invariant

The transport subsystem should obey:

> **Changing transport must not change application semantics.**

Ideally:

```text
Channel<T>
```

remains the application-facing abstraction regardless of whether communication is:

```text
same-thread
same-process
cross-process
shared-memory
cross-machine
ROS 2/DDS
```

Only performance characteristics, capabilities, and deployment constraints change.

## 26. ROS → NROS transformation so far

```text
ROS
 │
 ├── Node
 │      ↓
 │   NROS Component
 │
 ├── Topic
 │      ↓
 │   NROS Channel<T>
 │
 ├── Callback
 │      ↓
 │   Event / Activation
 │
 ├── Executor
 │      ↓
 │   Executor + Scheduler
 │
 ├── ROS Master
 │      ↓
 │   Runtime + Discovery
 │
 ├── rosbag
 │      ↓
 │   Record + Replay
 │
 ├── DDS
 │      ↓
 │   Transport Adapter
 │
 └── ROS Graph
        ↓
     Runtime Graph
```

The next major problem is **time and determinism**.

Once NROS has components, channels, scheduling and transports, we need to define precisely what **"real-time", "deterministic", "bounded", and "replayable"** mean—and how those claims can actually be verified rather than merely documented.

# NROS — Part VII: Time, Determinism & Real-Time Semantics

This is where NROS can move from a **ROS-inspired middleware** toward a **systems runtime**.

The critical distinction is:

> **Having a scheduler, deadlines, and priorities does not by itself make a system real-time.**

NROS needs explicit semantics and measurable guarantees.

## 1. Three different notions of time

NROS should distinguish at least:

```text
TIME
│
├── Wall Time
├── Monotonic Time
└── Logical / Simulation Time
```

### Wall time

Represents real-world calendar time.

```text
2026-08-21 00:30:00
```

Useful for:

- logs
- timestamps
- external events

But inappropriate for measuring elapsed execution because the wall clock can be adjusted.

### Monotonic time

Used for:

- deadlines
- periods
- execution duration
- timeout calculations

It should never move backward.

### Logical time

Used for:

- simulation
- replay
- deterministic testing
- accelerated execution

```text
Logical Clock
     │
     ├── advance()
     ├── pause()
     ├── step()
     └── set()
```

# 2. Clock abstraction

NROS should therefore avoid scattering calls to system time throughout the runtime.

Conceptually:

```text
trait Clock {
    fn now(&self) -> Instant;
}
```

Implementations:

```text
Clock
│
├── SystemClock
├── MonotonicClock
├── SimClock
└── VirtualClock
```

Production might use:

```text
MonotonicClock
```

while tests use:

```text
VirtualClock
```

# 3. Why virtual time matters

Consider:

```text
Component A
    │
    ├── waits 10 ms
    ▼
Component B
    │
    └── waits 50 ms
```

A normal test may require real elapsed time.

With virtual time:

```text
t = 0
  ↓
advance(10ms)
  ↓
Component A executes
  ↓
advance(50ms)
  ↓
Component B executes
```

A test can therefore execute temporal behavior without actually waiting 60 ms.

This is extremely valuable for deterministic testing.

# 4. Periodic tasks

A periodic component has:

```text
Task
├── period
├── phase
├── deadline
└── execution budget
```

Example:

```text
Controller

period   = 1 ms
phase    = 0
deadline = 800 µs
budget   = 500 µs
```

Its intended schedule is:

```text
0ms    1ms    2ms    3ms    4ms
│      │      │      │      │
▼      ▼      ▼      ▼      ▼
Run    Run    Run    Run    Run
```

# 5. Deadline semantics

A deadline is not simply a timeout.

If:

```text
release = 10.000 ms
deadline = 10.800 ms
```

then completion must occur by:

```text
10.800 ms
```

If it completes at:

```text
10.950 ms
```

the runtime should record:

```text
DeadlineMiss
{
    task: Controller,
    expected: 10.800ms,
    actual: 10.950ms
}
```

This creates observable timing evidence.

# 6. Execution budget

Deadline and budget are different.

```text
Deadline:
    maximum acceptable completion time

Budget:
    expected/allowed execution time
```

For example:

```text
period   = 1 ms
deadline = 800 µs
budget   = 400 µs
```

The controller might complete in:

```text
250 µs → normal
350 µs → normal
450 µs → budget exceeded
850 µs → deadline missed
```

The runtime can therefore distinguish **over-budget** from **deadline failure**.

# 7. Determinism

Determinism has multiple levels.

### Functional determinism

Same inputs:

```text
Input X
  +
Input Y
  ↓
Output Z
```

produce the same result.

### Scheduling determinism

The same events are executed in the same order.

### Temporal determinism

Execution occurs within predictable timing bounds.

### Replay determinism

A recorded execution produces the same observable behavior when replayed.

These should not be conflated.

# 8. Event ordering

Suppose:

```text
Event A
Event B
```

arrive nearly simultaneously.

The runtime needs an ordering rule.

Possible ordering keys:

```text
Event ordering
│
├── timestamp
├── sequence number
├── source
├── priority
└── deterministic tie-breaker
```

A deterministic tie-breaker is important.

Otherwise two executions could produce:

```text
Run 1:
A → B

Run 2:
B → A
```

even though the inputs appear identical.

# 9. Deterministic scheduler

A deterministic scheduler can maintain:

```text
RunnableQueue
│
├── deadline
├── priority
├── sequence
└── task identity
```

For example:

```text
sort by:

1. readiness
2. deadline
3. priority
4. deterministic task ID
```

The exact policy can evolve, but it must be **specified**, not accidental.

# 10. Real-time classes

NROS could distinguish:

```text
ExecutionClass
│
├── HardRealTime
├── SoftRealTime
├── BestEffort
└── Background
```

However, these labels should only have meaning if backed by measurable guarantees.

For example:

```text
HardRealTime
```

should imply a documented maximum acceptable latency and a verification mechanism.

Otherwise it is merely marketing terminology.

# 11. Allocation discipline

Real-time execution often conflicts with unrestricted dynamic allocation.

A component might therefore declare:

```text
AllocationPolicy
│
├── Dynamic
├── Bounded
├── Preallocated
└── None
```

A real-time component could use:

```text
Preallocated buffers
+
bounded queues
+
fixed resource limits
```

rather than allocating unpredictably during its control loop.

# 12. Memory pools

NROS can expose bounded memory pools:

```text
MemoryPool<T>
│
├── capacity
├── allocate()
├── release()
└── usage()
```

Example:

```text
ImagePool
capacity = 8 frames
```

This makes memory consumption observable.

# 13. Resource budgets

The same idea applies to CPU and other resources.

```text
ResourceBudget
│
├── CPU
├── Memory
├── Network
├── GPU
└── Storage
```

A component can declare:

```text
Perception
    memory ≤ 256 MB
    CPU ≤ configured budget
```

The runtime can monitor actual usage.

# 14. Admission control revisited

Now admission becomes more sophisticated:

```text
Component
    │
    ▼
Static validation
    │
    ├── types
    ├── dependencies
    └── configuration
    │
    ▼
Resource admission
    │
    ├── memory
    ├── CPU
    └── devices
    │
    ▼
Timing admission
    │
    ├── period
    ├── deadline
    └── budget
    │
    ▼
ACTIVATE
```

This is the foundation for eventually providing meaningful runtime guarantees.

# 15. Priority inversion

A serious runtime also needs to consider priority inversion.

Example:

```text
High priority task
       │
       ▼
 waits for resource
       │
       ▼
Low priority task owns resource
```

The high-priority task can be indirectly blocked.

Potential mechanisms include:

```text
SynchronizationPolicy
│
├── priority inheritance
├── priority ceiling
└── lock-free structures
```

This is one reason synchronization primitives cannot simply be treated as ordinary application details in a real-time runtime.

# 16. Lock-free communication

For high-frequency paths, NROS could provide bounded lock-free queues.

Conceptually:

```text
Producer
   │
   ▼
┌────────────────────┐
│ bounded ring buffer│
└────────────────────┘
   │
   ▼
Consumer
```

Advantages include:

- bounded memory
- predictable operations
- reduced contention

But such structures require careful correctness verification.

They should not be introduced merely because "lock-free" sounds faster.

# 17. Jitter

Even when average execution time is excellent:

```text
Average = 100 µs
```

the worst case might be:

```text
100
110
95
105
900  ← outlier
```

Real-time systems care strongly about that tail.

NROS tracing should therefore capture:

```text
Execution
│
├── release time
├── start time
├── completion time
├── latency
├── runtime
└── deadline status
```

This allows jitter analysis.

# 18. Timing trace

A trace might look conceptually like:

```text
Time ───────────────────────────────────────>

Controller   ███       ███       ███
Sensor       ██  ██    ██  ██    ██
Planner            █████████
Diagnostics                    ██
```

The runtime can then derive:

- latency
- utilization
- jitter
- deadline misses
- queue delay
- scheduling delay

# 19. Replay

Now combine time with recording:

```text
Execution
   │
   ▼
Trace
   │
   ├── events
   ├── messages
   ├── timing
   ├── state
   └── faults
   │
   ▼
Replay
   │
   ▼
Virtual Clock
   │
   ▼
NROS Runtime
```

The replay engine can reproduce the event sequence under controlled time.

# 20. Deterministic testing

A powerful NROS test could therefore say:

```text
Given:
    virtual time = 0
    sensor state = X
    component state = Y

When:
    event sequence E occurs

Then:
    state = Z
    output = O
    no deadline missed
    no resource violation
```

This transforms runtime semantics into executable tests.

# 21. Simulation

The same abstraction supports simulation:

```text
Simulation Clock
       │
       ▼
NROS Runtime
       │
       ├── simulated sensors
       ├── perception
       ├── planner
       └── controller
```

The application does not need to know whether time comes from:

```text
real hardware
```

or:

```text
simulation
```

# 22. Hardware-in-the-loop

The boundary can then become:

```text
                  NROS
                   │
          ┌────────┴────────┐
          ▼                 ▼
      Simulation        Hardware
          │                 │
          └────────┬────────┘
                   ▼
               Same APIs
```

This allows:

**simulation → HIL → real robot**

without redesigning the application architecture.

# 23. Real-time claims must be evidence-backed

This is particularly important for NROS.

Avoid:

```text
"NROS is real-time."
```

Prefer:

```text
NROS provides mechanisms for real-time-oriented execution,
including bounded queues, explicit scheduling constraints,
monotonic clocks, resource limits, and timing instrumentation.
```

Then separately verify a specific deployment:

```text
Platform
+
Kernel/RTOS
+
Scheduler
+
Workload
+
Configuration
+
Measurements
=
Evidence
```

Only then can a particular configuration make a defensible real-time claim.

# 24. NROS timing architecture

The resulting model is:

```text
                      TIME
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
     Monotonic       Logical        Wall
        │              │              │
        └──────────────┼──────────────┘
                       ▼
                    EVENTS
                       │
                       ▼
                   SCHEDULER
                       │
              ┌────────┼────────┐
              ▼        ▼        ▼
           Periodic  Deadline  Priority
              │        │        │
              └────────┼────────┘
                       ▼
                    EXECUTOR
                       │
                       ▼
                   COMPONENT
                       │
                       ▼
                    TRACE
                       │
                 ┌─────┴─────┐
                 ▼           ▼
               Record      Replay
```

This gives NROS a coherent temporal model instead of treating time as a collection of unrelated timers and timestamps.

# 25. ROS → NROS at this layer

The transformation now looks like:

```text
ROS
 │
 ├── ros::Time
 │       ↓
 │   NROS Clock
 │
 ├── callbacks
 │       ↓
 │   Events / Activations
 │
 ├── ros::Rate
 │       ↓
 │   Periodic scheduling
 │
 ├── Executor
 │       ↓
 │   Executor + Scheduler
 │
 ├── rosbag
 │       ↓
 │   Execution recording
 │
 └── simulation time
         ↓
     Logical/Virtual Clock
```

And the deeper evolution is:

```text
ROS:
    "When a message arrives, run this callback."

NROS:
    "An event activates a constrained unit of work,
     which is admitted, scheduled, executed, measured,
     and represented in the runtime's observable history."
```

That is a much stronger runtime model.

The next layer is **NROS safety and fault containment**: capability boundaries, resource ownership, component isolation, failure domains, safe-state transitions, and how Rust's ownership/type system can become part of the runtime's safety architecture rather than merely the implementation language.

# NROS — Part VIII: Safety, Isolation & Fault Containment

The next step is to make NROS capable of answering a harder question than *"how do components communicate?"*:

> **What happens when a component is wrong, compromised, overloaded, disconnected, or physically unavailable?**

ROS traditionally gives developers powerful composition mechanisms, but NROS can make **failure boundaries and resource ownership first-class runtime concepts**.

# 1. Safety starts with containment

A useful NROS principle is:

```text
Failure
   │
   ▼
Detect
   │
   ▼
Contain
   │
   ▼
Recover / Degrade
   │
   ▼
Safe State
```

Instead of:

```text
Component failure
       ↓
Entire robot becomes unpredictable
```

we want:

```text
Component A failure
       │
       ▼
Failure Domain A
       │
       ├── isolate
       ├── revoke resources
       ├── notify dependents
       └── transition system
```

# 2. Failure domains

NROS should explicitly model failure domains.

```text
NROS System
│
├── Domain A
│   ├── Camera
│   └── Vision
│
├── Domain B
│   ├── Planner
│   └── Localization
│
└── Domain C
    ├── Motor Controller
    └── Safety Monitor
```

If Vision crashes:

```text
Vision
  X
  │
  ▼
Domain A
```

the motor-control domain should not automatically crash with it.

# 3. Component isolation

There are several isolation levels:

```text
Isolation
│
├── Logical
├── Task
├── Thread
├── Process
├── Container
└── Machine
```

NROS should not force one universal model.

For example:

```text
Development:
    all components in one process

Production:
    safety-critical components isolated

Distributed:
    components across machines
```

The same component model can survive these deployment changes.

# 4. Capability-based resources

Instead of allowing arbitrary components to access global resources:

```text
Component
    │
    ├── open("/dev/...")
    ├── network
    ├── filesystem
    └── hardware
```

NROS can move toward:

```text
Component
    │
    ▼
Capabilities
    │
    ├── CameraHandle
    ├── MotorHandle
    ├── ChannelHandle
    └── TimerHandle
```

The component receives only the resources it was granted.

# 5. Resource ownership

Rust makes this particularly natural.

Conceptually:

```text
MotorHandle
```

represents permission to interact with a motor resource.

When the handle disappears:

```text
MotorHandle
    │
    ▼
Drop
    │
    ▼
Resource released
```

This is stronger than relying solely on conventions such as:

```text
motor.close()
```

# 6. Exclusive resources

Some resources should have one owner:

```text
MotorBus
   │
   └── exclusive → Controller
```

The runtime should reject:

```text
Controller A → MotorBus
Controller B → MotorBus
```

unless the resource explicitly supports shared ownership.

# 7. Shared resources

Other resources naturally support multiple consumers:

```text
Camera
 │
 ├── Vision
 ├── Recorder
 └── Diagnostics
```

The runtime can expose a read-oriented capability:

```text
CameraView
```

while maintaining exclusive control over the underlying device.

# 8. Capability delegation

A component can potentially delegate a restricted capability:

```text
Supervisor
    │
    ▼
CameraCapability
    │
    ├── read = true
    ├── configure = false
    └── shutdown = false
```

This produces a capability hierarchy rather than a flat permission system.

# 9. Resource revocation

Suppose a component becomes faulty:

```text
Component
    │
    ▼
FAULT
    │
    ▼
Capability Revocation
    │
    ├── Motor
    ├── Network
    ├── Storage
    └── Channels
```

This is important.

Restarting a process is insufficient if the failed component can continue affecting external resources.

# 10. Lifecycle + safety

The lifecycle from earlier can now incorporate fault states:

```text
                    CONFIGURED
                         │
                         ▼
                      ADMITTED
                         │
                         ▼
                       ACTIVE
                         │
              ┌──────────┴──────────┐
              ▼                     ▼
           HEALTHY                FAULT
              │                     │
              │              ┌──────┼──────┐
              │              ▼      ▼      ▼
              │           RECOVER ISOLATE SAFE
              │              │
              └──────────────┘
```

The runtime state machine therefore becomes part of safety behavior.

# 11. Health monitoring

NROS components can expose health:

```text
Health
│
├── Alive
├── Ready
├── Healthy
├── Degraded
├── Faulted
└── Unresponsive
```

A component can publish health information independently of its normal application data.

# 12. Heartbeats

A simple mechanism:

```text
Controller
   │
   ├── heartbeat
   ├── heartbeat
   ├── heartbeat
   └── heartbeat
```

If heartbeats stop:

```text
Watchdog
    │
    ▼
timeout
    │
    ▼
UNRESPONSIVE
```

The runtime then applies policy.

# 13. Watchdogs

Watchdogs can exist at multiple layers:

```text
Watchdogs
│
├── Component watchdog
├── Process watchdog
├── Runtime watchdog
├── Hardware watchdog
└── External safety watchdog
```

A critical actuator controller should ideally not depend exclusively on itself to detect its own failure.

# 14. Independent safety monitor

Consider:

```text
                 NROS
                  │
       ┌──────────┼──────────┐
       ▼          ▼          ▼
    Planner   Controller   Monitor
                            │
                            ▼
                         Safety
```

The monitor can enforce constraints independently.

For example:

```text
if velocity > limit:
    command = safe_command
```

This creates a safety boundary around normal autonomy.

# 15. Safety channels

Not all communication should have identical semantics.

A normal telemetry channel:

```text
/diagnostics
```

can tolerate loss.

A safety channel:

```text
/safety/stop
```

requires substantially stronger guarantees.

NROS should therefore permit explicit safety classification:

```text
ChannelClass
│
├── Telemetry
├── Control
├── Safety
├── Configuration
└── Diagnostic
```

This is semantic metadata—not automatically a safety certification.

# 16. Safe-state transitions

A robot should have defined safe states.

```text
RUNNING
   │
   │ critical fault
   ▼
SAFE_STOP
   │
   ├── actuators disabled
   ├── hazardous motion stopped
   └── diagnostics retained
```

For another robot:

```text
RUNNING
   │
   ▼
DEGRADED
   │
   ├── reduced speed
   ├── reduced autonomy
   └── restricted workspace
```

The appropriate response is system-specific.

# 17. Fault policy

NROS can model this explicitly:

```text
FaultPolicy
│
├── Detect
├── Classify
├── Isolate
├── Recover
├── Degrade
├── Stop
└── Escalate
```

Example:

```text
Camera failure
      │
      ▼
Classification:
"non-critical perception failure"
      │
      ▼
Degrade autonomy
```

Whereas:

```text
Motor safety controller failure
      │
      ▼
Classification:
"critical"
      │
      ▼
SAFE_STOP
```

# 18. Fault severity

A useful model:

```text
Severity
│
├── INFO
├── WARNING
├── DEGRADED
├── FAULT
└── CRITICAL
```

The runtime can map severity to policy.

# 19. Failure propagation

Failure propagation should be explicit.

```text
Sensor failure
    │
    ▼
Perception degraded
    │
    ▼
Planner degraded
    │
    ▼
Controller policy?
```

But NROS should avoid accidental cascading failure.

The dependency graph should indicate which failures are allowed to propagate.

```text
FailurePropagationPolicy
│
├── isolated
├── notify
├── degrade
└── escalate
```

# 20. Dependency graph

Earlier we had a computational graph.

Now we need a richer model:

```text
Runtime Graph
│
├── Data dependencies
├── Execution dependencies
├── Resource dependencies
├── Lifecycle dependencies
└── Safety dependencies
```

This is a significant evolution beyond a simple topic graph.

# 21. Example: mobile robot

Consider:

```text
LiDAR
  │
  ▼
Localization
  │
  ▼
Planner
  │
  ▼
Controller
  │
  ▼
Motor
```

Safety monitor:

```text
              Safety Monitor
                    │
          ┌─────────┼─────────┐
          ▼         ▼         ▼
       Planner   Controller   Motor
```

If localization fails:

```text
Localization
      X
      │
      ▼
Planner
      │
      ▼
Degraded navigation
```

If the motor controller becomes unsafe:

```text
Motor Controller
      X
      │
      ▼
Safety Monitor
      │
      ▼
SAFE_STOP
```

The distinction is crucial.

# 22. Memory safety is not system safety

NROS should be explicit about this.

Rust can help prevent classes of:

- use-after-free
- data races
- many ownership violations
- invalid memory access patterns

But Rust alone does **not** prove:

```text
"the robot is safe."
```

System safety also requires:

```text
Correct requirements
+
correct architecture
+
validated algorithms
+
hardware behavior
+
fault handling
+
timing analysis
+
verification
```

This distinction should remain central to NROS documentation.

# 23. Security and safety are related but different

Security asks:

> Who is allowed to perform this operation?

Safety asks:

> What happens if the operation or component behaves incorrectly?

So:

```text
Security
   ↓
Authorization

Safety
   ↓
Containment / safe state
```

NROS needs both.

# 24. Isolation model

We can now define a layered containment model:

```text
                 NROS
                  │
          ┌───────┴───────┐
          ▼               ▼
      Security         Safety
          │               │
      Capability       Policy
          │               │
          ▼               ▼
      Resource         Failure
      isolation        containment
          │               │
          └───────┬───────┘
                  ▼
             Safe Runtime
```

# 25. NROS safety invariant

A strong invariant emerges:

> **A component may only affect resources and execution domains for which it currently possesses valid capabilities.**

And:

> **A detected fault must not silently bypass the runtime's containment and lifecycle policies.**

These are architectural properties worth testing.

# 26. Verification implications

Each safety transition should produce evidence.

Example:

```text
FAULT_DETECTED
     │
     ├── component = controller
     ├── fault = heartbeat_timeout
     └── timestamp = T
          │
          ▼
CAPABILITIES_REVOKED
          │
          ▼
SAFE_STOP_REQUESTED
          │
          ▼
SAFE_STOP_CONFIRMED
```

Now safety behavior becomes observable.

That is extremely important for NROS's broader verification architecture.

# 27. ROS → NROS transformation

The conceptual mapping now becomes:

```text
ROS
 │
 ├── Node
 │      ↓
 │   Component + Failure Domain
 │
 ├── Parameter
 │      ↓
 │   Configuration Capability
 │
 ├── Topic
 │      ↓
 │   Typed Channel + Policy
 │
 ├── Service
 │      ↓
 │   Request/Response Capability
 │
 ├── Action
 │      ↓
 │   Managed Task / Goal
 │
 └── Process
        ↓
     Isolation Domain
```

But NROS adds:

```text
Capabilities
Resource Ownership
Fault Domains
Health
Watchdogs
Safe States
Failure Policies
```

These become **first-class runtime concepts** rather than conventions built around ROS nodes.

# 28. The emerging NROS architecture

At this point the architecture can be summarized as:

```text
┌───────────────────────────────────────────────┐
│                  NROS SYSTEM                  │
├───────────────────────────────────────────────┤
│                                               │
│  Components                                   │
│      │                                        │
│      ▼                                        │
│  Runtime Graph                                │
│      │                                        │
│  ┌───┼──────────┬────────────┐                │
│  ▼   ▼          ▼            ▼                │
│Time Scheduler Transport  Safety               │
│      │          │            │                │
│      └──────────┼────────────┘                │
│                 ▼                             │
│             Executor                         │
│                 │                             │
│                 ▼                             │
│          Resource Manager                     │
│                 │                             │
│        ┌────────┴────────┐                    │
│        ▼                 ▼                    │
│   Capabilities       Isolation                │
│                                               │
├───────────────────────────────────────────────┤
│ Observability / Trace / Replay / Diagnostics  │
└───────────────────────────────────────────────┘
```

The next critical layer is **NROS Actions and long-running work**.

ROS has `actionlib` because services are insufficient for operations such as navigation, manipulation, docking, and trajectory execution. NROS can generalize this into a stronger **Goal → Execution → Feedback → Result → Cancellation** model, integrated directly with scheduling, deadlines, lifecycle, resources, cancellation, and fault recovery.

# NROS — Part IX: Actions, Goals & Long-Running Work

ROS introduced **actions** because a simple request/response service is insufficient for long-running robotic operations.

Consider:

```text
NavigateTo(goal)
```

The operation may take:

```text
10 ms
      ↓
10 seconds
      ↓
several minutes
```

During that time the caller needs:

- progress
- feedback
- cancellation
- preemption
- final result
- failure information

NROS should make this a **native execution primitive**, not merely another messaging pattern.

# 1. From ROS actionlib to NROS Goal Execution

ROS conceptually provides:

```text
Client
  │
  ▼
Action Server
  │
  ├── Goal
  ├── Feedback
  ├── Result
  └── Cancel
```

NROS can evolve this into:

```text
Goal
 │
 ▼
Admission
 │
 ▼
Scheduling
 │
 ▼
Execution
 │
 ├── Feedback
 ├── Checkpoints
 ├── Cancellation
 ├── Deadline
 └── Resource ownership
 │
 ▼
Result
```

The important change is that a goal becomes a **runtime-managed unit of work**.

# 2. Goal as a first-class object

A goal should have an identity:

```text
Goal
│
├── goal_id
├── type
├── requester
├── priority
├── deadline
├── resource requirements
├── cancellation policy
└── execution policy
```

Example:

```text
NavigateGoal
│
├── target_pose
├── tolerance
├── max_velocity
├── deadline
└── priority
```

# 3. Goal lifecycle

A clean state machine:

```text
                  CREATED
                     │
                     ▼
                 SUBMITTED
                     │
                     ▼
                  ADMITTED
                     │
                     ▼
                 QUEUED
                     │
                     ▼
                 EXECUTING
                  │   │
          ┌───────┘   └────────┐
          ▼                    ▼
       CANCELLED             FAILED
          │                    │
          └────────┬───────────┘
                   ▼
                TERMINAL

EXECUTING
    │
    ▼
 SUCCEEDED
```

A goal must never jump arbitrarily between states.

# 4. Goal admission

The runtime can inspect:

```text
Goal
 │
 ├── required resources?
 ├── compatible controller?
 ├── deadline feasible?
 ├── priority allowed?
 ├── safety constraints?
 └── conflicting goals?
```

Then:

```text
ADMIT
```

or:

```text
REJECT
```

This is significantly stronger than immediately invoking a callback.

# 5. Goal conflicts

Imagine:

```text
Goal A:
    move arm → position X

Goal B:
    move arm → position Y
```

Both require:

```text
ArmController
```

NROS should detect the resource conflict.

Possible policies:

```text
ConflictPolicy
│
├── Reject
├── Queue
├── Preempt
├── Merge
└── Arbitration
```

# 6. Resource-aware goals

A goal can explicitly request resources:

```text
NavigateGoal
│
├── Localization
├── Planner
├── Controller
└── MotorInterface
```

The runtime can then build:

```text
Goal
  │
  ▼
Resource Reservation
  │
  ▼
Execution
```

This avoids hidden resource conflicts.

# 7. Goal execution as a state machine

A complex goal may consist of phases:

```text
Navigate
 │
 ├── Acquire localization
 │
 ├── Plan path
 │
 ├── Execute trajectory
 │
 ├── Verify arrival
 │
 └── Complete
```

This can be represented as:

```text
Goal
 │
 ▼
Phase 1
 │
 ▼
Phase 2
 │
 ▼
Phase 3
 │
 ▼
Phase 4
```

Each phase can have:

- inputs
- outputs
- timeout
- resources
- failure policy

# 8. Feedback

Feedback should be structured.

```text
Goal
 │
 ├── Progress
 ├── State
 ├── Metrics
 └── Diagnostics
```

Example:

```text
NavigationFeedback
│
├── distance_remaining
├── current_pose
├── velocity
├── current_phase
└── estimated_completion
```

Feedback should not be confused with the final result.

# 9. Result

The terminal result should explicitly distinguish:

```text
Result
│
├── Succeeded
├── Cancelled
├── Rejected
├── Failed
├── DeadlineExceeded
└── SafetyAborted
```

For example:

```text
NavigationResult
{
    status: SafetyAborted,
    reason: ObstacleDetected
}
```

This is much more useful than:

```text
false
```

# 10. Cancellation

Cancellation is not simply:

```text
stop()
```

The runtime should define semantics.

```text
Cancel Request
      │
      ▼
Cancellation Pending
      │
      ▼
Execution reaches cancellation point
      │
      ▼
Resources released
      │
      ▼
CANCELLED
```

This is particularly important for robotics because abrupt cancellation can itself be unsafe.

# 11. Cooperative cancellation

A component may periodically check:

```text
CancellationToken
```

Conceptually:

```text
if token.is_cancelled() {
    cleanup();
    return;
}
```

This allows controlled termination.

# 12. Forced cancellation

Some components may become unresponsive.

Then:

```text
Cooperative cancellation
        │
        │ timeout
        ▼
Forced termination
        │
        ▼
Isolation / recovery
```

This should be treated as a significantly more severe event.

# 13. Preemption

Suppose:

```text
Goal A
  ↓
normal navigation
```

Then:

```text
Goal B
  ↓
emergency docking
```

arrives.

NROS may allow:

```text
Goal A
    │
    ▼
PREEMPTED
    │
    ▼
Goal B
```

Preemption therefore becomes a scheduling operation.

# 14. Goal priority

Goals can have priority:

```text
Priority
│
├── Critical
├── High
├── Normal
├── Low
└── Background
```

But priority should never override safety constraints.

For example:

```text
Emergency goal
    ≠
permission to violate safety limits
```

Safety remains the upper-level constraint.

# 15. Deadline-aware goals

A goal may specify:

```text
deadline = T
```

The runtime can monitor:

```text
remaining_time
```

and make decisions:

```text
Goal nearing deadline
       │
       ├── continue
       ├── simplify
       ├── degrade
       └── abort safely
```

This allows adaptive execution.

# 16. Checkpoints

Long-running goals should be checkpointable.

```text
Goal
 │
 ├── Phase 1
 │
 ▼
Checkpoint A
 │
 ├── Phase 2
 │
 ▼
Checkpoint B
 │
 ├── Phase 3
 │
 ▼
Complete
```

If Phase 3 fails:

```text
Failure
  │
  ▼
Recovery
  │
  ▼
Checkpoint B
```

This is particularly powerful for complex autonomous operations.

# 17. Goal persistence

A goal can have an execution record:

```text
GoalRecord
│
├── identity
├── request
├── state transitions
├── feedback
├── checkpoints
├── resources
├── faults
└── result
```

This makes execution auditable.

# 18. Goal history

For example:

```text
Goal 0x42

SUBMITTED       t=0
ADMITTED        t=2ms
STARTED         t=3ms
CHECKPOINT      t=1.2s
FEEDBACK        t=2.0s
PREEMPTED       t=4.8s
CANCELLED       t=5.1s
```

A runtime can reconstruct exactly what happened.

# 19. Goal graphs

Some missions require multiple goals:

```text
Mission
 │
 ├── Goal A: localize
 │
 ├── Goal B: navigate
 │       │
 │       └── requires A
 │
 ├── Goal C: dock
 │       │
 │       └── requires B
 │
 └── Goal D: charge
         │
         └── requires C
```

This forms a **goal dependency graph**.

# 20. Parallel goals

Some goals can execute concurrently:

```text
             Mission
                │
        ┌───────┼───────┐
        ▼       ▼       ▼
      Sense   Localize  Monitor
        │       │       │
        └───────┼───────┘
                ▼
              Plan
```

NROS can determine which goals have compatible resource requirements.

# 21. Goal supervision

A supervisor can manage goals:

```text
                Supervisor
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
      Goal A      Goal B      Goal C
```

The supervisor can:

- start
- cancel
- retry
- prioritize
- checkpoint
- recover

without implementing the actual robotic algorithm.

# 22. Goal execution vs component execution

This distinction is important.

A **component** is a persistent runtime entity:

```text
Controller
```

A **goal** is a bounded unit of work:

```text
MoveTo(position)
```

Therefore:

```text
Component
    │
    ├── executes Goal A
    ├── executes Goal B
    └── executes Goal C
```

The component owns capabilities; the goals consume them under runtime supervision.

# 23. Actions as typed contracts

NROS can define:

```text
Action<Goal, Feedback, Result>
```

Conceptually:

```text
Action<G, F, R>
```

with:

```text
submit(G)
    ↓
feedback → F
    ↓
result → R
```

This provides compile-time structure around action interfaces.

# 24. Services vs actions vs channels

The communication model becomes clearer:

```text
Channel
   │
   └── continuous data flow

Service
   │
   └── short request/response

Action
   │
   └── long-running managed work
```

And:

```text
Channel<T>
Service<Request, Response>
Action<Goal, Feedback, Result>
```

become three complementary primitives.

# 25. Events

There is also a fourth primitive:

```text
Event<T>
```

for notifications such as:

```text
BatteryLow
ObstacleDetected
ComponentFaulted
GoalCompleted
```

So the communication model becomes:

```text
NROS Communication
│
├── Channel
├── Service
├── Action
└── Event
```

Each has explicit semantics.

# 26. Cancellation + safety

Consider a moving robot:

```text
NavigateGoal
      │
      ▼
Controller
      │
      ▼
Motor
```

A cancellation request should not necessarily mean:

```text
motor = 0 immediately
```

Instead:

```text
Cancel
  │
  ▼
Controlled stop
  │
  ▼
Velocity → 0
  │
  ▼
Motor safe
  │
  ▼
Goal CANCELLED
```

This makes cancellation part of the safety model.

# 27. Fault-aware actions

Suppose a planner crashes:

```text
Goal executing
      │
      ▼
Planner FAULT
      │
      ▼
Goal supervisor
      │
      ├── retry planner
      ├── alternate planner
      ├── degrade
      └── abort safely
```

The goal runtime therefore becomes an important recovery mechanism.

# 28. Actions + deterministic replay

Because every goal transition is observable:

```text
Goal
 │
 ├── request
 ├── events
 ├── feedback
 ├── checkpoints
 ├── timing
 └── result
```

the entire execution can be recorded.

Then:

```text
Recorded Goal
      │
      ▼
Replay Engine
      │
      ▼
Virtual Clock
      │
      ▼
NROS
```

This makes difficult autonomous behaviors reproducible.

# 29. ROS → NROS transformation

The mapping becomes:

```text
ROS actionlib
      │
      ▼
NROS Action
      │
      ├── Goal
      ├── Feedback
      ├── Result
      ├── Cancellation
      ├── Deadline
      ├── Priority
      ├── Resource reservation
      ├── Checkpoints
      └── Fault policy
```

The important evolution is:

```text
ROS:
    Action = communication protocol

NROS:
    Action = communication + managed execution
```

# 30. The NROS execution stack is now becoming coherent

We can now assemble the runtime layers:

```text
┌──────────────────────────────────────────────┐
│                 APPLICATION                  │
├──────────────────────────────────────────────┤
│ Components │ Goals │ Services │ Events       │
├──────────────────────────────────────────────┤
│              RUNTIME GRAPH                   │
├──────────────────────────────────────────────┤
│ Admission │ Lifecycle │ Fault Management     │
├──────────────────────────────────────────────┤
│ Scheduler │ Executor │ Timing │ Cancellation │
├──────────────────────────────────────────────┤
│ Channels │ Transport │ Serialization │ QoS   │
├──────────────────────────────────────────────┤
│ Resources │ Capabilities │ Isolation         │
├──────────────────────────────────────────────┤
│ Clock │ Trace │ Replay │ Diagnostics         │
├──────────────────────────────────────────────┤
│ OS / RTOS / Embedded Platform               │
└──────────────────────────────────────────────┘
```

This gives us a much more complete definition of **what NROS actually is**.

It is no longer simply:

```text
ROS rewritten in Rust
```

but rather:

> **A typed, resource-aware, lifecycle-managed, observable runtime for distributed robotic computation.**

# 31. The next architectural boundary

There is now one major question remaining before we can define the NROS core API:

### **How should NROS represent computation itself?**

ROS largely centers the programming model around:

```text
Node
Callback
Topic
Service
Action
```

NROS has introduced:

```text
Component
Event
Activation
Task
Goal
Resource
Capability
Channel
```

The next step is to unify these into a formal **NROS Execution Model**:

```text
               COMPONENT
                   │
          ┌────────┼────────┐
          ▼        ▼        ▼
       EVENT      TIMER     GOAL
          │        │        │
          └────────┼────────┘
                   ▼
               ACTIVATION
                   │
                   ▼
              SCHEDULABLE
                  WORK
                   │
                   ▼
               EXECUTOR
                   │
                   ▼
               RESOURCE
               CAPABILITIES
                   │
                   ▼
                EFFECTS
                   │
                   ▼
                OBSERVE
                   │
                   ▼
              TRACE / STATE
```

That execution model is the bridge between the **NROS conceptual architecture** and the actual **Rust crate/API architecture** of the `NROS` repository.

# NROS — Part X: The Unified Execution Model

We now reach the point where NROS should stop accumulating independent concepts and define **one execution semantics** connecting components, events, timers, goals, resources, scheduling, and effects.

The central question becomes:

> **What exactly is an executable unit in NROS?**

## 1. ROS starts from the Node

The traditional ROS mental model is:

```text
Node
 │
 ├── Subscriber callback
 ├── Publisher
 ├── Service server
 ├── Service client
 └── Timer callback
```

This works well for application development, but the runtime semantics are somewhat implicit.

NROS can make the execution boundary explicit:

```text
Component
    │
    ├── receives Event
    ├── receives Timer activation
    ├── receives Goal
    └── receives Resource signal
             │
             ▼
         Activation
             │
             ▼
        Executable Work
             │
             ▼
          Executor
```

# 2. Activation is the fundamental runtime event

Instead of saying:

> "A callback happened."

NROS can say:

> **An activation was created.**

An activation represents a request for a component to perform work.

Conceptually:

```text
Activation
│
├── activation_id
├── source
├── component
├── cause
├── priority
├── deadline
├── budget
├── timestamp
└── cancellation_token
```

Possible causes:

```text
ActivationCause
│
├── Message
├── Timer
├── Goal
├── ServiceRequest
├── Lifecycle
├── ExternalEvent
└── Recovery
```

# 3. One execution abstraction

This gives us a powerful unification:

```text
ROS callback
       ↓
NROS Activation
```

A timer:

```text
Timer
  ↓
Activation
```

A message:

```text
Message
  ↓
Activation
```

A service request:

```text
Request
  ↓
Activation
```

A goal:

```text
Goal
  ↓
Activation
```

They all eventually enter the same scheduling machinery.

# 4. Why this matters

Without a unified activation model, the runtime tends toward:

```text
TimerScheduler
MessageExecutor
ActionExecutor
ServiceExecutor
LifecycleExecutor
...
```

Each develops slightly different semantics.

Instead:

```text
             Event Sources
                  │
       ┌──────────┼──────────┐
       ▼          ▼          ▼
    Message     Timer      Goal
       │          │          │
       └──────────┼──────────┘
                  ▼
              Activation
                  │
                  ▼
              Scheduler
                  │
                  ▼
              Executor
```

One scheduling model.

# 5. Activation lifecycle

An activation can have its own state:

```text
CREATED
   │
   ▼
READY
   │
   ▼
SCHEDULED
   │
   ▼
RUNNING
   │
   ├──────────────┐
   ▼              ▼
COMPLETED      CANCELLED
   │
   ▼
RECORDED
```

If execution fails:

```text
RUNNING
   │
   ▼
FAILED
   │
   ├── retry
   ├── recover
   └── propagate
```

# 6. Activation identity

Every execution should be traceable.

For example:

```text
ActivationId = A-00000172
```

Then:

```text
A-00000172
│
├── cause: Message
├── source: /scan
├── target: localization
├── created: T1
├── scheduled: T2
├── started: T3
├── completed: T4
└── result: success
```

This provides a causal execution history.

# 7. Causality

Now NROS can model causality explicitly.

Suppose:

```text
LiDAR message
      │
      ▼
Localization activation
      │
      ▼
Pose update
      │
      ▼
Planner activation
      │
      ▼
Trajectory
      │
      ▼
Controller activation
```

The runtime can represent:

```text
A1 → A2 → A3 → A4
```

This is far more powerful than merely storing timestamps.

# 8. Causal graph

An execution trace can become:

```text
                 SensorEvent
                     │
                     ▼
                   A101
                     │
                 pose_update
                     │
                     ▼
                   A102
                     │
                 plan_ready
                     │
                     ▼
                   A103
                     │
                 trajectory
                     │
                     ▼
                   A104
```

This gives NROS an **execution provenance graph**.

# 9. Data dependency vs causal dependency

They are related but not identical.

### Data dependency

```text
A2 consumes output of A1
```

### Causal dependency

```text
A1 caused A2 to execute
```

Sometimes:

```text
A1 → A2
```

because a message arrived.

Other times:

```text
Timer → A2
```

even though no data dependency exists.

NROS should preserve both concepts.

# 10. Effects

Execution becomes interesting when components interact with the outside world.

Examples:

```text
MotorCommand
FileWrite
NetworkSend
GPIOChange
DatabaseUpdate
```

These are **effects**.

Conceptually:

```text
Activation
    │
    ▼
Compute
    │
    ▼
Effects
```

This creates a clean separation between:

```text
decision
```

and:

```text
external side effect
```

# 11. Effect representation

An effect can be represented conceptually as:

```text
Effect
│
├── effect_id
├── activation_id
├── resource
├── operation
├── payload
├── timestamp
└── result
```

Example:

```text
E-0042
activation = A-104
resource = MotorLeft
operation = SetVelocity
value = 0.4 m/s
```

Now the runtime knows not merely that a controller executed, but what external effect it attempted.

# 12. Why effects matter for safety

Suppose:

```text
Planner
   ↓
Controller
   ↓
Motor
```

The safety layer can intercept or validate effects:

```text
Controller
    │
    ▼
MotorEffect
    │
    ▼
Safety Policy
    │
    ├── ALLOW
    ├── MODIFY
    └── DENY
```

For example:

```text
requested velocity = 5 m/s
maximum allowed    = 1 m/s
```

The policy may reject the effect.

# 13. Capability + effect

The component needs two things:

```text
Capability
+
Effect
```

A capability says:

> You are authorized to interact with this resource.

The effect says:

> This is the operation you are attempting.

Thus:

```text
Component
    │
    ▼
Capability<Motor>
    │
    ▼
SetVelocity(0.5)
    │
    ▼
Policy
    │
    ▼
Motor
```

This is a powerful architectural boundary.

# 14. Pure vs effectful computation

NROS can distinguish:

```text
Pure computation
```

from:

```text
Effectful computation
```

Example:

```text
TrajectoryPlanner
    │
    ├── input trajectory
    └── output trajectory
```

can conceptually be pure.

Then:

```text
MotorController
    │
    └── MotorCommand
```

creates an external effect.

This distinction is useful for:

- deterministic testing
- replay
- simulation
- formal verification
- safety validation

# 15. Effect pipeline

A more complete execution pipeline becomes:

```text
INPUT
  │
  ▼
ACTIVATION
  │
  ▼
COMPUTATION
  │
  ▼
PROPOSED EFFECT
  │
  ▼
POLICY
  │
  ├── DENY
  ├── MODIFY
  └── ACCEPT
  │
  ▼
RESOURCE
  │
  ▼
OBSERVATION
```

This is an important foundation for NROS.

# 16. Observation closes the loop

The runtime should not assume that issuing an effect means it succeeded.

Example:

```text
Command:
    Motor = 0.5 m/s
```

doesn't prove:

```text
Actual motor velocity = 0.5 m/s
```

Therefore:

```text
Effect
   │
   ▼
External system
   │
   ▼
Observation
```

This enables closed-loop control.

# 17. The NROS control loop

The complete cycle becomes:

```text
┌──────────────────────────────┐
│                              │
│         OBSERVE              │
│            │                 │
│            ▼                 │
│          EVENT               │
│            │                 │
│            ▼                 │
│          PLAN                │
│            │                 │
│            ▼                 │
│         EXECUTE              │
│            │                 │
│            ▼                 │
│          EFFECT              │
│            │                 │
│            ▼                 │
│         OBSERVE              │
│                              │
└──────────────────────────────┘
```

This begins to align NROS naturally with autonomous-agent architectures.

# 18. Agent as a component

A conventional ROS component might be:

```text
LaserProcessor
```

An NROS autonomous component could be:

```text
NavigationAgent
```

with:

```text
Observe
   ↓
Plan
   ↓
Execute
   ↓
Reflect
   ↓
Checkpoint
```

But importantly, the agent still operates through the same runtime primitives.

It does **not** get an entirely separate runtime.

# 19. Agent execution

An agent activation could be:

```text
AgentActivation
│
├── observations
├── context
├── objective
├── constraints
├── available capabilities
└── deadline
```

The agent produces:

```text
Decision
│
├── actions
├── resource requests
├── sub-goals
└── checkpoint
```

The runtime validates those outputs.

# 20. NROS becomes suitable for agentic robotics

This creates:

```text
ROS
│
│   Robot middleware
▼
NROS
│
│   Deterministic runtime
│   + resources
│   + safety
│   + execution
│
▼
Agentic NROS
│
│   autonomous computation
│   represented as runtime work
▼
Robot / Edge / Embedded System
```

The key is that **agentic behavior becomes another workload type**, rather than something bolted onto the side.

# 21. Rust type architecture

At the API level, the concepts could eventually resemble:

```text
nros-core
├── Component
├── Activation
├── Event
├── Goal
├── Task
├── Effect
├── Resource
├── Capability
├── Lifecycle
├── Clock
└── Error
```

Then:

```text
nros-executor
├── Executor
├── Scheduler
├── Worker
├── Queue
└── Cancellation
```

And:

```text
nros-transport
├── Channel
├── Publisher
├── Subscriber
├── Service
└── Action
```

Then:

```text
nros-safety
├── Policy
├── Fault
├── Watchdog
├── Health
├── SafeState
└── Isolation
```

And:

```text
nros-observability
├── Trace
├── EventLog
├── Metrics
├── Replay
└── Diagnostics
```

# 22. The important architectural rule

These crates should not become isolated abstractions.

The dependency direction should remain controlled:

```text
                nros-core
              /     |      \
             /      |       \
            ▼       ▼        ▼
      executor  transport  safety
            \       |       /
             \      |      /
              ▼     ▼     ▼
             runtime
                 │
                 ▼
          observability
```

Application crates should depend on the stable runtime surface rather than internal implementation details.

# 23. Minimal execution kernel

The actual NROS kernel can remain surprisingly small.

Conceptually:

```text
                 NROS KERNEL
                     │
       ┌─────────────┼─────────────┐
       ▼             ▼             ▼
    Identity       Events       Resources
       │             │             │
       └─────────────┼─────────────┘
                     ▼
                 Activation
                     │
                     ▼
                 Scheduler
                     │
                     ▼
                 Executor
                     │
                     ▼
                  Effects
```

Everything else can be layered around this.

This follows the same philosophy that made ROS extensible:

> Keep the irreducible runtime core small; move specialized functionality into composable packages.

But NROS can make the **execution semantics** much more explicit.

# 24. NROS vs ROS: deeper comparison

| Dimension | ROS | NROS |
|---|---|---|
| Basic unit | Node | Component |
| Message trigger | Callback | Activation |
| Continuous data | Topic | Typed Channel |
| Request/response | Service | Service |
| Long-running work | Actionlib | Managed Goal |
| Scheduling | Executor/spinners | Explicit scheduler + executor |
| Time | ROS time | Clock abstraction |
| Resources | Mostly external conventions | First-class resources |
| Permissions | Limited/core-external | Capabilities |
| Failure | Node/process-oriented | Failure domains |
| Safety | Application-level | Runtime policy layer |
| Effects | Implicit | Explicit effect model |
| Traceability | rosbag/tools | Causal execution trace |
| Replay | Message replay | Execution + time + effect replay |
| Determinism | Limited/implementation dependent | Explicit runtime objective |
| Agent execution | External integration | Native workload model |

# 25. The deepest conceptual shift

ROS primarily models:

```text
DATA FLOW
```

NROS should model:

```text
DATA FLOW
+
CONTROL FLOW
+
RESOURCE FLOW
+
TIME
+
EFFECTS
+
FAILURE
+
CAUSALITY
```

That is the real transformation.

# 26. NROS as a runtime graph

The traditional ROS graph:

```text
Node ── Topic ── Node
```

evolves into:

```text
             ┌──────── TIME ────────┐
             │                       │
             ▼                       │
        ACTIVATION                   │
             │                       │
             ▼                       │
Component ─ EXECUTION ─ Resource     │
    │          │             │       │
    │          ▼             │       │
    │        EFFECT          │       │
    │          │             │       │
    ▼          ▼             ▼       │
 Channel ─── Observation ─ Policy ───┘
```

The runtime graph is therefore no longer merely a communication graph.

It is an **execution graph**.

# 27. This leads directly to the NROS specification

We can now formulate the major NROS specification layers:

```text
NROS Specification
│
├── 1. Identity Model
├── 2. Component Model
├── 3. Communication Model
├── 4. Execution Model
├── 5. Scheduling Model
├── 6. Time Model
├── 7. Resource Model
├── 8. Capability Model
├── 9. Lifecycle Model
├── 10. Goal Model
├── 11. Fault Model
├── 12. Safety Model
├── 13. Effect Model
├── 14. Observability Model
├── 15. Replay Model
└── 16. Deployment Model
```

That specification should precede aggressive API growth.

# 28. The next major problem: transport

Everything we've defined so far can theoretically operate:

```text
in-process
```

but robotics systems are inherently distributed:

```text
Robot Computer
      │
      ├── Sensors
      ├── Controllers
      └── Local NROS
             │
             │ network
             ▼
        Edge Computer
             │
             ├── Vision
             ├── Planning
             └── AI
             │
             ▼
          Cloud
```

Therefore the next major layer is:

# **NROS Distributed Transport & Discovery**

This is where we transform the ROS concepts of **ROS Master, DDS discovery, topics, QoS, serialization, and distributed nodes** into an NROS-native transport architecture.

The critical question will be:

> **Can NROS preserve the same execution semantics when a component moves from the same process, to another process, to another machine, or to an embedded controller?**

That is the boundary between a **Rust robotics framework** and a genuine **distributed robotics runtime**.
