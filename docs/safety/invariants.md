# Safety Invariants

A safety invariant is a property that must remain true within an explicitly defined operating and trust boundary.

## Required properties of an invariant

Each invariant should identify:

1. the protected property;
2. the system boundary;
3. assumptions;
4. enforcement mechanism;
5. verification method;
6. evidence status;
7. residual limitations.

## Typical NROS safety boundaries

Potential boundaries include:

- command validation;
- ownership and lifetime of shared resources;
- transport integrity;
- watchdog and timeout behavior;
- actuator command limits;
- fault containment;
- deterministic shutdown.

These are documentation categories, not claims that every mechanism is currently implemented.

## Evidence

An invariant becomes a verified property only when the repository contains appropriate implementation and verification evidence for its stated boundary.
