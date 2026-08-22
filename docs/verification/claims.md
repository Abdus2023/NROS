# NROS Claim Authority

This document defines the documentation boundary for technical claims.

## Claim hierarchy

```text
Architecture
    ↓
Implementation
    ↓
Test / CI / Benchmark / Simulation / Hardware Evidence
    ↓
Verification
    ↓
Validation
    ↓
Claim
```

## Required distinction

```text
Specified
   ≠ Implemented
   ≠ Tested
   ≠ Verified
   ≠ Validated
   ≠ Qualified
```

A claim must not be stronger than the evidence supporting it.

## Evidence sources

Primary repository evidence includes:

- `AUDIT.md` and audit records;
- `EVIDENCE_REGISTRY.md`;
- `.github/workflows/ci.yml` and executed workflow results;
- tests and source-level verification artifacts;
- benchmarks and their recorded conditions;
- simulation evidence where explicitly identified as simulation;
- physical/hardware evidence where actually executed.

## Architecture boundary

Architecture documents may define requirements, mechanisms, invariants, and intended guarantees. They do not establish that those guarantees are currently implemented or verified.

## Claim rule

> **No observed evidence → no verified claim.**

Strong claims must identify their scope, evidence, execution conditions, and remaining limitations.
