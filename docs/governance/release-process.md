# Release Process

A release should be treated as an evidence-backed snapshot of the repository, not only a version tag.

## Release checks

Before a release is described as ready, verify:

1. source and documentation are synchronized;
2. intended tests have executed successfully;
3. documented interfaces match the implementation;
4. known limitations are recorded;
5. safety-sensitive claims have appropriate evidence;
6. release notes distinguish implemented, simulated, and planned capabilities.

## Release status

Do not infer production readiness from a successful build or test suite alone. Release criteria must be explicit for the target deployment context.
