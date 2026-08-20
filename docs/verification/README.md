# NROS Verification

Verification answers a different question from architecture and specification:

> What does the repository actually demonstrate?

## Verification areas

- [Evidence Model](evidence-model.md) — evidence classes and confidence boundaries.
- [Claims](claims.md) — how documentation claims are tied to evidence.
- [Test Strategy](test-strategy.md) — automated and manual verification layers.
- [Benchmarks](benchmarks.md) — performance measurements and their interpretation.
- [Validation](validation.md) — integration, hardware, and acceptance validation.

## Evidence rule

A design statement, API surface, scaffold, passing unit test, benchmark, and hardware validation are different forms of evidence. None should be silently substituted for another.

## Status vocabulary

Use the repository-wide progression:

`PROPOSED → SPECIFIED → SCAFFOLDED → SIMULATED → IMPLEMENTED → TESTED → BENCHMARKED → INTEGRATION-TESTED → HARDWARE-VALIDATED → PRODUCTION-READY`

A claim may only use a status supported by evidence appropriate to that status.
