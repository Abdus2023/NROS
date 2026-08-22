# NROS Architecture Series — Index

> **Series status:** Parts **I–CXXX** are currently indexed across 18 architecture documents.
>
> **Purpose:** This file is the canonical reading-order and navigation index for the NROS Architecture Series. The series describes architectural intent and design evolution; it is **not, by itself, evidence that every described capability is implemented or validated**.

---

## 1. What This Series Is

The NROS Architecture Series is a continuous design narrative that begins with the ROS → NROS proposition and progressively develops a Rust-native, distributed, agent-oriented execution runtime.

The series covers:

- execution and lifecycle semantics;
- communication and transport fabrics;
- state, persistence, and recovery;
- scheduling and real-time behavior;
- resources, capabilities, and authority;
- distributed coordination and consensus;
- agents, workflows, and execution protocols;
- security, policy, and governance;
- observability, evidence, and diagnostics;
- hardware and deployment boundaries.

The series is an **architecture/design corpus**. Implementation maturity and verification status are governed by the repository's implementation and verification documentation.

---

## 2. Reading Order

| # | File | Parts | Primary theme |
|---:|---|---|---|
| 1 | [ROS_FOUNDATION.md](ROS_FOUNDATION.md) | I | ROS background and the NROS proposition |
| 2 | [NROS_CORE_CONCEPTS.md](NROS_CORE_CONCEPTS.md) | II–X | Core execution model, runtime boundary, lifecycle, transport, time, safety, actions |
| 3 | [NROS_DISTRIBUTED_SYSTEMS.md](NROS_DISTRIBUTED_SYSTEMS.md) | XI–XVI | Distribution, persistence, real-time execution, hardware, communication, discovery |
| 4 | [NROS_CONFIGURATION_AND_TIME.md](NROS_CONFIGURATION_AND_TIME.md) | XVII–XVIII | Configuration, calibration, secrets, clocks, deterministic time |
| 5 | [NROS_EXECUTION_AND_SCHEDULING.md](NROS_EXECUTION_AND_SCHEDULING.md) | XIX–XXIV | Communication runtime, state, discovery, lifecycle, scheduler, executor, RT model |
| 6 | [NROS_DISTRIBUTED_AND_COMMUNICATION.md](NROS_DISTRIBUTED_AND_COMMUNICATION.md) | XXV–XXVII | Distributed runtime, communication fabric, state/data fabric |
| 7 | [NROS_COMPONENT_AND_RESOURCE.md](NROS_COMPONENT_AND_RESOURCE.md) | XXVIII–XXX | Components, manifests, resources, devices, scheduling |
| 8 | [NROS_STATE_IDENTITY_SUPERVISION.md](NROS_STATE_IDENTITY_SUPERVISION.md) | XXXI–XXXIV | State, identity, capability, policy, supervision, recovery, temporal semantics |
| 9 | [NROS_LIFECYCLE_AND_PACKAGING.md](NROS_LIFECYCLE_AND_PACKAGING.md) | XXXV–XL | Agents, intent, resources, leases, events, composition, lifecycle, scheduling |
| 10 | [NROS_TRANSPORT_AND_MESSAGING.md](NROS_TRANSPORT_AND_MESSAGING.md) | XLI–L | State, authority, supervision, evidence, protocol, deployment, resources, knowledge, intent fabrics |
| 11 | [NROS_STATE_FABRIC_AND_COORDINATION.md](NROS_STATE_FABRIC_AND_COORDINATION.md) | LI–LX | Memory, evidence, trust, consensus, scheduling, world model, planning, resilience, kernel/state algebra |
| 12 | [NROS_PERSISTENCE_AND_RECOVERY.md](NROS_PERSISTENCE_AND_RECOVERY.md) | LXI–LXX | Federation, event log, checkpoints, execution, time/state/capability/resource/authority/evidence models |
| 13 | [NROS_SECURITY_AND_POLICY.md](NROS_SECURITY_AND_POLICY.md) | LXXI–LXXX | Failure semantics, messaging, scheduling, memory, leases, identity, delegation, transactions, time, fencing |
| 14 | [NROS_PLATFORM_AND_HARDWARE.md](NROS_PLATFORM_AND_HARDWARE.md) | LXXXI–XC | Work/DAG model, agents, coordination, governance, evidence, recovery, durable state, consensus, IPC |
| 15 | [NROS_OBSERVABILITY_AND_DIAGNOSTICS.md](NROS_OBSERVABILITY_AND_DIAGNOSTICS.md) | XCI–C | Security, observability, configuration, resources, scheduling, admission, execution, evidence, reconciliation |
| 16 | [NROS_AGENT_AND_WORKFLOW.md](NROS_AGENT_AND_WORKFLOW.md) | CI–CX | Failure semantics, journaling, arbitration, agent contract, protocol, recovery, API, security, evidence |
| 17 | [NROS_ADVANCED_SCHEDULER.md](NROS_ADVANCED_SCHEDULER.md) | CXI–CXX | Determinism, scheduling correctness, security, evidence, event sourcing, lifecycle, resources, history, protocol |
| 18 | [NROS_ADVANCED_EXECUTION.md](NROS_ADVANCED_EXECUTION.md) | CXXI–CXXX | Agents, workflows, scheduler, state store, runtime protocol, DSL, policy, consistency, event bus |

---

## 3. Series Map

```text
I–X
  Foundation + Core Runtime
       ↓
XI–XXIV
  Distribution + Time + Execution + Scheduling
       ↓
XXV–XL
  Distributed / Communication / Component / Resource / Lifecycle Fabrics
       ↓
XLI–LX
  Transport + State + Authority + Evidence + Coordination + Kernel Model
       ↓
LXI–LXXX
  Persistence + Recovery + Security + Policy + Deterministic Failure Semantics
       ↓
LXXXI–C
  Platform + Hardware + Observability + Diagnostics + Reconciliation
       ↓
CI–CXX
  Agent Execution + Workflow + Advanced Scheduling
       ↓
CXXI–CXXX
  Advanced Agent Runtime + Workflow + Scheduler + State + Policy + Event Bus
```

The series therefore evolves from **robotics runtime foundations** toward a more general **agent-native execution architecture**.

---

## 4. The Four Architectural Eras

### Era I — Runtime Foundation

**Parts I–XXIV** establish the basic NROS proposition:

- why NROS exists;
- execution model;
- Rust runtime boundary;
- lifecycle;
- communication;
- time and determinism;
- safety boundaries;
- hardware abstraction;
- discovery;
- scheduling and real-time semantics.

### Era II — Fabric Architecture

**Parts XXV–LXXX** decompose the runtime into interacting fabrics and durable system models:

- communication;
- state;
- capability and authority;
- supervision;
- evidence;
- protocol;
- resources;
- knowledge;
- intent;
- persistence;
- recovery;
- security;
- policy;
- temporal semantics.

### Era III — Platform and Agent Architecture

**Parts LXXXI–CXX** move toward an agent-native platform model:

- work and DAG semantics;
- agent lifecycle;
- coordination;
- governance;
- observability;
- evidence;
- self-healing;
- event sourcing;
- deterministic reconstruction;
- advanced scheduling;
- resource arbitration;
- protocol compatibility.

### Era IV — Advanced Execution Architecture

**Parts CXXI–CXXX** consolidate the agent/runtime execution model:

- agent registration and reconciliation;
- workflow/DAG execution;
- scheduler placement and admission;
- durable state;
- agent execution protocol;
- workflow DSL and compilation;
- policy evaluation;
- consistency architecture;
- distributed event coordination.

---

## 5. Architectural Status Boundary

The series MUST be read using the following distinction:

```text
Architecture Series
        │
        │ describes
        ▼
Architectural intent / design
        │
        ├───────────────┐
        ▼               ▼
Implementation      Verification
        │               │
        │               ├── tests
        │               ├── CI
        │               ├── benchmarks
        │               ├── simulation
        │               └── hardware
        │
        └───────────────┐
                        ▼
                 Validation / Claims
```

Therefore:

```text
Specified
   ≠ Implemented
   ≠ Tested
   ≠ Verified
   ≠ Validated
   ≠ Qualified
```

A Part may define a capability long before the repository implements or verifies it.

---

## 6. Relationship to Repository Documentation

The Architecture Series is one layer of the documentation system.

| Documentation layer | Primary question |
|---|---|
| Architecture Series | What is NROS designed to become? |
| Architecture docs | How is the system structured? |
| Specifications | What normative behavior is required? |
| Reference | What interfaces/types/configuration are exposed? |
| Implementation | What actually exists in source? |
| Verification | What has been demonstrated? |
| Validation | Does the system satisfy its defined use case? |
| Claims | What can the repository responsibly assert? |
| Repository Representation | How do all of these models connect? |

This prevents the architecture series from becoming an accidental substitute for implementation or verification evidence.

---

## 7. Part-Level Contract

Each Part should ideally contain:

```text
Purpose
  ↓
Scope
  ↓
Definitions
  ↓
Architecture
  ↓
Execution / interaction model
  ↓
Invariants
  ↓
Failure / boundary conditions
  ↓
Implementation implications
  ↓
Verification implications
  ↓
Architectural rule
```

When a Part makes a strong implementation, performance, safety, or hardware assertion, that assertion SHOULD be explicitly marked as one of:

- architectural requirement;
- design proposal;
- implementation observation;
- verified behavior;
- validated behavior;
- open/unverified claim.

---

## 8. CXXXI and the Next Series Boundary

The previous index ended with a teaser for **Part CXXXI — Agent Runtime & Execution Architecture**.

That teaser should no longer be treated as a completed Part. The current indexed series ends at **CXXX** until a new Part is actually added as a repository artifact.

The next Part should therefore be created only when its architecture is sufficiently distinct from CXXI–CXXX to justify extending the series.

Candidate continuation themes include:

```text
CXXXI
  Agent Runtime & Execution Architecture
       ↓
CXXXII+
  Runtime/Kernel convergence
  executable contracts
  evidence-native execution
  deployment/qualification boundaries
```

These are roadmap concepts, not current implementation claims.

---

## 9. Navigation Rules

Use this index when entering the series from the beginning.

For implementation-oriented work, start instead from:

- [Repository Representation](docs/REPOSITORY_REPRESENTATION.md)
- [Capability Evidence](EVIDENCE_REGISTRY.md)
- [Machine-readable Evidence](docs/representation/evidence.yaml)
- [Claim Policy](docs/representation/claims.yaml)
- [Architecture](docs/ARCHITECTURE.md)
- [Implementation Map](implementations/README.md)

For a specific architectural topic, use the Part map above rather than treating the 130 Parts as one undifferentiated document.

---

## 10. Current Series Summary

```text
Architecture documents: 18
Indexed Parts:          I–CXXX
Latest indexed Part:    CXXX
Next Part:              CXXXI (not yet added)
Series role:            Architectural design corpus
Evidence role:          Navigation / design context only
Evidence authority:     EVIDENCE_REGISTRY.md + docs/representation/evidence.yaml
Claim authority:        docs/representation/claims.yaml
Repository model:       docs/REPOSITORY_REPRESENTATION.md
```

> **Canonical rule:** The Architecture Series explains the architecture. The repository, evidence, verification, and validation records determine what NROS can currently claim about that architecture.
