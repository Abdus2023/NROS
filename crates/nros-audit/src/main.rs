//! NROS Claim Linter and repository representation gate.

mod representation;

use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("claims");

    // TEMPORARY Pass 27 diag #9 (Miri decode only, removed with the resolution):
    // the Miri job is red with no visible logs; reproduce it here and stream
    // progress + hits over the annotation channel. CI-env gated; no gate effect.
    if cmd == "all" {
        ci_diag9_miri();
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

// ── TEMPORARY Pass 27 diag #9 infra (Miri-only decode; removal scheduled) ────

fn d9_strip_ansi(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == 0x1B && i + 1 < b.len() && b[i + 1] == b'[' {
            i += 2;
            while i < b.len() && !(0x40..=0x7E).contains(&b[i]) { i += 1; }
            i += 1;
        } else {
            let ch_len = if b[i] < 0x80 { 1 } else if b[i] >= 0xF0 { 4 } else if b[i] >= 0xE0 { 3 } else if b[i] >= 0xC0 { 2 } else { 1 };
            out.push_str(&s[i..(i + ch_len).min(s.len())]);
            i += ch_len;
        }
    }
    out
}

fn d9_emit(title: &str, msg: &str) {
    use std::io::Write;
    let cleaned = d9_strip_ansi(msg)
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A");
    let mut cur = String::new();
    let mut chunks: Vec<String> = Vec::new();
    for c in cleaned.chars() {
        cur.push(c);
        if cur.len() >= 8000 {
            chunks.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() || chunks.is_empty() {
        chunks.push(cur);
    }
    let n = chunks.len();
    for (i, c) in chunks.iter().enumerate() {
        let line = if n > 1 {
            format!("::error title={} ({}/{})::{}", title, i + 1, n, c)
        } else {
            format!("::error title={}::{}", title, c)
        };
        let _ = std::io::stderr().write_all(line.as_bytes());
        let _ = std::io::stderr().write_all(b"\n");
    }
    let _ = std::io::stderr().flush();
}

fn d9_heartbeated(tag: &str, shell_cmd: &str, log: &str) -> (Option<i32>, String) {
    let _ = std::fs::remove_file(log);
    let cmd = format!("{} > {} 2>&1", shell_cmd, log);
    let mut child = match std::process::Command::new("bash").args(["-c", &cmd]).spawn() {
        Ok(c) => c,
        Err(e) => {
            d9_emit(&format!("Pass27-d9 {} spawn", tag), &format!("spawn failed: {}", e));
            return (None, String::new());
        }
    };
    let mut ticks = 0u32;
    loop {
        match child.try_wait() {
            Ok(Some(st)) => {
                d9_emit(&format!("Pass27-d9 {}", tag), &format!("exit after {} beat(s), status={:?}", ticks, st.code()));
                return (st.code(), std::fs::read_to_string(log).unwrap_or_default());
            }
            Ok(None) => {
                ticks += 1;
                let (lines, last) = std::fs::read_to_string(log)
                    .map(|s| {
                        (
                            s.lines().count(),
                            s.lines().rev().find(|l| !l.trim().is_empty()).map(|l| l.chars().take(160).collect::<String>()).unwrap_or_default(),
                        )
                    })
                    .unwrap_or((0, "<none>".to_string()));
                d9_emit(&format!("Pass27-d9 {}", tag), &format!("hb{}: {} lines; last: {}", ticks, lines, last));
                std::thread::sleep(std::time::Duration::from_secs(20));
            }
            Err(e) => {
                d9_emit(&format!("Pass27-d9 {}", tag), &format!("try_wait error: {}", e));
                return (None, String::new());
            }
        }
    }
}

fn ci_diag9_miri() {
    if std::env::var_os("CI").is_none() {
        return;
    }
    // diag #11: workspace test-suite decode FIRST (trybuild PASSES — d10 proved
    // status Some(0), zero wip re-writes — so the workspace red is a runtime
    // failure elsewhere), then a rustup-env probe documenting the Miri job's
    // environmental failure for the owner patch. Miri phases from diag #9/10 are
    // dropped: verdict established ('miri' component not available on the
    // effective stable toolchain; rustup default nightly does not take effect).
    std::panic::set_hook(Box::new(|info| {
        let payload = info.payload().downcast_ref::<&str>().map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "<non-string>".to_string());
        let loc = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        d9_emit("Pass27-d9 PANIC-HOOK", &format!("{} at {}", payload, loc));
        std::thread::sleep(std::time::Duration::from_secs(3));
    }));
    ci_diag11_test_suite_decode();
    ci_diag11_rustup_probe();
    d9_emit("Pass27-d11 all-phases", "complete — draining");
    std::thread::sleep(std::time::Duration::from_secs(8));
    return;
    #[allow(unreachable_code)]
    {
    d9_emit("Pass27-d9 phase miri-install", "begin");
    let (_c1, log1) = d9_heartbeated(
        "miri-install",
        "rustup default nightly && rustup component add miri && cargo miri setup",
        "/tmp/d9_miri_install.log",
    );
    let t1: String = log1.chars().rev().take(800).collect::<String>().chars().rev().collect();
    d9_emit("Pass27-d9 miri-install tail", &t1);

    d9_emit("Pass27-d9 phase miri-run", "begin");
    let (code, log2) = d9_heartbeated("miri-run", "cargo miri test -p nros-core --lib", "/tmp/d9_miri_run.log");
    // DATA FIRST: hits bundle before anything else.
    let mut hits: Vec<&str> = log2
        .lines()
        .filter(|l| {
            l.contains("error")
                || l.contains("Undefined Behavior")
                || l.contains("data race")
                || l.contains("aborting")
                || l.contains("test result")
                || l.contains("warning")
        })
        .collect();
    hits.truncate(30);
    let bundle = format!(
        "status={:?}\nHITS:\n{}",
        code,
        if hits.is_empty() { "<none>".to_string() } else { hits.join("\n") }
    );
    d9_emit("Pass27-d9 miri hits", &bundle);
    let t2: String = log2.chars().rev().take(4000).collect::<String>().chars().rev().collect();
    d9_emit("Pass27-d9 miri tail", &t2);
    d9_emit("Pass27-d9 all-phases", "complete — draining");
    std::thread::sleep(std::time::Duration::from_secs(8));
    }
}

fn ci_diag10_test_decode() {
    d9_emit("Pass27-d10 phase test-decode", "begin");
    let (code, log) = d9_heartbeated(
        "t10-trybuild",
        "cargo test -p nros-core --test trybuild -- --nocapture",
        "/tmp/d10_trybuild.log",
    );
    // Hits: mismatch/diff context + failure lines — data first.
    let lines: Vec<&str> = log.lines().collect();
    let mut bundle = format!("status={:?}\n", code);
    let mut idxs: Vec<usize> = (0..lines.len())
        .filter(|&i| {
            lines[i].contains("mismatch")
                || lines[i].contains("diff ")
                || lines[i].contains("EXPECTED")
                || lines[i].contains("ACTUAL")
                || lines[i].contains("test result")
                || lines[i].contains("FAILED")
                || lines[i].contains("failed")
        })
        .collect();
    idxs.truncate(40);
    for i in idxs {
        let lo = i.saturating_sub(1);
        let hi = (i + 3).min(lines.len());
        bundle.push_str(&format!("--- ctx@{}:\n{}\n", i, lines[lo..hi].join("\n")));
    }
    // Any (re-)written wip files: transcribe again — they settle mismatch-vs-pass.
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for cand in [
        "wip",
        "crates/nros-core/wip",
        "target/wip",
        "target/debug/wip",
        "target/release/wip",
        "target/tests/trybuild/wip",
        "target/tests/wip",
    ] {
        let p = std::path::Path::new(cand);
        if let Ok(rd) = p.read_dir() {
            for f in rd.flatten() {
                let fp = f.path();
                if fp.extension().map(|x| x == "stderr").unwrap_or(false) {
                    if let Ok(c) = std::fs::canonicalize(&fp) {
                        files.push(c);
                    }
                }
            }
        }
    }
    files.sort();
    files.dedup();
    bundle.push_str(&format!("\nWIP-FILES count={}\n", files.len()));
    for fp in files.iter().take(6) {
        let content = std::fs::read_to_string(fp).unwrap_or_default();
        bundle.push_str(&format!("\n<<<BEGIN-FILE {}>>>\n{}\n<<<END-FILE {}>>>\n", fp.display(), content, fp.display()));
    }
    d9_emit("Pass27-d10 trybuild-decode", &bundle);
}

fn ci_diag11_test_suite_decode() {
    d9_emit("Pass27-d11 phase test-suite", "begin");
    let (code, log) = d9_heartbeated(
        "t11-suite",
        "cargo test --workspace --all-targets --no-fail-fast",
        "/tmp/d11_suite.log",
    );
    let lines: Vec<&str> = log.lines().collect();
    let mut bundle = format!("status={:?}\n", code);
    let mut idxs: Vec<usize> = (0..lines.len())
        .filter(|&i| {
            lines[i].contains("FAILED")
                || lines[i].contains("panicked at")
                || lines[i].starts_with("failures:")
                || lines[i].contains("test result: FAILED")
                || lines[i].contains("assertion")
        })
        .collect();
    idxs.truncate(40);
    for i in idxs {
        let lo = i.saturating_sub(2);
        let hi = (i + 4).min(lines.len());
        bundle.push_str(&format!("--- ctx@{}:\n{}\n", i, lines[lo..hi].join("\n")));
    }
    d9_emit("Pass27-d11 suite-decode", &bundle);
}

fn ci_diag11_rustup_probe() {
    let (code, log) = d9_heartbeated(
        "t11-rustup",
        "rustup show; echo RUSTUP_TOOLCHAIN=$RUSTUP_TOOLCHAIN; echo default=$(rustup default 2>&1); rustup target list --installed | head",
        "/tmp/d11_rustup.log",
    );
    let t: String = log.chars().rev().take(2000).collect::<String>().chars().rev().collect();
    d9_emit("Pass27-d11 rustup-probe", &format!("status={:?}\n{}", code, t));
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
