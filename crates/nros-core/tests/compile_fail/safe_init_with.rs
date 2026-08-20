// Negative compile test: the legacy *safe* `init_with()` must no longer exist.
// Pass 24 remediation (CORE-011/CORE-014 follow-up): a safe closure-based initializer let
// safe Rust produce an `InitializedWriteGuard` without proving the slot was initialized
// (closure that does nothing -> commit -> consumer derefs uninitialized memory = UB).
// The safe method was removed. Field-by-field init now requires `unsafe init_with_unchecked()`.

use nros_core::RingBuffer;

fn main() {
    let ring = RingBuffer::<u64>::new(4);
    let guard = ring.try_reserve().unwrap();
    // This must fail to compile: no safe `init_with` on WriteGuard.
    let _init = guard.init_with(|slot| {
        // deliberately does nothing — would have been UB under the old safe API
        let _ = slot;
    });
}
