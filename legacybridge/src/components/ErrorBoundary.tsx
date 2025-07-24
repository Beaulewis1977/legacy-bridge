'use client';

import React, { Component, PropsWithChildren, ErrorInfo } from 'react';
import { AlertCircle, RefreshCw, Home, FileText } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert-dialog';

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
  errorId: string;
}

interface ErrorBoundaryProps extends PropsWithChildren {
  fallback?: React.ComponentType<ErrorFallbackProps>;
  onError?: (error: Error, errorInfo: ErrorInfo, errorId: string) => void;
  isolate?: boolean;
}

interface ErrorFallbackProps {
  error?: Error;
  errorInfo?: ErrorInfo;
  onRetry: () => void;
  errorId: string;
}

// Generate unique error ID for tracking
function generateErrorId(): string {
  return `error-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

// Default error fallback component
function DefaultErrorFallback({ error, errorInfo, onRetry, errorId }: ErrorFallbackProps) {
  const isDevelopment = process.env.NODE_ENV === 'development';

  return (
    <div className="min-h-[400px] flex items-center justify-center p-4">
      <Card className="max-w-lg w-full p-6 shadow-lg">
        <div className="space-y-4">
          <div className="flex items-center gap-3">
            <div className="p-3 bg-destructive/10 rounded-full">
              <AlertCircle className="w-6 h-6 text-destructive" />
            </div>
            <div>
              <h2 className="text-xl font-semibold">Something went wrong</h2>
              <p className="text-sm text-muted-foreground">Error ID: {errorId}</p>
            </div>
          </div>

          <Alert variant="destructive" className="border-destructive/20">
            <AlertTitle>Error Details</AlertTitle>
            <AlertDescription className="mt-2">
              {error?.message || 'An unexpected error occurred'}
            </AlertDescription>
          </Alert>

          {isDevelopment && errorInfo && (
            <details className="space-y-2">
              <summary className="cursor-pointer text-sm font-medium text-muted-foreground hover:text-foreground">
                Show stack trace
              </summary>
              <pre className="text-xs bg-muted p-3 rounded-md overflow-auto max-h-60">
                {errorInfo.componentStack}
              </pre>
              {error?.stack && (
                <pre className="text-xs bg-muted p-3 rounded-md overflow-auto max-h-60">
                  {error.stack}
                </pre>
              )}
            </details>
          )}

          <div className="flex gap-3">
            <Button onClick={onRetry} className="flex-1" variant="default">
              <RefreshCw className="w-4 h-4 mr-2" />
              Try Again
            </Button>
            <Button
              onClick={() => window.location.href = '/'}
              variant="outline"
              className="flex-1"
            >
              <Home className="w-4 h-4 mr-2" />
              Go Home
            </Button>
          </div>
        </div>
      </Card>
    </div>
  );
}

// Main error boundary component
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      errorId: generateErrorId(),
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return {
      hasError: true,
      error,
      errorId: generateErrorId(),
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    const { errorId } = this.state;
    
    // Log error to console in development
    if (process.env.NODE_ENV === 'development') {
      console.error('Error Boundary caught:', error, errorInfo);
    }

    // Report to error tracking service
    this.reportError(error, errorInfo, errorId);

    // Call custom error handler if provided
    if (this.props.onError) {
      this.props.onError(error, errorInfo, errorId);
    }
  }

  private reportError(error: Error, errorInfo: ErrorInfo, errorId: string) {
    // In production, this would send to an error tracking service
    // For now, we'll store in localStorage for debugging
    try {
      const errorLog = {
        id: errorId,
        timestamp: new Date().toISOString(),
        error: {
          message: error.message,
          stack: error.stack,
        },
        errorInfo: {
          componentStack: errorInfo.componentStack,
        },
        userAgent: navigator.userAgent,
        url: window.location.href,
      };

      // Store last 10 errors in localStorage
      const storedErrors = JSON.parse(localStorage.getItem('legacybridge_errors') || '[]');
      storedErrors.unshift(errorLog);
      localStorage.setItem(
        'legacybridge_errors',
        JSON.stringify(storedErrors.slice(0, 10))
      );
    } catch (e) {
      // Fail silently if localStorage is not available
    }
  }

  private handleRetry = () => {
    this.setState({
      hasError: false,
      error: undefined,
      errorInfo: undefined,
      errorId: generateErrorId(),
    });
  };

  render() {
    if (this.state.hasError) {
      const FallbackComponent = this.props.fallback || DefaultErrorFallback;
      
      return (
        <FallbackComponent
          error={this.state.error}
          errorInfo={this.state.errorInfo}
          onRetry={this.handleRetry}
          errorId={this.state.errorId}
        />
      );
    }

    return this.props.children;
  }
}

// Specialized error boundary for conversion operations
export class ConversionErrorBoundary extends Component<PropsWithChildren, ErrorBoundaryState> {
  constructor(props: PropsWithChildren) {
    super(props);
    this.state = {
      hasError: false,
      errorId: generateErrorId(),
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return {
      hasError: true,
      error,
      errorId: generateErrorId(),
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Conversion Error:', error, errorInfo);
    this.reportConversionError(error, errorInfo);
  }

  private reportConversionError(error: Error, errorInfo: ErrorInfo) {
    // Specialized error reporting for conversion errors
    try {
      const conversionError = {
        id: this.state.errorId,
        timestamp: new Date().toISOString(),
        type: 'conversion_error',
        error: {
          message: error.message,
          stack: error.stack,
        },
        context: {
          component: 'ConversionErrorBoundary',
          componentStack: errorInfo.componentStack,
        },
      };

      // In production, send to analytics/monitoring service
      console.log('Conversion error logged:', conversionError);
    } catch (e) {
      // Fail silently
    }
  }

  render() {
    if (this.state.hasError) {
      return (
        <Alert variant="destructive" className="m-4">
          <FileText className="w-4 h-4" />
          <AlertTitle>Conversion Error</AlertTitle>
          <AlertDescription>
            Failed to process the file conversion. Please check your file format and try again.
            {this.state.error && (
              <div className="mt-2 text-xs">
                Error: {this.state.error.message}
              </div>
            )}
          </AlertDescription>
          <Button
            size="sm"
            variant="outline"
            className="mt-3"
            onClick={() => this.setState({ hasError: false, error: undefined, errorInfo: undefined })}
          >
            <RefreshCw className="w-3 h-3 mr-1" />
            Retry
          </Button>
        </Alert>
      );
    }

    return this.props.children;
  }
}

// Hook for programmatic error handling
export function useErrorHandler() {
  return (error: Error) => {
    throw error;
  };
}

// Async error boundary wrapper
export function withAsyncErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  fallback?: React.ComponentType<ErrorFallbackProps>
) {
  return (props: P) => (
    <ErrorBoundary fallback={fallback}>
      <Component {...props} />
    </ErrorBoundary>
  );
}