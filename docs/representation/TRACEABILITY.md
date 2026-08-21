# NROS Representation Traceability

> **Status:** Capability-to-evidence reconciliation ledger.
>
> This document records the current traceability boundary between represented capabilities, claim policy, specifications, implementation, and verification evidence. It deliberately records missing evidence as a gap rather than inventing or inferring evidence.

## 1. Traceability model

```text
Capability
   ↓
Claim policy
   ↓
Specification
   ↓
Implementation
   ↓
Verification method
   ↓
Evidence record
   ↓
Verification conclusion
   ↓
Validation / qualification
```

Not every capability requires every layer. The required links depend on the claim being made.

## 2. Evidence status vocabulary

Use the following states for this ledger:

- **LINKED** — a concrete evidence record has been identified;
- **NOT FOUND** — no corresponding evidence record has yet been identified;
- **NOT REQUIRED** — the represented claim is explicitly forbidden or limited to specification/scaffolding;
- **NOT VERIFIED** — evidence exists, but it is insufficient for the represented conclusion;
- **BLOCKED** — required verification could not execute because of a prerequisite;
- **STALE** — evidence exists but no longer applies to the relevant revision/environment.

`NOT FOUND` is intentionally different from `PASS` and `NOT REQUIRED`.

## 3. Evidence catalog cross-reference

The repository already contains `docs/representation/evidence.yaml`. It records evidence dimensions rather than a single pass/fail value. In particular, a capability may have source and tests present while CI, Miri, or benchmark execution remains unknown or non-gating.

The ledger therefore distinguishes **evidence record linkage** from **verification conclusion**.

| Capability | State | Claim policy | Specification | Evidence record | Evidence summary | Current traceability state |
|---|---|---|---|---|---|---|
| `CORE-IPC-001` — SPSC ring buffer | `TESTED` | `allowed_with_scope` | `DESIGN.md#14.1` | `evidence.yaml: CORE-IPC-001` | source present; tests present; CI unknown; Miri unknown; benchmark present/non-gating | **LINKED — NOT VERIFIED** |
| `CORE-IPC-002` — shared-memory memfd/mmap IPC | `SPECIFIED` | `forbidden` | `DESIGN.md#14.2` | No current evidence required for forbidden claim | specification only | **NOT REQUIRED** |
| `NODE-001` — node lifecycle | `IMPLEMENTED` | `allowed_with_scope` | `DESIGN.md#3.1` | No matching record currently identified | evidence record not yet linked | **NOT FOUND** |
| `NODE-002` — compile-time graph/message validation | `SPECIFIED` | `forbidden` | `DESIGN.md#5` | No current evidence required for forbidden claim | specification only | **NOT REQUIRED** |
| `HAL-001` — unified sensor abstraction | `IMPLEMENTED` | `allowed_with_scope` | `DESIGN.md#6.1` | No matching record currently identified | evidence record not yet linked | **NOT FOUND** |
| `HAL-002` — real V4L2/DMA-BUF camera path | `SPECIFIED` | `forbidden` | `DESIGN.md#16.4` | No current evidence required for forbidden claim | source/tests/benchmark/hardware absent in evidence catalog | **NOT REQUIRED** |
| `TRANSPORT-001` — UDP transport | `IMPLEMENTED` | `allowed_with_scope` | `DESIGN.md#14.3` | No matching record currently identified | evidence record not yet linked | **NOT FOUND** |
| `TRANSPORT-002` — true zero-copy network serialization | `SCAFFOLDED` | `forbidden` | `DESIGN.md#14.3` | No current evidence required for forbidden claim | scaffold only | **NOT REQUIRED** |
| `DIST-001` — distributed leader-election state machine | `IMPLEMENTED` | `allowed_as_scaffolding` | `DESIGN.md#17.1` | No matching record currently identified | bounded scaffolding claim | **NOT FOUND** |
| `DIST-002` — real Raft protocol and replicated state | `SCAFFOLDED` | `forbidden` | `DESIGN.md#17.1` | `evidence.yaml: DIST-002` | source scaffolded only; state-machine-only tests; benchmark/hardware absent; CI unknown | **LINKED — NOT VERIFIED** |
| `SIM-001` — deterministic simulation primitives | `TESTED` | `allowed_with_scope` | `DESIGN.md#7.3` | No matching record currently identified | evidence record not yet linked | **NOT FOUND** |
| `STUDIO-001` — Studio HTTP dashboard architecture | `IMPLEMENTED` | `allowed_with_scope` | `DESIGN.md#7.2` | No matching record currently identified | evidence record not yet linked | **NOT FOUND** |
| `STUDIO-002` — production live telemetry | `SCAFFOLDED` | `forbidden` | `DESIGN.md#7.2` | `evidence.yaml: STUDIO-002` | scaffolded only; live-telemetry tests absent; benchmark/hardware absent; CI unknown | **LINKED — NOT VERIFIED** |
| `CLI-001` — `nros init` project generation | `TESTED` | `allowed_with_scope` | `AUDIT.md#NROS-011` | No matching record currently identified | evidence record not yet linked | **NOT FOUND** |
| `AUDIT-001` — repository evidence and claim validation | `IMPLEMENTED` | `allowed_with_scope` | `AUDIT.md` | No matching record currently identified | evidence record not yet linked | **NOT FOUND** |

### Important interpretation

`LINKED — NOT VERIFIED` means the representation has a concrete evidence record, but that record does not by itself establish the stronger verification conclusion. For example, `CORE-IPC-001` has source and tests present but its CI and Miri states are unknown and its benchmark is explicitly non-gating.

`NOT FOUND` means the capability catalog currently lacks a matching evidence record. It does **not** prove that no evidence exists elsewhere in the repository.

## 4. Claim-policy cross-check

The claim policy defines explicit bounded claims for important areas, including:

- SPSC ring-buffer behavior;
- repository-reported performance measurements;
- real DMA/V4L2 hardware integration;
- Raft consensus;
- production live telemetry;
- ISO 26262 / IEC 61508 qualification;
- CI passing;
- Miri validation passing.

The policy also establishes invariants against unsupported claims, including hardware claims without hardware evidence and CI/Miri pass claims without executed successful runs.

The capability ledger MUST remain consistent with these policies.

## 5. Interpretation rules

### Evidence record present

Do not automatically upgrade the capability to `VERIFIED`. Inspect the evidence dimensions, scope, revision, execution state, and limitations first.

### `TESTED` without a linked evidence record

Do not upgrade the capability to `VERIFIED`. Locate the underlying test/evidence artifact before changing the representation.

### `IMPLEMENTED` without verification evidence

The implementation state may remain `IMPLEMENTED`. Documentation must not describe it as `VERIFIED`, `VALIDATED`, or `PRODUCTION-READY` without corresponding evidence.

### `SCAFFOLDED` / `SPECIFIED`

A missing verification record is normally not a defect when the represented claim is explicitly forbidden or limited to scaffolding/specification.

### Historical evidence

Historical audit records can supply useful context, but they must be checked against repository revision and scope before being used as current evidence.

## 6. Reconciliation procedure

For each `NOT FOUND` capability:

1. Search tests and integration tests for the capability identifier/name.
2. Search audit/evidence registries for a concrete evidence record.
3. Identify the repository revision associated with the evidence.
4. Check that the evidence actually executes the claimed behavior.
5. Check scope and exclusions against `claims.yaml`.
6. Record the evidence reference.
7. Only then consider changing the capability or verification conclusion.

If execution is currently impossible, use `BLOCKED` rather than `PASS`.

## 7. No inference rule

The following transformations are prohibited without explicit evidence:

```text
Source exists
    → VERIFIED

Test file exists
    → TESTED

CI workflow exists
    → CI PASSED

Benchmark exists
    → REAL-TIME

Simulation exists
    → HARDWARE VALIDATED

Implementation exists
    → PRODUCTION READY
```

The evidence must establish the actual claim at the actual scope.

## 8. Related artifacts

- [Representation Contract](README.md)
- [Capability Catalog](capabilities.yaml)
- [Evidence Catalog](evidence.yaml)
- [Claim Policy](claims.yaml)
- [Verification Overview](../verification/README.md)
- [Evidence Model](../verification/evidence-model.md)
- [Claims](../verification/claims.md)
- [Test Strategy](../verification/test-strategy.md)
- [Benchmarks](../verification/benchmarks.md)
- [Validation](../verification/validation.md)
- [Terminology](../verification/terminology.md)
