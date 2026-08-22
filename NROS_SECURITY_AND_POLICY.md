# NROS Security & Policy (Part LXXI–LXXX)

The previous layer established that execution is not complete until its outcome is observed and verified.

That immediately creates the next requirement:

> **What happens when the observed world diverges from what NROS expected?**

A robust autonomous runtime cannot treat every abnormal condition as simply `ERROR`.

It needs a structured failure model.

# 1. Error ≠ Fault ≠ Failure

These terms should have distinct semantics.

### Error

An erroneous condition detected by the runtime.

```text
invalid_argument
timeout
policy_denied
```

### Fault

An underlying abnormal condition.

```text
sensor_fault
network_fault
hardware_fault
```

### Failure

The inability of an operation or objective to achieve its required outcome.

```text
door.close → FAILED
```

Conceptually:

```text
Fault
  ↓
Error / abnormal condition
  ↓
Failure of operation
```

But one fault does not necessarily imply immediate mission failure.

# 2. Expected vs Observed

Every Work has an expected execution model:

```text
Expected:
    motor.position → 30°
```

Reality produces:

```text
Observed:
    motor.position → 17°
```

NROS computes:

```text
Expected ≠ Observed
```

This is a **mismatch**.

# 3. Mismatch Is a First-Class Event

Do not hide mismatches inside logs.

Represent:

```text
Mismatch {
    expected
    observed
    timestamp
    work_id
    severity
}
```

This allows policy and recovery mechanisms to react systematically.

# 4. Failure Taxonomy

A useful first taxonomy:

```text
VALIDATION_FAILURE
AUTHORIZATION_FAILURE
RESOURCE_FAILURE
TEMPORAL_FAILURE
COMMUNICATION_FAILURE
EXECUTION_FAILURE
VERIFICATION_FAILURE
STATE_FAILURE
DEPENDENCY_FAILURE
SAFETY_FAILURE
RECOVERY_FAILURE
```

Each category can have different handling rules.

# 5. Validation Failure

Example:

```text
Action:
    arm.move(target="banana")
```

Schema validation fails.

The Work never becomes executable.

```text
PROPOSED
   ↓
INVALID
```

This is not an execution failure.

# 6. Authorization Failure

Example:

```text
Agent A
   ↓
arm.move
   ↓
policy
   ↓
DENIED
```

The correct result is:

```text
NOT_ADMITTED
```

not:

```text
EXECUTION_FAILED
```

This distinction matters for auditability.

# 7. Resource Failure

Example:

```text
arm.move
```

requires:

```text
ArmController
```

but:

```text
ArmController = FAULTED
```

The Work should enter something like:

```text
BLOCKED_RESOURCE
```

rather than falsely appearing runnable.

# 8. Temporal Failure

Suppose:

```text
deadline = 12:00
```

but execution cannot begin until:

```text
12:03
```

Then:

```text
DEADLINE_MISSED
```

The runtime should not silently execute an expired action unless policy explicitly permits it.

# 9. Communication Failure

Examples:

```text
network partition
packet loss
connection timeout
peer unavailable
message duplication
message reordering
```

Distributed execution must treat these as normal failure modes, not exceptional impossibilities.

# 10. Execution Failure

The executor itself reports failure:

```text
controller rejected command
hardware driver returned fault
process crashed
```

This is a direct execution failure.

# 11. Verification Failure

The executor reports:

```text
completed
```

but observation shows:

```text
expected postcondition = false
```

Then:

```text
EXECUTED
   ↓
VERIFICATION_FAILED
```

This is distinct from executor failure.

# 12. State Failure

The runtime may discover that its state model is invalid.

Example:

```text
State:
    door = CLOSED
```

but fresh trusted observation says:

```text
door = OPEN
```

This is a state inconsistency.

The runtime must reconcile rather than blindly continue.

# 13. Dependency Failure

Work may depend on another Work:

```text
W42
 ↓ depends on
W41
```

If:

```text
W41 = FAILED
```

then W42 may become:

```text
BLOCKED_DEPENDENCY
```

unless an alternative dependency path exists.

# 14. Safety Failure

A safety condition can invalidate execution:

```text
guard_open = true
```

while:

```text
motor_move
```

requires:

```text
guard_open = false
```

This should trigger the strongest appropriate containment behavior.

# 15. Failure Severity

Not every failure deserves the same response.

A useful severity hierarchy:

```text
INFO
WARNING
DEGRADED
ERROR
CRITICAL
EMERGENCY
```

For example:

```text
temporary network delay → WARNING

camera degraded → DEGRADED

motor controller fault → CRITICAL

unsafe physical state → EMERGENCY
```

# 16. Failure Policy

A failure should be classified first:

```text
Failure
 ↓
Classification
 ↓
Policy
 ↓
Response
```

Possible responses:

```text
RETRY
WAIT
REPLAN
COMPENSATE
ROLLBACK
ESCALATE
SAFE_STOP
ABORT
ENTER_DEGRADED_MODE
```

# 17. Retry

Retry is appropriate only when the operation is safely retryable.

Example:

```text
network request
```

may be retried.

But:

```text
open_valve
```

may require idempotency or state reconciliation first.

Therefore:

> **Retry must be a policy decision, not a generic executor behavior.**

# 18. Retry Budget

Infinite retries are dangerous.

Define:

```text
max_attempts
max_duration
backoff
jitter
```

Example:

```text
attempts ≤ 3
backoff = exponential
```

After the budget is exhausted:

```text
RETRY_EXHAUSTED
```

# 19. Idempotency

Actions should declare whether repeated execution is safe.

Possible classes:

```text
IDEMPOTENT
CONDITIONALLY_IDEMPOTENT
NON_IDEMPOTENT
UNKNOWN
```

Example:

```text
set_position(30°)
```

is typically more naturally idempotent than:

```text
increment_position(+5°)
```

# 20. Unknown Idempotency

If NROS does not know whether retry is safe:

```text
UNKNOWN
```

should not automatically mean:

```text
RETRY
```

Instead:

```text
RECONCILE STATE
```

may be required.

# 21. Timeout

A timeout means:

> NROS did not receive the expected event within the required interval.

It does **not** necessarily mean:

> The operation failed.

This distinction is crucial.

Example:

```text
command sent
 ↓
network lost
 ↓
timeout
```

The remote system may still have executed the command.

# 22. Timeout State

Therefore:

```text
TIMEOUT
   ↓
OUTCOME_UNKNOWN
```

may be the correct transition.

Then:

```text
RECONCILE
```

before retrying.

# 23. Cancellation

Cancellation means:

> A valid Work was intentionally prevented from continuing.

It differs from failure.

```text
FAILED
```

means:

> execution could not achieve its outcome.

```text
CANCELLED
```

means:

> execution was intentionally terminated.

# 24. Preemption

Preemption is a specialized interruption:

```text
RUNNING
   ↓
PREEMPT_REQUESTED
   ↓
SAFE_POINT
   ↓
PAUSED
```

The Work may later resume.

Cancellation normally means it will not.

# 25. Pause

A Work may enter:

```text
PAUSED
```

without losing its semantic identity.

Possible causes:

```text
resource unavailable
operator intervention
temporary safety condition
higher-priority Work
network outage
```

# 26. Degraded Mode

A system should not always choose between:

```text
NORMAL
```

and:

```text
FAILED
```

It may continue with reduced capability.

Example:

```text
Camera A failed
```

but:

```text
Camera B available
```

System enters:

```text
DEGRADED
```

and continues.

# 27. Capability Degradation

Capabilities themselves can have degraded modes.

Example:

```text
navigation
```

normally:

```text
GPS + LiDAR + Camera
```

after failure:

```text
LiDAR only
```

The capability remains available but with reduced quality.

# 28. Resource Degradation

Likewise:

```text
CPU capacity = 100%
```

becomes:

```text
CPU capacity = 50%
```

This is not necessarily a failure of the entire runtime.

Resource-aware planning can adapt.

# 29. Fault Containment

A fault should not automatically spread.

Example:

```text
Camera driver fault
```

should not necessarily cause:

```text
Scheduler failure
```

The architecture should isolate failures:

```text
Fault
 ↓
Contain
 ↓
Assess impact
 ↓
Propagate only necessary consequences
```

# 30. Failure Domains

NROS can define failure domains:

```text
Process
Node
Device
Subsystem
Robot
Network Segment
Cluster
Mission
```

A failure should be scoped to the smallest appropriate domain.

# 31. Dependency Graph

Suppose:

```text
Navigation
   ↓
Camera
   ↓
Camera Driver
```

A driver failure propagates upward:

```text
Driver failed
   ↓
Camera unavailable
   ↓
Navigation degraded
```

But unrelated:

```text
Audio
```

should remain unaffected.

# 32. Failure Propagation

This suggests explicit dependency semantics:

```text
Failure
 ↓
Affected dependencies
 ↓
Recompute feasibility
 ↓
Only affected Work changes state
```

This prevents unnecessary global shutdown.

# 33. Recovery

Recovery is the process of returning from an abnormal state.

Example:

```text
FAULTED
   ↓
DIAGNOSE
   ↓
RECOVER
   ↓
VERIFY
   ↓
AVAILABLE
```

Recovery itself must be observable.

# 34. Recovery Is Not Assumed

A reset command returning:

```text
OK
```

does not prove recovery.

Instead:

```text
reset
 ↓
observation
 ↓
verification
 ↓
resource health restored
```

Only then:

```text
RECOVERED
```

# 35. Recovery Strategies

Possible strategies:

```text
RETRY
RESET
RESTART
FAILOVER
RECONFIGURE
REPLAN
ROLLBACK
COMPENSATE
ESCALATE
```

Selection should depend on fault class.

# 36. Failover

If a resource has alternatives:

```text
Primary Camera
       ↓
FAILED
       ↓
Backup Camera
```

the Resource Manager can allocate the backup.

This connects resilience directly to the Resource Pool model.

# 37. Compensation

Not every action can be rolled back.

Example:

```text
transfer_money
```

cannot necessarily be undone by restoring memory.

Instead, NROS may perform a compensating action:

```text
transfer
   ↓
compensating transfer
```

Thus:

```text
rollback ≠ compensation
```

# 38. Rollback

Rollback attempts to restore a previous state.

Appropriate for:

```text
transactional configuration
database update
staged software deployment
```

Not always appropriate for physical actions.

# 39. Physical Irreversibility

A physical action may be irreversible:

```text
release object
break component
move robot
consume fuel
```

Therefore planning must account for:

```text
reversibility
```

before execution.

# 40. Compensation Graph

A Work can declare:

```text
Action:
    lock_door

Compensation:
    unlock_door
```

But compensation should itself be:

```text
authorized
resource-admitted
safe
verified
```

It is not an escape hatch around normal policy.

# 41. Recovery Policy

For each failure class, policy can specify:

```text
failure:
    network_timeout

strategy:
    retry

max_attempts:
    3

after_exhaustion:
    reconcile
```

This makes recovery deterministic and auditable.

# 42. Recovery State Machine

A robust Work lifecycle can now include:

```text
RUNNING
   ↓
FAULT
   ↓
CLASSIFY
   ↓
┌──────────┬───────────┬────────────┐
↓          ↓           ↓
RETRY    RECOVER     REPLAN
↓          ↓           ↓
RUNNING  VERIFY      NEW WORK
             │
             ↓
          RESUME
```

Failure:

```text
       ↓
    ABORT
```

# 43. Safe Stop

For physical systems:

```text
FAULT
 ↓
SAFE_STOP
```

may be required before any diagnostic or recovery operation.

Safe stop itself should have explicit semantics.

# 44. Emergency State

NROS should support a system-wide emergency state:

```text
NORMAL
   ↓
EMERGENCY
```

In emergency mode:

```text
normal work
   ↓
suspended / cancelled
```

while:

```text
safety actions
```

remain admissible.

# 45. Emergency Must Be Scoped

Not every fault should trigger global emergency mode.

Example:

```text
audio subsystem failure
```

does not necessarily require:

```text
robot emergency stop
```

unless policy says the dependency is safety-critical.

# 46. Safe Mode

Safe mode is different from emergency mode.

```text
DEGRADED
   ↓
SAFE_MODE
```

The system may remain operational but restrict capabilities.

Example:

```text
navigation allowed
high-speed motion denied
```

# 47. Recovery Verification

After recovery:

```text
Recovery Action
 ↓
Observation
 ↓
Health Check
 ↓
Capability Check
 ↓
Resource Check
 ↓
Policy Revalidation
```

Only then:

```text
RECOVERED
```

# 48. Authority After Recovery

Authority may have changed during a failure.

Therefore recovery must not blindly reuse an old authorization context.

```text
Recovery
 ↓
Revalidate Authority
```

This prevents stale permissions from surviving incidents.

# 49. Resource Reconciliation

Resources may also have changed.

Example:

```text
Checkpoint:
    Arm = allocated
```

after restart:

```text
actual:
    Arm = occupied
```

The Resource Manager must reconcile reality before reallocation.

# 50. State Reconciliation

Similarly:

```text
Checkpoint:
    door = CLOSED
```

but fresh observation:

```text
door = OPEN
```

Then:

```text
checkpoint state
```

must lose authority to the newer verified evidence.

# 51. Recovery Ordering

A safe recovery sequence is:

```text
1. Restore runtime
2. Detect external state
3. Reconcile resources
4. Reconcile authority
5. Reconcile active Work
6. Resolve unknown outcomes
7. Verify safety conditions
8. Resume or replan
```

Not:

```text
restore checkpoint
→ immediately continue execution
```

# 52. Recovery Checkpoint

Checkpoint metadata should distinguish:

```text
last_known_state
last_verified_state
pending_actions
unknown_actions
resource_leases
authority_leases
```

This is much safer than storing only application variables.

# 53. Failure Evidence

Failures themselves produce evidence.

Example:

```text
FailureEvidence {
    work_id
    fault
    observations
    policy_decision
    recovery_attempts
    final_outcome
}
```

The system can therefore learn from failures without rewriting history.

# 54. Failure Provenance

A final incident should be traceable:

```text
Goal
 ↓
Plan
 ↓
Action
 ↓
Resource
 ↓
Fault
 ↓
Observation
 ↓
Failure Classification
 ↓
Recovery
 ↓
Verification
 ↓
Outcome
```

This gives NROS a complete incident chain.

# 55. Incident

Multiple related failures can form an Incident:

```text
Incident
├── Fault
├── Affected Work
├── Resource impact
├── Policy response
├── Recovery actions
└── Final resolution
```

An Incident is a higher-level operational object.

# 56. Incident Correlation

Example:

```text
Network outage
 ├── Controller timeout
 ├── Sensor timeout
 ├── Work A unknown
 └── Work B paused
```

These should not necessarily appear as four unrelated failures.

They can correlate to:

```text
Incident: network_partition_42
```

# 57. Root Cause vs Symptom

NROS should distinguish:

```text
Root Cause:
    network interface failure

Symptoms:
    controller timeout
    sensor timeout
    heartbeat loss
```

Otherwise recovery may address symptoms without fixing the underlying fault.

# 58. Fault Diagnosis

Diagnosis can use:

```text
observations
dependencies
health data
historical evidence
failure patterns
```

to produce:

```text
hypothesis
```

But diagnosis itself may be uncertain.

Therefore:

```text
diagnosis ≠ proven root cause
```

unless verified.

# 59. Recovery Confidence

A recovery assessment can say:

```text
RECOVERED_CONFIRMED
RECOVERED_PROBABLE
RECOVERY_UNCERTAIN
RECOVERY_FAILED
```

This avoids binary assumptions.

# 60. Resilience

Resilience is broader than recovery.

A resilient system can:

```text
detect
contain
adapt
continue
recover
learn
```

while preserving safety and provenance.

# 61. Resilience Dimensions

NROS resilience can be viewed across:

```text
Detection
Containment
Redundancy
Degradation
Recovery
Reconciliation
Verification
Adaptation
```

# 62. Deterministic Failure Handling

A major architectural principle:

> **The same failure context should produce a reproducible policy decision.**

That means recovery should be based on explicit:

```text
failure class
policy version
state
resource state
authority
```

rather than hidden executor behavior.

# 63. Failure State Machine

A generalized system state model:

```text
                    ┌──────────────┐
                    │    NORMAL    │
                    └──────┬───────┘
                           ↓
                       ABNORMAL
                           ↓
                     CLASSIFY FAULT
                           │
             ┌─────────────┼─────────────┐
             ↓             ↓             ↓
          RECOVER        DEGRADE       ABORT
             ↓             ↓             ↓
          VERIFY        REPLAN        SAFE_STOP
             ↓             ↓
          RESUME        RESUME
             │             │
             └──────┬──────┘
                    ↓
                 NORMAL
```

# 64. Failure-Aware Work Lifecycle

The complete lifecycle is now closer to:

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
 ├── VERIFIED → COMPLETED
 │
 ├── FAILED
 │
 ├── UNKNOWN → RECONCILE
 │
 └── PARTIAL → REPLAN
```

And from execution:

```text
RUNNING
 ↓
FAULT
 ↓
RECOVERY
 ↓
VERIFY
 ↓
RESUME / FAIL
```

# 65. Critical Invariants

The failure subsystem should enforce:

```text
1. Timeout does not automatically imply failure.

2. Unknown outcome does not imply success or failure.

3. Retry requires explicit safety/idempotency semantics.

4. Infinite retry is prohibited.

5. Cancellation is distinct from failure.

6. Preemption is distinct from cancellation.

7. Recovery requires verification.

8. Recovery cannot bypass authorization.

9. Recovery cannot assume resources remain allocated.

10. Checkpoint state must be reconciled with reality.

11. Fault propagation must follow dependency boundaries.

12. Safety failures take precedence over ordinary recovery.

13. Compensation is not equivalent to rollback.

14. Root cause and symptom must remain distinguishable.

15. Every consequential failure should produce auditable evidence.
```

# 66. NROS Resilience Architecture

The runtime now becomes:

```text
                         NROS
                           │
              ┌────────────┼────────────┐
              ↓            ↓            ↓
           POLICY        STATE       RESOURCES
              │            │            │
              └────────────┼────────────┘
                           ↓
                        ACTION
                           ↓
                       ADMISSION
                           ↓
                          WORK
                           ↓
                       EXECUTION
                           ↓
                    ┌──────┴──────┐
                    ↓             ↓
                 SUCCESS        FAULT
                    │             │
                    ↓             ↓
                VERIFY         CLASSIFY
                    │             │
                    ↓       ┌─────┼─────┐
                COMPLETE     ↓     ↓     ↓
                          RETRY  RECOVER ABORT
                              │     │
                              └──┬──┘
                                 ↓
                              VERIFY
                                 ↓
                         RESUME / REPLAN
```

# 67. The NROS Safety Loop

The full autonomous loop is now:

```text
OBSERVE
   ↓
ASSESS STATE
   ↓
PLAN
   ↓
AUTHORIZE
   ↓
ADMIT
   ↓
RESERVE
   ↓
EXECUTE
   ↓
OBSERVE EFFECT
   ↓
VERIFY
   ↓
REFLECT
   ↓
CHECKPOINT
   ↓
RECOVER / REPLAN IF NECESSARY
   ↓
OBSERVE AGAIN
```

This is much more robust than a simple:

```text
Agent → Tool → Result
```

architecture.

# 68. Next Major Boundary — Communication

We have now modeled:

```text
Identity
Authority
Policy
Capabilities
Actions
Resources
Time
State
Observation
Evidence
Verification
Failure
Recovery
```

One major runtime substrate remains:

> **How do all these entities communicate reliably across process, machine, device, and network boundaries?**

That takes us to:

# Part LXXII — NROS Communication, Messaging & Distributed Execution Model

The next layer will define:

```text
Message
Envelope
Endpoint
Channel
Transport
Request
Response
Event
Stream
Subscription
Command
Delivery
Acknowledgement
Correlation
Ordering
Causality
Retry
Deduplication
Idempotency
Backpressure
Flow Control
Timeout
Partition
Discovery
Session
Heartbeat
Lease
Remote Work
Distributed Transaction
```

The key transition will be:

```text
LOCAL EXECUTION
      ↓
INTER-PROCESS EXECUTION
      ↓
DISTRIBUTED EXECUTION
      ↓
REMOTE AGENT EXECUTION
```

while preserving the same NROS semantic guarantees:

```text
Authority
+
State
+
Resources
+
Time
+
Evidence
+
Failure Semantics
```

across the communication boundary.

# NROS — Part LXXII: Communication, Messaging & Distributed Execution Model

The previous layer established failure and recovery semantics.

Now we cross the most important architectural boundary:

> **NROS must preserve its execution semantics even when the executor, resource, agent, or state lives somewhere else.**

A local function call is easy:

```text
Agent → Executor → Result
```

A distributed runtime is fundamentally different:

```text
Agent
  ↓
Message
  ↓
Transport
  ↓
Remote Node
  ↓
Executor
  ↓
Physical/Logical Effect
  ↓
Observation
  ↓
Verification
  ↓
Message
  ↓
Agent
```

The communication layer must therefore be more than a serialization utility.

# 1. Communication Is a Semantic Boundary

NROS should not model:

```text
send(bytes)
```

as its primary abstraction.

Instead:

```text
Intent
 ↓
Message
 ↓
Delivery
 ↓
Execution
 ↓
Evidence
```

The communication system transports **semantic events and commands**.

# 2. Message

A Message represents one semantic unit transferred between NROS participants.

Conceptually:

```rust
struct Message {
    id: MessageId,
    sender: EndpointId,
    recipient: EndpointId,
    kind: MessageKind,
    correlation: CorrelationId,
    timestamp: Time,
    payload: Payload,
}
```

The exact API can evolve later.

# 3. Message Identity

Every message needs a stable identifier:

```text
MessageId
```

Why?

Because distributed networks produce:

```text
duplicate delivery
retry
replay
reordering
```

Without identity, deduplication becomes unreliable.

# 4. Envelope vs Payload

Separate:

```text
Envelope
```

from:

```text
Payload
```

Example:

```text
Envelope
├── message_id
├── sender
├── recipient
├── correlation_id
├── timestamp
├── protocol_version
├── delivery_policy
└── security metadata

Payload
└── semantic content
```

This prevents transport metadata from contaminating application semantics.

# 5. Message Types

NROS should distinguish at least:

```text
COMMAND
REQUEST
RESPONSE
EVENT
OBSERVATION
ACKNOWLEDGEMENT
REJECTION
ERROR
HEARTBEAT
CANCEL
CONTROL
```

Different message classes have different delivery semantics.

# 6. Command

A Command asks a remote executor to perform an action.

```text
Command:
    arm.move
    target = 30°
```

But receiving the Command does not mean:

```text
action executed
```

It means only:

```text
command received
```

# 7. Request

A Request asks another component for information or computation.

Example:

```text
Request:
    get_sensor_state
```

The response:

```text
Response:
    temperature = 72°C
```

must preserve correlation.

# 8. Event

An Event communicates something that happened.

Example:

```text
Event:
    motor.position.changed
```

Events normally do not require a direct response.

# 9. Observation Message

An Observation is a specialized event carrying evidence:

```text
Observation:
    door = CLOSED
```

It should retain:

```text
source
observed_at
provenance
quality
```

from the previous layer.

# 10. Acknowledgement

An ACK means:

> The message was accepted at a particular protocol boundary.

It does **not** necessarily mean:

> The requested operation succeeded.

For example:

```text
COMMAND
 ↓
ACK
 ↓
EXECUTE
 ↓
VERIFY
```

# 11. Delivery Semantics

NROS should explicitly model delivery guarantees.

Common levels:

```text
AT_MOST_ONCE
AT_LEAST_ONCE
EFFECTIVELY_ONCE
EXACTLY_ONCE
```

But the last one deserves caution.

# 12. Exactly-Once Is Usually a Semantic Property

A transport cannot magically guarantee exactly-once physical execution.

For example:

```text
command sent
 ↓
remote executes
 ↓
response lost
```

The sender cannot know whether execution happened.

Retrying can execute twice.

Therefore:

> **Exactly-once effect usually requires application-level identity, deduplication, idempotency, or transactional coordination.**

# 13. At-Most-Once

```text
send once
```

Possible outcomes:

```text
delivered once
or
lost
```

Good for some telemetry where missing data is acceptable.

# 14. At-Least-Once

```text
retry until acknowledged
```

Possible outcomes:

```text
delivered once
or
delivered multiple times
```

Requires deduplication for non-idempotent operations.

# 15. Effectively-Once

NROS can achieve:

```text
multiple message deliveries
        ↓
one semantic effect
```

through:

```text
WorkId
+
AttemptId
+
deduplication
+
state verification
```

This is more realistic than relying on transport-level exactly-once guarantees.

# 16. Correlation

Distributed workflows require correlation:

```text
Request
 correlation_id = C42
```

then:

```text
Response
 correlation_id = C42
```

For Work execution:

```text
WorkId
ActionId
AttemptId
CorrelationId
```

should remain connected.

# 17. Causal Chain

Consider:

```text
Goal
 ↓
Plan
 ↓
Command
 ↓
Remote execution
 ↓
Observation
 ↓
Verification
```

Each message should preserve enough metadata to reconstruct this chain.

# 18. Causality

Simple timestamps are insufficient.

Two events can have:

```text
event A @ 10:00:01
event B @ 10:00:01
```

without knowing which caused which.

NROS may therefore use:

```text
parent_event_id
sequence
logical clock
causal context
```

where required.

# 19. Ordering

Messages may arrive:

```text
1 → 3 → 2
```

even though they were emitted:

```text
1 → 2 → 3
```

Therefore ordering must be explicit.

Possible semantics:

```text
UNORDERED
PER-SENDER
PER-STREAM
PER-KEY
GLOBAL
```

Global ordering should not be assumed without need.

# 20. Sequence Numbers

A stream may use:

```text
sequence = 101
sequence = 102
sequence = 103
```

A receiver can detect:

```text
missing 102
```

or:

```text
duplicate 102
```

# 21. Replay

An event stream may be replayed.

This is useful for:

```text
debugging
recovery
simulation
audit
state reconstruction
testing
```

But replayed events must not accidentally cause real-world actions.

# 22. Replay Safety

NROS should distinguish:

```text
REPLAY
```

from:

```text
LIVE
```

A replay environment must prevent:

```text
historical command
      ↓
real actuator
```

unless explicitly authorized.

# 23. Transport

Transport is responsible for moving envelopes.

Examples conceptually:

```text
Unix socket
TCP
QUIC
WebSocket
serial
shared memory
in-process channel
```

The semantic layer should not be tightly coupled to one transport.

# 24. Transport Abstraction

Conceptually:

```rust
trait Transport {
    send(...);
    receive(...);
    close(...);
}
```

But NROS's higher layer should operate on:

```text
Message
```

rather than raw bytes.

# 25. Channel

A Channel is a logical communication path.

```text
Channel:
    controller.commands
```

It may map onto:

```text
TCP
QUIC
Unix socket
```

without changing semantic meaning.

# 26. Endpoint

An Endpoint represents a communication participant.

Example:

```text
agent.planner
controller.motion
sensor.position
runtime.scheduler
```

Endpoints should have stable identity.

# 27. Node

A Node can host multiple endpoints:

```text
Node
├── planner
├── scheduler
├── executor
└── telemetry
```

A distributed NROS deployment may therefore look like:

```text
Node A
  ├── Agent
  └── Planner

Node B
  ├── Scheduler
  └── Executor

Node C
  └── Sensors
```

# 28. Session

A Session represents an active relationship between endpoints.

```text
CONNECT
 ↓
AUTHENTICATE
 ↓
NEGOTIATE
 ↓
ESTABLISH SESSION
 ↓
MESSAGES
 ↓
CLOSE
```

Sessions may maintain:

```text
capabilities
protocol version
heartbeat
leases
flow-control state
```

# 29. Discovery

A distributed runtime needs to discover available participants.

Conceptually:

```text
Discover
 ↓
Endpoint
 ↓
Capabilities
 ↓
Authority
 ↓
Health
```

Discovery must not imply permission.

Finding:

```text
motor.controller
```

does not mean the caller may command it.

# 30. Capability Advertisement

A remote endpoint can advertise:

```text
Capabilities:
    move
    stop
    position
```

The caller then asks:

```text
Can I invoke "move"?
```

The answer depends on both:

```text
capability
+
authority
```

# 31. Remote Admission

A particularly important rule:

> **Remote execution must pass the same admission semantics as local execution.**

Not:

```text
local:
    policy enforced

remote:
    send command directly
```

Instead:

```text
Intent
 ↓
Local policy
 ↓
Remote admission
 ↓
Remote resource check
 ↓
Execution
```

# 32. Distributed Authority

Authority may exist at multiple levels:

```text
Local Agent Authority
        ↓
Network Authority
        ↓
Remote Node Authority
        ↓
Device Authority
```

The effective permission is constrained by all applicable boundaries.

# 33. Lease

Distributed authority and resources often require leases.

Example:

```text
Resource Lease
    owner = Work42
    expires = T
```

If the lease expires:

```text
ownership
```

must not silently continue.

# 34. Lease Renewal

A long-running Work may renew:

```text
LEASE
 ↓
RENEW
 ↓
RENEW
 ↓
...
```

But renewal must itself be authorized.

# 35. Lease Loss

If renewal fails:

```text
LEASE_LOST
```

the Work may need:

```text
pause
safe stop
release resources
reconcile
```

rather than continuing indefinitely.

# 36. Heartbeat

A heartbeat provides liveness information:

```text
PING
 ↓
PONG
```

But:

> **Heartbeat proves communication, not application health.**

A node can be alive while its motor controller is broken.

# 37. Health

Health should therefore be layered:

```text
Transport healthy
       ↓
Node alive
       ↓
Service responsive
       ↓
Capability healthy
       ↓
Resource available
```

These are different signals.

# 38. Backpressure

A producer may generate events faster than a consumer can process them.

```text
Producer
   ↓↓↓↓↓↓↓
Consumer
   ↓
cannot keep up
```

Without backpressure:

```text
memory growth
latency explosion
event loss
```

NROS therefore needs explicit flow-control semantics.

# 39. Flow Control

Possible strategies:

```text
BLOCK
BUFFER
DROP
SAMPLE
COALESCE
REJECT
SPILL
```

The correct choice depends on message semantics.

# 40. Never Drop Safety Commands Silently

For:

```text
emergency_stop
```

dropping messages due to ordinary telemetry backpressure would be unacceptable.

Safety-critical traffic requires separate priority and delivery semantics.

# 41. Priority

Messages can carry priority:

```text
EMERGENCY
CRITICAL
HIGH
NORMAL
LOW
TELEMETRY
```

But priority must remain policy-controlled.

An arbitrary agent should not simply declare:

```text
priority = EMERGENCY
```

and bypass admission.

# 42. Queues

Logical queues can isolate traffic:

```text
control
commands
observations
telemetry
audit
diagnostics
```

This prevents high-volume telemetry from starving control traffic.

# 43. Message Expiration

Messages may have deadlines:

```text
expires_at
```

A stale command should not necessarily execute.

Example:

```text
move_to_position
deadline = 10:00
```

received at:

```text
10:05
```

should normally be rejected or revalidated.

# 44. Temporal Revalidation

Remote execution should revalidate:

```text
authority
state
resource
deadline
safety
```

at the point of admission.

A message being valid when created does not guarantee it remains valid when received.

# 45. Serialization

Payload serialization must preserve semantic types.

Avoid reducing everything to arbitrary strings.

Prefer explicit schemas:

```text
Command {
    action: ActionId,
    parameters: TypedValue
}
```

This enables validation before execution.

# 46. Schema Versioning

Distributed nodes may run different versions.

Messages therefore need:

```text
protocol_version
schema_version
```

Compatibility must be explicit.

# 47. Version Negotiation

Connection:

```text
Node A: protocol 2
Node B: protocol 3
```

can negotiate:

```text
compatible version = 2
```

or reject the connection.

Silent semantic mismatch is dangerous.

# 48. Compatibility

Compatibility can be:

```text
EXACT
BACKWARD_COMPATIBLE
FORWARD_COMPATIBLE
TRANSLATED
INCOMPATIBLE
```

A translation layer may be used where safe.

# 49. Security Boundary

Communication crosses trust boundaries.

Therefore messages may require:

```text
authentication
authorization
integrity
confidentiality
replay protection
```

The transport security mechanism should not replace NROS authority semantics.

# 50. Message Authentication

A receiver should be able to determine:

> Who sent this?

and:

> Was the message altered?

These are separate questions from:

> Is the sender authorized to perform the requested operation?

# 51. Replay Protection

An attacker or faulty network may replay:

```text
old command
```

NROS can mitigate using:

```text
MessageId
nonce
timestamp
sequence
expiration
```

combined with policy.

# 52. Distributed Verification

The remote side should produce evidence:

```text
Command
 ↓
Remote Work
 ↓
Execution
 ↓
Observation
 ↓
Verification
```

Then return:

```text
VerificationEvidence
```

rather than merely:

```text
"OK"
```

# 53. Remote Execution Receipt

A remote node can return:

```text
ExecutionReceipt {
    work_id
    remote_node
    attempt_id
    execution_state
    timestamps
}
```

The origin node can correlate this with its local Work.

# 54. Distributed Outcome

The origin should distinguish:

```text
REMOTE_ACCEPTED
REMOTE_RUNNING
REMOTE_EXECUTED
REMOTE_VERIFIED
REMOTE_FAILED
REMOTE_UNKNOWN
```

This preserves the semantics established in Part LXXI.

# 55. Network Partition

The hardest case:

```text
Node A  X  Node B
```

Neither knows whether the other is alive.

NROS must not infer:

```text
remote failure
```

from:

```text
communication loss
```

without qualification.

# 56. Partition Policy

During partition:

```text
safe actions
```

may continue locally.

But:

```text
actions requiring remote authority/state
```

may be blocked.

This is where distributed policy meets failure semantics.

# 57. Split-Brain Prevention

If two nodes believe they own the same resource:

```text
Node A → Resource X
Node B → Resource X
```

dangerous concurrent control can result.

Leases, fencing, or an authority mechanism may be required.

# 58. Fencing

Fencing means preventing an old/stale owner from continuing to control a resource.

Conceptually:

```text
Lease A expires
 ↓
Fence A
 ↓
Grant resource to B
```

Without fencing, lease expiration alone may be insufficient.

# 59. Distributed Resource Ownership

A remote resource therefore needs:

```text
ResourceId
Owner
Lease
Epoch
Authority
Health
```

The epoch helps distinguish stale ownership information.

# 60. Epoch

Example:

```text
Resource X
epoch = 41
```

After ownership changes:

```text
epoch = 42
```

Messages carrying epoch 41 can then be rejected as stale.

# 61. Distributed Transactions

Some workflows require several resources:

```text
Camera
+
Arm
+
Navigation
```

A distributed transaction might be tempting.

But NROS should avoid assuming traditional database transactions map cleanly to physical systems.

Instead, use semantic patterns:

```text
reserve
prepare
execute
verify
compensate
```

where appropriate.

# 62. Two-Phase Semantics

For resources that support preparation:

```text
PREPARE
   ↓
COMMIT
```

But physical execution may not be reversible.

Therefore:

> **Transactional semantics should be capability-specific, not universal.**

# 63. Distributed Work

A Work may span nodes:

```text
Work W42
├── Node A: planning
├── Node B: execution
└── Node C: verification
```

The Work identity must remain stable across all participants.

# 64. Distributed Work Graph

```text
             Work W42
                │
       ┌────────┼────────┐
       ↓        ↓        ↓
   Planner   Executor  Verifier
   Node A     Node B    Node C
```

Each produces evidence tied to W42.

# 65. Remote Agent

An agent can itself be remote:

```text
Agent A
   ↓ network
NROS Node B
   ↓
Resource
```

The architecture should not require the agent and executor to share a process.

# 66. Local and Remote Uniformity

A strong design goal:

```text
Local Work
```

and:

```text
Remote Work
```

should share the same semantic lifecycle.

Only the transport boundary differs.

This greatly simplifies reasoning.

# 67. Communication as a Runtime Service

The NROS runtime can expose:

```text
CommunicationService
├── send
├── request
├── subscribe
├── publish
├── acknowledge
├── cancel
├── discover
├── heartbeat
└── session
```

while lower layers provide transport implementations.

# 68. Event Bus

A logical event bus may provide:

```text
publish(event)
subscribe(filter)
```

Example:

```text
subscribe:
    observation.motor.*
```

The event bus should preserve:

```text
identity
ordering semantics
provenance
delivery policy
```

where required.

# 69. Request/Response vs Event

Do not force everything into request/response.

Use:

```text
Request/Response
```

for direct interaction.

Use:

```text
Events
```

for asynchronous state changes.

Use:

```text
Streams
```

for continuous observations.

# 70. Streaming

Sensors may produce:

```text
position
position
position
position
...
```

A stream should support:

```text
subscription
sequence
backpressure
sampling
cancellation
checkpoint/resume
```

# 71. Stream Resume

After temporary disconnect:

```text
last_received = 1042
```

the consumer can request:

```text
resume from 1043
```

if the producer retains the stream history.

# 72. Telemetry vs State

High-rate telemetry should not necessarily become durable state.

For example:

```text
10,000 temperature readings
```

may be summarized into:

```text
current temperature
trend
statistics
```

while raw evidence is retained according to policy.

# 73. Message Retention

Different messages may have different retention:

```text
Safety Event → durable
Audit Event → durable
Telemetry → bounded
Heartbeat → ephemeral
```

Retention policy is part of the communication semantics.

# 74. Distributed Audit Trail

For consequential remote execution:

```text
Origin
 ↓
Message
 ↓
Remote Node
 ↓
Execution
 ↓
Observation
 ↓
Verification
 ↓
Result
```

should remain auditable.

This connects Communication directly to the Evidence architecture.

# 75. Distributed Closed Loop

The complete system now becomes:

```text
        AGENT
          │
          ↓
        PLAN
          │
          ↓
      AUTHORIZE
          │
          ↓
       ADMIT
          │
          ↓
       MESSAGE
          │
     ┌────┴────┐
     ↓         ↓
  NETWORK    LOCAL
     │
     ↓
 REMOTE NODE
     │
     ↓
  EXECUTE
     │
     ↓
  OBSERVE
     │
     ↓
  VERIFY
     │
     ↓
  EVIDENCE
     │
     ↓
   MESSAGE
     │
     ↓
    AGENT
```

# 76. Distributed Failure Loop

If communication fails:

```text
MESSAGE
   ↓
TIMEOUT
   ↓
OUTCOME UNKNOWN
   ↓
RECONCILE
   ↓
OBSERVE REMOTE STATE
   ↓
VERIFY
   ↓
RESUME / RETRY / REPLAN
```

This is the correct continuation of the failure model.

# 77. Core Communication Invariants

NROS should enforce:

```text
1. Message receipt ≠ command execution.

2. ACK ≠ success.

3. Timeout ≠ failure.

4. Communication loss ≠ remote state loss.

5. Duplicate delivery must be detectable.

6. Retry must preserve Work identity.

7. Remote execution must obey admission policy.

8. Remote authority must be independently validated.

9. Message expiration must be enforced.

10. Ordering must never be assumed unless specified.

11. Replay must be explicitly controlled.

12. Network health ≠ application health.

13. Resource leases require expiration semantics.

14. Lease loss must trigger defined behavior.

15. Remote execution must produce traceable evidence.

16. Local and remote Work should share semantic lifecycle.

17. Safety-critical traffic requires explicit delivery guarantees.

18. Partition behavior must be policy-defined.

19. Stale ownership must be fenceable where required.

20. Distributed completion requires verification, not merely response receipt.
```

# 78. NROS Communication Architecture

The accumulated architecture now looks like:

```text
                         NROS
                           │
       ┌───────────────────┼───────────────────┐
       ↓                   ↓                   ↓
     AGENTS              POLICY              STATE
       │                   │                   │
       └───────────────────┼───────────────────┘
                           ↓
                       CAPABILITY
                           ↓
                         ACTION
                           ↓
                       AUTHORITY
                           ↓
                        ADMISSION
                           ↓
                          WORK
                           ↓
                    ┌──────┴──────┐
                    ↓             ↓
                 LOCAL          REMOTE
                    │             │
                    │        COMMUNICATION
                    │             │
                    └──────┬──────┘
                           ↓
                       EXECUTION
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
                           ↓
                    RECOVERY / REPLAN
```

# 79. Architectural Consequence

At this point NROS is no longer simply:

```text
Rust async runtime
```

in the conventional sense.

It is becoming a:

> **stateful, evidence-producing, policy-governed execution runtime whose scheduling units can represent autonomous Work and whose semantics remain valid across local and distributed boundaries.**

The next major problem is therefore not another ordinary subsystem.

It is the **execution model itself**.

# Part LXXIII — NROS Scheduler, Work Graph & Agent-Native Scheduling

We should next formalize:

```text
Work
WorkId
Task
Subtask
Dependency
Work Graph
Agent
Scheduler
Queue
Priority
Deadline
Preemption
Fairness
Resource Affinity
Capability Matching
Admission
Concurrency
Parallelism
Critical Path
Budget
Quota
Backoff
Starvation
Deadlock
Livelock
Scheduling Epoch
Checkpoint
Resume
```

The central architectural question will be:

> **If autonomous Agents—not Futures—are the primary scheduling units, what exactly does an NROS scheduler schedule?**

The resulting model should move from:

```text
Future → Executor → Wake
```

toward:

```text
Agent
  ↓
Work Graph
  ↓
Runnable Work
  ↓
Admission
  ↓
Resource/Capability Matching
  ↓
Scheduling
  ↓
Execution
  ↓
Evidence
  ↓
State Transition
  ↓
New Runnable Work
```

That is the point where the NROS concept becomes a genuinely **agent-native runtime scheduling architecture**.

# NROS — Part LXXIII: Scheduler, Work Graph & Agent-Native Scheduling

The previous layer established communication across local and distributed boundaries.

Now we can define the central execution mechanism:

> **What does NROS actually schedule?**

A conventional async runtime primarily schedules:

```text
Future
  ↓
Poll
  ↓
Wake
  ↓
Poll again
```

NROS needs a higher-level model:

```text
Agent
  ↓
Goal
  ↓
Work Graph
  ↓
Runnable Work
  ↓
Admission
  ↓
Scheduling
  ↓
Execution
  ↓
Observation
  ↓
Verification
  ↓
State transition
  ↓
New Work
```

The scheduler therefore becomes a **semantic scheduler**, not merely a CPU task dispatcher.

# 1. Scheduling Unit

The primary scheduling unit should be:

```text
Work
```

not:

```text
Future
```

A Work represents an intentional unit of execution with:

```text
WorkId
Agent
Intent
Action
Dependencies
Resources
Priority
Deadline
Policy
State
Evidence
RecoveryPolicy
```

# 2. Agent vs Work

An Agent is persistent.

A Work is bounded.

```text
Agent
 ├── Work A
 ├── Work B
 ├── Work C
 └── Work D
```

The Agent owns or initiates Work, while the scheduler decides which Work can progress.

# 3. Goal

A Goal describes the desired condition.

Example:

```text
Goal:
    elevator_at_floor = 7
```

The scheduler should not necessarily execute a direct command immediately.

It may require:

```text
Goal
 ↓
Planning
 ↓
Work Graph
```

# 4. Work Graph

A Work Graph represents dependencies:

```text
W1
 ↓
W2
 ↓
W3
```

or parallel work:

```text
        W2
       ↗
W1 ───
       ↘
        W3
```

The scheduler operates on the graph's currently runnable frontier.

# 5. Runnable Frontier

Given:

```text
W1 → W2 → W3
```

only:

```text
W1
```

is initially runnable.

After W1 completes:

```text
W2
```

becomes runnable.

This is more meaningful than maintaining one giant global queue.

# 6. Dependency Types

Dependencies should be typed.

Examples:

```text
DATA_DEPENDENCY
STATE_DEPENDENCY
RESOURCE_DEPENDENCY
TEMPORAL_DEPENDENCY
AUTHORITY_DEPENDENCY
SAFETY_DEPENDENCY
CAUSAL_DEPENDENCY
```

A Work may be blocked for different reasons.

# 7. Dependency Resolution

A Work becomes runnable only when required dependencies are satisfied.

Conceptually:

```text
Runnable(W)
=
DependenciesSatisfied
∧
AuthorityValid
∧
ResourcesAvailable
∧
PolicyAllows
∧
TemporalConstraintsValid
∧
SafetyConditionsValid
```

This becomes a fundamental scheduler predicate.

# 8. Runnable ≠ Admitted

This distinction is important.

A Work can be:

```text
RUNNABLE
```

but not yet:

```text
ADMITTED
```

because:

```text
resource unavailable
policy denies
quota exhausted
deadline conflict
safety condition false
```

Therefore:

```text
RUNNABLE
   ↓
ADMISSION
   ↓
ADMITTED
   ↓
SCHEDULED
```

# 9. Runnable Queue

The scheduler maintains candidate Work:

```text
RunnableQueue
├── W42
├── W51
├── W62
└── W77
```

But ordering must not be based on one universal priority number.

# 10. Scheduling Dimensions

A Work may be ranked using:

```text
priority
deadline
criticality
age
resource locality
dependency criticality
agent quota
fairness
estimated cost
risk
```

The scheduler should combine these according to explicit policy.

# 11. Priority

Basic priority:

```text
CRITICAL
HIGH
NORMAL
LOW
```

But priority alone causes starvation.

If high-priority work continuously arrives:

```text
LOW
```

could theoretically wait forever.

# 12. Aging

A scheduler can increase effective priority as Work waits.

Conceptually:

```text
effective_priority =
base_priority + waiting_factor
```

This creates eventual service for lower-priority Work.

# 13. Fairness

Fairness may operate across:

```text
Agents
Projects
Tenants
Work Classes
Resources
```

Example:

```text
Agent A → 100 runnable Works
Agent B → 2 runnable Works
```

Without fairness:

```text
A
A
A
A
A
...
```

could starve B.

# 14. Quotas

Agents may have quotas:

```text
max_concurrent_work = 8
max_cpu = 40%
max_memory = 1GB
max_network = ...
```

The scheduler must enforce these independently from priority.

# 15. Budget

Work may also carry budgets:

```text
time_budget
energy_budget
compute_budget
network_budget
financial_budget
retry_budget
```

Budget exhaustion should produce a defined state:

```text
BUDGET_EXHAUSTED
```

rather than an ambiguous failure.

# 16. Deadline

A Work can specify:

```text
deadline = T
```

Scheduling must consider:

```text
time_remaining
estimated_execution_time
resource_wait
dependencies
```

A Work that cannot meet its deadline may be rejected or replanned before consuming resources.

# 17. Deadline Feasibility

A useful concept:

```text
feasible =
current_time
+
estimated_wait
+
estimated_execution
≤ deadline
```

If false:

```text
DEADLINE_INFEASIBLE
```

This prevents wasting resources on impossible schedules.

# 18. Temporal Windows

Some Work is allowed only inside a window:

```text
start_after = T1
start_before = T2
```

Example:

```text
maintenance
allowed:
02:00–04:00
```

The scheduler must respect the temporal policy.

# 19. Periodic Work

Some Work recurs:

```text
every 10 seconds
```

The scheduler should distinguish:

```text
periodic schedule
```

from:

```text
repeated retry
```

These have fundamentally different semantics.

# 20. Event-Triggered Work

Work may become runnable because an event occurs:

```text
sensor.temperature > threshold
```

This suggests:

```text
Trigger
 ↓
Work instantiation
 ↓
Dependency evaluation
 ↓
Admission
```

# 21. Conditional Work

A Work Graph may contain branches:

```text
W1
 ↓
condition
 ├── true  → W2
 └── false → W3
```

The scheduler therefore needs graph semantics beyond simple DAG execution.

# 22. Work Graph Types

Possible graph classes:

```text
DAG
STATE_MACHINE
WORKFLOW
LOOP
EVENT_GRAPH
CONDITIONAL_GRAPH
HIERARCHICAL_GRAPH
```

The scheduler should not assume every Work Graph is a simple DAG.

# 23. Loops

Agentic execution naturally contains loops:

```text
Observe
 ↓
Plan
 ↓
Execute
 ↓
Reflect
 ↓
Observe
 ↓
...
```

Therefore the runtime must support controlled recurrence.

# 24. Loop Guards

A loop should have termination controls:

```text
max_iterations
deadline
budget
convergence_condition
failure_threshold
```

Otherwise:

```text
Observe → Plan → Execute
```

can become an infinite execution cycle.

# 25. Agentic Loop as Work Graph

The canonical loop can be represented:

```text
Observe
   ↓
Assess
   ↓
Plan
   ↓
Authorize
   ↓
Execute
   ↓
Verify
   ↓
Reflect
   └────────────→ Observe
```

The scheduler advances the graph according to state.

# 26. Scheduler Epoch

Scheduling decisions can occur in epochs:

```text
Epoch 100
 ↓
collect runnable work
 ↓
evaluate resources
 ↓
admit
 ↓
dispatch
 ↓
observe results
 ↓
Epoch 101
```

This gives a deterministic conceptual boundary.

# 27. Why Epochs Matter

They provide:

```text
consistent scheduling snapshot
reproducibility
diagnostics
fairness accounting
checkpoint boundary
```

without requiring the entire system to stop.

# 28. Dispatch

After admission:

```text
ADMITTED
   ↓
DISPATCH
   ↓
EXECUTING
```

Dispatch selects:

```text
executor
node
resource
execution context
```

appropriate for the Work.

# 29. Capability Matching

A Work may require:

```text
Capability:
    elevator.control
```

The scheduler searches:

```text
Executor A → elevator.control
Executor B → camera.capture
Executor C → elevator.control
```

Only compatible executors are candidates.

# 30. Resource Affinity

Some Work should prefer locality.

Example:

```text
Sensor data
```

is already on:

```text
Node B
```

so computation may be scheduled there.

This reduces:

```text
network latency
bandwidth
serialization
```

# 31. Anti-Affinity

Some Work must avoid co-location.

Example:

```text
two safety-critical controllers
```

should not depend on one physical failure domain.

The scheduler can express:

```text
anti_affinity:
    same_node = false
```

# 32. Resource Reservation

Admission can reserve resources before dispatch:

```text
Work
 ↓
Reserve
 ↓
Dispatch
 ↓
Execute
```

This prevents:

```text
admitted Work
```

from discovering too late that required resources disappeared.

# 33. Reservation Deadlock

However, reservations can create:

```text
W1 holds A → waits B
W2 holds B → waits A
```

This is deadlock.

The scheduler therefore needs deadlock prevention or detection.

# 34. Deadlock Detection

A wait-for graph:

```text
W1 → Resource B
Resource B → W2
W2 → Resource A
Resource A → W1
```

forms a cycle.

That indicates:

```text
DEADLOCK
```

# 35. Deadlock Resolution

Possible policies:

```text
abort youngest Work
rollback reservation
priority-based victim selection
resource preemption
replan
```

The policy must be deterministic.

# 36. Livelock

Livelock is different:

```text
W1 releases A
W2 releases B
W1 retries
W2 retries
...
```

The system is active but makes no progress.

NROS should detect lack of progress.

# 37. Progress

A Work can expose progress indicators:

```text
started
steps_completed
state_transitions
observations_verified
```

The scheduler can distinguish:

```text
RUNNING
```

from:

```text
RUNNING_BUT_NOT_PROGRESSING
```

# 38. Starvation

Starvation occurs when:

```text
Work remains runnable
```

but never receives execution opportunity.

Metrics:

```text
wait_duration
dispatch_count
preemption_count
```

allow detection.

# 39. Preemption

A scheduler may interrupt Work:

```text
W1 RUNNING
 ↓
higher-priority W2
 ↓
preempt W1
```

But preemption must happen at safe points.

# 40. Safe Points

A Work can declare:

```text
preemptible = true
```

and expose safe points:

```text
checkpoint
resource release
transaction boundary
observation boundary
```

The scheduler should not arbitrarily interrupt non-preemptible execution.

# 41. Cooperative Preemption

A semantic runtime can prefer:

```text
request preemption
 ↓
Work reaches safe point
 ↓
checkpoint
 ↓
PAUSED
```

over forcibly terminating execution.

# 42. Non-Preemptible Work

Some Work may be:

```text
NON_PREEMPTIBLE
```

during critical sections.

The scheduler should account for this when estimating latency.

# 43. Concurrency

Multiple Works can run simultaneously:

```text
W1 ────────→
W2 ────────→
W3 ────────→
```

provided:

```text
dependencies
resources
policy
safety
```

allow it.

# 44. Parallelism

Parallel execution should be represented explicitly.

```text
W1
 ↓
┌───────┬───────┐
↓       ↓       ↓
W2      W3      W4
└───────┴───────┘
        ↓
       W5
```

W5 waits for required predecessors.

# 45. Critical Path

For a Work Graph:

```text
W1 → W2 → W5
```

may be the critical path, while:

```text
W3 → W4
```

has slack.

The scheduler can prioritize critical-path Work when deadline pressure exists.

# 46. Slack

For each Work:

```text
slack =
deadline
-
estimated completion time
```

Low-slack Work may receive higher scheduling priority.

# 47. Cost-Aware Scheduling

A Work can estimate:

```text
CPU cost
memory cost
energy cost
network cost
execution duration
```

The scheduler can select among feasible alternatives.

# 48. Alternative Executors

Example:

```text
Work:
    image.embed

Executor A:
    GPU

Executor B:
    CPU

Executor C:
    remote accelerator
```

The scheduler selects based on:

```text
capability
cost
latency
availability
policy
```

# 49. Scheduling Decision

A conceptual decision function:

```text
score(work, executor) =
    priority
  + urgency
  + fairness
  + locality
  - estimated_cost
  - risk
```

But the exact function should remain policy-defined.

# 50. Determinism

Two identical scheduler inputs should ideally produce the same scheduling decision.

Therefore decisions should use explicit tie-breakers:

```text
priority
deadline
age
WorkId
```

rather than nondeterministic iteration order.

# 51. Scheduler State

A scheduler checkpoint may contain:

```text
epoch
runnable_work
running_work
blocked_work
reservations
leases
priorities
budgets
fairness_state
```

This enables recovery after runtime restart.

# 52. Scheduler Recovery

After restart:

```text
restore scheduler checkpoint
 ↓
reconcile running Work
 ↓
reconcile resources
 ↓
reconcile leases
 ↓
resolve unknown outcomes
 ↓
rebuild runnable frontier
```

Never simply assume:

```text
everything in RUNNING
```

is still running.

# 53. Unknown Running Work

A Work may have been executing remotely when the scheduler crashed.

After restart:

```text
W42 = UNKNOWN
```

The scheduler should query:

```text
remote executor
```

and verify state.

# 54. Scheduler as Control Loop

The scheduler itself is a feedback controller:

```text
Observe
 ↓
Evaluate
 ↓
Select
 ↓
Dispatch
 ↓
Observe outcome
 ↓
Recalculate
```

Therefore it fits naturally into the broader NROS agentic model.

# 55. Scheduler ≠ Planner

This distinction is essential.

### Planner

Answers:

```text
What should happen?
```

### Scheduler

Answers:

```text
When and where should runnable Work happen?
```

### Executor

Answers:

```text
How do we perform it?
```

### Verifier

Answers:

```text
Did the expected effect actually occur?
```

Thus:

```text
Planner
   ↓
Work Graph
   ↓
Scheduler
   ↓
Executor
   ↓
Verifier
```

# 56. Agent ≠ Scheduler

An Agent may propose:

```text
W42
```

but should not automatically control execution timing.

The scheduler remains an independent runtime authority.

This prevents an agent from bypassing:

```text
resource limits
fairness
safety
deadlines
policy
```

# 57. Agent Negotiation

An Agent can communicate:

```text
preferred deadline
priority
resource requirements
expected benefit
```

but the scheduler decides whether those claims are admissible.

# 58. Scheduler Governance

The scheduler itself must be policy-governed.

Policies may specify:

```text
max concurrency
priority bounds
quota
preemption rules
safety constraints
deadline behavior
resource fairness
```

# 59. Admission → Scheduling

The full pipeline becomes:

```text
Agent Proposal
      ↓
Validation
      ↓
Authorization
      ↓
Work Creation
      ↓
Dependency Analysis
      ↓
Runnable?
      ↓
Admission
      ↓
Resource Reservation
      ↓
Scheduling
      ↓
Dispatch
      ↓
Execution
```

This ordering prevents the scheduler from becoming an authorization bypass.

# 60. Scheduling → Verification

The loop continues:

```text
Execution
 ↓
Observation
 ↓
Verification
 ↓
State Update
 ↓
Work Graph Update
 ↓
New Runnable Frontier
```

This means the scheduler is continuously driven by evidence.

# 61. Evidence-Driven Scheduling

Suppose:

```text
W1:
    move elevator to floor 7
```

Verification produces:

```text
floor = 7
```

which activates:

```text
W2:
    open doors
```

Thus evidence directly changes scheduler state.

# 62. Failed Verification

If:

```text
W1
```

executes but verification fails:

```text
floor != 7
```

then W2 should not automatically run.

Instead:

```text
W1
 ↓
VERIFICATION_FAILED
 ↓
RECOVERY / REPLAN
```

This protects dependency correctness.

# 63. Partial Completion

A Work may produce partial progress:

```text
W1:
    move 0 → 10
```

expected:

```text
0 → 20
```

The scheduler may create:

```text
W1b:
    continue 10 → 20
```

or:

```text
replan
```

depending on semantics.

# 64. Work Identity Across Attempts

Retries should not create unrelated semantic identities.

Prefer:

```text
WorkId = W42

Attempt 1
Attempt 2
Attempt 3
```

rather than:

```text
W42
W43
W44
```

unless the planner explicitly creates new Work.

# 65. Work vs Attempt

This distinction becomes fundamental:

```text
Work
 ├── Attempt 1
 ├── Attempt 2
 └── Attempt 3
```

Work represents intent.

Attempt represents one execution realization.

# 66. Work State

A robust state machine:

```text
PROPOSED
 ↓
VALIDATED
 ↓
AUTHORIZED
 ↓
BLOCKED
 ↓
RUNNABLE
 ↓
ADMITTED
 ↓
RESERVED
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
RUNNING → PAUSED
RUNNING → CANCELLED
RUNNING → UNKNOWN
```

# 67. Scheduler Invariants

The scheduler should guarantee:

```text
1. Blocked Work is never dispatched.

2. Unauthorized Work is never dispatched.

3. Expired Work is never dispatched without revalidation.

4. Resource requirements are satisfied before execution.

5. Work identity remains stable across retries.

6. Attempts are independently observable.

7. Dependency failures propagate deterministically.

8. Preemption occurs only at valid boundaries.

9. Scheduler decisions are reproducible given equivalent state.

10. Recovery reconstructs the runnable frontier from verified state.

11. A successful execution does not imply successful Work completion until verification.

12. The scheduler never substitutes priority for safety policy.
```

# 68. Agent-Native Scheduler

The architectural distinction can now be stated precisely.

Traditional:

```text
Runtime
 └── Scheduler
      └── Futures
           └── Poll
```

NROS:

```text
Runtime
 ├── Agents
 ├── Work Graphs
 ├── Policy
 ├── Resources
 ├── State
 ├── Evidence
 └── Scheduler
      ├── Runnable Work
      ├── Admission
      ├── Resource Matching
      ├── Dispatch
      ├── Preemption
      └── Recovery
```

The scheduler is therefore scheduling **semantic work**, not merely computational continuations.

# 69. The Complete NROS Execution Architecture

At this point:

```text
                           ┌─────────────┐
                           │   AGENTS    │
                           └──────┬──────┘
                                  ↓
                              INTENTS
                                  ↓
                               PLANNER
                                  ↓
                            WORK GRAPH
                                  ↓
                             VALIDATOR
                                  ↓
                            AUTHORITY
                                  ↓
                             ADMISSION
                                  ↓
                         ┌───────────────┐
                         │   SCHEDULER   │
                         └───────┬───────┘
                                 ↓
                    ┌────────────┼────────────┐
                    ↓            ↓            ↓
                RESOURCE     EXECUTOR      REMOTE
                MATCHING                    NODE
                    │            │            │
                    └────────────┼────────────┘
                                 ↓
                              EXECUTION
                                 ↓
                              OBSERVE
                                 ↓
                              EVIDENCE
                                 ↓
                             VERIFY
                                 ↓
                    ┌────────────┴────────────┐
                    ↓                         ↓
                COMPLETE                 ABNORMAL
                                              ↓
                                          RECOVERY
                                              ↓
                                           REPLAN
                                              ↓
                                         SCHEDULER
```

# 70. The Key NROS Abstraction

The most important abstraction emerging from these layers is:

```text
Agent
   ↓
Work
   ↓
Attempt
   ↓
Execution
   ↓
Observation
   ↓
Evidence
   ↓
Verification
```

This gives NROS a stable semantic chain from **intent to observed reality**.

# 71. Next Boundary — Memory & State

The scheduler now depends heavily on persistent state.

An autonomous runtime must answer:

> What does the system remember between executions, failures, restarts, and agent reasoning cycles?

That takes us to:

# Part LXXIV — NROS State, Memory, Context & Checkpoint Architecture

We should next formalize:

```text
Runtime State
Agent State
Work State
World State
Resource State
Policy State
Execution State
Memory
Context
Snapshot
Checkpoint
Journal
Event Log
State Store
Cache
Ephemeral State
Durable State
Derived State
Observed State
Believed State
Verified State
Stale State
State Reconciliation
Recovery
Compaction
Garbage Collection
```

The central question becomes:

> **How can NROS distinguish what it currently believes from what it has actually verified—and recover that distinction after a crash?**

# NROS — Part LXXIV: State, Memory, Context & Checkpoint Architecture

The previous layer established the scheduler as the mechanism that turns **runnable semantic Work** into execution.

But scheduling depends on something deeper:

> **What state does NROS actually know, what does it merely believe, and what survives failure?**

This distinction is foundational for an agent-native runtime.

A conventional application often has:

```text
memory → variables → persistence
```

NROS needs something richer:

```text
Observation
    ↓
Evidence
    ↓
Verified State
    ↓
Derived State
    ↓
Agent Context
    ↓
Planning
    ↓
Work
```

And after failure:

```text
Checkpoint
    ↓
Recovery
    ↓
Reconciliation
    ↓
Verified State
```

# 1. State Is Not One Thing

NROS should distinguish several classes:

```text
EPHEMERAL_STATE
DURABLE_STATE
OBSERVED_STATE
VERIFIED_STATE
DERIVED_STATE
BELIEVED_STATE
STALE_STATE
UNKNOWN_STATE
```

Treating all of them as a generic `State` would create serious correctness problems.

# 2. World State

World State represents NROS's model of the external world.

Example:

```text
elevator.floor = 7
door.state = CLOSED
motor.state = STOPPED
```

But this model may be:

```text
observed
derived
stale
unknown
```

Therefore:

```text
WorldState ≠ Reality
```

It is a representation of reality supported by evidence.

# 3. Observed State

Suppose a sensor reports:

```text
floor = 7
```

NROS records:

```text
Observation
    source = floor_sensor
    value = 7
    observed_at = T
```

This becomes observed state.

# 4. Verified State

An observation becomes stronger when verified.

For example:

```text
sensor A → floor 7
sensor B → floor 7
```

or:

```text
sensor → floor 7
controller → floor 7
```

NROS may then establish:

```text
VerifiedState:
    floor = 7
```

The verification policy determines what evidence is sufficient.

# 5. Believed State

Agents may maintain hypotheses:

```text
I believe:
    elevator is probably at floor 7
```

This is useful for planning but must not automatically become verified state.

Thus:

```text
BELIEF
   ≠
VERIFIED FACT
```

# 6. Unknown State

Sometimes the correct state is:

```text
UNKNOWN
```

Example:

```text
command sent
↓
network partition
↓
remote outcome unknown
```

NROS should preserve:

```text
door.state = UNKNOWN
```

rather than guessing.

# 7. Stale State

A previously verified state may become stale.

Example:

```text
10:00 → floor = 7 verified
```

At:

```text
10:30
```

without new observations:

```text
floor = 7
```

may still be historically verified but no longer current.

This distinction should be represented.

# 8. State Freshness

State entries can carry:

```text
observed_at
verified_at
expires_at
source
confidence
```

Then consumers can decide whether the information is sufficiently fresh.

# 9. State Provenance

Every consequential state value should be traceable to its origin:

```text
State
 ↓
Evidence
 ↓
Observation
 ↓
Source
```

Example:

```text
door = CLOSED
    ↓
Observation #832
    ↓
door_sensor_2
    ↓
2026-08-21T05:32:11Z
```

# 10. State Version

A state store should support versions:

```text
door.state:
    v41 = OPEN
    v42 = CLOSED
    v43 = OPEN
```

This enables:

```text
history
comparison
rollback analysis
causal reconstruction
```

without confusing history with current state.

# 11. State Transition

State changes should be explicit:

```text
OPEN
 ↓
CLOSING
 ↓
CLOSED
```

rather than simply overwriting:

```text
state = CLOSED
```

The transition itself can carry evidence.

# 12. Event-Sourced State

One model is:

```text
Event Log
   ↓
State Projection
```

For example:

```text
DoorOpened
DoorClosing
DoorClosed
```

produce:

```text
door.state = CLOSED
```

This provides excellent traceability.

# 13. Snapshot-Based State

Pure event replay can become expensive.

NROS can periodically create:

```text
Snapshot
```

such as:

```text
Snapshot #1000
```

containing the current verified/derived state.

Recovery then becomes:

```text
Snapshot
 ↓
Replay subsequent events
 ↓
Current State
```

# 14. Journal

A journal records state-changing operations:

```text
timestamp
actor
operation
previous state
new state
evidence
```

The journal provides durable history.

# 15. Checkpoint

A checkpoint is different from a generic state snapshot.

It captures enough runtime information to resume safely.

For example:

```text
Checkpoint {
    runtime_state
    scheduler_state
    active_work
    attempts
    resources
    leases
    authority
    pending_messages
    state_versions
}
```

# 16. Checkpoint Is Not Reality

A checkpoint records what NROS knew at checkpoint time.

It does not guarantee that the external world remained unchanged.

Therefore:

```text
Checkpoint
   ≠
Current World
```

Recovery must reconcile.

# 17. Recovery Sequence

A safe recovery process:

```text
Load checkpoint
      ↓
Restore internal state
      ↓
Discover external state
      ↓
Reconcile resources
      ↓
Reconcile leases
      ↓
Resolve unknown Work
      ↓
Verify critical state
      ↓
Rebuild scheduler
      ↓
Resume / replan
```

# 18. Runtime State

Runtime state includes:

```text
scheduler
workers
queues
sessions
timers
subscriptions
resource leases
active Work
```

Some of this can be reconstructed.

Some must be persisted.

# 19. Reconstructible State

Derived state should not always be persisted.

For example:

```text
runnable_work
```

may be reconstructed from:

```text
Work state
+
dependencies
+
resource state
+
policy
```

This reduces persistence complexity.

# 20. Source-of-Truth Principle

NROS should explicitly identify authoritative sources.

For example:

```text
Hardware:
    source of truth for physical position

Resource Manager:
    source of truth for leases

Policy Engine:
    source of truth for authorization

Scheduler:
    source of truth for dispatch state
```

One subsystem should not silently override another's authority.

# 21. State Ownership

Every state domain should have an owner.

Example:

```text
World Position
    → Sensor/World-State subsystem

Resource Lease
    → Resource Manager

Work Lifecycle
    → Work/Scheduler subsystem

Authorization
    → Policy subsystem
```

This prevents competing writers.

# 22. Single Writer Principle

Where practical:

```text
one authoritative writer
many readers
```

This simplifies concurrency and auditability.

Distributed systems may require coordination, but semantic ownership should remain explicit.

# 23. State Store

Conceptually:

```text
StateStore
├── get
├── put
├── compare_and_set
├── snapshot
├── history
├── subscribe
└── reconcile
```

The interface should preserve semantic metadata, not merely key/value pairs.

# 24. Compare-and-Set

Concurrent agents may both attempt:

```text
door.state = OPEN
```

A version-aware operation:

```text
expected_version = 42
new_value = OPEN
```

prevents stale writes.

# 25. Optimistic Concurrency

Example:

```text
read v42
 ↓
plan based on v42
 ↓
another actor writes v43
 ↓
attempt write against v42
 ↓
CONFLICT
```

NROS can then:

```text
re-read
replan
retry
```

instead of silently overwriting newer state.

# 26. State Conflict

A conflict means:

```text
Expected state ≠ Current state
```

This should become a first-class condition:

```text
STATE_CONFLICT
```

rather than a generic database error.

# 27. Context

Agent Context is the subset of state relevant to an Agent's current reasoning.

It may include:

```text
goal
recent observations
active Work
constraints
available resources
policy constraints
relevant history
```

Context is therefore not identical to global state.

# 28. Context Window

The agent should not receive the entire runtime state.

Instead:

```text
Global State
    ↓
Relevance Filtering
    ↓
Policy Filtering
    ↓
Context Assembly
    ↓
Agent
```

This reduces noise and preserves boundaries.

# 29. Context Provenance

Each context item should retain:

```text
source
timestamp
confidence
state version
evidence reference
```

An agent should be able to distinguish:

```text
fresh observation
```

from:

```text
old inferred state
```

# 30. Memory

Memory should be divided by semantic role.

Possible classes:

```text
WORKING_MEMORY
EPISODIC_MEMORY
SEMANTIC_MEMORY
PROCEDURAL_MEMORY
OBSERVATIONAL_MEMORY
EVENT_MEMORY
POLICY_MEMORY
```

Not every memory type needs the same storage mechanism.

# 31. Working Memory

Short-lived information:

```text
current plan
current context
temporary calculations
active hypotheses
```

Usually ephemeral.

# 32. Episodic Memory

Records what happened:

```text
At T:
    Work W42 executed
    Observation X occurred
    Recovery Y followed
```

Useful for:

```text
history
diagnosis
reflection
audit
```

# 33. Semantic Memory

Generalized knowledge:

```text
motor controller supports:
    position
    speed
    stop
```

This is different from a specific execution episode.

# 34. Procedural Memory

Procedures or learned strategies:

```text
If controller timeout:
    verify remote state
    then retry if safe
```

Procedural memory must still obey runtime policy.

Memory cannot become an authorization bypass.

# 35. Memory Provenance

Any memory item that can influence execution should identify:

```text
origin
confidence
timestamp
version
evidence
authority
```

This is especially important for agent-generated memories.

# 36. Agent-Generated Memory

An Agent may infer:

```text
"The door is probably jammed."
```

That is a hypothesis.

It should not be stored as:

```text
door.jammed = true
```

without qualification.

Better:

```text
Hypothesis {
    subject: door
    proposition: jammed
    confidence: 0.72
    evidence: [...]
}
```

# 37. Belief Store

NROS may maintain:

```text
BeliefStore
```

separate from:

```text
VerifiedStateStore
```

This is a powerful distinction for autonomous reasoning.

# 38. Confidence

Beliefs may carry:

```text
confidence
supporting evidence
contradicting evidence
last updated
```

But confidence must not override hard safety requirements.

For example:

```text
confidence = 0.99
```

does not mean:

```text
safety condition satisfied
```

# 39. Contradictory Evidence

Suppose:

```text
Sensor A → CLOSED
Sensor B → OPEN
```

NROS should preserve the contradiction:

```text
STATE_CONFLICT
```

rather than selecting one arbitrarily.

The reconciliation subsystem determines what happens next.

# 40. Reconciliation

Reconciliation can use:

```text
source authority
sensor quality
freshness
redundancy
causal ordering
policy
```

to resolve or preserve uncertainty.

# 41. State Confidence

A state value may have:

```text
UNKNOWN
OBSERVED
CORROBORATED
VERIFIED
CONFLICTED
STALE
```

This gives the runtime richer semantics than a boolean `valid`.

# 42. State Freshness vs Confidence

These are independent.

Example:

```text
door = CLOSED
confidence = high
freshness = low
```

This means:

> The value was highly reliable when observed, but the observation is old.

That is different from:

```text
door = CLOSED
confidence = low
freshness = high
```

# 43. State Quality Model

A state record could conceptually contain:

```text
StateValue {
    value
    version
    observed_at
    verified_at
    freshness
    confidence
    provenance
    status
}
```

# 44. Ephemeral vs Durable

Not every state needs persistence.

### Ephemeral

```text
temporary buffers
network sockets
worker-local caches
current poll state
```

### Durable

```text
Work identity
audit events
critical checkpoints
resource ownership
verified history
policy state
```

# 45. Persistence Policy

Each state category should define:

```text
durability
retention
consistency
recovery behavior
```

rather than allowing storage implementation to decide implicitly.

# 46. Memory Retention

Memory should have lifecycle:

```text
CREATE
 ↓
ACTIVE
 ↓
AGING
 ↓
COMPACTED
 ↓
ARCHIVED
 ↓
EXPIRED
```

This prevents unbounded growth.

# 47. Compaction

An event stream:

```text
100,000 observations
```

may eventually become:

```text
Snapshot
+
important events
+
audit references
```

while preserving required provenance.

# 48. Garbage Collection

Temporary state should be reclaimable.

But GC must respect references from:

```text
active Work
evidence
audit records
checkpoints
memory
```

A referenced object must not disappear merely because it is old.

# 49. Memory Isolation

Different Agents may have different memory scopes:

```text
Agent A
 └── private memory

Agent B
 └── private memory

Shared
 └── explicitly authorized memory
```

An Agent should not automatically see another Agent's private context.

# 50. Memory Authority

Memory can influence planning, but:

```text
memory
```

should not automatically grant:

```text
authority
```

This remains consistent with the security model.

# 51. State Subscriptions

Agents and runtime components may subscribe:

```text
subscribe:
    elevator.floor
```

When state changes:

```text
State Update
 ↓
Event
 ↓
Subscriber
```

This can trigger new Work.

# 52. State-Driven Scheduling

The scheduler can consume state events:

```text
door = CLOSED
       ↓
condition satisfied
       ↓
W42 becomes RUNNABLE
```

This creates a direct relationship between state and scheduling.

# 53. Context Rebuild

After restart, an Agent's context should not simply be restored from a serialized prompt.

Instead:

```text
Durable State
+
Verified Evidence
+
Active Work
+
Current Policy
+
Relevant Memory
↓
Context Rebuild
```

This is much safer.

# 54. Checkpointed Agent State

A checkpoint may include:

```text
Agent identity
current goal
active Work
reasoning state summary
memory references
pending decisions
context references
```

But raw model internals should not necessarily be treated as authoritative runtime state.

# 55. Reasoning State vs Runtime State

This distinction is important.

```text
Runtime State
    → authoritative

Agent Reasoning State
    → advisory / reconstructible
```

The runtime should remain correct even if an Agent loses its reasoning context.

# 56. Agent Restart

If an Agent crashes:

```text
Agent
 ↓
CRASH
 ↓
Restart
 ↓
Load durable state
 ↓
Rebuild context
 ↓
Observe current world
 ↓
Resume/replan
```

The runtime should not require perfect restoration of every transient model thought.

# 57. Checkpoint Granularity

Checkpointing can occur at:

```text
Work boundary
Attempt boundary
State transition
Safe point
Scheduling epoch
Periodic interval
```

The appropriate granularity depends on recovery requirements.

# 58. Atomic Checkpoint

A critical checkpoint should have a consistent boundary:

```text
State
+
Scheduler
+
Work
+
Resource
+
Lease
```

must correspond to one coherent runtime view.

Partial checkpoint state can produce unsafe recovery.

# 59. Write-Ahead Journal

One approach:

```text
Intent
 ↓
Journal
 ↓
State mutation
 ↓
Execution
```

The journal records enough information to recover after interruption.

# 60. Journal vs Evidence

They are related but distinct.

### Journal

Answers:

> What did the runtime record as a state-changing operation?

### Evidence

Answers:

> What evidence supports what actually happened?

A journal entry may say:

```text
command dispatched
```

while evidence may later show:

```text
motor reached target
```

# 61. Checkpoint vs Journal

These serve different purposes:

```text
Journal
→ history

Checkpoint
→ recovery starting point

State Store
→ current view

Evidence Store
→ proof/provenance
```

Together they form the state substrate.

# 62. State Architecture

A conceptual architecture:

```text
                  NROS STATE
                      │
        ┌─────────────┼─────────────┐
        ↓             ↓             ↓
   CURRENT STATE    JOURNAL      EVIDENCE
        │             │             │
        ↓             ↓             ↓
    PROJECTIONS    HISTORY       PROVENANCE
        │
        ↓
   CHECKPOINTS
        │
        ↓
    RECOVERY
```

# 63. State Recovery Pipeline

After a crash:

```text
Persistent Storage
       ↓
Load latest checkpoint
       ↓
Replay journal
       ↓
Reconstruct state
       ↓
Discover external world
       ↓
Reconcile
       ↓
Verify
       ↓
Publish current state
```

# 64. Reconciliation Is Mandatory

This deserves an invariant:

> **Persisted state is evidence of what NROS previously knew, not proof of what the external world currently is.**

Therefore recovery must include fresh observation for critical state.

# 65. Critical State

Critical state includes things like:

```text
physical actuator status
resource ownership
safety conditions
remote Work outcome
authority leases
```

These require stronger reconciliation than ordinary cache state.

# 66. Derived State

Some values can be recomputed:

```text
runnable Work
priority score
cache indexes
aggregates
```

Persisting them may be optional.

This reduces inconsistency risk.

# 67. State Dependency Graph

Derived state should declare dependencies:

```text
WorldState
   ↓
DependencyEvaluation
   ↓
RunnableWork
   ↓
SchedulerQueue
```

If WorldState changes:

```text
invalidate
recompute
```

rather than trusting stale derived values.

# 68. Cache

A cache is:

```text
derived + disposable
```

It must never become the hidden source of truth.

If the cache disappears:

```text
recompute
```

must be possible.

# 69. Memory Retrieval

Agent memory retrieval should be:

```text
query
 ↓
candidate memories
 ↓
relevance
 ↓
authority filtering
 ↓
freshness filtering
 ↓
context assembly
```

Not simply:

```text
vector similarity
```

# 70. Semantic Memory and Runtime State

Vector retrieval can help agents remember:

```text
similar past incidents
procedures
documentation
experiences
```

but it should not replace authoritative state.

For example:

```text
Vector DB:
    "door was closed yesterday"
```

cannot establish:

```text
door is closed now
```

# 71. Memory Retrieval Safety

Memory used for action selection should be classified:

```text
INFORMATIVE
ADVISORY
AUTHORITATIVE
```

Only authorized authoritative sources should satisfy hard runtime constraints.

# 72. Context Freshness

Before executing consequential Work, the Agent/Scheduler may require:

```text
context_freshness ≤ threshold
```

If not:

```text
REFRESH OBSERVATION
```

before proceeding.

# 73. Context Invalidation

A new observation may invalidate prior reasoning:

```text
Agent believes:
    door = CLOSED

New observation:
    door = OPEN
```

The associated plan should become:

```text
STALE
```

and require reconsideration.

# 74. Plan Validity

A plan can carry assumptions:

```text
Plan P42
assumptions:
    floor = 3
    door = CLOSED
    battery > 30%
```

If any assumption changes:

```text
Plan P42
    ↓
INVALIDATED
```

or:

```text
REVALIDATION_REQUIRED
```

# 75. State-Dependent Planning

This gives us:

```text
Plan
+
Assumptions
+
State Version
```

The scheduler can then determine whether a plan remains valid.

# 76. State Epoch

A global or domain-specific state epoch can provide:

```text
WorldState epoch = 8842
```

A plan created against:

```text
epoch = 8838
```

may require revalidation if the relevant state changed.

# 77. Domain-Scoped Epochs

Global epochs can become expensive.

Instead:

```text
motion_state epoch = 91
door_state epoch = 18
resource_state epoch = 402
```

allow more precise invalidation.

# 78. Context Contract

An Agent should receive a structured context:

```text
Context {
    goal
    relevant_state
    constraints
    active_work
    available_capabilities
    resources
    recent_evidence
    relevant_memory
}
```

The runtime controls what enters this context.

# 79. Agent Output

The Agent should produce structured proposals:

```text
Proposal {
    intent
    work_graph
    assumptions
    expected_outcomes
    resource_requirements
    deadline
}
```

The runtime then validates the proposal.

# 80. Memory Does Not Execute

This gives a powerful boundary:

```text
Memory
 ↓
Context
 ↓
Reasoning
 ↓
Proposal
 ↓
Runtime validation
 ↓
Execution
```

An Agent cannot directly convert a remembered fact into an actuator command without runtime mediation.

# 81. State Invariants

NROS should enforce:

```text
1. Verified state must retain provenance.

2. Unknown state must not silently become success.

3. Stale state must be distinguishable from fresh state.

4. Belief must remain distinguishable from fact.

5. Checkpoints do not establish current external reality.

6. Critical state requires reconciliation after recovery.

7. Derived state must be reconstructible.

8. Cache must not become hidden authority.

9. State ownership must be explicit.

10. Conflicting observations must remain representable.

11. Agent memory cannot override runtime authority.

12. Plan assumptions must be invalidatable.

13. State mutations require version-aware concurrency control.

14. Runtime recovery must reconstruct scheduling from authoritative state.

15. Durable state and evidence must remain causally traceable.
```

# 82. Unified NROS State Model

The architecture now looks like:

```text
                 EXTERNAL WORLD
                       │
                       ↓
                  OBSERVATION
                       │
                       ↓
                    EVIDENCE
                       │
                       ↓
                   VERIFICATION
                       │
                       ↓
                 VERIFIED STATE
                       │
              ┌────────┴────────┐
              ↓                 ↓
        DERIVED STATE        MEMORY
              │                 │
              ↓                 ↓
          SCHEDULER          CONTEXT
              │                 │
              └────────┬────────┘
                       ↓
                     AGENT
                       ↓
                    PROPOSAL
                       ↓
                 VALIDATION
                       ↓
                  WORK GRAPH
                       ↓
                  EXECUTION
```

# 83. Crash Recovery Model

The complete recovery cycle is now:

```text
                    CRASH
                      ↓
                LOAD CHECKPOINT
                      ↓
                REPLAY JOURNAL
                      ↓
              RECONSTRUCT STATE
                      ↓
             DISCOVER WORLD STATE
                      ↓
              RECONCILE RESOURCES
                      ↓
               RECONCILE LEASES
                      ↓
              RESOLVE UNKNOWN WORK
                      ↓
              VERIFY CRITICAL STATE
                      ↓
              REBUILD WORK GRAPH
                      ↓
             REBUILD RUNNABLE FRONTIER
                      ↓
                RESUME / REPLAN
```

# 84. The Deeper NROS Model

At this stage, NROS can be understood as a sequence of transformations:

```text
Reality
  ↓
Observation
  ↓
Evidence
  ↓
State
  ↓
Context
  ↓
Agent Reasoning
  ↓
Intent
  ↓
Work
  ↓
Schedule
  ↓
Execution
  ↓
Observation
```

This is a **closed semantic control loop**.

# 85. The Core Boundary

The most important distinction is now:

```text
What the Agent thinks
        ≠
What the Runtime knows
        ≠
What the World currently is
```

NROS exists partly to maintain those boundaries while continuously reconciling them.

# 86. Next Major Layer — Resource Model

The scheduler currently depends on:

```text
resource availability
resource ownership
resource capacity
resource leases
resource health
resource affinity
```

So the next layer should formalize resources themselves.

# Part LXXV — NROS Resource, Capability, Lease & Allocation Architecture

We should define:

```text
Resource
ResourceId
ResourceClass
Capability
Capacity
Availability
Health
Ownership
Lease
Reservation
Allocation
Sharing
Exclusivity
Concurrency
Affinity
Anti-Affinity
Quota
Budget
Admission
Fencing
Resource Pool
Resource Graph
Composite Resource
Virtual Resource
Remote Resource
Device Resource
Compute Resource
Memory Resource
Network Resource
Temporal Resource
Energy Resource
```

The central question will be:

> **What exactly does it mean for an Agent or Work to "have" a resource in NROS—and how can the runtime guarantee that two independent agents cannot simultaneously believe they own the same scarce capability?**

# NROS — Part LXXV: Resource, Capability, Lease & Allocation Architecture

The previous layer established **state and memory as the substrate for recovery and reasoning**.

Now we need to formalize the thing the scheduler actually allocates:

> **Resources.**

In NROS, a resource should not be reduced to:

```text
CPU = 4
RAM = 8 GB
```

An agent may need something much richer:

```text
"exclusive control of elevator motor M1
for up to 5 seconds
on controller C7
under safety policy P3"
```

That is simultaneously a:

- resource requirement,
- capability requirement,
- authority requirement,
- temporal constraint,
- allocation,
- lease.

# 1. Resource

The base abstraction:

```text
Resource {
    id
    class
    capacity
    state
    capabilities
    ownership
    policy
}
```

Examples:

```text
CPU
Memory
GPU
Network
Disk
Device
Sensor
Actuator
PTY
SSH session
Database connection
Agent slot
Model endpoint
Physical machine
```

# 2. Resource Is Not Capability

These must remain distinct.

A resource answers:

> **What exists?**

A capability answers:

> **What can be done with it?**

Example:

```text
Resource:
    ElevatorController#7

Capabilities:
    read_position
    read_faults
    command_motion
```

Possessing the resource does not automatically imply permission to invoke every capability.

# 3. Capability

Conceptually:

```text
Capability {
    id
    operation
    target
    constraints
    authority_requirements
}
```

Example:

```text
elevator.motion.command
```

may require:

```text
SafetyAuthority
+
ControllerLease
+
MotionPolicy
```

# 4. Resource Classes

NROS should support typed resource classes:

```text
COMPUTE
MEMORY
STORAGE
NETWORK
DEVICE
SENSOR
ACTUATOR
PROCESS
SESSION
MODEL
SERVICE
CREDENTIAL
LOCK
TIME
ENERGY
```

The scheduler can then apply class-specific allocation rules.

# 5. Physical vs Virtual Resources

### Physical

```text
motor
sensor
GPU
network interface
```

### Virtual

```text
PTY
container
logical CPU quota
database connection
agent slot
```

Both should expose the same high-level resource contract where possible.

# 6. Resource Identity

Every resource requires a stable identity:

```text
ResourceId
```

For distributed systems:

```text
node-7/device/elevator/controller-3
```

is preferable to relying on transient process IDs.

# 7. Resource State

A resource may be:

```text
AVAILABLE
ALLOCATED
RESERVED
DEGRADED
UNAVAILABLE
FAILED
UNKNOWN
RETIRED
```

Again:

```text
UNKNOWN
```

must remain distinct from:

```text
AVAILABLE
```

# 8. Resource Capacity

Capacity can be scalar:

```text
CPU = 8 cores
```

or multidimensional:

```text
GPU:
    compute = 100
    memory = 24GB
```

or semantic:

```text
Controller:
    concurrent_commands = 1
```

# 9. Capacity Vector

A generalized model:

```text
Capacity {
    dimensions: Map<Dimension, Quantity>
}
```

Example:

```text
{
    cpu: 4,
    memory: 8192MB,
    bandwidth: 100Mbps
}
```

A Work can request a subset.

# 10. Consumable vs Non-Consumable

Some resources are consumed:

```text
CPU time
bandwidth
energy
storage
```

Others are held:

```text
device control
exclusive lock
PTY
database transaction
```

These require different allocation semantics.

# 11. Exclusive Resource

Example:

```text
motor_controller
```

Only one Work can own it at a time:

```text
W1 → OWNER
W2 → WAITING
```

# 12. Shared Resource

Some resources support concurrent readers:

```text
temperature_sensor
```

Possible model:

```text
READ → shared
WRITE → exclusive
```

This resembles reader/writer semantics.

# 13. Concurrency Limit

A resource can permit:

```text
max_concurrent_users = 4
```

Then:

```text
W1
W2
W3
W4
```

may execute, while:

```text
W5
```

waits.

# 14. Resource Requirement

A Work declares requirements:

```text
ResourceRequirement {
    class
    capability
    quantity
    mode
    constraints
}
```

Example:

```text
requires:
    capability = camera.capture
    mode = exclusive
```

# 15. Capability Matching

The scheduler performs:

```text
Work Requirement
       ↓
Capability Registry
       ↓
Candidate Resources
       ↓
Policy Filtering
       ↓
Availability
       ↓
Allocation
```

# 16. Resource Registry

NROS needs a discoverable registry:

```text
ResourceRegistry
├── register
├── unregister
├── describe
├── query
├── watch
└── health
```

Resources can dynamically appear and disappear.

# 17. Dynamic Resources

Example:

```text
USB camera plugged in
```

causes:

```text
RESOURCE_DISCOVERED
```

Removal causes:

```text
RESOURCE_LOST
```

The scheduler must react to both.

# 18. Resource Health

A resource may be:

```text
HEALTHY
DEGRADED
SUSPECT
FAILED
UNKNOWN
```

Health is separate from availability.

A resource can be:

```text
AVAILABLE + DEGRADED
```

and policy may prohibit critical Work from using it.

# 19. Resource Ownership

Ownership means:

> Which principal currently holds the allocation authority for this resource?

Example:

```text
motor M1
    ↓
Lease L42
    ↓
Work W17
```

Ownership should never exist merely as an in-memory boolean.

# 20. Lease

A lease is a **time-bounded authority to hold/use a resource**.

```text
Lease {
    id
    resource
    holder
    issued_at
    expires_at
    constraints
}
```

# 21. Why Leases?

Suppose:

```text
Agent A
```

acquires a resource and crashes.

Without expiration:

```text
resource = permanently locked
```

With a lease:

```text
A crashes
 ↓
lease expires
 ↓
resource becomes reclaimable
```

# 22. Lease Renewal

Long-running Work can renew:

```text
Lease L42
 ↓
renew
 ↓
new expiration
```

But renewal should be policy-controlled.

An unhealthy or abandoned Work must not renew forever.

# 23. Lease Ownership

The runtime must establish:

```text
LeaseId
+
ResourceId
+
HolderId
+
Version
```

as the authoritative ownership tuple.

# 24. Fencing

Leases alone are insufficient in distributed systems.

Suppose:

```text
Node A
```

loses connectivity but continues executing.

Meanwhile:

```text
Node B
```

acquires a new lease.

Now A still believes:

```text
"I own the resource."
```

This is the classic stale-owner problem.

# 25. Fencing Token

Each lease allocation should receive a monotonically increasing token:

```text
Lease 41 → fencing_token = 900
Lease 42 → fencing_token = 901
```

The resource accepts only operations carrying the latest valid token.

# 26. Fencing Principle

Therefore:

```text
old owner
    ↓
token 900
    ↓
REJECT
```

while:

```text
current owner
    ↓
token 901
    ↓
ACCEPT
```

This is critical for safety-sensitive resources.

# 27. Lease ≠ Authorization

A valid lease says:

> You hold this allocation.

Authorization says:

> You are permitted to perform this operation.

Both must be satisfied:

```text
Capability
∧
Authorization
∧
Lease
```

# 28. Allocation

Allocation is the transition:

```text
AVAILABLE
   ↓
ALLOCATED
```

It creates a binding:

```text
Work
 ↔
Resource
```

with explicit semantics.

# 29. Reservation

Reservation is different from allocation.

Reservation means:

> This resource is intended to be available for future Work.

Example:

```text
W42
deadline = 10:00
```

reserves:

```text
GPU#2
```

for a future execution window.

# 30. Reservation Timeline

```text
09:30
reserve

09:55
activate

10:00
execute

10:05
release
```

Expired reservations should be reclaimed automatically.

# 31. Allocation Graph

A resource manager can maintain:

```text
Resources
    ↓
Reservations
    ↓
Leases
    ↓
Allocations
    ↓
Active Work
```

This graph should be queryable.

# 32. Composite Resources

Some Work requires multiple resources:

```text
GPU
+
GPU memory
+
network
+
model endpoint
```

This is a composite allocation.

It must be treated atomically where possible.

# 33. Partial Allocation Problem

Bad sequence:

```text
allocate GPU
 ↓
allocate memory
 ↓
network unavailable
```

Now the scheduler must release previously acquired resources.

Better:

```text
feasibility check
 ↓
atomic reservation
 ↓
allocation
```

# 34. Transactional Allocation

Conceptually:

```text
BEGIN ALLOCATION
    reserve A
    reserve B
    reserve C
COMMIT
```

If any requirement fails:

```text
ROLLBACK
```

This prevents resource leakage.

# 35. Resource Deadlock

Composite allocation can produce:

```text
W1:
    holds A
    waits B

W2:
    holds B
    waits A
```

The resource graph therefore feeds directly into the scheduler's deadlock detector.

# 36. Resource Ordering

One prevention technique:

```text
always acquire resources
in canonical order
```

Example:

```text
A < B < C
```

Every Work must acquire:

```text
A → B → C
```

rather than arbitrary order.

# 37. Resource Preemption

Some resources are preemptible:

```text
CPU
GPU allocation
network bandwidth
```

Others may not be:

```text
physical actuator control
transaction
critical section
```

The resource declares:

```text
preemptibility
```

rather than assuming it globally.

# 38. Safe Resource Release

When Work ends:

```text
EXECUTION
 ↓
CLEANUP
 ↓
RELEASE
 ↓
VERIFY_RELEASE
```

The resource should not simply be marked free before release has actually been confirmed where that distinction matters.

# 39. Resource Release Failure

Suppose:

```text
W42 completes
```

but its external session remains active.

Then:

```text
release failed
```

must produce:

```text
RESOURCE_RETAINED
```

rather than:

```text
AVAILABLE
```

# 40. Resource Leak

NROS should detect:

```text
lease exists
AND
holder no longer exists
```

This is a leak candidate.

Recovery can:

```text
revoke
fence
reconcile
release
```

according to policy.

# 41. Capability Revocation

A capability can be revoked while Work is running.

Example:

```text
Work W42
 ↓
has capability C7
 ↓
policy changes
 ↓
C7 revoked
```

The runtime must define whether:

```text
execution stops
```

or:

```text
current operation completes
```

before revocation takes effect.

# 42. Capability Version

Capabilities can have versions:

```text
capability:
    elevator.motion
version:
    7
```

A Work can declare:

```text
requires >= 7
```

This avoids ambiguous compatibility.

# 43. Capability Attestation

For remote resources, NROS may need evidence that:

```text
resource R
```

really exposes:

```text
capability C
```

This can be established through attestation or trusted registration.

# 44. Resource Discovery

Discovery:

```text
Node
 ↓
announce Resource
 ↓
register capabilities
 ↓
health check
 ↓
resource available
```

should produce evidence.

# 45. Resource Loss

When a resource disappears:

```text
RESOURCE_LOST
```

dependent Work transitions:

```text
RUNNING
 ↓
RESOURCE_LOST
 ↓
PAUSED / FAILED / RECOVERING
```

depending on semantics.

# 46. Resource Migration

If Work is migratable:

```text
Executor A
   ↓
resource lost
   ↓
checkpoint
   ↓
Executor B
   ↓
resume
```

This requires portable execution state.

# 47. Resource Affinity

A Work can prefer:

```text
same node
same NUMA zone
same GPU
same network locality
```

to minimize latency.

# 48. Anti-Affinity

Conversely:

```text
not same host as Work B
```

can ensure redundancy.

Example:

```text
Replica A → Node 1
Replica B → Node 2
```

# 49. Resource Pools

Resources can belong to pools:

```text
GPU_POOL
├── GPU1
├── GPU2
└── GPU3
```

A Work can request:

```text
1 GPU
```

without naming a specific device.

The scheduler chooses the member.

# 50. Resource Classes vs Pools

A class answers:

```text
"What kind of resource?"
```

A pool answers:

```text
"Which resources are interchangeable for this Work?"
```

This distinction matters.

# 51. Resource Equivalence

Two resources may be equivalent for one Work but not another.

Example:

```text
GPU1 = GPU2
```

for:

```text
embedding
```

but:

```text
GPU1 ≠ GPU2
```

for:

```text
hardware-specific workload
```

Equivalence is therefore requirement-relative.

# 52. Resource Graph

NROS may model relationships:

```text
Node
 ├── CPU
 ├── Memory
 ├── GPU
 └── NIC
       ↓
    Network
```

This supports locality and failure-domain reasoning.

# 53. Hierarchical Resources

Example:

```text
Cluster
 └── Node
      └── GPU
           └── Memory
```

Allocating a child may consume parent capacity.

# 54. Parent-Child Accounting

If:

```text
GPU memory = 24GB
```

and:

```text
W1 → 12GB
W2 → 8GB
```

then:

```text
remaining = 4GB
```

The parent resource must reflect child allocations.

# 55. Resource Quotas

An Agent may have:

```text
CPU quota = 40%
GPU quota = 1
network = 20Mbps
```

Quota is different from capacity.

Capacity says:

```text
What exists?
```

Quota says:

```text
What may this principal consume?
```

# 56. Resource Budget

A budget constrains consumption over time:

```text
max_gpu_seconds = 10,000
```

or:

```text
max_energy = X
```

Budget exhaustion should be observable and enforceable.

# 57. Resource Priority

When scarce resources exist:

```text
W1
W2
W3
```

the scheduler combines:

```text
priority
deadline
fairness
resource policy
```

to select allocation order.

# 58. Resource Admission

A Work becomes admissible only if:

```text
required capability exists
AND
resource available
AND
quota permits
AND
budget permits
AND
policy permits
AND
lease can be established
```

# 59. Resource State Machine

A simplified lifecycle:

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
LEASED
    ↓
RELEASE_PENDING
    ↓
AVAILABLE
```

Failure paths:

```text
ANY
 ↓
DEGRADED
 ↓
FAILED
 ↓
RECOVERY
```

# 60. Resource Lease State

A lease can be:

```text
PENDING
ACTIVE
RENEWING
EXPIRED
REVOKED
LOST
FENCED
RELEASED
```

This makes distributed ownership observable.

# 61. Lease Expiration

Expiration should not rely only on the holder's local clock.

For distributed systems, NROS should define a time model and tolerate bounded clock uncertainty.

Otherwise:

```text
Node A:
    expires at 10:00

Node B:
    believes 09:59
```

can cause split-brain behavior.

# 62. Lease Renewal Safety

Renewal should require proof that:

```text
holder still owns lease
```

and:

```text
lease version still current
```

A stale holder cannot renew an old lease.

# 63. Lease Versioning

Use:

```text
LeaseId
LeaseVersion
FencingToken
```

together.

Example:

```text
L42 / version 7 / token 1009
```

This gives explicit ownership epochs.

# 64. Resource Authority

The resource manager becomes the authority for:

```text
allocation
lease
fencing
release
```

The scheduler requests allocation but should not forge ownership.

# 65. Scheduler–Resource Boundary

```text
Scheduler
    │
    │ allocate(requirements)
    ↓
Resource Manager
    │
    ├── capability match
    ├── policy
    ├── quota
    ├── lease
    └── fencing
    │
    ↓
Allocation
```

This keeps scheduling and resource authority separated.

# 66. Resource Manager–Policy Boundary

Resource allocation should evaluate:

```text
Can this Work use this resource?
```

Policy evaluates:

```text
Is this principal allowed to perform this operation?
```

Both are required.

# 67. Capability Security

A capability should be treated as an explicit authority-bearing object where appropriate.

Conceptually:

```text
CapabilityToken {
    subject
    operation
    resource
    constraints
    expiry
}
```

This prevents authority from being inferred merely from naming.

# 68. Resource Namespace

NROS should use explicit namespaces:

```text
resource://node-7/gpu/0
resource://node-7/device/elevator-3
resource://cluster/gpu-pool
```

This makes references portable and unambiguous.

# 69. Remote Resources

Remote resources require:

```text
network identity
capability discovery
lease authority
fencing
health
latency
failure semantics
```

Remote allocation is therefore fundamentally more complex than local allocation.

# 70. Network Partition

Suppose:

```text
Node A ↔ Node B
```

disconnects.

A must not assume:

```text
B is dead
```

and B must not assume:

```text
A released everything
```

The lease/fencing mechanism determines safe ownership.

# 71. Split-Brain Protection

The critical invariant:

> **At most one valid fencing epoch may control an exclusive resource.**

Even if multiple nodes believe they are active, the resource must reject stale epochs.

# 72. Resource Failure Domains

Resources can be tagged:

```text
zone
rack
host
power_domain
network_domain
physical_controller
```

The scheduler can then enforce:

```text
replicas must not share failure domain
```

# 73. Composite Capability

Some operations require multiple capabilities:

```text
elevator.motion
+
elevator.safety_override
```

The Work must explicitly declare both.

# 74. Capability Dependencies

Capabilities themselves may depend on other capabilities:

```text
motion.command
   ↓
controller.write
   ↓
controller.session
```

The runtime resolves the complete authority chain.

# 75. Capability Revocation Propagation

If:

```text
controller.session
```

is revoked, dependent capabilities become unusable:

```text
motion.command
   ↓
INVALID
```

The runtime must propagate this dependency.

# 76. Resource Context

An executor should receive a scoped resource context:

```text
ExecutionContext {
    work_id
    attempt_id
    resource_leases
    capabilities
    fencing_tokens
    deadlines
    cancellation
}
```

It should not receive unrestricted access to the entire runtime.

# 77. Resource Access

Execution should look conceptually like:

```text
Work
 ↓
Resource Lease
 ↓
Capability
 ↓
Scoped Operation
 ↓
Resource
```

not:

```text
Work
 ↓
global device handle
```

# 78. Resource Revocation During Execution

If a lease is revoked:

```text
Resource
 ↓
Lease revoked
 ↓
fencing epoch changes
 ↓
old operation rejected
```

The Work receives:

```text
RESOURCE_REVOKED
```

and enters recovery.

# 79. Resource Observability

Every allocation should generate events:

```text
RESOURCE_REQUESTED
RESOURCE_RESERVED
RESOURCE_ALLOCATED
LEASE_ISSUED
LEASE_RENEWED
RESOURCE_RELEASED
LEASE_EXPIRED
RESOURCE_REVOKED
RESOURCE_FENCED
```

This feeds the evidence/audit system.

# 80. Resource Metrics

Important metrics:

```text
allocation_latency
lease_duration
lease_expiration_count
resource_utilization
allocation_failures
contention
queue_time
preemption_count
resource_leaks
fencing_events
```

# 81. Resource Invariants

NROS should enforce:

```text
1. A resource has one authoritative ownership record.

2. An expired lease grants no authority.

3. A stale fencing token cannot control an exclusive resource.

4. Allocation must satisfy capability requirements.

5. Allocation must satisfy policy.

6. Quotas must be enforced independently of scheduler priority.

7. Failed allocation must not leak partial resources.

8. Resource release must be observable.

9. Unknown resource state must not be treated as healthy.

10. Resource loss must invalidate dependent execution appropriately.

11. Composite allocations must have transactional semantics where required.

12. Resource ownership must survive process crashes through durable coordination.

13. Capability possession does not imply unrestricted authorization.

14. Revocation must propagate to dependent operations.

15. Remote resources require explicit partition and fencing semantics.
```

# 82. Unified Resource Architecture

The complete relationship is:

```text
                        WORK
                          │
                          ↓
                  REQUIREMENTS
                          │
                          ↓
                    CAPABILITY
                       MATCH
                          │
                          ↓
                     RESOURCES
                          │
                 ┌────────┴────────┐
                 ↓                 ↓
              POLICY             QUOTA
                 │                 │
                 └────────┬────────┘
                          ↓
                     ADMISSION
                          ↓
                    RESERVATION
                          ↓
                      ALLOCATION
                          ↓
                        LEASE
                          ↓
                    FENCING TOKEN
                          ↓
                     EXECUTION
                          ↓
                      RELEASE
                          ↓
                      VERIFY
```

# 83. NROS Resource Model

We can now express the core model succinctly:

```text
Resource
    ↓
Capability
    ↓
Requirement
    ↓
Policy
    ↓
Allocation
    ↓
Lease
    ↓
Fencing
    ↓
Scoped Execution
```

This closes an important hole in the scheduler architecture.

# 84. The Deeper Principle

The runtime must never confuse:

```text
"I can reach it"
```

with:

```text
"I can use it"
```

or:

```text
"I used it"
```

with:

```text
"I still own it"
```

or:

```text
"I used to own it"
```

with:

```text
"I currently have authority."
```

Those distinctions are exactly what leases, capabilities, policy, and fencing provide.

# 85. Updated NROS Architecture

The system now has five major semantic layers:

```text
┌─────────────────────────────────────┐
│              AGENTS                 │
│       goals / reasoning / memory    │
├─────────────────────────────────────┤
│             WORK GRAPH              │
│       intent / dependencies         │
├─────────────────────────────────────┤
│             SCHEDULER               │
│ admission / ordering / dispatch     │
├─────────────────────────────────────┤
│         RESOURCE AUTHORITY          │
│ capability / allocation / leases    │
├─────────────────────────────────────┤
│          EXECUTION LAYER            │
│ devices / processes / network       │
└─────────────────────────────────────┘
```

Across all layers:

```text
State
Evidence
Policy
Identity
Recovery
```

provide the control plane.

# 86. Next Boundary — Identity & Authority

Resources have now exposed the next fundamental question.

If we have:

```text
Agent A
Work W42
Capability C7
Resource R9
Lease L12
```

we still need to answer:

> **Who exactly is A, who authorized W42, who issued C7, and why should R9 trust L12?**

That takes us to:

# Part LXXVI — NROS Identity, Principal, Authority, Capability & Trust Architecture

The next layer should formalize:

```text
Principal
Identity
Agent Identity
Runtime Identity
Node Identity
Resource Identity
Credential
Key
Attestation
Trust Domain
Authority
Delegation
Capability
Permission
Policy
Scope
Audience
Issuer
Subject
Proof
Signature
Revocation
Trust Chain
Identity Federation
Impersonation
Acting-As
On-Behalf-Of
```

The central invariant will be:

> **No execution authority should exist merely because an entity claims to possess it; authority must have an identifiable issuer, subject, scope, lifetime, and verification path.**

# NROS — Part LXXVI: Identity, Authority, Trust & Delegation Architecture

The resource layer established **what can be used**.

Now we establish **who is allowed to use it**.

The fundamental NROS relationship becomes:

```text
Principal
    ↓
Identity
    ↓
Authority
    ↓
Capability
    ↓
Resource
    ↓
Operation
```

The runtime must be able to answer, for every consequential operation:

> **Who requested this? Under whose authority? Who granted that authority? What exactly is authorized? For how long? Against which resource?**

# 1. Principal

A `Principal` is an entity that can be recognized by the runtime.

Possible principals:

```text
HUMAN
AGENT
SERVICE
RUNTIME
NODE
DEVICE
ORGANIZATION
WORKLOAD
```

A principal is not necessarily a user.

# 2. Identity

Identity establishes:

```text
Who is this principal?
```

Conceptually:

```text
Identity {
    principal_id
    identity_domain
    attributes
    credentials
    status
}
```

Example:

```text
agent://nros/agent-42
```

# 3. Stable Identity

Process IDs are not suitable as durable identity.

Bad:

```text
PID 1827
```

Better:

```text
agent://runtime/agent-42
```

A restart should not automatically create a completely unrelated identity unless policy requires it.

# 4. Identity vs Session

These are distinct:

```text
Identity
    ↓
Session
```

An identity may establish many sessions:

```text
Agent A
 ├── Session 1
 ├── Session 2
 └── Session 3
```

Session identity is therefore temporary.

# 5. Identity vs Credential

Identity says:

```text
"I am Agent A."
```

Credential provides evidence:

```text
"Here is proof that I control the identity."
```

Examples:

```text
key
certificate
signed token
attestation
hardware-backed credential
```

# 6. Authentication

Authentication establishes:

```text
claimed identity
        ↓
proof
        ↓
authenticated principal
```

NROS should not confuse authentication with authorization.

# 7. Authorization

Authorization asks:

```text
Can this principal perform this operation?
```

Therefore:

```text
Authentication
    ≠
Authorization
```

A perfectly authenticated Agent can still be denied.

# 8. Authority

Authority is the runtime-recognized ability to cause an operation.

Example:

```text
Agent A
    has authority
        to invoke
            elevator.stop
```

Authority should always have scope.

# 9. Authority Scope

An authority can be limited by:

```text
resource
operation
time
location
environment
Work
purpose
quantity
risk level
```

Example:

```text
Agent A
may:
    stop Elevator 7
until:
    10:30
for:
    Work W42
```

# 10. Least Authority

Agents should receive only what they need.

Instead of:

```text
Agent A → all devices
```

use:

```text
Agent A
  → elevator-7
  → stop
```

This sharply reduces blast radius.

# 11. Capability

The capability model from Part LXXV becomes an authority-bearing object.

Conceptually:

```text
Capability {
    issuer
    subject
    resource
    operation
    constraints
    issued_at
    expires_at
    identifier
}
```

# 12. Capability vs Permission

A permission is generally a policy decision:

```text
ALLOW operation X
```

A capability can be a concrete transferable/scoped representation of that authority:

```text
Capability C42
    grants X
    on R7
    to A
```

NROS can support both models.

# 13. Issuer

Every authority-bearing artifact needs an issuer:

```text
issuer = policy-authority-1
```

The issuer establishes:

> Who claims to have granted this authority?

# 14. Subject

The subject identifies the recipient:

```text
subject = agent-42
```

A capability issued to Agent A should not automatically be usable by Agent B.

# 15. Audience

For distributed systems, authority may also specify intended consumers:

```text
audience = resource-controller-7
```

This prevents a valid artifact from being replayed against an unintended service.

# 16. Delegation

An Agent may need to delegate limited authority:

```text
Human
 ↓
Agent A
 ↓
Agent B
```

But delegation must not silently increase authority.

# 17. Delegation Rule

A fundamental invariant:

```text
DelegatedAuthority
    ⊆
DelegatorAuthority
```

Agent B cannot receive:

```text
write_device
```

if Agent A only possesses:

```text
read_device
```

# 18. Attenuation

Delegation should normally reduce scope.

Example:

```text
A:
    control elevator fleet

delegates to B:
    control elevator-7
    stop only
    5 minutes
```

This is **attenuation**.

# 19. Delegation Chain

Authority can therefore be represented as:

```text
Human
 ↓
Authority A
 ↓
Agent A
 ↓
Delegation B
 ↓
Agent B
 ↓
Capability C
 ↓
Resource R
```

The complete chain should remain traceable.

# 20. On-Behalf-Of

An Agent may act:

```text
on behalf of Human H
```

This should not erase the Agent's identity.

Execution records should preserve:

```text
actor = Agent A
principal = Human H
```

or equivalent structured relationships.

# 21. Acting-As

Sometimes an Agent explicitly assumes a delegated role:

```text
Agent A
acting-as
MaintenanceOperator
```

The runtime should distinguish:

```text
actual identity
```

from:

```text
active authority role
```

# 22. Impersonation

Uncontrolled impersonation is dangerous.

NROS should distinguish:

```text
delegation
```

from:

```text
identity substitution
```

A delegated Agent should normally remain identifiable.

# 23. Principal Chain

An execution record might therefore contain:

```text
PrincipalChain {
    initiator
    delegator
    actor
    executor
    resource
}
```

Example:

```text
Human H
   ↓
Agent A
   ↓
Work W42
   ↓
Executor E7
   ↓
Controller C3
```

# 24. Authority Chain

Separately:

```text
Policy Authority
      ↓
Capability
      ↓
Lease
      ↓
Operation
```

These two chains should be correlatable.

# 25. Identity Domains

NROS may operate across domains:

```text
nros.local
cluster.example
device.fabric
external.service
```

Each domain can have its own identity authority.

# 26. Trust Domain

A trust domain establishes:

```text
which issuers
which credentials
which policies
```

are recognized.

Example:

```text
TrustDomain: production-cluster
```

may trust:

```text
cluster-ca
policy-authority
device-attestation-authority
```

# 27. Trust Is Not Binary

A principal may be:

```text
TRUSTED
LIMITED
UNKNOWN
REVOKED
QUARANTINED
```

This is more expressive than:

```text
trusted = true
```

# 28. Credential

Credentials prove control of an identity.

Possible forms:

```text
public/private key
certificate
signed assertion
hardware attestation
runtime-issued credential
```

Credentials should be scoped and revocable where appropriate.

# 29. Credential Rotation

Long-lived credentials increase risk.

NROS should support:

```text
credential A
    ↓
rotation
    ↓
credential B
```

while preserving identity continuity.

# 30. Key Rotation

A principal identity can remain:

```text
agent-42
```

while its cryptographic key changes.

Thus:

```text
Identity
    ≠
Cryptographic Key
```

# 31. Credential Revocation

A credential may become invalid because of:

```text
expiration
compromise
manual revocation
policy change
identity retirement
```

Revocation must propagate to dependent authority where necessary.

# 32. Capability Expiration

Capabilities should generally have bounded lifetimes for high-risk operations:

```text
issued_at
expires_at
```

Long-lived authority should require explicit policy justification.

# 33. Authority Epoch

For emergency revocation, NROS can maintain an authority epoch:

```text
epoch = 42
```

A capability created under:

```text
epoch = 41
```

may become invalid after a global security transition.

# 34. Policy Version

Authorization decisions should be associated with policy version:

```text
policy_version = 17
```

This allows auditing:

> Why was this operation allowed?

Answer:

```text
Policy v17
```

# 35. Authorization Decision

Conceptually:

```text
Decision {
    subject
    operation
    resource
    policy_version
    result
    constraints
    timestamp
}
```

Possible results:

```text
ALLOW
DENY
CONDITIONAL
UNKNOWN
```

# 36. UNKNOWN Authorization

Distributed authorization can encounter unavailable policy services.

The runtime should not automatically equate:

```text
policy unavailable
```

with:

```text
ALLOW
```

For critical operations:

```text
UNKNOWN → DENY
```

may be required.

For low-risk operations:

```text
UNKNOWN → LIMITED
```

might be acceptable.

Policy decides.

# 37. Authorization Context

A policy decision may depend on:

```text
principal
resource
operation
Work
environment
time
risk
resource state
location
delegation chain
```

Therefore authorization is contextual.

# 38. Policy Decision Point

NROS can separate:

```text
Policy Decision Point
```

from:

```text
Policy Enforcement Point
```

Architecture:

```text
Operation
   ↓
Enforcement Point
   ↓
Policy Decision
   ↓
ALLOW / DENY
   ↓
Execution
```

# 39. Policy Enforcement

The executor must not rely solely on the Agent's claim that authorization exists.

The enforcement boundary verifies it.

# 40. Resource-Level Enforcement

For high-value resources:

```text
Agent
 ↓
Runtime
 ↓
Resource Manager
 ↓
Device
```

multiple enforcement layers may exist.

Defense in depth is valuable.

# 41. Cryptographic Proof

Where distributed trust is required, authority artifacts can be signed:

```text
Issuer
   ↓
sign(Capability)
   ↓
Capability
   ↓
verify
```

The verifier checks:

```text
signature
issuer
subject
scope
expiration
revocation
audience
```

# 42. Proof of Possession

Merely presenting a public credential is not always enough.

A principal may need to prove:

```text
I actually control the private key
```

This prevents credential copying attacks.

# 43. Replay Protection

An old valid request should not automatically be reusable.

NROS can employ:

```text
nonce
request_id
timestamp
sequence_number
lease_version
fencing_token
```

depending on protocol.

# 44. Request Identity

Every consequential operation should have:

```text
request_id
```

Example:

```text
req-8f42
```

This enables:

```text
deduplication
audit
correlation
replay detection
```

# 45. Idempotency

If:

```text
request-42
```

arrives twice, the runtime should know whether the operation is:

```text
idempotent
```

or whether duplicate execution is dangerous.

# 46. Authority + Lease

The resource model and identity model now intersect:

```text
Authorization
       +
Capability
       +
Lease
       +
Fencing Token
       ↓
Authorized Resource Operation
```

All four answer different questions.

# 47. Capability Does Not Mean Current Ownership

An Agent can possess:

```text
capability = control motor
```

while not currently holding:

```text
lease = motor
```

Therefore:

```text
capability
    ≠
allocation
```

# 48. Lease Does Not Mean Unlimited Authority

An Agent can hold:

```text
lease = elevator-7
```

but only possess:

```text
read + stop
```

not:

```text
reconfigure controller
```

Therefore:

```text
lease
    ≠
authorization
```

# 49. Identity Does Not Mean Authority

This is perhaps the most important security invariant:

```text
I am Agent A
```

does not imply:

```text
I may perform operation X
```

Identity is merely one input into authorization.

# 50. Authority Does Not Mean Resource Availability

An Agent can be authorized to use a GPU while:

```text
GPU unavailable
```

Thus:

```text
authorization
    ≠
availability
```

# 51. Complete Admission Condition

A consequential operation can therefore require:

```text
AuthenticatedIdentity
AND
Authorization
AND
Capability
AND
ResourceAvailability
AND
Allocation
AND
ValidLease
AND
ValidFencingEpoch
AND
PolicyConstraints
```

Only then:

```text
EXECUTE
```

# 52. Authority Revocation

Suppose:

```text
Agent A
    capability C42
```

is revoked.

The system should propagate:

```text
Capability revoked
      ↓
authorization invalid
      ↓
future operations denied
      ↓
active leases evaluated
      ↓
dependent Work suspended/revoked
```

# 53. Emergency Revocation

Safety-critical systems need fast revocation.

An emergency authority can trigger:

```text
REVOKE_ALL
```

for a scope:

```text
resource
agent
capability class
trust domain
```

# 54. Quarantine

A suspicious principal can enter:

```text
QUARANTINED
```

meaning:

```text
existing authority frozen
new authority denied
resources reconciled
evidence retained
```

This is preferable to simply deleting the identity.

# 55. Identity Lifecycle

```text
PROVISIONED
    ↓
ACTIVE
    ↓
SUSPENDED
    ↓
REACTIVATED
    ↓
REVOKED
    ↓
RETIRED
```

Identity state must be explicit.

# 56. Agent Lifecycle

Agent identity can map to runtime lifecycle:

```text
CREATED
 ↓
AUTHENTICATED
 ↓
AUTHORIZED
 ↓
RUNNING
 ↓
SUSPENDED
 ↓
TERMINATED
```

Termination does not necessarily mean identity deletion.

# 57. Node Identity

Distributed NROS nodes require their own identity:

```text
node://cluster/node-7
```

The node may host:

```text
Agent A
Agent B
Agent C
```

Node identity and Agent identity remain distinct.

# 58. Runtime Identity

The runtime itself may be a principal:

```text
runtime://nros/runtime-3
```

This is important because some operations are performed by runtime infrastructure rather than by an Agent.

# 59. Executor Identity

An executor may be:

```text
process
thread
worker
remote service
device controller
```

Execution records should preserve which executor actually performed the operation.

# 60. Identity Correlation

An execution trace can therefore contain:

```text
initiator
   ↓
agent
   ↓
work
   ↓
runtime
   ↓
executor
   ↓
resource
```

This produces end-to-end accountability.

# 61. Delegation Limits

Delegation can be constrained by:

```text
maximum depth
maximum lifetime
resource scope
operation scope
quantity
risk level
non-delegable permissions
```

Example:

```text
Agent A
may delegate:
    read_sensor

may NOT delegate:
    emergency_override
```

# 62. Non-Delegable Authority

Certain authority should remain bound to its original principal.

Examples:

```text
root recovery authority
emergency stop authority
identity administration
trust-root management
```

Policy defines these.

# 63. Delegation Revocation

If Agent A delegates to B:

```text
A → B
```

and A loses the original authority:

```text
A revoked
```

then delegated authority should normally become invalid:

```text
B → invalid
```

unless policy explicitly permits independent authority.

# 64. Trust Chain

A distributed operation may validate:

```text
Root Trust
   ↓
Issuer
   ↓
Delegation
   ↓
Capability
   ↓
Lease
   ↓
Request
```

Each link must be valid.

# 65. Trust Failure

If any required link cannot be verified:

```text
trust chain incomplete
```

the operation enters:

```text
DENIED
```

or:

```text
AUTHORIZATION_UNKNOWN
```

according to policy.

# 66. Identity Discovery

NROS may support:

```text
discover principal
resolve identity
fetch credentials
validate issuer
resolve trust domain
```

but identity discovery should not itself grant authority.

# 67. Identity Metadata

Useful attributes:

```text
principal_id
type
domain
roles
status
created_at
expires_at
issuer
credential_refs
```

Attributes should be treated as policy inputs, not universal truth.

# 68. Role

A role groups permissions:

```text
MaintenanceOperator
```

might include:

```text
read_controller
stop_controller
inspect_faults
```

Roles simplify policy but should not replace explicit scope.

# 69. Role Assignment

Role assignment itself requires authority:

```text
Identity
 ↓
Role assignment
 ↓
Permissions
```

An Agent cannot grant itself a role.

# 70. Attribute-Based Authorization

Policies can also evaluate attributes:

```text
if:
    principal.type == AGENT
    AND
    resource.zone == zone-7
    AND
    work.risk <= medium
```

then:

```text
ALLOW
```

# 71. Role + Attribute Model

NROS can combine:

```text
RBAC
+
ABAC
+
Capability-based authority
```

rather than forcing one authorization paradigm onto every resource.

# 72. Risk Classification

Operations can be classified:

```text
LOW
MEDIUM
HIGH
CRITICAL
```

Higher-risk operations can require stronger evidence.

For example:

```text
LOW:
    cached metadata read

HIGH:
    actuator command

CRITICAL:
    safety override
```

# 73. Step-Up Authorization

A high-risk operation may require additional authorization:

```text
normal session
    ↓
critical operation
    ↓
step-up verification
    ↓
temporary capability
```

This avoids permanently granting maximum authority.

# 74. Purpose Binding

Authority can be bound to Work purpose:

```text
Capability:
    use camera
    only for Work W42
```

Then the capability cannot be reused for unrelated Work.

# 75. Context Binding

Authority can also bind to:

```text
environment
node
resource
time window
session
```

This further limits misuse.

# 76. Auditability

Every authorization decision should be traceable to:

```text
principal
operation
resource
policy version
authority artifact
delegation chain
decision
timestamp
```

This complements the state/evidence model.

# 77. Security Event Model

NROS should emit:

```text
IDENTITY_AUTHENTICATED
IDENTITY_SUSPENDED
CREDENTIAL_ROTATED
AUTHORIZATION_GRANTED
AUTHORIZATION_DENIED
CAPABILITY_ISSUED
CAPABILITY_REVOKED
DELEGATION_CREATED
DELEGATION_REVOKED
LEASE_GRANTED
LEASE_REVOKED
FENCING_EPOCH_CHANGED
TRUST_FAILURE
QUARANTINE_ENTERED
```

# 78. Security State Machine

At a high level:

```text
IDENTITY
   ↓
AUTHENTICATED
   ↓
AUTHORIZED
   ↓
CAPABILITY_GRANTED
   ↓
RESOURCE_ALLOCATED
   ↓
LEASE_ACTIVE
   ↓
OPERATION_ALLOWED
   ↓
EXECUTION
```

Any security failure can break the chain.

# 79. Unified Admission Pipeline

The complete NROS execution admission path is now:

```text
             REQUEST
                │
                ↓
           IDENTIFY
                │
                ↓
         AUTHENTICATE
                │
                ↓
        AUTHORIZE / POLICY
                │
                ↓
       VERIFY CAPABILITY
                │
                ↓
       MATCH RESOURCE
                │
                ↓
        CHECK QUOTA/BUDGET
                │
                ↓
          ALLOCATE
                │
                ↓
       ISSUE / VERIFY LEASE
                │
                ↓
       VALIDATE FENCING TOKEN
                │
                ↓
             EXECUTE
```

# 80. The Four Fundamental Proofs

Before consequential execution, NROS should conceptually establish four independent proofs:

```text
1. Identity Proof
   "Who is acting?"

2. Authority Proof
   "Why are they allowed?"

3. Resource Proof
   "What resource are they currently entitled to use?"

4. Freshness/Ownership Proof
   "Is that authority still valid right now?"
```

This is substantially stronger than a simple permission check.

# 81. NROS Trust Model

The resulting architecture:

```text
                 TRUST ROOTS
                      │
                      ↓
                 IDENTITIES
                      │
                      ↓
                 DELEGATION
                      │
                      ↓
                 AUTHORITY
                      │
                      ↓
                CAPABILITIES
                      │
                      ↓
                  POLICY
                      │
                      ↓
                 RESOURCES
                      │
                      ↓
                   LEASES
                      │
                      ↓
                  FENCING
                      │
                      ↓
                  EXECUTION
```

# 82. Core Security Invariants

NROS should enforce:

```text
1. Identity never implies authority.

2. Authentication never implies authorization.

3. Authorization never implies resource ownership.

4. Capability never implies current lease ownership.

5. Lease never expands authorization.

6. Delegation cannot increase authority.

7. Expired authority cannot authorize execution.

8. Revoked authority cannot authorize new operations.

9. Stale fencing tokens cannot control exclusive resources.

10. Every authority has an identifiable issuer.

11. Every authority has an identifiable subject.

12. High-risk authority must be explicitly scoped.

13. Delegation chains remain auditable.

14. Resource operations preserve actor identity.

15. Emergency revocation must invalidate affected authority.

16. Unknown trust state must not silently become trusted.

17. Security decisions must be reproducible from recorded evidence.

18. Runtime infrastructure cannot silently impersonate an Agent.

19. Agent memory cannot manufacture authority.

20. A valid identity chain is necessary but not sufficient for execution.
```

# 83. The NROS Semantic Chain

We can now extend the complete model considerably:

```text
REALITY
   ↓
OBSERVATION
   ↓
EVIDENCE
   ↓
STATE
   ↓
CONTEXT
   ↓
AGENT
   ↓
INTENT
   ↓
WORK
   ↓
IDENTITY
   ↓
AUTHORITY
   ↓
CAPABILITY
   ↓
RESOURCE
   ↓
LEASE
   ↓
FENCING
   ↓
SCHEDULER
   ↓
EXECUTOR
   ↓
OPERATION
   ↓
OBSERVATION
```

This forms the basis of a genuine **agent-native execution substrate** rather than merely an asynchronous task runtime.

# Part LXXVII — Protocol & Message Architecture

The next unresolved boundary is **communication**.

All of the previous layers eventually need to exchange messages:

```text
Agent
 ↔
Runtime
 ↔
Scheduler
 ↔
Resource Manager
 ↔
Policy Engine
 ↔
Executor
 ↔
Device
```

We therefore need a first-class NROS protocol model covering:

```text
Message
Envelope
Request
Response
Event
Command
Query
Notification
CorrelationId
RequestId
WorkId
AttemptId
PrincipalId
CapabilityId
ResourceId
LeaseId
FencingToken
Sequence
Epoch
Timestamp
Deadline
Priority
TraceId
Causality
Idempotency
Acknowledgement
Cancellation
Timeout
Retry
Backpressure
Flow Control
Ordering
Delivery Semantics
Serialization
Versioning
Compatibility
Schema Evolution
Error Model
Remote Execution
```

The central question becomes:

> **What is the smallest protocol envelope that can carry identity, authority, causality, deadlines, state versions, resource ownership, and execution semantics without turning every NROS message into an opaque blob?**

# NROS — Part LXXVII: Protocol, Message & Distributed Communication Architecture

The previous layer established **identity and authority**.

Now those concepts need a transportable representation.

NROS therefore needs a protocol model capable of carrying not merely:

```text
payload
```

but:

```text
intent
identity
authority
causality
state
resource context
timing
delivery semantics
```

The key principle is:

> **The payload describes what is being communicated; the envelope describes why it is trustworthy, what it belongs to, and how it must be processed.**

# 1. Message

The fundamental protocol object:

```text
Message {
    envelope
    payload
}
```

The payload should remain semantically independent from transport.

# 2. Envelope

Conceptually:

```text
Envelope {
    protocol_version
    message_id
    message_type
    sender
    recipient
    timestamp
    deadline
    correlation
    causality
    security
    delivery
}
```

The envelope is the protocol control plane.

# 3. Message Identity

Every message should have a globally unique:

```text
MessageId
```

Example:

```text
msg-01JXYZ...
```

This enables:

```text
deduplication
audit
correlation
replay detection
```

# 4. Request Identity

A request should additionally carry:

```text
RequestId
```

because one logical request can generate multiple messages:

```text
REQUEST
 ↓
RETRY
 ↓
RESPONSE
 ↓
FOLLOW-UP EVENT
```

All can reference the same `RequestId`.

# 5. Work Identity

Messages associated with Work should carry:

```text
WorkId
```

Example:

```text
work://agent-42/7c81
```

This lets the protocol connect transport events to scheduler state.

# 6. Attempt Identity

A retry is not necessarily the same execution attempt.

Therefore:

```text
WorkId
AttemptId
```

must remain distinct.

Example:

```text
Work W42
 ├── Attempt A1 → failed
 └── Attempt A2 → running
```

# 7. Correlation

A response should reference its originating request:

```text
Request:
    request_id = R42

Response:
    correlation_id = R42
```

This is more reliable than assuming request/response ordering.

# 8. Trace Identity

Distributed execution requires:

```text
TraceId
SpanId
ParentSpanId
```

Conceptually:

```text
Agent
 ↓
Scheduler
 ↓
Resource Manager
 ↓
Executor
 ↓
Device
```

all belong to one trace.

# 9. Causality

Correlation tells us:

> Which request does this response belong to?

Causality tells us:

> Which event caused this event?

Example:

```text
RESOURCE_LOST
      ↓
WORK_SUSPENDED
      ↓
CHECKPOINT_CREATED
      ↓
WORK_MIGRATED
```

Each transition should preserve causal relationships.

# 10. Causal Metadata

Conceptually:

```text
Causality {
    parent_event
    causal_chain
    sequence
}
```

This becomes valuable during recovery and audit.

# 11. Message Types

NROS should distinguish semantic message classes.

At minimum:

```text
COMMAND
QUERY
REQUEST
RESPONSE
EVENT
NOTIFICATION
ACKNOWLEDGEMENT
CANCELLATION
ERROR
```

# 12. Command

A command requests an action:

```text
COMMAND:
    resource.stop
```

Commands may have side effects.

Therefore they require stronger authorization semantics than ordinary observation.

# 13. Query

A query requests information:

```text
QUERY:
    resource.status
```

Queries should generally be side-effect free.

# 14. Event

An event reports something that happened:

```text
EVENT:
    RESOURCE_LOST
```

An event does not necessarily ask another component to act.

# 15. Notification

A notification is an informational message that may not require acknowledgment.

Example:

```text
NOTIFICATION:
    scheduler.load_changed
```

# 16. Response

Responses should explicitly identify outcome:

```text
Response {
    status
    result
    error
}
```

Possible status:

```text
SUCCESS
FAILURE
REJECTED
TIMEOUT
CANCELLED
UNKNOWN
PARTIAL
```

# 17. Acknowledgement

An acknowledgement means:

> The message was received/accepted.

It does **not** necessarily mean:

> The operation succeeded.

This distinction is critical.

```text
ACK
≠
SUCCESS
```

# 18. Delivery Semantics

NROS should explicitly model delivery guarantees.

Possible modes:

```text
AT_MOST_ONCE
AT_LEAST_ONCE
EFFECTIVELY_ONCE
```

Exactly-once execution is generally difficult across failure boundaries.

The protocol should therefore avoid pretending it exists when it does not.

# 19. At-Most-Once

```text
send
 ↓
if lost
 ↓
no retry
```

Advantages:

```text
low overhead
no duplicate execution
```

Disadvantage:

```text
messages may disappear
```

# 20. At-Least-Once

```text
send
 ↓
timeout
 ↓
retry
```

The receiver may receive:

```text
M
M
M
```

Therefore operations need idempotency or deduplication.

# 21. Idempotency Key

Commands with side effects should support:

```text
IdempotencyKey
```

Example:

```text
idempotency_key = stop:elevator-7:req-42
```

If the same command arrives twice:

```text
first → execute
second → return prior result
```

where supported.

# 22. Sequence Number

Ordered streams may use:

```text
SequenceNumber
```

Example:

```text
1
2
3
4
```

A receiver can detect:

```text
missing
duplicate
reordered
```

messages.

# 23. Sequence Scope

Sequence numbers need explicit scope.

For example:

```text
stream_id
sequence
```

rather than assuming one global sequence.

# 24. Epoch

A sequence number alone is insufficient after state reset.

Example:

```text
epoch 7:
    sequence 100

restart

epoch 8:
    sequence 1
```

The tuple:

```text
Epoch + Sequence
```

prevents ambiguity.

# 25. Ordering

NROS should distinguish:

```text
TOTAL_ORDER
PER_STREAM_ORDER
CAUSAL_ORDER
NO_ORDERING
```

Not every message requires global ordering.

# 26. Causal Ordering

For agent execution, causal ordering is often more useful than global ordering:

```text
A
 ↓
B
 ↓
C
```

while unrelated:

```text
X
Y
```

can proceed independently.

# 27. Deadline

Requests should carry:

```text
Deadline
```

Example:

```text
deadline = 2026-08-21T10:30:00Z
```

The receiver must not execute stale work indefinitely.

# 28. Timeout

A timeout is an execution policy.

A deadline is an absolute temporal constraint.

Prefer:

```text
deadline
```

for distributed propagation.

# 29. Remaining Budget

Each hop can derive:

```text
remaining_time
```

from the deadline.

Example:

```text
Agent:
    5 sec

Scheduler:
    4 sec

Resource manager:
    3 sec

Executor:
    2 sec
```

This prevents hidden downstream overruns.

# 30. Cancellation

Cancellation should be explicit:

```text
CANCEL {
    request_id
    reason
}
```

Cancellation is not the same as timeout.

# 31. Cancellation Propagation

If:

```text
Agent
 ↓
Work
 ↓
Subtask
```

is cancelled:

```text
Work cancellation
      ↓
subtask cancellation
      ↓
resource release
```

should occur according to policy.

# 32. Cancellation Authority

Not every principal may cancel every operation.

Cancellation itself is an authorized operation:

```text
can_cancel(work)
```

This prevents arbitrary agents from terminating other agents' critical work.

# 33. Priority

Messages may carry:

```text
Priority
```

but priority must not bypass security.

For example:

```text
priority = CRITICAL
```

does not mean:

```text
authorization = ALLOW
```

# 34. Priority Classes

Possible:

```text
BACKGROUND
NORMAL
HIGH
URGENT
CRITICAL
```

Policy defines which principals may submit each class.

# 35. Backpressure

A distributed runtime must handle overload.

If:

```text
producer rate > consumer capacity
```

the system needs:

```text
backpressure
queue limits
load shedding
admission control
```

# 36. Queue State

Queues should expose:

```text
depth
capacity
oldest_message
rejection_count
drop_count
processing_rate
```

This makes overload observable.

# 37. Flow Control

A receiver may advertise:

```text
available_capacity
```

The sender adjusts accordingly.

Conceptually:

```text
sender
  ↓
window = 100
  ↓
receiver
```

# 38. Backpressure Policy

Possible strategies:

```text
BLOCK
REJECT
DROP_OLDEST
DROP_NEWEST
SAMPLE
DEFER
SPILL_TO_DURABLE_QUEUE
```

Policy should depend on message class.

# 39. Critical Events

Critical events should not silently use:

```text
DROP_NEWEST
```

For example:

```text
SAFETY_FAULT
LEASE_REVOKED
RESOURCE_LOST
```

may require durable delivery.

# 40. Event Durability

NROS can classify events:

```text
EPHEMERAL
DURABLE
AUDIT_REQUIRED
```

An ephemeral metric update can disappear.

An authorization revocation event may require durable recording.

# 41. Message Persistence

Durable messages should survive:

```text
process crash
runtime restart
node restart
temporary network failure
```

where the delivery contract requires it.

# 42. Outbox

A component can use:

```text
state change
    ↓
transaction
    ├── state update
    └── outbox event
```

This prevents:

```text
state committed
but event lost
```

# 43. Inbox

Receivers can persist processed message IDs:

```text
Inbox {
    message_id
    status
    result_ref
}
```

This enables durable deduplication.

# 44. Exactly-Once Effect

Rather than promising exactly-once transport, NROS should target:

> **Exactly-once effect where the operation and persistence model permit it.**

That usually requires:

```text
idempotency
+
durable identity
+
transactional state
```

# 45. Protocol Error Model

Errors should be structured.

Example:

```text
Error {
    code
    category
    message
    retryability
    cause
    details
}
```

# 46. Error Categories

Possible categories:

```text
VALIDATION
AUTHENTICATION
AUTHORIZATION
RESOURCE
CONFLICT
TIMEOUT
CANCELLATION
NETWORK
PROTOCOL
STATE
INTERNAL
POLICY
```

# 47. Retryability

Every transient error should communicate whether retry is sensible:

```text
RETRYABLE
NOT_RETRYABLE
RETRY_AFTER
UNKNOWN
```

# 48. Retry-After

A receiver can provide:

```text
retry_after = 250ms
```

or:

```text
retry_after = timestamp
```

This helps prevent retry storms.

# 49. Exponential Backoff

Clients should normally avoid:

```text
retry
retry
retry
retry
```

at maximum speed.

Instead:

```text
100ms
200ms
400ms
800ms
...
```

with jitter.

# 50. Retry Budget

Retries themselves consume resources.

Therefore a request should carry:

```text
max_attempts
retry_budget
deadline
```

rather than retry indefinitely.

# 51. Retry Safety

A command should only be retried if:

```text
idempotent
```

or:

```text
deduplication guaranteed
```

or:

```text
operation semantics explicitly tolerate duplication
```

# 52. Serialization

The protocol needs a serialization layer independent of semantic definitions.

Possible implementation choices include:

```text
JSON
CBOR
MessagePack
Protobuf
custom binary
```

NROS should separate:

```text
semantic schema
```

from:

```text
wire encoding
```

# 53. Canonical Representation

Security-sensitive messages may require deterministic encoding for signing:

```text
Message
 ↓
canonical encoding
 ↓
signature
```

Two equivalent objects must not produce ambiguous signatures.

# 54. Schema

Every message type needs a versioned schema:

```text
schema_id
schema_version
```

Example:

```text
nros.work.execute
version = 3
```

# 55. Versioning

Protocol versioning must distinguish:

```text
wire protocol version
message schema version
semantic capability version
```

These are not necessarily the same.

# 56. Compatibility

NROS should define:

```text
BACKWARD_COMPATIBLE
FORWARD_COMPATIBLE
INCOMPATIBLE
```

for schema evolution.

# 57. Unknown Fields

Receivers should ideally preserve or safely ignore unknown fields when compatibility permits.

This allows:

```text
new sender
    ↓
older receiver
```

to continue operating safely.

# 58. Required vs Optional Fields

Schemas should clearly distinguish:

```text
required
optional
deprecated
extension
```

A security-critical field should never become accidentally optional.

# 59. Capability Negotiation

Before communication, peers may negotiate:

```text
supported protocol versions
message schemas
compression
serialization
features
security mechanisms
```

# 60. Handshake

A session handshake may establish:

```text
PeerIdentity
ProtocolVersion
Capabilities
SecurityContext
SessionId
```

After successful negotiation:

```text
READY
```

# 61. Session

A session is a communication context:

```text
Session {
    id
    peer
    protocol
    security
    sequence_state
    lifecycle
}
```

Sessions can expire independently of principal identity.

# 62. Connection vs Session

These must remain distinct.

```text
TCP connection
    ≠
NROS session
```

A session can potentially survive transport reconnection.

# 63. Transport Independence

NROS semantics should not depend directly on:

```text
TCP
QUIC
Unix socket
WebSocket
serial
shared memory
```

The transport adapter translates:

```text
NROS Message
        ↕
Transport
```

# 64. Local Fast Path

On one machine:

```text
Agent
 ↓
shared memory / channel
 ↓
Runtime
```

may avoid network serialization.

But semantic message identity should remain consistent.

# 65. Remote Path

Across nodes:

```text
Agent
 ↓
NROS envelope
 ↓
transport
 ↓
remote runtime
 ↓
executor
```

The same semantic protocol should apply.

# 66. Message Routing

A message may target:

```text
principal
node
service
resource
work
subscription
```

Therefore routing addresses should be typed.

# 67. Address

Conceptually:

```text
Address {
    domain
    namespace
    entity
}
```

Examples:

```text
agent://cluster/a42
service://cluster/scheduler
resource://node7/gpu0
work://agent42/w17
```

# 68. Broadcast

Broadcast should be explicit.

```text
EVENT
    audience = resource-watchers
```

rather than:

```text
recipient = "*"
```

which can accidentally create uncontrolled fan-out.

# 69. Subscription

Consumers can subscribe to:

```text
resource events
work events
agent events
security events
scheduler events
```

Subscriptions should themselves be authorized.

# 70. Event Filters

A subscription may specify:

```text
resource.class == GPU
AND
event.type == RESOURCE_LOST
```

This prevents unnecessary traffic.

# 71. Event Replay

Durable event streams may support:

```text
replay_from(sequence)
```

This is powerful for:

```text
recovery
debugging
state reconstruction
observability
```

# 72. Snapshot + Replay

A stateful component can recover through:

```text
snapshot
   +
event replay
```

rather than reconstructing everything from scratch.

This aligns directly with the NROS checkpoint model.

# 73. State Version

Messages concerning mutable state should carry:

```text
state_version
```

Example:

```text
expected_version = 42
```

Then the receiver can reject stale mutations.

# 74. Compare-and-Swap Semantics

For safe distributed state mutation:

```text
UPDATE if version == 42
```

otherwise:

```text
CONFLICT
```

This prevents lost updates.

# 75. Optimistic Concurrency

The protocol can therefore support:

```text
read version 42
 ↓
compute
 ↓
write expected version 42
 ↓
if changed:
    conflict
```

This is useful for scheduler state and resource metadata.

# 76. Command Preconditions

Commands may include:

```text
preconditions
```

Example:

```text
stop elevator
IF:
    controller_state == RUNNING
    lease_version == 7
    fencing_token == 1009
```

If conditions fail:

```text
PRECONDITION_FAILED
```

# 77. State Preconditions

This prevents stale agents from executing commands against changed state.

The command becomes:

```text
intent + expected state
```

rather than:

```text
blind mutation
```

# 78. Authority Preconditions

Similarly:

```text
required_capability = C42
required_lease = L17
required_epoch = 9
```

can accompany the operation.

# 79. Complete Command Envelope

A consequential command could therefore resemble:

```text
CommandEnvelope {
    protocol_version
    message_id
    request_id
    work_id
    attempt_id

    sender
    recipient

    trace_id
    causality

    timestamp
    deadline

    principal
    authority
    capability

    resource
    lease
    fencing_token

    state_version
    preconditions

    idempotency_key

    payload
}
```

This is the heart of the NROS protocol model.

# 80. Why the Envelope Is Rich

A simple RPC might say:

```text
stop(elevator7)
```

NROS needs to know:

```text
Who?
Why?
For which Work?
Under which authority?
Using which lease?
Against which resource epoch?
At what deadline?
Against which state version?
Is this retryable?
What caused it?
```

That is the difference between a generic RPC system and an **agent-native protocol**.

# 81. Message Lifecycle

A message moves through:

```text
CREATED
   ↓
AUTHORIZED
   ↓
SERIALIZED
   ↓
SENT
   ↓
RECEIVED
   ↓
VALIDATED
   ↓
DEDUPLICATED
   ↓
DISPATCHED
   ↓
PROCESSED
   ↓
ACKNOWLEDGED
   ↓
COMPLETED
```

Failure can occur at every stage.

# 82. Message Rejection

A receiver should reject malformed or unauthorized messages before dispatch.

Possible reasons:

```text
INVALID_SCHEMA
UNKNOWN_PROTOCOL
UNKNOWN_SENDER
INVALID_SIGNATURE
EXPIRED_DEADLINE
REPLAY_DETECTED
UNAUTHORIZED
INVALID_LEASE
STALE_FENCING_TOKEN
PRECONDITION_FAILED
UNSUPPORTED_VERSION
```

# 83. Security Ordering

Security validation should happen before expensive processing:

```text
parse minimal envelope
 ↓
authenticate
 ↓
authorize
 ↓
validate resource context
 ↓
validate payload
 ↓
execute
```

This reduces attack surface.

# 84. Payload Isolation

The payload should not be able to redefine envelope authority.

For example, a payload claiming:

```text
"role": "administrator"
```

must have no effect on actual authorization.

Authority comes from the trusted security context.

# 85. Message Size Limits

NROS should impose:

```text
max envelope size
max payload size
max nesting depth
max metadata size
```

to prevent resource exhaustion.

# 86. Compression

Large payloads may use compression:

```text
compression = zstd
```

but security metadata should remain independently verifiable.

# 87. Streaming

Some operations cannot fit naturally into one message:

```text
large model output
logs
file transfer
sensor stream
```

NROS should support:

```text
stream_id
chunk_sequence
stream_end
```

while preserving message semantics.

# 88. Stream Ownership

A stream should inherit:

```text
principal
authority
resource lease
deadline
```

from its initiating context unless explicitly changed.

# 89. Stream Backpressure

Streaming requires:

```text
window
acknowledgement
credit
cancel
resume
```

otherwise a fast producer can exhaust the receiver.

# 90. Partial Stream Failure

A stream can fail after receiving:

```text
chunks 1–500
```

The protocol may support:

```text
resume_from = 501
```

where semantics permit.

# 91. Heartbeats

Sessions may use:

```text
HEARTBEAT
```

to detect liveness.

But:

```text
heartbeat received
```

does not necessarily mean:

```text
agent healthy
```

It only establishes communication liveness.

# 92. Health vs Liveness

Separate:

```text
LIVENESS
```

from:

```text
READINESS
```

and:

```text
HEALTH
```

A node can be alive but unable to execute Work.

# 93. Protocol-Level Health

Peer state can be:

```text
CONNECTED
DEGRADED
UNRESPONSIVE
DISCONNECTED
QUARANTINED
```

This feeds scheduler decisions.

# 94. Message Observability

Every message should be traceable without logging sensitive payloads unnecessarily.

Useful fields:

```text
message_id
type
sender
recipient
trace_id
work_id
latency
status
error_code
```

# 95. Sensitive Payload Handling

The envelope may be logged while payload content is:

```text
redacted
hashed
encrypted
omitted
```

depending on sensitivity.

This fits the evidence/redaction architecture already established.

# 96. Protocol Evidence

For important operations, retain:

```text
request
authorization decision
resource allocation
lease
response
result
```

or cryptographic references to them.

This provides end-to-end provenance.

# 97. Protocol Invariants

NROS should enforce:

```text
1. Every message has a unique identity.

2. Every consequential request has a request identity.

3. Work-bound messages carry Work identity.

4. Retries can be distinguished from new attempts.

5. Security metadata cannot be overridden by payload data.

6. Expired messages cannot silently execute.

7. Duplicate delivery cannot silently duplicate unsafe effects.

8. ACK does not imply operation success.

9. Correlation and causality are distinct concepts.

10. Sequence numbers are scoped.

11. Sequence numbers require epochs when state can reset.

12. Cancellation is explicit.

13. Deadlines propagate downstream.

14. Retry behavior is bounded.

15. Errors explicitly declare retryability where possible.

16. Schema versions are explicit.

17. Transport is independent from protocol semantics.

18. Durable events have durable delivery semantics.

19. Critical state transitions are auditable.

20. Resource and authority context travels with consequential operations.
```

# 98. NROS Protocol Stack

We can now organize the communication architecture:

```text
┌───────────────────────────────────────┐
│           APPLICATION SEMANTICS      │
│ Agent / Work / Resource / Policy      │
├───────────────────────────────────────┤
│              MESSAGE MODEL           │
│ Command / Query / Event / Response    │
├───────────────────────────────────────┤
│               ENVELOPE               │
│ Identity / Authority / Causality      │
│ Deadline / Lease / Fencing / Trace   │
├───────────────────────────────────────┤
│              SESSION                 │
│ Handshake / Sequence / Flow Control  │
├───────────────────────────────────────┤
│             TRANSPORT                │
│ QUIC / TCP / IPC / SHM / Serial      │
└───────────────────────────────────────┘
```

# 99. NROS End-to-End Execution

The complete path now looks like:

```text
Agent
 │
 │ intent
 ↓
Work
 │
 │ identity
 ↓
Authorization
 │
 │ capability
 ↓
Scheduler
 │
 │ resource request
 ↓
Resource Manager
 │
 │ lease + fencing
 ↓
Protocol Envelope
 │
 │ authenticated message
 ↓
Executor
 │
 │ operation
 ↓
Resource
 │
 ↓
Result
 │
 ↓
Event
 │
 ↓
State / Evidence
```

This gives NROS a complete **control → execution → evidence loop**.

# 100. The Next Fundamental Problem

The protocol layer exposes another major issue.

Messages are not merely transmitted.

They cause **state transitions**.

If:

```text
REQUEST
```

arrives twice, or:

```text
RESPONSE
```

arrives after timeout, or:

```text
EVENT
```

arrives out of order, NROS needs deterministic semantics.

Therefore the next layer should formalize:

# Part LXXVIII — NROS State Machine, Event Log, Transactions & Deterministic Recovery

The next architecture will establish:

```text
State
State Version
Transition
Transition Preconditions
Event
Event Log
Command
Projection
Snapshot
Checkpoint
Transaction
Commit
Rollback
Compensation
Idempotency
Deduplication
Optimistic Concurrency
Conflict
Causal Ordering
Event Sourcing
State Reconstruction
Recovery
Replay
Determinism
Exactly-Once Effect
Consistency
Linearizability
Serializability
Isolation
Durability
Crash Recovery
Write-Ahead Log
Journal
Atomicity
Invariant Checking
```

The central invariant will be:

> **Every consequential NROS state transition must be explainable as an authorized command applied to a known prior state, producing a deterministic or explicitly modeled result and durable evidence of the transition.**

# NROS — Part LXXVIII: State Machines, Event Log, Transactions & Deterministic Recovery

The protocol layer established **how NROS communicates**.

Now we establish something even more fundamental:

> **How NROS changes state without losing causality, consistency, authority, or recoverability.**

An agentic runtime cannot safely operate on mutable state through arbitrary assignments:

```text
state.foo = value
```

Every consequential mutation needs semantics.

# 1. State

NROS state is the currently accepted representation of reality known to a component.

Conceptually:

```text
State {
    entity_id
    version
    data
    timestamp
}
```

But state alone is insufficient.

We also need to know:

```text
How did this state come to exist?
```

That is the role of the event/transition model.

# 2. State Version

Every mutable state domain should have a monotonically advancing version:

```text
v0
 ↓
v1
 ↓
v2
 ↓
v3
```

This enables:

```text
stale-write detection
optimistic concurrency
replay
audit
recovery
```

# 3. State Domain

NROS should not treat the entire runtime as one giant state object.

Instead:

```text
Runtime
├── AgentState
├── WorkState
├── SchedulerState
├── ResourceState
├── LeaseState
├── PolicyState
├── SessionState
└── EvidenceState
```

Each domain can have its own lifecycle and consistency requirements.

# 4. State Machine

Each important entity can be modeled as:

```text
State
 +
Command
 +
Preconditions
 ↓
Transition
 ↓
New State
 +
Event
```

Example:

```text
PENDING
   │
   │ AdmitWork
   ↓
READY
```

# 5. Explicit Transitions

Bad:

```text
work.status = RUNNING
```

Better:

```text
transition(
    WorkStarted {
        work_id
        executor
        attempt
    }
)
```

The transition explains **why** the state changed.

# 6. State Transition

Conceptually:

```text
Transition {
    transition_id
    entity
    from_version
    command
    actor
    authority
    preconditions
    result
    to_version
}
```

This becomes a first-class auditable object.

# 7. Transition Preconditions

Before mutation:

```text
expected_version == 42
AND
lease_valid
AND
capability_valid
AND
state == READY
```

Only then:

```text
READY → RUNNING
```

# 8. Compare-and-Swap

A basic mutation can be:

```text
UPDATE state
IF version == expected_version
```

If another actor already changed it:

```text
version != expected_version
```

the operation becomes:

```text
CONFLICT
```

rather than silently overwriting newer state.

# 9. Conflict

Conflict is not necessarily failure.

For agentic systems it may become:

```text
CONFLICT
 ↓
REPLAN
```

or:

```text
CONFLICT
 ↓
MERGE
```

or:

```text
CONFLICT
 ↓
ABORT
```

depending on policy.

# 10. Command

A command expresses:

> Please attempt this state transition.

Example:

```text
StartWork {
    work_id
    executor
}
```

The command is **not yet state**.

# 11. Event

An event expresses:

> This state transition actually happened.

Example:

```text
WorkStarted {
    work_id
    executor
    attempt_id
}
```

Therefore:

```text
Command
    ≠
Event
```

This distinction is essential.

# 12. Command Lifecycle

```text
COMMAND
   ↓
VALIDATE
   ↓
AUTHORIZE
   ↓
CHECK PRECONDITIONS
   ↓
EXECUTE TRANSITION
   ↓
COMMIT
   ↓
EVENT
```

A command that fails preconditions should not generate a false success event.

# 13. Event Log

NROS can maintain an append-oriented event log:

```text
EventLog
    1 → WorkCreated
    2 → WorkAdmitted
    3 → WorkStarted
    4 → ResourceAllocated
    5 → WorkCompleted
```

The log becomes durable history.

# 14. Event Identity

Every event needs:

```text
EventId
```

and ideally:

```text
stream_id
sequence
epoch
```

This allows precise ordering and deduplication.

# 15. Event Stream

Events should belong to streams:

```text
work/W42
agent/A7
resource/GPU2
scheduler/main
runtime/R3
```

Each stream can have independent sequence numbers.

# 16. Global Order

A global event sequence may exist:

```text
1
2
3
4
...
```

but NROS should not require every subsystem to depend on global ordering.

Global serialization is expensive and often unnecessary.

# 17. Causal Order

Prefer causal relationships where possible:

```text
ResourceLost
     ↓
WorkSuspended
     ↓
CheckpointCreated
```

rather than imposing:

```text
every event in the universe
```

into one total order.

# 18. Event Immutability

Once committed:

```text
Event #104
```

should not be silently edited.

Corrections should be represented as new events:

```text
Event #104
     ↓
Correction #109
```

This preserves provenance.

# 19. Append-Only Principle

The event log should conceptually follow:

```text
append
append
append
append
```

rather than:

```text
overwrite history
```

This is particularly important for security and recovery.

# 20. Event Payload

A durable event should contain enough context to explain itself:

```text
Event {
    event_id
    type
    entity
    stream
    sequence
    actor
    authority_ref
    timestamp
    causal_parent
    state_version
    payload
}
```

# 21. Event Provenance

An event should answer:

```text
Who caused it?
Under what authority?
What state did it modify?
What previous event caused it?
What version resulted?
```

This creates a complete provenance chain.

# 22. State Projection

The current state can be derived from events:

```text
Event 1
 ↓
Event 2
 ↓
Event 3
 ↓
Projection
 ↓
Current State
```

This is event-sourced state.

# 23. Projection

A projection transforms:

```text
Event Stream
```

into:

```text
Queryable State
```

For example:

```text
WorkCreated
WorkStarted
WorkCompleted
```

produces:

```text
Work {
    status = COMPLETED
}
```

# 24. Projection Failure

If a projection becomes corrupted:

```text
projection
    ↓
discard
    ↓
replay event stream
    ↓
rebuild
```

This is a powerful recovery property.

# 25. Snapshot

Full replay can become expensive.

NROS can periodically persist:

```text
Snapshot(version = 1000)
```

Then recovery becomes:

```text
Snapshot 1000
+
Events 1001..1050
=
Current State
```

# 26. Snapshot Invariant

A snapshot must identify:

```text
stream
version
schema
creation time
integrity metadata
```

Otherwise it cannot safely be replayed.

# 27. Checkpoint

A checkpoint is broader than a state snapshot.

It may contain:

```text
Agent state
Work state
Memory references
Resource leases
Execution position
Protocol session
Recovery metadata
```

A checkpoint represents a recoverable execution point.

# 28. Snapshot vs Checkpoint

Useful distinction:

```text
Snapshot:
    state representation

Checkpoint:
    recoverable execution context
```

A checkpoint may contain multiple snapshots plus execution metadata.

# 29. Write-Ahead Log

For transactional state:

```text
prepare mutation
     ↓
write journal
     ↓
fsync/commit durability
     ↓
apply state
```

The journal protects against process crashes.

# 30. Journal

The journal can contain:

```text
transaction_id
mutation
preconditions
actor
authority
timestamp
```

A recovery process can determine:

```text
committed?
aborted?
incomplete?
```

# 31. Transaction

A transaction groups state mutations:

```text
BEGIN
    update Work
    allocate Resource
    create Lease
COMMIT
```

Either the complete invariant is established or the operation is rolled back where atomicity is supported.

# 32. Atomicity Boundary

Not everything can be atomically committed.

For example:

```text
local database
+
physical motor
```

cannot necessarily participate in one ACID transaction.

NROS must therefore distinguish:

```text
local atomic transaction
```

from:

```text
distributed effect
```

# 33. Distributed Transactions

Two broad strategies:

```text
2-phase commit
```

or:

```text
Saga / compensation
```

For agentic distributed execution, compensation is often more practical for long-running operations.

# 34. Saga

Example:

```text
Allocate GPU
   ↓
Start model
   ↓
Open session
   ↓
Execute Work
```

If execution fails:

```text
close session
   ↓
stop model
   ↓
release GPU
```

These are compensating actions.

# 35. Compensation

A compensation is not necessarily the exact inverse.

For example:

```text
send physical command
```

may not be reversible.

The compensating action could instead be:

```text
enter safe state
```

This distinction matters enormously for physical systems.

# 36. Irreversible Operations

NROS should explicitly classify operations:

```text
REVERSIBLE
COMPENSATABLE
IRREVERSIBLE
UNKNOWN
```

Critical workflows must account for this.

# 37. Commit Point

A transition may contain a point after which rollback is impossible:

```text
prepare
   ↓
commit point
   ↓
external effect
```

The runtime should record this boundary.

# 38. External Side Effects

Consider:

```text
database commit
```

followed by:

```text
physical actuator
```

The event log must distinguish:

```text
state committed
```

from:

```text
external effect confirmed
```

They are not automatically equivalent.

# 39. Two-Phase Effect

A safer pattern:

```text
INTENT_RECORDED
      ↓
EFFECT_REQUESTED
      ↓
EFFECT_CONFIRMED
```

If confirmation never arrives:

```text
EFFECT_UNKNOWN
```

not:

```text
EFFECT_FAILED
```

# 40. UNKNOWN State

NROS should aggressively preserve uncertainty.

For example:

```text
command sent
connection lost
device response missing
```

Correct result:

```text
UNKNOWN
```

not:

```text
FAILED
```

This prevents unsafe duplicate execution.

# 41. Recovery From UNKNOWN

An unknown external effect may require:

```text
query device state
```

or:

```text
reconcile
```

before retry.

# 42. Reconciliation

Reconciliation compares:

```text
expected state
```

against:

```text
observed external state
```

Example:

```text
NROS:
    motor = STOPPED

Device:
    motor = RUNNING
```

Result:

```text
STATE_DIVERGENCE
```

# 43. Reconciliation Event

The system should record:

```text
RECONCILIATION_STARTED
RECONCILIATION_RESULT
STATE_DIVERGED
STATE_REPAIRED
```

This is valuable evidence.

# 44. Determinism

A state transition should ideally be:

```text
new_state =
    f(old_state, command, authoritative_context)
```

The function should not depend on hidden mutable state.

# 45. Nondeterminism

Some operations are inherently nondeterministic:

```text
network response
LLM generation
sensor reading
physical environment
```

NROS should not pretend they are deterministic.

Instead, nondeterministic inputs become explicit evidence:

```text
ObservedInput
```

which can be replayed or referenced.

# 46. Deterministic Core

The architecture should therefore maximize:

```text
deterministic state transition
```

while isolating:

```text
nondeterministic external observation
```

This is a major design principle for agentic runtimes.

# 47. Randomness

Random decisions should use explicit randomness sources:

```text
RandomSource
Seed
EntropyReference
```

when reproducibility matters.

# 48. Time

Time is another nondeterministic input.

State transitions should distinguish:

```text
event_time
observed_time
processing_time
logical_time
```

rather than using an implicit system clock everywhere.

# 49. Logical Clock

Distributed event streams can use:

```text
logical_clock
```

or:

```text
Lamport timestamp
```

to reason about causality.

NROS need not expose a specific algorithm at the semantic layer, but the protocol should support logical ordering metadata.

# 50. Hybrid Time

For distributed systems, NROS may eventually benefit from:

```text
physical timestamp
+
logical sequence
```

This gives both approximate wall-clock time and causal ordering.

# 51. Transition Determinism

Given:

```text
state S
command C
authority A
preconditions P
observations O
```

the transition should produce:

```text
S'
```

with explicit references to every nondeterministic input.

# 52. Invariant Checking

After every critical transition:

```text
state
 ↓
validate invariants
 ↓
commit
```

Example:

```text
lease.owner == allocation.owner
```

must remain true.

# 53. Cross-Domain Invariants

Important invariants can span subsystems:

```text
Work RUNNING
    ⇒
valid executor
    AND
required resource allocation
    AND
valid lease
    AND
authority
```

The state machine must be able to detect violations.

# 54. Invariant Violation

If:

```text
Work = RUNNING
```

but:

```text
Lease = EXPIRED
```

the runtime must not silently continue.

It enters:

```text
INVARIANT_VIOLATION
```

followed by recovery policy.

# 55. Recovery State Machine

A generic recovery flow:

```text
FAILURE_DETECTED
      ↓
CLASSIFY
      ↓
ISOLATE
      ↓
CHECKPOINT
      ↓
RECONCILE
      ↓
RECOVER / RETRY / COMPENSATE
      ↓
VERIFY
      ↓
RESUME / FAIL
```

# 56. Crash Recovery

After runtime crash:

```text
process restart
      ↓
load latest checkpoint
      ↓
recover journal
      ↓
replay events
      ↓
reconstruct state
      ↓
validate invariants
```

Only after validation:

```text
READY
```

# 57. Recovery Must Not Assume Success

If a crash occurred after:

```text
external command sent
```

but before:

```text
confirmation
```

recovery must retain:

```text
UNKNOWN
```

until reconciliation resolves it.

# 58. Duplicate Recovery

Recovery itself may be retried.

Therefore recovery commands need:

```text
idempotency
```

and:

```text
recovery_epoch
```

to prevent competing recovery processes.

# 59. Recovery Ownership

A recovery operation should itself have:

```text
principal
authority
lease
```

Recovery is not outside the security model.

# 60. Recovery Lease

A runtime may establish:

```text
RecoveryLease
```

to ensure only one coordinator performs recovery for a failed entity.

# 61. Recovery Epoch

Example:

```text
failure epoch = 7
```

If two recovery workers start:

```text
R1 → epoch 7
R2 → epoch 7
```

only one should acquire the authoritative recovery lease.

A new recovery attempt can become:

```text
epoch 8
```

after the previous attempt is invalidated.

# 62. Event Replay

Replay must distinguish:

```text
historical event
```

from:

```text
new side effect
```

Replaying an event should normally rebuild state, not execute its original external effect again.

# 63. Pure Event Application

Ideally:

```text
apply(event, state) → state'
```

is pure.

The dangerous operation is:

```text
re-execute(event)
```

which may duplicate external effects.

# 64. Event Sourcing Boundary

NROS can therefore divide:

```text
Event Store
```

from:

```text
Effect Executor
```

Architecture:

```text
Command
 ↓
Decision
 ↓
Event
 ↓
Event Store
 ↓
Projection
```

while external effects are separately coordinated.

# 65. Decision vs Event

An Agent may decide:

```text
"send STOP"
```

The runtime records:

```text
CommandAccepted
```

then:

```text
StopRequested
```

then eventually:

```text
StopConfirmed
```

These are different facts.

# 66. Fact vs Intention

This is a crucial semantic distinction:

```text
INTENT:
    "I want elevator stopped."

FACT:
    "Elevator is stopped."
```

NROS must never collapse the two.

# 67. Observation

The environment produces:

```text
Observation
```

Example:

```text
position = floor 7
motor = stopped
```

Observation becomes evidence for state reconciliation.

# 68. State Confidence

NROS may attach confidence/provenance to externally observed state:

```text
source
timestamp
freshness
quality
```

For critical state:

```text
stale observation
```

should not automatically satisfy a safety precondition.

# 69. Freshness

State can have:

```text
freshness_deadline
```

Example:

```text
sensor value valid for 100ms
```

After that:

```text
STALE
```

# 70. State Provenance

A state field should ideally be traceable to:

```text
event
observation
command result
reconciliation
```

This gives a provenance graph.

# 71. State Merkle / Integrity

For distributed or durable logs, NROS may optionally chain event integrity:

```text
Event N
  hash
   ↓
Event N+1
```

or use Merkle structures for large histories.

This can detect tampering.

# 72. Evidence Chain

A complete operation may become:

```text
Intent
 ↓
AuthorizedCommand
 ↓
ResourceAllocation
 ↓
LeaseIssued
 ↓
EffectRequested
 ↓
Observation
 ↓
StateTransition
 ↓
Result
```

This is the **NROS evidence chain**.

# 73. Audit Reconstruction

Given a final state:

```text
Work = COMPLETED
```

the system should be able to answer:

```text
Why?
```

by reconstructing:

```text
created
→ admitted
→ scheduled
→ authorized
→ allocated
→ executed
→ observed
→ completed
```

# 74. State Queries

NROS should support both:

```text
current state query
```

and:

```text
historical state query
```

Example:

```text
state at version 420
```

or:

```text
state at timestamp T
```

where event history permits it.

# 75. Temporal Queries

This enables questions like:

```text
Which resources did Work W42 hold at 10:31?
```

or:

```text
Which authority was active when command C17 executed?
```

This is extremely valuable for debugging and audits.

# 76. Event Retention

Not all events need infinite retention.

Retention classes could be:

```text
EPHEMERAL
SHORT_TERM
AUDIT
REGULATORY
IMMUTABLE
```

Policy determines storage duration.

# 77. Compaction

Event streams may become large.

NROS can compact through:

```text
snapshot
+
retained post-snapshot events
```

while preserving required audit history.

# 78. Compaction Safety

Compaction must never destroy information required for:

```text
security audit
recovery
causal reconstruction
compliance
```

unless policy explicitly permits it.

# 79. State Materialization

Frequently queried state should be materialized:

```text
Event Log
    ↓
Projection
    ↓
Indexed State
```

This gives efficient reads without sacrificing historical provenance.

# 80. Projection Consistency

A projection may temporarily lag:

```text
Event committed
    ↓
projection updating
```

Therefore the protocol should expose:

```text
projection_version
```

so callers can distinguish:

```text
current
```

from:

```text
eventually consistent
```

# 81. Read-Your-Writes

An Agent may need:

```text
write state
 ↓
immediately read same state
```

NROS can offer an explicit consistency mode:

```text
READ_YOUR_WRITES
```

rather than assuming all reads have strong consistency.

# 82. Consistency Classes

Possible semantic classes:

```text
EVENTUAL
CAUSAL
READ_YOUR_WRITES
MONOTONIC_READ
STRONG
```

The runtime chooses based on subsystem requirements.

# 83. Linearizability

Some operations require a single authoritative ordering:

```text
acquire exclusive lease
```

Such operations may require linearizable coordination.

# 84. Eventual Consistency

Other state can tolerate lag:

```text
metrics
telemetry
non-critical discovery
```

These can use eventual consistency.

# 85. Consistency Must Be Explicit

The API should not leave consistency semantics implicit.

For example:

```text
get_resource_state()
```

should specify whether it means:

```text
cached
fresh
strongly consistent
```

where relevant.

# 86. Transaction Context

A message can carry:

```text
transaction_id
```

allowing components to correlate related operations.

But transaction identity must not imply distributed atomicity unless the protocol actually provides it.

# 87. Nested Transactions

Agentic operations may create subtasks.

NROS should distinguish:

```text
parent transaction
```

from:

```text
child transaction
```

and define whether child commit depends on parent.

# 88. Compensation Tree

For long-running workflows:

```text
Transaction
├── Step A
│    └── compensation A'
├── Step B
│    └── compensation B'
└── Step C
     └── compensation C'
```

This forms a recovery graph.

# 89. Durable Intent

Before dangerous execution:

```text
intent
```

should be durable.

That means after restart NROS knows:

```text
what it intended to do
```

even if the outcome is unknown.

# 90. Intent vs Outcome

A robust record may therefore have:

```text
INTENT_RECORDED
EFFECT_STARTED
EFFECT_OBSERVED
OUTCOME_CONFIRMED
```

This avoids false assumptions during recovery.

# 91. Deterministic Scheduler State

Scheduler decisions should ideally be reconstructable from:

```text
state
events
policy version
resource observations
time model
random seed, if applicable
```

This allows post-failure explanation.

# 92. Decision Record

A scheduling decision can be recorded as:

```text
Decision {
    decision_id
    work_id
    candidates
    selected
    policy_version
    reasons
    inputs
}
```

This is crucial for explainability.

# 93. Agent Decisions

The same principle can apply to agent reasoning without requiring private chain-of-thought.

NROS can record:

```text
decision metadata
selected action
constraints
evidence references
policy references
```

rather than storing hidden reasoning.

# 94. Reproducibility

A scheduler decision should ideally be reproducible from:

```text
same state
+
same policy
+
same inputs
```

If not, the nondeterministic dependency must be explicit.

# 95. Failure Classification

NROS should distinguish:

```text
TRANSIENT
PERMANENT
DEPENDENCY
AUTHORITY
RESOURCE
CONFLICT
TIMEOUT
UNKNOWN
INVARIANT
```

This determines recovery strategy.

# 96. Retry Matrix

For example:

```text
TRANSIENT      → retry
TIMEOUT        → bounded retry
CONFLICT       → refresh/replan
AUTHORITY      → deny
RESOURCE       → wait/reallocate
PERMANENT      → fail
UNKNOWN        → reconcile
INVARIANT      → isolate + recover
```

# 97. Recovery Must Be Evidence-Driven

The runtime should not blindly:

```text
retry everything
```

Instead:

```text
failure
 ↓
evidence
 ↓
classification
 ↓
recovery policy
```

# 98. Recovery Invariants

During recovery:

```text
1. Never assume an external effect did not happen.

2. Never execute a non-idempotent command blindly after uncertainty.

3. Never reuse stale authority.

4. Never reuse stale leases.

5. Never discard causal evidence.

6. Never mark recovered state without verification.

7. Never allow two recovery owners simultaneously.
```

# 99. Unified State Architecture

The complete state subsystem becomes:

```text
                 COMMAND
                    │
                    ↓
              AUTHORIZATION
                    │
                    ↓
             PRECONDITIONS
                    │
                    ↓
               TRANSITION
                    │
          ┌─────────┴─────────┐
          ↓                   ↓
       EVENT              EFFECT
          │                   │
          ↓                   ↓
     EVENT STORE          EXTERNAL WORLD
          │                   │
          ↓                   ↓
      PROJECTION          OBSERVATION
          │                   │
          └─────────┬─────────┘
                    ↓
                RECONCILE
                    ↓
                  STATE
```

# 100. NROS State Invariants

The major invariants are now:

```text
1. State transitions have explicit causes.

2. State versions advance monotonically within their domain.

3. Stale mutations cannot overwrite newer state.

4. Events are immutable once committed.

5. Commands are not treated as facts.

6. Intent is not treated as outcome.

7. Unknown external effects remain UNKNOWN until reconciled.

8. Replay reconstructs state without accidentally repeating external effects.

9. Recovery is itself authorized and observable.

10. Checkpoints identify exact recoverable versions.

11. Snapshots are tied to event versions.

12. Critical transitions preserve provenance.

13. Invariant violations trigger explicit recovery behavior.

14. Transaction boundaries are explicit.

15. Distributed atomicity is never assumed without protocol support.

16. Irreversible operations have explicit semantics.

17. Compensation is modeled separately from rollback.

18. Consistency requirements are explicit.

19. Eventual consistency cannot masquerade as strong consistency.

20. The final state must be explainable from authoritative history.
```

# 101. The Full NROS Causal Loop

We can now connect nearly every layer developed so far:

```text
          OBSERVATION
               │
               ↓
            EVIDENCE
               │
               ↓
             STATE
               │
               ↓
            CONTEXT
               │
               ↓
             AGENT
               │
               ↓
             INTENT
               │
               ↓
              WORK
               │
               ↓
            IDENTITY
               │
               ↓
           AUTHORITY
               │
               ↓
          CAPABILITY
               │
               ↓
            RESOURCE
               │
               ↓
             LEASE
               │
               ↓
            FENCING
               │
               ↓
           SCHEDULER
               │
               ↓
            COMMAND
               │
               ↓
           PROTOCOL
               │
               ↓
          EXECUTION
               │
               ↓
           EXTERNAL
             EFFECT
               │
               ↓
          OBSERVATION
               │
               └──────────────→ ...
```

This is the fundamental **agentic-native feedback loop**.

# 102. The Important Architectural Consequence

NROS is no longer merely:

```text
Agent
    +
Scheduler
    +
Executor
```

It is becoming:

```text
┌─────────────────────────────────────────────┐
│                  AGENT LAYER                │
│ intent / planning / memory / reflection     │
├─────────────────────────────────────────────┤
│                 WORK LAYER                  │
│ tasks / dependencies / attempts / goals     │
├─────────────────────────────────────────────┤
│               AUTHORITY LAYER               │
│ identity / policy / capabilities / leases   │
├─────────────────────────────────────────────┤
│               SCHEDULING LAYER              │
│ admission / priorities / allocation         │
├─────────────────────────────────────────────┤
│                PROTOCOL LAYER               │
│ messages / events / streams / transport     │
├─────────────────────────────────────────────┤
│                 STATE LAYER                 │
│ transitions / events / snapshots / journal  │
├─────────────────────────────────────────────┤
│               EXECUTION LAYER               │
│ processes / devices / services / hardware   │
├─────────────────────────────────────────────┤
│              EVIDENCE LAYER                 │
│ provenance / audit / observations / traces  │
└─────────────────────────────────────────────┘
```

And the layers form a closed loop rather than a simple stack.

# 103. The Next Missing Primitive

There is now one particularly important abstraction that has not yet been fully formalized:

> **Time.**

Every NROS mechanism already depends on it:

```text
deadlines
leases
timeouts
TTL
scheduling
priorities
event ordering
retries
checkpoints
snapshots
expiration
budgets
health
recovery
```

Yet a distributed agentic runtime cannot simply call:

```text
SystemTime::now()
```

and assume that time is authoritative.

Therefore the next layer should formalize:

# Part LXXIX — NROS Temporal Model & Time Architecture

Including:

```text
Physical Time
Monotonic Time
Logical Time
Causal Time
Hybrid Time
Clock Sources
Clock Domains
Clock Skew
Clock Drift
Uncertainty
Deadlines
Timeouts
TTL
Leases
Expiration
Scheduling Time
Event Time
Processing Time
Observation Time
Validity Intervals
Temporal Constraints
Temporal Ordering
Temporal Consistency
Time Synchronization
Monotonicity
Clock Failure
Time Recovery
Historical Queries
Temporal Snapshots
Temporal State
Future Scheduling
Periodic Work
Timers
Cron-like Semantics
Temporal Triggers
```

The central invariant will be:

> **NROS must never use an ambiguous notion of time where correctness depends on distinguishing elapsed duration, wall-clock time, event ordering, and causal ordering.**

# NROS — Part LXXIX: Temporal Model & Time Architecture

The state machine established **what happened**.

The protocol established **how it is communicated**.

Now NROS must establish **when things happen**, without confusing several fundamentally different notions of time.

The central rule is:

> **Wall-clock time answers "when on the calendar?"; monotonic time answers "how much time elapsed?"; logical time answers "which event came before which?"; causal time answers "what could have caused what?"**

## 1. Time Is a Runtime Primitive

Time appears everywhere:

```text
deadline
timeout
lease
TTL
retry
backoff
scheduler
checkpoint
snapshot
heartbeat
expiration
event ordering
```

Therefore time cannot remain an incidental utility.

It belongs in the NROS semantic model.

# 2. The Four Primary Time Domains

NROS should distinguish at least:

```text
PhysicalTime
MonotonicTime
LogicalTime
CausalTime
```

They solve different problems.

# 3. Physical Time

Physical time represents wall-clock time:

```text
2026-08-21T06:42:13.482Z
```

Useful for:

```text
logs
audit
human interfaces
certificates
calendar scheduling
historical records
```

But physical clocks can jump.

# 4. Why Wall Clock Is Dangerous

Suppose:

```text
10:00:00
```

then NTP synchronization changes the clock:

```text
09:59:55
```

A calculation such as:

```text
deadline - now
```

can suddenly become incorrect.

Therefore wall-clock time should not normally drive elapsed-duration calculations.

# 5. Monotonic Time

Monotonic time measures elapsed duration.

Conceptually:

```text
t0 = 100.0
t1 = 103.7
```

Therefore:

```text
elapsed = 3.7s
```

even if the wall clock changes.

Use monotonic time for:

```text
timeouts
retry delays
execution budgets
lease durations
latency
heartbeats
performance measurement
```

# 6. Monotonicity Invariant

For a given monotonic clock:

```text
t2 >= t1
```

must always hold.

A backward jump is unacceptable for duration calculations.

# 7. Clock Domain

Different machines have different clocks:

```text
Node A
Node B
Node C
```

Therefore:

```text
monotonic(A)
```

cannot directly be compared with:

```text
monotonic(B)
```

without an explicit synchronization model.

# 8. Clock Source

NROS should abstract clock acquisition:

```text
Clock {
    physical()
    monotonic()
}
```

This allows testing with deterministic clocks.

# 9. Virtual Clock

Tests should be able to replace real time:

```text
VirtualClock
```

Example:

```text
clock.advance(5s)
```

instead of actually waiting five seconds.

This is essential for deterministic runtime tests.

# 10. Testability

Bad:

```text
sleep(30 seconds)
```

inside a state-machine test.

Better:

```text
clock.advance(30s)
```

and evaluate the transition immediately.

This can reduce hours of temporal testing to milliseconds.

# 11. Logical Time

Logical time does not represent physical time.

It represents ordering.

Example:

```text
A → B → C
```

means:

```text
A happened before B
B happened before C
```

without claiming:

```text
A occurred at 10:00:01
```

# 12. Lamport-Style Ordering

A logical counter can provide:

```text
1
2
3
4
```

and causal events advance accordingly.

Useful for:

```text
event ordering
distributed state
conflict resolution
causal analysis
```

# 13. Causal Time

Causal time answers:

> Could event B have been influenced by event A?

Example:

```text
ResourceLost
     ↓
WorkSuspended
```

The causal relationship is stronger than merely saying:

```text
timestamp(A) < timestamp(B)
```

# 14. Physical Order ≠ Causal Order

Consider:

```text
Node A:
    event E1 at 10:00:05

Node B:
    event E2 at 10:00:04
```

The timestamps alone do not establish causality.

NROS therefore needs causal metadata.

# 15. Event Timestamp

Every durable event should ideally contain:

```text
event_time
```

but also:

```text
logical_position
causal_parent
```

when relevant.

# 16. Time Tuple

A useful abstraction is:

```text
TemporalMetadata {
    wall_time
    monotonic_reference
    logical_time
    uncertainty
}
```

Not every field must be present on every wire message.

# 17. Uncertainty

Distributed physical time is never perfectly exact.

NROS should permit:

```text
timestamp = T
uncertainty = ±Δ
```

For example:

```text
T ± 4ms
```

This is more honest than pretending every node has identical time.

# 18. Clock Drift

Two clocks may initially agree:

```text
A = 10:00:00.000
B = 10:00:00.000
```

Later:

```text
A = 10:10:00.000
B = 10:10:00.127
```

The difference is clock drift/skew.

Critical temporal decisions must account for it.

# 19. Clock Skew

Define:

```text
skew(A,B)
```

as the estimated difference between clocks.

Policies can establish:

```text
maximum_allowed_skew
```

for operations that depend on wall-clock validity.

# 20. Time Synchronization

NROS does not need to mandate one synchronization mechanism.

The underlying system may use:

```text
NTP
PTP
GPS
hypervisor clock
platform synchronization
```

The temporal layer should consume the resulting clock quality.

# 21. Clock Quality

A clock source can expose:

```text
ClockQuality {
    synchronized
    estimated_error
    last_sync
    source
}
```

Then safety-critical decisions can reject poor-quality time.

# 22. Deadline

A deadline represents:

> The latest acceptable completion point.

Example:

```text
deadline = 10:30:00Z
```

But a distributed node should not simply compare remote wall time to local wall time without accounting for clock uncertainty.

# 23. Deadline Propagation

If:

```text
Agent
```

has:

```text
deadline = T
```

and sends work to:

```text
Scheduler
```

the scheduler inherits the same semantic deadline.

It should not silently reset:

```text
timeout = 30s
```

from the beginning.

# 24. Remaining Budget

A more robust representation is:

```text
deadline
+
remaining_budget
```

At each hop:

```text
remaining = deadline - current_time
```

subject to clock semantics.

# 25. Timeout

Timeout means:

> Stop waiting after this duration.

Example:

```text
timeout = 5s
```

This should use monotonic time.

# 26. Deadline vs Timeout

They are related but distinct:

```text
timeout:
    relative duration

deadline:
    absolute temporal boundary
```

Example:

```text
timeout = 5s
deadline = 10:30:05Z
```

# 27. Lease

A lease represents temporary authority.

Conceptually:

```text
Lease {
    owner
    resource
    issued_at
    expires_at
    epoch
}
```

But expiration requires careful clock semantics.

# 28. Lease Safety

A node must not continue treating a lease as valid indefinitely if:

```text
clock uncertainty
+
network partition
+
expiration
```

make its validity ambiguous.

# 29. Lease Renewal

A lease may transition:

```text
ACTIVE
 ↓
RENEWING
 ↓
ACTIVE
```

or:

```text
ACTIVE
 ↓
EXPIRED
```

Renewal itself must be authorized.

# 30. Fencing + Time

Time-based leases should never be the only protection against stale actors.

The stronger mechanism is:

```text
lease
+
fencing token
```

Therefore:

```text
old owner
    token = 7

new owner
    token = 8
```

The resource rejects token 7.

This protects against delayed messages.

# 31. Expiration

Anything temporal may expire:

```text
lease
credential
cache
observation
checkpoint
subscription
session
retry budget
```

NROS should represent expiration explicitly.

# 32. Validity Interval

Instead of a single expiration timestamp:

```text
valid_from
valid_until
```

can define:

```text
[Tstart, Tend)
```

This is particularly useful for scheduled work.

# 33. Half-Open Intervals

NROS should preferably use:

```text
[start, end)
```

rather than:

```text
[start, end]
```

because adjacent intervals then compose cleanly:

```text
[0,10)
[10,20)
```

without overlap.

# 34. Temporal Constraints

Work can specify:

```text
not_before
deadline
preferred_start
latest_start
duration_budget
```

Example:

```text
not_before = 10:00
deadline   = 10:30
```

# 35. Scheduling Window

A scheduler can calculate:

```text
window = [not_before, deadline]
```

and determine whether Work remains feasible.

# 36. Feasibility

If:

```text
current = 10:28
estimated_duration = 5m
deadline = 10:30
```

then:

```text
Work is not feasible
```

unless policy permits partial completion.

The scheduler should detect this **before admission** where possible.

# 37. Temporal Admission Control

Admission can therefore evaluate:

```text
resources
+
authority
+
dependencies
+
time feasibility
```

before Work enters execution.

# 38. Periodic Work

NROS may support:

```text
every 10s
```

but periodic execution needs a policy.

Two common semantics:

```text
fixed-rate
fixed-delay
```

# 39. Fixed Rate

```text
T0
T0 + 10s
T0 + 20s
T0 + 30s
```

Execution schedule is anchored to the original start.

# 40. Fixed Delay

```text
execute
 ↓
wait 10s
 ↓
execute
 ↓
wait 10s
```

The delay starts after completion.

# 41. Overrun

If a fixed-rate task takes longer than its period:

```text
period = 10s
duration = 17s
```

NROS must define whether to:

```text
skip
queue
overlap
coalesce
drop
degrade
```

# 42. No Implicit Overlap

For stateful Work, overlapping periodic executions should not happen accidentally.

A policy should explicitly permit:

```text
concurrency > 1
```

when safe.

# 43. Timer

A timer is a runtime object:

```text
Timer {
    id
    deadline
    callback/work
    policy
}
```

Timers should be integrated with scheduler state rather than being unmanaged background threads.

# 44. Timer Cancellation

Every timer should support explicit cancellation:

```text
cancel(timer_id)
```

and cancellation should be observable where necessary.

# 45. Timer Persistence

Not all timers should survive restart.

Classify:

```text
EPHEMERAL_TIMER
PERSISTENT_TIMER
RECOVERABLE_TIMER
```

# 46. Recoverable Timer

If a persistent Work has:

```text
deadline = T
```

and the runtime restarts at:

```text
T + 10s
```

the timer must not simply restart for another full duration.

Its original temporal semantics remain authoritative.

# 47. Temporal State Machine

Example:

```text
SCHEDULED
   │
   │ not_before reached
   ↓
ELIGIBLE
   │
   │ admitted
   ↓
READY
   │
   │ deadline approaching
   ↓
URGENT
   │
   ├── execute → RUNNING
   │
   └── deadline passed → EXPIRED
```

# 48. Expiration Is a State Transition

Do not implement expiration merely as:

```text
if now > deadline
```

hidden inside random code paths.

Instead:

```text
deadline reached
     ↓
EXPIRE transition
     ↓
event
```

This makes expiration auditable and replayable.

# 49. Lazy Expiration

Some state can be expired lazily:

```text
read
 ↓
detect expired
 ↓
transition
```

But safety-critical expiration may require proactive enforcement.

# 50. Proactive Expiration

For a resource lease:

```text
lease expires
 ↓
authority revoked
 ↓
fencing state updated
```

without waiting for another read.

# 51. Temporal Triggers

NROS should model temporal triggers as events:

```text
TIMER_FIRED
DEADLINE_REACHED
LEASE_EXPIRING
LEASE_EXPIRED
TTL_EXPIRED
SCHEDULE_WINDOW_OPENED
```

# 52. Time-Driven vs Event-Driven

NROS should avoid unnecessary polling:

```text
while true:
    check time
```

Prefer a timer/event mechanism:

```text
register deadline
 ↓
runtime wakes at relevant time
```

This improves efficiency.

# 53. Timer Wheel / Priority Queue

Implementation may use:

```text
timer wheel
binary heap
hierarchical timing wheel
priority queue
OS timer facilities
```

without exposing implementation details to the semantic layer.

# 54. Timer Determinism

When multiple timers become due simultaneously, ordering must be deterministic or explicitly unspecified.

Possible tie-breaker:

```text
deadline
+
priority
+
creation_sequence
```

# 55. Temporal Ordering of Events

If two events have identical timestamps:

```text
E1 @ T
E2 @ T
```

timestamp alone cannot order them.

Use:

```text
logical sequence
```

or:

```text
causal relation
```

# 56. Event Time vs Processing Time

Suppose a sensor reports:

```text
event_time = 10:00:00
```

but the runtime receives it at:

```text
processing_time = 10:00:03
```

Both timestamps matter.

# 57. Observation Time

For external state:

```text
observation_time
```

indicates when the physical state was measured.

This is different from:

```text
received_time
```

# 58. Freshness Calculation

A consumer may calculate:

```text
age =
processing_time - observation_time
```

subject to clock synchronization assumptions.

If:

```text
age > allowed_staleness
```

the observation becomes:

```text
STALE
```

# 59. Temporal Preconditions

Commands can require:

```text
observation_age < 100ms
```

For example:

```text
execute emergency action
IF sensor state is fresh
```

where policy requires fresh data.

# 60. Temporal Capability

Some capabilities may themselves be time-bounded:

```text
Capability {
    permission
    valid_from
    valid_until
}
```

Authorization must evaluate temporal validity.

# 61. Temporal Authority

Therefore:

```text
ALLOW
```

is not necessarily permanent.

It can mean:

```text
ALLOW during [T1,T2)
```

This is especially useful for leases and delegated authority.

# 62. Credential Expiration

Credentials should carry expiration semantics.

But credential expiration should not depend solely on an unreliable local wall clock.

NROS needs:

```text
clock quality
```

and potentially a trusted time source.

# 63. Temporal Revocation

A credential may be:

```text
valid until T
```

but revoked earlier:

```text
REVOKED @ T2
```

Revocation events therefore override scheduled expiration.

# 64. Temporal Precedence

For authority:

```text
ACTIVE
```

requires:

```text
not expired
AND
not revoked
AND
correct epoch
```

# 65. Clock Failure

What if:

```text
clock source unavailable
```

?

NROS should classify the clock as:

```text
DEGRADED
UNTRUSTED
UNAVAILABLE
```

rather than silently using questionable time.

# 66. Safety Policy Under Clock Failure

Different operations can have different fallback policies:

```text
non-critical telemetry:
    continue

new lease acquisition:
    deny

safety-critical action:
    enter defined safe mode

local monotonic timers:
    continue if valid
```

# 67. Monotonic Clock Failure

If the platform's monotonic clock cannot be trusted, elapsed-time guarantees become questionable.

The runtime should fail closed for operations whose safety depends on accurate duration.

# 68. Temporal Partition

A distributed system can experience:

```text
network partition
+
clock divergence
```

A lease holder might believe:

```text
lease valid
```

while the coordinator believes:

```text
lease expired
```

This is precisely why fencing is necessary.

# 69. Temporal Fencing

A stronger authority model becomes:

```text
Lease Epoch
+
Fencing Token
+
Temporal Validity
```

The resource validates all applicable dimensions.

# 70. Temporal Conflict

Suppose:

```text
Agent A:
    lease token 7

Agent B:
    lease token 8
```

Both claim authority.

The resource should accept only the highest valid fencing epoch according to the authority protocol.

# 71. Historical Queries

The temporal model enables:

```text
state_at(T)
```

and:

```text
events_between(T1,T2)
```

This should become a first-class API concept.

# 72. Temporal Reconstruction

Given:

```text
Snapshot @ 1000
```

and:

```text
events 1001..1200
```

NROS can reconstruct:

```text
state @ 1200
```

or intermediate states.

# 73. Temporal Snapshot

A snapshot should explicitly identify:

```text
state_version
event_position
logical_time
wall_time
schema_version
```

This creates a precise temporal boundary.

# 74. Future State

NROS may also model planned state:

```text
CURRENT
PLANNED
EXPECTED
OBSERVED
```

Example:

```text
current position = floor 3
planned position = floor 7
observed position = floor 5
```

This is useful for execution monitoring.

# 75. Temporal Divergence

If:

```text
expected = floor 7
observed = floor 5
```

at the expected completion time:

```text
TEMPORAL_DIVERGENCE
```

can trigger replanning.

# 76. Temporal Budgets

Work may have several budgets:

```text
total_budget
planning_budget
execution_budget
retry_budget
recovery_budget
```

These must not be confused.

# 77. Budget Propagation

Example:

```text
Work budget = 60s
```

Agent planning consumes:

```text
8s
```

Execution receives:

```text
52s
```

If recovery consumes:

```text
15s
```

the remaining budget becomes:

```text
37s
```

unless policy defines independent budgets.

# 78. Temporal Resource Reservation

Scheduler reservations can include:

```text
resource
start
end
priority
owner
```

Example:

```text
GPU0
[10:00, 10:30)
Work W42
```

# 79. Reservation Conflict

If another Work requests:

```text
GPU0
[10:20,10:40)
```

the scheduler detects overlap.

Possible policies:

```text
reject
delay
preempt
split
negotiate
```

# 80. Temporal Preemption

Preemption itself becomes a state transition:

```text
RUNNING
 ↓
PREEMPT_REQUESTED
 ↓
SUSPENDED
```

with explicit timestamps and authority.

# 81. Temporal Fairness

Scheduler fairness can be measured over time:

```text
CPU share
GPU share
queue wait time
deadline miss rate
```

This allows starvation detection.

# 82. Starvation

A Work that repeatedly loses scheduling opportunities may enter:

```text
STARVING
```

This can trigger priority escalation according to policy.

# 83. Aging

A scheduler may increase effective priority as wait time increases:

```text
effective_priority =
base_priority + aging(wait_duration)
```

The exact policy belongs to the scheduler, not the temporal substrate.

# 84. Temporal Dependencies

Work dependencies can specify:

```text
after(W1)
before(W3)
within(5m)
```

This creates a temporal DAG:

```text
W1
 ↓
W2
 ↓
W3
```

with temporal constraints on edges.

# 85. Temporal DAG Validation

A workflow containing:

```text
W1 after W2
W2 after W1
```

creates a cycle.

NROS should reject it before execution.

# 86. Temporal Deadlines in DAGs

A parent Work can have:

```text
deadline = T
```

while children have derived deadlines.

The scheduler must ensure child allocations do not make parent completion impossible.

# 87. Temporal Feasibility Analysis

A scheduler can estimate:

```text
critical path
+
resource availability
+
execution duration
```

and determine:

```text
feasible
infeasible
uncertain
```

before committing resources.

# 88. Uncertain Duration

Agentic Work may have unpredictable duration.

Represent:

```text
estimated_duration
confidence
minimum
maximum
```

instead of a fake precise number.

# 89. Temporal Risk

Example:

```text
estimated = 20s
p95 = 35s
deadline margin = 3s
```

The scheduler should recognize high temporal risk.

# 90. Temporal Policy

A policy can specify:

```text
minimum_deadline_margin
maximum_uncertainty
maximum_clock_error
maximum_observation_age
```

This makes temporal assumptions explicit.

# 91. Temporal Evidence

For critical actions, record:

```text
clock source
timestamp
clock uncertainty
deadline
observed state age
lease validity
```

This allows later verification.

# 92. Time in the Protocol Envelope

The protocol envelope now becomes richer:

```text
Envelope {
    message_id
    request_id
    work_id
    attempt_id

    sender
    recipient

    trace_id
    causality

    event_time
    deadline
    temporal_budget

    logical_time
    sequence
    epoch

    lease
    fencing_token

    payload
}
```

# 93. Temporal Invariants

NROS should enforce:

```text
1. Monotonic clocks never move backward.

2. Wall-clock timestamps are not treated as elapsed-time measurements.

3. Remote monotonic clocks are never directly compared without a defined model.

4. Deadlines propagate rather than silently resetting.

5. Timeouts use monotonic duration semantics.

6. Expiration is an explicit state transition where correctness requires it.

7. Lease expiration cannot be the sole stale-writer defense.

8. Fencing protects against delayed/stale actors.

9. Event timestamps do not alone establish causality.

10. Logical order is distinct from wall-clock order.

11. Observation time is distinct from processing time.

12. Stale observations cannot satisfy freshness-sensitive preconditions.

13. Clock uncertainty is represented when relevant.

14. Clock failure has an explicit policy.

15. Recovery preserves original temporal semantics.

16. Persistent timers do not restart their full duration after process restart.

17. Periodic execution semantics are explicit.

18. Temporal conflicts are deterministic or policy-defined.

19. Time-dependent decisions are reproducible when possible.

20. Temporal assumptions are observable and auditable.
```

# 94. Unified NROS Temporal Architecture

```text
                    TIME
                     │
        ┌────────────┼────────────┐
        ↓            ↓            ↓
   WALL CLOCK    MONOTONIC    LOGICAL/CAUSAL
        │            │            │
        ↓            ↓            ↓
    CALENDAR      DURATIONS      ORDER
        │            │            │
        └────────────┼────────────┘
                     ↓
              TEMPORAL MODEL
                     │
       ┌─────────────┼─────────────┐
       ↓             ↓             ↓
   DEADLINES       LEASES        TIMERS
       │             │             │
       ↓             ↓             ↓
  SCHEDULING     AUTHORITY      EVENTS
       │             │             │
       └─────────────┼─────────────┘
                     ↓
                  STATE
                     ↓
                 RECOVERY
```

# 95. NROS Is Now Becoming a Temporal Distributed Machine

At this point the architecture can be described as:

```text
Agentic
+
Distributed
+
Stateful
+
Temporal
+
Authorized
+
Event-driven
+
Recoverable
```

The runtime is no longer simply executing asynchronous tasks.

It is maintaining a **causally ordered, temporally constrained, authority-aware state machine**.

# 96. The Next Missing Foundation

One major dimension remains before the execution model is complete:

> **Resources and ownership.**

We already introduced:

```text
resource
lease
fencing
allocation
reservation
```

but these need a complete formal model.

The next section should therefore establish:

# Part LXXX — NROS Resource Model, Allocation, Leases, Capabilities & Fencing

Including:

```text
Resource Identity
Resource Types
Resource Classes
Resource Ownership
Resource State
Resource Capacity
Resource Dimensions
Resource Pools
Resource Discovery
Resource Registration
Resource Claims
Resource Reservations
Resource Allocation
Resource Binding
Resource Affinity
Resource Anti-Affinity
Resource Locality
Resource Quotas
Resource Limits
Resource Accounting
Resource Sharing
Exclusive Resources
Shared Resources
Preemptible Resources
Non-Preemptible Resources
Leases
Lease Epochs
Fencing Tokens
Delegation
Capability-Based Access
Resource Revocation
Resource Failure
Resource Loss
Resource Recovery
Resource Reconciliation
Resource Migration
Resource Fragmentation
Resource Scheduling
Resource Arbitration
Resource Fairness
Resource Isolation
Resource Security
```

The central invariant will be:

> **No agent may exercise authority over a resource merely because it once possessed a valid reference; resource authority must be explicitly bound to current ownership/allocation, lease validity, fencing state, and policy.**

# NROS — Part LXXX: Resource Model, Allocation, Leases, Capabilities & Fencing

The temporal model established **when authority and work are valid**.

Now we formalize **what the runtime controls**.

A resource in NROS is not merely:

```text
CPU
GPU
file
socket
device
process
```

It is an **authoritative runtime object with identity, state, capacity, ownership, allocation semantics, and lifecycle**.

The central principle is:

> **A reference to a resource is not ownership of that resource, and ownership is not necessarily permission to exercise every operation on it.**

# 1. Resource

Conceptually:

```text
Resource {
    resource_id
    resource_type
    state
    capacity
    availability
    capabilities
    policy
    owner
}
```

The resource identity must remain stable across ordinary state changes.

# 2. Resource Identity

Every managed resource needs:

```text
ResourceId
```

Examples:

```text
cpu/node-01
gpu/node-01/0
device/elevator/controller-7
pty/session-42
storage/volume-17
network/link-3
```

The identifier should not depend on a transient process.

# 3. Resource Type

The runtime needs a type system:

```text
ResourceType
```

Examples:

```text
COMPUTE
MEMORY
STORAGE
NETWORK
DEVICE
PROCESS
SESSION
PORT
FILE
SERVICE
ACCELERATOR
SECRET
```

# 4. Resource Class

Two resources may have the same broad type but different semantics.

Example:

```text
GPU
├── NVIDIA
├── AMD
└── integrated
```

Therefore:

```text
resource_type
resource_class
```

should be distinct.

# 5. Resource Metadata

A resource may expose:

```text
ResourceMetadata {
    manufacturer
    model
    architecture
    version
    location
    labels
    attributes
}
```

Metadata supports discovery and scheduling.

# 6. Resource State

Resource state should be explicit:

```text
DISCOVERED
REGISTERED
AVAILABLE
RESERVED
ALLOCATED
BUSY
DEGRADED
UNHEALTHY
LOST
REVOKED
RETIRED
```

Not every resource needs every state.

# 7. Resource Lifecycle

A generic lifecycle:

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
    ↓
AVAILABLE
```

Failure can branch:

```text
IN_USE
   ↓
DEGRADED
   ↓
LOST
```

# 8. Resource Availability

Availability is not binary.

A resource can be:

```text
AVAILABLE
AVAILABLE_WITH_LIMITS
DEGRADED
UNAVAILABLE
UNKNOWN
```

For example:

```text
GPU:
    available_memory = 2 GB
```

rather than simply:

```text
GPU = free
```

# 9. Capacity

A resource can expose one or more capacity dimensions:

```text
Capacity {
    cpu
    memory
    storage
    bandwidth
    concurrency
}
```

But different resource types have different dimensions.

# 10. Scalar Capacity

Example:

```text
CPU capacity = 8 cores
```

Allocation:

```text
Work A = 2
Work B = 4
```

remaining:

```text
2
```

# 11. Vector Capacity

Some resources need multiple dimensions:

```text
GPU {
    compute = 100
    memory = 24GB
    bandwidth = 900GB/s
}
```

A Work may constrain several dimensions simultaneously.

# 12. Capacity Is Not Permission

Having:

```text
available_memory = 8GB
```

does not mean an Agent may allocate it.

Authorization remains separate:

```text
capacity
+
authority
```

# 13. Resource Attributes

Scheduling may depend on:

```text
labels:
    architecture = arm64
    zone = tunis-a
    accelerator = npu
```

Attributes should be queryable without becoming hidden policy.

# 14. Resource Pool

Resources can be grouped:

```text
ResourcePool {
    pool_id
    members
    policy
    capacity
}
```

Example:

```text
gpu-pool
├── gpu0
├── gpu1
├── gpu2
└── gpu3
```

# 15. Dynamic Pools

Membership can change:

```text
gpu-pool
   ↓
gpu3 LOST
```

The pool's effective capacity changes accordingly.

# 16. Resource Discovery

NROS needs a discovery mechanism:

```text
discover()
```

which can produce:

```text
ResourceDiscovered
```

The discovery result should not automatically grant authority.

# 17. Discovery vs Registration

Important distinction:

```text
DISCOVERED
```

means:

> The runtime observed something.

```text
REGISTERED
```

means:

> The runtime accepted it into its authoritative resource registry.

# 18. Registration

Registration can require:

```text
identity verification
capability declaration
health verification
policy validation
```

before becoming:

```text
AVAILABLE
```

# 19. Resource Registry

Conceptually:

```text
ResourceRegistry
├── identity
├── metadata
├── state
├── capacity
├── health
├── allocation
└── authority
```

The registry is the authoritative catalog.

# 20. Resource Claim

An Agent should not directly mutate resource state.

It submits:

```text
ClaimResource {
    resource
    requirements
    duration
    purpose
}
```

# 21. Claim vs Allocation

A claim means:

> I request this resource.

Allocation means:

> The runtime has granted this resource.

Therefore:

```text
CLAIMED
≠
ALLOCATED
```

# 22. Reservation

A reservation is a temporal commitment:

```text
Reservation {
    resource
    owner
    interval
}
```

Example:

```text
GPU0
[12:00,12:30)
Work42
```

# 23. Reservation vs Allocation

Reservation:

```text
future right
```

Allocation:

```text
current binding
```

A reservation can become an allocation when its temporal window opens.

# 24. Resource Binding

A binding connects:

```text
Work
```

to:

```text
Resource
```

Example:

```text
Binding {
    work_id
    resource_id
    allocation_id
}
```

# 25. Allocation Identity

Every allocation should have its own stable identity:

```text
AllocationId
```

Why?

Because the same resource may be allocated repeatedly:

```text
GPU0
 ↓
Allocation A
 ↓
released
 ↓
Allocation B
```

`AllocationId` prevents confusing historical allocations.

# 26. Allocation Epoch

An allocation should also have an epoch:

```text
allocation_epoch
```

This protects against stale references.

# 27. Stale Reference

Suppose:

```text
GPU0
Allocation = A
```

is released.

Later:

```text
GPU0
Allocation = B
```

An old Agent still possesses:

```text
allocation_id = A
```

It must not be able to operate against allocation B.

# 28. Allocation Token

Operations can therefore carry:

```text
resource_id
allocation_id
epoch
fencing_token
```

The resource validates them.

# 29. Fencing

Fencing is the mechanism that prevents stale owners from acting.

Example:

```text
Owner A → token 10
Owner B → token 11
```

Once token 11 is authoritative:

```text
token 10 → REJECT
token 11 → ACCEPT
```

# 30. Why Leases Alone Are Insufficient

Suppose A's lease expires.

A does not know because:

```text
network partition
```

A continues sending commands.

The coordinator has already given the resource to B.

Without fencing:

```text
A and B
both act
```

With fencing:

```text
A token = 10 → rejected
B token = 11 → accepted
```

# 31. Fencing Token Properties

A fencing token should be:

```text
monotonically increasing
authoritative
allocation-specific
validated by the resource
```

# 32. Fencing Is Resource-Side

A coordinator saying:

```text
"your token is invalid"
```

is insufficient if the stale actor can directly reach the resource.

The **resource boundary** must enforce fencing whenever possible.

# 33. Capability

A capability represents authority to perform a defined operation.

Conceptually:

```text
Capability {
    subject
    resource
    operations
    constraints
    validity
}
```

# 34. Capability ≠ Resource Reference

A reference says:

```text
"This is resource R."
```

A capability says:

```text
"This actor may perform operation X on R under constraints Y."
```

# 35. Capability Scope

Example:

```text
Capability:
    resource = motor-7
    operation = READ_STATE
```

This does not automatically permit:

```text
WRITE_COMMAND
```

# 36. Least Authority

Capabilities should be minimal.

Prefer:

```text
READ_SENSOR_STATE
```

over:

```text
CONTROL_DEVICE
```

when only reading is required.

# 37. Capability Composition

A complex operation may require:

```text
Capability A
+
Capability B
+
Lease
+
Allocation
```

NROS should make these requirements explicit.

# 38. Capability Delegation

An Agent may delegate limited authority:

```text
Agent A
   ↓
delegate
   ↓
Agent B
```

The delegated capability should not automatically exceed A's authority.

# 39. Delegation Attenuation

If A has:

```text
READ + WRITE
```

A should be able to delegate:

```text
READ
```

but not:

```text
ADMIN
```

unless A itself possesses that authority.

# 40. Capability Expiration

Capabilities may have:

```text
valid_from
valid_until
```

and should integrate with the temporal model.

# 41. Capability Revocation

Capabilities may be revoked before expiration:

```text
Capability
    ↓
REVOKED
```

A revocation event should be observable.

# 42. Capability Epoch

For high-value resources:

```text
capability_epoch
```

can invalidate old delegated capabilities.

# 43. Resource Authorization Predicate

A resource operation can conceptually require:

```text
ALLOW =
    identity_valid
    AND capability_valid
    AND capability_allows(operation)
    AND allocation_valid
    AND lease_valid
    AND fencing_valid
    AND resource_available
    AND policy_allows
```

This becomes a central NROS invariant.

# 44. Resource Operation

The runtime should expose semantic operations:

```text
allocate
reserve
bind
release
renew
revoke
inspect
reconcile
```

rather than arbitrary mutation.

# 45. Resource Mutation

Bad:

```text
resource.owner = agent
```

Better:

```text
AllocateResource {
    resource_id
    work_id
    requirements
    duration
}
```

which passes through policy and scheduling.

# 46. Allocation Transaction

A successful allocation can generate:

```text
AllocationRequested
AllocationAuthorized
AllocationGranted
LeaseIssued
FencingEpochAdvanced
```

The resulting allocation is therefore auditable.

# 47. Allocation Failure

Possible outcomes:

```text
RESOURCE_BUSY
INSUFFICIENT_CAPACITY
UNAUTHORIZED
POLICY_DENIED
RESOURCE_UNHEALTHY
CONFLICT
DEADLINE_INFEASIBLE
RESOURCE_UNKNOWN
```

# 48. Resource Unknown

If the registry cannot establish resource state:

```text
resource = UNKNOWN
```

NROS should not optimistically allocate it for critical work.

# 49. Resource Health

Health can be modeled separately from availability:

```text
Health {
    status
    score
    last_observed
    diagnostics
}
```

Example:

```text
availability = AVAILABLE
health = DEGRADED
```

# 50. Health ≠ Availability

A resource can be:

```text
healthy + unavailable
```

or:

```text
degraded + available
```

depending on policy.

# 51. Resource Failure

A resource can transition:

```text
IN_USE
 ↓
FAILURE_DETECTED
 ↓
DEGRADED
 ↓
LOST
```

The allocations associated with it must then enter explicit failure states.

# 52. Allocation Failure Propagation

If:

```text
GPU0 → LOST
```

then:

```text
Work42 → RESOURCE_LOST
```

rather than:

```text
Work42 → COMPLETED
```

# 53. Work Recovery

The scheduler may then:

```text
checkpoint
 ↓
find replacement resource
 ↓
restore
 ↓
resume
```

if the Work is recoverable.

# 54. Resource Migration

Migration can be modeled:

```text
Allocation A
GPU0
   ↓
MigrationRequested
   ↓
GPU1
   ↓
MigrationConfirmed
```

The old allocation should not remain active indefinitely.

# 55. Migration Epoch

Migration should advance authority:

```text
GPU0 token = 20
GPU1 token = 21
```

so stale operations against the previous location can be rejected.

# 56. Resource Affinity

A Work may prefer:

```text
same node
same NUMA zone
same GPU
same network locality
```

This is:

```text
AFFINITY
```

# 57. Anti-Affinity

A Work may require:

```text
not same host as Work B
```

to improve fault tolerance.

This is:

```text
ANTI_AFFINITY
```

# 58. Locality

Resource placement may consider:

```text
network distance
memory locality
device proximity
latency
power domain
failure domain
```

# 59. Failure Domain

Resources can belong to:

```text
rack
zone
host
power domain
network segment
```

A resilient scheduler can distribute Work across independent failure domains.

# 60. Resource Constraints

A Work may specify:

```text
requires:
    architecture = arm64
    memory >= 4GB
    accelerator = npu
    zone != Z3
```

The scheduler matches requirements against resource attributes.

# 61. Resource Selector

Conceptually:

```text
Selector {
    required
    preferred
    forbidden
}
```

Example:

```text
required:
    accelerator=NPU

preferred:
    zone=A

forbidden:
    degraded=true
```

# 62. Quota

A principal can have:

```text
Quota {
    cpu
    memory
    gpu
    storage
    concurrency
}
```

Quota limits aggregate consumption.

# 63. Quota vs Capacity

Capacity:

```text
what physically exists
```

Quota:

```text
what an actor is allowed to consume
```

These must remain separate.

# 64. Budget vs Quota

Budget:

```text
how much consumption is permitted over a policy period
```

Quota:

```text
maximum simultaneous/aggregate allocation
```

They may interact but are not identical.

# 65. Resource Accounting

NROS should track:

```text
requested
reserved
allocated
consumed
released
```

This allows capacity accounting to be audited.

# 66. Double Allocation

A critical invariant:

```text
exclusive resource
```

must never have two active authoritative allocations.

Formally:

```text
active_allocations(resource) <= 1
```

for exclusive resources.

# 67. Shared Resources

For shared resources:

```text
active_allocations(resource) > 1
```

may be valid, but capacity constraints must still hold.

# 68. Overcommit

Some resources can support overcommit:

```text
CPU
memory
```

depending on platform semantics.

Overcommit must be explicit:

```text
overcommit_policy
```

not accidental.

# 69. Preemptible Resource

A resource may be marked:

```text
PREEMPTIBLE
```

meaning another Work can displace its allocation according to policy.

# 70. Non-Preemptible Resource

Examples:

```text
physical actuator
exclusive device
critical industrial controller
```

may require:

```text
PREEMPTION = FORBIDDEN
```

# 71. Preemption Protocol

A safe sequence:

```text
PreemptRequested
      ↓
WorkSuspensionRequested
      ↓
WorkSuspended
      ↓
AllocationRevoked
      ↓
FencingAdvanced
      ↓
ResourceReallocated
```

# 72. Immediate Preemption

Some resources cannot safely wait.

For those:

```text
emergency revoke
```

may be necessary.

But the resulting state must still be recorded.

# 73. Emergency Revocation

Example:

```text
Resource
 ↓
SAFETY_REVOCATION
 ↓
FENCE
 ↓
SAFE_STATE
```

This should be distinct from ordinary scheduling preemption.

# 74. Resource Isolation

Resource allocation should prevent unintended cross-tenant access.

Depending on resource:

```text
namespace
cgroup
sandbox
VM
process boundary
device ACL
capability
```

may provide isolation.

# 75. Resource Security Boundary

The resource adapter should enforce security wherever possible.

Architecture:

```text
Agent
 ↓
NROS authorization
 ↓
Resource adapter
 ↓
OS/device security
 ↓
Resource
```

Defense in depth is preferable.

# 76. Resource Adapter

Each resource type can implement:

```text
ResourceAdapter {
    inspect
    validate
    allocate
    release
    fence
    reconcile
}
```

This separates generic runtime semantics from platform-specific execution.

# 77. Adapter Failure

If the adapter cannot determine whether an operation succeeded:

```text
EFFECT_UNKNOWN
```

must propagate upward.

Do not silently report:

```text
RELEASED
```

when release status is uncertain.

# 78. Reconciliation

Resource reconciliation compares:

```text
Registry state
```

against:

```text
actual resource state
```

Example:

```text
Registry:
    GPU0 = ALLOCATED to W42

Host:
    GPU0 = free
```

This is:

```text
STATE_DIVERGENCE
```

# 79. Orphan Allocation

Opposite case:

```text
Registry:
    GPU0 = AVAILABLE

Host:
    GPU0 = occupied
```

This indicates an orphan or unmanaged allocation.

The runtime must not simply overwrite it.

# 80. Orphan Resolution

Policy may choose:

```text
adopt
terminate
quarantine
reconcile
manual intervention
```

depending on resource criticality.

# 81. Resource Quarantine

A suspicious resource can enter:

```text
QUARANTINED
```

meaning:

```text
discoverable
but not allocatable
```

until verification succeeds.

# 82. Resource Retirement

Retirement is different from failure:

```text
AVAILABLE
 ↓
RETIRE_REQUESTED
 ↓
DRAINING
 ↓
RETIRED
```

Existing allocations may be allowed to finish.

# 83. Draining

A draining resource:

```text
accepts no new allocations
```

but may continue serving existing ones.

This is essential for graceful maintenance.

# 84. Resource Drain

Sequence:

```text
DRAINING
 ↓
existing Work completes
 ↓
allocations released
 ↓
resource retired
```

# 85. Resource Re-registration

After restart:

```text
resource discovered
```

does not automatically mean:

```text
previous allocation restored
```

The runtime must reconcile allocation state.

# 86. Allocation Recovery

Persistent allocations require:

```text
allocation record
resource identity
work identity
epoch
fencing state
```

before recovery can safely proceed.

# 87. Resource Epoch

A resource itself may have:

```text
resource_epoch
```

which changes after major lifecycle transitions.

For example:

```text
boot 1 → epoch 1
device reset → epoch 2
replacement → epoch 3
```

This helps invalidate stale state.

# 88. Resource Identity Across Reboots

A physical device may preserve identity:

```text
device_serial
```

while runtime-specific state changes:

```text
resource_epoch
```

Therefore identity and lifecycle epoch should remain distinct.

# 89. Capability + Resource Epoch

A capability can bind to:

```text
resource_id
resource_epoch
```

so a capability from a previous device incarnation becomes invalid.

# 90. Lease + Allocation + Fencing

The complete authority chain becomes:

```text
Capability
    ↓
Allocation
    ↓
Lease
    ↓
Fencing Token
    ↓
Resource
```

Every layer answers a different question:

```text
Capability → may you?
Allocation → were you assigned it?
Lease      → is that assignment currently valid?
Fencing    → are you the newest authority?
Resource   → can the operation actually execute?
```

# 91. Resource Operation Predicate

The canonical authorization equation becomes approximately:

```text
PERMIT(operation) =
    identity_valid
 ∧  capability_valid
 ∧  capability_allows(operation)
 ∧  allocation_valid
 ∧  lease_valid
 ∧  fencing_valid
 ∧  resource_state_allows(operation)
 ∧  temporal_constraints_valid
 ∧  policy_allows(operation)
```

Failure of any mandatory predicate prevents execution.

# 92. Resource Events

The resource subsystem should emit events such as:

```text
ResourceDiscovered
ResourceRegistered
ResourceAvailable
ResourceReserved
ResourceAllocated
ResourceReleased
ResourceDegraded
ResourceLost
ResourceQuarantined
ResourceRetired
ResourceReconciled
```

Allocation events:

```text
AllocationRequested
AllocationGranted
AllocationRejected
AllocationRevoked
AllocationMigrated
```

Lease events:

```text
LeaseIssued
LeaseRenewed
LeaseExpired
LeaseRevoked
```

Fencing:

```text
FencingEpochAdvanced
StaleOperationRejected
```

# 93. Resource Evidence

A resource operation should be traceable:

```text
Request
 ↓
Authorization
 ↓
Allocation
 ↓
Lease
 ↓
Fencing
 ↓
Adapter
 ↓
External Effect
 ↓
Observation
```

This provides a complete resource provenance chain.

# 94. Resource Model — Core Invariants

```text
1. Resource identity is distinct from allocation identity.

2. A resource reference does not imply authority.

3. Discovery does not imply registration.

4. Registration does not imply allocation.

5. Allocation does not imply unrestricted operation.

6. Capability is distinct from ownership.

7. Ownership is distinct from lease validity.

8. Lease validity does not replace fencing.

9. Stale allocations cannot operate on newer allocations.

10. Exclusive resources cannot have multiple active authoritative owners.

11. Shared resources must respect capacity constraints.

12. Resource health and availability are separate concepts.

13. Unknown resource state cannot silently become AVAILABLE.

14. Lost resources invalidate or suspend dependent allocations according to policy.

15. Preemption is an explicit transition.

16. Resource migration advances authority.

17. Resource retirement prevents new allocations.

18. Reconciliation never silently overwrites contradictory external state.

19. External-effect uncertainty remains UNKNOWN until resolved.

20. Every critical allocation is auditable.
```

# 95. Unified Resource Architecture

```text
                    AGENT
                      │
                      ↓
                    WORK
                      │
                      ↓
                REQUIREMENTS
                      │
                      ↓
                 SCHEDULER
                      │
            ┌─────────┴─────────┐
            ↓                   ↓
       CAPABILITY            RESOURCE
            │                   │
            └────────┬──────────┘
                     ↓
                 ALLOCATION
                     │
                     ↓
                   LEASE
                     │
                     ↓
                  FENCING
                     │
                     ↓
               RESOURCE ADAPTER
                     │
                     ↓
              EXTERNAL RESOURCE
                     │
                     ↓
                OBSERVATION
                     │
                     ↓
                RECONCILIATION
                     │
                     ↓
                   STATE
```

# 96. Resource Lifecycle in One View

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
LEASED
    ↓
FENCED
    ↓
IN_USE
    │
    ├──────────────→ RELEASED → AVAILABLE
    │
    ├──────────────→ PREEMPTED → AVAILABLE
    │
    ├──────────────→ DEGRADED
    │                    ↓
    │                 RECOVERY
    │
    └──────────────→ LOST
                         ↓
                     RECONCILE
                         ↓
                 RECOVER / RETIRE
```

# 97. NROS Authority Stack

We can now make the ownership hierarchy precise:

```text
IDENTITY
   ↓
AUTHORITY
   ↓
CAPABILITY
   ↓
CLAIM
   ↓
RESERVATION
   ↓
ALLOCATION
   ↓
LEASE
   ↓
FENCING TOKEN
   ↓
RESOURCE OPERATION
```

This is much stronger than a simple:

```text
agent → resource
```

relationship.

# 98. The Next Architectural Layer

At this stage we have formalized:

```text
State
Events
Transactions
Recovery
Time
Resources
Allocation
Capabilities
Leases
Fencing
```

But the runtime still needs a formal mechanism for **how Work moves through the system**.

The next section should therefore define:

# Part LXXXI — NROS Work Model, Tasks, Attempts, DAGs, Dependencies & Execution Semantics

Including:

```text
Work Identity
Work Types
Task Identity
Task Graphs
DAGs
Dependencies
Prerequisites
Goals
Attempts
Retries
Execution Attempts
Attempt Identity
Attempt Epochs
Idempotency
Work Admission
Work Readiness
Work Blocking
Work Suspension
Work Resumption
Work Cancellation
Work Preemption
Work Completion
Partial Completion
Failure
Retry Policy
Backoff
Jitter
Dead-Letter Work
Compensation
Subtasks
Parent/Child Work
Work Delegation
Work Handoffs
Work Ownership
Work Priority
Work Fairness
Work Budgets
Work Deadlines
Work Checkpoints
Work Recovery
Work Reconciliation
Work Results
Work Evidence
```

The key invariant will be:

> **A Work item is a durable, identity-bearing unit of intent whose execution is represented by explicit attempts; an attempt may fail, be retried, suspended, or recovered without changing the identity or historical meaning of the Work itself.**
