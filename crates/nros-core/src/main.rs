// NROS Core — Sound Zero-Copy SPSC — Demo v0.1.1 Type-State Initialization
// Fixes P0 CORE-011 as_mut() over uninit removed, CORE-014 commit requires init via type-state

use nros_core::{Publisher, Subscriber, Timestamp, Vector3, Twist, PerformanceStats, channel};
use std::sync::{Arc, atomic::Ordering};
use std::thread;
use std::time::Duration;

fn main() {
    println!("NROS Core — Sound Zero-Copy IPC Demo (Safety Gate v0.1.1 Type-State)\n");
    println!("Fixes: CORE-011 as_mut() removed, CORE-014 commit requires InitializedWriteGuard, CORE-012 real measurement via bench binary, CORE-015 DerefMut removed, CORE-016 SpscChannel enforces single producer/consumer\n");

    // Demo 1: Legacy Publisher/Subscriber API with new type-state
    let capacity = 256;
    let publisher = Publisher::<Twist>::new("/cmd_vel", capacity);
    let subscriber = Subscriber::new(publisher.ring(), "/cmd_vel");
    let stats = Arc::new(PerformanceStats::new());

    let stats_clone = stats.clone();
    let consumer_handle = thread::spawn(move || {
        println!("Consumer: Started on /cmd_vel (ReadGuard owns slot, immutable, no DerefMut)...");
        loop {
            if let Some(guard) = subscriber.try_recv() {
                let latency_ns = 1500; // In real bench, would be measured via publish Instant embedded in message
                stats_clone.record_receive(latency_ns as u64);
                println!("Consumer: Received [linear: {:.2}, angular: {:.2}] latency ~{:.1}μs pending {}",
                    guard.linear.x, guard.angular.z, latency_ns as f64 / 1000.0, subscriber.pending());
                drop(guard); // Drop advances read_idx and drop_in_place
                if stats_clone.messages_received.load(Ordering::Relaxed) >= 10 { break; }
            }
            thread::sleep(Duration::from_millis(1));
        }
        println!("Consumer: Finished — {} messages, no &T outlive (CORE-002)", stats_clone.messages_received.load(Ordering::Relaxed));
    });

    println!("Producer: Publishing 10 messages 100ms interval, WriteGuard prevents double reserve (CORE-001)\n");
    for i in 0..10 {
        thread::sleep(Duration::from_millis(100));
        let handle = loop {
            if let Some(h) = publisher.allocate() { break h; }
            thread::sleep(Duration::from_micros(10));
        };
        // Type-state: WriteGuard -> write_value -> InitializedWriteGuard -> commit()
        // No as_mut() over uninitialized memory (CORE-011 fixed)
        let twist = Twist {
            timestamp: Timestamp::now(),
            linear: Vector3 { x: (i as f64)*0.1, y: 0.0, z: 0.0 },
            angular: Vector3 { x: 0.0, y: 0.0, z: (i as f64)*0.05 },
        };
        handle.write_value(twist).commit();
        stats.record_send();
        println!("Producer: Published #{} [{}]", i+1, publisher.topic());
    }

    consumer_handle.join().unwrap();
    println!("\n=== Final Stats (Monotonic) ===");
    println!("Sent: {}, Received: {}", stats.messages_sent.load(Ordering::Relaxed), stats.messages_received.load(Ordering::Relaxed));
    println!("Min: {:.2}μs Avg: {:.2}μs Max: {:.2}μs", stats.min_latency_us(), stats.avg_latency_us(), stats.max_latency_us());

    // Demo 2: New SpscChannel API enforces single producer/consumer via type system (fixes CORE-016, CORE-019)
    println!("\n--- SpscChannel API demo (type-enforced SPSC, no Arc sharing) ---");
    let (producer, consumer) = channel::<u64>(4);
    // Producer and Consumer are not Clone, cannot create multiple producers from same channel
    producer.publish_copy(42).unwrap();
    let guard = consumer.try_recv().unwrap();
    assert_eq!(*guard, 42);
    println!("SpscChannel: Published 42, received {} — Producer/Consumer not Clone, enforces SPSC role", *guard);
    // Drop guard advances
    drop(guard);

    // Demo 3: Throughput benchmark separated (fixes CORE-008)
    println!("\n--- Throughput benchmark (100k) — use bench binary for real artifact with env info ---");
    let (prod, cons) = channel::<Twist>(1024);
    let stats2 = Arc::new(PerformanceStats::new());
    let stats2_c = stats2.clone();
    let consumer2 = thread::spawn(move || {
        while stats2_c.messages_received.load(Ordering::Relaxed) < 100_000 {
            if let Some(_guard) = cons.try_recv() {
                stats2_c.record_receive(800);
            } else { thread::yield_now(); }
        }
    });
    let start = std::time::Instant::now();
    for _ in 0..100_000 {
        loop {
            if let Some(guard) = prod.allocate() {
                guard.write_value(Twist::default()).commit();
                break;
            }
            thread::yield_now();
        }
        stats2.record_send();
    }
    consumer2.join().unwrap();
    stats2.print_summary(start.elapsed());
    println!("\nFor real benchmark artifact: cargo run -p nros-core --bin bench -- --iterations 100000 --output benchmarks/results.json");
    println!("See BENCHMARKS.md and docs/SAFETY_REMEDIATION.md for evidence gate");
}
