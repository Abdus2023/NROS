// Complete NROS Node Implementation Example
// Demonstrates: Real-time control, parameter management, lifecycle, and services

use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::{Duration, Instant};
use std::f64::consts::PI;

// ============================================================================
// Core Types & Traits
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct Timestamp {
    pub sec: u64,
    pub nanosec: u32,
}

impl Timestamp {
    pub fn now() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        Timestamp {
            sec: now.as_secs(),
            nanosec: now.subsec_nanos(),
        }
    }
    
    pub fn to_duration(&self) -> Duration {
        Duration::new(self.sec, self.nanosec)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector3 { x, y, z }
    }
    
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Twist {
    pub timestamp: Timestamp,
    pub linear: Vector3,
    pub angular: Vector3,
}

#[derive(Debug, Clone, Copy)]
pub struct MotorCommand {
    pub timestamp: Timestamp,
    pub left_velocity: f64,
    pub right_velocity: f64,
    pub left_torque: f64,
    pub right_torque: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Odometry {
    pub timestamp: Timestamp,
    pub position: Vector3,
    pub orientation: Vector3,
    pub linear_velocity: Vector3,
    pub angular_velocity: Vector3,
}

// ============================================================================
// Node Lifecycle States
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Unconfigured,
    Inactive,
    Active,
    Finalized,
}

pub trait LifecycleNode {
    fn on_configure(&mut self) -> Result<(), String>;
    fn on_activate(&mut self) -> Result<(), String>;
    fn on_deactivate(&mut self) -> Result<(), String>;
    fn on_cleanup(&mut self) -> Result<(), String>;
    fn on_shutdown(&mut self) -> Result<(), String>;
}

// ============================================================================
// Parameter System
// ============================================================================

#[derive(Debug, Clone)]
pub enum ParameterValue {
    Float(f64),
    Int(i64),
    String(String),
    Bool(bool),
}

pub struct Parameter {
    pub name: String,
    pub value: ParameterValue,
    pub min: Option<ParameterValue>,
    pub max: Option<ParameterValue>,
    pub read_only: bool,
}

impl Parameter {
    pub fn validate(&self, new_value: &ParameterValue) -> Result<(), String> {
        // Type check
        match (&self.value, new_value) {
            (ParameterValue::Float(_), ParameterValue::Float(_)) |
            (ParameterValue::Int(_), ParameterValue::Int(_)) |
            (ParameterValue::String(_), ParameterValue::String(_)) |
            (ParameterValue::Bool(_), ParameterValue::Bool(_)) => {},
            _ => return Err("Type mismatch".to_string()),
        }
        
        // Range check
        if let (Some(min), Some(max)) = (&self.min, &self.max) {
            match (min, new_value, max) {
                (ParameterValue::Float(min_v), ParameterValue::Float(v), ParameterValue::Float(max_v)) => {
                    if v < min_v || v > max_v {
                        return Err(format!("Value {} out of range [{}, {}]", v, min_v, max_v));
                    }
                },
                _ => {}
            }
        }
        
        Ok(())
    }
}

pub struct ParameterServer {
    parameters: std::collections::HashMap<String, Parameter>,
}

impl ParameterServer {
    pub fn new() -> Self {
        ParameterServer {
            parameters: std::collections::HashMap::new(),
        }
    }
    
    pub fn declare(&mut self, param: Parameter) {
        self.parameters.insert(param.name.clone(), param);
    }
    
    pub fn get(&self, name: &str) -> Option<&Parameter> {
        self.parameters.get(name)
    }
    
    pub fn set(&mut self, name: &str, value: ParameterValue) -> Result<(), String> {
        let param = self.parameters.get_mut(name)
            .ok_or_else(|| format!("Parameter {} not found", name))?;
        
        if param.read_only {
            return Err("Parameter is read-only".to_string());
        }
        
        param.validate(&value)?;
        param.value = value;
        Ok(())
    }
}

// ============================================================================
// Real-Time Execution Context
// ============================================================================

pub struct ExecutionStats {
    pub callback_count: AtomicU64,
    pub total_execution_time_ns: AtomicU64,
    pub max_execution_time_ns: AtomicU64,
    pub deadline_misses: AtomicU64,
}

impl ExecutionStats {
    pub fn new() -> Self {
        ExecutionStats {
            callback_count: AtomicU64::new(0),
            total_execution_time_ns: AtomicU64::new(0),
            max_execution_time_ns: AtomicU64::new(0),
            deadline_misses: AtomicU64::new(0),
        }
    }
    
    pub fn record_execution(&self, duration_ns: u64, deadline_ns: u64) {
        self.callback_count.fetch_add(1, Ordering::Relaxed);
        self.total_execution_time_ns.fetch_add(duration_ns, Ordering::Relaxed);
        
        // Update max
        let mut current_max = self.max_execution_time_ns.load(Ordering::Relaxed);
        while duration_ns > current_max {
            match self.max_execution_time_ns.compare_exchange_weak(
                current_max,
                duration_ns,
                Ordering::Relaxed,
                Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
        
        if duration_ns > deadline_ns {
            self.deadline_misses.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    pub fn avg_execution_time_us(&self) -> f64 {
        let total = self.total_execution_time_ns.load(Ordering::Relaxed);
        let count = self.callback_count.load(Ordering::Relaxed);
        if count == 0 {
            0.0
        } else {
            (total as f64 / count as f64) / 1000.0
        }
    }
}

// ============================================================================
// Velocity Controller Node Implementation
// ============================================================================

pub struct VelocityController {
    // Node metadata
    name: String,
    state: LifecycleState,
    
    // Parameters
    params: ParameterServer,
    
    // Robot configuration
    wheel_base: f64,        // Distance between wheels (meters)
    wheel_radius: f64,      // Wheel radius (meters)
    max_speed: f64,         // Maximum linear speed (m/s)
    max_angular_speed: f64, // Maximum angular speed (rad/s)
    
    // Control state
    last_cmd_time: Option<Instant>,
    cmd_timeout: Duration,
    emergency_stop: Arc<AtomicBool>,
    
    // Statistics
    stats: Arc<ExecutionStats>,
    
    // Safety features
    safety_limits_enabled: bool,
}

impl VelocityController {
    pub fn new(name: &str) -> Self {
        let mut node = VelocityController {
            name: name.to_string(),
            state: LifecycleState::Unconfigured,
            params: ParameterServer::new(),
            wheel_base: 0.5,
            wheel_radius: 0.1,
            max_speed: 2.0,
            max_angular_speed: PI,
            last_cmd_time: None,
            cmd_timeout: Duration::from_millis(500),
            emergency_stop: Arc::new(AtomicBool::new(false)),
            stats: Arc::new(ExecutionStats::new()),
            safety_limits_enabled: true,
        };
        
        // Declare parameters
        node.params.declare(Parameter {
            name: "max_speed".to_string(),
            value: ParameterValue::Float(2.0),
            min: Some(ParameterValue::Float(0.1)),
            max: Some(ParameterValue::Float(5.0)),
            read_only: false,
        });
        
        node.params.declare(Parameter {
            name: "wheel_base".to_string(),
            value: ParameterValue::Float(0.5),
            min: Some(ParameterValue::Float(0.1)),
            max: Some(ParameterValue::Float(2.0)),
            read_only: false,
        });
        
        node.params.declare(Parameter {
            name: "cmd_timeout_ms".to_string(),
            value: ParameterValue::Int(500),
            min: Some(ParameterValue::Int(100)),
            max: Some(ParameterValue::Int(5000)),
            read_only: false,
        });
        
        node
    }
    
    // Real-time callback for velocity commands
    // Target: < 1ms execution time
    pub fn on_cmd_vel(&mut self, msg: &Twist) -> Result<MotorCommand, String> {
        let start = Instant::now();
        
        // Check emergency stop
        if self.emergency_stop.load(Ordering::Acquire) {
            return Ok(self.create_stop_command());
        }
        
        // Update last command time
        self.last_cmd_time = Some(Instant::now());
        
        // Extract velocities
        let linear_vel = msg.linear.x;
        let angular_vel = msg.angular.z;
        
        // Apply safety limits
        let (safe_linear, safe_angular) = if self.safety_limits_enabled {
            (
                linear_vel.clamp(-self.max_speed, self.max_speed),
                angular_vel.clamp(-self.max_angular_speed, self.max_angular_speed)
            )
        } else {
            (linear_vel, angular_vel)
        };
        
        // Differential drive kinematics
        // v_left = v_linear - (v_angular * wheel_base) / 2
        // v_right = v_linear + (v_angular * wheel_base) / 2
        let left_velocity = safe_linear - (safe_angular * self.wheel_base / 2.0);
        let right_velocity = safe_linear + (safe_angular * self.wheel_base / 2.0);
        
        // Convert to wheel angular velocities (rad/s)
        let left_wheel_vel = left_velocity / self.wheel_radius;
        let right_wheel_vel = right_velocity / self.wheel_radius;
        
        // Simple torque control (proportional to velocity error)
        let torque_gain = 1.0;
        let left_torque = left_wheel_vel * torque_gain;
        let right_torque = right_wheel_vel * torque_gain;
        
        let motor_cmd = MotorCommand {
            timestamp: Timestamp::now(),
            left_velocity: left_wheel_vel,
            right_velocity: right_wheel_vel,
            left_torque,
            right_torque,
        };
        
        // Record execution time
        let elapsed = start.elapsed().as_nanos() as u64;
        self.stats.record_execution(elapsed, 1_000_000); // 1ms deadline
        
        Ok(motor_cmd)
    }
    
    // Periodic safety check callback (runs at 10 Hz)
    pub fn safety_check(&mut self) -> Result<(), String> {
        // Check for command timeout
        if let Some(last_cmd) = self.last_cmd_time {
            if last_cmd.elapsed() > self.cmd_timeout {
                println!("[WARN] Command timeout - stopping robot");
                self.emergency_stop.store(true, Ordering::Release);
            }
        }
        
        Ok(())
    }
    
    // Emergency stop service
    pub fn emergency_stop_service(&mut self, enable: bool) -> Result<String, String> {
        self.emergency_stop.store(enable, Ordering::Release);
        
        if enable {
            Ok("Emergency stop ENGAGED".to_string())
        } else {
            // Reset command timeout
            self.last_cmd_time = Some(Instant::now());
            Ok("Emergency stop RELEASED".to_string())
        }
    }
    
    // Compute odometry from motor commands (simplified)
    pub fn compute_odometry(&self, motor_cmd: &MotorCommand, dt: f64) -> Odometry {
        // Convert wheel velocities back to robot velocities
        let left_vel = motor_cmd.left_velocity * self.wheel_radius;
        let right_vel = motor_cmd.right_velocity * self.wheel_radius;
        
        let linear_vel = (left_vel + right_vel) / 2.0;
        let angular_vel = (right_vel - left_vel) / self.wheel_base;
        
        Odometry {
            timestamp: Timestamp::now(),
            position: Vector3::new(0.0, 0.0, 0.0), // Would integrate velocities
            orientation: Vector3::new(0.0, 0.0, 0.0),
            linear_velocity: Vector3::new(linear_vel, 0.0, 0.0),
            angular_velocity: Vector3::new(0.0, 0.0, angular_vel),
        }
    }
    
    fn create_stop_command(&self) -> MotorCommand {
        MotorCommand {
            timestamp: Timestamp::now(),
            left_velocity: 0.0,
            right_velocity: 0.0,
            left_torque: 0.0,
            right_torque: 0.0,
        }
    }
    
    pub fn print_stats(&self) {
        let count = self.stats.callback_count.load(Ordering::Relaxed);
        let avg = self.stats.avg_execution_time_us();
        let max = self.stats.max_execution_time_ns.load(Ordering::Relaxed) as f64 / 1000.0;
        let misses = self.stats.deadline_misses.load(Ordering::Relaxed);
        
        println!("\n=== Node Statistics ===");
        println!("Callbacks executed:  {}", count);
        println!("Avg execution time:  {:.2} μs", avg);
        println!("Max execution time:  {:.2} μs", max);
        println!("Deadline misses:     {}", misses);
        println!("Miss rate:           {:.2}%", 
            if count > 0 { (misses as f64 / count as f64) * 100.0 } else { 0.0 });
    }
}

impl LifecycleNode for VelocityController {
    fn on_configure(&mut self) -> Result<(), String> {
        println!("[{}] Configuring...", self.name);
        
        // Load parameters
        if let Some(param) = self.params.get("max_speed") {
            if let ParameterValue::Float(v) = param.value {
                self.max_speed = v;
            }
        }
        
        if let Some(param) = self.params.get("wheel_base") {
            if let ParameterValue::Float(v) = param.value {
                self.wheel_base = v;
            }
        }
        
        self.state = LifecycleState::Inactive;
        println!("[{}] Configuration complete", self.name);
        Ok(())
    }
    
    fn on_activate(&mut self) -> Result<(), String> {
        println!("[{}] Activating...", self.name);
        self.state = LifecycleState::Active;
        self.last_cmd_time = Some(Instant::now());
        println!("[{}] Active and ready", self.name);
        Ok(())
    }
    
    fn on_deactivate(&mut self) -> Result<(), String> {
        println!("[{}] Deactivating...", self.name);
        self.emergency_stop.store(true, Ordering::Release);
        self.state = LifecycleState::Inactive;
        println!("[{}] Deactivated", self.name);
        Ok(())
    }
    
    fn on_cleanup(&mut self) -> Result<(), String> {
        println!("[{}] Cleaning up...", self.name);
        self.state = LifecycleState::Unconfigured;
        Ok(())
    }
    
    fn on_shutdown(&mut self) -> Result<(), String> {
        println!("[{}] Shutting down...", self.name);
        self.print_stats();
        self.state = LifecycleState::Finalized;
        Ok(())
    }
}

// ============================================================================
// Demo / Test
// ============================================================================

fn main() {
    println!("NROS Velocity Controller Node Demo\n");
    
    // Create and configure node
    let mut node = VelocityController::new("velocity_controller");
    
    // Lifecycle transitions
    node.on_configure().unwrap();
    node.on_activate().unwrap();
    
    println!("\nSimulating control loop...\n");
    
    // Simulate receiving velocity commands
    let test_commands = vec![
        Twist {
            timestamp: Timestamp::now(),
            linear: Vector3::new(1.0, 0.0, 0.0),
            angular: Vector3::new(0.0, 0.0, 0.0),
        },
        Twist {
            timestamp: Timestamp::now(),
            linear: Vector3::new(0.5, 0.0, 0.0),
            angular: Vector3::new(0.0, 0.0, 0.5),
        },
        Twist {
            timestamp: Timestamp::now(),
            linear: Vector3::new(0.0, 0.0, 0.0),
            angular: Vector3::new(0.0, 0.0, 1.0),
        },
    ];
    
    for (i, cmd) in test_commands.iter().enumerate() {
        println!("--- Command {} ---", i + 1);
        println!("Input: linear={:.2} m/s, angular={:.2} rad/s", 
            cmd.linear.x, cmd.angular.z);
        
        match node.on_cmd_vel(cmd) {
            Ok(motor_cmd) => {
                println!("Output:");
                println!("  Left motor:  {:.2} rad/s, {:.2} Nm", 
                    motor_cmd.left_velocity, motor_cmd.left_torque);
                println!("  Right motor: {:.2} rad/s, {:.2} Nm", 
                    motor_cmd.right_velocity, motor_cmd.right_torque);
                
                let odom = node.compute_odometry(&motor_cmd, 0.01);
                println!("Odometry: v={:.2} m/s, ω={:.2} rad/s",
                    odom.linear_velocity.x, odom.angular_velocity.z);
            },
            Err(e) => println!("Error: {}", e),
        }
        println!();
        
        std::thread::sleep(Duration::from_millis(50));
    }
    
    // Test emergency stop
    println!("--- Testing Emergency Stop ---");
    node.emergency_stop_service(true).unwrap();
    
    let cmd = Twist {
        timestamp: Timestamp::now(),
        linear: Vector3::new(1.0, 0.0, 0.0),
        angular: Vector3::new(0.0, 0.0, 0.0),
    };
    
    match node.on_cmd_vel(&cmd) {
        Ok(motor_cmd) => {
            println!("Motor command during e-stop:");
            println!("  Left:  {:.2} rad/s", motor_cmd.left_velocity);
            println!("  Right: {:.2} rad/s", motor_cmd.right_velocity);
            assert_eq!(motor_cmd.left_velocity, 0.0);
            assert_eq!(motor_cmd.right_velocity, 0.0);
        },
        Err(e) => println!("Error: {}", e),
    }
    
    // Performance test
    println!("\n--- Performance Test (10000 callbacks) ---");
    node.emergency_stop_service(false).unwrap();
    
    let test_cmd = Twist {
        timestamp: Timestamp::now(),
        linear: Vector3::new(0.5, 0.0, 0.0),
        angular: Vector3::new(0.0, 0.0, 0.2),
    };
    
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = node.on_cmd_vel(&test_cmd);
    }
    let elapsed = start.elapsed();
    
    println!("Total time: {:.2?}", elapsed);
    println!("Throughput: {:.0} callbacks/sec", 
        10000.0 / elapsed.as_secs_f64());
    
    node.print_stats();
    
    // Shutdown
    node.on_deactivate().unwrap();
    node.on_cleanup().unwrap();
    node.on_shutdown().unwrap();
}
