# Part XIII — Dataflow, Events & Flow Control

> **Series:** NROS Architecture Series  
> **Part:** XIII  
> **Role:** Messages, commands, events, streams, queues, buffering, delivery, ordering, and backpressure  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part V defined communication and transport semantics. Part XIII defines the semantic layer carried through those transports: messages, commands, events, streams, buffering, delivery, ordering, and flow control.

The central rule is:

> **NROS separates data semantics from transport mechanics and treats flow control as an explicit resource and correctness concern.**

## 2. Core Dataflow Model

```text
Producer
   ↓
Data item
   ↓
Channel / Stream
   ↓
Buffer
   ↓
Consumer
```

The data item may be a command, event, response, state update, or stream sample.

## 3. Message

A message is a transportable structured data unit.

```text
Message
├── type
├── schema/version
├── identity
├── timestamp
├── source
├── destination / channel
├── correlation
├── payload
└── metadata
```

The exact envelope is implementation-specific.

## 4. Command

A command requests an action.

```text
Command
   ↓
Requested operation
```

A command should define whether the sender expects:

```text
acceptance
completion
result
error
```

Acceptance does not imply completion.

## 5. Event

An event reports that something happened or that a defined state transition occurred.

```text
Event
   ↓
Observation
```

An event is not automatically a command and should not be interpreted as an instruction unless its contract explicitly says so.

## 6. Response

A response correlates with an earlier request or command.

```text
Request ─────→
             Response
       ←─────
```

Correlation identifiers should remain stable across retries where the protocol requires duplicate detection.

## 7. Stream

A stream represents an ordered or partially ordered sequence of data items.

```text
x1 → x2 → x3 → x4 → ...
```

The contract must define ordering, retention, loss, and termination behavior.

## 8. State Update

A state update describes a new observation or state representation.

State updates may be:

```text
full snapshot
incremental delta
patch
versioned state
```

A delta cannot be applied safely without its required base state/version.

## 9. Data Semantics vs Transport

```text
Data semantics
      ↓
Message / event / command contract
      ↓
Transport
      ↓
Network / IPC / shared memory / other mechanism
```

Changing transport should not silently change application-level meaning.

## 10. Channel

A channel is a logical communication path between producers and consumers.

```text
Channel
├── contract
├── producer policy
├── consumer policy
├── ordering
├── buffering
├── delivery semantics
└── flow-control policy
```

A channel may be implemented using different transport mechanisms.

## 11. Queue

A queue stores data items awaiting consumption.

```text
Producer
   ↓
[ a | b | c | d ]
              ↓
           Consumer
```

Queue semantics must define capacity and overflow behavior.

## 12. Buffering

Buffers absorb temporary producer/consumer rate differences.

```text
producer rate > consumer rate
        ↓
     buffer grows
```

A buffer is finite unless the implementation explicitly provides an effectively unbounded model with an acceptable resource policy.

## 13. Capacity

Capacity may be expressed in:

```text
items
bytes
frames
messages
time window
resource units
```

The capacity dimension must be explicit.

## 14. Backpressure

Backpressure communicates downstream inability to accept additional data.

```text
Consumer slows
      ↓
Channel pressure
      ↓
Producer adapts
```

Possible responses:

```text
block
throttle
batch
coalesce
drop
sample
spill
reject
```

Backpressure is a flow-control policy, not merely a queue implementation detail.

## 15. Overflow

When a finite buffer is full, the channel must define behavior.

```text
DROP_OLDEST
DROP_NEWEST
BLOCK
REJECT
COALESCE
SPILL
FAIL
```

Silent overflow behavior is prohibited at the architectural level.

## 16. Loss Semantics

Data loss may be permitted for some classes of data.

Examples:

```text
telemetry → lossy may be acceptable
command   → loss may be unacceptable
checkpoint → loss may be unacceptable
```

The data contract determines acceptable loss.

## 17. Delivery Semantics

Possible delivery models include:

```text
at-most-once
at-least-once
exactly-once effect
best-effort
```

"Exactly once" must refer to a precisely defined scope and effect; transport duplication and application-level idempotency remain separate concerns.

## 18. Duplicate Delivery

At-least-once delivery may produce:

```text
A A B C C
```

Consumers requiring duplicate suppression need:

```text
message identity
sequence number
idempotency key
or equivalent mechanism
```

## 19. Ordering

Ordering may be defined per:

```text
channel
producer
partition
entity
key
stream
```

Global ordering should not be assumed unless explicitly guaranteed.

## 20. Sequence Numbers

Ordered streams may expose:

```text
sequence = 100
sequence = 101
sequence = 102
```

A gap may indicate:

```text
loss
filtering
partition change
reset
producer restart
```

The interpretation must be defined by the protocol.

## 21. Replay

A persistent event stream may support replay:

```text
Stored events
   ↓
Consumer offset
   ↓
Replay
```

Replay requires explicit retention and ordering semantics.

## 22. Acknowledgement

Acknowledgement indicates a defined milestone, such as:

```text
received
validated
queued
processed
committed
```

These must not be conflated.

```text
Received
   ≠
Processed
   ≠
Committed
```

## 23. Correlation

Related operations may share a correlation identifier.

```text
Command
  correlation = X
     ↓
Response
  correlation = X
```

Correlation is an observability/protocol mechanism and does not itself guarantee ordering or delivery.

## 24. Fan-Out

One producer may publish to multiple consumers.

```text
          → C1
P → Channel → C2
          → C3
```

The architecture must specify whether slow C1 affects C2/C3.

Possible isolation policies include independent queues or shared backpressure.

## 25. Fan-In

Multiple producers may feed one consumer.

```text
P1 ─┐
P2 ─┼→ Channel → C
P3 ─┘
```

Ordering between producers must be explicit.

## 26. Partitioning

Streams may be partitioned by a key:

```text
key A → partition 1
key B → partition 2
key C → partition 3
```

Partitioning can improve scalability but changes ordering and failure semantics.

## 27. Rate Control

Producers and consumers may operate at different rates.

Flow-control mechanisms may include:

```text
rate limit
credit-based flow control
token bucket
windowing
pull-based consumption
adaptive throttling
```

The mechanism must respect resource budgets from Part VII.

## 28. Pull vs Push

### Push

```text
Producer → Consumer
```

The producer determines delivery timing.

### Pull

```text
Consumer → request
Producer → data
```

The consumer controls consumption rate.

Hybrid models are permitted.

## 29. Flow-Control Credits

A consumer may advertise capacity:

```text
credits = 8
```

Each accepted item consumes credit.

```text
credits → 7 → 6 → ...
```

Credits can then be replenished.

## 30. Deadlines

Data items may have temporal validity.

```text
Produced
  ↓
deadline
  ↓
expired
```

An expired item may be:

```text
dropped
rejected
processed anyway
converted to a fault
```

The contract must specify the policy.

## 31. Priority

Data items may carry priority.

Priority affects scheduling only when the channel/executor policy explicitly uses it.

```text
Priority metadata
    ≠ automatically prioritized execution
```

## 32. Cancellation

A producer or consumer may cancel a dataflow operation.

Cancellation semantics should distinguish:

```text
stop future production
remove queued data
interrupt current processing
ignore stale results
```

## 33. Poison / Invalid Data

Consumers should be able to classify invalid data:

```text
schema-invalid
malformed
unauthorized
stale
incompatible
corrupted
```

Policy may reject, quarantine, report, or terminate the channel depending on severity.

## 34. Security Integration

Part XI security semantics apply to dataflow:

```text
publish
subscribe
consume
acknowledge
replay
inspect
modify
```

A discoverable channel is not automatically an authorized channel.

## 35. Persistence Integration

Part XII allows selected streams/events to be persisted.

```text
Event
 ↓
Journal
 ↓
Replay
```

Persistence does not automatically make every event durable; the event contract must specify durability requirements.

## 36. Scheduling Integration

Dataflow events can trigger scheduler activity:

```text
Message arrival
      ↓
Activation release
      ↓
Ready queue
      ↓
Scheduler
```

Message arrival should not silently bypass admission, resource, or authorization rules.

## 37. Resource Integration

Channels consume:

```text
memory
CPU
I/O
network bandwidth
storage
queue capacity
```

Flow-control policy must therefore be compatible with resource budgets.

## 38. Observability

Dataflow events should support structured observation:

```text
MessagePublished
MessageAccepted
MessageRejected
MessageDropped
MessageDelivered
MessageAcknowledged
MessageExpired
QueueFull
BackpressureStarted
BackpressureReleased
StreamGapDetected
ReplayStarted
ReplayCompleted
```

## 39. Verification Matrix

| Property | Verification question |
|---|---|
| Semantics | Is the data item type unambiguous? |
| Delivery | Is the delivery guarantee explicit? |
| Ordering | Is the ordering scope defined? |
| Capacity | Is buffer capacity bounded and measurable? |
| Overflow | Is full-buffer behavior explicit? |
| Backpressure | Can downstream pressure propagate correctly? |
| Loss | Is permitted loss explicitly declared? |
| Duplicates | Can duplicate delivery be detected where required? |
| Replay | Can retained events be replayed according to contract? |
| Acknowledgement | Is each acknowledgement milestone unambiguous? |
| Deadlines | Are expired items handled according to policy? |
| Security | Are publish/subscribe/replay operations authorized? |
| Persistence | Are durability requirements explicit? |
| Scheduling | Does message arrival respect execution admission? |
| Resources | Is flow-control resource consumption bounded? |

## 40. What Part XIII Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- universal exactly-once delivery;
- global message ordering;
- lossless streams under arbitrary load;
- unbounded queues;
- automatic congestion control;
- durable replay for every event;
- transparent distributed backpressure;
- zero-copy transport for every data type.

Those properties require implementation and verification evidence.

## 41. Transition to Part XIV

Part XIII defines dataflow semantics and flow control.

Part XIV should define **observability, telemetry, tracing, metrics, diagnostics, and evidence**, turning the runtime's many explicit state transitions into a coherent verification and operations surface.

```text
Part XII
Persistence + state
        ↓
Part XIII
Dataflow + events + flow control
        ↓
Part XIV
Observability + telemetry + evidence
```

## Canonical rule

> **NROS treats messages, commands, events, streams, state updates, buffering, delivery, and flow control as explicit contracts; transport, scheduling, persistence, resources, and security enforce those contracts without silently changing their semantics.**
