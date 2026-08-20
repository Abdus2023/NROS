# NROS Type Specification

> **Status:** Normative specification.
>
> This document defines the principles that govern NROS data contracts. Concrete type names and layouts are normative only when explicitly specified by the relevant API or protocol contract.

## 1. Purpose

NROS types form the data contracts used across application, runtime, communication, configuration, and tooling boundaries.

A type contract defines more than a language-level representation. Where applicable it includes:

- semantic meaning;
- valid values;
- units and ranges;
- ownership and lifetime;
- serialization representation;
- compatibility rules;
- error behavior.

## 2. Type categories

| Category | Contract focus |
|---|---|
| Identifiers | Identity, scope, equality, lifetime |
| Configuration | Defaults, validation, compatibility |
| Messages | Schema, ownership, serialization |
| Commands | Preconditions, effects, errors |
| Telemetry | Units, timestamps, provenance |
| Errors | Classification, context, recovery semantics |
| Events | Ordering, source, payload, lifecycle |
| Handles | Ownership, validity, lifetime |

A category is a specification concept; its existence does not prove that a corresponding implementation exists.

## 3. Validity

Types representing externally supplied or remotely received data MUST define how invalid values are handled where invalidity can affect correctness or safety.

```text
Input
  ↓
Decode
  ↓
Validate
  ├── invalid → reject / error
  └── valid   → typed value
```

Validation rules should be explicit rather than relying on incidental language behavior.

## 4. Semantic units

Values with physical or domain meaning SHOULD identify their units and relevant interpretation.

Examples include:

- duration;
- frequency;
- distance;
- velocity;
- acceleration;
- temperature;
- pressure;
- angles.

A numeric type alone does not establish its unit.

## 5. Ownership and lifetime

Types that represent resources or buffers MUST define ownership expectations where ownership is not obvious from the API.

```text
Created
  ↓
Owned
  ↓
Borrowed / shared
  ↓
Released
```

The contract should answer:

- who owns the resource;
- who may mutate it;
- when it becomes invalid;
- who releases it;
- whether it may cross a process boundary.

## 6. Messages

Message types SHOULD be treated as protocol contracts when exchanged between independent components.

A message contract should define, where relevant:

- schema;
- field semantics;
- required/optional fields;
- default behavior;
- encoding;
- version compatibility;
- ordering expectations;
- invalid-message handling.

## 7. Commands

Command types represent requested actions rather than observations.

A command contract SHOULD define:

```text
Preconditions
     ↓
Requested operation
     ↓
Result / state transition
     ↓
Error semantics
```

A command type does not itself prove that the requested operation is implemented or safe to execute.

## 8. Telemetry

Telemetry SHOULD preserve sufficient provenance to interpret an observation correctly.

Where applicable, telemetry should identify:

- source;
- timestamp;
- time model;
- units;
- validity state;
- sequence information;
- software or schema version;
- simulation versus live origin.

## 9. Errors

Errors SHOULD preserve enough information to distinguish:

```text
Invalid input
    ≠
Unavailable resource
    ≠
Transport failure
    ≠
Timeout
    ≠
Peer failure
    ≠
Internal failure
```

Error categories should support appropriate recovery without silently converting failure into success.

## 10. Serialization and compatibility

Language-level representation and wire representation are separate concerns:

```text
In-memory type
      ↓
Serialization contract
      ↓
Wire representation
```

A compatible schema does not necessarily imply ABI compatibility, and ABI compatibility does not necessarily imply semantic compatibility.

Changes to serialized fields, encoding, required values, or interpretation MUST follow the relevant compatibility policy.

## 11. ABI and layout

Where binary interoperability matters, the contract must distinguish:

- semantic schema;
- memory layout;
- alignment;
- size;
- endianness;
- calling convention;
- ABI stability.

A stable source-level type name does not establish a stable ABI.

## 12. Evidence boundary

The following states remain distinct:

```text
Type specified
    ↓
Type implemented
    ↓
Type serialized
    ↓
Compatibility tested
    ↓
Integration validated
```

The existence of a Rust type, schema, enum, or serializer does not by itself establish conformance.

## 13. Verification requirements

| Claim | Evidence |
|---|---|
| Type contract exists | Specification inspection |
| Values are validated | Executed validation tests |
| Serialization works | Encode/decode tests |
| Cross-version compatibility | Compatibility test matrix |
| ABI/layout is stable | Target-specific layout/ABI evidence |
| Ownership rules hold | Lifetime/ownership tests or analysis |
| Invalid input is rejected safely | Negative-path tests |

## 14. Related specifications

- [Specifications Index](README.md)
- [Protocols](protocols.md)
- [IPC](ipc.md)
- [Transport](transport.md)
- [Safety](safety.md)
