// Unified Error API for LegacyBridge
//
// This module provides TypeScript API wrappers that properly handle
// the unified error responses from the Rust backend.

import { invoke } from '@tauri-apps/api/tauri';
import {
  ErrorResponse,
  LegacyBridgeError,
  ErrorHandler,
  ConversionError,
  IOError,
  ParseError,
  ValidationError,
  SystemError,
  ResourceLimitError,
  NotImplementedError,
} from '@/types/errors';

/**
 * Result type for API calls
 */
export type ApiResult<T> = {
  success: true;
  data: T;
} | {
  success: false;
  error: ErrorResponse;
};

/**
 * Conversion API with unified error handling
 */
export class ConversionAPI {
  /**
   * Convert RTF to Markdown with detailed error handling
   */
  static async convertRtfToMarkdown(rtfContent: string): Promise<ApiResult<string>> {
    try {
      const result = await invoke<string>('convert_rtf_to_markdown', {
        content: rtfContent,
      });
      
      return {
        success: true,
        data: result,
      };
    } catch (error) {
      const errorResponse = this.parseError(error);
      return {
        success: false,
        error: errorResponse,
      };
    }
  }

  /**
   * Convert Markdown to RTF with detailed error handling
   */
  static async convertMarkdownToRtf(markdownContent: string): Promise<ApiResult<string>> {
    try {
      const result = await invoke<string>('convert_markdown_to_rtf', {
        content: markdownContent,
      });
      
      return {
        success: true,
        data: result,
      };
    } catch (error) {
      const errorResponse = this.parseError(error);
      return {
        success: false,
        error: errorResponse,
      };
    }
  }

  /**
   * Convert file with progress tracking and error handling
   */
  static async convertFile(
    inputPath: string,
    outputPath: string,
    onProgress?: (progress: number) => void
  ): Promise<ApiResult<{ outputPath: string; processingTime: number }>> {
    try {
      const startTime = Date.now();
      
      // Set up progress listener
      const unlisten = await invoke('listen_conversion_progress', {
        callback: (progress: number) => {
          onProgress?.(progress);
        },
      });

      const result = await invoke<string>('convert_file', {
        inputPath,
        outputPath,
      });

      // Clean up listener
      await unlisten();

      const processingTime = Date.now() - startTime;

      return {
        success: true,
        data: {
          outputPath: result,
          processingTime,
        },
      };
    } catch (error) {
      const errorResponse = this.parseError(error);
      return {
        success: false,
        error: errorResponse,
      };
    }
  }

  /**
   * Batch convert files with individual error tracking
   */
  static async batchConvert(
    files: Array<{ input: string; output: string }>,
    onProgress?: (completed: number, total: number, errors: ErrorResponse[]) => void
  ): Promise<ApiResult<{
    successful: number;
    failed: number;
    results: Array<{
      input: string;
      output?: string;
      error?: ErrorResponse;
    }>;
  }>> {
    const results: Array<{
      input: string;
      output?: string;
      error?: ErrorResponse;
    }> = [];
    
    let successful = 0;
    let failed = 0;
    const errors: ErrorResponse[] = [];

    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      
      try {
        const result = await invoke<string>('convert_file', {
          inputPath: file.input,
          outputPath: file.output,
        });

        results.push({
          input: file.input,
          output: result,
        });
        successful++;
      } catch (error) {
        const errorResponse = this.parseError(error);
        results.push({
          input: file.input,
          error: errorResponse,
        });
        failed++;
        errors.push(errorResponse);
      }

      onProgress?.(i + 1, files.length, errors);
    }

    return {
      success: true,
      data: {
        successful,
        failed,
        results,
      },
    };
  }

  /**
   * Validate document with detailed error reporting
   */
  static async validateDocument(
    content: string,
    format: 'rtf' | 'markdown'
  ): Promise<ApiResult<{
    valid: boolean;
    errors: ValidationError[];
    warnings: ValidationError[];
  }>> {
    try {
      const result = await invoke<{
        valid: boolean;
        errors: Array<{
          field: string;
          expected: string;
          received: string;
          line?: number;
          column?: number;
        }>;
        warnings: Array<{
          field: string;
          expected: string;
          received: string;
          line?: number;
          column?: number;
        }>;
      }>('validate_document', {
        content,
        format,
      });

      // Convert to proper ValidationError format
      const errors: ValidationError[] = result.errors.map(e => ({
        type: 'ValidationError' as const,
        details: {
          field: e.field,
          expected: e.expected,
          received: e.received,
          location: e.line ? {
            line: e.line,
            column: e.column,
            context: {},
          } : undefined,
        },
      }));

      const warnings: ValidationError[] = result.warnings.map(w => ({
        type: 'ValidationError' as const,
        details: {
          field: w.field,
          expected: w.expected,
          received: w.received,
          location: w.line ? {
            line: w.line,
            column: w.column,
            context: {},
          } : undefined,
        },
      }));

      return {
        success: true,
        data: {
          valid: result.valid,
          errors,
          warnings,
        },
      };
    } catch (error) {
      const errorResponse = this.parseError(error);
      return {
        success: false,
        error: errorResponse,
      };
    }
  }

  /**
   * Parse error from Tauri invoke
   */
  private static parseError(error: unknown): ErrorResponse {
    // Try to parse as ErrorResponse first
    const parsed = ErrorHandler.parseErrorResponse(error);
    if (parsed) {
      return parsed;
    }

    // Fallback to generic error
    const message = error instanceof Error ? error.message : String(error);
    const systemError: SystemError = {
      type: 'SystemError',
      details: {
        component: 'API',
        errorCode: -1,
        description: message,
      },
    };

    return {
      errorType: 'SystemError',
      errorCode: -1,
      message: message,
      userMessage: 'An unexpected error occurred',
      details: systemError,
      suggestions: ['Please try again', 'Contact support if the problem persists'],
      recoverable: false,
      timestamp: new Date().toISOString(),
    };
  }
}

/**
 * Error recovery strategies
 */
export class ErrorRecovery {
  /**
   * Attempt to recover from a conversion error
   */
  static async attemptRecovery(
    error: ConversionError,
    content: string
  ): Promise<ApiResult<string>> {
    if (!error.details.recoverable) {
      return {
        success: false,
        error: {
          errorType: error.type,
          errorCode: -1002,
          message: 'Error is not recoverable',
          userMessage: 'This error cannot be automatically recovered',
          details: error,
          suggestions: error.details.suggestions,
          recoverable: false,
          timestamp: new Date().toISOString(),
        },
      };
    }

    // Try recovery strategies based on error type
    try {
      const result = await invoke<string>('attempt_error_recovery', {
        content,
        errorType: error.type,
        suggestions: error.details.suggestions,
      });

      return {
        success: true,
        data: result,
      };
    } catch (recoveryError) {
      return {
        success: false,
        error: ConversionAPI['parseError'](recoveryError),
      };
    }
  }

  /**
   * Clean and retry conversion
   */
  static async cleanAndRetry(
    content: string,
    format: 'rtf' | 'markdown'
  ): Promise<ApiResult<string>> {
    try {
      // First, clean the content
      const cleaned = await invoke<string>('clean_document', {
        content,
        format,
      });

      // Then retry conversion
      if (format === 'rtf') {
        return ConversionAPI.convertRtfToMarkdown(cleaned);
      } else {
        return ConversionAPI.convertMarkdownToRtf(cleaned);
      }
    } catch (error) {
      return {
        success: false,
        error: ConversionAPI['parseError'](error),
      };
    }
  }
}

/**
 * Error notification service
 */
export class ErrorNotificationService {
  private static listeners: Array<(error: ErrorResponse) => void> = [];

  /**
   * Subscribe to error notifications
   */
  static subscribe(callback: (error: ErrorResponse) => void): () => void {
    this.listeners.push(callback);
    
    // Return unsubscribe function
    return () => {
      const index = this.listeners.indexOf(callback);
      if (index > -1) {
        this.listeners.splice(index, 1);
      }
    };
  }

  /**
   * Notify all listeners of an error
   */
  static notify(error: ErrorResponse): void {
    this.listeners.forEach(listener => {
      try {
        listener(error);
      } catch (e) {
        console.error('Error in error notification listener:', e);
      }
    });
  }

  /**
   * Create a notification-enabled API wrapper
   */
  static wrapApi<T extends (...args: any[]) => Promise<ApiResult<any>>>(
    apiMethod: T
  ): T {
    return (async (...args: Parameters<T>) => {
      const result = await apiMethod(...args);
      
      if (!result.success) {
        this.notify(result.error);
      }
      
      return result;
    }) as T;
  }
}

// Export notification-enabled API methods
export const convertRtfToMarkdown = ErrorNotificationService.wrapApi(
  ConversionAPI.convertRtfToMarkdown
);

export const convertMarkdownToRtf = ErrorNotificationService.wrapApi(
  ConversionAPI.convertMarkdownToRtf
);

export const convertFile = ErrorNotificationService.wrapApi(
  ConversionAPI.convertFile
);

export const validateDocument = ErrorNotificationService.wrapApi(
  ConversionAPI.validateDocument
);