# nros-cli — CLI Tools

Complete command-line interface per DESIGN.md §7.1, §20 Workflow, §21 Deployment, §25 Artifact #6.

## Commands — `nros` binary

### Project Management

```bash
nros init my_robot --template=mobile_base|manipulator|perception|humanoid|basic
# Creates:
#   src/nodes/main.rs (with #[nros::node] + #[subscribe]/#[publish]/#[param])
#   src/lib.rs, src/messages/*.mdl (compile to C structs per §5.1)
#   config/robot.yaml, config/sensors.yaml, config/simulation.yaml
#   launch/robot.launch.yaml (static graph validation per §5.2)
#   tests/integration_test.rs, docs/architecture.md
#   nros.toml project config per §20.1 with [nodes], [build], [simulation], [resources]

nros build --profile=realtime --target=aarch64-unknown-linux-gnu  # -O3, LTO, static
# Profiles:
#   debug: 2.3 MB, no opt, symbols
#   release: 1.1 MB, -O3 LTO stripped
#   realtime: 950 KB, -O3 LTO CPU native, static pools, CPU pinning, deadline monitoring (§4.1)
#   embedded: 480 KB, size -Os LTO minimal runtime no-std static_pools (§21.1) → ~500KB binary 2MB RAM

nros run --inspect  # web dashboard http://localhost:8080: live node graph animation, 3D viz TF, timeline, metrics, remote breakpoints per §20.3
nros test --filter=... --realtime  # unit tests with timing validation
```

### Communication

```bash
nros topic list
# /cmd_vel geometry_msgs/Twist 10Hz 1.2KB/s avg=5.2μs p99=12.1μs max=18.7μs (NROS target vs ROS2 287μs)

nros topic info /cmd_vel  # publishers, subscribers, rate, bandwidth, latency
nros topic echo /cmd_vel  # zero-copy numpy view in Python per §19.2
nros topic hz /cmd_vel    # avg/min/max/std dev rate
nros topic bw /camera/image  # bandwidth measurement

nros topic pub /cmd_vel "{linear: {x:1.0}, angular: {z:0.5}}" --zero-copy.allocate
nros node list / info / kill
nros service list / call
```

### Record & Replay + Analysis

```bash
nros record /camera/* /lidar --output=recording.nros --duration=10s  # efficient binary vs ROS2 .db3
nros replay recording.nros --speed=0.5 --loop --analyze-latency  # §20.3 timing analysis
nros analyze recording.nros --bandwidth / --latency / --timing / --graph
# bandwidth: per-topic MB/s
# latency: P50=5.8μs P99=12.1μs Max=18.7μs (§18.1)
# timing: WCET analysis `nros check --timing`
# graph: validate robot.graph.yaml — inputs/outputs matching, type compatibility, cycle detection (§5.2)
nros capture /camera/* /lidar/* --duration=10s --output=capture.nros  # network analyzer
```

### Profiling — Built-in Profiler §4.3

```bash
nros profile --duration=60s --output=profile.svg --focus=control_loop --events=cache-misses
# Top functions CPU time %, callback execution avg/p99/max, flamegraph generation
nros profile --memory --show-leaks
```

### Fleet Management — §21.2 Edge Orchestration + §21.3 Cloud + §8.2 OTA

```bash
nros fleet list  # from fleet.yaml: name, robots id/location/hardware/version, deployment rolling max_unavailable 1 health_check 30s, updates channel stable auto_update rollback_on_failure

nros cloud login --fleet=warehouse_fleet  # TLS 1.3, node certificates, topic ACLs per §9.1
nros cloud list  # robot_001 online zone_a 1.0.0 healthy, robot_002 warning:low_battery
nros cloud exec robot_001 "navigate_to --x=10 --y=5"  # remote control
nros fleet deploy --version=1.1.0 --canary=25%  # atomic updates rollback on failure, differential only changed components, staged rollout, validation hooks custom health checks
nros cloud status  # updating 70%, queued
nros fleet status  # health dashboard: CPU/Mem/Network, warnings battery 18%, events black box tamper-proof per §9.2
```

### Static Analysis + Migration — §22

```bash
nros check --timing  # WCET analysis warnings
nros check --graph   # validate communication graph

nros migrate analyze src/my_ros2_pkg  # report nodes, topic deps, custom msgs, effort
nros migrate convert-msgs src/msgs  # .msg → .mdl with @required @range @unit @versioned @hash
nros migrate convert src/pkg --output=nros_pkg  # ROS2 Publisher/Subscriber/Service/Timer → NROS async/.await patterns
nros migrate test --original=ros2_bag.db3 --converted=nros_recording
nros migrate compare baseline.bag migrated.nros
nros bridge ros2 --start + add-topic /camera/image sensor_msgs/Image  # compatibility layer transparent conversion
```

## Implementation Details

- `ProjectInitializer::init(name, template)`: validates project name cargo conventions, creates dirs per §20.1, generates nros.toml with dependencies per template (mobile_base → nros-navigation), sample node with macro `#[nros::node(memory_pool="10MB", max_messages=1000)]`, `#[callback(realtime=true, deadline_us=1000, priority=200, cpu_affinity=[2,3])]`, `#[publish(qos=RealTime{max_latency_us:100})]`, launch yaml with graph validation, robot.yaml with sensors/params
- `BuildSystem::build(profile, target)`: steps — MDL parsing → C structs zero-cost, type bindings bounds/unit, graph validation, compiling nodes priority+affinity, linking -O3 LTO, WCET analysis, summary size/features, embedded 500KB warning
- `TopicInspector`: list/info/echo/hz/bw with latency stats NROS target 6.2μs vs ROS2 287μs §18.1, publishers/subscribers addresses
- `Profiler::profile(duration, focus)`: tracing callback times, top functions CPU %, WCET per callback, flamegraph.svg path
- `FleetManager`: list from fleet.yaml, deploy rolling canary differential + validation hooks health check + atomic rollback, status telemetry per §21.3 position 1Hz battery alert <20% errors count, black box logging, exec remote command, login TLS
- `Recorder`: record binary .nros efficient, replay speed loop timing heatmap, analyze bandwidth/latency/timing/graph/compare
- `MigrationTools`: analyze report nodes/topics/msgs/effort, convert .msg→.mdl, patterns create_publisher → publish.await

## Tests

- `test_project_name_validation` — alphanumeric _ -
- `test_build_profile_parsing` — realtime/embedded
- `test_build_system` — size >0, profile matches

Run:
```bash
cargo run -p nros-cli --bin nros -- help
cargo run -p nros-cli --bin nros-cli-demo  # full interactive showcase
cargo test -p nros-cli -- --nocapture
```

## Relation

- Depends on `nros-core` for future `Publisher::publish_inplace` zero-copy demo
- Integrates with `nros-node` VelocityController lifecycle via `nros run`
- `nros-hal` sensor discovery displayed in `nros topic list` bandwidth 25.8 MB/s camera
- `nros-transport` service discovery listed in fleet deployment
- `nros-distributed` fleet coordination via `fleet.yaml` rolling update
