//! nros-studio binary — Live inspector dashboard server
//! Per DESIGN.md §7.2 NROS Studio, §20.3 `nros run --inspect` opens http://localhost:8080

use nros_studio::StudioServer;

fn main() {
    // Per arena requirements: bind 0.0.0.0 not 127.0.0.1 for preview host
    // Preview env: https://{port}-{sandboxId}.e2b.app
    let addr = std::env::var("NROS_STUDIO_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    let server = StudioServer::new(&addr);

    println!("╔════════════════════════════════════════╗");
    println!("║         NROS Studio Server            ║");
    println!("║   Live Monitoring & Visualization     ║");
    println!("╚════════════════════════════════════════╝");
    println!();
    println!("Features per DESIGN.md §7.2:");
    println!(" - Live node graph with message flow animation (SVG layer)");
    println!(" - 3D visualization with automatic TF handling");
    println!(" - Timeline view with message timestamps");
    println!(" - Performance metrics dashboard (latency, throughput, CPU, memory)");
    println!(" - Remote debugging with breakpoint support");
    println!(" - Parameter values with live editing");
    println!();

    if let Err(e) = server.run() {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
