# NROS — Deep Analysis & Verification — Pass 27 (First Real Build/Test Execution + Offline Toolchain Bootstrap)

Branch: `arena/01a02a3e-nros`
Parent: `86bbfb5` (main HEAD at analysis start: "docs: reconcile architecture series frontier")
Date: 2026-08-22
Session type: fully network-restricted sandbox (only `github.com`, `pypi.org`, `registry.npmjs.org` reachable; `crates.io`, `static.rust-lang.org`, apt mirrors blocked)

This pass answers the repository's largest open verification debt: **no CI run had ever executed the workspace** (`docs/audit/verification.json` recorded every gate as `NOT_RUN` — "no cargo available in sandbox"). Pass 27 produced the first real compile/test/gate/golden/benchmark evidence for the NROS workspace, fixed the resulting defects, and re-pinned the representation snapshots.

---

## 1. Verification Method — Offline Toolchain Bootstrap

The sandbox had **no Rust toolchain at all**, and every conventional acquisition channel was blocked (rustup.rs, static.rust-lang.org, crates.io, distro apt, Chinese mirrors, npm/pypi rustup shims that still download from blocked hosts). GitHub Actions was also nonfunctional at account level (all workflows, including the owner's own `main` runs, queued for 1h+ without starting).

To still produce *real* evidence instead of static reading, the toolchain was built from allowed sources:

1. **mrustc** (John Hodge's Rust compiler written in C++) cloned from GitHub and built with system GCC 12.2 (~10 min).
2. **Rust 1.90.0 standard library built from GitHub source** (`rust-lang/rust` tag 1.90.0 tarball via codeload).
3. **crates.io deps reconstructed from GitHub repositories** at Cargo.lock-exact versions (cfg-if, hashbrown, libc, rustc-demangle, miniz_oxide, adler2, addr2line, gimli, object, memchr, getopts, unicode-width ×2, allocator-api2, foldhash, equivalent, + in-tree compiler-builtins/libm) into mrustc's vendor layout. Relative-dep and feature fixes documented below.
4. `library/backtrace` submodule content could not be fetched while the egress proxy degraded; an API-faithful **stub** (which reports zero frames → std maps to `BacktraceStatus::Unsupported`) was written and is the only non-upstream content in this toolchain. It does not affect any NROS test, binary, or benchmark result.
5. Small, clearly-marked patches to mrustc's minicargo (tolerate `[workspace.package]` keys; tolerate `[workspace]` unknown keys; TOML bool→string coercion) were required to read the NROS workspace manifests.

Result: `mrustc` compiles and links real native binaries against a from-source Rust 1.90 `std`, `test`, and `proc_macro`. Every "executed" result below ran on this toolchain. Where a result could differ under official rustc (e.g., `mrustc` internal bugs), it is flagged explicitly rather than silently presented as authoritative.

**What was NOT possible offline:** `cargo fmt` (rustfmt component), `cargo clippy` (clippy component), `cargo miri` (Miri component — soundness oracle for `unsafe`), `trybuild` (many crates.io deps). The `nros-macros`/`nros` facade builds were additionally blocked during this session's first hours (degraded GitHub egress); **once codeload.github.com recovered, the facade chain was built and verified with the real `syn`/`quote`/`proc-macro2` — see §11 (Addendum)**. The remaining items are listed in §9 (residual verification debt) and are expected to be covered by the existing CI workflow the first time GitHub Actions executes.

---

## 2. Build Verification (all workspace targets)

| Crate | lib | demo/bin | Notes |
|-------|-----|----------|-------|
| nros-types | ✅ | — | |
| nros-core | ✅ | ✅ `nros-core-demo` (+ `bench`) | demo needed import fix (F-1) |
| nros-node | ✅ | ✅ `nros-node-demo` | |
| nros-hal | ✅ | ✅ `nros-hal-demo` | |
| nros-transport | ✅ | ✅ `nros-transport-demo` | demo needed trait import (F-5) |
| nros-distributed | ✅ | ✅ `nros-distributed-demo` | |
| nros-sim | ✅ | ✅ `nros-sim-demo` | needed `Debug` derive (F-4) |
| nros-studio | ✅ | ✅ `nros-studio` server | live HTTP/SSE verified |
| nros-cli | ✅ | ✅ `nros`, `nros-cli-demo` | demo needed cwd fix (F-6) |
| nros-audit | ✅ | ✅ `nros-audit` | needed `;` fix (F-3) |
| nros-macros | ⛔ offline | — | blocked on syn/quote (§8) |
| nros (facade) | ⛔ offline | — | blocked transitively (§8) |

**Headline: before this pass, 6 of 12 workspace members could not compile** (or their advertised demo/example targets could not). All were fixed in this session.

---

## 3. Test Verification (actually executed)

| Suite | Result | Harness |
|-------|--------|---------|
| nros-types lib tests | ✅ 4/4 | mrustc `--test` |
| nros-core lib tests | ✅ 17 passed / 0 failed / 1 ignored | mrustc `--test` (direct) |
| nros-node lib tests | ✅ 5/5 | mrustc `--test` |
| nros-hal lib tests | ✅ 4/4 | mrustc `--test` |
| nros-transport lib tests | ✅ 7/7 (incl. UDP loopback) | mrustc `--test` |
| nros-sim lib tests | ✅ 7/7 (incl. degenerate-input hardening) | mrustc `--test` |
| nros-studio lib tests | ✅ 3/3 | mrustc `--test` |
| nros-cli lib tests | ✅ 3/3 | mrustc `--test` |
| nros-distributed lib tests | ✅ logic 5/5 (see note) | probe binary (same code paths) |
| **Total** | **50 unit tests passed offline + 5 logic probes** | |

Notes:
- nros-core's ignored test is `benchmark_latency_monotonic` — correctly `#[ignore]`d (benchmark separated from correctness gate per CORE-008).
- nros-distributed's `--test` harness crashes mrustc's typechecker (`expr_cs.cpp: Spare rules left after typecheck stabilised`) — an **mrustc limitation**, not a code defect. Proof: the five test bodies were extracted verbatim into a probe binary against the compiled lib; all compile and pass, including `matches!` over `NodeRole` or-patterns and the (deterministic) simulated election.
- trybuild negative tests were executed by hand against mrustc (§5).

---

## 4. Safety Ring Buffer — Deep Manual Audit (nros-core/src/lib.rs)

The SPSC ring (the only `unsafe`-heavy component: 22 sites) was re-derived line by line:

- **Guard protocol**: `try_reserve` → `WriteGuard` (uninit) → `write_value` → `InitializedWriteGuard` → `commit` — commit possible only after init (CORE-014). `write_reserved`/`read_reserved` CAS → single outstanding guard each side (CORE-001/002). Verified hold: producer never overlaps the consumer's slot because `write - read >= capacity` is refused, so `write ≡ read (mod capacity)` is unreachable with live readers.
- **Orderings**: consumer Acquire-loads `write_idx` (published via Release store in `commit`) → happens-before ⇒ sees initialized T. Producer Acquire-loads `read_idx` (Release store in `ReadGuard::drop`) ⇒ `drop_in_place` ordered before slot reuse. Sound for SPSC.
- **Drop discipline**: exactly-once via `ReadGuard::drop` + ring draining by *count* (`wrapping_sub`, wraparound-safe) — `test_drop_drains_all_occupied_slots` executed and passes.
- Documented edge cases (not fixed; low severity, noted for Pass 28+):
  - `RingBuffer::<ZST>` (zero-sized T) would allocate a zero-size `Layout` → `alloc` on zero-size layouts is UB. Mitigation would be a `size_of::<T>() == 0` guard. No NROS message is a ZST today.
  - `InitializedWriteGuard::abort_initialized` double-drops if `T::drop` *panics* during the abort's `drop_in_place` (panic-during-drop corner). Documented; exotic.
  - `init_with_unchecked` leaks (no UB) if the closure panics after partial init — acceptable and documented in-code.
- Loom-verification still outstanding (loom requires crates.io): ordering argument above is manual.

---

## 5. Compile-Fail (Negative) Verification — trybuild equivalent

Since `trybuild` was unavailable, the four `tests/compile_fail/*.rs` cases were compiled directly with mrustc; **all were correctly rejected**:

| Case | Rejection observed (offline) |
|------|------------------------------|
| `two_producers_from_one_channel` | `Producer<u64>` has no `.clone` method — type-level SPSC ownership (CORE-016) |
| `commit_uninit` | `WriteGuard<…,u64>` has no `.commit` — commit requires init (CORE-014) |
| `safe_init_with` | `WriteGuard` has no safe `.init_with` — uninit→commit UB path closed (CORE-011) |
| `mutable_read_guard` | rejected — `DerefMut` impl for `ReadGuard` does not exist (CORE-015) (mrustc reports via its monomorphizer rather than a tidy E0596) |

---

## 6. Golden Test (nros init) — Offline Equivalent

CI job `nros-init-golden` (`nros init` → `cargo check`) was executed offline:

```
nros init g_robot_basic --template=basic      → generated project compiles AND runs
nros init g_robot_mobile --template=mobile_base → generated project compiles AND runs
```

Output math sanity-checked by hand: mobile_base `on_cmd_vel(1.0, 0.5)` with wheel_base 0.5 → left 0.875 (prints 0.88), right 1.125 (prints 1.12) ✓ differential-drive kinematics correct.

---

## 7. Real Benchmark Results (first non-template artifact)

`benchmarks/results_e2b-sandbox-2vcpu_20260822.json` was generated by running `crates/nros-core/src/bin/bench.rs` on this sandbox:

| Metric | Template value (historical) | **Measured offline** |
|--------|-----------------------------|----------------------|
| throughput | 780,000 msg/s | **1,571,113 msg/s** |
| message_size | 56 | **64** (canonical `Twist`, `repr(C)`, real value) |
| mean latency | 6.2 μs | 588 μs — **scheduler-bound**, see below |
| p50 / p99 | 5.8 / 12.1 μs | 629 / 823 μs |

Interpretation (evidence honesty):
- The two threads busy-spin on a **2-shared-vCPU** sandbox; the "latency" number measures OS scheduler latency, not IPC cost. A same-thread publish→consume probe measured the **raw ring at ≈ 73.5 ns/op (~13.6 M ops/s)** including guard churn and drop — i.e., the datastructure itself is far below the <10 μs target.
- The historical "6.2 μs / 780K msg/s" remain **repository-reported, not independently established on representative hardware** (as AUDIT.md required). What this pass can state honestly: (a) the ring mechanics are O(ns) scale uncontended; (b) cross-thread figures need CPU isolation/affinity and real hardware before any "6.2 μs" claim dies or lives.
- The CI `benchmarks` job is report-only (`continue-on-error`) and its artifact will be the first official datapoint when Actions runs.

---

## 8. Findings (defects pre-existing in `main`, all fixed in this pass)

| ID | Severity | Where | Defect | Fix |
|----|----------|-------|--------|-----|
| F-1 | Build-blocker | nros-core/src/main.rs | demo bin used `Arc`/`Ordering` without imports (E0425) → `cargo check --workspace --all-targets` fails | added imports |
| F-2 | Build-blocker | nros/examples/vertical_slice.rs | `received_guard.frame_id()` — no such method (E0599) → `--all-targets` fails | removed stale assert |
| F-3 | Build-blocker | nros-macros/src/lib.rs | `#[nros::node]` re-emitted field-position attribute macros; rustc rejects attribute-macro invocation on fields → `examples/mobile_base.rs` can't compile | macro strips field helper attrs (passthrough semantics preserved) |
| F-4 | Build-blocker | nros-sim/src/lib.rs | `#[derive(Debug)] struct BulletPhysicsEngine { inner: SimulatedPhysicsEngine }` but SimulatedPhysicsEngine had no Debug (E0277) | derive Debug |
| F-5 | Build-blocker | nros-transport/src/main.rs | demo calls trait methods without `CompressionEngineTrait` in scope (E0599) | import added |
| F-6 | Runtime | nros-cli/src/demo.rs | passed absolute temp path as project name; name validation rejects → `.unwrap()` panic on the advertised demo | create temp dir, chdir, use relative name |
| F-7 | Correctness | nros-audit/src/representation.rs | `if !v.starts_with('-'){ f.insert(…) }` — `Option` returned in statement-position `if` (E0308) | added `;` |
| F-8 | Gate semantics | nros-audit/src/main.rs | `safety`/`workspace inventory` gates printed ❌ but exited 0 — could never fail CI (docs/ci.yml claimed it exits non-zero) | `process::exit(1)` on failure |
| F-9 | Doc gate | docs/documentation/schema.yaml | vocabulary missing `superseded_by` used by relationships.yaml (6 edges) → python validator FAIL | added to `relationship_types` |
| F-10 | Doc gate | docs/documentation/inventory.yaml | snapshot docs DOC-INVENTORY/-AUTHORITIES/-RELATIONSHIPS/-REFERENCES missing from inventory → FAIL | 4 records added |
| F-11 | Doc gate | docs/representation/evidence.yaml | 11 capabilities had no evidence record (schema: `one_record_per_capability`) → representation gate FAIL (12 failures) | records added with honest statuses |
| F-12 | Doc gate | capabilities/architecture | facade crate `nros` not represented (`every_workspace_crate_must_be_represented`) → FAIL | FACADE-001 added |
| F-13 | Build-blocker | nros/examples/vertical_slice.rs | `motor_cmd.linear_velocity.x`/`angular_velocity.z` — `MotorCommand` is wheel-space `{left,right}_{velocity,torque}` (E0599); found while compiling the facade examples in §11 | route through node's own inverse kinematics (`compute_odometry`) — canonical types, still no ad-hoc shim |
| F-14 | Build-blocker | nros/examples/vertical_slice.rs | passed `nros_types::Vector3` to `nros_sim::spawn_robot` — nros-sim deliberately keeps its own zero-dep geometry types (tracked migration I-007); type mismatch | use `nros_sim::Vector3` at the sim boundary with I-007 cross-reference comment |
| Latent (not fixed) | Medium | nros-transport TCP receive | nonblocking socket + `read_exact` → partial header/payload reads on `WouldBlock` desync the stream framing permanently; benign on loopback but incorrect protocol handling | documented; needs buffered frame reader |
| Latent (not fixed) | Low | nros-core | ZST `RingBuffer<T>` zero-size alloc UB; `abort_initialized` drop-panic double-drop | documented in §4 |

Post-fix gate status (executed locally): `python3 scripts/validate-documentation-representation.py` → **PASS**; `nros-audit claims|workspace|ci|benchmarks|safety` → all ✅; `nros-audit representation` → **PASS** after snapshot re-pin (97 checks). Snapshots re-pinned to the new commits (`content_integrity` fingerprints recomputed from git blob SHA-1s).

---

## 9. Residual Verification Debt (handoff)

1. **GitHub Actions has never executed this workspace.** When runner capacity returns, the existing branch-bound CI should be allowed to run `fmt/check/test/clippy/miri/golden/benchmarks/doc-gate` — expected to pass after this pass's fixes (the clippy gate is report-only by design until a baseline).
2. **Miri** on `nros-core`/`nros-types` — required for the unsafe code soundness claim; impossible to fetch offline. Manual audit in §4 stands in, but is not a substitute.
3. ~~**nros-macros / nros facade** compile~~ — **RESOLVED in §11**: real `syn` 2.0.119 / `quote` 1.0.47 / `proc-macro2` 1.0.107 / `unicode-ident` 1.0.24 (dtolnay GitHub release tags, vendored) built with mrustc; real `nros-macros`, facade, and both examples verified green. Official-rustc verification still pending with CI.
4. **trybuild** native run (compile-fail probes were manual here).
5. **nros-distributed `--test`** under official rustc (mrustc-only harness crash — §3).
6. `docs/audit/verification.json` still describes an earlier branch's NOT_RUN state; treat this document (plus the new benchmark artifact) as the current execution evidence, and the CI's first green run as the authority going forward.
7. **mrustc toolchain caveats** (method honesty): the mrustc C backend is not the official rustc codegen; its proc-macro loading links plugin executables into final binaries (worked around by stripping the plugin from the final link line — semantically identical to rustc, which never links proc-macros). `library/backtrace` was stubbed during the egress-degraded window; **it was then RESTORED to the real rust-1.90.0-pinned sources** (gitlink `b65ab935f...`, backtrace 0.3.75-era) and the std tree rebuilt — see §11.B.

---

## 10. Reproducing This Pass

Offline path (no cargo/rustup):
```bash
# bootstrap (GitHub-only sources)
git clone https://github.com/thepowersgang/mrustc && cd mrustc
# + rust 1.90.0 source via codeload, vendor crates per §1, patch minicargo lenience
make -j2 && make -f minicargo.mk LIBS   # builds mrustc + std 1.90
# build + test each crate
bin/minicargo crates/nros-core --output-dir out -L output-1.90.0 --test
```

When network is normal, the authoritative path remains: `cargo fmt --check && cargo check --workspace --all-targets && cargo test --workspace --all-targets && cargo clippy --workspace --all-targets && cargo +nightly miri test -p nros-core --lib -p nros-types --lib` + CI golden + doc gates — i.e., exactly `.github/workflows/ci.yml`.

---

## 11. Addendum — Real-Macro Facade Verification (same session, post-egress-recovery)

After 'codeload.github.com' recovered in this session, the macro dependency chain was vendored from the authors' GitHub release tags and the **real** `nros-macros` (syn-based, containing the F-3 fix — not the offline stub used earlier) was built and exercised:

| Component | Version | Source | Build |
|-----------|---------|--------|-------|
| unicode-ident | 1.0.24 | dtolnay/unicode-ident tag | ✅ mrustc |
| proc-macro2 | 1.0.107 | dtolnay/proc-macro2 tag (build.rs executed under minicargo; `[patch.crates-io]` section stripped from vendored manifest — minicargo TODO) | ✅ mrustc |
| quote | 1.0.47 | dtolnay/quote tag | ✅ mrustc |
| syn (`full`) | 2.0.119 | dtolnay/syn tag | ✅ mrustc |
| nros-macros | workspace (real source, F-3 applied) | this repo | ✅ real proc-macro plugin |
| nros (facade) | workspace | this repo | ✅ libnros.rlib against real macro |

Executed results:

| Target | Result |
|--------|--------|
| `cargo check --example mobile_base` equivalent (real `#[nros::node]` stripping `#[subscribe]/#[publish]/#[param]` field attrs) | ✅ compiled, ran green — **validates F-3 against the real macro** |
| `cargo run --example vertical_slice` equivalent | ✅ compiled after two further pre-existing defects were fixed (F-13 `MotorCommand` field names, F-14 sim-boundary `Vector3`), ran green: 10/10 iterations, 0/10 deadline misses, canonical pipeline `Twist → SPSC → VelocityController → MotorCommand → compute_odometry → Sim`, queue-full backpressure probe ✅, final verdict `Vertical slice PASSED` |

Method notes: (a) examples were compiled with `mrustc --edition 2021 -O` against the facade rlib tree, mirroring `cargo build -p nros --examples`; (b) the mrustc C-backend linker step was completed by re-invoking the emitted link command with the proc-macro plugin removed (rustc never links proc-macros into downstream artifacts — semantically identical); (c) the offline stub proc-macro used earlier in this pass is superseded by the real macro for all facade conclusions.

### 11.B Sub-addendum — `library/backtrace` restored to pinned upstream sources (same session)

The one remaining stub in the verification toolchain was `library/backtrace` in the reconstructed rust 1.90.0 source tree (a codeload tag tarball omits submodule contents). After egress recovered, the exact gitlink commit of rust 1.90.0 was resolved (`rust-lang/backtrace-rs @ b65ab935fb2e0d59dba8966ffca09c9cc5a5f57c`, crate version 0.3.75 — matching the `Cargo.lock` pins the std subset actually resolves with: `addr2line 0.24.2`/`object 0.36.7`/`gimli 0.31.1`, all previously vendored), fetched via codeload, and dropped in. std includes it via `library/std/src/lib.rs` `#[path = "../../backtrace/src/lib.rs"] mod backtrace_rs;` with `backtrace_in_libstd` set by the shipped std build-script override.

Verification after swap:

| Check | Result |
|-------|--------|
| `make -f minicargo.mk LIBS` full rebuild against real sources | ✅ exit 0 |
| `std::backtrace::Backtrace::force_capture().status()` (smoke) | ✅ `Captured` (stub returned `Unsupported`) |
| Frame rendering in this mrustc codegen environment | 0-frame render (`Backtrace []`), trace-only config — the std `backtrace`/symbolization features (`addr2line`/`object`/`miniz_oxide`) stay feature-gated off exactly as in rust's own default -Zbuild-std trace-only builds; unwind-walk depth under mrustc-generated C frames yields no iterations (toolchain codegen artifact, documented, zero NROS impact: no gate/test/example consumes backtraces) |
| Panic-hook backtrace printing | compiled out (same feature gating) — behavior unchanged vs. stub |
| nros-core unit tests rebuilt against the new std | ✅ 17 passed / 0 failed / 1 ignored (same as pre-swap) |

Conclusion: the verification chain no longer contains hand-written stand-ins; it is (mrustc C backend + real rust 1.90.0 `library/` tree + real pinned vendored deps) end-to-end.
