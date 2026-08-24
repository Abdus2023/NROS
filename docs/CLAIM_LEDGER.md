# NROS Claim Ledger

Pass 31 (2026-08-24) — first version of the per-claim ledger ordered by
`docs/audit/verification.json` (`claim_ledger.status: TODO`) and by the claim
authority rule in `docs/verification/claims.md`: **No observed evidence → no
verified claim.**

Every entry maps a claim in `docs/representation/claims.yaml` to its source,
implementation, executed evidence (run IDs resolve to immutable GitHub Actions
records per EVID-006), environment, current class, and the exact wording that
is permitted. A claim moving to a stronger class requires new executed
evidence recorded here first.

Executed-evidence citations:

- **Run 32718667809** — 2026-08-24, branch `arena/01a03356-nros`, head
  `2d4ea5b901bee386d44b1a7994cfa8088d4a26e2`. First fully green 9/9 CI in the
  repository's recorded history, including the first green hard Miri gate
  (`Miri on nros-core` + `Miri on nros-types` both success) and the first
  green doc-gate (python validator + `nros-audit all` + representation +
  structural safety, all success). This is the F30-01 remediation commit.
- **Run 32719445556** — 2026-08-24, branch `arena/01a03356-nros`, head
  `5599c78a5` (Pass 30 evidence-recording commit). 9/9 green; confirms the
  evidence catalog edits themselves pass the gates they describe.
- **Run 32707535971** — 2026-08-24, head `083e3ed9` (Pass 29.4). 7/9 green;
  Miri and doc-gate red. Historical reference for the decoded failure causes.

| Claim (id) | Subject | Source | Implementation | Evidence (executed) | Test | Benchmark | Environment | Class (policy) | Allowed wording |
|---|---|---|---|---|---|---|---|---|---|
| CLAIM-IPC-001 | SPSC ring buffer | DESIGN.md §14.1 | `crates/nros-core/src/lib.rs` — type-state guard SPSC ring | Run 32718667809 Miri green both crates; run 32707535971 full `cargo test` green; ASan/UBSan clean on 54 tests (F29-12, `tools/offline-mrustc/probes/sanitizer.sh`) | 20 native tests + 4 trybuild negative-compile fixtures, all green in CI | `tools/offline-mrustc/probes/microbench.rs` same-thread 110.90 ns/op; cross-thread scheduler-bound (not ring cost) | GitHub `ubuntu-latest` (rustc stable) + nightly Miri; offline mrustc sandbox | **allowed_with_scope** | "Guard-based SPSC ring buffer: native tests, Miri (both libtest suites) and ASan/UBSan pass on `arena/01a03356-nros` @ head `2d4ea5b90`". Excludes: MPMC, shared-memory, hardware real-time guarantee, loom interleaving proof (pending) |
| CLAIM-PERF-001 | Latency / throughput numbers | README.md, COMPARISON.md | `crates/nros-core/src/bin/bench.rs` + `tools/offline-mrustc/probes/microbench.rs` | benchmarks CI job completed and uploaded artifact first time in run 32707535971 (non-gating) | F29-09 termination-defect fixed: 20/20 (2k) and 5/5 (100k) runs, `sent == received == 100000` | Same-thread 110.90 ns/op ≈ 9.02M ops/s; cross-thread 156.52 μs (sandbox) | Shared unpinned 2-vCPU sandbox vs GitHub runner — NOT equivalent | **conditional** | "Same-thread SPSC ring cost measured at 110.90 ns/op on the recorded environment; `<10 µs` end-to-end remains a target, not a result". No "6.2 µs", "780K msg/s", "46× over ROS2" |
| CLAIM-HAL-001 | Real DMA / V4L2 hardware integration | NROS_PLATFORM_AND_HARDWARE.md | `crates/nros-hal` — abstraction + `SimulatedDmaBuffer` | None (no memfd/mmap/DMA-BUF/V4L2 path, no Jetson run) | HAL unit tests pass (simulated backend) | N/A | No hardware | **forbidden** | "HAL provides a hardware abstraction design with a simulated backend; hardware validation not run" |
| CLAIM-DIST-001 | Raft consensus | NROS_DISTRIBUTED_SYSTEMS.md | `crates/nros-distributed` — deterministic/pseudo election + simulated replication | None establishing a complete Raft protocol | State-machine unit tests green in CI | N/A | Single-process simulation | **forbidden** | "Distributed runtime is scaffolded/simulated — not a production Raft implementation" |
| CLAIM-STUDIO-001 | Production live telemetry | README.md studio section | `crates/nros-studio` — HTTP/SSE/REST + `DemoDataProvider` vs `LiveNrosDataProvider` | None establishing live-source telemetry | Studio tests green in CI | N/A | Demo/synthetic data source | **forbidden** | "Studio dashboard is implemented and exercised against demo telemetry; live operational telemetry is not validated" |
| CLAIM-SAFETY-001 | ISO 26262 / IEC 61508 qualification | EVIDENCE_REGISTRY.md taxonomy | Process evidence across repo | Partial only: rustc tests + Miri + ASan/UBSan green; **loom NOT RUN**; no hardware validation; no qualification process | All of the above | N/A | N/A | **forbidden** | No production-safety, soundness-complete, or standards-qualification wording for `nros-core`. "Substantially tested" is the strongest permitted summary |
| CLAIM-CI-001 | CI passing | `.github/workflows/ci.yml` | 9-job gate decomposition | Runs 32718667809 and 32719445556: **all 9 jobs success** (earlier: 7/9 in 32707535971) | Each job's step-level conclusions resolvable via GitHub API | benchmarks job is intentionally `continue-on-error` report-only | ubuntu-latest | **conditional** | "All nine CI jobs pass on `arena/01a03356-nros` @ `2d4ea5b90` / `5599c78a5` on 2026-08-24, per runs 32718667809 / 32719445556". Never citing workflow configuration alone as a pass |
| CLAIM-MIRI-001 | Miri validation passing | Safety-gate job | `cargo miri test -p nros-core --lib` + `-p nros-types --lib` | Run 32718667809: both steps success. Prior 40/40 red decoded (run 32707535971 log): wall-clock syscall in one test under Miri isolation, no UB diagnostic ever emitted. F30-01 removed the host-clock dependency (deterministic `Timestamp { sec: 1, nanosec: 0 }` in the affected test; `#[cfg_attr(miri, ignore)]` for the inherently wall-clock-bound nros-types test); isolation left enabled | 20+-test suite per crate executed under Miri | N/A | nightly-2026 toolchain, ubuntu-latest | **conditional → satisfied at scope** | "Miri executes green on nros-core and nros-types libtest suites in run 32718667809 (isolation enabled, no UB reports)". Do NOT generalize to "nros-core is sound" — loom interleaving proof and exhaustive unsafe review are still pending |

## Ledger maintenance rules

1. A claim's class may only be **strengthened** after the new executed
   evidence run ID is recorded in the Evidence column (EVID-006).
2. Narrative descriptions of runs are not evidence: the run must resolve.
3. When a CI gate regresses, the affected claims drop back to their prior
   class in the same commit that records the regression.
4. The `allowed_with_scope` / `conditional` / `forbidden` classes themselves
   live in `docs/representation/claims.yaml`; this ledger is the per-claim
   binding between that policy and the executed record.
