# NROS CLI Reference

> **Status:** Repository-grounded reference.
>
> This page documents the command dispatch currently present in `crates/nros-cli/src/main.rs`. Many commands are explicitly implemented as simulation/scaffold paths; those paths MUST NOT be documented as functional runtime capabilities.

## 1. Invocation

```text
nros <COMMAND> [OPTIONS]
```

The current binary prints help when invoked without a command and supports explicit help aliases.

## 2. Command inventory

| Command | Current implementation status |
|---|---|
| `init` | Dispatched to `CLI::run`; inspect backend before claiming project creation |
| `build` | **Simulated** result explicitly emitted |
| `run` | Dispatched to `CLI::run`; inspect backend before claiming node execution |
| `topic` | **Simulated** discovery/transport result explicitly emitted |
| `record` | **Simulated**; no file serialization backend |
| `replay` | **Simulated**; no replay backend |
| `analyze` | **Simulated**; fixed analysis values are emitted |
| `profile` | **Simulated**; no profiler/flamegraph backend |
| `fleet` | **Simulated**; no device/TLS/auth/OTA backend |
| `migrate` | **Simulated** migration scaffold |
| `check` | **Simulated** static-analysis scaffold |
| `help` | Implemented help path |

The source itself emits `verified:false` for the simulated operations. fileciteturn109file0

## 3. `init`

```text
nros init <name> [--template=<template>]
```

Documented templates in the CLI entry point include:

```text
mobile_base
manipulator
perception
basic
```

The command constructs `Command::Init` and dispatches it to `CLI::run`. The reference does not claim that a project is actually generated until the backend implementation and execution are independently verified. fileciteturn109file0

## 4. `build`

```text
nros build [--profile=<profile>] [--target=<target>]
```

Profiles parsed by the entry point include:

- `debug`
- `release`
- `realtime`
- `embedded`

The current entry point explicitly prints a `simulated` result and states that the Cargo backend is not installed in this path. The reported sizes are simulated and `verified:false` is emitted. fileciteturn109file0

Therefore this page does **not** claim that `nros build` currently performs a verified build.

## 5. `run`

```text
nros run [--inspect] [node]
```

The entry point constructs `Command::Run { node, inspect }`. The `--inspect` option is intended by the current source to request Studio inspection, but this reference does not claim that a live Studio endpoint is actually opened without backend verification. fileciteturn109file0

## 6. `topic`

```text
nros topic <list|info|echo|hz|bw|pub> [name] [data]
```

The dispatcher recognizes:

- `list`
- `info`
- `echo`
- `hz`
- `bw`
- `pub`

The entry point supplies fallback example names such as `/cmd_vel`, `/chatter`, and `/camera/image` when arguments are omitted. It then explicitly reports the operation as `simulated`, stating that topic discovery and transport are not implemented in this path. fileciteturn109file0

These names are therefore **examples/default arguments**, not evidence that those topics exist in a live runtime.

## 7. `record`

```text
nros record <topics...> [--output=<file>] [--duration=<seconds>s]
```

The entry point constructs a recording command but explicitly reports a simulated operation and states that no file serialization backend is implemented. fileciteturn109file0

Do not document the command as producing a valid `.nros` recording until executable artifact evidence exists.

## 8. `replay`

```text
nros replay <file> [--speed=<factor>] [--loop]
```

The current path parses the input file, speed, and loop flag, then emits an explicit simulated result. The source states that opening, validating, reading, and scheduling the recording are not implemented. fileciteturn109file0

## 9. `analyze`

```text
nros analyze <file> --bandwidth|--latency|--timing|--graph
```

The command selects an analysis type, but the current entry point explicitly reports a simulated backend and fixed latency values without reading the file. fileciteturn109file0

Those printed metrics MUST NOT be treated as measured NROS performance evidence.

## 10. `profile`

```text
nros profile [--duration=<seconds>s] [--focus=<function>]
```

The current dispatcher constructs the profile command but explicitly reports a simulated profiler backend and does not generate a verified flamegraph. fileciteturn109file0

## 11. `fleet`

```text
nros fleet <list|deploy|status|exec> [OPTIONS]
```

Supported dispatcher actions are:

- `list`
- `deploy --version=<version> --canary=<percent>`
- `status`
- `exec <robot> <command>`

The current implementation explicitly describes fleet behavior as simulated and identifies missing TLS, authentication, OTA, artifact verification, device communication, rollback, and health endpoints. fileciteturn109file0

## 12. `migrate`

```text
nros migrate <analyze|convert> <path> [output]
```

The entry point supports `analyze` and `convert`, but explicitly identifies the migration engine as a scaffold without AST transformation, source rewriting, message conversion, or validation. fileciteturn109file0

## 13. `check`

```text
nros check --timing --graph
```

The current dispatcher constructs the check command but explicitly reports a simulated static-analysis gate. The source states that graph loading, YAML parsing, cycle detection, and timing analysis are not implemented in this path. fileciteturn109file0

## 14. Exit status

The entry point exits with status `1` when `CLI::run` returns an error. Successful command dispatch otherwise reaches normal process termination. Unknown commands are reported as unsupported while help is displayed. fileciteturn109file0

The exact exit-code contract should be expanded only after it is verified across the CLI implementation and tests.

## 15. Evidence rule

The CLI currently demonstrates an important distinction:

```text
Command exists
      ↓
Arguments are parsed
      ↓
Command object is constructed
      ↓
Backend exists
      ↓
Real state transition occurs
      ↓
Observed result is verified
```

The first three states do not establish the latter three.

## 16. Related documentation

- [Reference Index](README.md)
- [Crates](crates.md)
- [Configuration](configuration.md)
- [Environment](environment.md)
- [API](api.md)
- [Specifications](../specifications/README.md)
- [Verification](../verification/README.md)
