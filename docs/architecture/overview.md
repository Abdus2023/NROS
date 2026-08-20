# Architecture Overview

> **Status:** Active architectural documentation.
>
> This document defines the system-level architectural decomposition. It describes responsibilities and dependencies; implementation maturity is established separately through repository evidence and verification.

## 1. System model

NROS is described as a layered robotics runtime in which application behavior is separated from execution, communication, hardware, simulation, and tooling concerns.

```text
┌───────────────────────────────────────┐
│ Applications / Robot Programs         │
├───────────────────────────────────────┤
│ Nodes / Entities / Public Interfaces   │
├───────────────────────────────────────┤
│ Runtime Services / Execution           │
├───────────────────────────────────────┤
│ Communication / IPC / Transport       │
├───────────────────────────────────────┤
│ Hardware Abstraction / Adapters       │
├───────────────────────────────────────┤
│ OS / Drivers / Physical Hardware      │
└───────────────────────────────────────┘
```

Simulation and development tooling surround this core model rather than being treated as proof of physical deployment:

```text
                 ┌───────────────┐
                 │  Simulation   │
                 └───────┬───────┘
                         │
Application ── Runtime ── Communication ── Hardware
                         │
                 ┌───────┴───────┐
                 │    Tooling    │
                 └───────────────┘
```

## 2. Responsibilities

### Applications

Robot programs express domain behavior using the public runtime model. Applications should not need to know the implementation details of every transport or hardware adapter.

### Nodes and entities

Nodes provide logical participants in the computation graph. Entities such as publishers, subscribers, services, actions, timers, and parameters belong to the runtime-facing model where implemented.

### Runtime and execution

The runtime coordinates lifecycle, execution, scheduling, and integration between application entities and lower layers. Timing guarantees are a separate verification concern.

### Communication and transport

The communication layer defines logical data exchange. IPC and network transport determine how that exchange is realized. Serialization, ownership, allocation, copying, and scheduling must not be conflated.

### Hardware abstraction

Hardware abstraction defines stable boundaries for device-specific implementations. A generic interface can exist independently of complete driver coverage for every target.

### Operating system and hardware

OS facilities, drivers, peripherals, and physical devices are outside the portable runtime abstraction. Hardware claims therefore require target-specific evidence.

### Simulation

Simulation provides controlled substitutes for selected physical resources and enables development, testing, and replay. Simulation results do not automatically establish physical-hardware behavior.

### Tooling

CLI, Studio, diagnostics, and observability surfaces provide development and operational interfaces. A documented interface is not evidence that a live provider or production telemetry path exists.

## 3. Dependency direction

The intended dependency direction is:

```text
Application
    ↓
Public Runtime Model
    ↓
Execution / Communication Services
    ↓
Transport / Hardware Adapters
    ↓
OS / Drivers / Hardware
```

Lower layers may provide mechanisms to higher layers, but higher-level documentation must not assume capabilities that the lower layers do not actually implement.

## 4. Replaceability

Transport and hardware mechanisms should remain behind explicit interfaces where practical. This allows development and validation against alternative implementations without changing the logical application model.

Replaceability does not imply that every alternative backend is currently available or production-ready.

## 5. Determinism and observability

Deterministic behavior is an architectural goal that must be decomposed into measurable properties such as scheduling behavior, allocation behavior, communication timing, and target characteristics.

Observability is similarly a system concern spanning runtime events, diagnostics, telemetry, and tooling. Individual telemetry interfaces require implementation evidence before they can be described as operational.

## 6. Evidence boundary

The architectural model must be read together with the evidence model:

```text
Architecture
   ↓ defines structure
Specification
   ↓ defines required behavior
Implementation
   ↓ provides mechanism
Verification
   ↓ establishes observed behavior
Validation
   ↓ establishes target/environment suitability
```

Therefore:

> **Architecture describes what NROS is designed to be; verification establishes what the repository demonstrates at a specific revision.**

See [Verification](../verification/README.md) and [Evidence Registry](../../EVIDENCE_REGISTRY.md).

## 7. Related architecture documents

- [Architecture Map](README.md)
- [System Model](system-model.md)
- [Runtime](runtime.md)
- [IPC](ipc.md)
- [Scheduling](scheduling.md)
- [Transport](transport.md)
- [Hardware](hardware.md)
- [Distributed](distributed.md)
- [Simulation](simulation.md)
- [Studio](studio.md)
