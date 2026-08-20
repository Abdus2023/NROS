# Troubleshooting

Use a layered investigation rather than assuming that a visible interface represents a complete subsystem.

## Investigation order

1. Confirm repository revision and environment.
2. Reproduce the issue.
3. Check build and test output.
4. Identify the concrete implementation path.
5. Separate simulated behavior from real execution.
6. Inspect logs and diagnostics.
7. Reduce the problem to a minimal failing case.
8. Record the evidence and remediation.

## Common documentation traps

- command exists, but backend operation is simulated;
- API exists, but implementation is incomplete;
- test passes only a local unit boundary;
- benchmark measures a mock or synthetic workload;
- documentation describes a planned feature as current behavior.

When evidence is insufficient, document the capability as unknown or incomplete rather than upgrading its status.
