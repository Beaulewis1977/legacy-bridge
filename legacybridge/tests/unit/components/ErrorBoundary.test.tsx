import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { jest } from '@jest/globals';
import userEvent from '@testing-library/user-event';
import { ErrorBoundary, ConversionErrorBoundary, useErrorHandler, withAsyncErrorBoundary } from '@/components/ErrorBoundary';
import { logger } from '@/lib/error-logger';

// Mock the error logger
jest.mock('@/lib/error-logger', () => ({
  logger: {
    fatal: jest.fn(),
    error: jest.fn(),
  },
}));

// Mock window.location
delete (window as any).location;
window.location = { href: '/' } as any;

// Test component that throws an error
const ThrowError: React.FC<{ shouldThrow?: boolean; error?: Error }> = ({ 
  shouldThrow = true, 
  error = new Error('Test error') 
}) => {
  if (shouldThrow) {
    throw error;
  }
  return <div>No error</div>;
};

// Test component that throws error in useEffect
const ThrowErrorInEffect: React.FC<{ delay?: number }> = ({ delay = 0 }) => {
  React.useEffect(() => {
    const timer = setTimeout(() => {
      throw new Error('Effect error');
    }, delay);
    return () => clearTimeout(timer);
  }, [delay]);
  
  return <div>Component rendered</div>;
};

// Test component for error hook
const ComponentWithErrorHook: React.FC<{ triggerError?: boolean }> = ({ triggerError = false }) => {
  const throwError = useErrorHandler();
  
  React.useEffect(() => {
    if (triggerError) {
      throwError(new Error('Hook error'));
    }
  }, [triggerError, throwError]);
  
  return <div>Component with hook</div>;
};

describe('ErrorBoundary Component', () => {
  const mockConsoleError = jest.spyOn(console, 'error').mockImplementation(() => {});
  const mockConsoleLog = jest.spyOn(console, 'log').mockImplementation(() => {});
  
  beforeEach(() => {
    jest.clearAllMocks();
    localStorage.clear();
    // Reset NODE_ENV for each test
    process.env.NODE_ENV = 'test';
  });

  afterAll(() => {
    mockConsoleError.mockRestore();
    mockConsoleLog.mockRestore();
  });

  describe('Error Catching', () => {
    it('should catch errors and display fallback UI', () => {
      render(
        <ErrorBoundary>
          <ThrowError />
        </ErrorBoundary>
      );

      expect(screen.getByText('Something went wrong')).toBeInTheDocument();
      expect(screen.getByText(/Error ID:/)).toBeInTheDocument();
      expect(screen.getByText('Test error')).toBeInTheDocument();
    });

    it('should display custom error message', () => {
      const customError = new Error('Custom error message');
      
      render(
        <ErrorBoundary>
          <ThrowError error={customError} />
        </ErrorBoundary>
      );

      expect(screen.getByText('Custom error message')).toBeInTheDocument();
    });

    it('should generate unique error IDs', () => {
      const { rerender } = render(
        <ErrorBoundary>
          <ThrowError />
        </ErrorBoundary>
      );

      const firstErrorId = screen.getByText(/Error ID:/).textContent;
      
      // Force re-render with new error
      rerender(
        <ErrorBoundary>
          <ThrowError error={new Error('Another error')} />
        </ErrorBoundary>
      );

      const secondErrorId = screen.getByText(/Error ID:/).textContent;
      expect(firstErrorId).not.toBe(secondErrorId);
    });

    it('should render children when no error', () => {
      render(
        <ErrorBoundary>
          <div>Child content</div>
        </ErrorBoundary>
      );

      expect(screen.getByText('Child content')).toBeInTheDocument();
      expect(screen.queryByText('Something went wrong')).not.toBeInTheDocument();
    });
  });

  describe('Error Recovery', () => {
    it('should recover when retry button is clicked', async () => {
      let shouldThrow = true;
      const { rerender } = render(
        <ErrorBoundary>
          <ThrowError shouldThrow={shouldThrow} />
        </ErrorBoundary>
      );

      expect(screen.getByText('Something went wrong')).toBeInTheDocument();
      
      // Update the variable to prevent throwing
      shouldThrow = false;
      
      // Click retry
      const retryButton = screen.getByRole('button', { name: /try again/i });
      fireEvent.click(retryButton);
      
      // Re-render with updated prop
      rerender(
        <ErrorBoundary>
          <ThrowError shouldThrow={shouldThrow} />
        </ErrorBoundary>
      );

      await waitFor(() => {
        expect(screen.queryByText('Something went wrong')).not.toBeInTheDocument();
        expect(screen.getByText('No error')).toBeInTheDocument();
      });
    });

    it('should navigate home when home button is clicked', () => {
      render(
        <ErrorBoundary>
          <ThrowError />
        </ErrorBoundary>
      );

      const homeButton = screen.getByRole('button', { name: /go home/i });
      fireEvent.click(homeButton);

      expect(window.location.href).toBe('/');
    });
  });

  describe('Error Logging', () => {
    it('should log errors with logger', () => {
      const testError = new Error('Test logging error');
      
      render(
        <ErrorBoundary>
          <ThrowError error={testError} />
        </ErrorBoundary>
      );

      expect(logger.fatal).toHaveBeenCalledWith(
        'ErrorBoundary',
        'Unhandled error caught by error boundary',
        testError,
        expect.objectContaining({
          errorId: expect.stringMatching(/^error-\d+-\w+$/),
          componentStack: expect.any(String),
          url: expect.any(String),
          userAgent: expect.any(String)
        })
      );
    });

    it('should store errors in localStorage', () => {
      const testError = new Error('LocalStorage test error');
      
      render(
        <ErrorBoundary>
          <ThrowError error={testError} />
        </ErrorBoundary>
      );

      const storedErrors = JSON.parse(localStorage.getItem('legacybridge_errors') || '[]');
      expect(storedErrors).toHaveLength(1);
      expect(storedErrors[0]).toMatchObject({
        id: expect.stringMatching(/^error-\d+-\w+$/),
        timestamp: expect.any(String),
        error: {
          message: 'LocalStorage test error',
          stack: expect.any(String)
        },
        errorInfo: {
          componentStack: expect.any(String)
        },
        userAgent: expect.any(String),
        url: expect.any(String)
      });
    });

    it('should limit stored errors to 10', () => {
      // Add 12 errors
      for (let i = 0; i < 12; i++) {
        const { unmount } = render(
          <ErrorBoundary>
            <ThrowError error={new Error(`Error ${i}`)} />
          </ErrorBoundary>
        );
        unmount();
      }

      const storedErrors = JSON.parse(localStorage.getItem('legacybridge_errors') || '[]');
      expect(storedErrors).toHaveLength(10);
      expect(storedErrors[0].error.message).toBe('Error 11');
      expect(storedErrors[9].error.message).toBe('Error 2');
    });

    it('should handle localStorage errors gracefully', () => {
      // Mock localStorage to throw error
      const originalSetItem = Storage.prototype.setItem;
      Storage.prototype.setItem = jest.fn(() => {
        throw new Error('Storage full');
      });

      // Should not throw when localStorage fails
      expect(() => {
        render(
          <ErrorBoundary>
            <ThrowError />
          </ErrorBoundary>
        );
      }).not.toThrow();

      Storage.prototype.setItem = originalSetItem;
    });
  });

  describe('Custom Error Handler', () => {
    it('should call custom onError handler', () => {
      const onError = jest.fn();
      const testError = new Error('Custom handler test');
      
      render(
        <ErrorBoundary onError={onError}>
          <ThrowError error={testError} />
        </ErrorBoundary>
      );

      expect(onError).toHaveBeenCalledWith(
        testError,
        expect.objectContaining({
          componentStack: expect.any(String)
        }),
        expect.stringMatching(/^error-\d+-\w+$/)
      );
    });
  });

  describe('Custom Fallback Component', () => {
    const CustomFallback: React.FC<{ error?: Error; onRetry: () => void }> = ({ error, onRetry }) => (
      <div>
        <h1>Custom Error UI</h1>
        <p>{error?.message}</p>
        <button onClick={onRetry}>Custom Retry</button>
      </div>
    );

    it('should render custom fallback component', () => {
      render(
        <ErrorBoundary fallback={CustomFallback}>
          <ThrowError />
        </ErrorBoundary>
      );

      expect(screen.getByText('Custom Error UI')).toBeInTheDocument();
      expect(screen.getByText('Test error')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Custom Retry' })).toBeInTheDocument();
    });

    it('should pass all props to custom fallback', () => {
      const CustomFallbackWithProps: React.FC<any> = ({ error, errorInfo, errorId, onRetry }) => (
        <div>
          <div>Error: {error?.message}</div>
          <div>Has errorInfo: {errorInfo ? 'yes' : 'no'}</div>
          <div>Error ID: {errorId}</div>
          <button onClick={onRetry}>Retry</button>
        </div>
      );

      render(
        <ErrorBoundary fallback={CustomFallbackWithProps}>
          <ThrowError />
        </ErrorBoundary>
      );

      expect(screen.getByText('Error: Test error')).toBeInTheDocument();
      expect(screen.getByText('Has errorInfo: yes')).toBeInTheDocument();
      expect(screen.getByText(/^Error ID: error-\d+-\w+$/)).toBeInTheDocument();
    });
  });

  describe('Development vs Production', () => {
    it('should show stack trace in development', () => {
      process.env.NODE_ENV = 'development';
      
      render(
        <ErrorBoundary>
          <ThrowError />
        </ErrorBoundary>
      );

      expect(screen.getByText('Show stack trace')).toBeInTheDocument();
    });

    it('should hide stack trace in production', () => {
      process.env.NODE_ENV = 'production';
      
      render(
        <ErrorBoundary>
          <ThrowError />
        </ErrorBoundary>
      );

      expect(screen.queryByText('Show stack trace')).not.toBeInTheDocument();
    });

    it('should expand/collapse stack trace', async () => {
      process.env.NODE_ENV = 'development';
      const user = userEvent.setup();
      
      render(
        <ErrorBoundary>
          <ThrowError />
        </ErrorBoundary>
      );

      const details = screen.getByText('Show stack trace');
      
      // Stack trace should be hidden initially
      expect(screen.queryByText(/at ThrowError/)).not.toBeInTheDocument();
      
      // Click to expand
      await user.click(details);
      
      // Stack trace should be visible
      await waitFor(() => {
        const preElements = screen.getAllByText((content, element) => {
          return element?.tagName === 'PRE' && content.includes('Error');
        });
        expect(preElements.length).toBeGreaterThan(0);
      });
    });
  });
});

describe('ConversionErrorBoundary Component', () => {
  const mockConsoleError = jest.spyOn(console, 'error').mockImplementation(() => {});
  const mockConsoleLog = jest.spyOn(console, 'log').mockImplementation(() => {});
  
  beforeEach(() => {
    jest.clearAllMocks();
  });

  afterAll(() => {
    mockConsoleError.mockRestore();
    mockConsoleLog.mockRestore();
  });

  it('should catch conversion errors with specialized message', () => {
    render(
      <ConversionErrorBoundary>
        <ThrowError error={new Error('Conversion failed')} />
      </ConversionErrorBoundary>
    );

    expect(screen.getByText('Conversion Error')).toBeInTheDocument();
    expect(screen.getByText(/Failed to process the file conversion/)).toBeInTheDocument();
    expect(screen.getByText('Error: Conversion failed')).toBeInTheDocument();
  });

  it('should log conversion errors with proper context', () => {
    const conversionError = new Error('RTF parsing failed');
    
    render(
      <ConversionErrorBoundary>
        <ThrowError error={conversionError} />
      </ConversionErrorBoundary>
    );

    expect(logger.error).toHaveBeenCalledWith(
      'ConversionBoundary',
      'Conversion error caught by error boundary',
      conversionError,
      expect.objectContaining({
        errorId: expect.stringMatching(/^error-\d+-\w+$/),
        componentStack: expect.any(String),
        errorType: 'conversion_error'
      })
    );
  });

  it('should allow retry for conversion errors', async () => {
    let shouldThrow = true;
    const { rerender } = render(
      <ConversionErrorBoundary>
        <ThrowError shouldThrow={shouldThrow} />
      </ConversionErrorBoundary>
    );

    expect(screen.getByText('Conversion Error')).toBeInTheDocument();
    
    shouldThrow = false;
    const retryButton = screen.getByRole('button', { name: /retry/i });
    fireEvent.click(retryButton);
    
    rerender(
      <ConversionErrorBoundary>
        <ThrowError shouldThrow={shouldThrow} />
      </ConversionErrorBoundary>
    );

    await waitFor(() => {
      expect(screen.queryByText('Conversion Error')).not.toBeInTheDocument();
      expect(screen.getByText('No error')).toBeInTheDocument();
    });
  });

  it('should render children when no conversion error', () => {
    render(
      <ConversionErrorBoundary>
        <div>Conversion content</div>
      </ConversionErrorBoundary>
    );

    expect(screen.getByText('Conversion content')).toBeInTheDocument();
    expect(screen.queryByText('Conversion Error')).not.toBeInTheDocument();
  });
});

describe('useErrorHandler Hook', () => {
  it('should throw errors and log them', () => {
    const TestComponent = () => {
      const throwError = useErrorHandler();
      
      return (
        <button onClick={() => throwError(new Error('Hook error'))}>
          Throw Error
        </button>
      );
    };

    render(
      <ErrorBoundary>
        <TestComponent />
      </ErrorBoundary>
    );

    const button = screen.getByRole('button', { name: 'Throw Error' });
    fireEvent.click(button);

    expect(logger.error).toHaveBeenCalledWith(
      'ProgrammaticError',
      'Error thrown programmatically',
      expect.objectContaining({ message: 'Hook error' }),
      { source: 'useErrorHandler' }
    );

    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
    expect(screen.getByText('Hook error')).toBeInTheDocument();
  });
});

describe('withAsyncErrorBoundary HOC', () => {
  const AsyncComponent: React.FC<{ shouldError?: boolean }> = ({ shouldError = false }) => {
    if (shouldError) {
      throw new Error('Async component error');
    }
    return <div>Async content</div>;
  };

  it('should wrap component with error boundary', () => {
    const WrappedComponent = withAsyncErrorBoundary(AsyncComponent);
    
    render(<WrappedComponent />);
    
    expect(screen.getByText('Async content')).toBeInTheDocument();
  });

  it('should catch errors in wrapped component', () => {
    const WrappedComponent = withAsyncErrorBoundary(AsyncComponent);
    
    render(<WrappedComponent shouldError />);
    
    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
    expect(screen.getByText('Async component error')).toBeInTheDocument();
  });

  it('should use custom fallback when provided', () => {
    const CustomFallback: React.FC<any> = ({ error }) => (
      <div>Custom async error: {error?.message}</div>
    );
    
    const WrappedComponent = withAsyncErrorBoundary(AsyncComponent, CustomFallback);
    
    render(<WrappedComponent shouldError />);
    
    expect(screen.getByText('Custom async error: Async component error')).toBeInTheDocument();
  });

  it('should pass props to wrapped component', () => {
    interface TestProps {
      message: string;
    }
    
    const TestComponent: React.FC<TestProps> = ({ message }) => <div>{message}</div>;
    const WrappedComponent = withAsyncErrorBoundary(TestComponent);
    
    render(<WrappedComponent message="Test prop" />);
    
    expect(screen.getByText('Test prop')).toBeInTheDocument();
  });
});

describe('Edge Cases', () => {
  it('should handle errors without message', () => {
    const errorWithoutMessage = new Error();
    errorWithoutMessage.message = '';
    
    render(
      <ErrorBoundary>
        <ThrowError error={errorWithoutMessage} />
      </ErrorBoundary>
    );

    expect(screen.getByText('An unexpected error occurred')).toBeInTheDocument();
  });

  it('should handle null error gracefully', () => {
    // Simulate a case where error might be null
    const NullErrorComponent = () => {
      throw null;
    };

    render(
      <ErrorBoundary>
        <NullErrorComponent />
      </ErrorBoundary>
    );

    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
    expect(screen.getByText('An unexpected error occurred')).toBeInTheDocument();
  });

  it('should handle errors thrown in event handlers', async () => {
    const ComponentWithEventError = () => {
      const handleClick = () => {
        throw new Error('Event handler error');
      };

      return <button onClick={handleClick}>Click me</button>;
    };

    // Event handler errors are not caught by error boundaries
    // This is expected React behavior
    render(
      <ErrorBoundary>
        <ComponentWithEventError />
      </ErrorBoundary>
    );

    const button = screen.getByRole('button', { name: 'Click me' });
    
    // This will throw and not be caught by ErrorBoundary
    expect(() => fireEvent.click(button)).toThrow('Event handler error');
    
    // Error boundary should not activate
    expect(screen.queryByText('Something went wrong')).not.toBeInTheDocument();
  });

  it('should handle multiple simultaneous errors', () => {
    const MultiErrorComponent = () => {
      throw new Error('First error');
      // Second throw is unreachable but tests error boundary behavior
    };

    render(
      <ErrorBoundary>
        <MultiErrorComponent />
        <ThrowError error={new Error('Second error')} />
      </ErrorBoundary>
    );

    // Should catch the first error
    expect(screen.getByText('First error')).toBeInTheDocument();
  });

  it('should maintain error boundary isolation', () => {
    render(
      <div>
        <ErrorBoundary>
          <ThrowError error={new Error('Isolated error 1')} />
        </ErrorBoundary>
        <ErrorBoundary>
          <div>This should render</div>
        </ErrorBoundary>
      </div>
    );

    expect(screen.getByText('Isolated error 1')).toBeInTheDocument();
    expect(screen.getByText('This should render')).toBeInTheDocument();
  });
});

describe('Accessibility', () => {
  it('should have proper ARIA labels and roles', () => {
    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    const alert = screen.getByRole('alert');
    expect(alert).toBeInTheDocument();
    
    const buttons = screen.getAllByRole('button');
    expect(buttons).toHaveLength(2);
    expect(buttons[0]).toHaveAccessibleName(/try again/i);
    expect(buttons[1]).toHaveAccessibleName(/go home/i);
  });

  it('should be keyboard navigable', async () => {
    const user = userEvent.setup();
    
    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    // Tab through interactive elements
    await user.tab();
    expect(screen.getByRole('button', { name: /try again/i })).toHaveFocus();
    
    await user.tab();
    expect(screen.getByRole('button', { name: /go home/i })).toHaveFocus();
  });

  it('should announce errors to screen readers', () => {
    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('Error Details');
    expect(alert).toHaveTextContent('Test error');
  });
});