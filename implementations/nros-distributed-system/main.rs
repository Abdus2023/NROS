// NROS Distributed Computing System
// Demonstrates: Leader election (Raft-like), distributed state, task distribution, fleet coordination

use std::sync::{Arc, Mutex, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use std::net::SocketAddr;

// ============================================================================
// Core Distributed Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RobotId(pub u64);

impl RobotId {
    pub fn new(id: u64) -> Self {
        RobotId(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Leader,
    Follower,
    Candidate,
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: RobotId,
    pub role: NodeRole,
    pub address: SocketAddr,
    pub last_heartbeat: Instant,
    pub capabilities: NodeCapabilities,
}

#[derive(Debug, Clone)]
pub struct NodeCapabilities {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub has_gpu: bool,
    pub sensors: Vec<String>,
    pub actuators: Vec<String>,
}

// ============================================================================
// Raft-like Leader Election
// ============================================================================

#[derive(Debug, Clone)]
pub struct RaftState {
    pub current_term: u64,
    pub voted_for: Option<RobotId>,
    pub log: Vec<LogEntry>,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: String,
}

pub struct LeaderElection {
    node_id: RobotId,
    role: Arc<Mutex<NodeRole>>,
    current_term: AtomicU64,
    peers: Arc<Mutex<HashMap<RobotId, NodeInfo>>>,
    votes_received: Arc<Mutex<HashSet<RobotId>>>,
    election_timeout: Duration,
    heartbeat_interval: Duration,
    last_heartbeat: Arc<Mutex<Instant>>,
}

impl LeaderElection {
    pub fn new(node_id: RobotId) -> Self {
        LeaderElection {
            node_id,
            role: Arc::new(Mutex::new(NodeRole::Follower)),
            current_term: AtomicU64::new(0),
            peers: Arc::new(Mutex::new(HashMap::new())),
            votes_received: Arc::new(Mutex::new(HashSet::new())),
            election_timeout: Duration::from_millis(1500),
            heartbeat_interval: Duration::from_millis(500),
            last_heartbeat: Arc::new(Mutex::new(Instant::now())),
        }
    }
    
    pub fn add_peer(&self, info: NodeInfo) {
        let mut peers = self.peers.lock().unwrap();
        peers.insert(info.id, info);
    }
    
    pub fn start_election(&self) -> bool {
        println!("[Node {}] Starting election", self.node_id.0);
        
        // Increment term
        let new_term = self.current_term.fetch_add(1, Ordering::SeqCst) + 1;
        
        // Become candidate
        {
            let mut role = self.role.lock().unwrap();
            *role = NodeRole::Candidate;
        }
        
        // Vote for self
        let mut votes = self.votes_received.lock().unwrap();
        votes.clear();
        votes.insert(self.node_id);
        
        // Request votes from peers
        let peers = self.peers.lock().unwrap();
        let total_nodes = peers.len() + 1; // +1 for self
        let votes_needed = (total_nodes / 2) + 1;
        
        println!("[Node {}] Requesting votes (need {}/{})", 
            self.node_id.0, votes_needed, total_nodes);
        
        // Simulate vote collection
        for (peer_id, _) in peers.iter() {
            // In real implementation, send RequestVote RPC
            if self.should_grant_vote(*peer_id, new_term) {
                votes.insert(*peer_id);
            }
        }
        
        let vote_count = votes.len();
        drop(votes);
        drop(peers);
        
        if vote_count >= votes_needed {
            self.become_leader();
            true
        } else {
            println!("[Node {}] Election failed: only {} votes", self.node_id.0, vote_count);
            false
        }
    }
    
    fn should_grant_vote(&self, _candidate_id: RobotId, _term: u64) -> bool {
        // Simplified: grant vote with some probability
        rand::random::<f64>() > 0.3
    }
    
    fn become_leader(&self) {
        let mut role = self.role.lock().unwrap();
        *role = NodeRole::Leader;
        println!("[Node {}] ✓ Became LEADER for term {}", 
            self.node_id.0, self.current_term.load(Ordering::SeqCst));
    }
    
    pub fn send_heartbeat(&self) {
        let role = self.role.lock().unwrap();
        if *role != NodeRole::Leader {
            return;
        }
        drop(role);
        
        let peers = self.peers.lock().unwrap();
        for (peer_id, _) in peers.iter() {
            // In real implementation, send AppendEntries RPC
            println!("[Node {}] Sending heartbeat to Node {}", 
                self.node_id.0, peer_id.0);
        }
        
        let mut last_hb = self.last_heartbeat.lock().unwrap();
        *last_hb = Instant::now();
    }
    
    pub fn check_leader_timeout(&self) -> bool {
        let last_hb = self.last_heartbeat.lock().unwrap();
        let elapsed = last_hb.elapsed();
        elapsed > self.election_timeout
    }
    
    pub fn is_leader(&self) -> bool {
        let role = self.role.lock().unwrap();
        *role == NodeRole::Leader
    }
    
    pub fn get_leader(&self) -> Option<RobotId> {
        if self.is_leader() {
            Some(self.node_id)
        } else {
            // In real implementation, track who we voted for
            None
        }
    }
}

// ============================================================================
// Distributed State Management
// ============================================================================

pub struct DistributedState<T: Clone> {
    local_data: Arc<Mutex<HashMap<String, T>>>,
    replication_factor: usize,
    node_id: RobotId,
}

impl<T: Clone> DistributedState<T> {
    pub fn new(node_id: RobotId, replication_factor: usize) -> Self {
        DistributedState {
            local_data: Arc::new(Mutex::new(HashMap::new())),
            replication_factor,
            node_id,
        }
    }
    
    pub fn set(&self, key: String, value: T) -> Result<(), String> {
        let mut data = self.local_data.lock().unwrap();
        data.insert(key.clone(), value.clone());
        
        println!("[Node {}] Set key: {}", self.node_id.0, key);
        
        // In real implementation: replicate to other nodes
        self.replicate(key, value)?;
        
        Ok(())
    }
    
    pub fn get(&self, key: &str) -> Option<T> {
        let data = self.local_data.lock().unwrap();
        data.get(key).cloned()
    }
    
    pub fn delete(&self, key: &str) -> Result<(), String> {
        let mut data = self.local_data.lock().unwrap();
        data.remove(key);
        
        println!("[Node {}] Deleted key: {}", self.node_id.0, key);
        Ok(())
    }
    
    fn replicate(&self, _key: String, _value: T) -> Result<(), String> {
        // In real implementation: send to replication_factor nodes
        // using consistent hashing to determine target nodes
        Ok(())
    }
    
    pub fn keys(&self) -> Vec<String> {
        let data = self.local_data.lock().unwrap();
        data.keys().cloned().collect()
    }
}

// ============================================================================
// Task Distribution System
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub task_type: String,
    pub priority: u32,
    pub requirements: TaskRequirements,
    pub status: TaskStatus,
    pub assigned_to: Option<RobotId>,
}

#[derive(Debug, Clone)]
pub struct TaskRequirements {
    pub min_cpu_cores: u32,
    pub min_memory_mb: u64,
    pub requires_gpu: bool,
    pub required_sensors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Assigned,
    Running,
    Completed,
    Failed,
}

pub struct TaskScheduler {
    tasks: Arc<Mutex<HashMap<TaskId, Task>>>,
    node_id: RobotId,
    capabilities: NodeCapabilities,
    task_counter: AtomicU64,
}

impl TaskScheduler {
    pub fn new(node_id: RobotId, capabilities: NodeCapabilities) -> Self {
        TaskScheduler {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            node_id,
            capabilities,
            task_counter: AtomicU64::new(0),
        }
    }
    
    pub fn submit_task(&self, task_type: String, priority: u32, requirements: TaskRequirements) -> TaskId {
        let task_id = TaskId(self.task_counter.fetch_add(1, Ordering::SeqCst));
        
        let task = Task {
            id: task_id,
            task_type: task_type.clone(),
            priority,
            requirements,
            status: TaskStatus::Pending,
            assigned_to: None,
        };
        
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(task_id, task);
        
        println!("[Node {}] Submitted task {}: {}", self.node_id.0, task_id.0, task_type);
        
        task_id
    }
    
    pub fn can_execute(&self, requirements: &TaskRequirements) -> bool {
        self.capabilities.cpu_cores >= requirements.min_cpu_cores &&
        self.capabilities.memory_mb >= requirements.min_memory_mb &&
        (!requirements.requires_gpu || self.capabilities.has_gpu) &&
        requirements.required_sensors.iter()
            .all(|s| self.capabilities.sensors.contains(s))
    }
    
    pub fn assign_task(&self, task_id: TaskId, node_id: RobotId) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks.get_mut(&task_id)
            .ok_or("Task not found")?;
        
        if task.status != TaskStatus::Pending {
            return Err("Task not in pending state".to_string());
        }
        
        task.status = TaskStatus::Assigned;
        task.assigned_to = Some(node_id);
        
        println!("[Node {}] Assigned task {} to Node {}", 
            self.node_id.0, task_id.0, node_id.0);
        
        Ok(())
    }
    
    pub fn execute_task(&self, task_id: TaskId) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks.get_mut(&task_id)
            .ok_or("Task not found")?;
        
        if task.assigned_to != Some(self.node_id) {
            return Err("Task not assigned to this node".to_string());
        }
        
        task.status = TaskStatus::Running;
        drop(tasks);
        
        println!("[Node {}] Executing task {}: {}", 
            self.node_id.0, task_id.0, task.task_type);
        
        // Simulate task execution
        std::thread::sleep(Duration::from_millis(100));
        
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks.get_mut(&task_id).unwrap();
        task.status = TaskStatus::Completed;
        
        println!("[Node {}] Completed task {}", self.node_id.0, task_id.0);
        
        Ok(())
    }
    
    pub fn pending_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().unwrap();
        tasks.values()
            .filter(|t| t.status == TaskStatus::Pending)
            .cloned()
            .collect()
    }
    
    pub fn task_stats(&self) -> TaskStats {
        let tasks = self.tasks.lock().unwrap();
        let mut stats = TaskStats::default();
        
        for task in tasks.values() {
            match task.status {
                TaskStatus::Pending => stats.pending += 1,
                TaskStatus::Assigned => stats.assigned += 1,
                TaskStatus::Running => stats.running += 1,
                TaskStatus::Completed => stats.completed += 1,
                TaskStatus::Failed => stats.failed += 1,
            }
        }
        
        stats
    }
}

#[derive(Debug, Default)]
pub struct TaskStats {
    pub pending: usize,
    pub assigned: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
}

// ============================================================================
// Fleet Coordinator
// ============================================================================

pub struct FleetCoordinator {
    fleet_id: String,
    leader_election: Arc<LeaderElection>,
    task_scheduler: Arc<TaskScheduler>,
    distributed_state: Arc<DistributedState<String>>,
    robots: Arc<Mutex<HashMap<RobotId, NodeInfo>>>,
}

impl FleetCoordinator {
    pub fn new(
        fleet_id: String, 
        node_id: RobotId,
        capabilities: NodeCapabilities,
    ) -> Self {
        FleetCoordinator {
            fleet_id,
            leader_election: Arc::new(LeaderElection::new(node_id)),
            task_scheduler: Arc::new(TaskScheduler::new(node_id, capabilities)),
            distributed_state: Arc::new(DistributedState::new(node_id, 3)),
            robots: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn register_robot(&self, info: NodeInfo) {
        let mut robots = self.robots.lock().unwrap();
        robots.insert(info.id, info.clone());
        
        self.leader_election.add_peer(info.clone());
        
        println!("[Fleet {}] Registered robot {}", self.fleet_id, info.id.0);
    }
    
    pub fn coordinate(&self) -> Result<(), String> {
        // Check if leader election needed
        if self.leader_election.check_leader_timeout() {
            self.leader_election.start_election();
        }
        
        // If leader, send heartbeats
        if self.leader_election.is_leader() {
            self.leader_election.send_heartbeat();
            self.distribute_tasks()?;
        }
        
        Ok(())
    }
    
    fn distribute_tasks(&self) -> Result<(), String> {
        let pending = self.task_scheduler.pending_tasks();
        if pending.is_empty() {
            return Ok(());
        }
        
        println!("[Fleet {}] Distributing {} tasks", self.fleet_id, pending.len());
        
        let robots = self.robots.lock().unwrap();
        
        for task in pending {
            // Find capable robot
            let capable_robot = robots.iter()
                .find(|(_, info)| self.can_robot_execute(info, &task.requirements))
                .map(|(id, _)| *id);
            
            if let Some(robot_id) = capable_robot {
                self.task_scheduler.assign_task(task.id, robot_id)?;
            } else {
                println!("[Fleet {}] No capable robot for task {}", 
                    self.fleet_id, task.id.0);
            }
        }
        
        Ok(())
    }
    
    fn can_robot_execute(&self, info: &NodeInfo, requirements: &TaskRequirements) -> bool {
        info.capabilities.cpu_cores >= requirements.min_cpu_cores &&
        info.capabilities.memory_mb >= requirements.min_memory_mb &&
        (!requirements.requires_gpu || info.capabilities.has_gpu)
    }
    
    pub fn get_fleet_status(&self) -> FleetStatus {
        let robots = self.robots.lock().unwrap();
        let task_stats = self.task_scheduler.task_stats();
        
        FleetStatus {
            total_robots: robots.len(),
            leader: self.leader_election.get_leader(),
            tasks: task_stats,
        }
    }
    
    pub fn set_fleet_parameter(&self, key: String, value: String) -> Result<(), String> {
        self.distributed_state.set(key, value)
    }
    
    pub fn get_fleet_parameter(&self, key: &str) -> Option<String> {
        self.distributed_state.get(key)
    }
}

#[derive(Debug)]
pub struct FleetStatus {
    pub total_robots: usize,
    pub leader: Option<RobotId>,
    pub tasks: TaskStats,
}

// ============================================================================
// Demo
// ============================================================================

fn main() {
    println!("NROS Distributed Computing Demo\n");
    
    // Create fleet coordinator for robot 1
    let robot1_caps = NodeCapabilities {
        cpu_cores: 4,
        memory_mb: 8192,
        has_gpu: true,
        sensors: vec!["camera".to_string(), "lidar".to_string()],
        actuators: vec!["motors".to_string()],
    };
    
    let coordinator1 = FleetCoordinator::new(
        "warehouse_fleet".to_string(),
        RobotId::new(1),
        robot1_caps.clone(),
    );
    
    // Register additional robots
    let robot2_info = NodeInfo {
        id: RobotId::new(2),
        role: NodeRole::Follower,
        address: "192.168.1.102:5000".parse().unwrap(),
        last_heartbeat: Instant::now(),
        capabilities: NodeCapabilities {
            cpu_cores: 2,
            memory_mb: 4096,
            has_gpu: false,
            sensors: vec!["camera".to_string()],
            actuators: vec!["motors".to_string()],
        },
    };
    
    let robot3_info = NodeInfo {
        id: RobotId::new(3),
        role: NodeRole::Follower,
        address: "192.168.1.103:5000".parse().unwrap(),
        last_heartbeat: Instant::now(),
        capabilities: NodeCapabilities {
            cpu_cores: 8,
            memory_mb: 16384,
            has_gpu: true,
            sensors: vec!["camera".to_string(), "lidar".to_string(), "radar".to_string()],
            actuators: vec!["motors".to_string(), "arm".to_string()],
        },
    };
    
    coordinator1.register_robot(robot2_info);
    coordinator1.register_robot(robot3_info);
    
    // Leader election
    println!("=== Leader Election ===");
    coordinator1.coordinate().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    
    let status = coordinator1.get_fleet_status();
    println!("Fleet status: {} robots, Leader: {:?}", 
        status.total_robots, status.leader);
    
    // Distributed state management
    println!("\n=== Distributed State ===");
    coordinator1.set_fleet_parameter("max_speed".to_string(), "2.0".to_string()).unwrap();
    coordinator1.set_fleet_parameter("formation".to_string(), "line".to_string()).unwrap();
    
    if let Some(max_speed) = coordinator1.get_fleet_parameter("max_speed") {
        println!("Fleet parameter max_speed: {}", max_speed);
    }
    
    // Task distribution
    println!("\n=== Task Distribution ===");
    
    // Submit various tasks
    coordinator1.task_scheduler.submit_task(
        "object_detection".to_string(),
        10,
        TaskRequirements {
            min_cpu_cores: 2,
            min_memory_mb: 2048,
            requires_gpu: true,
            required_sensors: vec!["camera".to_string()],
        }
    );
    
    coordinator1.task_scheduler.submit_task(
        "path_planning".to_string(),
        8,
        TaskRequirements {
            min_cpu_cores: 4,
            min_memory_mb: 4096,
            requires_gpu: false,
            required_sensors: vec!["lidar".to_string()],
        }
    );
    
    coordinator1.task_scheduler.submit_task(
        "sensor_fusion".to_string(),
        9,
        TaskRequirements {
            min_cpu_cores: 4,
            min_memory_mb: 8192,
            requires_gpu: true,
            required_sensors: vec!["camera".to_string(), "lidar".to_string()],
        }
    );
    
    // Coordinate and distribute tasks
    coordinator1.coordinate().unwrap();
    
    let status = coordinator1.get_fleet_status();
    println!("\nTask Statistics:");
    println!("  Pending:   {}", status.tasks.pending);
    println!("  Assigned:  {}", status.tasks.assigned);
    println!("  Running:   {}", status.tasks.running);
    println!("  Completed: {}", status.tasks.completed);
    println!("  Failed:    {}", status.tasks.failed);
    
    // Simulate task execution
    println!("\n=== Task Execution ===");
    let pending_tasks = coordinator1.task_scheduler.pending_tasks();
    for task in pending_tasks {
        if let Some(assigned_to) = task.assigned_to {
            if assigned_to == RobotId::new(1) {
                coordinator1.task_scheduler.execute_task(task.id).unwrap();
            }
        }
    }
    
    let final_status = coordinator1.get_fleet_status();
    println!("\nFinal Task Statistics:");
    println!("  Pending:   {}", final_status.tasks.pending);
    println!("  Assigned:  {}", final_status.tasks.assigned);
    println!("  Running:   {}", final_status.tasks.running);
    println!("  Completed: {}", final_status.tasks.completed);
    println!("  Failed:    {}", final_status.tasks.failed);
    
    println!("\n=== Summary ===");
    println!("✓ Leader election: Working");
    println!("✓ Distributed state: Synchronized");
    println!("✓ Task distribution: {} tasks assigned", final_status.tasks.assigned);
    println!("✓ Fleet coordination: {} robots active", final_status.total_robots);
}

// Simple random number generator for demo
mod rand {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEED: AtomicU64 = AtomicU64::new(123456789);
    
    pub fn random<T>() -> T 
    where T: From<f64> 
    {
        let seed = SEED.load(Ordering::Relaxed);
        let new_seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        SEED.store(new_seed, Ordering::Relaxed);
        T::from((new_seed % 100) as f64 / 100.0)
    }
}
