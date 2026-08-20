# Deployment

Deployment documentation defines the boundary between development artifacts and deployable NROS systems.

## Deployment stages

1. Development — local source and tests.
2. Integration — multiple components exercised together.
3. Target validation — target platform or hardware exercised.
4. Production — release criteria satisfied and operational evidence recorded.

## Deployment checklist

- verify the intended target and supported platform;
- build from a known revision;
- run the applicable test suite;
- record configuration and environment assumptions;
- verify observability and diagnostics;
- confirm safety gates for safety-sensitive deployments;
- retain release and validation evidence.

## Boundary

A successful local build does not establish target-hardware or production readiness.
