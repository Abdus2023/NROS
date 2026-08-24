// Same-thread SPSC throughput probe (Pass 27) — deterministic, no threads:
// measures publish+recv round at fixed capacity, reports ns/op and msg/s.
// (Threaded variant is dominated by sandbox scheduler jitter; this is the stable probe.
//  The CANONICAL committed benchmark artifact remains benchmarks/results_*.json,
//  produced by crates/nros-core/src/bin/bench.rs.)
use nros_core::channel;
use std::time::Instant;

#[derive(Clone, Copy)]
struct Msg64 { seq: u64, payload: [u8; 56] } // 64-byte message, matches benchmarks/ results sizing

fn main() {
    let n: u64 = 2_000_000;
    let (p, c) = channel::<Msg64>(1024);
    let t0 = Instant::now();
    for i in 0..n {
        p.publish_copy(Msg64 { seq: i, payload: [0u8; 56] }).unwrap();
        let g = c.try_recv().unwrap();
        std::hint::black_box(g.seq);
        drop(g);
    }
    let dt = t0.elapsed();
    let ns = dt.as_nanos() as f64 / n as f64;
    let mps = n as f64 / dt.as_secs_f64();
    println!("same-thread SPSC: {} msgs in {:?} -> {:.2} ns/op, {:.0} msg/s (64B payload, cap 1024)", n, dt, ns, mps);

    // burst shape: fill capacity fully, drain fully (throughput under full-ring pressure)
    let t1 = Instant::now();
    let bursts = 20_000;
    for _ in 0..bursts {
        for i in 0..1024u64 {
            p.publish_copy(Msg64 { seq: i, payload: [0u8; 56] }).unwrap();
        }
        for _ in 0..1024 {
            let g = c.try_recv().unwrap();
            std::hint::black_box(g.seq);
            drop(g);
        }
    }
    let dt1 = t1.elapsed();
    let total = bursts as f64 * 1024.0;
    println!("burst SPSC: {:.0} msgs in {:?} -> {:.2} ns/op, {:.0} msg/s", total, dt1, dt1.as_nanos() as f64 / total, total / dt1.as_secs_f64());
    println!("MICROBENCH OK");
}
