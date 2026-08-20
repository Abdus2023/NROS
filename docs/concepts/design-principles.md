# Design Principles

These principles describe the intended engineering direction of NROS. They are architectural guidance, not proof of implementation maturity.

## 1. Determinism where it matters

Time-sensitive execution should have explicit scheduling, timing, and resource semantics rather than relying on accidental behavior of general-purpose middleware.

## 2. Explicit ownership

Data ownership and lifetime should be visible in interfaces. Zero-copy designs must preserve Rust's aliasing and lifetime guarantees rather than trading safety for nominal performance.

## 3. Typed communication

Communication contracts should be represented by explicit types and validated at appropriate boundaries.

## 4. Hardware-aware abstraction

Hardware abstraction should make device capabilities visible without forcing application code to depend on one vendor or transport.

## 5. Separation of policy and mechanism

Scheduling, transport, storage, hardware access, and application policy should have clear boundaries so that each can evolve independently.

## 6. Simulation as an engineering tool

Simulation should support deterministic development, testing, replay, and experimentation. Simulated behavior must be identified as simulation and must not be presented as hardware validation.

## 7. Evidence-driven claims

Documentation should distinguish design intent, implementation, testing, benchmarking, integration testing, and hardware validation.

## 8. Safety before optimization

Performance work must not weaken memory safety, ownership guarantees, lifecycle invariants, or explicit safety gates.

## 9. Small composable components

The system should favor focused crates and interfaces over a monolithic runtime where practical.

## 10. Documentation is part of the engineering system

Specifications, implementation notes, tests, evidence, and historical records should have explicit roles and relationships. A document should not silently become authoritative merely because it exists.
