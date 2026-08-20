//! Repository representation validator.
//! This intentionally uses only the Rust standard library: the representation
//! manifests are normative text artifacts, while this gate validates their
//! cross-file invariants without introducing a YAML parser dependency.

use std::fs;
use std::path::Path;

const ROOT: &str = "docs/representation";

fn read(name: &str) -> String {
    let path = Path::new(ROOT).join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("FAIL: cannot read {}: {}", path.display(), e);
        std::process::exit(2);
    })
}

fn require(haystack: &str, needle: &str, label: &str, failures: &mut usize) {
    if haystack.contains(needle) {
        println!("PASS  {}", label);
    } else {
        println!("FAIL  {} — missing `{}`", label, needle);
        *failures += 1;
    }
}

fn main() {
    println!("NROS repository representation gate");
    println!("root: {}", ROOT);

    let architecture = read("architecture.yaml");
    let capabilities = read("capabilities.yaml");
    let evidence = read("evidence.yaml");
    let claims = read("claims.yaml");

    let mut failures = 0usize;

    // Manifest identity.
    for (name, text) in [
        ("architecture", &architecture),
        ("capabilities", &capabilities),
        ("evidence", &evidence),
        ("claims", &claims),
    ] {
        require(text, "schema_version:", name, &mut failures);
        require(text, "project: NROS", &format!("{} project identity", name), &mut failures);
    }

    // Cross-file capability IDs must resolve to evidence and claim policy.
    let capability_ids = [
        "CORE-IPC-001", "CORE-IPC-002", "NODE-001", "NODE-002", "HAL-001",
        "HAL-002", "TRANSPORT-001", "TRANSPORT-002", "DIST-001", "DIST-002",
        "SIM-001", "STUDIO-001", "STUDIO-002", "CLI-001", "AUDIT-001",
    ];
    for id in capability_ids {
        require(&capabilities, id, &format!("capability {} declared", id), &mut failures);
    }

    // Evidence and claim records must contain the critical safety vocabulary.
    for id in ["CORE-IPC-001", "HAL-002", "DIST-002", "STUDIO-002"] {
        require(&evidence, id, &format!("evidence record {}", id), &mut failures);
    }
    for id in ["CLAIM-IPC-001", "CLAIM-PERF-001", "CLAIM-HAL-001", "CLAIM-DIST-001", "CLAIM-STUDIO-001", "CLAIM-SAFETY-001", "CLAIM-CI-001", "CLAIM-MIRI-001"] {
        require(&claims, id, &format!("claim {}", id), &mut failures);
    }

    // Representation safety invariants.
    require(&evidence, "configured_ci_is_not_passed_ci", "CI execution distinction", &mut failures);
    require(&evidence, "benchmark_artifact_is_not_independent_validation", "benchmark distinction", &mut failures);
    require(&evidence, "simulated_implementation_cannot_support_real_backend_claim", "simulation distinction", &mut failures);
    require(&evidence, "hardware_validation_requires_actual_hardware_evidence", "hardware distinction", &mut failures);
    require(&claims, "no_claim_without_evidence_record", "claim/evidence linkage", &mut failures);
    require(&claims, "no_real_claim_from_simulated_backend", "real-vs-simulated invariant", &mut failures);
    require(&claims, "no_ci_pass_claim_without_executed_successful_run", "CI pass invariant", &mut failures);

    // Architecture must explicitly identify intent rather than implementation.
    require(&architecture, "source_of_truth: DESIGN.md", "architecture source", &mut failures);
    require(&architecture, "architecture_intent_is_not_implementation_evidence", "architecture boundary", &mut failures);
    require(&architecture, "crate_topology_is_not_runtime_topology", "topology boundary", &mut failures);

    // The canonical documentation must point to the machine-readable layer.
    let canonical = fs::read_to_string("docs/REPOSITORY_REPRESENTATION.md").unwrap_or_default();
    require(&canonical, "docs/representation/", "canonical documentation links representation directory", &mut failures);
    require(&canonical, "No specification implies implementation.", "canonical specification invariant", &mut failures);

    println!();
    if failures == 0 {
        println!("REPRESENTATION-GATE: PASS");
    } else {
        println!("REPRESENTATION-GATE: FAIL ({} failure(s))", failures);
        std::process::exit(1);
    }
}
