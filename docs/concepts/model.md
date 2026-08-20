# NROS Conceptual Model

> **Status:** Active conceptual documentation.
>
> This page defines vocabulary and conceptual boundaries used by the NROS documentation. It does not by itself establish implementation maturity or runtime guarantees.

## 1. What NROS is

NROS is a Rust-oriented robotics systems project whose documentation describes a layered runtime, communication, hardware-abstraction, simulation, tooling, and safety model.

The conceptual model separates **what the system is intended to provide** from **what the current repository demonstrates**.

## 2. Core concepts

### Node

A node is a logical participant in the robotics computation graph. A node owns or coordinates application-facing entities such as publishers, subscribers, services, actions, parameters, and lifecycle state where those facilities are implemented.

### Entity

An entity is a communication or runtime object associated with a node. Examples include publishers, subscribers, services, actions, timers, and parameters.

An entity name in documentation is not evidence that the corresponding backend is fully implemented.

### Message

A message is typed data exchanged between participants through a communication mechanism. Message definitions, serialization, ownership, and transport behavior are separate concerns.

### Topic

A topic represents a named publish/subscribe communication channel. Topic semantics describe the contract; the concrete transport determines how data is actually delivered.

### Service

A service represents request/response interaction. A documented service interface does not imply that a production service backend exists.

### Action

An action represents a longer-running operation with request, feedback, result, and cancellation semantics where supported.

### Executor / Scheduler

Execution infrastructure determines when callbacks, timers, lifecycle transitions, and other work are run. A scheduling abstraction is distinct from proof of deterministic or hard-real-time timing.

### Transport

Transport is the mechanism used to move data between communicating entities. Transport, serialization, ownership, memory allocation, and scheduling must be considered separately when evaluating performance.

### Hardware Abstraction

The hardware-abstraction layer defines boundaries between runtime/application logic and target-specific devices or peripherals. An abstraction can exist before every target has a concrete driver.

### Simulation

Simulation provides a controlled environment for development and validation. Simulation evidence is valuable, but it does not establish physical-hardware behavior by itself.

### Evidence

Evidence is an observation that supports a specific claim at a specific repository revision and environment. Evidence strength must match the claim being made.

## 3. Documentation state model

NROS documentation uses the following maturity vocabulary:

```text
PROPOSED
   ↓
SPECIFIED
   ↓
SCAFFOLDED
   ↓
SIMULATED
   ↓
IMPLEMENTED
   ↓
TESTED
   ↓
BENCHMARKED
   ↓
INTEGRATION-TESTED
   ↓
HARDWARE-VALIDATED
   ↓
PRODUCTION-READY
```

These are evidence states, not merely labels of documentation completeness.

## 4. Separation of concerns

The NROS model deliberately separates:

```text
Concept
  ↓
Architecture
  ↓
Specification
  ↓
Implementation
  ↓
Verification
  ↓
Validation
```

A concept can be documented without being implemented. An implementation can exist without having benchmark evidence. A benchmark can exist without hardware validation.

## 5. Claim discipline

The following implications are intentionally invalid:

```text
API exists             → feature is production-ready
Rust implementation   → real-time guarantee
Simulation passes     → hardware validated
Benchmark exists      → universal performance guarantee
Safety design exists  → safety certification
Historical PASS        → current PASS
```

The verification and evidence documentation determine the strongest defensible claim.

## 6. Related documentation

- [Architecture](../architecture/README.md)
- [Specifications](../specifications/README.md)
- [Reference](../reference/README.md)
- [Verification](../verification/README.md)
- [Safety](../safety/README.md)
- [Repository Representation](../REPOSITORY_REPRESENTATION.md)
- [Evidence Registry](../../EVIDENCE_REGISTRY.md)
