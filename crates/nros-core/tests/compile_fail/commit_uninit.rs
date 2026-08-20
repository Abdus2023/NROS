// Negative compile test: commit uninitialized WriteGuard should fail
// Per AUDIT Pass 16 §28 and Pass 8-12 CORE-014 — commit() does not require initialization
// After type-state fix, WriteGuard (Uninit) should NOT have commit() method, only InitializedWriteGuard should

use nros_core::RingBuffer;

fn main() {
    let ring = RingBuffer::<u64>::new(4);
    let guard = ring.try_reserve().unwrap();
    // Try to commit without initialization — should fail to compile
    // After fix, WriteGuard has no commit() method, only InitializedWriteGuard does
    guard.commit(); // This should fail to compile
}
