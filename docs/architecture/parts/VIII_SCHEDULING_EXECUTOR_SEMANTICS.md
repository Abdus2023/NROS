# Part VIII — Scheduling & Executor Semantics

> **Series:** NROS Architecture Series  
> **Part:** VIII  
> **Role:** Activation scheduling, execution policy, concurrency, and executor boundary  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part VII defined resources and budgets. Part VIII defines how admitted work is selected for execution and how that selected work is mapped onto execution mechanisms.

The central rule is:

> **Scheduling decides eligibility and ordering; the executor performs work; neither statement alone establishes a real-time guarantee.**

## 2. Activation as Scheduling Unit

NROS scheduling operates on activations rather than vague application activity.

```text
Activation
├── identity
├── entity
├── generation
├── release
├── deadline
├── budget
├── priority
├── affinity
├── dependencies
└── execution class
```

An activation is eligible only when its lifecycle, dependency, resource, and policy conditions permit execution.

## 3. Scheduling Pipeline

The conceptual pipeline is:

```text
Created
   ↓
Admission
   ↓
Eligible
   ↓
Ready queue
   ↓
Scheduler
   ↓
Selected activation
   ↓
Executor
   ↓
Running
```

The scheduler should not execute application code itself.

## 4. Eligibility

An activation may be eligible when:

```text
lifecycle permits execution
AND dependencies are satisfied
AND required resources are available
AND release time has arrived
AND cancellation has not invalidated it
AND policy permits execution
```

Eligibility is distinct from selection.

```text
Eligible
   ≠
Selected
```

## 5. Priority

Priority is one possible scheduling dimension.

```text
priority
```

must have defined semantics such as:

- numerical ordering;
- class ordering;
- inheritance behavior;
- tie-breaking.

A priority value without a scheduling policy has no operational meaning.

## 6. Deadline Scheduling

Deadline-aware scheduling may use:

```text
absolute deadline
relative deadline
laxity
```

Example:

```text
earliest deadline first
```

is a scheduling policy, not a universal guarantee of deadline satisfaction.

## 7. Periodic Scheduling

Periodic activations contain temporal metadata:

```text
period
phase / initial release
deadline
budget
```

The scheduler must distinguish whether an overrun:

```text
creates a new activation
coalesces releases
skips releases
extends execution
```

The policy must be explicit.

## 8. Budget-Aware Scheduling

An activation may carry an execution budget.

```text
Activation
   ├── budget
   └── consumed
```

The scheduler/executor may react to budget exhaustion through:

```text
throttle
preempt
suspend
cancel
record violation
```

Actual enforcement depends on the executor and platform.

## 9. Scheduling Policies

NROS should permit replaceable scheduling policies, such as:

```text
FIFO
RoundRobin
PriorityBased
EDF
FixedPriority
FairShare
ReservationBased
Custom policy
```

No single policy should be encoded as the universal meaning of NROS scheduling.

## 10. Tie-Breaking

A deterministic scheduler requires explicit tie-breaking when multiple activations have equivalent priority/deadline metadata.

Possible keys:

```text
sequence number
release order
entity identity
activation identity
explicit fairness state
```

Unspecified tie-breaking should be treated as unspecified behavior, not assumed deterministic behavior.

## 11. Ready Queues

Ready queues represent work that is eligible but not currently executing.

Possible structures include:

```text
FIFO queue
priority queue
time-ordered queue
multi-level queue
per-core queue
work-stealing pool
```

The data structure is an implementation mechanism; the scheduling semantics are the contract.

## 12. Preemption

Preemption interrupts execution so another activation can run.

Possible levels include:

```text
none
cooperative
thread-level
process-level
hardware-assisted
```

Preemption requires platform support and may have non-zero latency.

```text
Preemptible
   ≠
Instantaneously preemptible
```

## 13. Cooperative Execution

In cooperative execution, a running task yields explicitly or reaches a defined scheduling point.

```text
RUNNING
   ↓
YIELD
   ↓
READY
```

A cooperative runtime must account for tasks that fail to yield.

## 14. Executor Boundary

The executor maps selected activations to execution mechanisms.

```text
Scheduler
   ↓
Executor
   ├── worker thread
   ├── async task
   ├── dedicated worker
   ├── event loop
   └── embedded execution loop
```

The executor is responsible for execution mechanics, not for redefining scheduling policy.

## 15. Worker Model

An executor may use:

```text
single worker
fixed worker pool
dynamic worker pool
per-entity worker
per-resource worker
```

The chosen model affects concurrency, isolation, overhead, and fairness.

## 16. Concurrency

Multiple activations may execute concurrently.

```text
A ──→ Worker 1
B ──→ Worker 2
C ──→ Worker 3
```

Concurrency does not imply parallelism:

```text
Concurrency
→ multiple activities in progress

Parallelism
→ simultaneous physical execution
```

## 17. Affinity

An activation may specify execution affinity:

```text
CPU core
NUMA domain
GPU
device
worker class
```

Affinity restricts placement but does not itself guarantee exclusive execution.

## 18. Blocking

Blocking work can reduce scheduler responsiveness.

Examples:

```text
I/O wait
lock wait
network wait
device wait
sleep
```

The runtime should distinguish:

```text
RUNNING
WAITING
BLOCKED
READY
```

where the execution model requires those distinctions.

## 19. Priority Inversion

Resource contention may cause high-priority work to wait behind lower-priority work.

```text
High
  ↓ waits
Low
  ↑ blocked by
Medium
```

Mitigations may include:

```text
priority inheritance
priority ceiling
resource partitioning
lock-free structures
priority-aware admission
```

A mitigation mechanism does not automatically prove bounded latency.

## 20. Starvation

Starvation occurs when eligible work is repeatedly denied execution.

Possible causes:

```text
strict priority
unbounded higher-priority load
poor fairness policy
resource contention
incorrect queue management
```

Scheduling policies should define whether starvation is permitted, bounded, or prevented.

## 21. Fairness

Fairness can mean different things:

```text
CPU-share fairness
queue fairness
deadline fairness
resource fairness
entity fairness
```

The architecture must define the relevant fairness property rather than making an undifferentiated claim of "fair scheduling."

## 22. Cancellation

Cancellation interacts with scheduling at multiple stages:

```text
QUEUED
   ↓ cancel
REMOVED / CANCELLED

RUNNING
   ↓ cancel request
COOPERATIVE STOP / PREEMPT / DEFERRED STOP
```

A cancellation request does not prove that application execution has already stopped.

## 23. Overrun

An activation can exceed its temporal or resource budget.

```text
START
  ↓
EXECUTE
  ↓
BUDGET EXCEEDED
  ↓
Policy
```

Possible policies:

```text
continue
throttle
preempt
cancel
escalate
record violation
```

Overrun handling must be consistent with lifecycle and resource semantics.

## 24. Scheduling Context

A scheduler decision may depend on:

```text
current time
ready activations
priorities
deadlines
budgets
resource state
affinity
dependencies
execution class
fairness state
```

For deterministic scheduling, the relevant inputs and tie-breaking rules must be observable or reproducible.

## 25. Scheduler State

The scheduler may maintain state such as:

```text
ready queues
running set
blocked set
reservation state
fairness counters
budget state
priority state
```

This state is part of the scheduler implementation and should have explicit ownership.

## 26. Scheduling Points

A scheduling point is a moment at which the runtime may reconsider execution order.

Examples:

```text
activation completion
yield
block
unblock
higher-priority release
deadline event
budget exhaustion
resource availability
cancellation
```

The implementation should document which events cause rescheduling.

## 27. Admission vs Scheduling

The distinction remains fundamental:

```text
Admission
→ may this work enter the execution domain?

Scheduling
→ which eligible work executes next?

Execution
→ how does the selected work actually run?
```

Combining these concepts makes verification difficult and hides resource failures.

## 28. Scheduling Guarantees

Scheduling guarantees must state their assumptions.

Examples:

```text
No starvation under defined load
Bounded queueing under defined conditions
Deadline feasibility under specified workload
Maximum priority inversion under specified locking policy
```

A scheduling algorithm name alone is insufficient evidence for any of these claims.

## 29. Real-Time Boundary

NROS may support real-time-oriented scheduling semantics without claiming hard real-time behavior by default.

```text
Real-time policy
       ≠
Real-time operating system
       ≠
Hard real-time guarantee
```

A hard timing claim requires platform, workload, measurement, and analysis evidence.

## 30. Determinism

Deterministic scheduling requires control over relevant nondeterministic inputs.

Potential sources of nondeterminism include:

```text
thread races
queue insertion order
clock variation
OS scheduling
interrupt timing
resource availability
network arrival order
randomized work stealing
```

A deterministic policy cannot compensate for uncontrolled execution nondeterminism by itself.

## 31. Replay

A scheduler may support replay if sufficient scheduling evidence is recorded.

Possible evidence includes:

```text
activation releases
scheduler decisions
tie-break decisions
preemption points
resource state
clock values
completion events
```

Recording only final outcomes is generally insufficient to reconstruct scheduler decisions.

## 32. Faults

Scheduling/executor faults may include:

```text
WorkerFailure
QueueCorruption
ExecutionFailure
BudgetViolation
DeadlineMiss
StarvationDetected
ExecutorUnavailable
AffinityFailure
```

The runtime should propagate these through the lifecycle/fault model rather than inventing a second incompatible fault system.

## 33. Observability

Scheduling should expose observations where required:

```text
ActivationQueued
ActivationSelected
ActivationStarted
ActivationYielded
ActivationBlocked
ActivationResumed
ActivationPreempted
ActivationCompleted
DeadlineMissed
BudgetExceeded
```

These events support debugging, verification, profiling, and replay-oriented analysis.

## 34. Verification Matrix

| Property | Verification question |
|---|---|
| Eligibility | Are ineligible activations prevented from execution? |
| Ordering | Does the scheduler obey its declared policy? |
| Tie-breaking | Is equivalent work ordered according to defined rules? |
| Priority | Does priority affect selection as specified? |
| Deadline | Are deadline policies correctly applied? |
| Periodicity | Are release and overrun rules correct? |
| Budget | Is budget state correctly integrated with execution? |
| Preemption | Is preemption behavior consistent with platform capability? |
| Cancellation | Are queued and running cancellations handled correctly? |
| Fairness | Is the declared fairness property measurable? |
| Starvation | Can starvation be detected or bounded where required? |
| Blocking | Are blocking states and delays observable? |
| Affinity | Are placement constraints respected? |
| Determinism | Are relevant scheduling inputs reproducible? |
| Replay | Is sufficient evidence recorded to reconstruct decisions? |

## 35. What Part VIII Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- a production scheduler;
- hard real-time guarantees;
- bounded worst-case scheduling latency;
- starvation freedom under arbitrary load;
- bounded priority inversion;
- deterministic execution across arbitrary operating systems;
- complete preemption;
- scheduler replay fidelity.

Those claims require implementation and verification evidence.

## 36. Transition to Part IX

Part VIII defines scheduling and execution semantics.

Part IX should define **supervision, fault handling, recovery, and resilience**, connecting lifecycle faults, resource failures, communication failures, and executor failures into one coherent control model.

```text
Part VII
Resources + budgets
        ↓
Part VIII
Scheduling + executor
        ↓
Part IX
Supervision + recovery
```

## Canonical rule

> **NROS separates admission, scheduling, and execution: only eligible work may be scheduled, scheduling policy determines selection, and the executor maps that decision to platform mechanisms without silently creating timing guarantees.**
