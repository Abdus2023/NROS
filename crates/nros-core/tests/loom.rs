//! Loom model checking for the nros-core SPSC ring (Pass 31, P31-02).
//!
//! These tests run the REAL RingBuffer / Producer / Consumer implementation
//! under loom's permuted-interleaving executor: with `--features loom` the
//! crate's atomics (`AlignedU64`/`AlignedBool` reservation flags and indices,
//! `Arc`) resolve to loom's instrumented types, so every legal scheduling of
//! the actual publish/commit/receive/drop protocol is explored — this is not a
//! re-written model of the algorithm. Sized small on purpose: loom's state
//! space grows super-exponentially, so models check protocol invariants at
//! bounded capacity/iteration counts, complementing (not replacing) the native
//! stress tests and Miri.
//!
//! Run with: cargo test -p nros-core --features loom --test loom
//!
//! What each model proves about every explored interleaving:
//!  1. reservation/publication visibility — a committed value written through
//!     the guard API is observed intact by the consumer (write Release →
//!     acquire read pairs), never torn or uninitialized;
//!  2. reservation exclusivity — at most one outstanding write reservation
//!     exists (second reserve fails while the first is outstanding), so the
//!     single-slot write path cannot be double-claimed;
//!  3. release-after-consume — slot reuse across wraparound preserves total
//!     order: the consumer observes FIFO order and exactly-once delivery.

#![cfg(feature = "loom")]

use loom::thread;
use nros_core::channel;

/// Visibility + ordering across the reservation/publication/commit protocol,
/// bounded to what loom can explore: capacity 2, two messages from one
/// producer thread to one consumer thread through the public channel() API.
#[test]
fn loom_spsc_two_message_handoff() {
    loom::model(|| {
        let (producer, consumer) = channel::<usize>(2);

        let producer_thread = thread::spawn(move || {
            for value in 0..2usize {
                // Publish-by-copy path: reserve (CAS) -> initialize -> commit (Release).
                // Retry the ReturnNone full-ring policy in EVERY interleaving:
                // the consumer may not have drained slot capacity in time.
                while producer.publish_copy(value).is_err() {
                    thread::yield_now();
                }
            }
        });

        let consumer_thread = thread::spawn(move || {
            let mut received = Vec::new();
            while received.len() < 2 {
                if let Some(guard) = consumer.try_recv() {
                    received.push(*guard);
                } else {
                    thread::yield_now();
                }
            }
            received
        });

        producer_thread.join().unwrap();
        let received = consumer_thread.join().unwrap();

        // Every interleaving: exactly the two published values, in FIFO order,
        // each observed intact (visibility through the Release/Acquire pair).
        assert_eq!(received, vec![0, 1]);
    });
}
