# NROS Architecture — Canonical Implementation Clarification

> Addresses AUDIT.md Pass 5 finding: repository has two parallel implementation hierarchies `crates/` vs `implementations/` with ambiguous authoritative source.

## Canonical Mapping (P0 Fix)

```
DESIGN.md requirement (e.g., §14.1 Zero-Copy IPC)
        ↓
Artifact #N (e.g., nros-core-implementation per §25)
        ↓
implementations/nros-core-implementation/main.rs (archival artifact, original demonstration)
        ↓
crates/nros-core/src/lib.rs (authoritative current runtime implementation)
        ↓
tests (unit tests in same crate, e.g., test_double_reserve_prevention)
        ↓
benchmarks/results.json (machine-generated artifact with env info per AUDIT Pass 7 §12)
        ↓
CI verification (.github/workflows/ci.yml GitHub Actions PASS)
        ↓
claim_allowed (only if all above pass)
```

### Directory Roles

| Directory | Role | Status | Claim Allowed? |
|-----------|------|--------|----------------|
| `crates/` | **Authoritative current runtime implementation** — 10 crates, dependency-light, guard-based safety, evidence taxonomy implemented | IMPLEMENTED → TESTED for core after Safety Gate v0.1, SIMULATED for some HAL/transport/distributed features with explicit separation | Yes for IMPLEMENTED+TESTED, No for SIMULATED/SCAFFOLDED |
| `implementations/` | **Archival artifact / original demonstration** per DESIGN.md §25 — verbatim original code as submitted, preserved for historical evidence, not actively maintained | ARCHIVAL — may be stale compared to crates/ | No — use crates/ as source of truth, implementations/ as reference for original claim |
| `crates/nros/` + `crates/nros-macros/` | **Facade + proc-macros** — makes `nros init` generated projects compile with `use nros::prelude::*` and `#[nros::node]` etc (P0 fix for NROS-011) — macros currently passthrough SCAFFOLDED, real codegen future | SCAFFOLDED-IMPLEMENTED | Yes for compilation, No for full codegen yet |
| `benchmarks/` | **Machine-generated artifacts** — JSON with CPU model, OS, kernel, rustc version, commit, timestamp, capacity, iterations, affinity, message_size, latency distribution, throughput | SCAFFOLDED — generator exists, artifacts not yet generated on target hardware | No until artifact generated and linked in COMPARISON.md |
| `.github/workflows/` | **CI verification gate** — `ci.yml` with fmt, check, test correctness only, clippy, bench ignored, safety-gate Miri, nros-init golden test — needs manual addition via GitHub web UI due to workflows permission restriction (GitHub App cannot push workflow files) | SCAFFOLDED locally, not yet pushed to remote | No until GitHub Actions PASS |

## Evidence Taxonomy (per AUDIT)

For every advertised feature in README.md, DESIGN.md, COMPARISON.md:

```
SPECIFIED: Described in DESIGN.md, no code
SCAFFOLDED: API skeleton exists, internal placeholder (e.g., RealDmaBuffer::new_scaffolded, Lz4CompressionEngine without lz4_flex, RaftElection without RPC)
SIMULATED: Runs but uses Vec<u8> instead of DMA, random_bool instead of Raft, [1]+data instead of LZ4, pseudo_rand instead of real telemetry (executable fiction) — must be labeled
IMPLEMENTED: Real logic, not just println, but not yet thoroughly tested
TESTED: Has unit tests covering happy path + edge cases (e.g., double reserve, abandoned reservation, DropCounter)
BENCHMARKED: Has benchmark artifact with env info (benchmarks/results.json)
INTEGRATION-TESTED: Tested with other crates (e.g., core + node + hal)
HARDWARE-VALIDATED: Tested on real hardware (V4L2 camera, LiDAR, Jetson)
PRODUCTION-READY: CI passes, clippy clean, Miri/loom reviewed, docs complete
SAFETY-QUALIFIABLE: Ready for ISO 26262 / IEC 61508 process
```

See `EVIDENCE_REGISTRY.md` for full mapping per feature.

## Layered Architecture (per DESIGN.md §2.1)

```
┌─────────────────────────────────────────┐
│   Application Layer (Robot Programs)    │ ← examples in crates/nros/examples/mobile_base.rs using facade
├─────────────────────────────────────────┤
│     High-Level APIs & Tools             │ ← crates/nros-cli (init/build/run/topic/profile/fleet) — P0 fix: init now generates compilable project
├─────────────────────────────────────────┤
│       Core Services Layer               │ ← crates/nros-node (lifecycle, params runtime validation) — compile-time MDL still SPECIFIED
├─────────────────────────────────────────┤
│      Communication Substrate            │ ← crates/nros-core (guard-based SPSC) + crates/nros-transport (UDP/TCP/48B Twist/mDNS, Mock vs Lz4 separation, checksum verification, real multicast)
├─────────────────────────────────────────┤
│       NROS Microkernel/Scheduler        │ ← DESIGN target: preemptive priority scheduling, CPU affinity, NUMA, DMA coordination — currently SPECIFIED, library-level prototype
├─────────────────────────────────────────┤
│         Hardware Abstraction Layer      │ ← crates/nros-hal (SimulatedDmaBuffer vs RealDmaBuffer, Camera/LiDAR/IMU prototypes, sync 10ms)
└─────────────────────────────────────────┘
                          +
         Distributed Computing — crates/nros-distributed (SimulatedElection vs RaftElection, ReplicationMode Simulated vs Real)
         Simulation — crates/nros-sim (SimulatedPhysicsEngine vs BulletPhysicsEngine, sensor sim, deterministic replay)
         Studio — crates/nros-studio (DemoDataProvider vs LiveNrosDataProvider, SSE /api/stream, Three.js TF, force layout)
```

## Current Maturity (after Safety Gate v0.1 + P1 separation)

- **Architecture/design:** 9/10 (vision clear)
- **Core IPC prototype:** 7/10 → 8/10 after Safety Gate (guard-based, DropCounter, monotonic, benchmark separation)
- **Node middleware:** 6.5/10
- **HAL abstraction:** 6/10 (Simulated vs Real distinction now visible)
- **Actual hardware:** 2/10 (still Vec<u8> not memfd)
- **Transport:** 5/10 → 6/10 after multicast real + checksum verification + compression separation
- **Distributed:** 2/10 → 3/10 after Simulated vs Raft distinction
- **Real-time kernel:** 2/10 (still library, not microkernel)
- **Verification evidence:** 3/10 → 5/10 after CI workflow locally + evidence registry + safety docs
- **Production readiness:** 2-3/10 → 3-4/10 after fixes

## Next Milestones (per AUDIT)

1. **Safety Gate v0.1** ✅ Done: ownership + lifetime + destruction + concurrency proof for core
2. **CI Gate** 🟡 Partial: workflow file exists locally, needs manual addition via GitHub UI due to workflows permission
3. **nros init compile** ✅ Done: generates plain Rust that compiles, facade crate makes macro API compile
4. **One canonical implementation** ✅ Done: This doc clarifies crates/ is authoritative, implementations/ archival
5. **Feature status taxonomy** ✅ Done: EVIDENCE_REGISTRY.md
6. **Separate simulation from production APIs** ✅ Done: SimulatedDmaBuffer vs RealDmaBuffer, MockCompression vs Lz4Compression, SimulatedElection vs RaftElection, DemoDataProvider vs LiveNrosDataProvider, SimulatedPhysicsEngine vs BulletPhysicsEngine
7. **Real LZ4 + CRC32** 🟡 Partial: optional features real-compression/real-checksum added, need to enable and test with lz4_flex/crc32fast, add CI job with --features real-compression,real-checksum
8. **Bullet backend** 🔴 SPECIFIED: Need bullet crate integration
9. **Hardware validation** 🔴 Not yet: Need V4L2, LiDAR drivers, Jetson tests
10. **Benchmark artifacts** 🟡 Partial: generator exists, need to run on target hardware and commit results.json, update COMPARISON.md to link artifact commit

## References

- DESIGN.md §2.1 Layered Architecture, §14 Communication Substrate, §15 Real-Time Scheduler, §16 HAL, §17 Distributed
- AUDIT.md Pass 5-7: repository topology, safety audit, correctness proof
- EVIDENCE_REGISTRY.md: full feature mapping
- crates/nros-core/SAFETY.md: Safety Gate v0.1 invariants
- .github/workflows/ci.yml: Build Gate P0
