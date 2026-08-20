# NROS Evidence / Capability Registry

> **Status:** Active evidence registry.
>
> This document is the human-readable capability/evidence authority for the documentation rewrite. It records what repository evidence supports and what claims are permitted.

## Purpose

For each material capability, distinguish:

```text
Feature
  ↓
Specification
  ↓
Implementation
  ↓
Status
  ↓
Tests
  ↓
Benchmark
  ↓
Hardware validation
  ↓
Claim allowed
```

The registry exists to prevent executable prototypes, simulations, API scaffolds, and historical claims from being presented as equivalent evidence.

## Status definitions

- **SPECIFIED** — described by the specification/design; implementation not established.
- **SCAFFOLDED** — interface or structural skeleton exists, but the required backend is incomplete.
- **SIMULATED** — executable behavior intentionally models the capability rather than providing the real backend.
- **IMPLEMENTED** — substantive implementation exists, without implying comprehensive verification.
- **TESTED** — automated tests provide evidence for the stated behavior.
- **BENCHMARKED** — a reproducible benchmark artifact exists with environment and methodology.
- **INTEGRATION-TESTED** — multiple components have been verified together.
- **HARDWARE-VALIDATED** — behavior has been verified on the relevant physical target.
- **PRODUCTION-READY** — release, quality, operational, security, and verification gates have been satisfied for a defined target.
- **SAFETY-QUALIFIABLE** — sufficient evidence exists to enter the applicable formal safety process; this is not itself certification.

## Current evidence rules

1. A simulation must be labeled as simulation.
2. A scaffold must not be described as a working backend.
3. A benchmark result must identify its environment and revision.
4. A performance number without reproducible methodology is repository-reported, not independently verified.
5. Hardware claims require target-specific hardware evidence.
6. Safety qualification requires a defined safety process and appropriate evidence.
7. Historical evidence retains its original branch/revision context.

## Capability matrix

The detailed capability matrix from the historical audit remains preserved in Git history. During the documentation rewrite, entries should be migrated into this registry only after their current source, tests, and evidence have been reconciled.

The historical registry already distinguishes, among other examples:

- implemented/tested SPSC ring-buffer behavior;
- runtime node lifecycle and parameter validation;
- simulated versus real DMA paths;
- implemented basic UDP/TCP transport versus scaffolded zero-copy transport;
- simulated versus scaffolded distributed election;
- simulated versus scaffolded physics backends;
- simulated Studio telemetry versus live-provider architecture;
- CLI command architecture versus commands whose backend behavior remains simulated.

Those distinctions are important historical evidence and must not be erased during migration. fileciteturn60file0

## Ownership

- `docs/specifications/` defines what should be true.
- Source code defines what is implemented.
- Tests and benchmarks define executable evidence.
- This registry records the resulting capability classification.
- `docs/verification/` explains verification methodology and interpretation.
- `docs/documentation/` describes the documentation system itself.

## Update rule

A registry entry must be updated when a material implementation, test, benchmark, hardware-validation result, or claim policy changes. Entries must not be upgraded solely because documentation was rewritten.

## Completion requirement

Before the documentation rewrite is considered complete, the registry must be reconciled against the final source revision and the machine-readable repository representation snapshot.
