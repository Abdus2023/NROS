# NROS Studio Architecture

## Purpose

NROS Studio is the observability and operator-facing layer intended to expose runtime state through dashboards, diagnostics, telemetry, and inspection interfaces.

## Conceptual model

```text
NROS Runtime
     │
     ├── Nodes
     ├── Topics
     ├── Transforms
     ├── Metrics
     └── Parameters
          │
          ▼
   Studio Data Interface
          │
          ├── HTTP / REST
          ├── Streaming
          └── Dashboard
```

## Data-source boundary

A dashboard can display static, simulated, replayed, or live data. The presentation layer must therefore identify its data source explicitly.

`DemoDataProvider` and equivalent simulated sources must not be documented as live runtime telemetry.

## Operator responsibilities

Studio is an observability and interaction surface. Safety-critical control remains subject to the runtime's safety boundaries and must not depend solely on a visualization interface.

## Current status

The repository contains a Studio implementation. Its individual endpoints, data providers, and live-runtime integration should be classified through verification evidence before being described as production telemetry capabilities.
