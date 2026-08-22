# Part I — ROS Foundation & NROS Proposition

> **Series:** NROS Architecture Series  
> **Part:** I  
> **Role:** Foundational architecture and motivation  
> **Status:** Architectural design document — not implementation evidence
> **Contract:** Rewritten against the series contract and repository evidence model

## 1. Purpose

NROS begins with a simple architectural question:

> **What should a robotics runtime become if execution, communication, state, resources, timing, safety, and distributed coordination are treated as first-class runtime concerns?**

The answer is not to reproduce ROS with a different programming language. NROS uses the strengths of a Rust-native systems foundation to reconsider the runtime boundary itself.

This Part establishes the historical and conceptual foundation from which the later NROS architecture series develops.

## 2. Scope

Part I defines the motivation, vocabulary, system boundary, and claim discipline for the NROS Architecture Series.

It is in scope to:

- distinguish ROS 1, ROS 2, and the NROS proposition;
- define the architectural concerns NROS intends to make explicit;
- identify the role of Rust without converting language properties into system guarantees;
- establish foundational invariants for implementation and evidence;
- map the proposition to the repository's current capability and claim records.

It is out of scope to specify detailed APIs, wire protocols, scheduling algorithms, consensus protocols, hardware drivers, or qualification procedures. Later Parts may specify those subjects, but their implementation status remains independently governed by repository evidence.

## 3. Definitions

| Term | Meaning in this Part |
|---|---|
| **ROS** | The broader Robot Operating System framework and ecosystem; a generation must be named where ROS 1 and ROS 2 differ. |
| **NROS** | The Rust-native robotics and distributed-execution architecture proposed by this series, together with the repository prototypes that implement selected portions of it. |
| **Architecture** | Intended structure, behavior, constraints, and boundaries. Architecture is not execution evidence. |
| **Runtime** | The software boundary responsible for executing and coordinating NROS entities above the host OS, drivers, and hardware. |
| **Capability** | A bounded behavior tracked by an identity and maturity state in the repository representation. |
| **Evidence** | A source, test, CI run, benchmark, simulation, or hardware record interpreted only within its declared scope. |
| **Claim** | A statement about NROS whose permitted strength is constrained by the applicable evidence record and claim policy. |
| **Validation** | Evidence that a system satisfies a defined use case in a defined environment; validation is stronger and more contextual than source presence or a unit test. |

## 4. What ROS Provides

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

## 5. ROS as a Distributed Runtime Model

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

## 6. ROS 1 and ROS 2 Must Be Distinguished

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

## 7. The ROS Computation Graph

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

## 8. Why the ROS Model Is Valuable

ROS established several durable architectural ideas:

### 8.1 Componentization

Robot functionality can be decomposed into independently developed components.

### 8.2 Message-oriented communication

Data exchange can be modeled through typed interfaces rather than direct function coupling.

### 8.3 Distributed execution

Components can execute on different processes and machines while participating in one logical system.

### 8.4 Hardware abstraction

Applications can interact with standardized interfaces instead of depending directly on every device implementation.

### 8.5 Tooling around execution

Recording, replay, visualization, introspection, simulation, and launch/configuration tooling are part of the practical robotics development environment.

These principles remain important to NROS.

## 9. Where the NROS Question Begins

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

## 10. NROS Proposition

NROS is proposed as a **Rust-native robotics and distributed execution architecture** in which the runtime itself provides stronger foundations for deterministic execution, communication, state management, resource control, lifecycle management, distributed coordination, and evidence-aware operation.

The proposition is architectural. It does not imply that every capability described by the series currently exists in the repository.

The repository's implementation and verification documentation determines the current state of those capabilities.

## 11. Architectural Shift

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

## 12. Rust as a Systems Foundation

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

## 13. NROS Runtime Boundary

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

## 14. Execution and Interaction Model

At this foundational level, NROS is modeled as a set of application or agent intentions translated into bounded runtime work:

```text
Application / agent intent
          │
          ▼
Typed API, node, component, or workflow
          │
          ▼
Admission + lifecycle + authority checks
          │
          ▼
Scheduling and execution
          │
          ├── communication / state
          ├── time / resources
          ├── persistence / recovery
          └── observability / evidence
          │
          ▼
OS, transport, driver, or hardware effect
```

Each arrow is a contract boundary. A later Part must define the semantics of that boundary before an implementation can be evaluated against it. Implementations may realize only a subset of this model, and simulated effects must remain distinguishable from physical or production effects.

## 15. Core Architectural Questions

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

## 16. Architectural Invariants

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

### A8 — Backend identity must remain visible

A simulated, scaffolded, or archival backend must not be presented as a real or authoritative runtime backend.

### A9 — Authority follows the repository model

When architecture prose, source, evidence, and claims differ, the repository authority and evidence rules determine what may currently be asserted; architectural prose continues to describe intent only.

## 17. Failure and Boundary Conditions

Part I treats the following as foundational failure modes:

| Boundary failure | Required handling |
|---|---|
| Architecture is read as current behavior | Resolve the capability through source and evidence records before making a claim. |
| ROS 1 behavior is attributed to ROS 2, or vice versa | Name the ROS generation and relevant abstraction explicitly. |
| Rust properties are promoted to real-time, distributed-correctness, or safety guarantees | Reject the inference until system-level evidence exists. |
| A simulation or scaffold is described as a real backend | Preserve backend identity and limit the claim to simulated or scaffolded behavior. |
| A benchmark result is generalized beyond its environment | Retain the measured environment, revision, method, and distribution; do not infer a universal bound. |
| A configured workflow is described as a passing workflow | Require an observed successful run tied to the represented revision. |
| A repository snapshot becomes stale | Treat this Part's table as navigation and re-resolve the canonical manifests. |

The architectural boundary also excludes application-specific algorithm correctness, host-kernel guarantees, device behavior, network behavior, and physical safety unless those properties are explicitly contracted and evidenced.

## 18. Implementation Implications

An implementation conforming to the Part I proposition should:

1. keep authoritative runtime source separate from archival demonstrations;
2. expose the identity of simulated, scaffolded, and real backends;
3. use explicit lifecycle, ownership, failure, and resource boundaries rather than relying only on application convention;
4. preserve typed boundaries where state or authority changes;
5. make unsupported capabilities fail visibly rather than silently emulating stronger behavior;
6. provide stable capability identifiers that can be connected to tests and evidence;
7. avoid API names or documentation that imply a guarantee stronger than the implementation provides.

For the current repository, `crates/` is the authoritative implementation hierarchy and `implementations/` is archival. This classification is defined by [`implementations/README.md`](implementations/README.md), not by this Part.

## 19. Verification Implications

Part I is verified primarily as a documentation and traceability contract. Verification of individual runtime properties belongs to the capability that implements them.

A Part I documentation review should establish that:

- all internal links resolve;
- every current implementation observation names an authoritative source;
- every maturity statement resolves to the capability/evidence catalogs;
- every strong claim resolves to a claim class and scope;
- simulations, benchmarks, CI configuration, and hardware evidence remain distinct;
- unsupported production, real-time, consensus, zero-copy, hardware, and safety claims are explicitly excluded.

The evidence rules in [`docs/representation/evidence.yaml`](docs/representation/evidence.yaml) are controlling: source presence is not execution evidence, configured CI is not a passed run, a benchmark is not independent validation, simulation cannot support a real-backend claim, and hardware validation requires hardware evidence.

## 20. Current Repository Reconciliation

The following table is a navigation snapshot of the current machine-readable representation. It does not replace the linked manifests.

| Part I concern | Capability record | Represented state | Claim boundary |
|---|---|---:|---|
| Guard-based SPSC communication primitive | `CORE-IPC-001` | `TESTED` | Allowed only for the tested ring buffer; excludes MPMC, shared-memory IPC, and production real-time guarantees. |
| Shared-memory IPC | `CORE-IPC-002` | `SPECIFIED` | No implementation claim. |
| Node lifecycle | `NODE-001` | `IMPLEMENTED` | Allowed with implementation scope; not a complete runtime lifecycle guarantee. |
| Sensor abstraction | `HAL-001` | `IMPLEMENTED` | Software abstraction only. |
| Real V4L2/DMA-BUF path | `HAL-002` | `SPECIFIED` | Hardware/zero-copy claim forbidden. |
| UDP transport | `TRANSPORT-001` | `IMPLEMENTED` | Basic transport scope only. |
| True zero-copy network serialization | `TRANSPORT-002` | `SCAFFOLDED` | Claim forbidden. |
| Leader-election state machine | `DIST-001` | `IMPLEMENTED` | May be described only as scaffolding. |
| Complete Raft protocol and replicated state | `DIST-002` | `SCAFFOLDED` | Consensus claim forbidden. |
| Deterministic simulation primitives | `SIM-001` | `TESTED` | Simulation scope only; no physical validation. |
| Repository evidence/claim validation | `AUDIT-001` | `IMPLEMENTED` | Tooling exists; this does not imply every repository claim has passed verification. |

Authoritative reconciliation sources:

- [`docs/representation/capabilities.yaml`](docs/representation/capabilities.yaml) — capability identities and represented states;
- [`docs/representation/evidence.yaml`](docs/representation/evidence.yaml) — evidence records and evidence rules;
- [`docs/representation/claims.yaml`](docs/representation/claims.yaml) — allowed, conditional, and forbidden claim scopes;
- [`EVIDENCE_REGISTRY.md`](EVIDENCE_REGISTRY.md) — detailed human-readable feature evidence;
- [`docs/REPOSITORY_REPRESENTATION.md`](docs/REPOSITORY_REPRESENTATION.md) — canonical representation and authority model.

If this table conflicts with a newer canonical manifest, the newer manifest controls.

## 21. Relationship to the Repository

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

The canonical evidence and claim framework is represented by `docs/representation/evidence.yaml`, `docs/representation/claims.yaml`, and `EVIDENCE_REGISTRY.md`.

The repository-wide representation model is `docs/REPOSITORY_REPRESENTATION.md`.

## 22. What Part I Does Not Claim

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

## 23. Transition to Part II

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
