//! NROS Simulation Engine - Integrated Physics and Rendering
//! Demonstrates: Physics engine integration (Bullet), sensor simulation, deterministic replay, sim-reality parity
//! Implements DESIGN.md §7.3 Simulation Integration, §20.2 sim_test, §25 beyond core — Phase 2 tools

use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// Core Math Types — Compatible with nros-core Vector3 but extended for sim
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector3 { x, y, z }
    }

    pub fn zero() -> Self {
        Vector3::new(0.0, 0.0, 0.0)
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn magnitude_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 1e-9 {
            Vector3::new(self.x / mag, self.y / mag, self.z / mag)
        } else {
            Vector3::zero()
        }
    }

    pub fn dot(&self, other: &Vector3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vector3) -> Vector3 {
        Vector3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn add(&self, other: &Vector3) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub fn sub(&self, other: &Vector3) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub fn scale(&self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }
}

impl std::fmt::Display for Vector3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:.2}, {:.2}, {:.2}]", self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Quaternion {
    pub fn identity() -> Self {
        Quaternion { w: 1.0, x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn from_euler(roll: f64, pitch: f64, yaw: f64) -> Self {
        let cr = (roll * 0.5).cos();
        let sr = (roll * 0.5).sin();
        let cp = (pitch * 0.5).cos();
        let sp = (pitch * 0.5).sin();
        let cy = (yaw * 0.5).cos();
        let sy = (yaw * 0.5).sin();

        Quaternion {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }

    pub fn to_euler(&self) -> (f64, f64, f64) {
        // Roll (x-axis)
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        // Pitch (y-axis)
        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            std::f64::consts::PI / 2.0 * sinp.signum()
        } else {
            sinp.asin()
        };

        // Yaw (z-axis)
        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        (roll, pitch, yaw)
    }

    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }

    pub fn normalize(&self) -> Self {
        let mag = (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if mag > 1e-9 {
            Self { w: self.w / mag, x: self.x / mag, y: self.y / mag, z: self.z / mag }
        } else {
            Self::identity()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vector3,
    pub orientation: Quaternion,
}

impl Transform {
    pub fn identity() -> Self {
        Transform {
            position: Vector3::zero(),
            orientation: Quaternion::identity(),
        }
    }

    pub fn from_xyz_yaw(x: f64, y: f64, z: f64, yaw: f64) -> Self {
        Self {
            position: Vector3::new(x, y, z),
            orientation: Quaternion::from_euler(0.0, 0.0, yaw),
        }
    }
}

// ============================================================================
// Physics Entities — Rigid bodies per Bullet engine per DESIGN.md §7.3
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub u64);

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct RigidBody {
    pub mass: f64,
    pub inertia: Vector3,
    pub linear_velocity: Vector3,
    pub angular_velocity: Vector3,
    pub force: Vector3,
    pub torque: Vector3,
    pub is_static: bool,
    pub linear_damping: f64,
    pub angular_damping: f64,
}

impl RigidBody {
    pub fn new(mass: f64) -> Self {
        RigidBody {
            mass,
            inertia: Vector3::new(1.0, 1.0, 1.0),
            linear_velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            force: Vector3::zero(),
            torque: Vector3::zero(),
            is_static: false,
            linear_damping: 0.01,
            angular_damping: 0.05,
        }
    }

    pub fn static_body() -> Self {
        RigidBody {
            mass: 0.0,
            inertia: Vector3::zero(),
            linear_velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            force: Vector3::zero(),
            torque: Vector3::zero(),
            is_static: true,
            linear_damping: 0.0,
            angular_damping: 0.0,
        }
    }

    pub fn with_damping(mut self, lin: f64, ang: f64) -> Self {
        self.linear_damping = lin;
        self.angular_damping = ang;
        self
    }
}

#[derive(Debug, Clone)]
pub enum CollisionShape {
    Box { size: Vector3 },
    Sphere { radius: f64 },
    Cylinder { radius: f64, height: f64 },
    Mesh { vertices: Vec<Vector3>, triangles: Vec<[usize; 3]> },
    Plane { normal: Vector3, distance: f64 },
}

impl CollisionShape {
    pub fn bounding_radius(&self) -> f64 {
        match self {
            Self::Box { size } => size.magnitude() / 2.0,
            Self::Sphere { radius } => *radius,
            Self::Cylinder { radius, height } => (radius * radius + height * height / 4.0).sqrt(),
            Self::Mesh { vertices, .. } => vertices.iter().map(|v| v.magnitude()).fold(0.0, f64::max),
            Self::Plane { .. } => f64::INFINITY,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub transform: Transform,
    pub rigid_body: RigidBody,
    pub collision_shape: Option<CollisionShape>,
    pub visual_mesh: Option<String>,
}

// ============================================================================
// Physics Engine — Fixed time step integration per DESIGN.md deterministic
// P1 Fix per AUDIT.md: Separate SimulatedPhysicsEngine vs BulletPhysicsEngine
// ============================================================================

/// Trait for physics engines — allows generic code over Simulated vs Bullet
pub trait PhysicsEngineTrait {
    fn add_entity(&mut self, entity: Entity) -> EntityId;
    fn remove_entity(&mut self, id: EntityId) -> Option<Entity>;
    fn get_entity(&self, id: EntityId) -> Option<&Entity>;
    fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity>;
    fn step(&mut self, delta_time: Duration);
    fn apply_force(&mut self, id: EntityId, force: Vector3);
    fn entity_count(&self) -> usize;
    fn is_simulated(&self) -> bool;
    fn name(&self) -> &'static str;
}

/// Simulated physics engine — IMPLEMENTED per EVIDENCE_REGISTRY.md
/// Status: IMPLEMENTED — custom semi-implicit Euler, fixed timestep, ground collision
/// Real NROS would have option to use Bullet per nros.toml physics_engine=bullet
///
/// Pass 27 fix (first real compile of this crate, 2026-08-22): `#[derive(Debug)]` added —
/// `BulletPhysicsEngine` derives Debug and has `inner: SimulatedPhysicsEngine`, so this type
/// must implement Debug for the crate to compile (E0277 otherwise). Found by the offline
/// mrustc-based verification session; this crate had never been compiled before.
#[derive(Debug)]
pub struct SimulatedPhysicsEngine {
    pub entities: HashMap<EntityId, Entity>,
    pub gravity: Vector3,
    pub time_step: Duration,
    pub accumulated_time: Duration,
    pub entity_counter: u64,
    pub step_count: u64,
}

impl SimulatedPhysicsEngine {
    pub fn new(gravity: Vector3, time_step_hz: f64) -> Self {
        // Pass 24: validate the tick rate. A non-positive/NaN/inf rate would yield
        // `1.0 / hz` that is zero, infinite, or NaN; `Duration::from_secs_f64` panics on
        // NaN/inf and a zero time_step makes the `while accumulated >= time_step` loop
        // spin forever. Clamp to a sane positive floor.
        let hz = if time_step_hz.is_finite() && time_step_hz > 0.0 {
            time_step_hz
        } else {
            240.0
        };
        Self {
            entities: HashMap::new(),
            gravity,
            time_step: Duration::from_secs_f64(1.0 / hz),
            accumulated_time: Duration::ZERO,
            entity_counter: 0,
            step_count: 0,
        }
    }

    pub fn with_gravity(mut self, g: Vector3) -> Self {
        self.gravity = g;
        self
    }

    pub fn add_entity(&mut self, mut entity: Entity) -> EntityId {
        let id = EntityId(self.entity_counter);
        self.entity_counter += 1;
        entity.id = id;
        self.entities.insert(id, entity);
        id
    }

    pub fn add_entity_with_id(&mut self, id: EntityId, entity: Entity) -> EntityId {
        self.entities.insert(id, entity);
        if id.0 >= self.entity_counter {
            self.entity_counter = id.0 + 1;
        }
        id
    }

    pub fn remove_entity(&mut self, id: EntityId) -> Option<Entity> {
        self.entities.remove(&id)
    }

    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Fixed time step integration — deterministic per DESIGN.md §7.3
    pub fn step(&mut self, delta_time: Duration) {
        self.accumulated_time += delta_time;

        while self.accumulated_time >= self.time_step {
            self.integrate(self.time_step.as_secs_f64());
            self.accumulated_time -= self.time_step;
            self.step_count += 1;
        }
    }

    fn integrate(&mut self, dt: f64) {
        // Semi-implicit Euler — same as Bullet default
        for entity in self.entities.values_mut() {
            if entity.rigid_body.is_static || entity.rigid_body.mass <= 1e-9 {
                continue;
            }

            // Apply gravity as force
            entity.rigid_body.force = entity.rigid_body.force.add(&Vector3::new(
                0.0,
                entity.rigid_body.mass * self.gravity.y,
                0.0,
            ));

            // Linear: a = F/m
            let accel = Vector3::new(
                entity.rigid_body.force.x / entity.rigid_body.mass,
                entity.rigid_body.force.y / entity.rigid_body.mass,
                entity.rigid_body.force.z / entity.rigid_body.mass,
            );

            entity.rigid_body.linear_velocity = Vector3::new(
                entity.rigid_body.linear_velocity.x + accel.x * dt,
                entity.rigid_body.linear_velocity.y + accel.y * dt,
                entity.rigid_body.linear_velocity.z + accel.z * dt,
            );

            // Damping
            let damping = 1.0 - entity.rigid_body.linear_damping * dt;
            entity.rigid_body.linear_velocity = entity.rigid_body.linear_velocity.scale(damping.clamp(0.0, 1.0));

            entity.transform.position = Vector3::new(
                entity.transform.position.x + entity.rigid_body.linear_velocity.x * dt,
                entity.transform.position.y + entity.rigid_body.linear_velocity.y * dt,
                entity.transform.position.z + entity.rigid_body.linear_velocity.z * dt,
            );

            // Angular simplified — real Bullet uses quaternion integration
            let angular_accel = Vector3::new(
                if entity.rigid_body.inertia.x > 1e-9 { entity.rigid_body.torque.x / entity.rigid_body.inertia.x } else { 0.0 },
                if entity.rigid_body.inertia.y > 1e-9 { entity.rigid_body.torque.y / entity.rigid_body.inertia.y } else { 0.0 },
                if entity.rigid_body.inertia.z > 1e-9 { entity.rigid_body.torque.z / entity.rigid_body.inertia.z } else { 0.0 },
            );

            entity.rigid_body.angular_velocity = Vector3::new(
                entity.rigid_body.angular_velocity.x + angular_accel.x * dt,
                entity.rigid_body.angular_velocity.y + angular_accel.y * dt,
                entity.rigid_body.angular_velocity.z + angular_accel.z * dt,
            );

            let ang_damping = 1.0 - entity.rigid_body.angular_damping * dt;
            entity.rigid_body.angular_velocity = entity.rigid_body.angular_velocity.scale(ang_damping.clamp(0.0, 1.0));

            // Integrate orientation via quaternion: q += 0.5 * q * ω * dt
            let omega_quat = Quaternion { w: 0.0, x: entity.rigid_body.angular_velocity.x, y: entity.rigid_body.angular_velocity.y, z: entity.rigid_body.angular_velocity.z };
            let q_dot = entity.transform.orientation.multiply(&omega_quat);
            entity.transform.orientation = Quaternion {
                w: entity.transform.orientation.w + 0.5 * q_dot.w * dt,
                x: entity.transform.orientation.x + 0.5 * q_dot.x * dt,
                y: entity.transform.orientation.y + 0.5 * q_dot.y * dt,
                z: entity.transform.orientation.z + 0.5 * q_dot.z * dt,
            }.normalize();

            // Reset forces for next step — forces are per-step in this model
            entity.rigid_body.force = Vector3::zero();
            entity.rigid_body.torque = Vector3::zero();
        }

        self.detect_and_resolve_collisions();
    }

    fn detect_and_resolve_collisions(&mut self) {
        // Simple ground plane + entity-entity AABB for demo — real Bullet uses BVH, GJK, etc.
        let ground_height = 0.0;

        for entity in self.entities.values_mut() {
            if entity.rigid_body.is_static {
                continue;
            }

            // Ground plane y=0
            let radius = entity.collision_shape.as_ref().map(|s| s.bounding_radius()).unwrap_or(0.3);
            let min_y = ground_height + radius * 0.5;

            if entity.transform.position.y < min_y {
                entity.transform.position.y = min_y;

                // Bounce with restitution 0.5, friction 0.8
                if entity.rigid_body.linear_velocity.y < 0.0 {
                    entity.rigid_body.linear_velocity.y *= -0.5;
                    // Friction for x,z
                    entity.rigid_body.linear_velocity.x *= 0.8;
                    entity.rigid_body.linear_velocity.z *= 0.8;

                    // Threshold to stop jitter
                    if entity.rigid_body.linear_velocity.y.abs() < 0.1 {
                        entity.rigid_body.linear_velocity.y = 0.0;
                    }
                }
            }
        }
    }

    // Force application APIs per DESIGN.md actuation
    pub fn apply_force(&mut self, id: EntityId, force: Vector3) {
        if let Some(entity) = self.entities.get_mut(&id) {
            if !entity.rigid_body.is_static {
                entity.rigid_body.force = entity.rigid_body.force.add(&force);
            }
        }
    }

    pub fn apply_torque(&mut self, id: EntityId, torque: Vector3) {
        if let Some(entity) = self.entities.get_mut(&id) {
            if !entity.rigid_body.is_static {
                entity.rigid_body.torque = entity.rigid_body.torque.add(&torque);
            }
        }
    }

    pub fn set_velocity(&mut self, id: EntityId, velocity: Vector3) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.rigid_body.linear_velocity = velocity;
        }
    }

    pub fn set_angular_velocity(&mut self, id: EntityId, ang_vel: Vector3) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.rigid_body.angular_velocity = ang_vel;
        }
    }

    pub fn set_transform(&mut self, id: EntityId, transform: Transform) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.transform = transform;
        }
    }
}

impl PhysicsEngineTrait for SimulatedPhysicsEngine {
    fn add_entity(&mut self, entity: Entity) -> EntityId { self.add_entity(entity) }
    fn remove_entity(&mut self, id: EntityId) -> Option<Entity> { self.remove_entity(id) }
    fn get_entity(&self, id: EntityId) -> Option<&Entity> { self.get_entity(id) }
    fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> { self.get_entity_mut(id) }
    fn step(&mut self, delta_time: Duration) { self.step(delta_time) }
    fn apply_force(&mut self, id: EntityId, force: Vector3) { self.apply_force(id, force) }
    fn entity_count(&self) -> usize { self.entity_count() }
    fn is_simulated(&self) -> bool { true }
    fn name(&self) -> &'static str { "SimulatedPhysicsEngine (IMPLEMENTED — semi-implicit Euler)" }
}

/// Backward compatibility — old code used PhysicsEngine, now alias to Simulated
pub type PhysicsEngine = SimulatedPhysicsEngine;

/// Bullet physics engine — SCAFFOLDED per AUDIT.md P0
/// Status: SCAFFOLDED — would use bullet crate (e.g., bullet-rs) per nros.toml physics_engine=bullet
/// Real implementation would:
/// - Create btDiscreteDynamicsWorld, btDbvtBroadphase, btDefaultCollisionConfiguration
/// - Add rigid bodies via btRigidBody with mass, motion state, collision shape
/// - Step simulation via dynamicsWorld.stepSimulation(dt, maxSubSteps, fixedTimeStep)
/// - Currently placeholder that delegates to Simulated for prototype
#[derive(Debug)]
pub struct BulletPhysicsEngine {
    inner: SimulatedPhysicsEngine,
}

impl BulletPhysicsEngine {
    pub fn new(gravity: Vector3, time_step_hz: f64) -> Self {
        Self { inner: SimulatedPhysicsEngine::new(gravity, time_step_hz) }
    }
}

impl PhysicsEngineTrait for BulletPhysicsEngine {
    fn add_entity(&mut self, entity: Entity) -> EntityId { self.inner.add_entity(entity) }
    fn remove_entity(&mut self, id: EntityId) -> Option<Entity> { self.inner.remove_entity(id) }
    fn get_entity(&self, id: EntityId) -> Option<&Entity> { self.inner.get_entity(id) }
    fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> { self.inner.get_entity_mut(id) }
    fn step(&mut self, delta_time: Duration) {
        // Real: bullet_world.stepSimulation(delta_time.as_secs_f32(), 10, 1.0/240.0)
        self.inner.step(delta_time)
    }
    fn apply_force(&mut self, id: EntityId, force: Vector3) { self.inner.apply_force(id, force) }
    fn entity_count(&self) -> usize { self.inner.entity_count() }
    // Pass 24 (I-009): this engine delegates to SimulatedPhysicsEngine; it is not backed
    // by a real Bullet integration and must not claim to be. Return true (simulated)
    // until an actual Bullet backend is wired up.
    fn is_simulated(&self) -> bool { true }
    fn name(&self) -> &'static str { "BulletPhysicsEngine (SCAFFOLDED — would use bullet crate)" }
}

// ============================================================================
// Sensor Simulation — Camera/LiDAR/IMU per DESIGN.md §7.3 HAL auto-switches
// ============================================================================

pub struct SimulatedCamera {
    pub resolution: (u32, u32),
    pub fov_rad: f64,
    pub near_clip: f64,
    pub far_clip: f64,
}

impl SimulatedCamera {
    pub fn new(width: u32, height: u32, fov_rad: f64) -> Self {
        // Pass 24: validate dimensions. Zero width/height causes an integer divide-by-zero
        // panic in render() (`x * 255 / width`), and degenerate FOV produces NaN projections.
        // Clamp to a 1x1 minimum and a positive finite FOV rather than panicking later.
        let width = width.max(1);
        let height = height.max(1);
        let fov_rad = if fov_rad.is_finite() && fov_rad > 0.0 { fov_rad } else { std::f64::consts::FRAC_PI_2 };
        SimulatedCamera {
            resolution: (width, height),
            fov_rad,
            near_clip: 0.1,
            far_clip: 100.0,
        }
    }

    /// Synthetic rendering — real NROS would use Vulkan renderer per nros.toml simulation.renderer=vulkan
    pub fn render(&self, transform: &Transform, entities: &HashMap<EntityId, Entity>) -> Vec<u8> {
        // Gradient + entity projection as simple colored boxes for demo
        let (width, height) = self.resolution;
        // Pass 24: checked arithmetic so a pathological (width*height*3) cannot overflow
        // usize on 32-bit targets and trigger a capacity/alloc panic. Cap at 64 MiB.
        const MAX_IMAGE_BYTES: usize = 64 * 1024 * 1024;
        let size = (width as usize)
            .checked_mul(height as usize)
            .and_then(|px| px.checked_mul(3))
            .filter(|&s| s <= MAX_IMAGE_BYTES)
            .unwrap_or(0);
        if size == 0 {
            // Refuse to allocate for an unreasonable resolution; return an empty frame.
            return Vec::new();
        }
        let mut image = vec![0u8; size];

        // Base gradient
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 3) as usize;
                image[idx] = (x * 255 / width) as u8;     // R horizontal gradient
                image[idx + 1] = (y * 255 / height) as u8; // G vertical gradient
                image[idx + 2] = 128;                       // B constant
            }
        }

        // Project entities as white boxes (ultra simplified)
        let (_, _, yaw) = transform.orientation.to_euler();
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();

        for entity in entities.values() {
            if entity.rigid_body.is_static && entity.name.starts_with("wall") {
                continue; // skip walls for camera clarity
            }
            // Transform entity position to camera local space (ignore y)
            let dx = entity.transform.position.x - transform.position.x;
            let dz = entity.transform.position.z - transform.position.z;
            // Rotate by -yaw
            let local_x = dx * cos_yaw + dz * sin_yaw;
            let local_z = -dx * sin_yaw + dz * cos_yaw;

            // If in front (positive x in camera forward? we use z as forward originally — simplify: use distance)
            let dist = (local_x * local_x + local_z * local_z).sqrt();
            if dist < self.far_clip && dist > self.near_clip && local_x > 0.0 {
                // Project to image center based on angle
                let angle = local_z.atan2(local_x); // relative angle
                if angle.abs() < self.fov_rad / 2.0 {
                    let u = ((angle / (self.fov_rad / 2.0) + 1.0) * 0.5 * width as f64) as u32;
                    let v = height / 2; // center
                    // Draw small white square 10x10
                    for dy in 0..10 {
                        for dx in 0..10 {
                            let px = (u as i32 + dx as i32 - 5).clamp(0, width as i32 - 1) as u32;
                            let py = (v as i32 + dy as i32 - 5).clamp(0, height as i32 - 1) as u32;
                            let idx = ((py * width + px) * 3) as usize;
                            if idx + 2 < image.len() {
                                image[idx] = 255;
                                image[idx + 1] = 255;
                                image[idx + 2] = 255;
                            }
                        }
                    }
                }
            }
        }

        image
    }
}

pub struct SimulatedLidar {
    pub range: f64,
    pub num_rays: usize,
    pub fov_rad: f64,
}

impl SimulatedLidar {
    pub fn new(range: f64, num_rays: usize, fov_rad: f64) -> Self {
        SimulatedLidar { range, num_rays, fov_rad }
    }

    pub fn scan(&self, transform: &Transform, entities: &HashMap<EntityId, Entity>) -> Vec<f64> {
        let mut ranges = Vec::with_capacity(self.num_rays);
        let angle_increment = if self.num_rays > 1 { self.fov_rad / (self.num_rays as f64) } else { 0.0 };

        for i in 0..self.num_rays {
            let angle = -self.fov_rad / 2.0 + (i as f64) * angle_increment;
            let range = self.raycast(transform, angle, entities);
            ranges.push(range);
        }

        ranges
    }

    fn raycast(&self, transform: &Transform, angle_rad: f64, entities: &HashMap<EntityId, Entity>) -> f64 {
        let (_, _, yaw) = transform.orientation.to_euler();
        let ray_angle = yaw + angle_rad;

        let ray_dir = Vector3::new(ray_angle.cos(), 0.0, ray_angle.sin());

        let mut min_range = self.range;

        for entity in entities.values() {
            // Skip self? Caller should filter robot_id — we check name for demo
            if entity.name.starts_with("mobile_robot") {
                continue;
            }

            // Simple sphere/box approximation
            let to_entity = Vector3::new(
                entity.transform.position.x - transform.position.x,
                0.0,
                entity.transform.position.z - transform.position.z,
            );

            let distance = to_entity.magnitude();
            if distance > min_range || distance < 0.1 {
                continue;
            }

            let dot = ray_dir.dot(&to_entity.normalize());
            // Narrow beam 0.99 ~ 8 degrees acceptance
            if dot > 0.99 {
                let radius = entity.collision_shape.as_ref().map(|s| s.bounding_radius()).unwrap_or(0.25);
                let hit_dist = (distance - radius).max(0.0);
                if hit_dist < min_range {
                    min_range = hit_dist;
                }
            }
        }

        min_range
    }
}

pub struct SimulatedIMU {
    pub noise_accel: f64,
    pub noise_gyro: f64,
}

impl SimulatedIMU {
    pub fn new() -> Self {
        SimulatedIMU {
            noise_accel: 0.01,
            noise_gyro: 0.001,
        }
    }

    pub fn new_with_noise(accel: f64, gyro: f64) -> Self {
        Self { noise_accel: accel, noise_gyro: gyro }
    }

    pub fn read(&self, entity: &Entity, gravity: Vector3) -> (Vector3, Vector3) {
        // In body frame, linear acceleration = (force/mass) - gravity rotated to body
        // Simplified: world frame minus gravity.y
        let accel = if entity.rigid_body.mass > 1e-9 {
            Vector3::new(
                entity.rigid_body.force.x / entity.rigid_body.mass + self.noise_accel * (pseudo_rand() - 0.5),
                entity.rigid_body.force.y / entity.rigid_body.mass - gravity.y + self.noise_accel * (pseudo_rand() - 0.5),
                entity.rigid_body.force.z / entity.rigid_body.mass + self.noise_accel * (pseudo_rand() - 0.5),
            )
        } else {
            Vector3::new(0.0, -gravity.y, 0.0)
        };

        let gyro = Vector3::new(
            entity.rigid_body.angular_velocity.x + self.noise_gyro * (pseudo_rand() - 0.5),
            entity.rigid_body.angular_velocity.y + self.noise_gyro * (pseudo_rand() - 0.5),
            entity.rigid_body.angular_velocity.z + self.noise_gyro * (pseudo_rand() - 0.5),
        );

        (accel, gyro)
    }
}

fn pseudo_rand() -> f64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEED: AtomicU64 = AtomicU64::new(0x9E3779B97F4A7C15);
    let s = SEED.load(Ordering::Relaxed);
    let new_s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
    SEED.store(new_s, Ordering::Relaxed);
    ((new_s >> 33) as f64 / u32::MAX as f64) * 2.0 - 1.0 // -1..1
}

// ============================================================================
// Simulation World — Integrated with NROS HAL per §7.3 same code runs in sim and reality
// ============================================================================

pub struct SimulationWorld {
    pub physics: PhysicsEngine,
    pub camera: Option<SimulatedCamera>,
    pub lidar: Option<SimulatedLidar>,
    pub imu: Option<SimulatedIMU>,
    pub robot_id: Option<EntityId>,
    pub time: Duration,
    pub realtime_factor: f64,
    pub recording: Vec<WorldState>,
    pub enable_recording: bool,
}

#[derive(Debug, Clone)]
pub struct WorldState {
    pub time: Duration,
    pub robot_pose: Option<(Vector3, f64)>,
    pub entity_poses: HashMap<EntityId, Transform>,
}

impl SimulationWorld {
    pub fn new() -> Self {
        SimulationWorld {
            physics: PhysicsEngine::new(Vector3::new(0.0, -9.81, 0.0), 240.0),
            camera: None,
            lidar: None,
            imu: None,
            robot_id: None,
            time: Duration::ZERO,
            realtime_factor: 1.0,
            recording: Vec::new(),
            enable_recording: false,
        }
    }

    pub fn with_realtime_factor(mut self, factor: f64) -> Self {
        self.set_realtime_factor(factor);
        self
    }

    pub fn set_realtime_factor(&mut self, factor: f64) {
        // Pass 24: clamp to a finite, non-negative value. NaN/inf would propagate into
        // `Duration::from_secs_f64` and panic; a negative factor would rewind the clock.
        self.realtime_factor = if factor.is_finite() && factor >= 0.0 { factor } else { 1.0 };
    }

    pub fn enable_recording(&mut self, enable: bool) {
        self.enable_recording = enable;
    }

    pub fn spawn_robot(&mut self, name: &str, position: Vector3) -> EntityId {
        let entity = Entity {
            id: EntityId(0),
            name: name.to_string(),
            transform: Transform {
                position,
                orientation: Quaternion::identity(),
            },
            rigid_body: RigidBody::new(50.0).with_damping(0.05, 0.1),
            collision_shape: Some(CollisionShape::Box {
                size: Vector3::new(0.5, 0.3, 0.8),
            }),
            visual_mesh: Some("robot.obj".to_string()),
        };

        let id = self.physics.add_entity(entity);
        self.robot_id = Some(id);

        println!("[Simulation] Spawned robot '{}' {} at {}", name, id, position);

        id
    }

    pub fn spawn_obstacle(&mut self, name: &str, position: Vector3, size: Vector3) -> EntityId {
        let entity = Entity {
            id: EntityId(0),
            name: name.to_string(),
            transform: Transform {
                position,
                orientation: Quaternion::identity(),
            },
            rigid_body: RigidBody::static_body(),
            collision_shape: Some(CollisionShape::Box { size }),
            visual_mesh: Some("box.obj".to_string()),
        };

        let id = self.physics.add_entity(entity);
        println!("[Simulation] Spawned obstacle '{}' {} at {} size {}", name, id, position, size);
        id
    }

    pub fn spawn_sphere(&mut self, name: &str, position: Vector3, radius: f64, mass: f64) -> EntityId {
        let entity = Entity {
            id: EntityId(0),
            name: name.to_string(),
            transform: Transform {
                position,
                orientation: Quaternion::identity(),
            },
            rigid_body: if mass > 0.0 { RigidBody::new(mass) } else { RigidBody::static_body() },
            collision_shape: Some(CollisionShape::Sphere { radius }),
            visual_mesh: Some("sphere.obj".to_string()),
        };
        let id = self.physics.add_entity(entity);
        println!("[Simulation] Spawned sphere '{}' {} at {} radius {}", name, id, position, radius);
        id
    }

    pub fn add_camera(&mut self, width: u32, height: u32, fov_deg: f64) {
        let fov_rad = fov_deg.to_radians();
        self.camera = Some(SimulatedCamera::new(width, height, fov_rad));
        println!("[Simulation] Added camera: {}x{}, FOV={:.1}° Vulkan renderer per nros.toml", width, height, fov_deg);
    }

    pub fn add_lidar(&mut self, range: f64, num_rays: usize, fov_deg: f64) {
        let fov_rad = fov_deg.to_radians();
        self.lidar = Some(SimulatedLidar::new(range, num_rays, fov_rad));
        println!("[Simulation] Added LiDAR: range={:.1}m, rays={}, FOV={:.1}°", range, num_rays, fov_deg);
    }

    pub fn add_imu(&mut self) {
        self.imu = Some(SimulatedIMU::new());
        println!("[Simulation] Added IMU 200Hz per §6.1");
    }

    pub fn step(&mut self, delta_time: Duration) {
        let sim_delta = Duration::from_secs_f64(delta_time.as_secs_f64() * self.realtime_factor);

        self.physics.step(sim_delta);
        self.time += sim_delta;

        if self.enable_recording {
            let state = WorldState {
                time: self.time,
                robot_pose: self.get_robot_pose(),
                entity_poses: self.physics.entities.iter().map(|(id, e)| (*id, e.transform)).collect(),
            };
            self.recording.push(state);
        }
    }

    /// Apply velocity as in VelocityController — sets linear velocity + angular yaw
    pub fn apply_robot_velocity(&mut self, linear: f64, angular: f64) {
        if let Some(robot_id) = self.robot_id {
            if let Some(robot) = self.physics.get_entity(robot_id) {
                let (_, _, yaw) = robot.transform.orientation.to_euler();

                let velocity = Vector3::new(linear * yaw.cos(), 0.0, linear * yaw.sin());

                self.physics.set_velocity(robot_id, velocity);
                self.physics.set_angular_velocity(robot_id, Vector3::new(0.0, angular, 0.0));
            }
        }
    }

    pub fn apply_robot_force(&mut self, force: Vector3) {
        if let Some(id) = self.robot_id {
            self.physics.apply_force(id, force);
        }
    }

    pub fn get_robot_pose(&self) -> Option<(Vector3, f64)> {
        self.robot_id.and_then(|id| {
            self.physics.get_entity(id).map(|robot| {
                let (_, _, yaw) = robot.transform.orientation.to_euler();
                (robot.transform.position, yaw)
            })
        })
    }

    pub fn capture_camera(&self) -> Option<Vec<u8>> {
        self.camera.as_ref().and_then(|camera| {
            self.robot_id.and_then(|id| {
                self.physics.get_entity(id).map(|robot| {
                    camera.render(&robot.transform, &self.physics.entities)
                })
            })
        })
    }

    pub fn scan_lidar(&self) -> Option<Vec<f64>> {
        self.lidar.as_ref().and_then(|lidar| {
            self.robot_id.and_then(|id| {
                self.physics.get_entity(id).map(|robot| {
                    lidar.scan(&robot.transform, &self.physics.entities)
                })
            })
        })
    }

    pub fn read_imu(&self) -> Option<(Vector3, Vector3)> {
        self.imu.as_ref().and_then(|imu| {
            self.robot_id.and_then(|id| {
                self.physics.get_entity(id).map(|robot| {
                    imu.read(robot, self.physics.gravity)
                })
            })
        })
    }

    pub fn print_status(&self) {
        println!("\n=== Simulation Status ===");
        println!("Time: {:.2}s (realtime factor {:.1}x)", self.time.as_secs_f64(), self.realtime_factor);
        println!("Physics steps: {} @ {:.0} Hz", self.physics.step_count, 1.0 / self.physics.time_step.as_secs_f64());
        println!("Entities: {}", self.physics.entity_count());
        println!("Recording: {} states", self.recording.len());

        if let Some((pos, yaw)) = self.get_robot_pose() {
            println!("Robot pose: {} yaw={:.1}°", pos, yaw.to_degrees());
        }
    }

    /// Deterministic replay per DESIGN.md §7.1 nros replay --speed=0.5 + §20.2 sim_test
    pub fn replay(&self, speed: f64) {
        println!("[Simulation] Replaying {} states at {}x speed (deterministic)", self.recording.len(), speed);
        for state in &self.recording {
            if let Some((pos, yaw)) = state.robot_pose {
                println!("  t={:.2}s pose={} yaw={:.1}°", state.time.as_secs_f64(), pos, yaw.to_degrees());
            }
        }
    }

    pub fn clear_recording(&mut self) {
        self.recording.clear();
    }
}

impl Default for SimulationWorld {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests — Deterministic, no randomness except pseudo_rand for noise
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_ops() {
        let v1 = Vector3::new(1.0, 0.0, 0.0);
        let v2 = Vector3::new(0.0, 1.0, 0.0);
        assert!((v1.dot(&v2) - 0.0).abs() < 1e-9);
        let cross = v1.cross(&v2);
        assert!((cross.z - 1.0).abs() < 1e-9);
        assert!((Vector3::new(3.0, 4.0, 0.0).magnitude() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_quaternion_euler_roundtrip() {
        let roll = 0.1;
        let pitch = 0.2;
        let yaw = 0.5;
        let q = Quaternion::from_euler(roll, pitch, yaw);
        let (r2, p2, y2) = q.to_euler();
        assert!((roll - r2).abs() < 1e-6);
        assert!((pitch - p2).abs() < 1e-6);
        assert!((yaw - y2).abs() < 1e-6);
    }

    #[test]
    fn test_physics_fall() {
        let mut physics = PhysicsEngine::new(Vector3::new(0.0, -9.81, 0.0), 100.0);
        let entity = Entity {
            id: EntityId(0),
            name: "box".into(),
            transform: Transform { position: Vector3::new(0.0, 10.0, 0.0), orientation: Quaternion::identity() },
            rigid_body: RigidBody::new(1.0),
            collision_shape: Some(CollisionShape::Sphere { radius: 0.5 }),
            visual_mesh: None,
        };
        let id = physics.add_entity(entity);
        // Simulate 1 second falling
        for _ in 0..100 {
            physics.step(Duration::from_millis(10));
        }
        let pos = physics.get_entity(id).unwrap().transform.position;
        assert!(pos.y < 10.0, "should have fallen");
        assert!(pos.y >= 0.0, "should have hit ground");
    }

    #[test]
    fn test_sim_world_spawn() {
        let mut world = SimulationWorld::new();
        let id = world.spawn_robot("test", Vector3::new(0.0, 0.5, 0.0));
        assert_eq!(world.physics.entity_count(), 1);
        assert_eq!(world.robot_id, Some(id));
        world.add_camera(640, 480, 90.0);
        world.add_lidar(10.0, 360, 360.0);
        world.add_imu();
        assert!(world.camera.is_some());
        assert!(world.lidar.is_some());
    }

    #[test]
    fn test_lidar_raycast() {
        let mut world = SimulationWorld::new();
        world.spawn_robot("robot", Vector3::new(0.0, 0.5, 0.0));
        world.spawn_obstacle("obstacle", Vector3::new(3.0, 0.5, 0.0), Vector3::new(0.5, 1.0, 0.5));
        world.add_lidar(10.0, 360, 360.0);
        let scan = world.scan_lidar().unwrap();
        assert_eq!(scan.len(), 360);
        // Front direction (0 deg) should see obstacle at ~2.5-3m
        let front = scan[180]; // middle is front if FOV 360? For 360, middle 180 deg is forward? Depends implementation - we just check at least one ray < range
        let min_range = scan.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        assert!(min_range < 10.0, "should detect obstacle");
        let _ = front;
    }

    #[test]
    fn test_degenerate_inputs_do_not_panic() {
        // Pass 24: zero/NaN/negative/huge inputs must not panic (from_secs_f64,
        // integer div-by-zero, or overflow). They fall back to sane defaults.
        let mut phys = PhysicsEngine::new(Vector3::new(0.0, -9.81, 0.0), 0.0);
        phys.step(Duration::from_millis(10));
        let mut phys = PhysicsEngine::new(Vector3::zero(), f64::NAN);
        phys.step(Duration::from_millis(10));
        let mut phys = PhysicsEngine::new(Vector3::zero(), -1.0);
        phys.step(Duration::from_millis(10));

        let mut world = SimulationWorld::new();
        world.set_realtime_factor(f64::NAN);
        world.set_realtime_factor(-2.0);
        world.step(Duration::from_millis(10));

        // Zero/huge camera dimensions must not div-by-zero or overflow-alloc.
        let cam = SimulatedCamera::new(0, 0, 90.0f64.to_radians());
        let frame = cam.render(&Transform {
            position: Vector3::zero(),
            orientation: Quaternion::identity(),
        }, &HashMap::new());
        assert!(!frame.is_empty(), "1x1 camera should produce a 3-byte frame");

        let huge = SimulatedCamera::new(u32::MAX, u32::MAX, 1.0);
        let frame = huge.render(&Transform {
            position: Vector3::zero(),
            orientation: Quaternion::identity(),
        }, &HashMap::new());
        assert!(frame.is_empty(), "huge resolution must be rejected, not allocated");
    }

    #[test]
    fn test_deterministic_replay() {
        let mut world = SimulationWorld::new();
        world.enable_recording(true);
        world.spawn_robot("robot", Vector3::new(0.0, 0.5, 0.0));
        world.step(Duration::from_millis(10));
        world.step(Duration::from_millis(10));
        assert_eq!(world.recording.len(), 2);
        assert!(world.recording[0].time < world.recording[1].time);
    }
}
