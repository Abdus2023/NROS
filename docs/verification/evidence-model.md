# Evidence Model

NROS documentation uses evidence to distinguish intended behavior from demonstrated behavior.

## Evidence classes

| Evidence | Establishes | Does not establish |
|---|---|---|
| Design text | intended behavior | implementation |
| Specification | required contract | conformance |
| Scaffold | structural presence | functional correctness |
| Simulation | modeled behavior | physical behavior |
| Unit test | tested local behavior | system-wide correctness |
| Integration test | cross-component behavior | hardware validation |
| Benchmark | measured performance for a defined setup | universal performance |
| Hardware validation | behavior on specified hardware | arbitrary hardware |

## Evidence requirements

Claims should identify the artifact, test, measurement, or validation result that supports them. When evidence is incomplete, documentation should state the limitation explicitly.

## Negative evidence

An absent implementation, failing test, unavailable hardware path, or unverified integration is meaningful repository state. Documentation should not convert missing evidence into a positive claim.
