# NROS Evidence / Capability Registry — per AUDIT.md Pass 3-5 Recommendation

> For every advertised feature: feature, specification, implementation, status, test, benchmark, hardware_validation, claim_allowed
> Statuses: SPECIFIED, SCAFFOLDED, SIMULATED, IMPLEMENTED, TESTED, BENCHMARKED, INTEGRATION-TESTED, HARDWARE-VALIDATED, PRODUCTION-READY, SAFETY-QUALIFIABLE

This file addresses P0 remediation from AUDIT.md: separate executable fiction from verified implementation.

## Status Definitions

- **SPECIFIED**: Described in DESIGN.md, no code yet
- **SCAFFOLDED**: API skeleton exists, internal placeholder
- **SIMULATED**: Runs but uses `Vec<u8>` instead of DMA, `random_bool` instead of Raft, `[1]+data` instead of LZ4, `pseudo_rand()` instead of real telemetry (executable fiction)
- **IMPLEMENTED**: Real logic, not just println, but not yet thoroughly tested
- **TESTED**: Has unit tests covering happy path + some edge cases
- **BENCHMARKED**: Has benchmark artifact with environment info, not just `#[test]` assert
- **INTEGRATION-TESTED**: Tested with other crates (e.g., core + node + hal)
- **HARDWARE-VALIDATED**: Tested on real hardware (e.g., V4L2 camera, LiDAR, Jetson)
- **PRODUCTION-READY**: CI passes, clippy clean, Miri/loom reviewed, docs complete
- **SAFETY-QUALIFIABLE**: Ready for ISO 26262 / IEC 61508 process (requires formal proof, MC/DC, etc.)

## Core — `crates/nros-core`

| Feature | Spec | Implementation | Status | Test | Benchmark | HW Validation | Claim Allowed |
|---------|------|----------------|--------|------|-----------|---------------|---------------|
| SPSC ring buffer lock-free | DESIGN.md §14.1 | `crates/nros-core/src/lib.rs` RingBuffer | IMPLEMENTED → TESTED after Safety Gate v0.1 | `test_spsc_ordering`, `test_double_reserve_prevention` | `benchmark_latency_monotonic` ignored, not CI gate | Not required | Yes (with guard API) |
| Generic T destruction | DESIGN.md §14.1 | `Drop for RingBuffer` drains [read,write) + `ReadGuard::drop drop_in_place` | IMPLEMENTED → TESTED | `test_generic_t_destruction` DropCounter | N/A | N/A | Yes |
| WriteGuard single outstanding | AUDIT CORE-001 | `write_reserved: AtomicBool` CAS | IMPLEMENTED → TESTED | `test_double_reserve_prevention` | N/A | N/A | Yes |
| ReadGuard owns slot | AUDIT CORE-002 | `ReadGuard` Deref + Drop advances read_idx | IMPLEMENTED → TESTED | `test_read_guard_lifetime` | N/A | N/A | Yes |
| Monotonic clock | AUDIT CORE-007 | `MonotonicTimestamp` Instant | IMPLEMENTED | Manual | N/A | N/A | Yes |
| Backpressure policy | AUDIT CORE-009 | `BackpressurePolicy::ReturnNone` + `is_full()` | SCAFFOLDED | `test_ring_buffer_full` | N/A | N/A | No (only ReturnNone) |
| MPMC | DESIGN.md §14.2 | None | SPECIFIED | None | None | None | No |
| Shared memory memfd_create + mmap | DESIGN.md §14.2 | None (comment placeholder) | SPECIFIED | None | None | None | No |
| FD passing for large payloads | DESIGN.md §14.2 | None | SPECIFIED | None | None | None | No |

## Node — `crates/nros-node`

| Feature | Spec | Implementation | Status | Test | Benchmark | HW Validation | Claim Allowed |
|---------|------|----------------|--------|------|-----------|---------------|---------------|
| Lifecycle states Unconfigured/Inactive/Active/Finalized | DESIGN.md §3.1 | `crates/nros-node/src/lib.rs` | IMPLEMENTED | `test_lifecycle` | N/A | N/A | Yes |
| Parameter runtime validation type/min/max | DESIGN.md §17.3 | `Parameter::validate()` | IMPLEMENTED → TESTED | `test_parameter_validation` | N/A | N/A | Yes (runtime) |
| Compile-time bounds checking / MDL compiler | DESIGN.md §5.1 | None | SPECIFIED | None | None | None | No |
| Compile-time units / graph validation | DESIGN.md §5.2 | None | SPECIFIED | None | None | None | No |
| Proc macro `#[nros::node]` ergonomic API | DESIGN.md §3.1 | None (generated code in old CLI template) | SPECIFIED | None | None | None | No |
| Real-time callback deadline monitoring | DESIGN.md §4.1 | `ExecutionStats` deadline_misses | IMPLEMENTED → TESTED | `test_performance_timing` | N/A | N/A | Yes (with Instant) |
| Emergency stop atomic flag | DESIGN.md §15.2 | `emergency_stop: Arc<AtomicBool>` | IMPLEMENTED → TESTED | `test_emergency_stop` | N/A | N/A | Yes |

## HAL — `crates/nros-hal`

| Feature | Spec | Implementation | Status | Test | Benchmark | HW Validation | Claim Allowed |
|---------|------|----------------|--------|------|-----------|---------------|---------------|
| Unified sensor trait | DESIGN.md §6.1 | `Sensor` trait | IMPLEMENTED | `test_sensor_manager` | N/A | N/A | Yes |
| Device metadata / capabilities | DESIGN.md §16.1 | `DeviceInfo`, `SensorCapabilities` | IMPLEMENTED | `test_capabilities` | N/A | N/A | Yes |
| Camera driver prototype | DESIGN.md §6.1 | `CameraDriver` | IMPLEMENTED | `test_camera_dma` | N/A | Not hardware | Yes (prototype) |
| DmaBuffer | DESIGN.md §16.4 | `SimulatedDmaBuffer { data: Vec<u8> }` SIMULATED + `RealDmaBuffer { backing, is_gpu_accessible }` SCAFFOLDED (would use memfd_create + mmap + DMA-BUF) | SIMULATED (Real) vs IMPLEMENTED (Simulated distinction visible via trait DmaBufferTrait) | `test_camera_dma` (checks id, not DMA) | None | None | Partially fixed P1: API now `SimulatedDmaBuffer` vs `RealDmaBuffer` makes fiction visible per AUDIT |
| Zero-copy camera path | DESIGN.md §16.4 | `buf.data.clone()` (acknowledged safety copy) | SIMULATED | None | None | None | No — currently copies |
| Real V4L2 / DMA-BUF / GPU sharing | DESIGN.md §16.4 | None (comment says real would use memfd_create) | SPECIFIED | None | None | None | No |
| Multi-sensor sync 10ms tolerance | DESIGN.md §16.2 | `SensorSynchronizer` | IMPLEMENTED → TESTED | `test_synchronizer_tolerance` | N/A | N/A | Yes (software sync) |
| Hardware-triggered capture GPIO | DESIGN.md §16.2 | `TriggerMode::External{pin}` stored but not using GPIO | SCAFFOLDED | None | None | None | No |

## Transport — `crates/nros-transport`

| Feature | Spec | Implementation | Status | Test | Benchmark | HW Validation | Claim Allowed |
|---------|------|----------------|--------|------|-----------|---------------|---------------|
| UDP transport | DESIGN.md §14.3 | `UdpSocket` bind, `add_peer`, `publish`, `receive` | IMPLEMENTED | `test_udp_loopback` | Pending | N/A | Yes (basic) |
| TCP transport | DESIGN.md §14.3 | `TcpListener`, `TcpStream`, `set_nodelay` | IMPLEMENTED | Manual | Pending | N/A | Yes (basic) |
| Serialization 48B Twist | DESIGN.md §14.3 | `Serializable` trait `Vector3` 24B + `Twist` 48B | IMPLEMENTED | `test_twist_serialization` | N/A | N/A | Yes (48B measured) |
| Compression LZ4 | DESIGN.md §14.3 | `MockCompressionEngine` [1]+data SIMULATED + `Lz4CompressionEngine` [2]+data SCAFFOLDED would use lz4_flex, trait `CompressionEngineTrait` with `is_simulated()`, `name()` | SIMULATED (Mock) vs SCAFFOLDED (Lz4) distinction visible — `CompressionEngine = MockCompressionEngine` alias backward compat | `test_compression_flag` (flag only) | None | None | Partially fixed P1: separated Mock vs Lz4, ratio still assumed not measured, must not be benchmark evidence |
| Checksum verification | DESIGN.md §14.3 | `with_checksum` wrapping_add + new `verify_checksum()` checks sum vs expected, now called in `UdpTransport::receive` and `TcpTransport::receive` before decompress | IMPLEMENTED (simple sum, not CRC32, but verification now enforced) | Manual via loopback test | None | None | Fixed P1: checksum now verified, corruption detected per AUDIT Pass 3 |
| Zero-copy network serialization FlatBuffers | DESIGN.md §14.3 | `Vec<u8>` serialize → packet Vec → UDP send, decomp Vec → deserialize → new T | SCAFFOLDED / SIMULATED | None | None | None | No |
| Multicast | DESIGN.md §14.3 | `multicast_group()` now parses group IP, `set_multicast_ttl_v4(ttl)` + `join_multicast_v4(group, UNSPECIFIED)` real join per §14.3 | IMPLEMENTED (real) | Manual test via `cargo test test_udp_loopback` not yet dedicated multicast test | None | None | Fixed P1: now real multicast join, not stub println |
| Service discovery mDNS | DESIGN.md §14.3 | `ServiceDiscovery` broadcast | IMPLEMENTED (basic) | `test_service_discovery` | None | N/A | Yes (basic) |

## Distributed — `crates/nros-distributed`

| Feature | Spec | Implementation | Status | Test | Benchmark | HW Validation | Claim Allowed |
|---------|------|----------------|--------|------|-----------|---------------|---------------|
| RobotId, NodeRole, term, peer registry | DESIGN.md §17.1 | `RobotId`, `NodeRole`, `NodeInfo` | IMPLEMENTED | `test_leader_election` (state) | N/A | N/A | Yes |
| Leader election state machine | DESIGN.md §17.1 | `LeaderElection` with term, role, votes, heartbeat timers | IMPLEMENTED | `test_leader_election` | N/A | N/A | Yes (scaffolding) |
| Real Raft RequestVote RPC / log replication | DESIGN.md §17.1 | `SimulatedElection = LeaderElection` uses `random_bool(0.7)` SIMULATED + `RaftElection` SCAFFOLDED with fields current_term, voted_for, log, commit_index, last_applied, methods request_vote_rpc, append_entries_rpc placeholder | SIMULATED (SimulatedElection) vs SCAFFOLDED (RaftElection) distinction visible via trait `ElectionEngine` is_simulated(), name() | `test_leader_election` state transitions | None | None | Partially fixed P1: separated SimulatedElection vs RaftElection, must not call current election Raft without implementing protocol |
| Distributed state replication | DESIGN.md §17.1 | `DistributedState::set()` local HashMap + `replicate()` returns Ok(()) stub | SIMULATED | `test_distributed_state` (local only) | None | None | No |
| Consistent hash abstraction | DESIGN.md §17.1 | `consistent_hash_shard()` FNV-1a | IMPLEMENTED | `test_consistent_hash` determinism | N/A | N/A | Yes (hash only) |
| Fleet coordination | DESIGN.md §17.1 | `FleetCoordinator::distribute_tasks` capability matching | IMPLEMENTED (basic) | Manual | None | None | Yes (basic) |
| Split-brain prevention / commit index / majority persistence | Raft §5.2 | None | SPECIFIED | None | None | None | No |

## Simulation — `crates/nros-sim`

| Feature | Spec | Implementation | Status | Test | Benchmark | HW Validation | Claim Allowed |
|---------|------|----------------|--------|------|-----------|---------------|---------------|
| Vector3/Quaternion/Transform/RigidBody | DESIGN.md §7.3 | Math types, fixed timestep accumulated_time | IMPLEMENTED → TESTED | `test_vector_ops`, `test_quaternion_euler_roundtrip` | N/A | N/A | Yes |
| Physics integration semi-implicit Euler | DESIGN.md §7.3 | `PhysicsEngine::integrate()` gravity, damping, quaternion integration | IMPLEMENTED | `test_physics_fall` | N/A | N/A | Yes (model) |
| Bullet backend | DESIGN.md §7.3 nros.toml physics_engine=bullet | `SimulatedPhysicsEngine` IMPLEMENTED semi-implicit Euler + `BulletPhysicsEngine` SCAFFOLDED would use btDiscreteDynamicsWorld, stepSimulation, trait `PhysicsEngineTrait` is_simulated(), name() | SIMULATED (Simulated) vs SCAFFOLDED (Bullet) distinction visible | `test_physics_fall`, `test_sim_world_spawn` | None | None | Partially fixed P1: separated SimulatedPhysicsEngine vs BulletPhysicsEngine |
| Sensor simulation camera/lidar/imu | DESIGN.md §7.3 | `SimulatedCamera` gradient + white boxes, `SimulatedLidar` raycast dot>0.99, `SimulatedIMU` noise | IMPLEMENTED | `test_sim_world_spawn`, `test_lidar_raycast` | N/A | N/A | Yes (simulated sensors) |
| Deterministic replay | DESIGN.md §7.1 `nros replay` | `WorldState` recording vec, `replay()` prints poses | IMPLEMENTED | `test_deterministic_replay` | N/A | N/A | Yes (basic) |
| Sim/real parity same node/message/timing | DESIGN.md §7.3 | Data structures compatible, but no automated test proving SIM ≈ REAL | SCAFFOLDED | None | None | None | No |

## Studio — `crates/nros-studio`

| Feature | Spec | Implementation | Status | Test | Benchmark | HW Validation | Claim Allowed |
|---------|------|----------------|--------|------|-----------|---------------|---------------|
| HTTP server dashboard serving | DESIGN.md §7.2 | `StudioServer` TcpListener, serves `index.html` | IMPLEMENTED | `test_studio_state` | N/A | N/A | Yes |
| Node/topic/TF model | DESIGN.md §7.2 | `NodeInfo`, `TopicInfo`, `TfFrame` hard-coded in `new()` | IMPLEMENTED | `test_nodes_topics` | N/A | N/A | Yes (model) |
| Metrics API | DESIGN.md §7.2 | `/api/metrics` returns JSON | IMPLEMENTED | Manual | N/A | N/A | Yes (API) |
| SSE streaming architecture | DESIGN.md §7.2 | `/api/stream` text/event-stream 500ms loop 120 msgs | IMPLEMENTED | Manual via curl | N/A | N/A | Yes (architecture) |
| Real node telemetry CPU/memory/latency | DESIGN.md §7.2 | `DemoDataProvider` SIMULATED pseudo_rand + hard-coded nodes + `LiveNrosDataProvider` SCAFFOLDED would collect PerformanceStats ExecutionStats sysinfo, trait `DataProvider` is_simulated(), name() + `to_metric_json` now uses provider and includes provider name + simulated bool in JSON for transparency | SIMULATED (Demo) vs SCAFFOLDED (Live) distinction visible | Manual via preview dashboard | None | Partially fixed P1: separated Demo vs Live provider, makes fiction visible, but still synthetic |
| Live parameter control | DESIGN.md §17.3 | `update_param()` modifies `StudioState.nodes` HashMap, prints hot-reload | SIMULATED | None | None | None | No |
| Real breakpoints / graph introspection | DESIGN.md §7.2 | Buttons trigger alerts, no real debugging | SIMULATED | None | None | None | No |
| 3D TF visualization | DESIGN.md §7.2 | Three.js r160 GridHelper boxes, fetch /api/tf | IMPLEMENTED (basic) | Manual preview | N/A | N/A | Yes (basic) |
| Force-directed layout | Design proposal | `applyForceLayout()` repulsion 8000/dist² attraction 0.02 | IMPLEMENTED | Manual | N/A | N/A | Yes |

## CLI — `crates/nros-cli`

| Feature | Spec | Implementation | Status | Test | Benchmark | HW Validation | Claim Allowed |
|---------|------|----------------|--------|------|-----------|---------------|---------------|
| Command architecture init/build/run/topic/record/replay/analyze/profile/fleet/migrate/check | DESIGN.md §7.1 | `Command` enum, `CLI::run()` match | IMPLEMENTED | `test_build_profile_parsing` | N/A | N/A | Yes (architecture) |
| `nros init` generates buildable project | AUDIT P0 NROS-011 | Old template used `nros::prelude::*` and non-existent crates → not buildable. New template generates plain Rust that compiles (fixed in this commit) | IMPLEMENTED → TESTED | `test_project_name_validation` + golden test `cargo check` in CI | N/A | N/A | Yes after fix |
| Build system realtime/embedded profiles 950KB/480KB | DESIGN.md §7.1 | `BuildSystem::build()` simulates steps 100-500ms, prints size 950KB/480KB | SIMULATED | `test_build_system` size>0 | None | None | No — size is simulated not measured |
| Topic inspector list/info/echo/hz/bw | DESIGN.md §7.1 | `TopicInspector::list()` hard-coded topics with latency 5.2μs etc | SIMULATED | None | None | None | No — not reading real NROS topics |
| Profiler flamegraph | DESIGN.md §4.3 | `Profiler::profile()` prints hard-coded functions 245.3ms 45.2% | SIMULATED | None | None | None | No |
| Fleet deploy canary differential validation hooks atomic rollback | DESIGN.md §8.2 §21.2 | `FleetManager::deploy()` prints stages 1000-1500ms and validation hooks ✓ | SIMULATED | None | None | None | No |

## Performance Claims — `COMPARISON.md`

| Claim | Source | Evidence Required | Current Status | Claim Allowed |
|-------|--------|-------------------|----------------|---------------|
| 6.2 μs mean latency, 780K msg/s | README.md Achieved | Benchmark artifact with CPU model, OS, compiler, commit, affinity, iterations, distribution (per AUDIT Pass 7 §12) | Repository has `benchmark_latency_monotonic` ignored test with monotonic clock but no CI artifact, uses timestamp + OS scheduler + thread migration + CPU freq + cache + busy-spin | 🟡 Repository-reported, not independently verified — downgrade to "Target: <10μs, Prototype measurement: ~Xμs in local run" |
| 46x faster than ROS2, 15x throughput, etc. | COMPARISON.md | Independent reproducible benchmark ROS2 vs NROS same hardware, same message size, same conditions | No ROS2 baseline in this repo | 🔴 Not independently established |

## Overall Verdict (per AUDIT Pass 5)

- Architecture/specification: HIGH
- Rust scaffolding: HIGH
- Core IPC prototype: HIGH (after Safety Gate v0.1)
- Node abstractions: MEDIUM-HIGH
- Simulation: MEDIUM
- CLI: MEDIUM
- Studio: LOW-MEDIUM (after SSE + 3D + force layout, improved to MEDIUM)
- Hardware integration: LOW
- Network advanced features: LOW
- Distributed consensus: VERY LOW
- Real-time kernel: VERY LOW
- Production evidence: LOW → MEDIUM after CI added

**Current branch verdict:** 🟡 **ARCHITECTURE-COMPLETE PROTOTYPE / SAFETY GATE FIXED FOR CORE / VERIFICATION IN PROGRESS**

Next gates per AUDIT:

- P0: Make nros-core sound → Done in this commit (WriteGuard/ReadGuard, Drop, monotonic, benchmark separation)
- P0: CI workflow → Added `.github/workflows/ci.yml` with fmt, check, test, clippy, bench ignored, safety gate tests, nros init golden test
- P0: Make nros init compile → Fixed template to plain Rust that compiles
- P0: One canonical implementation → Docs: `crates/` is authoritative, `implementations/` is archival artifact per §25
- P1: Feature status taxonomy → This file

