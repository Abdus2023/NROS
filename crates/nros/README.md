# nros — Facade Crate

> Makes `nros init` generated projects compile with `use nros::prelude::*` and `#[nros::node]` etc.
> Status: SCAFFOLDED-IMPLEMENTED per AUDIT.md Pass 5 — macros are passthrough now, real codegen future.
> Fixes P0 NROS-011: generated app must be buildable.

## Purpose

Before this crate, `nros init` generated:

```rust
use nros::prelude::*;
#[nros::node]
struct MobileBase {
    #[subscribe(topic = "/cmd_vel")]
    cmd_vel: Subscriber<Twist>,
}
```

But workspace had no `nros` crate, so `cargo check` failed → **P0 developer workflow defect**.

This facade crate re-exports all core crates + proc macros, so generated code now compiles.

## What's Implemented

- `lib.rs`:
  - `pub use nros_macros::{node, subscribe, publish, param, service, callback, ...}` — allows `#[nros::node]` and `#[subscribe]` etc
  - `pub mod prelude` re-exports `Publisher, Subscriber, RingBuffer, WriteGuard, ReadGuard, VelocityController, Sensor, UdpTransport, RobotId, SimulationWorld, Duration` etc
  - `VERSION`, `NROS_VERSION`, `init()`, `spin()`, `time` module
  - `pub mod core/node/hal/transport/distributed/sim/studio/cli` re-exports crates for advanced usage
  - `pub mod macros` re-exports `nros_macros::*`

- `nros-macros`:
  - `#[proc_macro_attribute] pub fn node` etc — all passthrough SCAFFOLDED (returns input unchanged), real would generate lifecycle impl, parameter handling, QoS, etc.
  - Allows field attributes like `#[subscribe(topic = "/cmd_vel")]` to compile

## What's Still SCAFFOLDED (per Evidence Taxonomy)

- Real codegen for `#[nros::node]` → lifecycle states, parameter validation, publisher/subscriber wiring, QoS, CPU affinity
- Real `#[callback(realtime=true, deadline_us=1000)]` → deadline monitoring, priority scheduling
- Real `#[param(default=1.0, min=0.1, max=10.0)]` → compile-time bounds checking per DESIGN.md §5.1 MDL
- Real graph validation per §5.2

Currently macros are no-op, so struct definition compiles but doesn't generate extra impl. That's intentional for Safety Gate v0.1 — first make it compile, then add real codegen incrementally with tests.

## Example

`crates/nros/examples/mobile_base.rs`:

```rust
use nros::prelude::*;

#[nros::node]
struct MobileBase {
    #[subscribe(topic = "/cmd_vel")]
    cmd_vel: Subscriber<Twist>,
    #[publish(topic = "/odom")]
    odom_pub: Publisher<Odometry>,
    #[param(default = 1.0, min = 0.1, max = 10.0)]
    max_speed: f64,
}

fn main() {
    nros::init();
    println!("Compiles thanks to facade + passthrough macros");
}
```

Run:

```bash
cargo check -p nros --example mobile_base
cargo run -p nros --example mobile_base
```

## Relation to `nros-cli` Generator

- `nros-cli` `ProjectInitializer::generate_toml` now produces minimal `Cargo.toml` without non-existent crates (plain Rust) that compiles standalone — fixes P0
- Optionally, it could generate `Cargo.toml` with `nros = { path = "/path/to/NROS/crates/nros" }` to use full API with macros — commented example in generated file
- Golden test in `.github/workflows/ci.yml` checks generated project compiles: `cargo check`

## Future

- Implement real proc-macro codegen for `#[nros::node]` → generate `new()`, `on_configure`, `on_activate`, etc from DESIGN.md §3.1
- Implement `#[param]` → compile-time validation via const generics
- Implement `#[subscribe]` / `#[publish]` → generate topic registration + QoS + graph validation per §5.2
- Add `nros-codegen` crate for MDL compiler (§5.1): `*.mdl` → Rust structs with `@unit`, `@range`, `@maxlen`, `@versioned`, `@hash`

## Evidence

- Status: SCAFFOLDED-IMPLEMENTED — compiles, but real codegen future
- Test: `cargo check -p nros --examples` must pass
- Claim allowed: Yes for compilation, No for full codegen yet per EVIDENCE_REGISTRY
