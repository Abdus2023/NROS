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
        "all" => {
            check_workspace_inventory();
            check_claims();
            check_ci();
            check_benchmarks();
        }
        _ => {
            println!("Usage: nros-audit [claims|workspace|ci|benchmarks|all]");
            println!("  claims      — Check README claims vs EVIDENCE_REGISTRY claim_allowed");
            println!("  workspace   — Check README 8 crates vs Cargo.toml 11 crates (DOC-001)");
            println!("  ci          — Check .github/workflows/ci.yml exists on audited ref");
            println!("  benchmarks  — Check benchmark claims vs artifacts");
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
        ("CI added", "README claims CI added", "workflow absent", !Path::new(".github/workflows/ci.yml").exists()),
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
        if content.contains("|| echo") {
            println!("❌ CI-002: Miri safety gate suppresses failures via || echo — must be hard failure (no || echo)");
        } else {
            println!("✅ CI Miri gate hard failure (no || echo) — fixes CI-002");
        }
        if content.contains("cargo run -p nros-cli --bin nros -- init") {
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
