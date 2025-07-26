/**
 * Comprehensive Error Logging System for LegacyBridge
 * 
 * Features:
 * - Multiple log levels (debug, info, warn, error, fatal)
 * - Structured logging with context
 * - File-based persistence
 * - Remote logging support
 * - Performance tracking
 * - Error aggregation and analytics
 */

import { format } from 'date-fns';
import { v4 as uuidv4 } from 'uuid';

// Log levels
export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
  FATAL = 4
}

// Log entry interface
export interface LogEntry {
  id: string;
  timestamp: Date;
  level: LogLevel;
  category: string;
  message: string;
  context?: Record<string, any>;
  stackTrace?: string;
  userId?: string;
  sessionId?: string;
  errorCode?: string;
  duration?: number;
  metadata?: Record<string, any>;
}

// Error analytics
export interface ErrorAnalytics {
  errorCount: number;
  errorRate: number;
  topErrors: Array<{
    message: string;
    count: number;
    lastOccurred: Date;
  }>;
  errorsByCategory: Record<string, number>;
  errorsByLevel: Record<LogLevel, number>;
}

// Logger configuration
export interface LoggerConfig {
  logLevel: LogLevel;
  logToFile: boolean;
  logToConsole: boolean;
  logToRemote: boolean;
  maxFileSize: number; // in MB
  maxFiles: number;
  remoteEndpoint?: string;
  remoteApiKey?: string;
  enableAnalytics: boolean;
  enablePerformanceTracking: boolean;
}

class ErrorLogger {
  private static instance: ErrorLogger;
  private config: LoggerConfig;
  private logBuffer: LogEntry[] = [];
  private analytics: ErrorAnalytics = {
    errorCount: 0,
    errorRate: 0,
    topErrors: [],
    errorsByCategory: {},
    errorsByLevel: {}
  };
  private logDirectory: string;
  private currentLogFile: string;
  private sessionId: string;

  private constructor(config: LoggerConfig) {
    this.config = config;
    this.sessionId = uuidv4();
    
    // Initialize log directory (browser-compatible)
    this.logDirectory = 'logs';
    this.currentLogFile = this.getLogFileName();
    
    // Initialize analytics
    Object.values(LogLevel).forEach(level => {
      if (typeof level === 'number') {
        this.analytics.errorsByLevel[level] = 0;
      }
    });
    
    // Set up periodic analytics update
    if (this.config.enableAnalytics) {
      setInterval(() => this.updateAnalytics(), 60000); // Every minute
    }
    
    // Set up log rotation
    setInterval(() => this.rotateLogsIfNeeded(), 3600000); // Every hour
  }

  public static getInstance(config?: LoggerConfig): ErrorLogger {
    if (!ErrorLogger.instance) {
      ErrorLogger.instance = new ErrorLogger(config || {
        logLevel: LogLevel.INFO,
        logToFile: true,
        logToConsole: true,
        logToRemote: false,
        maxFileSize: 10, // 10MB
        maxFiles: 5,
        enableAnalytics: true,
        enablePerformanceTracking: true
      });
    }
    return ErrorLogger.instance;
  }

  // Main logging methods
  public debug(category: string, message: string, context?: Record<string, any>): void {
    this.log(LogLevel.DEBUG, category, message, context);
  }

  public info(category: string, message: string, context?: Record<string, any>): void {
    this.log(LogLevel.INFO, category, message, context);
  }

  public warn(category: string, message: string, context?: Record<string, any>): void {
    this.log(LogLevel.WARN, category, message, context);
  }

  public error(category: string, message: string, error?: Error | any, context?: Record<string, any>): void {
    const errorContext = {
      ...context,
      errorName: error?.name,
      errorMessage: error?.message,
      errorStack: error?.stack
    };
    this.log(LogLevel.ERROR, category, message, errorContext, error?.stack);
  }

  public fatal(category: string, message: string, error?: Error, context?: Record<string, any>): void {
    const errorContext = {
      ...context,
      errorName: error?.name,
      errorMessage: error?.message,
      errorStack: error?.stack
    };
    this.log(LogLevel.FATAL, category, message, errorContext, error?.stack);
  }

  // Performance tracking
  public startPerformanceTimer(operationName: string): () => void {
    const startTime = Date.now();
    
    return () => {
      const duration = Date.now() - startTime;
      this.info('Performance', `Operation "${operationName}" completed`, {
        operation: operationName,
        duration,
        durationMs: duration,
        durationSeconds: duration / 1000
      });
    };
  }

  // Structured logging
  private log(
    level: LogLevel,
    category: string,
    message: string,
    context?: Record<string, any>,
    stackTrace?: string
  ): void {
    // Check if we should log this level
    if (level < this.config.logLevel) {
      return;
    }

    const entry: LogEntry = {
      id: uuidv4(),
      timestamp: new Date(),
      level,
      category,
      message,
      context,
      stackTrace,
      sessionId: this.sessionId,
      userId: context?.userId || this.getCurrentUserId(),
      errorCode: context?.errorCode,
      duration: context?.duration,
      metadata: {
        environment: process.env.NODE_ENV || 'development',
        version: process.env.APP_VERSION || '1.0.0',
        platform: typeof window !== 'undefined' ? 'browser' : 'node',
        userAgent: typeof window !== 'undefined' ? window.navigator.userAgent : 'unknown'
      }
    };

    // Add to buffer
    this.logBuffer.push(entry);

    // Log to various destinations
    if (this.config.logToConsole) {
      this.logToConsole(entry);
    }

    if (this.config.logToFile) {
      this.logToFile(entry);
    }

    if (this.config.logToRemote) {
      this.logToRemote(entry);
    }

    // Update analytics
    if (this.config.enableAnalytics) {
      this.updateAnalyticsForEntry(entry);
    }
  }

  // Console logging with colors
  private logToConsole(entry: LogEntry): void {
    const colors = {
      [LogLevel.DEBUG]: '\x1b[36m', // Cyan
      [LogLevel.INFO]: '\x1b[32m',  // Green
      [LogLevel.WARN]: '\x1b[33m',  // Yellow
      [LogLevel.ERROR]: '\x1b[31m', // Red
      [LogLevel.FATAL]: '\x1b[35m'  // Magenta
    };

    const reset = '\x1b[0m';
    const color = colors[entry.level];
    const levelName = LogLevel[entry.level];
    const timestamp = format(entry.timestamp, 'yyyy-MM-dd HH:mm:ss.SSS');

    console.log(
      `${color}[${timestamp}] [${levelName}] [${entry.category}]${reset} ${entry.message}`
    );

    if (entry.context && Object.keys(entry.context).length > 0) {
      console.log('Context:', JSON.stringify(entry.context, null, 2));
    }

    if (entry.stackTrace) {
      console.log('Stack Trace:', entry.stackTrace);
    }
  }

  // File logging (browser-compatible using localStorage)
  private logToFile(entry: LogEntry): void {
    if (typeof window === 'undefined') return; // Skip during SSR
    
    try {
      const logKey = `legacybridge_logs_${this.currentLogFile}`;
      const existingLogs = localStorage.getItem(logKey) || '';
      const logLine = JSON.stringify(entry) + '\n';
      
      localStorage.setItem(logKey, existingLogs + logLine);
    } catch (error) {
      console.error('Failed to write to log storage:', error);
    }
  }

  // Remote logging (for production monitoring)
  private async logToRemote(entry: LogEntry): Promise<void> {
    if (!this.config.remoteEndpoint) {
      return;
    }

    try {
      const response = await fetch(this.config.remoteEndpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${this.config.remoteApiKey}`
        },
        body: JSON.stringify({
          logs: [entry]
        })
      });

      if (!response.ok) {
        console.error('Failed to send logs to remote server:', response.statusText);
      }
    } catch (error) {
      console.error('Error sending logs to remote server:', error);
    }
  }

  // Analytics methods
  private updateAnalyticsForEntry(entry: LogEntry): void {
    if (entry.level >= LogLevel.ERROR) {
      this.analytics.errorCount++;
      this.analytics.errorsByLevel[entry.level]++;
      this.analytics.errorsByCategory[entry.category] = 
        (this.analytics.errorsByCategory[entry.category] || 0) + 1;

      // Update top errors
      const existingError = this.analytics.topErrors.find(e => e.message === entry.message);
      if (existingError) {
        existingError.count++;
        existingError.lastOccurred = entry.timestamp;
      } else {
        this.analytics.topErrors.push({
          message: entry.message,
          count: 1,
          lastOccurred: entry.timestamp
        });
      }

      // Keep only top 10 errors
      this.analytics.topErrors.sort((a, b) => b.count - a.count);
      this.analytics.topErrors = this.analytics.topErrors.slice(0, 10);
    }
  }

  private updateAnalytics(): void {
    // Calculate error rate (errors per minute)
    const recentErrors = this.logBuffer.filter(entry => 
      entry.level >= LogLevel.ERROR &&
      entry.timestamp.getTime() > Date.now() - 60000
    );
    
    this.analytics.errorRate = recentErrors.length;

    // Clean old entries from buffer (keep last hour)
    const oneHourAgo = Date.now() - 3600000;
    this.logBuffer = this.logBuffer.filter(entry => 
      entry.timestamp.getTime() > oneHourAgo
    );
  }

  // Log rotation (browser-compatible)
  private rotateLogsIfNeeded(): void {
    if (typeof window === 'undefined') return; // Skip during SSR
    
    try {
      const logKey = `legacybridge_logs_${this.currentLogFile}`;
      const logData = localStorage.getItem(logKey) || '';
      const logSizeKB = new Blob([logData]).size / 1024;

      if (logSizeKB >= this.config.maxFileSize * 1024) {
        this.currentLogFile = this.getLogFileName();
        this.cleanOldLogs();
      }
    } catch (error) {
      // Storage doesn't exist yet, ignore
    }
  }

  private cleanOldLogs(): void {
    if (typeof window === 'undefined') return; // Skip during SSR
    
    try {
      const logKeys = Object.keys(localStorage)
        .filter(key => key.startsWith('legacybridge_logs_'))
        .sort()
        .reverse();

      // Keep only the most recent log files
      if (logKeys.length > this.config.maxFiles) {
        logKeys.slice(this.config.maxFiles).forEach(key => {
          localStorage.removeItem(key);
        });
      }
    } catch (error) {
      console.error('Error cleaning old logs:', error);
    }
  }

  private getLogFileName(): string {
    const timestamp = format(new Date(), 'yyyy-MM-dd-HH');
    return `legacybridge-${timestamp}.log`;
  }

  private getCurrentUserId(): string | undefined {
    // This would be implemented based on your authentication system
    return undefined;
  }

  // Public methods for retrieving logs and analytics
  public getRecentLogs(count: number = 100): LogEntry[] {
    return this.logBuffer.slice(-count);
  }

  public getLogsByLevel(level: LogLevel, count: number = 100): LogEntry[] {
    return this.logBuffer
      .filter(entry => entry.level === level)
      .slice(-count);
  }

  public getLogsByCategory(category: string, count: number = 100): LogEntry[] {
    return this.logBuffer
      .filter(entry => entry.category === category)
      .slice(-count);
  }

  public getAnalytics(): ErrorAnalytics {
    return { ...this.analytics };
  }

  public exportLogs(startDate: Date, endDate: Date): LogEntry[] {
    return this.logBuffer.filter(entry =>
      entry.timestamp >= startDate && entry.timestamp <= endDate
    );
  }

  // Search functionality
  public searchLogs(query: string, options?: {
    level?: LogLevel;
    category?: string;
    startDate?: Date;
    endDate?: Date;
  }): LogEntry[] {
    return this.logBuffer.filter(entry => {
      const matchesQuery = entry.message.toLowerCase().includes(query.toLowerCase()) ||
        JSON.stringify(entry.context).toLowerCase().includes(query.toLowerCase());
      
      const matchesLevel = !options?.level || entry.level === options.level;
      const matchesCategory = !options?.category || entry.category === options.category;
      const matchesStartDate = !options?.startDate || entry.timestamp >= options.startDate;
      const matchesEndDate = !options?.endDate || entry.timestamp <= options.endDate;

      return matchesQuery && matchesLevel && matchesCategory && matchesStartDate && matchesEndDate;
    });
  }
}

// Export singleton instance
export const logger = ErrorLogger.getInstance();

// Export convenience functions
export const logDebug = (category: string, message: string, context?: Record<string, any>) =>
  logger.debug(category, message, context);

export const logInfo = (category: string, message: string, context?: Record<string, any>) =>
  logger.info(category, message, context);

export const logWarn = (category: string, message: string, context?: Record<string, any>) =>
  logger.warn(category, message, context);

export const logError = (category: string, message: string, error?: Error | any, context?: Record<string, any>) =>
  logger.error(category, message, error, context);

export const logFatal = (category: string, message: string, error?: Error, context?: Record<string, any>) =>
  logger.fatal(category, message, error, context);

export const startTimer = (operationName: string) =>
  logger.startPerformanceTimer(operationName);

export default logger;