# NROS Documentation

> **Status:** Repository documentation hub.
>
> This hub separates **intent, normative requirements, implementation, usage, and evidence**. Each layer has a different purpose and authority.

## Start here

| Goal | Start with |
|---|---|
| Understand NROS concepts | [Concepts](./concepts/README.md) |
| Understand system architecture | [Architecture](./architecture/README.md) |
| Build or run NROS | [Getting Started](./getting-started/README.md) |
| Find APIs, crates, and commands | [Reference](./reference/README.md) |
| Develop NROS | [Development](./development/README.md) |
| Determine what is actually demonstrated | [Verification](./verification/README.md) |
| Understand safety boundaries | [Safety](./safety/README.md) |
| Deploy and operate NROS | [Operations](./operations/README.md) |
| Follow project decisions | [Governance](./governance/README.md) |

## Documentation model

```text
Concepts
   │
   ▼
Architecture ──────► system structure and design intent
   │
   ▼
Specifications ────► normative requirements
   │
   ▼
Implementation ────► repository state
   │
   ├───────────────► Reference / usage
   │
   ▼
Verification ──────► demonstrated properties
   │
   ▼
Validation ────────► use-case acceptance
   │
   ▼
Qualification ─────► explicit acceptance decision
```

These layers MUST NOT be treated as interchangeable evidence.

A specification says what should be true. Source establishes what exists. Tests establish only the behavior they execute. Benchmarks establish measurements under stated conditions. Validation establishes acceptance within stated scope.

## Authority model

When documents disagree, determine which statement is authoritative for the question being asked:

1. **Normative specification** — defines required behavior.
2. **Current implementation** — establishes repository state.
3. **Executable evidence** — establishes observed behavior for its revision and environment.
4. **Historical/audit material** — preserves previous findings and context.

A README, diagram, roadmap, or design statement MUST NOT upgrade implementation or verification status.

## Status vocabulary

NROS uses two deliberately separate vocabularies.

### Capability maturity

These describe implementation/evidence maturity:

- `PROPOSED`
- `SPECIFIED`
- `SCAFFOLDED`
- `SIMULATED`
- `IMPLEMENTED`
- `TESTED`
- `BENCHMARKED`
- `INTEGRATION-TESTED`
- `HARDWARE-VALIDATED`
- `PRODUCTION-READY`

These are not automatic proof chains. A capability can be implemented while its integration, performance, or hardware behavior remains unverified.

### Evidence conclusions

These describe a specific claim/evidence result:

- `OBSERVED`
- `VERIFIED`
- `PARTIALLY VERIFIED`
- `BLOCKED`
- `NOT VERIFIED`
- `FAILED`
- `STALE`

`PASS` is an execution result, not a universal documentation maturity state.

### Validation conclusions

Validation adds use-case scope:

- `VALIDATED`
- `PARTIALLY VALIDATED`
- `BLOCKED`
- `FAILED`
- `NOT VALIDATED`
- `STALE`

`VALIDATED` MUST refer to an explicit acceptance criterion.

## Evidence invariant

Significant claims should follow this chain:

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

If a link is missing, the documentation must expose the gap rather than infer it.

## Verification navigation

The verification layer is intentionally split into focused contracts:

- [Verification overview](./verification/README.md)
- [Evidence model](./verification/evidence-model.md)
- [Claims](./verification/claims.md)
- [Test strategy](./verification/test-strategy.md)
- [Benchmarks](./verification/benchmarks.md)
- [Validation](./verification/validation.md)

## Existing authoritative material

During migration, historical and specialized documents remain available:

- [Architecture](./ARCHITECTURE.md)
- [Repository Representation](./REPOSITORY_REPRESENTATION.md)
- [Safety Remediation](./SAFETY_REMEDIATION.md)
- [Threat Model](./THREAT_MODEL.md)
- [Audit material](./audit/)
- [Repository representation material](./representation/)
- [CI specification](./ci.yml)

Important root-level sources include:

- [Design](../DESIGN.md)
- [Comparison](../COMPARISON.md)
- [Audit](../AUDIT.md)
- [Evidence Registry](../EVIDENCE_REGISTRY.md)
- [Core Safety](../crates/nros-core/SAFETY.md)

These remain historical or specialized until explicitly migrated or superseded.

## Migration policy

Documentation migration follows this order:

1. Establish entry points.
2. Inventory existing material.
3. Assign authority and scope.
4. Split broad documents into focused topics.
5. Add cross-references and evidence links.
6. Verify technical claims against repository state.
7. Reconcile terminology and status vocabulary.
8. Retire or redirect obsolete documents only after their information is preserved.

Historical evidence MUST NOT be rewritten as current evidence merely to make the documentation cleaner.

## Navigation principle

Prefer the smallest document that answers the question:

- **What and why?** → Concepts
- **How is it structured?** → Architecture
- **What MUST be true?** → Specifications
- **What exists?** → Reference / implementation
- **How do I use it?** → Getting Started
- **How do I change it?** → Development
- **What proves it?** → Verification
- **Does it satisfy the intended use case?** → Validation
- **What must never be violated?** → Safety
- **How is it operated?** → Operations
- **What is the project deciding?** → Governance
