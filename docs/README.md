# NROS Documentation

> **Status:** Documentation hub for the NROS repository.

NROS documentation is organized by purpose rather than by the historical order in which material was written. The goal is to keep **intent, specification, implementation, usage, and evidence** distinguishable.

## Start Here

| If you want to... | Start with... |
|---|---|
| Understand NROS concepts | [Concepts](./concepts/README.md) |
| Understand the system architecture | [Architecture](./architecture/README.md) |
| Build or run the repository | [Getting Started](./getting-started/README.md) |
| Find APIs, crates, and commands | [Reference](./reference/README.md) |
| Develop NROS | [Development](./development/README.md) |
| Understand what is actually demonstrated | [Verification](./verification/README.md) |
| Understand safety constraints | [Safety](./safety/README.md) |
| Deploy or operate NROS | [Operations](./operations/README.md) |
| Follow project decisions and direction | [Governance](./governance/README.md) |

Some sections are still being established as part of the documentation rewrite. Existing material remains available during migration and is explicitly identified below.

## Documentation Model

NROS documentation uses complementary layers with different authority and purpose:

```text
Concepts
   │
   ▼
Architecture ──────► Design intent and system boundaries
   │
   ▼
Specifications ────► Normative requirements
   │
   ▼
Implementation ────► What exists in the repository
   │
   ├───────────────► Reference / usage
   │
   ▼
Verification ──────► What has actually been demonstrated
   │
   ▼
Validation / Qualification
```

A specification describes what should be true. Source describes what is implemented. Tests and other evidence demonstrate defined properties under defined conditions. Documentation MUST NOT use one layer as implicit proof of another.

## Documentation authority

When documents disagree, resolve the conflict by checking their scope and authority:

1. **Normative specification** — defines required behavior.
2. **Current implementation** — establishes repository state.
3. **Executable evidence** — establishes observed behavior for its stated environment and revision.
4. **Historical/audit material** — preserves prior findings and context but does not automatically describe the current state.

A README, architecture diagram, or roadmap cannot upgrade implementation or verification status.

## Status vocabulary

NROS uses two related but distinct vocabularies.

### Capability maturity

These describe implementation/evidence maturity and must not be treated as automatic proof chains:

- `PROPOSED` — future direction or idea.
- `SPECIFIED` — explicitly defined by a specification.
- `SCAFFOLDED` — structural implementation exists, but the capability is incomplete.
- `SIMULATED` — behavior is represented in a model or development environment.
- `IMPLEMENTED` — functional implementation exists within its stated scope.
- `TESTED` — automated tests provide evidence for defined behavior.
- `BENCHMARKED` — performance has been measured under defined conditions.
- `INTEGRATION-TESTED` — multiple components have been exercised together.
- `HARDWARE-VALIDATED` — the defined capability has been validated on identified physical hardware.
- `PRODUCTION-READY` — explicit project-defined production criteria have been satisfied.

A capability may have a mixed status across dimensions. For example, code can be `IMPLEMENTED` while hardware support remains `NOT VERIFIED`.

### Verification conclusions

These describe the conclusion for a specific evidence record or claim:

- `OBSERVED`
- `VERIFIED`
- `PARTIALLY VERIFIED`
- `BLOCKED`
- `NOT VERIFIED`
- `FAILED`
- `STALE`

`PASS` is not a universal maturity state.

## Evidence rule

The core documentation invariant is:

```text
Claim
  ↓
Requirement / rationale
  ↓
Implementation
  ↓
Verification method
  ↓
Observation
  ↓
Evidence
  ↓
Conclusion
  ↓
Limitations
```

If evidence is missing, blocked, stale, or outside the claim's scope, the documentation must say so.

## Existing authoritative material

During migration, existing documents remain part of the knowledge base:

- [Architecture](./ARCHITECTURE.md)
- [Repository Representation](./REPOSITORY_REPRESENTATION.md)
- [Safety Remediation](./SAFETY_REMEDIATION.md)
- [Threat Model](./THREAT_MODEL.md)
- [Audit material](./audit/)
- [Repository representation material](./representation/)
- [CI specification](./ci.yml)

At the repository root, important historical/specification sources include:

- [Design](../DESIGN.md)
- [Comparison](../COMPARISON.md)
- [Audit](../AUDIT.md)
- [Evidence Registry](../EVIDENCE_REGISTRY.md)
- [Core Safety](../crates/nros-core/SAFETY.md)

These sources retain their historical or specialized role until explicitly migrated or superseded.

## Documentation migration

The rewrite follows this sequence:

1. Establish documentation entry points.
2. Inventory and classify existing material.
3. Assign authority and scope.
4. Split broad documents into focused topics.
5. Add cross-references and evidence links.
6. Validate technical claims against repository state.
7. Reconcile terminology and status vocabulary.
8. Retire or redirect obsolete documents only after their information has been preserved.

Migration is intentionally incremental. Historical audit and evidence records are preserved rather than rewritten as if they were current normative specifications.

## Navigation principle

Prefer the smallest document that answers the question:

- **What and why?** → Concepts
- **How is it structured?** → Architecture
- **What MUST be true?** → Specifications
- **What exists?** → Reference / implementation documentation
- **How do I use it?** → Getting Started
- **How do I change it?** → Development
- **What proves it?** → Verification
- **Does it satisfy the intended use case?** → Validation
- **What must never be violated?** → Safety
- **How is it operated?** → Operations
- **What is the project deciding?** → Governance
