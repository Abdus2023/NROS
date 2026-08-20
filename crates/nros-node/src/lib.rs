//! NROS Node Implementation Example
//! Demonstrates: Real-time control, parameter management, lifecycle, services, safety, deadline monitoring
//! Implements DESIGN.md §3 Programming Model, §4 Real-Time Guarantees, §25 Artifact #2

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};
use std::f64::consts::PI;
use std::collections::HashMap;

// ============================================================================
// Core Types & Traits — Canonical types from nros-types per AUDIT Pass 12 INTEGRATION-001
// Fixes duplication: nros-core::Twist vs nros-node::Twist — now single source of truth nros-types
// ============================================================================

pub use nros_types::{
    WallTimestamp, Vector3, Twist, MotorCommand, Odometry,
};

// Backward compatibility aliases — old code used Timestamp, Vector3, etc in nros-node
pub type Timestamp = WallTimestamp;

// NOTE (Pass 24 remediation): An `impl Timestamp { ... }` block was removed here. It was a hard
// compile error (E0116): you cannot define an inherent impl for a type alias whose underlying type
// (`WallTimestamp`) is declared in another crate (`nros-types`). The canonical
// `WallTimestamp::to_duration()` is available directly. For latency/deadline measurement use
// `MonotonicInstant`, not wall-clock arithmetic.

// Note: Twist, MotorCommand, Odometry now re-exported from nros-types, no duplication

// ============================================================================
// Node Lifecycle States — Matches DESIGN.md §3.1 Lifecycle
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Unconfigured,
    Inactive,
    Active,
    Finalized,
}

impl std::fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unconfigured => write!(f, "Unconfigured"),
            Self::Inactive => write!(f, "Inactive"),
            Self::Active => write!(f, "Active"),
            Self::Finalized => write!(f, "Finalized"),
        }
    }
}

pub trait LifecycleNode {
    fn on_configure(&mut self) -> Result<(), String>;
    fn on_activate(&mut self) -> Result<(), String>;
    fn on_deactivate(&mut self) -> Result<(), String>;
    fn on_cleanup(&mut self) -> Result<(), String>;
    fn on_shutdown(&mut self) -> Result<(), String>;
    fn state(&self) -> LifecycleState;
}

// ============================================================================
// Parameter System — Runtime validation per DESIGN.md §5, §17.3
// ============================================================================

#[derive(Debug, Clone)]
pub enum ParameterValue {
    Float(f64),
    Int(i64),
    String(String),
    Bool(bool),
}

impl std::fmt::Display for ParameterValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float(v) => write!(f, "{}", v),
            Self::Int(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
            Self::Bool(v) => write!(f, "{}", v),
        }
    }
}

pub struct Parameter {
    pub name: String,
    pub value: ParameterValue,
    pub min: Option<ParameterValue>,
    pub max: Option<ParameterValue>,
    pub read_only: bool,
    pub description: String,
}

impl Parameter {
    pub fn new_float(name: &str, default: f64, min: f64, max: f64, desc: &str) -> Self {
        Self {
            name: name.to_string(),
            value: ParameterValue::Float(default),
            min: Some(ParameterValue::Float(min)),
            max: Some(ParameterValue::Float(max)),
            read_only: false,
            description: desc.to_string(),
        }
    }

    pub fn new_int(name: &str, default: i64, min: i64, max: i64, desc: &str) -> Self {
        Self {
            name: name.to_string(),
            value: ParameterValue::Int(default),
            min: Some(ParameterValue::Int(min)),
            max: Some(ParameterValue::Int(max)),
            read_only: false,
            description: desc.to_string(),
        }
    }

    pub fn validate(&self, new_value: &ParameterValue) -> Result<(), String> {
        // Type check
        match (&self.value, new_value) {
            (ParameterValue::Float(_), ParameterValue::Float(_))
            | (ParameterValue::Int(_), ParameterValue::Int(_))
            | (ParameterValue::String(_), ParameterValue::String(_))
            | (ParameterValue::Bool(_), ParameterValue::Bool(_)) => {}
            _ => return Err(format!("Type mismatch for parameter '{}': expected {:?}, got {:?}", self.name, self.value, new_value)),
        }

        // Range check for float
        if let (Some(min), Some(max)) = (&self.min, &self.max) {
            match (min, new_value, max) {
                (ParameterValue::Float(min_v), ParameterValue::Float(v), ParameterValue::Float(max_v)) => {
                    if v < min_v || v > max_v {
                        return Err(format!("Value {} out of range [{}, {}] for '{}'", v, min_v, max_v, self.name));
                    }
                }
                (ParameterValue::Int(min_v), ParameterValue::Int(v), ParameterValue::Int(max_v)) => {
                    if v < min_v || v > max_v {
                        return Err(format!("Value {} out of range [{}, {}] for '{}'", v, min_v, max_v, self.name));
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

pub struct ParameterServer {
    parameters: HashMap<String, Parameter>,
}

impl ParameterServer {
    pub fn new() -> Self {
        ParameterServer {
            parameters: HashMap::new(),
        }
    }

    pub fn declare(&mut self, param: Parameter) {
        self.parameters.insert(param.name.clone(), param);
    }

    pub fn get(&self, name: &str) -> Option<&Parameter> {
        self.parameters.get(name)
    }

    pub fn get_float(&self, name: &str) -> Option<f64> {
        match self.parameters.get(name)?.value {
            ParameterValue::Float(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_int(&self, name: &str) -> Option<i64> {
        match self.parameters.get(name)?.value {
            ParameterValue::Int(v) => Some(v),
            _ => None,
        }
    }

    pub fn set(&mut self, name: &str, value: ParameterValue) -> Result<ParameterValue, String> {
        let param = self.parameters.get_mut(name)
            .ok_or_else(|| format!("Parameter {} not found", name))?;

        if param.read_only {
            return Err(format!("Parameter '{}' is read-only", name));
        }

        param.validate(&value)?;
        let old = std::mem::replace(&mut param.value, value);
        Ok(old)
    }

    pub fn list(&self) -> Vec<&String> {
        self.parameters.keys().collect()
    }
}

impl Default for ParameterServer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Real-Time Execution Context — DESIGN.md §4, §15
// ============================================================================

pub struct ExecutionStats {
    pub callback_count: AtomicU64,
    pub total_execution_time_ns: AtomicU64,
    pub max_execution_time_ns: AtomicU64,
    pub min_execution_time_ns: AtomicU64,
    pub deadline_misses: AtomicU64,
}

impl ExecutionStats {
    pub fn new() -> Self {
        ExecutionStats {
            callback_count: AtomicU64::new(0),
            total_execution_time_ns: AtomicU64::new(0),
            max_execution_time_ns: AtomicU64::new(0),
            min_execution_time_ns: AtomicU64::new(u64::MAX),
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
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }

        // Update min
        let mut current_min = self.min_execution_time_ns.load(Ordering::Relaxed);
        while duration_ns < current_min {
            match self.min_execution_time_ns.compare_exchange_weak(
                current_min,
                duration_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
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

    pub fn max_execution_time_us(&self) -> f64 {
        self.max_execution_time_ns.load(Ordering::Relaxed) as f64 / 1000.0
    }

    pub fn min_execution_time_us(&self) -> f64 {
        let v = self.min_execution_time_ns.load(Ordering::Relaxed);
        if v == u64::MAX {
            0.0
        } else {
            v as f64 / 1000.0
        }
    }

    pub fn miss_rate(&self) -> f64 {
        let count = self.callback_count.load(Ordering::Relaxed);
        let misses = self.deadline_misses.load(Ordering::Relaxed);
        if count == 0 {
            0.0
        } else {
            (misses as f64 / count as f64) * 100.0
        }
    }
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Velocity Controller Node Implementation — Target <1ms execution
// ============================================================================

pub struct VelocityController {
    // Node metadata
    name: String,
    state: LifecycleState,

    // Parameters
    params: ParameterServer,

    // Robot configuration (cached from params for realtime path)
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

    // Optional: odometry integration state (for real robot would be more complex)
    odom_x: f64,
    odom_y: f64,
    odom_theta: f64,
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
            odom_x: 0.0,
            odom_y: 0.0,
            odom_theta: 0.0,
        };

        // Declare parameters with validation — matches DESIGN.md §17.3 dynamic reconfigure
        node.params.declare(Parameter {
            name: "max_speed".to_string(),
            value: ParameterValue::Float(2.0),
            min: Some(ParameterValue::Float(0.1)),
            max: Some(ParameterValue::Float(5.0)),
            read_only: false,
            description: "Maximum linear speed m/s".to_string(),
        });

        node.params.declare(Parameter {
            name: "wheel_base".to_string(),
            value: ParameterValue::Float(0.5),
            min: Some(ParameterValue::Float(0.1)),
            max: Some(ParameterValue::Float(2.0)),
            read_only: false,
            description: "Distance between wheels".to_string(),
        });

        node.params.declare(Parameter {
            name: "wheel_radius".to_string(),
            value: ParameterValue::Float(0.1),
            min: Some(ParameterValue::Float(0.01)),
            max: Some(ParameterValue::Float(0.5)),
            read_only: false,
            description: "Wheel radius meters".to_string(),
        });

        node.params.declare(Parameter {
            name: "cmd_timeout_ms".to_string(),
            value: ParameterValue::Int(500),
            min: Some(ParameterValue::Int(100)),
            max: Some(ParameterValue::Int(5000)),
            read_only: false,
            description: "Command timeout before emergency stop".to_string(),
        });

        node.params.declare(Parameter {
            name: "max_angular_speed".to_string(),
            value: ParameterValue::Float(PI),
            min: Some(ParameterValue::Float(0.1)),
            max: Some(ParameterValue::Float(2.0 * PI)),
            read_only: false,
            description: "Maximum angular speed rad/s".to_string(),
        });

        node.params.declare(Parameter {
            name: "safety_limits_enabled".to_string(),
            value: ParameterValue::Bool(true),
            min: None,
            max: None,
            read_only: false,
            description: "Enable safety clamping".to_string(),
        });

        node
    }

    /// Real-time callback for velocity commands — Target: <1ms execution, deadline 1000us
    /// This would be #[callback(realtime=true, deadline_us=1000, priority=200)] in real NROS macro
    pub fn on_cmd_vel(&mut self, msg: &Twist) -> Result<MotorCommand, String> {
        let start = Instant::now();

        // Check emergency stop — atomic flag propagation (lockless)
        if self.emergency_stop.load(Ordering::Acquire) {
            let elapsed = start.elapsed().as_nanos() as u64;
            self.stats.record_execution(elapsed, 1_000_000);
            return Ok(self.create_stop_command());
        }

        // Update last command time
        self.last_cmd_time = Some(Instant::now());

        // Extract velocities
        let linear_vel = msg.linear.x;
        let angular_vel = msg.angular.z;

        // Apply safety limits with clamping
        let (safe_linear, safe_angular) = if self.safety_limits_enabled {
            (
                linear_vel.clamp(-self.max_speed, self.max_speed),
                angular_vel.clamp(-self.max_angular_speed, self.max_angular_speed),
            )
        } else {
            (linear_vel, angular_vel)
        };

        // Differential drive kinematics — no heap allocation, stack only (realtime safe)
        // v_left = v_linear - (v_angular * wheel_base) / 2
        // v_right = v_linear + (v_angular * wheel_base) / 2
        let left_velocity = safe_linear - (safe_angular * self.wheel_base / 2.0);
        let right_velocity = safe_linear + (safe_angular * self.wheel_base / 2.0);

        // Convert to wheel angular velocities (rad/s)
        let left_wheel_vel = left_velocity / self.wheel_radius;
        let right_wheel_vel = right_velocity / self.wheel_radius;

        // Simple torque control (proportional to velocity error) — would be PID in real
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

        // Record execution time — <1us overhead
        let elapsed = start.elapsed().as_nanos() as u64;
        self.stats.record_execution(elapsed, 1_000_000); // 1ms deadline

        Ok(motor_cmd)
    }

    /// Periodic safety check callback (runs at 10 Hz) — #[callback(frequency=10)]
    pub fn safety_check(&mut self) -> Result<(), String> {
        // Check for command timeout — triggers emergency stop if no cmd within timeout
        if let Some(last_cmd) = self.last_cmd_time {
            if last_cmd.elapsed() > self.cmd_timeout {
                println!("[WARN][{}] Command timeout ({} ms) - stopping robot", self.name, self.cmd_timeout.as_millis());
                self.emergency_stop.store(true, Ordering::Release);
            }
        }
        Ok(())
    }

    /// Emergency stop service — #[service(name="/emergency_stop")]
    pub fn emergency_stop_service(&mut self, enable: bool) -> Result<String, String> {
        self.emergency_stop.store(enable, Ordering::Release);

        if enable {
            Ok("Emergency stop ENGAGED".to_string())
        } else {
            // Reset command timeout to allow recovery
            self.last_cmd_time = Some(Instant::now());
            Ok("Emergency stop RELEASED".to_string())
        }
    }

    /// Compute odometry from motor commands with integration — simplified model
    pub fn compute_odometry(&mut self, motor_cmd: &MotorCommand, dt: f64) -> Odometry {
        // Convert wheel velocities back to robot velocities
        let left_vel = motor_cmd.left_velocity * self.wheel_radius;
        let right_vel = motor_cmd.right_velocity * self.wheel_radius;

        let linear_vel = (left_vel + right_vel) / 2.0;
        let angular_vel = (right_vel - left_vel) / self.wheel_base;

        // Integrate odometry (simple Euler)
        self.odom_theta += angular_vel * dt;
        // Normalize theta to [-pi, pi]
        self.odom_theta = self.odom_theta.sin().atan2(self.odom_theta.cos());
        self.odom_x += linear_vel * dt * self.odom_theta.cos();
        self.odom_y += linear_vel * dt * self.odom_theta.sin();

        Odometry {
            timestamp: Timestamp::now(),
            position: Vector3::new(self.odom_x, self.odom_y, 0.0),
            orientation: Vector3::new(0.0, 0.0, self.odom_theta),
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

    pub fn stats(&self) -> Arc<ExecutionStats> {
        self.stats.clone()
    }

    pub fn emergency_stop_flag(&self) -> Arc<AtomicBool> {
        self.emergency_stop.clone()
    }

    pub fn parameters(&self) -> &ParameterServer {
        &self.params
    }

    pub fn parameters_mut(&mut self) -> &mut ParameterServer {
        &mut self.params
    }

    pub fn print_stats(&self) {
        let count = self.stats.callback_count.load(Ordering::Relaxed);
        let avg = self.stats.avg_execution_time_us();
        let max = self.stats.max_execution_time_us();
        let min = self.stats.min_execution_time_us();
        let misses = self.stats.deadline_misses.load(Ordering::Relaxed);

        println!("\n=== Node '{}' Statistics ===", self.name);
        println!("State:               {}", self.state);
        println!("Callbacks executed:  {}", count);
        println!("Min execution time:  {:.2} μs", min);
        println!("Avg execution time:  {:.2} μs", avg);
        println!("Max execution time:  {:.2} μs", max);
        println!("Deadline misses:     {} ({:.2}%)", misses, self.stats.miss_rate());
        println!("Emergency stop:      {}", self.emergency_stop.load(Ordering::Relaxed));
        println!("Wheel base:          {} m", self.wheel_base);
        println!("Max speed:           {} m/s", self.max_speed);
    }

    /// Hot-reload parameters from server into cached realtime fields
    pub fn reload_parameters(&mut self) {
        if let Some(v) = self.params.get_float("max_speed") {
            self.max_speed = v;
        }
        if let Some(v) = self.params.get_float("wheel_base") {
            self.wheel_base = v;
        }
        if let Some(v) = self.params.get_float("wheel_radius") {
            self.wheel_radius = v;
        }
        if let Some(v) = self.params.get_float("max_angular_speed") {
            self.max_angular_speed = v;
        }
        if let Some(v) = self.params.get_int("cmd_timeout_ms") {
            self.cmd_timeout = Duration::from_millis(v as u64);
        }
        if let Some(param) = self.params.get("safety_limits_enabled") {
            if let ParameterValue::Bool(b) = param.value {
                self.safety_limits_enabled = b;
            }
        }
    }
}

impl LifecycleNode for VelocityController {
    fn on_configure(&mut self) -> Result<(), String> {
        println!("[{}] Configuring (state: {})...", self.name, self.state);
        self.reload_parameters();
        self.state = LifecycleState::Inactive;
        println!("[{}] Configuration complete - max_speed: {} m/s, wheel_base: {} m", self.name, self.max_speed, self.wheel_base);
        Ok(())
    }

    fn on_activate(&mut self) -> Result<(), String> {
        println!("[{}] Activating (state: {})...", self.name, self.state);
        self.state = LifecycleState::Active;
        self.last_cmd_time = Some(Instant::now());
        self.emergency_stop.store(false, Ordering::Release);
        println!("[{}] Active and ready", self.name);
        Ok(())
    }

    fn on_deactivate(&mut self) -> Result<(), String> {
        println!("[{}] Deactivating (state: {})...", self.name, self.state);
        self.emergency_stop.store(true, Ordering::Release);
        self.state = LifecycleState::Inactive;
        println!("[{}] Deactivated - emergency stop engaged", self.name);
        Ok(())
    }

    fn on_cleanup(&mut self) -> Result<(), String> {
        println!("[{}] Cleaning up (state: {})...", self.name, self.state);
        self.state = LifecycleState::Unconfigured;
        self.odom_x = 0.0;
        self.odom_y = 0.0;
        self.odom_theta = 0.0;
        Ok(())
    }

    fn on_shutdown(&mut self) -> Result<(), String> {
        println!("[{}] Shutting down (state: {})...", self.name, self.state);
        self.print_stats();
        self.state = LifecycleState::Finalized;
        println!("[{}] Finalized", self.name);
        Ok(())
    }

    fn state(&self) -> LifecycleState {
        self.state
    }
}

// ============================================================================
// Unit Tests — Validates realtime guarantees
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle() {
        let mut node = VelocityController::new("test");
        assert_eq!(node.state(), LifecycleState::Unconfigured);
        node.on_configure().unwrap();
        assert_eq!(node.state(), LifecycleState::Inactive);
        node.on_activate().unwrap();
        assert_eq!(node.state(), LifecycleState::Active);
        node.on_deactivate().unwrap();
        assert_eq!(node.state(), LifecycleState::Inactive);
        node.on_cleanup().unwrap();
        assert_eq!(node.state(), LifecycleState::Unconfigured);
        node.on_configure().unwrap();
        node.on_activate().unwrap();
        node.on_shutdown().unwrap();
        assert_eq!(node.state(), LifecycleState::Finalized);
    }

    #[test]
    fn test_velocity_kinematics() {
        let mut node = VelocityController::new("test");
        node.on_configure().unwrap();
        node.on_activate().unwrap();

        // Pure forward
        let cmd = Twist {
            timestamp: Timestamp::now(),
            linear: Vector3::new(1.0, 0.0, 0.0),
            angular: Vector3::new(0.0, 0.0, 0.0),
        };
        let motor = node.on_cmd_vel(&cmd).unwrap();
        // For forward, left and right should be equal
        assert!((motor.left_velocity - motor.right_velocity).abs() < 1e-6);
        assert!((motor.left_velocity - 10.0).abs() < 1e-6); // 1.0 m/s / 0.1 m radius

        // Pure rotation
        let cmd = Twist {
            timestamp: Timestamp::now(),
            linear: Vector3::new(0.0, 0.0, 0.0),
            angular: Vector3::new(0.0, 0.0, 1.0),
        };
        let motor = node.on_cmd_vel(&cmd).unwrap();
        // Left should be negative, right positive, magnitude equal
        assert!((motor.left_velocity + motor.right_velocity).abs() < 1e-6);
    }

    #[test]
    fn test_parameter_validation() {
        let mut node = VelocityController::new("test");
        // Valid
        assert!(node.parameters_mut().set("max_speed", ParameterValue::Float(3.0)).is_ok());
        // Out of range
        assert!(node.parameters_mut().set("max_speed", ParameterValue::Float(10.0)).is_err());
        // Type mismatch
        assert!(node.parameters_mut().set("max_speed", ParameterValue::Int(2)).is_err());
        // Not found
        assert!(node.parameters_mut().set("unknown", ParameterValue::Float(1.0)).is_err());
    }

    #[test]
    fn test_emergency_stop() {
        let mut node = VelocityController::new("test");
        node.on_configure().unwrap();
        node.on_activate().unwrap();

        let cmd = Twist {
            timestamp: Timestamp::now(),
            linear: Vector3::new(1.0, 0.0, 0.0),
            angular: Vector3::new(0.0, 0.0, 0.0),
        };

        // Normal
        let motor = node.on_cmd_vel(&cmd).unwrap();
        assert!(motor.left_velocity > 0.0);

        // E-stop
        node.emergency_stop_service(true).unwrap();
        let motor = node.on_cmd_vel(&cmd).unwrap();
        assert_eq!(motor.left_velocity, 0.0);
        assert_eq!(motor.right_velocity, 0.0);

        // Release
        node.emergency_stop_service(false).unwrap();
        let motor = node.on_cmd_vel(&cmd).unwrap();
        assert!(motor.left_velocity > 0.0);
    }

    #[test]
    fn test_performance_timing() {
        let mut node = VelocityController::new("test");
        node.on_configure().unwrap();
        node.on_activate().unwrap();

        let cmd = Twist {
            timestamp: Timestamp::now(),
            linear: Vector3::new(0.5, 0.0, 0.0),
            angular: Vector3::new(0.0, 0.0, 0.2),
        };

        let start = Instant::now();
        for _ in 0..10000 {
            let _ = node.on_cmd_vel(&cmd);
        }
        let elapsed = start.elapsed();

        let avg_us = node.stats().avg_execution_time_us();
        println!("10000 callbacks in {:?}, avg {:.2} μs", elapsed, avg_us);
        // Target <1000 μs deadline, should be << that, typically <5 μs
        assert!(avg_us < 100.0, "avg {} μs too high", avg_us);
        assert_eq!(node.stats().deadline_misses.load(Ordering::Relaxed), 0);
    }
}
