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
            println!(
                "Usage: nros-audit [claims|workspace|ci|benchmarks|safety|representation|all]"
            );
        }
    }
}

fn gate_fail(msg: String) -> ! {
    // Pass 27 observability fix: print each hard failure BOTH as prose and as a GitHub
    // Actions workflow-command annotation, so the reason is visible on the check-run even
    // where raw job logs are unreachable (annotations ride the API, logs ride blob hosts).
    println!("❌ {}", msg);
    println!(
        "::error title=NROS audit gate::{}",
        msg.replace('%', "%25")
            .replace('\r', "%0D")
            .replace('\n', "%0A")
    );
    std::process::exit(1);
}

fn check_workspace_inventory() {
    println!("🔍 DOC-GATE: workspace inventory");
    let cargo_toml = fs::read_to_string("Cargo.toml").unwrap_or_default();
    let readme = fs::read_to_string("README.md").unwrap_or_default();
    let cargo_crates: Vec<&str> = cargo_toml
        .lines()
        .filter(|l| l.contains("crates/"))
        .collect();
    println!("Cargo.toml workspace members: {}", cargo_crates.len());
    if readme.contains("8 crates") && cargo_crates.len() >= 10 {
        gate_fail("DOC-001: stale README crate inventory".to_string());
    } else {
        println!("✅ Workspace inventory does not show the known stale-8-crates mismatch");
    }
}

fn check_claims() {
    println!("🔍 Claim Strength Gate");
    let readme = fs::read_to_string("README.md").unwrap_or_default();
    let evidence = fs::read_to_string("EVIDENCE_REGISTRY.md").unwrap_or_default();
    // Pass 29 (F29-08): these branches used to print ⚠️ and fall through, so the
    // sub-gate could never fail — `nros-audit -- claims` exited 0 even with no evidence
    // taxonomy at all. README documents this as a gate, so make it one.
    if evidence.contains("SIMULATED") || readme.contains("SIMULATED") {
        println!("✅ Simulation/evidence labeling present");
    } else {
        gate_fail(
            "DOC-002: evidence taxonomy labeling not detected in README.md or EVIDENCE_REGISTRY.md"
                .to_string(),
        );
    }
    if evidence.contains("claim_allowed") {
        println!("✅ Evidence registry exposes claim_allowed");
    } else {
        gate_fail("DOC-003: EVIDENCE_REGISTRY.md does not expose claim_allowed".to_string());
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
        // Pass 29 (F29-08): was `println!("❌ ...")` followed by a fall-through to a
        // successful exit, so a missing workflow did not fail the gate.
        gate_fail(
            "CI-004: no CI workflow found (.github/workflows/ci.yml or docs/ci.yml)".to_string(),
        );
    }
}

fn check_benchmarks() {
    println!("🔍 Benchmark Claims ↔ Artifacts Gate");
    let results = Path::new("benchmarks/results.json");
    if results.exists() {
        println!("✅ Benchmark artifact exists");
    } else {
        // Pass 29 (F29-08): README quotes benchmark figures, so a missing artifact is a
        // claim-without-evidence, not a warning. (The artifact being *present* still does
        // not validate the numbers — see the `benchmark_artifact_is_not_independent_validation`
        // invariant enforced by the representation gate.)
        gate_fail(
            "BENCH-005: benchmarks/results.json missing while README quotes benchmark figures"
                .to_string(),
        );
    }
}

fn check_safety_invariants() {
    println!("🔒 SAFETY-GATE: structural source checks");
    let core = fs::read_to_string("crates/nros-core/src/lib.rs").unwrap_or_default();
    let mut failures = 0;
    if core.contains("pub fn init_with<F>") {
        failures += 1;
        println!("❌ safe init_with regression");
        println!("::error title=NROS safety gate::safe init_with regression");
    }
    if core.contains("pub fn as_mut_ptr(&self)")
        && !core.contains("pub unsafe fn as_mut_ptr(&self)")
    {
        failures += 1;
        println!("❌ safe as_mut_ptr regression");
        println!("::error title=NROS safety gate::safe as_mut_ptr regression");
    }
    if failures == 0 {
        println!("✅ structural safety checks passed");
    } else {
        // Pass 27 fix (F-8): hard gate (was exit-0 always). Observability: ::error annotation.
        gate_fail(format!("{} safety regression(s)", failures));
    }
}
