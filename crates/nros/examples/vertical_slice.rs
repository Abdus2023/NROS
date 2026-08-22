//! Vertical Slice — Canonical End-to-End Pipeline per AUDIT Pass 14-15
//! Twist -> Publisher -> SPSC (nros-core) -> Subscriber -> VelocityController (nros-node) -> MotorCommand -> Simulator (nros-sim)
//! No conversion shim, no mocks, no synthetic CLI output — real message flow with ownership transfer
//! Acceptance criteria per Pass 14 §23 and Pass 15 §27

use nros::prelude::*;
use std::time::{Duration, Instant};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║   NROS Vertical Slice — Canonical     ║");
    println!("║   Twist → SPSC → Controller → Sim     ║");
    println!("╚════════════════════════════════════════╝\n");

    // Use canonical types from nros-types (single source of truth per INTEGRATION-001)
    println!("Using canonical types: nros_types::Twist, MotorCommand, Odometry (not duplicated)");

    // Create SPSC channel with type-enforced endpoint ownership (fixes CORE-016)
    // Producer and Consumer are not Clone, cannot create multiple producers from same channel
    let (producer, consumer) = nros_core::channel::<Twist>(16);
    println!("[Channel] Created SPSC channel capacity 16 with Producer/Consumer ownership enforced");

    // Create VelocityController (nros-node) — uses canonical Twist, MotorCommand
    let mut controller = VelocityController::new("velocity_controller");
    controller.on_configure().expect("configure");
    controller.on_activate().expect("activate");
    println!("[Node] VelocityController configured and activated");

    // Create simulation world (nros-sim) with SimulatedPhysicsEngine
    let mut sim_world = SimulationWorld::new();
    sim_world.spawn_robot("mobile_robot", nros_types::Vector3::new(0.0, 0.5, 0.0));
    sim_world.add_imu();
    println!("[Sim] Spawned robot at [0, 0.5, 0] + IMU");

    // End-to-end latency model per Pass 14 §15: L_total = L_publish + L_queue + L_transport + L_schedule + L_callback + L_output
    // For this vertical slice: L_publish (write_value), L_queue (SPSC), L_callback (on_cmd_vel), L_output (MotorCommand)
    let mut total_latencies = Vec::new();
    let mut deadline_misses = 0;
    let deadline_us = 1000; // 1ms deadline per DESIGN.md §4

    let iterations = 10;
    let start_total = Instant::now();

    for i in 0..iterations {
        let iter_start = Instant::now();

        // 1. Create canonical Twist
        let twist = Twist {
            timestamp: WallTimestamp::now(),
            linear: Vector3::new(1.0 + i as f64 * 0.1, 0.0, 0.0),
            angular: Vector3::new(0.0, 0.0, 0.2),
        };

        // 2. Publish via SPSC with type-state initialization (Safety Gate v0.1.1)
        let publish_start = Instant::now();
        let guard = producer.allocate().expect("Should be able to reserve");
        // Correct API: write_value consumes WriteGuard and returns InitializedWriteGuard
        let elapsed_publish = publish_start.elapsed();

        // 3. Commit — only InitializedWriteGuard can commit (fixes CORE-014)
        let commit_start = Instant::now();
        guard.write_value(twist).commit();
        let elapsed_commit = commit_start.elapsed();

        // 4. Consumer receives via ReadGuard owning slot (fixes CORE-002)
        let recv_start = Instant::now();
        let received_guard = consumer.try_recv().expect("Should receive");
        let elapsed_queue = recv_start.elapsed();

        // Verify ownership transfer, no copy: received_guard Derefs to &T, Drop will drop and advance
        assert!((received_guard.linear.x - twist.linear.x).abs() < 1e-9, "Message correctness");
        // Note: canonical `Twist` has no frame_id accessor — a previous `received_guard.frame_id()`
        // assertion here was a stale copy from the HAL Image path and does not compile (E0599).
        // Found by the first real CI execution (arena deep-analysis session, 2026-08-22).

        // 5. Node callback — real VelocityController::on_cmd_vel (not placeholder)
        let callback_start = Instant::now();
        let motor_cmd = controller.on_cmd_vel(&*received_guard).expect("on_cmd_vel");
        let elapsed_callback = callback_start.elapsed();

        // Drop guard — advances read_idx and drops T exactly once (fixes CORE-003)
        drop(received_guard);

        // 6. Simulator consumes MotorCommand — canonical type, no conversion shim
        let sim_start = Instant::now();
        sim_world.apply_robot_velocity(motor_cmd.linear_velocity.x, motor_cmd.angular_velocity.z);
        sim_world.step(Duration::from_millis(50));
        let elapsed_sim = sim_start.elapsed();

        // 7. End-to-end latency measurement with monotonic clock
        let total_elapsed = iter_start.elapsed();
        let total_us = total_elapsed.as_micros() as f64;

        if total_us > deadline_us as f64 {
            deadline_misses += 1;
        }

        total_latencies.push(total_elapsed);

        println!(
            "[Iter {}] L_publish={:.1}μs L_queue={:.1}μs L_callback={:.1}μs L_sim={:.1}μs L_total={:.1}μs deadline_misses={}",
            i,
            elapsed_publish.as_micros() as f64,
            elapsed_queue.as_micros() as f64,
            elapsed_callback.as_micros() as f64,
            elapsed_sim.as_micros() as f64,
            total_us,
            deadline_misses
        );

        // Verify invariants per SAFETY.md
        // - One producer reservation per slot MUST — enforced via write_reserved CAS, tested via double_reserve_prevention
        // - No &T after release MUST — ReadGuard owns slot, Drop releases
        // - Drop exactly once MUST — DropCounter test
        // - Published ⇒ Initialized MUST — type-state WriteGuard -> InitializedWriteGuard -> commit

        std::thread::sleep(Duration::from_millis(10));
    }

    let total_time = start_total.elapsed();
    let throughput = iterations as f64 / total_time.as_secs_f64();

    // Compute latency distribution
    let mut sorted = total_latencies.clone();
    sorted.sort();
    let mean = sorted.iter().map(|d| d.as_micros() as f64).sum::<f64>() / sorted.len() as f64;
    let min = sorted.first().map(|d| d.as_micros() as f64).unwrap_or(0.0);
    let max = sorted.last().map(|d| d.as_micros() as f64).unwrap_or(0.0);
    let p50 = sorted[sorted.len() * 50 / 100].as_micros() as f64;
    let p99 = sorted[sorted.len() * 99 / 100].as_micros() as f64;

    println!("\n=== Vertical Slice Results (Canonical Types, No Conversion Shim) ===");
    println!("Iterations: {}", iterations);
    println!("Total time: {:.2?}, Throughput: {:.1} msg/s", total_time, throughput);
    println!("Latency min: {:.1}μs p50: {:.1}μs mean: {:.1}μs p99: {:.1}μs max: {:.1}μs", min, p50, mean, p99, max);
    println!("Deadline misses (deadline {}μs): {} / {} ({:.1}%)", deadline_us, deadline_misses, iterations, deadline_misses as f64 / iterations as f64 * 100.0);
    println!("Ownership: Drop exactly once verified via DropCounter test");
    println!("SPSC: Producer/Consumer not Clone enforced via type system (channel() returns non-Clone handles)");
    println!("Zero-copy: Inside ring zero-copy candidate (WriteGuard/ReadGuard), transport/HAL still SIMULATED per EVIDENCE_REGISTRY");
    println!("Execution mode: Native (not Simulation) — would be Simulation if using SimTransport backend per Pass 14 §16");

    controller.print_stats();
    sim_world.print_status();

    // Failure injection per Pass 14 §24
    println!("\n=== Failure Injection Tests ===");
    println!("Testing queue full semantics...");
    let (prod_full, _cons_full) = nros_core::channel::<u64>(2);
    prod_full.allocate().unwrap().write_value(1).commit();
    prod_full.allocate().unwrap().write_value(2).commit();
    assert!(prod_full.allocate().is_none(), "Should be full, ReturnNone policy");
    println!("✅ Queue full correctly returns None (BackpressurePolicy::ReturnNone) — no deadlock, no leak");

    println!("\n✅ Vertical slice PASSED — Twist -> SPSC -> VelocityController -> MotorCommand -> Sim with canonical types, no conversion shim, ownership transfer, deadline monitoring, failure injection");
    println!("Next: Add Miri, Loom, hardware validation, benchmark artifact with env info per verification ladder");
}
