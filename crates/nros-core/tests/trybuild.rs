//! Trybuild negative compile tests per AUDIT Pass 16 §28
//! Tests things that must NOT compile: two Producers from one SPSC channel, commit uninitialized WriteGuard, mutable ReadGuard
//! Run: cargo test --test trybuild -- --nocapture

#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
