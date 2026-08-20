# NROS — Native Robotics Operating System

NROS is an experimental robotics software stack exploring a simpler, more deterministic alternative to conventional robotics middleware. The project focuses on explicit communication semantics, real-time-oriented execution, hardware abstraction, simulation, developer tooling, and safety-conscious verification.

> **Project status:** NROS is under active development. The repository contains a mixture of specifications, scaffolding, simulations, implementations, and verification artifacts. A documented capability must not be assumed to be production-ready merely because it appears in the architecture or roadmap.

## What NROS Is

NROS is organized around a small set of architectural concerns:

- **Core runtime and communication** — execution, messaging, and low-level communication primitives.
- **Node model** — lifecycle, parameters, deadlines, and application-facing behavior.
- **Hardware abstraction** — interfaces intended to separate robotics software from hardware-specific implementations.
- **Transport and distributed operation** — local and network communication, with distributed-system functionality developed incrementally.
- **Simulation** — deterministic physics-oriented and sensor abstractions for development and testing.
- **Developer tooling** — CLI and Studio components for project creation, inspection, and experimentation.
- **Verification and safety** — explicit evidence, safety gates, testing, and audit artifacts.

## Current Evidence Model

NROS distinguishes design intent from repository evidence.

| Status | Meaning |
|---|---|
| `PROPOSED` | Future direction or idea. |
| `SPECIFIED` | Defined by an explicit design/specification. |
| `SCAFFOLDED` | Structural implementation exists, but the intended capability is incomplete. |
| `SIMULATED` | Behavior is represented for development or demonstration rather than provided by the real subsystem. |
| `IMPLEMENTED` | Functional implementation exists in the repository. |
| `TESTED` | Automated tests provide evidence for the implementation. |
| `BENCHMARKED` | Performance measurements have been collected. |
| `INTEGRATION-TESTED` | Multiple components have been verified together. |
| `HARDWARE-VALIDATED` | Behavior has been validated against the relevant physical hardware. |
| `PRODUCTION-READY` | The project-defined production criteria have been satisfied. |

Higher status must not be inferred without the corresponding evidence.

## Architecture at a Glance

```text
┌──────────────────────────────────────────┐
│             Robot Applications           │
├──────────────────────────────────────────┤
│          APIs / CLI / NROS Studio        │
├──────────────────────────────────────────┤
│             Core Services                │
├──────────────────────────────────────────┤
│       Communication / Transport          │
├──────────────────────────────────────────┤
│        Core Runtime / Execution          │
├──────────────────────────────────────────┤
│       Hardware Abstraction Layer         │
└──────────────────────────────────────────┘
```

The architecture is specified more deeply in the design and architecture documentation. The implementation status of individual components must be read together with their evidence and verification records.

## Repository Status

The repository currently contains both implemented functionality and experimental/scaffolded components. Examples include:

- `nros-core` — core communication/runtime primitives with safety-oriented work and tests.
- `nros-node` — node lifecycle, parameters, deadline monitoring, and related runtime behavior.
- `nros-hal` — hardware-abstraction work, including simulated DMA-oriented components.
- `nros-transport` — UDP/TCP and transport experimentation, with some capabilities still simulated or scaffolded.
- `nros-distributed` — distributed-system structures with simulated/incomplete consensus behavior.
- `nros-cli` — CLI architecture and project-generation functionality.
- `nros-sim` — deterministic simulation-oriented functionality and replay work.
- `nros-studio` — development/inspection UI with both implemented infrastructure and simulated data paths.
- `nros-types`, `nros-macros`, `nros` — supporting types, macros, and facade layers.
- `nros-audit` — repository-oriented verification and claim-analysis tooling.

For authoritative status, consult the verification and evidence documentation rather than relying on this summary alone.

## Documentation

Start with the documentation hub:

- **[Documentation](./docs/README.md)** — documentation map and recommended reading paths.
- **[Architecture](./docs/ARCHITECTURE.md)** — current architecture overview.
- **[Repository Representation](./docs/REPOSITORY_REPRESENTATION.md)** — repository knowledge/representation model.
- **[Safety Remediation](./docs/SAFETY_REMEDIATION.md)** — safety remediation record.
- **[Threat Model](./docs/THREAT_MODEL.md)** — security and threat model.
- **[Design Specification](./DESIGN.md)** — detailed historical/current design specification.
- **[Comparison](./COMPARISON.md)** — NROS and ROS2 comparison material.
- **[Audit](./AUDIT.md)** — repository audit and verification history.
- **[Evidence Registry](./EVIDENCE_REGISTRY.md)** — feature, implementation, test, benchmark, and claim evidence.
- **[Core Safety](./crates/nros-core/SAFETY.md)** — safety-related constraints and implementation notes for the core crate.

## Building and Testing

NROS is a Rust workspace. With an appropriate Rust toolchain installed, begin with:

```bash
cargo check --workspace
cargo test --workspace
```

For component-specific examples, tests, CI requirements, and verification procedures, use the documentation and repository workflows rather than treating the commands above as a complete validation procedure.

## Repository Layout

```text
NROS/
├── README.md
├── DESIGN.md
├── COMPARISON.md
├── AUDIT.md
├── EVIDENCE_REGISTRY.md
├── docs/                 # Project documentation and verification material
├── crates/               # Rust workspace crates
├── implementations/      # Implementation/reference artifacts
├── examples/             # Examples, where present
└── .github/              # CI and repository automation
```

## Development Direction

NROS is being developed incrementally. Near-term work centers on strengthening the existing foundations, separating real implementations from simulations and scaffolding, improving verification coverage, and making the documentation accurately reflect repository state.

Longer-term architectural work includes deeper hardware integration, distributed operation, robotics applications, richer tooling, safety qualification work, and other capabilities described by the project specifications. Those directions are **not claims of current availability**.

## Contributing

Contributions should preserve the distinction between specification, implementation, and evidence. When adding a capability:

1. Define or update the relevant specification.
2. Implement the smallest verifiable increment.
3. Add appropriate tests and evidence.
4. Update the corresponding status and documentation.
5. Do not describe simulated or scaffolded behavior as production functionality.

See the repository documentation for the detailed development and verification workflow.

## License and Project Links

Refer to the repository metadata and current project documentation for authoritative licensing and community information.

---

**NROS documentation principle:** describe what the repository can demonstrate today, distinguish it from what it specifies for tomorrow, and make the evidence for important claims discoverable.