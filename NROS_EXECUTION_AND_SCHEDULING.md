# NROS Execution & Scheduling (Part XIX–XXIV)

We now move from **what exists** in NROS to **what actually executes**.

The central architectural shift is:

> **ROS schedules callbacks inside nodes. NROS schedules typed activations under explicit temporal, resource, dependency, and safety contracts.**

This is one of the most important differences between a ROS-inspired middleware and a genuinely new robotics runtime.

## 1. ROS execution model

A simplified ROS-style architecture looks like:

```text
                ROS Graph
                   │
       ┌───────────┼───────────┐
       ▼           ▼           ▼
     Node A      Node B      Node C
       │           │           │
   callbacks   callbacks   callbacks
       │           │           │
       └────────── Executor ───┘
```

The fundamental unit is generally:

```text
node → callback → executor
```

This works well for conventional robotics applications.

But it hides important information:

```text
Why did this callback execute?
When must it execute?
What deadline does it have?
What caused it?
What resources may it consume?
What depends on it?
What happens if it misses its deadline?
```

NROS should expose these concepts explicitly.

# 2. NROS execution model

The fundamental chain becomes:

```text
                  Runtime
                     │
                  Component
                     │
                 Activation
                     │
              Scheduling Policy
                     │
                  Executor
                     │
                 Resource
                     │
                  Effect
```

Therefore:

```text
Component ≠ Activation
```

A component is a persistent computational entity.

An activation is one particular execution instance.

# 3. Component

A component describes **what can execute**.

Conceptually:

```rust
struct Component {
    id: ComponentId,
    metadata: ComponentMetadata,
    capabilities: Capabilities,
    contracts: Contracts,
}
```

It may represent:

```text
sensor driver
controller
planner
perception pipeline
state machine
agent
hardware interface
```

# 4. Activation

An activation describes:

> **This particular piece of work is now eligible to execute.**

For example:

```text
Camera frame #847
       │
       ▼
Detection activation #231
```

The activation contains contextual information:

```text
Activation
├── id
├── component
├── cause
├── release_time
├── deadline
├── priority
├── criticality
├── budget
├── dependencies
└── cancellation
```

# 5. Why this distinction matters

A perception component may execute thousands of times:

```text
PerceptionComponent
   │
   ├── Activation #1
   ├── Activation #2
   ├── Activation #3
   ├── ...
   └── Activation #100000
```

The runtime can therefore reason about each execution independently.

# 6. Activation lifecycle

A useful lifecycle:

```text
             CREATED
                │
                ▼
             RELEASED
                │
                ▼
              READY
                │
                ▼
            SCHEDULED
                │
                ▼
             RUNNING
             /     \
            /       \
           ▼         ▼
      COMPLETED    FAILED
           │         │
           └────┬────┘
                ▼
             FINALIZED
```

With additional transitions:

```text
READY → CANCELLED
RUNNING → PREEMPTED
RUNNING → DEADLINE_MISSED
```

# 7. Activation causes

An activation can originate from:

```text
message arrival
timer
hardware interrupt
service request
action goal
state transition
external event
agent decision
resource completion
dependency completion
```

So instead of:

```text
callback()
```

NROS has:

```text
Activation {
    cause: Event
}
```

# 8. The scheduler

The scheduler answers:

> Which ready activation should execute now, on which resource?

Conceptually:

```text
READY QUEUE
    │
    ▼
┌─────────────────────┐
│ NROS Scheduler      │
├─────────────────────┤
│ deadline            │
│ priority            │
│ criticality         │
│ dependencies        │
│ resource affinity   │
│ budget              │
│ policy              │
└──────────┬──────────┘
           │
           ▼
      EXECUTION PLAN
```

# 9. Scheduling should be policy-driven

NROS should not hard-code one scheduler.

Possible policies:

```text
FIFO
Priority
EDF
Rate Monotonic
Deadline + Priority
Criticality-aware
Resource-aware
Deterministic
Real-time
Agent-aware
```

The runtime provides the mechanism.

The deployment chooses the policy.

# 10. Earliest Deadline First

For activations:

```text
A deadline = 20ms
B deadline = 10ms
C deadline = 30ms
```

EDF selects:

```text
B → A → C
```

This is valuable for workloads dominated by deadlines.

# 11. Priority scheduling

Alternatively:

```text
A priority = 100
B priority = 50
C priority = 10
```

The scheduler may select:

```text
A → B → C
```

But NROS should avoid allowing priority alone to obscure temporal violations.

# 12. Hybrid scheduling

A more sophisticated policy can calculate urgency from:

```text
urgency =
    deadline pressure
  + criticality
  + priority
  + dependency pressure
  + resource availability
```

This gives the scheduler a richer decision space.

# 13. Resource model

Execution doesn't happen in an abstract vacuum.

NROS needs an explicit resource model:

```text
Resources
├── CPU cores
├── threads
├── GPU
├── NPU
├── FPGA
├── DMA
├── network
├── storage
└── hardware devices
```

An activation declares what it needs.

# 14. Resource requirements

Example:

```text
PerceptionActivation

CPU:
    2 cores

GPU:
    optional

Memory:
    256 MB

deadline:
    30 ms
```

The scheduler can then determine whether execution is possible.

# 15. Resource reservations

Critical workloads may reserve resources:

```text
Controller
    CPU 0
    reservation 40%
```

while an AI workload gets:

```text
Agent
    CPU 3
    reservation 20%
```

This provides isolation.

# 16. Executor

The executor converts scheduler decisions into actual execution.

```text
Scheduler
    │
    ▼
Executor
    │
    ├── CPU executor
    ├── async executor
    ├── GPU executor
    ├── device executor
    └── remote executor
```

Thus:

```text
Scheduler = decides
Executor  = performs
```

# 17. Executor abstraction

Conceptually:

```rust
trait Executor {
    fn submit(
        &self,
        activation: Activation
    ) -> ExecutionHandle;
}
```

The actual implementations may vary substantially.

# 18. Local execution

The simplest case:

```text
Activation
    │
    ▼
Local CPU
    │
    ▼
Function
```

But NROS should not assume every activation executes locally.

# 19. Remote execution

An activation could execute elsewhere:

```text
Robot
 │
 ├── local controller
 │
 └── edge computer
       │
       └── GPU inference
```

The runtime therefore becomes:

```text
Scheduler
   │
   ├── local executor
   └── remote executor
```

# 20. Remote execution requires a contract

Before dispatch:

```text
Task
├── required capability
├── deadline
├── input requirements
├── security policy
└── resource requirements
```

The runtime finds an eligible executor.

# 21. Capability-based execution

Instead of saying:

```text
run on machine X
```

the activation can say:

```text
requires:
    vision.acceleration
    model=XYZ
    memory>=4GB
```

Discovery finds suitable resources.

This connects execution directly to the NROS capability model.

# 22. Dependency-aware scheduling

Suppose:

```text
A → B → C
```

where:

```text
B depends on A
C depends on B
```

Then:

```text
A READY
B BLOCKED
C BLOCKED
```

After A completes:

```text
A COMPLETE
B READY
C BLOCKED
```

After B:

```text
B COMPLETE
C READY
```

The scheduler should operate directly on this dependency state.

# 23. DAG execution

Many robotics pipelines naturally form DAGs:

```text
Camera
  │
  ├── Detection ──┐
  │               │
  └── Depth ──────┤
                  ▼
               Fusion
                  │
                  ▼
               Planning
                  │
                  ▼
               Control
```

NROS can schedule independent branches concurrently.

# 24. Parallelism

If:

```text
Detection
Depth
Localization
```

are independent, the scheduler can execute:

```text
        ┌── Detection ──┐
Camera ─┼── Depth ──────┼──► Fusion
        └── Localization┘
```

rather than serializing them.

# 25. Backpressure

Fast producers can overwhelm slow consumers.

Example:

```text
Camera = 120 FPS
Inference = 20 FPS
```

Naive buffering creates:

```text
queue
████████████████████████
```

and increasing latency.

NROS should allow explicit policies:

```text
drop_oldest
drop_newest
latest_only
bounded_queue
block
sample
throttle
```

# 26. Freshness-aware scheduling

Suppose an image is already:

```text
80ms old
```

while its maximum freshness is:

```text
50ms
```

The scheduler should not blindly execute it.

Instead:

```text
Activation
    │
    ▼
Freshness check
    │
    ├── valid → execute
    │
    └── stale → discard/recover
```

# 27. Cancellation

Cancellation must be first-class.

An activation may be cancelled because:

```text
goal superseded
deadline impossible
data stale
safety state changed
operator request
higher-priority task
shutdown
resource unavailable
```

# 28. Cooperative cancellation

For ordinary computation:

```rust
if ctx.cancelled() {
    return;
}
```

But NROS should also model cancellation at the runtime level.

# 29. Preemption

Real-time workloads may require:

```text
Activation A RUNNING
        │
        ▼
Activation B becomes urgent
        │
        ▼
A PREEMPTED
        │
        ▼
B RUNNING
```

Whether preemption is supported depends on executor and workload class.

# 30. Non-preemptible regions

Hardware operations may need:

```text
critical section
```

NROS should expose bounded non-preemptible regions where required.

But they must have explicit limits.

# 31. Execution budgets

An activation can receive:

```text
budget = 2ms
```

Then:

```text
0ms ─────────────── 2ms
      execution
```

If it exceeds its budget:

```text
BudgetExceeded
```

The policy decides:

```text
throttle
cancel
degrade
preempt
escalate
```

# 32. Deadline vs budget

These are different.

```text
Budget:
how much execution time may be consumed?

Deadline:
when must execution finish?
```

Example:

```text
budget   = 2ms
deadline = 10ms
```

A task could consume 1.8ms and still miss its deadline because it waited too long.

# 33. Queueing delay

Therefore NROS must measure:

```text
release
   │
   ├── queue delay
   ▼
start
   │
   ├── execution
   ▼
finish
```

This makes scheduler performance observable.

# 34. Executor contexts

Each activation should receive an execution context:

```rust
struct ExecutionContext {
    activation_id: ActivationId,
    deadline: Deadline,
    budget: Budget,
    cancellation: CancellationToken,
    clock: ClockHandle,
    trace: TraceContext,
}
```

This gives application code access to runtime semantics without global state.

# 35. No hidden global runtime state

A major NROS principle should be:

> **Execution context must be explicit.**

Avoid application code depending on invisible global:

```text
master
global parameter server
global clock
global executor
```

Instead:

```text
RuntimeContext
```

provides the required capabilities.

# 36. Component isolation

A component should not automatically have access to everything.

For example:

```text
Perception
 ├── camera
 ├── model
 └── localization
```

but not:

```text
motor.write
```

unless explicitly authorized.

This connects execution to capability-based security.

# 37. Effect authorization

An activation attempting:

```text
motor.write(command)
```

must have:

```text
Capability::MotorWrite
```

The runtime can therefore enforce:

```text
identity
+
capability
+
temporal validity
+
safety policy
```

before permitting an effect.

# 38. Execution as a state machine

A powerful abstraction is:

```text
             ┌───────────┐
             │  CREATED  │
             └─────┬─────┘
                   ▼
             ┌───────────┐
             │  RELEASED │
             └─────┬─────┘
                   ▼
             ┌───────────┐
             │   READY   │
             └─────┬─────┘
                   ▼
             ┌───────────┐
             │ SCHEDULED │
             └─────┬─────┘
                   ▼
             ┌───────────┐
             │  RUNNING  │
             └─────┬─────┘
              ┌────┴─────┐
              ▼          ▼
          COMPLETED    FAILED
              │          │
              └────┬─────┘
                   ▼
              FINALIZED
```

The runtime can record every transition.

# 39. Execution trace

That gives us:

```text
Activation #921
  created     t=100.0
  released    t=100.2
  scheduled   t=100.3
  started     t=100.4
  completed   t=102.1
```

From this we derive:

```text
queue delay = 0.2ms
execution   = 1.7ms
```

and compare against:

```text
deadline
budget
```

# 40. Scheduler observability

NROS should expose scheduler metrics such as:

```text
ready_queue_depth
activation_latency
queue_latency
execution_time
deadline_misses
budget_exhaustions
preemptions
cancellations
resource_contention
```

This makes runtime behavior measurable.

# 41. Scheduler as a policy engine

The scheduler should not be a monolithic algorithm.

Architecturally:

```text
                 Scheduler Core
                       │
             ┌─────────┼─────────┐
             ▼         ▼         ▼
           Policy    Resource   Temporal
             │       Resolver    Validator
             └─────────┼─────────┘
                       ▼
                 Scheduling Decision
```

# 42. Scheduling decision

A decision can be modeled as:

```rust
struct ScheduleDecision {
    activation: ActivationId,
    executor: ExecutorId,
    resources: ResourceSet,
    start_constraint: StartConstraint,
}
```

This makes scheduling inspectable and testable.

# 43. Scheduling is not execution

This distinction is crucial.

```text
Decision:
"run activation A on CPU 2"

Execution:
"activation A actually ran on CPU 2"
```

The trace should record both.

This helps diagnose scheduler-vs-platform failures.

# 44. Failure model

NROS should distinguish:

```text
ActivationFailure
SchedulerFailure
ExecutorFailure
ResourceFailure
TransportFailure
DeviceFailure
ContractViolation
```

Do not collapse all failures into:

```text
ERROR
```

# 45. Recovery

An execution failure may trigger:

```text
retry
restart
failover
degrade
cancel dependent work
switch executor
enter safe state
```

These policies should be declarative where possible.

# 46. Retry policy

For example:

```text
RetryPolicy
├── max_attempts = 3
├── backoff = exponential
├── deadline = inherited
└── failure_classes = transient
```

Critically, retries must respect the original temporal contract.

A retry that guarantees a deadline miss is pointless.

# 47. Deadline propagation

If:

```text
Goal deadline = 1s
```

and:

```text
A → B → C
```

the runtime can propagate remaining temporal budget:

```text
A:
remaining = 1s

B:
remaining = 700ms

C:
remaining = 300ms
```

This enables deadline-aware planning.

# 48. Agent execution

Now consider an NROS agent:

```text
Agent
 │
 ├── observe
 ├── reason
 ├── plan
 ├── invoke tool
 └── verify
```

Each step can create an activation.

Thus:

```text
Agent Goal
   │
   ▼
Activation Graph
   │
   ├── Observe
   ├── Reason
   ├── Plan
   ├── Execute
   └── Verify
```

This is where NROS begins to unify robotics and agentic execution.

# 49. Agent activation graph

Example:

```text
             Goal
              │
           Observe
          /       \
     Camera       Lidar
          \       /
           Fusion
              │
           Reason
              │
            Plan
          /      \
      Move       Inspect
          \      /
           Verify
              │
          Goal Result
```

The scheduler can execute independent observations concurrently.

# 50. Agent reasoning must remain bounded

An LLM-based component should not be able to monopolize the runtime.

Therefore:

```text
AgentReasoning
    budget = 500ms
    priority = low
    criticality = mission
```

while:

```text
MotorControl
    budget = 100µs
    deadline = 1ms
    criticality = safety
```

The scheduler enforces separation.

# 51. Physical effects

Eventually an activation produces an effect:

```text
Activation
    │
    ▼
Authorization
    │
    ▼
Safety Validation
    │
    ▼
Hardware Effect
```

For example:

```text
planner
   ↓
velocity command
   ↓
policy check
   ↓
motor controller
```

This prevents computational intent from automatically becoming physical action.

# 52. Execution boundary

We can now define a critical NROS boundary:

```text
                COMPUTE DOMAIN
────────────────────────────────────────
planning
reasoning
perception
simulation
state transformation
────────────────────────────────────────
             EFFECT BOUNDARY
────────────────────────────────────────
motor
brake
gripper
arm
power
industrial actuator
────────────────────────────────────────
```

Crossing this boundary requires explicit authorization.

# 53. NROS runtime stack

We can now assemble the complete execution path:

```text
Application / Agent
        │
        ▼
Component
        │
        ▼
Activation
        │
        ├── temporal contract
        ├── resource contract
        ├── security contract
        └── dependency contract
        │
        ▼
Scheduler
        │
        ▼
Executor
        │
        ▼
Resource
        │
        ▼
Effect / State
        │
        ▼
Trace
```

# 54. The major architectural difference

ROS:

```text
Node
 ↓
Callback
 ↓
Executor
```

NROS:

```text
Component
 ↓
Activation
 ↓
Dependency Graph
 ↓
Temporal Contract
 ↓
Resource Policy
 ↓
Security Policy
 ↓
Scheduler
 ↓
Executor
 ↓
Effect
 ↓
Trace
```

That is a substantially richer execution model.

# 55. NROS execution kernel

This suggests a minimal runtime kernel:

```text
nros-runtime
├── activation
├── scheduler
├── executor
├── resource
├── deadline
├── cancellation
├── dependency
├── capability
├── effect
└── trace
```

Everything else should build above this layer.

# 56. Rust workspace direction

A possible NROS workspace:

```text
nros/
├── crates/
│   ├── nros-core
│   ├── nros-runtime
│   ├── nros-scheduler
│   ├── nros-executor
│   ├── nros-resource
│   ├── nros-time
│   ├── nros-communication
│   ├── nros-discovery
│   ├── nros-state
│   ├── nros-security
│   ├── nros-trace
│   └── nros-hardware
```

This keeps the architecture modular.

# 57. Core dependency direction

The dependency graph should remain approximately:

```text
                 nros-core
                /    |    \
               /     |     \
            time  protocol  types
              \      |      /
               \     |     /
              nros-runtime
             /      |       \
      scheduler  executor  resource
          │          │          │
          └──────────┼──────────┘
                     ▼
                applications
```

Higher layers should not leak assumptions into the kernel.

# 58. The NROS principle

We can now formulate a central principle:

> **An NROS system does not merely execute callbacks. It schedules causally connected activations under explicit temporal, resource, capability, and safety contracts.**

That sentence captures a major part of the architectural transition.

# 59. ROS → NROS evolution

The transformation now looks like:

```text
ROS
│
├── Nodes
├── Topics
├── Services
├── Parameters
├── Executor
└── Packages
        │
        ▼
NROS
│
├── Components
├── Channels
├── Operations
├── State
├── Activations
├── Temporal Contracts
├── Resource Contracts
├── Capability Contracts
├── Scheduler
├── Executors
├── Effects
├── Recovery
└── Execution Trace
```

# 60. Next: Part XX — Communication Runtime

The next layer should return to the ROS communication model, but reinterpret it completely.

We need to transform:

```text
ROS Topics
ROS Services
ROS Actions
ROS Parameters
```

into an NROS communication substrate based on:

```text
Channels
Streams
Requests
Commands
Events
State
Leases
QoS
Backpressure
Deadlines
Ownership
Capabilities
```

The key question becomes:

> **What should replace ROS's topic/service/action abstraction when communication itself carries temporal, causal, security, ownership, and resource semantics?**

That will give us the foundation for the **NROS data plane**.

# NROS — Part XX: The Communication Runtime

The next architectural step is to replace ROS's communication primitives with a more explicit **NROS communication model**.

The key transition is:

> **ROS treats communication primarily as message transport. NROS treats communication as typed, observable, policy-governed interaction between runtime entities.**

## 1. ROS communication model

ROS 1 broadly gives us:

```text
Topic
Service
Parameter Server
Action
```

The conceptual mapping is:

```text
Topic
  → asynchronous stream

Service
  → request / response

Action
  → long-running goal

Parameter
  → shared configuration/state
```

These abstractions are useful, but they don't fully express:

```text
ownership
deadlines
leases
backpressure
causality
security
resource limits
delivery guarantees
freshness
cancellation
```

NROS should make these first-class.

# 2. NROS communication model

A proposed NROS communication layer:

```text
                    Communication
                         │
        ┌────────────────┼────────────────┐
        ▼                ▼                ▼
      Stream           Request           Command
        │                │                │
        ▼                ▼                ▼
      Event            Response          Result
        │
        ▼
       State
```

And every interaction can carry:

```text
Identity
Causality
Deadline
Priority
QoS
Capability
Ownership
Trace context
```

# 3. Communication is typed

An NROS channel should not be an untyped pipe.

Conceptually:

```rust
Channel<T>
```

For example:

```rust
Channel<LaserScan>
Channel<VelocityCommand>
Channel<RobotPose>
Channel<Detection>
```

The type is part of the contract.

# 4. Message envelope

The payload alone isn't enough.

Instead:

```rust
struct Envelope<T> {
    header: Header,
    payload: T,
}
```

with:

```text
Header
├── message_id
├── source
├── destination
├── timestamp
├── sequence
├── deadline
├── priority
├── trace_id
├── causality_id
└── schema
```

This turns communication into an observable runtime event.

# 5. Message identity

Every message should be distinguishable.

```text
message_id
    ↓
7f6a...
```

This enables:

```text
deduplication
tracing
replay
correlation
diagnostics
auditing
```

# 6. Causality

Suppose:

```text
Camera
  ↓
Detection
  ↓
Planner
  ↓
Controller
```

NROS should preserve causal relationships:

```text
CameraFrame #100
      │
      └── causes
           │
           ▼
Detection #421
           │
           └── causes
                │
                ▼
             Plan #88
```

The runtime can therefore answer:

> Why did this actuator command happen?

# 7. Causal chain

Conceptually:

```text
cause_id
parent_id
trace_id
span_id
```

Example:

```text
Trace: T1

CameraFrame
  span S1

Detection
  parent S1
  span S2

Plan
  parent S2
  span S3

MotorCommand
  parent S3
  span S4
```

This is extremely valuable for robotics debugging.

# 8. Streams

ROS topics map most closely to NROS streams.

```text
Publisher
    │
    ▼
Stream<T>
    │
    ├── Subscriber A
    ├── Subscriber B
    └── Subscriber C
```

But NROS adds explicit stream policy.

# 9. Stream contract

A stream may declare:

```text
Stream<LidarScan>
├── capacity = 8
├── delivery = best_effort
├── freshness = 100ms
├── ordering = source_order
├── retention = none
└── overflow = drop_oldest
```

Now the behavior is deterministic and inspectable.

# 10. Backpressure

A central problem in distributed robotics is:

```text
producer speed > consumer speed
```

For example:

```text
Camera:
120 FPS

Detector:
20 FPS
```

Without policy:

```text
120 → queue → 20
       ↑
     growing
```

NROS must explicitly define what happens.

# 11. Backpressure strategies

Possible policies:

```text
DROP_OLDEST
DROP_NEWEST
LATEST_ONLY
BLOCK_PRODUCER
BUFFER
SAMPLE
THROTTLE
REJECT
```

Different robotics workloads require different policies.

# 12. Latest-only streams

For state-like data:

```text
robot_pose
```

old values may have almost no value.

Therefore:

```text
Pose:
P1 P2 P3 P4 P5 P6
            │
            ▼
        latest = P6
```

NROS can avoid wasting memory and latency on obsolete state.

# 13. Event streams

Other data should not be collapsed.

For example:

```text
EmergencyStop
FaultDetected
DoorOpened
CollisionDetected
```

Each event matters.

Therefore:

```text
delivery = reliable
retention = bounded
ordering = strict
```

# 14. State channels

A third semantic category is persistent state.

```text
State<T>
```

Example:

```text
RobotMode
RobotPose
BatteryState
MissionState
```

A state channel can provide:

```text
current value
version
timestamp
owner
revision
```

# 15. Stream vs State

This distinction matters:

```text
Stream:
"What happened?"

State:
"What is true now?"
```

ROS topics often mix both semantics.

NROS should distinguish them explicitly.

# 16. Request / Response

ROS services become:

```text
Request<TReq, TResp>
```

Conceptually:

```text
Client
  │
  │ request
  ▼
Server
  │
  │ response
  ▼
Client
```

But NROS adds:

```text
request_id
deadline
cancellation
authorization
retry policy
```

# 17. Request identity

Example:

```text
Request
├── request_id = R91
├── caller = planner
├── target = navigation
├── deadline = 500ms
└── payload = Navigate(...)
```

Response:

```text
Response
├── request_id = R91
├── status = SUCCESS
└── result = ...
```

# 18. Deadline-aware requests

A request should never become an immortal operation.

```text
deadline = 500ms
```

If the deadline expires:

```text
Request
   │
   ▼
EXPIRED
```

The runtime can cancel associated work.

# 19. Cancellation propagation

Consider:

```text
Planner
   │
   ▼
Navigate request
   │
   ▼
Path planner
   │
   ▼
Controller
```

If the original request is cancelled:

```text
Planner
   │
   X
Navigate
   │
   X
Path planner
   │
   X
Controller work
```

Cancellation should propagate through the causal execution graph where appropriate.

# 20. Commands

NROS should distinguish **commands** from ordinary requests.

A command expresses:

> Perform an externally meaningful operation.

Examples:

```text
MoveArm
OpenGripper
SetVelocity
StartMission
StopRobot
```

Commands cross the effect boundary.

# 21. Command lifecycle

A command may have:

```text
ACCEPTED
   ↓
VALIDATING
   ↓
AUTHORIZED
   ↓
EXECUTING
   ↓
COMPLETED
```

or:

```text
REJECTED
CANCELLED
EXPIRED
FAILED
```

This gives commands a formal lifecycle.

# 22. ROS Actions → NROS Operations

ROS actions are particularly important because they represent:

```text
goal
feedback
result
cancellation
```

NROS can generalize this as:

```text
Operation<TGoal, TFeedback, TResult>
```

# 23. Operation model

```text
Client
 │
 │ Goal
 ▼
Operation
 ├── Feedback ───────► Client
 ├── Progress ───────► Observers
 ├── State ──────────► Runtime
 └── Result ─────────► Client
```

This is more general than a robotics-specific action API.

# 24. Operation state machine

```text
CREATED
   │
   ▼
ACCEPTED
   │
   ▼
RUNNING
   │
   ├── PAUSED
   ├── CANCELLED
   ├── FAILED
   └── COMPLETED
```

The runtime owns the lifecycle.

# 25. Leases

A powerful addition is the concept of a **lease**.

Suppose an agent obtains control of an actuator.

Instead of permanent ownership:

```text
Agent → Motor
```

NROS can establish:

```text
Lease
├── owner
├── resource
├── expiry
├── permissions
└── renewal policy
```

# 26. Why leases matter

If the controller crashes:

```text
Agent
  X
  │
Lease expires
  │
  ▼
Motor ownership released
```

This is much safer than leaving resources indefinitely assigned.

# 27. Resource ownership

Example:

```text
Camera
 └── lease → Perception

Arm
 └── lease → Manipulation

Base
 └── lease → Navigation
```

A competing component cannot simply seize them.

# 28. Communication authorization

Every communication operation can be checked against capabilities.

Example:

```text
Perception
   │
   └── publish Detection
        ✓ allowed

Perception
   │
   └── command Motor
        ✗ denied
```

Thus the communication layer participates in security enforcement.

# 29. Capability-scoped channels

A channel can itself require a capability:

```text
Channel<MotorCommand>
requires:
    capability = actuator.base.write
```

This is much stronger than relying only on naming conventions.

# 30. Namespaces

ROS has namespaces.

NROS should retain logical naming, but separate:

```text
identity
name
location
capability
```

For example:

```text
/robot/base/velocity
```

does not imply:

```text
who owns it
who may publish
who may subscribe
```

Those become separate metadata.

# 31. Discovery

NROS needs a discovery layer.

A component should be able to discover:

```text
available streams
available operations
available services
available resources
available capabilities
```

Conceptually:

```text
Discovery
   │
   ├── StreamRegistry
   ├── OperationRegistry
   ├── ComponentRegistry
   └── ResourceRegistry
```

# 32. Discovery ≠ communication

This distinction is important.

Discovery answers:

> **What exists?**

Transport answers:

> **How do I communicate with it?**

Execution answers:

> **How do I run the associated work?**

NROS should keep these concerns separate.

# 33. Transport abstraction

Communication should sit above transport.

```text
NROS Communication
        │
        ▼
Transport Abstraction
   ┌────┼────┬────┐
   ▼    ▼    ▼    ▼
 SHM  UDP  QUIC  Serial
```

Different deployments can select different transports.

# 34. Shared memory

For high-rate local communication:

```text
Camera
   │
   ▼
Shared Memory
   │
   ├── detector
   ├── tracker
   └── recorder
```

This avoids unnecessary copies.

# 35. Network transport

Distributed robots may use:

```text
Robot A
   │
   │ network
   ▼
Robot B
```

The same logical channel can remain:

```text
Stream<LidarScan>
```

while the transport changes underneath.

# 36. Zero-copy

NROS should support zero-copy where feasible.

Instead of:

```text
sensor
 ↓ copy
middleware
 ↓ copy
consumer
```

use:

```text
sensor
   │
   ▼
shared buffer
   │
   ├── consumer A
   └── consumer B
```

Ownership/lifetime semantics then become critical.

# 37. Buffer ownership

A message buffer might have:

```text
OWNER
BORROWER
LEASE
REFERENCE COUNT
```

NROS must prevent use-after-release and unauthorized mutation.

Rust is particularly well suited to enforcing these invariants.

# 38. Serialization boundary

Not every message needs serialization.

Local:

```text
typed object
```

Remote:

```text
typed object
   ↓
serialization
   ↓
wire representation
```

Thus:

```text
Transport<T>
```

can select an appropriate codec.

# 39. Schema identity

Every wire-compatible type should have stable schema identity:

```text
schema_id
schema_version
encoding
compatibility
```

Example:

```text
Detection
schema = nros.detection
version = 2
```

# 40. Version compatibility

NROS should explicitly define:

```text
v1 producer → v1 consumer
v2 producer → v1 consumer
v1 producer → v2 consumer
```

Possible policies:

```text
EXACT
BACKWARD_COMPATIBLE
FORWARD_COMPATIBLE
ADAPTER_REQUIRED
INCOMPATIBLE
```

# 41. QoS

ROS 2's QoS model is an important foundation, but NROS can extend the concept.

Potential dimensions:

```text
reliability
durability
history
depth
deadline
lifespan
liveliness
priority
freshness
ordering
backpressure
security
```

The last several should be treated as first-class runtime semantics.

# 42. Reliability

Possible levels:

```text
BEST_EFFORT
RELIABLE
EXACTLY_ONCE
```

But NROS should be careful with `EXACTLY_ONCE`.

In distributed systems it is often more accurate to provide:

```text
at-least-once delivery
+
deduplication identity
```

rather than making unrealistic guarantees.

# 43. Freshness

Robotics introduces a dimension often overlooked in generic middleware:

```text
Is this information still useful?
```

Example:

```text
Camera image:
timestamp = t-300ms
```

Even if delivery succeeds:

```text
delivery = SUCCESS
```

the data may be operationally useless.

Hence:

```text
freshness < 100ms
```

can become a contract.

# 44. Temporal communication contract

An NROS message can therefore carry:

```text
published_at
expires_at
deadline
```

giving:

```text
message validity interval
```

```text
[t_published ───────── t_expires]
```

# 45. Temporal validity

Consumers can reject:

```text
if now > expires_at:
    stale()
```

This is preferable to blindly processing obsolete sensor data.

# 46. Priority

Not all communication has equal urgency.

```text
EmergencyStop       CRITICAL
MotorCommand        HIGH
Pose                NORMAL
Diagnostics         LOW
DebugLog            BACKGROUND
```

Transport and scheduler policies can cooperate using these priorities.

# 47. Priority inversion

If a low-priority workload holds a resource required by a high-priority operation:

```text
HIGH → waits → LOW
```

NROS needs resource policies such as:

```text
priority inheritance
priority ceiling
bounded ownership
```

where appropriate.

# 48. Communication + scheduler integration

Now the architecture becomes:

```text
Message arrival
      │
      ▼
Communication Runtime
      │
      ▼
Activation creation
      │
      ▼
Scheduler
      │
      ▼
Executor
```

This is a critical connection.

Receiving a message isn't merely "calling a callback."

It creates an **activation**.

# 49. Topic → Activation

For example:

```text
Camera publishes Frame
        │
        ▼
NROS Stream
        │
        ▼
Detector subscription
        │
        ▼
Activation #882
        │
        ▼
Scheduler
```

Now the scheduler knows exactly why the work exists.

# 50. Causal communication graph

The graph becomes:

```text
             STREAM
               │
               ▼
           ACTIVATION
               │
               ▼
            EFFECT
               │
               ▼
            EVENT
               │
               ▼
           ACTIVATION
```

Communication and computation therefore become one traceable causal system.

# 51. Observability

NROS should be able to answer:

```text
Where did this message originate?

Which component consumed it?

Which activation processed it?

How long did it wait?

Which executor ran it?

What effect did it produce?

What caused that effect?
```

This is far beyond basic topic introspection.

# 52. Communication graph

The NROS graph becomes multidimensional:

```text
                 COMPONENT
                    │
          ┌─────────┼─────────┐
          ▼         ▼         ▼
       STREAM    OPERATION   STATE
          │         │         │
          ▼         ▼         ▼
      ACTIVATION ACTIVATION RESOURCE
          │         │         │
          └─────────┼─────────┘
                    ▼
                  EFFECT
```

This is closer to a **runtime graph** than merely a topic graph.

# 53. ROS → NROS mapping

| ROS | NROS |
|---|---|
| Node | Component |
| Topic | Stream |
| Message | Envelope |
| Service | Request/Response |
| Action | Operation |
| Parameter Server | State/Configuration |
| Master/Discovery | Discovery Runtime |
| Executor | Scheduler + Executor |
| rosbag | Event/Trace Recorder |
| QoS | Communication Contract |
| Namespace | Logical Identity |
| Callback | Activation |

The important point is that these are **semantic transformations**, not merely renamed APIs.

# 54. Communication crate architecture

A possible implementation:

```text
nros-communication/
├── channel
├── stream
├── request
├── response
├── command
├── operation
├── envelope
├── schema
├── qos
├── backpressure
├── freshness
├── discovery
└── transport
```

Transport implementations should remain replaceable.

# 55. Suggested Rust traits

Conceptually:

```rust
trait Publisher<T> {
    fn publish(&self, message: Envelope<T>) -> Result<PublishReceipt>;
}

trait Subscriber<T> {
    fn receive(&mut self) -> Result<Envelope<T>>;
}

trait Requester<Req, Resp> {
    async fn request(
        &self,
        request: Req,
        deadline: Deadline,
    ) -> Result<Resp>;
}
```

The actual NROS repository API should only be shaped this way after checking the current crate architecture.

# 56. Communication receipt

Publishing can return structured information:

```rust
struct PublishReceipt {
    message_id: MessageId,
    accepted: bool,
    recipients: usize,
    timestamp: Timestamp,
}
```

For some transports this can additionally expose delivery state.

# 57. No false guarantees

NROS should distinguish:

```text
accepted
queued
transmitted
received
processed
effected
```

These are different states.

For example:

```text
publish() == success
```

does **not** necessarily mean:

```text
robot actuator received command
```

# 58. Command acknowledgement

For physical commands:

```text
Command
   │
   ▼
Accepted
   │
   ▼
Authorized
   │
   ▼
Dispatched
   │
   ▼
Received
   │
   ▼
Executed
```

The runtime can report the exact stage.

# 59. Safety-critical communication

For safety-related channels:

```text
EmergencyStop
```

NROS should support stronger contracts:

```text
high priority
bounded latency
reliable delivery
authentication
authorization
freshness
redundancy
audit trail
```

The precise guarantees depend on the underlying transport and platform.

# 60. The resulting NROS data plane

We can now define the communication architecture as:

```text
                 NROS DATA PLANE
                       │
       ┌───────────────┼────────────────┐
       ▼               ▼                ▼
    Streams         Operations        State
       │               │                │
       └───────────────┼────────────────┘
                       ▼
                   Envelope
                       │
       ┌───────────────┼────────────────┐
       ▼               ▼                ▼
    Security         QoS             Causality
       │               │                │
       └───────────────┼────────────────┘
                       ▼
                   Transport
                ┌──────┼──────┐
                ▼      ▼      ▼
               SHM    UDP    QUIC
```

And above it:

```text
Communication
      │
      ▼
Activation
      │
      ▼
Scheduler
      │
      ▼
Executor
```

# 61. The deeper NROS transformation

At this point, the original ROS computation graph:

```text
Nodes ── Topics ── Nodes
```

has evolved into:

```text
Components
    │
    ▼
Communication Contracts
    │
    ▼
Activations
    │
    ▼
Execution Graph
    │
    ▼
Effects
    │
    ▼
Trace / State
```

This gives NROS a much stronger foundation for:

- deterministic robotics
- distributed robotics
- embedded systems
- safety-oriented control
- heterogeneous compute
- autonomous agents
- simulation
- edge/cloud robotics

## Next — Part XXI: State, Parameters & Configuration

The next major transformation is the ROS **parameter server**.

Instead of reproducing:

```text
Parameter Server
       │
       ├── get
       └── set
```

NROS should develop a unified **State & Configuration Plane**:

```text
                 NROS STATE
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
    Runtime       Config      Persistent
      State        State         State
        │           │             │
        └───────────┼─────────────┘
                    ▼
                 Versioned
                   State
                    │
          ┌─────────┼─────────┐
          ▼         ▼         ▼
       Observe     Update    Subscribe
```

The critical question will be:

> **How should NROS represent mutable robot state without falling back to ROS 1's globally shared, weakly governed parameter model?**

That becomes the next architectural layer.

# NROS — Part XXI: State, Parameters & Configuration

We now replace the ROS 1 **Parameter Server** concept with something substantially more rigorous:

> **NROS should treat state as a versioned, typed, scoped, observable resource—not merely as a global key/value database.**

This distinction is important because configuration, runtime state, persistent state, and mission state have very different semantics.

# 1. The ROS parameter model

ROS 1 essentially provides:

```text
Parameter Server
       │
       ├── /robot/max_velocity
       ├── /robot/frame_id
       ├── /robot/controller/gain
       └── /robot/name
```

Nodes can:

```text
get parameter
set parameter
```

This is convenient, but it doesn't answer:

```text
Who owns this value?

Who may change it?

When does a change become effective?

Which version is current?

Can the value be rolled back?

Is it configuration or runtime state?

Does changing it require restarting a component?

What caused the change?
```

NROS should answer these explicitly.

# 2. Four kinds of state

NROS should distinguish at least:

```text
                  STATE
                    │
       ┌────────────┼────────────┐
       ▼            ▼            ▼
 Configuration   Runtime      Mission
       │           State         State
       │            │             │
       └────────────┴─────────────┘
                    │
                    ▼
               Persistent State
```

More precisely:

### Configuration

"What should the system use?"

### Runtime state

"What is happening now?"

### Mission state

"What is the robot currently trying to accomplish?"

### Persistent state

"What must survive restart?"

These should not be conflated.

# 3. Configuration

Examples:

```text
controller.max_velocity
controller.max_acceleration
camera.exposure
lidar.range
planner.algorithm
```

Configuration usually changes relatively infrequently.

It should therefore have:

```text
type
schema
version
scope
owner
permissions
validation
```

# 4. Runtime state

Runtime state changes continuously:

```text
robot.pose
robot.velocity
battery.level
controller.mode
localization.confidence
```

This should generally not be stored in a configuration database.

Instead it belongs naturally in the NROS state/data plane.

# 5. Mission state

Mission state represents higher-level execution:

```text
mission.id
mission.status
mission.current_goal
mission.progress
mission.phase
```

For example:

```text
Mission
   │
   ├── NavigateToWarehouse
   │
   ├── InspectDoor
   │
   └── ReturnHome
```

This state may itself be driven by the NROS activation graph.

# 6. Persistent state

Some state must survive process or robot restart:

```text
calibration
persistent counters
learned maps
device identity
deployment configuration
operator preferences
```

This requires durable storage.

# 7. State hierarchy

A useful model:

```text
NROS State
│
├── Ephemeral
│   └── valid only during execution
│
├── Runtime
│   └── current operational state
│
├── Mission
│   └── current mission state
│
├── Configuration
│   └── deployment parameters
│
└── Persistent
    └── durable state
```

# 8. State ownership

Every mutable state item should have an owner.

Example:

```text
robot.mode
owner = controller
```

Other components may observe it:

```text
planner ──read──► robot.mode
UI      ──read──► robot.mode
logger  ──read──► robot.mode
```

but cannot modify it unless explicitly authorized.

# 9. Read/write capabilities

State access becomes capability-based:

```text
Capability
├── state.read
├── state.write
├── state.observe
└── state.admin
```

Example:

```text
Planner:
    read pose       ✓
    write pose      ✗

Localization:
    read pose       ✓
    write pose      ✓
```

# 10. State identity

A state item should have a stable identifier:

```rust
StateId
```

rather than depending solely on strings.

A human-readable name can still exist:

```text
/robot/navigation/current_goal
```

but internally:

```text
StateId = 0x91...
```

This allows efficient references and prevents accidental identity collisions.

# 11. Typed state

Instead of:

```text
parameter = "42"
```

NROS should know:

```text
State<u32>
```

or:

```text
State<VelocityLimit>
```

The type is part of the contract.

# 12. Schema

A state definition can include:

```text
StateDefinition
├── id
├── name
├── type
├── schema_version
├── scope
├── owner
├── mutability
├── persistence
├── validation
└── access_policy
```

# 13. Example

Consider:

```text
robot.max_velocity
```

NROS could represent:

```text
type:
    Velocity

unit:
    m/s

range:
    0.0 ..= 2.0

owner:
    safety_controller

mutability:
    runtime

persistence:
    configuration
```

Now the runtime has enough information to validate changes.

# 14. Validation

A state update should not simply be:

```text
set(key, value)
```

Instead:

```text
propose update
      │
      ▼
schema validation
      │
      ▼
policy validation
      │
      ▼
authorization
      │
      ▼
commit
```

# 15. Transactional update

Suppose a controller requires:

```text
max_velocity
max_acceleration
```

to remain mutually consistent.

Updating one without the other could temporarily produce an invalid configuration.

NROS should therefore support:

```text
Transaction
├── update A
├── update B
└── commit
```

Either:

```text
both succeed
```

or:

```text
neither becomes visible
```

# 16. Versioning

Every committed state should have a revision:

```text
Revision 1
Revision 2
Revision 3
...
```

For example:

```text
robot.max_velocity

v17 = 1.0
v18 = 1.2
v19 = 1.5
```

Consumers can reason about exactly which version they observed.

# 17. Compare-and-swap

For concurrent updates:

```text
update if version == 18
```

If the current version is 19:

```text
CONFLICT
```

This prevents silent overwrites.

Conceptually:

```rust
compare_and_set(
    state_id,
    expected_version,
    new_value
)
```

# 18. Optimistic concurrency

This gives NROS a useful model:

```text
read v10
   │
   ▼
compute new state
   │
   ▼
commit if still v10
```

If another component changes it:

```text
v10 → v11
```

the commit fails safely.

# 19. State subscriptions

Components should be able to subscribe to state changes:

```text
subscribe(
    "/robot/controller/mode"
)
```

Then:

```text
State v10
   │
   ▼
State v11
   │
   ▼
StateChanged event
```

This makes state reactive without polling.

# 20. State snapshots

NROS should support consistent snapshots:

```text
Snapshot #1024
├── robot.pose = ...
├── robot.velocity = ...
├── battery = ...
└── mission.state = ...
```

This is particularly useful for:

- diagnostics
- replay
- checkpointing
- debugging
- recovery

# 21. Checkpointing

The runtime can periodically record:

```text
Checkpoint
├── runtime state
├── mission state
├── component state
├── activation state
└── persistent metadata
```

Then after failure:

```text
failure
  │
  ▼
checkpoint
  │
  ▼
restore
  │
  ▼
resume
```

This becomes especially valuable for autonomous systems.

# 22. Configuration lifecycle

Configuration itself should have lifecycle states:

```text
DRAFT
   ↓
VALIDATED
   ↓
APPROVED
   ↓
STAGED
   ↓
ACTIVE
   ↓
SUPERSEDED
```

This is much safer than directly modifying live parameters.

# 23. Configuration rollout

Suppose we change:

```text
controller.max_velocity
1.0 → 1.5
```

NROS can support:

```text
prepare
  ↓
validate
  ↓
stage
  ↓
activate
```

If activation fails:

```text
rollback
```

# 24. Effective configuration

A configuration value may come from multiple scopes:

```text
global
  ↓
robot
  ↓
subsystem
  ↓
component
  ↓
instance
```

For example:

```text
/global/max_velocity
/robot01/max_velocity
/robot01/base/max_velocity
```

The effective value is resolved according to a deterministic precedence rule.

# 25. Avoid hidden configuration inheritance

However, inheritance should never become mysterious.

NROS should be able to explain:

```text
effective value = 1.2 m/s

source:
    /robot01/base/max_velocity

overrides:
    /global/max_velocity = 2.0
```

This is essential for debugging deployments.

# 26. Configuration provenance

Every configuration change should have provenance:

```text
ChangedBy
├── identity
├── component
├── timestamp
├── reason
├── previous_version
└── transaction_id
```

So the system can answer:

> Who changed the robot's maximum speed?

# 27. State provenance

The same principle applies to runtime state.

Suppose:

```text
robot.mode = SAFE
```

NROS should be able to determine:

```text
source:
    safety_controller

cause:
    obstacle_detected

activation:
    #78291

timestamp:
    ...
```

# 28. State causality

This creates a powerful relationship:

```text
Event
  │
  ▼
Activation
  │
  ▼
State transition
  │
  ▼
New activation
```

For example:

```text
ObstacleDetected
      │
      ▼
SafetyActivation
      │
      ▼
robot.mode = SAFE
      │
      ▼
NavigationCancelled
```

Now state changes become part of the causal graph.

# 29. State machines

NROS should support explicit state machines.

Example:

```text
                    ┌─────────┐
                    │  IDLE   │
                    └────┬────┘
                         │ start
                         ▼
                    ┌─────────┐
                    │ RUNNING │
                    └────┬────┘
                    stop │ │ fault
                         │ │
              ┌──────────┘ └─────────┐
              ▼                      ▼
          ┌─────────┐            ┌─────────┐
          │ STOPPED │            │  FAULT  │
          └─────────┘            └─────────┘
```

Transitions can carry:

```text
guard
event
action
authorization
```

# 30. State transition contract

A transition:

```text
RUNNING → SAFE
```

could require:

```text
cause = emergency_stop
priority = critical
authorization = safety_controller
```

This makes safety behavior explicit.

# 31. Parameters become state declarations

Instead of:

```text
rosparam set /foo/bar 42
```

NROS conceptually uses:

```text
state.update(
    id = FooBar,
    value = 42,
    expected_version = 17
)
```

The runtime then evaluates:

```text
type
schema
authorization
ownership
transaction
version
policy
```

before committing.

# 32. State plane architecture

The resulting architecture:

```text
                    NROS STATE PLANE
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
   Configuration        Runtime            Mission
        │                State               State
        │                  │                  │
        └──────────────────┼──────────────────┘
                           ▼
                    State Registry
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
       Versioning      Transactions     Subscriptions
          │                │                │
          └────────────────┼────────────────┘
                           ▼
                     Persistence
```

# 33. State + communication

State should integrate naturally with the communication plane.

```text
State update
    │
    ├── commit
    │
    ▼
StateChanged event
    │
    ▼
Stream
    │
    ▼
Subscriber
    │
    ▼
Activation
```

Thus:

> **A state change can become an execution trigger.**

# 34. State + scheduler

For example:

```text
battery.level < 10%
```

can cause:

```text
BatteryPolicyActivation
```

which schedules:

```text
ReturnToBase
```

The chain becomes:

```text
State
 ↓
Event
 ↓
Activation
 ↓
Scheduler
 ↓
Operation
 ↓
State
```

This forms a closed-loop runtime.

# 35. State + security

State access is also a security boundary.

Example:

```text
Operator
   │
   └── read battery ✓

Operator
   │
   └── write motor_limit ✗

SafetyController
   │
   └── write motor_limit ✓
```

This should be enforced by the runtime rather than merely documented.

# 36. State + leases

Some state may be temporarily owned.

Example:

```text
mission.current_goal
```

could be controlled by:

```text
MissionManager
```

under a lease:

```text
Lease:
MissionManager
expires: T+60s
```

If ownership expires, the runtime can reject further writes or transition to a recovery policy.

# 37. State + real-time constraints

Not all state access is equal.

A real-time controller should not perform:

```text
blocking database lookup
```

during its critical execution path.

NROS should therefore distinguish:

```text
real-time state
non-real-time state
persistent state
```

# 38. Real-time state

Real-time state should favor:

```text
preallocated storage
lock-free access where appropriate
bounded operations
deterministic memory behavior
no uncontrolled I/O
```

Example:

```text
controller.velocity
controller.mode
safety.limit
```

# 39. Persistent state

Persistent state can use slower mechanisms:

```text
filesystem
database
flash
remote storage
```

but must not accidentally leak these costs into real-time execution.

# 40. State storage abstraction

A possible interface:

```rust
trait StateStore<T> {
    fn read(&self, id: StateId) -> Result<StateSnapshot<T>>;
    fn commit(&self, update: StateUpdate<T>) -> Result<StateRevision>;
}
```

Different implementations can provide:

```text
InMemoryStateStore
RealtimeStateStore
PersistentStateStore
DistributedStateStore
```

# 41. Distributed state

For multi-robot systems:

```text
Robot A
   │
   ├── local state
   │
   ▼
shared state
   ▲
   │
   └── Robot B
```

But distributed state introduces:

```text
consistency
conflicts
partitions
latency
clock differences
```

NROS should not pretend these problems disappear.

# 42. Explicit consistency

Possible state consistency policies:

```text
LOCAL
EVENTUAL
CAUSAL
STRONG
QUORUM
```

A robot's instantaneous motor state should normally remain local.

A shared mission state might tolerate eventual consistency.

# 43. State authority

For distributed state, one of the most important concepts is:

```text
Who is authoritative?
```

Example:

```text
Robot A:
authoritative for /robotA/pose

Mission server:
authoritative for /mission/state
```

NROS should encode authority explicitly.

# 44. Conflict resolution

If two nodes attempt:

```text
mission.state = PAUSED
mission.state = CANCELLED
```

the runtime should not silently pick one.

Possible policies:

```text
reject
priority
authority
version
transaction
application-defined resolver
```

# 45. State event sourcing

Another powerful model is recording transitions:

```text
State:
v1
  ↓ event A
v2
  ↓ event B
v3
  ↓ event C
v4
```

This allows reconstruction:

```text
state(t)
```

from the event history.

# 46. State replay

During debugging:

```text
recorded events
      │
      ▼
replay runtime
      │
      ▼
reconstructed state
```

This complements the communication recorder introduced earlier.

# 47. Unified event model

At this point, NROS can treat:

```text
message
state change
activation
command
result
failure
resource event
```

as observable runtime events.

This gives us the basis for a unified event log.

# 48. NROS state event

Conceptually:

```rust
struct StateChanged<T> {
    state_id: StateId,
    previous_version: Version,
    new_version: Version,
    value: T,
    cause: CausalContext,
}
```

This makes state transitions traceable.

# 49. The State → Activation bridge

A particularly important primitive:

```text
on_change(state_id)
```

could produce:

```text
StateChanged
    │
    ▼
Trigger
    │
    ▼
Activation
```

This enables reactive systems without requiring every component to poll state.

# 50. Configuration → Runtime bridge

Configuration changes should similarly be controlled:

```text
Configuration Update
        │
        ▼
Validation
        │
        ▼
Authorization
        │
        ▼
Transaction
        │
        ▼
Activation / Reload
        │
        ▼
New Runtime State
```

This is much safer than changing a parameter behind a component's back.

# 51. NROS State API

A conceptual high-level API:

```text
state.define()
state.read()
state.snapshot()
state.update()
state.transaction()
state.subscribe()
state.watch()
state.compare_and_set()
state.checkpoint()
state.restore()
```

# 52. State crate

A possible workspace organization:

```text
nros-state/
├── id
├── schema
├── value
├── revision
├── snapshot
├── transaction
├── subscription
├── ownership
├── lease
├── validation
├── persistence
└── consistency
```

# 53. Parameter compatibility layer

NROS can still provide a ROS-compatible facade:

```text
ROS Parameter API
        │
        ▼
NROS Configuration API
```

This helps migration.

But the compatibility layer should not dictate the internal architecture.

# 54. ROS → NROS parameter transformation

```text
ROS:
global key/value

        ↓

NROS:
typed state declaration
        +
scope
        +
owner
        +
schema
        +
version
        +
authorization
        +
lifecycle
```

This is a genuine semantic upgrade.

# 55. NROS unified runtime loop

We can now connect the first three major planes:

```text
             ┌──────────────────┐
             │ Communication    │
             │ Plane            │
             └────────┬─────────┘
                      │
                      ▼
                  Activation
                      │
                      ▼
             ┌──────────────────┐
             │ Execution        │
             │ Runtime          │
             └────────┬─────────┘
                      │
                      ▼
                   Effects
                      │
                      ▼
             ┌──────────────────┐
             │ State Plane      │
             └────────┬─────────┘
                      │
                      ▼
                  State Event
                      │
                      └──────────────► Activation
```

This is the beginning of the **NROS closed-loop runtime architecture**.

# 56. The emerging NROS architecture

We now have:

```text
                        NROS
                         │
        ┌────────────────┼─────────────────┐
        │                │                 │
        ▼                ▼                 ▼
 Communication       Execution          State
    Plane             Runtime           Plane
        │                │                 │
        ▼                ▼                 ▼
 Streams             Scheduler       Configuration
 Operations          Executor        Runtime State
 Commands            Resources       Mission State
 Events              Activations     Persistence
        │                │                 │
        └────────────────┼─────────────────┘
                         ▼
                      Effects
                         │
                         ▼
                       Trace
```

# 57. The central abstraction is emerging

Across these layers, one concept keeps appearing:

```text
Activation
```

Communication creates activations.

State changes create activations.

Timers create activations.

Commands create activations.

Agent decisions create activations.

Resource completions create activations.

Therefore:

> **Activation is becoming the universal bridge between information, computation, and action in NROS.**

This is potentially one of the defining architectural properties of NROS.

# 58. From ROS Graph to NROS Runtime Graph

ROS primarily gives:

```text
Node
 │
Topic
 │
Node
```

NROS evolves this into:

```text
                    EVENT
                      │
                      ▼
                  ACTIVATION
                      │
             ┌────────┼────────┐
             ▼        ▼        ▼
          STATE     RESOURCE  MESSAGE
             │        │        │
             └────────┼────────┘
                      ▼
                  SCHEDULER
                      │
                      ▼
                  EXECUTOR
                      │
                      ▼
                    EFFECT
                      │
                      ▼
                    EVENT
```

This is not merely a graph of communication.

It is a graph of **causality and execution**.

# 59. Next — Part XXII: Discovery, Identity & Distributed NROS

The next layer is the one that allows the runtime to escape a single process or machine.

We need to transform ROS's:

```text
ROS Master
node registration
topic discovery
service discovery
network URI resolution
```

into a distributed NROS control plane:

```text
                         NROS CONTROL PLANE
                                  │
          ┌───────────────────────┼───────────────────────┐
          ▼                       ▼                       ▼
      Identity                Discovery                Capability
          │                       │                       │
          ▼                       ▼                       ▼
       Trust                 Membership               Resources
          │                       │                       │
          └───────────────────────┼───────────────────────┘
                                  ▼
                              Routing
                                  │
                                  ▼
                            Data Plane
```

The crucial question becomes:

> **How can NROS discover components, capabilities, resources and communication endpoints across machines while preserving identity, authorization, locality, failure detection and deterministic behavior?**

That is the next architectural boundary.

# NROS — Part XXII: Discovery, Identity & the Distributed Runtime

We now move from the **local NROS runtime** to the **distributed NROS system**.

ROS was designed around networked nodes, and ROS 1 uses the **Master** as a discovery and coordination mechanism. NROS should avoid reproducing a central registry as a mandatory architectural dependency.

The target becomes:

> **A distributed runtime in which identity, discovery, capability, routing, and communication are separate but composable services.**

# 1. ROS 1's Master

The ROS 1 architecture can be simplified as:

```text
                  ROS MASTER
                  /        \
                 /          \
              Node A       Node B
                │             │
                └─────┬───────┘
                      │
                 peer-to-peer
                 communication
```

The Master primarily establishes relationships.

It does **not** normally carry the actual topic traffic.

That was a strong design choice.

But the architecture still introduces a critical dependency:

```text
Node
  │
  └── register → Master
```

# 2. NROS separation

NROS should split this into independent concepts:

```text
Identity
    │
    ▼
Discovery
    │
    ▼
Capability
    │
    ▼
Endpoint
    │
    ▼
Routing
    │
    ▼
Communication
```

No single "master" needs to own all of these responsibilities.

# 3. Identity

Every NROS component needs a stable runtime identity.

Conceptually:

```rust
ComponentId
```

Example:

```text
robot-01/navigation/planner
```

But the logical name should not itself be the cryptographic identity.

Therefore:

```text
LogicalName
      │
      ▼
ComponentId
      │
      ▼
Identity Credential
```

# 4. Logical identity vs cryptographic identity

This distinction is fundamental.

A component can be called:

```text
/robot01/planner
```

while possessing an independent identity:

```text
IdentityId = 7a91...
```

If the component is restarted:

```text
/robot01/planner
```

may remain the same logical component.

But the runtime instance can receive:

```text
InstanceId = 9821...
```

# 5. Three identities

A useful model:

```text
                Identity
                   │
       ┌───────────┼───────────┐
       ▼           ▼           ▼
   Logical       Instance    Principal
    Identity      Identity     Identity
```

### Logical identity

"What component is this?"

### Instance identity

"Which execution instance is this?"

### Principal identity

"Who is authorized to operate it?"

These should not be conflated.

# 6. Component registration

When a component starts:

```text
Component
   │
   ├── identity
   ├── capabilities
   ├── endpoints
   ├── metadata
   └── health
          │
          ▼
      Discovery
```

The runtime publishes a **component advertisement**.

# 7. Component advertisement

Conceptually:

```rust
struct ComponentAdvertisement {
    component_id: ComponentId,
    instance_id: InstanceId,
    endpoints: Vec<Endpoint>,
    capabilities: Vec<Capability>,
    metadata: Metadata,
    health: HealthStatus,
}
```

This is more useful than simply saying:

```text
"node exists"
```

# 8. Capability discovery

NROS should allow a component to ask:

> What can this component actually do?

Example:

```text
Planner
├── subscribe Pose
├── publish Path
├── request Map
└── execute Navigate
```

Another component:

```text
SafetyController
├── observe Velocity
├── write SafetyState
└── command EmergencyStop
```

Capabilities become discoverable objects.

# 9. Capability is not permission

Important distinction:

```text
Capability advertised
        ≠
Permission granted
```

Discovery answers:

> What exists?

Authorization answers:

> What am I allowed to use?

# 10. Capability negotiation

Two components may discover:

```text
A supports:
Detection v2

B supports:
Detection v1
```

NROS can negotiate:

```text
A
 │
 ▼
schema compatibility
 │
 ▼
adapter
 │
 ▼
B
```

This makes heterogeneous deployments easier.

# 11. Endpoint discovery

A communication capability must resolve to an endpoint.

Example:

```text
Stream:
    /robot01/lidar/scan

Endpoint:
    shm://segment/91
```

or:

```text
quic://10.0.0.21:7447/channel/91
```

The logical communication name remains stable.

The physical endpoint may change.

# 12. Location transparency

Therefore:

```text
Logical Stream
      │
      ▼
Discovery
      │
      ▼
Current Endpoint
```

The component doesn't need to know whether the receiver is:

```text
same thread
same process
same machine
another robot
edge server
cloud
```

# 13. Locality

However, NROS should expose locality to the runtime.

For example:

```text
LOCAL_THREAD
LOCAL_PROCESS
LOCAL_MACHINE
LOCAL_NETWORK
REMOTE_NETWORK
```

Why?

Because locality determines:

```text
latency
copy cost
failure modes
security boundary
transport
```

# 14. Local optimization

If publisher and subscriber share a process:

```text
Publisher
   │
   ▼
in-memory channel
```

If they share a host:

```text
Publisher
   │
   ▼
shared memory
```

If remote:

```text
Publisher
   │
   ▼
network transport
```

The logical channel stays unchanged.

# 15. Discovery architecture

A possible architecture:

```text
                     NROS DISCOVERY
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
       Local            Cluster          Remote
      Discovery        Discovery        Discovery
          │                │                │
          └────────────────┼────────────────┘
                           ▼
                     Discovery API
```

# 16. Local discovery

For one machine, NROS can use:

```text
Unix sockets
shared memory
OS IPC
local registry
```

The objective is low overhead.

# 17. Cluster discovery

For a robot fleet:

```text
Robot A
   │
   ├──────────────┐
   │              │
Robot B          Robot C
   │              │
   └──────┬───────┘
          ▼
      discovery
```

This could use a distributed discovery protocol rather than requiring a single master.

# 18. Failure detection

Distributed discovery must detect:

```text
component crashed
network disconnected
endpoint unavailable
identity expired
lease expired
```

A simple mechanism is a lease.

# 19. Discovery leases

When a component registers:

```text
Component
   │
   ▼
Lease
expires = T+5s
```

It periodically renews:

```text
heartbeat
heartbeat
heartbeat
```

If renewal stops:

```text
lease expires
      │
      ▼
component unavailable
```

# 20. Why leases are better than "alive" flags

A Boolean:

```text
alive = true
```

can become stale.

A lease:

```text
valid until T
```

has a defined expiration.

This provides a clear failure boundary.

# 21. Health vs availability

NROS should distinguish:

```text
Identity exists
       ≠
Component available
       ≠
Component healthy
       ≠
Component capable
```

For example:

```text
Planner
  identity: ✓
  reachable: ✓
  healthy: ✗
  accepting goals: ✗
```

These are different states.

# 22. Health model

A component could expose:

```text
UNKNOWN
STARTING
READY
DEGRADED
FAILED
STOPPING
```

And individual capabilities may have their own health.

# 23. Capability health

Example:

```text
Perception
├── camera_capture     READY
├── object_detection   DEGRADED
└── segmentation       FAILED
```

The component itself still exists.

NROS therefore shouldn't reduce health to a single Boolean.

# 24. Routing

Once discovery identifies an endpoint:

```text
logical target
      │
      ▼
endpoint resolution
      │
      ▼
routing
      │
      ▼
transport
```

Routing should remain separate from discovery.

Discovery says:

> Endpoint exists.

Routing says:

> This is how to reach it.

# 25. Multiple endpoints

A stream could have:

```text
Primary endpoint
Backup endpoint
Local endpoint
Remote endpoint
```

Example:

```text
/robot/map
    │
    ├── local SHM
    ├── robot-network QUIC
    └── remote gateway
```

Routing can select according to policy.

# 26. Routing policy

Possible policies:

```text
LOCAL_FIRST
LOWEST_LATENCY
HIGHEST_RELIABILITY
PRIMARY_ONLY
FAILOVER
LOAD_BALANCED
```

This turns topology into runtime policy rather than hardcoded addresses.

# 27. Failover

Suppose:

```text
Primary endpoint
      X
      │
      ▼
Backup endpoint
```

The logical communication contract remains unchanged.

This is particularly useful for distributed robotics.

# 28. Identity and routing

Security must participate in endpoint selection.

Suppose an endpoint claims:

```text
/robot01/motor
```

NROS should verify:

```text
Who owns this endpoint?
Is the identity authentic?
Is the certificate valid?
Is the capability still active?
```

before trusting it.

# 29. Secure discovery

Discovery messages themselves should not automatically be trusted.

A malicious component could advertise:

```text
/motor_controller
```

and attempt to intercept commands.

Therefore:

```text
Advertisement
      │
      ▼
Identity verification
      │
      ▼
Authorization
      │
      ▼
Trusted endpoint
```

# 30. Trust domains

A deployment can be divided into trust domains:

```text
             Robot
               │
      ┌────────┴────────┐
      ▼                 ▼
 Safety Domain       Application
      │                 │
      ▼                 ▼
trusted devices     user software
```

Cross-domain communication should require explicit policy.

# 31. Discovery scopes

Discovery itself can be scoped:

```text
LOCAL
ROBOT
FLEET
SITE
REMOTE
```

A component doesn't necessarily need global visibility.

This reduces:

```text
network traffic
metadata leakage
attack surface
```

# 32. Namespace + discovery scope

For example:

```text
/robot01/base/velocity
```

might be visible within:

```text
ROBOT
```

while:

```text
/mission/current
```

might be visible across:

```text
FLEET
```

# 33. Resource discovery

NROS discovery should not be limited to communication endpoints.

Components may discover:

```text
CPU
GPU
NPU
camera
lidar
motor
storage
network interface
simulation backend
```

These become **resources**.

# 34. Resource model

Conceptually:

```rust
Resource {
    id: ResourceId,
    kind: ResourceKind,
    owner: PrincipalId,
    capabilities: Vec<Capability>,
    availability: Availability,
}
```

Example:

```text
GPU-0
├── compute
├── inference
└── memory
```

# 35. Resource allocation

Now NROS can express:

```text
Agent
   │
   ▼
request GPU
   │
   ▼
resource manager
   │
   ▼
lease
   │
   ▼
GPU
```

This goes beyond conventional ROS node discovery.

# 36. Discovery becomes resource-aware

The question is no longer simply:

> Where is the navigation node?

It becomes:

> Where is a healthy navigation capability with the required resources, schema compatibility, authorization, and latency budget?

That is a fundamentally richer discovery problem.

# 37. Query model

Conceptually:

```rust
discover(
    capability = "navigation.execute",
    locality = LocalMachine,
    max_latency = 10.ms,
    required_schema = NavigationV2,
)
```

Result:

```text
Candidate A
Candidate B
Candidate C
```

The runtime can select the best candidate according to policy.

# 38. Capability graph

Discovery can therefore form:

```text
                  CAPABILITY GRAPH

Component A
 ├── publishes Pose
 ├── consumes LaserScan
 └── provides Localization

Component B
 ├── consumes Pose
 ├── provides Navigation
 └── requires Map

Component C
 └── provides Map
```

NROS can reason over this graph.

# 39. Automatic composition

Suppose an application requests:

```text
Navigate
```

Discovery can find:

```text
Navigation
    │
    ├── requires Pose
    │       │
    │       └── Localization
    │
    └── requires Map
            │
            └── Mapping
```

This becomes a dependency graph.

# 40. Dependency resolution

NROS can construct:

```text
Mapping
   │
   ▼
Map
   │
   ▼
Navigation
   ▲
   │
Pose
   │
   ▲
Localization
```

This is significantly more expressive than manually launching a fixed set of ROS nodes.

# 41. But discovery should not become orchestration

Important architectural boundary:

```text
Discovery:
"What exists?"

Scheduler:
"When should it run?"

Orchestrator:
"What should be deployed?"

Policy:
"What is allowed?"

Communication:
"How do they exchange data?"
```

NROS should preserve these boundaries.

# 42. Discovery and lifecycle

A component may transition:

```text
DISCOVERED
   ↓
STARTING
   ↓
READY
   ↓
DEGRADED
   ↓
STOPPING
   ↓
REMOVED
```

The discovery system observes this lifecycle.

# 43. Component lifecycle

A richer component lifecycle:

```text
                 ┌───────────┐
                 │ DISCOVERED│
                 └─────┬─────┘
                       ▼
                  INITIALIZING
                       │
                       ▼
                     READY
                       │
             ┌─────────┼─────────┐
             ▼         ▼         ▼
          DEGRADED   STOPPING   FAILED
             │         │
             └─────────┘
                       ▼
                    REMOVED
```

This gives the runtime a formal state machine.

# 44. Startup

A component startup sequence becomes:

```text
Process starts
     │
     ▼
Acquire identity
     │
     ▼
Load configuration
     │
     ▼
Initialize resources
     │
     ▼
Register capabilities
     │
     ▼
Publish endpoints
     │
     ▼
READY
```

Only then should it advertise itself as operational.

# 45. Shutdown

Shutdown should be equally explicit:

```text
STOP REQUEST
     │
     ▼
STOP ACCEPTING NEW WORK
     │
     ▼
CANCEL OPERATIONS
     │
     ▼
FLUSH REQUIRED STATE
     │
     ▼
RELEASE LEASES
     │
     ▼
WITHDRAW ENDPOINTS
     │
     ▼
STOP
```

# 46. Crash behavior

If the process disappears unexpectedly:

```text
process
   X
   │
lease expires
   │
   ├── endpoints invalidated
   ├── resources released
   ├── operations failed
   └── state recovery triggered
```

This connects discovery with the recovery system.

# 47. Discovery + state recovery

Suppose the navigation component crashes.

The runtime detects:

```text
Navigation lease expired
```

Then:

```text
Discovery
   │
   ▼
failure event
   │
   ▼
Recovery policy
   │
   ├── restart
   ├── failover
   ├── restore checkpoint
   └── safe-stop
```

This is a major step toward autonomous infrastructure.

# 48. NROS Control Plane

We can now define a coherent control plane:

```text
                      CONTROL PLANE
                           │
       ┌───────────────────┼───────────────────┐
       ▼                   ▼                   ▼
    Identity            Discovery           Policy
       │                   │                   │
       ▼                   ▼                   ▼
    Trust              Membership         Authorization
       │                   │                   │
       └───────────────────┼───────────────────┘
                           ▼
                        Routing
                           │
                           ▼
                       Lifecycle
                           │
                           ▼
                     Resource Mgmt
```

# 49. Data Plane vs Control Plane

NROS now has a clean separation:

```text
CONTROL PLANE
────────────────────────
identity
discovery
routing
lifecycle
policy
resource management

DATA PLANE
────────────────────────
streams
messages
requests
commands
operations
state events
```

This is one of the most important architectural boundaries.

# 50. Failure isolation

If discovery temporarily fails:

```text
Discovery X
```

existing communication may still continue:

```text
Existing endpoint
       │
       ▼
Data Plane
       │
       ▼
Communication continues
```

provided the relevant lease and endpoint remain valid.

This prevents the control plane from becoming an unnecessary data-plane bottleneck.

# 51. Cached discovery

Components can cache:

```text
known endpoint
known schema
known capability
known identity
```

with expiration:

```text
cache
  │
  ├── valid
  └── stale
```

A stale cache entry must not automatically be treated as authoritative.

# 52. Epochs

Distributed systems benefit from an epoch/generation concept.

Example:

```text
Robot01 epoch 41
```

After restart:

```text
Robot01 epoch 42
```

This prevents an old instance from accidentally being mistaken for the current instance.

# 53. Fencing

Epochs can support **fencing**.

Suppose:

```text
Old controller = epoch 41
New controller = epoch 42
```

The motor controller can reject commands from epoch 41.

Thus:

```text
old authority
      X
      │
      ▼
actuator
```

This is extremely valuable for safe failover.

# 54. Identity + lease + epoch

These three concepts combine into:

```text
Principal
   │
   ▼
Component Identity
   │
   ▼
Instance Identity
   │
   ▼
Epoch
   │
   ▼
Lease
```

Together they provide a strong foundation for distributed ownership.

# 55. The NROS distributed graph

The resulting graph is now:

```text
                         NROS
                          │
             ┌────────────┴────────────┐
             ▼                         ▼
        CONTROL PLANE               DATA PLANE
             │                         │
     ┌───────┼────────┐        ┌───────┼────────┐
     ▼       ▼        ▼        ▼       ▼        ▼
 Identity Discovery Policy   Stream Operation State
     │       │        │        │       │        │
     └───────┼────────┘        └───────┼────────┘
             ▼                         ▼
          Routing                  Activation
             │                         │
             └────────────┬────────────┘
                          ▼
                      Execution
                          │
                          ▼
                        Effects
```

# 56. ROS → NROS transformation

The conceptual migration now looks like:

```text
ROS 1
─────────────────────────────
Master
Node registration
Topic discovery
Service discovery
Parameter Server
Node lifecycle
Network URI
```

becomes:

```text
NROS
─────────────────────────────
Identity
Discovery
Capability Registry
Endpoint Registry
State Plane
Lifecycle Manager
Routing
Resource Manager
```

No single component needs to become the new "ROS Master."

# 57. The deeper architectural pattern

At this point, NROS has three fundamental planes:

```text
┌──────────────────────────────────────────────┐
│                  NROS                        │
│                                              │
│  CONTROL PLANE                               │
│  Identity • Discovery • Policy • Routing     │
│                                              │
│  DATA PLANE                                  │
│  Streams • Operations • Commands • Events    │
│                                              │
│  STATE PLANE                                 │
│  Runtime • Configuration • Mission • State  │
│                                              │
└──────────────────────────────────────────────┘
```

These converge through:

```text
                 ACTIVATION
                     │
                     ▼
                 SCHEDULER
                     │
                     ▼
                 EXECUTOR
                     │
                     ▼
                   EFFECT
```

This gives us the skeleton of a true NROS runtime.

# 58. Next — Part XXIII: Lifecycle, Executors & Scheduling

The next layer is where NROS begins to depart most dramatically from traditional ROS architecture.

ROS fundamentally revolves around:

```text
callbacks
executors
spinning
```

NROS should instead define:

```text
Events
   ↓
Activations
   ↓
Scheduling
   ↓
Execution
   ↓
Effects
   ↓
Checkpoint
```

The central question becomes:

> **What is the correct execution model when the primary scheduling unit is not a callback or Future, but a causally-triggered autonomous runtime activity?**

That leads directly into **NROS's scheduler, executor, lifecycle, cancellation, priorities, deadlines, resource accounting, and deterministic execution model**.

# NROS — Part XXIII: Lifecycle, Scheduler & Executor

This is the point where **NROS stops being merely a robotics middleware redesign and starts becoming a runtime architecture**.

ROS traditionally centers execution around:

```text
Node
  ↓
Callback
  ↓
Executor
```

NROS should instead center execution around:

```text
Event
  ↓
Activation
  ↓
Scheduling
  ↓
Execution
  ↓
Effect
  ↓
State / Event
```

The difference is fundamental.

# 1. The ROS execution model

A simplified ROS-style model is:

```text
Topic
  │
  ▼
Callback
  │
  ▼
Executor
  │
  ▼
User code
```

The callback is effectively the unit of work.

This works well for conventional reactive robotics.

But it becomes increasingly awkward when we need:

- priorities
- deadlines
- cancellation
- resource budgets
- causality
- autonomous planning
- checkpointing
- admission control
- deterministic execution
- distributed execution

# 2. NROS execution model

NROS should promote the **Activation** to the primary execution object:

```text
                  EVENT
                    │
                    ▼
                ACTIVATION
                    │
             ┌──────┼──────┐
             ▼      ▼      ▼
          priority deadline resources
             │      │      │
             └──────┼──────┘
                    ▼
                SCHEDULER
                    │
                    ▼
                 EXECUTOR
                    │
                    ▼
                  EFFECT
```

# 3. What is an Activation?

An activation represents:

> **A bounded unit of runtime work caused by a specific event or condition.**

Examples:

```text
MessageReceived
TimerExpired
StateChanged
CommandAccepted
ResourceAvailable
OperationProgressed
AgentWakeup
RecoveryTriggered
```

Each can produce an activation.

# 4. Activation identity

Every activation gets a stable identity:

```text
ActivationId
```

For example:

```text
Activation #9182
```

with metadata:

```text
Activation
├── id
├── source
├── cause
├── priority
├── deadline
├── budget
├── resource requirements
├── state
└── trace context
```

# 5. Activation causality

Suppose:

```text
LaserScan
   ↓
ObstacleDetection
   ↓
SafetyDecision
   ↓
EmergencyStop
```

NROS records:

```text
Activation A
    │
    └── causes
         │
         ▼
Activation B
         │
         └── causes
              │
              ▼
           Activation C
```

The runtime can therefore reconstruct why work happened.

# 6. Activation lifecycle

An activation should have an explicit lifecycle:

```text
CREATED
   ↓
ADMITTED
   ↓
READY
   ↓
RUNNING
   ↓
COMPLETED
```

Alternative paths:

```text
READY
  ├── CANCELLED
  ├── EXPIRED
  ├── REJECTED
  └── FAILED
```

# 7. Admission control

Creating work does not mean the runtime must execute it immediately.

Instead:

```text
Event
  ↓
Activation
  ↓
Admission Control
  │
  ├── ACCEPT
  └── REJECT
```

This allows NROS to protect the system from overload.

# 8. Why admission matters

Suppose 10,000 sensor events arrive:

```text
10,000 events
      │
      ▼
10,000 activations
```

A naive runtime can collapse under its own workload.

NROS can instead enforce:

```text
max pending activations = 100
```

and apply a defined overload policy.

# 9. Overload policy

Possible policies:

```text
DROP
COALESCE
SAMPLE
DEFER
BACKPRESSURE
PRIORITIZE
ESCALATE
FAIL_SAFE
```

The policy depends on the activation type.

# 10. Activation coalescing

Consider:

```text
PoseChanged
PoseChanged
PoseChanged
PoseChanged
```

A planner may only need:

```text
Latest PoseChanged
```

NROS can coalesce:

```text
A1
A2
A3
A4
 ↓
A4
```

while preserving the latest state.

# 11. Critical events cannot always coalesce

For:

```text
EmergencyStop
FaultDetected
SafetyViolation
```

every event may matter.

Therefore:

```text
coalescing = forbidden
```

This demonstrates why event semantics must be explicit.

# 12. Scheduling dimensions

NROS scheduling should consider:

```text
priority
deadline
release time
resource requirements
CPU affinity
memory budget
criticality
dependency readiness
causal constraints
```

A single FIFO queue is insufficient.

# 13. Priority

Example:

```text
CRITICAL
    EmergencyStop

HIGH
    MotorControl

NORMAL
    Navigation

LOW
    Mapping

BACKGROUND
    Logging
```

The scheduler should make priority an explicit policy.

# 14. Deadline

An activation can declare:

```text
deadline = 10ms
```

The scheduler knows:

```text
current time
deadline
estimated execution cost
```

and can determine whether execution remains useful.

# 15. Deadline expiration

If:

```text
now > deadline
```

the runtime can transition:

```text
READY
  │
  ▼
EXPIRED
```

rather than executing obsolete work.

For safety-critical operations, the appropriate behavior must be defined by the system's safety policy.

# 16. Execution budget

An activation can also have a budget:

```text
CPU budget = 2ms
memory budget = 4MB
```

Conceptually:

```text
Activation
    │
    ├── deadline
    ├── CPU budget
    └── memory budget
```

This allows resource accounting.

# 17. Budget exhaustion

If an activation exceeds its budget:

```text
RUNNING
   │
   ▼
BUDGET_EXCEEDED
```

Possible policy:

```text
terminate
defer
throttle
escalate
mark degraded
```

The correct choice depends on the workload.

# 18. Deterministic scheduling

For certain robotics workloads, NROS should support deterministic scheduling policies.

For example:

```text
deadline + priority + fixed ordering
```

with deterministic tie-breaking:

```text
priority
   ↓
deadline
   ↓
activation_id
```

This makes behavior reproducible.

# 19. Real-time scheduling

For real-time execution, the runtime must avoid uncontrolled operations such as:

```text
dynamic allocation
blocking I/O
unbounded locks
unbounded queues
unbounded computation
```

within critical paths.

NROS should therefore have an explicit **real-time execution profile**.

# 20. Execution profiles

Rather than assuming every workload is real-time:

```text
NROS Execution Profiles

REALTIME
SOFT_REALTIME
BEST_EFFORT
BATCH
BACKGROUND
SIMULATION
```

Each profile can impose different guarantees.

# 21. Executor

The executor consumes admitted activations:

```text
Scheduler
    │
    ▼
Ready Queue
    │
    ▼
Executor
    │
    ├── Worker 1
    ├── Worker 2
    ├── Worker 3
    └── Worker N
```

The executor is responsible for actual computation.

# 22. Scheduler ≠ Executor

This distinction is important.

### Scheduler

Decides:

> **What should run next?**

### Executor

Performs:

> **Run this activation.**

This separation allows different scheduling policies without rewriting execution infrastructure.

# 23. Single-threaded executor

For deterministic systems:

```text
Activation Queue
      │
      ▼
┌─────────────┐
│ Single Core │
└─────────────┘
```

Advantages:

- predictable ordering
- simpler synchronization
- easier debugging
- deterministic replay

# 24. Multi-threaded executor

For throughput:

```text
                 Scheduler
                     │
        ┌────────────┼────────────┐
        ▼            ▼            ▼
     Worker 1     Worker 2     Worker 3
```

But concurrency introduces:

```text
races
ordering differences
contention
priority inversion
non-determinism
```

NROS should make these tradeoffs explicit.

# 25. Work stealing

For CPU-heavy workloads:

```text
Worker A ← queue
Worker B ← queue
Worker C ← queue
```

Workers can steal work when idle.

Useful for:

```text
perception
planning
simulation
AI inference
```

Less suitable for tightly deterministic control loops.

# 26. CPU affinity

An activation can optionally specify:

```text
CPU affinity = {2,3}
```

For example:

```text
SafetyController
   → CPU 0

Perception
   → CPU 2-5

Logging
   → any CPU
```

# 27. Resource affinity

Scheduling can extend beyond CPUs:

```text
GPU
NPU
DSP
FPGA
```

An activation can declare:

```text
requires = GPU
```

Then:

```text
Scheduler
   │
   ▼
Resource allocator
   │
   ▼
GPU executor
```

# 28. Resource-aware scheduling

The scheduler now evaluates:

```text
Can run?
 ├── CPU available?
 ├── memory available?
 ├── GPU available?
 ├── capability authorized?
 ├── dependencies satisfied?
 └── deadline feasible?
```

Only then is an activation admitted.

# 29. Dependency constraints

Some activations depend on previous results:

```text
A
 ↓
B
 ↓
C
```

NROS can express:

```text
B cannot execute until A completed.
```

This creates an execution DAG.

# 30. Execution DAG

Example:

```text
Camera
  │
  ├──► Detection ──┐
  │                │
  └──► Depth ──────┤
                   ▼
                 Fusion
                   │
                   ▼
                Planning
```

The scheduler can execute Detection and Depth concurrently.

# 31. Parallelism

Instead of:

```text
Camera
 ↓
Detection
 ↓
Depth
 ↓
Fusion
```

we can execute:

```text
        ┌─ Detection ─┐
Camera ─┤             ├─► Fusion
        └─ Depth ─────┘
```

This exposes available parallelism.

# 32. Critical path

The runtime can identify:

```text
Camera → Detection → Fusion → Planning
```

as the critical path.

Optimization can then focus on the longest causal chain rather than arbitrary functions.

# 33. Cancellation

Cancellation must be a first-class runtime primitive.

```text
Activation
    │
    ▼
Cancellation requested
    │
    ▼
CANCELLED
```

Cancellation should propagate through dependent work where policy allows.

# 34. Cancellation tree

Suppose:

```text
Mission
 ├── Navigation
 │    ├── Planning
 │    └── Control
 └── Perception
```

Cancelling Mission can propagate:

```text
Mission X
   │
   ├── Navigation X
   │     ├── Planning X
   │     └── Control X
   │
   └── Perception X
```

This is more powerful than individually stopping callbacks.

# 35. Structured concurrency

NROS can adopt a structured concurrency principle:

> Child activities must have a well-defined relationship with their parent activity.

For example:

```text
Mission
 ├── Perception
 ├── Planning
 └── Navigation
```

The runtime knows when the mission's child activities are:

```text
started
completed
failed
cancelled
```

# 36. Failure propagation

If a critical child fails:

```text
Planning
   X
```

the parent can apply policy:

```text
retry
replace
degrade
cancel mission
safe-stop
```

Failure therefore becomes part of orchestration rather than an unhandled exception.

# 37. Supervisor

NROS should provide a supervision mechanism.

```text
Supervisor
   │
   ├── Component A
   ├── Component B
   └── Component C
```

The supervisor observes:

```text
health
failures
timeouts
resource exhaustion
```

and applies recovery policies.

# 38. Supervisor hierarchy

Large systems can have:

```text
System Supervisor
       │
       ├── Navigation Supervisor
       │      ├── Planner
       │      └── Controller
       │
       └── Perception Supervisor
              ├── Camera
              └── Detector
```

This provides hierarchical fault containment.

# 39. Lifecycle manager

Component lifecycle should be independent of the scheduler:

```text
Lifecycle Manager
      │
      ├── configure
      ├── activate
      ├── deactivate
      ├── cleanup
      └── shutdown
```

This resembles useful ideas from ROS 2 lifecycle nodes but becomes a general NROS runtime facility.

# 40. Component lifecycle

A component could follow:

```text
UNCONFIGURED
      │
      ▼
INACTIVE
      │
      ▼
ACTIVE
      │
      ▼
DEACTIVATING
      │
      ▼
INACTIVE
```

Failure transitions:

```text
ERROR
   │
   ├── RECOVER
   └── FINALIZE
```

# 41. Why lifecycle matters

A component should not necessarily be:

```text
process exists = operational
```

Instead:

```text
process exists
      ≠
initialized
      ≠
configured
      ≠
ready
      ≠
active
```

This distinction is essential for distributed startup.

# 42. Activation lifecycle + component lifecycle

The runtime now has two levels:

```text
COMPONENT
   │
   └── owns
         │
         ▼
     ACTIVATIONS
```

Component:

```text
ACTIVE
```

while an individual activation can be:

```text
RUNNING
```

This separation prevents confusing component state with individual work state.

# 43. Execution context

Every activation should execute inside a context:

```text
ExecutionContext
├── component
├── activation
├── cancellation token
├── deadline
├── budget
├── capabilities
├── trace context
└── state access
```

This gives user code a controlled runtime environment.

# 44. Effects

NROS should explicitly model effects.

An activation might produce:

```text
Effect
├── publish message
├── update state
├── issue command
├── acquire resource
├── release resource
└── spawn activation
```

Therefore execution becomes:

```text
Activation
   │
   ▼
Computation
   │
   ▼
Effects
```

# 45. Effect boundary

This is an extremely useful architectural boundary.

Pure computation:

```text
input → output
```

can be tested deterministically.

Effects:

```text
motor command
filesystem write
network transmission
state mutation
```

can be controlled and audited.

# 46. Effect authorization

Before committing an effect:

```text
Activation
   │
   ▼
Effect
   │
   ▼
Policy
   │
   ├── allowed
   └── denied
```

This provides another security boundary.

# 47. Effect receipts

An effect can produce:

```text
EffectReceipt
├── effect_id
├── status
├── timestamp
├── resource
├── causal activation
└── resulting state
```

For example:

```text
MotorCommand
    │
    ▼
Effect #772
    │
    ▼
accepted
```

# 48. Deterministic replay

Because NROS records:

```text
events
activations
ordering
state revisions
effects
```

it can potentially replay:

```text
Event log
   │
   ▼
Scheduler
   │
   ▼
Executor
   │
   ▼
Reconstructed execution
```

This is extremely valuable for robotics debugging.

# 49. Simulation

The same runtime architecture can operate in simulation:

```text
NROS Runtime
    │
    ├── Physical Transport
    │
    └── Simulation Transport
```

The communication contracts remain unchanged.

# 50. Virtual time

Simulation can introduce:

```text
VirtualClock
```

Then:

```text
Timer
Deadline
Timeout
Lease
Lifecycle
```

can operate against simulation time.

This makes deterministic simulation much easier.

# 51. Time abstraction

NROS should not scatter direct system-clock calls throughout the runtime.

Instead:

```rust
trait Clock {
    fn now(&self) -> Timestamp;
}
```

Implementations:

```text
SystemClock
MonotonicClock
SimulatedClock
ReplayClock
```

# 52. Scheduler + time

The scheduler then evaluates:

```text
now
release_time
deadline
timeout
lease_expiry
```

through a single time abstraction.

This reduces temporal inconsistencies.

# 53. Runtime observability

Every activation can generate:

```text
ActivationCreated
ActivationAdmitted
ActivationStarted
ActivationSuspended
ActivationResumed
ActivationCompleted
ActivationFailed
ActivationCancelled
ActivationExpired
```

These events feed the trace system.

# 54. Scheduler telemetry

The runtime should expose:

```text
queue depth
activation latency
execution duration
deadline misses
budget overruns
CPU utilization
resource contention
cancellation rate
```

This turns scheduler behavior into measurable data.

# 55. Deadline miss

A deadline miss should be observable:

```text
Activation #901
deadline = 10ms
actual = 14ms

→ DEADLINE_MISSED
```

The runtime can then apply policy:

```text
record
retry
degrade
escalate
fail-safe
```

# 56. Runtime health

NROS health becomes multidimensional:

```text
Runtime
├── scheduler health
├── executor health
├── communication health
├── state health
├── resource health
└── component health
```

No single "system healthy" Boolean is sufficient.

# 57. Executor architecture

A possible crate decomposition:

```text
nros-runtime/
├── activation
├── scheduler
├── executor
├── lifecycle
├── supervisor
├── cancellation
├── budget
├── deadline
├── resource
├── effect
├── clock
└── trace
```

The exact boundaries should follow the existing NROS workspace rather than being imposed blindly.

# 58. Core runtime traits

Conceptually:

```rust
trait Scheduler {
    fn submit(&mut self, activation: Activation);
    fn next(&mut self) -> Option<Activation>;
}

trait Executor {
    fn execute(
        &self,
        activation: Activation,
    ) -> ExecutionResult;
}
```

Real NROS APIs should subsequently encode stronger ownership and lifecycle guarantees.

# 59. Runtime loop

A simplified NROS runtime loop:

```text
┌───────────────────────────────┐
│           EVENT LOOP          │
└───────────────┬───────────────┘
                │
                ▼
             Receive
                │
                ▼
          Create Activation
                │
                ▼
          Admission Control
                │
                ▼
             Schedule
                │
                ▼
             Execute
                │
                ▼
             Effects
                │
                ▼
           Commit State
                │
                ▼
          Emit Events
                │
                └──────────────► loop
```

This is the heart of NROS.

# 60. ROS callback vs NROS activation

The conceptual transformation:

```text
ROS

message
  ↓
callback
  ↓
executor
```

becomes:

```text
NROS

event
  ↓
activation
  ↓
admission
  ↓
scheduler
  ↓
executor
  ↓
effects
  ↓
state/event
```

The latter provides many more control points.

# 61. Autonomous agents

This architecture also naturally accommodates autonomous agents.

An agent can become a runtime component:

```text
Agent
  │
  ├── observes
  ├── plans
  ├── executes
  ├── reflects
  └── checkpoints
```

Each cycle produces activations:

```text
Observation
    ↓
Agent Activation
    ↓
Plan
    ↓
Execution Activation
    ↓
Effect
    ↓
Observation
```

This connects directly to an **agentic-native runtime model**.

# 62. Agent scheduling

Instead of treating an agent as an enormous callback:

```text
agent.run()
```

NROS can schedule agent activities independently:

```text
Agent
├── perception activation
├── reasoning activation
├── tool activation
├── action activation
└── checkpoint activation
```

Each can have separate:

```text
priority
budget
deadline
capabilities
```

# 63. Robot control + agents

This also gives us a hierarchy:

```text
                NROS
                 │
       ┌─────────┴──────────┐
       ▼                    ▼
 Real-time Control      Agent Runtime
       │                    │
       ▼                    ▼
 Actuator Effects       Planning Effects
       │                    │
       └──────────┬─────────┘
                  ▼
                State
```

The real-time layer can remain deterministic while the agent layer operates under different execution guarantees.

# 64. The NROS runtime stack

We can now visualize the architecture as:

```text
┌────────────────────────────────────────────┐
│              APPLICATIONS                  │
│ Robots • Agents • Missions • Controllers   │
├────────────────────────────────────────────┤
│              NROS COMPONENTS               │
├────────────────────────────────────────────┤
│              ACTIVATION MODEL              │
├────────────────────────────────────────────┤
│          SCHEDULER / EXECUTORS             │
├────────────────────────────────────────────┤
│          COMMUNICATION / STATE             │
├────────────────────────────────────────────┤
│       DISCOVERY / IDENTITY / POLICY        │
├────────────────────────────────────────────┤
│           TRANSPORT / RESOURCES            │
├────────────────────────────────────────────┤
│                 OS / RTOS                  │
├────────────────────────────────────────────┤
│                 HARDWARE                   │
└────────────────────────────────────────────┘
```

# 65. The major conceptual shift

ROS asks:

> **Which callback should execute?**

NROS asks:

> **Which causally-ready activity should execute, under which temporal, resource, security, and lifecycle constraints, and what effects is it authorized to produce?**

That is the fundamental runtime transformation.

# 66. NROS execution equation

We can summarize an activation as:

```text
Activation
=
Cause
+
Intent
+
Priority
+
Deadline
+
Budget
+
Resources
+
Capabilities
+
Lifecycle
+
Causality
```

Its execution produces:

```text
Effects
+
State Changes
+
Events
```

which generate future activations.

# 67. Closed-loop NROS

The architecture therefore becomes:

```text
              ┌───────────────┐
              │    EVENT      │
              └───────┬───────┘
                      ▼
              ┌───────────────┐
              │  ACTIVATION   │
              └───────┬───────┘
                      ▼
              ┌───────────────┐
              │  SCHEDULER    │
              └───────┬───────┘
                      ▼
              ┌───────────────┐
              │   EXECUTOR    │
              └───────┬───────┘
                      ▼
              ┌───────────────┐
              │    EFFECT     │
              └───────┬───────┘
                      ▼
              ┌───────────────┐
              │     STATE     │
              └───────┬───────┘
                      ▼
              ┌───────────────┐
              │     EVENT     │
              └───────────────┘
```

This is the **NROS Runtime Loop**.

# 68. What NROS now has

After the previous architectural layers, we have established:

| Layer | NROS abstraction |
|---|---|
| Communication | Streams, Requests, Operations, Commands |
| Data | Typed Envelopes |
| State | Versioned State |
| Configuration | Validated, scoped State |
| Discovery | Capability/Endpoint Discovery |
| Identity | Component/Instance/Principal |
| Security | Capabilities + Policy |
| Resources | Resource + Lease |
| Lifecycle | Explicit component lifecycle |
| Execution | Activations |
| Scheduling | Priority/Deadline/Resource-aware |
| Executor | Controlled execution |
| Effects | Explicit side-effect boundary |
| Causality | Event → Activation → Effect |
| Recovery | Supervisor + checkpoint foundation |

This is now much more than a ROS API rewrite.

# 69. Next — Part XXIV: Time, Determinism & Real-Time NROS

The next layer is critical for robotics.

We need to formalize:

```text
Clock
Time
Deadline
Period
Jitter
Latency
WCET
Budget
Priority
Scheduling
Determinism
```

and establish the boundary between:

```text
┌──────────────────────────┐
│      HARD REAL-TIME      │
├──────────────────────────┤
│      SOFT REAL-TIME      │
├──────────────────────────┤
│      BEST EFFORT         │
├──────────────────────────┤
│      AUTONOMOUS/AI       │
└──────────────────────────┘
```

The central question becomes:

> **How can NROS provide a deterministic execution substrate without pretending that every part of a modern robotic/agentic system can—or should—be hard real-time?**

That will define the **NROS temporal model** and establish the foundation for integrating real-time controllers, simulation, distributed systems, and autonomous agents in one runtime.

# NROS — Part XXIV: Time, Determinism & Real-Time Model

The next step is to make **time a first-class runtime primitive**.

ROS traditionally treats time largely as an API concern. NROS should treat time as a **system-level semantic constraint** affecting scheduling, state, communication, execution, simulation, replay, and safety.

## 1. NROS temporal model

The runtime should distinguish at least four concepts:

```text
Time
├── Instant
├── Duration
├── Deadline
└── Period
```

For example:

```text
Instant   = 12.450 s
Duration  = 2 ms
Deadline  = 12.460 s
Period    = 10 ms
```

These must not be conflated.

# 2. Clock abstraction

NROS should never assume that `system time` is the only clock.

```text
trait Clock {
    fn now(&self) -> Instant;
}
```

Possible implementations:

```text
SystemClock
MonotonicClock
SimClock
ReplayClock
ExternalClock
```

The runtime can therefore operate in physical, simulated, or replayed time.

# 3. Wall clock vs monotonic clock

A wall clock can jump:

```text
12:00
12:01
11:58   ← synchronization adjustment
```

A monotonic clock should not:

```text
0
1
2
3
4
```

Scheduling and timeout calculations should normally use a monotonic time source.

Wall-clock time should be reserved for:

```text
timestamps
logging
calendar semantics
external synchronization
```

# 4. Temporal domains

NROS should explicitly model the possibility of multiple temporal domains:

```text
                    NROS
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
     Physical       Simulation     Replay
       Time           Time          Time
```

An activation must know which clock governs its temporal semantics.

# 5. Deadlines

A deadline means:

> **The latest instant at which completion remains temporally valid.**

Example:

```text
Activation A
release = 10ms
deadline = 20ms
```

The scheduler evaluates:

```text
remaining = deadline - now
```

# 6. Periodic activities

Some robotic workloads are periodic:

```text
Control loop → every 1 ms
State update  → every 10 ms
Localization  → every 20 ms
Diagnostics   → every 1 s
```

NROS should represent periodic work explicitly rather than implementing it as arbitrary timer callbacks.

```text
PeriodicActivity
├── period
├── phase
├── deadline
└── execution budget
```

# 7. Release time

A periodic activation has a release time:

```text
t0
 │
 ├── activation #1
 │
t0 + T
 │
 ├── activation #2
 │
t0 + 2T
 │
 └── activation #3
```

This allows the scheduler to distinguish:

```text
not released yet
ready
late
expired
```

# 8. Jitter

Suppose a controller should execute every:

```text
1 ms
```

but actual execution occurs at:

```text
1.00 ms
1.03 ms
0.98 ms
1.07 ms
```

The deviation is **jitter**.

NROS should measure it explicitly:

```text
release_jitter
start_jitter
completion_jitter
```

# 9. Latency

We should distinguish several latencies.

### Communication latency

```text
publish → receive
```

### Scheduling latency

```text
ready → running
```

### Execution latency

```text
running → completed
```

### End-to-end latency

```text
source event → final effect
```

These should not collapse into one metric.

# 10. End-to-end timing

Consider:

```text
Sensor
  ↓
Transport
  ↓
Perception
  ↓
Planning
  ↓
Controller
  ↓
Actuator
```

NROS should be able to measure:

```text
T_sensor
+ T_transport
+ T_schedule
+ T_perception
+ T_planning
+ T_control
+ T_actuation
```

The total is the actual control-path latency.

# 11. Timing budget

A mission can specify:

```text
end-to-end budget = 20 ms
```

The runtime can distribute that budget:

```text
20 ms
├── transport       2 ms
├── perception      5 ms
├── planning        8 ms
├── control         3 ms
└── margin          2 ms
```

This enables compositional timing analysis.

# 12. WCET

For hard real-time work we need the concept of:

**Worst-Case Execution Time (WCET).**

For example:

```text
Controller WCET = 700 µs
Period          = 1 ms
```

That gives the scheduler meaningful information.

But NROS should distinguish:

```text
declared WCET
measured execution
verified WCET
```

A user declaration is not automatically a guarantee.

# 13. Timing assurance levels

A useful NROS model:

```text
Timing Assurance

UNSPECIFIED
MEASURED
BOUNDED
ANALYZED
VERIFIED
CERTIFIED
```

This prevents documentation from claiming stronger guarantees than the evidence supports.

# 14. Hard real-time

Hard real-time means:

> Missing a required temporal constraint constitutes system failure.

Example:

```text
Emergency control
Safety interlock
Motor protection
```

These should receive special treatment.

# 15. Soft real-time

Soft real-time means:

> Missing deadlines degrades quality but does not necessarily invalidate the system.

Examples:

```text
camera processing
visualization
mapping
speech recognition
AI inference
```

# 16. Best-effort

Some work has no meaningful deadline:

```text
logs
telemetry
indexing
analytics
maintenance
```

NROS should not waste hard-real-time resources protecting these workloads.

# 17. Four execution classes

A practical model:

```text
┌──────────────────────────────┐
│ HARD_RT                      │
│ deterministic / bounded      │
├──────────────────────────────┤
│ SOFT_RT                      │
│ deadline-sensitive           │
├──────────────────────────────┤
│ BEST_EFFORT                  │
│ throughput-oriented          │
├──────────────────────────────┤
│ BACKGROUND                   │
│ opportunistic                │
└──────────────────────────────┘
```

# 18. Why AI belongs outside HARD_RT

Modern AI inference can exhibit:

```text
variable execution time
dynamic memory usage
GPU scheduling
thermal throttling
model-dependent latency
driver variability
```

Therefore an NROS AI planner should normally be:

```text
SOFT_RT
```

or:

```text
BEST_EFFORT
```

while the low-level controller remains:

```text
HARD_RT
```

# 19. Hierarchical timing

This creates:

```text
Mission
 │
 ├── AI Planning      SOFT_RT
 │
 ├── Navigation       SOFT_RT
 │
 └── Motor Control    HARD_RT
```

The system does not need to make the entire stack hard real-time.

# 20. Temporal isolation

A low-priority AI workload must not destroy the control loop.

Conceptually:

```text
CPU
├── reserved RT capacity
│    └── controller
│
└── elastic capacity
     ├── AI
     ├── perception
     └── logging
```

This is temporal isolation.

# 21. Priority inversion

Suppose:

```text
HIGH priority controller
       │
       ▼
needs lock
       │
       ▼
LOW priority logger owns lock
```

The controller can become blocked by the logger.

NROS must therefore account for synchronization effects.

Possible mechanisms include:

```text
priority inheritance
priority ceiling
lock-free structures
ownership transfer
```

# 22. Real-time memory

Hard real-time components should avoid uncontrolled allocation.

Instead:

```text
startup
   ↓
allocate
   ↓
initialize pools
   ↓
runtime
   ↓
no unexpected allocation
```

Possible NROS primitives:

```text
FixedPool
StaticBuffer
BoundedQueue
PreallocatedMessage
```

# 23. Bounded queues

An unbounded queue:

```text
∞
```

is dangerous for deterministic systems.

NROS should support:

```text
BoundedQueue<N>
```

with explicit overflow behavior:

```text
DROP_NEWEST
DROP_OLDEST
REJECT
BLOCK
COALESCE
ESCALATE
```

# 24. Backpressure

For non-critical streams:

```text
Producer
   │
   ▼
Queue
   │
   ▼
Consumer
```

If the consumer cannot keep up:

```text
Producer ← BACKPRESSURE
```

This prevents silent memory growth.

# 25. Deadline-aware queues

A stronger queue can order activations according to:

```text
priority
deadline
release time
criticality
```

rather than simply arrival order.

# 26. Temporal priority

Consider:

```text
A priority = HIGH
deadline = 100ms

B priority = NORMAL
deadline = 2ms
```

A pure priority scheduler executes:

```text
A → B
```

A deadline-aware scheduler may choose:

```text
B → A
```

This demonstrates why NROS should separate **policy** from **mechanism**.

# 27. Scheduling policies

NROS can support several policies:

```text
FIFO
Priority
EDF
Rate Monotonic
Deadline + Priority
Criticality-aware
Resource-aware
Custom
```

The runtime should not hard-code one universal policy.

# 28. Policy as a runtime component

Conceptually:

```rust
trait SchedulingPolicy {
    fn select(
        &mut self,
        ready: &[Activation],
        now: Instant,
    ) -> Option<ActivationId>;
}
```

Then:

```text
Scheduler
   │
   └── Policy
        ├── EDF
        ├── Priority
        └── Custom
```

# 29. Deterministic tie-breaking

Even deterministic policies can encounter ties.

NROS should define a stable tie-breaker:

```text
priority
↓
deadline
↓
release_time
↓
activation_id
```

This prevents hidden nondeterminism.

# 30. Determinism has levels

NROS should distinguish:

```text
Functional determinism
Temporal determinism
Scheduling determinism
Communication determinism
Replay determinism
```

A system may be deterministic in one dimension but not another.

# 31. Functional determinism

Same input:

```text
Input X
```

produces:

```text
Output Y
```

every time.

This is the simplest form.

# 32. Scheduling determinism

Given identical:

```text
events
timing
resources
state
```

the runtime produces identical activation ordering.

This is much stronger.

# 33. Replay determinism

A recorded execution:

```text
Trace
```

can be replayed and produce equivalent:

```text
state transitions
activation ordering
effects
```

This becomes one of NROS's major debugging capabilities.

# 34. Deterministic replay architecture

```text
             LIVE SYSTEM
                  │
                  ▼
             Event Trace
                  │
          ┌───────┴───────┐
          ▼               ▼
       Analysis         Replay
                          │
                          ▼
                    NROS Runtime
```

# 35. Event sourcing

If state transitions are represented as events:

```text
S0
 ↓ Event A
S1
 ↓ Event B
S2
 ↓ Event C
S3
```

we can reconstruct state.

This complements the versioned-state architecture developed earlier.

# 36. Checkpoints

For large systems, replaying the entire history may be expensive.

Therefore:

```text
Event 1
Event 2
Event 3
Checkpoint
Event 4
Event 5
```

Recovery starts from the checkpoint.

# 37. Temporal checkpoints

A checkpoint should capture enough information to reconstruct:

```text
state
component lifecycle
pending activations
clock position
scheduler state
resource leases
```

where required by the chosen consistency model.

# 38. Simulation determinism

Simulation should be able to freeze physical time:

```text
t = 10.0
```

execute:

```text
activation A
```

then advance:

```text
t = 10.001
```

This allows precise testing.

# 39. Virtual scheduler

A simulation executor can therefore run:

```text
while events remain:
    next_event = scheduler.next()
    virtual_clock.advance(...)
    executor.execute(next_event)
```

without depending on wall-clock speed.

# 40. Faster-than-real-time simulation

If the simulation is not tied to physical time:

```text
1 second simulated
=
0.1 seconds real
```

or:

```text
1 hour simulated
=
1 minute real
```

This becomes useful for large-scale testing.

# 41. Slow-motion debugging

The inverse is equally useful:

```text
simulation speed = 0.01×
```

A developer can inspect every activation.

# 42. Temporal debugging

Imagine:

```text
Activation #881
deadline: 50ms
started: 47ms
completed: 54ms
```

NROS can answer:

```text
Why was it late?
```

Trace:

```text
queued  ── 31ms
blocked ── 12ms
executing ── 7ms
```

The problem is immediately visible.

# 43. Causal timing graph

The trace can become:

```text
Sensor
 │
 └─ 0.4ms
    ▼
Perception
 │
 └─ 4.1ms
    ▼
Planner
 │
 └─ 7.8ms
    ▼
Controller
 │
 └─ 0.6ms
    ▼
Actuator
```

NROS can therefore expose end-to-end latency rather than isolated callback timings.

# 44. Temporal contracts

Components should declare contracts such as:

```text
Input:
    period <= 10ms

Execution:
    WCET <= 2ms

Output:
    deadline <= 5ms
```

This creates machine-readable timing expectations.

# 45. Contract verification

At runtime:

```text
Declared contract
       │
       ▼
Observed execution
       │
       ▼
Contract monitor
       │
   ┌───┴────┐
   ▼        ▼
PASS      VIOLATION
```

Violations become events.

# 46. Runtime assurance

This produces a powerful architecture:

```text
Component
   │
   ├── executes
   │
   └── monitored by
          │
          ▼
     Runtime Assurance
```

The monitor can detect:

```text
deadline miss
budget overrun
resource exhaustion
unexpected lifecycle transition
invalid effect
```

# 47. Safety boundary

For a robotic system:

```text
                 AI
                  │
             planning
                  │
                  ▼
          Runtime Assurance
                  │
             policy check
                  │
                  ▼
          safety controller
                  │
                  ▼
               actuator
```

The AI does not automatically gain unrestricted control over physical effects.

# 48. NROS temporal architecture

We can now extend the stack:

```text
┌─────────────────────────────────────────┐
│ Applications / Agents / Robots          │
├─────────────────────────────────────────┤
│ Lifecycle + Supervision                 │
├─────────────────────────────────────────┤
│ Activation Model                        │
├─────────────────────────────────────────┤
│ Scheduling + Admission                  │
├─────────────────────────────────────────┤
│ Temporal Model                           │
│ clocks • deadlines • budgets • periods  │
├─────────────────────────────────────────┤
│ Execution + Effects                     │
├─────────────────────────────────────────┤
│ State + Events + Trace                  │
├─────────────────────────────────────────┤
│ Transport + Resources                   │
├─────────────────────────────────────────┤
│ OS / RTOS / Hardware                    │
└─────────────────────────────────────────┘
```

# 49. The key NROS principle

The runtime should **never imply a guarantee it has not actually established**.

Therefore:

```text
Measured ≠ Guaranteed
Declared ≠ Verified
Fast ≠ Real-Time
Low Latency ≠ Deterministic
Best Effort ≠ Deadline Safe
```

This distinction is essential for an engineering-grade robotics runtime.

# 50. From ROS to NROS

The transformation now looks like:

```text
ROS
│
├── Nodes
├── Topics
├── Services
├── Parameters
├── Executors
└── Packages
        │
        ▼
NROS
│
├── Components
├── Streams
├── Operations
├── State
├── Events
├── Activations
├── Scheduler
├── Executors
├── Resources
├── Capabilities
├── Effects
├── Lifecycle
├── Supervision
├── Temporal Contracts
├── Deterministic Replay
└── Runtime Assurance
```

This is the architectural jump from **middleware** toward a **robotics runtime**.

# 51. Next: Part XXV — Distributed NROS

The next major problem is no longer local execution.

It is:

> **How does the NROS model behave when components are distributed across machines, CPUs, robots, edge devices, and cloud systems?**

We need to define:

```text
Node identity
      ↓
Component identity
      ↓
Process identity
      ↓
Machine identity
      ↓
Robot identity
      ↓
Cluster identity
```

and then solve:

```text
Discovery
Naming
Remote activation
Distributed scheduling
Clock synchronization
Network partitions
Delivery guarantees
Backpressure
Failure detection
Leader election
Resource locality
Security across domains
```

That becomes the foundation for a **distributed NROS graph** rather than the traditional ROS computation graph.
