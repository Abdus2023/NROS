# NROS Specifications

> **Status:** Normative documentation index.

Specifications define the externally meaningful contracts, invariants, state transitions, compatibility rules, and behavioral requirements that NROS implementations are expected to satisfy.

## 1. Documentation hierarchy

NROS documentation uses distinct layers:

```text
Architecture
    ↓
Specification
    ↓
Reference
    ↓
Verification
    ↓
Safety / Operations
```

### Architecture

Describes the intended structure, boundaries, responsibilities, and relationships between components.

Architecture does **not** prove implementation completeness.

### Specification

Defines normative behavior: contracts, invariants, required transitions, interfaces, error semantics, and compatibility requirements.

A specification does **not** prove that the implementation satisfies it.

### Reference

Shows concrete APIs, configuration, examples, workflows, and implementation-oriented usage.

### Verification

Records observed evidence that a stated implementation or behavior exists and works under a declared environment.

### Safety / Operations

Defines safety constraints, deployment requirements, operational procedures, failure handling, and release/production conditions where applicable.

## 2. Specification areas

| Area | Scope |
|---|---|
| [Types](types.md) | Data representations and type-system contracts |
| [Protocols](protocols.md) | Communication protocols and state machines |
| [IPC](ipc.md) | Inter-process communication behavior |
| [Transport](transport.md) | Transport contracts and delivery semantics |
| [Safety](safety.md) | Safety-related invariants and boundaries |

Additional specification areas should be added only when a stable normative contract exists.

## 3. Normative language

Unless a document explicitly states otherwise:

- **MUST** indicates a mandatory requirement.
- **MUST NOT** indicates a prohibited behavior.
- **SHOULD** indicates a recommended behavior with an accepted reason for deviation.
- **SHOULD NOT** indicates a normally prohibited behavior with an accepted reason for deviation.
- **MAY** indicates an optional behavior.

These terms describe requirements; they do not describe implementation status.

## 4. Requirement versus implementation

The following states are deliberately separate:

```text
Requirement exists
      ↓
Interface specified
      ↓
Implementation exists
      ↓
Tests exist
      ↓
Tests pass
      ↓
Integration validated
      ↓
Operationally validated
```

A requirement must not be marked implemented merely because an API, stub, type, configuration option, or documentation statement exists.

## 5. Invariants

Specifications should identify invariants that must remain true across valid executions. Examples include:

- lifecycle state validity;
- ownership and lifetime rules;
- message/schema compatibility;
- ordering requirements;
- error propagation;
- cancellation behavior;
- resource bounds;
- security constraints.

An invariant should be testable or otherwise accompanied by a clear verification method.

## 6. Versioning and compatibility

Changes to normative contracts should identify their compatibility impact.

```text
Specification version
        ↓
Compatibility rule
        ↓
Implementation version
        ↓
Verification evidence
```

Backward compatibility must not be inferred from matching names or types alone.

## 7. Evidence boundary

The specification layer should link to verification evidence where available, but evidence belongs to the verification layer.

```text
Specification:
    "MUST preserve ordering"

Verification:
    "Test X observed preserved ordering under Y"
```

This prevents normative requirements and observed implementation behavior from becoming indistinguishable.

## 8. Change control

Normative specification changes should be traceable to:

1. the affected requirement or contract;
2. the architectural component;
3. the implementation impact;
4. affected tests;
5. compatibility impact;
6. verification evidence after implementation.

## 9. Related documentation

- [Architecture](../architecture/README.md)
- [Reference](../reference/README.md)
- [Verification](../verification/README.md)
- [Safety](../safety/README.md)
- [Operations](../operations/README.md)
