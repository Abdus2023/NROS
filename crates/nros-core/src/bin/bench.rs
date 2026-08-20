//! NROS Core Benchmark Artifact Generator — per AUDIT Pass 7 §12
//! Generates benchmark artifact with environment info: CPU model, OS, compiler, commit, affinity, iterations, distribution
//! Run: cargo run -p nros-core --bin bench -- --iterations 100000 --output benchmarks/results.json
//! This separates correctness (cargo test) from performance (cargo bench / artifact)

use nros_core::{channel, Twist, Vector3, Timestamp, PerformanceStats};
use std::sync::{Arc, atomic::Ordering};
use std::thread;
use std::time::{Duration, Instant};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct BenchmarkEnv {
    cpu_model: String,
    os: String,
    kernel: String,
    rustc_version: String,
    commit: String,
    timestamp: String,
    capacity: usize,
    iterations: usize,
    affinity: String,
    message_size: usize,
}

#[derive(Debug)]
struct LatencyDistribution {
    min_us: f64,
    p50_us: f64,
    p95_us: f64,
    p99_us: f64,
    p999_us: f64,
    max_us: f64,
    mean_us: f64,
    stddev_us: f64,
}

#[derive(Debug)]
struct BenchmarkResult {
    env: BenchmarkEnv,
    throughput_msg_per_sec: f64,
    latency: LatencyDistribution,
    total_time_ms: f64,
    messages_sent: usize,
    messages_received: usize,
}

fn get_cpu_model() -> String {
    #[cfg(target_os = "linux")]
    {
        fs::read_to_string("/proc/cpuinfo")
            .ok()
            .and_then(|s| {
                s.lines().find(|l| l.starts_with("model name")).map(|l| {
                    l.split(':').nth(1).unwrap_or("unknown").trim().to_string()
                })
            })
            .unwrap_or_else(|| "unknown".to_string())
    }
    #[cfg(not(target_os = "linux"))]
    {
        "unknown (non-linux)".to_string()
    }
}

fn get_os_info() -> (String, String) {
    let os = std::env::consts::OS.to_string();
    let kernel = fs::read_to_string("/proc/version")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    (os, kernel)
}

fn get_rustc_version() -> String {
    // Try to run rustc --version
    std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "rustc unknown (cargo not in PATH)".to_string())
        .trim()
        .to_string()
}

fn get_commit() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown (not git repo or git not available)".to_string())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut iterations = 100_000;
    let mut output = PathBuf::from("benchmarks/results.json");
    let mut capacity = 1024;

    // Simple arg parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--iterations" | "-n" => {
                if i + 1 < args.len() {
                    iterations = args[i + 1].parse().unwrap_or(iterations);
                    i += 1;
                }
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--capacity" => {
                if i + 1 < args.len() {
                    capacity = args[i + 1].parse().unwrap_or(capacity);
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    println!("NROS Core Benchmark Artifact Generator");
    println!("Iterations: {}, Capacity: {}, Output: {}", iterations, capacity, output.display());
    println!("Using monotonic clock (Instant) per AUDIT CORE-007 fix");

    let (os, kernel) = get_os_info();
    let env = BenchmarkEnv {
        cpu_model: get_cpu_model(),
        os,
        kernel,
        rustc_version: get_rustc_version(),
        commit: get_commit(),
        timestamp: chrono_like_timestamp(),
        capacity,
        iterations,
        affinity: "not pinned (would need core_affinity crate for CPU pinning)".to_string(),
        message_size: std::mem::size_of::<Twist>(),
    };

    println!("Environment: {:#?}", env);

    // Run benchmark with guard-based API (Safety Gate v0.1)
    // Real end-to-end latency measurement via MonotonicTimestamp embedded in publish path
    // Producer stores publish Instant in shared queue, consumer pops and computes elapsed

    use std::collections::VecDeque;

    // Pass 24: use the type-enforced `channel()` API instead of the deprecated raw-ring
    // Publisher/Subscriber pair (which exposed Arc<RingBuffer> and weakened SPSC).
    let (publisher, subscriber) = channel::<Twist>(capacity);

    // Shared queue of publish Instants for true end-to-end latency measurement (fixes CORE-012)
    let publish_queue: Arc<std::sync::Mutex<VecDeque<Instant>>> = Arc::new(std::sync::Mutex::new(VecDeque::with_capacity(iterations)));
    let publish_queue_clone = publish_queue.clone();

    let latencies = Arc::new(std::sync::Mutex::new(Vec::with_capacity(iterations)));
    let latencies_clone = latencies.clone();

    let consumer = thread::spawn(move || {
        let mut local_latencies = Vec::with_capacity(iterations);

        loop {
            if let Some(_guard) = subscriber.try_recv() {
                // Real latency: now - publish Instant from queue
                let now = Instant::now();
                let publish_instant = {
                    let mut q = publish_queue_clone.lock().unwrap();
                    q.pop_front()
                };
                if let Some(pub_instant) = publish_instant {
                    let latency_ns = now.duration_since(pub_instant).as_nanos() as u64;
                    local_latencies.push(latency_ns);
                }

                if local_latencies.len() >= iterations {
                    break;
                }
            } else {
                thread::yield_now();
            }
        }

        *latencies_clone.lock().unwrap() = local_latencies;
    });

    let start = Instant::now();

    for _ in 0..iterations {
        let publish_time = Instant::now();
        loop {
            if let Some(guard) = publisher.allocate() {
                let twist = Twist { timestamp: Timestamp::now(), linear: Vector3 { x: 1.0, y: 0.0, z: 0.0 }, angular: Vector3 { x: 0.0, y: 0.0, z: 0.5 } };
                guard.write_value(twist).commit();
                // Store publish Instant for latency measurement (must be after commit to measure queue + transport)
                // Actually store before commit for more accurate: publish_time is before commit, but we want to measure time from publish call
                // For simplicity, push publish_time into queue
                publish_queue.lock().unwrap().push_back(publish_time);
                break;
            } else {
                thread::yield_now();
            }
        }
    }

    consumer.join().unwrap();
    let elapsed = start.elapsed();

    let latencies_vec = latencies.lock().unwrap().clone();

    // Compute distribution
    let mut sorted = latencies_vec.clone();
    sorted.sort_unstable();

    let mean = if sorted.is_empty() { 0.0 } else { sorted.iter().sum::<u64>() as f64 / sorted.len() as f64 / 1000.0 };
    let min = sorted.first().map(|v| *v as f64 / 1000.0).unwrap_or(0.0);
    let max = sorted.last().map(|v| *v as f64 / 1000.0).unwrap_or(0.0);
    let p50 = percentile(&sorted, 50.0);
    let p95 = percentile(&sorted, 95.0);
    let p99 = percentile(&sorted, 99.0);
    let p999 = percentile(&sorted, 99.9);

    let variance = if sorted.is_empty() { 0.0 } else {
        let mean_ns = sorted.iter().sum::<u64>() as f64 / sorted.len() as f64;
        sorted.iter().map(|v| (*v as f64 - mean_ns).powi(2)).sum::<f64>() / sorted.len() as f64
    };
    let stddev = variance.sqrt() / 1000.0;

    let distribution = LatencyDistribution {
        min_us: min,
        p50_us: p50,
        p95_us: p95,
        p99_us: p99,
        p999_us: p999,
        max_us: max,
        mean_us: mean,
        stddev_us: stddev,
    };

    let throughput = iterations as f64 / elapsed.as_secs_f64();

    let result = BenchmarkResult {
        env,
        throughput_msg_per_sec: throughput,
        latency: distribution,
        total_time_ms: elapsed.as_secs_f64() * 1000.0,
        messages_sent: iterations,
        messages_received: sorted.len(),
    };

    println!("\n=== Benchmark Result ===");
    println!("Throughput: {:.0} msg/s", result.throughput_msg_per_sec);
    println!("Latency mean: {:.2} μs", result.latency.mean_us);
    println!("Latency p50: {:.2} μs", result.latency.p50_us);
    println!("Latency p99: {:.2} μs", result.latency.p99_us);
    println!("Latency max: {:.2} μs", result.latency.max_us);
    println!("Total time: {:.2} ms", result.total_time_ms);

    // Write JSON artifact
    // Note: serde_json not in dependencies, so we manually write simple JSON-like output
    // For real artifact, would use serde_json crate
    let json_output = format!(
        r#"{{
  "env": {{
    "cpu_model": "{}",
    "os": "{}",
    "kernel": "{}",
    "rustc_version": "{}",
    "commit": "{}",
    "timestamp": "{}",
    "capacity": {},
    "iterations": {},
    "affinity": "{}",
    "message_size": {}
  }},
  "throughput_msg_per_sec": {:.2},
  "latency": {{
    "min_us": {:.2},
    "p50_us": {:.2},
    "p95_us": {:.2},
    "p99_us": {:.2},
    "p999_us": {:.2},
    "max_us": {:.2},
    "mean_us": {:.2},
    "stddev_us": {:.2}
  }},
  "total_time_ms": {:.2},
  "messages_sent": {},
  "messages_received": {}
}}
"#,
        result.env.cpu_model.replace('"', "'"),
        result.env.os,
        result.env.kernel.replace('"', "'").replace('\n', " "),
        result.env.rustc_version.replace('"', "'"),
        result.env.commit,
        result.env.timestamp,
        result.env.capacity,
        result.env.iterations,
        result.env.affinity,
        result.env.message_size,
        result.throughput_msg_per_sec,
        result.latency.min_us,
        result.latency.p50_us,
        result.latency.p95_us,
        result.latency.p99_us,
        result.latency.p999_us,
        result.latency.max_us,
        result.latency.mean_us,
        result.latency.stddev_us,
        result.total_time_ms,
        result.messages_sent,
        result.messages_received
    );

    if let Some(parent) = output.parent() {
        let _ = fs::create_dir_all(parent);
    }

    fs::write(&output, json_output).expect("Failed to write benchmark artifact");
    println!("\n✅ Benchmark artifact written to {}", output.display());
    println!("   This artifact should be committed and referenced in COMPARISON.md for reproducibility per AUDIT Pass 7 §12");
    println!("   Required fields: CPU model, OS, kernel, rustc version, commit, timestamp, capacity, iterations, affinity, message_size, latency distribution p50/p95/p99/p99.9/max/mean/stddev");
}

fn percentile(sorted: &[u64], perc: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((perc / 100.0) * sorted.len() as f64).floor() as usize;
    let idx = idx.min(sorted.len() - 1);
    sorted[idx] as f64 / 1000.0
}

fn chrono_like_timestamp() -> String {
    // Simple timestamp without chrono crate
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    format!("{}", now.as_secs())
}

// We need serde for Serialize derive, but we don't have serde dependency
// To avoid adding dependency, we implement manual Serialize via Debug? 
// Actually we have serde::Serialize derive but crate doesn't depend on serde
// Let's make it compile without serde by removing derive and using manual JSON as above
// The structs above have #[derive(serde::Serialize)] which will fail without serde
// So we remove serde dependency and keep manual JSON
// For compilation, we need to remove serde derives

mod serde {
    pub trait Serialize {}
}
