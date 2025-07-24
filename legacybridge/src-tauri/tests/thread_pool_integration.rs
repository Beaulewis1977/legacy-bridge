// Integration test demonstrating thread pool performance improvements
//
// This test compares:
// 1. Original concurrent processor (rayon-based)
// 2. Enhanced concurrent processor with adaptive thread pool

use legacybridge::pipeline::concurrent_processor::{
    ConcurrentProcessor, ConversionRequest, ConversionContent, ConversionOptions
};
use legacybridge::pipeline::concurrent_processor_v2::ConcurrentProcessorV2;
use std::time::Instant;
use tokio::runtime::Runtime;

/// Generate realistic test documents
fn generate_test_documents(count: usize, size_kb: usize) -> Vec<String> {
    (0..count).map(|i| {
        format!(
            "# Document {}\n\n{}",
            i,
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(size_kb * 10)
        )
    }).collect()
}

#[test]
fn test_throughput_improvement() {
    let rt = Runtime::new().unwrap();
    let doc_count = 100;
    let documents = generate_test_documents(doc_count, 10); // 10KB documents

    // Test v1 processor
    let processor_v1 = ConcurrentProcessor::new(None);
    let start_v1 = Instant::now();
    
    rt.block_on(async {
        let requests: Vec<_> = documents.iter().enumerate()
            .map(|(i, doc)| ConversionRequest {
                id: format!("v1-{}", i),
                content: ConversionContent::Memory(doc.clone()),
                options: ConversionOptions::default(),
            })
            .collect();
        
        let responses = processor_v1.process_batch(requests).await;
        assert_eq!(responses.len(), doc_count);
    });
    
    let duration_v1 = start_v1.elapsed();
    println!("V1 Processor: {} documents in {:?}", doc_count, duration_v1);

    // Test v2 processor
    let processor_v2 = ConcurrentProcessorV2::new();
    let start_v2 = Instant::now();
    
    rt.block_on(async {
        let requests: Vec<_> = documents.iter().enumerate()
            .map(|(i, doc)| ConversionRequest {
                id: format!("v2-{}", i),
                content: ConversionContent::Memory(doc.clone()),
                options: ConversionOptions::default(),
            })
            .collect();
        
        let responses = processor_v2.process_batch(requests).await;
        assert_eq!(responses.len(), doc_count);
    });
    
    let duration_v2 = start_v2.elapsed();
    println!("V2 Processor: {} documents in {:?}", doc_count, duration_v2);

    // Calculate improvement
    let improvement = duration_v1.as_secs_f64() / duration_v2.as_secs_f64();
    println!("Performance improvement: {:.2}x", improvement);

    // Get statistics
    let stats = processor_v2.get_pool_statistics();
    println!("Thread pool stats: {:?}", stats);

    // Assert at least some improvement
    assert!(improvement > 1.0, "V2 should be faster than V1");
}

#[test]
fn test_concurrent_user_handling() {
    let rt = Runtime::new().unwrap();
    let processor = ConcurrentProcessorV2::new();
    
    // Simulate 1000 concurrent users
    let users = 1000;
    let docs_per_user = 5;
    
    let start = Instant::now();
    
    rt.block_on(async {
        // Create all requests
        let mut all_requests = Vec::new();
        for user in 0..users {
            for doc in 0..docs_per_user {
                all_requests.push(ConversionRequest {
                    id: format!("user-{}-doc-{}", user, doc),
                    content: ConversionContent::Memory(
                        format!("User {} Document {}\n\nContent here.", user, doc)
                    ),
                    options: ConversionOptions::default(),
                });
            }
        }
        
        // Process all requests
        let responses = processor.process_batch(all_requests).await;
        
        // Verify all were processed
        assert_eq!(responses.len(), users * docs_per_user);
        
        // Count successful conversions
        let successful = responses.iter().filter(|r| r.result.is_ok()).count();
        println!("Successful conversions: {}/{}", successful, responses.len());
        
        // Most should succeed
        assert!(successful > responses.len() * 9 / 10);
    });
    
    let duration = start.elapsed();
    let total_docs = users * docs_per_user;
    let throughput = total_docs as f64 / duration.as_secs_f64();
    
    println!("Processed {} documents from {} users in {:?}", total_docs, users, duration);
    println!("Throughput: {:.2} docs/second", throughput);
    
    // Get final metrics
    let metrics = processor.get_metrics();
    println!("Processing metrics: {:?}", metrics);
    
    // Check backpressure handling
    if metrics.backpressure_events > 0 {
        println!("Handled {} backpressure events", metrics.backpressure_events);
    }
}

#[test]
fn test_work_stealing_efficiency() {
    let rt = Runtime::new().unwrap();
    let processor = ConcurrentProcessorV2::new();
    
    // Create mixed workload - some small, some large documents
    let mut requests = Vec::new();
    
    // 90 small documents (1KB)
    for i in 0..90 {
        requests.push(ConversionRequest {
            id: format!("small-{}", i),
            content: ConversionContent::Memory("Small content".repeat(50)),
            options: ConversionOptions::default(),
        });
    }
    
    // 10 large documents (100KB)
    for i in 0..10 {
        requests.push(ConversionRequest {
            id: format!("large-{}", i),
            content: ConversionContent::Memory("Large content ".repeat(5000)),
            options: ConversionOptions {
                parallel_chunks: true,
                chunk_size: 10 * 1024,
                ..Default::default()
            },
        });
    }
    
    let start = Instant::now();
    
    rt.block_on(async {
        let responses = processor.process_batch(requests).await;
        assert_eq!(responses.len(), 100);
    });
    
    let duration = start.elapsed();
    println!("Mixed workload processed in {:?}", duration);
    
    // Check work stealing statistics
    let stats = processor.get_pool_statistics();
    println!("Work stealing stats: {} tasks stolen", stats.tasks_stolen);
    
    // Should have some work stealing with mixed workload
    assert!(stats.tasks_stolen > 0, "Work stealing should occur with mixed workload");
}

#[test]
fn test_adaptive_scaling() {
    let rt = Runtime::new().unwrap();
    let processor = ConcurrentProcessorV2::new();
    
    // Get initial thread count
    let initial_stats = processor.get_pool_statistics();
    let initial_threads = initial_stats.total_threads;
    println!("Initial threads: {}", initial_threads);
    
    // Create high load
    let high_load_docs = 500;
    let start = Instant::now();
    
    rt.block_on(async {
        let requests: Vec<_> = (0..high_load_docs)
            .map(|i| ConversionRequest {
                id: format!("load-{}", i),
                content: ConversionContent::Memory("High load content".repeat(100)),
                options: ConversionOptions::default(),
            })
            .collect();
        
        processor.process_batch(requests).await;
    });
    
    // Check if thread pool scaled up
    let loaded_stats = processor.get_pool_statistics();
    println!("Under load - threads: {}, load factor: {:.2}", 
             loaded_stats.total_threads, loaded_stats.load_factor);
    
    // Wait for idle timeout
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    // Process minimal load
    rt.block_on(async {
        let request = ConversionRequest {
            id: "minimal".to_string(),
            content: ConversionContent::Memory("Minimal content".to_string()),
            options: ConversionOptions::default(),
        };
        
        processor.process_single(request).await;
    });
    
    let final_stats = processor.get_pool_statistics();
    println!("After idle - threads: {}", final_stats.total_threads);
    
    println!("Processed {} documents in {:?}", high_load_docs, start.elapsed());
}