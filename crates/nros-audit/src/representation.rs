//! Repository representation validator.
//! The manifests are intentionally simple YAML-like text; this gate validates
//! structural cross-references without introducing a parser dependency.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "docs/representation";
const STATES: &[&str] = &["SPECIFIED","SCAFFOLDED","SIMULATED","IMPLEMENTED","TESTED","BENCHMARKED","INTEGRATION-TESTED","HARDWARE-VALIDATED","PRODUCTION-READY","SAFETY-QUALIFIABLE"];
const CLAIM_CLASSES: &[&str] = &["allowed_with_scope","allowed_as_scaffolding","conditional","forbidden"];

fn read(name: &str) -> String {
    let path = Path::new(ROOT).join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| { eprintln!("FAIL: cannot read {}: {}", path.display(), e); std::process::exit(2); })
}

fn require(ok: bool, label: &str, failures: &mut usize) {
    if ok { println!("PASS  {}", label); } else { println!("FAIL  {}", label); *failures += 1; }
}

fn workspace_members() -> Vec<String> {
    fs::read_to_string("Cargo.toml").unwrap_or_default().lines().filter_map(|line| {
        let line = line.trim();
        if line.starts_with('"') && line.contains("crates/") { Some(line.trim_matches(',').trim_matches('"').to_string()) } else { None }
    }).collect()
}

fn records(text: &str, marker: &str) -> Vec<(String, BTreeMap<String, String>)> {
    let mut out = Vec::new();
    let mut current: Option<(String, BTreeMap<String, String>)> = None;
    for line in text.lines() {
        if let Some(id) = line.trim().strip_prefix(marker) {
            if let Some(r) = current.take() { out.push(r); }
            current = Some((id.trim().to_string(), BTreeMap::new()));
        } else if let Some((_, fields)) = current.as_mut() {
            let t = line.trim();
            if let Some((k, v)) = t.split_once(": ") {
                if !v.starts_with('-') { fields.insert(k.to_string(), v.trim().to_string()); }
            }
        }
    }
    if let Some(r) = current { out.push(r); }
    out
}

fn record_block(text: &str, marker: &str, key: &str) -> Option<String> {
    let start = text.find(&format!("{}{}", marker, key))?;
    let tail = &text[start..];
    Some(tail.split("\n  - ").next().unwrap_or(tail).to_string())
}

pub fn run() {
    println!("NROS repository representation gate");
    let architecture = read("architecture.yaml");
    let capabilities = read("capabilities.yaml");
    let evidence = read("evidence.yaml");
    let claims = read("claims.yaml");
    let canonical = fs::read_to_string("docs/REPOSITORY_REPRESENTATION.md").unwrap_or_default();
    let mut failures = 0usize;

    for (name, text) in [("architecture", &architecture),("capabilities", &capabilities),("evidence", &evidence),("claims", &claims)] {
        require(text.contains("schema_version:"), &format!("{} has schema_version", name), &mut failures);
        require(text.contains("project: NROS"), &format!("{} identifies NROS", name), &mut failures);
    }

    let members = workspace_members();
    println!("DISCOVERY workspace members: {}", members.len());
    for member in &members { require(PathBuf::from(member).join("Cargo.toml").exists(), &format!("workspace member {} has Cargo.toml", member), &mut failures); }

    // Dynamic capability discovery.
    let cap_records = records(&capabilities, "- id: ");
    let mut cap_ids = BTreeSet::new();
    let mut cap_crates = BTreeSet::new();
    for (id, fields) in &cap_records {
        require(cap_ids.insert(id.clone()), &format!("capability {} unique", id), &mut failures);
        for f in ["name","crate","specification","state","claim"] { require(fields.contains_key(f), &format!("capability {} has {}", id, f), &mut failures); }
        if let Some(state) = fields.get("state") { require(STATES.contains(&state.as_str()), &format!("capability {} state {} valid", id, state), &mut failures); }
        if let Some(c) = fields.get("crate") {
            cap_crates.insert(c.clone());
            let p = format!("crates/{}", c);
            require(Path::new(&p).is_dir(), &format!("capability {} maps to {}", id, p), &mut failures);
        }
    }

    // Dynamic evidence discovery and bidirectional linkage.
    let evidence_records = records(&evidence, "- capability: ");
    let mut evidence_caps = BTreeSet::new();
    for (cap, _) in &evidence_records {
        require(evidence_caps.insert(cap.clone()), &format!("evidence {} unique", cap), &mut failures);
        require(cap_ids.contains(cap), &format!("evidence {} references known capability", cap), &mut failures);
        if let Some(block) = record_block(&evidence, "- capability: ", cap) {
            for dimension in ["source:","tests:","ci:","miri:","benchmark:","hardware:"] {
                require(block.contains(dimension), &format!("evidence {} has {} dimension", cap, dimension.trim_end_matches(':')), &mut failures);
            }
        }
    }
    for cap in &cap_ids { require(evidence_caps.contains(cap), &format!("capability {} has evidence record", cap), &mut failures); }

    // Dynamic claim discovery and class validation.
    let claim_records = records(&claims, "- id: ");
    let mut claim_ids = BTreeSet::new();
    for (id, fields) in &claim_records {
        require(claim_ids.insert(id.clone()), &format!("claim {} unique", id), &mut failures);
        require(fields.contains_key("subject"), &format!("claim {} has subject", id), &mut failures);
        if let Some(class) = fields.get("class") { require(CLAIM_CLASSES.contains(&class.as_str()), &format!("claim {} class {} valid", id, class), &mut failures); }
        else { require(false, &format!("claim {} has class", id), &mut failures); }
    }

    // Reverse inventory: every workspace crate is represented by capability or architecture.
    let architecture_ids: BTreeSet<String> = architecture.lines().filter_map(|line| line.trim().strip_prefix("- id: ").map(str::to_string)).filter(|id| id.starts_with("nros-")).collect();
    for member in &members {
        let name = member.strip_prefix("crates/").unwrap_or(member);
        require(cap_crates.contains(name) || architecture_ids.contains(name), &format!("reverse inventory {} represented", member), &mut failures);
    }

    // Normative non-inference invariants.
    for needle in ["configured_ci_is_not_passed_ci","benchmark_artifact_is_not_independent_validation","simulated_implementation_cannot_support_real_backend_claim","hardware_validation_requires_actual_hardware_evidence"] { require(evidence.contains(needle), &format!("evidence invariant {}", needle), &mut failures); }
    for needle in ["no_claim_without_evidence_record","no_real_claim_from_simulated_backend","no_ci_pass_claim_without_executed_successful_run"] { require(claims.contains(needle), &format!("claim invariant {}", needle), &mut failures); }
    for needle in ["source_of_truth: DESIGN.md","architecture_intent_is_not_implementation_evidence","crate_topology_is_not_runtime_topology"] { require(architecture.contains(needle), &format!("architecture invariant {}", needle), &mut failures); }
    require(canonical.contains("docs/representation/"), "canonical representation directory", &mut failures);
    require(canonical.contains("No specification implies implementation."), "canonical specification invariant", &mut failures);

    if failures == 0 { println!("REPRESENTATION-GATE: PASS"); } else { println!("REPRESENTATION-GATE: FAIL ({} failure(s))", failures); std::process::exit(1); }
}
