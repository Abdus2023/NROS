# Threat Model

The NROS threat model identifies assets, trust boundaries, threats, assumptions, mitigations, and residual risk.

## Core assets

- actuator commands;
- sensor data;
- node identity and authorization state;
- communication channels;
- runtime state;
- configuration and deployment artifacts;
- safety-related logs and evidence.

## Threat categories

- malformed or invalid commands;
- compromised or unexpected peers;
- transport disruption;
- stale or replayed data;
- resource exhaustion;
- configuration errors;
- software faults crossing safety boundaries.

## Trust boundaries

```text
Application
    │
    ├── Runtime boundary
    │
    ├── IPC / transport boundary
    │
    ├── Network boundary
    │
    └── Hardware boundary
```

## Evidence boundary

A threat model identifies risks and controls; it does not prove that mitigations are effective. Effectiveness requires implementation and verification evidence appropriate to the threat and operating boundary.
