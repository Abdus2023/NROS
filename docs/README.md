# NROS Documentation

This is the documentation hub for NROS.

NROS documentation is organized by purpose rather than by the historical order in which material was written. The goal is to make architecture, implementation status, usage, and verification easy to distinguish and navigate.

## Start Here

| If you want to... | Start with... |
|---|---|
| Understand NROS | [Concepts](./concepts/README.md) |
| Understand the system architecture | [Architecture](./architecture/README.md) |
| Build or run the repository | [Getting Started](./getting-started/README.md) |
| Find APIs, crates, and commands | [Reference](./reference/README.md) |
| Develop NROS | [Development](./development/README.md) |
| Understand what is actually verified | [Verification](./verification/README.md) |
| Understand safety constraints | [Safety](./safety/README.md) |
| Deploy or operate NROS | [Operations](./operations/README.md) |
| Follow project decisions and direction | [Governance](./governance/README.md) |

Some of these sections are being established as part of the documentation rewrite. Existing documents remain available during migration and are linked below.

## Documentation Model

The documentation uses four complementary layers:

```text
Concepts
   │
   ▼
Architecture
   │
   ▼
Specifications ───────► Implementation
                              │
                              ▼
                         Verification
                              │
                              ▼
                            Claims
```

A specification describes intended behavior. An implementation demonstrates repository behavior. Verification provides evidence about that behavior. Documentation claims should be supported by the strongest available evidence.

## Evidence Status

NROS uses the following vocabulary when describing capability maturity:

- `PROPOSED` — future direction or idea.
- `SPECIFIED` — explicitly defined by a specification.
- `SCAFFOLDED` — structural implementation exists, but the capability is incomplete.
- `SIMULATED` — behavior is represented for development or demonstration.
- `IMPLEMENTED` — functional implementation exists.
- `TESTED` — automated tests provide evidence.
- `BENCHMARKED` — performance has been measured.
- `INTEGRATION-TESTED` — multiple components have been verified together.
- `HARDWARE-VALIDATED` — validated against relevant physical hardware.
- `PRODUCTION-READY` — project-defined production criteria have been satisfied.

A higher status must not be inferred without corresponding evidence.

## Existing Authoritative Material

During the rewrite, the existing documents remain part of the knowledge base:

- [Architecture](./ARCHITECTURE.md)
- [Repository Representation](./REPOSITORY_REPRESENTATION.md)
- [Safety Remediation](./SAFETY_REMEDIATION.md)
- [Threat Model](./THREAT_MODEL.md)
- [Audit material](./audit/)
- [Repository representation material](./representation/)
- [CI specification](./ci.yml)

At the repository root, the main historical/specification sources include:

- [Design](../DESIGN.md)
- [Comparison](../COMPARISON.md)
- [Audit](../AUDIT.md)
- [Evidence Registry](../EVIDENCE_REGISTRY.md)
- [Core Safety](../crates/nros-core/SAFETY.md)

## Documentation Migration

The rewrite follows this sequence:

1. Establish the documentation entry points.
2. Inventory and classify existing documents.
3. Assign authority and scope to each document.
4. Split broad documents into focused topics.
5. Add cross-references and evidence links.
6. Validate technical claims against repository state.
7. Retire or redirect obsolete documents only after their information has been preserved.

The migration is intentionally incremental. Existing audit and evidence records are preserved rather than rewritten as if they were current normative specifications.

## Navigation Principle

Prefer the smallest document that answers the question:

- **What and why?** → Concepts
- **How?** → Architecture
- **What exactly is specified?** → Specifications
- **How do I use it?** → Getting Started
- **What is the API?** → Reference
- **How do I change it?** → Development
- **What proves it?** → Verification
- **What must never be violated?** → Safety
- **How is it operated?** → Operations
- **What is the project deciding or planning?** → Governance
