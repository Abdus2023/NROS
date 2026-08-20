//! Compression example — Mock vs Lz4 per AUDIT P1 separation
//! Run: cargo run -p nros-transport --example compression
//! With real compression: cargo run -p nros-transport --example compression --features real-compression

use nros_transport::{MockCompressionEngine, Lz4CompressionEngine, CompressionEngineTrait};

fn main() {
    let data = vec![b'A'; 5000]; // 5KB of repetitive data — LZ4 should compress well

    let mock = MockCompressionEngine::new(1024);
    let lz4 = Lz4CompressionEngine::new(1024);

    println!("Original size: {} bytes", data.len());
    println!("Mock should_compress: {} (threshold 1024)", mock.should_compress(&data));
    println!("Lz4 should_compress: {}", lz4.should_compress(&data));

    let mock_compressed = mock.compress(&data);
    println!("\nMock compression: {} -> {} bytes, ratio {:.2}, name: {}, simulated: {}",
        data.len(), mock_compressed.len(), mock_compressed.len() as f64 / data.len() as f64,
        mock.name(), mock.is_simulated());

    let lz4_compressed = lz4.compress(&data);
    println!("Lz4 compression: {} -> {} bytes, ratio {:.2}, name: {}, simulated: {}",
        data.len(), lz4_compressed.len(), lz4_compressed.len() as f64 / data.len() as f64,
        lz4.name(), lz4.is_simulated());

    // Decompress
    let mock_decompressed = mock.decompress(&mock_compressed).unwrap();
    let lz4_decompressed = lz4.decompress(&lz4_compressed).unwrap();

    assert_eq!(mock_decompressed, data);
    println!("\nMock decompress ok: {}", mock_decompressed.len() == data.len());

    #[cfg(feature = "real-compression")]
    {
        assert_eq!(lz4_decompressed, data);
        println!("Lz4 decompress ok (real lz4_flex): {} -> {} bytes, ratio {:.2} measured, not assumed",
            data.len(), lz4_compressed.len(), lz4_compressed.len() as f64 / data.len() as f64);
        println!("Evidence: REAL compression measured, not estimated_ratio 0.6 assumed");
    }
    #[cfg(not(feature = "real-compression"))]
    {
        // In scaffolded mode, lz4 still returns original data (since we prefix flag only)
        assert_eq!(lz4_decompressed, data);
        println!("Lz4 decompress ok (scaffolded, still [2]+data, not real LZ4 — enable --features real-compression for real)");
        println!("Evidence: SCAFFOLDED — would use lz4_flex::compress_prepend_size in real, currently flag only");
    }

    println!("\nStatus per EVIDENCE_REGISTRY:");
    println!("- MockCompression: SIMULATED — does NOT actually compress, just [1]+data, estimated_ratio 0.6 assumed not measured");
    println!("- Lz4Compression: {} — would use lz4_flex, currently {}",
        if cfg!(feature = "real-compression") { "REAL (lz4_flex)" } else { "SCAFFOLDED [2]+data" },
        if cfg!(feature = "real-compression") { "measured ratio" } else { "flag only, needs --features real-compression" }
    );
}
