//! NROS Claim Linter and repository representation gate.

mod representation;

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
        "representation" => representation::run(),
        "all" => {
            check_workspace_inventory();
            check_claims();
            check_ci();
            check_benchmarks();
            check_safety_invariants();
            representation::run();
        }
        _ => {
            println!("Usage: nros-audit [claims|workspace|ci|benchmarks|safety|representation|all]");
        }
    }
}

fn check_workspace_inventory() {
    println!("🔍 DOC-GATE: workspace inventory");
    let cargo_toml = fs::read_to_string("Cargo.toml").unwrap_or_default();
    let readme = fs::read_to_string("README.md").unwrap_or_default();
    let cargo_crates: Vec<&str> = cargo_toml.lines().filter(|l| l.contains("crates/")).collect();
    println!("Cargo.toml workspace members: {}", cargo_crates.len());
    if readme.contains("8 crates") && cargo_crates.len() >= 10 {
        println!("❌ DOC-001: stale README crate inventory");
    } else {
        println!("✅ Workspace inventory does not show the known stale-8-crates mismatch");
    }
}

fn check_claims() {
    println!("🔍 Claim Strength Gate");
    let readme = fs::read_to_string("README.md").unwrap_or_default();
    let evidence = fs::read_to_string("EVIDENCE_REGISTRY.md").unwrap_or_default();
    if evidence.contains("SIMULATED") || readme.contains("SIMULATED") {
        println!("✅ Simulation/evidence labeling present");
    } else {
        println!("⚠️ Evidence taxonomy labeling not detected");
    }
    if evidence.contains("claim_allowed") {
        println!("✅ Evidence registry exposes claim_allowed");
    } else {
        println!("⚠️ Evidence registry claim_allowed field not detected");
    }
}

fn check_ci() {
    println!("🔍 CI Gate");
    let active = Path::new(".github/workflows/ci.yml");
    let staged = Path::new("docs/ci.yml");
    if active.exists() {
        println!("✅ CI workflow active: {}", active.display());
    } else if staged.exists() {
        println!("⚠️ CI workflow staged: {}", staged.display());
    } else {
        println!("❌ CI workflow not found");
    }
}

fn check_benchmarks() {
    println!("🔍 Benchmark Claims ↔ Artifacts Gate");
    let results = Path::new("benchmarks/results.json");
    if results.exists() {
        println!("✅ Benchmark artifact exists");
    } else {
        println!("⚠️ Benchmark artifact not present");
    }
}

fn check_safety_invariants() {
    println!("🔒 SAFETY-GATE: structural source checks");
    let core = fs::read_to_string("crates/nros-core/src/lib.rs").unwrap_or_default();
    let mut failures = 0;
    if core.contains("pub fn init_with<F>") { failures += 1; println!("❌ safe init_with regression"); }
    if core.contains("pub fn as_mut_ptr(&self)") && !core.contains("pub unsafe fn as_mut_ptr(&self)") { failures += 1; println!("❌ safe as_mut_ptr regression"); }
    if failures == 0 { println!("✅ structural safety checks passed"); } else { println!("❌ {} safety regression(s)", failures); }
}
