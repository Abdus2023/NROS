# Type Specification

## Purpose

NROS uses explicit data contracts to make communication, configuration, and component boundaries understandable and verifiable.

## Contract principles

1. Data exchanged across subsystem boundaries should have an explicit representation.
2. Units and semantic meaning should be documented where values are safety- or control-relevant.
3. Ownership and lifetime must be explicit for buffers and shared resources.
4. Serialization formats must define compatibility expectations.
5. Invalid values must have defined handling rather than relying on implicit behavior.

## Type categories

| Category | Contract focus |
|---|---|
| Identifiers | Stable identity and scope |
| Configuration | Defaults, validation, compatibility |
| Messages | Schema, ownership, serialization |
| Commands | Preconditions, effects, errors |
| Telemetry | Units, timestamps, provenance |
| Errors | Classification and recovery semantics |

## Evidence boundary

The existence of a Rust type or schema does not by itself establish conformance to this specification. Conformance requires implementation and verification evidence.
