# NROS Repository Representation

**Status:** Initial canonical representation

**Purpose:** Provide a repository-level representation of NROS that separates architectural intent, implementation reality, evidence, verification state, and claim strength.

## 1. Representation Model

NROS is represented across five dimensions:

1. **Architecture** — what the system is designed to be.
2. **Implementation** — what exists in the repository.
3. **Evidence** — what is demonstrated by source, tests, benchmarks, CI, and hardware.
4. **Claims** — what the available evidence permits the project to assert.
5. **State** — the repository/commit/CI/toolchain state at which the representation was generated.

The representation must never infer implementation maturity from specification alone.

## 2. Source of Truth Hierarchy

When sources disagree, use this precedence:

1. Executed verification evidence (CI, tests, Miri, hardware validation).
2. Repository source at the represented commit.
3. Evidence Registry and audit artifacts.
4. Design/specification documents.
5. README and other narrative summaries.

A stronger narrative claim must not override weaker executable evidence.

## 3. Architectural Model

```text
Application Layer
        |
High-Level APIs / Tools
        |
Core Services
        |
Communication Substrate
        |
NROS Runtime / Scheduler
        |
Hardware Abstraction Layer
```

This is the **intent model**. It is not a claim that every architectural component is fully implemented.

## 4. Workspace Model

The current Rust workspace contains twelve crates:

| Layer / Role | Crate | Representation |
|---|---|---|
| Types | `nros-types` | Canonical shared robotics types |
| Runtime | `nros-core` | Core execution and IPC primitives |
| Node model | `nros-node` | Lifecycle, parameters, execution statistics |
| Hardware | `nros-hal` | Sensor/HAL abstractions and prototypes |
| Transport | `nros-transport` | UDP/TCP and serialization primitives |
| Distributed | `nros-distributed` | Fleet/election/replication abstractions |
| CLI | `nros-cli` | Command architecture and project tooling |
| Simulation | `nros-sim` | Deterministic simulation primitives |
| Studio | `nros-studio` | HTTP dashboard and telemetry architecture |
| Macros | `nros-macros` | Procedural-macro surface/scaffolding |
| Facade | `nros` | Public aggregation/facade crate |
| Audit | `nros-audit` | Structural/evidence/claim validation tooling |

## 5. Capability State Model

Every capability should have an explicit evidence state:

```text
SPECIFIED
   -> SCAFFOLDED
   -> SIMULATED
   -> IMPLEMENTED
   -> TESTED
   -> BENCHMARKED
   -> INTEGRATION-TESTED
   -> HARDWARE-VALIDATED
   -> PRODUCTION-READY
   -> SAFETY-QUALIFIABLE
```

These states are **not automatically linear**. A capability may be IMPLEMENTED and TESTED without being BENCHMARKED or HARDWARE-VALIDATED.

## 6. Capability Record

Each capability should be representable as:

```text
Capability
├── identity
├── specification reference
├── implementation paths
├── implementation state
├── tests
├── CI evidence
├── benchmark evidence
├── hardware evidence
├── invariants
├── limitations
└── claim policy
```

Example:

```text
CORE-IPC-001
  Capability: SPSC ring buffer
  Specification: DESIGN.md §14.1
  Implementation: crates/nros-core
  Tests: present
  Benchmark: repository artifact, non-gating
  Hardware: N/A
  State: TESTED
  Claim: allowed with stated scope
```

## 7. Claim Discipline

Claims are derived from evidence, not from design intent.

### Allowed

A claim is allowed when the required evidence exists and its scope is explicit.

### Conditional

A claim is conditional when implementation exists but important validation is incomplete. Examples include simulated DMA, simulated distributed election, or synthetic Studio telemetry.

### Forbidden

A claim is forbidden when the feature is only specified, scaffolded, simulated without real validation, or otherwise lacks the evidence necessary to support the claim.

Examples that must remain explicitly bounded include:

- real V4L2/DMA-BUF hardware integration;
- real Raft protocol and replicated state;
- true zero-copy network serialization;
- production live telemetry;
- safety qualification;
- independent validation of headline performance numbers.

## 8. Evidence Graph

The canonical traceability path is:

```text
Requirement
    |
    v
Design section
    |
    v
Capability
    |
    v
Implementation
    |
    +--> Unit / integration test
    |
    +--> CI execution
    |
    +--> Miri / concurrency validation
    |
    +--> Benchmark
    |
    +--> Hardware validation
    |
    v
Claim decision
```

Missing links must remain visible. They must not be silently filled by narrative assumptions.

## 9. Repository State

A representation snapshot should record at minimum:

```text
repository: Abdus2023/NROS
branch: <represented branch>
commit: <represented commit SHA>
workspace_crates: 12
rust_edition: 2021
verification_timestamp: <UTC timestamp>
CI_workflow: .github/workflows/ci.yml
```

The commit SHA is mandatory for a reproducible representation.

## 10. CI Representation

CI is evidence only when the workflow actually executes.

The representation must distinguish:

```text
workflow defined
      !=
workflow triggered
      !=
job executed
      !=
job passed
```

Likewise:

```text
Miri configured
      !=
Miri installed
      !=
Miri executed
      !=
Miri passed
```

A toolchain change must be recorded explicitly. The project's normal stable toolchain should not be implicitly reclassified as nightly merely because a specialized verification job requires nightly.

## 11. Repository Topology vs Architecture

The following are separate models:

```text
DESIGN.md
  = architectural intent

Cargo workspace
  = code topology

Runtime graph
  = execution topology

EVIDENCE_REGISTRY.md
  = evidence topology

CI / Miri / benchmarks / hardware
  = verification topology
```

The representation exists to connect these models without conflating them.

## 12. Required Invariants

1. **No specification implies implementation.**
2. **No source existence implies correctness.**
3. **No passing unit test implies production readiness.**
4. **No benchmark artifact implies independently verified performance.**
5. **No simulated backend may be represented as a real backend.**
6. **No CI configuration may be represented as executed evidence until the workflow runs.**
7. **No hardware capability may be represented as hardware-validated without actual hardware evidence.**
8. **Every strong public claim must resolve to an evidence record.**

## 13. Next Representation Layer

The next step is to make this model machine-readable and auditable by introducing a structured capability manifest, for example:

```text
repository representation
├── docs/REPOSITORY_REPRESENTATION.md   # normative model
├── docs/representation/
│   ├── capabilities.yaml               # capability records
│   ├── architecture.yaml               # architectural model
│   ├── evidence.yaml                    # evidence links
│   └── claims.yaml                      # claim policy
└── crates/nros-audit/                   # executable consistency checks
```

The structured files should be generated or checked against the actual repository rather than becoming another manually maintained description.
