# nros-distributed — Distributed Computing System

Raft-like leader election, replicated state, capability-based task scheduling, fleet coordination per DESIGN.md §17.1 and §25 Artifact #5.

## Features

### Raft-like Leader Election — Split-brain prevention

```rust
struct LeaderElection {
    node_id: RobotId,
    role: Arc<Mutex<NodeRole>>, // Leader/Follower/Candidate
    current_term: AtomicU64,
    peers: Arc<Mutex<HashMap<RobotId, NodeInfo>>>,
    votes_received: Arc<Mutex<HashSet<RobotId>>>,
    election_timeout: Duration, // 1500ms
    heartbeat_interval: Duration, // 500ms
    last_heartbeat: Arc<Mutex<Instant>>,
    leader_id: Arc<Mutex<Option<RobotId>>>,
}

impl LeaderElection {
    fn start_election(&self) -> bool { /* increment term, RequestVote RPC, majority */ }
    fn become_leader(&self) // term, heartbeat
    fn send_heartbeat(&self) // AppendEntries empty per Raft §5.2
    fn receive_heartbeat(&self, leader, term) // update term, become follower, reset timer
    fn check_leader_timeout(&self) -> bool // elapsed > election_timeout
}
```

- **Term increment** per §5.2, vote for self, request votes from peers with `should_grant_vote` (real: check log up-to-date + not voted this term)
- **Majority**: `(n/2)+1` for quorum
- **Heartbeat**: leader sends `AppendEntries` empty to maintain authority, prevents split-brain
- **Timeout**: follower detects `elapsed > 1500ms` → starts election

### Distributed State — Replicated + Consistent Hash Ring

```rust
struct DistributedState<T> {
    local_data: Arc<Mutex<HashMap<String, T>>>,
    replication_factor: usize, // 3 per spec
    node_id: RobotId,
    version: AtomicU64,
}

fn set(&self, key: String, value: T) -> Result<u64, String> // version bump, replicate to 3 nodes via hash_ring
fn consistent_hash_shard(&self, key: &str, shards: usize) -> usize // FNV-1a
```

- Mirrors DESIGN.md `DistributedMap` with `hash_ring: ConsistentHashRing`, `peer_shards: Vec<RemoteShardHandle>`
- `get` local if `shard_id == local_shard_id` else remote
- Versioning for conflict detection, real: Raft log replication

### Task Distribution — Capability Matching per §25

```rust
struct Task {
    id: TaskId,
    task_type: String, // "object_detection", "path_planning", "sensor_fusion", "collaborative_mapping"
    priority: u32, // higher first
    requirements: TaskRequirements { min_cpu_cores, min_memory_mb, requires_gpu, required_sensors },
    status: TaskStatus, // Pending/Assigned/Running/Completed/Failed
    assigned_to: Option<RobotId>,
}
```

- `NodeCapabilities`: cpu_cores, memory_mb, has_gpu, sensors, actuators + `matches()` + factories high_end/mid_range/low_end
- `TaskScheduler::submit_task()` atomic counter, priority sorting, capability filtering
- `assign_task()` checks Pending, assigns to capable node with highest CPU (real: score throughput/latency/efficiency per ComputeScheduler §16.3)
- `execute_task()` simulates 80-200ms per type, real: dispatch to `ComputeScheduler::select_device()` auto GPU/NPU/CPU

### Fleet Coordinator — Multi-Robot

```rust
struct FleetCoordinator {
    fleet_id: String, // "warehouse_fleet"
    leader_election: Arc<LeaderElection>,
    task_scheduler: Arc<TaskScheduler>,
    distributed_state: Arc<DistributedState<String>>,
    robots: Arc<Mutex<HashMap<RobotId, NodeInfo>>>,
}

fn register_robot(&self, info: NodeInfo) // add peer + capability log
fn coordinate(&self) // leader election timeout + heartbeat + distribute_tasks if leader
fn distribute_tasks(&self) // filter capable robots, sort by CPU, assign
```

- Automatic leader election + task distribution loop
- Heterogeneous fleet per §16.3: Pi (2 cores no GPU), Jetson (4 cores GPU), Xavier (8 cores 16GB GPU)
- Formation via `#[shared_state(consensus="raft")]` per §17.1
- Fleet status: total robots, leader, term, task stats Display

## Performance (§25)

- Leader election with configurable timeouts, heartbeat prevents split-brain
- Task scheduling based on node capabilities, priority ordering
- Distributed parameter storage versioned
- Failover and recovery automatic via heartbeat timeout

## Tests

- `test_leader_election` — role transitions
- `test_distributed_state` — set/get/delete versioning
- `test_task_scheduling` — submit/assign/stats
- `test_capability_matching` — GPU task requires GPU
- `test_consistent_hash` — same key same shard determinism

Run:
```bash
cargo test -p nros-distributed -- --nocapture
cargo run -p nros-distributed --bin nros-distributed-demo
```

## Relation

- Uses `nros-core` for future `Publisher<Task>` / `Subscriber<TaskStatus>`
- Compute acceleration via `nros-hal` GPU DMA buffers per §16.4
- Fleet YAML orchestration per §21.2: rolling deployment, health checks
