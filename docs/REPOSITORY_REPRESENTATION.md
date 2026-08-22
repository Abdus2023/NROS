# NROS Repository Representation

**Status:** Canonical repository representation model

**Purpose:** Provide a repository-level representation of NROS that separates architectural intent, implementation reality, evidence, verification state, claim strength, and represented repository state.

## 1. Representation Model

NROS is represented across six dimensions:

1. **Architecture** — what the system is designed to be.
2. **Implementation** — what exists in the repository.
3. **Evidence** — what is demonstrated by source, tests, benchmarks, CI, simulation, and hardware.
4. **Verification** — what technical criteria have actually been demonstrated.
5. **Claims** — what the available evidence permits the project to assert.
6. **State** — the repository, revision, environment, and tooling state represented by the snapshot.

The representation MUST NOT infer implementation maturity from specification or architecture alone.

## 2. Source-of-Truth Hierarchy

When sources disagree, prefer the most direct and current evidence:

1. Executed verification evidence attributable to the represented revision/environment.
2. Repository source at the represented revision.
3. Evidence records and audit artifacts.
4. Current design/specification documents.
5. README and other narrative summaries.
6. Historical documents, unless explicitly being used as historical evidence.

This hierarchy does not mean that executable evidence overrides a normative requirement. It means that claims about **what exists or what happened** must be grounded in executable/repository evidence rather than narrative intent.

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

This is the **intent model**. It is not a claim that every architectural component is fully implemented or validated.

## 4. Workspace Model

The Rust workspace is represented by its actual crate topology. Current documentation identifies the following twelve roles:

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

The table describes repository topology; it does not imply equal implementation maturity across crates.

## 5. Capability State Model

Implementation/evidence maturity may be represented as:

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

The state vocabulary describes capability maturity; it does not replace the verification conclusions defined in `docs/verification/`.

## 6. Verification Conclusion Model

Verification conclusions are separate from implementation state:

```text
OBSERVED
VERIFIED
PARTIALLY VERIFIED
BLOCKED
NOT VERIFIED
FAILED
STALE
```

For validation, the corresponding bounded conclusions are:

```text
VALIDATED
PARTIALLY VALIDATED
BLOCKED
FAILED
NOT VALIDATED
STALE
```

A capability being `IMPLEMENTED` does not make it `VERIFIED`, and a verified technical criterion does not automatically make the whole system `VALIDATED`.

## 7. Capability Record

Each significant capability should be representable as:

```text
Capability
├── identity
├── specification reference
├── implementation paths
├── implementation state
├── tests
├── CI evidence
├── benchmark evidence
├── simulation evidence
├── hardware evidence
├── verification conclusion
├── validation conclusion
├── invariants
├── limitations
└── claim policy
```

Example:

```text
CORE-IPC-001
  Capability: SPSC ring buffer
  Specification: relevant specification/design section
  Implementation: crates/nros-core
  Tests: present
  Benchmark: non-gating benchmark evidence
  Hardware: N/A
  Implementation state: TESTED
  Verification: VERIFIED within tested scope
  Claim: allowed with stated scope
```

## 8. Claim Discipline

Claims are derived from evidence, not design intent.

### Allowed

A claim is allowed when the required evidence exists and its scope is explicit.

### Conditional

A claim is conditional when implementation exists but important verification or validation remains incomplete. Examples may include simulated DMA, simulated distributed election, or synthetic Studio telemetry.

### Not verified / unsupported

A claim must remain unverified when the feature is only specified, scaffolded, simulated without the required physical validation, or otherwise lacks evidence appropriate to the claim.

Examples requiring explicit scope include:

- real V4L2/DMA-BUF hardware integration;
- real Raft protocol and replicated state;
- true end-to-end zero-copy network serialization;
- production live telemetry;
- safety qualification;
- independent validation of headline performance numbers.

## 9. Evidence Graph

The canonical traceability path is:

```text
Requirement / use case
        |
        v
Design / specification
        |
        v
Capability / claim
        |
        v
Implementation
        |
        +--> Unit / component test
        |
        +--> Integration / E2E test
        |
        +--> CI execution
        |
        +--> Miri / concurrency validation
        |
        +--> Benchmark
        |
        +--> Simulation / replay
        |
        +--> Hardware validation
        |
        v
Observation + evidence
        |
        v
Verification / validation conclusion
        |
        v
Claim decision
```

Missing links MUST remain visible. Narrative documentation must not silently fill them.

## 10. Evidence Object

Evidence should be attributable through the repository verification model:

```text
Evidence
├── id
├── claim
├── class
├── repository revision
├── source/artifact
├── execution context
├── method
├── observation
├── result
├── provenance
├── timestamp
├── freshness
└── limitations
```

Evidence levels are descriptive:

```text
E0  No supporting evidence
E1  Documentary/source evidence
E2  Automated local execution
E3  Integration/target execution
E4  Reproducible controlled validation
E5  Explicit qualification/acceptance evidence
```

The level does not replace the underlying artifact.

## 11. Repository State

A representation snapshot MUST record at minimum:

```text
repository: Abdus2023/NROS
branch: <represented branch>
commit: <represented commit SHA>
workspace_crates: <observed count>
rust_edition: <observed edition>
verification_timestamp: <UTC timestamp>
CI_workflow: .github/workflows/ci.yml
```

The commit SHA is mandatory for a reproducible representation.

## 12. CI Representation

CI is evidence only when the workflow actually executes.

The representation MUST distinguish:

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

A toolchain change must be recorded explicitly. A specialized verification requirement must not silently redefine the project's baseline toolchain.

## 13. Repository Topology vs Architecture

The following are separate models:

```text
DESIGN.md / architecture docs
  = architectural intent

Cargo workspace
  = code topology

Runtime graph
  = execution topology

Evidence registry / manifests
  = evidence topology

CI / tests / Miri / benchmarks / hardware
  = verification topology
```

The representation exists to connect these models without conflating them.

## 14. Required Invariants

1. **No specification implies implementation.**
2. **No source existence implies correctness.**
3. **No passing unit test implies production readiness.**
4. **No benchmark artifact implies independently verified performance.**
5. **No simulated backend may be represented as a real backend.**
6. **No CI configuration may be represented as executed evidence until the workflow runs.**
7. **No hardware capability may be represented as hardware-validated without actual hardware evidence.**
8. **Every strong public claim must resolve to an evidence record or be explicitly marked unverified.**
9. **Historical evidence must remain attributable to its represented revision/environment.**
10. **Validation claims require explicit acceptance criteria.**

## 15. Machine-Readable Representation

The canonical model should ultimately be represented by structured manifests such as:

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

Structured manifests MUST be generated or checked against the actual repository rather than becoming another manually maintained description.

## 16. Related Documentation

- [Architecture](architecture/README.md)
- [Specifications](specifications/README.md)
- [Reference](reference/README.md)
- [Verification](verification/README.md)
- [Safety](safety/README.md)
- [Operations](operations/README.md)
- [Documentation Inventory](documentation/inventory.yaml)
