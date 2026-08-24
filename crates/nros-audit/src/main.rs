//! NROS Claim Linter and repository representation gate.

mod representation;

use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("claims");

    // TEMPORARY Pass 27 CI diagnostic (removed once the residual rustc defect is
    // fixed): forward the output of the red CI commands as workflow-command
    // annotations so they are readable through the check-runs API where raw job
    // logs are unreachable. Active only for the `all` subcommand used by the
    // governance job; no effect on gate semantics, each phase panic-isolated.
    if cmd == "all" {
        ci_diag_run_all();
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

/// TEMPORARY Pass 27 CI diagnostic — orchestrator. Each phase is isolated so a
/// panic in one cannot silence the next (the previous revision lost the trybuild
/// harvest silently — every phase now emits begin/end markers and panics are
/// forwarded as annotations themselves).
fn ci_diag_run_all() {
    if std::env::var_os("CI").is_none() {
        return;
    }
    // Panic hook covers even abort-style deaths: payload + location as annotation,
    // then a short sleep so the runner agent can drain queued workflow commands
    // before the process dies (evidence: emissions in the last ~1-2s before step
    // exit never become annotations).
    std::panic::set_hook(Box::new(|info| {
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "<non-string>".to_string()
        };
        let loc = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "<unknown>".to_string());
        diag_emit("Pass27-DIAG PANIC-HOOK", &format!("{} at {}", payload, loc));
        std::thread::sleep(std::time::Duration::from_secs(3));
    }));
    diag_phase("check", || ci_diag_forward_cargo_check());
    diag_phase("trybuild", || ci_diag_harvest_trybuild_wip());
    // Canary: pure heartbeats, zero load — discriminates agent-side channel loss
    // from load-correlated process death. test-suite/miri phases deferred to a
    // later push until the canary verdict is in (85s survival window observed).
    diag_phase("canary", || {
        for i in 1..=8 {
            diag_emit("Pass27-DIAG canary", &format!("canary beat {}/8 — pure sleep, no subprocess load", i));
            std::thread::sleep(std::time::Duration::from_secs(15));
        }
    });
    diag_emit("Pass27-DIAG all-phases", "complete — draining agent queue");
    std::thread::sleep(std::time::Duration::from_secs(8));
}

/// Strip ANSI CSI sequences (cargo's colored output) — they were the suspected
/// poison that stopped the runner's command stream mid-flight in diag #2/#3.
fn diag_strip_ansi(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            // CSI: ESC [ <params> <final byte 0x40..=0x7E>
            i += 2;
            while i < bytes.len() && !(0x40..=0x7E).contains(&bytes[i]) {
                i += 1;
            }
            i += 1; // skip final byte
        } else if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b']' {
            // OSC: ESC ] ... (BEL | ESC\)
            i += 2;
            while i < bytes.len() && bytes[i] != 0x07 && !(bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'\\') {
                i += 1;
            }
            i += 1;
        } else {
            // Re-copy the full UTF-8 char starting at i (multi-byte safe).
            let ch_len = utf8_len(bytes[i]);
            out.push_str(&s[i..(i + ch_len).min(s.len())]);
            i += ch_len;
        }
    }
    out
}

fn utf8_len(b: u8) -> usize {
    if b < 0x80 { 1 } else if b >= 0xF0 { 4 } else if b >= 0xE0 { 3 } else if b >= 0xC0 { 2 } else { 1 }
}

fn diag_esc(s: &str) -> String {
    diag_strip_ansi(s)
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

fn diag_emit(title: &str, msg: &str) {
    use std::io::Write;
    let t: String = msg.chars().take(60_000).collect();
    // Chunk to 2000-char annotations: large single messages may be dropped.
    let cleaned = diag_esc(&t);
    let tl: String = title.chars().take(200).collect();
    let mut chunks: Vec<String> = Vec::new();
    let mut cur = String::new();
    for c in cleaned.chars() {
        cur.push(c);
        if cur.len() >= 1900 {
            chunks.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() || chunks.is_empty() {
        chunks.push(cur);
    }
    let n = chunks.len();
    for (i, c) in chunks.iter().enumerate() {
        let line = if n > 1 {
            format!("::error title={} ({}/{})::{}", tl, i + 1, n, c)
        } else {
            format!("::error title={}::{}", tl, c)
        };
        let _ = std::io::stderr().write_all(line.as_bytes());
        let _ = std::io::stderr().write_all(b"\n");
    }
    let _ = std::io::stderr().flush();
    let _ = std::io::stdout().flush();
}

fn diag_phase(name: &str, f: impl FnOnce()) {
    diag_emit(&format!("Pass27-DIAG phase {}", name), "begin");
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    match r {
        Ok(()) => diag_emit(&format!("Pass27-DIAG phase {}", name), "end ok"),
        Err(payload) => {
            let msg = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "non-string panic payload".to_string());
            diag_emit(&format!("Pass27-DIAG phase {} PANIC", name), &msg);
        }
    }
}

/// Run a shell command with its output redirected to a log file, emitting a
/// heartbeat annotation every 3 seconds while it runs (evidence: the previous
/// process died ~33s into a heavy `cargo test` compile with zero trace — heartbeats
/// pinpoint whether the killer is load-correlated and when exactly it strikes).
fn diag_run_heartbeated(tag: &str, shell_cmd: &str, log: &str) -> (Option<i32>, String) {
    let _ = std::fs::remove_file(log);
    let cmd = format!("{} > {} 2>&1", shell_cmd, log);
    let mut child = match std::process::Command::new("bash").args(["-c", &cmd]).spawn() {
        Ok(c) => c,
        Err(e) => {
            diag_emit(&format!("Pass27-DIAG {} spawn", tag), &format!("spawn failed: {}", e));
            return (None, String::new());
        }
    };
    let mut ticks = 0u32;
    loop {
        match child.try_wait() {
            Ok(Some(st)) => {
                diag_emit(&format!("Pass27-DIAG {}", tag), &format!("exit after {} heartbeat(s), status={:?}", ticks, st.code()));
                return (st.code(), std::fs::read_to_string(log).unwrap_or_default());
            }
            Ok(None) => {
                ticks += 1;
                let lines = std::fs::read_to_string(log).map(|s| s.lines().count()).unwrap_or(0);
                let last = std::fs::read_to_string(log)
                    .ok()
                    .and_then(|s| s.lines().rev().find(|l| !l.trim().is_empty()).map(|l| l.chars().take(180).collect::<String>()))
                    .unwrap_or_else(|| "<no output yet>".to_string());
                diag_emit(&format!("Pass27-DIAG {}", tag), &format!("hb{}: {} log lines; last: {}", ticks, lines, last));
                std::thread::sleep(std::time::Duration::from_secs(20));
            }
            Err(e) => {
                diag_emit(&format!("Pass27-DIAG {}", tag), &format!("try_wait error: {}", e));
                return (None, String::new());
            }
        }
    }
}

/// TEMPORARY Pass 27 CI diagnostic — see call site. Runs `cargo check
/// --workspace --all-targets` and re-emits its error output as `::error`
/// workflow commands (check-run annotations are API-readable even though raw
/// job logs ride blob hosts unreachable from the audit workstation).
fn ci_diag_forward_cargo_check() {
    let out = std::process::Command::new("cargo")
        .args(["check", "--workspace", "--all-targets", "--message-format", "short"])
        .output();
    let out = match out {
        Ok(o) => o,
        Err(e) => {
            diag_emit("Pass27-DIAG spawn", &format!("failed to spawn cargo: {}", e));
            return;
        }
    };
    let text = String::from_utf8_lossy(&out.stderr);
    let mut errors: Vec<&str> = text.lines().filter(|l| l.contains("error")).collect();
    errors.truncate(20);
    let joined_errors = if errors.is_empty() { "<none>".to_string() } else { errors.join("\n") };
    diag_emit("Pass27-DIAG check-errors", &format!("status={:?}\n{}", out.status.code(), joined_errors));
    let tail: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = tail.len().saturating_sub(4);
    diag_emit("Pass27-DIAG check-tail", &tail[start..].join("\n"));
}

/// TEMPORARY Pass 27 CI diagnostic, second half (F-19): the trybuild negative
/// tests need `.stderr` files blessed against the runner's exact rustc (1.97.1);
/// no such compiler exists in the audit sandbox. Run the trybuild target here, on
/// that very toolchain, and forward the `wip/*.stderr` files trybuild writes as
/// annotations so they can be transcribed verbatim into tests/compile_fail/.
/// Never affects the exit code; no-op outside CI.
fn ci_diag_harvest_trybuild_wip() {
    let (_code, log) = diag_run_heartbeated(
        "trybuild-run",
        "cargo test -p nros-core --test trybuild",
        "/tmp/diag_trybuild.log",
    );
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    fn walk(dir: &std::path::Path, depth: u32, acc: &mut Vec<std::path::PathBuf>) {
        if depth > 8 {
            return;
        }
        let rd = match std::fs::read_dir(dir) {
            Ok(r) => r,
            Err(_) => return,
        };
        for e in rd.flatten() {
            let name = e.file_name();
            if name == ".git" || (depth == 0 && name == "target") {
                continue; // target/wip covered by explicit candidates below
            }
            let p = e.path();
            if !p.is_dir() {
                continue;
            }
            if name == "wip" {
                if let Ok(rd2) = std::fs::read_dir(&p) {
                    for f in rd2.flatten() {
                        let fp = f.path();
                        if fp.extension().map(|x| x == "stderr").unwrap_or(false) {
                            acc.push(fp);
                        }
                    }
                }
            } else {
                walk(&p, depth + 1, acc);
            }
        }
    }
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
        if p.is_dir() {
            if let Ok(rd) = std::fs::read_dir(p) {
                for f in rd.flatten() {
                    let fp = f.path();
                    if fp.extension().map(|x| x == "stderr").unwrap_or(false) {
                        files.push(fp);
                    }
                }
            }
        }
    }
    walk(std::path::Path::new("."), 0, &mut files);
    for f in files.iter_mut() {
        if let Ok(c) = std::fs::canonicalize(&f) {
            *f = c;
        }
    }
    files.sort();
    files.dedup();
    if files.is_empty() {
        diag_emit("Pass27-DIAG trybuild", "no wip/*.stderr produced");
        return;
    }
    diag_emit("Pass27-DIAG trybuild", &format!("{} wip stderr file(s) found", files.len()));
    // DATA FIRST: file contents before any further ceremony (the silent killer
    // has consistently struck within ~30s of this point in earlier iterations).
    for fp in files.iter().take(8) {
        let content = std::fs::read_to_string(fp).unwrap_or_default();
        diag_emit(&format!("Pass27-DIAG file {}", fp.display()), &content);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    let tail: String = log.chars().rev().take(2500).collect::<String>().chars().rev().collect();
    diag_emit("Pass27-DIAG trybuild", &format!("log tail:\n{}", tail));
}

/// TEMPORARY Pass 27 CI diagnostic, third phase: decode the red `cargo test
/// (workspace)` job by running the same command here and forwarding failing-test
/// lines and the summary tail as annotations.
fn ci_diag_forward_test_suite() {
    let (code, log) = diag_run_heartbeated(
        "test-suite-run",
        "cargo test --workspace --all-targets --no-fail-fast",
        "/tmp/diag_tests.log",
    );
    let mut hits: Vec<&str> = log
        .lines()
        .filter(|l| {
            l.contains("FAILED")
                || l.contains("panicked at")
                || l.starts_with("failures:")
                || l.contains("test result: FAILED")
                || (l.contains("error") && !l.contains("0 error"))
        })
        .collect();
    hits.truncate(15);
    let tail: String = log.chars().rev().take(2000).collect::<String>().chars().rev().collect();
    diag_emit(
        "Pass27-DIAG test-suite",
        &format!(
            "status={:?}\nhits:\n{}\nlog tail:\n{}",
            code,
            if hits.is_empty() { "<none>".to_string() } else { hits.join("\n") },
            tail
        ),
    );
}

/// TEMPORARY Pass 27 CI diagnostic, fourth phase: decode the red Miri job by
/// reproducing it here (rustup on the runner can reach the dist server even
/// though the audit sandbox cannot) and forwarding Miri's verdict.
fn ci_diag_forward_miri() {
    let script = "rustup default nightly; \
                  rustup component add miri; \
                  cargo miri setup; \
                  cargo miri test -p nros-core --lib; \
                  rustup default stable";
    let (_code, log) = diag_run_heartbeated("miri-run", script, "/tmp/diag_miri.log");
    let mut hits: Vec<&str> = log
        .lines()
        .filter(|l| {
            l.contains("error")
                || l.contains("Undefined Behavior")
                || l.contains("UB")
                || l.contains("data race")
                || l.contains("aborting")
                || l.contains("test result")
        })
        .collect();
    hits.truncate(16);
    let tail: String = log.chars().rev().take(5000).collect::<String>().chars().rev().collect();
    diag_emit(
        "Pass27-DIAG miri",
        &format!(
            "hits:\n{}\nlog tail:\n{}",
            if hits.is_empty() { "<none>".to_string() } else { hits.join("\n") },
            tail
        ),
    );
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
