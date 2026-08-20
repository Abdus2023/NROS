# NROS Verification

> **Status:** Repository-wide verification framework.

Verification answers a different question from architecture, specification, and reference documentation:

> **What does the repository actually demonstrate?**

## 1. Verification boundary

NROS documentation MUST distinguish these states:

```text
Intent
  ↓
Requirement
  ↓
Design
  ↓
Implementation
  ↓
Execution
  ↓
Observation
  ↓
Evidence
  ↓
Verification conclusion
```

A source declaration, passing unit test, benchmark result, integration run, and hardware validation are different evidence classes. One MUST NOT silently substitute for another.

## 2. Verification areas

- [Evidence Model](evidence-model.md) — evidence classes, provenance, and confidence boundaries.
- [Claims](claims.md) — how documentation claims map to requirements and evidence.
- [Test Strategy](test-strategy.md) — automated and manual verification layers.
- [Benchmarks](benchmarks.md) — performance measurements and interpretation.
- [Validation](validation.md) — integration, hardware, and acceptance validation.

## 3. Repository status vocabulary

Use the repository-wide progression where applicable:

```text
PROPOSED
   ↓
SPECIFIED
   ↓
SCAFFOLDED
   ↓
SIMULATED
   ↓
IMPLEMENTED
   ↓
TESTED
   ↓
BENCHMARKED
   ↓
INTEGRATION-TESTED
   ↓
HARDWARE-VALIDATED
   ↓
PRODUCTION-READY
```

These statuses are **not automatically sequential proof obligations for every feature**. They describe increasing evidence/implementation maturity. A claim may use only the strongest status that its evidence actually supports.

## 4. Claim strength

A documentation claim should be expressible as a traceable record:

```text
Claim
  ↓
Requirement / rationale
  ↓
Implementation location
  ↓
Verification method
  ↓
Observed result
  ↓
Evidence artifact
  ↓
Status
```

If one of these links is unavailable, the documentation should identify the gap rather than imply verification.

## 5. Evidence classes

Typical evidence classes include:

| Evidence | Establishes | Does not automatically establish |
|---|---|---|
| Architecture text | Intended design | Implementation |
| Specification | Normative requirement | Compliance |
| Source/API | Declared implementation surface | Correct behavior |
| Unit test | Local behavior | System integration |
| Integration test | Cross-component behavior | Hardware correctness |
| Benchmark | Measured performance under stated conditions | Universal performance guarantee |
| Simulation | Behavior in modeled environment | Physical-system correctness |
| HIL test | Hardware interaction under tested conditions | General certification |
| CI result | Reproducible automated execution in that environment | Untested environments |
| Production qualification | Explicit qualification evidence | Unbounded future behavior |

## 6. Negative evidence matters

Verification records MUST preserve failures, skipped tests, blocked prerequisites, unavailable toolchains, and unsupported environments when they affect the conclusion.

```text
No evidence
   ≠
Pass
```

Likewise:

```text
Test not executed
   ≠
Test passed
```

## 7. Freshness

Evidence is revision-sensitive.

A verification result should identify the relevant repository revision or artifact version. Historical evidence MUST NOT automatically be presented as evidence for a newer implementation.

## 8. Reproducibility

Where practical, verification records should capture:

- repository revision;
- test/benchmark command;
- toolchain;
- target platform;
- enabled features;
- relevant configuration;
- required hardware;
- result;
- artifact/log location.

## 9. Safety and performance boundary

Performance, real-time, safety, and hardware claims require evidence appropriate to the claim.

For example:

```text
Callback target < 1 ms
        ≠
Observed callback < 1 ms
        ≠
Worst-case callback < 1 ms
        ≠
Certified real-time behavior
```

The same principle applies to safety claims:

```text
Safety mechanism exists
        ≠
Safety mechanism tested
        ≠
System safety validated
        ≠
Safety certified
```

## 10. Verification conclusion vocabulary

Use precise conclusions such as:

- **Observed** — directly observed in an execution or artifact.
- **Verified** — acceptance criterion satisfied with appropriate evidence.
- **Partially verified** — only a defined subset is established.
- **Blocked** — prerequisite prevented execution.
- **Not verified** — no sufficient evidence currently exists.
- **Failed** — executed verification did not satisfy the criterion.
- **Stale** — evidence no longer corresponds to the relevant revision/environment.

Avoid using `PASS` as a generic synonym for "the code looks correct."

## 11. Verification gate

A claim should be considered verified only when:

```text
Criterion defined
      AND
Relevant implementation identified
      AND
Appropriate verification executed
      AND
Observed result recorded
      AND
Evidence is attributable to the relevant revision
```

If any required condition is absent, the result must be qualified accordingly.

## 12. Related documentation

- [Architecture](../architecture/README.md)
- [Specifications](../specifications/README.md)
- [Reference](../reference/README.md)
- [Safety](../safety/README.md)
- [Operations](../operations/README.md)
