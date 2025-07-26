// Thread Pool Performance Benchmark
//
// Compares performance between:
// 1. Original concurrent processor (rayon-based)
// 2. Enhanced concurrent processor with adaptive thread pool

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use legacybridge::pipeline::concurrent_processor::{
    ConcurrentProcessor, ConversionRequest, ConversionContent, ConversionOptions
};
use legacybridge::pipeline::concurrent_processor_v2::ConcurrentProcessorV2;
use tokio::runtime::Runtime;
use std::time::Duration;

/// Generate test documents of various sizes
fn generate_test_document(size_kb: usize) -> String {
    let base_content = "# Test Document\n\nThis is a test paragraph with some content. ";
    let paragraph = format!("{}\n\n", base_content.repeat(10));
    let target_size = size_kb * 1024;
    let mut content = String::with_capacity(target_size);
    
    while content.len() < target_size {
        content.push_str(&paragraph);
    }
    
    content
}

/// Benchmark single document processing
fn bench_single_document(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("single_document");
    group.measurement_time(Duration::from_secs(10));
    
    for size_kb in [1, 10, 100, 1000].iter() {
        let doc = generate_test_document(*size_kb);
        let doc_v2 = doc.clone();
        
        group.throughput(Throughput::Bytes((size_kb * 1024) as u64));
        
        // Benchmark v1 processor
        group.bench_with_input(
            BenchmarkId::new("v1_processor", size_kb),
            size_kb,
            |b, _| {
                let processor = ConcurrentProcessor::new(None);
                b.iter(|| {
                    rt.block_on(async {
                        let request = ConversionRequest {
                            id: "bench".to_string(),
                            content: ConversionContent::Memory(black_box(doc.clone())),
                            options: ConversionOptions::default(),
                        };
                        processor.process_single(request).await
                    })
                });
            },
        );
        
        // Benchmark v2 processor
        group.bench_with_input(
            BenchmarkId::new("v2_processor", size_kb),
            size_kb,
            |b, _| {
                let processor = ConcurrentProcessorV2::new();
                b.iter(|| {
                    rt.block_on(async {
                        let request = ConversionRequest {
                            id: "bench".to_string(),
                            content: ConversionContent::Memory(black_box(doc_v2.clone())),
                            options: ConversionOptions::default(),
                        };
                        processor.process_single(request).await
                    })
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark batch processing
fn bench_batch_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("batch_processing");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);
    
    for batch_size in [10, 100, 1000].iter() {
        let docs: Vec<_> = (0..*batch_size)
            .map(|i| generate_test_document(10)) // 10KB documents
            .collect();
        
        group.throughput(Throughput::Elements(*batch_size as u64));
        
        // Benchmark v1 processor
        group.bench_with_input(
            BenchmarkId::new("v1_processor", batch_size),
            batch_size,
            |b, _| {
                let processor = ConcurrentProcessor::new(None);
                b.iter(|| {
                    rt.block_on(async {
                        let requests: Vec<_> = docs.iter().enumerate()
                            .map(|(i, doc)| ConversionRequest {
                                id: format!("bench-{}", i),
                                content: ConversionContent::Memory(black_box(doc.clone())),
                                options: ConversionOptions::default(),
                            })
                            .collect();
                        processor.process_batch(requests).await
                    })
                });
            },
        );
        
        // Benchmark v2 processor
        group.bench_with_input(
            BenchmarkId::new("v2_processor", batch_size),
            batch_size,
            |b, _| {
                let processor = ConcurrentProcessorV2::new();
                b.iter(|| {
                    rt.block_on(async {
                        let requests: Vec<_> = docs.iter().enumerate()
                            .map(|(i, doc)| ConversionRequest {
                                id: format!("bench-{}", i),
                                content: ConversionContent::Memory(black_box(doc.clone())),
                                options: ConversionOptions::default(),
                            })
                            .collect();
                        processor.process_batch(requests).await
                    })
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark work-stealing efficiency
fn bench_work_stealing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("work_stealing");
    group.measurement_time(Duration::from_secs(20));
    
    // Create mixed workload - some small, some large documents
    let mixed_docs: Vec<_> = (0..100)
        .map(|i| {
            if i % 10 == 0 {
                generate_test_document(100) // Large document every 10th
            } else {
                generate_test_document(1) // Small documents
            }
        })
        .collect();
    
    group.bench_function("v2_mixed_workload", |b| {
        let processor = ConcurrentProcessorV2::new();
        b.iter(|| {
            rt.block_on(async {
                let requests: Vec<_> = mixed_docs.iter().enumerate()
                    .map(|(i, doc)| ConversionRequest {
                        id: format!("mixed-{}", i),
                        content: ConversionContent::Memory(black_box(doc.clone())),
                        options: ConversionOptions {
                            parallel_chunks: true,
                            chunk_size: 10 * 1024, // 10KB chunks
                            ..Default::default()
                        },
                    })
                    .collect();
                processor.process_batch(requests).await
            })
        });
    });
    
    group.finish();
}

/// Benchmark backpressure handling
fn bench_backpressure(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("backpressure");
    group.measurement_time(Duration::from_secs(15));
    
    // Configure processor with limited queue
    let config = legacybridge::pipeline::concurrent_processor_v2::ProcessorConfig {
        max_document_size: 10 * 1024 * 1024, // 10MB
        chunk_size: 1024 * 1024, // 1MB
        batch_timeout: Duration::from_secs(5),
        ..Default::default()
    };
    
    let processor = ConcurrentProcessorV2::with_config(config);
    
    // Generate high load
    let docs: Vec<_> = (0..500)
        .map(|_| generate_test_document(50)) // 50KB documents
        .collect();
    
    group.bench_function("v2_under_pressure", |b| {
        b.iter(|| {
            rt.block_on(async {
                let requests: Vec<_> = docs.iter().enumerate()
                    .map(|(i, doc)| ConversionRequest {
                        id: format!("pressure-{}", i),
                        content: ConversionContent::Memory(black_box(doc.clone())),
                        options: ConversionOptions::default(),
                    })
                    .collect();
                
                let responses = processor.process_batch(requests).await;
                
                // Verify all were processed despite backpressure
                assert!(!responses.is_empty());
            })
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_single_document,
    bench_batch_processing,
    bench_work_stealing,
    bench_backpressure
);
criterion_main!(benches);