//! NROS canonical types — single source of truth for messages, time, geometry
//! Fixes AUDIT Pass 12 INTEGRATION-001: nros-core::Twist ≠ nros-node::Twist duplication
//! Implements recommendation: dedicated canonical crate nros-types / nros-msg + nros-time
//! Status: IMPLEMENTED — canonical types for Twist, Odometry, Vector3, etc.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ── Time abstractions — explicit domains per AUDIT Pass 12 TIME-002 ────────

/// Wall-clock time — Unix epoch, for external timestamps, wire protocol, ROS time
/// Based on SystemTime, may jump due to NTP, not suitable for latency measurement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct WallTimestamp {
    pub sec: u64,
    pub nanosec: u32,
}

impl WallTimestamp {
    pub fn now() -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Self { sec: now.as_secs(), nanosec: now.subsec_nanos() }
    }

    pub fn to_duration(&self) -> Duration {
        Duration::new(self.sec, self.nanosec)
    }
}

impl Default for WallTimestamp {
    fn default() -> Self { Self { sec: 0, nanosec: 0 } }
}

/// Monotonic instant — local latency measurement, deadline monitoring
/// Based on Instant, never jumps, suitable for elapsed time per AUDIT CORE-007
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonotonicInstant {
    instant: Instant,
}

impl MonotonicInstant {
    pub fn now() -> Self { Self { instant: Instant::now() } }
    pub fn elapsed(&self) -> Duration { self.instant.elapsed() }
    pub fn elapsed_ns(&self) -> u64 { self.instant.elapsed().as_nanos() as u64 }
    pub fn duration_since(&self, earlier: Self) -> Duration {
        self.instant.duration_since(earlier.instant)
    }
}

/// Backwards compatibility: Timestamp alias = WallTimestamp (legacy)
/// New code should use WallTimestamp for wire/external, MonotonicInstant for latency
pub type Timestamp = WallTimestamp;

/// Type alias for monotonic timestamp used in latency benchmarks
pub type MonotonicTimestamp = MonotonicInstant;

// ── Geometry ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z } }
    pub fn zero() -> Self { Self::new(0.0, 0.0, 0.0) }
    pub fn magnitude(&self) -> f64 { (self.x*self.x + self.y*self.y + self.z*self.z).sqrt() }
}

impl Default for Vector3 {
    fn default() -> Self { Self::zero() }
}

// ── Messages — canonical definitions ───────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Twist {
    pub timestamp: WallTimestamp,
    pub linear: Vector3,
    pub angular: Vector3,
}

impl Default for Twist {
    fn default() -> Self {
        Self { timestamp: WallTimestamp::default(), linear: Vector3::default(), angular: Vector3::default() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct MotorCommand {
    pub timestamp: WallTimestamp,
    pub left_velocity: f64,
    pub right_velocity: f64,
    pub left_torque: f64,
    pub right_torque: f64,
}

impl Default for MotorCommand {
    fn default() -> Self {
        Self { timestamp: WallTimestamp::default(), left_velocity: 0.0, right_velocity: 0.0, left_torque: 0.0, right_torque: 0.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Odometry {
    pub timestamp: WallTimestamp,
    pub position: Vector3,
    pub orientation: Vector3,
    pub linear_velocity: Vector3,
    pub angular_velocity: Vector3,
}

impl Default for Odometry {
    fn default() -> Self {
        Self {
            timestamp: WallTimestamp::default(),
            position: Vector3::default(),
            orientation: Vector3::default(),
            linear_velocity: Vector3::default(),
            angular_velocity: Vector3::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub intensity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointCloud {
    pub timestamp: WallTimestamp,
    pub points: Vec<Point3D>,
    pub scan_id: u64,
}

// ── Image ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    RGB8,
    RGBA8,
    BGR8,
    MONO8,
    MONO16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub timestamp: WallTimestamp,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub data: Vec<u8>,
    pub frame_id: u64,
}

// ── IMU ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImuData {
    pub timestamp: WallTimestamp,
    pub linear_acceleration: Vector3,
    pub angular_velocity: Vector3,
    pub orientation: Vector3,
}

// ── Execution statistics — canonical ───────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct ExecutionStats {
    pub callback_count: u64,
    pub total_execution_time_ns: u64,
    pub max_execution_time_ns: u64,
    pub min_execution_time_ns: u64,
    pub deadline_misses: u64,
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wall_timestamp_now() {
        let ts = WallTimestamp::now();
        assert!(ts.sec > 0);
    }

    #[test]
    fn test_monotonic_elapsed() {
        let t1 = MonotonicInstant::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = t1.elapsed();
        assert!(elapsed.as_millis() >= 10);
    }

    #[test]
    fn test_vector3_magnitude() {
        let v = Vector3::new(3.0, 4.0, 0.0);
        assert!((v.magnitude() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_twist_default() {
        let t = Twist::default();
        assert_eq!(t.linear.x, 0.0);
    }
}
