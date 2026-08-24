# NROS — Deep Analysis & Verification — Pass 31 (SPSC Ownership Closure, Loom Wiring, Claim Ledger)

Branch: `arena/01a03356-nros`
Parent: Pass 30 (F30-01 Miri unblock, `2d4ea5b90` / evidence recording `5599c78a5`)
Date: 2026-08-24

This pass continues verification over adding new specification. It does three
bounded things ordered by the prior audits' finding lists: **P31-01** closes
the SPSC raw-ring escape hatch (priority P1 since Pass 20-23), **P31-02**
wires Loom concurrency-model checking of the SPSC protocol, and **P31-03**
creates the per-claim ledger and makes it a hard gate.

## P31-01 — SPSC raw-ring escape hatch closed (REMOVED, not deprecated)

### Finding

`verification.json` `B_Ownership.no_public_raw_ring_sharing` recorded
**PARTIAL** at Pass 29: `Producer`/`Consumer` enforce single-producer/
single-consumer by construction, but the deprecated compatibility API —
`Publisher::from_ring()`, `Publisher::ring() -> Arc<RingBuffer<T>>`, and
`Subscriber::new(Arc<RingBuffer<T>>, topic)` — still handed the raw `Arc` to
arbitrary safe code. Pass 24 already established the principle: *`#[deprecated]`
on a safe API does not close a capability; safe Rust must never be able to
reach the weakened path.* While those methods existed, the type-enforcement
argument applied only to callers who chose `channel()`.

### Fix

The three raw-ring constructors/accessors are **deleted**.

- `Publisher<T>` now holds a `Producer<T>`; `Subscriber<T>` holds a `Consumer<T>`.
- Sole construction path: `Publisher::declare(topic, capacity) -> (Publisher, Subscriber)`,
  which delegates to `channel()` and returns the matched pair.
- All method surfaces (`allocate`, `publish_copy`, `try_recv`, `pending`,
  `topic`, `len`, `is_empty`) delegate to the type-enforced endpoints — neither
  endpoint is `Clone`, so SPSC exclusivity is enforced by the type system on
  every path, including the pub/sub facade.
- `RingBuffer` intentionally **stays public** as the documented low-level
  building block: the soundness argument does not depend on hiding it (the
  CAS reservation flags serialize the write/read sides regardless of how many
  `&RingBuffer` handles exist; the type-state guards make published values
  always initialized). The `trybuild` negative-compile suite — which proves
  commit-without-init, safe `init_with`, mutable `ReadGuard`, and producer
  cloning all fail to compile — targets `RingBuffer`'s public API, so
  privatizing it would have destroyed that evidence without adding safety.

Callers migrated: `crates/nros-core/src/main.rs` Demo 1 (previously exercised
the deprecated API deliberately) now uses `Publisher::declare`. No other
workspace code referenced the removed items (facade re-exports only names;
examples use the types in struct fields only; `nros init` golden templates are
self-contained).

## P31-02 — Loom wired for the SPSC protocol (NOT YET EXECUTED)

### What was built

- `crates/nros-core/src/lib.rs`: synchronization primitives are now selected
  by cfg — with the optional `loom` feature enabled, the ring's
  `AtomicU64`/`AtomicBool` reservation flags and indices and `Arc` resolve to
  `loom::sync::...` (`loom = "0.7"` optional dependency). Default builds use
  `std` types: native tests, Miri and the offline mrustc pipeline are
  bit-identical to before.
- `crates/nros-core/tests/loom.rs`: three `loom::model` models that run the
  REAL `channel()`/`Producer`/`Consumer`/guard API — not a re-written spec
  model — under permuted interleavings:
  1. `loom_spsc_two_message_handoff` — capacity 2: FIFO visibility through the
     write-Release / read-Acquire chain in every explored schedule;
  2. `loom_spsc_guard_protocol_wraparound` — capacity 1 (maximal slot reuse),
     read guard deliberately held across a scheduling point: exactly-once FIFO
     delivery, no two owners of one slot;
  3. `loom_spsc_reservation_commit_visibility` — second reserve fails while a
     reservation is outstanding (CAS exclusivity), an uncommitted reservation
     is never observable as a message, the committed value is observed after
     commit.

  Assertions are restricted to facts true in **every** interleaving (loom
  enumerates schedules; a schedule-specific assertion would fail spuriously).
  Interpreter note: loom verifies the synchronization protocol (atomic
  orderings); it does not instrument raw-pointer data access the way Miri
  does — the two tools are complementary, per SAFETY.md.

### What is NOT done (and why)

Loom has not been *executed*: running it needs the `loom` crate from crates.io
(unavailable in the offline audit environment) or a CI job. The CI job is
staged as `docs/audit/P31-02-ci-loom-job.patch` — the app token lacks the
`workflows` scope to push `.github/workflows/ci.yml` changes (the same
limitation that previously kept F-20/F-25/F-29 as staged patches). Status is
honestly `WIRED_NOT_RUN` in `docs/audit/verification.json`; claiming loom as
green before an executed run would violate EVID-002/EVID-006.

## P31-03 — Claim ledger created and made a hard gate

- `docs/CLAIM_LEDGER.md` created: every claim in
  `docs/representation/claims.yaml` (CLAIM-IPC-001 ... CLAIM-MIRI-001) is bound
  to source, implementation, executed evidence (run IDs resolve per EVID-006),
  environment, policy class and the exact permitted wording. Includes
  maintenance rules (class strengthening requires recording the new executed
  run first; regressions demote in the same commit).
- `nros-audit` claim gate extended with **DOC-006** (hard failure): the doc
  gate now fails if `docs/CLAIM_LEDGER.md` is missing or does not bind claims
  to allowed wording. This executes in the existing doc-gate CI job with no
  workflow change.

## Evidence updates

- `docs/audit/verification.json`: `B_Ownership.no_public_raw_ring_sharing`
  PARTIAL → CLOSED; `loom` NOT_RUN → WIRED_NOT_RUN (with command + model
  details); `claim_ledger` TODO → IMPLEMENTED.
- `docs/representation/evidence.yaml`: loom marker updated.
- `crates/nros-core/SAFETY.md`: status header and outstanding-items section.
- `EVIDENCE_REGISTRY.md`: SPSC row records Miri PASS and the Pass 31
  ownership/loom dispositions.

## What Pass 31 deliberately did NOT do

- No new architecture/specification documents.
- No clippy `-D warnings` flip (P2; needs a warning-clean baseline first).
- No workflow edits (token lacks the scope — staged as patch instead).
- No hardware/DMA work (still simulated; remains open).
- No weakening of any existing gate; loom's eventual CI job is a hard gate by
  design in the staged patch.
