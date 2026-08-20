//! Repository representation validator.
//!
//! The representation manifests are normative text artifacts. This gate keeps
//! the audit dependency-free and discovers capability records dynamically.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "docs/representation";
const STATES: &[&str] = &[
    "SPECIFIED", "SCAFFOLDED", "SIMULATED", "IMPLEMENTED", "TESTED",
    "BENCHMARKED", "INTEGRATION-TESTED", "HARDWARE-VALIDATED",
    "PRODUCTION-READY", "SAFETY-QUALIFIABLE",
];

fn read(name: &str) -> String {
    let path = Path::new(ROOT).join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("FAIL: cannot read {}: {}", path.display(), e);
        std::process::exit(2);
    })
}

fn require(ok: bool, label: &str, failures: &mut usize) {
    if ok { println!("PASS  {}", label); } else { println!("FAIL  {}", label); *failures += 1; }
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

/// Parse the deliberately simple capability-record subset used by the
/// repository manifest. This is intentionally not a general YAML parser.
fn capability_records(text: &str) -> Vec<(String, BTreeMap<String, String>)> {
    let mut records = Vec::new();
    let mut current: Option<(String, BTreeMap<String, String>)> = None;
    for line in text.lines() {
        if let Some(id) = line.trim().strip_prefix("- id: ") {
            if let Some(record) = current.take() { records.push(record); }
            current = Some((id.trim().to_string(), BTreeMap::new()));
        } else if let Some((_, fields)) = current.as_mut() {
            let trimmed = line.trim();
            if let Some((key, value)) = trimmed.split_once(": ") {
                if !value.starts_with('-') { fields.insert(key.to_string(), value.trim().to_string()); }
            }
        }
    }
    if let Some(record) = current { records.push(record); }
    records
}

pub fn run() {
    println!("NROS repository representation gate");
    let architecture = read("architecture.yaml");
    let capabilities = read("capabilities.yaml");
    let evidence = read("evidence.yaml");
    let claims = read("claims.yaml");
    let canonical = fs::read_to_string("docs/REPOSITORY_REPRESENTATION.md").unwrap_or_default();
    let mut failures = 0usize;

    for (name, text) in [("architecture", &architecture), ("capabilities", &capabilities), ("evidence", &evidence), ("claims", &claims)] {
        require(text.contains("schema_version:"), &format!("{} has schema_version", name), &mut failures);
        require(text.contains("project: NROS"), &format!("{} identifies NROS", name), &mut failures);
    }

    let members = workspace_members();
    println!("DISCOVERY workspace members: {}", members.len());
    for member in &members {
        require(PathBuf::from(member).join("Cargo.toml").exists(), &format!("workspace member {} has Cargo.toml", member), &mut failures);
    }

    // Dynamic capability discovery: adding/removing a capability no longer
    // requires changing this Rust source file.
    let records = capability_records(&capabilities);
    require(!records.is_empty(), "capability catalog contains records", &mut failures);
    let mut ids = BTreeSet::new();
    let mut represented_crates = BTreeSet::new();
    for (id, fields) in &records {
        require(ids.insert(id.clone()), &format!("capability {} has unique ID", id), &mut failures);
        for field in ["name", "crate", "specification", "state", "claim"] {
            require(fields.contains_key(field), &format!("capability {} has {}", id, field), &mut failures);
        }
        if let Some(state) = fields.get("state") {
            require(STATES.contains(&state.as_str()), &format!("capability {} has valid state {}", id, state), &mut failures);
        }
        if let Some(crate_name) = fields.get("crate") {
            represented_crates.insert(crate_name.clone());
            let path = format!("crates/{}", crate_name);
            require(Path::new(&path).is_dir(), &format!("capability {} maps to {}", id, path), &mut failures);
            if let Some(state) = fields.get("state") {
                if ["IMPLEMENTED", "TESTED", "BENCHMARKED", "INTEGRATION-TESTED", "HARDWARE-VALIDATED", "PRODUCTION-READY", "SAFETY-QUALIFIABLE"].contains(&state.as_str()) {
                    require(Path::new(&format!("{}/src", path)).is_dir(), &format!("capability {} state {} has source tree", id, state), &mut failures);
                }
            }
        }
    }

    // Reverse inventory: every workspace crate must appear in either the
    // capability catalog or the architecture manifest.
    let architecture_crates: BTreeSet<String> = architecture.lines()
        .filter_map(|line| line.trim().strip_prefix("id: ").map(str::to_string))
        .filter(|id| id.starts_with("nros-"))
        .collect();
    for member in &members {
        let crate_name = member.strip_prefix("crates/").unwrap_or(member);
        require(
            represented_crates.contains(crate_name) || architecture_crates.contains(crate_name),
            &format!("reverse inventory {} is represented", member),
            &mut failures,
        );
    }

    for needle in [
        "configured_ci_is_not_passed_ci",
        "benchmark_artifact_is_not_independent_validation",
        "simulated_implementation_cannot_support_real_backend_claim",
        "hardware_validation_requires_actual_hardware_evidence",
    ] { require(evidence.contains(needle), &format!("evidence invariant {}", needle), &mut failures); }
    for needle in [
        "no_claim_without_evidence_record",
        "no_real_claim_from_simulated_backend",
        "no_ci_pass_claim_without_executed_successful_run",
    ] { require(claims.contains(needle), &format!("claim invariant {}", needle), &mut failures); }
    for needle in [
        "source_of_truth: DESIGN.md",
        "architecture_intent_is_not_implementation_evidence",
        "crate_topology_is_not_runtime_topology",
    ] { require(architecture.contains(needle), &format!("architecture invariant {}", needle), &mut failures); }

    require(canonical.contains("docs/representation/"), "canonical documentation links representation directory", &mut failures);
    require(canonical.contains("No specification implies implementation."), "canonical specification invariant", &mut failures);

    if failures == 0 { println!("REPRESENTATION-GATE: PASS"); }
    else { println!("REPRESENTATION-GATE: FAIL ({} failure(s))", failures); std::process::exit(1); }
}
