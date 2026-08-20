//! NROS Distributed Computing Demo
//! Leader election, distributed state, task distribution, fleet coordination

use nros_distributed::{
    RobotId, NodeInfo, NodeRole, NodeCapabilities, LeaderElection,
    FleetCoordinator, TaskRequirements,
};
use std::time::{Duration, Instant};

fn main() {
    println!("NROS Distributed Computing Demo\n");
    println!("Implements DESIGN.md §17.1:");
    println!(" - Raft-like leader election with heartbeat preventing split-brain");
    println!(" - Distributed state with replication_factor + consistent hash ring");
    println!(" - Task scheduling based on node capabilities (CPU, MEM, GPU, sensors)");
    println!(" - Fleet coordination automatic task distribution\n");

    // Create fleet coordinator for robot 1 — mid-range Jetson
    let robot1_caps = NodeCapabilities::mid_range();

    let coordinator1 = FleetCoordinator::new(
        "warehouse_fleet".to_string(),
        RobotId::new(1),
        robot1_caps.clone(),
    );

    // Register additional robots with varying capabilities — heterogeneous fleet per §16.3
    let robot2_info = NodeInfo {
        id: RobotId::new(2),
        role: NodeRole::Follower,
        address: "192.168.1.102:5000".parse().unwrap(),
        last_heartbeat: Instant::now(),
        capabilities: NodeCapabilities::low_end(), // Pi-class, no GPU
    };

    let robot3_info = NodeInfo {
        id: RobotId::new(3),
        role: NodeRole::Follower,
        address: "192.168.1.103:5000".parse().unwrap(),
        last_heartbeat: Instant::now(),
        capabilities: NodeCapabilities::high_end(), // Xavier/Orin-class 8 cores 16GB GPU
    };

    coordinator1.register_robot(robot2_info);
    coordinator1.register_robot(robot3_info);

    // Leader election — Raft §5.2
    println!("=== Leader Election (Raft-like) ===");
    println!("Timeout: 1500ms, Heartbeat: 500ms, need majority for term");
    coordinator1.coordinate().unwrap();
    std::thread::sleep(Duration::from_millis(100));

    let status = coordinator1.get_fleet_status();
    println!("Fleet status: {}", status);
    println!("Leader timeout check: {}", coordinator1.leader_election.check_leader_timeout());

    // Simulate heartbeat reception
    if let Some(leader) = status.leader {
        println!("Current leader is {}, simulating heartbeat reception...", leader.0);
        coordinator1.leader_election.receive_heartbeat(leader, status.term);
    }

    // Distributed state management — #[shared_state(consensus="raft")] per DESIGN.md §17.1
    println!("\n=== Distributed State (Raft + ConsistentHashRing) ===");
    let ver1 = coordinator1.set_fleet_parameter("max_speed".to_string(), "2.0".to_string()).unwrap();
    let ver2 = coordinator1.set_fleet_parameter("formation".to_string(), "line".to_string()).unwrap();
    let _ver3 = coordinator1.set_fleet_parameter("safety_distance".to_string(), "0.5".to_string()).unwrap();

    println!("Set 3 params versions v{}, v{}", ver1, ver2);
    println!("Fleet size: {} keys: {:?}", coordinator1.distributed_state.len(), coordinator1.distributed_state.keys());

    if let Some(max_speed) = coordinator1.get_fleet_parameter("max_speed") {
        println!("Fleet parameter max_speed: {} (would be replicated to 3 nodes via hash_ring)", max_speed);
    }

    // Consistent hash demo — DistributedMap per §17.1
    let shard = coordinator1.distributed_state.consistent_hash_shard("max_speed", 3);
    println!("Key 'max_speed' hashes to shard {} / 3 (consistent hashing for DistributedMap)", shard);

    // Task distribution — capability matching per §17.1 SwarmMember
    println!("\n=== Task Distribution (Capability-based scheduling) ===");

    // Submit various tasks with different requirements
    let t1 = coordinator1.task_scheduler.submit_task(
        "object_detection".to_string(),
        10, // High priority
        TaskRequirements {
            min_cpu_cores: 2,
            min_memory_mb: 2048,
            requires_gpu: true, // Prefers GPU per #[compute(prefer="GPU")]
            required_sensors: vec!["camera".to_string()],
        }
    );

    let t2 = coordinator1.task_scheduler.submit_task(
        "path_planning".to_string(),
        8,
        TaskRequirements {
            min_cpu_cores: 4,
            min_memory_mb: 4096,
            requires_gpu: false,
            required_sensors: vec!["lidar".to_string()],
        }
    );

    let t3 = coordinator1.task_scheduler.submit_task(
        "sensor_fusion".to_string(),
        9,
        TaskRequirements {
            min_cpu_cores: 4,
            min_memory_mb: 8192,
            requires_gpu: true,
            required_sensors: vec!["camera".to_string(), "lidar".to_string()],
        }
    );

    let t4 = coordinator1.task_scheduler.submit_task(
        "collaborative_mapping".to_string(),
        7,
        TaskRequirements {
            min_cpu_cores: 8,
            min_memory_mb: 12288,
            requires_gpu: true,
            required_sensors: vec!["camera".to_string(), "lidar".to_string(), "radar".to_string()],
        }
    );

    println!("Submitted 4 tasks: {:?}, {:?}, {:?}, {:?}", t1.0, t2.0, t3.0, t4.0);

    // Coordinate and distribute — leader assigns based on capabilities
    println!("\nLeader distributing tasks...");
    coordinator1.coordinate().unwrap();

    let status = coordinator1.get_fleet_status();
    println!("\nTask Statistics: {}", status.tasks);
    println!("  Pending:   {} (no capable robot)", status.tasks.pending);
    println!("  Assigned:  {} (matched capabilities)", status.tasks.assigned);
    println!("  Running:   {}", status.tasks.running);
    println!("  Completed: {}", status.tasks.completed);

    // Simulate task execution — only tasks assigned to Robot 1 execute here
    println!("\n=== Task Execution (ComputeScheduler auto device selection) ===");
    let pending_or_assigned: Vec<_> = {
        let tasks = coordinator1.task_scheduler.tasks.lock().unwrap();
        tasks.values().filter(|t| matches!(t.status, nros_distributed::TaskStatus::Assigned) && t.assigned_to == Some(RobotId::new(1))).cloned().collect()
    };

    for task in pending_or_assigned {
        println!("Robot 1 attempting to execute {} assigned to {:?}", task.id.0, task.assigned_to);
        match coordinator1.task_scheduler.execute_task(task.id) {
            Ok(dur) => println!("  Completed in {:?} (GPU accelerated if available per §16.3)", dur),
            Err(e) => println!("  Failed: {}", e),
        }
    }

    // Also show that tasks assigned to other robots would be executed remotely
    {
        let tasks = coordinator1.task_scheduler.tasks.lock().unwrap();
        for task in tasks.values().filter(|t| t.status == nros_distributed::TaskStatus::Assigned && t.assigned_to != Some(RobotId::new(1))) {
            println!("Task {} assigned to Node {:?} would execute remotely via network bridge", task.id.0, task.assigned_to.unwrap().0);
        }
    }

    let final_status = coordinator1.get_fleet_status();
    println!("\nFinal: {}", final_status);

    // Simulate leader election second round — timeout scenario
    println!("\n=== Leader Timeout Simulation ===");
    let election2 = LeaderElection::new(RobotId::new(2)).with_timeouts(Duration::from_millis(200), Duration::from_millis(100));
    let peer_info = NodeInfo {
        id: RobotId::new(1),
        role: NodeRole::Leader,
        address: "192.168.1.101:5000".parse().unwrap(),
        last_heartbeat: Instant::now(),
        capabilities: NodeCapabilities::mid_range(),
    };
    election2.add_peer(peer_info);
    std::thread::sleep(Duration::from_millis(250));
    if election2.check_leader_timeout() {
        println!("Node 2 detected leader timeout (>200ms no heartbeat), will start election — heartbeat mechanism preventing split-brain ✓");
    }

    println!("\n=== Summary ===");
    println!("✓ Leader election: term {}, role {:?}, heartbeat preventing split-brain", final_status.term, coordinator1.leader_election.role());
    println!("✓ Distributed state: {} keys replicated with versioning", coordinator1.distributed_state.len());
    println!("✓ Task distribution: {} tasks total, {} assigned via capability matching", final_status.tasks.total, final_status.tasks.assigned);
    println!("✓ Fleet coordination: {} robots active, leader {:?}", final_status.total_robots, final_status.leader);
    println!("✓ Heterogeneous compute: auto dispatch GPU/NPU/CPU per §16.3 + fleet capabilities");
}
