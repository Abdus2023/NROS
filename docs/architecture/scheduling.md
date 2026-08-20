# NROS Scheduling

> **Status:** Active architectural documentation.
>
> This document defines the conceptual scheduling model. It does not establish deterministic, bounded-latency, or hard-real-time behavior without executable evidence on a declared target.

## 1. Purpose

Scheduling determines when executable work becomes eligible to run and how execution resources are selected.

```text
Events / timers / messages / lifecycle work
                    ↓
                 Readiness
                    ↓
                Scheduler
                    ↓
                 Executor
                    ↓
                 Callback
                    ↓
             State / output
```

## 2. Work sources

Potential work sources include incoming communication, timers, lifecycle transitions, application callbacks, internal runtime events, and shutdown/cancellation events. The existence of a work-source abstraction does not establish that every source is implemented.

## 3. Readiness

A scheduler operates on an explicit notion of readiness. Readiness may depend on data availability, timer expiration, lifecycle state, resource availability, or cancellation.

```text
Work exists
   ↓
Eligible?
   ├── no → remain pending
   └── yes → ready queue / executor
```

## 4. Scheduling policy

Scheduling policy may define priority, fairness, ordering, affinity, queueing, preemption, or cooperative execution.

These properties are independent:

```text
Priority
Fairness
Ordering
Preemption
Parallelism
Latency
Determinism
```

One property must not be inferred from another.

## 5. Concurrency model

```text
Concurrency  = multiple activities can be in progress
Parallelism  = activities execute simultaneously
Determinism  = behavior is reproducible under defined conditions
Real-time    = timing obligations have defined bounds
```

A multi-threaded executor therefore does not automatically provide deterministic or real-time behavior.

## 6. Timing model

Meaningful timing claims require a declared environment:

```text
Scheduler + executor
        +
CPU / OS / runtime configuration
        +
workload
        +
measurement method
        ↓
Timing evidence
```

Claims such as "low latency" or "real-time" without a workload, target, measurement method, and observed bounds are insufficient.

## 7. Synchronization

Concurrent scheduling requires explicit synchronization around shared state and queues. Lock-free or wait-free terminology must only be used when the implementation and evidence support the specific claim.

## 8. Starvation and overload

```text
Work arrival > service capacity
             ↓
          backlog
             ↓
 priority / fairness / drop / block / reject policy
```

A documented queue policy is not proof that the runtime enforces it under all workloads.

## 9. Cancellation and shutdown

Cancellation should prevent obsolete work from being executed where the contract permits. Shutdown must coordinate cancellation, queued work, active callbacks, and resource release.

## 10. Verification requirements

| Claim | Evidence |
|---|---|
| Scheduler API exists | Source/interface inspection |
| Work is scheduled | Executed scheduler test |
| Priority ordering | Controlled ordering test |
| Fairness | Repeated workload measurement |
| Bounded latency | Targeted benchmark with declared bound |
| Determinism | Repeated controlled execution |
| Real-time suitability | Target-specific timing validation |
| No starvation | Stress test under declared workload |
| Safe shutdown | Cancellation/lifecycle integration test |

## 11. Related documents

- [Architecture Overview](overview.md)
- [System Model](system-model.md)
- [Runtime](runtime.md)
- [IPC](ipc.md)
- [Transport](transport.md)
- [Verification](../verification/README.md)
- [Safety](../safety/README.md)
