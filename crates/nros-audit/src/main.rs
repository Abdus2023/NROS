//! NROS Claim Linter and repository representation gate.

mod representation;

use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("claims");

    // TEMPORARY Pass 27 CI diagnostic (removed once the residual rustc defect is
    // fixed): forward the errors of the failing `cargo check --workspace
    // --all-targets` as workflow-command annotations so they are readable through
    // the check-runs API where raw job logs are unreachable. Active only for the
    // `all` subcommand used by the governance job; no effect on gate semantics.
    if cmd == "all" {
        ci_diag_forward_cargo_check();
    }

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

/// TEMPORARY Pass 27 CI diagnostic — see call site. Runs the command that is red
/// in CI (`cargo check --workspace --all-targets`) and re-emits its error output
/// as `::error` workflow commands, which surface as check-run annotations
/// (API-readable) even though raw job logs (Azure blob hosts) are unreachable
/// from the audit workstation. No-ops outside GitHub Actions. Gate semantics are
/// unchanged: this function never exits non-zero by itself.
fn ci_diag_forward_cargo_check() {
    if std::env::var_os("CI").is_none() {
        return;
    }
    eprintln!("::error title=Pass27-DIAG marker::diag channel active — forwarding cargo check stderr");
    let out = std::process::Command::new("cargo")
        .args(["check", "--workspace", "--all-targets", "--message-format", "short"])
        .output();
    let out = match out {
        Ok(o) => o,
        Err(e) => {
            eprintln!("::error title=Pass27-DIAG spawn::failed to spawn cargo: {}", e);
            return;
        }
    };
    let text = String::from_utf8_lossy(&out.stderr);
    let esc = |s: &str| -> String {
        s.replace('%', "%25").replace('\r', "%0D").replace('\n', "%0A")
    };
    // Error-bearing lines first, then the compiler/cargo tail for context. Capped
    // so we stay well under the 50-annotation check-run limit.
    let mut emitted = 0usize;
    for line in text.lines().filter(|l| l.contains("error")) {
        if emitted >= 30 {
            break;
        }
        let t: String = line.chars().take(480).collect();
        eprintln!("::error title=Pass27-DIAG rustc::{}", esc(&t));
        emitted += 1;
    }
    let tail: Vec<&str> = text.lines().collect();
    for line in tail.iter().rev().take(12).rev() {
        if emitted >= 40 {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let t: String = line.chars().take(480).collect();
        eprintln!("::error title=Pass27-DIAG tail::{}", esc(&t));
        emitted += 1;
    }
}

fn gate_fail(msg: String) -> ! {
    // Pass 27 observability fix: print each hard failure BOTH as prose and as a GitHub
    // Actions workflow-command annotation, so the reason is visible on the check-run even
    // where raw job logs are unreachable (annotations ride the API, logs ride blob hosts).
    println!("❌ {}", msg);
    println!("::error title=NROS audit gate::{}", msg.replace('%', "%25").replace('\r', "%0D").replace('\n', "%0A"));
    std::process::exit(1);
}

fn check_workspace_inventory() {
    println!("🔍 DOC-GATE: workspace inventory");
    let cargo_toml = fs::read_to_string("Cargo.toml").unwrap_or_default();
    let readme = fs::read_to_string("README.md").unwrap_or_default();
    let cargo_crates: Vec<&str> = cargo_toml.lines().filter(|l| l.contains("crates/")).collect();
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
    if core.contains("pub fn init_with<F>") { failures += 1; println!("❌ safe init_with regression");  println!("::error title=NROS safety gate::safe init_with regression"); }
    if core.contains("pub fn as_mut_ptr(&self)") && !core.contains("pub unsafe fn as_mut_ptr(&self)") { failures += 1; println!("❌ safe as_mut_ptr regression");  println!("::error title=NROS safety gate::safe as_mut_ptr regression"); }
    if failures == 0 {
        println!("✅ structural safety checks passed");
    } else {
        // Pass 27 fix (F-8): hard gate (was exit-0 always). Observability: ::error annotation.
        gate_fail(format!("{} safety regression(s)", failures));
    }
}
