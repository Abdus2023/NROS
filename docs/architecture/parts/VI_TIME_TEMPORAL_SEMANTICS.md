# Part VI — Time & Temporal Semantics

> **Series:** NROS Architecture Series  
> **Part:** VI  
> **Role:** Time, clocks, deadlines, temporal ordering, and replay semantics  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part V defined communication semantics. Part VI defines how NROS represents time and reasons about temporal behavior.

The central rule is:

> **Time is a typed runtime dependency. A timestamp has meaning only when its clock domain, resolution, ordering semantics, and validity are known.**

## 2. Why Time Must Be Explicit

Robotic systems simultaneously operate with several notions of time:

```text
Physical / wall time
Monotonic elapsed time
Logical application time
Simulation time
Distributed time
Replay time
```

Treating these as one clock creates ambiguity around deadlines, ordering, simulation, replay, and synchronization.

## 3. Clock Domains

NROS should distinguish clock domains explicitly.

### Wall / realtime clock

Represents civil or externally synchronized time.

Useful for:

- human-facing timestamps;
- logs;
- correlation with external systems;
- calendar-oriented events.

It may jump because of synchronization or clock correction.

### Monotonic clock

Represents elapsed time that should not move backwards during normal operation.

Useful for:

- deadlines;
- timeouts;
- durations;
- scheduling measurements;
- latency measurement.

### Logical clock

Represents application-defined progression of time.

Useful for:

- deterministic workflows;
- discrete-event systems;
- replay;
- causal reasoning.

### Simulation clock

Represents virtual time controlled by a simulator or test harness.

Useful for:

- simulation;
- accelerated execution;
- paused execution;
- deterministic testing.

## 4. Clock Interface

Conceptually:

```text
Clock
├── now()
├── elapsed_since()
├── sleep_until()
├── sleep_for()
└── domain()
```

A runtime component should not silently assume that `now()` means wall-clock time.

The clock domain must be known by the consumer.

## 5. Timestamp Model

A timestamp should be represented conceptually as:

```text
Timestamp
├── value
├── clock_domain
├── resolution
└── validity
```

Therefore:

```text
Timestamp(100 ms, monotonic)
```

is not automatically comparable with:

```text
Timestamp(100 ms, realtime)
```

without an explicit conversion or correlation model.

## 6. Duration

A duration represents elapsed time rather than a point on a clock.

```text
Duration
├── magnitude
└── resolution
```

Durations can generally be compared without requiring a common absolute clock, provided their units and representation are compatible.

## 7. Deadline

A deadline is a temporal constraint on an activation, request, or operation.

Conceptually:

```text
Deadline
├── timestamp
├── clock_domain
└── policy
```

A deadline may be:

```text
hard
soft
advisory
```

These terms require explicit operational definitions.

## 8. Deadline Semantics

The runtime should distinguish:

```text
Deadline configured
        ↓
Deadline observed
        ↓
Deadline enforced
        ↓
Deadline met / missed
```

A configured deadline is metadata.

A missed deadline is an observation.

An enforcement mechanism requires additional runtime/platform behavior.

## 9. Periodic Work

Periodic activation can be expressed as:

```text
period = P
next_release = previous_release + P
```

Two policies must be distinguished:

```text
Release based on scheduled timeline
Release based on completion time
```

They produce different behavior under execution overruns.

For real-time-oriented workloads, the choice must be explicit.

## 10. Jitter

Jitter describes variation in temporal behavior relative to an expected schedule.

For a release event:

```text
jitter = actual_release - expected_release
```

Jitter measurements must specify:

- clock domain;
- reference schedule;
- sampling interval;
- measurement overhead;
- environment.

A measured average jitter does not establish a worst-case jitter bound.

## 11. Latency

Latency is elapsed time between defined points.

For example:

```text
message sent
      ↓
message received
```

or:

```text
activation created
      ↓
activation completed
```

A latency claim is meaningless unless the start and end events are explicitly defined.

## 12. Temporal Ordering

Temporal ordering can mean different things:

```text
Physical timestamp ordering
Causal ordering
Sequence ordering
Scheduler ordering
Logical ordering
```

A timestamp alone does not establish causal ordering.

Similarly:

```text
A happened earlier in wall time
```

does not necessarily prove:

```text
A caused B
```

## 13. Sequence Numbers

For communication, sequence numbers can establish local ordering independent of clock precision.

```text
Message
├── sequence = N
└── timestamp = T
```

Sequence order and timestamp order should not be conflated.

## 14. Distributed Time

Across machines, clocks may differ.

NROS should therefore distinguish:

```text
Local monotonic time
Local wall time
Synchronized wall time
Distributed logical time
```

Clock synchronization can reduce disagreement but does not automatically create a perfectly shared clock.

Distributed temporal claims must state synchronization assumptions and observed error bounds.

## 15. Time Synchronization

A synchronization mechanism may establish an estimated relationship:

```text
Clock A ≈ Clock B + offset
```

Relevant properties include:

```text
offset
skew
drift
uncertainty
synchronization interval
```

A synchronized clock should therefore be treated as an estimate with an uncertainty model when precision matters.

## 16. Timers

A timer creates an activation based on a temporal condition.

Conceptually:

```text
Timer
├── clock domain
├── release time
├── period
├── tolerance
├── callback/activation
└── cancellation state
```

Timer cancellation must have explicit semantics.

```text
cancel requested
      ≠
callback cannot already be running
```

The runtime must define the race between expiration and cancellation.

## 17. Sleep

Sleeping is a scheduling request, not a guarantee that execution resumes at an exact instant.

```text
sleep_until(T)
```

means conceptually:

```text
Do not intentionally resume before T
```

subject to the guarantees of the underlying execution environment.

The actual resume time may be later.

## 18. Temporal Budgets

Execution can carry a temporal budget:

```text
Activation
├── release time
├── deadline
├── budget
└── completion time
```

The budget represents an execution constraint and should be distinguished from wall-clock elapsed time.

A budget violation is an observable event; it is not necessarily a scheduler failure.

## 19. Simulation Time

Simulation may control the clock independently from host time.

```text
Host time
    │
    ▼
Simulation engine
    │
    ▼
Simulation clock
    │
    ▼
NROS runtime
```

Simulation can therefore support:

- pause;
- resume;
- acceleration;
- deterministic stepping;
- time reset.

Components using simulation time must not accidentally use host wall time for the same logical operation.

## 20. Replay Time

Replay may reconstruct events according to recorded timestamps or logical sequence.

Possible policies include:

```text
Real-time replay
Accelerated replay
Step-by-step replay
Deterministic logical replay
```

Replay fidelity depends on what was recorded.

A timestamped event log cannot reconstruct information that was never captured.

## 21. Deterministic Time

Deterministic execution may require an injected clock:

```text
Runtime
   ↓
Clock interface
   ↓
Deterministic test clock
```

This permits tests such as:

```text
advance(10 ms)
assert deadline behavior
```

without depending on host scheduling.

However, deterministic logical time does not prove deterministic physical execution timing.

## 22. Time and Scheduling

The scheduler consumes temporal metadata:

```text
Activation
├── release
├── period
├── deadline
├── budget
└── priority
```

The scheduler can then select eligible work according to its policy.

The architecture intentionally separates:

```text
Temporal constraint
        ≠
Scheduling algorithm
        ≠
Physical timing guarantee
```

## 23. Time and Communication

Communication metadata may include:

```text
source timestamp
receive timestamp
sequence number
deadline
lifespan
```

These enable policies such as:

```text
Drop stale message
Reject expired request
Measure transport latency
Detect ordering anomalies
```

A stale message should be rejected according to explicit age/lifespan semantics rather than an arbitrary timestamp comparison.

## 24. Lifespan

A message or request may have a validity interval:

```text
created_at + lifespan = expiration
```

After expiration:

```text
deliver
reject
archive
```

must be defined by the communication contract.

## 25. Temporal State Machines

Lifecycle transitions may have temporal constraints.

Example:

```text
STARTING
   ↓
within 500 ms
   ↓
RUNNING
```

If the transition does not occur within the defined interval:

```text
DeadlineMissed
      ↓
Recovery / Fault policy
```

The timeout policy must identify the clock domain and the transition being measured.

## 26. Temporal Evidence

Timing claims should preserve raw enough evidence to support the conclusion.

A timing record may include:

```text
activation_id
clock_domain
start
end
duration
deadline
result
scheduler_context
platform
workload
```

Derived statistics such as average latency or percentile latency should not replace the underlying measurement context when stronger claims are made.

## 27. Verification Matrix

| Property | Verification question |
|---|---|
| Clock domain | Is every temporal operation using the intended clock? |
| Monotonicity | Does the selected clock avoid backward movement where required? |
| Deadline | Are deadline misses detected correctly? |
| Timer cancellation | Are expiration/cancellation races handled? |
| Periodicity | Does the release policy match its specification? |
| Jitter | Is the measurement methodology explicit? |
| Latency | Are start/end events unambiguous? |
| Ordering | Are sequence and timestamp semantics distinct? |
| Synchronization | Are clock offset/skew/uncertainty measured? |
| Simulation | Can runtime time be controlled independently of host time? |
| Replay | Is recorded information sufficient for the claimed fidelity? |
| Lifespan | Are expired messages/requests handled according to policy? |

## 28. What Part VI Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- hard real-time timing;
- bounded worst-case latency;
- bounded worst-case jitter;
- globally synchronized clocks;
- deterministic physical execution;
- deterministic distributed replay;
- hardware-level time guarantees.

Those require implementation, measurement, and—where appropriate—formal or physical validation evidence.

## 29. Transition to Part VII

Part VI defines temporal semantics.

Part VII should establish the **resource model**: CPU, memory, devices, network, storage, budgets, ownership, accounting, admission, and enforcement.

```text
Part V
Communication + transport
        ↓
Part VI
Time + temporal semantics
        ↓
Part VII
Resources + budgets
```

## Canonical rule

> **NROS treats time as a typed runtime dependency: deadlines, timers, ordering, replay, and scheduling semantics must identify their clock domain and must not be presented as physical timing guarantees without evidence.**
