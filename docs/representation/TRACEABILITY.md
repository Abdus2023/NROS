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

- **LINKED** — a concrete evidence reference has been identified;
- **NOT FOUND** — the expected evidence reference has not yet been identified;
- **NOT REQUIRED** — the capability is only specified/scaffolded and no stronger evidence is claimed;
- **NOT VERIFIED** — evidence may exist, but it is insufficient for the represented conclusion;
- **BLOCKED** — the required verification could not execute because of a prerequisite;
- **STALE** — evidence exists but no longer applies to the relevant revision/environment.

`NOT FOUND` is intentionally different from `PASS` and `NOT REQUIRED`.

## 3. Current capability ledger

| Capability | State | Claim policy | Specification | Evidence link | Current traceability state |
|---|---|---|---|---|---|
| `CORE-IPC-001` — SPSC ring buffer | `TESTED` | `allowed_with_scope` | `DESIGN.md#14.1` | Not identified in the capability record | **NOT FOUND** |
| `CORE-IPC-002` — shared-memory memfd/mmap IPC | `SPECIFIED` | `forbidden` | `DESIGN.md#14.2` | Not required for current forbidden claim | **NOT REQUIRED** |
| `NODE-001` — node lifecycle | `IMPLEMENTED` | `allowed_with_scope` | `DESIGN.md#3.1` | Not identified in the capability record | **NOT FOUND** |
| `NODE-002` — compile-time graph/message validation | `SPECIFIED` | `forbidden` | `DESIGN.md#5` | Not required for current forbidden claim | **NOT REQUIRED** |
| `HAL-001` — unified sensor abstraction | `IMPLEMENTED` | `allowed_with_scope` | `DESIGN.md#6.1` | Not identified in the capability record | **NOT FOUND** |
| `HAL-002` — real V4L2/DMA-BUF camera path | `SPECIFIED` | `forbidden` | `DESIGN.md#16.4` | Not required for current forbidden claim | **NOT REQUIRED** |
| `TRANSPORT-001` — UDP transport | `IMPLEMENTED` | `allowed_with_scope` | `DESIGN.md#14.3` | Not identified in the capability record | **NOT FOUND** |
| `TRANSPORT-002` — true zero-copy network serialization | `SCAFFOLDED` | `forbidden` | `DESIGN.md#14.3` | Not required for current forbidden claim | **NOT REQUIRED** |
| `DIST-001` — distributed leader-election state machine | `IMPLEMENTED` | `allowed_as_scaffolding` | `DESIGN.md#17.1` | Not identified in the capability record | **NOT FOUND** |
| `DIST-002` — real Raft protocol and replicated state | `SCAFFOLDED` | `forbidden` | `DESIGN.md#17.1` | Not required for current forbidden claim | **NOT REQUIRED** |
| `SIM-001` — deterministic simulation primitives | `TESTED` | `allowed_with_scope` | `DESIGN.md#7.3` | Not identified in the capability record | **NOT FOUND** |
| `STUDIO-001` — Studio HTTP dashboard architecture | `IMPLEMENTED` | `allowed_with_scope` | `DESIGN.md#7.2` | Not identified in the capability record | **NOT FOUND** |
| `STUDIO-002` — production live telemetry | `SCAFFOLDED` | `forbidden` | `DESIGN.md#7.2` | Not required for current forbidden claim | **NOT REQUIRED** |
| `CLI-001` — `nros init` project generation | `TESTED` | `allowed_with_scope` | `AUDIT.md#NROS-011` | Not identified in the capability record | **NOT FOUND** |
| `AUDIT-001` — repository evidence and claim validation | `IMPLEMENTED` | `allowed_with_scope` | `AUDIT.md` | Not identified in the capability record | **NOT FOUND** |

This table is deliberately conservative. `NOT FOUND` means that the capability catalog itself does not currently identify a concrete evidence record; it does **not** prove that no evidence exists elsewhere in the repository.

## 4. Claim-policy cross-check

The claim policy already defines explicit bounded claims for important areas, including:

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

### `TESTED` without a linked evidence record

Do not upgrade the capability to `VERIFIED`. Record the missing linkage and locate the underlying test/evidence artifact before changing state.

### `IMPLEMENTED` without verification evidence

The implementation state may remain `IMPLEMENTED`. The documentation must not describe it as `VERIFIED`, `VALIDATED`, or `PRODUCTION-READY` without the corresponding evidence.

### `SCAFFOLDED` / `SPECIFIED`

A missing verification record is normally not a defect when the represented claim is explicitly forbidden or limited to scaffolding/specification.

### Historical evidence

Historical audit records can supply useful context, but they must be checked against repository revision and scope before being used as current evidence.

## 6. Reconciliation procedure

For each `NOT FOUND` capability:

1. Search tests and integration tests for the capability identifier/name.
2. Search audit/evidence registries for a concrete evidence ID.
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
- [Claim Policy](claims.yaml)
- [Verification Overview](../verification/README.md)
- [Evidence Model](../verification/evidence-model.md)
- [Claims](../verification/claims.md)
- [Test Strategy](../verification/test-strategy.md)
- [Benchmarks](../verification/benchmarks.md)
- [Validation](../verification/validation.md)
- [Terminology](../verification/terminology.md)
