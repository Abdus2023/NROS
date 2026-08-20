# Simulation Architecture

## Purpose

The NROS simulation layer provides a controlled environment for developing and evaluating robot behavior without requiring physical hardware for every iteration.

## Conceptual model

```text
Simulation World
      │
      ├── Physics
      ├── Sensors
      ├── Robot State
      ├── Time
      └── Replay
            │
            ▼
        NROS Interfaces
            │
            ▼
       Robot Application
```

## Determinism

Simulation should make time, physics, sensor generation, and replay behavior explicit. A deterministic simulation claim requires controlled inputs, time semantics, and repeatable outputs; the existence of a fixed-step loop alone is not sufficient evidence.

## Simulation versus hardware

Simulation is a development and verification aid. It must not be presented as equivalent to physical-hardware validation unless the relevant hardware behavior has independently been validated.

## Current status

The repository contains simulation-oriented implementation material. Specific capabilities and their maturity are tracked by verification evidence rather than inferred from this architectural description.
