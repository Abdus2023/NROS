# Documentation Terminology

> **Status:** Repository-wide terminology contract for technical claims.

This document defines how maturity, verification, validation, performance, and support language MUST be used in NROS documentation.

## 1. Core rule

Use the weakest term that is fully supported by current evidence.

```text
Intent        → PROPOSED
Structure     → SCAFFOLDED
Implementation→ IMPLEMENTED
Execution     → TESTED / OBSERVED
Evidence      → VERIFIED
Use-case      → VALIDATED
Acceptance    → QUALIFIED
```

Do not promote a statement from one category to another without the corresponding evidence.

## 2. Implementation terms

### Proposed

Use for planned, experimental, or future behavior that is not established as implemented.

### Specified

Use when a requirement or contract is explicitly defined. `SPECIFIED` does not imply implementation.

### Scaffolded

Use when structural code, interfaces, placeholders, or partial paths exist but the complete capability is not demonstrated.

### Implemented

Use only when the functional implementation exists within an explicitly stated scope.

`IMPLEMENTED` does not imply tested, integrated, validated, or production-ready.

## 3. Evidence terms

### Observed

A behavior, artifact, or result was directly observed under stated conditions.

### Tested

An executable test evaluated a defined criterion.

### Verified

Appropriate evidence demonstrates that a defined technical criterion is satisfied for its stated scope.

### Partially verified

Only a defined subset of the criterion is established.

### Not verified

There is insufficient evidence to establish the claim.

### Blocked

The required verification could not execute because a prerequisite was unavailable.

### Failed

The verification executed and the criterion was not satisfied.

### Stale

Evidence no longer applies to the relevant revision, configuration, or environment.

## 4. Validation terms

### Validated

A defined system/use-case acceptance criterion has been demonstrated within an explicitly stated scope.

### Qualified

An explicit qualification or acceptance decision exists against defined criteria and evidence.

Neither term should be used merely because tests pass.

## 5. Performance language

Avoid unqualified terms such as:

- `fast`;
- `low latency`;
- `high performance`;
- `deterministic`;
- `real-time`;
- `zero-copy`;
- `zero-allocation`.

When used, define the exact scope and evidence.

Prefer:

```text
Measured at X under environment Y
Observed p99 latency of Z
Target is X
Maximum observed was Y
```

Do not write `real-time` when only a timing target or benchmark has been established.

## 6. Determinism

`Deterministic` MUST identify what is deterministic and under which assumptions.

Examples:

```text
Deterministic serialization for identical inputs
```

is materially different from:

```text
NROS is deterministic
```

The second statement is too broad unless supported by system-level evidence.

## 7. Zero-copy and allocation language

Use `zero-copy` only for a defined data path for which the relevant copies have been demonstrated absent.

Use `zero-allocation` only when the relevant allocation domain and execution path have been instrumented or otherwise established.

The existence of an efficient primitive does not establish an end-to-end property.

## 8. Real-time language

Distinguish:

```text
real-time target
observed timing
statistical timing result
worst-case bound
real-time qualification
```

These are different claims and require progressively stronger evidence.

## 9. Safety language

Distinguish:

```text
safety mechanism exists
safety mechanism tested
failure mode validated
system safety validated
certified
```

`Safe`, `safety-critical`, and `certified` are high-strength claims and require explicit scope and evidence.

## 10. Support language

`Supported` MUST identify the support boundary.

Prefer:

```text
Builds on target X
Tested on target X
Hardware-validated on target X
```

over:

```text
Supports all ARM boards
```

unless the broader claim has evidence.

## 11. Production language

Avoid calling a feature `production-ready` merely because it compiles or passes unit tests.

Production readiness requires explicit project-defined criteria covering the applicable implementation, testing, operational, security, performance, safety, and deployment requirements.

## 12. Forbidden implication patterns

The following transformations are not valid without additional evidence:

```text
implemented  → verified
unit-tested  → system-validated
benchmark    → real-time
simulation   → hardware support
API exists   → feature works end-to-end
one target   → all targets
CI green     → production-ready
```

## 13. Documentation review checklist

When reviewing a technical claim, ask:

1. What exactly is being claimed?
2. What is its scope?
3. Is it normative, descriptive, or evidentiary?
4. Where is it implemented?
5. What evidence supports it?
6. Which revision/environment does that evidence cover?
7. What remains outside the evidence?
8. Is there a weaker, more accurate term?

## 14. Related documentation

- [Verification Overview](README.md)
- [Evidence Model](evidence-model.md)
- [Claims](claims.md)
- [Test Strategy](test-strategy.md)
- [Benchmarks](benchmarks.md)
- [Validation](validation.md)
