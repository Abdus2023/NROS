# Evidence Model

> **Status:** Repository-wide evidence contract.

NROS uses evidence to distinguish **what is intended**, **what exists**, and **what has actually been demonstrated**.

## 1. Evidence object

A verification record SHOULD be representable as:

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

Not every evidence item requires every field, but the fields necessary to reproduce and interpret the conclusion MUST be retained.

## 2. Evidence classes

| Class | Establishes | Does not establish |
|---|---|---|
| Design | Intended behavior | Implementation |
| Specification | Required contract | Conformance |
| Scaffold | Structural presence | Functional correctness |
| Source/API | Declared implementation surface | Correct behavior |
| Simulation | Behavior in a model | Physical behavior |
| Unit test | Local tested behavior | System-wide correctness |
| Integration test | Cross-component behavior | Hardware validation |
| Benchmark | Measurement for a defined setup | Universal performance |
| CI execution | Automated execution in a defined environment | Untested environments |
| Hardware validation | Behavior on specified hardware | Arbitrary hardware |
| Qualification | Explicit acceptance under defined criteria | Unlimited future behavior |

## 3. Provenance

Evidence MUST be attributable to what produced it.

At minimum, repository-derived evidence SHOULD identify:

```text
repository
revision / commit
path or artifact
command / procedure
execution environment
result
```

For external evidence, identify the external artifact and its version or date where available.

## 4. Observation versus interpretation

Keep the observed fact separate from the conclusion drawn from it.

```text
Observation:
  `cargo test` exited successfully on revision X.

Conclusion:
  The tested suite passed in that environment.

Not established:
  All platforms are correct.
```

This distinction is mandatory for performance, safety, portability, and hardware claims.

## 5. Freshness

Evidence is revision- and environment-sensitive.

```text
Evidence(revision X, environment A)
              ↓
        does not automatically prove
              ↓
Evidence(revision Y, environment B)
```

Historical evidence may remain useful, but documentation MUST identify it as historical when it does not apply directly to the current revision.

## 6. Confidence boundary

Evidence strength depends on the claim.

```text
Existence claim
    → source inspection may be sufficient

Behavior claim
    → execution/test evidence expected

Performance claim
    → controlled measurement expected

Real-time claim
    → appropriate timing analysis + target evidence expected

Hardware claim
    → target/HIL evidence expected

Production claim
    → explicit qualification evidence expected
```

There is no universal evidence class that proves every kind of claim.

## 7. Negative evidence

Negative evidence is first-class repository state.

Examples include:

- implementation absent;
- command simulated;
- test failed;
- test not executed;
- prerequisite unavailable;
- hardware unavailable;
- benchmark not reproducible;
- integration path incomplete;
- evidence stale.

These states MUST remain visible when they affect a documentation conclusion.

```text
Missing evidence
      ≠
Negative proof
      ≠
Positive proof
```

The correct conclusion may simply be **Not verified** or **Blocked**.

## 8. Evidence lifecycle

```text
Captured
   ↓
Identified
   ↓
Attributed
   ↓
Evaluated
   ↓
Accepted / Qualified / Rejected
   ↓
Retained
   ↓
Revalidated or marked stale
```

Evidence should not silently become current again after a repository or environment change.

## 9. Evidence levels

For practical documentation, evidence may be summarized as:

| Level | Meaning |
|---|---|
| E0 | No supporting evidence |
| E1 | Documentary/source evidence |
| E2 | Automated local execution |
| E3 | Integration/target execution |
| E4 | Reproducible controlled validation |
| E5 | Explicit qualification/acceptance evidence |

Levels are **descriptive**, not a substitute for the underlying artifact. A claim SHOULD link to the evidence record whenever possible.

## 10. Claim mapping

Every strong repository claim should be traceable:

```text
Claim
 ↓
Evidence ID(s)
 ↓
Artifact / execution
 ↓
Observation
 ↓
Conclusion
 ↓
Limitations
```

If no evidence ID exists, the claim should be downgraded to an appropriate non-verified state.

## 11. Performance evidence

Performance evidence MUST record the conditions that materially affect the measurement, including where relevant:

- hardware;
- OS/kernel;
- compiler/toolchain;
- optimization profile;
- target triple;
- feature flags;
- workload;
- sample count;
- warm-up conditions;
- measurement method.

A single favorable measurement is not a worst-case guarantee.

## 12. Safety evidence

Safety evidence requires explicit scope.

```text
Mechanism exists
      ≠
Mechanism tested
      ≠
Failure mode tested
      ≠
System safety validated
      ≠
Certified safety
```

The documentation MUST identify which level is actually supported.

## 13. Verification status mapping

Evidence contributes to conclusions, but does not automatically dictate them:

```text
No evidence          → NOT VERIFIED
Execution blocked    → BLOCKED
Criterion failed     → FAILED
Subset demonstrated  → PARTIALLY VERIFIED
Criterion satisfied  → VERIFIED
Evidence superseded  → STALE
```

The underlying evidence remains the source of truth.

## 14. Related documentation

- [Verification Overview](README.md)
- [Claims](claims.md)
- [Test Strategy](test-strategy.md)
- [Benchmarks](benchmarks.md)
- [Validation](validation.md)
- [Reference](../reference/README.md)
