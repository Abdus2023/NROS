# Diagnostics

Diagnostics provide structured information for identifying failures and verifying runtime state.

## Diagnostic workflow

1. Identify the exact repository revision.
2. Capture the command, configuration, and environment.
3. Collect logs and structured diagnostic output.
4. Determine whether the observed path is implemented, simulated, or scaffolded.
5. Reproduce the failure with the smallest useful test.
6. Preserve evidence before changing the environment.

## Evidence discipline

Diagnostic output should be treated as evidence only for the behavior it actually observes. A diagnostic field named `verified`, `ready`, or similar is not sufficient proof without a defined verification procedure.
