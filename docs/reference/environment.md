# Environment Reference

This page records environment assumptions that are evidenced by the repository.

## Rust workspace

NROS is organized as a Cargo workspace. The workspace currently declares twelve members, including types, core, node, HAL, transport, distributed, CLI, simulation, Studio, macros, the top-level `nros` crate, and `nros-audit`.

## Platform assumptions

Platform-specific requirements must be documented from source, build configuration, CI, or executable verification. Do not infer hardware support merely from crate names or architecture documents.

## Environment variables

No environment variable is documented here unless a concrete implementation reads it. This prevents speculative configuration from becoming accidental API.
