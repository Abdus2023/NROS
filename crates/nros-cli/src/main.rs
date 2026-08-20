//! nros binary — CLI entry point per DESIGN.md §7.1
//! Usage: nros init my_robot --template=mobile_base
//!        nros build --profile=realtime
//!        nros run --inspect
//!        nros topic list / echo / hz / bw
//!        nros fleet deploy --version=1.1.0 --canary=10%

use nros_cli::{Command, BuildProfile, TopicAction, CLI};
use std::env;
use std::path::PathBuf;
use std::time::Duration;

fn print_help() {
    println!(r#"
NROS CLI — Native Robotics Operating System tools

USAGE:
    nros <COMMAND> [OPTIONS]

COMMANDS:
    init <name> [--template=mobile_base|manipulator|perception|basic]  Create project
    build [--profile=debug|release|realtime|embedded] [--target=...]   Build project
    run [--inspect] [node]                                            Run nodes
    topic list | info <name> | echo <name> | hz <name> | bw <name>    Topic tools
    record <topics> --output=file --duration=sec                      Record binary .nros
    replay <file> [--speed=0.5] [--loop]                              Replay recording
    analyze <file> --bandwidth|--latency|--timing|--graph             Analyze capture
    profile [--duration=60s] [--focus=...]                            Profile with flamegraph
    fleet list | deploy --version=X --canary=N | status | exec        Fleet mgmt per §21.2
    migrate analyze <path> | convert <in> <out>                       ROS2 migration per §22
    check --timing --graph                                            Static analysis

EXAMPLES:
    nros init my_robot --template=mobile_base
    nros build --profile=realtime  # -O3, LTO, static linking
    nros run --inspect  # Opens http://localhost:8080 NROS Studio live graph
    nros record /camera/* /lidar --duration=10s
    nros replay recording.nros --speed=0.5 --analyze-latency
    nros topic list
    nros fleet list
    nros cloud login --fleet=warehouse_fleet
    nros cloud deploy --version=1.1.0 --canary=10%

See DESIGN.md §7.1 CLI Tools, §20 Workflow, §21 Deployment.
"#);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let cmd = args[1].as_str();

    // Structured result protocol per AUDIT Pass 10 recommendation
    // Every CLI operation should return deterministic result contract: Command -> structured request -> backend -> real state transition -> structured result -> exit code
    // For now, we print both human-readable and JSON result with status simulated/unsupported/success

    let result = match cmd {
        "init" => {
            if args.len() < 3 {
                eprintln!("Usage: nros init <name> [--template=...]");
                return;
            }
            let name = args[2].clone();
            let template = args.iter().find(|a| a.starts_with("--template=")).map(|s| s.split('=').nth(1).unwrap_or("basic").to_string());
            CLI::run(Command::Init { name, template })
        }
        "build" => {
            let profile_str = args.iter().find(|a| a.starts_with("--profile=")).map(|s| s.split('=').nth(1).unwrap_or("debug")).unwrap_or("debug");
            let profile = BuildProfile::from_str(profile_str).unwrap_or(BuildProfile::Debug);
            let target = args.iter().find(|a| a.starts_with("--target=")).map(|s| s.split('=').nth(1).unwrap_or("").to_string());
            let res = CLI::run(Command::Build { profile, target });
            println!("\n{{\"status\":\"simulated\",\"operation\":\"build\",\"reason\":\"cargo backend not installed, simulated sizes 950KB/480KB, would measure target binary via fs::metadata in real\",\"verified\":false}}");
            res
        }
        "run" => {
            let inspect = args.contains(&"--inspect".to_string());
            let node = if args.len() > 2 && !args[2].starts_with("--") { Some(args[2].clone()) } else { None };
            CLI::run(Command::Run { node, inspect })
        }
        "topic" => {
            if args.len() < 3 {
                eprintln!("Usage: nros topic <list|info|echo|hz|bw> [name]");
                return;
            }
            let action = match args[2].as_str() {
                "list" => TopicAction::List,
                "info" => TopicAction::Info { name: args.get(3).cloned().unwrap_or_else(|| "/cmd_vel".to_string()) },
                "echo" => TopicAction::Echo { name: args.get(3).cloned().unwrap_or_else(|| "/chatter".to_string()) },
                "hz" => TopicAction::Hz { name: args.get(3).cloned().unwrap_or_else(|| "/cmd_vel".to_string()) },
                "bw" => TopicAction::Bw { name: args.get(3).cloned().unwrap_or_else(|| "/camera/image".to_string()) },
                "pub" => TopicAction::Pub { name: args.get(3).cloned().unwrap_or_else(|| "/cmd_vel".to_string()), data: args.get(4).cloned().unwrap_or_else(|| "{}".to_string()) },
                _ => TopicAction::List,
            };
            let res = CLI::run(Command::Topic { action });
            println!("\n{{\"status\":\"simulated\",\"operation\":\"topic\",\"reason\":\"topic discovery and transport backend not implemented, hard-coded topics /cmd_vel, /odom, /camera/image with latency 5.2μs\",\"verified\":false}}");
            res
        }
        "record" => {
            let topics: Vec<String> = args.iter().skip(2).filter(|a| !a.starts_with("--")).cloned().collect();
            let output = args.iter().find(|a| a.starts_with("--output=")).map(|s| PathBuf::from(s.split('=').nth(1).unwrap_or("recording.nros"))).unwrap_or_else(|| PathBuf::from("recording.nros"));
            let duration = args.iter().find(|a| a.starts_with("--duration=")).map(|s| {
                let secs_str = s.split('=').nth(1).unwrap_or("10s");
                let secs = secs_str.trim_end_matches('s').parse::<u64>().unwrap_or(10);
                Duration::from_secs(secs)
            });
            let res = CLI::run(Command::Record { topics, output, duration });
            println!("\n{{\"status\":\"simulated\",\"operation\":\"record\",\"reason\":\"recording backend not implemented, no file serialization, prints Saved but no artifact\",\"verified\":false}}");
            res
        }
        "replay" => {
            let input = args.get(2).map(|s| PathBuf::from(s)).unwrap_or_else(|| PathBuf::from("recording.nros"));
            let speed = args.iter().find(|a| a.starts_with("--speed=")).map(|s| s.split('=').nth(1).unwrap_or("1.0").parse::<f64>().unwrap_or(1.0)).unwrap_or(1.0);
            let loop_playback = args.contains(&"--loop".to_string());
            let res = CLI::run(Command::Replay { input, speed, loop_playback });
            println!("\n{{\"status\":\"simulated\",\"operation\":\"replay\",\"reason\":\"replay backend not implemented, no open file, validate format, read messages, schedule timestamps\",\"verified\":false}}");
            res
        }
        "analyze" => {
            let input = args.get(2).map(|s| PathBuf::from(s)).unwrap_or_else(|| PathBuf::from("recording.nros"));
            let analysis_type = if args.iter().any(|a| a.contains("bandwidth")) {
                nros_cli::AnalysisType::Bandwidth
            } else if args.iter().any(|a| a.contains("latency")) {
                nros_cli::AnalysisType::Latency
            } else if args.iter().any(|a| a.contains("graph")) {
                nros_cli::AnalysisType::Graph
            } else if args.iter().any(|a| a.contains("timing")) {
                nros_cli::AnalysisType::Timing
            } else {
                nros_cli::AnalysisType::Bandwidth
            };
            let res = CLI::run(Command::Analyze { input, analysis_type });
            println!("\n{{\"status\":\"simulated\",\"operation\":\"analyze\",\"reason\":\"analysis backend not implemented, prints fixed P50=5.8μs P99=12.1μs without reading file\",\"verified\":false}}");
            res
        }
        "profile" => {
            let duration = args.iter().find(|a| a.starts_with("--duration=")).map(|s| {
                let secs_str = s.split('=').nth(1).unwrap_or("60s");
                let secs = secs_str.trim_end_matches('s').parse::<u64>().unwrap_or(60);
                Duration::from_secs(secs)
            }).unwrap_or_else(|| Duration::from_secs(60));
            let focus = args.iter().find(|a| a.starts_with("--focus=")).map(|s| s.split('=').nth(1).unwrap_or("").to_string());
            let res = CLI::run(Command::Profile { duration, focus });
            println!("\n{{\"status\":\"simulated\",\"operation\":\"profile\",\"reason\":\"profiler backend not implemented, hard-coded functions 245ms and flamegraph claim without generating flamegraph\",\"verified\":false}}");
            res
        }
        "fleet" => {
            if args.len() < 3 {
                eprintln!("Usage: nros fleet <list|deploy|status|exec> [options]");
                return;
            }
            let action = match args[2].as_str() {
                "list" => nros_cli::FleetAction::List,
                "deploy" => {
                    let version = args.iter().find(|a| a.starts_with("--version=")).map(|s| s.split('=').nth(1).unwrap_or("1.0.0").to_string()).unwrap_or_else(|| "1.0.0".to_string());
                    let canary = args.iter().find(|a| a.starts_with("--canary=")).map(|s| s.split('=').nth(1).unwrap_or("0").trim_end_matches('%').parse::<u32>().unwrap_or(0));
                    nros_cli::FleetAction::Deploy { version, canary }
                },
                "status" => nros_cli::FleetAction::Status,
                "exec" => {
                    let robot = args.get(3).cloned().unwrap_or_else(|| "robot_001".to_string());
                    let command = args.get(4).cloned().unwrap_or_else(|| "echo hello".to_string());
                    nros_cli::FleetAction::Exec { robot, command }
                },
                _ => nros_cli::FleetAction::List,
            };
            let res = CLI::run(Command::Fleet { action });
            println!("\n{{\"status\":\"simulated\",\"operation\":\"fleet\",\"reason\":\"fleet backend not implemented: no TLS, auth, OTA, artifact verification, device communication, rollback, health endpoint — prints hard-coded 4 robots and sleeps, reports success without operation\",\"verified\":false}}");
            res
        }
        "migrate" => {
            if args.len() < 3 {
                eprintln!("Usage: nros migrate <analyze|convert> <path>");
                return;
            }
            let action = match args[2].as_str() {
                "analyze" => {
                    let path = args.get(3).map(|s| PathBuf::from(s)).unwrap_or_else(|| PathBuf::from("src"));
                    nros_cli::MigrateAction::Analyze { path }
                },
                "convert" => {
                    let input = args.get(3).map(|s| PathBuf::from(s)).unwrap_or_else(|| PathBuf::from("src"));
                    let output = args.get(4).map(|s| PathBuf::from(s)).unwrap_or_else(|| PathBuf::from("nros_pkg"));
                    nros_cli::MigrateAction::Convert { input, output }
                },
                _ => {
                    let path = args.get(3).map(|s| PathBuf::from(s)).unwrap_or_else(|| PathBuf::from("src"));
                    nros_cli::MigrateAction::Analyze { path }
                }
            };
            let res = CLI::run(Command::Migrate { action });
            println!("\n{{\"status\":\"simulated\",\"operation\":\"migrate\",\"reason\":\"migration engine scaffold — prints text create_publisher<Msg> -> publish<T> without AST transformation, source rewriting, message conversion, validation\",\"verified\":false}}");
            res
        }
        "check" => {
            let timing = args.contains(&"--timing".to_string());
            let graph = args.contains(&"--graph".to_string());
            let res = CLI::run(Command::Check { timing, graph });
            println!("\n{{\"status\":\"simulated\",\"operation\":\"check\",\"reason\":\"static analysis gate not implemented — prints WCET analysis without loading graph, no YAML parsed, no cycle detection, no timing analysis\",\"verified\":false}}");
            res
        }
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        _ => {
            println!("Unknown command: {} — showing help", cmd);
            print_help();
            println!("\n{{\"status\":\"unsupported\",\"operation\":\"{}\",\"reason\":\"command not dispatched — help shown\",\"verified\":false}}", cmd);
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        println!("{{\"status\":\"failed\",\"error\":\"{}\"}}", e);
        std::process::exit(1);
    }
}
