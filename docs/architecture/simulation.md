# NROS Simulation Architecture

> **Status:** Active architectural documentation.
>
> Simulation is a controlled development and validation environment. It must not be treated as automatic evidence of physical-hardware behavior.

## 1. Purpose

Simulation allows NROS applications and runtime components to execute against controlled representations of devices, time, communication, and faults.

```text
Application
    ↓
NROS Runtime
    ↓
Simulation Boundary
 ┌──┼───────────┐
 ↓  ↓           ↓
Time Devices  Transport
 ↓  ↓           ↓
 └──┴──────┬────┘
            ↓
       Simulated World
```

## 2. Simulation components

A simulation environment may contain:

- virtual nodes;
- virtual sensors and actuators;
- simulated transport;
- simulated clocks;
- deterministic event sources;
- fault injection;
- state recording and replay;
- scenario configuration.

The presence of a simulation interface does not establish that every component is implemented.

## 3. Time model

Simulation may use wall-clock time, accelerated time, paused time, or a deterministic virtual clock.

```text
Wall clock
    ≠
Virtual simulation time
    ≠
Physical system time
```

Tests that depend on timing should state which time model is active.

## 4. Determinism

A deterministic simulation aims to produce reproducible results under identical inputs and configuration.

Determinism depends on more than a virtual clock. Sources of nondeterminism can include concurrency, random seeds, unordered collections, external I/O, scheduling, and platform behavior.

Therefore:

> **A deterministic simulation environment does not prove that the physical deployment is deterministic.**

## 5. Virtual hardware

Simulation adapters can implement hardware-abstraction interfaces:

```text
                ┌── Physical Adapter ── Device
NROS HAL ───────┤
                └── Simulation Adapter ── Virtual Device
```

This enables application-level testing without requiring physical hardware, while preserving the boundary needed for later integration.

## 6. Fault injection

Simulation can provide controlled failures such as:

- sensor failure;
- actuator failure;
- transport loss;
- delayed messages;
- malformed data;
- resource exhaustion;
- process failure;
- clock anomalies.

Fault injection is valuable for testing recovery paths, but the simulated fault must correspond sufficiently to the physical failure mode before conclusions are generalized.

## 7. Replay and evidence

Recorded inputs and events can support reproducible debugging:

```text
Scenario
   ↓
Recorded inputs
   ↓
Simulation
   ↓
Observed outputs
   ↓
Evidence / regression test
```

A replay result should record the scenario, configuration, software revision, time model, and relevant environment.

## 8. Simulation versus validation

Simulation provides evidence about the simulated environment.

```text
Simulation test
      ↓
Simulation evidence
      ↓
Integration hypothesis
      ↓
Physical integration
      ↓
Hardware validation
```

Passing a simulation test must not be promoted directly to hardware validation.

## 9. Verification requirements

| Claim | Evidence |
|---|---|
| Simulation API exists | Source/interface inspection |
| Scenario executes | Automated simulation test |
| Simulation is repeatable | Repeated controlled runs |
| Virtual device behaves as specified | Scenario tests |
| Fault recovery works | Fault-injection test |
| Replay is reproducible | Recorded scenario + repeated execution |
| Physical behavior is equivalent | Separate hardware validation |
| Real-time behavior transfers to hardware | Target-specific timing validation |

## 10. Related documents

- [Architecture Overview](overview.md)
- [System Model](system-model.md)
- [Hardware](hardware.md)
- [Distributed](distributed.md)
- [Runtime](runtime.md)
- [Verification](../verification/README.md)
- [Safety](../safety/README.md)
