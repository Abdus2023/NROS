//! NROS CLI Tools Demo — Full interactive showcase per DESIGN.md §7, §20, §21

use nros_cli::{Command, BuildProfile, TopicAction, FleetAction, CLI};
use std::time::Duration;
use std::path::PathBuf;

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║   NROS Command-Line Interface Demo    ║");
    println!("║   Implements §7.1, §20, §21, §22      ║");
    println!("╚════════════════════════════════════════╝\n");

    // Demo: Init project
    println!("=== Command: nros init my_robot --template=mobile_base ===\n");
    // Pass 27 fix (first real CI run, 2026-08-22): previously the demo passed an absolute
    // temp path as the project NAME. `ProjectInitializer::is_valid_project_name` rejects
    // absolute paths (they contain '/'), so `nros-cli-demo` always panicked at this
    // `.unwrap()` — the advertised `cargo run -p nros-cli --bin nros-cli-demo` could never
    // complete. Create the temp dir, chdir into it, and use a plain relative name instead.
    let tmp_dir = std::env::temp_dir().join(format!("nros_demo_init_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
    std::fs::create_dir_all(&tmp_dir).expect("create demo temp dir");
    std::env::set_current_dir(&tmp_dir).expect("chdir into demo temp dir");
    let demo_proj_name = "my_robot".to_string();
    CLI::run(Command::Init {
        name: demo_proj_name.clone(),
        template: Some("mobile_base".to_string()),
    }).unwrap();
    println!("   (demo project generated under {})", tmp_dir.display());

    std::thread::sleep(Duration::from_millis(800));

    // Demo: Build
    println!("\n\n=== Command: nros build --profile=realtime ===\n");
    CLI::run(Command::Build {
        profile: BuildProfile::Realtime,
        target: None,
    }).unwrap();

    std::thread::sleep(Duration::from_millis(800));

    // Demo: Build embedded
    println!("\n\n=== Command: nros build --profile=embedded --target=armv7-unknown-linux-gnueabihf ===\n");
    CLI::run(Command::Build {
        profile: BuildProfile::Embedded,
        target: Some("armv7-unknown-linux-gnueabihf".to_string()),
    }).unwrap();

    std::thread::sleep(Duration::from_millis(800));

    // Demo: Topic list
    println!("\n\n=== Command: nros topic list ===\n");
    CLI::run(Command::Topic {
        action: TopicAction::List,
    }).unwrap();

    std::thread::sleep(Duration::from_millis(800));

    // Demo: Topic info
    println!("\n\n=== Command: nros topic info /cmd_vel ===\n");
    CLI::run(Command::Topic {
        action: TopicAction::Info { name: "/cmd_vel".to_string() },
    }).unwrap();

    std::thread::sleep(Duration::from_millis(800));

    // Demo: Topic hz
    println!("\n\n=== Command: nros topic hz /cmd_vel ===\n");
    CLI::run(Command::Topic {
        action: TopicAction::Hz { name: "/cmd_vel".to_string() },
    }).unwrap();

    std::thread::sleep(Duration::from_millis(800));

    // Demo: Profile
    println!("\n\n=== Command: nros profile --duration=10s ===\n");
    CLI::run(Command::Profile {
        duration: Duration::from_secs(10),
        focus: None,
    }).unwrap();

    std::thread::sleep(Duration::from_millis(800));

    // Demo: Record
    println!("\n\n=== Command: nros record /camera/* /lidar --duration=10s ===\n");
    CLI::run(Command::Record {
        topics: vec!["/camera/*".into(), "/lidar".into()],
        output: PathBuf::from("recording.nros"),
        duration: Some(Duration::from_secs(10)),
    }).unwrap();

    std::thread::sleep(Duration::from_millis(800));

    // Demo: Analyze
    println!("\n\n=== Command: nros analyze recording.nros --bandwidth ===\n");
    CLI::run(Command::Analyze {
        input: PathBuf::from("recording.nros"),
        analysis_type: nros_cli::AnalysisType::Bandwidth,
    }).unwrap();

    // Demo: Fleet list
    println!("\n\n=== Command: nros fleet list (fleet.yaml §21.2) ===\n");
    CLI::run(Command::Fleet {
        action: FleetAction::List,
    }).unwrap();

    std::thread::sleep(Duration::from_millis(800));

    // Demo: Fleet status
    println!("\n\n=== Command: nros fleet status (cloud telemetry §21.3) ===\n");
    CLI::run(Command::Fleet {
        action: FleetAction::Status,
    }).unwrap();

    std::thread::sleep(Duration::from_millis(800));

    // Demo: Fleet deploy
    println!("\n\n=== Command: nros fleet deploy --version=1.1.0 --canary=25 (OTA §8.2) ===\n");
    CLI::run(Command::Fleet {
        action: FleetAction::Deploy {
            version: "1.1.0".to_string(),
            canary: Some(25),
        },
    }).unwrap();

    println!("\n\n╔════════════════════════════════════════╗");
    println!("║        CLI Demo Complete!              ║");
    println!("║  Validates DESIGN.md §7, §20, §21, §22 ║");
    println!("╚════════════════════════════════════════╝");
    println!("\nNext steps:");
    println!("  nros run --inspect  # http://localhost:8080 live graph, metrics, 3D viz");
    println!("  nros check --timing # WCET analysis");
    println!("  nros check --graph  # Validate robot.graph.yaml inputs/outputs/type/cycle");
}
