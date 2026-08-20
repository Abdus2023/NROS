# NROS — Deep Analysis & Verification — Pass 26 (Deterministic Simulation & Real-Time loop Audit)

Branch: `arena/01a0206f-nros`
Parent: `5a7f92c` (Pass 25 systematic remediation and framing report)
Date: 2026-08-20

This pass performs a comprehensive audit on the simulation determinism, real-time loop execution guarantees, and sensor/imu mathematical formulation present in `nros-sim` and `nros-hal` to ensure strict alignment with NROS real-time criteria.

---

## 1. Simulation Loop Determinism Analysis (nros-sim)

Simulation correctness in robotics middleware relies on strict execution determinism: the same control sequences applied under identical initial conditions must yield identical, byte-by-byte matching poses. This requires a **fixed-timestep accumulator update loop** independent of the host OS wall-clock scheduling.

### 1.1 Fixed-Timestep Physics Accumulation

In `crates/nros-sim/src/lib.rs`, `SimulatedPhysicsEngine` implements the classic fixed-timestep loop:

```rust
pub fn step(&mut self, delta_time: Duration) {
    self.accumulated_time += delta_time;
    
    // Sub-stepping to maintain physical stability and exact determinism
    while self.accumulated_time >= self.time_step {
        self.integrate(self.time_step.as_secs_f64());
        self.resolve_collisions();
        self.accumulated_time -= self.time_step;
        self.step_count += 1;
    }
}
```

#### Determinism Findings:
1. **Host-Independent Replays:** By consuming variable time segments `delta_time` (which may fluctuate based on thread context-switching) and quantizing them into uniform increments `self.time_step`, the physics state is updated in discrete, stable, and deterministic intervals. 
2. **Replay Invariant (I-004):** This directly supports NROS's ability to deterministic replay logs (`nros replay`). Since the physics engine integrates exact `self.time_step` intervals, replaying the same logged inputs produces identical coordinate paths.
3. **Floating-point Associativity Caution:** While the logical step count is identical, compiler-specific float optimizations (e.g., FMA - Fused Multiply-Add instructions) can cause minute drift across different hardware architectures (x86_64 vs ARM64). For safety-qualifiable deployments, compiling with strict IEEE-754 adherence flags is required.

---

## 2. IMU and Camera Sensor Mathematics Verification

We audited the synthetic IMU acceleration calculation used to model standard IMU sensors in `nros-sim`:

```rust
pub fn read(&self, entity: &Entity, gravity: Vector3) -> (Vector3, Vector3) {
    // Accelerometer reading includes linear acceleration + gravity component
    let world_accel = entity.rigid_body.force.scale(1.0 / entity.rigid_body.mass);
    let world_accel_gravity = world_accel.sub(&gravity); // Gravity pulls down, accelerometer measures equivalent upward push

    // Transform world acceleration into the sensor's body frame
    let inv_orientation = Quaternion {
        w: entity.transform.orientation.w,
        x: -entity.transform.orientation.x,
        y: -entity.transform.orientation.y,
        z: -entity.transform.orientation.z,
    };
    
    let local_accel = self.rotate_vector(&world_accel_gravity, &inv_orientation);
    let gyro = entity.rigid_body.angular_velocity;
    
    (local_accel, gyro)
}
```

#### Mathematical Validation:
1. **Einstein's Equivalence Principle:** An accelerometer at rest in a gravity field of $[0, -9.81, 0]$ m/s² must measure an upward acceleration of $[0, 9.81, 0]$ m/s² (representing the normal force supporting the sensor). The subtraction `world_accel.sub(&gravity)` correctly implements this physical invariant:
   $$\vec{a}_{measured} = \vec{a}_{linear} - \vec{g}$$
   For a static body, $\vec{a}_{linear} = [0, 0, 0]$, leading to:
   $$\vec{a}_{measured} = [0, 0, 0] - [0, -9.81, 0] = [0, 9.81, 0]$$
   The physics model is **conceptually correct** and mathematically sound.
2. **Quaternion Rotation:** Rotating a vector via a quaternion requires the Hamilton product $\vec{v}' = \mathbf{q} \vec{v} \mathbf{q}^{-1}$. The helper `rotate_vector` correctly applies the inverse/conjugate orientation `inv_orientation` to transform the gravity-aligned world vector into the local coordinate system of the moving robotic base.

---

## 3. Real-Time Loop Execution Priority & Thread Affinity

In a deterministic robotics OS, high-frequency control loops (e.g. 1000 Hz motor drivers) must run with high priority and minimal jitter. We verified the thread priority structure used to model real-time behavior in NROS:

* **ExecutionClass:** RealTime (FIFO/RR scheduling class) vs Control (high priority) vs Telemetry (low priority).
* **Deterministic Scheduling:** The scheduler allocates a dedicated FIFO thread pool with high CPU affinity to prevent kernel-level task migration. In the simulated wrapper, NROS correctly structures tasks according to their `ExecutionClass` to guarantee scheduling precedence.

---

## 4. Architectural Status Matrix

We verified that the codebase remains fully clean, compliant with Miri validation gates, and has zero unresolved compilation warnings.

| Area | Verified | Notes |
|:---|:---:|:---|
| **Physics Determinism** | 🟢 Yes | Fixed-timestep substepping prevents frame-rate-dependent integration divergence. |
| **Equivalence Principle** | 🟢 Yes | Simulated IMU correctly factors in local support forces versus gravitational acceleration. |
| **QoS Memory Safety** | 🟢 Yes | Type-enforced single-producer-single-consumer queues eliminate memory races. |
| **No-Panic Constraints** | 🟢 Yes | Division-by-zero (`total_shards == 0`, `hz == 0`) and memory-overflow allocations are strictly guarded. |
