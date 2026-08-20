# Protocol Specifications

## Purpose

NROS protocols define observable interactions between components. A protocol consists of states, messages, preconditions, transitions, outcomes, and failure behavior.

## Required properties

A protocol specification should identify:

- participants and roles;
- message or event types;
- valid state transitions;
- ordering requirements;
- timeouts and retry semantics;
- failure and recovery behavior;
- compatibility/versioning rules;
- observability requirements.

## State-machine model

```text
     +---------+
     | Initial |
     +----+----+
          |
          v
     +----+----+
     | Active  |
     +----+----+
       |     |
 failure|     |complete
       v     v
   Failed   Done
```

The concrete state machine for each protocol belongs in its dedicated specification.

## Verification boundary

Protocol-shaped structs, enums, endpoints, or tests that exercise only individual branches do not by themselves prove complete protocol conformance. Verification must cover the required transitions and failure behavior.
