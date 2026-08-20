# NROS Conceptual Overview

NROS (Native Robotics Operating System) is a robotics middleware and runtime project aimed at providing a smaller, more deterministic foundation for robot software.

The project focuses on several recurring systems concerns:

- deterministic execution and scheduling;
- efficient local communication;
- explicit hardware boundaries;
- typed interfaces between components;
- simulation and observability as first-class development concerns; and
- a development model that can distinguish specification from verified implementation.

## Conceptual layers

At a high level, NROS is described as a stack of:

```text
Robot applications
       │
High-level APIs and tools
       │
Core services
       │
Communication substrate
       │
Runtime / scheduling
       │
Hardware abstraction
       │
Hardware / operating system
```

This is a conceptual model, not a claim that every layer is production-complete in the current repository.

## Design boundary

NROS documentation intentionally separates three questions:

1. **What should the system do?** — specifications and design.
2. **What exists in the repository?** — implementation documentation.
3. **What has been demonstrated?** — tests, benchmarks, integration evidence, and validation.

Keeping these questions separate is a core documentation principle.

## Current project reality

The repository contains a mixture of implemented components, scaffolding, simulations, specifications, and experimental artifacts. Readers should therefore consult the [verification documentation](../verification/README.md) before treating a documented capability as validated behavior.

## Next

- Read the [Design Principles](design-principles.md).
- Continue to the [Architecture documentation](../architecture/README.md).
- Review the [Verification model](../verification/README.md) to understand evidence levels.
