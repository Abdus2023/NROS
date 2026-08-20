# Test Strategy

Verification should proceed from small, deterministic checks toward system-level validation.

## Layers

1. **Static checks** — formatting, linting, type checking, and structural validation.
2. **Unit tests** — local component behavior and invariants.
3. **Integration tests** — interaction between crates, services, transports, and runtime components.
4. **End-to-end tests** — complete workflows across defined boundaries.
5. **Simulation tests** — deterministic modeled environments and replayable scenarios.
6. **Hardware validation** — behavior observed on explicitly identified target hardware.

## Interpretation

A passing lower-level test is evidence for the tested behavior only. It should not be generalized to unrelated system properties.

## Reproducibility

Verification records should identify the relevant revision, environment, command, configuration, and result whenever those details materially affect interpretation.
