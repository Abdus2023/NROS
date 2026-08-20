# NROS Audit

> **Documentation status:** Historical verification entry point.
>
> This file records earlier repository-level verification work. It is retained for traceability and must be interpreted against the branch and revision identified by each audit pass.

## Current audit authority

For the active documentation rewrite, use:

- [Documentation Integrity Audit](docs/verification/documentation-integrity.md)
- [Evidence Model](docs/verification/evidence-model.md)
- [Claims](docs/verification/claims.md)
- [Validation](docs/verification/validation.md)
- [Evidence Registry](EVIDENCE_REGISTRY.md)
- [Repository Representation](docs/REPOSITORY_REPRESENTATION.md)

## Historical-audit rule

An audit result is evidence about the repository state that was actually inspected. It is not automatically a statement about the current branch.

Every historical audit should therefore be interpreted using:

```text
Audit claim
    ↓
Represented branch
    ↓
Represented revision
    ↓
Observed source/tests/workflows
    ↓
Result
```

A historical PASS may be reused only after the relevant evidence is shown to remain valid or is re-executed.

## Evidence terminology

The repository uses the following progression for capability maturity:

`PROPOSED → SPECIFIED → SCAFFOLDED → SIMULATED → IMPLEMENTED → TESTED → BENCHMARKED → INTEGRATION-TESTED → HARDWARE-VALIDATED → PRODUCTION-READY`

These states are not interchangeable. In particular:

- implementation is not benchmark evidence;
- benchmark evidence is not hardware validation;
- simulation is not physical validation;
- a safety-oriented design is not safety certification;
- a passing historical audit is not current verification.

## Historical findings

Earlier audits identified real implementation in areas such as the core SPSC ring buffer, node lifecycle/parameters, HAL abstractions, transport primitives, simulation, Studio, and CLI tooling. They also identified important gaps between architectural claims and demonstrated implementation.

The historical audit specifically recorded that the repository contained genuine Rust implementation while warning that README performance and maturity claims were not independently established. fileciteturn58file0

The detailed evidence/capability matrix is retained in `EVIDENCE_REGISTRY.md`; that registry is the better machine-readable starting point for current claim reconciliation.

## Migration policy

This document is no longer the place to append new audit prose. New verification work should be recorded in focused verification documents with explicit scope, revision, evidence, and limitations.

Historical audit passes remain valuable and must not be rewritten to make their old conclusions appear current. If a historical conclusion changes, record the new verification separately and preserve the original record.

See [Documentation Migration](docs/migration/README.md) for the broader migration policy.
