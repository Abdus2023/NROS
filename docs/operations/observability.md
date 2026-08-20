# Observability

Observability makes runtime behavior inspectable through logs, metrics, traces, events, and diagnostics.

## Principles

- make component identity explicit;
- preserve timestamps and relevant correlation information;
- distinguish simulated telemetry from live telemetry;
- avoid presenting counters or placeholders as measurements of real hardware;
- retain enough context to reproduce or investigate failures.

## Evidence boundary

A telemetry schema or dashboard is an observability interface. It is not, by itself, evidence that the underlying subsystem is active, complete, or production-ready.
