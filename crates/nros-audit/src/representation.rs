//! Repository representation validator.
//! The manifests are normative text; this gate performs repository discovery
//! without requiring a YAML parser dependency.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "docs/representation";

fn read(name: &str) -> String {
    let path = Path::new(ROOT).join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("FAIL: cannot read {}: {}", path.display(), e);
        std::process::exit(2);
    })
}

fn require(haystack: &str, needle: &str, label: &str, failures: &mut usize) {
    if haystack.contains(needle) { println!("PASS  {}", label); }
    else { println!("FAIL  {} — missing `{}`", label, needle); *failures += 1; }
}

fn workspace_members() -> Vec<String> {
    fs::read_to_string("Cargo.toml").unwrap_or_default().lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with('"') && line.contains("crates/") {
                Some(line.trim_matches(',').trim_matches('"').to_string())
            } else { None }
        }).collect()
}

fn capability_field(text: &str, id: &str, field: &str) -> Option<String> {
    let marker = format!("  - id: {}", id);
    let block = text.split(&marker).nth(1)?.split("\n  - id:").next().unwrap_or("");
    block.lines().find_map(|line| line.trim().strip_prefix(&format!("{}: ", field)).map(str::to_string))
}

fn represented_crates(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in text.lines() {
        if let Some(value) = line.trim().strip_prefix("crate: ") {
            out.insert(value.trim().to_string());
        }
    }
    out
}

pub fn run() {
    println!("NROS repository representation gate");
    let architecture = read("architecture.yaml");
    let capabilities = read("capabilities.yaml");
    let evidence = read("evidence.yaml");
    let claims = read("claims.yaml");
    let mut failures = 0usize;

    for (name, text) in [("architecture", &architecture), ("capabilities", &capabilities), ("evidence", &evidence), ("claims", &claims)] {
        require(text, "schema_version:", name, &mut failures);
        require(text, "project: NROS", &format!("{} project identity", name), &mut failures);
    }

    // Forward direction: representation → repository.
    let members = workspace_members();
    println!("DISCOVERY workspace members: {}", members.len());
    for member in &members {
        let manifest = PathBuf::from(member).join("Cargo.toml");
        if manifest.exists() { println!("PASS  workspace member {} exists", member); }
        else { println!("FAIL  workspace member {} has no Cargo.toml", member); failures += 1; }
    }

    let ids = ["CORE-IPC-001","CORE-IPC-002","NODE-001","NODE-002","HAL-001","HAL-002","TRANSPORT-001","TRANSPORT-002","DIST-001","DIST-002","SIM-001","STUDIO-001","STUDIO-002","CLI-001","AUDIT-001"];
    for id in ids {
        require(&capabilities, id, &format!("capability {} declared", id), &mut failures);
        let Some(crate_name) = capability_field(&capabilities, id, "crate") else {
            println!("FAIL  {} has no crate mapping", id); failures += 1; continue;
        };
        let crate_path = format!("crates/{}", crate_name);
        if Path::new(&crate_path).is_dir() { println!("PASS  {} → {} exists", id, crate_path); }
        else { println!("FAIL  {} → {} is missing", id, crate_path); failures += 1; }
        let state = capability_field(&capabilities, id, "state").unwrap_or_default();
        if ["IMPLEMENTED","TESTED","BENCHMARKED","INTEGRATION-TESTED","HARDWARE-VALIDATED","PRODUCTION-READY","SAFETY-QUALIFIABLE"].contains(&state.as_str()) && !Path::new(&format!("{}/src", crate_path)).is_dir() {
            println!("FAIL  {} state={} but source tree is absent", id, state); failures += 1;
        }
    }

    // Reverse direction: repository → representation. Every workspace crate
    // must be represented either by a capability mapping or by an explicit
    // architecture-level infrastructure declaration.
    let capability_crates = represented_crates(&capabilities);
    let architecture_crates = represented_crates(&architecture);
    for member in &members {
        let crate_name = member.strip_prefix("crates/").unwrap_or(member);
        if capability_crates.contains(crate_name) || architecture_crates.contains(crate_name) {
            println!("PASS  reverse inventory {} represented", member);
        } else {
            println!("FAIL  reverse inventory {} has no representation entry", member);
            failures += 1;
        }
    }

    // Detect duplicate capability IDs in the canonical catalog.
    for id in ids {
        let count = capabilities.matches(&format!("  - id: {}", id)).count();
        if count == 1 { println!("PASS  capability {} unique", id); }
        else { println!("FAIL  capability {} appears {} times", id, count); failures += 1; }
    }

    // Evidence/claim inventory and non-inference invariants.
    for id in ["CORE-IPC-001","HAL-002","DIST-002","STUDIO-002"] { require(&evidence, id, &format!("evidence record {}", id), &mut failures); }
    for id in ["CLAIM-IPC-001","CLAIM-PERF-001","CLAIM-HAL-001","CLAIM-DIST-001","CLAIM-STUDIO-001","CLAIM-SAFETY-001","CLAIM-CI-001","CLAIM-MIRI-001"] { require(&claims, id, &format!("claim {}", id), &mut failures); }
    for (text, needle, label) in [
        (&evidence,"configured_ci_is_not_passed_ci","CI execution distinction"),
        (&evidence,"benchmark_artifact_is_not_independent_validation","benchmark distinction"),
        (&evidence,"simulated_implementation_cannot_support_real_backend_claim","simulation distinction"),
        (&evidence,"hardware_validation_requires_actual_hardware_evidence","hardware distinction"),
        (&claims,"no_claim_without_evidence_record","claim/evidence linkage"),
        (&claims,"no_real_claim_from_simulated_backend","real-vs-simulated invariant"),
        (&claims,"no_ci_pass_claim_without_executed_successful_run","CI pass invariant"),
        (&architecture,"source_of_truth: DESIGN.md","architecture source"),
        (&architecture,"architecture_intent_is_not_implementation_evidence","architecture boundary"),
        (&architecture,"crate_topology_is_not_runtime_topology","topology boundary"),
    ] { require(text, needle, label, &mut failures); }

    let canonical = fs::read_to_string("docs/REPOSITORY_REPRESENTATION.md").unwrap_or_default();
    require(&canonical,"docs/representation/","canonical representation directory",&mut failures);
    require(&canonical,"No specification implies implementation.","canonical specification invariant",&mut failures);

    if failures == 0 { println!("REPRESENTATION-GATE: PASS"); }
    else { println!("REPRESENTATION-GATE: FAIL ({} failure(s))", failures); std::process::exit(1); }
}
