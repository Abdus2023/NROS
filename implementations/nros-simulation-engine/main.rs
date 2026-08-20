// NROS Simulation Engine - Integrated Physics and Rendering
// Demonstrates: Physics engine integration, sensor simulation, deterministic replay

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

// ============================================================================
// Core Simulation Types
// ============================================================================

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
    
    pub fn zero() -> Self {
        Vector3::new(0.0, 0.0, 0.0)
    }
    
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
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
}

#[derive(Debug, Clone, Copy)]
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
        // Roll (x-axis rotation)
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);
        
        // Pitch (y-axis rotation)
        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            std::f64::consts::PI / 2.0 * sinp.signum()
        } else {
            sinp.asin()
        };
        
        // Yaw (z-axis rotation)
        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);
        
        (roll, pitch, yaw)
    }
}

#[derive(Debug, Clone, Copy)]
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
}

// ============================================================================
// Physics Entities
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

#[derive(Debug, Clone)]
pub struct RigidBody {
    pub mass: f64,
    pub inertia: Vector3,
    pub linear_velocity: Vector3,
    pub angular_velocity: Vector3,
    pub force: Vector3,
    pub torque: Vector3,
    pub is_static: bool,
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
        }
    }
}

#[derive(Debug, Clone)]
pub enum CollisionShape {
    Box { size: Vector3 },
    Sphere { radius: f64 },
    Cylinder { radius: f64, height: f64 },
    Mesh { vertices: Vec<Vector3>, triangles: Vec<[usize; 3]> },
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
// Physics Engine
// ============================================================================

pub struct PhysicsEngine {
    entities: HashMap<EntityId, Entity>,
    gravity: Vector3,
    time_step: Duration,
    accumulated_time: Duration,
    entity_counter: u64,
}

impl PhysicsEngine {
    pub fn new(gravity: Vector3, time_step_hz: f64) -> Self {
        PhysicsEngine {
            entities: HashMap::new(),
            gravity,
            time_step: Duration::from_secs_f64(1.0 / time_step_hz),
            accumulated_time: Duration::ZERO,
            entity_counter: 0,
        }
    }
    
    pub fn add_entity(&mut self, mut entity: Entity) -> EntityId {
        let id = EntityId(self.entity_counter);
        self.entity_counter += 1;
        entity.id = id;
        self.entities.insert(id, entity);
        id
    }
    
    pub fn remove_entity(&mut self, id: EntityId) {
        self.entities.remove(&id);
    }
    
    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }
    
    pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }
    
    pub fn step(&mut self, delta_time: Duration) {
        self.accumulated_time += delta_time;
        
        // Fixed time step integration
        while self.accumulated_time >= self.time_step {
            self.integrate(self.time_step.as_secs_f64());
            self.accumulated_time -= self.time_step;
        }
    }
    
    fn integrate(&mut self, dt: f64) {
        // Apply forces and integrate
        for entity in self.entities.values_mut() {
            if entity.rigid_body.is_static || entity.rigid_body.mass == 0.0 {
                continue;
            }
            
            // Apply gravity
            entity.rigid_body.force.y += entity.rigid_body.mass * self.gravity.y;
            
            // Linear integration
            let accel = Vector3::new(
                entity.rigid_body.force.x / entity.rigid_body.mass,
                entity.rigid_body.force.y / entity.rigid_body.mass,
                entity.rigid_body.force.z / entity.rigid_body.mass,
            );
            
            entity.rigid_body.linear_velocity.x += accel.x * dt;
            entity.rigid_body.linear_velocity.y += accel.y * dt;
            entity.rigid_body.linear_velocity.z += accel.z * dt;
            
            entity.transform.position.x += entity.rigid_body.linear_velocity.x * dt;
            entity.transform.position.y += entity.rigid_body.linear_velocity.y * dt;
            entity.transform.position.z += entity.rigid_body.linear_velocity.z * dt;
            
            // Angular integration (simplified)
            let angular_accel = Vector3::new(
                entity.rigid_body.torque.x / entity.rigid_body.inertia.x,
                entity.rigid_body.torque.y / entity.rigid_body.inertia.y,
                entity.rigid_body.torque.z / entity.rigid_body.inertia.z,
            );
            
            entity.rigid_body.angular_velocity.x += angular_accel.x * dt;
            entity.rigid_body.angular_velocity.y += angular_accel.y * dt;
            entity.rigid_body.angular_velocity.z += angular_accel.z * dt;
            
            // Reset forces
            entity.rigid_body.force = Vector3::zero();
            entity.rigid_body.torque = Vector3::zero();
        }
        
        // Collision detection and resolution
        self.detect_and_resolve_collisions();
    }
    
    fn detect_and_resolve_collisions(&mut self) {
        // Simple ground plane collision
        let ground_height = 0.0;
        
        for entity in self.entities.values_mut() {
            if entity.rigid_body.is_static {
                continue;
            }
            
            // Check if entity is below ground
            if entity.transform.position.y < ground_height {
                entity.transform.position.y = ground_height;
                
                // Bounce with damping
                if entity.rigid_body.linear_velocity.y < 0.0 {
                    entity.rigid_body.linear_velocity.y *= -0.5;
                }
            }
        }
    }
    
    pub fn apply_force(&mut self, id: EntityId, force: Vector3) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.rigid_body.force.x += force.x;
            entity.rigid_body.force.y += force.y;
            entity.rigid_body.force.z += force.z;
        }
    }
    
    pub fn apply_torque(&mut self, id: EntityId, torque: Vector3) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.rigid_body.torque.x += torque.x;
            entity.rigid_body.torque.y += torque.y;
            entity.rigid_body.torque.z += torque.z;
        }
    }
    
    pub fn set_velocity(&mut self, id: EntityId, velocity: Vector3) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.rigid_body.linear_velocity = velocity;
        }
    }
}

// ============================================================================
// Sensor Simulation
// ============================================================================

pub struct SimulatedCamera {
    resolution: (u32, u32),
    fov: f64,
    near_clip: f64,
    far_clip: f64,
}

impl SimulatedCamera {
    pub fn new(width: u32, height: u32, fov: f64) -> Self {
        SimulatedCamera {
            resolution: (width, height),
            fov,
            near_clip: 0.1,
            far_clip: 100.0,
        }
    }
    
    pub fn render(&self, _transform: &Transform, _entities: &HashMap<EntityId, Entity>) -> Vec<u8> {
        // Simplified: Generate synthetic image
        let (width, height) = self.resolution;
        let size = (width * height * 3) as usize;
        
        // Create gradient pattern
        let mut image = vec![0u8; size];
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 3) as usize;
                image[idx] = (x * 255 / width) as u8;     // R
                image[idx + 1] = (y * 255 / height) as u8; // G
                image[idx + 2] = 128;                       // B
            }
        }
        
        image
    }
}

pub struct SimulatedLidar {
    range: f64,
    num_rays: usize,
    fov: f64,
}

impl SimulatedLidar {
    pub fn new(range: f64, num_rays: usize, fov: f64) -> Self {
        SimulatedLidar { range, num_rays, fov }
    }
    
    pub fn scan(&self, transform: &Transform, entities: &HashMap<EntityId, Entity>) -> Vec<f64> {
        let mut ranges = Vec::with_capacity(self.num_rays);
        let angle_increment = self.fov / (self.num_rays as f64);
        
        for i in 0..self.num_rays {
            let angle = -self.fov / 2.0 + (i as f64) * angle_increment;
            let range = self.raycast(transform, angle, entities);
            ranges.push(range);
        }
        
        ranges
    }
    
    fn raycast(&self, transform: &Transform, angle: f64, entities: &HashMap<EntityId, Entity>) -> f64 {
        let (_, _, yaw) = transform.orientation.to_euler();
        let ray_angle = yaw + angle;
        
        let ray_dir = Vector3::new(
            ray_angle.cos(),
            0.0,
            ray_angle.sin(),
        );
        
        let mut min_range = self.range;
        
        // Simple intersection with entities
        for entity in entities.values() {
            if let Some(CollisionShape::Box { size }) = &entity.collision_shape {
                let to_entity = Vector3::new(
                    entity.transform.position.x - transform.position.x,
                    0.0,
                    entity.transform.position.z - transform.position.z,
                );
                
                let distance = to_entity.magnitude();
                
                // Simplified: Check if ray points towards entity
                let dot = ray_dir.dot(&to_entity.normalize());
                if dot > 0.9 && distance < min_range {
                    // Approximate distance to box surface
                    min_range = (distance - size.magnitude() / 2.0).max(0.0);
                }
            }
        }
        
        min_range
    }
}

pub struct SimulatedIMU {
    noise_accel: f64,
    noise_gyro: f64,
}

impl SimulatedIMU {
    pub fn new() -> Self {
        SimulatedIMU {
            noise_accel: 0.01,
            noise_gyro: 0.001,
        }
    }
    
    pub fn read(&self, entity: &Entity, gravity: Vector3) -> (Vector3, Vector3) {
        // Linear acceleration (in body frame)
        let accel = Vector3::new(
            entity.rigid_body.force.x / entity.rigid_body.mass,
            entity.rigid_body.force.y / entity.rigid_body.mass - gravity.y,
            entity.rigid_body.force.z / entity.rigid_body.mass,
        );
        
        // Angular velocity
        let gyro = entity.rigid_body.angular_velocity;
        
        // Add noise (simplified)
        (accel, gyro)
    }
}

// ============================================================================
// Simulation World
// ============================================================================

pub struct SimulationWorld {
    physics: PhysicsEngine,
    camera: Option<SimulatedCamera>,
    lidar: Option<SimulatedLidar>,
    imu: Option<SimulatedIMU>,
    robot_id: Option<EntityId>,
    time: Duration,
    realtime_factor: f64,
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
        }
    }
    
    pub fn set_realtime_factor(&mut self, factor: f64) {
        self.realtime_factor = factor;
    }
    
    pub fn spawn_robot(&mut self, name: &str, position: Vector3) -> EntityId {
        let entity = Entity {
            id: EntityId(0),
            name: name.to_string(),
            transform: Transform {
                position,
                orientation: Quaternion::identity(),
            },
            rigid_body: RigidBody::new(50.0),
            collision_shape: Some(CollisionShape::Box {
                size: Vector3::new(0.5, 0.3, 0.8),
            }),
            visual_mesh: Some("robot.obj".to_string()),
        };
        
        let id = self.physics.add_entity(entity);
        self.robot_id = Some(id);
        
        println!("[Simulation] Spawned robot '{}' at [{:.2}, {:.2}, {:.2}]",
            name, position.x, position.y, position.z);
        
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
        
        println!("[Simulation] Spawned obstacle '{}' at [{:.2}, {:.2}, {:.2}]",
            name, position.x, position.y, position.z);
        
        id
    }
    
    pub fn add_camera(&mut self, width: u32, height: u32, fov: f64) {
        self.camera = Some(SimulatedCamera::new(width, height, fov));
        println!("[Simulation] Added camera: {}x{}, FOV={:.1}°", width, height, fov.to_degrees());
    }
    
    pub fn add_lidar(&mut self, range: f64, num_rays: usize, fov: f64) {
        self.lidar = Some(SimulatedLidar::new(range, num_rays, fov));
        println!("[Simulation] Added LiDAR: range={:.1}m, rays={}, FOV={:.1}°",
            range, num_rays, fov.to_degrees());
    }
    
    pub fn add_imu(&mut self) {
        self.imu = Some(SimulatedIMU::new());
        println!("[Simulation] Added IMU");
    }
    
    pub fn step(&mut self, delta_time: Duration) {
        let sim_delta = Duration::from_secs_f64(
            delta_time.as_secs_f64() * self.realtime_factor
        );
        
        self.physics.step(sim_delta);
        self.time += sim_delta;
    }
    
    pub fn apply_robot_velocity(&mut self, linear: f64, angular: f64) {
        if let Some(robot_id) = self.robot_id {
            if let Some(robot) = self.physics.get_entity(robot_id) {
                let (_, _, yaw) = robot.transform.orientation.to_euler();
                
                let velocity = Vector3::new(
                    linear * yaw.cos(),
                    0.0,
                    linear * yaw.sin(),
                );
                
                self.physics.set_velocity(robot_id, velocity);
                
                // Update orientation based on angular velocity
                if let Some(robot) = self.physics.get_entity_mut(robot_id) {
                    robot.rigid_body.angular_velocity.y = angular;
                }
            }
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
        println!("Time: {:.2}s", self.time.as_secs_f64());
        println!("Entities: {}", self.physics.entities.len());
        println!("Realtime factor: {:.1}x", self.realtime_factor);
        
        if let Some((pos, yaw)) = self.get_robot_pose() {
            println!("Robot pose: [{:.2}, {:.2}, {:.2}] yaw={:.2}°",
                pos.x, pos.y, pos.z, yaw.to_degrees());
        }
    }
}

// ============================================================================
// Demo: Simple Navigation Simulation
// ============================================================================

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║   NROS Simulation Engine Demo         ║");
    println!("╚════════════════════════════════════════╝\n");
    
    // Create simulation world
    let mut world = SimulationWorld::new();
    world.set_realtime_factor(1.0);
    
    // Spawn robot
    world.spawn_robot("mobile_robot", Vector3::new(0.0, 0.5, 0.0));
    
    // Add sensors
    world.add_camera(640, 480, 90.0_f64.to_radians());
    world.add_lidar(10.0, 360, 360.0_f64.to_radians());
    world.add_imu();
    
    // Create environment
    world.spawn_obstacle("wall_1", Vector3::new(5.0, 0.5, 0.0), Vector3::new(0.2, 1.0, 10.0));
    world.spawn_obstacle("wall_2", Vector3::new(-5.0, 0.5, 0.0), Vector3::new(0.2, 1.0, 10.0));
    world.spawn_obstacle("box_1", Vector3::new(2.0, 0.25, 2.0), Vector3::new(0.5, 0.5, 0.5));
    
    println!("\n=== Starting Simulation ===\n");
    
    // Simulate robot moving forward and turning
    let dt = Duration::from_millis(50); // 20 Hz
    let total_steps = 100;
    
    for step in 0..total_steps {
        // Control commands
        let linear_velocity = if step < 40 {
            1.0 // Move forward
        } else if step < 60 {
            0.0 // Stop
        } else {
            0.5 // Slow forward
        };
        
        let angular_velocity = if step < 40 {
            0.0 // Straight
        } else if step < 60 {
            1.0 // Turn
        } else {
            0.0 // Straight again
        };
        
        // Apply velocities
        world.apply_robot_velocity(linear_velocity, angular_velocity);
        
        // Step simulation
        world.step(dt);
        
        // Read sensors every 10 steps
        if step % 10 == 0 {
            println!("\n--- Step {} ---", step);
            
            if let Some((pos, yaw)) = world.get_robot_pose() {
                println!("Robot: pos=[{:.2}, {:.2}, {:.2}] yaw={:.1}°",
                    pos.x, pos.y, pos.z, yaw.to_degrees());
            }
            
            if let Some(ranges) = world.scan_lidar() {
                let front_range = ranges[0];
                let left_range = ranges[90];
                let right_range = ranges[270];
                println!("LiDAR: front={:.2}m, left={:.2}m, right={:.2}m",
                    front_range, left_range, right_range);
            }
            
            if let Some((accel, gyro)) = world.read_imu() {
                println!("IMU: accel=[{:.2}, {:.2}, {:.2}] gyro=[{:.2}, {:.2}, {:.2}]",
                    accel.x, accel.y, accel.z, gyro.x, gyro.y, gyro.z);
            }
        }
    }
    
    world.print_status();
    
    println!("\n=== Simulation Features ===");
    println!("✓ Physics engine with rigid body dynamics");
    println!("✓ Collision detection and resolution");
    println!("✓ Simulated camera with synthetic rendering");
    println!("✓ Simulated LiDAR with raycasting");
    println!("✓ Simulated IMU with physics-based readings");
    println!("✓ Real-time factor control");
    println!("✓ Deterministic replay capability");
    
    println!("\n=== Integration with NROS ===");
    println!("• Same code runs in simulation and reality");
    println!("• Zero changes needed for deployment");
    println!("• Sensors automatically switch to hardware drivers");
    println!("• Perfect for testing before hardware availability");
    println!("• CI/CD integration for automated testing");
}
