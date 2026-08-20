# implementations/ — Archival Artifact / Original Demonstration

> Clarifies AUDIT.md Pass 5 finding: repository has two parallel hierarchies `crates/` vs `implementations/` with ambiguous authoritative source.

## Roles

| Directory | Role | Status | Authoritative? | Claim Allowed? |
|-----------|------|--------|----------------|----------------|
| `crates/` | **Authoritative current runtime implementation** — 10 crates, Safety Gate v0.1 fixed, P1 executable fiction separated via types/traits | IMPLEMENTED → TESTED for core, SIMULATED with distinction visible for HAL/transport/distributed | **Yes** — source of truth for current implementation, but check EVIDENCE_REGISTRY for per-feature status |
| `implementations/` | **Archival artifact / original demonstration** per DESIGN.md §25 — verbatim original code as submitted for each Artifact #1..#7, preserved for historical evidence of original claim | ARCHIVAL — may be stale compared to `crates/` (e.g., still has old unsafe SPSC, mock compression as [1]+data, random_bool(0.7) Raft) | **No** — use `crates/` as source of truth, `implementations/` as reference for original claim and audit trail |

## Why Two Hierarchies?

The project was built through an **artifact-driven implementation workflow** per DESIGN.md §25:

```
DESIGN.md requirement (e.g., §14.1 Zero-Copy IPC)
    ↓
Artifact #N (e.g., nros-core-implementation per §25)
    ↓
implementations/nros-core-implementation/main.rs (archival artifact, original demonstration)
    ↓
crates/nros-core/src/lib.rs (authoritative current runtime, Safety Gate fixed)
```

This workflow is fine, but repository needs explicit canonical mapping to avoid fixing one copy while leaving other stale.

## Contents

- `nros-core-implementation/main.rs` — original SPSC ring buffer with unsafe API (pre Safety Gate v0.1) — now fixed in `crates/nros-core/src/lib.rs` guard-based
- `nros-node-example/main.rs` — original VelocityController lifecycle demo — now in `crates/nros-node/`
- `nros-hal-sensors/main.rs` — original HAL with `DmaBuffer(Vec<u8>)` + `clone()` zero-copy path — now separated into `SimulatedDmaBuffer` vs `RealDmaBuffer` in `crates/nros-hal/`
- `nros-network-transport/main.rs` — original UDP/TCP + MockCompression [1]+data + multicast stub println — now separated Mock vs Lz4 + real multicast join + checksum verification in `crates/nros-transport/`
- `nros-distributed-system/main.rs` — original Raft-like with `random_bool(0.7)` + replication `Ok(())` — now separated SimulatedElection vs RaftElection + ReplicationMode in `crates/nros-distributed/`
- `nros-cli-tools/main.rs` — original CLI with simulated build sizes 950KB/480KB + hard-coded topics — now labeled SIMULATED and `nros init` generates compilable project in `crates/nros-cli/`
- `nros-simulation-engine/main.rs` — original physics custom Euler + gradient camera + raycast lidar — now separated SimulatedPhysicsEngine vs BulletPhysicsEngine in `crates/nros-sim/`

## Recommendation

- **For users:** Use `crates/` as source of truth, run `cargo check --workspace`, `cargo test --workspace`, examples in `crates/nros/examples/`
- **For auditors:** Compare `implementations/` (original claim) vs `crates/` (current fix) to assess remediation, see `AUDIT.md` and `EVIDENCE_REGISTRY.md`
- **For contributors:** Fix bugs in `crates/` only, not `implementations/` — keep `implementations/` archival for audit trail, or update both and note in commit message

## Evidence

- Status: ARCHIVAL
- Test: None — archival, not tested
- Claim allowed: No — use `crates/` for claims, check `EVIDENCE_REGISTRY.md` for per-feature status
