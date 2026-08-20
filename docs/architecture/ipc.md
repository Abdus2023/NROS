# NROS IPC

> **Status:** Active architectural documentation.
>
> This document defines the logical IPC boundary. It does not claim a particular IPC backend, zero-copy implementation, or performance level unless supported by repository evidence.

## 1. Purpose

Inter-process communication (IPC) connects logical runtime participants that do not share the same execution context. The IPC boundary separates application-level communication semantics from process-level transport and synchronization mechanisms.

```text
Publisher / Producer
        ↓
Logical Message
        ↓
IPC Boundary
        ↓
Process-local representation
        ↓
Subscriber / Consumer
```

## 2. IPC versus communication semantics

```text
Topic / Service / Action semantics
            ≠
IPC mechanism
            ≠
Network transport
```

The same logical communication contract may be realized by different mechanisms.

## 3. Data path

```text
Producer
  ↓
Message representation
  ↓
Serialization / encoding (if required)
  ↓
IPC transport
  ↓
Synchronization / delivery
  ↓
Deserialization / decoding (if required)
  ↓
Consumer
```

An implementation may remove or optimize individual stages, but such behavior must be demonstrated by concrete implementation evidence.

## 4. Ownership

IPC design must distinguish message ownership, buffer ownership, transport ownership, producer lifetime, consumer lifetime, and synchronization ownership.

A "zero-copy" claim requires evidence covering allocation, ownership transfer or sharing, synchronization, lifetime, and the actual communication path.

## 5. Synchronization

IPC may require synchronization around shared state, queues, buffers, or process lifecycle. The synchronization boundary must remain explicit.

```text
Data availability
      ↓
Synchronization
      ↓
Consumption
      ↓
Release / reuse
```

The selected primitive and memory-ordering properties are implementation concerns.

## 6. Failure modes

Relevant IPC failures include endpoint unavailability, process termination, queue or buffer exhaustion, serialization failure, synchronization failure, malformed data, timeout/cancellation, and stale endpoint state.

Failure handling should preserve the distinction between communication failure and application failure.

## 7. Backpressure

IPC systems may require explicit handling of producer/consumer rate mismatch.

```text
Producer rate > Consumer rate
          ↓
       backlog
          ↓
 queue policy / backpressure / drop policy
```

Loss, ordering, blocking, or bounded-latency guarantees require a documented contract and corresponding tests.

## 8. Ordering and delivery

The following properties are independent:

```text
Delivery
Ordering
Reliability
At-most-once / at-least-once semantics
Latency
Determinism
```

Evidence must support whichever properties the implementation claims.

## 9. IPC and network transport

IPC is not automatically equivalent to network transport. A local IPC path may use shared memory, OS primitives, sockets, queues, or other mechanisms, while distributed communication introduces additional concerns such as network failure, discovery, serialization compatibility, and topology.

See [Transport](transport.md) for the network boundary.

## 10. Verification requirements

| Claim | Minimum evidence |
|---|---|
| IPC interface exists | Source/interface inspection |
| Data crosses process boundary | Executed multi-process test |
| Ordering guarantee | Reproducible ordering test |
| Bounded queue | Configuration + implementation + exhaustion test |
| Zero-copy | Buffer/ownership/path evidence |
| Failure recovery | Process/endpoint failure test |
| Latency bound | Controlled benchmark on declared target |
| Real-time suitability | Target-specific timing validation |

## 11. Related documents

- [Architecture Overview](overview.md)
- [System Model](system-model.md)
- [Runtime](runtime.md)
- [Scheduling](scheduling.md)
- [Transport](transport.md)
- [Verification](../verification/README.md)
- [Evidence Registry](../../EVIDENCE_REGISTRY.md)
