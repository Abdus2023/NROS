# Inter-Process Communication

## Purpose

NROS IPC is the communication boundary between cooperating runtime components. The architecture emphasizes typed communication, predictable ownership, and the possibility of zero-copy data movement where the platform permits it.

## Conceptual model

```text
Producer
   │
   ▼
Typed message / buffer
   │
   ▼
IPC transport
   │
   ▼
Consumer
```

## Design goals

- explicit ownership and lifetime rules;
- minimal unnecessary copying;
- deterministic behavior where required;
- type-safe interfaces;
- clear separation between local IPC and network transport;
- observable failure and backpressure behavior.

## Zero-copy boundary

Zero-copy is an architectural goal, not a blanket implementation claim. A buffer is only considered zero-copy when the relevant ownership, allocation, transport, and consumer path demonstrate that no unintended data copy occurs.

Platform-specific mechanisms such as shared memory, memory mapping, or DMA buffers belong to the implementation layer and must be verified independently.

## Current repository status

The repository contains IPC-related implementation and safety work. This document intentionally describes the architecture without promoting a prototype, simulation, or scaffold to a validated zero-copy implementation. See verification records for evidence.
