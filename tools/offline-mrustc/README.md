# Offline Rust toolchain skill kit (mrustc) — NROS Pass 27

This directory is the **surviving backup of the offline verification toolchain, its
recipes ("tricks"), and the probe suite** used by AUDIT Pass 27 (2026-08-22/23). The
live toolchain was hosted on sandbox storage that was wiped *twice* during the pass
(`/tmp` first, then `/home/user/toolchain`); this kit exists so the whole chain — and
the know-how — no longer depends on any sandbox filesystem.

Produced evidence (all executed, recorded in `AUDIT_PASS_27.md` §1–§11.F): 54 unit
tests green across 8 crates, adversarial ring probes, distributed logic probes,
210k+-input robustness fuzz, compile-fail parity, golden templates, real-macro facade
(substitute-free chain: real rust 1.90.0 `library/` tree + real pinned vendored deps
+ real proc-macro plugins built from upstream sources).

## When you need this

Any sandbox/agent environment where `crates.io` and `static.rust-lang.org` are
unreachable (no cargo, no rustc, no rustup) but **github.com / codeload.github.com
are reachable**. Everything lands on disk in one root (recommended:
`/home/user/toolchain` — persistent, unlike `/tmp`).

## The recipe (three stages)

```bash
bash stage1-bootstrap.sh        # zlib, mrustc (master), rustc-1.90.0 source tree, minicargo patches
bash stage2-vendor-stdlib.sh    # vendored std deps + macro chain from GitHub tag tarballs
bash stage3-build-nros.sh       # NROS rlibs/tests/demos/golden + real-macro facade + probes
bash probes/compile-fail.sh     # negative-compile parity check (trybuild equivalent)
bash snapshot-dance.sh          # 2-commit snapshot re-pin discipline (repo hygiene)
```

Stage 3 auto-runs the probe binaries at the end. Expected good output ends with:
`ALL RING PROBES PASS`, `ALL DISTRIBUTED PROBES PASS`, `MICROBENCH OK`,
`ALL ROBUSTNESS PROBES PASS`, `ALL COMPILE-FAIL PROBES PASS`.

## The tricks (each one cost real debugging time — do not relearn them)

1. **Toolchain env (must be exported everywhere, always):**
   `RUSTC_VERSION=1.90.0 MRUSTC_TARGET_VER=1.90 OUTDIR_SUF=-1.90.0
   MINICARGO_DEFER_CODEGEN=0 PARLEVEL=2`
   Without `MRUSTC_TARGET_VER=1.90`, mrustc master silently operates in its *1.29*
   mode, tries to fetch rustc-1.29.0 sources, and dies on network. `OUTDIR_SUF` must
   match mrustc's computed output-dir naming. `MINICARGO_DEFER_CODEGEN=0` avoids a
   deferred-codegen scheduling deadlock (`BUG: Nothing runnable`) — if you ever hit
   it anyway, codegen steps are emitted as `<artifact>_cmd.txt`; run
   `x86_64-linux-gnu-gcc @"file_cmd.txt"` by hand and re-run make (semantically
   identical).
2. **No crates.io → GitHub tag tarballs.** Every dependency comes from its upstream
   GitHub repository tag via `codeload.github.com/<org>/<repo>/tar.gz/<ref>`.
   Version *truth* comes from `rustc-1.90.0-src/library/Cargo.lock` inside the rust
   source tree (stage 2 prints it for cross-checking).
3. **`make RUSTCSRC` tarball trick.** mrustc expects `rustc-1.90.0-src.tar.gz` with the
   top-level directory named exactly `rustc-1.90.0-src/`. The codeload rust tag
   tarball extracts to `rust-1.90.0/` — rename before re-tarring or mrustc can't
   find it.
4. **minicargo source patches (stage 1 applies them):**
   - `tools/minicargo/manifest.cpp`: tolerate unknown `[workspace]` /
     `[workspace.package]` keys (NROS crates inherit version/authors/... from the
     workspace — minicargo errors on unknown keys upstream).
   - `tools/minicargo/toml.h`: `as_string()` must coerce Integer/Boolean values to
     their textual form (declare `m_str_value` mutable) — otherwise `version = 1.99.0`-
     style numeric-looking fields trip it.
   Anchor strings can drift on mrustc master; the patch script prints FAILED anchors
   loudly instead of silently skipping.
5. **compiler_builtins ↔ core build ordering (determinism fix, learned the hard way).**
   The vendored `compiler_builtins` manifest must keep a **non-optional**
   `core = { path = "../../library/core" }` dependency. Deleting it (the first
   working recipe did) leaves minicargo without an edge; with `-j2` it can schedule
   compiler_builtins before `libcore.rlib` exists → `Unable to locate crate 'core'`
   at t=0. The edge is semantically neutral (only `#[cfg(test)] extern crate core`
   exists in its sources; rustc wires compiler_builtins through
   `rustc-std-workspace-core`, not directly).
6. **Downstream minicargo builds need `-L <stdlib output dir>`** (e.g.
   `-L .../mrustc-master/output-1.90.0`) or every crate fails with
   `Unable to locate crate 'std'`.
7. **`--script-overrides` obligations.** Any vendored crate with a `build.rs` needs a
   replay file `build_<crate>.txt`; contents must equal what that build.rs would emit
   for REAL stable rustc 1.90.x (verified line-by-line against upstream):
   - `build_proc-macro2.txt`: `cargo:rustc-cfg=wrap_proc_macro`
   - `build_quote.txt`: (empty)
   - `build_syn.txt`: `cargo:rustc-cfg=check_cfg` + `cargo:rustc-cfg=syn_disable_nightly_tests`
   - `unicode-ident` has no build.rs → no file needed.
   (None of the old-rustc `no_*` cfgs, and none of the nightly `proc_macro_span*`
   cfgs, apply on 1.90 stable.)
8. **mrustc builds proc-macro crates as executables** (`libnros_macros-plugin`).
   mrustc handles the `--extern <name>=<plugin>` path at expansion time. If a final
   bin link line ever contains the plugin path, strip it from `<bin>_cmd.txt` and
   re-run the emitted gcc command (rustc never links proc-macros into downstream
   artifacts — identical semantics).
9. **Missing output dirs cause the nondescript** `Failed to open ... for writing`.
   `mkdir -p` your output root first.
10. **nros-node tests need explicit externs** (`--extern nros_types=... --extern
    nros_core=...`) or resolution fails with `Cannot find component 0 of crate::nros_types::...`.
11. **nros-distributed `--test` harness crashes mrustc's typechecker** ("Spare rules
    left after typecheck stabilised"). Treat a crash at an inference site as
    **indeterminate**, never as "toolchain shrug": Pass 27's crash sat exactly on an
    un-annotated `DistributedState::new(...)` that real rustc rejects with E0282
    (CI run 32601114459 found it first — F-22). Correct offline protocol: run that
    crate's test bodies as probe binaries (stage 3 does) AND re-check the crash site
    for genuine ambiguity.
12. **mrustc diagnostic irregularity is not a repo defect:** 3/4 compile-fail cases
    die with clean typeck errors; `mutable_read_guard` dies at Trans Enumerate
    (`Item not found ... DerefMut::deref_mut`) — rejection is still total; rustc
    emits E0596. Parity table in AUDIT_PASS_27 §11.E.
13. **`nros init` templates are self-contained** (no facade/macro deps) — validate the
    golden path with a bare mrustc compile + run.
14. **The repo gates need full git history in CI** (`fetch-depth: 0`): the
    representation gate resolves git blob SHA-1s at the snapshot's pinned
    `source_revision` (HEAD~1 in the 2-commit dance). Shallow clones break the
    doc-gate (F-20; patch at `docs/audit/F-20-ci-fetch-depth.patch`).
15. **Snapshot dance discipline (repo convention):** content commit → recompute git
    blob SHA-1s of the 4 representation + 5 documentation manifests at that commit →
    re-pin both `snapshot.yaml` files → second commit. `snapshot-dance.sh` automates
    it; the representation gate and `scripts/validate-documentation-representation.py`
    must both pass after each dance.
16. **Evidence grammar:** SPECIFIED / SCAFFOLDED / SIMULATED / IMPLEMENTED / TESTED.
    CI configured ≠ CI passed (EVID-002); a queued run produces no evidence; a harness
    crash is indeterminate (see 11). Session pass numbering continues the AUDIT_PASS_*
    series (this kit was produced during Pass 27).
17. **Sandbox reality:** `/tmp` is wiped on suspension; `/home/user` survived once and
    was wiped once. **The Git repository is the only reliable storage** — which is why
    this kit now lives here.

## Pinning facts (verified during Pass 27)

- rustc `1.90.0` source tree (mrustc then reports `1.90.100`), mrustc master
  (thepowersgang/mrustc) as of 2026-08-22, system gcc 12.2
- `library/backtrace` in the rust tree is a **gitlink** (submodule-less tarball
  subtree) — restore it from `rust-lang/backtrace-rs @ b65ab935fb2e0d59dba8966ffca09c9cc5a5f57c`
  (crate 0.3.75; matches the lockfile's addr2line 0.24.2 / object 0.36.7 / gimli 0.31.1)
- macro chain (real sources, from their GitHub tags): unicode-ident 1.0.24 →
  proc-macro2 1.0.107 → quote 1.0.47 → syn 2.0.119
- std vendored crates: resolve/verify against `rustc-1.90.0-src/library/Cargo.lock`;
  Pass 27 used libc 0.2.174, cfg-if 1.0.1, hashbrown 0.15.4 (+ allocator-api2 0.2.21,
  equivalent 1.0.2, foldhash), addr2line 0.24.2, gimli 0.31.1, object 0.36.7,
  miniz_oxide 0.8.x (+ adler2), memchr 2.7.x, getopts 0.2.23, unicode-width (0.x and
  0.2.x — two lockfile entries), rustc-demangle 0.1.24; plus in-tree copies of
  compiler_builtins/libm/rustc-std-workspace-* dropped into vendor/ with the path
  fixups in stage 2.

## Known hardware/environment numbers this chain produced

- Host: 2-vCPU sandbox, gcc 12.2, Linux x86_64
- Same-thread SPSC probe: ~8.9 M msg/s (112 ns/op), 64 B payload, cap 1024
- Threaded benchmark artifact: `benchmarks/results_e2b-sandbox-2vcpu_20260822.json`
  (1.57 M msg/s under contention) — the canonical committed record
