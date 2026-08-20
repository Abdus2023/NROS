//! Repository representation validator.
//! The manifests are intentionally simple YAML-like text; this gate validates
//! structural cross-references and snapshot content integrity without a parser dependency.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ROOT: &str = "docs/representation";
const STATES: &[&str] = &["SPECIFIED","SCAFFOLDED","SIMULATED","IMPLEMENTED","TESTED","BENCHMARKED","INTEGRATION-TESTED","HARDWARE-VALIDATED","PRODUCTION-READY","SAFETY-QUALIFIABLE"];
const CLAIM_CLASSES: &[&str] = &["allowed_with_scope","allowed_as_scaffolding","conditional","forbidden"];
const MANIFESTS: &[&str] = &["architecture.yaml","capabilities.yaml","evidence.yaml","claims.yaml"];

fn read(name: &str) -> String { let path=Path::new(ROOT).join(name); fs::read_to_string(&path).unwrap_or_else(|e| { eprintln!("FAIL: cannot read {}: {}",path.display(),e); std::process::exit(2); }) }
fn require(ok: bool,label:&str,failures:&mut usize){if ok{println!("PASS  {}",label)}else{println!("FAIL  {}",label);*failures+=1}}
fn workspace_members()->Vec<String>{fs::read_to_string("Cargo.toml").unwrap_or_default().lines().filter_map(|line|{let l=line.trim();if l.starts_with('"')&&l.contains("crates/"){Some(l.trim_matches(',').trim_matches('"').to_string())}else{None}}).collect()}
fn records(text:&str,marker:&str)->Vec<(String,BTreeMap<String,String>)>{let mut out=Vec::new();let mut cur=None;for line in text.lines(){if let Some(id)=line.trim().strip_prefix(marker){if let Some(r)=cur.take(){out.push(r)}cur=Some((id.trim().to_string(),BTreeMap::new()))}else if let Some((_,f))=cur.as_mut(){let t=line.trim();if let Some((k,v))=t.split_once(": "){if !v.starts_with('-'){f.insert(k.to_string(),v.trim().to_string())}}}}if let Some(r)=cur{out.push(r)}out}
fn snapshot_value(text:&str, key:&str)->Option<String>{text.lines().find_map(|l|{let t=l.trim();let prefix=format!("{}: ",key);t.strip_prefix(&prefix).map(|v|v.trim().trim_matches('"').to_string())})}
fn git(args:&[&str])->Option<String>{let out=Command::new("git").args(args).output().ok()?;if !out.status.success(){return None}Some(String::from_utf8_lossy(&out.stdout).trim().to_string())}

fn verify_snapshot(snapshot:&str, failures:&mut usize){
    let commit=snapshot_value(snapshot,"commit");
    require(commit.as_ref().is_some_and(|c|c.len()==40),"snapshot source revision has 40-character SHA",failures);
    let Some(commit)=commit else{return};
    require(git(&["cat-file","-e",&format!("{}^{{commit}}",commit)]).is_some(),"snapshot source revision resolves",failures);
    let Some(treeish)=git(&["rev-parse",&commit]) else {return};
    for manifest in MANIFESTS {
        let object=git(&["rev-parse",&format!("{}:docs/representation/{}",treeish,manifest)]);
        require(object.is_some(),&format!("snapshot manifest {} exists at source revision",manifest),failures);
        let recorded=snapshot.lines().find_map(|l|{let t=l.trim();let prefix=format!("{}: ",manifest);t.strip_prefix(&prefix).map(|v|v.trim().trim_matches('"').to_string())});
        require(recorded.is_some(),&format!("snapshot records fingerprint for {}",manifest),failures);
        if let (Some(actual),Some(expected))=(object,recorded){require(actual==expected,&format!("snapshot fingerprint matches {}",manifest),failures)}
    }
}

pub fn run(){
 println!("NROS repository representation gate");
 let architecture=read("architecture.yaml");let capabilities=read("capabilities.yaml");let evidence=read("evidence.yaml");let claims=read("claims.yaml");let snapshot=read("snapshot.yaml");let canonical=fs::read_to_string("docs/REPOSITORY_REPRESENTATION.md").unwrap_or_default();let mut failures=0;
 for(name,text)in[("architecture",&architecture),("capabilities",&capabilities),("evidence",&evidence),("claims",&claims)]{require(text.contains("schema_version:"),&format!("{} has schema_version",name),&mut failures);require(text.contains("project: NROS"),&format!("{} identifies NROS",name),&mut failures)}
 verify_snapshot(&snapshot,&mut failures);
 let members=workspace_members();println!("DISCOVERY workspace members: {}",members.len());for m in &members{require(PathBuf::from(m).join("Cargo.toml").exists(),&format!("workspace member {} has Cargo.toml",m),&mut failures)}
 let caps=records(&capabilities,"- id: ");let mut ids=BTreeSet::new();let mut cap_crates=BTreeSet::new();
 for(id,f)in&caps{require(ids.insert(id.clone()),&format!("capability {} unique",id),&mut failures);for k in ["name","crate","specification","state","claim"]{require(f.contains_key(k),&format!("capability {} has {}",id,k),&mut failures)}if let Some(s)=f.get("state"){require(STATES.contains(&s.as_str()),&format!("capability {} state {} valid",id,s),&mut failures)}if let Some(c)=f.get("crate"){cap_crates.insert(c.clone());let p=format!("crates/{}",c);require(Path::new(&p).is_dir(),&format!("capability {} maps to {}",id,p),&mut failures)}}
 let ev=records(&evidence,"- capability: ");let mut ev_caps=BTreeSet::new();for(cap,f)in&ev{require(ev_caps.insert(cap.clone()),&format!("evidence {} unique",cap),&mut failures);require(ids.contains(cap),&format!("evidence {} references known capability",cap),&mut failures);for k in ["source","tests","ci","miri","benchmark","hardware"]{require(f.contains_key(k)||evidence.contains(&format!("{}:\n",k)),&format!("evidence {} has {} dimension",cap,k),&mut failures)}}for cap in &ids{require(ev_caps.contains(cap),&format!("capability {} has evidence record",cap),&mut failures)}
 let cls=records(&claims,"- id: ");let mut claim_ids=BTreeSet::new();for(id,f)in&cls{require(claim_ids.insert(id.clone()),&format!("claim {} unique",id),&mut failures);require(f.contains_key("subject"),&format!("claim {} has subject",id),&mut failures);require(f.get("class").is_some_and(|c|CLAIM_CLASSES.contains(&c.as_str())),&format!("claim {} has valid class",id),&mut failures)}
 let arch_ids:BTreeSet<String>=architecture.lines().filter_map(|l|l.trim().strip_prefix("- id: ").map(str::to_string)).filter(|id|id.starts_with("nros-")).collect();for m in &members{let n=m.strip_prefix("crates/").unwrap_or(m);require(cap_crates.contains(n)||arch_ids.contains(n),&format!("reverse inventory {} represented",m),&mut failures)}
 for n in ["configured_ci_is_not_passed_ci","benchmark_artifact_is_not_independent_validation","simulated_implementation_cannot_support_real_backend_claim","hardware_validation_requires_actual_hardware_evidence"]{require(evidence.contains(n),&format!("evidence invariant {}",n),&mut failures)}for n in ["no_claim_without_evidence_record","no_real_claim_from_simulated_backend","no_ci_pass_claim_without_executed_successful_run"]{require(claims.contains(n),&format!("claim invariant {}",n),&mut failures)}for n in ["source_of_truth: DESIGN.md","architecture_intent_is_not_implementation_evidence","crate_topology_is_not_runtime_topology"]{require(architecture.contains(n),&format!("architecture invariant {}",n),&mut failures)}require(canonical.contains("docs/representation/"),"canonical representation directory",&mut failures);require(canonical.contains("No specification implies implementation."),"canonical specification invariant",&mut failures);
 if failures==0{println!("SNAPSHOT-INTEGRITY: PASS");println!("REPRESENTATION-GATE: PASS")}else{println!("SNAPSHOT-INTEGRITY/REPRESENTATION-GATE: FAIL ({} failure(s))",failures);std::process::exit(1)}
}
