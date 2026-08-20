# NROS Environment Reference

> **Status:** Repository-grounded reference.
>
> This page records environment assumptions that can currently be established from repository source, build configuration, CI, or other executable evidence. It intentionally does not manufacture an environment-variable API.

## 1. Workspace environment

NROS is a Cargo workspace with twelve declared members and Cargo resolver `2`. The workspace package metadata specifies Rust edition `2021` and package version `0.1.0`. fileciteturn111file0

These are repository build characteristics, not a complete runtime deployment contract.

## 2. Toolchain assumptions

The repository's Rust configuration establishes edition and Cargo workspace semantics, but an exact supported compiler-version policy should be documented only when the repository pins or otherwise verifies it through toolchain configuration or CI.

A developer should therefore distinguish:

```text
Rust edition
    ≠
Exact compiler version
    ≠
Supported host platform
    ≠
Verified deployment target
```

## 3. Platform assumptions

Platform-specific requirements MUST be derived from one or more of:

- source-level platform gates;
- Cargo target configuration;
- build scripts;
- CI matrices;
- executable tests;
- hardware-in-the-loop evidence.

Crate names, architectural aspirations, and manifest descriptions are insufficient evidence of platform support.

## 4. Environment variables

Repository search currently establishes **no documented NROS-specific environment-variable contract**.

In particular, this reference does not currently define variables such as:

```text
NROS_*
RUST_LOG
NROS_CONFIG
NROS_HOME
```

unless an implementation that reads them is added and verified.

This is intentional: an undocumented environment variable should not silently become a public configuration API.

## 5. Filesystem assumptions

Commands that accept filesystem paths are not, by themselves, evidence of a required directory layout.

For example, CLI commands accepting recording, migration, or output paths should not be documented as requiring a specific `$HOME`, workspace directory, cache directory, or configuration directory unless the implementation actually establishes that behavior. fileciteturn109file0

## 6. Network assumptions

The presence of a transport crate does not establish a particular network interface, port, DNS convention, multicast address, TLS configuration, or service-discovery mechanism.

Those values belong here only when backed by implementation or deployment configuration.

## 7. Hardware assumptions

The presence of `nros-hal` does not, by itself, establish support for a particular board, sensor, bus, DMA engine, actuator, or real-time hardware target.

Hardware support requires corresponding implementation and verification evidence.

## 8. Environment precedence

No NROS-specific environment precedence chain is currently established.

Do not assume:

```text
Built-in defaults
      ↓
Config file
      ↓
Environment
      ↓
CLI
```

unless the repository implements and tests that ordering.

## 9. Reproducibility

An environment claim should identify enough context to reproduce the observation, including where applicable:

- repository revision;
- operating system;
- architecture;
- Rust toolchain;
- Cargo version;
- enabled features;
- target triple;
- required hardware;
- relevant environment variables.

This is especially important for performance, timing, transport, and hardware claims.

## 10. Evidence model

```text
Environment assumption
        ↓
Source/build configuration
        ↓
Executable observation
        ↓
CI / target verification
        ↓
Documented support claim
```

The absence of evidence should be documented as unknown rather than converted into an assumed requirement.

## 11. Verification requirements

| Claim | Evidence |
|---|---|
| OS is supported | CI or executable test |
| Architecture is supported | Target build/test evidence |
| Compiler version is supported | Toolchain configuration + CI |
| Environment variable is supported | Reader + integration test |
| Filesystem location is required | Source + execution evidence |
| Network configuration is supported | Transport/deployment evidence |
| Hardware target is supported | Target/HIL evidence |
| Environment is reproducible | Revision-pinned environment record |

## 12. Related documentation

- [Reference Index](README.md)
- [Crates](crates.md)
- [CLI](cli.md)
- [Configuration](configuration.md)
- [API](api.md)
- [Specifications](../specifications/README.md)
- [Verification](../verification/README.md)
