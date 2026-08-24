//! NROS facade crate — aggregates all core crates + macros
//! This crate makes `nros init` generated projects compile with `use nros::prelude::*` and `#[nros::node]` etc
//! Status: SCAFFOLDED-IMPLEMENTED per AUDIT.md — macros are passthrough now, real codegen future
//! Implements P0 fix for NROS-011: generated app must be buildable

// Re-export macros as `nros::node`, `nros::subscribe`, etc
pub use nros_macros::{
    algorithm, algorithm_impl, callback, compute, distributed_node, interrupt, node, param, plugin,
    plugin_impl, publish, service, shared_state, sim, subscribe, task, telemetry, time_sync,
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
    pub use nros_macros::{
        callback, compute, distributed_node, interrupt, node, param, publish, service,
        shared_state, sim, subscribe, task, time_sync,
    };

    // Canonical domain types — single source of truth
    pub use nros_types::{
        ExecutionStats as NodeExecutionStats, Image, ImageFormat, ImuData, MonotonicInstant,
        MotorCommand, Odometry, Point3D, PointCloud, Twist, Vector3, WallTimestamp,
    };
    /// Backward-compat alias.
    pub type Timestamp = WallTimestamp;
    /// Backward-compat alias.
    pub type MonotonicTimestamp = MonotonicInstant;

    // Core IPC — Producer/Consumer are the type-enforced SPSC endpoints;
    // Publisher/Subscriber are the topic-labeled facade over them (Pass 31: the
    // deprecated raw-ring constructors were removed; Publisher::declare() is the
    // sole construction path).
    pub use nros_core::{
        channel, BackpressurePolicy, ChannelConfig, Consumer, DeliveryPolicy, ExecutionClass,
        InitializedWriteGuard, PerformanceStats, Producer, Publisher, ReadGuard, RingBuffer,
        Subscriber, WriteGuard,
    };

    // Node — avoid re-exporting the canonical type names that now come from nros_types.
    pub use nros_node::{
        LifecycleNode, LifecycleState, Parameter, ParameterServer, ParameterValue,
        VelocityController,
    };

    // HAL
    pub use nros_hal::{
        CameraDriver, DeviceInfo, ImuDriver, LidarDriver, Sensor, SensorCapabilities, SensorConfig,
        SensorData,
    };

    // Transport
    pub use nros_transport::{
        MessageHeader, Serializable, ServiceDiscovery, TcpTransport, UdpTransport,
    };

    // Distributed
    pub use nros_distributed::{
        DistributedState, LeaderElection, NodeRole, RobotId, TaskScheduler,
    };

    // Sim — re-exported under aliases to avoid clashing with canonical Vector3
    pub use nros_sim::{Quaternion, SimulationWorld, Transform};

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
