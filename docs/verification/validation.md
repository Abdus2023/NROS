# Validation

> **Status:** Repository-wide system validation contract.

Validation asks whether an implemented system is acceptable for a defined use case, environment, and acceptance criterion. It is stronger and broader than a local test, but it is still bounded by its scope and evidence.

## 1. Validation chain

```text
Requirement / use case
        ↓
Acceptance criterion
        ↓
Validation scenario
        ↓
System under test
        ↓
Observed behavior
        ↓
Evidence
        ↓
Validation conclusion
```

A validation statement without an explicit acceptance criterion is incomplete.

## 2. Validation levels

### Integration validation

Multiple components operate together according to their defined interfaces.

Examples may include:

- node ↔ core;
- node ↔ transport;
- serialization ↔ transport;
- CLI ↔ runtime.

Integration validation does not automatically establish complete system suitability.

### System validation

A defined end-to-end workflow satisfies explicit acceptance criteria.

The record should identify the complete workflow and observable outputs rather than only internal unit results.

### Simulation validation

A defined scenario passes against an explicitly identified model or simulator.

The validation record MUST identify model assumptions and exclusions.

```text
Simulation validation
        ≠
Physical validation
```

### Hardware validation

The implementation is exercised on identified physical hardware.

The record SHOULD identify:

```text
Target hardware
Firmware / image
Software revision
Configuration
Peripherals
Procedure
Stimulus
Expected result
Observed result
Logs / artifacts
Limitations
```

### Operational validation

Deployment, startup, observability, recovery, and operational procedures work under defined conditions.

Operational validation should include the environment and procedure being validated, not merely the fact that software starts locally.

## 3. Acceptance criteria

Acceptance criteria MUST be observable and scoped.

Weak:

```text
System is reliable.
```

Stronger:

```text
Under scenario X, condition Y is maintained for Z duration
and the observable failure count remains below criterion C.
```

The exact criterion depends on the system requirement.

## 4. Validation versus testing

Testing asks whether a defined behavior passes a test.

Validation asks whether the system satisfies the intended acceptance criteria for its defined use case.

```text
Test passes
   ↓
Evidence about tested behavior

Validation passes
   ↓
Acceptance criterion demonstrated
```

A passing test without an acceptance criterion should not automatically be described as full system validation.

## 5. Validation versus verification

Use the terms deliberately:

```text
Verification
→ Did the implementation satisfy the defined technical criterion?

Validation
→ Does the resulting system satisfy the defined use-case acceptance criterion?
```

A project may have strong verification evidence without having completed system validation.

## 6. Safety validation

Safety validation requires explicit scope and failure scenarios.

A safety mechanism being implemented or unit-tested does not by itself establish system-level safety.

```text
Mechanism
   ↓
Failure-mode test
   ↓
System response
   ↓
Acceptance criterion
   ↓
Safety validation
```

Certification is a separate claim requiring the applicable certification evidence.

## 7. Performance validation

Performance validation must identify the workload, environment, acceptance threshold, and measurement procedure.

For timing:

```text
Target
   ↓
Acceptance threshold
   ↓
Measurement
   ↓
Observed distribution
   ↓
Decision
```

A benchmark may support validation, but the benchmark itself is not the acceptance criterion unless explicitly defined as such.

## 8. Hardware and physical boundaries

Hardware validation is scoped to the hardware and conditions actually tested.

```text
Board A validated
      ≠
Board B validated
```

Likewise, software simulation cannot establish electrical, mechanical, thermal, physical timing, or production behavior unless those properties are explicitly modeled and the claim is limited to the model.

## 9. Operational acceptance

Operational validation should cover, where relevant:

- installation;
- configuration;
- startup/shutdown;
- health/observability;
- failure recovery;
- upgrade/migration;
- rollback;
- data/log handling;
- resource constraints.

Each accepted behavior should have an observable criterion and evidence.

## 10. Validation record

A significant validation record SHOULD contain:

```text
Validation ID
Requirement / use case
Acceptance criterion
System revision
Environment
Configuration
Scenario / procedure
Expected result
Observed result
Evidence artifacts
Conclusion
Limitations
Date / evidence timestamp
```

## 11. Validation conclusions

Use:

- **Validated** — acceptance criterion satisfied within stated scope;
- **Partially validated** — only a defined subset is demonstrated;
- **Blocked** — required validation could not execute;
- **Failed** — criterion executed and was not satisfied;
- **Not validated** — insufficient evidence;
- **Stale** — evidence no longer applies to the relevant system revision/environment.

Do not use `Validated` as a synonym for `Implemented` or `Tested`.

## 12. Qualification boundary

Qualification is an explicit acceptance decision based on defined criteria and evidence.

```text
Test
  ↓
Verification
  ↓
Validation
  ↓
Qualification
```

The arrows represent increasing scope/decision context, not an automatic process. A project should claim qualification only when an explicit qualification criterion and decision record exist.

## 13. Related documentation

- [Verification Overview](README.md)
- [Evidence Model](evidence-model.md)
- [Claims](claims.md)
- [Test Strategy](test-strategy.md)
- [Benchmarks](benchmarks.md)
- [Reference](../reference/README.md)
