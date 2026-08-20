// NROS Core Implementation: Zero-Copy Inter-Process Communication
// This demonstrates the core lock-free ring buffer and message passing system

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::ptr;
use std::alloc::{alloc, dealloc, Layout};
use std::marker::PhantomData;
use std::thread;

// ============================================================================
// Lock-Free Ring Buffer (SPSC - Single Producer Single Consumer)
// ============================================================================

const CACHE_LINE_SIZE: usize = 64;

#[repr(align(64))] // Align to cache line to prevent false sharing
struct AlignedU64(AtomicU64);

pub struct RingBuffer<T> {
    buffer: *mut T,
    capacity: usize,
    
    // Separate cache lines for write/read indices to prevent false sharing
    write_idx: AlignedU64,
    _pad1: [u8; CACHE_LINE_SIZE - 8],
    read_idx: AlignedU64,
    _pad2: [u8; CACHE_LINE_SIZE - 8],
    
    _phantom: PhantomData<T>,
}

unsafe impl<T: Send> Send for RingBuffer<T> {}
unsafe impl<T: Send> Sync for RingBuffer<T> {}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two(), "Capacity must be power of 2");
        
        let layout = Layout::array::<T>(capacity).unwrap();
        let buffer = unsafe { alloc(layout) as *mut T };
        
        RingBuffer {
            buffer,
            capacity,
            write_idx: AlignedU64(AtomicU64::new(0)),
            _pad1: [0; CACHE_LINE_SIZE - 8],
            read_idx: AlignedU64(AtomicU64::new(0)),
            _pad2: [0; CACHE_LINE_SIZE - 8],
            _phantom: PhantomData,
        }
    }
    
    /// Try to reserve a slot for writing (producer side)
    pub fn try_reserve(&self) -> Option<ReservedSlot<T>> {
        let write = self.write_idx.0.load(Ordering::Relaxed);
        let read = self.read_idx.0.load(Ordering::Acquire);
        
        // Check if buffer is full
        if write - read >= self.capacity as u64 {
            return None;
        }
        
        let idx = (write as usize) & (self.capacity - 1);
        
        Some(ReservedSlot {
            ptr: unsafe { self.buffer.add(idx) },
            ring: self,
            write_idx: write,
        })
    }
    
    /// Commit a write (called by ReservedSlot::commit)
    fn commit_write(&self, write_idx: u64) {
        // Release ensures all writes to the slot are visible
        self.write_idx.0.store(write_idx + 1, Ordering::Release);
    }
    
    /// Try to read a message (consumer side)
    pub fn try_read(&self) -> Option<&T> {
        let read = self.read_idx.0.load(Ordering::Relaxed);
        let write = self.write_idx.0.load(Ordering::Acquire);
        
        // Check if buffer is empty
        if read >= write {
            return None;
        }
        
        let idx = (read as usize) & (self.capacity - 1);
        Some(unsafe { &*self.buffer.add(idx) })
    }
    
    /// Advance read pointer after processing message
    pub fn consume(&self) {
        let read = self.read_idx.0.load(Ordering::Relaxed);
        self.read_idx.0.store(read + 1, Ordering::Release);
    }
    
    pub fn len(&self) -> usize {
        let write = self.write_idx.0.load(Ordering::Acquire);
        let read = self.read_idx.0.load(Ordering::Acquire);
        (write - read) as usize
    }
}

impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        let layout = Layout::array::<T>(self.capacity).unwrap();
        unsafe { dealloc(self.buffer as *mut u8, layout) };
    }
}

// ============================================================================
// Reserved Slot - RAII handle for zero-copy writes
// ============================================================================

pub struct ReservedSlot<'a, T> {
    ptr: *mut T,
    ring: &'a RingBuffer<T>,
    write_idx: u64,
}

impl<'a, T> ReservedSlot<'a, T> {
    /// Get mutable access to write data directly into shared memory
    pub fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
    
    /// Commit the write, making it visible to readers
    pub fn commit(self) {
        self.ring.commit_write(self.write_idx);
        std::mem::forget(self); // Don't run Drop
    }
}

impl<'a, T> Drop for ReservedSlot<'a, T> {
    fn drop(&mut self) {
        // If commit() wasn't called, the write is abandoned
        // No need to update write_idx, slot remains reserved
    }
}

// ============================================================================
// Message Types
// ============================================================================

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Timestamp {
    pub sec: u64,
    pub nanosec: u32,
}

impl Timestamp {
    pub fn now() -> Self {
        // In real implementation, use clock_gettime(CLOCK_MONOTONIC)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        Timestamp {
            sec: now.as_secs(),
            nanosec: now.subsec_nanos(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Twist {
    pub timestamp: Timestamp,
    pub linear: Vector3,
    pub angular: Vector3,
}

// ============================================================================
// Publisher/Subscriber
// ============================================================================

pub struct Publisher<T> {
    ring: Arc<RingBuffer<T>>,
    topic: String,
}

impl<T> Publisher<T> {
    pub fn new(topic: &str, capacity: usize) -> Self {
        Publisher {
            ring: Arc::new(RingBuffer::new(capacity)),
            topic: topic.to_string(),
        }
    }
    
    /// Allocate space in shared memory for zero-copy publishing
    pub fn allocate(&self) -> Option<MessageHandle<T>> {
        self.ring.try_reserve().map(|slot| MessageHandle { slot })
    }
    
    /// Publish by copying (fallback for small messages)
    pub fn publish_copy(&self, msg: T) -> Result<(), &'static str> {
        let mut handle = self.allocate().ok_or("Buffer full")?;
        unsafe {
            ptr::write(handle.slot.ptr, msg);
        }
        handle.commit();
        Ok(())
    }
}

pub struct MessageHandle<'a, T> {
    slot: ReservedSlot<'a, T>,
}

impl<'a, T> MessageHandle<'a, T> {
    /// Get mutable reference to write message data
    pub fn as_mut(&mut self) -> &mut T {
        self.slot.as_mut()
    }
    
    /// Commit and publish the message
    pub fn commit(self) {
        self.slot.commit();
    }
}

pub struct Subscriber<T> {
    ring: Arc<RingBuffer<T>>,
    topic: String,
}

impl<T> Subscriber<T> {
    pub fn new(ring: Arc<RingBuffer<T>>, topic: &str) -> Self {
        Subscriber {
            ring,
            topic: topic.to_string(),
        }
    }
    
    /// Non-blocking receive
    pub fn try_recv(&self) -> Option<&T> {
        self.ring.try_read()
    }
    
    /// Mark message as consumed
    pub fn consume(&self) {
        self.ring.consume();
    }
    
    /// Get number of pending messages
    pub fn pending(&self) -> usize {
        self.ring.len()
    }
}

// ============================================================================
// Performance Monitoring
// ============================================================================

pub struct PerformanceStats {
    pub messages_sent: AtomicUsize,
    pub messages_received: AtomicUsize,
    pub total_latency_ns: AtomicU64,
    pub max_latency_ns: AtomicU64,
}

impl PerformanceStats {
    pub fn new() -> Self {
        PerformanceStats {
            messages_sent: AtomicUsize::new(0),
            messages_received: AtomicUsize::new(0),
            total_latency_ns: AtomicU64::new(0),
            max_latency_ns: AtomicU64::new(0),
        }
    }
    
    pub fn record_send(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_receive(&self, latency_ns: u64) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ns.fetch_add(latency_ns, Ordering::Relaxed);
        
        // Update max latency
        let mut current_max = self.max_latency_ns.load(Ordering::Relaxed);
        while latency_ns > current_max {
            match self.max_latency_ns.compare_exchange_weak(
                current_max,
                latency_ns,
                Ordering::Relaxed,
                Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }
    
    pub fn avg_latency_us(&self) -> f64 {
        let total = self.total_latency_ns.load(Ordering::Relaxed);
        let count = self.messages_received.load(Ordering::Relaxed);
        if count == 0 {
            0.0
        } else {
            (total as f64 / count as f64) / 1000.0
        }
    }
    
    pub fn max_latency_us(&self) -> f64 {
        self.max_latency_ns.load(Ordering::Relaxed) as f64 / 1000.0
    }
}

// ============================================================================
// Example Usage & Benchmark
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use std::thread;
    
    #[test]
    fn test_zero_copy_pubsub() {
        let capacity = 1024;
        let publisher = Publisher::<Twist>::new("/cmd_vel", capacity);
        let subscriber = Subscriber::new(publisher.ring.clone(), "/cmd_vel");
        
        // Zero-copy publish
        let mut msg_handle = publisher.allocate().unwrap();
        {
            let msg = msg_handle.as_mut();
            msg.timestamp = Timestamp::now();
            msg.linear = Vector3 { x: 1.0, y: 0.0, z: 0.0 };
            msg.angular = Vector3 { x: 0.0, y: 0.0, z: 0.5 };
        }
        msg_handle.commit();
        
        // Receive
        let received = subscriber.try_recv().unwrap();
        assert!((received.linear.x - 1.0).abs() < 1e-10);
        subscriber.consume();
    }
    
    #[test]
    fn benchmark_latency() {
        let capacity = 1024;
        let publisher = Publisher::<Twist>::new("/benchmark", capacity);
        let subscriber = Subscriber::new(publisher.ring.clone(), "/benchmark");
        let stats = Arc::new(PerformanceStats::new());
        
        let stats_clone = stats.clone();
        let consumer = thread::spawn(move || {
            loop {
                if let Some(msg) = subscriber.try_recv() {
                    let now = Timestamp::now();
                    let latency_ns = (now.sec - msg.timestamp.sec) * 1_000_000_000
                        + (now.nanosec as u64)
                        - (msg.timestamp.nanosec as u64);
                    
                    stats_clone.record_receive(latency_ns);
                    subscriber.consume();
                    
                    if stats_clone.messages_received.load(Ordering::Relaxed) >= 100_000 {
                        break;
                    }
                }
            }
        });
        
        let start = Instant::now();
        for _ in 0..100_000 {
            let mut handle = publisher.allocate().unwrap();
            {
                let msg = handle.as_mut();
                msg.timestamp = Timestamp::now();
                msg.linear = Vector3 { x: 1.0, y: 0.0, z: 0.0 };
                msg.angular = Vector3 { x: 0.0, y: 0.0, z: 0.5 };
            }
            handle.commit();
            stats.record_send();
        }
        
        consumer.join().unwrap();
        let elapsed = start.elapsed();
        
        println!("\n=== NROS Zero-Copy Performance ===");
        println!("Messages sent:     {}", stats.messages_sent.load(Ordering::Relaxed));
        println!("Messages received: {}", stats.messages_received.load(Ordering::Relaxed));
        println!("Total time:        {:.2?}", elapsed);
        println!("Throughput:        {:.0} msg/s", 
            100_000.0 / elapsed.as_secs_f64());
        println!("Avg latency:       {:.2} μs", stats.avg_latency_us());
        println!("Max latency:       {:.2} μs", stats.max_latency_us());
        
        // NROS target: < 10 μs average latency
        assert!(stats.avg_latency_us() < 10.0, 
            "Average latency {} μs exceeds target", stats.avg_latency_us());
    }
    
    #[test]
    fn test_ring_buffer_full() {
        let ring = RingBuffer::<u64>::new(4);
        
        // Fill buffer
        for i in 0..4 {
            let mut slot = ring.try_reserve().unwrap();
            unsafe { ptr::write(slot.ptr, i); }
            slot.commit();
        }
        
        // Should be full now
        assert!(ring.try_reserve().is_none());
        
        // Consume one
        let _ = ring.try_read().unwrap();
        ring.consume();
        
        // Should have space again
        assert!(ring.try_reserve().is_some());
    }
}

// ============================================================================
// Main - Interactive Demo
// ============================================================================

fn main() {
    println!("NROS Core Implementation - Zero-Copy IPC Demo\n");
    
    let capacity = 256;
    let publisher = Publisher::<Twist>::new("/cmd_vel", capacity);
    let subscriber = Subscriber::new(publisher.ring.clone(), "/cmd_vel");
    let stats = Arc::new(PerformanceStats::new());
    
    // Spawn consumer thread
    let stats_clone = stats.clone();
    let consumer_handle = thread::spawn(move || {
        println!("Consumer: Started");
        loop {
            if let Some(msg) = subscriber.try_recv() {
                let now = Timestamp::now();
                let latency_ns = (now.sec - msg.timestamp.sec) * 1_000_000_000
                    + (now.nanosec as u64)
                    - (msg.timestamp.nanosec as u64);
                
                stats_clone.record_receive(latency_ns);
                
                println!("Consumer: Received [linear: {:.2}, angular: {:.2}] - latency: {:.2} μs",
                    msg.linear.x, msg.angular.z, latency_ns as f64 / 1000.0);
                
                subscriber.consume();
                
                if stats_clone.messages_received.load(Ordering::Relaxed) >= 10 {
                    break;
                }
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
        println!("Consumer: Finished");
    });
    
    // Producer thread
    println!("Producer: Publishing 10 messages with 100ms interval\n");
    for i in 0..10 {
        thread::sleep(std::time::Duration::from_millis(100));
        
        let mut handle = publisher.allocate()
            .expect("Buffer should not be full");
        
        {
            let msg = handle.as_mut();
            msg.timestamp = Timestamp::now();
            msg.linear = Vector3 { 
                x: (i as f64) * 0.1, 
                y: 0.0, 
                z: 0.0 
            };
            msg.angular = Vector3 { 
                x: 0.0, 
                y: 0.0, 
                z: (i as f64) * 0.05 
            };
        }
        
        handle.commit();
        stats.record_send();
        
        println!("Producer: Published message #{}", i + 1);
    }
    
    consumer_handle.join().unwrap();
    
    println!("\n=== Final Statistics ===");
    println!("Messages sent:     {}", stats.messages_sent.load(Ordering::Relaxed));
    println!("Messages received: {}", stats.messages_received.load(Ordering::Relaxed));
    println!("Avg latency:       {:.2} μs", stats.avg_latency_us());
    println!("Max latency:       {:.2} μs", stats.max_latency_us());
}
