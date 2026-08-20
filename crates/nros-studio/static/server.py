#!/usr/bin/env python3
"""
NROS Studio API Server — Python equivalent of Rust StudioServer for preview
Serves: / -> index.html, /api/status, /api/nodes, /api/topics, /api/tf, /api/metrics, /api/stream (SSE), /api/params
Implements DESIGN.md §7.2 live monitoring per `nros run --inspect`
"""
import json, time, random, urllib.parse, os
from http.server import HTTPServer, SimpleHTTPRequestHandler
from socketserver import ThreadingMixIn
import threading

PORT = int(os.environ.get("PORT", "8080"))
BIND = os.environ.get("BIND", "0.0.0.0")

# Shared state (matches Rust StudioState)
nodes = {
    "velocity_controller": {"name":"velocity_controller","rate_hz":1000.0,"cpu_pct":12.5,"memory_mb":45.0,"status":"Running","priority":200,"deadline_misses":0,"params":{"max_speed":"2.0","wheel_base":"0.5"}},
    "camera_driver": {"name":"camera_driver","rate_hz":30.0,"cpu_pct":18.0,"memory_mb":120.0,"status":"Running","priority":150,"deadline_misses":0},
    "lidar_processor": {"name":"lidar_processor","rate_hz":10.0,"cpu_pct":8.0,"memory_mb":60.0,"status":"Running","priority":100,"deadline_misses":1},
    "path_planner": {"name":"path_planner","rate_hz":5.0,"cpu_pct":5.0,"memory_mb":80.0,"status":"Running","priority":100,"deadline_misses":0},
}
topics = {
    "/cmd_vel": {"name":"/cmd_vel","type":"geometry_msgs/Twist","rate_hz":10.0,"bandwidth":"1.2 KB/s","latency":{"avg_us":5.2,"p99_us":12.1,"max_us":18.7},"qos":"realtime max_latency_us:100"},
    "/odom": {"name":"/odom","type":"nav_msgs/Odometry","rate_hz":50.0,"bandwidth":"8.5 KB/s","latency":{"avg_us":6.1,"p99_us":11.0,"max_us":16.0},"qos":"reliable"},
    "/camera/image": {"name":"/camera/image","type":"sensor_msgs/Image","rate_hz":30.0,"bandwidth":"25.8 MB/s","latency":{"avg_us":8.5,"p99_us":15.0,"max_us":22.0},"qos":"best_effort drop=oldest"},
    "/scan": {"name":"/scan","type":"sensor_msgs/LaserScan","rate_hz":10.0,"bandwidth":"450 KB/s","latency":{"avg_us":7.0,"p99_us":13.0,"max_us":19.0},"qos":"best_effort"},
}
tf_frames = [
    {"frame":"odom","parent":"map","translation":[0,0,0],"rotation_q":[0,0,0,1]},
    {"frame":"base_link","parent":"odom","translation":[0.5,0,0.2],"rotation_q":[0,0,0,1]},
    {"frame":"camera","parent":"base_link","translation":[0.3,0,0.5],"rotation_q":[0,0,0,1]},
    {"frame":"lidar","parent":"base_link","translation":[0,0,0.6],"rotation_q":[0,0,0,1]},
    {"frame":"imu","parent":"base_link","translation":[0,0,0.4],"rotation_q":[0,0,0,1]},
]

start_time = time.time()

def make_metric():
    now_ms = int(time.time()*1000)
    latency = 1.5 + (now_ms % 100)*0.02 + random.uniform(-0.5,0.5)
    throughput = 450.0 + (now_ms % 200)*0.5 + random.uniform(-10,10)
    cpu = 35.0 + (now_ms % 100)*0.1 + random.uniform(-2,2)
    mem = 1.0 + (now_ms % 100)*0.01 + random.uniform(-0.2,0.2)
    misses = 1 if random.random()<0.04 else 0
    return {
        "timestamp_ms": now_ms,
        "latency_us": round(latency,2),
        "throughput_kmsg": round(throughput,1),
        "cpu_pct": round(cpu,1),
        "memory_gb": round(mem,2),
        "deadline_misses": misses,
        "uptime_sec": int(time.time()-start_time)
    }

class Handler(SimpleHTTPRequestHandler):
    def end_headers(self):
        # CORS for arena preview https://{port}-{sandbox}.e2b.app
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        super().end_headers()

    def do_OPTIONS(self):
        self.send_response(200)
        self.end_headers()

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path
        query = urllib.parse.parse_qs(parsed.query)

        if path in ("/", "/index.html"):
            # Serve index.html
            return super().do_GET()

        elif path == "/api/status":
            data = {
                "uptime_sec": int(time.time()-start_time),
                "nodes": nodes,
                "topics": len(topics),
                "tf_frames": len(tf_frames),
                "alerts": 0
            }
            self.send_json(data)

        elif path == "/api/nodes":
            self.send_json(list(nodes.values()))

        elif path == "/api/topics":
            self.send_json(list(topics.values()))

        elif path == "/api/tf":
            self.send_json(tf_frames)

        elif path == "/api/metrics":
            self.send_json(make_metric())

        elif path == "/api/stream":
            # SSE
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Cache-Control", "no-cache")
            self.send_header("Connection", "keep-alive")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            try:
                for _ in range(120):  # 60 seconds at 2 Hz
                    metric = make_metric()
                    line = f"data: {json.dumps(metric)}\n\n"
                    self.wfile.write(line.encode())
                    self.wfile.flush()
                    time.sleep(0.5)
            except BrokenPipeError:
                pass
            return

        elif path.startswith("/api/params"):
            # GET with query ?node=...&key=...&value=...
            node = query.get("node", ["velocity_controller"])[0]
            key = query.get("key", ["max_speed"])[0]
            value = query.get("value", ["2.0"])[0]
            if node in nodes:
                if "params" not in nodes[node]:
                    nodes[node]["params"] = {}
                nodes[node]["params"][key] = value
                print(f"[Studio] Param updated: {node}/{key} = {value} (hot-reload per §17.3)")
                self.send_json({"ok": True, "node": node, "key": key, "value": value})
            else:
                self.send_json({"ok": False, "error": f"Node {node} not found"}, status=404)

        else:
            self.send_error(404, f"Not Found: {path}")

    def do_POST(self):
        # Support POST /api/params with body node=...&key=...&value=...
        parsed = urllib.parse.urlparse(self.path)
        if parsed.path == "/api/params":
            length = int(self.headers.get('Content-Length', 0))
            body = self.rfile.read(length).decode() if length>0 else ""
            params = urllib.parse.parse_qs(body)
            # Merge query params
            query = urllib.parse.parse_qs(parsed.query)
            for k,v in query.items():
                if k not in params:
                    params[k]=v
            node = params.get("node", ["velocity_controller"])[0]
            key = params.get("key", ["max_speed"])[0]
            value = params.get("value", ["2.0"])[0]
            if node in nodes:
                nodes[node].setdefault("params", {})[key]=value
                print(f"[Studio] Param POST: {node}/{key}={value}")
                self.send_json({"ok": True, "node": node, "key": key, "value": value})
            else:
                self.send_json({"ok": False, "error": f"Node {node} not found"}, status=404)
        else:
            self.send_error(404)

    def send_json(self, data, status=200):
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

class ThreadedHTTPServer(ThreadingMixIn, HTTPServer):
    daemon_threads = True

if __name__ == "__main__":
    os.chdir(os.path.dirname(__file__))
    print(f"🎨 NROS Studio Python API Server listening at http://{BIND}:{PORT}/")
    print(f"   Endpoints: / /api/status /api/nodes /api/topics /api/tf /api/metrics /api/stream (SSE) /api/params")
    print(f"   Live inspector per `nros run --inspect` — shows node graph SVG flow, 3D TF, timeline, metrics")
    server = ThreadedHTTPServer((BIND, PORT), Handler)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nStopping...")
