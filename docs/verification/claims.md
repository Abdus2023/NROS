# Claims

> **Status:** Repository-wide claim traceability contract.

A claim is a statement about NROS behavior, implementation, performance, safety, compatibility, maturity, or support. Claims are stronger than descriptions: they imply that the repository can support the statement with identifiable evidence.

## 1. Claim structure

A strong claim SHOULD be traceable through:

```text
Claim
  ↓
Requirement / rationale
  ↓
Specification
  ↓
Implementation artifact
  ↓
Verification method
  ↓
Evidence record
  ↓
Observed result
  ↓
Conclusion / limitations
```

The chain may be shorter for simple existence claims, but documentation MUST NOT imply links that do not exist.

## 2. Claim classes

| Class | Example | Minimum evidence expectation |
|---|---|---|
| Existence | `nros-core` exposes a channel API | Source/API inspection |
| Behavioral | Channel rejects invalid capacity | Executed test |
| Integration | Node communicates through transport | Integration execution |
| Performance | Callback completes within target | Controlled benchmark |
| Real-time | Worst-case execution meets bound | Appropriate timing analysis + target evidence |
| Safety | Emergency stop prevents command output | Failure-mode test |
| Portability | Target runs successfully | Target-specific build/test |
| Hardware | Driver works on specified board | Hardware/HIL evidence |
| Production | Feature is production-qualified | Explicit qualification evidence |

## 3. Claim strength

Avoid converting a weaker observation into a stronger claim.

```text
Source contains feature
        ≠
Feature executes
        ≠
Feature passes tests
        ≠
Feature works in integration
        ≠
Feature is production-ready
```

Likewise:

```text
One benchmark result
        ≠
Worst-case guarantee
        ≠
Real-time certification
```

## 4. Required claim fields

For significant claims, documentation SHOULD record:

```text
Claim ID
Statement
Scope
Requirement / rationale
Implementation path(s)
Verification method
Evidence ID(s)
Repository revision
Status
Limitations
Owner / review context
```

Not every lightweight documentation statement requires all fields, but safety, performance, compatibility, and production claims should be treated as significant by default.

## 5. Status vocabulary

Use the verification conclusions defined by the evidence model:

- **Observed** — directly observed;
- **Verified** — criterion satisfied with appropriate evidence;
- **Partially verified** — only a defined subset is demonstrated;
- **Blocked** — verification could not execute because a prerequisite was unavailable;
- **Not verified** — insufficient evidence;
- **Failed** — verification executed and criterion was not satisfied;
- **Stale** — evidence no longer applies to the relevant revision/environment.

`Implemented` is an implementation state, not a verification conclusion.

## 6. Current high-value claim categories

NROS documentation should explicitly track claims concerning:

### Determinism

Do not use "deterministic" as a blanket property. Specify the scope, scheduling model, inputs, environment, and measured/verified conditions.

### Zero-copy / allocation behavior

A zero-copy claim requires evidence covering the relevant data path. The existence of `MaybeUninit`, ring buffers, or references is not by itself proof that the complete end-to-end path performs no copies or allocations.

### Real-time behavior

Separate:

```text
Target deadline
Observed execution time
Measured distribution
Worst-case bound
Real-time qualification
```

### Safety

Identify the mechanism, failure mode, test stimulus, expected response, observed response, and remaining scope.

### Distributed operation

Separate protocol/type definitions from actual network communication, discovery, authentication, fault handling, and deployment behavior.

### Hardware support

Name the exact target and evidence. A HAL abstraction alone does not establish hardware support.

## 7. Claim review rule

When documentation and repository evidence disagree:

1. inspect the current implementation;
2. identify the relevant revision;
3. inspect executable evidence;
4. correct the documentation, or explicitly label it historical/proposed;
5. preserve the discrepancy when it is important to the audit trail.

A README statement cannot upgrade evidence.

## 8. Claim matrix template

Use this structure for future claim inventories:

| ID | Claim | Scope | Implementation | Verification | Evidence | Status | Limitations |
|---|---|---|---|---|---|---|---|
| NROS-CLM-001 | Example claim | Define scope | `path/to/code` | Test/benchmark | `E-...` | Not verified | Define gap |

IDs should remain stable when the claim is edited so that downstream evidence can continue to reference it.

## 9. Claim change control

A claim MUST be reconsidered when:

- its implementation changes;
- its verification test changes materially;
- its target environment changes;
- its evidence is superseded;
- its scope expands;
- its wording becomes stronger;
- a failure or regression invalidates the prior conclusion.

Documentation review is therefore part of engineering change control, not a final prose-only activity.

## 10. Related documentation

- [Verification Overview](README.md)
- [Evidence Model](evidence-model.md)
- [Test Strategy](test-strategy.md)
- [Benchmarks](benchmarks.md)
- [Validation](validation.md)
- [Reference](../reference/README.md)
