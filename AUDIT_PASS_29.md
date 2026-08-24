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

### F29-03 (P0) — `cargo fmt --check` fails 40/40: the workspace has never been formatted. NOT FIXED (needs CI's exact rustfmt)

Measured with rustfmt compiled to WASM (`@scalar/rust-fmt` 0.2.0, `edition = 2021`):
**27 of 33** `.rs` files under `crates/` differ from rustfmt's output. The 6 that are
already clean show the tool is discriminating rather than always differing.

Differing: every file in `nros-audit`, `nros-cli`, `nros-core` (incl. `bin/bench.rs`,
`executor.rs`, `tests/compile_fail/two_producers_from_one_channel.rs`),
`nros-distributed`, `nros-hal`, `nros-macros`, `nros-node`, `nros-sim`, `nros-studio`,
`nros-transport` (incl. `examples/compression.rs`), `nros-types`,
`crates/nros/src/lib.rs`, `crates/nros/examples/vertical_slice.rs`.

*Why this was not auto-fixed:* the WASM rustfmt's version is not necessarily the same
build as CI's stable rustfmt, so applying its output could leave the gate red while
producing a large, hard-to-review diff. The correct fix is one `cargo fmt --all`
committed from a machine with the toolchain, then keeping the gate hard.

---

### F29-04 (P0) — Miri gate fails 40/40; cause not decodable from this sandbox. NOT FIXED

Step-level detail for job `97332119664`: `Install Miri` **success**, `Miri on nros-core`
**failure** (exit 1), `Miri on nros-types` skipped. There is no NROS-authored
annotation for it, and the raw log is on the blocked blob host, so the failure cannot
be decoded here.

The repo's own `docs/audit/F-25-ci-miri-toolchain.patch` argues the cause is
environmental (`RUSTUP_TOOLCHAIN=stable` outranking `rustup default nightly`) and
switches to `rustup toolchain install nightly --component miri` + `cargo +nightly`.
**That patch is also unapplied, and this pass could not verify its hypothesis** — so it
was deliberately *not* applied. Note the step-level evidence above is in mild tension
with F-25's premise: the install step *succeeded*, which is not what F-25 predicts.

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

### F29-06 (P2) — `nros-node` keeps a wall-clock assertion inside `cargo test`

`crates/nros-node/src/lib.rs:764`

```rust
assert!(avg_us < 100.0, "avg {} μs too high", avg_us);
```

`crates/nros-core/src/lib.rs:612` states the opposite rule for the same repo:
`// ── Tests — Correctness only, no perf asserts (fixes CORE-008) ──`. This is a latent
flake on a loaded runner. It did **not** fire in 240 local runs nor in 20 runs under 4×
CPU load, so it is reported rather than "fixed" — but it is the same class of defect as
F29-01 and belongs behind `#[ignore]` with the other benchmarks.

### F29-07 (P2) — `ServiceDiscovery` announces to port 0

`crates/nros-transport/src/lib.rs:1063`

```rust
broadcast_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), bind_port),
```

`bind_port` is the *requested* port, not the port actually bound. `ServiceDiscovery::new(0)`
(the documented "bind Any" path, and what `test_service_discovery` uses) yields
`255.255.255.255:0`, so `announce()`'s `send_to` can never reach a real listener. The
error is swallowed (`let _ = ...`) and `discover()` only reads a local `HashMap`, so the
test passes regardless. Should use `socket.local_addr()?.port()`.

### F29-08 (P2) — three `nros-audit` sub-gates can never fail

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

---

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
| Claim/evidence/representation | `nros-audit all` | exit 0, 354 PASS, 0 FAIL |
| Documentation representation | `scripts/validate-documentation-representation.py` | `DOCUMENTATION REPRESENTATION: PASS`, exit 0 |
| README ↔ CI consistency | 10 documented gates vs `ci.yml` | all present; 12 workspace members as claimed; clippy report-only as documented; benchmarks `continue-on-error` as documented |

## 5. What remains NOT verified

* **`cargo fmt --check`, `cargo check`, `cargo test`, `cargo clippy`, `cargo miri` with
  the real toolchain.** No rustc/rustfmt/clippy/miri is obtainable here. Everything in
  §4 was produced by mrustc 1.90-mode, which does not borrow-check and is not Miri.
  F29-01's fix is therefore verified *behaviourally* (240+ runs), not by Miri.
* **The cause of the Miri failure** (F29-04) — logs are on a blocked host.
* **Whether the WASM rustfmt's output matches CI's stable rustfmt** (F29-03).
* **Benchmarks as performance claims.** The numbers in `benchmarks/results.json` remain
  repository-reported; nothing in this pass independently validates the
  "6.2 μs / 780K msg/s" or "46× latency" claims, and `nros-core`'s in-tree
  `benchmark_latency_monotonic` still pushes a hard-coded `1000` with a `TODO` where the
  real publish-instant delta belongs.
* **Anything requiring hardware.** HAL DMA remains `SimulatedDmaBuffer`.

---

## 6. Changes made in this pass

| File | Change |
|---|---|
| `crates/nros-transport/src/lib.rs` | F29-01 — `TcpTransport::receive` no longer discards a complete buffered frame on peer EOF |
| `.github/workflows/ci.yml` | F29-02 — **prepared and verified, NOT committed**: `git apply docs/audit/F-20-ci-fetch-depth.patch` needs the `workflows` scope, which this session's token lacks. Applies cleanly; needs an owner to push |
| `tools/offline-mrustc/stage1-bootstrap.sh` | F29-05 items 1–5 |
| `tools/offline-mrustc/stage2-vendor-stdlib.sh` | F29-05 items 6–9; §3 pin-table correction |
| `tools/offline-mrustc/stage3-build-nros.sh` | F29-05 item 10 (suites now actually run, and fail the stage) |
| `tools/offline-mrustc/README.md` | Pinning facts corrected to the real 1.90.0 lockfile; new tricks recorded |

Not changed, deliberately: the rustfmt reformat (F29-03), the Miri workflow (F29-04),
the `nros-node` perf assert (F29-06), `ServiceDiscovery` (F29-07) and the audit
sub-gates (F29-08) — each is reported with its evidence and left to an owner decision.
