# Part I — ROS Foundation & NROS Proposition

> **Series:** NROS Architecture Series  
> **Part:** I  
> **Role:** Foundational architecture and motivation  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

NROS begins with a simple architectural question:

> **What should a robotics runtime become if execution, communication, state, resources, timing, safety, and distributed coordination are treated as first-class runtime concerns?**

The answer is not to reproduce ROS with a different programming language. NROS uses the strengths of a Rust-native systems foundation to reconsider the runtime boundary itself.

This Part establishes the historical and conceptual foundation from which the later NROS architecture series develops.

## 2. What ROS Provides

The Robot Operating System (ROS) is a robotics software framework and middleware ecosystem. Despite its name, ROS is not an operating system.

ROS provides abstractions and tooling for building distributed robot software, including:

- computational nodes;
- message-based communication;
- services and actions;
- parameters and configuration;
- discovery and graph introspection;
- hardware and device integration;
- package/build tooling;
- recording and replay;
- visualization and debugging;
- simulation integration;
- a broad ecosystem of robotics algorithms and packages.

The central contribution is an architectural vocabulary that allows independently developed software components to cooperate as a robotic system.

## 3. ROS as a Distributed Runtime Model

A useful abstraction is:

```text
Applications
     │
     ▼
Robotics Components / Nodes
     │
     ▼
Communication + Discovery + Execution APIs
     │
     ▼
Middleware / Transport
     │
     ▼
Host OS / Runtime
     │
     ▼
Hardware
```

ROS therefore occupies a middleware/runtime layer above the operating system and below most robot applications.

The exact implementation differs between ROS generations.

## 4. ROS 1 and ROS 2 Must Be Distinguished

The historical ROS 1 architecture should not be generalized to ROS as a whole.

| Concern | ROS 1 | ROS 2 |
|---|---|---|
| Discovery | ROS Master + XML-RPC | DDS-based discovery |
| Core transport model | ROS-specific transports | DDS/RTPS through RMW |
| Topics | Yes | Yes |
| Services | Yes | Yes |
| Actions | `actionlib` | First-class ROS 2 actions |
| Parameters | Central parameter server | Node-local parameter model |
| QoS | Limited | Extensive DDS QoS |
| Build ecosystem | catkin | ament + colcon |
| C++ client | roscpp | rclcpp |
| Python client | rospy | rclpy |
| Launch | XML-oriented | Python/XML/YAML support |
| Composition | nodelets | composable nodes |
| Real-time support | External integration required | Designed with real-time use cases in mind |

This distinction matters because NROS is inspired by the **problem domain and architectural lessons of ROS**, not by a requirement to preserve ROS 1 implementation mechanisms.

## 5. The ROS Computation Graph

The ROS graph provides a useful mental model:

```text
        ┌─────────┐             ┌─────────┐
        │ Node A  │── publish ─▶│ Topic X │
        └─────────┘             └────┬────┘
                                     │
                                  subscribe
                                     │
                                     ▼
                                ┌─────────┐
                                │ Node B  │
                                └─────────┘
```

But the graph is richer than topic edges alone. A complete robotics graph can include:

- publishers/subscribers;
- service servers/clients;
- action servers/clients;
- parameters;
- discovery relationships;
- lifecycle state;
- component composition;
- hardware interfaces.

The graph abstraction is therefore a useful **logical model**, not a complete description of runtime execution.

## 6. Why the ROS Model Is Valuable

ROS established several durable architectural ideas:

### 6.1 Componentization

Robot functionality can be decomposed into independently developed components.

### 6.2 Message-oriented communication

Data exchange can be modeled through typed interfaces rather than direct function coupling.

### 6.3 Distributed execution

Components can execute on different processes and machines while participating in one logical system.

### 6.4 Hardware abstraction

Applications can interact with standardized interfaces instead of depending directly on every device implementation.

### 6.5 Tooling around execution

Recording, replay, visualization, introspection, simulation, and launch/configuration tooling are part of the practical robotics development environment.

These principles remain important to NROS.

## 7. Where the NROS Question Begins

The NROS proposition starts where a conventional middleware abstraction becomes insufficient for the desired runtime model.

The architecture asks whether the runtime should treat the following as first-class concerns rather than external conventions:

```text
Execution
Lifecycle
Scheduling
Communication
Time
State
Resources
Capabilities
Authority
Persistence
Recovery
Security
Observability
Evidence
Distributed coordination
Hardware boundaries
```

The objective is therefore broader than:

```text
ROS + Rust
```

A better formulation is:

```text
ROS architectural lessons
        +
Rust-native systems foundation
        +
explicit execution semantics
        +
stronger state/resource/authority models
        +
verification-aware architecture
        ↓
NROS
```

## 8. NROS Proposition

NROS is proposed as a **Rust-native robotics and distributed execution architecture** in which the runtime itself provides stronger foundations for deterministic execution, communication, state management, resource control, lifecycle management, distributed coordination, and evidence-aware operation.

The proposition is architectural. It does not imply that every capability described by the series currently exists in the repository.

The repository's implementation and verification documentation determines the current state of those capabilities.

## 9. Architectural Shift

The conceptual shift can be summarized as:

```text
ROS-centric view

Application
    ↓
Nodes
    ↓
Middleware
    ↓
Operating System
    ↓
Hardware
```

versus the broader NROS target model:

```text
Applications / Agents
          ↓
     Workflows / APIs
          ↓
   Runtime Execution Model
          ↓
 ┌─────────────────────────┐
 │ Scheduling              │
 │ Communication           │
 │ Lifecycle               │
 │ State                   │
 │ Resources               │
 │ Capabilities / Authority│
 │ Persistence / Recovery  │
 │ Security / Policy       │
 │ Observability / Evidence│
 └─────────────────────────┘
          ↓
     Hardware / OS
```

The second model makes runtime semantics explicit instead of leaving many of them to application conventions or external infrastructure.

## 10. Rust as a Systems Foundation

NROS uses Rust because its systems-level properties are relevant to the intended runtime boundary, including:

- ownership and borrowing;
- strong type checking;
- explicit concurrency models;
- memory-safety guarantees without a tracing garbage collector;
- predictable resource ownership;
- low-level control suitable for embedded and systems programming;
- a modern package/build ecosystem.

Rust does **not** automatically provide determinism, real-time guarantees, safety qualification, or correct distributed behavior.

Those properties still require architectural constraints, implementation discipline, measurement, and verification.

```text
Rust
  ≠
Real-time guarantee

Rust
  ≠
Distributed correctness

Rust
  ≠
Safety qualification
```

This distinction is fundamental to NROS claim discipline.

## 11. NROS Runtime Boundary

The NROS architecture progressively moves functionality that is essential to execution correctness toward an explicit runtime boundary.

Conceptually:

```text
┌───────────────────────────────────────────┐
│ Applications / Robotics Algorithms        │
├───────────────────────────────────────────┤
│ NROS APIs / Nodes / Components / Agents   │
├───────────────────────────────────────────┤
│ NROS Runtime                               │
│                                           │
│ execution │ scheduling │ lifecycle        │
│ communication │ state │ resources         │
│ capabilities │ authority │ recovery       │
├───────────────────────────────────────────┤
│ OS / HAL / Drivers / Hardware             │
└───────────────────────────────────────────┘
```

This is a design boundary, not a statement that every box is already implemented.

## 12. Core Architectural Questions

The subsequent Parts of the series progressively answer questions such as:

1. What is the fundamental execution unit?
2. How are workloads scheduled?
3. How are lifecycle transitions represented?
4. How is time modeled?
5. How do components communicate?
6. How is state persisted and recovered?
7. How are resources represented and allocated?
8. How are capabilities and authority constrained?
9. How does distributed coordination work?
10. How are failures detected and recovered?
11. How are security and policy enforced?
12. How is runtime behavior observed?
13. How is evidence attached to system behavior?
14. How are hardware-specific properties separated from portable abstractions?
15. How can architectural claims be verified rather than merely described?

These questions motivate the Parts that follow.

## 13. Architectural Invariants

Part I establishes the following principles for the series:

### A1 — Middleware is not an operating system

NROS should not reproduce the historical ambiguity of the ROS name. The runtime boundary must be explicit.

### A2 — Architecture is not implementation

A design document describes intended behavior; source code establishes what exists.

### A3 — Implementation is not verification

A feature existing in source does not establish that its behavior is correct.

### A4 — Measurement is not a universal guarantee

A benchmark establishes measured behavior under defined conditions. It does not automatically establish a worst-case property.

### A5 — Simulation is not physical validation

A simulated hardware or distributed environment must not be represented as equivalent to physical execution without appropriate evidence.

### A6 — Rust is a foundation, not a guarantee

Memory safety and type safety do not eliminate the need for real-time, concurrency, distributed-systems, or safety analysis.

### A7 — Claims require scope

Every strong technical claim must identify the evidence and scope that justify it.

## 14. Relationship to the Repository

Part I is architectural context.

Current implementation status must be established from repository artifacts and verification records:

```text
Architecture Series
        ↓
Design intent
        ↓
Repository source
        ↓
Tests / CI / benchmarks / simulation / hardware
        ↓
Verification
        ↓
Validation
        ↓
Claim
```

The authoritative verification framework is documented under `docs/verification/`.

The repository-wide representation model is `docs/REPOSITORY_REPRESENTATION.md`.

## 15. What Part I Does Not Claim

This Part does **not** by itself claim that NROS currently provides:

- a production real-time runtime;
- a certified safety system;
- production-grade distributed consensus;
- universal zero-copy execution;
- complete hardware support;
- production telemetry;
- deterministic execution under all workloads;
- qualification for any particular robotic platform.

Those claims require implementation-specific evidence.

## 16. Transition to Part II

Part I establishes the proposition.

The next stage is to define the **NROS core execution model**: what executes, what owns execution state, how runtime entities interact, and where lifecycle and scheduling semantics begin.

```text
Part I
ROS foundation + NROS proposition
          ↓
Part II
NROS core concepts
          ↓
Part III+
Execution, lifecycle, communication,
time, safety, distribution, and runtime fabrics
```

## Canonical Rule

> **NROS is not defined by reproducing ROS mechanisms. It is defined by the runtime semantics and system guarantees the architecture intends to make explicit, composable, and verifiable.**
