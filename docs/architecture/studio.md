# NROS Studio Architecture

> **Status:** Active architectural documentation.
>
> Studio is an observability and operator-facing layer. Its existence does not establish live telemetry, production readiness, or safety-critical control capability.

## 1. Purpose

NROS Studio provides interfaces for inspecting and interacting with runtime state.

```text
NROS Runtime
     │
     ├── Nodes
     ├── Topics
     ├── Transforms
     ├── Parameters
     ├── Events
     └── Metrics
          │
          ▼
   Studio Data Interface
          │
     ┌────┼─────────┐
     ↓    ↓         ↓
   HTTP Streaming Dashboard
```

Studio should remain downstream of runtime state rather than becoming the authoritative owner of that state.

## 2. Data-source boundary

Studio data may originate from different providers:

```text
Live runtime
    ├── telemetry
    ├── state inspection
    └── diagnostics

Simulation
    ├── virtual state
    └── simulated telemetry

Replay / fixture
    ├── recorded events
    └── test data
```

The source must be identifiable. A simulated or fixture provider must not be presented as live runtime telemetry.

## 3. Observation versus control

Studio should distinguish:

```text
Observation
    ≠
Configuration
    ≠
Operational control
    ≠
Safety-critical control
```

An operator-facing button or API endpoint is not evidence that the corresponding runtime operation is implemented safely or that it is suitable for safety-critical use.

## 4. Data contracts

Studio interfaces should expose explicit contracts for:

- node/entity state;
- communication graph information;
- parameters;
- transforms;
- metrics;
- events;
- diagnostics;
- health state;
- command results where control operations are supported.

The presentation format may change independently from the underlying runtime contract.

## 5. Freshness and provenance

Observability data should preserve enough metadata to distinguish:

- source;
- timestamp or time model;
- software revision where relevant;
- simulation versus live origin;
- stale versus current state;
- error or degraded state.

A dashboard value without provenance can be visually convincing while being operationally misleading.

## 6. Streaming and polling

Studio may obtain state through polling, streaming, event subscriptions, or other mechanisms.

```text
Polling
  ≠
Streaming
  ≠
Real-time telemetry
```

A streaming transport does not automatically provide bounded freshness or real-time guarantees.

## 7. Availability and degradation

Studio should tolerate partial loss of observability where possible:

```text
Runtime healthy
      ↓
Studio connected
      ↓
Fresh state
```

versus:

```text
Runtime healthy
      ↓
Studio disconnected
      ↓
Runtime continues independently
```

The observability layer should not become an accidental single point of failure for core runtime execution unless the architecture explicitly requires such coupling.

## 8. Security boundary

Studio can expose sensitive operational information or control surfaces. Authentication, authorization, transport security, auditability, and least-privilege access therefore belong in the Studio deployment model where applicable.

Network reachability alone must not be treated as operator authorization.

## 9. Safety boundary

Safety-critical functions must not rely solely on visualization, browser state, or operator-interface availability.

```text
Studio
  ↓
Observation / operator intent
  ↓
Runtime safety boundary
  ↓
Actuation
```

The runtime and relevant hardware safety mechanisms remain authoritative for safe execution.

## 10. Verification requirements

| Claim | Evidence |
|---|---|
| Studio interface exists | Source/interface inspection |
| Endpoint returns runtime data | Executed integration test |
| Data is live | Live provider + runtime integration evidence |
| Data freshness is bounded | Measured end-to-end observation latency |
| Simulation is correctly identified | Provider/provenance test |
| Commands reach runtime | End-to-end control-path test |
| Authorization is enforced | Security/integration test |
| Studio failure does not stop runtime | Failure-isolation test |
| Production observability is ready | Deployment and operational validation |

## 11. Related documents

- [Architecture Overview](overview.md)
- [System Model](system-model.md)
- [Runtime](runtime.md)
- [Simulation](simulation.md)
- [Distributed](distributed.md)
- [Transport](transport.md)
- [Verification](../verification/README.md)
- [Safety](../safety/README.md)
