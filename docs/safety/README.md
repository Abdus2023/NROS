# NROS Safety

Safety documentation defines the safety boundary for NROS and provides navigation to invariants, gates, remediation records, and threat analysis.

## Safety areas

- [Invariants](invariants.md) — properties that must remain true within a defined safety boundary.
- [Safety Gates](safety-gates.md) — conditions that must be satisfied before advancing a safety-sensitive state.
- [Remediation](remediation.md) — tracked corrective actions and their evidence.
- [Threat Model](threat-model.md) — threats, assumptions, mitigations, and residual risk.

## Fundamental rule

NROS software architecture, simulation, or passing software tests must not be represented as proof of physical safety unless the required hardware, integration, and validation evidence exists.

Safety claims must identify their boundary, assumptions, evidence, and remaining limitations.
