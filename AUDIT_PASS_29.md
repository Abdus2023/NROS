# AUDIT Pass 29 — Deep analysis & verification of NROS @ `48069dce`

**Date:** 2026-08-24
**Branch:** `arena/01a0325b-nros` (based on `main` @ `48069dce9d9eb240adc2190c02a1dc1e98963e0b`)
**Scope:** independent re-verification of the repository's own claims — CI status, the
test suite, the documentation/representation gates, the benchmark claims, and the
`tools/offline-mrustc/` verification kit.

> Numbering note: a concurrent session used the label "Pass 28" on branch
> `arena/01a03242-nros`. This pass takes 29 to avoid two different documents claiming
> the same pass number if both branches merge.

---

## 0. Method — what was actually executed

This sandbox has **no Rust toolchain** and no route to one from the usual places:

| Host | Reachable |
|---|---|
| `github.com`, `api.github.com`, `codeload.github.com` | yes |
| `pypi.org`, `files.pythonhosted.org`, `registry.npmjs.org` | yes |
| `crates.io`, `static.crates.io`, `static.rust-lang.org`, `sh.rustup.rs` | **no** (TLS `SSL_ERROR_SYSCALL`) |
| `raw.githubusercontent.com`, `objects.githubusercontent.com`, `deb.debian.org` | **no** |

`cargo`, `rustc`, `rustup`, `rustfmt`, `clippy` and `miri` are all absent, and none can
be installed. So instead of accepting "NOT_RUN" (which is what `docs/audit/verification.json`
records for every gate), the offline kit in `tools/offline-mrustc/` was repaired until it
actually ran, and the verification was executed for real:

* mrustc (master) + the real rust `1.90.0` source tree + a vendored `libstd`, gcc 12.2,
  2 vCPU, 3.9 GB RAM.
* rustfmt was obtained separately as `@scalar/rust-fmt` 0.2.0 from npm (rustfmt compiled
  to WASM) — see the caveat on F29-03.
* CI truth was taken from the GitHub Actions API (job conclusions and check-run
  annotations), because job *logs* are served from `*.blob.core.windows.net`, which is
  blocked here.

Environment: `Linux x86_64`, `gcc (Debian 12.2.0-14+deb12u1) 12.2.0`, `GNU Make 4.3`,
`Python 3.11.2`, `node v22.22.3`.

---

## 1. Headline result

**The repository's CI is red on `main` at the commit this branch is based on, and has
never been green.** Run [`32693805857`](https://github.com/Abdus2023/NROS/actions/runs/32693805857)
(`main` @ `48069dce`, 2026-08-24T05:30:55Z):

| Job | Conclusion |
|---|---|
| Provenance / SHA manifest | success |
| cargo check (workspace, all targets) | success |
| cargo clippy (workspace) | success |
| nros init generates a buildable NROS project | success |
| **cargo fmt --check** | **failure** (exit 1) |
| **cargo test (workspace)** | **failure** (exit 101) |
| **Claim / evidence / representation gate** | **failure** (exit 1) |
| **Safety gate (Miri, hard)** | **failure** (exit 1) |
| Benchmarks (report-only) | in_progress |

Across the **40 most recent runs** (2026-08-22 → 2026-08-24) every completed run is
`failure`, and per-job:

* `cargo fmt --check` — **failure in 40/40**
* `Claim / evidence / representation gate` — **failure in 40/40**
* `Safety gate (Miri, hard)` — **failure in 40/40**
* `cargo test (workspace)` — failure in 37/40, **success in 3**

So the README's "**CI Gate:** … `cargo fmt --check`, … hard failure (no `|| echo`)"
describes a gate configuration that has never produced a green run.

---

## 2. Findings

### F29-01 (P0) — `cargo test` is nondeterministic; root cause is frame loss in `TcpTransport::receive`. FIXED

**Proof of nondeterminism (same commit, opposite outcomes):**

| Run | `head_sha` | event | `cargo test` |
|---|---|---|---|
| 32694486471 | `dee4f028c2d829a300929330e2a002b5eacb7faa` | push | **success** |
| 32695031562 | `dee4f028c2d829a300929330e2a002b5eacb7faa` | pull_request | **failure** |

`dee4f028`'s only parent is `48069dce` and `git diff 48069dce dee4f028` touches only
`AUDIT_PASS_28.md` and one `.arena/` file, so the PR merge tree is content-identical to
the push tree. Same source, different result ⇒ flaky.

**Reproduced locally** with the offline toolchain — `test-nros_transport` failed
**6/40 runs (15%)**, always the same test:

```
---- ::tests::test_tcp_fragmented_delivery_no_desync stdout ----
thread '::tests::test_tcp_fragmented_delivery_no_desync' panicked at :0:0:
called `Result::unwrap()` on an `Err` value: "TCP connection closed by peer (topic: /chatter)"
test result: FAILED. 7 passed; 1 failed; ...
```

**Root cause** (`crates/nros-transport/src/lib.rs`, the drain loop in
`TcpTransport::receive`): a zero-length read was converted into an error *before any
parse was attempted*:

```rust
Ok(0) => return Err(format!("TCP connection closed by peer (topic: {})", topic)),
```

TCP delivers buffered data before the FIN, so when the test's sender thread writes the
last payload byte and then exits (dropping the socket), a reader that polls after the
FIN has been processed reads the remaining bytes and *then* sees `Ok(0)` — returning
`Err` and discarding a complete, checksum-valid frame that was already sitting in
`conn.rx_buf`. This is not only a flaky test: **a clean peer shutdown loses the last
frame it sent.**

**Fix applied:** EOF now ends the *drain*, not the *call*. `peer_closed` is recorded,
control falls through to the existing framing logic, and the shutdown is only reported
once no complete frame remains (with the buffered/expected byte counts in the message).

**Verification after the fix:**

| Check | Result |
|---|---|
| `test-nros_transport` × 100 | **0 failures** (was 6/40) |
| all 8 suites × 30 each (240 runs) | **0 failures** |
| `nros_core` / `nros_transport` / `nros_node` × 20 each under 4× CPU load | **0 failures** |

---

### F29-02 (P0) — Representation gate failed 40/40 because the repo's own F-20 patch was never applied. FIXED

`docs/audit/F-20-ci-fetch-depth.patch` exists in-tree and adds `fetch-depth: 0` to every
job, with the reason spelled out. It was **not applied**: `.github/workflows/ci.yml`
carried `fetch-depth: 0` in `provenance` only, leaving `doc-gate` on the default
shallow (depth 1) checkout.

Check-run annotations for the failing `doc-gate` job (`97332119610`) say exactly that:

```
failure: NROS representation gate :: snapshot source revision resolves
failure: NROS representation gate :: snapshot manifest architecture.yaml exists at source revision
failure: NROS representation gate :: snapshot manifest capabilities.yaml exists at source revision
failure: NROS representation gate :: snapshot manifest evidence.yaml exists at source revision
failure: NROS representation gate :: snapshot manifest claims.yaml exists at source revision
failure: NROS representation gate :: 5 failure(s) — see FAIL lines above
```

`crates/nros-audit/src/representation.rs` resolves the snapshot's pinned
`source_revision.commit` (`6a9cdec772d2fe83cf084aceb202704ee400360e`) via
`git cat-file -e` / `git rev-parse <rev>:docs/representation/<manifest>`; a depth-1
clone does not contain that object.

**The snapshot data itself is correct.** Independently checked against the GitHub
contents API at the pinned revision:

| Manifest | Recorded blob | Actual blob | |
|---|---|---|---|
| architecture.yaml | `04d3a0b15f27` | `04d3a0b15f27` | MATCH |
| capabilities.yaml | `075ca21cb52d` | `075ca21cb52d` | MATCH |
| evidence.yaml | `de3ba49b73c3` | `de3ba49b73c3` | MATCH |
| claims.yaml | `f4773c3a2bb3` | `f4773c3a2bb3` | MATCH |

**Fix:** `git apply docs/audit/F-20-ci-fetch-depth.patch` — all 9 jobs then check out
full history, and the YAML still parses.

**This change could not be pushed from this session.** `git push` is rejected with
`refusing to allow a GitHub App to create or update workflow .github/workflows/ci.yml
without 'workflows' permission` — the same limitation the F-20 and F-25 patch headers
already record. The fix therefore has to be applied by someone with the `workflows`
scope; it is a one-command apply of the patch that is already in-tree, and it applies
cleanly to the current `ci.yml`.

**Verified by running the real gate binary** (built from `crates/nros-audit` by the
offline toolchain) in a repository where the pinned revision resolves:

```
SNAPSHOT-INTEGRITY: PASS
REPRESENTATION-GATE: PASS          # exit 0
$ nros-audit all                   # exit 0, 354 PASS lines, 0 FAIL
$ python3 scripts/validate-documentation-representation.py
DOCUMENTATION REPRESENTATION: PASS # exit 0
```

---

### F29-03 (P0) — `cargo fmt --check` failed 40/40: the workspace had never been formatted. FIXED and confirmed by CI

Measured with rustfmt compiled to WASM (`@scalar/rust-fmt` 0.2.0, `edition = 2021`):
**27 of 33** `.rs` files under `crates/` differ from rustfmt's output. The 6 that are
already clean show the tool is discriminating rather than always differing.

Differing: every file in `nros-audit`, `nros-cli`, `nros-core` (incl. `bin/bench.rs`,
`executor.rs`, `tests/compile_fail/two_producers_from_one_channel.rs`),
`nros-distributed`, `nros-hal`, `nros-macros`, `nros-node`, `nros-sim`, `nros-studio`,
`nros-transport` (incl. `examples/compression.rs`), `nros-types`,
`crates/nros/src/lib.rs`, `crates/nros/examples/vertical_slice.rs`.

*Resolved:* the WASM rustfmt's version could not be determined, so rather than guess, the
reformat was committed and CI was used as the oracle. Run `32704793107` reports
**`cargo fmt --check` → success** — CI's real stable rustfmt agrees with the output, and
the first hard gate that had never been green is now green. See §7.

---

### F29-04 (P0) — Miri gate fails 40/40; cause not decodable from this sandbox. NOT FIXED — diagnostics patch supplied

Step-level detail for job `97332119664`: `Install Miri` **success**, `Miri on nros-core`
**failure** (exit 1), `Miri on nros-types` skipped. There is no NROS-authored
annotation for it, and the raw log is on the blocked blob host, so the failure cannot
be decoded here.

The repo's own `docs/audit/F-25-ci-miri-toolchain.patch` argues the cause is
environmental (`RUSTUP_TOOLCHAIN=stable` outranking `rustup default nightly`) and
switches to `rustup toolchain install nightly --component miri` + `cargo +nightly`.
**That patch is also unapplied, and this pass could not verify its hypothesis** — so it
was deliberately *not* applied.

The step-level evidence is in direct tension with F-25's premise: if the `miri` component
were unavailable, the `cargo miri setup` inside that same `Install Miri` step would have
failed, and it did not. And F-25 has never been exercised — the branch whose run is
titled "apply F-25 Miri toolchain fix" (`arena/01a03242-nros` @ `dee4f028`) contains no
`ci.yml` change at all; its only files are `AUDIT_PASS_28.md` and one `.arena/` file.

Two candidate explanations remain, and they have very different severity:

1. `cargo miri` is not resolvable in the step that runs it → a toolchain problem.
2. `cargo miri` runs and **finds undefined behaviour in `nros-core`** → a P0 soundness
   bug in the `MaybeUninit` / `drop_in_place` / raw-pointer code, which is precisely the
   code this repository's safety case rests on. Miri is the only gate that can see it.

**Supplied:** `docs/audit/F-29-ci-miri-diagnostics.patch` — a gate-semantics-neutral
change that adds a read-only toolchain probe (`cargo miri --version`, `rustup show`,
`RUSTUP_TOOLCHAIN`) and mirrors the Miri failure reason into workflow-command
annotations, which ride the API instead of the blob host and are readable with
`gh api repos/Abdus2023/NROS/check-runs/<job-id>/annotations`. It applies cleanly to the
current `ci.yml` (`git apply --check` verified). It answers the question above in one CI
cycle. It is not pushed for the same reason as F-20: no `workflows` scope.

---

### F29-05 (P1) — `tools/offline-mrustc/` did not run on a clean sandbox. FIXED (all three stages now exit 0)

The README states the kit "exists so the whole chain — and the know-how — no longer
depends on any sandbox filesystem." On a clean 2-vCPU Debian 12 box it failed at six
distinct points, each fixed and each verified by re-execution:

| # | Stage | Defect | Evidence |
|---|---|---|---|
| 1 | 1 | `tar xzf dl/mrustc.tgz; mv mrustc-master mrustc-master` — a self-move; under `set -e` this aborts every clean run | `mv: cannot move 'mrustc-master' to a subdirectory of itself` |
| 2 | 1 | zlib is built locally but never put on the compiler/linker search path; mrustc's `src/memory_dump.cpp` does `#include <zlib.h>` and the Makefile links `-lz` | `fatal error: zlib.h: No such file or directory`, then `/usr/bin/ld: cannot find -lz` |
| 3 | 1 | The minicargo patch targets `tools/minicargo/toml.h`, which no longer exists (moved to `tools/common/toml.h`, `TypeError` instead of `runtime_error`, booleans in `m_int_value`) | `FileNotFoundError: ... 'tools/minicargo/toml.h'` |
| 4 | 1 | Patch 4a's anchors had drifted: `key == "members"` is now `key == "members" \|\| key == "exclude"`, and the `key == "version"` anchor that *did* match is in the **`[package]`** branch, not `[workspace.package]` | `!! ANCHOR NOT FOUND`; see #5 |
| 5 | 1 | **Patch 4a is actively harmful.** It widened `[package]`'s `key == "version"` branch to also swallow `authors`/`description`/`license`/`repository`, and that branch calls `as_string()`. Any manifest with a list-valued `authors` therefore throws | `EXCEPTION: Error loading manifest '.../compiler-builtins/Cargo.toml' - toml type error` |
| 6 | 2 | The vendor pin table did not match the `library/Cargo.lock` of the very 1.90.0 tree the script downloads, and 4 of its URLs were dead | two `curl: (22) ... 404`; see §3 |
| 7 | 2 | `object` and `adler2` were downloaded but never copied into `vendor/` | absent from the `put` list |
| 8 | 2 | The in-tree path fixups did not match 1.90.0 (`../../../library/core`, `../alloc`, `../std`, `../compiler-builtins/compiler-builtins`) | `UNRESOLVED rustc-std-workspace-core: path = "../compiler_builtins"` |
| 9 | 2 | `make -f minicargo.mk LIBS -j2` exports a jobserver that minicargo inherits (`jobs.cpp`: `num_jobs == 0` → `JobServer::create(0)`), so two mrustc+gcc runs on libcore's ~37 MB of generated C ran at once | `Process was terminated with signal 9` (OOM) on `core` |
| 10 | 3 | **`for t in $NOUT/test-nros_*; do ./$t; done` — `$NOUT` is absolute, so every suite failed to launch.** The pipeline's status came from `tail` and the script runs under `set -u` (not `-e`), so stage 3 printed `STAGE3_COMPLETE` having run **zero** tests | `.//home/user/.cache/nros-toolchain/nros-out/test-nros_core: No such file or directory` ×8 |

Item 10 is the most consequential: the kit's headline evidence ("54 unit tests green
across 8 crates") was not being produced by the step that claims to produce it.

**All three stages now exit 0**, and the test step really runs.

Also recorded because it is easy to misread as corruption: mrustc deliberately writes a
**0-byte `.rlib` placeholder** and keeps the code in the sibling `.o`
(`src/trans/codegen_c.cpp`: *"HACK! Static libraries aren't implemented properly yet,
just touch the output file"*). A "clean up zero-byte rlibs" step would force full
rebuilds; do not add one.

---

### F29-06 (P2) — `nros-node` kept a wall-clock assertion inside `cargo test`. FIXED

`crates/nros-node/src/lib.rs:764`

```rust
assert!(avg_us < 100.0, "avg {} μs too high", avg_us);
```

`crates/nros-core/src/lib.rs:612` states the opposite rule for the same repo:
`// ── Tests — Correctness only, no perf asserts (fixes CORE-008) ──`. It is a latent
flake on a loaded runner — the same class of defect as F29-01. It did **not** fire in 240
local runs nor in 20 runs under 4× CPU load, so it was not the cause of the CI flake, but
the threshold measures the machine, not the code.

**Fix applied:** the threshold is replaced with bookkeeping assertions that cannot depend
on machine speed — every one of the 10 000 callbacks accounted for, and `avg_us` finite
and non-zero. The `deadline_misses == 0` assertion is kept (that is correctness, not
timing). Threshold measurement stays with the benchmark binaries.

**Verified:** `test-nros_node` 0/50 failures, 5 passed.

### F29-07 (P2) — `ServiceDiscovery` announced to port 0. FIXED

`crates/nros-transport/src/lib.rs:1063`

```rust
broadcast_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), bind_port),
```

`bind_port` is the *requested* port, not the port actually bound. `ServiceDiscovery::new(0)`
(the documented "bind Any" path, and what `test_service_discovery` uses) yields
`255.255.255.255:0`, so `announce()`'s `send_to` can never reach a real listener. The
error is swallowed (`let _ = ...`) and `discover()` only reads a local `HashMap`, so the
test passes regardless.

**Fix applied:** `broadcast_addr` now uses `socket.local_addr()?.port()`, and a failure to
read the bound address is a construction error rather than a silent wrong port.

**Verified:** `test-nros_transport` 0/50 failures, 8 passed.

### F29-08 (P2) — three `nros-audit` sub-gates could never fail. FIXED

`check_ci()`, `check_claims()` and `check_benchmarks()` in
`crates/nros-audit/src/main.rs` contain **zero** `gate_fail` calls (verified by
counting). `check_ci()` prints `❌ CI workflow not found` and still exits 0. Only
`check_workspace_inventory()`, `check_safety_invariants()` and `representation::run()`
can actually fail. The README's "CI Gate" wording overstates what `nros-audit -- ci`
enforces.

`check_safety_invariants()` is also narrower than its name: it greps for exactly two
regressions (`pub fn init_with<F>`, and a *safe* `as_mut_ptr`). It does not check the
type-state chain, the absence of `DerefMut` on `ReadGuard`, or the guard-drop
discipline that `crates/nros-core/SAFETY.md` describes.

**Fix applied:** all three now call `gate_fail` (`CI-004`, `DOC-002`/`DOC-003`,
`BENCH-005`). The narrowness of `check_safety_invariants()` is left as-is — widening it
is a design decision, not a defect fix.

**Verified both ways.** From the repo root all three pass and `nros-audit all` still exits
0 with 354 PASS / 0 FAIL. From an empty directory the same binary now fails, which is the
part that was previously impossible:

```
$ cd /tmp/emptygate
nros-audit ci          exit=1  ❌ CI-004: no CI workflow found (...)
nros-audit claims      exit=1  ❌ DOC-002: evidence taxonomy labeling not detected ...
nros-audit benchmarks  exit=1  ❌ BENCH-005: benchmarks/results.json missing ...
```

---

### F29-09 (P1) — the benchmark binary hangs, which is why the CI `benchmarks` job has never completed. FIXED

The `benchmarks` job showed `in_progress` in every run inspected — including runs 40+
minutes old where all eight other jobs had finished. `continue-on-error: true` does not
bound a hang, so the job runs until GitHub's job limit.

**Reproduced:** `bench --iterations 2000` timed out (20 s) on **2 of 12** runs, and
`bench --iterations 100000` timed out (45 s) on **1 of 3**.

**Root cause** (`crates/nros-core/src/bin/bench.rs`). The consumer's exit condition was
`local_latencies.len() >= iterations`, but a latency sample is only recorded when the
shared instant queue yields one. The producer committed the message *first* and pushed
the publish instant *afterwards*:

```rust
guard.write_value(twist).commit();
publish_queue.lock().unwrap().push_back(publish_time);   // after the message is visible
```

so the consumer could receive a message while the queue was still empty. That receive
consumed a message without advancing the counter — and since the producer publishes
exactly `iterations` messages, a single such race made the exit condition permanently
unreachable. The consumer then spun forever after the producer finished.

**Fix applied (both halves):**
1. the producer now enqueues the instant **before** `commit()`, which closes the race and
   brackets the whole publish path; and
2. the consumer terminates on messages **received**, not on latency samples recorded, so
   the exit condition cannot depend on a second data structure staying in step.

**Verified:** 20/20 completions at 2 000 iterations and 5/5 at 100 000, **0 timeouts**
(was 2/12 and 1/3), with `messages_sent == messages_received == 100000` in the artifact —
which is itself the proof that no sample is dropped now.

### F29-10 (P1) — the published benchmark claims are not supported by any executed measurement. ARTIFACT PRODUCED

`benchmarks/results.json` states in its own `notes` that it is *"a TEMPLATE artifact"*
carrying *"repository-reported … 6.2μs mean, 780K msg/s … NOT independently verified"*.
Those are the figures `README.md` repeats ("Prototype measurement repository-reported
6.2 μs avg, 780K msg/s") and that `COMPARISON.md` turns into a headline:
`| Mean Latency | 287 μs | 6.2 μs | **46x faster** |`.

Two real runs of the repo's own harness now exist, plus one from Pass 27. None is close:

| Source | Cross-thread mean | Throughput | Notes |
|---|---|---|---|
| `results.json` (template) | 6.2 μs | 780 000 msg/s | **never executed** — self-declared template |
| Pass 27 artifact (2026-08-22) | 588.21 μs | 1 571 113 msg/s | real, 2-vCPU sandbox |
| **This pass** (2026-08-24, `835165df`) | **156.52 μs** | **3 335 788 msg/s** | real, same class of sandbox |
| This pass, repeat run | 234.71 μs | 3 245 774 msg/s | ~1.5× run-to-run spread |

The cross-thread figure is **scheduler-bound, not IPC latency**: the harness runs a
producer and a consumer that spin-contend for 2 shared vCPUs, unpinned. The number that
actually characterises the ring is the same-thread measurement, which is reproducible
across sessions:

| Measurement | This pass | Pass 27 |
|---|---|---|
| same-thread SPSC publish+consume | **110.90 ns/op (9.02M msg/s)** | 112 ns/op (8.9M msg/s) |

Also corrected: `message_size` is **64 B** (canonical `nros_types::Twist`, `repr(C)`), not
the template's 56 — Pass 27 reached the same correction independently.

**Deliverable:** `benchmarks/results_e2b-sandbox-2vcpu_20260824.json`, generated by the
fixed binary at commit `835165df`, following the repo's own
`results_<host>_<date>.json` convention, with the environment, the scheduler-bound
caveat, the same-thread figure, and the conclusion stated in its `notes`.

**Loose end closed.** `nros-core`'s in-tree `benchmark_latency_monotonic` pushed a
hard-coded `1000` into its latency vector with a `TODO` beside it, then printed a note
admitting the numbers meant nothing — worse than no benchmark, because it reads like a
measurement. It now uses the same publish-`Instant` technique as `bench.rs`, with both
F29-09 lessons applied (enqueue before commit; terminate on messages received). Run
explicitly it produces real figures and terminates:

```
Throughput: 3450431 msg/s, elapsed: 28.98ms, latency samples: 100000
Latency us - mean 186.06, p50 190.52, p95 337.23, p99 360.69, max 396.78
```

`latency samples: 100000` is the check that matters — every sample is accounted for, so
the exit condition is reachable. The mean corroborates the `bench` binary's 156–235 μs
range independently. It stays `#[ignore]`d, so `cargo test` is unaffected.

**What this does and does not establish.** It does not prove the ring is slow — 110 ns/op
same-thread is a good number and it reproduces. It does establish that **no executed
measurement in this repository has ever produced 6.2 μs**, so the README's "Prototype
measurement" and `COMPARISON.md`'s "46x faster" are claims without evidence and should be
scoped or withdrawn per the repo's own rule (*"No observed evidence → no verified
claim"*, `docs/verification/claims.md`).

### F29-11 (P1) — the evidence registry had already ordered the performance claims downgraded; the README never was. APPLIED

`EVIDENCE_REGISTRY.md` §Performance Claims has, since before this pass, carried explicit
dispositions:

> | 6.2 μs mean latency, 780K msg/s | … | 🟡 Repository-reported, not independently
> verified — **downgrade** to "Target: <10μs, Prototype measurement: ~Xμs in local run" |
> | 46x faster than ROS2, 15x throughput, etc. | … | 🔴 **Not independently established** |

So this was not an open policy question — the registry had already decided, and the
decision had simply never been carried into the documents that make the claims. With real
measurements now in hand (F29-10), the `~Xμs` placeholder could finally be filled in:

* `README.md` — the "Prototype measurement repository-reported 6.2 μs avg, 780K msg/s"
  line replaced with a table of what was actually executed, the scheduler-bound caveat,
  and an explicit statement that `<10 μs` is a target, not a result. The
  "46× latency, 15× throughput" summary line is now labelled a design target and points at
  the registry.
* `COMPARISON.md` — a caveat added directly above the §2.1 latency table (which is the
  source of the 46x/42x/74x/112x ratios), and the "NROS Strengths" bullets reworded so the
  ratios are no longer presented bare.
* `EVIDENCE_REGISTRY.md` — both Performance Claims rows updated with the executed numbers
  and marked as applied; the SPSC row moved to BENCHMARKED (same-thread only) and the TCP
  row given the F29-01 defect, fix and verification.
* `docs/audit/verification.json` — rewritten. It still claimed branch
  `arena/01a0188d-nros` and recorded **every gate as `NOT_RUN`**; it now records the
  executed status of all eleven gates with the run IDs and the two remaining blockers.

No claim was strengthened anywhere. Every edit either scopes a claim down or replaces an
unverified number with a measured one plus its conditions.

### F29-12 (P1) — no executed memory-safety evidence existed for `nros-core`'s unsafe code. SANITIZER PASS ADDED

Miri is the only UB detector wired into CI and it has never produced a passing result
(F29-04), and loom was never wired up at all. So the crate that carries the whole safety
case — `MaybeUninit`, `drop_in_place`, raw pointer arithmetic over a shared ring — had
**zero executed memory-safety evidence**.

That gap is narrower than it looks. mrustc emits C, so the generated sources can simply be
recompiled with gcc's sanitizers; no Rust toolchain is needed. Done:

**All 8 suites, 54 tests, rebuilt with `-fsanitize=address,undefined
-fno-sanitize-recover=all` and run: zero sanitizer reports.** No heap-buffer-overflow, no
use-after-free, no double-free, no UBSan diagnostic (misalignment, signed overflow, null
deref, invalid shift).

```
CLEAN test-nros_types-asan      4 passed      CLEAN test-nros_sim-asan        7 passed
CLEAN test-nros_core-asan      20 passed      CLEAN test-nros_transport-asan  8 passed
CLEAN test-nros_node-asan       5 passed      CLEAN test-nros_cli-asan        3 passed
CLEAN test-nros_hal-asan        4 passed      CLEAN test-nros_studio-asan     3 passed
```

LeakSanitizer does report 1877 bytes leaked at exit from `test-nros_core-asan`. That was
chased down rather than assumed away, and it is **not** NROS:

* a control binary with no unsafe code at all (`test-nros_types-asan`, 4 trivial tests)
  leaks 317 bytes in 8 allocations, so there is a fixed runtime baseline;
* **0 leak frames have a symbol naming `RingBuffer`**;
* the deepest recurring frame is `std::thread::Thread::new`, and the only test function
  appearing in any leak stack is `test_spsc_ordering` — the one that spawns threads.
  Per-thread `Thread` objects are not freed by the mrustc libstd port;
* the drop-accounting tests that would catch a ring leak
  (`test_generic_t_destruction`, `test_drop_drains_all_occupied_slots`,
  `test_zst_with_drop_dropped_exactly_once`) all pass.

Two measurement errors of my own were caught and corrected while doing this, both of which
would have produced a false claim: grepping raw ASan frame lines matches the *binary path*
(`.../test-nros_core-asan+0x…`), which reported 96 "nros_core frames" where the true count
is 0; and this binary's crate mangles as `bin`, not `nros_core`, so the crate name is not a
usable symbol match at all. The committed probe extracts the symbol only.

**This is not a Miri substitute, and the probe says so in its header.** ASan/UBSan do not
detect reading uninitialized memory (that is MemorySanitizer, which needs every object file
including libstd plus an instrumented libc — not practical here), nor Rust-specific UB with
no C analogue: invalid reference or `&mut` alias construction, invalid enum discriminants,
`Pin` violations. A clean run narrows the risk; it does not close the Miri gap.

**Deliverable:** `tools/offline-mrustc/probes/sanitizer.sh` — rebuilds every test binary
from mrustc's generated C with sanitizers, runs them, fails on any report, and prints the
leak attribution so the runtime baseline stays visible instead of being silently ignored.
Verified end-to-end: `ALL SANITIZER PROBES CLEAN`, exit 0.

## 3. The stage-2 pin table vs the real 1.90.0 lockfile

The README's "Pinning facts (verified during Pass 27)" list does not match
`rustc-1.90.0-src/library/Cargo.lock`, which the same recipe downloads and which the
script itself calls "version *truth*":

| Crate | Recipe | 1.90.0 lockfile |
|---|---|---|
| `allocator-api2` | 0.2.21 | **not in lockfile** (both pinned URLs dead) |
| `foldhash` | 0.1.5 | **not in lockfile** |
| `equivalent` | 1.0.2 | **not in lockfile** |
| `addr2line` | 0.24.2 | 0.25.0 |
| `gimli` | 0.31.1 | 0.32.0 |
| `object` | 0.36.7 | 0.37.1 |
| `memchr` | 2.7.6 | 2.7.5 |
| `rustc-demangle` | `v0.1.24` (tag does not exist) | 0.1.25 (never tagged; `0.1.24` used) |
| `unicode-width` | 0.1.14 **and** 0.2.1 | 0.2.1 only |
| `adler2` | tag `v2.0.1` (no tags exist upstream) | 2.0.1 → pinned by commit `89a031a0` |

`allocator-api2` / `foldhash` / `equivalent` are hashbrown's *default-feature*
dependencies; `library/std/Cargo.toml` requests
`hashbrown = { version = "0.15", default-features = false, features = ['rustc-dep-of-std'] }`,
and the lockfile's `hashbrown 0.15.4` entry depends only on
`rustc-std-workspace-{alloc,core}`. They must not be vendored.

Two more facts worth keeping: `libm` is **not** a dependency crate here — it is pulled in
by source path (`#[path = "../../libm/src/libm_math.rs"]` in compiler_builtins'
`src/math/mod.rs`), so it must sit beside the vendored copy; and `compiler_builtins`
**must** keep its vendored copy (because `library/std` reaches it as a registry
dependency, `^0.1.2`) while its `[features]` lists must **not** be edited — the feature
set is what the rlib crate tag (`Hxx`) is derived from, and editing it made the built
tag diverge from the one `library/alloc` asked for.

---

## 4. What was verified green (executed, not inferred)

| Gate | Command | Result |
|---|---|---|
| Unit tests | 8 suite binaries from `crates/*/src/lib.rs` | **54 passed, 0 failed, 1 ignored** (`types` 4, `core` 20+1 ignored, `node` 5, `hal` 4, `sim` 7, `transport` 8, `cli` 3, `studio` 3) |
| Flake regression | `test-nros_transport` ×100; all suites ×30; 3 suites ×20 under 4× load | **0 failures** |
| Ring/IPC probes | `probes/ring-probe.rs` | `ALL RING PROBES PASS` |
| Distributed probes | `probes/dist-probe.rs` | `ALL DISTRIBUTED PROBES PASS` |
| Microbenchmark | `probes/microbench.rs` | `MICROBENCH OK` |
| Robustness fuzz | `probes/fuzz-head.rs` | `ALL ROBUSTNESS PROBES PASS` (210k+ adversarial inputs + hostile TCP sequences) |
| Compile-fail parity | `probes/compile-fail.sh` | `ALL COMPILE-FAIL PROBES PASS` (4/4 rejected) |
| Demos | 6 demo binaries | all exit 0 |
| Golden templates | `nros init` → `basic`, `mobile_base`, compiled + run | both exit 0 |
| Real macro chain + facade | minicargo over `crates/nros/`, then both examples | built and ran |
| Benchmark harness | `nros-core --bin bench`, 100 000 iterations ×6 | completes, `sent == received == 100000` (after F29-09) |
| Same-thread ring cost | `probes/microbench.rs` | 110.90 ns/op, 9.02M msg/s (Pass 27: 112 ns/op) |
| Memory safety (ASan+UBSan) | `probes/sanitizer.sh`, all 8 suites | **0 sanitizer reports**; leaks attributed to the mrustc libstd port, not NROS |
| Claim/evidence/representation | `nros-audit all` | exit 0, 354 PASS, 0 FAIL |
| Documentation representation | `scripts/validate-documentation-representation.py` | `DOCUMENTATION REPRESENTATION: PASS`, exit 0 |
| README ↔ CI consistency | 10 documented gates vs `ci.yml` | all present; 12 workspace members as claimed; clippy report-only as documented; benchmarks `continue-on-error` as documented |

## 5. What remains NOT verified

* **Borrow-checking and Miri.** No rustc, clippy or miri is obtainable here. Everything
  in §4 was produced by mrustc in 1.90 mode, which does not borrow-check and is not Miri.
  `cargo check`, `cargo test`, `cargo clippy` and `cargo fmt` *were* exercised by the
  real toolchain in CI on this branch (§5b); `cargo miri` was not, and cannot be from
  here. F29-12 partially substitutes — ASan+UBSan over all 54 tests found nothing — but
  that covers heap errors, not uninitialized reads or Rust-specific UB. F29-01's fix is
  verified behaviourally (240+ local runs, plus a green `cargo test` in CI), not by Miri.
* **The cause of the Miri failure** (F29-04). Job logs are on a blocked blob host, so it
  is still unknown whether `cargo miri` is even resolvable in CI, or whether it is
  reporting real undefined behaviour in `nros-core`.
  `docs/audit/F-29-ci-miri-diagnostics.patch` settles that in one run.
* **Benchmark claims as *performance guarantees*.** F29-10 establishes that 6.2 μs was
  never measured by anything, but the converse does not follow either: a shared,
  unpinned 2-vCPU sandbox cannot validate a "<10 μs" real-time target. Confirming that
  needs CPU pinning on the target hardware. Separately, `nros-core`'s in-tree
  (`benchmark_latency_monotonic`'s hard-coded `1000` has since been replaced with a real
  measurement — see F29-10.)
* **Anything requiring hardware.** HAL DMA remains `SimulatedDmaBuffer`.

---

## 5b. Independent confirmation from real CI on this branch

Pushing this branch triggered two runs on GitHub-hosted runners, i.e. the *real*
toolchain: [`32702101917`](https://github.com/Abdus2023/NROS/actions/runs/32702101917)
after the F29-01 fix, and
[`32704793107`](https://github.com/Abdus2023/NROS/actions/runs/32704793107) after the
F29-03 reformat.

| Job | Base `48069dce` | After F29-01 | After F29-03 | After F29-09 |
|---|---|---|---|---|
| **cargo test (workspace)** | **failure** (exit 101) | **success** | **success** | **success** |
| **cargo fmt --check** | **failure** | failure | **success** | **success** |
| **Benchmarks (report-only)** | never completed | never completed | never completed | **success** |
| cargo check (workspace, all targets) | success | success | success | success |
| cargo clippy (workspace) | success | success | success | success |
| nros init generates a buildable NROS project | success | success | success | success |
| Provenance / SHA manifest | success | success | success | success |
| Claim / evidence / representation gate | failure | failure | failure | failure |
| Safety gate (Miri, hard) | failure | failure | failure | failure |

**7 of 9 jobs green, up from 4 of 9 on the base commit** — and the ninth, `Benchmarks`,
had never once reached a completed state before F29-09 (run `32707535971` completed it and
uploaded a 681-byte `benchmark-results` artifact).

Every fix in this pass was confirmed by the real toolchain, not only by mrustc.

The two remaining red jobs are exactly the two this pass could not fix from here:

* **Representation gate** — the fix (F-20) is verified but not pushable; see F29-02.
* **Miri** — see F29-04. The step signature is unchanged by anything in this pass (which
  does not touch `nros-core`'s unsafe code): `Install Miri` **success**,
  `Miri on nros-core` **failure**, `Miri on nros-types` skipped.

## 6. Changes made in this pass

| File | Change |
|---|---|
| `crates/nros-transport/src/lib.rs` | F29-01 — `TcpTransport::receive` no longer discards a complete buffered frame on peer EOF; F29-07 — `ServiceDiscovery` announces to the bound port |
| `crates/nros-node/src/lib.rs` | F29-06 — wall-clock threshold in `cargo test` replaced with machine-independent bookkeeping assertions |
| `crates/nros-audit/src/main.rs` | F29-08 — `check_ci` / `check_claims` / `check_benchmarks` can now actually fail |
| `.github/workflows/ci.yml` | F29-02 — **prepared and verified, NOT committed**: `git apply docs/audit/F-20-ci-fetch-depth.patch` needs the `workflows` scope, which this session's token lacks. Applies cleanly; needs an owner to push |
| `tools/offline-mrustc/stage1-bootstrap.sh` | F29-05 items 1–5 |
| `tools/offline-mrustc/stage2-vendor-stdlib.sh` | F29-05 items 6–9; §3 pin-table correction |
| `tools/offline-mrustc/stage3-build-nros.sh` | F29-05 item 10 (suites now actually run, and fail the stage) |
| `tools/offline-mrustc/README.md` | Pinning facts corrected to the real 1.90.0 lockfile; new tricks recorded |
| `docs/audit/F-29-ci-miri-diagnostics.patch` | F29-04 — gate-neutral diagnostics that make the Miri failure reason readable via the API; `git apply --check` verified |
| `crates/nros-core/src/bin/bench.rs` | F29-09 — consumer exit condition no longer unreachable; publish instant enqueued before commit |
| `benchmarks/results_e2b-sandbox-2vcpu_20260824.json` | F29-10 — real, environment-stamped artifact with the scheduler-bound caveat and the conclusion on the published claims |
| `README.md`, `COMPARISON.md`, `EVIDENCE_REGISTRY.md` | F29-11 — the downgrade the registry had already ordered, applied; measured numbers replace the unverified ones |
| `docs/audit/verification.json` | F29-11 — rewritten from "every gate NOT_RUN" to the executed status of all eleven gates, with run IDs |
| `tools/offline-mrustc/probes/sanitizer.sh` | F29-12 — new: rebuild every test binary from mrustc's generated C with ASan+UBSan, run them, and attribute any leak |
| 26 `.rs` files under `crates/` | F29-03 — reformatted; see §7. Formatting-only, no behaviour change; `tests/compile_fail/` fixtures excluded |

## 7. The rustfmt reformat (F29-03)

The 26 non-fixture `.rs` files under `crates/` were reformatted. The changes are all
long-stable rustfmt defaults — one-line fn bodies expanded, struct literals expanded,
imports reordered/grouped, trailing-comment alignment — nothing gated on a
`style_edition`, and the repo has no `rustfmt.toml`.

Two deliberate exclusions:

* `crates/nros-core/tests/compile_fail/*.rs` (4 files). `cargo fmt --all` formats
  declared/auto-discovered *targets*; those files are trybuild **fixtures**, not targets,
  so the gate never inspects them. Reformatting them would only churn files whose
  committed `.stderr` quotes source lines verbatim.
* No `rustfmt.toml` was added — the gate should keep checking rustfmt's defaults.

**Verified locally:** the full offline stage 3 was re-run from a clean output directory
against the reformatted tree — 54 tests pass, 6 demos, both golden templates, both facade
examples (real `#[nros::node]` expansion), and all five probe suites still pass, with
`STAGE3_COMPLETE` / exit 0.

**Adjudicated by CI:** the rustfmt used here is a WASM build whose version could not be
determined, so instead of asserting that its output matches CI's stable rustfmt, the
reformat was pushed and the real gate was allowed to decide. Run `32704793107` reports
**`cargo fmt --check` → success**. The diff is formatting-only and carries no behaviour
change, which the local stage-3 re-run confirms independently.

## 8. Not changed, deliberately

* **The Miri workflow (F29-04).** The repo's F-25 patch is unapplied, and the branch
  whose run is titled "apply F-25 Miri toolchain fix" (`arena/01a03242-nros` @
  `dee4f028`) contains no `ci.yml` change at all — its only files are
  `AUDIT_PASS_28.md` and one `.arena/` file. So there is still **no evidence either way**
  on whether F-25 works, and applying an unverifiable workflow change on top of a gate
  that is already red would not be an improvement anyone could check.
* **The scope of `check_safety_invariants()` (F29-08).** Deciding which additional
  invariants it should enforce is a design decision, not a defect fix.
