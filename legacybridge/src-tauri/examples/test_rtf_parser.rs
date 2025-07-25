// Example program to test RTF parsing without Tauri dependencies

use legacybridge::conversion;

fn main() {
    // Test basic RTF to Markdown conversion
    let rtf_samples = vec![
        (r"{\rtf1 Hello World\par}", "Simple text"),
        (r"{\rtf1 Normal {\b Bold} text\par}", "Bold formatting"),
        (r"{\rtf1 Normal {\i Italic} text\par}", "Italic formatting"),
        (r"{\rtf1 First paragraph\par Second paragraph\par}", "Multiple paragraphs"),
        (r"{\rtf1 Text with\line line break\par}", "Line break"),
    ];

    println!("RTF Parser Test Results:");
    println!("========================\n");

    for (rtf, description) in rtf_samples {
        println!("Test: {}", description);
        println!("RTF: {}", rtf);
        
        match conversion::rtf_to_markdown(rtf) {
            Ok(markdown) => {
                println!("Markdown: {}", markdown);
                println!("✓ Success");
            }
            Err(e) => {
                println!("✗ Error: {}", e);
            }
        }
        println!("---\n");
    }
}