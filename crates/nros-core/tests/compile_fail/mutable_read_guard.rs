// Negative compile test: mutable ReadGuard should fail (consumer should not mutate published message)
// Per AUDIT Pass 18 CORE-015 — ReadGuard exposes DerefMut, should be removed

use nros_core::channel;

fn main() {
    let (producer, consumer) = channel::<u64>(4);
    producer.publish_copy(42).unwrap();
    let mut guard = consumer.try_recv().unwrap();
    // Try to mutate via DerefMut — should fail to compile after fix (only Deref, not DerefMut)
    *guard = 100; // This should fail if DerefMut removed
}
