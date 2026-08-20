// Negative compile test: two Producers from one SPSC channel should fail (SPSC role enforcement)
// Per AUDIT Pass 16 §28 — NROS should test things that must NOT compile

use nros_core::channel;

fn main() {
    let (producer, _consumer) = channel::<u64>(4);
    let producer2 = producer; // Move producer
    // Try to clone producer — should fail because Producer is not Clone
    let _producer_clone = producer2.clone(); // This should fail to compile: no Clone impl
}
