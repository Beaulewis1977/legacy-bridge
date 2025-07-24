// Simple test to verify MD→RTF conversion works

use legacybridge::pipeline::{convert_markdown_to_rtf_with_pipeline, PipelineConfig};

fn main() {
    println!("Testing Markdown to RTF conversion...\n");
    
    let markdown_content = r#"
# Test Document

This is a **bold** and *italic* test.

## Features

- Bullet point 1
- Bullet point 2
  - Nested item

### Code Example

Here's some inline `code` text.

| Column 1 | Column 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |
"#;

    let config = PipelineConfig::default();
    
    match convert_markdown_to_rtf_with_pipeline(markdown_content, Some(config)) {
        Ok((rtf_content, context)) => {
            println!("✓ Conversion successful!");
            println!("\nGenerated RTF ({} bytes):\n", rtf_content.len());
            println!("{}", &rtf_content[..rtf_content.len().min(500)]);
            
            println!("\n\nValidation results: {}", context.validation_results.len());
            for result in &context.validation_results {
                println!("  - [{:?}] {}: {}", result.level, result.code, result.message);
            }
            
            println!("\nRecovery actions: {}", context.recovery_actions.len());
            for action in &context.recovery_actions {
                println!("  - [{:?}] {}", action.action_type, action.description);
            }
        }
        Err(e) => {
            println!("✗ Conversion failed: {}", e);
        }
    }
}