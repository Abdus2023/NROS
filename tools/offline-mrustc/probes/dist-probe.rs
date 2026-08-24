// nros-distributed logic probes (Pass 27) — run as a plain binary because mrustc's
// typechecker crashes on the --test harness for this crate (toolchain-only limitation;
// the same bodies pass under cargo — see README trick #11 and AUDIT_PASS_27 §11.F/F-22).
use nros_distributed::*;
use std::time::Duration;

fn main() {
    // 1. Leader election: single node wins, transitions and term accounting
    {
        let election = LeaderElection::new(RobotId::new(1));
        assert!(!election.is_leader());
        assert!(election.role() == NodeRole::Follower);
        let won = election.start_election();
        assert!(won, "single-node election must win");
        assert!(election.is_leader());
        assert!(election.get_leader() == Some(RobotId::new(1)));
        assert!(election.term() >= 1, "term must increment on election");
        election.become_follower(RobotId::new(2), election.term() + 1);
        assert!(!election.is_leader());
        assert!(election.get_leader() == Some(RobotId::new(2)));
        assert!(!election.check_leader_timeout(), "fresh heartbeat must not time out");
    }
    println!("[1] leader election role/term transitions: PASS");

    // 2. Election timeout actually fires when heartbeat window elapses
    {
        let election = LeaderElection::new(RobotId::new(9))
            .with_timeouts(Duration::from_millis(20), Duration::from_millis(5));
        election.become_follower(RobotId::new(3), 1);
        std::thread::sleep(Duration::from_millis(40));
        assert!(election.check_leader_timeout(), "leader timeout must fire after window");
        election.receive_heartbeat(RobotId::new(3), 2);
        assert!(election.term() >= 2);
        assert!(!election.check_leader_timeout(), "heartbeat must reset timeout");
    }
    println!("[2] heartbeat/timeout window discipline: PASS");

    // 3. DistributedState: CRUD + versioning + consistent-hash determinism/shard-bounds
    {
        let state: DistributedState<i32> = DistributedState::new(RobotId::new(1), 3);
        assert!(state.is_simulated(), "default replication mode must be honestly SIMULATED");
        let v1 = state.set("max_speed".to_string(), 5).unwrap();
        let v2 = state.set("min_speed".to_string(), 1).unwrap();
        assert!(v2 > v1, "version must be monotonic");
        assert_eq!(state.get("max_speed"), Some(5));
        assert_eq!(state.get("nonexistent"), None);
        assert_eq!(state.len(), 2);
        state.delete("min_speed").unwrap();
        assert_eq!(state.len(), 1);
        let s1 = state.consistent_hash_shard("max_speed", 10);
        let s2 = state.consistent_hash_shard("max_speed", 10);
        assert_eq!(s1, s2, "consistent hash must be deterministic");
        assert!(s1 < 10, "shard must be within bounds");
        assert_eq!(state.consistent_hash_shard("max_speed", 0), 0, "0 shards must not panic (Pass 24 guard)");
        let mut spread = std::collections::HashSet::new();
        for i in 0..1000 { spread.insert(state.consistent_hash_shard(&format!("key_{}", i), 16)); }
        assert!(spread.len() >= 8, "FNV-1a spread over 16 shards must use most buckets (got {})", spread.len());
    }
    println!("[3] DistributedState CRUD/version/hash determinism+spread: PASS");

    // 4. Task scheduling: submit/assign/execute lifecycle + priority ordering + stats
    {
        let scheduler = TaskScheduler::new(RobotId::new(1), NodeCapabilities::high_end());
        let t_low = scheduler.submit_task("sensor_fusion".to_string(), 1, TaskRequirements::default());
        let t_high = scheduler.submit_task("path_planning".to_string(), 9, TaskRequirements::default());
        assert_eq!(scheduler.count(), 2);
        let pending = scheduler.pending_tasks();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].id, t_high, "pending list must be priority-sorted (high first)");
        scheduler.assign_task(t_high, RobotId::new(1)).unwrap();
        assert!(scheduler.assign_task(t_high, RobotId::new(1)).is_err(), "double assign must fail (not pending)");
        let dur = scheduler.execute_task(t_high).unwrap();
        assert_eq!(dur, Duration::from_millis(80), "path_planning simulated duration is fixed at 80ms");
        assert!(scheduler.execute_task(t_low).is_err(), "executing an unassigned task must fail");
        assert!(scheduler.execute_task(t_high).is_err(), "re-executing a completed task must fail (F-21)");
        let stats = scheduler.task_stats();
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.pending, 1);
        let other = TaskScheduler::new(RobotId::new(7), NodeCapabilities::low_end());
        assert!(other.assign_task(t_low, RobotId::new(7)).is_err(), "foreign scheduler must not see the task");
    }
    println!("[4] task lifecycle/priority/stats + negative transitions: PASS");

    // 4b. execute_task rejects tasks assigned to a DIFFERENT node
    {
        let scheduler = TaskScheduler::new(RobotId::new(1), NodeCapabilities::high_end());
        let t = scheduler.submit_task("misc".to_string(), 1, TaskRequirements::default());
        scheduler.assign_task(t, RobotId::new(99)).unwrap(); // assigned elsewhere
        assert!(scheduler.execute_task(t).is_err(), "node must refuse to run a task assigned elsewhere");
    }
    println!("[4b] wrong-node execution refusal: PASS");

    // 5. Capability matching matrix (GPU / sensors / default)
    {
        let high = NodeCapabilities::high_end();
        let low = NodeCapabilities::low_end();
        let gpu_task = TaskRequirements {
            min_cpu_cores: 2,
            min_memory_mb: 2048,
            requires_gpu: true,
            required_sensors: vec!["camera".into()],
        };
        assert!(high.matches(&gpu_task), "high-end node must match GPU+camera demand");
        assert!(!low.matches(&gpu_task), "low-end node must NOT match GPU demand");
        assert!(low.matches(&TaskRequirements::default()), "low-end node must match default task");
        assert!(!low.matches(&TaskRequirements { required_sensors: vec!["radar".into()], ..Default::default() }),
            "low-end node lacks radar — sensor demand must fail");
        assert!(high.matches(&TaskRequirements { required_sensors: vec!["radar".into(), "imu".into()], ..Default::default() }),
            "high-end node has radar+imu — multi-sensor demand must pass");
    }
    println!("[5] capability matching matrix: PASS");

    println!("ALL DISTRIBUTED PROBES PASS");
}
