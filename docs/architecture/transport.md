# Transport

## Purpose

Transport moves NROS data between communication endpoints. Local IPC and network transport are related but distinct architectural concerns.

## Layers

```text
Application message
      │
      ▼
Typed communication API
      │
      ├── Local IPC
      │
      └── Network transport
               │
               ├── Serialization
               ├── Framing
               ├── Reliability / delivery semantics
               └── Discovery / addressing
```

## Design principles

- keep transport policy separate from application semantics;
- make serialization explicit;
- expose failure and delivery semantics;
- avoid claiming zero-copy when serialization or buffering copies data;
- keep simulated and production transport implementations distinguishable.

## Verification boundary

A transport implementation must be evaluated independently for correctness, performance, reliability, and zero-copy behavior. A transport API or prototype does not establish those properties by itself.
