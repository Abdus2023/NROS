//! NROS Claim Linter — per AUDIT Pass 11-12 recommendation
//! Checks: README ↔ Cargo.toml, README ↔ EVIDENCE_REGISTRY, AUDIT ↔ EVIDENCE_REGISTRY, DESIGN ↔ implementation, CI ↔ workflow files, benchmark claims ↔ artifacts
//! Run: cargo run -p nros-audit -- claims
//! Would be used in CI as DOC-GATE and Claim Strength gate

use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("claims");

    match cmd {
        "claims" => check_claims(),
        "workspace" => check_workspace_inventory(),
        "ci" => check_ci(),
        "benchmarks" => check_benchmarks(),
        "safety" => check_safety_invariants(),
        "all" => {
            check_workspace_inventory();
            check_claims();
            check_ci();
            check_benchmarks();
            check_safety_invariants();
        }
        _ => {
            println!("Usage: nros-audit [claims|workspace|ci|benchmarks|safety|all]");
            println!("  claims      — Check README claims vs EVIDENCE_REGISTRY claim_allowed");
            println!("  workspace   — Check README 8 crates vs Cargo.toml 11 crates (DOC-001)");
            println!("  ci          — Check .github/workflows/ci.yml exists on audited ref");
            println!("  benchmarks  — Check benchmark claims vs artifacts");
            println!("  safety      — Check Pass 24 soundness invariants in source");
            println!("  all         — Run all checks");
        }
    }
}

fn check_workspace_inventory() {
    println!("🔍 DOC-GATE: README ↔ Cargo.toml workspace inventory");
    let cargo_toml = fs::read_to_string("Cargo.toml").unwrap_or_default();
    let readme = fs::read_to_string("README.md").unwrap_or_default();

    let cargo_crates: Vec<String> = cargo_toml.lines()
        .filter(|l| l.contains("crates/"))
        .map(|l| l.trim().to_string())
        .collect();

    println!("Cargo.toml workspace members:");
    for c in &cargo_crates {
        println!("  {}", c);
    }

    let readme_mentions_8 = readme.contains("8 crates");
    let readme_mentions_11 = readme.contains("11 crates") || readme.contains("10 crates") || readme.contains("8 crates — 6/6");

    if readme_mentions_8 && cargo_crates.len() >= 10 {
        println!("❌ DOC-001: README says 8 crates but Cargo.toml has {} members (stale inventory)", cargo_crates.len());
        println!("   Fix: Update README to say {} crates", cargo_crates.len());
    } else {
        println!("✅ Workspace inventory: README mentions {} crates, Cargo.toml has {} members", 
            if readme_mentions_11 { "10/11" } else { "unknown" }, cargo_crates.len());
    }
}

fn check_claims() {
    println!("\n🔍 Claim Strength Gate: README ↔ EVIDENCE_REGISTRY");
    let readme = fs::read_to_string("README.md").unwrap_or_default();
    let evidence = fs::read_to_string("EVIDENCE_REGISTRY.md").unwrap_or_default();

    let checks = vec![
        ("Raft implemented", "README says Raft implemented", "SIMULATED", evidence.contains("Raft") && evidence.contains("SIMULATED")),
        ("6.2 μs benchmark", "README 6.2 μs", "repository-reported", evidence.contains("6.2 μs") && evidence.contains("repository-reported")),
        // Pass 24: CI workflow now lives on this branch. If it is absent, flag it;
        // if present, this check is satisfied (the dedicated check_ci() job does
        // deeper validation of Miri gating and nros-init).
        ("CI added", "README claims CI added", "workflow present", Path::new(".github/workflows/ci.yml").exists()),
        ("8 crates", "README 8 crates", "Cargo 10/11 crates", true), // checked in workspace gate
        ("DMA zero-copy", "README DMA zero-copy", "SIMULATED", evidence.contains("Real DMA") && evidence.contains("SPECIFIED")),
    ];

    for (claim, _desc, expected_status, condition) in checks {
        if condition {
            println!("⚠️  Potential mismatch: {} — expected status {} — check EVIDENCE_REGISTRY (manual review needed)", claim, expected_status);
        } else {
            println!("✅ Claim check: {} — no obvious mismatch", claim);
        }
    }

    // Check for executable fiction labeling
    let has_simulated_label = readme.contains("SIMULATED") || readme.contains("Evidence Taxonomy");
    if has_simulated_label {
        println!("✅ README has evidence taxonomy / SIMULATED labeling (good per AUDIT P1)");
    } else {
        println!("❌ README missing evidence taxonomy — should reference EVIDENCE_REGISTRY.md and label SIMULATED vs IMPLEMENTED");
    }

    // Check for claim_allowed
    if evidence.contains("claim_allowed") {
        println!("✅ EVIDENCE_REGISTRY has claim_allowed column (per AUDIT recommendation)");
    } else {
        println!("❌ EVIDENCE_REGISTRY missing claim_allowed — needed for claim linter");
    }
}

fn check_ci() {
    println!("\n🔍 CI Gate: .github/workflows/ci.yml existence on audited ref");
    let ci_path = Path::new(".github/workflows/ci.yml");
    let ci_alt = Path::new(".github/workflows/Ci.yml");
    if ci_path.exists() {
        println!("✅ CI workflow exists at {:?}", ci_path);
        let content = fs::read_to_string(ci_path).unwrap_or_default();
        // Pass 24: check that the Miri gate specifically doesn't suppress failures.
        // We must ignore comments and unrelated `|| echo` (e.g. toolchain detection),
        // so strip comment lines before searching.
        let code_only: String = content
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        let miri_suppressed = code_only
            .lines()
            .any(|l| l.contains("cargo miri") && l.contains("|| echo"));
        if miri_suppressed {
            println!("❌ CI-002: A `cargo miri` invocation is suppressed by `|| echo` — must be a hard failure");
        } else {
            println!("✅ CI Miri gate: no `cargo miri ... || echo` suppression found (hard failure) — fixes CI-002");
        }
        if content.contains("cargo run -p nros-cli --bin nros -- init")
            || content.contains("/target/debug/nros\" init")
            || content.contains("/target/debug/nros init")
        {
            println!("✅ CI nros-init golden test actually runs nros init (fixes CI-001)");
        } else {
            println!("⚠️  CI nros-init test may still be trivial — should run real nros init + cargo check");
        }
    } else if ci_alt.exists() {
        println!("⚠️  CI workflow exists at alternate capitalization {:?} — should be ci.yml lowercase", ci_alt);
    } else {
        println!("❌ CI-003: Documentation claims workflow that is not present on audited ref — .github/workflows/ci.yml returns 404");
        println!("   Fix: Add workflow file via GitHub web UI (needs workflows permission, cannot push via API with GitHub App token lacking workflows scope)");
        println!("   Content kept locally at .github/workflows/ci.yml (untracked) — needs manual addition");
        println!("   See docs/ARCHITECTURE.md and EVIDENCE_REGISTRY.md");
    }
}

fn check_benchmarks() {
    println!("\n🔍 Benchmark Claims ↔ Artifacts Gate (BENCH-001)");
    let comparison = fs::read_to_string("COMPARISON.md").unwrap_or_default();
    let evidence = fs::read_to_string("EVIDENCE_REGISTRY.md").unwrap_or_default();

    let has_46x = comparison.contains("46") && comparison.contains("faster");
    let has_6_2us = comparison.contains("6.2") && comparison.contains("μs");

    if has_46x || has_6_2us {
        println!("⚠️  COMPARISON.md contains headline performance claims (46×, 6.2μs) — must be labeled TARGET/HYPOTHESIS not PERFORMANCE per AUDIT");
        println!("   Evidence registry says: No ROS2 baseline in this repo — comparison not independently established");
        if evidence.contains("No ROS2 baseline") {
            println!("   ✅ Evidence registry correctly notes ROS2 baseline absence — good discipline");
        }
    }

    let results_path = Path::new("benchmarks/results.json");
    if results_path.exists() {
        println!("✅ Benchmark artifact exists at {:?}", results_path);
        let content = fs::read_to_string(results_path).unwrap_or_default();
        let required_fields = ["cpu_model", "os", "rustc_version", "commit", "timestamp", "capacity", "iterations", "affinity", "message_size", "p50_us", "p99_us"];
        for field in required_fields {
            if content.contains(field) {
                println!("  ✅ Contains field: {}", field);
            } else {
                println!("  ❌ Missing field: {} — required per Pass 7 §12", field);
            }
        }
        if content.contains("repository-reported") || content.contains("TEMPLATE") {
            println!("  ⚠️  Artifact is TEMPLATE not independently verified — should be labeled as such, not as verified benchmark");
        }
    } else {
        println!("❌ Benchmark artifact missing at {:?} — run cargo run -p nros-core --bin bench -- --output benchmarks/results.json", results_path);
    }

    // Check for monotonic clock usage
    let core_lib = fs::read_to_string("crates/nros-core/src/lib.rs").unwrap_or_default();
    if core_lib.contains("MonotonicInstant") || core_lib.contains("Instant::now") {
        println!("✅ Core uses monotonic clock (Instant) per CORE-007 fix");
    } else {
        println!("❌ Core still uses SystemTime for latency — should use Instant");
    }

    // Check for as_mut() over uninit
    if core_lib.contains("pub fn as_mut(&mut self) -> &mut T") && core_lib.contains("MaybeUninit") {
        // Need to distinguish WriteGuard vs InitializedWriteGuard
        // WriteGuard should NOT have as_mut() returning &mut T over uninit
        // InitializedWriteGuard can have as_mut() because T initialized
        if core_lib.contains("impl<'a, T> WriteGuard") && core_lib.contains("pub fn as_mut(&mut self) -> &mut T") {
            // Check if it's in WriteGuard (uninit) vs InitializedWriteGuard (init)
            // Simple heuristic: if file contains "WriteGuard" as_mut and also "InitializedWriteGuard" as_mut, the first may be problematic
            println!("⚠️  Potential CORE-011: WriteGuard::as_mut() -> &mut T over MaybeUninit may still exist — should be as_mut_uninit() only for uninit, as_mut() only for InitializedWriteGuard");
        }
    } else {
        println!("✅ No obvious as_mut() over uninit in WriteGuard (checked via grep)");
    }
}

/// Pass 24 safety-invariants gate (SAFETY-GATE).
///
/// Greps the source for the structural markers that establish the soundness fixes
/// from AUDIT_PASS_24. These are *string* checks, not a compiler — they guard
/// against accidental regression of the safe-API surface between proper Miri runs.
/// Each check is a necessary (not sufficient) condition; Miri/trybuild remain the
/// authoritative evidence.
fn check_safety_invariants() {
    println!("\n🔒 SAFETY-GATE: Pass 24 soundness invariants (structural)");

    let core_lib = fs::read_to_string("crates/nros-core/src/lib.rs").unwrap_or_default();
    let transport_lib = fs::read_to_string("crates/nros-transport/src/lib.rs").unwrap_or_default();
    let facade_lib = fs::read_to_string("crates/nros/src/lib.rs").unwrap_or_default();
    let node_lib = fs::read_to_string("crates/nros-node/src/lib.rs").unwrap_or_default();

    let mut failures = 0;

    // I-001 / CORE-011: the safe closure-based initializer must NOT exist.
    if core_lib.contains("pub fn init_with<F>") {
        println!("❌ I-001 REGRESSED: safe WriteGuard::init_with() is present again — safe UB (no-op closure -> commit -> deref uninit)");
        failures += 1;
    } else {
        println!("✅ I-001: no safe init_with(); field-by-field init requires unsafe init_with_unchecked()");
    }

    // I-001: write_value must exist as the 100%-safe initializer.
    if core_lib.contains("pub fn write_value(self, value: T)") {
        println!("✅ I-001: write_value(self, T) safe initializer present");
    } else {
        println!("❌ I-001: write_value(self, T) missing — no safe path to InitializedWriteGuard");
        failures += 1;
    }

    // CORE-011: as_mut_ptr must be declared `unsafe fn`, not a safe fn.
    // We look for the safe-form `pub fn as_mut_ptr(&self)` which would be unsound.
    if core_lib.contains("pub fn as_mut_ptr(&self)") {
        println!("❌ CORE-011 REGRESSED: as_mut_ptr() is a safe fn again — must be `pub unsafe fn`");
        failures += 1;
    } else if core_lib.contains("pub unsafe fn as_mut_ptr(&self)") {
        println!("✅ CORE-011: as_mut_ptr() is `pub unsafe fn` (raw init escape hatch marked unsafe)");
    }

    // CORE-014: only InitializedWriteGuard exposes commit (structurally: commit is
    // defined in the InitializedWriteGuard impl, not WriteGuard). We check that the
    // WriteGuard impl block does not contain `pub fn commit`.
    if let Some(write_guard_block) = core_lib.split("impl<'a, T> WriteGuard<'a, T>").nth(1) {
        let block_until_next = write_guard_block.split("// ── InitializedWriteGuard").next().unwrap_or(write_guard_block);
        if block_until_next.contains("pub fn commit") {
            println!("❌ CORE-014 REGRESSED: WriteGuard exposes commit() — uninitialized commit possible");
            failures += 1;
        } else {
            println!("✅ CORE-014: WriteGuard has no commit(); only InitializedWriteGuard can commit");
        }
    }

    // CORE-015: ReadGuard must NOT implement DerefMut.
    if core_lib.contains("impl<'a, T> DerefMut for ReadGuard") {
        println!("❌ CORE-015 REGRESSED: ReadGuard implements DerefMut — consumer can mutate published data");
        failures += 1;
    } else {
        println!("✅ CORE-015: ReadGuard is immutable (Deref only, no DerefMut)");
    }

    // I-002/I-003: Producer/Consumer must not be Clone (type-enforced SPSC).
    // We check there is no `impl<T> Clone for Producer` / Consumer.
    for endpoint in &["Producer", "Consumer"] {
        let pattern = format!("impl<T> Clone for {}", endpoint);
        if core_lib.contains(&pattern) {
            println!("❌ I-002/003 REGRESSED: {} is Clone — SPSC role enforcement broken", endpoint);
            failures += 1;
        }
    }
    if !core_lib.contains("impl<T> Clone for Producer") && !core_lib.contains("impl<T> Clone for Consumer") {
        println!("✅ I-002/003: Producer/Consumer are not Clone (single endpoints enforced by type system)");
    }

    // TRANSPORT-001: MessageHeader::SIZE must be the explicit 36-byte wire size,
    // NOT size_of (48 due to #[repr(C)] padding). Only inspect code (not comments),
    // since the remediation comment itself references `size_of`.
    let uses_sizeof = transport_lib.lines().any(|l| {
        let code = l.split("//").next().unwrap_or("");
        code.contains("size_of::<MessageHeader>") && code.contains("SIZE")
    });
    if uses_sizeof {
        println!("❌ TRANSPORT-001 REGRESSED: SIZE uses size_of (48) instead of explicit 36-byte wire size — packets misparse");
        failures += 1;
    } else if transport_lib.contains("pub const SIZE: usize = 36") {
        println!("✅ TRANSPORT-001: MessageHeader::SIZE = 36 (matches to_bytes/from_bytes wire format)");
    } else {
        println!("⚠️  TRANSPORT-001: could not confirm SIZE == 36 — inspect MessageHeader");
    }

    // E0116: nros-node must not inherent-impl the Timestamp alias to a foreign type.
    // Check only non-comment lines (the audit report explains the removal in a comment).
    let node_has_impl = node_lib.lines().any(|l| {
        let code = l.split("//").next().unwrap_or("").trim();
        code.starts_with("impl Timestamp")
    });
    if node_has_impl {
        println!("❌ E0116 REGRESSED: nros-node has `impl Timestamp` for a foreign type alias");
        failures += 1;
    } else {
        println!("✅ E0116: no foreign-type inherent impl in nros-node");
    }

    // E0252: nros facade prelude must source canonical types from ONE crate.
    // Heuristic: it must NOT import Twist/Vector3 from both nros_core and nros_node.
    let prelude = facade_lib.split("pub mod prelude").nth(1).unwrap_or("");
    let core_line = prelude.lines().any(|l| l.contains("nros_core::{") && (l.contains("Twist") || l.contains("Vector3")));
    let node_line = prelude.lines().any(|l| l.contains("nros_node::{") && (l.contains("Twist") || l.contains("Vector3")));
    if core_line && node_line {
        println!("❌ E0252 REGRESSED: prelude imports Twist/Vector3 from BOTH nros_core and nros_node");
        failures += 1;
    } else if prelude.contains("nros_types") {
        println!("✅ E0252: prelude sources canonical domain types from nros_types (single source)");
    }

    // RingBuffer Drop must iterate by count (wrapping_sub), not `for idx in read..write`,
    // which leaks slots when u64 indices wrap (Pass 24 §4.6).
    let drop_block = core_lib.split("impl<T> Drop for RingBuffer<T>").nth(1);
    if let Some(block) = drop_block {
        let body = block.split("\n}\n").next().unwrap_or(block);
        if body.contains("for idx in read..write") {
            println!("❌ Drop REGRESSED: RingBuffer uses `for idx in read..write` (leaks on u64 wraparound)");
            failures += 1;
        } else if body.contains("wrapping_sub(read)") {
            println!("✅ Drop: RingBuffer drains by wrapping_sub count (u64-wraparound safe)");
        }
    }

    println!("\n🔒 SAFETY-GATE result: {} failure(s)", failures);
    if failures > 0 {
        // Exit non-zero so CI doc-gate fails on structural regression.
        std::process::exit(1);
    }
}
