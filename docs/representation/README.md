# NROS Representation

> **Status:** Machine-readable capability and claim representation contract.

The `docs/representation/` directory is the structured representation layer for repository capabilities and claim policy. It complements, rather than replaces, the human-readable verification documentation.

## Purpose

The representation layer answers two related questions:

1. **What capabilities does NROS represent as existing, specified, scaffolded, tested, or otherwise qualified?**
2. **What claims are permitted, scoped, conditional, or forbidden given the available evidence?**

The current primary artifacts are:

- [`capabilities.yaml`](./capabilities.yaml) — capability catalog and implementation/evidence maturity;
- [`claims.yaml`](./claims.yaml) — claim classes, scope rules, exclusions, and invariants.

## Separation of responsibilities

```text
Representation
├── capabilities.yaml
│      └── What capability/state is represented?
│
└── claims.yaml
       └── What claim is permitted at that state/evidence level?

Verification
└── docs/verification/
       ├── evidence-model.md
       ├── claims.md
       ├── test-strategy.md
       ├── benchmarks.md
       ├── validation.md
       └── terminology.md

Human documentation
└── docs/**
       └── How should the repository state be explained?
```

The YAML files do not replace evidence. They represent the current policy/catalog derived from implementation and evidence records.

## Capability state

`capabilities.yaml` uses the project capability vocabulary:

```text
SPECIFIED
SCAFFOLDED
SIMULATED
IMPLEMENTED
TESTED
BENCHMARKED
INTEGRATION-TESTED
HARDWARE-VALIDATED
PRODUCTION-READY
SAFETY-QUALIFIABLE
```

A capability state is not a universal verification conclusion. In particular:

```text
IMPLEMENTED ≠ VERIFIED
TESTED ≠ VALIDATED
BENCHMARKED ≠ REAL-TIME QUALIFIED
HARDWARE-VALIDATED ≠ PRODUCTION-READY
```

A capability may therefore be `IMPLEMENTED` while its permitted claim remains `allowed_as_scaffolding` or otherwise constrained.

## Claim policy

`claims.yaml` classifies claims as:

- `allowed_with_scope` — evidence supports the claim at its declared scope;
- `allowed_as_scaffolding` — implementation exists, but protocol or validation is incomplete;
- `conditional` — some implementation/evidence exists but required validation is missing;
- `forbidden` — evidence is insufficient or the capability is only specified/scaffolded for the requested claim.

Claim policy MUST be interpreted together with its `scope`, `excludes`, and `reason`/`rule` fields.

## Canonical traceability

A represented capability should be traceable through the following chain where applicable:

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

The absence of one of these links does not automatically invalidate a simple existence record, but documentation MUST NOT imply a link that is not present.

## Evidence authority

The representation layer follows the repository evidence contract:

- source inspection establishes existence/structure claims;
- executed tests establish tested behavior;
- benchmarks establish measured results under defined conditions;
- integration/system tests establish cross-component behavior;
- hardware evidence establishes claims for the identified target;
- validation establishes satisfaction of an explicit acceptance criterion;
- qualification requires an explicit qualification criterion and decision.

Configuration alone is never execution evidence. A configured CI workflow is not a passing CI run; an installed Miri tool is not Miri validation.

## Invariants

The representation MUST preserve these invariants:

```text
No claim without an evidence record.
No real claim from a simulated backend.
No hardware claim without hardware evidence.
No CI pass claim without an executed successful run.
No performance validation claim from a non-gating benchmark alone.
```

These invariants are already encoded in `claims.yaml` and should be treated as policy, not optional prose.

## Relationship to human documentation

Human-facing documentation may summarize represented state, but it MUST NOT silently strengthen it.

For example:

```text
capabilities.yaml:
  state: SCAFFOLDED
  claim: forbidden
```

must not become a README statement such as:

```text
NROS supports the capability.
```

without new evidence and a corresponding representation update.

## Historical records

Historical audit and evidence documents are preserved for traceability. They may contain terminology that was accurate at the time but is not an appropriate description of current repository state.

Do not rewrite historical records merely to normalize current terminology. Instead, distinguish historical context from current representation.

## Change control

When implementation or evidence changes materially:

1. update the relevant capability state;
2. update claim policy/scope if required;
3. add or update evidence references;
4. reconcile human-readable documentation;
5. run the representation/documentation consistency checks.

A documentation edit alone MUST NOT promote a capability state or claim class.

## Related documentation

- [Documentation Hub](../README.md)
- [Verification Overview](../verification/README.md)
- [Evidence Model](../verification/evidence-model.md)
- [Claims](../verification/claims.md)
- [Terminology](../verification/terminology.md)
