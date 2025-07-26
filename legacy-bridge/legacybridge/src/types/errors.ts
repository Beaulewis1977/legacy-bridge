// Unified Error Types for LegacyBridge
//
// These types match the Rust error structures for consistent error handling
// across TypeScript frontend, Rust backend, and C FFI layer.

/**
 * Error context with location information
 */
export interface ErrorContext {
  line?: number;
  column?: number;
  offset?: number;
  filePath?: string;
  context: Record<string, string>;
}

/**
 * Base error response from backend
 */
export interface ErrorResponse {
  errorType: string;
  errorCode: number;
  message: string;
  userMessage: string;
  details: LegacyBridgeError;
  suggestions: string[];
  recoverable: boolean;
  timestamp: string;
}

/**
 * Unified error types matching Rust implementation
 */
export type LegacyBridgeError =
  | ParseError
  | ConversionError
  | IOError
  | ValidationError
  | SystemError
  | ResourceLimitError
  | NotImplementedError;

/**
 * Parse error with location information
 */
export interface ParseError {
  type: 'ParseError';
  details: {
    message: string;
    line: number;
    column: number;
    expected?: string;
    found?: string;
    filePath?: string;
  };
}

/**
 * Conversion error between formats
 */
export interface ConversionError {
  type: 'ConversionError';
  details: {
    sourceFormat: string;
    targetFormat: string;
    details: string;
    recoverable: boolean;
    suggestions: string[];
  };
}

/**
 * IO error with operation context
 */
export interface IOError {
  type: 'IOError';
  details: {
    operation: string;
    path: string;
    cause: string;
    errorCode?: number;
  };
}

/**
 * Validation error with field information
 */
export interface ValidationError {
  type: 'ValidationError';
  details: {
    field: string;
    expected: string;
    received: string;
    location?: ErrorContext;
  };
}

/**
 * System error from internal components
 */
export interface SystemError {
  type: 'SystemError';
  details: {
    component: string;
    errorCode: number;
    description: string;
    internalMessage?: string;
  };
}

/**
 * Resource limit exceeded error
 */
export interface ResourceLimitError {
  type: 'ResourceLimitError';
  details: {
    resource: string;
    limit: string;
    actual: string;
    suggestion: string;
  };
}

/**
 * Feature not implemented error
 */
export interface NotImplementedError {
  type: 'NotImplementedError';
  details: {
    feature: string;
    workaround?: string;
    plannedVersion?: string;
  };
}

/**
 * Error handler class for consistent error processing
 */
export class ErrorHandler {
  /**
   * Parse error response from backend
   */
  static parseErrorResponse(response: unknown): ErrorResponse | null {
    try {
      if (typeof response === 'string') {
        return JSON.parse(response) as ErrorResponse;
      }
      if (typeof response === 'object' && response !== null) {
        return response as ErrorResponse;
      }
      return null;
    } catch {
      return null;
    }
  }

  /**
   * Get user-friendly error message
   */
  static getUserMessage(error: LegacyBridgeError | ErrorResponse): string {
    if ('userMessage' in error) {
      return error.userMessage;
    }

    switch (error.type) {
      case 'ParseError':
        return 'The document format is invalid or corrupted';
      case 'ConversionError':
        return `Failed to convert from ${error.details.sourceFormat} to ${error.details.targetFormat}`;
      case 'IOError':
        return `Failed to ${error.details.operation} file`;
      case 'ValidationError':
        return `Invalid ${error.details.field}`;
      case 'SystemError':
        return 'An internal error occurred';
      case 'ResourceLimitError':
        return `${error.details.resource} limit exceeded`;
      case 'NotImplementedError':
        return `'${error.details.feature}' is not available yet`;
      default:
        return 'An unknown error occurred';
    }
  }

  /**
   * Get error suggestions
   */
  static getSuggestions(error: LegacyBridgeError | ErrorResponse): string[] {
    if ('suggestions' in error) {
      return error.suggestions;
    }

    switch (error.type) {
      case 'ParseError':
        return [
          'Check if the file is a valid RTF document',
          'Ensure the file is not corrupted',
        ];
      case 'ConversionError':
        return error.details.suggestions || [];
      case 'IOError':
        return [
          'Check file permissions',
          'Ensure the file path is correct',
          'Verify disk space is available',
        ];
      case 'ValidationError':
        return [`Ensure the value matches the expected format: ${error.details.expected}`];
      case 'ResourceLimitError':
        return [error.details.suggestion];
      case 'NotImplementedError':
        return error.details.workaround ? [error.details.workaround] : [];
      default:
        return [];
    }
  }

  /**
   * Check if error is recoverable
   */
  static isRecoverable(error: LegacyBridgeError | ErrorResponse): boolean {
    if ('recoverable' in error) {
      return error.recoverable;
    }

    switch (error.type) {
      case 'ConversionError':
        return error.details.recoverable;
      case 'ValidationError':
        return true;
      default:
        return false;
    }
  }

  /**
   * Format error for display
   */
  static formatError(error: LegacyBridgeError | ErrorResponse): string {
    const userMessage = ErrorHandler.getUserMessage(error);
    const suggestions = ErrorHandler.getSuggestions(error);

    let formatted = userMessage;
    
    if (suggestions.length > 0) {
      formatted += '\n\nSuggestions:\n';
      suggestions.forEach((suggestion, index) => {
        formatted += `${index + 1}. ${suggestion}\n`;
      });
    }

    return formatted;
  }

  /**
   * Create error from legacy format
   */
  static fromLegacyError(error: { code: string; message: string; details?: string }): SystemError {
    return {
      type: 'SystemError',
      details: {
        component: 'Legacy',
        errorCode: parseInt(error.code) || -1,
        description: error.message,
        internalMessage: error.details,
      },
    };
  }
}

/**
 * Type guards for error types
 */
export const isParseError = (error: LegacyBridgeError): error is ParseError =>
  error.type === 'ParseError';

export const isConversionError = (error: LegacyBridgeError): error is ConversionError =>
  error.type === 'ConversionError';

export const isIOError = (error: LegacyBridgeError): error is IOError =>
  error.type === 'IOError';

export const isValidationError = (error: LegacyBridgeError): error is ValidationError =>
  error.type === 'ValidationError';

export const isSystemError = (error: LegacyBridgeError): error is SystemError =>
  error.type === 'SystemError';

export const isResourceLimitError = (error: LegacyBridgeError): error is ResourceLimitError =>
  error.type === 'ResourceLimitError';

export const isNotImplementedError = (error: LegacyBridgeError): error is NotImplementedError =>
  error.type === 'NotImplementedError';

/**
 * Error severity levels
 */
export enum ErrorSeverity {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  CRITICAL = 'critical',
}

/**
 * Get error severity
 */
export function getErrorSeverity(error: LegacyBridgeError): ErrorSeverity {
  switch (error.type) {
    case 'NotImplementedError':
      return ErrorSeverity.LOW;
    case 'ValidationError':
      return ErrorSeverity.MEDIUM;
    case 'ConversionError':
      return error.details.recoverable ? ErrorSeverity.MEDIUM : ErrorSeverity.HIGH;
    case 'ParseError':
    case 'IOError':
      return ErrorSeverity.HIGH;
    case 'SystemError':
    case 'ResourceLimitError':
      return ErrorSeverity.CRITICAL;
    default:
      return ErrorSeverity.MEDIUM;
  }
}