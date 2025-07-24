// Thread Pool Performance Demonstration
//
// This example shows the performance improvements of the adaptive thread pool
// Run with: cargo run --example thread_pool_demo

use legacybridge::pipeline::concurrent_processor::{
    ConcurrentProcessor, ConversionRequest, ConversionContent, ConversionOptions
};
use legacybridge::pipeline::concurrent_processor_v2::{ConcurrentProcessorV2, ProcessorConfig};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

fn generate_documents(count: usize, size_kb: usize) -> Vec<String> {
    println!("Generating {} documents of {}KB each...", count, size_kb);
    (0..count).map(|i| {
        let content = format!(
            "# Document {}\n\n",
            i
        );
        let paragraph = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                        Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. ";
        let target_size = size_kb * 1024;
        let mut doc = content;
        while doc.len() < target_size {
            doc.push_str(paragraph);
            doc.push('\n');
        }
        doc
    }).collect()
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs_f64();
    if secs < 1.0 {
        format!("{:.2}ms", duration.as_millis())
    } else {
        format!("{:.2}s", secs)
    }
}

fn main() {
    println!("=== LegacyBridge Thread Pool Performance Demo ===\n");

    let rt = Runtime::new().unwrap();

    // Test configurations
    let test_sizes = vec![
        (100, 10),   // 100 docs × 10KB
        (500, 5),    // 500 docs × 5KB
        (1000, 2),   // 1000 docs × 2KB
    ];

    for (doc_count, size_kb) in test_sizes {
        println!("\n--- Testing {} documents of {}KB each ---", doc_count, size_kb);
        
        let documents = generate_documents(doc_count, size_kb);
        let total_size_mb = (doc_count * size_kb) as f64 / 1024.0;
        println!("Total data size: {:.2}MB", total_size_mb);

        // Test V1 Processor
        println!("\n[V1 Processor - Rayon-based]");
        let processor_v1 = ConcurrentProcessor::new(None);
        let start_v1 = Instant::now();
        
        let v1_responses = rt.block_on(async {
            let requests: Vec<_> = documents.iter().enumerate()
                .map(|(i, doc)| ConversionRequest {
                    id: format!("v1-{}", i),
                    content: ConversionContent::Memory(doc.clone()),
                    options: ConversionOptions::default(),
                })
                .collect();
            
            processor_v1.process_batch(requests).await
        });
        
        let duration_v1 = start_v1.elapsed();
        let throughput_v1 = doc_count as f64 / duration_v1.as_secs_f64();
        let mbps_v1 = total_size_mb / duration_v1.as_secs_f64();
        
        println!("  Time: {}", format_duration(duration_v1));
        println!("  Throughput: {:.2} docs/sec", throughput_v1);
        println!("  Speed: {:.2} MB/s", mbps_v1);
        println!("  Success rate: {}%", 
            v1_responses.iter().filter(|r| r.result.is_ok()).count() * 100 / doc_count);

        // Test V2 Processor
        println!("\n[V2 Processor - Adaptive Thread Pool]");
        let processor_v2 = ConcurrentProcessorV2::new();
        let start_v2 = Instant::now();
        
        let v2_responses = rt.block_on(async {
            let requests: Vec<_> = documents.iter().enumerate()
                .map(|(i, doc)| ConversionRequest {
                    id: format!("v2-{}", i),
                    content: ConversionContent::Memory(doc.clone()),
                    options: ConversionOptions::default(),
                })
                .collect();
            
            processor_v2.process_batch(requests).await
        });
        
        let duration_v2 = start_v2.elapsed();
        let throughput_v2 = doc_count as f64 / duration_v2.as_secs_f64();
        let mbps_v2 = total_size_mb / duration_v2.as_secs_f64();
        
        println!("  Time: {}", format_duration(duration_v2));
        println!("  Throughput: {:.2} docs/sec", throughput_v2);
        println!("  Speed: {:.2} MB/s", mbps_v2);
        println!("  Success rate: {}%", 
            v2_responses.iter().filter(|r| r.result.is_ok()).count() * 100 / doc_count);

        // Show improvement
        let improvement = duration_v1.as_secs_f64() / duration_v2.as_secs_f64();
        println!("\n[Performance Improvement]");
        println!("  Speed-up: {:.2}x faster", improvement);
        println!("  Time saved: {}", format_duration(duration_v1.saturating_sub(duration_v2)));

        // Show V2 statistics
        let stats = processor_v2.get_pool_statistics();
        let metrics = processor_v2.get_metrics();
        
        println!("\n[Thread Pool Statistics]");
        println!("  Total threads: {}", stats.total_threads);
        println!("  Tasks completed: {}", stats.tasks_completed);
        println!("  Tasks stolen: {} ({:.1}%)", 
            stats.tasks_stolen, 
            stats.tasks_stolen as f64 * 100.0 / stats.tasks_completed as f64);
        println!("  Average task time: {:.2}ms", stats.average_task_time_ms);
        println!("  Load factor: {:.2}", stats.load_factor);
        
        println!("\n[Processing Metrics]");
        println!("  Total processed: {}", metrics.total_processed);
        println!("  Average latency: {:.2}ms", metrics.average_latency_ms);
        println!("  P99 latency: {:.2}ms", metrics.p99_latency_ms);
        println!("  Error rate: {:.2}%", metrics.error_rate * 100.0);
        println!("  Memory pool hit rate: {:.1}%", 
            metrics.memory_pool_hits as f64 * 100.0 / 
            (metrics.memory_pool_hits + metrics.memory_pool_misses) as f64);
        
        if metrics.backpressure_events > 0 {
            println!("  Backpressure events: {}", metrics.backpressure_events);
        }
    }

    // Test concurrent users
    println!("\n\n--- Testing Concurrent User Handling ---");
    let users = 1000;
    let docs_per_user = 5;
    
    println!("Simulating {} concurrent users with {} documents each...", users, docs_per_user);
    
    let processor_v2 = ConcurrentProcessorV2::with_config(ProcessorConfig {
        max_concurrent_per_tenant: 100,
        ..Default::default()
    });
    
    let start = Instant::now();
    
    let responses = rt.block_on(async {
        let mut all_requests = Vec::new();
        for user in 0..users {
            for doc in 0..docs_per_user {
                all_requests.push(ConversionRequest {
                    id: format!("user-{}-doc-{}", user, doc),
                    content: ConversionContent::Memory(
                        format!("User {} Document {}\n\nShort content for testing concurrent access.", user, doc)
                    ),
                    options: ConversionOptions::default(),
                });
            }
        }
        
        processor_v2.process_batch(all_requests).await
    });
    
    let duration = start.elapsed();
    let total_docs = users * docs_per_user;
    let throughput = total_docs as f64 / duration.as_secs_f64();
    let successful = responses.iter().filter(|r| r.result.is_ok()).count();
    
    println!("\n[Results]");
    println!("  Total documents: {}", total_docs);
    println!("  Processing time: {}", format_duration(duration));
    println!("  Throughput: {:.2} docs/sec", throughput);
    println!("  Success rate: {:.2}%", successful as f64 * 100.0 / total_docs as f64);
    
    let stats = processor_v2.get_pool_statistics();
    println!("\n[Final Thread Pool State]");
    println!("  Active threads: {}/{}", stats.active_threads, stats.total_threads);
    println!("  Queue depth: {}", stats.queue_depth);
    println!("  Load factor: {:.2}", stats.load_factor);
    
    println!("\n=== Demo Complete ===");
}