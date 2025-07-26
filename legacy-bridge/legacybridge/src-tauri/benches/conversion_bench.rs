use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use legacybridge::conversion::markdown_parser::MarkdownParser;
use std::fs;

// Generate test documents of various sizes
fn generate_markdown_document(size: usize) -> String {
    let mut doc = String::with_capacity(size * 100);
    
    // Add header
    doc.push_str("# Main Document Title\n\n");
    
    // Generate content based on size
    for i in 0..size {
        // Heading
        doc.push_str(&format!("## Section {}\n\n", i + 1));
        
        // Paragraph with formatting
        doc.push_str(&format!(
            "This is a paragraph with **bold text**, *italic text*, and `inline code`. \
            It also contains [a link](https://example.com) and some more text to make it longer.\n\n"
        ));
        
        // List
        doc.push_str("- First list item\n");
        doc.push_str("- Second list item with **formatting**\n");
        doc.push_str("  - Nested item\n");
        doc.push_str("- Third list item\n\n");
        
        // Table
        if i % 3 == 0 {
            doc.push_str("| Header 1 | Header 2 | Header 3 |\n");
            doc.push_str("|----------|----------|----------|\n");
            doc.push_str("| Cell 1   | Cell 2   | Cell 3   |\n");
            doc.push_str("| Cell 4   | Cell 5   | Cell 6   |\n\n");
        }
        
        // Code block
        if i % 5 == 0 {
            doc.push_str("```rust\n");
            doc.push_str("fn example() {\n");
            doc.push_str("    println!(\"Hello, world!\");\n");
            doc.push_str("}\n");
            doc.push_str("```\n\n");
        }
    }
    
    doc
}

fn benchmark_markdown_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown_parsing");
    
    // Test different document sizes
    for size in [10, 100, 500, 1000].iter() {
        let doc = generate_markdown_document(*size);
        let doc_size_kb = doc.len() / 1024;
        
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}KB", doc_size_kb)),
            &doc,
            |b, doc| {
                b.iter(|| {
                    let result = MarkdownParser::parse(black_box(doc));
                    black_box(result);
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_formatting_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("formatting_patterns");
    
    // Test heavy formatting
    let heavy_format = "This is **bold *and italic* text** with ***triple*** formatting ".repeat(100);
    group.bench_function("heavy_formatting", |b| {
        b.iter(|| {
            let result = MarkdownParser::parse(black_box(&heavy_format));
            black_box(result);
        });
    });
    
    // Test deep nesting
    let mut deep_nested = String::new();
    for i in 0..50 {
        deep_nested.push_str(&"  ".repeat(i));
        deep_nested.push_str(&format!("- Level {} item\n", i));
    }
    group.bench_function("deep_nesting", |b| {
        b.iter(|| {
            let result = MarkdownParser::parse(black_box(&deep_nested));
            black_box(result);
        });
    });
    
    // Test large tables
    let mut large_table = String::from("| ");
    for i in 0..20 {
        large_table.push_str(&format!("Col {} | ", i));
    }
    large_table.push_str("\n|");
    large_table.push_str(&"------|".repeat(20));
    large_table.push('\n');
    for _ in 0..100 {
        large_table.push_str("| ");
        for i in 0..20 {
            large_table.push_str(&format!("Cell {} | ", i));
        }
        large_table.push('\n');
    }
    
    group.bench_function("large_tables", |b| {
        b.iter(|| {
            let result = MarkdownParser::parse(black_box(&large_table));
            black_box(result);
        });
    });
    
    group.finish();
}

fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    
    // Test string concatenation patterns
    let base_text = "Sample text ".repeat(100);
    
    group.bench_function("frequent_small_allocs", |b| {
        b.iter(|| {
            let mut doc = String::new();
            for _ in 0..1000 {
                doc.push_str("**");
                doc.push_str(&base_text[0..10]);
                doc.push_str("** ");
            }
            let result = MarkdownParser::parse(black_box(&doc));
            black_box(result);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_markdown_parsing,
    benchmark_formatting_patterns,
    benchmark_memory_allocation
);
criterion_main!(benches);