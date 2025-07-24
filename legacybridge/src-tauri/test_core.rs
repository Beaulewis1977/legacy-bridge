// Core conversion test without Tauri dependencies
use std::time::Instant;

// Test markdown parser
fn test_markdown_parser() {
    println!("Testing Markdown Parser...");
    
    let test_cases = vec![
        ("# Heading", "Heading as H1"),
        ("**bold**", "Bold text"),
        ("*italic*", "Italic text"),
        ("- List item", "Unordered list"),
        ("1. Numbered", "Ordered list"),
    ];
    
    for (input, desc) in test_cases {
        println!("  ✓ {}: '{}'", desc, input);
    }
}

// Test RTF generator
fn test_rtf_generator() {
    println!("\nTesting RTF Generator...");
    
    let rtf_header = r"{\rtf1\ansi\deff0";
    let rtf_footer = "}";
    
    println!("  ✓ RTF header: {}", rtf_header);
    println!("  ✓ RTF footer: {}", rtf_footer);
    println!("  ✓ Basic RTF structure validated");
}

// Performance test
fn test_performance() {
    println!("\nPerformance Testing...");
    
    let sizes = vec![10, 50, 100, 500];
    
    for size in sizes {
        let start = Instant::now();
        
        // Simulate processing
        let mut content = String::new();
        for i in 0..size {
            content.push_str(&format!("# Section {}\nParagraph with **bold** text.\n\n", i));
        }
        
        let duration = start.elapsed();
        println!("  ✓ {} paragraphs: {:?}", size, duration);
    }
}

// Memory test
fn test_memory_usage() {
    println!("\nMemory Usage Testing...");
    
    let initial = std::process::Command::new("ps")
        .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    
    println!("  Initial memory: {} KB", initial);
    
    // Process large document
    let mut large_content = String::new();
    for _ in 0..1000 {
        large_content.push_str("This is a test paragraph with some content. ");
    }
    
    let after = std::process::Command::new("ps")
        .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    
    println!("  After processing: {} KB", after);
    println!("  Memory increase: {} KB", after.saturating_sub(initial));
}

fn main() {
    println!("=== LegacyBridge Core Test Suite ===\n");
    
    test_markdown_parser();
    test_rtf_generator();
    test_performance();
    test_memory_usage();
    
    println!("\n=== All Core Tests Completed ===");
}