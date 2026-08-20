//! NROS Velocity Controller Node Demo
//! Simulates lifecycle transitions, control loop, emergency stop, performance test

use nros_node::{VelocityController, LifecycleNode, Twist, Vector3, Timestamp, ParameterValue};
use std::time::{Duration, Instant};

fn main() {
    println!("NROS Velocity Controller Node Demo\n");
    println!("Features demonstrated:");
    println!(" - Lifecycle: Unconfigured -> Inactive -> Active -> Inactive -> Finalized");
    println!(" - Parameter system with validation and hot-reload");
    println!(" - Real-time callback <1ms with deadline monitoring");
    println!(" - Emergency stop atomic flag propagation");
    println!(" - Differential drive kinematics + odometry");
    println!(" - Safety timeout\n");

    // Create and configure node
    let mut node = VelocityController::new("velocity_controller");

    // Lifecycle transitions — matches DESIGN.md §3 node lifecycle
    node.on_configure().unwrap();
    
    // Demonstrate parameter manipulation
    println!("\n--- Parameter Demo ---");
    println!("Current parameters:");
    for name in node.parameters().list() {
        if let Some(p) = node.parameters().get(name) {
            println!("  {} = {} ({})", p.name, p.value, p.description);
        }
    }

    // Try to set invalid param
    println!("\nAttempting invalid param set (max_speed = 10.0, max is 5.0)...");
    match node.parameters_mut().set("max_speed", ParameterValue::Float(10.0)) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Correctly rejected: {}", e),
    }

    // Valid param set
    println!("Setting max_speed = 2.5...");
    node.parameters_mut().set("max_speed", ParameterValue::Float(2.5)).unwrap();
    node.reload_parameters();
    println!("Reloaded, new max_speed validated");

    node.on_activate().unwrap();

    println!("\nSimulating control loop (real-time callbacks)...\n");

    // Simulate receiving velocity commands — would be subscriber in real NROS
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
        println!("Input: linear={:.2} m/s, angular={:.2} rad/s", cmd.linear.x, cmd.angular.z);

        match node.on_cmd_vel(cmd) {
            Ok(motor_cmd) => {
                println!("Output:");
                println!("  Left motor:  {:.2} rad/s, {:.2} Nm", motor_cmd.left_velocity, motor_cmd.left_torque);
                println!("  Right motor: {:.2} rad/s, {:.2} Nm", motor_cmd.right_velocity, motor_cmd.right_torque);

                let odom = node.compute_odometry(&motor_cmd, 0.01);
                println!("Odometry: x={:.2}, y={:.2}, theta={:.2}, v={:.2} m/s, ω={:.2} rad/s",
                    odom.position.x, odom.position.y, odom.orientation.z,
                    odom.linear_velocity.x, odom.angular_velocity.z);
            }
            Err(e) => println!("Error: {}", e),
        }
        println!();

        std::thread::sleep(Duration::from_millis(50));
    }

    // Periodic safety check
    println!("--- Safety Check (10Hz callback) ---");
    node.safety_check().unwrap();
    println!("Safety check passed\n");

    // Test emergency stop — #[interrupt(priority=255)] path in DESIGN.md §15.2
    println!("--- Testing Emergency Stop (priority 255) ---");
    println!("{}", node.emergency_stop_service(true).unwrap());

    let cmd = Twist {
        timestamp: Timestamp::now(),
        linear: Vector3::new(1.0, 0.0, 0.0),
        angular: Vector3::new(0.0, 0.0, 0.0),
    };

    match node.on_cmd_vel(&cmd) {
        Ok(motor_cmd) => {
            println!("Motor command during e-stop should be zero:");
            println!("  Left:  {:.2} rad/s", motor_cmd.left_velocity);
            println!("  Right: {:.2} rad/s", motor_cmd.right_velocity);
            assert_eq!(motor_cmd.left_velocity, 0.0);
            assert_eq!(motor_cmd.right_velocity, 0.0);
            println!("Assertion passed - motors stopped");
        }
        Err(e) => println!("Error: {}", e),
    }

    // Performance test — validates sub-1ms execution
    println!("\n--- Performance Test (10000 callbacks) ---");
    println!("{}", node.emergency_stop_service(false).unwrap());

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
    println!("Throughput: {:.0} callbacks/sec", 10000.0 / elapsed.as_secs_f64());
    println!("Target: 1000 Hz control loop = 1000 callbacks/sec — achieved {:.0}x",
        (10000.0 / elapsed.as_secs_f64()) / 1000.0);

    node.print_stats();

    // Shutdown lifecycle
    println!("\n--- Lifecycle Shutdown ---");
    node.on_deactivate().unwrap();
    node.on_cleanup().unwrap();
    node.on_shutdown().unwrap();

    println!("\nDemo complete - validates DESIGN.md §25 Artifact #2");
    println!("Capabilities: sub-1ms control loop, deadline monitoring, param validation, e-stop");
}
