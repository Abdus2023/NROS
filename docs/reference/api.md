# NROS API Reference

> **Status:** Repository-grounded developer reference.
>
> The Rust source is authoritative for API signatures and implementation semantics. This page summarizes the currently identifiable public surfaces and points readers toward the owning crate.

## 1. API sources

The primary API sources are the workspace crates. Documentation should link to concrete modules, types, functions, and traits rather than copying large source listings.

Current public surfaces examined for this reference include:

- `nros-types` canonical data and time types;
- `nros-core` ring-buffer/channel primitives;
- `nros-node` lifecycle, parameters, and execution statistics;
- `nros-cli` command dispatch.

## 2. Canonical types: `nros-types`

`nros-types` is the current canonical source for shared domain types. It defines wall-clock and monotonic time abstractions, geometry, motion messages, sensor data, images, and execution statistics. fileciteturn116file0

### Time

```rust
WallTimestamp::now()
WallTimestamp::to_duration()
MonotonicInstant::now()
MonotonicInstant::elapsed()
MonotonicInstant::elapsed_ns()
MonotonicInstant::duration_since(...)
```

`Timestamp` is a compatibility alias for `WallTimestamp`; `MonotonicTimestamp` aliases `MonotonicInstant`. The source explicitly separates wall-clock time from monotonic elapsed-time measurement. fileciteturn116file0

### Geometry

`Vector3` provides:

- `new(x, y, z)`;
- `zero()`;
- `magnitude()`.

It is `#[repr(C)]`, copyable, and has a zero-valued default. fileciteturn116file0

### Core message types

The canonical crate currently exposes:

```text
Twist
MotorCommand
Odometry
Point3D
PointCloud
ImageFormat
Image
ImuData
ExecutionStats
```

The message structures use `WallTimestamp` for external timestamps. fileciteturn116file0

## 3. IPC API: `nros-core`

The core crate exposes a typed SPSC ring-buffer/channel model. The implementation uses `MaybeUninit<T>` internally and a type-state transition from an uninitialized write guard to an initialized write guard before publication. fileciteturn117file0

### Channel construction

```rust
let (producer, consumer) = nros_core::channel::<T>(capacity);
```

The capacity must be a positive power of two; invalid capacity is rejected by the constructor. fileciteturn117file0

### Producer

The public producer surface includes:

```rust
allocate()
publish_copy(value)
len()
is_empty()
capacity()
```

`allocate()` returns a write guard when a slot is available. `publish_copy()` reserves a slot, initializes it through the safe `write_value` path, and commits it. fileciteturn117file0

### Consumer

The consumer exposes:

```rust
try_recv()
pending()
is_empty()
```

`try_recv()` returns an owning `ReadGuard`. Dropping that guard releases the element and advances the read index. The consumer guard deliberately does not implement `DerefMut`. fileciteturn117file0

### Unsafe escape hatch

`WriteGuard::as_mut_ptr()` and `WriteGuard::init_with_unchecked()` are explicitly unsafe. Their contracts place complete initialization responsibility on the caller. The safe path is `write_value(T)`. fileciteturn117file0

This is a critical API boundary:

```text
Safe write
    → write_value(T)
    → InitializedWriteGuard
    → commit()

Advanced raw path
    → unsafe API
    → caller assumes initialization obligation
```

## 4. Node API: `nros-node`

The node crate currently exposes lifecycle, parameter, and execution-statistics APIs alongside the `VelocityController` implementation. fileciteturn118file0

### Lifecycle

`LifecycleState` currently contains:

```text
Unconfigured
Inactive
Active
Finalized
```

`LifecycleNode` defines callbacks for configure, activate, deactivate, cleanup, shutdown, and state inspection. fileciteturn118file0

### Parameters

`ParameterValue` supports:

```text
Float
Int
String
Bool
```

`Parameter` provides constructors for bounded floating-point and integer parameters and a `validate()` method that checks type compatibility and configured ranges. fileciteturn118file0

`ParameterServer` provides:

```rust
new()
declare(parameter)
get(name)
get_float(name)
get_int(name)
set(name, value)
list()
```

`set()` rejects missing parameters, read-only parameters, type mismatches, and values outside configured numeric ranges. fileciteturn118file0

### Execution statistics

`ExecutionStats` records callback count, total execution time, minimum/maximum execution time, and deadline misses. It provides derived average execution time, maximum/minimum execution time, and miss-rate calculations. fileciteturn118file0

These statistics are implementation facilities; their existence does not by itself prove a real-time guarantee.

## 5. Velocity-controller surface

`VelocityController::new(name)` constructs the example controller with default parameters including wheel geometry, maximum speed, command timeout, angular speed, and safety-limit enablement. fileciteturn118file0

The controller exposes `on_cmd_vel(&Twist) -> Result<MotorCommand, String>` as its velocity-command callback. The implementation measures execution time, checks emergency-stop state, applies configured limits, and computes differential-drive commands. fileciteturn118file0

The source describes a target callback deadline, but this documentation does **not** convert that target into a verified real-time guarantee.

## 6. CLI API boundary

The CLI exposes command-level interfaces documented separately in [CLI Reference](cli.md). The existence of a parsed command or public command enum does not prove that its backend is implemented. Several current commands explicitly report simulated behavior. fileciteturn109file0

## 7. API maturity model

Every API documented by NROS should be classified where evidence permits:

```text
Declared
   ↓
Implemented
   ↓
Unit-tested
   ↓
Integration-tested
   ↓
Verified under target environment
   ↓
Production-qualified
```

These states are independent. A public function existing in Rust source is not sufficient evidence for the later states.

## 8. Error semantics

Where current APIs return `Option`, `Result`, or explicit state values, documentation should preserve those semantics rather than replacing them with prose such as "always succeeds".

Examples:

```text
try_reserve() → Option
try_read()    → Option
publish_copy() → Result
ParameterServer::set() → Result
```

The concrete source remains authoritative for error values and signatures. fileciteturn117file0turn118file0

## 9. Compatibility

`nros-node` re-exports canonical types from `nros-types` and retains compatibility aliases such as `Timestamp`. fileciteturn118file0

This documentation therefore treats the canonical crate as the ownership point for shared type definitions while documenting aliases as compatibility surfaces rather than independent types.

## 10. Verification requirements

| API claim | Evidence |
|---|---|
| Public symbol exists | Source/API inspection |
| Signature is correct | Compiled API |
| Basic behavior works | Unit tests |
| Cross-crate behavior works | Integration tests |
| Safety invariant holds | Negative-path/concurrency tests |
| Timing claim holds | Repeatable target benchmark |
| Hardware behavior works | Target/HIL evidence |
| Production qualification exists | Explicit qualification record |

## 11. Related documentation

- [Reference Index](README.md)
- [Crates](crates.md)
- [CLI](cli.md)
- [Configuration](configuration.md)
- [Environment](environment.md)
- [Specifications](../specifications/README.md)
- [Verification](../verification/README.md)
