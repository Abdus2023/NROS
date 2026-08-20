//! NROS facade crate — aggregates all core crates + macros
//! This crate makes `nros init` generated projects compile with `use nros::prelude::*` and `#[nros::node]` etc
//! Status: SCAFFOLDED-IMPLEMENTED per AUDIT.md — macros are passthrough now, real codegen future
//! Implements P0 fix for NROS-011: generated app must be buildable

// Re-export macros as `nros::node`, `nros::subscribe`, etc
pub use nros_macros::{
    algorithm, algorithm_impl, callback, compute, distributed_node, interrupt, param, plugin, plugin_impl, publish, service,
    shared_state, sim, subscribe, task, telemetry, time_sync,
    node,
};

// Re-export core types for prelude
pub mod prelude {
    //! Prelude for `use nros::prelude::*;` per DESIGN.md §3.1
    //!
    //! Canonical domain types (Twist, Vector3, MotorCommand, Odometry, timestamps, ...)
    //! are re-exported from `nros-types` ONLY — the single source of truth (Pass 24 fix).
    //! Previously this prelude glob-imported the same names from BOTH `nros_core` and
    //! `nros_node`, causing E0252 "the name ... is defined multiple times" and making
    //! every generated `nros init` project fail to compile.
    // Import attribute macros directly from nros_macros (not via crate::) to avoid any
    // ambiguity with the `pub mod node`/`pub mod sim` modules below, which share those names
    // in the type namespace. Macros live in the macro namespace, but sourcing them directly
    // keeps the prelude unambiguous.
    pub use nros_macros::{node, subscribe, publish, param, service, callback, time_sync, compute, interrupt, distributed_node, shared_state, task, sim};

    // Canonical domain types — single source of truth
    pub use nros_types::{
        WallTimestamp, MonotonicInstant, Vector3, Twist, MotorCommand, Odometry,
        Point3D, PointCloud, ImageFormat, Image, ImuData,
        ExecutionStats as NodeExecutionStats,
    };
    /// Backward-compat alias.
    pub type Timestamp = WallTimestamp;
    /// Backward-compat alias.
    pub type MonotonicTimestamp = MonotonicInstant;

    // Core IPC — note: Producer/Consumer are the preferred type-enforced SPSC endpoints;
    // Publisher/Subscriber remain for backward compatibility.
    pub use nros_core::{
        channel, Producer, Consumer, Publisher, Subscriber, RingBuffer,
        WriteGuard, InitializedWriteGuard, ReadGuard, PerformanceStats,
        BackpressurePolicy, ChannelConfig, DeliveryPolicy, ExecutionClass,
    };

    // Node — avoid re-exporting the canonical type names that now come from nros_types.
    pub use nros_node::{
        VelocityController, LifecycleState, LifecycleNode,
        ParameterServer, Parameter, ParameterValue,
    };

    // HAL
    pub use nros_hal::{Sensor, SensorData, SensorConfig, DeviceInfo, SensorCapabilities, CameraDriver, LidarDriver, ImuDriver};

    // Transport
    pub use nros_transport::{Serializable, MessageHeader, UdpTransport, TcpTransport, ServiceDiscovery};

    // Distributed
    pub use nros_distributed::{RobotId, NodeRole, LeaderElection, DistributedState, TaskScheduler};

    // Sim — re-exported under aliases to avoid clashing with canonical Vector3
    pub use nros_sim::{SimulationWorld, Quaternion, Transform};

    // Common
    pub use std::time::Duration;
}

/// Re-export crates for advanced usage
pub mod core {
    pub use nros_core::*;
}
pub mod node {
    pub use nros_node::*;
}
pub mod hal {
    pub use nros_hal::*;
}
pub mod transport {
    pub use nros_transport::*;
}
pub mod distributed {
    pub use nros_distributed::*;
}
pub mod sim {
    pub use nros_sim::*;
}
pub mod studio {
    pub use nros_studio::*;
}
pub mod cli {
    pub use nros_cli::*;
}

// Version info per DESIGN.md nros_version field
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NROS_VERSION: &str = "0.1";

/// Init function for nodes — placeholder for real NROS runtime init
/// Real would initialize scheduler, HAL, logging, etc.
pub fn init() {
    // In real: initialize NROS microkernel, scheduler, HAL discovery, etc.
    println!("[NROS {}] Initialized (facade v{})", NROS_VERSION, VERSION);
}

/// Spin node — placeholder
pub fn spin<T>(_node: T) {
    println!("[NROS] Spinning node (placeholder) — real would start scheduler event loop");
}

/// Time utilities — would use monotonic clock in real
pub mod time {
    pub use std::time::{Duration, Instant};
    pub type Timestamp = super::prelude::Timestamp;
}

// Re-export macros at crate root for `#[nros::node]` style
// Allows both `#[nros::node]` and `#[nros_macros::node]`
pub mod macros {
    pub use nros_macros::*;
}
