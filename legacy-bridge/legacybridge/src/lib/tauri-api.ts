import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { logger, startTimer } from './error-logger';

// Types for conversion results and errors
export interface ConversionResult {
  success: boolean;
  content?: string;
  error?: string;
  metadata?: {
    originalFormat: string;
    convertedFormat: string;
    timestamp: number;
  };
}

export interface ConversionError {
  code: string;
  message: string;
  details?: string;
}

export interface FileInfo {
  name: string;
  path: string;
  size: number;
  type: 'rtf' | 'md';
}

// Pipeline types
export interface PipelineConfig {
  strict_validation?: boolean;
  auto_recovery?: boolean;
  template?: string;
  preserve_formatting?: boolean;
  legacy_mode?: boolean;
}

export interface ValidationResult {
  level: string;
  code: string;
  message: string;
  location?: string;
}

export interface RecoveryAction {
  action_type: string;
  description: string;
  applied: boolean;
}

export interface PipelineConversionResult {
  success: boolean;
  markdown?: string;
  validation_results?: ValidationResult[];
  recovery_actions?: RecoveryAction[];
  error?: string;
}

// Streaming types
export interface StreamUpdateData {
  progress?: number;
  content?: string;
  validation?: ValidationResult[];
  error?: ConversionError;
  metadata?: Record<string, unknown>;
}

export interface StreamUpdate {
  type: 'progress' | 'partial' | 'validation' | 'complete' | 'error';
  data: StreamUpdateData;
  timestamp: number;
}

// Tauri command invocations
export const tauriApi = {
  // Convert RTF to Markdown
  async convertRtfToMarkdown(filePath: string): Promise<ConversionResult> {
    const endTimer = startTimer('convertRtfToMarkdown');
    logger.info('Conversion', 'Starting RTF to Markdown conversion', { filePath });
    
    try {
      const result = await invoke<string>('convert_rtf_to_markdown', {
        filePath
      });
      
      endTimer();
      logger.info('Conversion', 'RTF to Markdown conversion successful', { 
        filePath,
        resultLength: result.length 
      });
      
      return {
        success: true,
        content: result,
        metadata: {
          originalFormat: 'rtf',
          convertedFormat: 'md',
          timestamp: Date.now()
        }
      };
    } catch (error) {
      endTimer();
      logger.error('Conversion', 'RTF to Markdown conversion failed', error, { filePath });
      
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred'
      };
    }
  },

  // Convert Markdown to RTF
  async convertMarkdownToRtf(filePath: string): Promise<ConversionResult> {
    const endTimer = startTimer('convertMarkdownToRtf');
    logger.info('Conversion', 'Starting Markdown to RTF conversion', { filePath });
    
    try {
      const result = await invoke<string>('convert_markdown_to_rtf', {
        filePath
      });
      
      endTimer();
      logger.info('Conversion', 'Markdown to RTF conversion successful', { 
        filePath,
        resultLength: result.length 
      });
      
      return {
        success: true,
        content: result,
        metadata: {
          originalFormat: 'md',
          convertedFormat: 'rtf',
          timestamp: Date.now()
        }
      };
    } catch (error) {
      endTimer();
      logger.error('Conversion', 'Markdown to RTF conversion failed', error, { filePath });
      
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred'
      };
    }
  },

  // Batch convert multiple files
  async batchConvert(
    filePaths: string[],
    targetFormat: 'rtf' | 'md'
  ): Promise<ConversionResult[]> {
    const endTimer = startTimer('batchConvert');
    logger.info('Conversion', 'Starting batch conversion', { 
      fileCount: filePaths.length,
      targetFormat 
    });
    
    const results = await Promise.all(
      filePaths.map(path =>
        targetFormat === 'md'
          ? this.convertRtfToMarkdown(path)
          : this.convertMarkdownToRtf(path)
      )
    );
    
    endTimer();
    const successCount = results.filter(r => r.success).length;
    logger.info('Conversion', 'Batch conversion completed', { 
      totalFiles: filePaths.length,
      successCount,
      failureCount: filePaths.length - successCount,
      targetFormat
    });
    
    return results;
  },

  // Save converted file
  async saveConvertedFile(
    content: string,
    originalPath: string,
    format: 'rtf' | 'md'
  ): Promise<{ success: boolean; path?: string; error?: string }> {
    logger.info('FileOperation', 'Saving converted file', { 
      originalPath,
      format,
      contentLength: content.length 
    });
    
    try {
      const savedPath = await invoke<string>('save_converted_file', {
        content,
        originalPath,
        format
      });
      
      logger.info('FileOperation', 'File saved successfully', { 
        originalPath,
        savedPath,
        format 
      });
      
      return {
        success: true,
        path: savedPath
      };
    } catch (error) {
      logger.error('FileOperation', 'Failed to save file', error, { 
        originalPath,
        format 
      });
      
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred'
      };
    }
  },

  // Convert using pipeline with validation and recovery
  async convertWithPipeline(
    content: string,
    sourceType: 'rtf' | 'markdown',
    config?: PipelineConfig
  ): Promise<PipelineConversionResult> {
    const endTimer = startTimer('convertWithPipeline');
    logger.info('Pipeline', 'Starting pipeline conversion', { 
      sourceType,
      contentLength: content.length,
      config 
    });
    
    try {
      if (sourceType === 'rtf') {
        const result = await invoke<PipelineConversionResult>('rtf_to_markdown_pipeline', {
          rtfContent: content,
          config
        });
        
        endTimer();
        
        if (result.validation_results && result.validation_results.length > 0) {
          logger.warn('Pipeline', 'Validation issues found', { 
            issueCount: result.validation_results.length,
            issues: result.validation_results 
          });
        }
        
        if (result.recovery_actions && result.recovery_actions.length > 0) {
          logger.info('Pipeline', 'Recovery actions applied', { 
            actionCount: result.recovery_actions.length,
            actions: result.recovery_actions 
          });
        }
        
        return result;
      } else {
        // For now, markdown to RTF doesn't use pipeline
        const result = await invoke<string>('markdown_to_rtf', {
          markdownContent: content
        });
        
        endTimer();
        
        return {
          success: true,
          markdown: result,
          validation_results: [],
          recovery_actions: []
        };
      }
    } catch (error) {
      endTimer();
      logger.error('Pipeline', 'Pipeline conversion failed', error, { 
        sourceType,
        config 
      });
      
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred',
        validation_results: [],
        recovery_actions: []
      };
    }
  },

  // Convert file using pipeline
  async convertFileWithPipeline(
    filePath: string,
    config?: PipelineConfig
  ): Promise<PipelineConversionResult> {
    try {
      const result = await invoke<PipelineConversionResult>('read_rtf_file_pipeline', {
        filePath,
        config
      });
      return result;
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred',
        validation_results: [],
        recovery_actions: []
      };
    }
  },

  // Stream conversion updates for real-time preview
  async streamConversion(
    content: string,
    sourceType: 'rtf' | 'markdown',
    onUpdate: (update: StreamUpdate) => void
  ): Promise<() => void> {
    // Create a unique channel for this conversion
    const channelId = `conversion-${Date.now()}-${Math.random()}`;
    logger.info('Streaming', 'Starting stream conversion', { 
      channelId,
      sourceType,
      contentLength: content.length 
    });
    
    // Listen for updates
    const unlisten = await listen<StreamUpdate>(channelId, (event) => {
      if (event.payload.type === 'error') {
        logger.error('Streaming', 'Stream conversion error', event.payload.data.error, { 
          channelId 
        });
      } else if (event.payload.type === 'validation') {
        logger.warn('Streaming', 'Stream validation issues', { 
          channelId,
          issues: event.payload.data.validation 
        });
      }
      onUpdate(event.payload);
    });

    // Start the conversion
    invoke('stream_conversion', {
      content,
      sourceType,
      channelId
    }).catch(error => {
      logger.error('Streaming', 'Failed to start stream conversion', error, { 
        channelId,
        sourceType 
      });
      
      onUpdate({
        type: 'error',
        data: { error: error.message },
        timestamp: Date.now()
      });
    });

    // Return cleanup function
    return () => {
      logger.info('Streaming', 'Cleaning up stream conversion', { channelId });
      unlisten();
    };
  },

  // Get conversion statistics
  async getConversionStats(): Promise<{
    totalConversions: number;
    successfulConversions: number;
    failedConversions: number;
    averageProcessingTime: number;
  }> {
    try {
      const stats = await invoke<{
        totalConversions: number;
        successfulConversions: number;
        failedConversions: number;
        averageProcessingTime: number;
      }>('get_conversion_stats');
      return stats;
    } catch (error) {
      return {
        totalConversions: 0,
        successfulConversions: 0,
        failedConversions: 0,
        averageProcessingTime: 0
      };
    }
  },

  // Read file content directly
  async readFileContent(filePath: string): Promise<{ success: boolean; content?: string; error?: string }> {
    logger.info('FileOperation', 'Reading file content', { filePath });
    
    try {
      // Use the existing read_rtf_file command which returns content
      const response = await invoke<{ success: boolean; content?: string; error?: string }>('read_rtf_file', {
        filePath
      });
      
      if (response.success && response.content) {
        // Return the original RTF content, not the converted markdown
        const rtfContent = await invoke<{ success: boolean; content?: string; error?: string }>('read_file_base64', {
          filePath
        });
        
        if (rtfContent.success && rtfContent.content) {
          const content = atob(rtfContent.content);
          logger.info('FileOperation', 'File read successfully', { 
            filePath,
            contentLength: content.length 
          });
          return {
            success: true,
            content
          };
        }
      }
      
      // If RTF read fails, try reading as plain text via base64
      const base64Response = await invoke<{ success: boolean; content?: string; error?: string }>('read_file_base64', {
        filePath
      });
      
      if (base64Response.success && base64Response.content) {
        const content = atob(base64Response.content);
        logger.info('FileOperation', 'File read successfully via base64', { 
          filePath,
          contentLength: content.length 
        });
        return {
          success: true,
          content
        };
      } else {
        logger.error('FileOperation', 'Failed to read file', new Error(base64Response.error || 'Unknown error'), { 
          filePath 
        });
        return {
          success: false,
          error: base64Response.error || 'Failed to read file'
        };
      }
    } catch (error) {
      logger.error('FileOperation', 'Exception while reading file', error, { filePath });
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Failed to read file'
      };
    }
  }
};