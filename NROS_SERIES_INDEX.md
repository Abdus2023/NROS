# NROS Architecture Series — Index

This is the reading-order index for the **NROS architecture document series** (Parts I–CXXX), a continuous design narrative that begins with the ROS → NROS proposition and progressively formalizes a distributed, agent-native, evidence-based execution runtime: execution model, communication fabrics, state and persistence, scheduling, security, observability, and recovery.

The series is split across **18 markdown files** in the repository root. Each file covers a contiguous range of Parts; each Part is a self-contained architectural layer, and most conclude with numbered invariants, canonical ASCII diagrams, and a final architectural rule.

---

## Reading Order

| # | File | Parts | Theme |
|---|------|-------|-------|
| 1 | [ROS_FOUNDATION.md](ROS_FOUNDATION.md) | I | ROS background, ROS1 vs ROS2, and the NROS proposition |
| 2 | [NROS_CORE_CONCEPTS.md](NROS_CORE_CONCEPTS.md) | II–X | Execution model, runtime boundary, Rust workspace, lifecycle, transport, time, safety, actions |
| 3 | [NROS_DISTRIBUTED_SYSTEMS.md](NROS_DISTRIBUTED_SYSTEMS.md) | XI–XVI | Distribution, persistence/checkpointing, real-time execution, hardware, comms, discovery/identity |
| 4 | [NROS_CONFIGURATION_AND_TIME.md](NROS_CONFIGURATION_AND_TIME.md) | XVII–XVIII | State/config/calibration/secrets; time, clocks & deterministic execution |
| 5 | [NROS_EXECUTION_AND_SCHEDULING.md](NROS_EXECUTION_AND_SCHEDULING.md) | XIX–XXIV | Communication runtime, state & parameters, discovery, lifecycle/scheduler/executor, RT model |
| 6 | [NROS_DISTRIBUTED_AND_COMMUNICATION.md](NROS_DISTRIBUTED_AND_COMMUNICATION.md) | XXV–XXVII | Distributed runtime invariants, communication fabric, state & data fabric |
| 7 | [NROS_COMPONENT_AND_RESOURCE.md](NROS_COMPONENT_AND_RESOURCE.md) | XXVIII–XXX | Component model & manifests, resource/device fabric, scheduler & execution fabric |
| 8 | [NROS_STATE_IDENTITY_SUPERVISION.md](NROS_STATE_IDENTITY_SUPERVISION.md) | XXXI–XXXIV | State fabric, identity/capability/policy, supervision & fault domains, temporal semantics |
| 9 | [NROS_LIFECYCLE_AND_PACKAGING.md](NROS_LIFECYCLE_AND_PACKAGING.md) | XXXV–XL | Agents & intent, resources/leases, event fabric & execution ledger, comm fabric, composition, scheduler |
| 10 | [NROS_TRANSPORT_AND_MESSAGING.md](NROS_TRANSPORT_AND_MESSAGING.md) | XLI–L | The fabric decomposition: state, capability/authority, supervision, evidence, protocol, domain, resource, knowledge, intent |
| 11 | [NROS_STATE_FABRIC_AND_COORDINATION.md](NROS_STATE_FABRIC_AND_COORDINATION.md) | LI–LX | Memory/event/evidence, trust & security, consensus, RT scheduling, world model, planning, resilience, kernel architecture, state algebra |
| 12 | [NROS_PERSISTENCE_AND_RECOVERY.md](NROS_PERSISTENCE_AND_RECOVERY.md) | LXI–LXX | Federation, event log & checkpoints, execution engine, comm fabric, time/state/capability/resource/authority/evidence models |
| 13 | [NROS_SECURITY_AND_POLICY.md](NROS_SECURITY_AND_POLICY.md) | LXXI–LXXX | Failure semantics, messaging, agent-native scheduling, memory/context, leases, identity & delegation, protocol, transactions, time, fencing |
| 14 | [NROS_PLATFORM_AND_HARDWARE.md](NROS_PLATFORM_AND_HARDWARE.md) | LXXXI–XC | Work model & DAGs, protocol, agent lifecycle, coordination & teams, governance, evidence, self-healing, durable state, consensus, IPC/streaming |
| 15 | [NROS_OBSERVABILITY_AND_DIAGNOSTICS.md](NROS_OBSERVABILITY_AND_DIAGNOSTICS.md) | XCI–C | Security architecture, observability, configuration, resource accounting, scheduling theory, admission/dispatch, execution semantics, event model, three-plane model, reconciliation |
| 16 | [NROS_AGENT_AND_WORKFLOW.md](NROS_AGENT_AND_WORKFLOW.md) | CI–CX | Failure semantics & exactly-once effects, journaling, arbitration, agent contract, wire protocol, crash recovery, public API, security, evidence, fault tolerance |
| 17 | [NROS_ADVANCED_SCHEDULER.md](NROS_ADVANCED_SCHEDULER.md) | CXI–CXX | Determinism & scheduling correctness, security & trust boundaries, evidence architecture, event sourcing, linearization, agent lifecycle, work model, resource model, durable history, wire compatibility |
| 18 | [NROS_ADVANCED_EXECUTION.md](NROS_ADVANCED_EXECUTION.md) | CXXI–CXXX | Agent model & reconciliation, workflow/DAG engine, scheduler placement, state store, agent runtime protocol, workflow DSL & compilation, admission & fairness, policy engine, consistency architecture, event bus |

> Part CXXXI ("Agent Runtime & Execution Architecture") is teased at the end of file 18 and has not yet been added to the series.

---

## Detailed Contents

### 1. [ROS_FOUNDATION.md](ROS_FOUNDATION.md) — Part I
ROS background and history, ROS1 vs ROS2 comparison, and the case for NROS as a from-scratch, Rust-native successor.

### 2. [NROS_CORE_CONCEPTS.md](NROS_CORE_CONCEPTS.md) — Parts II–X
- Part II — Core concepts & execution model *(opening layer)*
- Part III — The Core Runtime Boundary
- Part IV — From Core Concepts to a Rust Workspace
- Part V — The Runtime Lifecycle
- Part VI — Transport, Communication & Distribution
- Part VII — Time, Determinism & Real-Time Semantics
- Part VIII — Safety, Isolation & Fault Containment
- Part IX — Actions, Goals & Long-Running Work
- Part X — The Unified Execution Model

### 3. [NROS_DISTRIBUTED_SYSTEMS.md](NROS_DISTRIBUTED_SYSTEMS.md) — Parts XI–XVI
- Part XI — Transport & distribution *(opening layer)*
- Part XII — State, Persistence, Checkpointing & Recovery
- Part XIII — Real-Time Execution Model
- Part XIV — Hardware & Device Model
- Part XV — Communication & Transport Architecture
- Part XVI — Discovery, Naming & Identity

### 4. [NROS_CONFIGURATION_AND_TIME.md](NROS_CONFIGURATION_AND_TIME.md) — Parts XVII–XVIII
- Part XVII — State, Configuration, Calibration & Secrets *(opening layer)*
- Part XVIII — Time, Clocks & Deterministic Execution

### 5. [NROS_EXECUTION_AND_SCHEDULING.md](NROS_EXECUTION_AND_SCHEDULING.md) — Parts XIX–XXIV
- Part XIX — Execution & scheduling *(opening layer)*
- Part XX — The Communication Runtime
- Part XXI — State, Parameters & Configuration
- Part XXII — Discovery, Identity & the Distributed Runtime
- Part XXIII — Lifecycle, Scheduler & Executor
- Part XXIV — Time, Determinism & Real-Time Model

### 6. [NROS_DISTRIBUTED_AND_COMMUNICATION.md](NROS_DISTRIBUTED_AND_COMMUNICATION.md) — Parts XXV–XXVII
- Part XXV — Distributed runtime *(opening layer)*
- Part XXVI — Communication Fabric
- Part XXVII — State & Data Fabric

### 7. [NROS_COMPONENT_AND_RESOURCE.md](NROS_COMPONENT_AND_RESOURCE.md) — Parts XXVIII–XXX
- Part XXVIII — Component model *(opening layer)*
- Part XXIX — Resource & Device Fabric
- Part XXX — Scheduler & Execution Fabric

### 8. [NROS_STATE_IDENTITY_SUPERVISION.md](NROS_STATE_IDENTITY_SUPERVISION.md) — Parts XXXI–XXXIV
- Part XXXI — State fabric *(opening layer)*
- Part XXXII — Identity, Capability & Policy Fabric
- Part XXXIII — Supervision, Fault Domains & Recovery
- Part XXXIV — Time, Deadlines & Temporal Semantics

### 9. [NROS_LIFECYCLE_AND_PACKAGING.md](NROS_LIFECYCLE_AND_PACKAGING.md) — Parts XXXV–XL
- Part XXXV — Agents, Intent, Planning & Decision Execution *(opening layer)*
- Part XXXVI — Resource Model, Ownership, Leases & Physical Effects
- Part XXXVII — Event Fabric, Causality & the Execution Ledger
- Part XXXVIII — Communication Fabric
- Part XXXIX — Runtime Composition & Lifecycle
- Part XL — Scheduler & Execution Model

### 10. [NROS_TRANSPORT_AND_MESSAGING.md](NROS_TRANSPORT_AND_MESSAGING.md) — Parts XLI–L
- Part XLI — Transport & messaging *(opening layer)*
- Part XLII — State Fabric
- Part XLIII — Capability & Authority Fabric
- Part XLIV — Supervision & Recovery Fabric
- Part XLV — Observability & Evidence Fabric
- Part XLVI — Protocol & Type Fabric
- Part XLVII — Domain & Deployment Fabric
- Part XLVIII — Resource & Allocation Fabric
- Part XLIX — Knowledge & State Fabric
- Part L — Intent & Planning Fabric

### 11. [NROS_STATE_FABRIC_AND_COORDINATION.md](NROS_STATE_FABRIC_AND_COORDINATION.md) — Parts LI–LX
- Part LI — Memory, Event & Evidence Fabric *(opening layer)*
- Part LII — Identity, Trust & Security Fabric
- Part LIII — Coordination, Consensus & Distributed Orchestration Fabric
- Part LIV — Resource, Scheduling & Real-Time Execution Fabric
- Part LV — World Model, Knowledge Graph & Belief-State Fabric
- Part LVI — Intent, Planning, Policy & Decision Fabric
- Part LVII — Supervision, Fault Model, Recovery & Resilience Fabric
- Part LVIII — Temporal & Causality Fabric
- Part LIX — Kernel Architecture — From Semantic Model to Rust Runtime
- Part LX — Canonical Object Model & State Algebra

### 12. [NROS_PERSISTENCE_AND_RECOVERY.md](NROS_PERSISTENCE_AND_RECOVERY.md) — Parts LXI–LXX
- Part LXI — Distributed Coordination & Federation *(opening layer)*
- Part LXII — State, Persistence, Event Log & Checkpoint Architecture
- Part LXIII — Execution Engine, Scheduler, Executors & Real-Time Boundaries
- Part LXIV — Communication Fabric
- Part LXV — Time Model
- Part LXVI — State Model
- Part LXVII — Capability & Action Model
- Part LXVIII — Resource & Reservation Model
- Part LXIX — Authority, Policy & Governance Model
- Part LXX — Observation, Evidence & Verification Model

### 13. [NROS_SECURITY_AND_POLICY.md](NROS_SECURITY_AND_POLICY.md) — Parts LXXI–LXXX
- Part LXXI — Failure semantics *(opening layer)*
- Part LXXII — Communication, Messaging & Distributed Execution Model
- Part LXXIII — Scheduler, Work Graph & Agent-Native Scheduling
- Part LXXIV — State, Memory, Context & Checkpoint Architecture
- Part LXXV — Resource, Capability, Lease & Allocation Architecture
- Part LXXVI — Identity, Authority, Trust & Delegation Architecture
- Part LXXVII — Protocol, Message & Distributed Communication Architecture
- Part LXXVIII — State Machines, Event Log, Transactions & Deterministic Recovery
- Part LXXIX — Temporal Model & Time Architecture
- Part LXXX — Resource Model, Allocation, Leases, Capabilities & Fencing

### 14. [NROS_PLATFORM_AND_HARDWARE.md](NROS_PLATFORM_AND_HARDWARE.md) — Parts LXXXI–XC
- Part LXXXI — Work Model, Tasks, Attempts, DAGs, Dependencies & Execution Semantics *(opening layer)*
- Part LXXXII — Messaging & Protocol Model
- Part LXXXIII — Agent Model & Agent Lifecycle
- Part LXXXIV — Agent Coordination, Teams, Negotiation & Distributed Control
- Part LXXXV — Policy, Governance, Safety & Admission
- Part LXXXVI — Evidence, Observability, Provenance & Runtime Truth
- Part LXXXVII — Recovery, Fault Tolerance, Checkpointing & Self-Healing
- Part LXXXVIII — Persistence, State Machines, Transactions & Durable Runtime State
- Part LXXXIX — Distributed Coordination, Consensus, Ownership & Scheduling
- Part XC — Communication, Messaging, Protocols, IPC, Streaming & Inter-Agent Interaction

### 15. [NROS_OBSERVABILITY_AND_DIAGNOSTICS.md](NROS_OBSERVABILITY_AND_DIAGNOSTICS.md) — Parts XCI–C
- Part XCI — Security Architecture, Trust Boundaries, Identity, Secrets & Capability Security *(opening layer)*
- Part XCII — Observability, Telemetry, Tracing, Diagnostics & Explainability
- Part XCIII — Configuration, Runtime Parameters, Feature Flags & Reconfiguration
- Part XCIV — Resource Model, Capacity, Allocation, Reservations, Quotas & Accounting
- Part XCV — Scheduling Theory, Placement, Priorities, Fairness & Preemption
- Part XCVI — Execution Admission, Dispatch, Leases & Work Lifecycle
- Part XCVII — Execution Semantics, Concurrency, Cancellation & Failure Recovery
- Part XCVIII — Event Model, Evidence, Causality, Provenance & Deterministic Reconstruction
- Part XCIX — Control Plane, Data Plane & Evidence Plane
- Part C — State Authority, Reconciliation & Recovery Protocol

### 16. [NROS_AGENT_AND_WORKFLOW.md](NROS_AGENT_AND_WORKFLOW.md) — Parts CI–CX
- Part CI — Failure Semantics & Exactly-Once Effects *(opening layer)*
- Part CII — Durable State, Journaling & Storage Semantics
- Part CIII — Scheduling Semantics & Resource Arbitration
- Part CIV — Agent Execution Contract & Data-Plane Semantics
- Part CV — Protocol & Wire-Level Semantics
- Part CVI — Durable State, Event Log & Crash-Recovery Semantics
- Part CVII — Public API & Control-Plane Surface
- Part CVIII — Security & Trust Architecture
- Part CIX — Observability, Telemetry & Evidence Architecture
- Part CX — Reliability, Fault Tolerance & Recovery Architecture

### 17. [NROS_ADVANCED_SCHEDULER.md](NROS_ADVANCED_SCHEDULER.md) — Parts CXI–CXX
- Part CXI — Determinism, Scheduling Semantics & Runtime Correctness *(opening layer)*
- Part CXII — Security Architecture, Identity, Authorization & Trust Boundaries
- Part CXIII — Observability, Telemetry, Tracing & Evidence Architecture
- Part CXIV — Persistence, Event Sourcing, Snapshots & Recovery Semantics
- Part CXV — Concurrency Control, Scheduling Consistency, Distributed Coordination & Linearization Semantics
- Part CXVI — Agent Lifecycle, Capability Negotiation, Heartbeats, Liveness, Failure Detection & Reconciliation
- Part CXVII — Work Model, Job/Task Semantics, Dependencies, DAG Execution, Priorities, Deadlines, Retries & Cancellation
- Part CXVIII — Resource Model, Capacity Accounting, Reservations, Pools, Affinity, Placement, Quotas & Multi-Tenant Scheduling
- Part CXIX — Persistence, Event Sourcing, State Materialization, Transactions, Snapshots, Recovery, Compaction & Durable History
- Part CXX — Protocol & Messaging Model, Commands, Events, Envelopes, Correlation, Delivery Semantics, Backpressure, Ordering, Idempotency & Wire Compatibility

### 18. [NROS_ADVANCED_EXECUTION.md](NROS_ADVANCED_EXECUTION.md) — Parts CXXI–CXXX
- Part CXXI — Agent Model, Registration, Identity, Incarnation, Capabilities, Health, Heartbeats, Leases, Execution Control & Reconciliation *(opening layer)*
- Part CXXII — Workflow & Dependency Engine, DAG Semantics, Conditions, Gates, Fan-Out/Fan-In, Retries, Compensation, Deadlines & Workflow Recovery
- Part CXXIII — Scheduler Architecture, Placement, Resource Accounting, Fairness, Priority, Queues, Admission Control, Preemption, Capacity, Bin-Packing & Scheduling Correctness
- Part CXXIV — Persistence & State Store Architecture, Transactions, Event Log, Snapshots, Indexes, Concurrency Control, Recovery, Durability, Compaction & Consistency
- Part CXXV — Agent Runtime & Execution Protocol, Registration, Heartbeats, Capabilities, Command Delivery, Execution Lifecycle, Sandboxing, Process Supervision, Checkpointing, Result Reporting & Agent Recovery
- Part CXXVI — Workflow Definition & DSL Architecture, Schema, Versioning, Validation, DAG Semantics, Expressions, Parameters, Templates, Conditions, Loops, Fan-Out/Fan-In, Dynamic Graphs & Compilation
- Part CXXVII — Scheduler Architecture, Admission Control, Queueing, Priority, Fairness, Resource Matching, Reservations, Backfilling, Preemption, Deadlines, Quotas, Multi-Tenancy & Scheduling Correctness
- Part CXXVIII — Policy Engine & Governance Architecture, Authorization, Admission Policies, RBAC/ABAC, Tenancy, Resource Governance, Security Policies, Approval Gates, Policy Versioning, Evaluation Semantics, Overrides, Auditability & Enforcement
- Part CXXIX — State Store & Consistency Architecture, Event Sourcing, Durable State Machines, Transactions, Optimistic Concurrency, Leases, Locks, Snapshots, Projections, Idempotency, Exactly-Once Effects, Recovery & Reconciliation
- Part CXXX — Event Bus, Messaging & Distributed Coordination Architecture

---

## Thematic Arc

The series evolves through roughly four phases:

1. **Robotics runtime foundation (I–XXIV)** — replacing ROS with a Rust-native runtime: execution model, transport, time/determinism, real-time semantics, hardware.
2. **Semantic fabric decomposition (XXV–LX)** — the runtime is decomposed into named fabrics (communication, state, capability, supervision, evidence, protocol, resource, knowledge, intent), culminating in the kernel architecture and state algebra (Sₜ₊₁ = Reduce(Sₜ, Eₜ)).
3. **Agent-native distributed substrate (LXI–CX)** — federation, durable state, agent contracts, wire protocols, public APIs, security/trust, evidence, and recovery for an agent execution platform.
4. **Distributed-systems correctness (CXI–CXXX)** — determinism, linearization, leases/epochs/fencing, work & workflow models, resource accounting, policy governance, state-store consistency, and messaging — each layer closed with explicit invariants.

Recurring principles throughout: **UNKNOWN is a first-class state**, **evidence over observation**, **authority requires epochs and fencing**, **restart ≠ recovery**, **replay ≠ re-execution**, **priority ≠ authority**, and **absence of evidence is never evidence of success**.

---

## Related Repository Documents

These pre-existing documents are separate from the numbered series: [README.md](README.md), [DESIGN.md](DESIGN.md), [COMPARISON.md](COMPARISON.md), [EVIDENCE_REGISTRY.md](EVIDENCE_REGISTRY.md), [REPOSITORY_REPRESENTATION.md](REPOSITORY_REPRESENTATION.md), the `AUDIT*.md` files, and the contents of `docs/`.
