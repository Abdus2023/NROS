# NROS vs ROS 2 — Evidence-Bounded Comparison

> **Status:** Comparative analysis, not an independent benchmark or certification report.
>
> This document compares architectural goals and currently documented repository evidence. It intentionally does **not** present design intent, simulated behavior, or repository-reported measurements as universally validated NROS capabilities.

## 1. Purpose

NROS and ROS 2 address overlapping robotics-system concerns, but they make different architectural trade-offs.

This comparison is useful for understanding those trade-offs. It is not a substitute for running equivalent workloads under controlled conditions.

## 2. Architectural comparison

| Area | ROS 2 | NROS design direction | Evidence boundary |
|---|---|---|---|
| Client/runtime model | Mature ROS client-library ecosystem | Rust-native runtime and APIs | Implementation status must be checked per crate |
| Middleware | RMW/DDS ecosystem | Native transport/IPC abstractions | Native abstractions do not by themselves prove zero-copy or lower latency |
| Scheduling | Executor and OS/runtime facilities | Explicit scheduling model | Real-time guarantees require platform-specific validation |
| Hardware | Broad driver ecosystem | Unified HAL direction | Hardware support must be verified against actual drivers/targets |
| Simulation | External/integrated ecosystem | Integrated simulation direction | Simulation is not hardware validation |
| Distribution | DDS-based discovery and communication options | Native distributed architecture direction | Consensus/replication claims require protocol evidence |

## 3. Performance claims

Historical versions of this document contained concrete latency, throughput, memory, startup, power, and real-time numbers. Those numbers are **not retained here as current universal claims** because the repository does not establish independent, reproducible validation for all of the stated conditions.

If a performance number is published, it must include at minimum:

- repository revision;
- exact benchmark command;
- workload and message size;
- hardware and operating-system configuration;
- compiler/toolchain configuration;
- warm-up and measurement methodology;
- statistical summary;
- comparison methodology;
- raw or reproducible benchmark evidence.

### Required claim form

```text
Claim
  ↓
Measurement
  ↓
Environment
  ↓
Repository revision
  ↓
Reproduction procedure
  ↓
Limitations
```

## 4. Real-time and safety

NROS documentation may describe deterministic scheduling, deadline handling, safety boundaries, and real-time design goals. These should not be represented as universal hard-real-time or safety guarantees without target-specific evidence.

In particular:

- a scheduler API is not proof of bounded worst-case latency;
- a deadline field is not proof that deadlines cannot be missed;
- Rust memory safety is not equivalent to functional safety certification;
- a simulation passing is not hardware validation;
- a safety-oriented architecture is not an ISO 26262 or IEC 61508 certification.

See [Safety](docs/safety/README.md) and [Verification](docs/verification/README.md).

## 5. Feature comparison

Features should be described using repository evidence rather than checkmarks inferred from API names.

| Capability | NROS documentation position |
|---|---|
| Publish/subscribe | Architectural and implementation surfaces exist; verify current backend behavior before claiming production readiness. |
| Services/actions | Defined as concepts/specifications; implementation maturity must be verified per interface. |
| Zero-copy | Design goal / conditional capability; requires ownership and transport evidence. |
| Real-time scheduling | Architectural capability; requires target-specific timing evidence for guarantees. |
| Distributed operation | Architectural capability; consensus/replication claims require protocol-level evidence. |
| Simulation | Supported as a development/validation boundary; simulation results do not establish hardware behavior. |
| Studio/observability | Interface and tooling claims require live-provider evidence before being described as operational telemetry. |
| Migration tooling | CLI surfaces may exist while backend operations remain simulated; consult the CLI reference and source implementation. |

## 6. Developer experience

NROS uses Rust and Cargo as primary development technologies. ROS 2 has a mature multi-language ecosystem and extensive tooling.

Neither project should be assigned a universal "easier" or "better" rating without defining the workload, team experience, ecosystem requirements, and measured outcomes.

## 7. When the comparison should be revisited

This document should be updated when NROS has reproducible evidence for a specific comparison, especially:

1. controlled IPC latency benchmarks;
2. throughput benchmarks;
3. scheduler/deadline measurements;
4. memory-footprint measurements;
5. startup measurements;
6. target-hardware validation;
7. independently reproducible safety/real-time evidence.

## 8. Historical record

The previous comparison contained numerous strong numerical and maturity claims. Those claims are intentionally removed from the active comparison surface rather than silently carried forward. Historical Git history remains available for traceability.

The current documentation policy is:

> **No benchmark claim without reproducible benchmark evidence; no safety claim without appropriate validation evidence; no production claim without production-level evidence.**
