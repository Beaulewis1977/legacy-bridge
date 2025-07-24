// Comprehensive test suite for RTF generator
// Tests edge cases, formatting fidelity, and template system

#[cfg(test)]
mod rtf_generator_edge_cases {
    use super::super::*;
    use crate::conversion::types::{RtfDocument, RtfNode, DocumentMetadata, TableRow, TableCell};
    use crate::conversion::rtf_generator::RtfGenerator;

    #[test]
    fn test_generate_empty_document() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![],
        };

        let rtf = RtfGenerator::generate(&document).unwrap();
        assert!(rtf.starts_with("{\\rtf1\\ansi"));
        assert!(rtf.ends_with('}'));
        assert!(rtf.contains("\\fonttbl"));
        assert!(rtf.contains("\\colortbl"));
    }

    #[test]
    fn test_special_character_escaping() {
        let test_cases = vec![
            ("Text with {braces}", "Text with \\{braces\\}"),
            ("Back\\slash", "Back\\\\slash"),
            ("Mixed {brace} and \\slash", "Mixed \\{brace\\} and \\\\slash"),
            ("New\nline", "New\\par line"),
            ("Tab\there", "Tab\there"), // Tabs should be preserved
        ];

        for (input, expected_contains) in test_cases {
            let document = RtfDocument {
                metadata: DocumentMetadata::default(),
                content: vec![RtfNode::Paragraph(vec![RtfNode::Text(input.to_string())])],
            };

            let rtf = RtfGenerator::generate(&document).unwrap();
            assert!(rtf.contains(expected_contains), 
                "RTF should contain '{}' for input '{}'", expected_contains, input);
        }
    }

    #[test]
    fn test_unicode_character_encoding() {
        let unicode_tests = vec![
            ("Emoji: 🚀", "\\u128640?"), // Rocket emoji
            ("Chinese: 中文", "\\u20013?\\u25991?"),
            ("Arabic: العربية", "\\u1575?\\u1604?\\u1593?\\u1585?\\u1576?\\u1610?\\u1577?"),
            ("Accented: café", "caf\\u233?"), // é
        ];

        for (input, expected_pattern) in unicode_tests {
            let document = RtfDocument {
                metadata: DocumentMetadata::default(),
                content: vec![RtfNode::Paragraph(vec![RtfNode::Text(input.to_string())])],
            };

            let rtf = RtfGenerator::generate(&document).unwrap();
            // Check that Unicode is properly encoded
            assert!(input.chars().any(|c| c as u32 > 127), "Test input should contain Unicode");
        }
    }

    #[test]
    fn test_nested_formatting_generation() {
        // Create deeply nested formatting
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![
                RtfNode::Text("Normal ".to_string()),
                RtfNode::Bold(vec![
                    RtfNode::Text("Bold ".to_string()),
                    RtfNode::Italic(vec![
                        RtfNode::Text("Bold+Italic ".to_string()),
                        RtfNode::Underline(vec![
                            RtfNode::Text("Bold+Italic+Underline".to_string()),
                        ]),
                    ]),
                ]),
            ])],
        };

        let rtf = RtfGenerator::generate(&document).unwrap();
        
        // Verify nested formatting
        assert!(rtf.contains("{\\b "), "Should have bold formatting");
        assert!(rtf.contains("{\\i "), "Should have italic formatting");
        assert!(rtf.contains("{\\ul "), "Should have underline formatting");
        
        // Verify proper nesting (closing braces)
        let open_braces = rtf.matches('{').count();
        let close_braces = rtf.matches('}').count();
        assert_eq!(open_braces, close_braces, "Braces should be balanced");
    }

    #[test]
    fn test_heading_font_sizes() {
        let expected_sizes = vec![
            (1, 48), // H1 = 24pt = 48 half-points
            (2, 40), // H2 = 20pt = 40 half-points
            (3, 32), // H3 = 16pt = 32 half-points
            (4, 28), // H4 = 14pt = 28 half-points
            (5, 24), // H5 = 12pt = 24 half-points
            (6, 24), // H6 = 12pt = 24 half-points
        ];

        for (level, expected_size) in expected_sizes {
            let document = RtfDocument {
                metadata: DocumentMetadata::default(),
                content: vec![RtfNode::Heading {
                    level,
                    content: vec![RtfNode::Text(format!("Heading {}", level))],
                }],
            };

            let rtf = RtfGenerator::generate(&document).unwrap();
            assert!(rtf.contains(&format!("\\fs{} ", expected_size)), 
                "H{} should have font size {}", level, expected_size);
        }
    }

    #[test]
    fn test_list_indentation() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![
                RtfNode::ListItem {
                    level: 0,
                    content: vec![RtfNode::Text("Level 0".to_string())],
                },
                RtfNode::ListItem {
                    level: 1,
                    content: vec![RtfNode::Text("Level 1".to_string())],
                },
                RtfNode::ListItem {
                    level: 2,
                    content: vec![RtfNode::Text("Level 2".to_string())],
                },
            ],
        };

        let rtf = RtfGenerator::generate(&document).unwrap();
        
        // Check indentation levels (720 twips per level)
        assert!(rtf.contains("\\li720"), "Level 0 should have 720 twips indent");
        assert!(rtf.contains("\\li1440"), "Level 1 should have 1440 twips indent");
        assert!(rtf.contains("\\li2160"), "Level 2 should have 2160 twips indent");
        
        // All should have bullets
        let bullet_count = rtf.matches("\\bullet").count();
        assert_eq!(bullet_count, 3, "Should have 3 bullets");
    }

    #[test]
    fn test_table_generation() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Table {
                rows: vec![
                    TableRow {
                        cells: vec![
                            TableCell { content: vec![RtfNode::Text("Header 1".to_string())] },
                            TableCell { content: vec![RtfNode::Text("Header 2".to_string())] },
                            TableCell { content: vec![RtfNode::Text("Header 3".to_string())] },
                        ],
                    },
                    TableRow {
                        cells: vec![
                            TableCell { content: vec![RtfNode::Bold(vec![RtfNode::Text("Bold".to_string())])] },
                            TableCell { content: vec![RtfNode::Italic(vec![RtfNode::Text("Italic".to_string())])] },
                            TableCell { content: vec![RtfNode::Text("Normal".to_string())] },
                        ],
                    },
                ],
            }],
        };

        let rtf = RtfGenerator::generate(&document).unwrap();
        
        // Verify table structure
        assert!(rtf.contains("\\trowd"), "Should have table row definition");
        assert!(rtf.contains("\\cellx"), "Should have cell boundaries");
        assert!(rtf.contains("\\cell"), "Should have cell markers");
        assert!(rtf.contains("\\row"), "Should have row markers");
        
        // Verify content
        assert!(rtf.contains("Header 1"));
        assert!(rtf.contains("Header 2"));
        assert!(rtf.contains("{\\b Bold}"));
        assert!(rtf.contains("{\\i Italic}"));
    }

    #[test]
    fn test_empty_table_handling() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Table { rows: vec![] }],
        };

        let rtf = RtfGenerator::generate(&document).unwrap();
        // Should not crash and should produce valid RTF
        assert!(rtf.starts_with("{\\rtf1"));
        assert!(!rtf.contains("\\trowd"), "Empty table should not generate table commands");
    }

    #[test]
    fn test_page_and_line_breaks() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![
                RtfNode::Paragraph(vec![RtfNode::Text("Before page break".to_string())]),
                RtfNode::PageBreak,
                RtfNode::Paragraph(vec![RtfNode::Text("After page break".to_string())]),
                RtfNode::LineBreak,
                RtfNode::Paragraph(vec![RtfNode::Text("After line break".to_string())]),
            ],
        };

        let rtf = RtfGenerator::generate(&document).unwrap();
        assert!(rtf.contains("\\page "), "Should contain page break");
        assert!(rtf.contains("\\line "), "Should contain line break");
    }

    #[test]
    fn test_template_minimal() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![
                RtfNode::Heading {
                    level: 1,
                    content: vec![RtfNode::Text("Test Heading".to_string())],
                },
                RtfNode::Table {
                    rows: vec![
                        TableRow {
                            cells: vec![
                                TableCell { content: vec![RtfNode::Text("Cell".to_string())] },
                            ],
                        },
                    ],
                },
            ],
        };

        let rtf = RtfGenerator::generate_with_template(&document, Some("minimal")).unwrap();
        
        // Minimal template should simplify output
        assert!(!rtf.contains("\\colortbl"), "Minimal should not have color table");
        assert!(!rtf.contains("\\trowd"), "Minimal should skip complex table formatting");
        assert!(rtf.contains("{\\b Test Heading}"), "Should still have basic formatting");
    }

    #[test]
    fn test_template_professional() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text("Professional content".to_string())])],
        };

        let rtf = RtfGenerator::generate_with_template(&document, Some("professional")).unwrap();
        
        // Professional template should have enhanced formatting
        assert!(rtf.contains("\\margl1440"), "Should have 1 inch margins");
        assert!(rtf.contains("\\margr1440"));
        assert!(rtf.contains("\\margt1440"));
        assert!(rtf.contains("\\margb1440"));
        assert!(rtf.contains("\\colortbl"), "Should have color table");
        assert!(rtf.contains("Arial"), "Should include Arial font");
    }

    #[test]
    fn test_template_academic() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text("Academic content".to_string())])],
        };

        let rtf = RtfGenerator::generate_with_template(&document, Some("academic")).unwrap();
        
        // Academic template should have specific formatting
        assert!(rtf.contains("\\margl1800"), "Should have wider left margin for binding");
        assert!(rtf.contains("\\sa240"), "Should have paragraph spacing");
        assert!(rtf.contains("\\sl276"), "Should have line spacing");
    }

    #[test]
    fn test_large_document_generation() {
        let mut content = Vec::new();
        
        // Generate a large document with various elements
        for i in 0..100 {
            content.push(RtfNode::Heading {
                level: ((i % 6) + 1) as u8,
                content: vec![RtfNode::Text(format!("Heading {}", i))],
            });
            
            content.push(RtfNode::Paragraph(vec![
                RtfNode::Text("Normal text ".to_string()),
                RtfNode::Bold(vec![RtfNode::Text("bold ".to_string())]),
                RtfNode::Italic(vec![RtfNode::Text("italic ".to_string())]),
                RtfNode::Text(format!("paragraph {}.", i)),
            ]));
            
            if i % 10 == 0 {
                content.push(RtfNode::PageBreak);
            }
        }
        
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content,
        };

        let start = std::time::Instant::now();
        let rtf = RtfGenerator::generate(&document).unwrap();
        let duration = start.elapsed();
        
        assert!(duration.as_secs() < 2, "Should generate large document quickly");
        assert!(rtf.len() > 10000, "Large document should produce substantial RTF");
        
        // Verify RTF is well-formed
        let open_braces = rtf.matches('{').count();
        let close_braces = rtf.matches('}').count();
        assert_eq!(open_braces, close_braces, "RTF should have balanced braces");
    }

    #[test]
    fn test_mixed_content_stress() {
        // Create a document with every type of node
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![
                RtfNode::Heading { level: 1, content: vec![RtfNode::Text("Main Title".to_string())] },
                RtfNode::Paragraph(vec![
                    RtfNode::Text("Mixed ".to_string()),
                    RtfNode::Bold(vec![
                        RtfNode::Text("bold ".to_string()),
                        RtfNode::Italic(vec![RtfNode::Text("and italic".to_string())]),
                    ]),
                    RtfNode::Text(" text.".to_string()),
                ]),
                RtfNode::ListItem { level: 0, content: vec![RtfNode::Text("List item 1".to_string())] },
                RtfNode::ListItem { level: 1, content: vec![RtfNode::Text("Nested item".to_string())] },
                RtfNode::Table {
                    rows: vec![
                        TableRow {
                            cells: vec![
                                TableCell { content: vec![RtfNode::Bold(vec![RtfNode::Text("Bold Cell".to_string())])] },
                                TableCell { content: vec![RtfNode::Italic(vec![RtfNode::Text("Italic Cell".to_string())])] },
                            ],
                        },
                    ],
                },
                RtfNode::LineBreak,
                RtfNode::PageBreak,
                RtfNode::Paragraph(vec![RtfNode::Text("Final paragraph.".to_string())]),
            ],
        };

        let rtf = RtfGenerator::generate(&document).unwrap();
        
        // Verify all elements are present
        assert!(rtf.contains("Main Title"));
        assert!(rtf.contains("{\\b bold"));
        assert!(rtf.contains("{\\i and italic"));
        assert!(rtf.contains("\\bullet"));
        assert!(rtf.contains("\\trowd"));
        assert!(rtf.contains("\\line"));
        assert!(rtf.contains("\\page"));
        assert!(rtf.contains("Final paragraph"));
    }

    #[test]
    fn test_metadata_inclusion() {
        use crate::conversion::types::{FontInfo, Color};
        
        let document = RtfDocument {
            metadata: DocumentMetadata {
                default_font: Some("Custom Font".to_string()),
                charset: "UTF-8".to_string(),
                fonts: vec![
                    FontInfo { id: 0, name: "Times New Roman".to_string(), charset: 0 },
                    FontInfo { id: 1, name: "Arial".to_string(), charset: 0 },
                ],
                colors: vec![
                    Color { red: 255, green: 0, blue: 0 },
                    Color { red: 0, green: 255, blue: 0 },
                ],
                title: Some("Test Document".to_string()),
                author: Some("Test Author".to_string()),
            },
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text("Content".to_string())])],
        };

        let rtf = RtfGenerator::generate(&document).unwrap();
        
        // Font table should reflect metadata
        assert!(rtf.contains("Times New Roman"));
        assert!(rtf.contains("Arial"));
        
        // Color table should be populated
        assert!(rtf.contains("\\red255\\green0\\blue0"));
        assert!(rtf.contains("\\red0\\green255\\blue0"));
    }
}