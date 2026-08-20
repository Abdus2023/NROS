# nros-hal — Hardware Abstraction Layer

Unified sensor interface, zero-copy DMA, multi-sensor synchronization per DESIGN.md §6, §16, §25 Artifact #3.

## Features

### Unified Sensor Trait

```rust
trait Sensor: Send + Sync {
    fn device_info(&self) -> &DeviceInfo;
    fn capabilities(&self) -> SensorCapabilities;
    fn configure(&mut self, config: SensorConfig) -> Result<()>;
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn is_streaming(&self) -> bool;
}
```

- **DeviceInfo**: vendor_id, product_id, class, serial — for HAL discovery
- **Capabilities**: HW trigger, timestamp, zero-copy, DMA, min/max rate
- Auto-validation of rate against capabilities

### Camera Driver — V4L2 + DMA zero-copy

- `DmaBuffer`: simulates `memfd_create + mmap + dma_buf attach` — real NROS would mmap V4L2 buffers and share with GPU via DMABUF
- `register_dma_targets()`: mirrors DESIGN.md §16.4 — allocate GPU memory camera can DMA into
- Zero-copy path: `capture_frame()` returns reference to existing DMA buffer, no memcpy; copy path fallback
- Resolution + buffer_count configurable

```rust
let mut cam = CameraDriver::new("front");
cam.configure(SensorConfig { rate_hz: 30.0, use_dma: true, resolution: Some((640,480)), buffer_count: 4, ..Default::default() })?;
cam.start()?;
let frame = cam.capture_frame()?; // Image { data: &[u8] zero-copy view, dma_buffer_id: Some(id) }
```

Real pipeline: camera → DMA → GPU memory → CUDA kernel launch with no CPU involvement.

### LiDAR & IMU

- **LidarDriver**: 10-20Hz, point cloud generation, external trigger support (GPIO pin)
- **ImuDriver**: 200-1000Hz, SPI/I2C DMA, sequence tracking, high-rate path

### Multi-Sensor Synchronization — 10ms tolerance

Implements `SyncGroup` from §16.2:

```rust
struct SyncGroup { master_trigger: GpioPin, sensors: Vec<Box<dyn Sensor>> }
async fn synchronized_capture() -> SyncedData { trigger.pulse(); join_all(sensors.read()) }
```

- `SensorSynchronizer::new(10)` — 10ms tolerance per spec
- Buffers 10 frames per sensor, finds closest timestamp to camera anchor
- Drops oldest on drift to prevent bloat
- Tracks success_rate, sync_quality_ms

### SensorManager — HAL Discovery

- `register_sensor(name, Box<dyn Sensor>)` — plugin system
- `discover()` — would enumerate USB, SPI, I2C, PCIe in real NROS
- `configure_all/start_all/stop_all` — coordinated fleet operation
- `list_capabilities()` — prints Hz ranges, HW trigger, zero-copy, DMA flags

## Zero-Copy DMA Pipeline Demo

```
Camera --DMA--> GPU Buffer (memfd, mmap, DMABUF)
   |                |
   +-- no memcpy ---+--> CUDA kernel launch
```

In real implementation:
```rust
let gpu_buffers: Vec<GpuBuffer> = (0..NUM).map(|_| gpu.allocate_dma_buffer(size)).collect();
camera.register_dma_targets(&gpu_buffers)?;
camera.start_dma_stream(|idx| gpu.launch_async(process_kernel, gpu_buffers[idx]));
```

## Performance Characteristics (per DESIGN.md §16)

- Camera @ 30Hz RGB8 640x480: ~0.9MB/frame, 4 DMA buffers ~3.6MB pinned
- LiDAR @ 10Hz 1000 pts: ~16KB/scan, 160KB/s
- IMU @ 200Hz: 72 bytes/sample, 14.4KB/s
- Sync overhead: <100μs to find closest timestamps in buffers of 10

## Tests

- `test_camera_dma` — DMA buffer allocation + zero-copy id
- `test_sensor_manager` — register/configure/start/stop lifecycle
- `test_synchronizer_tolerance` — same timestamp → sync success
- `test_capabilities` — camera zero-copy true, IMU high rate

Run:
```bash
cargo test -p nros-hal -- --nocapture
cargo run -p nros-hal --bin nros-hal-demo
```

## Relation to Other Crates

- Depends on `nros-core` for future `Publisher<Image>` zero-copy sharing
- Could publish synchronized sets via `nros_node::VelocityController` odometry fusion
- HAL plugins loaded from `~/.nros/plugins/*.so` per §19.1
