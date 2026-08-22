# NROS Configuration & Time (Part XVII–XVIII)

The ROS1 parameter server is useful, but architecturally it combines several different concepts into one mechanism.

NROS should **not** simply reproduce:

```text
Global Key → Value
```

Instead, state must have explicit **ownership, lifetime, scope, type, mutability, persistence, and authority**.

# 1. The fundamental separation

NROS should distinguish at least:

```text
                    NROS STATE
                        │
       ┌────────────────┼────────────────┐
       ▼                ▼                ▼
 Configuration      Runtime State      Persistent State
       │                │                │
       ▼                ▼                ▼
 Calibration         Parameters       Checkpoints
 Secrets             Status           History
 Policies            Counters         Artifacts
```

And these should **not** share identical semantics.

# 2. Why the ROS parameter-server model is insufficient

Consider:

```text
robot.mass
```

This might be:

- deployment configuration,
- physical calibration,
- immutable device metadata,
- runtime-adjustable parameter,
- safety-critical value.

A generic parameter server cannot express these distinctions adequately.

NROS should.

# 3. State is typed

A state entry should have a declared type:

```text
State<T>
```

For example:

```rust
State<f32>
State<u32>
State<Pose>
State<ControllerConfig>
State<Calibration>
```

No implicit:

```text
string → arbitrary runtime interpretation
```

should be required.

# 4. State descriptors

Conceptually:

```text
StateDescriptor<T>
│
├── id
├── type
├── schema
├── owner
├── scope
├── mutability
├── persistence
├── authority
├── version
└── validation
```

This turns state into a first-class runtime resource.

# 5. Configuration

Configuration describes how a component should operate.

Examples:

```text
controller.frequency
planner.max_velocity
camera.resolution
navigation.frame
```

Configuration is generally:

```text
known before activation
```

and should be validated before execution.

# 6. Configuration lifecycle

A configuration should follow:

```text
UNLOADED
   ↓
LOADED
   ↓
VALIDATED
   ↓
APPLIED
   ↓
ACTIVE
```

Invalid configuration must stop the transition:

```text
LOAD
 ↓
VALIDATE
 ↓
FAIL
```

not:

```text
LOAD
 ↓
RUN WITH UNKNOWN VALUE
```

# 7. Configuration precedence

NROS may have several configuration layers:

```text
Built-in defaults
       ↓
Package configuration
       ↓
Robot configuration
       ↓
Deployment configuration
       ↓
Instance configuration
       ↓
Runtime override
```

The effective configuration becomes:

```text
EffectiveConfig =
    merge(defaults,
          package,
          robot,
          deployment,
          instance,
          approved_override)
```

But merging must be typed and deterministic.

# 8. Runtime parameters

Some values legitimately need to change during execution:

```text
controller.gain
camera.exposure
planner.cost_weight
```

These should be explicitly declared:

```text
mutability = RUNTIME
```

rather than assuming every configuration value can be changed.

# 9. Parameter mutation must be transactional

Suppose:

```text
kp = 2.0
ki = 0.5
kd = 0.1
```

A caller should not accidentally update only half of a logically coupled configuration.

NROS should support:

```text
BeginTransaction
    ↓
Set kp
Set ki
Set kd
    ↓
Validate
    ↓
Commit
```

or:

```text
Rollback
```

# 10. Atomic state updates

For safety-critical configurations:

```text
Old Configuration
       │
       ▼
Candidate Configuration
       │
       ▼
Validation
       │
       ├── FAIL → discard
       │
       ▼
Atomic Commit
       │
       ▼
New Configuration
```

No partially applied configuration should become visible.

# 11. Validation

Validation can operate at multiple levels.

### Type validation

```text
gain: f32
```

### Range validation

```text
0.0 ≤ gain ≤ 10.0
```

### Structural validation

```text
PIDConfig
├── kp
├── ki
└── kd
```

### Semantic validation

```text
controller_frequency > 2 × sensor_frequency
```

### Safety validation

```text
max_velocity <= certified_limit
```

# 12. State ownership

Every mutable state value should have an owner.

```text
controller.gain
       │
       ▼
ControllerComponent
```

Other components may have:

```text
READ
```

but not automatically:

```text
WRITE
```

# 13. Authority

Ownership is not enough.

NROS should distinguish:

```text
owner
authority
reader
observer
```

For example:

```text
SafetyController
    └── authority: WRITE

Planner
    └── permission: READ

Operator
    └── permission: PROPOSE

Telemetry
    └── permission: OBSERVE
```

# 14. Configuration vs commands

A critical distinction:

```text
parameter:
controller.max_velocity = 1.2
```

versus:

```text
command:
set_velocity(1.2)
```

The first changes state.

The second requests an action.

They should not share the same interface.

# 15. State vs event

Similarly:

```text
state:
motor.status = RUNNING
```

is different from:

```text
event:
MotorStarted
```

The event says:

> Something happened.

The state says:

> What is true now?

NROS should support both.

# 16. State subscriptions

A component should be able to observe state changes:

```text
State<T>
   │
   ├── read()
   └── subscribe()
```

Example:

```text
battery.level
```

can be consumed as:

```text
current value
```

or:

```text
change stream
```

# 17. State history

Some state should retain history:

```text
controller.gain
    │
    ├── v1
    ├── v2
    ├── v3
    └── v4
```

This allows:

```text
audit
rollback
reproduction
debugging
```

# 18. Versioned state

Every meaningful mutation can increment a version:

```text
version 41
    ↓
version 42
    ↓
version 43
```

Readers can then use:

```text
read_if_version()
```

or:

```text
compare_and_swap()
```

for concurrency control.

# 19. Optimistic concurrency

For distributed systems:

```text
Read version 10
       ↓
Modify
       ↓
Commit if version == 10
```

If another actor changed it:

```text
version = 11
```

the commit fails.

This prevents silent lost updates.

# 20. Configuration provenance

Every important configuration should answer:

```text
Who created it?
Where did it come from?
When was it changed?
Why was it changed?
Which version produced it?
```

Conceptually:

```text
ConfigValue
│
├── value
├── version
├── author
├── timestamp
├── source
├── reason
└── parent_version
```

This creates an audit trail.

# 21. Calibration

Calibration deserves its own category.

Examples:

```text
encoder.offset
camera.intrinsics
imu.bias
joint.zero_position
```

Calibration is generally:

```text
physical metadata
```

rather than ordinary runtime configuration.

# 22. Calibration lifecycle

```text
UNKNOWN
   ↓
MEASURED
   ↓
VALIDATED
   ↓
APPROVED
   ↓
ACTIVE
   ↓
SUPERSEDED
```

An unvalidated calibration should not silently become active.

# 23. Calibration provenance

A calibration record should include:

```text
device identity
procedure
instrument
operator/process
timestamp
environment
uncertainty
validation status
```

This is essential for serious robotics deployments.

# 24. Secrets

Secrets must be separated entirely.

Examples:

```text
network credentials
TLS private keys
API credentials
device certificates
```

These should **not** be stored in ordinary parameter/state stores.

Conceptually:

```text
Application
    │
    ▼
Secret Handle
    │
    ▼
Secure Provider
```

The application should preferably receive a capability or ephemeral secret rather than a globally readable database entry.

# 25. Secret lifecycle

```text
PROVISIONED
   ↓
AVAILABLE
   ↓
USED
   ↓
ROTATED
   ↓
REVOKED
```

A secret may also have:

```text
expiration
scope
audience
usage policy
```

# 26. Persistent state

Some state must survive process restart:

```text
odometry metadata
mission progress
device counters
learned calibration
configuration versions
```

Other state must disappear:

```text
temporary buffers
locks
active subscriptions
in-flight callbacks
```

Therefore:

```text
persistence = explicit
```

rather than automatic.

# 27. State lifetime

NROS should support explicit lifetimes:

```text
EPHEMERAL
PROCESS
COMPONENT
SESSION
ROBOT
DEPLOYMENT
PERSISTENT
```

For example:

```text
temporary_goal
    lifetime = SESSION
```

while:

```text
robot.serial_number
    lifetime = PERSISTENT
```

# 28. Checkpoints

This connects directly to NROS's recovery architecture.

A checkpoint represents:

```text
recoverable execution state
```

not simply configuration.

Example:

```text
Mission
 ├── current waypoint
 ├── completed actions
 ├── planner state
 └── recovery metadata
```

can be checkpointed.

# 29. Checkpoint model

```text
Checkpoint
│
├── checkpoint_id
├── parent
├── execution_id
├── component states
├── version
├── timestamp
├── integrity hash
└── recovery policy
```

# 30. Checkpoint chain

```text
C0
 │
 ▼
C1
 │
 ▼
C2
 │
 ▼
C3
```

If C3 is corrupted:

```text
C3
 ↓
invalid
 ↓
C2
 ↓
restore
```

This creates deterministic recovery points.

# 31. Checkpoint consistency

A checkpoint should not capture arbitrary inconsistent state.

For example:

```text
planner = waypoint 50
controller = waypoint 43
```

may not be a valid recoverable state.

NROS therefore needs checkpoint barriers:

```text
REQUEST
   ↓
QUIESCE
   ↓
CONSISTENCY POINT
   ↓
SNAPSHOT
   ↓
COMMIT
   ↓
RESUME
```

# 32. Distributed checkpointing

For multiple components:

```text
Planner
Controller
Mission
Localization
```

NROS may create:

```text
Global Checkpoint C42
```

containing coordinated snapshots.

This becomes particularly valuable for autonomous missions.

# 33. State store abstraction

A generic state API could look conceptually like:

```rust
StateStore
├── get<T>()
├── set<T>()
├── update<T>()
├── watch<T>()
├── transaction()
├── snapshot()
└── restore()
```

But specialized stores can implement it.

# 34. Storage backends

Possible implementations:

```text
nros-state
│
├── memory
├── file
├── sqlite
├── embedded-flash
├── shared-memory
└── remote
```

The state abstraction remains independent from storage.

# 35. Embedded systems

A microcontroller might use:

```text
Flash
EEPROM
FRAM
```

while Linux might use:

```text
filesystem
SQLite
database
```

NROS should not force one persistence technology.

# 36. Atomic persistence

Persistent state must survive power failure safely.

The store should support mechanisms such as:

```text
write
 ↓
validate
 ↓
commit marker
 ↓
durable
```

If power fails before commit:

```text
old state remains valid
```

rather than leaving corrupted state.

# 37. State integrity

Important records should have integrity metadata:

```text
State
 ├── version
 ├── checksum/hash
 └── provenance
```

For security-sensitive state:

```text
signature
```

may also be required.

# 38. State replication

A robot may replicate important state:

```text
Local State
    │
    ├── Robot storage
    │
    └── Fleet storage
```

But replication policy must be explicit.

For example:

```text
Telemetry:
best effort

Mission checkpoint:
durable

Safety state:
local authoritative
```

# 39. Conflict resolution

Distributed state introduces conflicts.

Example:

```text
Robot A:
mission.version = 42

Robot B:
mission.version = 43
```

NROS must not blindly choose one.

Possible policies:

```text
last-write-wins
version conflict
authoritative source
merge
application-defined
```

For safety-critical state, explicit conflict detection is preferable.

# 40. State authority graph

The trust model can be reused:

```text
MissionState
     │
     ├── MissionManager → WRITE
     ├── Planner        → READ
     ├── Operator       → PROPOSE
     └── Telemetry      → OBSERVE
```

This creates a clear authority boundary.

# 41. Parameter access should be capability-based

Instead of:

```text
set("/robot/controller/gain", value)
```

NROS should conceptually require:

```text
ControllerConfigHandle
```

with permissions.

This prevents arbitrary global mutation.

# 42. No universal mutable global namespace

This is a major architectural rule:

> **NROS must not recreate a globally writable parameter database.**

Global discovery may exist.

Global observability may exist.

But mutable state should have:

```text
owner
authority
scope
lifetime
```

# 43. Configuration manifests

Deployment can be declarative:

```yaml
controller:
  type: pid
  frequency: 500Hz

planner:
  max_velocity: 1.2

camera:
  resolution: 1920x1080
```

The deployment system translates this into typed configuration resources.

# 44. Configuration compilation

A useful pipeline:

```text
Manifest
   ↓
Parse
   ↓
Schema validation
   ↓
Type validation
   ↓
Semantic validation
   ↓
Policy validation
   ↓
Compiled Configuration
   ↓
Deployment
```

Thus configuration becomes part of admission control.

# 45. Configuration drift

A running robot can diverge from its declared deployment.

NROS should detect:

```text
Declared Config
       ≠
Effective Config
```

and expose:

```text
CONFIG_DRIFT
```

This is crucial for reproducibility.

# 46. Reconciliation

A controller can enforce:

```text
desired state
      ↓
actual state
      ↓
difference
      ↓
reconciliation
```

This brings a useful idea from distributed systems into robotics.

# 47. Desired vs observed state

NROS can distinguish:

```text
Desired:
motor.mode = ACTIVE

Observed:
motor.mode = FAULT
```

The runtime can then represent:

```text
desired ≠ observed
```

instead of pretending the command succeeded.

# 48. State machine integration

This connects directly to the NROS lifecycle:

```text
UNCONFIGURED
      ↓
CONFIGURED
      ↓
INACTIVE
      ↓
ACTIVE
```

Each transition can depend on state validation.

For example:

```text
CONFIGURE
  ↓
load configuration
  ↓
validate
  ↓
load calibration
  ↓
validate
  ↓
CONFIGURED
```

# 49. State and effects

The distinction between state and physical effects should remain strict.

```text
State:
motor.commanded_velocity = 1.0
```

does not guarantee:

```text
motor.actual_velocity = 1.0
```

The latter must come from observation.

This gives:

```text
Commanded State
       ≠
Observed State
```

unless verified.

# 50. State reconciliation with hardware

A typical actuator loop becomes:

```text
Desired State
      │
      ▼
Controller
      │
      ▼
Command
      │
      ▼
Hardware
      │
      ▼
Sensor
      │
      ▼
Observed State
      │
      └──────► Reconciliation
```

This is much safer than treating commands as facts.

# 51. NROS State Plane

We can now introduce a dedicated architectural plane:

```text
                    NROS
                      │
       ┌──────────────┼──────────────┐
       ▼              ▼              ▼
   Execution      Communication      State
       │              │              │
       │              │       ┌──────┼──────┐
       │              │       ▼      ▼      ▼
       │              │    Config  Runtime Persistent
       │              │       │      │      │
       └──────────────┴───────┼──────┼──────┘
                              ▼
                          Recovery
```

# 52. State architecture

```text
┌───────────────────────────────────────────────┐
│                 NROS STATE                    │
├───────────────────────────────────────────────┤
│ Configuration                                 │
│ Runtime Parameters                            │
│ Calibration                                   │
│ Desired State                                 │
│ Observed State                                │
│ Persistent State                              │
│ Checkpoints                                   │
├───────────────────────────────────────────────┤
│ Type │ Scope │ Owner │ Authority │ Lifetime   │
├───────────────────────────────────────────────┤
│ Validation │ Versioning │ Provenance           │
├───────────────────────────────────────────────┤
│ Memory │ File │ Flash │ DB │ Remote Storage   │
└───────────────────────────────────────────────┘
```

# 53. ROS → NROS state transformation

ROS:

```text
Parameter Server
       │
       └── key/value
```

NROS:

```text
Configuration
Parameters
Calibration
Secrets
Desired State
Observed State
Persistent State
Checkpoints
       │
       ▼
Typed State Resources
       │
       ├── ownership
       ├── authority
       ├── lifetime
       ├── validation
       ├── versioning
       └── persistence
```

This is a much cleaner semantic model.

# 54. Proposed crate boundary

A possible workspace organization:

```text
rust/
└── nros-core/
    ├── nros-types
    ├── nros-schema
    ├── nros-state
    ├── nros-config
    ├── nros-discovery
    ├── nros-security
    ├── nros-transport
    ├── nros-runtime
    └── nros-recovery
```

The exact names should remain provisional until the repository architecture is reconciled with the actual NROS codebase.

# 55. The deeper architectural progression

We now have:

```text
Part XIV
Hardware / Device Model
        ↓
Part XV
Communication
        ↓
Part XVI
Discovery / Identity
        ↓
Part XVII
State / Configuration
```

These are no longer independent subsystems.

They form a chain:

```text
WHAT EXISTS?
    ↓
Hardware / Resources

WHO CAN PROVIDE IT?
    ↓
Identity / Discovery

HOW DO WE COMMUNICATE?
    ↓
Channels / Transport

WHAT STATE GOVERNS IT?
    ↓
Configuration / State

HOW DO WE RECOVER?
    ↓
Checkpoint / Recovery
```

# 56. The NROS control loop

This leads toward a unified runtime loop:

```text
             ┌──────────────────────┐
             │       OBSERVE        │
             └──────────┬───────────┘
                        ↓
             ┌──────────────────────┐
             │       DISCOVER       │
             └──────────┬───────────┘
                        ↓
             ┌──────────────────────┐
             │       VALIDATE       │
             └──────────┬───────────┘
                        ↓
             ┌──────────────────────┐
             │        PLAN          │
             └──────────┬───────────┘
                        ↓
             ┌──────────────────────┐
             │       EXECUTE        │
             └──────────┬───────────┘
                        ↓
             ┌──────────────────────┐
             │        VERIFY        │
             └──────────┬───────────┘
                        ↓
             ┌──────────────────────┐
             │      CHECKPOINT      │
             └──────────┬───────────┘
                        │
                        └──────────────► OBSERVE
```

This is where the architecture starts moving beyond **robot middleware** toward a **robot-native execution substrate**.

# 57. Next — Part XVIII: Time, Clocks & Deterministic Execution

The next missing foundation is **time**.

ROS traditionally exposes time primarily as timestamps and simulated time.

NROS needs something considerably deeper:

```text
                         TIME
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
   Wall Clock         Monotonic          Logical
        │                 │                 │
        ▼                 ▼                 ▼
    Calendar          Duration           Causality
        │                 │                 │
        └─────────────────┼─────────────────┘
                          ▼
                  Temporal Contracts
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
          Deadline     Timeout      Periodicity
             │
             ▼
          Scheduler
             │
             ▼
       Deterministic Execution
```

This layer is particularly important because NROS is intended to combine **robotics, real-time execution, distributed communication, recovery, and autonomous/agentic workloads**.

The central question becomes:

> **How should NROS represent time so that simulation time, physical time, monotonic deadlines, distributed causality, real-time scheduling, and deterministic replay all coexist without conflating them?**

# NROS — Part XVIII: Time, Clocks & Deterministic Execution

Time is not merely metadata attached to ROS messages.

For NROS, time must become a **runtime primitive** because it governs:

- scheduling,
- deadlines,
- sensor freshness,
- control loops,
- synchronization,
- simulation,
- replay,
- distributed causality,
- checkpointing,
- deterministic execution.

The key principle is:

> **NROS must never treat all clocks as if they were the same clock.**

# 1. The NROS temporal model

We need several distinct concepts:

```text
                         NROS TIME
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
    Physical Time       Monotonic Time      Logical Time
        │                   │                   │
        ▼                   ▼                   ▼
    Wall Clock          Deadlines            Causality
    UTC/TAI             Timeouts             Ordering
        │                   │                   │
        └───────────────────┼───────────────────┘
                            ▼
                     Temporal Contracts
                            │
                            ▼
                       Scheduler
```

# 2. Physical time

Physical time answers:

> What time is it in the external world?

Examples:

```text
UTC
TAI
GPS time
PTP synchronized time
```

Physical time is useful for:

```text
logs
telemetry
events
sensor timestamps
cross-system correlation
```

But physical time is **not necessarily suitable for scheduling**.

# 3. Wall clock

A wall clock may jump:

```text
12:00:00
     ↓
12:00:01
     ↓
12:00:00
```

because of synchronization.

Therefore this is dangerous:

```rust
sleep_until(wall_clock + 10ms)
```

for timing-sensitive scheduling.

# 4. Monotonic time

A monotonic clock should satisfy:

```text
t₂ >= t₁
```

for observations made later.

It is the preferred clock for:

```text
timeouts
deadlines
durations
scheduling
latency measurements
watchdogs
```

Conceptually:

```text
MonotonicInstant
MonotonicDuration
```

should be first-class NROS types.

# 5. Logical time

Logical time does not necessarily represent physical time.

It answers:

> Which event happened before which other event?

For example:

```text
SensorEvent #41
      ↓
Inference #12
      ↓
Plan #7
      ↓
Command #92
```

Even if machines disagree about wall-clock time, causal ordering can remain correct.

# 6. Three clocks, three responsibilities

| Clock | Primary purpose |
|---|---|
| Physical | external-world timestamps |
| Monotonic | execution and deadlines |
| Logical | causality and ordering |

The rule:

> **Never use physical time where monotonic time is required, and never use wall-clock ordering as a substitute for causality.**

# 7. Clock abstraction

NROS should expose a clock interface:

```rust
trait Clock {
    type Instant;

    fn now(&self) -> Self::Instant;
}
```

But implementations can differ:

```text
SystemClock
MonotonicClock
SimClock
ReplayClock
LogicalClock
SynchronizedClock
```

# 8. Clock domains

Every timestamp should conceptually identify its domain.

Instead of:

```text
timestamp = 123456
```

NROS should know:

```text
timestamp:
    value = 123456
    domain = monotonic
```

or:

```text
timestamp:
    value = ...
    domain = simulation
```

# 9. Why clock domains matter

Imagine:

```text
Camera timestamp:
10.000 s simulation time
```

and:

```text
CPU timestamp:
8.217 s wall time
```

Comparing them directly is meaningless.

Therefore:

```text
Timestamp
├── value
├── clock_domain
└── uncertainty
```

should be part of the model.

# 10. Simulation time

Simulation is fundamental to robotics.

NROS should support:

```text
Simulation Clock
      │
      ├── pause
      ├── resume
      ├── accelerate
      ├── slow down
      └── seek
```

Components should be able to run against simulation time without changing their application logic.

# 11. Simulation clock injection

Conceptually:

```text
Runtime
  │
  ├── RealClock
  │
  └── SimClock
```

A component requests:

```rust
runtime.clock()
```

rather than directly calling:

```rust
system_time()
```

This is critical for deterministic simulation.

# 12. Replay clock

Recorded execution can be replayed:

```text
Recorded Timeline
       │
       ▼
Replay Clock
       │
       ▼
NROS Components
```

The runtime can reproduce:

```text
event timing
message timing
activation timing
```

according to the recording.

# 13. Deterministic replay

A strong NROS target is:

```text
same inputs
+
same configuration
+
same seed
+
same execution policy
=
same observable result
```

This requires controlling more than timestamps.

# 14. Sources of nondeterminism

Potential sources include:

```text
thread scheduling
message ordering
randomness
network timing
clock synchronization
concurrent state updates
memory allocation
hardware interrupts
```

NROS should identify these explicitly.

# 15. Deterministic execution mode

NROS could support:

```text
ExecutionMode
├── realtime
├── best_effort
├── deterministic
└── replay
```

Each mode has different guarantees.

# 16. Determinism is not the same as real-time

These concepts must remain separate.

### Real-time

Means:

> execution meets temporal constraints.

### Deterministic

Means:

> equivalent execution produces predictable equivalent ordering/results.

A system can be:

```text
real-time but nondeterministic
```

or:

```text
deterministic but not real-time
```

NROS should support both dimensions.

# 17. Temporal contracts

Communication already introduced:

```text
deadline
timeout
freshness
periodicity
```

These should become a unified temporal contract:

```text
TemporalContract
│
├── release
├── period
├── deadline
├── timeout
├── freshness
├── lifespan
├── jitter
└── synchronization
```

# 18. Periodic execution

A controller may specify:

```text
period = 2ms
```

The scheduler should reason about:

```text
release
execution
deadline
next release
```

rather than simply:

```text
sleep(2ms)
```

# 19. Release time

An activation becomes eligible at:

```text
t_release
```

Then:

```text
READY
```

becomes true.

The scheduler decides when it actually runs.

# 20. Deadline

If:

```text
deadline = 5ms
```

then:

```text
release = 100ms
deadline = 105ms
```

Missing the deadline should create structured runtime state:

```text
DeadlineMiss
│
├── activation
├── expected
├── actual
├── lateness
└── consequence
```

# 21. Deadline misses are first-class events

A missed deadline should not merely produce:

```text
log: warning
```

It should be available to:

```text
monitoring
recovery
safety policy
scheduler
telemetry
```

# 22. Temporal budget

An activation can have a budget:

```text
id="s9g4x7"
budget = 1ms
```

If execution consumes:

```text
850µs
```

the runtime knows:

```text
remaining = 150µs
```

This can propagate to downstream work.

# 23. End-to-end temporal budget

Consider:

```text
Sensor
  │ 2ms
  ▼
Perception
  │ 4ms
  ▼
Planning
  │ 3ms
  ▼
Control
  │ 1ms
  ▼
Actuator
```

Total:

```text
10ms
```

NROS can model this as:

```text
EndToEndContract = 10ms
```

with internal budgets.

# 24. Temporal admission control

Before starting a deployment:

```text
Required latency = 10ms

Available:
CPU = 8ms
Network = 5ms
Planner = 6ms
```

NROS should detect:

```text
10ms requirement impossible
```

before activation.

This is preferable to discovering the problem during physical operation.

# 25. Scheduler

The scheduler now becomes the bridge between:

```text
temporal contracts
```

and:

```text
execution resources
```

Conceptually:

```text
Temporal Contract
        │
        ▼
    Scheduler
        │
        ├── CPU
        ├── thread
        ├── accelerator
        └── device
```

# 26. Activation scheduling

The scheduler should operate on:

```text
Activation
```

rather than merely:

```text
Thread
```

An activation has:

```text
Activation
├── source
├── release
├── deadline
├── priority
├── budget
├── dependencies
└── cancellation state
```

This is much closer to the execution model required by NROS.

# 27. Dependencies

An activation can depend on another:

```text
CameraCapture
      │
      ▼
ObjectDetection
      │
      ▼
Planning
      │
      ▼
Control
```

The scheduler can therefore reason about a dependency graph.

# 28. Scheduling graph

```text
              Sensor
                │
                ▼
           Perception
             /     \
            ▼       ▼
       Localization Detection
            \       /
             ▼     ▼
              Planner
                 │
                 ▼
              Control
```

Each node has temporal constraints.

# 29. Priority

Priority should not be the only scheduling criterion.

NROS can consider:

```text
priority
deadline
criticality
resource availability
dependency readiness
budget
```

A low-priority task with an imminent hard deadline may become more urgent than a high-priority background task.

# 30. Criticality

Robotics systems may contain:

```text
Safety controller
Mission planner
Telemetry
Logging
AI inference
Visualization
```

They do not have equal criticality.

NROS should support:

```text
Criticality
├── safety
├── control
├── mission
├── perception
├── background
```

or an equivalent extensible model.

# 31. Mixed-criticality execution

A robot may run:

```text
Hard/strict:
motor control

Soft:
perception

Best effort:
LLM reasoning
```

NROS should allow these to coexist without allowing an expensive AI workload to starve the control loop.

# 32. Agent workloads

This becomes particularly important for NROS.

An agent may perform:

```text
Observe
   ↓
Reason
   ↓
Plan
   ↓
Tool execution
```

Reasoning may take:

```text
50ms
500ms
5s
```

while the controller requires:

```text
1ms
```

Therefore:

```text
agentic scheduling
```

must be isolated from:

```text
hard control scheduling
```

# 33. Temporal isolation

A resource policy might specify:

```text
Control:
CPU reservation = 30%

Perception:
CPU reservation = 30%

Planning:
CPU reservation = 20%

Agent:
CPU reservation = 15%

Background:
CPU reservation = 5%
```

This prevents uncontrolled resource interference.

# 34. Jitter

For periodic tasks:

```text
expected:
1ms
1ms
1ms
1ms
```

actual:

```text
1.02ms
0.98ms
1.13ms
0.91ms
```

The variation is jitter.

NROS should expose:

```text
jitter_min
jitter_max
jitter_mean
jitter_p99
```

where meaningful.

# 35. Temporal observability

The runtime should record:

```text
release time
start time
finish time
deadline
lateness
queue delay
execution duration
```

Then:

```text
release → start
```

measures scheduler delay.

And:

```text
start → finish
```

measures execution time.

# 36. End-to-end latency

NROS should be able to reconstruct:

```text
sensor acquisition
      ↓
transport
      ↓
queue
      ↓
activation
      ↓
computation
      ↓
command
      ↓
device
```

and calculate:

```text
physical-to-physical latency
```

rather than merely callback duration.

# 37. Synchronization

Distributed robots may require synchronized clocks.

NROS can support:

```text
PTP
GNSS
hardware timestamping
logical synchronization
```

but synchronization should expose uncertainty.

# 38. Time uncertainty

A timestamp is not necessarily exact.

Instead:

```text
TimeObservation
│
├── timestamp
├── clock domain
└── uncertainty
```

Example:

```text
timestamp = 100.234ms
uncertainty = ±3µs
```

This is much more useful in distributed robotics.

# 39. Timestamp provenance

A sensor timestamp should identify its origin:

```text
Sensor hardware
     ↓
Driver
     ↓
NROS timestamp
```

NROS should preserve the distinction between:

```text
measurement time
arrival time
processing time
publication time
```

# 40. Four important timestamps

For a sensor message:

```text
T_measure
T_capture
T_publish
T_receive
```

For example:

```text
hardware capture
     │
     │ 100.000ms
     ▼
driver
     │
     │ 100.150ms
     ▼
NROS publish
     │
     │ 100.500ms
     ▼
consumer receive
```

These differences reveal transport and scheduling latency.

# 41. Simulation and physical time

NROS should not allow simulation time to silently masquerade as physical time.

Instead:

```text
ClockDomain::Simulation
```

must remain explicit.

This prevents confusing:

```text
simulation timestamp
```

with:

```text
real-world timestamp
```

# 42. Replay semantics

Replay can control:

```text
clock
message delivery
activation order
random seed
external inputs
```

A replay session should have:

```text
ReplayId
Timeline
Clock
InputTrace
ExecutionPolicy
```

# 43. Deterministic replay pipeline

```text
Recorded Run
    │
    ├── inputs
    ├── timing
    ├── configuration
    ├── random seeds
    └── environment
          │
          ▼
      Replay Runtime
          │
          ▼
      NROS Components
          │
          ▼
       New Trace
          │
          ▼
       Compare
```

This creates a powerful verification loop.

# 44. Record/replay is more than rosbag

A traditional message recorder captures:

```text
messages
```

NROS should optionally capture:

```text
messages
+
activations
+
state transitions
+
clock
+
configuration versions
+
discovery
+
resource decisions
+
random seeds
+
effects
```

That creates an **execution trace**, not merely a message bag.

# 45. Execution trace

Conceptually:

```text
ExecutionTrace
│
├── ClockEvents
├── CommunicationEvents
├── ActivationEvents
├── StateEvents
├── DiscoveryEvents
├── ResourceEvents
├── SecurityEvents
└── EffectEvents
```

This can become the basis for debugging and formal verification.

# 46. Temporal causality

Combine logical time with execution trace:

```text
Sensor #100
    │
    ├── cause
    ▼
Detection #52
    │
    ├── cause
    ▼
Plan #18
    │
    ├── cause
    ▼
Command #77
```

Now NROS can answer:

> Why did actuator command #77 happen?

# 47. Temporal contracts and communication

A channel can specify:

```text
ImageStream
├── freshness ≤ 50ms
├── deadline ≤ 20ms
├── max_jitter = 5ms
└── history = latest
```

The transport and scheduler jointly enforce these requirements.

# 48. Temporal contracts and state

A state value can also have freshness:

```text
LocalizationPose
freshness ≤ 100ms
```

If the last observation is:

```text
250ms old
```

the runtime can classify it:

```text
STALE
```

rather than silently treating it as current.

# 49. Temporal validity

Every important runtime resource can therefore have:

```text
valid_from
valid_until
```

For example:

```text
Calibration
valid_until = 2027-01-01
```

or:

```text
Sensor observation
valid_until = capture + 50ms
```

# 50. Temporal safety

Suppose:

```text
motor command freshness <= 20ms
```

and the controller stops receiving updates.

NROS can transition:

```text
ACTIVE
  ↓
COMMAND_STALE
  ↓
SAFE_HOLD
```

This is much stronger than merely reporting a communication timeout.

# 51. Time-triggered execution

For highly deterministic systems:

```text
t0 ── sensor
t1 ── perception
t2 ── planning
t3 ── control
t4 ── actuator
```

NROS could support time-triggered execution where appropriate.

This can coexist with event-driven execution.

# 52. Event-driven vs time-driven

NROS should support both:

```text
Event-driven:
message arrives → execute

Time-driven:
clock reaches t → execute
```

and hybrid:

```text
clock release
     +
data dependency
     ↓
activation
```

# 53. Time-triggered control

A control loop can be defined:

```text
period = 1ms
phase = 0
deadline = 800µs
```

The scheduler then knows the intended temporal structure.

# 54. Temporal resource admission

Before starting a controller:

```text
CPU capacity
+
scheduler latency
+
interrupt load
+
transport latency
+
device latency
```

must satisfy:

```text
end-to-end temporal contract
```

This is where NROS becomes a true execution platform rather than a messaging framework.

# 55. Clock hierarchy

A useful architecture is:

```text
                Clock Provider
                     │
       ┌─────────────┼─────────────┐
       ▼             ▼             ▼
   Physical       Monotonic      Logical
       │             │             │
       ▼             ▼             ▼
   Timestamp      Scheduler      Causality
       │             │             │
       └─────────────┼─────────────┘
                     ▼
             Temporal Runtime
```

# 56. Proposed time types

Conceptually:

```rust
Instant
Duration
Deadline
Period
Timestamp
ClockId
ClockDomain
TimeRange
TimeUncertainty
```

The API should make invalid clock mixing difficult or impossible.

# 57. Type-safe clock domains

An advanced Rust design could distinguish:

```rust
PhysicalInstant
MonotonicInstant
SimulationInstant
LogicalTime
```

rather than having:

```rust
u64 timestamp
```

everywhere.

Then this becomes a compile-time error:

```text
PhysicalInstant - SimulationInstant
```

unless an explicit conversion exists.

# 58. Temporal errors

NROS should define structured temporal failures:

```text
DeadlineMiss
Timeout
ClockUnavailable
ClockDiscontinuity
ClockDesynchronized
TimestampInvalid
DataStale
BudgetExceeded
TemporalContractViolation
```

These become observable runtime events.

# 59. Time and recovery

A checkpoint should record temporal context:

```text
Checkpoint
├── state
├── configuration
├── logical time
├── execution time
├── clock domains
└── active deadlines
```

Recovery can then reconstruct the correct temporal state.

# 60. Time and security

Security events also need temporal context:

```text
Credential issued
Credential expires
Session established
Session revoked
```

This becomes important for leases and authorization.

# 61. Time and discovery

Recall the discovery lease:

```text
registration
    ↓
expires at T
```

That lease should use a monotonic or appropriately synchronized temporal model rather than an arbitrary wall-clock timestamp.

# 62. Time and resource management

A task can have:

```text
CPU budget = 2ms
deadline = 5ms
```

NROS can track:

```text
budget remaining
deadline remaining
```

and use those values for scheduling.

# 63. Time and agents

An autonomous agent should have a bounded execution contract:

```text
Observe:
  deadline = 20ms

Reason:
  soft deadline = 500ms

Act:
  hard deadline = 10ms
```

This prevents agentic reasoning from becoming an uncontrolled runtime activity.

# 64. Agent planning under temporal constraints

The planner can reason:

```text
Goal deadline = 2s

Available actions:

A = 100ms
B = 400ms
C = 900ms
```

The runtime can reject a plan whose minimum achievable execution time exceeds the goal deadline.

# 65. Temporal planning

This suggests an important future NROS interface:

```text
Plan
│
├── actions
├── dependencies
├── temporal constraints
├── resource requirements
└── expected effects
```

The scheduler can then validate the plan before execution.

# 66. NROS temporal architecture

```text
┌─────────────────────────────────────────────┐
│                 NROS TIME                   │
├─────────────────────────────────────────────┤
│ Physical │ Monotonic │ Simulation │ Logical │
├─────────────────────────────────────────────┤
│ Clock Domains / Synchronization             │
├─────────────────────────────────────────────┤
│ Timestamp / Duration / Deadline / Period     │
├─────────────────────────────────────────────┤
│ Temporal Contracts                           │
├─────────────────────────────────────────────┤
│ Scheduler / Activation Runtime               │
├─────────────────────────────────────────────┤
│ Budgets / Priority / Criticality / Jitter    │
├─────────────────────────────────────────────┤
│ Replay / Determinism / Execution Trace       │
└─────────────────────────────────────────────┘
```

# 67. ROS → NROS temporal transformation

ROS-style:

```text
timestamp
sleep
rate
timeout
```

becomes:

```text
NROS

Clock Domains
      │
Temporal Contracts
      │
Activation Scheduling
      │
Deadlines / Budgets
      │
Causal Time
      │
Replay
      │
Deterministic Execution
```

Time becomes part of the runtime semantics.

# 68. The emerging NROS architecture

After Parts XIV–XVIII:

```text
                         NROS
                          │
 ┌────────────┬───────────┼───────────┬─────────────┐
 ▼            ▼           ▼           ▼             ▼
Hardware   Discovery  Communication  State         Time
   │          │           │           │             │
Devices    Identity     Channels    Config       Clocks
Drivers    Capability   Services    Runtime      Deadlines
Buses      Binding      Events      Persistent   Scheduling
   │          │           │           │             │
   └──────────┴───────────┼───────────┴─────────────┘
                          ▼
                    NROS Runtime
                          │
              ┌───────────┼───────────┐
              ▼           ▼           ▼
          Execution     Recovery    Observability
              │           │           │
              └───────────┼───────────┘
                          ▼
                     Physical World
```

# 69. Next — Part XIX: Execution Runtime & Scheduler

The next logical step is to finally assemble the pieces into the **NROS execution engine**.

We have defined:

```text
Hardware       → what can execute
Discovery      → who/what exists
Communication  → how information moves
State          → what is true/configured
Time           → when execution is valid
```

Now we need:

```text
                    EXECUTION
                       │
       ┌───────────────┼────────────────┐
       ▼               ▼                ▼
   Components      Activations        Agents
       │               │                │
       └───────────────┼────────────────┘
                       ▼
                    Scheduler
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
        CPU          Worker       Hardware
        Core         Pool         Executor
          │            │            │
          └────────────┼────────────┘
                       ▼
                  Effects / State
```

The key question for the next part is:

> **What replaces the ROS node + callback + executor model when NROS treats components, activations, goals, agents, deadlines, resources, and physical effects as first-class execution entities?**

That is the point where **NROS stops being merely "ROS rewritten in Rust" and becomes a genuinely different runtime architecture.**
