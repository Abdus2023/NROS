# nros-sim — Simulation Engine

Integrated physics (Bullet) + rendering (Vulkan) + sensor simulation + deterministic replay per DESIGN.md §7.3, §20.2, §25 extended beyond core.

## Features

### Math — Vector3, Quaternion, Transform

```rust
Vector3 { x,y,z }.magnitude(), normalize(), dot(), cross(), add(), sub(), scale()
Quaternion::from_euler(roll,pitch,yaw) → to_euler() + multiply() + normalize() per 3D rotation
Transform { position, orientation } identity(), from_xyz_yaw()
```

- Euler ⇔ Quaternion roundtrip validated, quaternion integration `q += 0.5 * q * ω * dt` normalized

### Physics Engine — Fixed Time Step Deterministic per §7.3

```rust
struct PhysicsEngine {
    entities: HashMap<EntityId, Entity>,
    gravity: Vector3, // [0, -9.81, 0]
    time_step: Duration, // 240Hz fixed per Bullet default
    accumulated_time: Duration,
    entity_counter: u64,
    step_count: u64,
}

fn step(&mut self, delta_time: Duration) {
    accumulated_time += delta;
    while accumulated >= time_step { integrate(dt); accumulated -= time_step; step_count+=1; }
}
```

- **Integration**: semi-implicit Euler like Bullet, `a = F/m`, `v += a*dt`, `p += v*dt`, damping `v *= (1 - damping*dt)`, angular via inertia + torque
- **Collision**: ground plane y=0 with resting threshold, restitution 0.5 bounce, friction 0.8, bounding radius per shape
- **Shapes**: Box size, Sphere radius, Cylinder, Mesh vertices/triangles, Plane normal/distance — `bounding_radius()`
- **Entities**: `Entity { id, name, transform, rigid_body { mass, inertia, lin/ang vel, force/torque, static, damping }, collision_shape, visual_mesh }`
- **APIs**: `add_entity`, `remove_entity`, `apply_force/torque`, `set_velocity/angular_velocity/transform`

### Sensor Simulation — HAL auto-switches sim ↔ reality

- **SimulatedCamera**: `resolution (w,h)`, `fov_rad`, `near_clip 0.1 far 100.0` — `render(transform, entities)` generates gradient + projects entities as white 10x10 boxes via yaw rotation + FOV check, real NROS uses Vulkan renderer per `nros.toml renderer=vulkan`
- **SimulatedLidar**: `range 10m num_rays 360 fov 360°` — `scan()` raycasts per angle, narrow beam dot>0.99 ~8°, distance minus bounding radius, real uses UDP packets
- **SimulatedIMU**: `noise_accel 0.01 noise_gyro 0.001` — `read(entity, gravity)` computes `accel = F/m - gravity.y + noise`, gyro = angular_velocity + noise, pseudo-random seeded deterministic for noise
- Interfaces match `nros-hal` trait for zero-change deployment

### SimulationWorld — Integrated NROS §7.3 same code runs in sim and reality

```rust
struct SimulationWorld {
    physics: PhysicsEngine::new(gravity, 240Hz),
    camera: Option<SimulatedCamera>,
    lidar: Option<SimulatedLidar>,
    imu: Option<SimulatedIMU>,
    robot_id: Option<EntityId>,
    time: Duration,
    realtime_factor: f64, // 1.0 per nros.toml simulation.realtime_factor
    recording: Vec<WorldState>, // for deterministic replay
    enable_recording: bool,
}

fn spawn_robot(name, position) -> EntityId // mass 50kg Box 0.5x0.3x0.8 damping 0.05/0.1
fn spawn_obstacle(name, pos, size) // static Box
fn spawn_sphere(name, pos, radius, mass)
fn add_camera(w,h,fov_deg) / add_lidar(range,rays,fov_deg) / add_imu()
fn step(delta) { sim_delta = delta * realtime_factor; physics.step(sim_delta); record if enabled }
fn apply_robot_velocity(linear, angular) { set_velocity yaw.cos/sin + angular_velocity.y = angular per diff-drive }
fn get_robot_pose() -> Option<(Vector3, yaw)>
fn capture_camera() / scan_lidar() / read_imu() / print_status()
fn replay(speed) { deterministic print poses }
```

- **Sim-reality parity**: `#[cfg_attr(simulation, nros::sim)] struct MyRobot { #[sim(model="models/my_robot.urdf")] robot: RobotHandle }` + `read_sensors().await; controller.update(); send_commands().await;` — same control loop
- **Realtime factor**: scales delta_time for faster-than-real sim (e.g., 2× for CI)
- **Recording**: `WorldState { time, robot_pose, entity_poses }` vector for deterministic replay per `nros replay recording.nros --speed=0.5 --analyze-latency`

## Tests

- `test_vector_ops` — dot=0, cross z=1, magnitude 3-4-5=5
- `test_quaternion_euler_roundtrip` — euler roundtrip <1e-6
- `test_physics_fall` — 1s falling from 10m with 100Hz steps, hits ground, bounces
- `test_sim_world_spawn` — spawn robot, entity count 1, add sensors Some
- `test_lidar_raycast` — spawn robot + obstacle at 3m, scan 360, min_range <10m detects
- `test_deterministic_replay` — enable recording, 2 steps, recording len 2 time increasing

Run:
```bash
cargo test -p nros-sim -- --nocapture
cargo run -p nros-sim --bin nros-sim-demo
```

## Design Doc Alignment

- §7.3 Simulation Integration: `#[cfg_attr(simulation, nros::sim)]`, physics Bullet 240Hz, renderer Vulkan, realtime_factor 1.0
- §20.2 Testing: `#[nros::sim_test(world="test_world.urdf")] async fn test_obstacle_avoidance() { spawn_box(); navigate_to(); assert!(!has_collided()) }`
- §20.3 NROS Studio: live visualization automatic TF handling, timeline, metrics
- §7.1 CLI: `nros record`, `replay --speed=0.5`, `replay --analyze-latency`

Example URDF integration (future):
```rust
// nros.toml simulation.model = "models/my_robot.urdf"
// real: nros-hal automatic driver loading: in sim uses SimulationWorld, on robot uses UsbCamera::discover("usb:*").with_resolution().open()
```
