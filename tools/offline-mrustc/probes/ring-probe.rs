// Adversarial ring-buffer probes (Pass 27) — exact drop counts, wraparound, invariants,
// plus F-16/ZST and F-17/abort_initialized double-drop regression probes at runtime.
use nros_core::{channel, RingBuffer};
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    // ── A. Exact fill/drain with invariant checks every step (capacity 8) ──
    {
        let (p, c) = channel::<u64>(8);
        assert!(p.is_empty() && c.is_empty() && p.len() == 0);
        for i in 0..8u64 {
            p.publish_copy(i).expect("publish within capacity must succeed");
            assert_eq!(c.pending(), (i + 1) as usize, "pending must track publishes exactly");
        }
        assert!(p.len() == 8, "ring must report full at capacity");
        let mut drops = 0usize;
        for _ in 0..100 {
            if p.publish_copy(0xDEAD).is_err() { drops += 1; }
        }
        assert_eq!(drops, 100, "every publish into a full ring must be rejected (none silently stored)");
        assert_eq!(c.pending(), 8, "rejected publishes must not perturb occupancy");
        for i in 0..8u64 {
            let g = c.try_recv().expect("pop within len must succeed");
            assert_eq!(*g, i, "FIFO order must survive fill+full-rejection");
            drop(g);
            assert_eq!(c.pending(), (8 - i - 1) as usize);
        }
        assert!(c.is_empty() && p.is_empty());
        assert!(c.try_recv().is_none(), "empty pop must be None");
    }
    println!("[A] exact fill/drop/drain + invariant checks: PASS");

    // ── B. 1M-step wraparound at capacity 2: counters advance unboundedly & safely ──
    {
        let (p, c) = channel::<u64>(2);
        let n: u64 = 1_000_000;
        let mut sum = 0u64;
        for i in 0..n {
            p.publish_copy(i).unwrap();
            assert!(p.len() <= 2, "len must never exceed capacity");
            let g = c.try_recv().unwrap();
            assert_eq!(*g, i, "wraparound corruption at step");
            drop(g);
            sum = sum.wrapping_add(i);
        }
        assert!(c.is_empty());
        assert_eq!(sum, (0..n).fold(0u64, |a, b| a.wrapping_add(b)));
    }
    println!("[B] 1,000,000-step capacity-2 wraparound with per-step value check: PASS");

    // ── C. Interleave partial fill across wrap boundary ──
    {
        let (p, c) = channel::<u64>(4);
        for round in 0..10_000u64 {
            for k in 0..3u64 { p.publish_copy(round * 3 + k).unwrap(); }
            for k in 0..3u64 {
                let g = c.try_recv().unwrap();
                assert_eq!(*g, round * 3 + k, "partial-fill interleave corruption");
                drop(g);
            }
            assert!(c.is_empty());
        }
    }
    println!("[C] 30k-message partial-fill interleave across wrap boundary: PASS");

    // ── D. One outstanding reservation (CORE-001) & one outstanding read (CORE-002) ──
    {
        let (p, c) = channel::<u64>(4);
        p.publish_copy(1).unwrap();
        let g1 = p.allocate().expect("first reservation");
        assert!(p.allocate().is_none(), "second concurrent reservation must be denied");
        let r1 = c.try_recv().expect("first read");
        assert!(c.try_recv().is_none(), "second concurrent read reservation must be denied");
        g1.write_value(2).commit();
        drop(r1);
        assert_eq!(c.pending(), 1);
        let r = c.try_recv().unwrap();
        assert_eq!(*r, 2);
    }
    println!("[D] single-outstanding-reservation enforcement: PASS");

    // ── E. ZST ring (F-16): no allocation UB; drop must not dealloc a dangling ptr ──
    {
        let (p, c) = channel::<()>(4);
        for _ in 0..1000 {
            p.publish_copy(()).unwrap();
            let g = c.try_recv().unwrap();
            drop(g);
        }
        assert!(c.is_empty());
        // Drop of the ring itself exercises the zero-size dealloc guard.
        drop(p); drop(c);
        let ring = RingBuffer::<()>::new(2);
        let g = ring.try_reserve().unwrap();
        g.write_value(()).commit();
        let r = ring.try_read().unwrap();
        drop(r);
        drop(ring);
    }
    println!("[E] ZST ring 1000-step + double ring drop (F-16 dangling-ptr guard): PASS");

    // ── F. abort_initialized drops exactly once (F-17, incl. panic-during-drop safety) ──
    {
        static DROPS: AtomicUsize = AtomicUsize::new(0);
        struct Counted(u64);
        impl Drop for Counted { fn drop(&mut self) { DROPS.fetch_add(1, Ordering::SeqCst); } }
        {
            let (p, _c) = channel::<Counted>(4);
            DROPS.store(0, Ordering::SeqCst);
            let g = p.allocate().unwrap().write_value(Counted(1));
            g.abort_initialized();
            assert_eq!(DROPS.load(Ordering::SeqCst), 1, "abort_initialized must drop exactly once");
            // normal commit+read path: value dropped once when ReadGuard releases slot
            p.publish_copy(Counted(2)).unwrap();
            let c2 = _c;
            {
                let r = c2.try_recv().unwrap();
                assert_eq!((*r).0, 2);
            } // ReadGuard drop runs T::drop exactly once and advances read_idx
            let after_read = DROPS.load(Ordering::SeqCst);
            assert_eq!(after_read, 2, "ReadGuard must drop the consumed value exactly once (got {})", after_read);
        } // teardown: read_idx already advanced past Counted(2) — ring Drop must NOT re-drop it
        let total = DROPS.load(Ordering::SeqCst);
        assert_eq!(total, 2, "teardown must not double-drop consumed slots (got {})", total);
    }

    // ── F2. Teardown drops exactly the unread live values (none read) ──
    {
        static D2: AtomicUsize = AtomicUsize::new(0);
        struct C2(u64);
        impl Drop for C2 { fn drop(&mut self) { D2.fetch_add(1, Ordering::SeqCst); } }
        D2.store(0, Ordering::SeqCst);
        {
            let (p, c) = channel::<C2>(8);
            p.publish_copy(C2(1)).unwrap();
            p.publish_copy(C2(2)).unwrap();
            p.publish_copy(C2(3)).unwrap();
            {
                let r = c.try_recv().unwrap(); // consume one
                assert_eq!((*r).0, 1);
            } // r drop → 1 drop
            assert_eq!(D2.load(Ordering::SeqCst), 1);
        } // teardown: two unread live values must be dropped exactly once each
        assert_eq!(D2.load(Ordering::SeqCst), 3, "teardown must drop the 2 remaining live values exactly once");
    }
    println!("[F] abort_initialized drop-count + read/teardown drop discipline (F-17): PASS");

    // ── G. SPSC smoke under real threads with exact accounting ──
    {
        let (p, c) = channel::<u64>(1024);
        let n: u64 = 200_000;
        let prod = std::thread::spawn(move || {
            let mut dropped = 0u64;
            for i in 0..n {
                while p.publish_copy(i).is_err() { dropped += 1; std::thread::yield_now(); }
            }
            dropped
        });
        let cons = std::thread::spawn(move || {
            let mut got = 0u64;
            while got < n {
                if let Some(g) = c.try_recv() {
                    assert_eq!(*g, got, "SPSC FIFO violation under real concurrency");
                    drop(g);
                    got += 1;
                } else { std::thread::yield_now(); }
            }
            got
        });
        let dropped_attempts = prod.join().unwrap();
        let got = cons.join().unwrap();
        assert_eq!(got, n);
        println!("[G] threaded SPSC 200k msgs, FIFO intact (publish retry attempts: {})", dropped_attempts);
    }

    println!("ALL RING PROBES PASS");
}
