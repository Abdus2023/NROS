# NROS IPC Specification

> **Status:** Normative specification.
>
> IPC defines the contract for communication between independently executing components. A concrete IPC backend must satisfy the applicable endpoint, ownership, delivery, resource, and lifecycle requirements.

## 1. Endpoint model

An IPC endpoint MUST have an identifiable lifecycle and ownership model.

```text
Create
  ↓
Initialize
  ↓
Ready
  ↓
Communicate
  ↓
Close / Shutdown
```

The contract should define whether endpoints may reconnect, be reused, or be invalidated after peer failure.

## 2. Message framing

IPC MUST define how message boundaries are established.

```text
Producer
   ↓
Message
   ↓
Frame / record boundary
   ↓
Consumer
```

A byte stream, record queue, shared-memory ring, or other mechanism may use different framing rules. Consumers MUST NOT infer message boundaries from incidental implementation behavior.

## 3. Ownership and lifetime

IPC messages and buffers MUST have explicit ownership semantics where ownership is not inherent in the underlying mechanism.

```text
Allocate
   ↓
Owned by producer
   ↓
Transferred / shared
   ↓
Consumed
   ↓
Released / reclaimed
```

The contract should identify who may mutate data, when a buffer becomes invalid, and whether ownership may cross process boundaries.

## 4. Synchronization

IPC synchronization MUST define the conditions under which producers and consumers may access shared communication state.

The implementation may use locks, atomics, OS primitives, queues, shared-memory protocols, or other mechanisms. The normative contract concerns observable correctness, not an assumed primitive.

## 5. Delivery and ordering

Where relevant, an IPC contract MUST define:

- delivery behavior;
- ordering scope;
- duplication behavior;
- loss behavior;
- blocking/non-blocking behavior;
- timeout behavior.

These properties are independent:

```text
Delivery
  ≠ Ordering
  ≠ Reliability
  ≠ Exactly-once
  ≠ Bounded latency
```

## 6. Resource limits

IPC implementations SHOULD define limits for resources such as:

- queue depth;
- message size;
- shared buffers;
- endpoint count;
- outstanding operations.

When a limit is reached, the contract MUST define the resulting behavior where the condition can affect correctness: block, reject, drop, fail, or apply another explicit policy.

## 7. Backpressure

Producer/consumer rate mismatch must have a defined policy where overload is possible.

```text
Producer
   ↓
IPC capacity
   ↓
Consumer
```

An implementation MUST NOT silently discard data when the contract requires reliable delivery.

## 8. Failure semantics

IPC failures MUST be observable and MUST NOT silently become successful application operations.

Relevant failures include:

- endpoint unavailable;
- peer termination;
- queue exhaustion;
- malformed message;
- synchronization failure;
- timeout;
- shutdown race.

Recovery semantics should specify whether an endpoint remains usable after each failure class.

## 9. Zero-copy and shared memory

A zero-copy or shared-memory claim requires an end-to-end contract covering:

```text
Allocation
   ↓
Mapping / sharing
   ↓
Ownership transfer
   ↓
Synchronization
   ↓
Consumption
   ↓
Reclamation
```

An API named `shared`, `zero_copy`, or similar is not sufficient evidence of zero-copy behavior.

## 10. Shutdown and lifecycle

IPC shutdown MUST define what happens to:

- queued messages;
- active operations;
- blocked producers/consumers;
- shared buffers;
- pending notifications;
- peer endpoints.

A close request does not automatically imply that all outstanding work has completed or been cancelled.

## 11. Compatibility

IPC peers that exchange structured messages MUST define compatibility expectations for:

- schema/version;
- framing;
- encoding;
- required fields;
- error representation.

ABI compatibility and wire compatibility are separate properties.

## 12. Verification requirements

| Claim | Evidence |
|---|---|
| Endpoint lifecycle is correct | Lifecycle integration tests |
| Message framing is correct | Boundary/fragmentation tests |
| Ownership is correct | Ownership/lifetime tests or analysis |
| Ordering is guaranteed | Controlled ordering tests |
| Backpressure is correct | Capacity/exhaustion tests |
| Failure is observable | Fault-injection tests |
| Shutdown is safe | Concurrent shutdown tests |
| Zero-copy works | End-to-end buffer/path evidence |
| Cross-version IPC works | Compatibility matrix + integration tests |

## 13. Related specifications

- [Specifications Index](README.md)
- [Types](types.md)
- [Protocols](protocols.md)
- [Transport](transport.md)
- [Safety](safety.md)
