# NROS Reference

> **Status:** Developer-facing reference documentation.

Reference documentation describes concrete repository interfaces and explains how to use them. It is implementation-facing and must remain synchronized with the code and current verification evidence.

## 1. Documentation hierarchy

```text
Architecture
    ↓
Specification
    ↓
Reference
    ↓
Verification
    ↓
Safety / Operations
```

Reference documentation answers **how to use an existing interface**. It does not create a normative requirement and does not turn a proposed capability into an implemented one.

## 2. Reference areas

| Area | Purpose |
|---|---|
| [Crates](crates.md) | Workspace and crate responsibilities |
| [CLI](cli.md) | Command-line interfaces and supported commands |
| [Configuration](configuration.md) | Configuration surfaces, defaults, and precedence |
| [Environment](environment.md) | Environment variables and platform assumptions |
| [API](api.md) | Public APIs and integration surfaces |
| Getting Started | Installation and first executable workflow |
| Messaging | Concrete message and communication usage |
| Runtime | Runtime construction and lifecycle usage |
| Simulation | Running supported simulation workflows |
| Studio | Using supported observability interfaces |
| Distributed | Configuring supported multi-process/multi-node deployments |
| Examples | Minimal and task-oriented examples |

A reference page should be added only when the corresponding interface or workflow can be grounded in the repository.

## 3. Accuracy rule

Reference pages MUST describe interfaces that exist in the repository at the documented revision.

Proposed or future interfaces belong in specifications or architecture documentation and MUST be labeled accordingly.

## 4. Source of truth

For implementation-facing details, the source code and executable configuration are authoritative. Reference pages explain those interfaces but must not silently redefine them.

When behavior cannot be confirmed from the repository, the reference must state the uncertainty rather than inventing a usage pattern.

## 5. Status markers

Reference pages SHOULD distinguish relevant implementation states, for example:

```text
Implemented
Implemented but limited
Experimental
Deprecated
Unavailable
Planned
```

`Planned` and `Proposed` content must not be presented as currently usable API.

## 6. Examples

Examples SHOULD be:

- minimal;
- executable where practical;
- pinned to documented APIs;
- explicit about prerequisites;
- clear about expected output;
- linked to verification evidence when a behavioral claim matters.

A code block alone is not evidence that the example works.

## 7. Versioning

Reference documentation should identify version-sensitive interfaces where applicable, including:

- API version;
- protocol version;
- configuration schema version;
- CLI version;
- compatibility constraints.

Breaking implementation changes require corresponding reference updates.

## 8. Deprecation

Deprecated interfaces should identify:

1. the deprecated item;
2. current status;
3. replacement, if one exists;
4. removal or migration expectations;
5. compatibility implications.

## 9. Evidence boundary

Reference documentation and verification evidence remain separate:

```text
Reference:
    "Run command X with configuration Y."

Verification:
    "Command X executed successfully under environment Z."
```

The reference describes usage; verification establishes observed behavior.

## 10. Change discipline

When an implementation interface changes, update the affected reference page in the same documentation change whenever practical.

The update should consider:

- API signatures;
- CLI options;
- configuration defaults;
- environment assumptions;
- examples;
- compatibility notes;
- verification links.

## 11. Related documentation

- [Architecture](../architecture/README.md)
- [Specifications](../specifications/README.md)
- [Verification](../verification/README.md)
- [Safety](../safety/README.md)
- [Operations](../operations/README.md)
