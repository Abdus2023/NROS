# NROS Component & Resource (Part XXVIII–XXX)

The **Component** should become the fundamental deployable and executable unit of NROS.

ROS starts with:

> **Node = process participating in the computation graph.**

NROS should move toward:

> **Component = a governed runtime entity possessing identity, lifecycle, state, capabilities, interfaces, resources, and execution behavior.**

## 1. Node → Component

ROS:

```text
Node
├── Publishers
├── Subscribers
├── Services
└── Parameters
```

NROS:

```text
Component
├── Identity
├── Lifecycle
├── State
├── Interfaces
├── Capabilities
├── Resources
├── Policy
├── Activations
├── Persistence
└── Observability
```

The important change is that communication is no longer the component's definition.

Communication becomes **one capability of a component**.

# 2. Component identity

Every component needs a stable identity.

Conceptually:

```text
ComponentId
├── namespace
├── name
├── instance
├── version
└── identity metadata
```

Example:

```text
robot://alpha/navigation/planner/instance-01
```

Identity should remain stable enough for:

```text
discovery
authorization
tracing
state ownership
resource ownership
checkpointing
```

# 3. Component instance vs component type

Separate:

```text
ComponentType
```

from:

```text
ComponentInstance
```

For example:

```text
NavigationPlanner
      │
      ├── planner-alpha
      ├── planner-beta
      └── planner-simulation
```

The type describes behavior.

The instance describes a concrete runtime entity.

# 4. Component manifest

A component should declare its contract.

Conceptually:

```yaml
component:
  name: navigation-planner
  version: 1.2.0

interfaces:
  consumes:
    - Pose
    - Map
    - NavigationGoal

  produces:
    - Path
    - NavigationStatus

capabilities:
  - planning
  - replanning

resources:
  cpu:
    min: 1
  memory:
    min: 128MiB
```

This becomes machine-readable runtime metadata.

# 5. Component lifecycle

ROS node startup is relatively simple.

NROS should expose a richer lifecycle:

```text
CREATED
   ↓
DISCOVERING
   ↓
INITIALIZING
   ↓
READY
   ↓
ACTIVE
   ↓
QUIESCING
   ↓
STOPPED
```

Failure paths:

```text
INITIALIZING → FAILED
ACTIVE       → DEGRADED
ACTIVE       → FAILED
```

# 6. Lifecycle transitions are guarded

A component must not arbitrarily transition from:

```text
CREATED
```

to:

```text
ACTIVE
```

without satisfying prerequisites.

For example:

```text
INITIALIZING
      │
      ├── configuration valid?
      ├── dependencies available?
      ├── capabilities granted?
      ├── resources allocated?
      ├── state restored?
      └── safety checks passed?
             │
             ▼
           READY
```

This follows the NROS principle:

> **No observed prerequisite → no valid state transition.**

# 7. READY vs ACTIVE

These should be distinct.

### READY

The component can execute.

### ACTIVE

The component is currently participating in operational execution.

Example:

```text
Planner
   ↓
READY
```

but not necessarily:

```text
ACTIVE
```

until a mission requires it.

This enables resource-efficient systems.

# 8. Quiescing

A component should be able to stop accepting new work while finishing existing work.

```text
ACTIVE
   │
   ▼
QUIESCING
   │
   ├── finish activation A
   ├── finish activation B
   └── reject new work
          │
          ▼
       STOPPED
```

This is essential for safe shutdown and upgrades.

# 9. Component state machine

A complete conceptual model:

```text
                  ┌────────────┐
                  │   CREATED  │
                  └─────┬──────┘
                        ▼
                  ┌────────────┐
                  │ DISCOVERING│
                  └─────┬──────┘
                        ▼
                  ┌────────────┐
                  │INITIALIZING│
                  └─────┬──────┘
                        │
              ┌─────────┴─────────┐
              ▼                   ▼
           FAILED               READY
                                  │
                                  ▼
                               ACTIVE
                              /      \
                             /        \
                       DEGRADED    QUIESCING
                           │            │
                           ▼            ▼
                        RECOVERING    STOPPED
                           │
                           └──→ ACTIVE
```

# 10. Component capabilities

A component may advertise capabilities such as:

```text
planning
localization
perception
manipulation
navigation
simulation
diagnostics
reasoning
```

But capabilities should be **typed contracts**, not merely strings.

# 11. Capability descriptor

Conceptually:

```text
Capability
├── id
├── version
├── inputs
├── outputs
├── prerequisites
├── resource requirements
├── timing requirements
├── security requirements
└── implementation
```

Example:

```text
Capability:
    navigation.plan

Input:
    Pose
    Goal
    Map

Output:
    Path

Deadline:
    50ms
```

# 12. Capability discovery

A planner can ask:

```text
Who can provide navigation.plan?
```

NROS returns:

```text
planner-A
planner-B
remote-planner
```

The runtime can then select an implementation.

This makes components replaceable.

# 13. Capability invocation

Instead of coupling directly to a node:

```text
call planner-A
```

the system can request:

```text
invoke capability:
navigation.plan
```

The runtime chooses an eligible provider.

This is a major architectural upgrade.

# 14. Capability selection

Selection can consider:

```text
capability compatibility
location
latency
resource availability
trust
cost
load
deadline
priority
```

Example:

```text
Planner A
latency = 8ms
load = 90%

Planner B
latency = 14ms
load = 20%
```

For a relaxed deadline, B may be preferable.

# 15. Component interfaces

Interfaces should be explicit.

```text
Component
│
├── Input Interfaces
├── Output Interfaces
├── Request Interfaces
├── Command Interfaces
├── State Interfaces
└── Capability Interfaces
```

This makes the component contract inspectable.

# 16. Input interface

An input interface defines what the component consumes.

```text
Input
├── schema
├── QoS
├── deadline
├── buffering
└── validation
```

Example:

```text
PoseEstimate
QoS = Reliable
Deadline = 20ms
```

# 17. Output interface

An output defines:

```text
schema
rate
QoS
priority
retention
```

Example:

```text
Path
rate = on-demand
reliability = reliable
```

# 18. Resource ownership

A component may own resources:

```text
CPU
Memory
GPU
NPU
Network
Sensors
Actuators
File handles
Storage
Locks
Device interfaces
```

Resource ownership must be explicit.

# 19. Resource claims

A component can declare:

```text
CPU:
    min = 0.5 core
    max = 2 cores

Memory:
    min = 128 MB
    max = 512 MB
```

The scheduler can then make placement decisions.

# 20. Resource reservations

Some components require guarantees.

A motor controller might require:

```text
CPU reservation
deadline guarantee
memory reservation
exclusive device access
```

This moves NROS toward deterministic robotics execution.

# 21. Resource capabilities

A component should not automatically gain access to every resource.

For example:

```text
Camera
   ↓
Capability token
   ↓
Vision component
```

Likewise:

```text
Motor
   ↓
Actuation capability
   ↓
Controller
```

# 22. Exclusive resources

Certain resources cannot safely be shared.

Example:

```text
/robot/A/motor_controller
```

could have:

```text
ownership = exclusive
```

Only one component can hold the active control lease.

# 23. Resource leases

Resource ownership should use leases:

```text
Component A
    │
    ▼
Acquire(Motor)
    │
    ▼
Lease
    │
    ├── renew
    └── expire
```

If the component dies:

```text
lease expires
      ↓
resource recovered
```

This is critical for actuator safety.

# 24. Component supervision

Every important component should have a supervisor.

```text
Supervisor
    │
    ├── observe
    ├── health-check
    ├── restart
    ├── isolate
    ├── recover
    └── terminate
```

This replaces the simplistic assumption that process failure is the only failure model.

# 25. Health model

Health should be multidimensional:

```text
Health
├── process
├── communication
├── state
├── resource
├── dependency
├── capability
└── safety
```

A component can be:

```text
process = healthy
communication = healthy
resource = constrained
```

and therefore:

```text
overall = DEGRADED
```

# 26. Dependency graph

Components can declare dependencies:

```text
Planner
 ├── Localization
 ├── Map
 └── Transform
```

The supervisor can then understand:

```text
Planner unavailable
because
Localization unavailable
```

rather than treating Planner failure as an isolated event.

# 27. Dependency semantics

Dependencies can be:

```text
REQUIRED
OPTIONAL
DEGRADABLE
CONDITIONAL
```

Example:

```text
GPU perception
   = OPTIONAL
```

If GPU disappears:

```text
CPU fallback
```

may be selected.

# 28. Failure domains

NROS should group components into failure domains.

```text
Robot
│
├── Control Domain
├── Perception Domain
├── Planning Domain
└── UI Domain
```

A failure in UI should not necessarily affect control.

# 29. Isolation

Components may run:

```text
same process
different process
different container
different machine
different robot
```

without changing their semantic interface.

This is one of the key benefits of the NROS model.

# 30. In-process components

For extremely low latency:

```text
Component A
     │
     ▼
Component B
```

may execute in the same address space.

Possible advantages:

```text
zero-copy
shared memory
lower latency
```

But isolation is weaker.

# 31. Process isolation

For stronger fault containment:

```text
Process A
   │
 IPC
   │
Process B
```

A crash in B does not necessarily corrupt A.

NROS should allow the deployment layer to choose this.

# 32. Remote components

The same component model can span machines:

```text
Robot Computer
      │
      │ secure transport
      ▼
Edge Computer
      │
      ▼
Cloud
```

The component contract remains stable.

# 33. Component placement

Placement can become an optimization problem:

```text
Component
    │
    ├── CPU requirement
    ├── GPU requirement
    ├── latency requirement
    ├── data locality
    ├── security
    └── availability
```

The runtime chooses:

```text
Machine A
Machine B
Machine C
```

# 34. Data locality

Suppose camera data is 500 MB/s.

Moving it:

```text
Camera → network → GPU
```

may be wasteful.

NROS can instead place perception near the camera/GPU.

Thus:

> **Data locality becomes a scheduling constraint.**

# 35. Component migration

Some components may be migratable:

```text
Component A
    │
    ▼
Checkpoint
    │
    ▼
Transfer
    │
    ▼
Restore
    │
    ▼
Component B
```

This requires integration between:

```text
State Fabric
Execution Fabric
Communication Fabric
```

# 36. Component checkpoint

A checkpoint should contain enough information to reconstruct execution:

```text
ComponentCheckpoint
├── component identity
├── version
├── lifecycle state
├── state revision
├── active operations
├── resource metadata
├── pending messages
└── recovery metadata
```

Not every implementation must persist all fields.

The checkpoint policy determines what is required.

# 37. Hot upgrade

A component can be upgraded:

```text
v1
 │
 ├── checkpoint
 │
 ▼
v2
 │
 ├── restore
 │
 ├── validate
 │
 └── activate
```

The runtime should reject the upgrade if the new component cannot consume the checkpoint schema.

# 38. Compatibility

Compatibility should cover:

```text
API
schema
capabilities
state
protocol
configuration
checkpoint
```

Therefore:

```text
version = 2.0
```

alone is insufficient information.

# 39. Component contract

We can now define the conceptual contract:

```text
ComponentContract
├── Identity
├── Version
├── Interfaces
├── Capabilities
├── State Schema
├── Resource Requirements
├── Lifecycle
├── Dependencies
├── Security Policy
├── Timing Requirements
└── Recovery Policy
```

This becomes a central NROS artifact.

# 40. Component registration

At startup:

```text
Component
   │
   ▼
Register
   │
   ├── identity
   ├── capabilities
   ├── interfaces
   ├── resources
   └── lifecycle
   │
   ▼
NROS Registry
```

Registration should be validated rather than blindly accepted.

# 41. Registry

The registry can answer:

```text
What components exist?
What capabilities exist?
Where are they?
What versions are active?
What state do they expose?
What resources do they hold?
```

But unlike ROS 1's Master, the registry should not necessarily sit in the critical data path.

# 42. Discovery vs registry

Separate:

```text
Discovery
```

from:

```text
Registry
```

Discovery answers:

> **Who is currently reachable?**

Registry answers:

> **What entities and capabilities are known?**

This separation makes distributed operation more robust.

# 43. Component events

Components should emit lifecycle events:

```text
ComponentCreated
ComponentReady
ComponentActivated
ComponentDegraded
ComponentFailed
ComponentRecovered
ComponentStopped
```

These events can feed:

```text
monitoring
audit
automation
agents
```

# 44. Components as observable entities

The NROS graph therefore becomes:

```text
Component
   │
   ├── state
   ├── capabilities
   ├── resources
   ├── interfaces
   ├── activations
   └── lifecycle
```

This makes the runtime graph semantically rich.

# 45. Component → Activation

A component does not necessarily run continuously.

Instead:

```text
Component
   │
   ├── Activation A
   ├── Activation B
   └── Activation C
```

An activation represents a unit of execution.

This is a critical distinction.

# 46. Activation

Conceptually:

```text
Activation
├── id
├── component
├── trigger
├── inputs
├── state context
├── deadline
├── priority
├── resources
├── execution
└── effects
```

The scheduler can reason about activations independently.

# 47. Trigger

An activation can be triggered by:

```text
Event
Message
Timer
State change
Command
Dependency completion
Agent decision
External signal
```

Therefore:

```text
trigger → activation
```

becomes a universal execution bridge.

# 48. Activation lifecycle

```text
CREATED
   ↓
READY
   ↓
SCHEDULED
   ↓
RUNNING
   ↓
COMPLETED
```

Failure:

```text
RUNNING
   ├── FAILED
   ├── CANCELLED
   └── DEADLINE_MISSED
```

# 49. Activation context

An activation should know:

```text
who triggered me?
what state revision am I using?
what deadline applies?
what capabilities do I have?
what resources are reserved?
what is my parent activation?
```

This gives NROS a causal execution model.

# 50. Parent-child activations

Example:

```text
Mission Activation
       │
       ├── Planning Activation
       │       ├── Map Query
       │       └── Path Optimization
       │
       └── Navigation Activation
               └── Control Activation
```

This forms an execution tree.

# 51. Execution tree + state + communication

The same operation can now be represented through three dimensions:

```text
                 Mission
                    │
          ┌─────────┼─────────┐
          ▼         ▼         ▼
       State     Activation Communication
```

Together they produce a complete causal picture.

# 52. NROS component architecture

```text
                         COMPONENT
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
     Identity              State             Capabilities
        │                    │                    │
        └────────────────────┼────────────────────┘
                             │
                         Interfaces
                             │
                    ┌────────┴────────┐
                    ▼                 ▼
               Activations         Resources
                    │                 │
                    └────────┬────────┘
                             ▼
                         Execution
                             │
                             ▼
                           Effects
                             │
              ┌──────────────┴──────────────┐
              ▼                             ▼
        Communication                    State Update
```

# 53. The deeper transformation

The ROS abstraction:

```text
Node
```

was designed primarily around **distributed software processes**.

The NROS abstraction:

```text
Component
```

is designed around **governed computational entities**.

That distinction allows NROS to support:

```text
robot controllers
drivers
services
AI agents
planners
simulators
edge workloads
remote capabilities
autonomous workflows
```

using the same runtime model.

# 54. ROS package → NROS component bundle

ROS packages primarily organize software.

NROS should additionally describe deployability.

A bundle might contain:

```text
nros-component/
├── manifest
├── executable
├── schemas
├── capabilities
├── policies
├── configuration
├── migration
├── tests
└── metadata
```

The package becomes a **deployable semantic unit**.

# 55. Toward the NROS application model

A complete application could therefore look like:

```text
NROS Application
│
├── Components
│   ├── Perception
│   ├── Localization
│   ├── Planning
│   └── Control
│
├── State
│   ├── Robot
│   ├── World
│   └── Mission
│
├── Capabilities
│   ├── Navigate
│   ├── Manipulate
│   └── Inspect
│
├── Policies
│   ├── Safety
│   ├── Security
│   └── Resource
│
└── Runtime
    ├── Scheduler
    ├── Supervisor
    └── Transport
```

# 56. The next missing layer: Resources

We have defined components, but one major question remains:

> **How does NROS represent CPU, memory, accelerators, devices, network bandwidth, actuators, locks, and other resources as schedulable runtime objects?**

That becomes **Part XXIX — NROS Resource & Device Fabric**.

The target architecture is:

```text
                     NROS
                      │
       ┌──────────────┼──────────────┐
       ▼              ▼              ▼
   Components      State          Resources
       │              │              │
       └──────────────┼──────────────┘
                      ▼
                 Activations
                      │
                      ▼
                  Scheduler
                      │
          ┌───────────┼───────────┐
          ▼           ▼           ▼
         CPU         GPU        Devices
                                  │
                         ┌────────┴────────┐
                         ▼                 ▼
                      Sensors          Actuators
```

This is where NROS moves from a **middleware architecture** toward a genuine **robotics runtime architecture**: resources themselves become observable, leaseable, schedulable, and policy-controlled entities.

# NROS — Part XXIX: Resource & Device Fabric

The next architectural step is to make **resources first-class runtime entities**.

ROS generally treats hardware and computational resources through nodes, drivers, parameters, and operating-system facilities.

NROS should instead expose a unified model:

> **A resource is something an execution unit may observe, consume, reserve, own, share, control, or release.**

That includes both computational resources and physical resources.

# 1. Resource taxonomy

```text
NROS Resources
│
├── Compute
│   ├── CPU
│   ├── GPU
│   ├── NPU
│   └── Accelerator
│
├── Memory
│   ├── RAM
│   ├── Shared Memory
│   ├── Device Memory
│   └── Persistent Storage
│
├── Network
│   ├── Bandwidth
│   ├── Interfaces
│   └── Channels
│
├── Device
│   ├── Sensor
│   ├── Actuator
│   ├── Camera
│   ├── LiDAR
│   └── IMU
│
└── Logical
    ├── Lock
    ├── Lease
    ├── Capability
    └── Execution Slot
```

# 2. Why resources need a common model

Consider these two operations:

```text
GPU
```

and:

```text
Robot arm
```

They look completely different.

But from the runtime's perspective both require:

```text
discover
authorize
reserve
use
monitor
release
recover
```

That common lifecycle is what NROS should model.

# 3. Resource identity

Every resource should have a stable identity:

```text
ResourceId
```

Example:

```text
robot://alpha/device/lidar/front
robot://alpha/device/motor/left
host://alpha/compute/gpu/0
host://alpha/memory/shared/0
```

This gives resources a position in the runtime graph.

# 4. Resource descriptor

Conceptually:

```text
Resource
├── id
├── type
├── capabilities
├── capacity
├── availability
├── owner
├── policy
├── health
└── metadata
```

# 5. Capacity

A resource may have measurable capacity.

CPU:

```text
4 cores
```

GPU:

```text
8 GB VRAM
```

Network:

```text
1 Gbit/s
```

Battery:

```text
80 Wh
```

The scheduler can reason about these quantities.

# 6. Resource units

NROS should avoid arbitrary units where possible.

Examples:

```text
CPU → cores / utilization
Memory → bytes
Storage → bytes
Network → bits/s
Power → watts
Energy → joules / Wh
```

Device-specific resources can define domain-specific units.

# 7. Resource availability

Capacity and availability are different.

Example:

```text
GPU capacity = 100%
GPU available = 25%
```

because:

```text
75%
```

is currently reserved.

Therefore:

```text
capacity ≠ availability
```

# 8. Resource state

A resource should have lifecycle/state:

```text
UNKNOWN
DISCOVERED
AVAILABLE
RESERVED
ACTIVE
DEGRADED
FAILED
RECOVERING
OFFLINE
```

For example:

```text
LiDAR
   ↓
AVAILABLE
   ↓
RESERVED
   ↓
ACTIVE
```

# 9. Resource ownership

Resources can have:

```text
owner = component
```

or:

```text
owner = none
```

For shared resources:

```text
owners = [A, B, C]
```

depending on the resource policy.

# 10. Resource sharing

Sharing modes:

```text
EXCLUSIVE
SHARED
PARTITIONED
TIME_SLICED
VIRTUALIZED
```

Examples:

```text
Motor → EXCLUSIVE
CPU → TIME_SLICED
GPU → PARTITIONED
Camera → SHARED
```

# 11. Resource lease

A resource reservation should usually be lease-based.

```text
Component A
     │
     ▼
Acquire(Resource)
     │
     ▼
Lease
     │
 ┌───┴────┐
 ▼        ▼
Renew   Release
```

If the component crashes:

```text
lease expires
     ↓
resource recovered
```

# 12. Lease safety

A lease should contain:

```text
LeaseId
ResourceId
Owner
IssuedAt
ExpiresAt
Generation
Capabilities
```

The **generation** prevents an old owner from accidentally controlling a resource after ownership has changed.

# 13. Device resource

A physical device is more than an object file.

Example:

```text
Front LiDAR
│
├── Identity
├── Driver
├── Transport
├── Configuration
├── State
├── Capabilities
├── Health
└── Data Interfaces
```

This creates a unified device abstraction.

# 14. Device capabilities

A camera might expose:

```text
camera.capture
camera.configure
camera.stream
camera.calibrate
```

A motor controller:

```text
motor.enable
motor.set_velocity
motor.set_position
motor.stop
```

Capabilities become the interface between software and hardware.

# 15. Hardware abstraction

NROS should preserve hardware abstraction, but avoid hiding capabilities that matter.

Instead of:

```text
generic_device.write()
```

prefer:

```text
motor.set_velocity(...)
```

while the underlying implementation remains hardware-specific.

# 16. Driver architecture

A device driver becomes a component:

```text
                NROS
                 │
             Device API
                 │
             Driver Component
                 │
          ┌──────┴──────┐
          ▼             ▼
       Transport     Hardware
```

The driver therefore participates in:

```text
lifecycle
state
security
resource management
supervision
tracing
```

just like any other component.

# 17. Driver failure

If the driver fails:

```text
Driver
  X
  │
  ▼
Device
```

NROS should distinguish:

```text
device physically failed
```

from:

```text
driver software failed
```

and:

```text
transport failed
```

These are different failure domains.

# 18. Device state hierarchy

For a motor:

```text
Motor
│
├── device state
│   ├── connected
│   └── temperature
│
├── control state
│   ├── enabled
│   └── mode
│
└── operational state
    ├── velocity
    └── position
```

This allows precise diagnosis.

# 19. Sensors

A sensor is both:

```text
Resource
```

and:

```text
Data producer
```

For example:

```text
LiDAR
   │
   ├── Resource
   ├── Configuration
   ├── Health
   └── Stream<PointCloud>
```

The resource model governs access.

The communication fabric carries the data.

# 20. Actuators

Actuators are more sensitive.

A motor is:

```text
Resource
+
Command target
+
Safety boundary
```

Therefore actuator control should require stronger policy.

# 21. Actuation authorization

A component may be authorized to:

```text
READ motor.state
```

without being authorized to:

```text
COMMAND motor
```

And a diagnostic component might have:

```text
READ
```

only.

This creates a clean security boundary.

# 22. Safety capabilities

Actuators can expose safety-specific capabilities:

```text
motor.safe_stop
motor.disable
motor.reset_fault
```

These may have higher priority than ordinary commands.

For example:

```text
normal command
      ↓
priority = 50

safe stop
      ↓
priority = 1000
```

The scheduler and resource layer must understand this distinction.

# 23. Resource priority

Resources can have priority classes:

```text
CRITICAL
HIGH
NORMAL
LOW
BACKGROUND
```

A safety-critical controller should outrank:

```text
logging
visualization
telemetry
```

when resources become constrained.

# 24. Resource preemption

Some resources can be preempted.

Example:

```text
GPU
│
├── Vision workload
└── Training workload
```

If an urgent perception task arrives:

```text
Training
   ↓
SUSPEND
   ↓
GPU
   ↓
Perception
```

But a physical actuator generally should not be preempted in the same simplistic way.

Therefore each resource declares a **preemption policy**.

# 25. Preemption policies

```text
NONE
COOPERATIVE
SAFE_POINT
FORCED
EMERGENCY
```

Examples:

```text
Motor control → SAFE_POINT
GPU workload → COOPERATIVE
Safety override → EMERGENCY
```

# 26. Resource budgets

A component may receive a budget:

```text
CPU:
200 ms / second

Network:
20 MB / second

Storage:
100 MB / hour
```

The scheduler can enforce these budgets.

# 27. Resource accounting

NROS should record:

```text
CPU consumed
memory allocated
network transferred
GPU time
device utilization
energy consumed
```

This enables:

```text
optimization
billing
admission control
diagnostics
fleet management
```

# 28. Energy as a resource

Robotics makes energy especially important.

Battery state:

```text
energy.available
```

becomes a resource constraint.

A task may require:

```text
estimated_energy = 120 Wh
```

while:

```text
available_energy = 90 Wh
```

The scheduler should reject or modify the plan.

# 29. Energy-aware scheduling

The runtime can choose:

```text
Plan A
high CPU
high energy

Plan B
low CPU
low energy
```

depending on:

```text
deadline
mission priority
battery state
```

This makes resource scheduling mission-aware.

# 30. Resource dependencies

Resources themselves can depend on others.

Example:

```text
Camera
   ↓
USB controller
   ↓
CPU
   ↓
Power
```

If power becomes unavailable:

```text
Camera unavailable
```

This should be represented in the resource graph.

# 31. Resource graph

We therefore obtain:

```text
Resource Graph

Power
 │
 ├── CPU
 │    ├── Planner
 │    └── Perception
 │
 ├── GPU
 │    └── Vision
 │
 └── Sensors
      ├── Camera
      └── LiDAR
```

This graph can be queried by the scheduler.

# 32. Device topology

NROS should represent physical topology:

```text
Robot
│
├── Compute Unit
│   ├── CPU
│   ├── GPU
│   └── NPU
│
├── Sensor Bus
│   ├── Camera
│   ├── LiDAR
│   └── IMU
│
└── Actuator Bus
    ├── Motor L
    ├── Motor R
    └── Arm
```

This is useful for placement and diagnostics.

# 33. Bus as resource

A bus can itself be a resource:

```text
CAN Bus
├── bandwidth
├── latency
├── arbitration
└── connected devices
```

NROS can therefore reason about bus saturation.

# 34. Device discovery

Hardware discovery should produce resource records:

```text
Device discovered
      ↓
Resource descriptor
      ↓
Driver matching
      ↓
Capability registration
      ↓
Health check
      ↓
AVAILABLE
```

# 35. Driver matching

A device may expose identifiers:

```text
vendor
model
protocol
version
```

The runtime can select an appropriate driver.

Conceptually:

```text
Device
  │
  ▼
Driver Registry
  │
  ▼
Compatible Driver
```

# 36. Dynamic devices

Devices may appear and disappear.

For example:

```text
USB Camera
   ↓
CONNECTED
   ↓
AVAILABLE
   ↓
DISCONNECTED
```

NROS should propagate this through the resource/state/communication fabrics.

# 37. Hot-plug propagation

A hot-plug event becomes:

```text
DeviceConnected
      ↓
ResourceCreated
      ↓
DriverStarted
      ↓
CapabilityAvailable
      ↓
Consumers notified
```

Likewise on removal:

```text
DeviceDisconnected
      ↓
Capability revoked
      ↓
Resource released
      ↓
Dependent components degraded
```

# 38. Capability revocation

This is especially important.

If:

```text
Camera capability
```

is revoked, consumers should not continue assuming it exists.

The runtime can transition them:

```text
ACTIVE
  ↓
DEGRADED
  ↓
FALLBACK
```

if an alternative capability exists.

# 39. Resource-aware planning

The planner should be able to query:

```text
Can I execute this plan?
```

before committing.

The runtime evaluates:

```text
required capabilities
+
resource requirements
+
deadlines
+
policy
+
availability
```

# 40. Admission control

This creates an explicit admission stage:

```text
Request
   ↓
Validate
   ↓
Authorize
   ↓
Estimate resources
   ↓
Check availability
   ↓
Reserve
   ↓
Accept
```

Only then:

```text
Execute
```

# 41. Resource reservation

A plan can reserve resources before execution:

```text
Navigation Mission
│
├── CPU reservation
├── Localization capability
├── LiDAR
├── Map
└── Controller
```

This prevents the runtime from starting work it cannot sustain.

# 42. Reservation failure

If resources cannot be reserved:

```text
Mission
   ↓
ADMISSION
   ↓
REJECTED
```

rather than:

```text
Mission
   ↓
RUNNING
   ↓
resource unavailable
   ↓
FAIL
```

Early rejection is much safer.

# 43. Partial admission

Some tasks can degrade.

Example:

```text
Mission
├── obstacle detection — REQUIRED
├── object recognition — OPTIONAL
└── video recording — OPTIONAL
```

If GPU is unavailable:

```text
Obstacle detection → admitted
Object recognition → disabled
Recording → disabled
```

The mission can continue.

# 44. Resource profiles

Components can declare profiles:

```text
BEST_EFFORT
NORMAL
REAL_TIME
SAFETY_CRITICAL
ENERGY_SAVER
```

Each profile changes scheduling/resource requirements.

# 45. Real-time profile

A real-time component might declare:

```text
period = 1ms
deadline = 1ms
jitter_max = 100µs
priority = critical
```

The runtime can determine whether the environment can satisfy it.

Crucially:

> **NROS should never claim a real-time guarantee merely because a component requested one.**

The runtime must verify the prerequisite environment.

# 46. Real-time admission

Conceptually:

```text
Requested:
deadline = 1ms

Runtime checks:
├── scheduler capability
├── CPU isolation
├── memory locking
├── transport latency
├── interrupt behavior
└── executor guarantees
```

If these are not satisfied:

```text
REAL_TIME GUARANTEE = NOT ESTABLISHED
```

This preserves the evidence-driven architecture we've been building.

# 47. Resource evidence

Resource guarantees should be evidence-backed:

```text
ResourceClaim
├── requirement
├── observed capability
├── verification
├── timestamp
└── validity
```

For example:

```text
CPU isolation:
OBSERVED

deadline guarantee:
NOT VERIFIED
```

The system should not silently promote this to:

```text
VERIFIED
```

# 48. Resource failure

Resources can fail independently:

```text
CPU overload
GPU failure
memory exhaustion
network congestion
sensor failure
actuator fault
power shortage
```

The supervisor should correlate these failures with affected components.

# 49. Failure propagation

Example:

```text
GPU failure
   │
   ├── Vision component
   │       ↓
   │    DEGRADED
   │
   ├── Object detector
   │       ↓
   │    FALLBACK CPU
   │
   └── Logger
           ↓
        unaffected
```

The runtime therefore performs **failure containment**, not merely failure detection.

# 50. Resource recovery

A resource recovery workflow:

```text
FAILED
  ↓
ISOLATE
  ↓
RESET
  ↓
REINITIALIZE
  ↓
VALIDATE
  ↓
AVAILABLE
```

Only after validation should the resource become usable again.

# 51. Device reset safety

For actuators, reset may be dangerous.

Therefore:

```text
Motor FAILED
```

should not automatically imply:

```text
RESET
```

The safety policy may require:

```text
physical safe state
+
operator authorization
+
mechanical conditions
```

before reset.

# 52. Resource policies

Each resource can define:

```text
ownership policy
sharing policy
preemption policy
security policy
safety policy
recovery policy
```

This makes resource behavior explicit.

# 53. Unified resource API

Conceptually:

```rust
resource.discover()
resource.describe()
resource.reserve()
resource.acquire()
resource.release()
resource.observe()
resource.health()
```

Device-specific capabilities sit above this generic layer.

# 54. Device capability API

For example:

```rust
motor.command(...)
camera.capture(...)
lidar.configure(...)
```

while all still inherit:

```text
resource lifecycle
resource policy
resource ownership
resource health
```

# 55. Resource Fabric architecture

```text
                    RESOURCE FABRIC
                          │
                    Resource Registry
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
      Compute           Memory            Devices
        │                 │                 │
    CPU/GPU/NPU       RAM/Storage       Sensors/Actuators
        │                 │                 │
        └─────────────────┼─────────────────┘
                          ▼
                    Resource Manager
                          │
                 ┌────────┴────────┐
                 ▼                 ▼
             Reservations        Leases
                 │                 │
                 └────────┬────────┘
                          ▼
                      Scheduler
```

# 56. The four NROS fabrics

We now have:

```text
                         NROS
                          │
      ┌───────────────────┼───────────────────┐
      ▼                   ▼                   ▼
Communication          State              Resource
   Fabric              Fabric               Fabric
      │                   │                   │
      └───────────────────┼───────────────────┘
                          ▼
                      Execution
                         Fabric
```

More accurately, execution sits across all three:

```text
Communication ──────┐
                     │
State ───────────────┼──→ Execution
                     │
Resources ───────────┘
```

# 57. The runtime control loop

An NROS operation now becomes:

```text
REQUEST
   │
   ▼
DISCOVER
   │
   ▼
AUTHORIZE
   │
   ▼
ADMIT
   │
   ▼
RESERVE RESOURCES
   │
   ▼
CREATE ACTIVATION
   │
   ▼
SCHEDULE
   │
   ▼
EXECUTE
   │
   ▼
COMMIT EFFECT
   │
   ▼
UPDATE STATE
   │
   ▼
PUBLISH EVENT
   │
   ▼
OBSERVE / TRACE
```

This is much richer than:

```text
message → callback
```

# 58. Why this matters for NROS

ROS's elegance came from keeping the middleware relatively lightweight.

NROS should preserve that composability while adding explicit semantics around:

```text
identity
state
execution
resources
capabilities
policy
timing
safety
recovery
```

The result is not simply:

> **ROS but written in Rust.**

It is:

> **A resource-aware runtime fabric for distributed robotic and autonomous computation.**

# 59. NROS architecture at this point

```text
                           NROS
                            │
       ┌────────────────────┼────────────────────┐
       │                    │                    │
       ▼                    ▼                    ▼
  Communication          Components           State
     Fabric                Model              Fabric
       │                    │                    │
       │                    ▼                    │
       │                Activations              │
       │                    │                    │
       └────────────────────┼────────────────────┘
                            │
                            ▼
                      Resource Fabric
                            │
             ┌──────────────┼──────────────┐
             ▼              ▼              ▼
            CPU            GPU           Devices
                            │
                            ▼
                         Scheduler
                            │
                            ▼
                         Effects
```

# 60. Next: Part XXX — NROS Scheduler & Execution Fabric

We now have enough foundations to define the **actual execution engine**.

The next question is:

> **Given components, activations, deadlines, state, capabilities, and resources, how does NROS decide what executes, when, where, with what priority, and under what guarantees?**

That leads to the core scheduler model:

```text
                    NROS EXECUTION
                         │
                    Admission
                         │
                    Activation
                         │
                    Scheduling
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
       Priority       Deadline        Resource
       Policy          Policy          Policy
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                     Executor
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
             CPU        GPU       Remote
                         │
                         ▼
                       Effect
                         │
                         ▼
                   State + Trace
```

And this is where NROS's most important departure from ROS becomes explicit:

**ROS schedules callbacks/processes.**

**NROS schedules governed activations under temporal, resource, state, capability, and policy constraints.**

# NROS — Part XXX: Scheduler & Execution Fabric

We now reach the **execution core**.

The previous layers answer:

- **What exists?** → Components
- **What can it do?** → Capabilities
- **What does it use?** → Resources
- **What state does it have?** → State Fabric
- **How does it communicate?** → Communication Fabric

The Scheduler answers:

> **What executes, when, where, with which resources, under which constraints, and with what guarantees?**

# 1. ROS executor → NROS execution engine

A conventional ROS mental model is roughly:

```text
Messages
   ↓
Callbacks
   ↓
Executor
   ↓
Threads
```

NROS should model:

```text
Event / Request / Timer / State Change
                 │
                 ▼
             Activation
                 │
                 ▼
              Admission
                 │
                 ▼
             Scheduling
                 │
                 ▼
          Resource Binding
                 │
                 ▼
             Execution
                 │
                 ▼
              Effects
                 │
                 ▼
          State Transition
```

The **activation** is therefore the schedulable unit.

# 2. Why activation matters

A component can remain alive while many independent activations execute.

```text
Planner Component
│
├── Activation #101
├── Activation #102
├── Activation #103
└── Activation #104
```

Each activation can have its own:

```text
priority
deadline
budget
state revision
resources
parent
cancellation token
trace context
```

This is considerably more expressive than treating the entire node/process as the scheduling unit.

# 3. Activation descriptor

Conceptually:

```rust
struct Activation {
    id: ActivationId,
    component: ComponentId,

    trigger: Trigger,
    priority: Priority,

    deadline: Option<Deadline>,
    budget: Option<ResourceBudget>,

    state_revision: StateRevision,

    requirements: Requirements,

    parent: Option<ActivationId>,

    cancellation: CancellationToken,
}
```

This is not necessarily the final Rust API, but it captures the semantic boundary.

# 4. Trigger model

An activation can originate from:

```text
Message
Timer
Command
State transition
Capability request
Dependency completion
External event
System event
```

Example:

```text
LiDAR frame
    ↓
Perception activation
```

or:

```text
Mission request
    ↓
Planning activation
```

# 5. Admission before scheduling

An activation should not immediately enter an executor queue.

First:

```text
ACTIVATION_CREATED
        │
        ▼
     ADMISSION
```

Admission verifies:

```text
component valid
capabilities available
dependencies satisfied
resources available
policy permits execution
deadline feasible
state context valid
```

Only then:

```text
ADMITTED
```

# 6. Rejected activation

An activation may be rejected before consuming meaningful execution resources.

```text
REQUEST
  ↓
ADMISSION
  ↓
REJECTED
```

Reasons should be explicit:

```text
RESOURCE_UNAVAILABLE
CAPABILITY_UNAVAILABLE
POLICY_DENIED
DEADLINE_INFEASIBLE
INVALID_STATE
DEPENDENCY_UNAVAILABLE
```

This makes failures diagnosable.

# 7. Scheduling is a policy decision

NROS should not hard-code one universal scheduling algorithm.

Instead:

```text
Scheduler
   │
   ├── FIFO
   ├── Priority
   ├── Deadline
   ├── Rate-monotonic
   ├── Fair-share
   ├── Resource-aware
   ├── Energy-aware
   └── Custom policy
```

Different robots can select different policies.

# 8. Priority

An activation can carry priority:

```text
CRITICAL
HIGH
NORMAL
LOW
BACKGROUND
```

But priority alone is insufficient.

Consider:

```text
A: priority HIGH, deadline 10s
B: priority NORMAL, deadline 5ms
```

The scheduler needs more information than priority.

# 9. Deadline scheduling

For real-time workloads:

```text
deadline = t₀ + D
```

The scheduler should track:

```text
remaining_time
deadline_distance
execution_budget
```

An activation approaching its deadline can be promoted.

# 10. Deadline miss

A deadline miss is an explicit runtime event:

```text
RUNNING
   │
   ▼
DEADLINE_MISSED
```

The runtime should preserve:

```text
activation ID
deadline
actual completion
resource usage
scheduler decision
```

for diagnosis.

# 11. Timing guarantees

NROS must distinguish:

```text
requested
estimated
admitted
observed
verified
guaranteed
```

For example:

```text
deadline: 1 ms

requested: YES
admitted: YES
observed: 0.73 ms
verified: YES
guaranteed: NO
```

Unless the runtime has actually established the necessary guarantees.

# 12. Deterministic execution

For certain control workloads:

```text
same input
+
same state
+
same configuration
```

should ideally produce:

```text
same execution ordering
```

where the system's semantics require it.

Determinism becomes a scheduler property rather than merely an application aspiration.

# 13. Execution classes

NROS can classify activations:

```text
CONTROL
REALTIME
INTERACTIVE
BEST_EFFORT
BACKGROUND
BATCH
```

Each class can map to different scheduler policies.

# 14. Executor architecture

Instead of one universal executor:

```text
                    Scheduler
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
      RT Executor    General       Background
          │          Executor        Executor
          │             │              │
          ▼             ▼              ▼
         CPU           CPU            CPU/GPU
```

This provides isolation between workloads.

# 15. Execution domains

An execution domain defines the environment in which activations run.

```text
ExecutionDomain
├── scheduler
├── executor
├── CPU affinity
├── memory policy
├── transport policy
├── timing policy
└── isolation policy
```

Example:

```text
/control
/perception
/planning
/background
```

# 16. CPU affinity

A control activation may require:

```text
CPU 2
```

while perception uses:

```text
CPU 3–5
```

This prevents heavy perception workloads from starving control.

# 17. Executor isolation

A useful architecture:

```text
CPU
│
├── Core 0–1 → System
├── Core 2   → Control
├── Core 3–5 → Perception
└── Core 6–7 → Planning
```

The exact topology is deployment-specific.

The important principle is:

> **Resource isolation becomes explicit runtime state.**

# 18. Work queues

Each execution domain can maintain queues:

```text
Queue
├── Critical
├── High
├── Normal
└── Background
```

But queues should not become the semantic source of truth.

The scheduler's decision remains authoritative.

# 19. Scheduling decision

Conceptually:

```text
next = scheduler.select(
    ready_activations,
    available_resources,
    deadlines,
    priorities,
    policies
)
```

The result:

```text
SchedulingDecision
├── activation
├── executor
├── resources
├── start constraint
├── deadline
└── reason
```

# 20. Explainable scheduling

Every important scheduling decision should be explainable.

Example:

```text
Selected Activation #481

Reason:
  deadline = 2.1ms
  priority = HIGH
  resources = available
  dependency state = valid
  safety policy = permitted
```

This is invaluable for robotics debugging.

# 21. Scheduling trace

The runtime can produce:

```text
Activation #481
  CREATED      t=100.0
  ADMITTED     t=100.1
  SCHEDULED    t=100.2
  STARTED      t=100.3
  COMPLETED    t=101.0
```

Now latency can be decomposed:

```text
admission latency
queue latency
dispatch latency
execution latency
commit latency
```

# 22. End-to-end latency

For a sensor-to-actuator path:

```text
Sensor
  ↓
Transport
  ↓
Activation
  ↓
Admission
  ↓
Scheduling
  ↓
Execution
  ↓
Controller
  ↓
Actuator
```

NROS should measure the entire path.

Not merely:

```text
callback execution time
```

# 23. Causal execution

Each activation should carry a causal context:

```text
TraceId
ParentActivationId
TriggerId
StateRevision
```

Then:

```text
Camera frame
   ↓
Perception
   ↓
Obstacle detection
   ↓
Planner
   ↓
Controller
   ↓
Motor command
```

can be reconstructed as one causal chain.

# 24. Execution tree

The runtime can construct:

```text
Mission #1
│
├── Perception #10
│   └── Detection #11
│
├── Planning #20
│   ├── Map Query #21
│   └── Path Search #22
│
└── Control #30
    ├── Velocity #31
    └── Safety Check #32
```

This becomes an **execution tree**.

# 25. Parallel execution

Independent activations can execute concurrently:

```text
                 Mission
                    │
          ┌─────────┼─────────┐
          ▼         ▼         ▼
       Camera      LiDAR      IMU
          │         │         │
          └─────────┼─────────┘
                    ▼
               Sensor Fusion
```

The scheduler can exploit parallelism while respecting dependencies.

# 26. Dependency-aware scheduling

Suppose:

```text
Fusion
```

depends on:

```text
Camera
LiDAR
IMU
```

Then:

```text
Camera ───┐
LiDAR ────┼──→ Fusion
IMU ──────┘
```

Fusion cannot become runnable until required inputs arrive.

# 27. Readiness

An activation becomes:

```text
READY
```

only when:

```text
dependencies satisfied
+
required state available
+
resources reserved
+
policy valid
```

This gives the scheduler a precise runnable set.

# 28. Backpressure

If consumers cannot keep up:

```text
Producer
   ↓
Queue
   ↓
Consumer
```

NROS should expose explicit backpressure semantics:

```text
DROP
BLOCK
BUFFER
COALESCE
SAMPLE
THROTTLE
```

# 29. Message-driven activation

Instead of automatically invoking a callback for every message:

```text
message
   ↓
callback
```

NROS can decide:

```text
message
   ↓
activation policy
   ├── execute immediately
   ├── coalesce
   ├── batch
   └── drop
```

This is especially useful for high-rate sensors.

# 30. Coalescing

Suppose a camera produces:

```text
1000 frames/s
```

but the planner only needs:

```text
30 decisions/s
```

The runtime can coalesce:

```text
Frame 1
Frame 2
Frame 3
...
Frame N
   ↓
latest-state activation
```

This prevents unnecessary execution.

# 31. Stateful activation

An activation can operate against a specific state revision:

```text
StateRevision = 1042
```

If state changes while execution is in progress:

```text
1042 → 1043
```

the runtime can decide whether the activation:

```text
continues
restarts
aborts
revalidates
```

# 32. Optimistic execution

For non-critical workloads:

```text
read state 100
    ↓
execute
    ↓
state now 101
```

The activation can attempt to commit.

If conflict occurs:

```text
COMMIT
   ↓
CONFLICT
```

it can retry or abort.

# 33. Transactional effects

For certain operations:

```text
read state
   ↓
compute
   ↓
validate
   ↓
commit effects
```

This resembles transactional execution.

Not every robotic operation can be transactional, but the model is valuable for software state.

# 34. Physical effects

Physical actuators require special handling.

You cannot simply:

```text
rollback motor movement
```

Therefore NROS should distinguish:

```text
logical effect
```

from:

```text
physical effect
```

Physical effects require explicit compensation or safety mechanisms.

# 35. Effect classes

```text
READ_ONLY
REVERSIBLE
COMPENSATABLE
IRREVERSIBLE
SAFETY_CRITICAL
```

Example:

```text
Read camera → READ_ONLY

Update planner state → REVERSIBLE

Move robot arm → PHYSICAL / COMPENSATABLE

Emergency stop → SAFETY_CRITICAL
```

# 36. Effect authorization

Before an activation produces an effect:

```text
Activation
   ↓
Capability check
   ↓
Resource lease
   ↓
Policy check
   ↓
Effect
```

This creates a strong control boundary.

# 37. Cancellation

Every long-running activation should support cancellation where semantically safe:

```text
RUNNING
   │
   ▼
CANCELLATION_REQUESTED
   │
   ▼
QUIESCING
   │
   ▼
CANCELLED
```

For actuator operations, cancellation may instead mean:

```text
safe-stop procedure
```

rather than abruptly killing execution.

# 38. Preemption

Preemption is distinct from cancellation.

```text
Preemption:
pause now, resume later
```

```text
Cancellation:
terminate this activation
```

Not every activation needs to support preemption.

# 39. Suspension

A suspendable activation:

```text
RUNNING
   ↓
SUSPENDED
   ↓
RESUMED
```

requires state preservation.

This is especially useful for:

```text
AI inference
planning
simulation
background optimization
```

# 40. Execution budgets

An activation can receive:

```text
CPU budget
memory budget
GPU budget
network budget
energy budget
```

Example:

```text
Planning activation
CPU ≤ 100ms
Memory ≤ 256MB
Deadline = 500ms
```

Exceeding the budget becomes an observable event.

# 41. Budget enforcement

Possible policies:

```text
WARN
THROTTLE
PREEMPT
CANCEL
FAIL
DEGRADE
```

Different components can choose different policies.

# 42. Scheduler hierarchy

For a complex robot:

```text
Global Scheduler
       │
       ├── Robot Scheduler
       │      ├── Control
       │      ├── Perception
       │      └── Planning
       │
       └── Edge Scheduler
              ├── Mapping
              └── AI
```

Scheduling can therefore be hierarchical.

# 43. Distributed scheduling

Across machines:

```text
Robot
 │
 ├── Local Scheduler
 │
 └── Remote Scheduler
        │
        ├── GPU
        └── AI inference
```

The scheduler must account for:

```text
network latency
bandwidth
remote availability
failure
data locality
```

# 44. Remote execution

A capability request:

```text
vision.detect
```

could resolve to:

```text
Local GPU
```

or:

```text
Edge GPU
```

or:

```text
Remote inference service
```

without changing the application-level capability.

# 45. Placement decision

Placement considers:

```text
latency
resource availability
energy
security
data locality
cost
reliability
```

Thus:

```text
Capability selection
```

and:

```text
Execution placement
```

become related but distinct decisions.

# 46. Scheduler + Resource Fabric

The architecture now becomes:

```text
                  Scheduler
                     │
        ┌────────────┼────────────┐
        ▼            ▼            ▼
   Activations    Deadlines    Priorities
        │            │            │
        └────────────┼────────────┘
                     ▼
               Resource Manager
                     │
        ┌────────────┼────────────┐
        ▼            ▼            ▼
       CPU          GPU         Device
                     │
                     ▼
                  Executor
```

# 47. Executor contract

The executor should be relatively dumb.

It receives:

```text
ExecutionPlan
```

and performs it.

Conceptually:

```text
execute(plan)
```

The scheduler decides:

```text
what
when
where
with what
```

The executor handles:

```text
how
```

# 48. Execution plan

```text
ExecutionPlan
├── activation
├── executor
├── resources
├── environment
├── state revision
├── constraints
└── cancellation policy
```

This becomes the handoff between scheduling and execution.

# 49. Scheduling pipeline

The complete pipeline:

```text
Trigger
  ↓
Activation
  ↓
Admission
  ↓
Dependency resolution
  ↓
Resource reservation
  ↓
Scheduling
  ↓
Placement
  ↓
Execution plan
  ↓
Executor
  ↓
Effect
  ↓
State commit
  ↓
Trace
```

This is the core NROS execution lifecycle.

# 50. The critical invariant

NROS should enforce:

> **No activation enters execution unless its required prerequisites have been established.**

Formally:

```text
Executable(A)
    ⇔
    Valid(A)
    ∧ DependenciesSatisfied(A)
    ∧ ResourcesReserved(A)
    ∧ PolicyAllows(A)
    ∧ StateContextValid(A)
```

This is an important architectural invariant.

# 51. Execution evidence

Every activation should produce evidence:

```text
ActivationEvidence
├── activation_id
├── trigger
├── admission
├── scheduling
├── resource allocation
├── execution interval
├── result
├── effects
└── state transition
```

This makes the runtime auditable.

# 52. Scheduler verification

The scheduler itself should be testable against explicit properties:

```text
P1: no unauthorized execution
P2: no execution without required resources
P3: no duplicate exclusive ownership
P4: deadlines are tracked
P5: cancellation is observable
P6: resource leases are released
P7: failed activations reach terminal states
```

These become candidates for NROS runtime invariants and tests.

# 53. Scheduler failure

The scheduler itself is a critical component.

Therefore NROS should not assume:

```text
scheduler = infallible
```

Possible architecture:

```text
Scheduler
   │
   ├── Health monitor
   ├── Checkpoint
   ├── Recovery
   └── Fail-safe policy
```

For safety-critical robots, failure of scheduling infrastructure must have a defined physical response.

# 54. Fail-safe execution

If the control scheduler disappears:

```text
Scheduler failure
      ↓
Control watchdog
      ↓
Safe state
```

For example:

```text
stop actuator
hold position
reduce speed
engage safety mode
```

The exact policy belongs to the robot.

NROS provides the runtime mechanism.

# 55. NROS execution model

At this point:

```text
ROS:

Node
 └── Callback
      └── Executor
```

becomes:

```text
NROS:

Component
 └── Activation
      ├── Admission
      ├── Dependencies
      ├── Resources
      ├── Policy
      ├── Scheduling
      ├── Execution
      └── Effects
```

That is a fundamental semantic expansion.

# 56. NROS runtime stack

We can now draw the emerging stack:

```text
┌───────────────────────────────────────────────┐
│              NROS APPLICATION                 │
├───────────────────────────────────────────────┤
│ Components / Capabilities / Missions          │
├───────────────────────────────────────────────┤
│             Execution Fabric                  │
│ Activation / Scheduler / Executors            │
├───────────────────────────────────────────────┤
│              Resource Fabric                  │
│ CPU / GPU / Devices / Leases / Budgets        │
├───────────────────────────────────────────────┤
│                State Fabric                   │
│ State / Checkpoints / Revisions / Recovery    │
├───────────────────────────────────────────────┤
│           Communication Fabric                │
│ Topics / Requests / Events / Streams          │
├───────────────────────────────────────────────┤
│              Transport Layer                  │
│ Local IPC / Shared Memory / Network / DDS...  │
├───────────────────────────────────────────────┤
│                 Platform                      │
│ Linux / RTOS / Embedded / WASM / Hardware     │
└───────────────────────────────────────────────┘
```

# 57. The next architectural problem

We now have:

```text
Communication
State
Resources
Execution
```

But these systems need a **single semantic authority for truth and change**.

For example:

```text
Where is the robot?
What configuration is active?
Which map revision is valid?
Which component owns the motor?
Which resource lease is current?
Which mission state is authoritative?
```

This leads to the next major layer:

# **Part XXXI — NROS State Fabric & Event-Sourced Runtime**

The target model will be:

```text
                  STATE FABRIC
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
        State        Events      Revisions
          │            │            │
          └────────────┼────────────┘
                       ▼
                  State Store
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
        Components           Activations
             │                   │
             └─────────┬─────────┘
                       ▼
                  Checkpoints
                       │
                       ▼
                    Recovery
```

The key question becomes:

> **How does NROS turn distributed robot activity into a coherent, versioned, observable state machine rather than a collection of loosely synchronized processes?**
