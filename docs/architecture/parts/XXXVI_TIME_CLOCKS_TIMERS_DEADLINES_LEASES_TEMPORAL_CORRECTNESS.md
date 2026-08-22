# Part XXXVI — Time, Clocks, Timers, Deadlines, Leases & Temporal Correctness

> **Series:** NROS Architecture Series  
> **Part:** XXXVI  
> **Role:** Temporal semantics, clock domains, monotonic time, wall time, logical time, timers, deadlines, leases, clock uncertainty, temporal ordering, and time-aware correctness  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXXV established durable workflow and orchestration semantics. Part XXXVI defines the temporal foundation on which workflows, retries, deadlines, leases, sessions, distributed coordination, event ordering, persistence, and autonomous execution depend.

The central rule is:

> **NROS never treats “time” as one interchangeable value: every temporal operation declares its clock domain, precision, ordering meaning, uncertainty, and failure behavior. Monotonic time governs elapsed-duration reasoning; wall time governs externally meaningful timestamps; logical time governs causality; deadlines and leases are interpreted only within their declared temporal model.**

## 2. Fundamental Distinctions

```text
wall-clock time
  ≠
monotonic time
  ≠
logical time
  ≠
event time
  ≠
processing time
  ≠
deadline
  ≠
lease expiry
```

## 3. Clock Domain

Every time value belongs to a clock domain:

```text
Clock Domain
 ├─ identity
 ├─ source
 ├─ unit
 ├─ resolution
 ├─ monotonicity
 ├─ uncertainty
 └─ validity assumptions
```

A timestamp from one domain must not be compared directly with a timestamp from another domain unless the conversion contract explicitly permits it.

## 4. Wall Time

Wall time approximates civil or externally meaningful time:

```text
2026-08-22T12:00:00Z
```

It can move forward or backward because of clock correction, synchronization, virtualization, or administrative changes.

## 5. Monotonic Time

Monotonic time is intended for elapsed-duration measurement:

```text
start = mono_now()
...
elapsed = mono_now() - start
```

It should not be interpreted as a civil timestamp.

## 6. Logical Time

Logical time represents ordering or causality rather than physical time:

```text
L(A) < L(B)
```

Logical clocks are governed by Part XXXIII's causal semantics.

## 7. Event Time

Event time is the time associated with occurrence semantics:

```text
Event occurrence
      ↓
Event time
```

It must not automatically be treated as delivery or processing time.

## 8. Processing Time

Processing time is when a consumer handles an event:

```text
Event occurs at T1
 ↓
delivered at T2
 ↓
processed at T3
```

Usually:

```text
T1 ≠ T2 ≠ T3
```

## 9. Commit Time

Commit time identifies when a durable state or event became committed under the relevant persistence contract.

It must not be inferred solely from the time an operation began.

## 10. Time Precision

A time source has finite resolution:

```text
Resolution = R
```

The system must not claim precision finer than the source can establish.

## 11. Time Accuracy

Accuracy describes how close a clock is to a reference time.

```text
precision ≠ accuracy
```

High-resolution clocks are not necessarily accurate wall clocks.

## 12. Clock Uncertainty

Distributed time should be treated as an interval when uncertainty matters:

```text
T ∈ [Tmin, Tmax]
```

Temporal decisions must account for the uncertainty bound when safety depends on it.

## 13. Clock Synchronization

Wall-clock synchronization can reduce disagreement but does not eliminate uncertainty.

```text
Node A ─┐
        ├─ synchronization
Node B ─┘
        ↓
bounded disagreement
```

The bound must be established rather than assumed.

## 14. Clock Jumps

Wall time may jump:

```text
12:00
 ↓
11:59
```

Duration calculations must therefore use monotonic time rather than wall time.

## 15. Sleep and Suspend

System suspend can affect elapsed-time behavior depending on the clock source.

NROS components must define whether timers and deadlines advance during suspend, hibernation, or process pause.

## 16. Timer

A timer schedules an action relative to a clock:

```text
Clock
 ↓
duration / deadline
 ↓
Timer
 ↓
Wake / callback / event
```

Timer semantics include cancellation, coalescing, missed deadlines, and resource limits.

## 17. Timer Identity

Long-lived timers should have stable identifiers when cancellation or recovery requires them:

```text
Timer ID
 + owner
 + generation
```

## 18. Timer Cancellation

A timer cancellation must define the race with expiration:

```text
Cancel ─┐
        ├─ race
Expire ─┘
```

The authority must define whether the callback can still execute after cancellation is accepted.

## 19. Timer Accuracy

A timer should be treated as an execution eligibility boundary, not a guarantee of exact physical execution time.

```text
deadline reached
      ↓
eligible to run
      ≠
executed at exact instant
```

## 20. Timer Overrun

If execution takes longer than its interval:

```text
periodic timer
 ↓
execution exceeds period
```

The contract must define whether to:

```text
skip
coalesce
queue
catch up
terminate
```

## 21. Periodic Scheduling

A periodic schedule can be defined by:

```text
start
period
clock domain
jitter policy
missed-tick policy
```

## 22. Drift

Repeatedly scheduling from the previous completion can drift:

```text
T1 → run
T1+P → expected
actual + execution delay → drift
```

For stable periodic schedules, absolute schedule points may be preferable:

```text
T0 + nP
```

## 23. Deadline

A deadline is an absolute boundary within a declared clock domain:

```text
Deadline D

now < D → eligible
now ≥ D → expired
```

## 24. Deadline Propagation

A workflow may propagate a deadline:

```text
Parent deadline
      ↓
Child task
      ↓
Network request
      ↓
Storage operation
```

A child must not receive a deadline later than the parent unless explicit semantics permit it.

## 25. Remaining Budget

A component can derive a remaining temporal budget:

```text
Budget = Deadline - MonotonicNow
```

The actual calculation must use a compatible clock model.

## 26. Timeout

A timeout is a duration bound:

```text
Start + Timeout = Boundary
```

It differs from a propagated absolute deadline.

## 27. Timeout and External Effects

A local timeout does not prove that an external operation stopped:

```text
Local timeout
      ≠
Remote cancellation
      ≠
Remote completion
```

The external effect must have its own acknowledgement or reconciliation semantics.

## 28. Lease

A lease grants temporary authority:

```text
Lease Granted
 ↓
Authority valid
 ↓
Expiry
 ↓
Authority invalid
```

Lease semantics require an explicit clock model and expiration rule.

## 29. Lease Duration

A lease may be represented as:

```text
issued_at + duration
```

but distributed safety must account for clock uncertainty and communication delay.

## 30. Lease Renewal

```text
Lease
 ↓
Renew
 ↓
New validity interval
```

Renewal must happen before the current authority expires according to the relevant safety margin.

## 31. Lease Expiry

Expiry is a state transition, not merely a timestamp:

```text
Active
 ↓
Expiry established
 ↓
Expired
```

Stale holders must be prevented from continuing authoritative operations.

## 32. Fencing Tokens

Leases should be paired with monotonically increasing fencing tokens where stale holders can cause dangerous effects:

```text
Token 10 → valid
Token 9  → reject
```

This protects against delayed messages from expired lease holders.

## 33. Lease and Clock Uncertainty

If uncertainty is ε:

```text
Safe expiry boundary
    ≠
nominal expiry timestamp alone
```

Safety margins must account for the declared uncertainty model.

## 34. Temporal Ordering

Temporal ordering can be based on:

```text
wall time
monotonic time
logical clock
sequence number
causal relation
```

The selected ordering must be explicit.

## 35. Physical Time Does Not Prove Causality

```text
Timestamp(A) < Timestamp(B)
```

does not by itself prove:

```text
Cause(A, B)
```

Causality remains a Part XXXIII semantic relation.

## 36. Logical Ordering

When physical clocks cannot establish ordering, logical clocks or sequence numbers may provide the required relation:

```text
L(A) < L(B)
```

## 37. Hybrid Temporal Metadata

A system may carry both:

```text
physical timestamp
 + logical timestamp
 + sequence
```

These values serve different purposes and must not be collapsed into one field.

## 38. Temporal Windows

Policies may define valid windows:

```text
NotBefore
      ↓
   Valid
      ↓
NotAfter
```

Window validation must use the correct clock and uncertainty model.

## 39. Not-Before Constraints

An operation may be prohibited before a specified time:

```text
now < NotBefore
      ↓
reject / defer
```

## 40. Not-After Constraints

An operation may be rejected after expiry:

```text
now ≥ NotAfter
      ↓
expired
```

This is common for capabilities, credentials, leases, and workflow deadlines.

## 41. Clock Failure

If a required clock becomes unavailable or untrustworthy:

```text
Clock failure
 ↓
Temporal confidence degraded
 ↓
Fail safe / defer / switch source
```

The system must not silently treat an untrusted clock as authoritative.

## 42. Clock Source Selection

A component may have multiple clock sources:

```text
Primary
Secondary
Fallback
```

Switching sources must account for discontinuities and must not silently break monotonic assumptions.

## 43. Monotonicity Invariant

For a monotonic clock:

```text
read(t2) ≥ read(t1)
```

for `t2` after `t1` according to the clock's execution semantics.

## 44. Deadline Invariant

For an expired deadline:

```text
now ≥ D
    ⇒
operation is not eligible to begin
```

unless an explicit override policy exists.

## 45. Lease Safety Invariant

After a lease is no longer valid:

```text
LeaseExpired(L)
    ⇒
HolderCannotPerformAuthoritativeAction(L)
```

Fencing may be required to enforce this across delayed messages.

## 46. Timer Invariant

Cancellation semantics must ensure that:

```text
Cancelled(Timer)
```

cannot produce an unaccounted-for authoritative action after cancellation has committed.

## 47. Temporal Budgeting

Workflows can allocate temporal budgets:

```text
Parent Budget
 ├─ Task A
 ├─ Task B
 └─ Task C
```

Budgets should account for scheduling, retries, network latency, and recovery overhead.

## 48. Retry and Time

Retries must obey both attempt and temporal budgets:

```text
RetryCount ≤ MaxAttempts
AND
Now < Deadline
```

A task should not start another attempt when its deadline has already expired.

## 49. Backoff and Deadline

A retry delay should be capped by remaining time:

```text
delay = min(policy_delay, remaining_budget)
```

If the resulting budget is insufficient for a viable attempt, the retry should be abandoned.

## 50. Scheduling and Time

Scheduler decisions may depend on:

```text
priority
arrival time
deadline
fairness
resource availability
```

Time-dependent scheduling must specify which clock supplies each value.

## 51. Queue Visibility Timeout

A queue may use a visibility timeout:

```text
Message claimed
 ↓
visibility window
 ↓
ACK
```

If the timeout expires, redelivery may occur. This is not proof of duplicate logical operations; it is an execution-attempt concern.

## 52. Temporal Idempotency

Idempotency keys may have temporal retention:

```text
Key K
 ↓ valid until T
 ↓ expired
```

Retention must cover the maximum retry/replay window required by the operation contract.

## 53. Event Retention and Time

Part XXXIII retention policies may be expressed using time:

```text
retain for D
```

The retention clock and archival guarantees must be explicit.

## 54. Time and Persistence

Persistent timestamps should identify their clock semantics:

```text
wall timestamp
logical timestamp
monotonic duration
```

A raw integer timestamp without clock-domain metadata is insufficient for portable semantic interpretation.

## 55. Time and Serialization

Part XXXII governs representation:

```text
Temporal Value
 ↓
Schema
 ↓
Canonical Encoding
```

The schema must identify units, epoch, timezone/offset semantics where applicable, and clock domain.

## 56. Time and Identity

Temporal metadata may be combined with identity generation:

```text
Identity
 + Generation
 + Time metadata
```

Time must not be used as a substitute for unique identity unless uniqueness guarantees are explicitly established.

## 57. Time and Security

Temporal security policies include:

```text
credential expiry
capability validity
replay windows
nonce lifetime
lease duration
```

Security-critical expiry must account for clock uncertainty.

## 58. Replay Windows

A message may be accepted only within a defined window:

```text
Issued
 ↓
Acceptable window
 ↓
Expired
```

The verifier must use an appropriate trusted time source and defined skew allowance.

## 59. Temporal Replay Protection

A timestamp alone is not sufficient replay protection when an attacker can replay a still-valid message.

Replay protection may require:

```text
nonce
sequence
idempotency key
stateful seen-set
```

## 60. Time and Distributed Coordination

Consensus and leader election may use timeouts for failure detection, but timeout expiration is not itself proof that another node has stopped.

Fencing and authority epochs remain necessary where stale actors can perform effects.

## 61. Failure Detection

```text
No heartbeat
 ↓
timeout
 ↓
suspect failure
```

A timeout indicates lack of observation, not necessarily physical failure.

## 62. Temporal Assumptions

Every distributed temporal guarantee should state assumptions such as:

```text
maximum clock skew
maximum message delay
maximum processing delay
lease duration
renewal margin
```

Without explicit assumptions, temporal safety claims are incomplete.

## 63. Deadline Propagation Across Networks

A propagated deadline should account for transport and processing overhead:

```text
Parent Deadline
      ↓
Network Budget
      ↓
Remote Deadline
```

A remote component must not continue work after its effective deadline merely because the original caller has disconnected.

## 64. Temporal Backpressure

If work cannot meet its deadline because of queue delay:

```text
Queued
 ↓
deadline approaches
 ↓
admission decision
```

The scheduler may reject work early rather than wasting capacity on guaranteed failure.

## 65. Time Zones

Civil timestamps may carry timezone or UTC-offset information.

Distributed protocol semantics should generally use an unambiguous representation such as UTC-based timestamps, while presentation layers may localize them.

## 66. Calendar Time

Calendar calculations such as “next day” or “next month” are not equivalent to fixed durations:

```text
calendar period
 ≠
86400 seconds in every civil-time context
```

NROS must distinguish calendar semantics from duration semantics.

## 67. Duration

Durations represent elapsed quantities:

```text
5 seconds
100 milliseconds
2 hours
```

They should use monotonic time for execution measurement where available.

## 68. Temporal Serialization Safety

A serialized temporal field should specify at minimum:

```text
value
unit
clock/epoch semantics
precision
optional uncertainty
```

## 69. Temporal Observability

Diagnostics should expose enough metadata to explain timing behavior:

```text
clock domain
timestamp/deadline
remaining budget
clock uncertainty
timer state
lease epoch/token
```

Sensitive data must still follow Part XXIX controls.

## 70. Formal Temporal Invariant

```text
DurationMeasurement
    =
MonotonicEnd - MonotonicStart
```

where the selected clock is guaranteed monotonic for the required execution scope.

## 71. Formal Deadline Invariant

```text
StartExecution(T)
    ⇒
Now < EffectiveDeadline(T)
```

unless the operation explicitly permits late execution.

## 72. Formal Lease Invariant

```text
LeaseValid(H, t)
 ∧ FencingToken(H) < CurrentToken
    ⇒
RejectAuthoritativeEffect(H)
```

## 73. Formal Retry-Time Invariant

```text
AttemptStarted
    ⇒
AttemptCount ≤ MaxAttempts
 ∧
Now < Deadline
```

## 74. Verification Matrix

| Property | Verification question |
|---|---|
| Clock domain | Is every temporal value tied to a defined clock? |
| Monotonicity | Are durations measured with monotonic time? |
| Wall time | Are civil timestamps clearly distinguished? |
| Logical time | Is causality separated from physical time? |
| Precision | Is claimed precision supported by the source? |
| Accuracy | Is clock uncertainty understood? |
| Timers | Are cancellation and expiry races defined? |
| Periodic work | Is drift/missed-tick behavior explicit? |
| Deadline | Is the boundary and clock domain explicit? |
| Timeout | Is duration distinguished from deadline? |
| Lease | Is authority expiry enforceable? |
| Fencing | Can stale lease holders be rejected? |
| Retry | Are retry count and temporal budget bounded? |
| Security | Are expiry/replay windows safe under clock skew? |
| Persistence | Are temporal values semantically serializable? |
| Distribution | Are timing assumptions explicit? |
| Observability | Can temporal decisions be diagnosed? |
| Formal assurance | Are deadline, lease, and duration invariants explicit? |

## 75. What Part XXXVI Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a unified distributed clock service;
- globally synchronized clocks;
- bounded clock uncertainty everywhere;
- universally precise timers;
- production-grade lease enforcement;
- complete fencing-token implementation;
- formally verified temporal guarantees;
- deterministic scheduling under all environments;
- universal calendar/time-zone semantics.

Those require implementation-specific evidence.

## 76. Transition to Part XXXVII

Part XXXVI establishes temporal semantics.

Part XXXVII should define **resource accounting, quotas, admission control, scheduling fairness, memory/CPU/I/O budgets, pressure propagation, and overload behavior**, connecting temporal execution constraints with finite system capacity.

```text
Part XXXV
Workflows + orchestration + jobs + tasks + retries + compensation + scheduling
        ↓
Part XXXVI
Time + clocks + timers + deadlines + leases + temporal correctness
        ↓
Part XXXVII
Resource accounting + quotas + admission + fairness + pressure + overload
```

## Canonical rule

> **NROS treats time as typed architectural data: elapsed work uses monotonic time, external timestamps use explicit wall-time semantics, causality uses logical ordering, deadlines and leases carry clock-domain assumptions, and every temporal guarantee states its precision, uncertainty, failure, and enforcement model.**
