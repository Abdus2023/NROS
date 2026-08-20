//! NROS Hardware Abstraction Layer - Sensor Integration
//! Demonstrates: Unified sensor interface, zero-copy DMA, multi-sensor sync, hardware triggers
//! Implements DESIGN.md §6 Hardware Integration, §16 Deep Dive HAL, §25 Artifact #3

use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// Core Types — Compatible with nros-core
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Timestamp {
    pub sec: u64,
    pub nanosec: u32,
}

impl Timestamp {
    pub fn now() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        Timestamp {
            sec: now.as_secs(),
            nanosec: now.subsec_nanos(),
        }
    }

    pub fn to_millis(&self) -> u64 {
        self.sec * 1000 + (self.nanosec as u64 / 1_000_000)
    }

    pub fn from_millis(ms: u64) -> Self {
        Self {
            sec: ms / 1000,
            nanosec: ((ms % 1000) * 1_000_000) as u32,
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        Self::now().to_millis().wrapping_sub(self.to_millis())
    }

    pub fn difference_ms(&self, other: &Self) -> i64 {
        self.to_millis() as i64 - other.to_millis() as i64
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self { sec: 0, nanosec: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceClass {
    Camera,
    Lidar,
    Imu,
    Gps,
    Radar,
    Ultrasonic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_class: DeviceClass,
    pub name: String,
    pub serial_number: String,
}

impl std::fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}] {:04x}:{:04x} SN:{}", self.name, 
            format!("{:?}", self.device_class), self.vendor_id, self.product_id, self.serial_number)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SensorCapabilities {
    pub supports_hardware_trigger: bool,
    pub supports_timestamp: bool,
    pub supports_zero_copy: bool,
    pub supports_dma: bool,
    pub max_rate_hz: f64,
    pub min_rate_hz: f64,
}

impl Default for SensorCapabilities {
    fn default() -> Self {
        Self {
            supports_hardware_trigger: false,
            supports_timestamp: true,
            supports_zero_copy: false,
            supports_dma: false,
            max_rate_hz: 30.0,
            min_rate_hz: 1.0,
        }
    }
}

pub trait SensorData: Send + Sync {
    fn timestamp(&self) -> Timestamp;
    fn size_bytes(&self) -> usize;
    fn frame_id(&self) -> u64 {
        0
    }
}

pub trait Sensor: Send + Sync {
    fn device_info(&self) -> &DeviceInfo;
    fn capabilities(&self) -> SensorCapabilities;
    fn configure(&mut self, config: SensorConfig) -> Result<(), String>;
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn is_streaming(&self) -> bool;
}

// ============================================================================
// Sensor Configuration — Trigger modes per DESIGN.md §16.2
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerMode {
    FreeRun,
    External { pin: u8 },
    Software,
}

impl Default for TriggerMode {
    fn default() -> Self {
        Self::FreeRun
    }
}

#[derive(Debug, Clone)]
pub struct SensorConfig {
    pub rate_hz: f64,
    pub trigger_mode: TriggerMode,
    pub use_dma: bool,
    pub resolution: Option<(u32, u32)>,
    pub buffer_count: usize,
}

impl Default for SensorConfig {
    fn default() -> Self {
        SensorConfig {
            rate_hz: 30.0,
            trigger_mode: TriggerMode::FreeRun,
            use_dma: true,
            resolution: None,
            buffer_count: 4,
        }
    }
}

impl SensorConfig {
    pub fn with_rate(mut self, hz: f64) -> Self {
        self.rate_hz = hz;
        self
    }

    pub fn with_trigger(mut self, mode: TriggerMode) -> Self {
        self.trigger_mode = mode;
        self
    }

    pub fn with_dma(mut self, use_dma: bool) -> Self {
        self.use_dma = use_dma;
        self
    }
}

// ============================================================================
// Zero-Copy DMA Buffer — Simulates DESIGN.md §16.4 DMA pipeline
// P1 Fix per AUDIT.md: Separate SimulatedDmaBuffer vs RealDmaBuffer API
// P1 Fix per AUDIT Pass 14 HAL-001: Make zero-copy view via Arc, not clone()
// ============================================================================

/// Simulated DMA buffer — uses Arc<Vec<u8>> for zero-copy sharing, NOT real DMA
/// Status: IMPLEMENTED for zero-copy simulation (Arc refcount, no memcpy) per EVIDENCE_REGISTRY.md
/// Real would use memfd_create + mmap + DMA-BUF + GPU-accessible memory
#[derive(Debug, Clone)]
pub struct SimulatedDmaBuffer {
    pub id: usize,
    pub size: usize,
    pub data: std::sync::Arc<Vec<u8>>, // Arc for zero-copy sharing — clone only increments refcount, not bytes
    pub is_mapped: bool,
}

impl SimulatedDmaBuffer {
    pub fn new(id: usize, size: usize) -> Self {
        // Real NROS: memfd_create + mmap + dma_buf attach
        // Here: simulated with Arc<Vec<u8>> for zero-copy view demonstration
        Self {
            id,
            size,
            data: std::sync::Arc::new(vec![0u8; size]),
            is_mapped: true,
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.size
    }

    /// Fill buffer with pattern — uses Arc::make_mut to get &mut Vec<u8> without extra allocation if uniquely owned
    pub fn fill_pattern(&mut self, frame_count: u64) {
        let vec_mut = std::sync::Arc::make_mut(&mut self.data);
        for (i, byte) in vec_mut.iter_mut().enumerate() {
            *byte = ((frame_count + i as u64) % 256) as u8;
        }
    }
}

/// Real DMA buffer — SCAFFOLDED per AUDIT.md, would use memfd_create + mmap + DMA-BUF attachment
/// Currently still Vec<u8> for prototype, but API distinction makes executable fiction visible
/// Status: SCAFFOLDED — not yet HARDWARE-VALIDATED
#[derive(Debug)]
pub struct RealDmaBuffer {
    pub id: usize,
    pub size: usize,
    // In real: raw fd from memfd_create, mmap pointer, dma_buf fd
    // For now: simulated backing store, but marked as Real path
    backing: Vec<u8>,
    pub is_gpu_accessible: bool,
}

impl RealDmaBuffer {
    pub fn new_scaffolded(id: usize, size: usize) -> Self {
        // Real implementation would:
        // let fd = memfd_create("nros_dma", MFD_CLOEXEC)
        // ftruncate(fd, size)
        // let ptr = mmap(null, size, PROT_READ|PROT_WRITE, MAP_SHARED, fd, 0)
        // dma_buf = dma_buf_attach + prime fd
        // gpu_memory = gpu.allocate_dma_buffer that camera can DMA into
        Self {
            id,
            size,
            backing: vec![0u8; size],
            is_gpu_accessible: true,
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.backing.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_real_dma(&self) -> bool {
        false // Would be true when backed by actual memfd + DMA-BUF
    }
}

/// Type alias for backward compatibility — currently Simulated, will migrate to Real when hardware validated
/// Per AUDIT P1: separate types to make distinction visible
pub type DmaBuffer = SimulatedDmaBuffer;

/// Unified trait for DMA buffer abstraction — allows generic code over Simulated vs Real
pub trait DmaBufferTrait {
    fn id(&self) -> usize;
    fn size(&self) -> usize;
    fn is_simulated(&self) -> bool;
}

impl DmaBufferTrait for SimulatedDmaBuffer {
    fn id(&self) -> usize { self.id }
    fn size(&self) -> usize { self.size }
    fn is_simulated(&self) -> bool { true }
}

impl DmaBufferTrait for RealDmaBuffer {
    fn id(&self) -> usize { self.id }
    fn size(&self) -> usize { self.size }
    fn is_simulated(&self) -> bool { false }
}

// ── DMA Ownership State Machine — per AUDIT Pass 14 DMA-001, CACHE-001 ──────
// Rust ownership for CPU vs DMA engine, plus cache coherency

pub struct OwnedByCpu;
pub struct OwnedByDevice;

/// Type-state DmaBuffer with ownership — compiler prevents CPU modifies DMA-owned memory
/// State machine: CPUOwned --submit()--> DMAOwned --complete()--> CPUOwned
/// Also tracks cache coherency: CPU cache <-> memory <-> DMA with explicit sync
#[derive(Debug)]
pub struct DmaBufferState<State> {
    pub id: usize,
    pub size: usize,
    pub data: std::sync::Arc<Vec<u8>>,
    pub _marker: std::marker::PhantomData<State>,
}

impl DmaBufferState<OwnedByCpu> {
    pub fn new(id: usize, size: usize) -> Self {
        Self { id, size, data: std::sync::Arc::new(vec![0u8; size]), _marker: std::marker::PhantomData }
    }

    /// Submit to device — transfers ownership CPU -> Device, requires cache clean
    pub fn submit(self) -> DmaBufferState<OwnedByDevice> {
        // Real would: cache clean (clean cache to memory), memory barrier, DMA fence
        // println!("[DMA] CPU -> Device ownership transfer, cache clean, id {}", self.id);
        DmaBufferState { id: self.id, size: self.size, data: self.data, _marker: std::marker::PhantomData }
    }

    pub fn as_slice(&self) -> &[u8] { &self.data }
    pub fn as_mut_slice(&mut self) -> &mut Vec<u8> {
        std::sync::Arc::make_mut(&mut self.data)
    }
}

impl DmaBufferState<OwnedByDevice> {
    /// Complete DMA — device finished, transfer ownership Device -> CPU, requires cache invalidate
    pub fn complete(self) -> DmaBufferState<OwnedByCpu> {
        // Real would: DMA fence, cache invalidate (invalidate cache to see device writes), memory barrier
        // println!("[DMA] Device -> CPU ownership transfer, cache invalidate, id {}", self.id);
        DmaBufferState { id: self.id, size: self.size, data: self.data, _marker: std::marker::PhantomData }
    }

    // No as_mut_slice here — prevents CPU modifies DMA-owned memory without unsafe escape hatch
    pub fn as_slice(&self) -> &[u8] { &self.data }
}

/// Cache coherency contract per AUDIT Pass 14 CACHE-001
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheState {
    Clean,
    Dirty,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheOperation {
    Clean,      // Clean cache to memory
    Invalidate, // Invalidate cache to see device writes
    CleanInvalidate,
    MemoryBarrier,
    DmaFence,
}

// ============================================================================
// Camera Implementation — V4L2 + DMA zero-copy per §16.1
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    RGB8,
    RGBA8,
    BGR8,
    MONO8,
    MONO16,
}

impl ImageFormat {
    pub fn bpp(&self) -> usize {
        match self {
            Self::RGB8 | Self::BGR8 => 3,
            Self::RGBA8 => 4,
            Self::MONO8 => 1,
            Self::MONO16 => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Image {
    pub timestamp: Timestamp,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub data: std::sync::Arc<Vec<u8>>, // Arc for zero-copy sharing — clone only increments refcount, not bytes (fixes HAL-001)
    pub frame_id: u64,
    pub dma_buffer_id: Option<usize>,
}

impl SensorData for Image {
    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    fn size_bytes(&self) -> usize {
        self.data.len()
    }

    fn frame_id(&self) -> u64 {
        self.frame_id
    }
}

pub struct CameraDriver {
    device_info: DeviceInfo,
    config: SensorConfig,
    is_streaming: bool,
    frame_count: u64,

    // Simulated DMA buffers — real: V4L2 reqbufs + mmap + DMABUF
    dma_buffers: Vec<DmaBuffer>,
    current_buffer: usize,
}

impl CameraDriver {
    pub fn new(name: &str) -> Self {
        CameraDriver {
            device_info: DeviceInfo {
                vendor_id: 0x1234,
                product_id: 0x5678,
                device_class: DeviceClass::Camera,
                name: name.to_string(),
                serial_number: "CAM001".to_string(),
            },
            config: SensorConfig::default(),
            is_streaming: false,
            frame_count: 0,
            dma_buffers: Vec::new(),
            current_buffer: 0,
        }
    }

    pub fn with_device_info(mut self, info: DeviceInfo) -> Self {
        self.device_info = info;
        self
    }

    /// Zero-copy frame access — returns DmaBuffer index + Image referencing it
    /// In real NROS: camera DMA's directly to GPU memory, no CPU copy
    /// P1 Fix HAL-001: Now uses Arc for zero-copy view, not clone() of bytes
    pub fn capture_frame(&mut self) -> Result<Image, String> {
        if !self.is_streaming {
            return Err("Camera not streaming".to_string());
        }

        let width = self.config.resolution.unwrap_or((640, 480)).0;
        let height = self.config.resolution.unwrap_or((640, 480)).1;
        let format = ImageFormat::RGB8;
        let expected_size = (width * height * format.bpp() as u32) as usize;

        // Simulate frame capture with DMA — real: VIDIOC_DQBUF + memmap pointer
        let (data, dma_id) = if self.config.use_dma && !self.dma_buffers.is_empty() {
            // Zero-copy path: fill DMA buffer and return Arc clone (refcount only, no memcpy)
            let buf = &mut self.dma_buffers[self.current_buffer];
            buf.fill_pattern(self.frame_count);
            let dma_id = buf.id;
            let data_arc = std::sync::Arc::clone(&buf.data); // Zero-copy: Arc clone, not byte clone (fixes HAL-001)
            self.current_buffer = (self.current_buffer + 1) % self.dma_buffers.len();
            (data_arc, Some(dma_id))
        } else {
            // Copy path — allocate new buffer (still Arc for uniform API, but new allocation)
            (std::sync::Arc::new(vec![0u8; expected_size]), None)
        };

        self.frame_count += 1;

        Ok(Image {
            timestamp: Timestamp::now(),
            width,
            height,
            format,
            data,
            frame_id: self.frame_count,
            dma_buffer_id: dma_id,
        })
    }

    /// DMA registration — real: camera.register_dma_targets(&gpu_buffers) per §16.4
    pub fn register_dma_targets(&mut self, count: usize, size: usize) {
        self.dma_buffers = (0..count).map(|i| DmaBuffer::new(i, size)).collect();
    }
}

impl Sensor for CameraDriver {
    fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    fn capabilities(&self) -> SensorCapabilities {
        SensorCapabilities {
            supports_hardware_trigger: true,
            supports_timestamp: true,
            supports_zero_copy: true,
            supports_dma: true,
            max_rate_hz: 120.0,
            min_rate_hz: 1.0,
        }
    }

    fn configure(&mut self, config: SensorConfig) -> Result<(), String> {
        if config.rate_hz < self.capabilities().min_rate_hz
            || config.rate_hz > self.capabilities().max_rate_hz
        {
            return Err(format!(
                "Rate {:.1} out of range [{:.1}, {:.1}]",
                config.rate_hz,
                self.capabilities().min_rate_hz,
                self.capabilities().max_rate_hz
            ));
        }

        self.config = config.clone();

        // Allocate DMA buffers if needed — real: V4L2 REQBUFS + mmap
        if config.use_dma {
            let (w, h) = config.resolution.unwrap_or((640, 480));
            let buffer_size = (w * h * 3) as usize; // RGB8
            self.dma_buffers = (0..config.buffer_count)
                .map(|i| DmaBuffer::new(i, buffer_size))
                .collect();
            println!("[Camera] Allocated {} DMA buffers {} bytes each (zero-copy)", config.buffer_count, buffer_size);
        }

        Ok(())
    }

    fn start(&mut self) -> Result<(), String> {
        println!(
            "[Camera] Starting stream at {:.1} Hz, DMA: {}, Trigger: {:?}",
            self.config.rate_hz, self.config.use_dma, self.config.trigger_mode
        );
        self.is_streaming = true;
        self.frame_count = 0;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        println!("[Camera] Stopping stream — {} frames captured", self.frame_count);
        self.is_streaming = false;
        Ok(())
    }

    fn is_streaming(&self) -> bool {
        self.is_streaming
    }
}

// ============================================================================
// LiDAR Implementation
// ============================================================================

#[derive(Debug, Clone)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub intensity: f32,
}

#[derive(Debug, Clone)]
pub struct PointCloud {
    pub timestamp: Timestamp,
    pub points: Vec<Point3D>,
    pub scan_id: u64,
}

impl SensorData for PointCloud {
    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    fn size_bytes(&self) -> usize {
        self.points.len() * std::mem::size_of::<Point3D>()
    }

    fn frame_id(&self) -> u64 {
        self.scan_id
    }
}

pub struct LidarDriver {
    device_info: DeviceInfo,
    config: SensorConfig,
    is_scanning: bool,
    scan_count: u64,
}

impl LidarDriver {
    pub fn new(name: &str) -> Self {
        LidarDriver {
            device_info: DeviceInfo {
                vendor_id: 0xABCD,
                product_id: 0xEF01,
                device_class: DeviceClass::Lidar,
                name: name.to_string(),
                serial_number: "LIDAR001".to_string(),
            },
            config: SensorConfig::default(),
            is_scanning: false,
            scan_count: 0,
        }
    }

    pub fn capture_scan(&mut self) -> Result<PointCloud, String> {
        if !self.is_scanning {
            return Err("LiDAR not scanning".to_string());
        }

        // Simulate point cloud capture — real: Velodyne/Hesai UDP packets + DMA
        let num_points = 1000 + (self.scan_count % 200) as usize; // Variable for realistic size
        let points: Vec<Point3D> = (0..num_points)
            .map(|i| {
                let angle = (i as f32) * 2.0 * std::f32::consts::PI / (num_points as f32);
                let range = 5.0 + (i as f32 * 0.01).sin();
                Point3D {
                    x: range * angle.cos(),
                    y: range * angle.sin(),
                    z: ((self.scan_count as f32 * 0.1).sin()) * 0.5,
                    intensity: 1.0,
                }
            })
            .collect();

        self.scan_count += 1;

        Ok(PointCloud {
            timestamp: Timestamp::now(),
            points,
            scan_id: self.scan_count,
        })
    }
}

impl Sensor for LidarDriver {
    fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    fn capabilities(&self) -> SensorCapabilities {
        SensorCapabilities {
            supports_hardware_trigger: true,
            supports_timestamp: true,
            supports_zero_copy: false, // Point clouds typically need processing
            supports_dma: false,
            max_rate_hz: 20.0,
            min_rate_hz: 1.0,
        }
    }

    fn configure(&mut self, config: SensorConfig) -> Result<(), String> {
        self.config = config;
        Ok(())
    }

    fn start(&mut self) -> Result<(), String> {
        println!("[LiDAR] Starting scan at {:.1} Hz, Trigger: {:?}", self.config.rate_hz, self.config.trigger_mode);
        self.is_scanning = true;
        self.scan_count = 0;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        println!("[LiDAR] Stopping scan — {} scans", self.scan_count);
        self.is_scanning = false;
        Ok(())
    }

    fn is_streaming(&self) -> bool {
        self.is_scanning
    }
}

// ============================================================================
// IMU Implementation — High frequency 100-1000 Hz
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImuData {
    pub timestamp: Timestamp,
    pub linear_acceleration: Vector3,
    pub angular_velocity: Vector3,
    pub orientation: Vector3,
    pub sequence: u64,
}

impl SensorData for ImuData {
    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    fn size_bytes(&self) -> usize {
        std::mem::size_of::<ImuData>()
    }

    fn frame_id(&self) -> u64 {
        self.sequence
    }
}

pub struct ImuDriver {
    device_info: DeviceInfo,
    config: SensorConfig,
    is_streaming: bool,
    sequence: u64,
}

impl ImuDriver {
    pub fn new(name: &str) -> Self {
        ImuDriver {
            device_info: DeviceInfo {
                vendor_id: 0x9999,
                product_id: 0x0001,
                device_class: DeviceClass::Imu,
                name: name.to_string(),
                serial_number: "IMU001".to_string(),
            },
            config: SensorConfig { rate_hz: 200.0, ..Default::default() },
            is_streaming: false,
            sequence: 0,
        }
    }

    pub fn read_data(&mut self) -> Result<ImuData, String> {
        if !self.is_streaming {
            return Err("IMU not streaming".to_string());
        }

        // Simulate IMU reading — real: SPI/I2C DMA + interrupt
        self.sequence += 1;
        Ok(ImuData {
            timestamp: Timestamp::now(),
            linear_acceleration: Vector3 { x: 0.02 * (self.sequence as f64 * 0.01).sin(), y: 0.0, z: 9.81 },
            angular_velocity: Vector3 { x: 0.0, y: 0.0, z: 0.01 * (self.sequence as f64 * 0.005).cos() },
            orientation: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            sequence: self.sequence,
        })
    }
}

impl Sensor for ImuDriver {
    fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    fn capabilities(&self) -> SensorCapabilities {
        SensorCapabilities {
            supports_hardware_trigger: false,
            supports_timestamp: true,
            supports_zero_copy: true,
            supports_dma: true,
            max_rate_hz: 1000.0,
            min_rate_hz: 1.0,
        }
    }

    fn configure(&mut self, config: SensorConfig) -> Result<(), String> {
        self.config = config;
        Ok(())
    }

    fn start(&mut self) -> Result<(), String> {
        println!("[IMU] Starting stream at {:.1} Hz", self.config.rate_hz);
        self.is_streaming = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        println!("[IMU] Stopping stream — {} samples", self.sequence);
        self.is_streaming = false;
        Ok(())
    }

    fn is_streaming(&self) -> bool {
        self.is_streaming
    }
}

// ============================================================================
// Multi-Sensor Synchronization — 10ms tolerance per DESIGN.md §16.2
// ============================================================================

pub struct SynchronizedData {
    pub timestamp: Timestamp,
    pub camera: Option<Image>,
    pub lidar: Option<PointCloud>,
    pub imu: Option<ImuData>,
    pub sync_quality_ms: u64,
}

pub struct SensorSynchronizer {
    pub tolerance_ms: u64,
    pub camera_buffer: Vec<Image>,
    pub lidar_buffer: Vec<PointCloud>,
    pub imu_buffer: Vec<ImuData>,
    pub max_buffer_size: usize,
    pub sync_attempts: u64,
    pub sync_success: u64,
}

impl SensorSynchronizer {
    pub fn new(tolerance_ms: u64) -> Self {
        SensorSynchronizer {
            tolerance_ms,
            camera_buffer: Vec::new(),
            lidar_buffer: Vec::new(),
            imu_buffer: Vec::new(),
            max_buffer_size: 10,
            sync_attempts: 0,
            sync_success: 0,
        }
    }

    pub fn add_camera_frame(&mut self, frame: Image) {
        self.camera_buffer.push(frame);
        if self.camera_buffer.len() > self.max_buffer_size {
            self.camera_buffer.remove(0);
        }
    }

    pub fn add_lidar_scan(&mut self, scan: PointCloud) {
        self.lidar_buffer.push(scan);
        if self.lidar_buffer.len() > self.max_buffer_size {
            self.lidar_buffer.remove(0);
        }
    }

    pub fn add_imu_data(&mut self, data: ImuData) {
        self.imu_buffer.push(data);
        if self.imu_buffer.len() > self.max_buffer_size {
            self.imu_buffer.remove(0);
        }
    }

    /// Try to find synchronized set within tolerance — matches DESIGN.md SyncGroup
    pub fn try_synchronize(&mut self) -> Option<SynchronizedData> {
        self.sync_attempts += 1;

        if self.camera_buffer.is_empty() || self.lidar_buffer.is_empty() || self.imu_buffer.is_empty() {
            return None;
        }

        // Use earliest camera frame as anchor — real: hardware trigger pulse timestamp
        let camera = &self.camera_buffer[0];
        let camera_time = camera.timestamp.to_millis();

        // Find closest LiDAR and IMU to camera time
        let lidar_idx = self.find_closest_timestamp(&self.lidar_buffer, camera_time);
        let lidar = &self.lidar_buffer[lidar_idx];
        let lidar_time = lidar.timestamp.to_millis();

        let imu_idx = self.find_closest_timestamp(&self.imu_buffer, camera_time);
        let imu = &self.imu_buffer[imu_idx];
        let imu_time = imu.timestamp.to_millis();

        // Check if within tolerance — 10ms per spec
        let max_t = camera_time.max(lidar_time).max(imu_time);
        let min_t = camera_time.min(lidar_time).min(imu_time);
        let diff = max_t - min_t;

        if diff <= self.tolerance_ms {
            self.sync_success += 1;
            let synced = SynchronizedData {
                timestamp: camera.timestamp,
                camera: Some(self.camera_buffer.remove(0)),
                lidar: Some(self.lidar_buffer.remove(lidar_idx)),
                imu: Some(self.imu_buffer.remove(imu_idx)),
                sync_quality_ms: diff,
            };
            Some(synced)
        } else {
            // Drop oldest if too much drift to prevent buffer bloat
            if self.camera_buffer.len() >= self.max_buffer_size {
                self.camera_buffer.remove(0);
            }
            None
        }
    }

    fn find_closest_timestamp<T: SensorData>(&self, buffer: &[T], target_ms: u64) -> usize {
        buffer
            .iter()
            .enumerate()
            .min_by_key(|(_, data)| {
                let data_ms = data.timestamp().to_millis();
                ((data_ms as i64) - (target_ms as i64)).abs()
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    pub fn success_rate(&self) -> f64 {
        if self.sync_attempts == 0 {
            0.0
        } else {
            (self.sync_success as f64 / self.sync_attempts as f64) * 100.0
        }
    }

    pub fn buffered_counts(&self) -> (usize, usize, usize) {
        (self.camera_buffer.len(), self.lidar_buffer.len(), self.imu_buffer.len())
    }
}

// ============================================================================
// Sensor Manager — Discovery, hot-plug, coordinated operation
// ============================================================================

pub struct SensorManager {
    pub sensors: HashMap<String, Box<dyn Sensor>>,
}

impl SensorManager {
    pub fn new() -> Self {
        SensorManager {
            sensors: HashMap::new(),
        }
    }

    pub fn register_sensor(&mut self, name: String, sensor: Box<dyn Sensor>) {
        println!("[SensorManager] Registering {}: {}", name, sensor.device_info());
        self.sensors.insert(name, sensor);
    }

    pub fn discover(&mut self) {
        // Real NROS: HAL discovers via USB, SPI, I2C, PCIe enumeration
        // For demo, we assume registration is discovery
        println!("[SensorManager] Discovering sensors via HAL...");
        for (name, sensor) in self.sensors.iter() {
            println!("  Found: {} -> {}", name, sensor.device_info());
        }
    }

    pub fn configure_all(&mut self, config: SensorConfig) -> Result<(), String> {
        for (name, sensor) in self.sensors.iter_mut() {
            println!("[SensorManager] Configuring {}", name);
            sensor.configure(config.clone())?;
        }
        Ok(())
    }

    pub fn start_all(&mut self) -> Result<(), String> {
        for (name, sensor) in self.sensors.iter_mut() {
            println!("[SensorManager] Starting {}", name);
            sensor.start()?;
        }
        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<(), String> {
        for (name, sensor) in self.sensors.iter_mut() {
            println!("[SensorManager] Stopping {}", name);
            sensor.stop()?;
        }
        Ok(())
    }

    pub fn list_capabilities(&self) {
        println!("\n=== Sensor Capabilities ===");
        for (name, sensor) in self.sensors.iter() {
            let caps = sensor.capabilities();
            println!(
                "  {}: {:.0}-{:.0} Hz, HW trigger: {}, Zero-copy: {}, DMA: {}",
                name, caps.min_rate_hz, caps.max_rate_hz, caps.supports_hardware_trigger, caps.supports_zero_copy, caps.supports_dma
            );
        }
    }

    pub fn count(&self) -> usize {
        self.sensors.len()
    }
}

impl Default for SensorManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_dma() {
        let mut cam = CameraDriver::new("test");
        cam.configure(SensorConfig { use_dma: true, buffer_count: 4, ..Default::default() }).unwrap();
        cam.start().unwrap();
        let frame = cam.capture_frame().unwrap();
        assert_eq!(frame.width, 640);
        assert!(frame.dma_buffer_id.is_some());
        assert!(frame.size_bytes() > 0);
    }

    #[test]
    fn test_sensor_manager() {
        let mut mgr = SensorManager::new();
        mgr.register_sensor("cam".into(), Box::new(CameraDriver::new("Cam")));
        mgr.register_sensor("lidar".into(), Box::new(LidarDriver::new("Lidar")));
        assert_eq!(mgr.count(), 2);
        mgr.configure_all(SensorConfig::default()).unwrap();
        mgr.start_all().unwrap();
        mgr.stop_all().unwrap();
    }

    #[test]
    fn test_synchronizer_tolerance() {
        let mut sync = SensorSynchronizer::new(10);
        assert!(sync.try_synchronize().is_none()); // empty

        // Create frames with close timestamps
        let ts = Timestamp::now();
        let img = Image {
            timestamp: ts,
            width: 640,
            height: 480,
            format: ImageFormat::RGB8,
            data: std::sync::Arc::new(vec![0; 10]),
            frame_id: 1,
            dma_buffer_id: None,
        };
        let cloud = PointCloud {
            timestamp: ts,
            points: vec![Point3D { x: 0.0, y: 0.0, z: 0.0, intensity: 1.0 }],
            scan_id: 1,
        };
        let imu = ImuData {
            timestamp: ts,
            linear_acceleration: Vector3::new(0.0, 0.0, 9.81),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            orientation: Vector3::new(0.0, 0.0, 0.0),
            sequence: 1,
        };

        sync.add_camera_frame(img);
        sync.add_lidar_scan(cloud);
        sync.add_imu_data(imu);

        let synced = sync.try_synchronize();
        assert!(synced.is_some());
        assert_eq!(sync.sync_success, 1);
    }

    #[test]
    fn test_capabilities() {
        let cam = CameraDriver::new("test");
        assert!(cam.capabilities().supports_zero_copy);
        assert!(cam.capabilities().supports_hardware_trigger);

        let lidar = LidarDriver::new("test");
        assert!(lidar.capabilities().max_rate_hz <= 30.0);

        let imu = ImuDriver::new("test");
        assert!(imu.capabilities().max_rate_hz >= 500.0);
    }
}
