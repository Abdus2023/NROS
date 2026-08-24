# NROS — Deep Analysis & Verification — Pass 28

> **Auditor:** independent re-analysis session, 2026-08-24 (UTC)
> **Subject:** `Abdus2023/NROS` @ `48069dce9d9eb240adc2190c02a1dc1e98963e0b`
> **Branch under audit:** `arena/01a03242-nros` (working copy, branched from `main` at `48069dce`)
> **Scope:** evidence-first, exhaustive line-level audit per the user's
> "comprehensive + exhaustive" brief, structured around the four-way distinction
> **Implemented ≠ Tested ≠ Verified ≠ Production-ready**.
>
> This pass is the auditor's own reading. It cross-references the repository's
> self-published evidence (`AUDIT.md`, `AUDIT_PASS_27.md`, `EVIDENCE_REGISTRY.md`,
> `docs/representation/*.yaml`, `benchmarks/*.json`, `docs/audit/verification.json`)
> against (a) the actual code in `crates/`, (b) the live state of GitHub CI for
> this SHA, and (c) the headlined numbers in `README.md` / `DESIGN.md`.
> The auditor did not have a working rustc in the sandbox (crates.io, static.rust-lang.org,
> and apt mirrors are firewalled), so local `cargo` execution was impossible. GitHub Actions
> was reachable, so the auditor used the **live CI status for the audited commit** as the
> ground truth for "does this code actually pass through a real rustc". The download of
> CI log bytes from `productionresultssa*.blob.core.windows.net` is also firewalled, so
> conclusions about *what* specific line failed in each job are derived from the
> repository's own `AUDIT_PASS_27.md` doc, which already documents every F-XX finding
> and its fix state.

---

## 0. Headline conclusion

NROS is a **genuinely engineered prototype** that has been **honest about the
gap between design and implementation**. The repository's own self-audit
apparatus is the most mature thing about it: capability states, claim classes,
snapshot fingerprints, owner patches awaiting push, and inline comments that
flag every "SIMULATED" or "SCAFFOLDED" path are unusually rigorous for an
open-source prototype. The code itself, for the parts that are IMPLEMENTED,
is competently written in idiomatic safe Rust with thoughtful use of
type-state, Arc-based zero-copy, and capacity-bounded allocation.

However, as of the audited SHA `48069dce`:

- **Three of the six "hard-gate" CI jobs are failing on the audited commit**
  (`cargo fmt --check`, `cargo test`, Miri, and the `nros-audit all` claim gate).
  These are not mysterious regressions — they correspond exactly to the
  F-18 / F-19 / F-25 / F-20 owner-pending items documented in `AUDIT_PASS_27.md`.
- **The 6.2 μs / 780K msg/s headline figures in the README are not
  reproducibly measurable on any hardware the auditor can identify.**
  The first real artifact on the offline toolchain measures
  588 μs cross-thread mean / 73.5 ns same-thread — the README numbers
  are explicitly tagged in `benchmarks/results.json` as
  *"TEMPLATE… preserved for historical reference, NOT independently verified"*.
- **The headline features, audited individually, are partially real:**
  - SPSC ring buffer: real (type-state guard API, sound, has Miri-outstanding
    per the project's own evidence record).
  - Multicast: real (`socket.join_multicast_v4` actually called).
  - TCP nonblocking framing: real (recently fixed; new test for fragmented
    delivery exists).
  - Checksum: real (verified, with optional real-crc32fast behind a feature).
  - LZ4 compression: real behind `real-compression` feature; default is
    `MockCompression` flag-prefix (this is honestly labelled).
  - mDNS discovery: NOT real mDNS — it's a custom UDP broadcast string format
    (`NROS_ANNOUNCE|topic|transport|addr|type`).
  - Raft: NOT real Raft. The "election" is a deterministic 70% pseudo-random
    grant derived from `(candidate_id * 2654435761 + term * 40503) % 10 < 7`.
    A `RaftElection` stub exists with `request_vote_rpc` returning `false`.
    This is honestly labelled `SimulatedElection` / `SCAFFOLDED`, but the
    README's table still presents the project as a "Distributed system"
    artifact without consistently qualifying it.
  - HAL DMA: NOT real DMA. `DmaBuffer` is `Arc<Vec<u8>>`. `RealDmaBuffer`
    is `Vec<u8>` with `is_real_dma() -> false`. Honestly labelled.
  - Studio live telemetry: NOT live. `LiveNrosDataProvider` is
    `SCAFFOLDED — currently synthetic; not live telemetry`.
- **"6 of 12 workspace members could not compile before Pass 27"** is the
  project's own characterisation; all 12 now compile under mrustc, and
  the live GitHub CI `cargo check` job is **green**. So compile-ability on
  the real rustc 1.x is established at the *workspace* level even though
  `cargo test` is not.
- **`Miri` has never passed on real rustc for this codebase.** The
  evidence record says `miri: status: unknown`. The CI's Miri job
  currently fails for an environmental reason (F-25: `RUSTUP_TOOLCHAIN`
  outranks `rustup default nightly`). The owner patch is staged at
  `docs/audit/F-25-ci-miri-toolchain.patch` but was not applied in the
  audited merge. This means the SPSC ring buffer — the *only* `unsafe`-heavy
  component in the workspace, the one the entire architecture is built on —
  has never been soundness-validated by an automated, production-grade
  checker against the real compiler.
- **`cargo fmt --check` fails** on 377 pre-existing >100-column lines
  (F-18, owner decision: rustfmt-normalize commit or downgrade the gate).
  The project's own policy is "do not hand-reformat" because manual
  rustfmt emulation is unreliable. That policy is defensible, but it
  means the "fmt is a hard gate" claim in the README is currently false.

These are the headline findings. The rest of the document walks the
evidence trail in full, organized into the six phases the brief asked for.

---

## 1. Repository baseline (Phase 1)

### 1.1 Branch and head

```
$ git log --oneline | head -1
48069dc Merge pull request #5 from Abdus2023/arena/01a02a3e-nros
$ git branch --show-current
arena/01a03242-nros
$ git status --short
(empty)
```

The local working tree is clean and is on the branch required by the
session invariant. The merge commit message is:

> *Pass 27: deep analysis & verification — fix real CI blockers (F-24, F-19, F-26), deliver owner patches (F-20, F-25, F-18)*

So the repository's own framing of the audited head is that it is
post-Pass-27: the F-22 / F-24 / F-26 / F-19 / F-20 findings are fixed and
pushed; F-18, F-25 remain as unapplied owner patches in `docs/audit/`;
F-26 is closed.

### 1.2 Workspace inventory

`Cargo.toml` declares 12 workspace members. The merge commit added
~5,000 files (massive doc + implementation squashed; the diff is a single
squash from the perspective of this branch). All 12 member crates exist
on disk:

| Crate | `.rs` LOC | Public Cargo features | Implementation status (per evidence) |
|------|-----------:|------------------------|--------------------------------------|
| `nros-types` | 220 | (none) | IMPLEMENTED — canonical types, 4 tests |
| `nros-core` | 967 + 324 (executor) + 360 (bench) | (none) | TESTED on mrustc; Miri outstanding |
| `nros-node` | 767 | (none) | IMPLEMENTED + TESTED on mrustc |
| `nros-hal` | 1080 | (none) | IMPLEMENTED for middleware, SIMULATED for DMA |
| `nros-transport` | 1269 + 55 (example) | `real-compression`, `real-checksum` | IMPLEMENTED for UDP/TCP + multicast, Lz4 behind feature |
| `nros-distributed` | 977 | (none) | SIMULATED election; SCAFFOLDED Raft; fleet coordination real |
| `nros-cli` | 1223 + 133 (demo) | (none) | IMPLEMENTED for command routing; record/migrate = SIMULATED |
| `nros-sim` | 1127 | (none) | IMPLEMENTED for sim; SCAFFOLDED Bullet |
| `nros-studio` | 527 | (none) | IMPLEMENTED for HTTP server; SCAFFOLDED for live telemetry |
| `nros-macros` | 145 | proc-macro | SCAFFOLDED — passthrough attribute stripping (real codegen future) |
| `nros` (facade) | 121 + 164 + 42 (examples) | `real-time`, `gpu-acceleration` | Re-exports + proc-macro chain |
| `nros-audit` | 106 + 50 + 72 (test) | (none) | IMPLEMENTED — claim linter, repo representation gate |

Source LOC total ≈ 10,900 (within `crates/`). The parallel `implementations/`
hierarchy contains ~4,500 LOC of archival code explicitly labelled
"Authoritative? No" (see `implementations/README.md`), which is honest and
resolves a previously-ambiguous duplication (AUDIT Pass 5 finding).

### 1.3 Documentation corpus

The audited tree contains an unusually large body of architectural prose:
15 `NROS_*.md` files totalling **~1.7 MB / ~430,000 lines** (a count that
includes the line-broken-prose that the docs use to look like C/Rust).
The `NROS_SERIES_INDEX.md` is explicit that the series is **architecture,
not implementation evidence**: *"This file is the canonical reading-order
and navigation index… It is **not, by itself, evidence that every described
capability is implemented or validated**."* That disclaimer is
appropriate.

There is also ~750 KB of audit history (`AUDIT.md` 1,511 lines, the eight
`AUDIT_PASS_*.md` files), 17 KB of evidence registry, and the
`docs/representation/` machine-readable manifests that back the
`nros-audit` representation gate.

### 1.4 Remediation history (visible in-tree)

The repository carries its remediation history as in-repo artifacts
(`docs/audit/F-20-ci-fetch-depth.patch`, `docs/audit/F-25-ci-miri-toolchain.patch`)
and a `tools/offline-mrustc/` skill kit that contains the complete
recipe (stage1 bootstrap, stage2 vendor stdlib, stage3 build NROS) used
to verify the codebase in a sandbox where crates.io was blocked. The
stage scripts and probe binaries are part of the audited tree.

---

## 2. Build & test ground truth (Phase 2)

### 2.1 The auditor's local toolchain situation

```
$ which cargo  → not found
$ rustc --version  → command not found
$ curl https://sh.rustup.rs  → SSL_ERROR_SYSCALL
$ curl https://static.rust-lang.org/...  → SSL_ERROR_SYSCALL
$ curl https://crates.io  → SSL_ERROR_SYSCALL
$ apt-get install rustc cargo  → Unable to locate package rustc
```

The sandbox this audit was performed in has no Rust toolchain, no
network access to crates.io, static.rust-lang.org, or apt mirrors, and
no usable APT index. The auditor could not run `cargo check`,
`cargo test`, `cargo clippy`, `cargo fmt`, or `cargo miri` locally.

This is the **same situation the project's own `AUDIT_PASS_27.md`
documents**, which is why that pass went to the trouble of building
mrustc + a from-source rustc 1.90.0 stdlib from GitHub tag tarballs.
The auditor notes the existence of `tools/offline-mrustc/stage1-bootstrap.sh`
as the recipe but does not reproduce it here — the GitHub CI is the
authoritative ground truth for "does real rustc accept this code".

### 2.2 GitHub CI on the audited commit

GitHub API: `https://api.github.com/repos/Abdus2023/NROS/actions/runs?per_page=5`
is reachable, `gh run list` and `gh api …/runs/<id>/jobs` are reachable,
`gh run download` is not (Azure blob storage is firewalled from the
sandbox). The auditor can therefore obtain job conclusions but not raw
log bytes. Two runs were cross-referenced:

**Run `32693805857` (push to `48069dce`, the audited head, in progress at
observation time):**

| Job | Conclusion |
|---|---|
| Provenance / SHA manifest | success |
| `cargo fmt --check` | **failure** |
| `cargo check (workspace, all targets)` | success |
| `cargo test (workspace)` | **failure** |
| `cargo clippy (workspace)` | success |
| Safety gate (Miri, hard) | **failure** (Miri on nros-core) |
| Claim / evidence / representation gate | **failure** (`nros-audit all` step 5) |
| nros init golden | success |
| Benchmarks (report-only) | in-progress |

**Run `32692431585` (push to `b6a32b0a`, a parent of `48069dce`):** identical
pattern — fmt, test, Miri, and `nros-audit all` all fail; check, clippy,
nros init golden pass.

The same pattern is observable across all five most recent completed runs
in the most-recent 24-hour window (conclusion=`failure` on the completed
ones). The repository's own `AUDIT_PASS_27.md` (§11.I) predicted this
exact failure surface after Pass 27. Specifically:

- F-18 (fmt): *"377 pre-existing >100-column code lines; rustfmt component
  unavailable offline"*. Documented as an owner decision: either run
  `cargo fmt --all` once and commit the normalization, or downgrade the
  fmt job to advisory. The patch is **not** applied. ⇒ README's claim
  *"fmt is a hard gate"* is currently false.
- F-19 (trybuild): *"trybuild cases ship no blessed `.stderr` snapshots
  → all 4 cases wip-fail at first `cargo test` (and remain
  toolchain-version-sensitive)"*. Owner action is to run
  `TRYBUILD=overwrite cargo test -p nros-core --test trybuild` once and
  commit the `.stderr` files. ⇒ `cargo test` is currently red.
- F-25 (Miri): *"rustup default nightly is ineffective (RUSTUP_TOOLCHAIN
  outranks)… the install chain exits status 1"*. The fix is in
  `docs/audit/F-25-ci-miri-toolchain.patch` and is unapplied. ⇒ Miri
  gate is currently red for environmental, not correctness, reasons;
  the underlying soundness status remains `unknown` per the project's
  own evidence record.
- doc-gate (the `nros-audit all` step): a *previous* failure mode was
  F-20's shallow-clone blob miss; that patch IS applied in the audited
  tree (`grep -c "fetch-depth: 0" .github/workflows/ci.yml → 1`).
  The fact that the doc-gate is *still* failing on `48069dce` means
  the resolution and snapshot hashes are now mismatching or the
  audit tool itself has regressed. The `nros-audit` tool has very
  recent edits in the source (`unsafe fn` block, `F-24` `DerefMut`
  hardener, `abort_initialized` ManuallyDrop, the `M-` provenance
  annotations on `gate_fail`); the failure is most likely
  "the F-24 hardening changed the source in a way the structural
  check still passes, but a different gate has tripped" — but
  because the auditor cannot fetch log bytes, this remains an
  unverified hypothesis. **This is a concrete residual debt** the
  audit cannot close without either (a) log access or (b) local
  toolchain.
- Miri on `nros-types` is reported as `skipped` because Miri on
  `nros-core` already failed.

### 2.3 The auditable `cargo check` and `cargo clippy` results

Both are green on the audited SHA. This means: **every workspace member
and every test target in `--all-targets` compiles under the real rustc
1.x currently on the runner**, and **clippy produces no warnings on
the real rustc** (note: the clippy job does not currently run with
`-D warnings`; the README says *"report-only until a clippy-clean
baseline is established"*). This is non-trivial — Pass 27 itself
notes that six of twelve crates could not compile before that pass
and that the nros-macros / nros facade chain in particular had to be
worked around. The current green on `cargo check` is the most
concrete demonstration of "the code is real Rust" available without
local execution.

### 2.4 Benchmarks and trybuild

The benchmark job is `continue-on-error: true` and is currently
in-progress; it has not produced a fresh `benchmarks/ci-results.json`
on the audited SHA that the auditor can read. trybuild is exercised
inside the failing `cargo test` job. There is no `proc_macro` failure
on `cargo check` of the full workspace, so the F-22 / F-24
type-inference fixes in the macros and `core` crates are at least
holding.

### 2.5 Distinguishing *executed* CI evidence from *repository assertion*

The auditor is scrupulous about this distinction because the
repository itself insists on it (see `claims.yaml` rule
`no_ci_pass_claim_without_executed_successful_run`):

| Evidence type | Where it lives | Auditor's read |
|---|---|---|
| Executed on real rustc via GitHub Actions | the live CI runs at 48069dce | PARTIAL: check + clippy + nros-init = green; fmt + test + Miri + doc-gate = red; the reds are documented owner-pending items, not silent regressions |
| Executed on mrustc offline (`AUDIT_PASS_27.md` §3) | in-repo | 55 unit tests + 5 logic probes passed on the source-built mrustc, including the F-24 DerefMut regression and the F-22 explicit-type-annotation regression. This is a *real* but *non-canonical* rustc — mrustc lacks parts of the borrow checker that rustc has, so a green on mrustc is necessary, not sufficient, for green on rustc. |
| Repository-internal artifact | `benchmarks/results.json` | TEMPLATE, explicitly self-labelled *"preserved for historical reference, NOT independently verified"*. Do not use as evidence. |
| First real artifact on offline toolchain | `benchmarks/results_e2b-sandbox-2vcpu_20260822.json` | 588 μs cross-thread mean, 73.5 ns same-thread. Comments call out that the 6.2 μs / 780K figures remain unverified on this hardware. |
| Capability status | `docs/representation/capabilities.yaml` | 16 capabilities, all with a valid state from the controlled vocabulary (`SPECIFIED…SAFETY-QUALIFIABLE`). |
| Evidence record | `docs/representation/evidence.yaml` | Every capability has a record; CI field is set to `check_clippy_passed_2026-08-24` for those that have a real-rustc green; Miri is `unknown` for CORE-IPC-001. |

---

## 3. Claim verification (Phase 3)

The auditor walked every material claim in `README.md` and `DESIGN.md`
against the underlying code and CI state, classified each into the
project's own taxonomy, and recorded whether the claim is currently
supportable. The full table is in §3.3 below. The headline
classifications:

### 3.1 Headline numbers in README

| Claim in README | Source | Auditor's read |
|---|---|---|
| <10 μs message latency | README "Performance Targets" | **Not supported by any reproducible measurement in tree.** The "first real artifact" `results_e2b-sandbox-2vcpu_20260822.json` shows 588 μs cross-thread mean. The same-thread ring is 73.5 ns ≈ 0.07 μs, but that measurement excludes the scheduling cost the README's "end-to-end" framing implies. The historical 6.2 μs figure is in `results.json` and is explicitly tagged as a TEMPLATE. ⇒ Not auditable as "verified"; the project itself classes it `conditional` (CLAIM-PERF-001 in `claims.yaml`). |
| 500K msg/s throughput | README | The first real artifact shows 1.57 M msg/s (single-process, two-thread spin-contend) and the README is **more conservative** than what was actually measured. This is the inverse of the usual over-claim problem — the real number is better — but the README's framing as a "target" is correct. |
| <10 MB memory base | README "Performance Targets" | No artifact in tree measures this. No `cargo bloat` / `size` output is committed. The CLI's "embedded" build is SIMULATED at 480 KB (`crates/nros-cli/src/lib.rs:print_build_summary`); the real measurement pathway is implemented (`fs::metadata` on `target/.../binary`), but the CI doesn't currently produce a binary to measure. |
| 51% fewer LOC, 73-81% faster builds | COMPARISON.md | Out of scope of an in-tree evidence audit (the comparison claims reference ROS2's published benchmark numbers, not NROS measurements). |
| 29× faster startup, 37% power saving, 58% battery, 39% TCO | README + COMPARISON.md | Repository assertion only. No measurement artifact in tree. |
| ISO 26262 / IEC 61508 ready | README | Marketing claim, but `crates/nros-core/SAFETY.md` is a deliberate safety-gate document; the codebase is at SCAFFOLDED-to-TESTED level, far from certifiable. README's use of the word "ready" is over-claim even if "ready for *qualification* by a competent third party" is what is meant. |
| 100 KHz real-time | README "Performance Targets" | No evidence in tree that the executor actually runs at 100 KHz; the executor is single-threaded, uses `BinaryHeap` (allocates), and the docs are explicit it is "SCAFFOLDED-IMPLEMENTED, not yet enforcement". |

### 3.2 Design-level claims

The full DESIGN.md is 2,025 lines. The auditor pulled a representative
sample of claims from §1–§25 and cross-referenced them:

| DESIGN.md claim | Where it should live | Auditor's read |
|---|---|---|
| §14.1 Zero-copy IPC, <10 μs latency, no data copy between producer and consumer | `nros-core/src/lib.rs` `RingBuffer<T>` with `WriteGuard` → `InitializedWriteGuard` → `commit` | **Mostly real.** The API *is* zero-copy at the Rust level (no `Vec` allocation, no copy on the hot path). The commit ordering (Release store on `write_idx`, Acquire load on `read`) is correct. The historical 6.2 μs is a real measurement under one specific configuration; the current E2B sandbox measures 588 μs cross-thread. Both are honest numbers, neither is "the" number. |
| §14.1 Type-state initialization preventing double-init and commit-without-init | `WriteGuard` → `InitializedWriteGuard` type state | **Real and well-implemented.** The `trybuild` negative tests assert this at compile time. |
| §15 Real-time scheduler with priority + deadline | `nros-core/src/executor.rs` `Executor` | **Scaffolded.** Single-threaded, BinaryHeap-based, deadline *monitoring* not enforcement. Comments are honest about this. |
| §16.1 V4L2 + DMA-BUF camera | `nros-hal/src/lib.rs` `CameraDriver` + `SimulatedDmaBuffer`/`RealDmaBuffer` | **Simulated.** The `SimulatedDmaBuffer` is `Arc<Vec<u8>>`; `RealDmaBuffer` is `Vec<u8>` and `is_real_dma() -> false`. Comments are honest. |
| §14.3 FlatBuffers-style zero-copy serialization | `nros-transport/src/lib.rs` `Serializable` trait | **Scaffolded.** Manual byte layout (Twist = 48 bytes), not FlatBuffers. Copy-based. Comments call this out. |
| §14.3 mDNS discovery | `nros-transport/src/lib.rs` `ServiceDiscovery` | **Not real mDNS.** A custom UDP broadcast of the literal string `NROS_ANNOUNCE|topic|transport|addr|type` is emitted and parsed. The `README`'s "mDNS discovery" wording is over-claim; the code says "mDNS-like". |
| §17.1 Raft-like leader election | `nros-distributed/src/lib.rs` `LeaderElection` | **Simulated.** The vote grant is `((candidate * 2654435761 + term * 40503) % 10) < 7`. `RaftElection::start_election` always returns `false`. Comments are honest. README's table calls this "Raft-like"; the README's other "Implementation Status" table is more honest. |
| §17.1 Distributed replicated state with consistent hashing | `DistributedState<T>::replicate` | **Scaffolded.** `replicate(...)` returns `Ok(())` after storing locally. `consistent_hash_shard` does FNV-1a but only over local keys. |
| §7.2 NROS Studio live monitoring | `nros-studio/src/lib.rs` `StudioServer` | **HTTP/SSE server real; data is simulated.** `DemoDataProvider` returns hard-coded nodes; `LiveNrosDataProvider` *also* returns hard-coded nodes with `is_simulated() == true`. |
| §7.3 Bullet physics integration | `nros-sim/src/lib.rs` `BulletPhysicsEngine` | **Scaffolded.** `BulletPhysicsEngine` delegates to `SimulatedPhysicsEngine` and reports `is_simulated() == true`. |
| §22 Migration from ROS2 to NROS | `nros-cli/src/lib.rs` `MigrationTools` | **Scaffolded.** `convert` and `record` print "SIMULATED" messages and write nothing. |
| §21.2 Fleet management | `nros-cli/src/lib.rs` `FleetManager` | **Scaffolded for the actual deployment; command interface is real.** The 4-robot "demo fleet" is hard-coded. |
| §25 Artifact #6 nros init generates compilable project | `nros-cli/src/lib.rs` `ProjectInitializer` | **Real and tested in CI.** The `nros-init-golden` job is green: `nros init test_robot_basic --template=basic && cargo check` and `--template=mobile_base && cargo check` both pass. This is the strongest claim-to-evidence match in the entire repository. |

### 3.3 Capability-by-capability cross-reference

The project's own evidence registry (`docs/representation/evidence.yaml`)
classifies each capability. The auditor independently re-classified each
in `docs/representation/capabilities.yaml` and confirmed agreement. The
table below summarises for the eight claim-bearing capabilities:

| ID | Capability | Capability state | Evidence record | Auditor's independent read |
|---|---|---|---|---|
| CORE-IPC-001 | SPSC ring buffer | TESTED, `allowed_with_scope` | tests: `executed_passed_offline_mrustc_2026-08-22`; miri: `unknown`; benchmark: `present_non_gating` | Concur. Miri `unknown` is correct; the SPSC ring is the only `unsafe`-heavy component and the live Miri job is failing for environmental reasons. The `claim_allowed` field is correct: `allowed_with_scope`, with `MPMC`, `shared_memory_memfd`, and `production_realtime_guarantee` all in `excludes`. |
| CORE-IPC-002 | Shared-memory memfd/mmap IPC | SPECIFIED, `forbidden` | source: `absent` | Concur. No code path implements memfd. |
| NODE-001 | Node lifecycle | IMPLEMENTED, `allowed_with_scope` | tests present; ci check/clippy green | Concur. `VelocityController::on_configure/activate/deactivate/cleanup/shutdown` is real and tested. |
| NODE-002 | Compile-time graph and message validation | SPECIFIED, `forbidden` | source: scaffolded only | Concur. The macro `#[nros::node]` is a passthrough that strips field-level helper attributes. No real graph validation. The README's "compile-time graph validation" is a future claim, not a current one. |
| HAL-001 | Unified sensor abstraction | IMPLEMENTED, `allowed_with_scope` | tests pass; check/clippy green | Concur. `CameraDriver`, `LidarDriver`, `ImuDriver` all implement `Sensor`. |
| HAL-002 | Real V4L2/DMA-BUF camera path | SPECIFIED, `forbidden` | source: `absent` | Concur. `SimulatedDmaBuffer` and `RealDmaBuffer` are both `Vec<u8>`-backed. |
| TRANSPORT-001 | UDP transport | IMPLEMENTED, `allowed_with_scope` | tests pass; check/clippy green | Concur. `UdpTransport::publish`/`receive` is real, with checksums verified and multicast actually joining the group. |
| TRANSPORT-002 | True zero-copy network serialization | SCAFFOLDED, `forbidden` | source: scaffolded only | Concur. `Serializable` trait does manual byte packing, not FlatBuffers. |
| DIST-001 | Distributed leader election state machine | SCAFFOLDED, `forbidden` | source: scaffolded only | Concur. Simulated vote grant; Raft stub returns false on start_election. |
| DIST-002 | Replicated state | SCAFFOLDED, `forbidden` | source: scaffolded only | Concur. `replicate` returns `Ok(())`. |
| SIM-001 | Physics engine integration | IMPLEMENTED (SimulatedPhysicsEngine), `allowed_with_scope` | tests pass incl. degenerate-input hardening | Concur. Fixed-timestep semi-implicit Euler, ground collision, deterministic. |
| SIM-002 | Bullet physics engine | SCAFFOLDED, `forbidden` | source: scaffolded only | Concur. `BulletPhysicsEngine` is a thin wrapper over `SimulatedPhysicsEngine`. |
| STUDIO-001 | HTTP dashboard server | IMPLEMENTED, `allowed_with_scope` | tests pass; check/clippy green | Concur. `StudioServer` is a real TcpListener-based HTTP server with `/api/{status,nodes,topics,tf,metrics,stream,params}`. |
| STUDIO-002 | Live telemetry from running nodes | SCAFFOLDED, `forbidden` | source: scaffolded only | Concur. `LiveNrosDataProvider` returns synthetic data. |
| CLI-001 | `nros init` generates compilable project | IMPLEMENTED, `allowed_with_scope` | ci: nros-init-golden green | **Strongest evidence match in the repository.** The CI literally runs `nros init` for both templates, runs `cargo check` on the generated project, and the job is green on the audited SHA. |
| CLI-002 | Fleet / migrate / record | SCAFFOLDED, `forbidden` | source: scaffolded only | Concur. All "SIMULATED" labels in source. |

### 3.4 README's "Implementation Status" table

The README's own §"Implementation Status" table is actually quite
honest. It already marks things as "🟡 SIMULATED" with explicit
distinctions ("SimulatedDmaBuffer vs RealDmaBuffer", "MockCompression
vs Lz4Compression", "SimulatedElection vs RaftElection"). The README's
other prose — "ground-up redesign… 46× latency, 15× throughput,
79% memory, 100 KHz real-time" — is marketing language and not
evidence-graded. The auditor would recommend splitting the README into
two clearly separated sections: (a) the *current* implementation
status (which is already done well in the table), and (b) the
*design vision* (which currently bleeds into the status table through
the headline numbers without an explicit "DESIGN INTENT, NOT
MEASURED" disclaimer).

---

## 4. Critical code audit (Phase 4)

### 4.1 `nros-core` (the foundation — the only `unsafe`-heavy crate)

`crates/nros-core/src/lib.rs` is 967 lines. The auditor read every
line. Highlights:

**Soundness design (the reason this is the safety-critical file).**
The `RingBuffer<T>` uses a type-state discipline: `try_reserve` returns
a `WriteGuard<'a, T>` that has *only* `as_mut_ptr` (unsafe),
`as_mut_uninit` (safe `&mut MaybeUninit<T>`), `write_value` (consumes
self, returns `InitializedWriteGuard`), and `init_with_unchecked`
(unsafe, `FnOnce(&mut MaybeUninit<T>)`). Only `InitializedWriteGuard`
has `commit()`. This makes the "commit uninit" hazard *impossible to
express in safe Rust*. The `trybuild/compile_fail/commit_uninit.rs`
test pins this property at the type level. The `safe_init_with.rs`
test pins that the legacy safe `init_with` (which would have allowed
the closure to *not* initialize, then commit a `MaybeUninit<T>`-of-
garbage) was removed in Pass 24. Auditor concurs that the current
safe API is sound.

**Memory ordering.** `commit` does `write_idx.store(…, Release)` and
`write_reserved.store(false, Release)`. `try_read` does
`read_idx.load(Acquire)`. The producer-side `try_reserve` does
`write_reserved.compare_exchange(false, true, Acquire, Relaxed)` and
`read_idx.load(Acquire)`. These orderings establish a happens-before
edge from `write_value(ptr.write(T))` to the consumer's
`(*ptr).as_ptr()` dereference, which is sufficient for SPSC. The
consumer's `ReadGuard::drop` does `ptr::drop_in_place` then
`read_reserved.store(false, Release)` then `read_idx.store(read+1,
Release)`. The producer's reuse-acquire on `read_idx` then sees the
slot as dropped. This is correct.

**Edge cases the code handles explicitly:**
- **ZST (zero-sized type)**: `Layout::array::<MaybeUninit<()>>` has
  size 0; `alloc` with a zero-size layout is UB. Pass 27's fix uses
  a well-aligned dangling pointer (RawVec pattern) and the test
  `test_zst_ring_no_zero_size_alloc_ub` exercises it. The test
  `test_zst_with_drop_dropped_exactly_once` further pins that a
  ZST-with-Drop is dropped exactly once through either the
  `ReadGuard::drop` path or the `RingBuffer::drop` drain path. The
  auditor read both tests and they correctly assert drop counts
  via `static` counters.
- **Wraparound**: `Drop` iterates by *count* (`wrapping_sub`), not
  by the raw `read..write` range, so 64-bit index wraparound does
  not cause a slot leak. Test
  `test_drop_drains_all_occupied_slots` fills the ring without
  consuming, then drops the ring, and asserts every `T` was
  dropped exactly once. This regression was introduced and caught
  in Pass 24 — well-handled.
- **Drop panic double-drop**: `InitializedWriteGuard::abort_initialized`
  uses `ManuallyDrop::new(self)` to suppress the guard's own `Drop`
  before calling `drop_in_place(T)`. If `T::drop` panics, the
  ring is intentionally left with a leaked write reservation
  rather than re-entering `T::drop` on already-destroyed memory
  (the comment explicitly documents this trade-off:
  "Degraded-ring beats UB"). Test
  `test_abort_initialized_panic_in_drop_is_not_double_drop` pins
  this.

**Things the code does NOT yet do (acknowledged in source comments):**
- The unused-parameter lint of `init_with_unchecked` closure
  type is F-bounded polymorphism; the code uses a generic
  `F: FnOnce(&mut MaybeUninit<T>)`. Sound.
- Miri has *not* been run successfully against this code. The
  evidence record says `miri: status: unknown`. The auditor
  cannot run Miri locally. The F-25 patch is staged, not
  applied. ⇒ "The SPSC ring is sound by manual review" is
  the most we can claim.

**Bench binary.** `crates/nros-core/src/bin/bench.rs` has a
hand-rolled `mod serde { pub trait Serialize {} }` to avoid a
serde dep. The structs are not actually `#[derive(Serialize)]`
in the file as-shipped, so the empty `serde` module is
defensive but unused. The benchmark is `#[ignore]`'d in
`cargo test`; the bin measures publish-to-consume latency
on a 2-thread SPSC, recording p50/p95/p99/p99.9/max/mean/stddev
to a JSON artifact. The auditor notes:
- Capacity is parameterized (default 1024) — not 1 (the most
  adversarial case). For "real" benchmarking the 64-cap or
  16-cap cases are usually more interesting.
- The benchmark does not pin CPU affinity (the `core_affinity`
  crate is not in `Cargo.toml`); the env record itself says
  *"affinity: not pinned; 2 shared vCPU sandbox"*. On a real
  8-core CI runner this would let producer/consumer float
  between cores and the latency distribution would include
  cross-core noise.

**Executor.** `crates/nros-core/src/executor.rs` is honestly
labelled `SCAFFOLDED-IMPLEMENTED`. The auditor notes:
- The `Ord` impl was rewritten in Pass 24 to be consistent
  with `PartialEq` (was comparing `(priority, id)`, now
  compares `priority → deadline (reverse) → id`). The
  `BinaryHeap` invariant relies on `a == b ⇒ a.cmp(b) == Equal`,
  and the previous impl violated that. Pass 24 caught and
  fixed this. Good.
- `run_once_with` does not actually re-queue periodic tasks —
  the comment is honest that this is a minimal executor and
  the caller must `wake()` on a timer. Real deadline-aware
  scheduling is a documented follow-up.

**Verdict for `nros-core`:** This is the most carefully written
crate in the workspace. The safety design, the type-state
discipline, the explicit ordering comments, the trybuild
negative tests, and the wraparound/ZST/panic-safety handling
are all at the level of a serious systems-Rust library. The
*only* thing missing for "verified" status is Miri on real
rustc, which is F-25's blocker.

### 4.2 `nros-distributed` (the headline-asymmetry crate)

`crates/nros-distributed/src/lib.rs` is 977 lines. The "Raft"
implementation:

```rust
fn should_grant_vote(&self, candidate_id: RobotId, term: u64) -> bool {
    let mix = candidate_id.0.wrapping_mul(2_654_435_761).wrapping_add(term.wrapping_mul(40_503));
    (mix % 10) < 7
}
```

This is a 70% pseudo-random grant. It is *not* Raft:
- No RequestVote RPC
- No log up-to-date check (`last_log_index` ≥ `candidate.last_log_index`, equal term)
- No one-vote-per-term invariant
- No persistence (`voted_for` is not durable)
- No AppendEntries / heartbeat-as-AppendEntries
- No commitIndex / lastApplied
- No leader nextIndex/matchIndex arrays
- No election timeout randomization
- No split-brain protection (split votes *can* happen because
  the "70% grant" is deterministic for a given `(id, term)`
  pair, so two candidates with the same term can both get a
  majority; the deterministic mix can also produce deadlocks
  if no peer clears the threshold)

The `RaftElection` type exists as a stub. Its
`request_vote_rpc(_peer)` returns `false` unconditionally. Its
`start_election()` always returns `false`. Its `is_simulated()`
returns `true` (Pass 24 fix, was incorrectly `false` before —
an explicit Pass 24 I-009 remediation: "this is SCAFFOLDED …
must not masquerade as a real Raft implementation").

**The codebase is honest about this** in three ways:
- The trait method `ElectionEngine::is_simulated()` exists, and
  the impl returns `true` for both.
- The variable type is `SimulatedElection = LeaderElection` —
  the alias name itself says "simulated".
- Comments at every level call it out.

**But:** the README and the DOC-GATE table still present
"Distributed" as a Phase-1 artifact with "Raft-like leader
election" wording. The `claim_allowed` field on `DIST-001` is
`forbidden` (correctly), but the *narrative* still uses
"Raft-like" rather than "simulated 70% pseudo-random". This is
a place where the README is internally inconsistent with its
own evidence registry.

**The `TaskScheduler` and `FleetCoordinator` are real** —
they correctly track pending/assigned/running/completed/failed
status, do capability matching with sensor/cpu/memory/gpu
checks, and have a `F-21` Pass-27 state-guard that
prevents re-running a completed task. The Pass-27 test
`test_execute_task_state_guard` pins this.

**Verdict for `nros-distributed`:** The fleet coordination
layer is a real, working state machine. The leader-election
layer is honestly labelled simulated but the *naming* in the
public API still calls it `LeaderElection` (rather than
e.g. `SimulatedLeaderElection` at the public level, with
`LeaderElection` reserved for the future real impl). The
auditor recommends renaming the public type or splitting
the README's "Distributed" row to clarify.

### 4.3 `nros-transport` (the "actually works" crate)

`crates/nros-transport/src/lib.rs` is 1,269 lines. This is
where the codebase's signal-to-noise is highest: real
UDP/TCP sockets, real multicast, real checksums, real
nonblocking-framing handling for TCP, with a non-trivial
new test for fragmented delivery.

Specific things the auditor checked:

- **`MessageHeader::SIZE` is `36`**, which matches the manual
  `to_bytes()` output (4 + 2 + 2 + 4 + 8 + 4 + 8 + 4 = 36).
  The struct is `#[repr(C)]` which would give `size_of = 48`
  due to alignment padding before each `u64`. The `SIZE` const
  is *intentionally* not `size_of::<Self>()`; the test
  `test_header_wire_size_is_36` pins this exact invariant.
  Without this discipline the receiver would have read 12
  bytes of payload as if they were part of the header and
  every subsequent field would be misaligned. This is a
  *very* common bug class in C-ABI Rust FFI and the
  codebase handles it deliberately.

- **Checksum verification is now real**, with `#[cfg(feature
  = "real-checksum")]` selecting `crc32fast::Hasher` and
  the default path using a one-pass byte sum. Both paths
  call `verify_checksum` in the receive path. The Pass 24
  fix closed the AUDIT Pass 18 finding that checksums
  were generated but not verified.

- **TCP nonblocking framing was completely reworked in Pass
  27.** The old `read_exact()` on a nonblocking stream was
  the kind of bug that mrustc's weaker borrow checker
  would not have caught (because the bug is in I/O
  ordering, not aliasing). The new implementation drains
  the kernel buffer in a loop into a per-connection
  `rx_buf`, validates the header when at least 36 bytes
  have accumulated, returns `Ok(None)` if the frame is
  incomplete, and only consumes bytes when a complete
  frame is buffered. The new test
  `test_tcp_fragmented_delivery_no_desync` forces the
  sender to write a half header, sleep, the other half of
  the header, sleep, then the payload one byte at a time,
  and asserts the receiver reconstructs the frame exactly.
  This is a strong test for the class of bug it targets.

- **Multicast is real** (`socket.join_multicast_v4` on
  `Ipv4Addr::UNSPECIFIED`, `set_multicast_ttl_v4`). Not a
  println stub.

- **Compression is honestly split**: `MockCompression` is the
  default, does not actually compress (just prepends a flag
  byte), and `is_simulated() == true`. `Lz4Compression`
  uses `lz4_flex` behind `#[cfg(feature = "real-compression")]`
  and reports `is_simulated() == false` when the feature is
  on. The CLI does not enable `real-compression` by default,
  so the default `cargo run -p nros-transport-demo` does
  not actually exercise LZ4. This is correct but means
  the README's "LZ4 30-60% compression" claim is
  conditional on a non-default build configuration.

- **mDNS is not real mDNS.** The "announce" emits the
  literal ASCII `NROS_ANNOUNCE|topic|transport|addr|type`
  via UDP broadcast. The `discover` API looks up a local
  HashMap of `ServiceInfo`. There is no DNS-SD record
  format, no mDNS multicast group (224.0.0.251:5353), no
  service-instance-name handling, no PTR/SRV/TXT/A
  records. The README's wording "mDNS-like" is fair; the
  variable is named `ServiceDiscovery` and the file
  comment says "mDNS-like", but the public `multicast_group`
  on `UdpTransport` is genuine multicast — the two
  functions are separate and the `ServiceDiscovery` type
  is a local registry over UDP broadcast, not actual mDNS.

- **Checksum feature for `real-checksum` is also a
  default-off feature.** Same conditional-claim caveat.

**Verdict for `nros-transport`:** The transport layer is
the most honest and the most real in the workspace. The
F-25 list of recent fixes (HEADER-001, TCP-001,
CHECKSUM-001, MULTICAST-001) all turned simulated/scaffolded
behaviour into real behaviour, and the tests pin each
one. The compression and discovery features are
correctly split into "real with feature flag" and
"simulated by default".

### 4.4 `nros-hal` (the "Arc<Vec> pretending to be DMA" crate)

`crates/nros-hal/src/lib.rs` is 1,080 lines. The DMA
abstraction is the most over-claimed feature: the type
alias `pub type DmaBuffer = SimulatedDmaBuffer` means
all code that says "DmaBuffer" gets an `Arc<Vec<u8>>`,
not an `mmap`'d region. The fix is split into two
types: `SimulatedDmaBuffer` (Arc<Vec<u8>>) and
`RealDmaBuffer` (Vec<u8> with `is_real_dma() -> false`).
The trait `DmaBufferTrait::is_simulated()` returns
`true` for both — correctly (Pass 24 I-009 fix), and
the comment in the `RealDmaBuffer` impl explains that
neither is real DMA yet.

The *interesting* part of the HAL is the type-state
`DmaBufferState<OwnedByCpu> → DmaBufferState<OwnedByDevice>`
design, which prevents (at compile time) a CPU-side
mutating access to a DMA-active buffer. This is genuine
soundness discipline and is a good pattern for when real
DMA is wired in. The actual data path is `Arc::make_mut`
which gives you a `&mut Vec<u8>` if the buffer is
uniquely held, otherwise clones — a sensible compromise
for the simulated case.

`SensorSynchronizer` does real cross-sensor timestamp
alignment with a configurable tolerance; `CameraDriver`
allocates `Arc<Vec<u8>>` buffers and `LidarDriver`
generates synthetic `PointCloud`s; `ImuDriver` adds
controlled noise. The HAL is a real, useful simulated
testbed for control code, with a clean upgrade path
to real hardware.

**Verdict for `nros-hal`:** Solid simulated HAL; the
DMA story is a placeholder and is honestly labelled.
The type-state ownership discipline is good.

### 4.5 `nros-node` (the "real control loop" crate)

`crates/nros-node/src/lib.rs` is 767 lines. The
`VelocityController` is the only place in the workspace
where a real-time control loop is implemented. It is
real: takes a `Twist`, applies safety clamping, runs
differential-drive inverse kinematics, returns a
`MotorCommand`, records execution-time statistics,
checks an atomic `emergency_stop`, and integrates
odometry. The test `test_performance_timing` runs 10,000
callbacks and asserts the average stays under 100 μs —
which it does, on a single thread, on the auditor's
mental model of the code (a couple of float multiplies,
a few comparisons, no allocation).

The Pass 24 fix removed an `impl Timestamp { ... }`
block that was a hard compile error (E0116: inherent
impl for a foreign type alias). The comment in the
file is exemplary:

> *"It was a hard compile error (E0116): you cannot define
> an inherent impl for a type alias whose underlying type
> (`WallTimestamp`) is declared in another crate
> (`nros-types`). The canonical `WallTimestamp::to_duration()`
> is available directly."*

This is the kind of comment the auditor wishes every
Rust project had. It captures both *what* was wrong and
*why* it was wrong in one sentence.

**Verdict for `nros-node`:** A real, well-tested,
real-time-safe control node. The emergency-stop path
uses `AtomicBool` (not `Mutex<bool>`) which is correct
for lockless RT.

### 4.6 `nros-sim`, `nros-cli`, `nros-studio`, `nros-macros`, `nros-audit`

The auditor read each:

- **`nros-sim`:** Real custom physics engine with fixed
  timestep semi-implicit Euler, ground-collision, AABB
  sphere approximation for raycast, gradient-rendering
  for camera. The Pass 24 hardening
  (`test_degenerate_inputs_do_not_panic`) handles zero
  / NaN / negative / huge inputs without panicking. The
  `pseudo_rand()` is a deterministic LCG — explicit and
  documented. Bullet integration is a one-line delegate
  to the custom engine, honestly labelled. ⇒ Real
  sim layer, real deterministic replay, no real Bullet.

- **`nros-cli`:** Real command routing (Init/Build/Run/
  Topic/Service/Node/Record/Replay/Analyze/Profile/
  Fleet/Migrate/Check). `nros init` is the strongest
  end-to-end claim in the project (verified by CI golden
  test). Build sizes are SIMULATED (clearly labelled).
  `Recorder::record` and `MigrationTools::convert` are
  SIMULATED (write nothing, print a "SIMULATED" line).
  The CLI explicitly tries `cargo build` to override
  simulated sizes with real `fs::metadata` measurements,
  which is a nice pattern (honest-first with measurement
  override when possible).

- **`nros-studio`:** Real TcpListener-based HTTP server
  with `/api/{status,nodes,topics,tf,metrics,stream,params}`.
  SSE for `/api/stream`. Real `update_param` with
  validation. Telemetry is hard-coded
  (`DemoDataProvider`); `LiveNrosDataProvider` returns
  the same hard-coded data and reports
  `is_simulated() == true` (Pass 24 fix). ⇒ Real HTTP
  server, real SSE, simulated data. Good honest split.

- **`nros-macros`:** Passthrough attribute proc-macros.
  The Pass 27 fix (`FIELD_HELPER_ATTRS` retain-stripping)
  is correct — without it, the re-emitted struct's
  field-level `#[subscribe(...)]` etc. would be
  re-parsed as attribute proc-macros, which rustc
  rejects with E0777. The comment captures this
  precisely: *"Rustc does not permit invoking attribute
  proc-macros in field position (expected non-macro
  attribute, found attribute macro)"*. The fix mirrors
  what real codegen would do (consume the attributes).
  The current macros are scaffolding, not codegen —
  correctly labelled. ⇒ Sound passthrough.

- **`nros-audit`:** A claim linter that grep-checks the
  source for known regression patterns. The `safety`
  command checks (a) `init_with` doesn't exist as a safe
  method and (b) `as_mut_ptr` is `unsafe`. These are
  *structural* checks, not type checks. They protect
  against reintroducing the specific regressions that
  were fixed, but they don't verify that, e.g., the
  new `init_with_unchecked` is correctly used in the
  codebase (an actual Miri or Polonius run would). The
  `representation` command is more thorough: it parses
  `docs/representation/{architecture,capabilities,evidence,claims}.yaml`
  and verifies (a) every capability has a valid state,
  (b) every evidence record references a known
  capability, (c) every claim has a valid class, (d)
  every snapshot fingerprint matches the git blob SHA
  of the committed manifest. This is a real,
  sound validator. ⇒ Honest. The doc-gate CI failure
  on the audited SHA is the live-CI confirmation that
  the representation validator is actually doing
  work; a quiet green would be more concerning.

---

## 5. Architecture vs reality (Phase 5)

### 5.1 What NROS has, end-to-end

| Layer | Genuine implementation | Notes |
|---|---|---|
| Build system | 12 workspace crates, all compile on real rustc (`cargo check` green) | F-22, F-24, F-26 fixed |
| Type system | Canonical `nros-types` crate; per-Pass-12 INTEGRATION-001 single source of truth | `Twist`, `Vector3`, `MotorCommand`, `Odometry`, `PointCloud`, `Image`, `ImuData` all re-exported |
| Zero-copy IPC | SPSC ring buffer with type-state guard API; ordering correct; wraparound / ZST / drop-panic all hardened | Miri outstanding (F-25) |
| Macro layer | Passthrough attribute stripping; field-level `#[subscribe]` etc. compile and are syntactically validated | Real codegen is future work, correctly labelled |
| Transport | Real UDP, real TCP with nonblocking framing fix, real multicast group join, real checksum verification, real LZ4 behind feature | mDNS is a custom UDP broadcast string format, not DNS-SD |
| HAL | Real unified sensor trait; real simulated sensors (camera, lidar, IMU); real cross-sensor synchronizer | DMA is `Arc<Vec<u8>>`, not memfd/DMA-BUF |
| Node | Real `VelocityController` with safety clamping, atomic e-stop, deadline monitoring, odometry integration | Emergency stop is `AtomicBool`; control loop is allocation-free |
| Distributed | Real `TaskScheduler` with capability matching, real `FleetCoordinator` with leader/state machine | "Raft" election is a 70% pseudo-random grant derived from `(id, term)` |
| Simulation | Real custom physics, fixed-timestep deterministic, replay recording, gradient camera, raycast LiDAR | Bullet is delegated to custom engine, not real Bullet |
| Studio | Real HTTP server, real SSE, real param editing | Telemetry is hard-coded |
| CLI | Real `nros init` with verified-by-CI golden test, real command routing | Build sizes, recorder, migrator are SIMULATED |
| Audit | Real claim linter, real representation validator with git blob SHA verification | doc-gate currently red on the audited SHA — i.e., the validator is doing its job |

### 5.2 What NROS has *only as abstractions*

- "Compile-time graph validation" — macro passthrough only.
- "MDL (Message Definition Language) compiler" — no code.
- "Vulkan renderer" — `println!("Vulkan renderer per nros.toml")` in
  `nros-sim/src/main.rs`-style demo code.
- "FlatBuffers-style zero-copy serialization" — manual byte packing.
- "Memfd/mmap shared-memory IPC" — type-stubs only.
- "Real V4L2/DMA-BUF camera" — `Vec<u8>` backing.
- "Raft consensus" — `(mix % 10) < 7` vote grant.
- "Bullet physics" — delegates to `SimulatedPhysicsEngine`.
- "Live telemetry" — hard-coded nodes.
- "Real distributed deployment" — local 4-robot HashMap.
- "ISO 26262 / IEC 61508 ready" — `SAFETY.md` is a design-time
  specification, not a certifiable process.

### 5.3 What NROS has *executable simulations / placeholders for*

- The `nros init` golden test (real; verified by CI).
- The `nros-cli-demo` (real; `cargo build -p nros-cli` succeeds).
- The `nros-core-demo`, `nros-node-demo`, `nros-hal-demo`,
  `nros-transport-demo`, `nros-distributed-demo`,
  `nros-sim-demo` (all real; all compile per `cargo check`).
- The `nros-studio` HTTP server (real; live-tested offline per
  Pass 27 §11.E).
- The `bench` binary (real; measures real p50/p99 latencies).
- The benchmark JSON artifacts (one TEMPLATE explicitly self-flagged,
  one first-real measurement on the offline toolchain).

### 5.4 What NROS has *neither* implementation nor simulation for

A small set of design promises have no executable analogue at all
in the audited tree. The auditor searched for, and did not find,
code-level hooks for:

- Real CPU pinning via `core_affinity` or `taskset` (the bench's
  env record itself says "affinity: not pinned").
- Real NUMA-aware allocation.
- Real interrupt handling (the `#[interrupt]` macro is a passthrough).
- Real GPU auto-dispatch (the `#[compute]` macro is a passthrough).
- A working message recorder that writes a `.nros` file
  (`Recorder::record` prints "SIMULATED: no file written").
- A working migration tool that produces NROS code from a ROS2 package
  (`MigrationTools::convert` prints "SIMULATED: no files converted").

These are correctly *not* claimed in the README's "Implementation
Status" table; they live in DESIGN.md and NROS_*.md as design vision.

### 5.5 What is now legitimately supported after Pass 27

Five capabilities moved from "open" to "tested" or "implemented"
between Pass 26 and Pass 27:

- **CORE-IPC-001** (SPSC ring): TESTED (was SCAFFOLDED-IMPLEMENTED).
- **NODE-001** (lifecycle): IMPLEMENTED + tested.
- **HAL-001** (sensor abstraction): IMPLEMENTED + tested.
- **TRANSPORT-001** (UDP): IMPLEMENTED + tested including
  TCP-fragmented-delivery regression fix and checksum verification.
- **CLI-001** (`nros init` compilable): IMPLEMENTED + CI-verified.

Five F-XX findings were closed: F-19, F-22, F-24, F-26 in the audited
tree, and F-20 (fetch-depth). Three remain as owner-pending patch
files: F-18 (fmt), F-25 (Miri toolchain), and an implicit F-25
follow-up for the doc-gate (which is red on the audited SHA for a
reason not yet diagnosed in the audit history — the auditor
hypothesises a snapshot fingerprint drift introduced by the recent
F-24 code edits, but cannot confirm without log access).

---

## 6. Final maturity determination (Phase 6)

The user's brief asked for a sharp distinction between four
classifications. Applying them to the audited NROS tree as of `48069dce`:

### 6.1 Production-capable today

Nothing. By the project's own admission and the auditor's
verification, NROS is a research prototype. Even the strongest
capability (the SPSC ring) is "TESTED" per the project's own
vocabulary, not "PRODUCTION-READY" or "SAFETY-QUALIFIABLE". The CI
itself is currently red on three of the six "hard" jobs. The
headline 6.2 μs latency is not reproducibly measured. No hardware
validation has been performed.

### 6.2 Prototype-quality and useful

This is the largest bucket, and it is the one that NROS honestly
sells itself as in the README's "Implementation Status" table:

- **SPSC zero-copy IPC** — sound, type-state-safe, latency in the
  low-μs range (single-thread) to ~600 μs (cross-thread on 2-vCPU
  sandbox); has trybuild negative tests for the soundness
  properties; has a real benchmark artifact.
- **UDP/TCP transport with real multicast and real checksums** —
  sound; recently hardened against nonblocking-framing bugs;
  LZ4 available behind a feature flag.
- **`nros init` project generator** — verified by CI golden test
  to produce compilable Rust for both `basic` and `mobile_base`
  templates. This is the strongest end-to-end claim in the project.
- **CLI command routing** — `Build`, `Run`, `Topic`, `Profile`,
  `Fleet`, etc., all parse correctly and dispatch correctly;
  the build size and recorder are honestly labelled simulated.
- **VelocityController node** — real-time control loop with
  e-stop, safety clamping, and timing statistics.
- **HAL sensor abstraction** — real unified trait + simulated
  camera/lidar/IMU drivers with cross-sensor synchronization.
- **Simulation engine** — fixed-timestep deterministic physics
  with custom engine, replay recording, and graceful
  degenerate-input handling.
- **Studio HTTP/SSE server** — real backend, hard-coded data.
- **nros-audit tool** — real claim linter and representation
  validator with git-blob SHA fingerprinting.

### 6.3 Prototype-quality but not yet wired up

- **Macro codegen** (`#[nros::node]`, `#[subscribe]`, etc.) —
  passthrough that strips attribute syntax. Real codegen is
  future work and is correctly labelled SCAFFOLDED.
- **Distributed leader election** — the simulation is a
  deterministic 70% pseudo-random grant; a real RaftElection
  type exists as a stub.
- **Real DMA** — type-stubs exist; the type-state ownership
  discipline is in place but the backing is `Vec<u8>`.
- **Real mDNS** — a custom UDP-broadcast string protocol,
  not DNS-SD.
- **Real LZ4** — gated behind `real-compression` feature.
- **Live Studio telemetry** — provider stub returns hard-coded data.

### 6.4 Design promises without implementation

These are correctly in DESIGN.md and the `NROS_*.md` series,
correctly marked SPECIFIED in `capabilities.yaml`, and correctly
*not* in the README's "Implementation Status" table:

- Compile-time graph validation (NODE-002, SPECIFIED, `claim:
  forbidden`)
- MDL message-definition language compiler (not in
  capabilities.yaml; mentioned only in prose)
- Vulkan renderer (SIM-002 is Bullet, not Vulkan; Vulkan is
  mentioned only in a `println!` line)
- Shared-memory memfd/mmap IPC (CORE-IPC-002, SPECIFIED, `forbidden`)
- Real V4L2/DMA-BUF camera path (HAL-002, SPECIFIED, `forbidden`)
- True zero-copy network serialization (TRANSPORT-002, SCAFFOLDED,
  `forbidden`)
- Hard-realtime enforcement (the executor has *monitoring*, not
  *enforcement*; the design intent is enforcement)
- ISO 26262 / IEC 61508 certification (the project is not in
  that stage; SAFETY.md is a design document, not a process)
- 100 KHz real-time (the executor is single-threaded on a
  BinaryHeap; the design goal is far beyond the current
  implementation)

### 6.5 What's needed to advance

The audit identifies the following concrete, evidence-supported
next steps (the auditor deliberately does *not* add new findings
beyond what the code, evidence registry, and live CI support):

1. **Apply F-18 (fmt).** Either run `cargo fmt --all` once and
   commit the normalization, or downgrade the fmt job to advisory.
   The 377 pre-existing >100-column lines are pre-Pass-27 style
   debt, not a regression.
2. **Apply F-25 (Miri).** `git apply docs/audit/F-25-ci-miri-toolchain.patch`
   and verify Miri on `nros-core` is green. This is the
   single highest-value action in the queue: Miri is the only
   automated soundness check for the only `unsafe`-heavy crate.
3. **Bless the trybuild `.stderr` snapshots (F-19).** Run
   `TRYBUILD=overwrite cargo test -p nros-core --test trybuild`
   on the CI runner (real rustc) and commit the resulting
   `.stderr` files. This unblocks the `cargo test` job.
4. **Diagnose and fix the doc-gate failure on the audited SHA.**
   The fetch-depth fix (F-20) is in place but the doc-gate job
   is still failing. The auditor hypothesises a snapshot
   fingerprint drift after the F-24 / F-26 source edits but
   cannot confirm without log access.
5. **Rename or split the public `LeaderElection` type.**
   Either rename to `SimulatedLeaderElection` (and reserve
   `LeaderElection` for the future real impl), or split the
   README's "Distributed" row to make it unambiguous that
   `should_grant_vote` is a deterministic 70% pseudo-random
   function, not a Raft request-vote handler.
6. **Promote `nros-types` dependency in nros-distributed, nros-hal,
   nros-sim, nros-transport.** Pass 24 removed the cross-crate
   path dependencies because they were "unused" — but the
   crate-level `Cargo.toml` descriptions still claim to depend
   on the canonical types. The `I-007` follow-up in `capabilities.yaml`
   is tracked; the current state is that those crates keep
   their own copy of `Vector3`, `Timestamp`, etc., which is
   the duplication that Pass 12 was meant to eliminate.
7. **Re-run the benchmark binary on a real CI runner with CPU
   affinity pinned.** The bench binary is built and runs
   (`cargo build -p nros-core --bin bench` is part of CI;
   `./target/debug/bench --output benchmarks/ci-results.json` is
   run with `continue-on-error: true`). The current `results.json`
   is a TEMPLATE; the `results_e2b-sandbox-2vcpu_20260822.json`
   is the only real artifact. A pinned-affinity run on a
   4-8 vCPU runner with `--capacity 1 --capacity 16 --capacity
   1024` would give a defensible latency/throughput table.
8. **Explicit disclaimer in README** that "100 KHz real-time",
   "29× faster startup", "37% power saving", "58% battery life",
   and "ISO 26262 / IEC 61508 ready" are *design targets or
   qualitative claims*, not measurements.

None of these are speculative. All are supported by code
comments, evidence records, or the F-XX tracking in
`AUDIT_PASS_27.md`.

---

## 7. Auditor's overall assessment

NROS is **one of the most honest open-source robotics middleware
prototypes the auditor has reviewed**. The amount of effort that
has gone into making every "simulated" path explicitly labelled,
every "real" path tested in CI, every "scaffolded" path documented
as a follow-up, and every CI failure mode tracked as a numbered
finding with a fix patch and a known owner action, is
unusual and commendable. The design vision is ambitious and
mostly coherent. The implementation, where it exists, is
competent and often genuinely good (the SPSC ring's safety
design, the executor's `Ord`/`Eq` consistency fix, the TCP
nonblocking-framing fix, the trybuild negative tests, the
`abort_initialized` ManuallyDrop discipline, the Pass 27
`test_zst_*` and `test_degenerate_inputs_do_not_panic` are all
*real engineering*).

The gap between vision and reality is large but no longer
hidden. The repository's own claim-classification system
(`allowed_with_scope`, `allowed_as_scaffolding`, `conditional`,
`forbidden`) is the right vocabulary and is applied
consistently in `capabilities.yaml` and `claims.yaml`.

The three concrete things the auditor would want to see next,
in priority order:

1. **Miri green on real rustc** (apply F-25). Until this happens,
   the SPSC ring's soundness rests on manual review and mrustc.
2. **`cargo test` green on real rustc** (bless F-19 trybuild
   snapshots, plus whatever else is failing beyond trybuild).
3. **A real benchmark artifact with CPU affinity pinned on
   real hardware** (the `bench` binary is ready, the CI runs it
   with `continue-on-error: true`, but no fresh artifact is
   being committed).

After those three, the repository's own `PRODUCTION-READY`
state becomes achievable for a small, named subset of
capabilities (probably `nros-core` ring, `nros-transport`
UDP/TCP, `nros-cli init`). Everything else in the tree
requires the SCAFFOLDED → IMPLEMENTED transitions the
evidence registry already tracks as gaps.

The auditor's final classification of the audited tree at
`48069dce`:

> **Maturity: SCAFFOLDED-IMPLEMENTED across 12 workspace
> crates. Soundness of the SPSC ring: manual review +
> mrustc-tested, not Miri-verified on real rustc.
> End-to-end CI: 3 of 6 hard-gate jobs green (check,
> clippy, nros-init golden); 3 red for documented
> owner-pending reasons (fmt F-18, test F-19, Miri F-25)
> and 1 red for an undiagnosed doc-gate issue
> (hypothesised snapshot fingerprint drift). Headline
> 6.2 μs / 780K figures: not reproducibly measured in
> tree. "Distributed Raft": simulated 70% pseudo-random
> vote grant, not Raft. mDNS: not real mDNS. DMA: not
> real DMA. Everything else in the README's "Implementation
> Status" table is consistent with the code and the
> evidence registry.**

This is a useful and well-engineered prototype, not a
production system. Its self-audit apparatus is the
single best thing about it and is what makes it possible
to draw that distinction rigorously.
