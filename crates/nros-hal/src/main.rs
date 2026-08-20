//! NROS Hardware Abstraction Layer Demo
//! Simulates sensor discovery, config, zero-copy capture, multi-sensor sync

use nros_hal::{
    Sensor, SensorConfig, TriggerMode, CameraDriver, LidarDriver, ImuDriver,
    SensorManager, SensorSynchronizer, ImageFormat,
};
use std::time::Duration;

fn main() {
    println!("NROS Hardware Abstraction Layer Demo\n");
    println!("Implements DESIGN.md §6, §16 HAL:");
    println!(" - Unified Sensor trait with capability discovery");
    println!(" - Zero-copy DMA buffers (camera DMAs to GPU memory)");
    println!(" - Hardware-triggered sync (10ms tolerance)");
    println!(" - SensorManager hot-plug + coordinated operation\n");

    // Create sensor manager — discovery per §16.1
    let mut manager = SensorManager::new();

    // Register sensors — real: automatic HAL driver loading via ~/.nros/plugins/
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

    manager.discover();
    manager.list_capabilities();

    // Configure sensors — with resolution, trigger mode, DMA
    println!("\n=== Configuration (30Hz, DMA enabled, FreeRun) ===");
    let config = SensorConfig {
        rate_hz: 30.0,
        trigger_mode: TriggerMode::FreeRun,
        use_dma: true,
        resolution: Some((640, 480)),
        buffer_count: 4,
    };
    manager.configure_all(config).unwrap();

    // Start sensors
    println!("\n=== Starting Sensors ===");
    manager.start_all().unwrap();

    // Simulate synchronized capture — DESIGN.md §16.2 SyncGroup with hardware trigger
    println!("\n=== Synchronized Capture Demo (Hardware-triggered would use GPIO pulse) ===");
    let mut camera = CameraDriver::new("Demo Camera");
    let mut lidar = LidarDriver::new("Demo LiDAR");
    let mut imu = ImuDriver::new("Demo IMU");

    camera.configure(SensorConfig { rate_hz: 30.0, use_dma: true, resolution: Some((640,480)), buffer_count: 4, ..Default::default() }).unwrap();
    lidar.configure(SensorConfig { rate_hz: 10.0, trigger_mode: TriggerMode::External{ pin: 4 }, ..Default::default() }).unwrap();
    imu.configure(SensorConfig { rate_hz: 200.0, use_dma: true, ..Default::default() }).unwrap();

    camera.start().unwrap();
    lidar.start().unwrap();
    imu.start().unwrap();

    let mut synchronizer = SensorSynchronizer::new(10); // 10ms tolerance per spec

    println!("Capturing 5 synchronized frames (simulating 30Hz camera, 10Hz LiDAR, 200Hz IMU)...\n");
    for i in 0..5 {
        std::thread::sleep(Duration::from_millis(33)); // ~30Hz

        if let Ok(frame) = camera.capture_frame() {
            println!("Frame {} captured: {}x{} {:?} ({} bytes) DMA buf: {:?}",
                frame.frame_id, frame.width, frame.height, frame.format, frame.data.len(), frame.dma_buffer_id);
            synchronizer.add_camera_frame(frame);
        }

        // LiDAR @ 10Hz — only every 3rd iteration
        if i % 3 == 0 {
            if let Ok(scan) = lidar.capture_scan() {
                println!("Scan {} captured: {} points ({} bytes)",
                    scan.scan_id, scan.points.len(), scan.points.len() * std::mem::size_of::<nros_hal::Point3D>());
                synchronizer.add_lidar_scan(scan);
            }
        }

        // IMU @ 200Hz — multiple per camera frame
        for _ in 0..6 {
            if let Ok(data) = imu.read_data() {
                if i == 0 {
                    println!("IMU data seq {}: acc=[{:.2}, {:.2}, {:.2}]",
                        data.sequence,
                        data.linear_acceleration.x,
                        data.linear_acceleration.y,
                        data.linear_acceleration.z);
                }
                synchronizer.add_imu_data(data);
            }
        }

        // Try to get synchronized data — mirrors DESIGN.md SyncGroup::synchronized_capture()
        if let Some(synced) = synchronizer.try_synchronize() {
            println!("✓ Synchronized set {} ready! sync_quality={}ms, ts={}ms", 
                i + 1, synced.sync_quality_ms, synced.timestamp.to_millis());
        } else {
            let (c,l,imu_c) = synchronizer.buffered_counts();
            println!("  Buffers: cam={}, lidar={}, imu={} — waiting for sync within 10ms...", c,l,imu_c);
        }
        println!();
    }

    println!("Sync success rate: {:.1}% ({} / {} attempts)", synchronizer.success_rate(), synchronizer.sync_success, synchronizer.sync_attempts);

    // Demonstrate zero-copy DMA pipeline per §16.4
    println!("\n=== Zero-Copy DMA Pipeline Demo (Camera -> GPU) ===");
    println!("Real NROS: camera.register_dma_targets(&gpu_buffers) -> camera DMA directly to GPU memory, no CPU copy");
    let mut cam2 = CameraDriver::new("GPU Camera");
    let gpu_buffer_size = 640*480*3;
    cam2.register_dma_targets(4, gpu_buffer_size);
    cam2.configure(SensorConfig { use_dma: true, buffer_count: 4, ..Default::default() }).unwrap();
    cam2.start().unwrap();
    for j in 0..3 {
        let frame = cam2.capture_frame().unwrap();
        println!("  DMA frame {}: GPU buffer id {:?}, ptr={:p} (would launch CUDA kernel directly)", 
            j, frame.dma_buffer_id, frame.data.as_ptr());
    }

    // Stop sensors
    println!("\n=== Stopping Sensors ===");
    manager.stop_all().unwrap();
    camera.stop().unwrap();
    lidar.stop().unwrap();
    imu.stop().unwrap();

    println!("\nDemo complete! Implements:");
    println!(" - Camera @ 30Hz RGB8 640x480, DMA zero-copy");
    println!(" - LiDAR @ 10Hz 1000+ points");
    println!(" - IMU @ 200Hz");
    println!(" - Sync 10ms tolerance, buffered multi-sensor fusion");
}
