# Part I — ROS Foundation & NROS Proposition

> **Series:** NROS Architecture Series  
> **Part:** I  
> **Role:** Foundational architecture and motivation  
> **Canonical source:** `ROS_FOUNDATION.md`  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

NROS begins with a simple architectural question:

> **What should a robotics runtime become if execution, communication, state, resources, timing, safety, and distributed coordination are treated as first-class runtime concerns?**

The answer is not to reproduce ROS with a different programming language. NROS uses the strengths of a Rust-native systems foundation to reconsider the runtime boundary itself.

This Part establishes the historical and conceptual foundation from which the later NROS architecture series develops.

## 2. ROS Foundation

The Robot Operating System (ROS) is a robotics software framework and middleware ecosystem. Despite its name, ROS is not an operating system.

ROS provides abstractions and tooling for distributed robot software, including:

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
- a broad robotics ecosystem.

Its durable contribution is an architectural vocabulary allowing independently developed components to cooperate as one robotic system.

## 3. ROS 1 and ROS 2

NROS distinguishes ROS generations rather than treating historical ROS 1 mechanisms as universal ROS properties.

| Concern | ROS 1 | ROS 2 |
|---|---|---|
| Discovery | ROS Master + XML-RPC | DDS-based discovery |
| Transport | ROS-specific transports | DDS/RTPS through RMW |
| Topics | Yes | Yes |
| Services | Yes | Yes |
| Actions | `actionlib` | First-class ROS 2 actions |
| Parameters | Central parameter server | Node-local parameter model |
| QoS | Limited | Extensive DDS QoS |
| Build ecosystem | catkin | ament + colcon |
| C++ client | roscpp | rclcpp |
| Python client | rospy | rclpy |
| Composition | nodelets | composable nodes |

NROS is inspired by the robotics problem domain and architectural lessons of ROS, not by preservation of ROS 1 implementation mechanisms.

## 4. ROS Computation Graph

A simplified ROS graph is:

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

A complete logical graph may also contain services, actions, parameters, discovery relationships, lifecycle state, component composition, and hardware interfaces.

The graph is therefore a logical model, not a complete execution model.

## 5. Durable ROS Architectural Lessons

ROS establishes several ideas that remain valuable to NROS:

### Componentization

Robot functionality can be decomposed into independently developed components.

### Message-oriented communication

Typed interfaces can decouple producers and consumers.

### Distributed execution

Components can execute across processes and machines while participating in one logical system.

### Hardware abstraction

Applications can use standardized interfaces instead of directly depending on every device implementation.

### Development tooling

Recording, replay, visualization, introspection, simulation, launch, and configuration form an important robotics development environment.

## 6. Why NROS Exists

The NROS proposition begins where a conventional middleware abstraction becomes insufficient for a runtime model that needs explicit semantics for:

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

The objective is broader than:

```text
ROS + Rust
```

A more accurate formulation is:

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

## 7. NROS Proposition

NROS is proposed as a **Rust-native robotics and distributed execution architecture** in which the runtime itself provides stronger foundations for deterministic execution, communication, state management, resource control, lifecycle management, distributed coordination, and evidence-aware operation.

This is an architectural proposition. It does not imply that every capability described by the series currently exists in the repository.

Implementation and verification artifacts determine the actual state of each capability.

## 8. Architectural Shift

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

## 9. Rust as Systems Foundation

Rust is relevant to the intended runtime boundary because of properties including:

- ownership and borrowing;
- strong type checking;
- explicit concurrency models;
- memory safety without a tracing garbage collector;
- predictable resource ownership;
- low-level systems and embedded control;
- a modern package/build ecosystem.

Rust does not automatically provide determinism, real-time guarantees, safety qualification, or distributed correctness.

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

Those properties still require architectural constraints, implementation discipline, measurement, and verification.

## 10. NROS Runtime Boundary

Conceptually:

```text
┌───────────────────────────────────────────┐
│ Applications / Robotics Algorithms        │
├───────────────────────────────────────────┤
│ NROS APIs / Nodes / Components / Agents   │
├───────────────────────────────────────────┤
│ NROS Runtime                              │
│ execution │ scheduling │ lifecycle        │
│ communication │ state │ resources         │
│ capabilities │ authority │ recovery       │
├───────────────────────────────────────────┤
│ OS / HAL / Drivers / Hardware             │
└───────────────────────────────────────────┘
```

This is a design boundary, not a claim that every box is already implemented.

## 11. Core Architectural Questions

The subsequent Parts progressively answer:

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

## 12. Architectural Invariants

### A1 — Middleware is not an operating system

The runtime boundary must be explicit.

### A2 — Architecture is not implementation

A design document describes intended behavior; source code establishes what exists.

### A3 — Implementation is not verification

A feature existing in source does not establish that its behavior is correct.

### A4 — Measurement is not a universal guarantee

A benchmark establishes measured behavior under defined conditions; it does not automatically establish a worst-case property.

### A5 — Simulation is not physical validation

Simulation must not be represented as physical execution evidence without appropriate validation.

### A6 — Rust is a foundation, not a guarantee

Memory safety and type safety do not eliminate real-time, concurrency, distributed-systems, or safety analysis.

### A7 — Claims require scope

Strong technical claims must identify the evidence and scope that justify them.

## 13. Documentation and Evidence Boundary

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

Part I is architectural context. It does not substitute for implementation or verification evidence.

## 14. What Part I Does Not Claim

This Part does not by itself claim that NROS currently provides:

- a production real-time runtime;
- a certified safety system;
- production-grade distributed consensus;
- universal zero-copy execution;
- complete hardware support;
- production telemetry;
- deterministic execution under all workloads;
- qualification for any particular robotic platform.

Those claims require implementation-specific evidence.

## 15. Transition to Part II

Part I establishes the proposition.

Part II defines the NROS core execution vocabulary: what executes, what owns execution state, how runtime entities interact, and where lifecycle and scheduling semantics begin.

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
