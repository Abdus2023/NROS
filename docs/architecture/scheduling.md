# Scheduling

## Purpose

Scheduling determines when runtime work is eligible to execute and provides the foundation for predictable timing behavior.

## Architectural goals

NROS scheduling is designed around:

- explicit execution priorities;
- deadline awareness;
- bounded and observable scheduling behavior;
- separation of real-time work from non-real-time work;
- deterministic execution where the application requires it.

## Conceptual execution path

```text
Work item
   │
   ▼
Eligibility / deadline
   │
   ▼
Scheduler
   │
   ▼
Execution context
   │
   ▼
Completion / deadline result
```

## Real-time boundary

A real-time claim requires more than selecting a high-priority thread. The complete execution path—including allocation, synchronization, IPC, I/O, timers, and external dependencies—must satisfy the required timing bound.

Therefore documentation must distinguish a scheduling design from demonstrated real-time behavior.

## Verification

Timing claims belong to measured benchmark or validation evidence. Architectural intent alone does not establish a real-time guarantee.
