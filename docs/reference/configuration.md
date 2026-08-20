# NROS Configuration Reference

> **Status:** Repository-grounded reference.
>
> This page documents configuration surfaces that can currently be grounded in the `arena/documentation-rewrite` repository. It deliberately does not invent runtime configuration files, environment variables, or precedence rules that are not implemented.

## 1. Current workspace configuration

The root `Cargo.toml` is the authoritative workspace configuration for the Rust workspace. It currently defines:

- 12 workspace members;
- Cargo resolver `2`;
- workspace package version `0.1.0`;
- Rust edition `2021`;
- project authorship metadata;
- MIT license metadata;
- repository metadata;
- workspace description;
- an empty shared-dependency section reserved for future ecosystem dependencies. fileciteturn111file0

The manifest is build-system configuration, not proof of a separate NROS runtime configuration system.

## 2. Workspace members

The currently declared members are:

```text
crates/nros-types
crates/nros-core
crates/nros-node
crates/nros-hal
crates/nros-transport
crates/nros-distributed
crates/nros-cli
crates/nros-sim
crates/nros-studio
crates/nros-macros
crates/nros
crates/nros-audit
```

These values should be derived from `Cargo.toml` rather than duplicated manually in other configuration documentation. fileciteturn111file0

## 3. CLI configuration surfaces

The current CLI entry point parses command-line options directly for several commands, including:

```text
init       --template=
build      --profile= --target=
run        --inspect
topic      <action> [name] [data]
record     --output= --duration=
replay     --speed= --loop
analyze    --bandwidth / --latency / --timing / --graph
profile    --duration= --focus=
fleet      --version= --canary=
migrate    <action> <path> [output]
check      --timing --graph
```

These are CLI argument surfaces, not evidence of a persistent configuration schema. The implementation also contains command-specific fallback values; those should not be mistaken for global configuration defaults. fileciteturn109file0

## 4. Runtime configuration status

No separate repository-grounded runtime configuration schema is established by the current reference evidence.

In particular, this page does **not** currently claim the existence of:

- `nros.yaml` or another canonical runtime configuration file;
- a stable environment-variable namespace;
- configuration-file discovery rules;
- configuration precedence between file, environment, and CLI;
- a validated runtime configuration schema;
- persistent deployment configuration.

Those should be documented only after their parser and consuming implementation are identified.

## 5. Build profiles

The CLI accepts the profile names `debug`, `release`, `realtime`, and `embedded`. fileciteturn109file0

However, accepting a profile name is not equivalent to proving that a corresponding optimized build configuration exists and is executed. The current `build` command explicitly reports a simulated backend. fileciteturn109file0

Therefore this reference does not assign unverified compiler flags, linker settings, allocation policies, or timing guarantees to those profiles.

## 6. Configuration precedence

No general precedence chain is currently established by repository evidence.

The following should **not** be assumed until implemented and verified:

```text
Defaults
   ↓
Config file
   ↓
Environment
   ↓
CLI
```

This is a common design, not an NROS fact.

## 7. Configuration validation

A future runtime configuration system should document, where implemented:

- schema/version;
- required fields;
- defaults;
- allowed values/ranges;
- cross-field constraints;
- error messages/status;
- compatibility/migration;
- effective configuration after precedence resolution.

Until those consumers exist, design-level configuration belongs in the specification layer.

## 8. Source-of-truth rule

Configuration documentation must distinguish:

```text
Configuration key exists
        ↓
Parser recognizes it
        ↓
Value is validated
        ↓
Value reaches consumer
        ↓
Consumer changes behavior
```

Only the later states establish that a configuration option is operationally meaningful.

## 9. Verification requirements

| Claim | Evidence |
|---|---|
| Configuration schema exists | Schema/parser source |
| Default is correct | Parser/default tests |
| Environment variable works | Environment integration test |
| CLI override works | CLI integration test |
| Precedence is correct | Precedence matrix tests |
| Invalid configuration is rejected | Negative-path tests |
| Configuration reaches runtime | End-to-end observation |
| Migration is compatible | Versioned migration tests |

## 10. Related documentation

- [Reference Index](README.md)
- [Crates](crates.md)
- [CLI](cli.md)
- [Environment](environment.md)
- [API](api.md)
- [Specifications](../specifications/README.md)
- [Architecture](../architecture/README.md)
