// Unified Error Display Component
//
// This component demonstrates how to properly display errors
// using the unified error handling system.

import React, { useState, useCallback } from 'react';
import { AlertCircle, AlertTriangle, XCircle, Info, RefreshCw } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import {
  ErrorResponse,
  LegacyBridgeError,
  ErrorHandler,
  getErrorSeverity,
  ErrorSeverity,
  isConversionError,
  isParseError,
  isIOError,
  isValidationError,
} from '@/types/errors';
import { ErrorRecovery } from '@/lib/unified-error-api';

interface UnifiedErrorDisplayProps {
  error: ErrorResponse | LegacyBridgeError;
  onRetry?: () => void;
  onDismiss?: () => void;
  showDetails?: boolean;
  className?: string;
}

export const UnifiedErrorDisplay: React.FC<UnifiedErrorDisplayProps> = ({
  error,
  onRetry,
  onDismiss,
  showDetails = true,
  className,
}) => {
  const [isDetailsOpen, setIsDetailsOpen] = useState(false);
  const [isRecovering, setIsRecovering] = useState(false);

  // Normalize error to ErrorResponse format
  const errorResponse: ErrorResponse = 'errorType' in error
    ? error as ErrorResponse
    : {
        errorType: error.type,
        errorCode: -1,
        message: ErrorHandler.formatError(error),
        userMessage: ErrorHandler.getUserMessage(error),
        details: error,
        suggestions: ErrorHandler.getSuggestions(error),
        recoverable: ErrorHandler.isRecoverable(error),
        timestamp: new Date().toISOString(),
      };

  // Get severity for styling
  const severity = getErrorSeverity(errorResponse.details);
  
  // Get icon based on severity
  const getIcon = () => {
    switch (severity) {
      case ErrorSeverity.CRITICAL:
        return <XCircle className="h-5 w-5" />;
      case ErrorSeverity.HIGH:
        return <AlertCircle className="h-5 w-5" />;
      case ErrorSeverity.MEDIUM:
        return <AlertTriangle className="h-5 w-5" />;
      default:
        return <Info className="h-5 w-5" />;
    }
  };

  // Get alert variant based on severity
  const getVariant = (): 'default' | 'destructive' => {
    return severity === ErrorSeverity.CRITICAL || severity === ErrorSeverity.HIGH
      ? 'destructive'
      : 'default';
  };

  // Handle recovery attempt
  const handleRecovery = useCallback(async () => {
    if (!isConversionError(errorResponse.details)) {
      return;
    }

    setIsRecovering(true);
    try {
      // This would need the original content passed in as a prop
      // For demo purposes, we'll just show the recovery attempt
      console.log('Attempting recovery with suggestions:', errorResponse.suggestions);
      
      // In a real implementation:
      // const result = await ErrorRecovery.attemptRecovery(errorResponse.details, originalContent);
      // if (result.success) { ... }
      
      // Simulate recovery attempt
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      if (onRetry) {
        onRetry();
      }
    } catch (e) {
      console.error('Recovery failed:', e);
    } finally {
      setIsRecovering(false);
    }
  }, [errorResponse, onRetry]);

  // Render error location if available
  const renderLocation = () => {
    const error = errorResponse.details;
    
    if (isParseError(error) && error.details.line) {
      return (
        <span className="text-sm text-muted-foreground">
          Line {error.details.line}, Column {error.details.column || 0}
          {error.details.filePath && ` in ${error.details.filePath}`}
        </span>
      );
    }
    
    if (isIOError(error) && error.details.path) {
      return (
        <span className="text-sm text-muted-foreground">
          File: {error.details.path}
        </span>
      );
    }
    
    if (isValidationError(error) && error.details.location?.line) {
      return (
        <span className="text-sm text-muted-foreground">
          Line {error.details.location.line}
          {error.details.location.column && `, Column ${error.details.location.column}`}
        </span>
      );
    }
    
    return null;
  };

  // Render error-specific details
  const renderErrorDetails = () => {
    const error = errorResponse.details;
    
    if (isParseError(error)) {
      return (
        <div className="space-y-2">
          {error.details.expected && (
            <div>
              <span className="font-medium">Expected:</span>{' '}
              <code className="text-sm bg-muted px-1 py-0.5 rounded">
                {error.details.expected}
              </code>
            </div>
          )}
          {error.details.found && (
            <div>
              <span className="font-medium">Found:</span>{' '}
              <code className="text-sm bg-muted px-1 py-0.5 rounded">
                {error.details.found}
              </code>
            </div>
          )}
        </div>
      );
    }
    
    if (isIOError(error)) {
      return (
        <div className="space-y-2">
          <div>
            <span className="font-medium">Operation:</span> {error.details.operation}
          </div>
          {error.details.errorCode && (
            <div>
              <span className="font-medium">System Error Code:</span> {error.details.errorCode}
            </div>
          )}
        </div>
      );
    }
    
    if (isValidationError(error)) {
      return (
        <div className="space-y-2">
          <div>
            <span className="font-medium">Field:</span> {error.details.field}
          </div>
          <div>
            <span className="font-medium">Expected:</span>{' '}
            <code className="text-sm bg-muted px-1 py-0.5 rounded">
              {error.details.expected}
            </code>
          </div>
          <div>
            <span className="font-medium">Received:</span>{' '}
            <code className="text-sm bg-muted px-1 py-0.5 rounded">
              {error.details.received}
            </code>
          </div>
        </div>
      );
    }
    
    return null;
  };

  return (
    <Alert variant={getVariant()} className={className}>
      <div className="flex items-start gap-3">
        {getIcon()}
        <div className="flex-1">
          <AlertTitle className="flex items-center justify-between">
            <span>{errorResponse.userMessage}</span>
            {onDismiss && (
              <Button
                variant="ghost"
                size="sm"
                onClick={onDismiss}
                className="h-6 w-6 p-0"
              >
                <XCircle className="h-4 w-4" />
              </Button>
            )}
          </AlertTitle>
          
          <AlertDescription className="mt-2 space-y-3">
            {renderLocation()}
            
            {/* Suggestions */}
            {errorResponse.suggestions.length > 0 && (
              <div className="space-y-1">
                <p className="font-medium text-sm">Suggestions:</p>
                <ul className="list-disc list-inside space-y-1">
                  {errorResponse.suggestions.map((suggestion, index) => (
                    <li key={index} className="text-sm">
                      {suggestion}
                    </li>
                  ))}
                </ul>
              </div>
            )}
            
            {/* Actions */}
            <div className="flex gap-2 mt-3">
              {errorResponse.recoverable && (
                <Button
                  size="sm"
                  variant="outline"
                  onClick={handleRecovery}
                  disabled={isRecovering}
                >
                  {isRecovering ? (
                    <>
                      <RefreshCw className="mr-2 h-3 w-3 animate-spin" />
                      Recovering...
                    </>
                  ) : (
                    'Attempt Recovery'
                  )}
                </Button>
              )}
              
              {onRetry && (
                <Button size="sm" variant="outline" onClick={onRetry}>
                  <RefreshCw className="mr-2 h-3 w-3" />
                  Retry
                </Button>
              )}
            </div>
            
            {/* Technical Details */}
            {showDetails && (
              <Collapsible open={isDetailsOpen} onOpenChange={setIsDetailsOpen}>
                <CollapsibleTrigger asChild>
                  <Button variant="ghost" size="sm" className="p-0 h-auto font-normal">
                    {isDetailsOpen ? 'Hide' : 'Show'} technical details
                  </Button>
                </CollapsibleTrigger>
                <CollapsibleContent className="mt-2 space-y-3">
                  <div className="bg-muted p-3 rounded-md space-y-2 text-sm">
                    <div>
                      <span className="font-medium">Error Type:</span> {errorResponse.errorType}
                    </div>
                    <div>
                      <span className="font-medium">Error Code:</span> {errorResponse.errorCode}
                    </div>
                    <div>
                      <span className="font-medium">Timestamp:</span>{' '}
                      {new Date(errorResponse.timestamp).toLocaleString()}
                    </div>
                    
                    {renderErrorDetails()}
                    
                    {/* Developer message */}
                    <div className="mt-3">
                      <span className="font-medium">Technical Message:</span>
                      <pre className="mt-1 text-xs bg-background p-2 rounded overflow-x-auto">
                        {errorResponse.message}
                      </pre>
                    </div>
                  </div>
                </CollapsibleContent>
              </Collapsible>
            )}
          </AlertDescription>
        </div>
      </div>
    </Alert>
  );
};

// Example usage component
export const ErrorDisplayExample: React.FC = () => {
  const [error, setError] = useState<ErrorResponse | null>(null);

  // Simulate different error types
  const triggerParseError = () => {
    const parseError: ErrorResponse = {
      errorType: 'ParseError',
      errorCode: -1001,
      message: 'Parse error: Unexpected token at line 42 column 15',
      userMessage: 'The document format is invalid or corrupted',
      details: {
        type: 'ParseError',
        details: {
          message: 'Unexpected token',
          line: 42,
          column: 15,
          expected: '}',
          found: 'EOF',
          filePath: 'document.rtf',
        },
      },
      suggestions: [
        'Check if the file is a valid RTF document',
        'Ensure the file is not corrupted',
      ],
      recoverable: false,
      timestamp: new Date().toISOString(),
    };
    setError(parseError);
  };

  const triggerConversionError = () => {
    const conversionError: ErrorResponse = {
      errorType: 'ConversionError',
      errorCode: -1002,
      message: 'Conversion error from RTF to Markdown: Complex table structure not supported',
      userMessage: 'Failed to convert from RTF to Markdown',
      details: {
        type: 'ConversionError',
        details: {
          sourceFormat: 'RTF',
          targetFormat: 'Markdown',
          details: 'Complex table structure not supported',
          recoverable: true,
          suggestions: [
            'Simplify the table structure',
            'Remove nested tables',
            'Use the table simplification option',
          ],
        },
      },
      suggestions: [
        'Simplify the table structure',
        'Remove nested tables',
        'Use the table simplification option',
      ],
      recoverable: true,
      timestamp: new Date().toISOString(),
    };
    setError(conversionError);
  };

  return (
    <div className="space-y-4">
      <div className="flex gap-2">
        <Button onClick={triggerParseError}>Trigger Parse Error</Button>
        <Button onClick={triggerConversionError}>Trigger Conversion Error</Button>
      </div>
      
      {error && (
        <UnifiedErrorDisplay
          error={error}
          onRetry={() => console.log('Retry clicked')}
          onDismiss={() => setError(null)}
        />
      )}
    </div>
  );
};