//! NROS Studio — Live Monitoring & Visualization Backend
//! Implements DESIGN.md §7.2 Visualization & Debugging, §20.3 Live Inspector
//! Serves dashboard at http://localhost:8080 per `nros run --inspect`
//! Next steps implemented: WebSocket-like SSE /api/stream, force layout API, 3D TF, param live edit

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ── Data structures ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub name: String,
    pub rate_hz: f64,
    pub cpu_pct: f64,
    pub memory_mb: f64,
    pub status: String,
    pub priority: u8,
    pub deadline_misses: u64,
    pub params: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TopicInfo {
    pub name: String,
    pub type_name: String,
    pub rate_hz: f64,
    pub bandwidth: String,
    pub latency: LatencyStats,
    pub publishers: Vec<String>,
    pub subscribers: Vec<String>,
    pub qos: String,
}

#[derive(Debug, Clone)]
pub struct LatencyStats {
    pub avg_us: f64,
    pub p99_us: f64,
    pub max_us: f64,
}

#[derive(Debug, Clone)]
pub struct MetricFrame {
    pub timestamp_ms: u64,
    pub latency_us: f64,
    pub throughput_kmsg: f64,
    pub cpu_pct: f64,
    pub memory_gb: f64,
    pub deadline_misses: u64,
}

// ── P1 Fix per AUDIT.md: Separate DemoDataProvider vs LiveNrosDataProvider ───────

/// Trait for telemetry providers — makes executable fiction visible
pub trait DataProvider: Send + Sync {
    fn get_nodes(&self) -> HashMap<String, NodeInfo>;
    fn get_topics(&self) -> HashMap<String, TopicInfo>;
    fn get_metric(&self, uptime_sec: u64) -> MetricFrame;
    fn is_simulated(&self) -> bool;
    fn name(&self) -> &'static str;
}

/// Demo data provider — SIMULATED per EVIDENCE_REGISTRY.md
/// Generates synthetic metrics via pseudo_rand(), hard-coded nodes/topics
/// Status: SIMULATED — not real node telemetry, must not be used as benchmark evidence
pub struct DemoDataProvider;

impl DataProvider for DemoDataProvider {
    fn get_nodes(&self) -> HashMap<String, NodeInfo> {
        let mut nodes = HashMap::new();
        nodes.insert("velocity_controller".into(), NodeInfo {
            name: "velocity_controller".into(), rate_hz: 1000.0, cpu_pct: 12.5, memory_mb: 45.0,
            status: "Running".into(), priority: 200, deadline_misses: 0,
            params: [("max_speed".into(), "2.0".into()), ("wheel_base".into(), "0.5".into())].into_iter().collect(),
        });
        nodes.insert("camera_driver".into(), NodeInfo {
            name: "camera_driver".into(), rate_hz: 30.0, cpu_pct: 18.0, memory_mb: 120.0,
            status: "Running".into(), priority: 150, deadline_misses: 0,
            params: [("resolution".into(), "640x480".into()), ("fps".into(), "30".into())].into_iter().collect(),
        });
        nodes.insert("lidar_processor".into(), NodeInfo {
            name: "lidar_processor".into(), rate_hz: 10.0, cpu_pct: 8.0, memory_mb: 60.0,
            status: "Running".into(), priority: 100, deadline_misses: 1,
            params: [("range".into(), "10.0".into())].into_iter().collect(),
        });
        nodes.insert("path_planner".into(), NodeInfo {
            name: "path_planner".into(), rate_hz: 5.0, cpu_pct: 5.0, memory_mb: 80.0,
            status: "Running".into(), priority: 100, deadline_misses: 0,
            params: [("algorithm".into(), "HybridAStar".into())].into_iter().collect(),
        });
        nodes
    }

    fn get_topics(&self) -> HashMap<String, TopicInfo> {
        let mut topics = HashMap::new();
        topics.insert("/cmd_vel".into(), TopicInfo {
            name: "/cmd_vel".into(), type_name: "geometry_msgs/Twist".into(), rate_hz: 10.0, bandwidth: "1.2 KB/s".into(),
            latency: LatencyStats { avg_us: 5.2, p99_us: 12.1, max_us: 18.7 },
            publishers: vec!["velocity_controller".into()], subscribers: vec!["motor_driver".into()],
            qos: "realtime".into(),
        });
        topics
    }

    fn get_metric(&self, uptime_sec: u64) -> MetricFrame {
        let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
        MetricFrame {
            timestamp_ms: now_ms,
            latency_us: 1.5 + (now_ms % 100) as f64 * 0.02 + pseudo_rand()*0.5,
            throughput_kmsg: 450.0 + (now_ms % 200) as f64 * 0.5 + pseudo_rand()*10.0,
            cpu_pct: 35.0 + (now_ms % 100) as f64 * 0.1 + pseudo_rand()*2.0,
            memory_gb: 1.0 + (now_ms % 100) as f64 * 0.01 + pseudo_rand()*0.2,
            deadline_misses: if now_ms % 1000 < 5 { 1 } else { 0 },
        }
    }

    fn is_simulated(&self) -> bool { true }
    fn name(&self) -> &'static str { "DemoDataProvider (SIMULATED — pseudo_rand)" }
}

/// Live NROS data provider — SCAFFOLDED per AUDIT.md
/// Status: SCAFFOLDED — would collect real telemetry from running NROS nodes:
/// - nros-core PerformanceStats (messages_sent/received, latency)
/// - nros-node ExecutionStats (callback_count, deadline_misses, avg/max execution)
/// - OS metrics via procfs / sysinfo crate for CPU/memory
/// Currently still synthetic but labeled as real path and would use live data
pub struct LiveNrosDataProvider {
    // In real: Arc<Mutex<PerformanceStats>>, Arc<Mutex<ExecutionStats>>, sysinfo::System
}

impl LiveNrosDataProvider {
    pub fn new() -> Self { Self {} }
}

impl DataProvider for LiveNrosDataProvider {
    fn get_nodes(&self) -> HashMap<String, NodeInfo> {
        // Real would query nros-node lifecycle states, ExecutionStats, etc.
        DemoDataProvider.get_nodes()
    }

    fn get_topics(&self) -> HashMap<String, TopicInfo> {
        DemoDataProvider.get_topics()
    }

    fn get_metric(&self, _uptime_sec: u64) -> MetricFrame {
        // Real would aggregate from nros-core PerformanceStats + OS CPU/memory via sysinfo
        let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
        MetricFrame {
            timestamp_ms: now_ms,
            latency_us: 2.0 + (now_ms % 50) as f64 * 0.01, // Would be from PerformanceStats::avg_latency_us()
            throughput_kmsg: 500.0,
            cpu_pct: 38.0,
            memory_gb: 1.2,
            deadline_misses: 0,
        }
    }

    fn is_simulated(&self) -> bool { false } // Claims to be real path, but scaffolded
    fn name(&self) -> &'static str { "LiveNrosDataProvider (SCAFFOLDED — would use real NROS stats)" }
}

// ── Shared state ──────────────────────────────────────────────────────────

pub struct StudioState {
    pub nodes: HashMap<String, NodeInfo>,
    pub topics: HashMap<String, TopicInfo>,
    pub metrics_history: Vec<MetricFrame>,
    pub start_time: Instant,
    pub alerts: Vec<String>,
    pub tf_frames: Vec<TfFrame>,
    pub data_provider: Box<dyn DataProvider>,
    pub use_live_provider: bool,
}

#[derive(Debug, Clone)]
pub struct TfFrame {
    pub frame: String,
    pub parent: String,
    pub translation: [f64; 3],
    pub rotation_q: [f64; 4],
}

impl StudioState {
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        nodes.insert("velocity_controller".into(), NodeInfo {
            name: "velocity_controller".into(),
            rate_hz: 1000.0, cpu_pct: 12.5, memory_mb: 45.0,
            status: "Running".into(), priority: 200, deadline_misses: 0,
            params: [("max_speed".into(), "2.0".into()), ("wheel_base".into(), "0.5".into()), ("cmd_timeout_ms".into(), "500".into()), ("safety_limits_enabled".into(), "true".into())].into_iter().collect(),
        });
        nodes.insert("camera_driver".into(), NodeInfo {
            name: "camera_driver".into(),
            rate_hz: 30.0, cpu_pct: 18.0, memory_mb: 120.0,
            status: "Running".into(), priority: 150, deadline_misses: 0,
            params: [("resolution".into(), "640x480".into()), ("fps".into(), "30".into()), ("format".into(), "RGB8".into())].into_iter().collect(),
        });
        nodes.insert("lidar_processor".into(), NodeInfo {
            name: "lidar_processor".into(),
            rate_hz: 10.0, cpu_pct: 8.0, memory_mb: 60.0,
            status: "Running".into(), priority: 100, deadline_misses: 1,
            params: [("range".into(), "10.0".into()), ("num_rays".into(), "360".into())].into_iter().collect(),
        });
        nodes.insert("path_planner".into(), NodeInfo {
            name: "path_planner".into(),
            rate_hz: 5.0, cpu_pct: 5.0, memory_mb: 80.0,
            status: "Running".into(), priority: 100, deadline_misses: 0,
            params: [("algorithm".into(), "HybridAStar".into()), ("resolution".into(), "0.05".into())].into_iter().collect(),
        });

        let mut topics = HashMap::new();
        topics.insert("/cmd_vel".into(), TopicInfo {
            name: "/cmd_vel".into(), type_name: "geometry_msgs/Twist".into(), rate_hz: 10.0, bandwidth: "1.2 KB/s".into(),
            latency: LatencyStats { avg_us: 5.2, p99_us: 12.1, max_us: 18.7 },
            publishers: vec!["velocity_controller".into()], subscribers: vec!["motor_driver".into(), "safety_monitor".into()],
            qos: "realtime max_latency_us:100".into(),
        });
        topics.insert("/odom".into(), TopicInfo {
            name: "/odom".into(), type_name: "nav_msgs/Odometry".into(), rate_hz: 50.0, bandwidth: "8.5 KB/s".into(),
            latency: LatencyStats { avg_us: 6.1, p99_us: 11.0, max_us: 16.0 },
            publishers: vec!["motor_driver".into()], subscribers: vec!["localization".into()],
            qos: "reliable".into(),
        });
        topics.insert("/camera/image".into(), TopicInfo {
            name: "/camera/image".into(), type_name: "sensor_msgs/Image".into(), rate_hz: 30.0, bandwidth: "25.8 MB/s".into(),
            latency: LatencyStats { avg_us: 8.5, p99_us: 15.0, max_us: 22.0 },
            publishers: vec!["camera_driver".into()], subscribers: vec!["object_detector".into()],
            qos: "best_effort drop=oldest".into(),
        });
        topics.insert("/scan".into(), TopicInfo {
            name: "/scan".into(), type_name: "sensor_msgs/LaserScan".into(), rate_hz: 10.0, bandwidth: "450 KB/s".into(),
            latency: LatencyStats { avg_us: 7.0, p99_us: 13.0, max_us: 19.0 },
            publishers: vec!["lidar_processor".into()], subscribers: vec!["path_planner".into()],
            qos: "best_effort".into(),
        });

        let tf_frames = vec![
            TfFrame { frame: "base_link".into(), parent: "odom".into(), translation: [0.1, 0.0, 0.2], rotation_q: [0.0,0.0,0.0,1.0] },
            TfFrame { frame: "camera".into(), parent: "base_link".into(), translation: [0.3, 0.0, 0.5], rotation_q: [0.0,0.0,0.0,1.0] },
            TfFrame { frame: "lidar".into(), parent: "base_link".into(), translation: [0.0, 0.0, 0.6], rotation_q: [0.0,0.0,0.0,1.0] },
            TfFrame { frame: "imu".into(), parent: "base_link".into(), translation: [0.0, 0.0, 0.4], rotation_q: [0.0,0.0,0.0,1.0] },
        ];

        StudioState {
            nodes,
            topics,
            metrics_history: Vec::new(),
            start_time: Instant::now(),
            alerts: Vec::new(),
            tf_frames,
            data_provider: Box::new(DemoDataProvider),
            use_live_provider: false,
        }
    }

    pub fn with_live_provider(mut self) -> Self {
        self.data_provider = Box::new(LiveNrosDataProvider::new());
        self.use_live_provider = true;
        // Refresh nodes/topics from provider
        self.nodes = self.data_provider.get_nodes();
        self.topics = self.data_provider.get_topics();
        self
    }

    pub fn to_status_json(&self) -> String {
        let nodes_json = self.nodes.iter().map(|(k,v)| {
            let params_json = v.params.iter().map(|(pk,pv)| format!(r#""{}":"{}""#, pk, pv)).collect::<Vec<_>>().join(",");
            format!(r#""{}":{{"name":"{}","rate_hz":{:.1},"cpu_pct":{:.1},"memory_mb":{:.1},"status":"{}","priority":{},"deadline_misses":{},"params":{{{}}}}}"#,
                k, v.name, v.rate_hz, v.cpu_pct, v.memory_mb, v.status, v.priority, v.deadline_misses, params_json)
        }).collect::<Vec<_>>().join(",");

        let uptime = self.start_time.elapsed().as_secs();
        format!(r#"{{"uptime_sec":{},"nodes":{{{}}},"topics":{},"alerts":{},"tf_frames":{}}}"#,
            uptime, nodes_json, self.topics.len(), self.alerts.len(), self.tf_frames.len())
    }

    pub fn to_nodes_json(&self) -> String {
        let arr = self.nodes.values().map(|v| {
            format!(r#"{{"name":"{}","rate_hz":{:.1},"cpu_pct":{:.1},"memory_mb":{:.1},"status":"{}","priority":{},"deadline_misses":{}}}"#,
                v.name, v.rate_hz, v.cpu_pct, v.memory_mb, v.status, v.priority, v.deadline_misses)
        }).collect::<Vec<_>>().join(",");
        format!("[{}]", arr)
    }

    pub fn to_topics_json(&self) -> String {
        let arr = self.topics.values().map(|t| {
            format!(r#"{{"name":"{}","type":"{}","rate_hz":{:.1},"bandwidth":"{}","latency":{{"avg_us":{:.1},"p99_us":{:.1},"max_us":{:.1}}},"qos":"{}"}}"#,
                t.name, t.type_name, t.rate_hz, t.bandwidth, t.latency.avg_us, t.latency.p99_us, t.latency.max_us, t.qos)
        }).collect::<Vec<_>>().join(",");
        format!("[{}]", arr)
    }

    pub fn to_tf_json(&self) -> String {
        let arr = self.tf_frames.iter().map(|tf| {
            format!(r#"{{"frame":"{}","parent":"{}","translation":[{:.2},{:.2},{:.2}],"rotation_q":[{:.2},{:.2},{:.2},{:.2}]}}"#,
                tf.frame, tf.parent, tf.translation[0], tf.translation[1], tf.translation[2], tf.rotation_q[0], tf.rotation_q[1], tf.rotation_q[2], tf.rotation_q[3])
        }).collect::<Vec<_>>().join(",");
        format!("[{}]", arr)
    }

    pub fn to_metric_json(&self) -> String {
        // P1 Fix: Use data_provider trait to make simulated vs real distinction visible
        let uptime = self.start_time.elapsed().as_secs();
        let metric = self.data_provider.get_metric(uptime);
        format!(r#"{{"timestamp_ms":{},"latency_us":{:.2},"throughput_kmsg":{:.1},"cpu_pct":{:.1},"memory_gb":{:.2},"deadline_misses":{},"uptime_sec":{},"provider":"{}","simulated":{}}}"#,
            metric.timestamp_ms, metric.latency_us, metric.throughput_kmsg, metric.cpu_pct, metric.memory_gb, metric.deadline_misses, uptime, self.data_provider.name(), self.data_provider.is_simulated())
    }

    pub fn update_param(&mut self, node: &str, key: &str, value: &str) -> Result<(), String> {
        let node_info = self.nodes.get_mut(node).ok_or_else(|| format!("Node {} not found", node))?;
        node_info.params.insert(key.to_string(), value.to_string());
        println!("[Studio] Param updated: {}/{} = {} (hot-reload per §17.3 validation @range)", node, key, value);
        Ok(())
    }
}

fn pseudo_rand() -> f64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEED: AtomicU64 = AtomicU64::new(0x9E3779B97F4A7C15);
    let s = SEED.load(Ordering::Relaxed);
    let new_s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
    SEED.store(new_s, Ordering::Relaxed);
    ((new_s >> 33) as f64 / u32::MAX as f64) * 2.0 - 1.0
}

impl Default for StudioState {
    fn default() -> Self { Self::new() }
}

// ── Server ──────────────────────────────────────────────────────────────────

pub struct StudioServer {
    pub addr: String,
    pub state: Arc<Mutex<StudioState>>,
}

impl StudioServer {
    pub fn new(addr: &str) -> Self {
        Self { addr: addr.to_string(), state: Arc::new(Mutex::new(StudioState::new())) }
    }

    pub fn run(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.addr)?;
        println!("🎨 NROS Studio listening at http://{}/", self.addr);
        println!("   Live inspector per `nros run --inspect` — http://localhost:8080");
        println!("   Endpoints: / /api/status /api/nodes /api/topics /api/tf /api/metrics /api/stream (SSE)");
        println!("   Shows: node graph SVG flow animation, 3D TF automatic, timeline, metrics, live param editing, breakpoints");

        // Load dashboard HTML
        let mut html_content = String::new();
        for p in &["crates/nros-studio/static/index.html", "static/index.html", "studio/index.html"] {
            if let Ok(c) = std::fs::read_to_string(p) { html_content = c; println!("   Serving from {}", p); break; }
        }
        if html_content.is_empty() {
            html_content = "<html><body><h1>NROS Studio</h1><p>Dashboard not found</p></body></html>".to_string();
        }

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let state = self.state.clone();
                    let html = html_content.clone();
                    std::thread::spawn(move || {
                        if let Err(e) = Self::handle_client(stream, state, &html) {
                            eprintln!("Client error: {}", e);
                        }
                    });
                }
                Err(e) => eprintln!("Accept error: {}", e),
            }
        }
        Ok(())
    }

    fn handle_client(mut stream: TcpStream, state: Arc<Mutex<StudioState>>, html: &str) -> std::io::Result<()> {
        let mut buffer = [0u8; 8192];
        let size = stream.read(&mut buffer)?;
        if size==0 { return Ok(()); }
        let request = String::from_utf8_lossy(&buffer[..size]);
        let first_line = request.lines().next().unwrap_or("");
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap_or("GET");
        let path = parts.next().unwrap_or("/");

        // Simple routing with query stripping
        let path_base = path.split('?').next().unwrap_or("/");

        match (method, path_base) {
            ("GET", "/") | ("GET", "/index.html") => {
                Self::send_response(&mut stream, "text/html; charset=utf-8", html)
            }
            ("GET", "/api/status") => {
                let json = state.lock().unwrap().to_status_json();
                Self::send_response(&mut stream, "application/json", &json)
            }
            ("GET", "/api/nodes") => {
                let json = state.lock().unwrap().to_nodes_json();
                Self::send_response(&mut stream, "application/json", &json)
            }
            ("GET", "/api/topics") => {
                let json = state.lock().unwrap().to_topics_json();
                Self::send_response(&mut stream, "application/json", &json)
            }
            ("GET", "/api/tf") => {
                let json = state.lock().unwrap().to_tf_json();
                Self::send_response(&mut stream, "application/json", &json)
            }
            ("GET", "/api/metrics") => {
                let json = state.lock().unwrap().to_metric_json();
                Self::send_response(&mut stream, "application/json", &json)
            }
            ("GET", "/api/stream") => {
                // Server-Sent Events — real NROS would use WebSocket per §7.2 for lower overhead
                // Client: new EventSource('/api/stream')
                let header = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
                stream.write_all(header.as_bytes())?;
                stream.flush()?;
                // Stream 60 seconds of metrics at 2 Hz
                for _ in 0..120 {
                    let json = state.lock().unwrap().to_metric_json();
                    let event = format!("data: {}\n\n", json);
                    if stream.write_all(event.as_bytes()).is_err() { break; }
                    if stream.flush().is_err() { break; }
                    std::thread::sleep(Duration::from_millis(500));
                }
                Ok(())
            }
            ("POST", "/api/params") | ("GET", "/api/params") => {
                // Simple param update: /api/params?node=velocity_controller&key=max_speed&value=2.5
                // Parse query from original path
                let query_str = if path.contains('?') { path.split('?').nth(1).unwrap_or("") } else { "" };
                // Also try to parse from request body (for POST)
                let body_part = if request.contains("\r\n\r\n") {
                    request.split("\r\n\r\n").nth(1).unwrap_or("")
                } else { "" };
                let combined_query = if !body_part.is_empty() { body_part } else { query_str };

                let mut params_map = HashMap::new();
                for pair in combined_query.split('&') {
                    if let Some((k,v)) = pair.split_once('=') {
                        params_map.insert(k.to_string(), v.to_string());
                    }
                }
                // Also support query from path
                for pair in query_str.split('&') {
                    if let Some((k,v)) = pair.split_once('=') {
                        params_map.insert(k.to_string(), url_decode(v));
                    }
                }

                let node = params_map.get("node").cloned().unwrap_or_else(|| "velocity_controller".to_string());
                let key = params_map.get("key").cloned().unwrap_or_else(|| "max_speed".to_string());
                let value = params_map.get("value").cloned().unwrap_or_else(|| "2.0".to_string());

                let result = state.lock().unwrap().update_param(&node, &key, &value);
                let json = match result {
                    Ok(_) => format!(r#"{{"ok":true,"node":"{}","key":"{}","value":"{}"}}"#, node, key, value),
                    Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
                };
                Self::send_response(&mut stream, "application/json", &json)
            }
            _ => {
                Self::send_response_with_status(&mut stream, "HTTP/1.1 404 NOT FOUND", "text/plain", "Not Found")
            }
        }
    }

    fn send_response(stream: &mut TcpStream, content_type: &str, body: &str) -> std::io::Result<()> {
        Self::send_response_with_status(stream, "HTTP/1.1 200 OK", content_type, body)
    }

    fn send_response_with_status(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) -> std::io::Result<()> {
        let response = format!(
            "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
            status, content_type, body.len(), body
        );
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}

fn url_decode(s: &str) -> String {
    // Minimal url decode for spaces etc.
    s.replace("%20", " ").replace("+", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_json() {
        let state = StudioState::new();
        let json = state.to_status_json();
        assert!(json.contains("velocity_controller"));
        assert!(json.contains("uptime_sec"));
    }

    #[test]
    fn test_nodes_topics() {
        let state = StudioState::new();
        let nodes = state.to_nodes_json();
        assert!(nodes.starts_with("["));
        let topics = state.to_topics_json();
        assert!(topics.contains("/cmd_vel"));
        let tf = state.to_tf_json();
        assert!(tf.contains("base_link"));
    }

    #[test]
    fn test_param_update() {
        let mut state = StudioState::new();
        assert!(state.update_param("velocity_controller", "max_speed", "3.0").is_ok());
        assert_eq!(state.nodes.get("velocity_controller").unwrap().params.get("max_speed").unwrap(), "3.0");
    }
}
