# Architecture Overview

NROS is organized around a layered runtime model in which application code depends on explicit services and communication interfaces rather than directly coupling every component to hardware or transport details.

```text
┌───────────────────────────────────────┐
│ Applications / Robot Programs         │
├───────────────────────────────────────┤
│ APIs, CLI, Studio and Developer Tools │
├───────────────────────────────────────┤
│ Core Services                         │
├───────────────────────────────────────┤
│ Communication / IPC / Transport       │
├───────────────────────────────────────┤
│ Runtime / Scheduling                  │
├───────────────────────────────────────┤
│ Hardware Abstraction                  │
├───────────────────────────────────────┤
│ OS / Drivers / Hardware              │
└───────────────────────────────────────┘
```

## Architectural goals

The architecture is intended to provide:

- explicit execution and lifecycle boundaries;
- efficient local communication;
- replaceable transport mechanisms;
- hardware-independent interfaces where practical;
- deterministic simulation and replay capabilities; and
- observable runtime behavior.

## Component boundaries

### Runtime

Owns execution, lifecycle, scheduling, and runtime coordination.

### Communication

Defines the mechanisms through which components exchange typed data locally or across a network.

### Hardware abstraction

Provides stable interfaces between robotics software and device-specific implementations.

### Simulation

Provides controlled virtual environments for development, testing, and replay.

### Studio

Provides development-time visualization, inspection, and telemetry interfaces.

## Evidence boundary

The architecture represents the intended system organization. Individual components can have different implementation states. For example, a repository component may expose an API while an underlying hardware mechanism remains simulated or scaffolded.

Therefore:

> Architecture describes what the system is designed to be; verification establishes what the repository currently demonstrates.

See [Verification](../verification/README.md).
