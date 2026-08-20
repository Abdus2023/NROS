# Development

This section describes how to work on the NROS repository as a developer.

## Guides

- [Setup](setup.md) — prepare the Rust development environment.
- [Workspace](workspace.md) — understand the Cargo workspace and crate boundaries.
- [Coding Style](coding-style.md) — repository conventions and quality expectations.
- [Testing](testing.md) — build and test expectations.
- [Debugging](debugging.md) — diagnose failures without confusing simulation with implementation evidence.
- [Contributing](contributing.md) — contribution workflow and documentation expectations.

## Development principle

Changes should preserve the distinction between architectural intent, implementation, and verification evidence. A new feature is not documented as implemented merely because its API or scaffold exists.
