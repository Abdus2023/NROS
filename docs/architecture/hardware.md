# NROS Hardware Architecture

> **Status:** Active architectural documentation.
>
> This document defines the boundary between portable NROS software and target-specific hardware. It does not imply driver availability, physical support, DMA integration, or hardware validation.

## 1. Hardware boundary

The hardware architecture separates application/runtime concerns from device-specific mechanisms:

```text
Application / Runtime
        ↓
NROS Hardware Abstraction
        ↓
Adapter / Driver Interface
        ↓
OS / HAL / Firmware
        ↓
Physical Device
```

The abstraction should expose stable capabilities while keeping target-specific implementation details below the boundary.

## 2. Device categories

The logical hardware model may include:

- sensors;
- actuators;
- communication peripherals;
- clocks and timers;
- memory and buffers;
- storage;
- power/control interfaces;
- platform-specific services.

A category in the architecture is not evidence that a corresponding device driver exists.

## 3. Capability model

Hardware support should be described at the capability level:

```text
Capability declared
      ↓
Interface available
      ↓
Adapter implemented
      ↓
Target integrated
      ↓
Hardware tested
      ↓
Validated under workload
```

These states must remain distinguishable.

## 4. Ownership and lifetime

Hardware resources have lifetimes that may cross runtime boundaries. Documentation should identify ownership of:

- device handles;
- DMA-capable buffers;
- mapped memory;
- interrupts;
- timers;
- device sessions;
- shutdown/reinitialization state.

Resource ownership must be compatible with the runtime lifecycle and concurrency model.

## 5. DMA and zero-copy

DMA, shared buffers, and zero-copy require an end-to-end chain of evidence:

```text
Allocation
   ↓
Physical / DMA capability
   ↓
Mapping
   ↓
Ownership / synchronization
   ↓
Device transfer
   ↓
Consumer lifetime
```

A DMA buffer type or HAL interface alone does not establish physical DMA, DMA-BUF, cache coherency, or zero-copy behavior.

## 6. Interrupts and timing

Interrupt-driven operation introduces target-specific timing and synchronization concerns. A documented interrupt interface does not establish interrupt latency or real-time guarantees.

Meaningful timing evidence must identify the target, OS/runtime configuration, workload, measurement method, and observed bounds.

## 7. Simulation boundary

Simulated devices can implement the same logical interface as physical adapters:

```text
                ┌── Physical Adapter ── Driver ── Device
NROS HAL ───────┤
                └── Simulation Adapter
```

This supports deterministic development and testing, but:

> **Simulation compatibility does not establish physical-hardware compatibility.**

## 8. Safety boundary

Hardware control may have consequences outside the software process. Safety documentation must therefore identify relevant failure modes, safe states, watchdog behavior, fault handling, and external safety mechanisms where applicable.

A software abstraction is not itself a safety certification.

## 9. Target support

Target support should be reported explicitly by target and capability rather than with broad statements such as "hardware supported."

A useful representation is:

| Target | Adapter | Driver | Integration | Hardware validation |
|---|---|---|---|---|
| Target A | Unknown | Unknown | Unknown | Not established |
| Target B | Unknown | Unknown | Unknown | Not established |

This table is intentionally evidence-driven; values must be populated only from repository and hardware evidence.

## 10. Verification requirements

| Claim | Evidence |
|---|---|
| HAL interface exists | Source/interface inspection |
| Adapter exists | Concrete implementation inspection |
| Device communicates | Target integration test |
| DMA works | Target-specific DMA evidence |
| Zero-copy works | End-to-end ownership/path evidence |
| Interrupt timing is bounded | Target timing measurement |
| Device failure is handled | Fault-injection or hardware test |
| Safety behavior is effective | Appropriate validation evidence |
| Hardware is production-ready | Target-specific integration and operational evidence |

## 11. Related documents

- [Architecture Overview](overview.md)
- [System Model](system-model.md)
- [Runtime](runtime.md)
- [Scheduling](scheduling.md)
- [Simulation](simulation.md)
- [Specifications](../specifications/README.md)
- [Verification](../verification/README.md)
- [Safety](../safety/README.md)
