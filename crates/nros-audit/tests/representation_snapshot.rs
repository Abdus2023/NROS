//! Regression tests for repository-representation snapshot invariants.
//!
//! These tests intentionally exercise the representation contract at the text
//! and repository-boundary level. The production gate remains authoritative;
//! these tests prevent accidental weakening of its required inputs.

use std::fs;
use std::path::Path;

const ROOT: &str = "docs/representation";
const MANIFESTS: &[&str] = &["architecture.yaml", "capabilities.yaml", "evidence.yaml", "claims.yaml"];

fn read(name: &str) -> String {
    fs::read_to_string(Path::new(ROOT).join(name)).expect("representation fixture must exist")
}

#[test]
fn snapshot_declares_source_and_snapshot_revision_separately() {
    let text = read("snapshot.yaml");
    assert!(text.contains("source_revision:"));
    assert!(text.contains("snapshot_revision:"));
    assert!(text.contains("relation: represented_repository_state"));
    assert!(text.contains("relation: commit_containing_this_snapshot"));
}

#[test]
fn snapshot_declares_all_normative_manifest_fingerprints() {
    let text = read("snapshot.yaml");
    for manifest in MANIFESTS {
        assert!(text.contains(&format!("{}: \"", manifest)), "missing fingerprint for {manifest}");
    }
    assert!(text.contains("algorithm: git_blob_sha1"));
}

#[test]
fn snapshot_requires_content_match_and_fails_closed() {
    let text = read("snapshot.yaml");
    assert!(text.contains("source_revision_resolves"));
    assert!(text.contains("manifest_exists_at_source_revision"));
    assert!(text.contains("manifest_blob_matches_recorded_fingerprint"));
    assert!(text.contains("failure_policy: representation_gate_fails"));
}

#[test]
fn snapshot_does_not_equate_integrity_with_execution_success() {
    let text = read("snapshot.yaml");
    assert!(text.contains("content_fingerprint_is_not_verification_success"));
    assert!(text.contains("source_revision_is_not_execution_success"));
}

#[test]
fn schema_and_snapshot_are_present() {
    assert!(Path::new(ROOT).join("schema.yaml").is_file());
    assert!(Path::new(ROOT).join("snapshot.yaml").is_file());
}
