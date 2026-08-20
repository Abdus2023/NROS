# Documentation Integrity Audit

This document defines the integrity gate for the NROS documentation rewrite.

## Purpose

The documentation system must remain synchronized with repository state, source interfaces, evidence records, and documented authority relationships.

## Integrity dimensions

| Dimension | Question | Required result |
|---|---|---|
| Existence | Does the referenced document exist? | PASS |
| Role | Does the document have one clear purpose? | PASS |
| Authority | Is its authority explicitly defined? | PASS / HISTORICAL |
| Navigation | Do important links resolve? | PASS / EXPLICITLY UNRESOLVED |
| Claims | Are material claims evidence-backed? | PASS / CONDITIONAL |
| Status | Is implementation status current? | PASS / STALE |
| Revision | Is source/evidence revision identified where required? | PASS / PINNED |
| Migration | Is legacy material classified? | ACTIVE / MIGRATED / REDIRECTED / HISTORICAL / ARCHIVED |

## Initial migration status

| Area | Status | Notes |
|---|---|---|
| New documentation hubs | ACTIVE | Domain-oriented structure established under `docs/`. |
| README entry point | ACTIVE | Rewritten as project gateway. |
| Legacy `DESIGN.md` | MIGRATING | Contains both specification and implementation-looking material; must be decomposed carefully. |
| `COMPARISON.md` | MIGRATING | Headline performance/readiness claims require evidence reconciliation. |
| `AUDIT.md` | HISTORICAL / MIGRATING | Preserve audit history; extract durable policy into focused documentation. |
| `EVIDENCE_REGISTRY.md` | ACTIVE / MIGRATING | Preserve evidence authority while representation evolves. |
| `docs/representation/` | ACTIVE | Repository capability/evidence representation. |
| `docs/documentation/` | ACTIVE | Documentation inventory, authority, relationships, and references. |
| Historical `AUDIT_PASS_*.md` | HISTORICAL | Must retain represented branch/revision context. |
| Documentation snapshot | STALE | Must be regenerated at the final migration revision. |
| Repository representation snapshot | STALE | Must be regenerated after the final documentation rewrite state is established. |

## Claim integrity rules

1. Architectural prose is not implementation evidence.
2. An API surface is not proof of backend behavior.
3. A simulation result is not hardware validation.
4. A benchmark is not a universal performance guarantee.
5. A historical audit result applies to its represented revision unless explicitly revalidated.
6. Safety claims require evidence appropriate to their safety boundary.
7. `verified: true` in an application-facing response is not by itself independent verification.

## Completion gate

The documentation rewrite is complete only when all material documents have a known role and authority, important links resolve, material claims have evidence classifications, stale snapshots are regenerated, and the final branch revision is represented by the machine-readable documentation and repository manifests.

## Final audit procedure

```text
1. Enumerate documentation files.
2. Compare against documentation inventory.
3. Validate authority assignments.
4. Validate cross-document references.
5. Scan claims for unsupported maturity/performance/safety language.
6. Compare concrete API references with the source tree.
7. Check historical documents for revision context.
8. Regenerate machine-readable snapshots.
9. Re-run repository and documentation validation.
10. Record the final audit revision.
```
