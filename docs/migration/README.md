# Documentation Migration

This section governs the transition from the historical NROS documentation set to the new domain-oriented documentation system.

## Migration principle

Historical material is preserved for traceability. It is not silently promoted to current implementation authority.

```text
Legacy document
      ↓
Inventory
      ↓
Classify authority and purpose
      ↓
Extract durable information
      ↓
Rewrite into focused documentation
      ↓
Cross-reference evidence
      ↓
Mark legacy source as historical, redirected, or retained
```

## Legacy-document map

| Legacy source | Primary role | Migration destination | Action |
|---|---|---|---|
| `DESIGN.md` | system design and target behavior | `docs/architecture/`, `docs/specifications/` | Split by topic; retain as canonical design source until migration is complete |
| `COMPARISON.md` | comparative claims | `docs/verification/`, `docs/reference/` | Reconcile every performance/readiness claim with evidence |
| `AUDIT.md` | repository audit | `docs/verification/`, `docs/safety/` | Retain as audit history; extract current rules into focused docs |
| `EVIDENCE_REGISTRY.md` | capability/evidence registry | `docs/verification/` and `docs/representation/` | Preserve as evidence authority while machine-readable manifests mature |
| `docs/ARCHITECTURE.md` | implementation clarification | `docs/architecture/` | Decompose into focused architecture pages |
| `docs/REPOSITORY_REPRESENTATION.md` | representation model | `docs/verification/` + `docs/documentation/` | Keep as canonical representation model; link from new navigation |
| `docs/SAFETY_REMEDIATION.md` | remediation history | `docs/safety/remediation.md` | Preserve history; do not treat remediation text as validation evidence |
| `docs/THREAT_MODEL.md` | threat/safety analysis | `docs/safety/threat-model.md` | Reuse as authoritative threat material |
| `AUDIT_PASS_*.md` | historical audits | `history` / repository root historical records | Preserve; newer evidence supersedes older observations where explicitly represented |

## Known reconciliation findings

### 1. `DESIGN.md` contains both specification and implementation-looking examples

Examples include APIs, schedulers, DMA, zero-copy paths, distributed protocols, and tooling. These examples must be classified as specification unless executable evidence supports a stronger state. The repository representation explicitly requires this separation. See [Repository Representation](../REPOSITORY_REPRESENTATION.md).

### 2. `COMPARISON.md` contains headline performance and readiness claims

The comparison document states concrete latency, throughput, real-time, safety, and production claims. These cannot be treated as independently verified merely because benchmark or demonstration code exists. The claim policy classifies headline performance as conditional and excludes independent validation and universal hardware guarantees.

### 3. Historical audit passes use different repository branches/revisions

Audit records must retain their represented branch and revision. A historical PASS statement must not be interpreted as a statement about the current documentation-rewrite branch unless the corresponding evidence has been re-executed or explicitly reconciled.

### 4. Representation snapshots can become stale

The machine-readable representation contains explicit source revisions and blob fingerprints. When documentation or repository state changes, the snapshot must be regenerated rather than silently reused. The current migration therefore treats stale snapshot identity as a reconciliation item, not as current verification evidence.

## Completion criteria

Migration is complete only when:

- every active document has an explicit role and authority;
- broad legacy documents have focused successors where appropriate;
- historical audit material remains discoverable;
- every important cross-reference resolves or is explicitly marked unresolved;
- public claims resolve to evidence and claim policy;
- machine-readable documentation manifests match the repository state;
- the final representation snapshot is regenerated at the final migration revision.
