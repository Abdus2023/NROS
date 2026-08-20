# Validation

Validation establishes that a system behaves acceptably for a defined use case and environment.

## Validation levels

- **Integration validation** — multiple components operate together as intended.
- **System validation** — an end-to-end workflow satisfies its acceptance criteria.
- **Simulation validation** — modeled behavior satisfies explicitly defined scenarios.
- **Hardware validation** — the specified implementation is exercised on identified physical hardware.
- **Operational validation** — deployment and observability procedures work under defined conditions.

## Boundary

Simulation can validate software behavior against a model; it cannot by itself establish physical hardware behavior, electrical safety, timing under real hardware load, or production suitability.

Hardware validation must record the target hardware, software revision, configuration, procedure, and observed result.

## Acceptance

A validation claim should point to explicit acceptance criteria. Passing a test without defined acceptance criteria is useful evidence, but it should not automatically be described as full system validation.
