# NROS Part-Level Architecture

This directory contains the fine-grained representation of the NROS Architecture Series.

## Canonical relationship

```text
Root consolidated corpus
        ↓
NROS_SERIES_INDEX.md
        ↓
docs/architecture/parts/
```

The Part-level documents are a decomposition of the same architecture narrative. They are not a second independent series.

## Current boundary

- **Part I** — ROS Foundation & NROS Proposition
- **Part II** — NROS Core Concepts
- **Part III** — NROS Runtime & Kernel Boundary
- …
- **Part LXXI** — Failure Semantics, Mismatch, Containment & Recovery
- **Part LXXII** — Failure Policy, Retry, Idempotency & Compensation
- **Part LXXIII** — State & Resource Reconciliation & Convergence

**LXXIV is not yet established.**

## Part contract

Every Part should identify, where applicable:

1. Part number and title;
2. purpose and scope;
3. relationship to the previous Part;
4. concepts introduced;
5. concepts consumed from earlier Parts;
6. invariants and boundaries;
7. canonical consolidated source;
8. implementation implications;
9. verification implications;
10. explicit architectural status.

## Representation discipline

A repeated concept is not automatically duplication. A later Part may legitimately refine an earlier concept at a stronger abstraction level.

For example:

```text
Security
  → foundational trust / authorization
  → distributed security plane
  → governance / policy lifecycle

Resources
  → budgets
  → lifecycle / ownership
  → accounting / admission
  → distributed leases / reclamation

Coordination
  → foundational distributed coordination
  → consensus / membership / leadership
  → state reconciliation
  → authority / fencing / takeover
```

The audit must distinguish **reuse**, **refinement**, **duplication**, and **forward dependency**.

## Evidence boundary

Architecture is not implementation evidence.

```text
Specified
   ≠ Implemented
   ≠ Tested
   ≠ Verified
   ≠ Validated
   ≠ Qualified
```

Strong implementation, performance, safety, or correctness claims require corresponding evidence in the repository's verification/evidence layer.

## Audit rule

Before creating a new Part:

```text
Existing corpus
    ↓
Numbering check
    ↓
Scope check
    ↓
Prerequisite check
    ↓
Canonical-source check
    ↓
Implementation/evidence check
    ↓
Novel contribution check
    ↓
Only then create the next Part
```

This prevents architectural drift caused by advancing the series faster than its representation can be reconciled.
