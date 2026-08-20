# Test Strategy

> **Status:** Repository-wide verification strategy.

Testing is an evidence-generation mechanism. A test result establishes only the behavior covered by its test, under the environment in which it executed.

## 1. Verification layers

NROS verification progresses from inexpensive deterministic checks toward increasingly system-specific validation:

```text
Static checks
    ↓
Unit tests
    ↓
Component / crate tests
    ↓
Integration tests
    ↓
End-to-end tests
    ↓
Simulation / replay
    ↓
Target / hardware validation
    ↓
Acceptance / qualification
```

Not every feature requires every layer. The required layer is determined by the claim being verified.

## 2. Static checks

Static checks include, where configured:

- formatting;
- linting;
- compilation/type checking;
- dependency and manifest validation;
- structural/documentation checks.

A successful static check establishes that the checked property holds for that invocation. It does not establish runtime behavior.

```text
cargo check passes
      ≠
program behavior verified
```

## 3. Unit tests

Unit tests should establish local invariants and deterministic component behavior.

Typical targets include:

- data-type invariants;
- parameter validation;
- lifecycle transitions;
- ring-buffer/channel behavior;
- error paths;
- boundary conditions;
- safety-related local guards.

Unit tests SHOULD include negative-path cases where the contract has rejection behavior.

## 4. Component / crate tests

Crate-level testing verifies behavior across the internal implementation boundary of a crate.

Examples include:

```text
nros-types
nros-core
nros-node
nros-transport
nros-hal
```

A passing crate test does not establish that independently tested crates integrate correctly.

## 5. Integration tests

Integration tests verify contracts across component boundaries.

Depending on the feature, this may include:

- node ↔ core communication;
- node ↔ transport;
- transport ↔ serialization;
- CLI ↔ runtime;
- configuration ↔ consuming subsystem;
- multiple crates participating in one workflow.

Integration evidence is required for claims that depend on interactions rather than isolated behavior.

## 6. End-to-end tests

End-to-end tests exercise a complete supported workflow across its defined boundaries.

An E2E test should identify:

```text
Input / trigger
      ↓
System path
      ↓
Expected observable result
      ↓
Actual result
```

A passing E2E test establishes the tested workflow, not every internal property of that workflow.

## 7. Simulation and replay

Simulation is useful for deterministic, repeatable scenarios where physical hardware is unavailable or undesirable.

Simulation evidence MUST state:

- what is modeled;
- what is not modeled;
- simulator version/revision;
- scenario/input;
- expected result;
- observed result.

```text
Simulation passes
      ≠
Physical system validated
```

Replay evidence has the same boundary: replaying an artifact verifies behavior against that artifact and replay environment, not arbitrary live inputs.

## 8. Hardware validation

Hardware validation is required for claims that depend materially on physical targets.

The record SHOULD identify:

- exact board/device;
- firmware/image;
- host environment;
- target configuration;
- connected peripherals;
- test procedure;
- observed result.

A successful test on one board does not automatically establish support for another board.

## 9. Concurrency and safety testing

Concurrency-sensitive components require tests appropriate to their invariants, including where relevant:

- producer/consumer contention;
- full/empty transitions;
- ordering;
- memory visibility;
- cancellation/drop behavior;
- invalid-state rejection;
- shutdown behavior.

Safety claims require explicit failure-mode stimuli and expected responses. Happy-path tests alone are insufficient evidence for safety behavior.

## 10. Performance and timing tests

Performance tests MUST identify the measurement environment and workload.

For timing claims, distinguish:

```text
Target
Observed sample
Distribution
Maximum observed
Worst-case bound
Qualification
```

A benchmark MUST NOT be documented as a real-time proof merely because measured samples are below a target.

## 11. Test reproducibility

A verification record SHOULD include:

```text
Repository revision
Test command
Toolchain
Target triple / hardware
Features
Configuration
Input/scenario
Result
Logs/artifacts
```

If a prerequisite prevents execution, record **BLOCKED** rather than converting the missing run into PASS.

## 12. Test status

Use precise states:

- **PASS / Verified** — criterion executed and satisfied;
- **FAIL / Failed** — criterion executed and not satisfied;
- **BLOCKED** — prerequisite prevented execution;
- **NOT RUN** — test exists but was not executed;
- **NOT VERIFIED** — insufficient evidence;
- **STALE** — result no longer applies to the relevant revision/environment.

`NOT RUN` and `BLOCKED` MUST NOT be rendered as successful evidence.

## 13. Mapping tests to claims

Each significant test should answer:

```text
Which claim does this test support?
What exact criterion does it evaluate?
What environment did it run in?
What was observed?
What remains outside its scope?
```

This prevents broad claims from being inferred from narrow tests.

## 14. CI interpretation

CI is an execution environment, not an abstract guarantee.

A green workflow proves the checks that actually ran in that workflow. It does not automatically prove:

- other operating systems;
- other architectures;
- unavailable hardware;
- unexecuted feature combinations;
- production workloads.

CI evidence SHOULD therefore retain workflow, job, revision, and relevant matrix context.

## 15. Test selection rule

Choose the lowest-cost test layer that is **sufficient for the claim**, then add stronger layers when the claim requires them.

```text
Existence claim
    → source/static evidence

Local behavior
    → unit test

Cross-component behavior
    → integration test

Complete workflow
    → E2E

Physical behavior
    → hardware validation

Performance guarantee
    → controlled measurement + appropriate analysis
```

## 16. Related documentation

- [Verification Overview](README.md)
- [Evidence Model](evidence-model.md)
- [Claims](claims.md)
- [Benchmarks](benchmarks.md)
- [Validation](validation.md)
- [Reference](../reference/README.md)
