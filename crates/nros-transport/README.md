# nros-transport — Network Transport Layer

UDP/TCP with efficient binary serialization, compression, multicast, mDNS discovery per DESIGN.md §14.3, §14.4, §25 Artifact #4.

## Features

### Serialization — FlatBuffers-style

- `Serializable` trait: `serialize`, `deserialize`, `serialized_size`
- **Twist**: 48 bytes (2x Vector3 24 bytes) vs ROS2 CDR ~200+ bytes
- `MessageHeader` 36 bytes: magic `0x4E524F53` ("NROS"), version, message_type (MDL hash), payload_size, timestamp (for 5ms sync tolerance), sequence, checksum CRC32
- Future: FlatBuffers zero-copy deserialization, memory-mapped buffers, direct serialization to UDP buffer `udp_buffer.get()` to avoid allocation

```rust
let header = MessageHeader::new(message_type, payload.len() as u32, seq).with_checksum(&payload);
let mut packet = header.to_bytes(); // 36 bytes
packet.extend_from_slice(&payload);
```

### Compression — LZ4 threshold 1KB

- `CompressionEngine::new(1024)` — compress if >1KB per §14.3
- Placeholder with flag byte (0=uncompressed, 1=compressed), real NROS uses LZ4/Zstd giving 30-60% bandwidth reduction for images/pointclouds
- Stats track `compressed_messages` ratio

### Large Message Optimization — FD Passing per §14.2

- Threshold: if `data.len() > ZERO_COPY_THRESHOLD` (1KB), create `memfd_create` anonymous shmem, `ftruncate`, `mmap(MAP_SHARED)`, copy once, `send_fd()` 4-byte FD over socket
- Receiver `recv_fd()` + `mmap()` — zero-copy for images/pointclouds
- Demo shows `LargePayload` 5KB serialization path

### UDP Transport — BestEffort QoS

- Non-blocking `UdpSocket`, `HashMap<String, SocketAddr>` peers per topic
- `publish`: serialize, optional compress, header+checksum, `send_to` — measures send time via `Instant`
- `receive`: `recv_from` loop, parse header, validate magic/version, decompress, deserialize — returns `Option<(T, MessageHeader)>`
- `multicast_group(group, ttl)` — `IP_MULTICAST_TTL`, `IP_ADD_MEMBERSHIP` for efficient one-to-many without DDS overhead (§14.3)
- TTL 5 limits to local network
- Latency target: <100μs localhost

```rust
let pub = Publisher::new("/global/status").multicast_group("224.0.0.1:5000")?.ttl(5)?.build()?;
```

### TCP Transport — Reliable QoS

- Server: `TcpListener::bind`, non-blocking accept loop (future)
- Client: `TcpStream::connect`, `set_nodelay(true)` for low latency, non-blocking
- `publish` reliable with retries, `max_retries`, `timeout_ms` per QoS profile
- Latency target: <200μs localhost

### TransportStats — <1μs overhead

- Atomic counters: messages_sent/received, bytes_sent/received, compressed_messages, total/max send time
- `avg_send_time_us()`, `print()` MB conversion, compressed ratio

### Service Discovery — mDNS-like

- `ServiceDiscovery::new(port)` binds UDP broadcast, `set_broadcast(true)`, non-blocking
- `announce(ServiceInfo { topic, transport: "udp"|"tcp"|"udp-multicast", address, message_type })` — broadcasts `NROS_ANNOUNCE|...`
- `discover(topic)` local cache, `list_services()`
- Mirrors DESIGN.md `nros::discovery::MDnsDiscovery::announce_publisher("/camera/image", PublisherInfo{ transport: "udp-multicast", address: "224.0.0.1:5001", message_type })`

## QoS Profiles (§14.4)

```rust
enum QosProfile {
    RealTime { max_latency_us: u32 }, // ultra-low latency, lossy OK for control loops
    Reliable { max_retries: u32, timeout_ms: u32 }, // commands
    BestEffort { queue_size: usize, drop_policy: DropOldest/Newest }, // sensor data
    Durable { storage_path: PathBuf, max_size_mb: u32 }, // logs with disk backing
}
```

## Tests

- `test_header_validation` — magic check
- `test_header_roundtrip` — to_bytes/from_bytes size 36
- `test_twist_serialization` — 48 bytes, roundtrip values
- `test_compression_flag` — threshold logic + flag 0/1
- `test_udp_loopback` — publish 127.0.0.1:0 → receive, validates stats
- `test_service_discovery` — announce + discover

Run:
```bash
cargo test -p nros-transport -- --nocapture
cargo run -p nros-transport --bin nros-transport-demo
```

## Performance (§18)

- UDP localhost <100μs, TCP <200μs (target)
- Serialization 48 bytes Twist, automatic compression 30-60% saving
- Multicast efficient one-to-many without per-subscriber overhead
- Zero-copy deserialization via FlatBuffers in real NROS
