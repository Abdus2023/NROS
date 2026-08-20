// NROS Hardware Abstraction Layer - Sensor Integration
// Demonstrates: Unified sensor interface, zero-copy DMA, multi-sensor sync

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;

// ============================================================================
// Core Sensor Traits
// ============================================================================

#[derive(Debug, Clone, Copy)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    Camera,
    Lidar,
    Imu,
    Gps,
    Radar,
    Ultrasonic,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_class: DeviceClass,
    pub name: String,
    pub serial_number: String,
}

#[derive(Debug, Clone)]
pub struct SensorCapabilities {
    pub supports_hardware_trigger: bool,
    pub supports_timestamp: bool,
    pub supports_zero_copy: bool,
    pub max_rate_hz: f64,
    pub min_rate_hz: f64,
}

pub trait SensorData: Send + Sync {
    fn timestamp(&self) -> Timestamp;
    fn size_bytes(&self) -> usize;
}

pub trait Sensor: Send + Sync {
    fn device_info(&self) -> &DeviceInfo;
    fn capabilities(&self) -> SensorCapabilities;
    fn configure(&mut self, config: SensorConfig) -> Result<(), String>;
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
}

// ============================================================================
// Sensor Configuration
// ============================================================================

#[derive(Debug, Clone)]
pub enum TriggerMode {
    FreeRun,
    External { pin: u8 },
    Software,
}

#[derive(Debug, Clone)]
pub struct SensorConfig {
    pub rate_hz: f64,
    pub trigger_mode: TriggerMode,
    pub use_dma: bool,
}

impl Default for SensorConfig {
    fn default() -> Self {
        SensorConfig {
            rate_hz: 30.0,
            trigger_mode: TriggerMode::FreeRun,
            use_dma: true,
        }
    }
}

// ============================================================================
// Camera Implementation
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    RGB8,
    RGBA8,
    BGR8,
    MONO8,
    MONO16,
}

#[derive(Debug, Clone)]
pub struct Image {
    pub timestamp: Timestamp,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub data: Vec<u8>,
    pub frame_id: u64,
}

impl SensorData for Image {
    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
    
    fn size_bytes(&self) -> usize {
        self.data.len()
    }
}

pub struct CameraDriver {
    device_info: DeviceInfo,
    config: SensorConfig,
    is_streaming: bool,
    frame_count: u64,
    
    // Simulated DMA buffers
    dma_buffers: Vec<Vec<u8>>,
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
    
    pub fn capture_frame(&mut self) -> Result<Image, String> {
        if !self.is_streaming {
            return Err("Camera not streaming".to_string());
        }
        
        // Simulate frame capture with DMA
        let width = 640;
        let height = 480;
        let format = ImageFormat::RGB8;
        
        // In real implementation, this would be a pointer to DMA buffer
        let data = if self.config.use_dma {
            // Zero-copy: return reference to DMA buffer
            self.dma_buffers[self.current_buffer].clone()
        } else {
            // Copy path
            vec![0u8; (width * height * 3) as usize]
        };
        
        self.current_buffer = (self.current_buffer + 1) % self.dma_buffers.len();
        self.frame_count += 1;
        
        Ok(Image {
            timestamp: Timestamp::now(),
            width,
            height,
            format,
            data,
            frame_id: self.frame_count,
        })
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
            max_rate_hz: 120.0,
            min_rate_hz: 1.0,
        }
    }
    
    fn configure(&mut self, config: SensorConfig) -> Result<(), String> {
        self.config = config;
        
        // Allocate DMA buffers if needed
        if config.use_dma {
            let buffer_size = 640 * 480 * 3; // RGB8
            self.dma_buffers = (0..4)
                .map(|_| vec![0u8; buffer_size])
                .collect();
        }
        
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), String> {
        println!("[Camera] Starting stream at {:.1} Hz", self.config.rate_hz);
        self.is_streaming = true;
        self.frame_count = 0;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), String> {
        println!("[Camera] Stopping stream");
        self.is_streaming = false;
        Ok(())
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
        
        // Simulate point cloud capture
        let num_points = 1000;
        let points: Vec<Point3D> = (0..num_points)
            .map(|i| {
                let angle = (i as f32) * 2.0 * std::f32::consts::PI / (num_points as f32);
                let range = 5.0;
                Point3D {
                    x: range * angle.cos(),
                    y: range * angle.sin(),
                    z: 0.0,
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
            supports_zero_copy: false,
            max_rate_hz: 20.0,
            min_rate_hz: 1.0,
        }
    }
    
    fn configure(&mut self, config: SensorConfig) -> Result<(), String> {
        self.config = config;
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), String> {
        println!("[LiDAR] Starting scan at {:.1} Hz", self.config.rate_hz);
        self.is_scanning = true;
        self.scan_count = 0;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), String> {
        println!("[LiDAR] Stopping scan");
        self.is_scanning = false;
        Ok(())
    }
}

// ============================================================================
// IMU Implementation
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct ImuData {
    pub timestamp: Timestamp,
    pub linear_acceleration: Vector3,
    pub angular_velocity: Vector3,
    pub orientation: Vector3,
}

impl SensorData for ImuData {
    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
    
    fn size_bytes(&self) -> usize {
        std::mem::size_of::<ImuData>()
    }
}

pub struct ImuDriver {
    device_info: DeviceInfo,
    config: SensorConfig,
    is_streaming: bool,
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
            config: SensorConfig::default(),
            is_streaming: false,
        }
    }
    
    pub fn read_data(&self) -> Result<ImuData, String> {
        if !self.is_streaming {
            return Err("IMU not streaming".to_string());
        }
        
        // Simulate IMU reading
        Ok(ImuData {
            timestamp: Timestamp::now(),
            linear_acceleration: Vector3 { x: 0.0, y: 0.0, z: 9.81 },
            angular_velocity: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            orientation: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
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
        println!("[IMU] Stopping stream");
        self.is_streaming = false;
        Ok(())
    }
}

// ============================================================================
// Multi-Sensor Synchronization
// ============================================================================

pub struct SynchronizedData {
    pub timestamp: Timestamp,
    pub camera: Option<Image>,
    pub lidar: Option<PointCloud>,
    pub imu: Option<ImuData>,
}

pub struct SensorSynchronizer {
    tolerance_ms: u64,
    camera_buffer: Vec<Image>,
    lidar_buffer: Vec<PointCloud>,
    imu_buffer: Vec<ImuData>,
    max_buffer_size: usize,
}

impl SensorSynchronizer {
    pub fn new(tolerance_ms: u64) -> Self {
        SensorSynchronizer {
            tolerance_ms,
            camera_buffer: Vec::new(),
            lidar_buffer: Vec::new(),
            imu_buffer: Vec::new(),
            max_buffer_size: 10,
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
    
    pub fn try_synchronize(&mut self) -> Option<SynchronizedData> {
        if self.camera_buffer.is_empty() || 
           self.lidar_buffer.is_empty() || 
           self.imu_buffer.is_empty() {
            return None;
        }
        
        // Find closest timestamps
        let camera = &self.camera_buffer[0];
        let camera_time = camera.timestamp.sec * 1000 + 
                         (camera.timestamp.nanosec / 1_000_000) as u64;
        
        // Find closest LiDAR scan
        let lidar_idx = self.find_closest_timestamp(&self.lidar_buffer, camera_time);
        let lidar = &self.lidar_buffer[lidar_idx];
        let lidar_time = lidar.timestamp.sec * 1000 + 
                        (lidar.timestamp.nanosec / 1_000_000) as u64;
        
        // Find closest IMU data
        let imu_idx = self.find_closest_timestamp(&self.imu_buffer, camera_time);
        let imu = &self.imu_buffer[imu_idx];
        let imu_time = imu.timestamp.sec * 1000 + 
                      (imu.timestamp.nanosec / 1_000_000) as u64;
        
        // Check if within tolerance
        let max_diff = camera_time.max(lidar_time).max(imu_time) - 
                      camera_time.min(lidar_time).min(imu_time);
        
        if max_diff <= self.tolerance_ms {
            // Synchronized data found
            let synced = SynchronizedData {
                timestamp: camera.timestamp,
                camera: Some(self.camera_buffer.remove(0)),
                lidar: Some(self.lidar_buffer.remove(lidar_idx)),
                imu: Some(self.imu_buffer.remove(imu_idx)),
            };
            Some(synced)
        } else {
            None
        }
    }
    
    fn find_closest_timestamp<T: SensorData>(&self, buffer: &[T], target_ms: u64) -> usize {
        buffer.iter()
            .enumerate()
            .min_by_key(|(_, data)| {
                let data_ms = data.timestamp().sec * 1000 + 
                             (data.timestamp().nanosec / 1_000_000) as u64;
                ((data_ms as i64) - (target_ms as i64)).abs()
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }
}

// ============================================================================
// Sensor Manager
// ============================================================================

pub struct SensorManager {
    sensors: HashMap<String, Box<dyn Sensor>>,
}

impl SensorManager {
    pub fn new() -> Self {
        SensorManager {
            sensors: HashMap::new(),
        }
    }
    
    pub fn register_sensor(&mut self, name: String, sensor: Box<dyn Sensor>) {
        println!("[SensorManager] Registering {}: {}", 
            name, sensor.device_info().name);
        self.sensors.insert(name, sensor);
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
            println!("{}: {:.0}-{:.0} Hz, HW trigger: {}, Zero-copy: {}",
                name,
                caps.min_rate_hz,
                caps.max_rate_hz,
                caps.supports_hardware_trigger,
                caps.supports_zero_copy
            );
        }
    }
}

// ============================================================================
// Demo
// ============================================================================

fn main() {
    println!("NROS Hardware Abstraction Layer Demo\n");
    
    // Create sensor manager
    let mut manager = SensorManager::new();
    
    // Register sensors
    manager.register_sensor(
        "front_camera".to_string(),
        Box::new(CameraDriver::new("Front Camera"))
    );
    manager.register_sensor(
        "front_lidar".to_string(),
        Box::new(LidarDriver::new("Front LiDAR"))
    );
    manager.register_sensor(
        "imu".to_string(),
        Box::new(ImuDriver::new("IMU"))
    );
    
    // Show capabilities
    manager.list_capabilities();
    
    // Configure sensors
    println!("\n=== Configuration ===");
    let config = SensorConfig {
        rate_hz: 30.0,
        trigger_mode: TriggerMode::FreeRun,
        use_dma: true,
    };
    manager.configure_all(config).unwrap();
    
    // Start sensors
    println!("\n=== Starting Sensors ===");
    manager.start_all().unwrap();
    
    // Simulate synchronized capture
    println!("\n=== Synchronized Capture Demo ===");
    let mut camera = CameraDriver::new("Demo Camera");
    let mut lidar = LidarDriver::new("Demo LiDAR");
    let mut imu = ImuDriver::new("Demo IMU");
    
    camera.configure(SensorConfig::default()).unwrap();
    lidar.configure(SensorConfig::default()).unwrap();
    imu.configure(SensorConfig::default()).unwrap();
    
    camera.start().unwrap();
    lidar.start().unwrap();
    imu.start().unwrap();
    
    let mut synchronizer = SensorSynchronizer::new(10); // 10ms tolerance
    
    println!("Capturing 5 synchronized frames...\n");
    for i in 0..5 {
        // Simulate sensor captures with slight timing variations
        std::thread::sleep(Duration::from_millis(30));
        
        if let Ok(frame) = camera.capture_frame() {
            println!("Frame {} captured: {}x{} ({} bytes)",
                frame.frame_id, frame.width, frame.height, frame.size_bytes());
            synchronizer.add_camera_frame(frame);
        }
        
        if let Ok(scan) = lidar.capture_scan() {
            println!("Scan {} captured: {} points ({} bytes)",
                scan.scan_id, scan.points.len(), scan.size_bytes());
            synchronizer.add_lidar_scan(scan);
        }
        
        if let Ok(data) = imu.read_data() {
            println!("IMU data: acc=[{:.2}, {:.2}, {:.2}]",
                data.linear_acceleration.x,
                data.linear_acceleration.y,
                data.linear_acceleration.z);
            synchronizer.add_imu_data(data);
        }
        
        // Try to get synchronized data
        if let Some(synced) = synchronizer.try_synchronize() {
            println!("✓ Synchronized set {} ready!", i + 1);
        }
        println!();
    }
    
    // Stop sensors
    println!("=== Stopping Sensors ===");
    manager.stop_all().unwrap();
    
    println!("\nDemo complete!");
}
