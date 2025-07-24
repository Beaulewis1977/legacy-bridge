// Core file types
export type FileFormat = 'rtf' | 'markdown' | 'md';
export type ConversionDirection = 'rtf-to-markdown' | 'markdown-to-rtf';

// File-related interfaces
export interface FileMetadata {
  id: string;
  name: string;
  path: string;
  size: number;
  type: FileFormat;
  lastModified: Date;
}

export interface ProcessedFile extends FileMetadata {
  content?: string;
  encoding?: string;
  mimeType?: string;
}

// Conversion-related interfaces
export interface ConversionOptions {
  preserveFormatting: boolean;
  strictValidation: boolean;
  autoRecovery: boolean;
  template?: string;
  customOptions?: Record<string, unknown>;
}

export interface ConversionResult {
  id: string;
  fileName: string;
  originalFormat: FileFormat;
  convertedFormat: FileFormat;
  status: 'pending' | 'processing' | 'completed' | 'error';
  progress: number;
  content?: string;
  error?: ConversionError;
  metadata?: ConversionMetadata;
  timestamp: Date;
}

export interface ConversionError {
  code: string;
  message: string;
  details?: string;
  recoverable: boolean;
  suggestions?: string[];
}

export interface ConversionMetadata {
  originalSize: number;
  convertedSize: number;
  processingTime: number;
  validationResults?: ValidationResult[];
  recoveryActions?: RecoveryAction[];
}

// Validation and recovery interfaces
export interface ValidationResult {
  level: 'error' | 'warning' | 'info';
  code: string;
  message: string;
  location?: {
    line?: number;
    column?: number;
    offset?: number;
  };
  suggestion?: string;
}

export interface RecoveryAction {
  actionType: string;
  description: string;
  applied: boolean;
  result?: 'success' | 'failed' | 'skipped';
  details?: string;
}

// API response interfaces
export interface TauriApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  errorCode?: string;
  metadata?: Record<string, unknown>;
}

export interface FileReadResponse {
  success: boolean;
  content?: string;
  encoding?: string;
  error?: string;
}

export interface ConversionStatsResponse {
  totalConversions: number;
  successfulConversions: number;
  failedConversions: number;
  averageProcessingTime: number;
  conversionsByType: Record<ConversionDirection, number>;
}

// Stream update interfaces
export interface StreamUpdate {
  type: 'progress' | 'partial' | 'validation' | 'complete' | 'error';
  data: StreamUpdateData;
  timestamp: number;
}

export interface StreamUpdateData {
  progress?: number;
  content?: string;
  validation?: ValidationResult[];
  error?: ConversionError;
  metadata?: Record<string, unknown>;
}

// File upload and processing
export interface FileUpload {
  id: string;
  name: string;
  size: number;
  type: string;
  lastModified: Date;
  content?: string;
  file?: File;
}

export interface FileWithStatus extends FileMetadata {
  status: 'idle' | 'converting' | 'completed' | 'error';
  result?: ConversionResult;
  progress?: number;
  error?: ConversionError;
}

// Settings and configuration
export interface ConversionSettings {
  preserveFormatting: boolean;
  outputFormat: FileFormat;
  templateId?: string;
  customOptions: Record<string, unknown>;
  validation: {
    strict: boolean;
    autoRecover: boolean;
    reportWarnings: boolean;
  };
}

// Pipeline configuration
export interface PipelineConfig {
  strict_validation?: boolean;
  auto_recovery?: boolean;
  template?: string;
  preserve_formatting?: boolean;
  legacy_mode?: boolean;
  max_recovery_attempts?: number;
  validation_rules?: string[];
}

export interface PipelineConversionResult {
  success: boolean;
  markdown?: string;
  validation_results?: ValidationResult[];
  recovery_actions?: RecoveryAction[];
  error?: string;
  processing_time?: number;
  metadata?: Record<string, unknown>;
}

// Component prop types
export interface PreviewPanelProps {
  sourceContent: string;
  sourceType: FileFormat;
  fileName?: string;
  onContentChange?: (content: string) => void;
  className?: string;
}

export interface ConversionProgressProps {
  files?: FileWithStatus[];
  onDownload: (file: FileWithStatus) => void;
  onPreview: (file: FileWithStatus) => void;
  onRetry: (file: FileWithStatus) => void;
  className?: string;
}

export interface DragDropZoneProps {
  onFilesSelected?: (files: File[]) => void;
  accept?: string[];
  maxFiles?: number;
  maxSize?: number;
  className?: string;
}

// Utility types
export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export type AsyncResult<T> = Promise<{ success: true; data: T } | { success: false; error: string }>;

export type ConversionHandler = (
  file: ProcessedFile,
  options: ConversionOptions
) => AsyncResult<ConversionResult>;

// Type guards
export function isFileFormat(value: unknown): value is FileFormat {
  return typeof value === 'string' && ['rtf', 'markdown', 'md'].includes(value);
}

export function isConversionError(error: unknown): error is ConversionError {
  return (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    'message' in error &&
    'recoverable' in error
  );
}

export function isValidationResult(result: unknown): result is ValidationResult {
  return (
    typeof result === 'object' &&
    result !== null &&
    'level' in result &&
    'code' in result &&
    'message' in result
  );
}