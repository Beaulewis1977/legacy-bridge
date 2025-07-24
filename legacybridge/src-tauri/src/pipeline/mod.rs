// Document Processing Pipeline - Main orchestrator
// 
// Pipeline flow:
// RTF Documents → Parser → Formatting Engine → Markdown Generator
//                     ↓
//            Template System → Validation Layer → Error Recovery → Output
//                     ↓
//         Legacy Integration → VB6/VFP9 Function Calls → Enterprise Systems

pub mod formatting_engine;
pub mod formatting_engine_optimized;
pub mod validation_layer;
pub mod error_recovery;
pub mod template_system;
pub mod concurrent_processor;

#[cfg(test)]
mod test_pipeline;

use crate::conversion::types::{ConversionError, ConversionResult, RtfDocument, RtfToken};
use crate::conversion::{rtf_lexer, RtfParser, SecureRtfParser, MarkdownGenerator, MarkdownParser, RtfGenerator, InputValidator};

/// Pipeline configuration options
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Enable strict validation mode
    pub strict_validation: bool,
    /// Enable automatic error recovery
    pub auto_recovery: bool,
    /// Template to use (if any)
    pub template: Option<String>,
    /// Preserve all RTF formatting
    pub preserve_formatting: bool,
    /// Enable legacy compatibility mode
    pub legacy_mode: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            strict_validation: true,
            auto_recovery: true,
            template: None,
            preserve_formatting: true,
            legacy_mode: false,
        }
    }
}

/// Pipeline execution context for RTF → MD
pub struct PipelineContext {
    /// Original RTF content
    pub rtf_content: String,
    /// Parsed tokens
    pub tokens: Option<Vec<RtfToken>>,
    /// Parsed document
    pub document: Option<RtfDocument>,
    /// Validation results
    pub validation_results: Vec<ValidationResult>,
    /// Error recovery actions taken
    pub recovery_actions: Vec<RecoveryAction>,
    /// Generated markdown
    pub markdown: Option<String>,
}

/// Pipeline execution context for MD → RTF
pub struct MarkdownPipelineContext {
    /// Original Markdown content
    pub markdown_content: String,
    /// Parsed document
    pub document: Option<RtfDocument>,
    /// Validation results
    pub validation_results: Vec<ValidationResult>,
    /// Error recovery actions taken
    pub recovery_actions: Vec<RecoveryAction>,
    /// Generated RTF
    pub rtf: Option<String>,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub level: ValidationLevel,
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationLevel {
    Info,
    Warning,
    Error,
}

/// Recovery action taken by error recovery system
#[derive(Debug, Clone)]
pub struct RecoveryAction {
    pub action_type: RecoveryType,
    pub description: String,
    pub applied: bool,
}

#[derive(Debug, Clone)]
pub enum RecoveryType {
    StructureRepair,
    EncodingFix,
    FormatCorrection,
    MissingDataInsertion,
    RemoveInvalid,
}

/// Main Document Processing Pipeline
pub struct DocumentPipeline {
    config: PipelineConfig,
}

impl DocumentPipeline {
    /// Create a new pipeline with default configuration
    pub fn new() -> Self {
        Self {
            config: PipelineConfig::default(),
        }
    }

    /// Create a new pipeline with custom configuration
    pub fn with_config(config: PipelineConfig) -> Self {
        Self { config }
    }

    /// Process RTF content through the pipeline
    pub fn process(&self, rtf_content: &str) -> ConversionResult<PipelineContext> {
        // SECURITY: Validate input size first (10MB limit)
        let validator = InputValidator::new();
        validator.validate_size(rtf_content, "RTF content")?;
        
        let mut context = PipelineContext {
            rtf_content: rtf_content.to_string(),
            tokens: None,
            document: None,
            validation_results: Vec::new(),
            recovery_actions: Vec::new(),
            markdown: None,
        };

        // Stage 1: Pre-validation (including security validation)
        if self.config.strict_validation {
            // Use InputValidator for comprehensive security checks
            validator.pre_validate_rtf(rtf_content)?;
            self.pre_validate(&mut context)?;
        }

        // Stage 2: Tokenization with error recovery
        let tokens = match rtf_lexer::tokenize(rtf_content) {
            Ok(tokens) => tokens,
            Err(e) => {
                if self.config.auto_recovery {
                    self.recover_tokenization(&mut context, e)?
                } else {
                    return Err(e);
                }
            }
        };
        context.tokens = Some(tokens);

        // Stage 3: Parsing with formatting preservation
        let document = match self.parse_with_formatting(&context) {
            Ok(doc) => doc,
            Err(e) => {
                if self.config.auto_recovery {
                    self.recover_parsing(&mut context, e)?
                } else {
                    return Err(e);
                }
            }
        };
        context.document = Some(document);

        // Stage 4: Template application (if configured)
        if let Some(template_name) = &self.config.template {
            self.apply_template(&mut context, template_name)?;
        }

        // Stage 5: Post-validation
        if self.config.strict_validation {
            self.post_validate(&mut context)?;
        }

        // Stage 6: Markdown generation
        let markdown = self.generate_markdown(&context)?;
        context.markdown = Some(markdown);

        Ok(context)
    }

    /// Process Markdown content through the pipeline to generate RTF
    pub fn process_markdown(&self, markdown_content: &str) -> ConversionResult<MarkdownPipelineContext> {
        // SECURITY: Validate input size first (10MB limit)
        let validator = InputValidator::new();
        validator.validate_size(markdown_content, "Markdown content")?;
        
        let mut context = MarkdownPipelineContext {
            markdown_content: markdown_content.to_string(),
            document: None,
            validation_results: Vec::new(),
            recovery_actions: Vec::new(),
            rtf: None,
        };

        // Stage 1: Pre-validation (including security validation)
        if self.config.strict_validation {
            // Use InputValidator for comprehensive security checks
            validator.pre_validate_markdown(markdown_content)?;
            self.pre_validate_markdown(&mut context)?;
        }

        // Stage 2: Markdown parsing
        let document = match MarkdownParser::parse(markdown_content) {
            Ok(doc) => doc,
            Err(e) => {
                if self.config.auto_recovery {
                    self.recover_markdown_parsing(&mut context, e)?
                } else {
                    return Err(e);
                }
            }
        };
        context.document = Some(document);

        // Stage 3: Template application (if configured)
        if let Some(template_name) = &self.config.template {
            self.apply_template_markdown(&mut context, template_name)?;
        }

        // Stage 4: Post-validation
        if self.config.strict_validation {
            self.post_validate_markdown(&mut context)?;
        }

        // Stage 5: RTF generation
        let rtf = self.generate_rtf(&context)?;
        context.rtf = Some(rtf);

        Ok(context)
    }

    /// Pre-validation stage
    fn pre_validate(&self, context: &mut PipelineContext) -> ConversionResult<()> {
        let validator = validation_layer::Validator::new();
        let results = validator.pre_validate(&context.rtf_content);
        
        for result in &results {
            if result.level == ValidationLevel::Error && !self.config.auto_recovery {
                return Err(ConversionError::ValidationError(result.message.clone()));
            }
        }
        
        context.validation_results.extend(results);
        Ok(())
    }

    /// Parse with formatting preservation
    fn parse_with_formatting(&self, context: &PipelineContext) -> ConversionResult<RtfDocument> {
        let tokens = context.tokens.as_ref().unwrap();
        
        // SECURITY: Use SecureRtfParser by default for all parsing
        if self.config.preserve_formatting {
            let formatter = formatting_engine::FormattingEngine::new();
            // Note: FormattingEngine should also use SecureRtfParser internally
            formatter.parse_with_fidelity(tokens.clone())
        } else {
            SecureRtfParser::parse(tokens.clone())
        }
    }

    /// Recover from tokenization errors
    fn recover_tokenization(
        &self,
        context: &mut PipelineContext,
        error: ConversionError,
    ) -> ConversionResult<Vec<RtfToken>> {
        let recovery = error_recovery::ErrorRecovery::new();
        let (tokens, actions) = recovery.recover_tokenization(&context.rtf_content, error)?;
        
        context.recovery_actions.extend(actions);
        Ok(tokens)
    }

    /// Recover from parsing errors
    fn recover_parsing(
        &self,
        context: &mut PipelineContext,
        error: ConversionError,
    ) -> ConversionResult<RtfDocument> {
        let recovery = error_recovery::ErrorRecovery::new();
        let tokens = context.tokens.as_ref().unwrap();
        let (document, actions) = recovery.recover_parsing(tokens, error)?;
        
        context.recovery_actions.extend(actions);
        Ok(document)
    }

    /// Apply template to document
    fn apply_template(
        &self,
        context: &mut PipelineContext,
        template_name: &str,
    ) -> ConversionResult<()> {
        let template_system = template_system::TemplateSystem::new();
        let document = context.document.as_mut().unwrap();
        
        template_system.apply_template(document, template_name)?;
        Ok(())
    }

    /// Post-validation stage
    fn post_validate(&self, context: &mut PipelineContext) -> ConversionResult<()> {
        let validator = validation_layer::Validator::new();
        let document = context.document.as_ref().unwrap();
        let results = validator.post_validate(document);
        
        for result in &results {
            if result.level == ValidationLevel::Error {
                return Err(ConversionError::ValidationError(result.message.clone()));
            }
        }
        
        context.validation_results.extend(results);
        Ok(())
    }

    /// Generate markdown with formatting preservation
    fn generate_markdown(&self, context: &PipelineContext) -> ConversionResult<String> {
        let document = context.document.as_ref().unwrap();
        
        if self.config.preserve_formatting {
            let formatter = formatting_engine::FormattingEngine::new();
            formatter.generate_markdown_with_fidelity(document)
        } else {
            MarkdownGenerator::generate(document)
        }
    }

    /// Pre-validation stage for Markdown
    fn pre_validate_markdown(&self, context: &mut MarkdownPipelineContext) -> ConversionResult<()> {
        let _validator = validation_layer::Validator::new();
        // For now, basic validation - check if content is not empty
        if context.markdown_content.trim().is_empty() {
            let result = ValidationResult {
                level: ValidationLevel::Error,
                code: "EMPTY_CONTENT".to_string(),
                message: "Markdown content is empty".to_string(),
                location: None,
            };
            context.validation_results.push(result);
            return Err(ConversionError::ValidationError("Empty Markdown content".to_string()));
        }
        Ok(())
    }

    /// Recover from markdown parsing errors
    fn recover_markdown_parsing(
        &self,
        context: &mut MarkdownPipelineContext,
        error: ConversionError,
    ) -> ConversionResult<RtfDocument> {
        let recovery = error_recovery::ErrorRecovery::new();
        // For now, create a simple recovery document
        let action = RecoveryAction {
            action_type: RecoveryType::StructureRepair,
            description: format!("Created fallback document due to parsing error: {}", error),
            applied: true,
        };
        context.recovery_actions.push(action);

        // Create a simple document with the raw markdown as text
        use crate::conversion::types::{DocumentMetadata, RtfNode};
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text(context.markdown_content.clone())])],
        };
        Ok(document)
    }

    /// Apply template to markdown document
    fn apply_template_markdown(
        &self,
        context: &mut MarkdownPipelineContext,
        template_name: &str,
    ) -> ConversionResult<()> {
        let template_system = template_system::TemplateSystem::new();
        let document = context.document.as_mut().unwrap();
        
        template_system.apply_template(document, template_name)?;
        Ok(())
    }

    /// Post-validation stage for markdown
    fn post_validate_markdown(&self, context: &mut MarkdownPipelineContext) -> ConversionResult<()> {
        let validator = validation_layer::Validator::new();
        let document = context.document.as_ref().unwrap();
        let results = validator.post_validate(document);
        
        for result in &results {
            if result.level == ValidationLevel::Error {
                return Err(ConversionError::ValidationError(result.message.clone()));
            }
        }
        
        context.validation_results.extend(results);
        Ok(())
    }

    /// Generate RTF with formatting preservation
    fn generate_rtf(&self, context: &MarkdownPipelineContext) -> ConversionResult<String> {
        let document = context.document.as_ref().unwrap();
        
        if let Some(template_name) = &self.config.template {
            RtfGenerator::generate_with_template(document, Some(template_name))
        } else if self.config.preserve_formatting {
            // Use standard generation with full formatting
            RtfGenerator::generate(document)
        } else {
            // Use minimal generation for simple documents
            RtfGenerator::generate_with_template(document, Some("minimal"))
        }
    }
}

/// Public API for pipeline conversion
pub fn convert_rtf_to_markdown_with_pipeline(
    rtf_content: &str,
    config: Option<PipelineConfig>,
) -> ConversionResult<(String, PipelineContext)> {
    let pipeline = match config {
        Some(cfg) => DocumentPipeline::with_config(cfg),
        None => DocumentPipeline::new(),
    };
    
    let context = pipeline.process(rtf_content)?;
    let markdown = context.markdown.as_ref().unwrap().clone();
    
    Ok((markdown, context))
}

/// Public API for MD→RTF pipeline conversion
pub fn convert_markdown_to_rtf_with_pipeline(
    markdown_content: &str,
    config: Option<PipelineConfig>,
) -> ConversionResult<(String, MarkdownPipelineContext)> {
    let pipeline = match config {
        Some(cfg) => DocumentPipeline::with_config(cfg),
        None => DocumentPipeline::new(),
    };
    
    let context = pipeline.process_markdown(markdown_content)?;
    let rtf = context.rtf.as_ref().unwrap().clone();
    
    Ok((rtf, context))
}

// Add ValidationError variant to ConversionError if not already present
impl From<String> for ConversionError {
    fn from(msg: String) -> Self {
        ConversionError::ParseError(msg)
    }
}