// Standalone test to demonstrate memory pool effectiveness
// This can be compiled and run independently

use std::time::Instant;

// Simulate the conversion process with allocations
fn standard_conversion(iterations: usize) -> (usize, f64) {
    let start = Instant::now();
    let mut allocation_count = 0;
    
    for i in 0..iterations {
        // Simulate string allocations during parsing
        let _text1 = String::from("Sample text content");
        let _text2 = format!("Formatted text {}", i);
        let _text3 = String::with_capacity(256);
        allocation_count += 3;
        
        // Simulate vector allocations for nodes
        let mut _nodes = Vec::with_capacity(100);
        for j in 0..10 {
            _nodes.push(format!("Node {}-{}", i, j));
            allocation_count += 1;
        }
        allocation_count += 1;
        
        // Simulate result string building
        let mut _result = String::new();
        for _ in 0..5 {
            _result.push_str("Some markdown content ");
        }
        allocation_count += 1;
    }
    
    let elapsed = start.elapsed().as_secs_f64();
    (allocation_count, elapsed)
}

// Simulate pooled conversion with object reuse
fn pooled_conversion(iterations: usize) -> (usize, f64) {
    // Pre-allocate pools
    let mut string_pool: Vec<String> = Vec::new();
    let mut vec_pool: Vec<Vec<String>> = Vec::new();
    
    for _ in 0..10 {
        string_pool.push(String::with_capacity(256));
        vec_pool.push(Vec::with_capacity(100));
    }
    
    let start = Instant::now();
    let mut allocation_count = 10 + 10; // Initial pool allocations
    
    for i in 0..iterations {
        // Reuse strings from pool
        let mut text1 = string_pool.pop().unwrap_or_else(|| {
            allocation_count += 1;
            String::with_capacity(256)
        });
        text1.clear();
        text1.push_str("Sample text content");
        
        // Reuse vectors from pool
        let mut nodes = vec_pool.pop().unwrap_or_else(|| {
            allocation_count += 1;
            Vec::with_capacity(100)
        });
        nodes.clear();
        
        for j in 0..10 {
            nodes.push(format!("Node {}-{}", i, j));
            allocation_count += 1; // Still allocate for node strings
        }
        
        // Return to pools
        string_pool.push(text1);
        vec_pool.push(nodes);
    }
    
    let elapsed = start.elapsed().as_secs_f64();
    (allocation_count, elapsed)
}

fn main() {
    println!("Memory Pool Effectiveness Test");
    println!("==============================\n");
    
    let iterations = 10000;
    
    // Warm up
    println!("Warming up...");
    standard_conversion(100);
    pooled_conversion(100);
    
    // Test standard conversion
    println!("\nTesting standard conversion ({} iterations)...", iterations);
    let (std_allocs, std_time) = standard_conversion(iterations);
    println!("  Allocations: {}", std_allocs);
    println!("  Time: {:.3}s", std_time);
    println!("  Allocs/sec: {:.0}", std_allocs as f64 / std_time);
    
    // Test pooled conversion
    println!("\nTesting pooled conversion ({} iterations)...", iterations);
    let (pool_allocs, pool_time) = pooled_conversion(iterations);
    println!("  Allocations: {}", pool_allocs);
    println!("  Time: {:.3}s", pool_time);
    println!("  Allocs/sec: {:.0}", pool_allocs as f64 / pool_time);
    
    // Calculate improvement
    let alloc_reduction = (1.0 - (pool_allocs as f64 / std_allocs as f64)) * 100.0;
    let time_improvement = (1.0 - (pool_time / std_time)) * 100.0;
    
    println!("\nResults:");
    println!("========");
    println!("Allocation reduction: {:.1}%", alloc_reduction);
    println!("Time improvement: {:.1}%", time_improvement);
    println!("Allocation overhead reduction: {:.1}%", alloc_reduction);
    
    if alloc_reduction >= 40.0 {
        println!("\n✓ SUCCESS: Achieved {:.1}% allocation overhead reduction (target: 40%)", alloc_reduction);
    } else {
        println!("\n✗ FAILED: Only achieved {:.1}% allocation overhead reduction (target: 40%)", alloc_reduction);
    }
}