import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import userEvent from '@testing-library/user-event';
import { DragDropZone } from '@/components/DragDropZone';
import { ConversionProgress } from '@/components/ConversionProgress';
import { MarkdownPreview } from '@/components/MarkdownPreview';
import { ErrorBoundary } from '@/components/ErrorBoundary';
import { MonitoringDashboard } from '@/components/monitoring/MonitoringDashboard';
import { useFileStore } from '@/lib/stores/files';

// Add jest-axe matchers
expect.extend(toHaveNoViolations);

// Mock components for testing
jest.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

jest.mock('@/lib/stores/files');

// Mock child components for MonitoringDashboard
jest.mock('@/components/monitoring/BuildProgressRing', () => ({
  BuildProgressRing: () => <div>Build Progress</div>,
}));

jest.mock('@/components/monitoring/PerformanceChart', () => ({
  PerformanceChart: () => <div>Performance Chart</div>,
}));

jest.mock('@/components/monitoring/FunctionCallMatrix', () => ({
  FunctionCallMatrix: () => <div>Function Matrix</div>,
}));

jest.mock('@/components/monitoring/SystemHealthCard', () => ({
  SystemHealthCard: () => <div>System Health</div>,
}));

jest.mock('@/components/monitoring/LogStreamViewer', () => ({
  LogStreamViewer: () => <div>Log Stream</div>,
}));

jest.mock('../ErrorLogViewer', () => ({
  default: () => <div>Error Logs</div>,
}));

describe('Component Accessibility Tests', () => {
  const mockUseFileStore = useFileStore as unknown as jest.MockedFunction<typeof useFileStore>;
  
  beforeEach(() => {
    jest.clearAllMocks();
    mockUseFileStore.mockReturnValue({
      files: [],
      addFiles: jest.fn(),
      removeFile: jest.fn(),
      updateFileStatus: jest.fn(),
      updateFileProgress: jest.fn(),
      clearFiles: jest.fn(),
      getFileById: jest.fn(),
    });
  });

  describe('DragDropZone Accessibility', () => {
    it('should have no accessibility violations', async () => {
      const { container } = render(<DragDropZone />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have proper ARIA labels', () => {
      render(<DragDropZone />);
      
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      expect(dropZone).toBeInTheDocument();
      
      const fileInput = dropZone.parentElement?.querySelector('input[type="file"]');
      expect(fileInput).toHaveAttribute('aria-label');
    });

    it('should be keyboard navigable', async () => {
      const user = userEvent.setup();
      render(<DragDropZone />);
      
      // Tab to drop zone
      await user.tab();
      
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      expect(dropZone).toHaveFocus();
      
      // Should be able to activate with Enter/Space
      await user.keyboard('{Enter}');
      // File dialog would open in real browser
    });

    it('should announce drag state changes', () => {
      render(<DragDropZone />);
      
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      
      // Simulate drag enter
      fireEvent.dragEnter(dropZone, {
        dataTransfer: { files: [], items: [] }
      });
      
      // Should update text for screen readers
      expect(screen.getByText('Drop your files here')).toBeInTheDocument();
    });

    it('should have accessible file list', () => {
      const mockFiles = [{
        id: 'file-1',
        name: 'document.rtf',
        size: 1024,
        type: 'rtf',
        status: 'idle' as const,
        path: 'document.rtf',
      }];
      
      mockUseFileStore.mockReturnValue({
        files: mockFiles,
        addFiles: jest.fn(),
        removeFile: jest.fn(),
        updateFileStatus: jest.fn(),
        updateFileProgress: jest.fn(),
        clearFiles: jest.fn(),
        getFileById: jest.fn(),
      });
      
      render(<DragDropZone />);
      
      // File list should be in a semantic list
      const fileList = screen.getByRole('list');
      expect(fileList).toBeInTheDocument();
      
      // Remove button should have accessible label
      const removeButton = screen.getByRole('button', { name: '' });
      expect(removeButton).toHaveAttribute('aria-label');
    });
  });

  describe('ConversionProgress Accessibility', () => {
    it('should have no accessibility violations', async () => {
      const { container } = render(<ConversionProgress />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have proper ARIA attributes for progress bars', () => {
      const mockFiles = [{
        id: 'file-1',
        file: {
          name: 'test.rtf',
          size: 1024,
          type: 'application/rtf',
          lastModified: Date.now(),
        } as File,
        status: 'converting' as const,
        progress: 50,
      }];
      
      mockUseFileStore.mockReturnValue(mockFiles as any);
      
      render(<ConversionProgress />);
      
      const progressBars = screen.getAllByRole('progressbar');
      progressBars.forEach(bar => {
        expect(bar).toHaveAttribute('aria-valuenow');
        expect(bar).toHaveAttribute('aria-valuemin', '0');
        expect(bar).toHaveAttribute('aria-valuemax', '100');
        expect(bar).toHaveAttribute('aria-label');
      });
    });

    it('should be keyboard navigable for actions', async () => {
      const user = userEvent.setup();
      const onPreview = jest.fn();
      const onDownload = jest.fn();
      
      const mockFiles = [{
        id: 'file-1',
        file: {
          name: 'test.rtf',
          size: 1024,
          type: 'application/rtf',
          lastModified: Date.now(),
        } as File,
        status: 'completed' as const,
        progress: 100,
        output: 'Converted content',
      }];
      
      mockUseFileStore.mockReturnValue(mockFiles as any);
      
      render(<ConversionProgress onPreview={onPreview} onDownload={onDownload} />);
      
      // Tab to preview button
      await user.tab();
      const previewButton = screen.getByRole('button', { name: /preview/i });
      expect(previewButton).toHaveFocus();
      
      // Activate with Enter
      await user.keyboard('{Enter}');
      expect(onPreview).toHaveBeenCalled();
      
      // Tab to download button
      await user.tab();
      const downloadButton = screen.getByRole('button', { name: /download/i });
      expect(downloadButton).toHaveFocus();
      
      // Activate with Space
      await user.keyboard(' ');
      expect(onDownload).toHaveBeenCalled();
    });

    it('should announce status changes', () => {
      const mockFiles = [{
        id: 'file-1',
        file: {
          name: 'test.rtf',
          size: 1024,
          type: 'application/rtf',
          lastModified: Date.now(),
        } as File,
        status: 'error' as const,
        progress: 0,
        error: 'Conversion failed',
      }];
      
      mockUseFileStore.mockReturnValue(mockFiles as any);
      
      render(<ConversionProgress />);
      
      // Error state should be announced
      const alert = screen.getByRole('alert');
      expect(alert).toHaveTextContent('Conversion failed');
    });
  });

  describe('MarkdownPreview Accessibility', () => {
    it('should have no accessibility violations', async () => {
      const content = '# Heading\n\nParagraph with **bold** text.';
      const { container } = render(<MarkdownPreview content={content} />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have proper heading hierarchy', () => {
      const content = '# H1\n## H2\n### H3';
      const { container } = render(<MarkdownPreview content={content} />);
      
      const h1 = container.querySelector('h1');
      const h2 = container.querySelector('h2');
      const h3 = container.querySelector('h3');
      
      expect(h1).toBeInTheDocument();
      expect(h2).toBeInTheDocument();
      expect(h3).toBeInTheDocument();
    });

    it('should have accessible links', () => {
      const content = '[Visit Example](https://example.com)';
      const { container } = render(<MarkdownPreview content={content} />);
      
      const link = container.querySelector('a');
      expect(link).toHaveAttribute('href', 'https://example.com');
      expect(link).toHaveTextContent('Visit Example');
    });

    it('should be readable with line numbers', () => {
      const content = 'Line 1\nLine 2\nLine 3';
      render(<MarkdownPreview content={content} showLineNumbers />);
      
      // Line numbers should not interfere with content reading
      expect(screen.getByText('Line 1')).toBeInTheDocument();
      expect(screen.getByText('Line 2')).toBeInTheDocument();
      expect(screen.getByText('Line 3')).toBeInTheDocument();
      
      // Line numbers should be marked as decorative
      const lineNumbers = screen.getAllByText(/^[1-3]$/);
      lineNumbers.forEach(num => {
        expect(num).toHaveClass('select-none');
      });
    });
  });

  describe('ErrorBoundary Accessibility', () => {
    const ThrowError = () => {
      throw new Error('Test error');
    };

    it('should have no accessibility violations in error state', async () => {
      const { container } = render(
        <ErrorBoundary>
          <ThrowError />
        </ErrorBoundary>
      );
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have accessible error messages', () => {
      render(
        <ErrorBoundary>
          <ThrowError />
        </ErrorBoundary>
      );
      
      // Error should be in an alert
      const alert = screen.getByRole('alert');
      expect(alert).toBeInTheDocument();
      expect(alert).toHaveTextContent('Error Details');
      expect(alert).toHaveTextContent('Test error');
    });

    it('should have keyboard accessible recovery options', async () => {
      const user = userEvent.setup();
      
      render(
        <ErrorBoundary>
          <ThrowError />
        </ErrorBoundary>
      );
      
      // Tab to Try Again button
      await user.tab();
      const retryButton = screen.getByRole('button', { name: /try again/i });
      expect(retryButton).toHaveFocus();
      
      // Tab to Go Home button
      await user.tab();
      const homeButton = screen.getByRole('button', { name: /go home/i });
      expect(homeButton).toHaveFocus();
    });
  });

  describe('MonitoringDashboard Accessibility', () => {
    it('should have no accessibility violations', async () => {
      const { container } = render(<MonitoringDashboard />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have accessible tab navigation', async () => {
      const user = userEvent.setup();
      render(<MonitoringDashboard />);
      
      // Find tab list
      const tabList = screen.getByRole('tablist');
      expect(tabList).toBeInTheDocument();
      
      // All tabs should be accessible
      const tabs = screen.getAllByRole('tab');
      expect(tabs).toHaveLength(4);
      
      // Tab through tabs
      await user.tab();
      expect(tabs[0]).toHaveFocus();
      
      // Arrow navigation
      await user.keyboard('{ArrowRight}');
      expect(tabs[1]).toHaveFocus();
    });

    it('should have proper heading hierarchy', () => {
      render(<MonitoringDashboard />);
      
      const mainHeading = screen.getByRole('heading', { level: 2 });
      expect(mainHeading).toHaveTextContent('LegacyBridge Monitor');
    });

    it('should have accessible metric cards', () => {
      render(<MonitoringDashboard />);
      
      // Metric labels should be associated with values
      expect(screen.getByText('Conversions/sec')).toBeInTheDocument();
      expect(screen.getByText('Memory Usage')).toBeInTheDocument();
      expect(screen.getByText('Active Builds')).toBeInTheDocument();
      expect(screen.getByText('System Health')).toBeInTheDocument();
    });
  });

  describe('Focus Management', () => {
    it('should trap focus in modals and dialogs', async () => {
      const user = userEvent.setup();
      
      // Example with ErrorBoundary as a modal-like component
      render(
        <ErrorBoundary>
          <div>
            <button>Outside button</button>
            {(() => { throw new Error('Test'); })()}
          </div>
        </ErrorBoundary>
      );
      
      // Focus should be contained within error boundary
      await user.tab();
      expect(screen.getByRole('button', { name: /try again/i })).toHaveFocus();
      
      await user.tab();
      expect(screen.getByRole('button', { name: /go home/i })).toHaveFocus();
      
      // Should cycle back
      await user.tab();
      expect(screen.getByRole('button', { name: /try again/i })).toHaveFocus();
    });
  });

  describe('Color Contrast', () => {
    it('should have sufficient color contrast for text', async () => {
      // This would require a more sophisticated setup with actual styles
      // For now, we ensure components render with proper structure
      
      const { container } = render(
        <div>
          <DragDropZone />
          <ConversionProgress />
          <MarkdownPreview content="# Test Content" />
        </div>
      );
      
      // Run axe with color contrast rules
      const results = await axe(container, {
        rules: {
          'color-contrast': { enabled: true },
        },
      });
      
      // Filter out known issues (if any)
      const violations = results.violations.filter(
        v => v.id === 'color-contrast'
      );
      
      expect(violations).toHaveLength(0);
    });
  });

  describe('Screen Reader Announcements', () => {
    it('should announce file upload status', async () => {
      render(<DragDropZone />);
      
      const file = new File(['content'], 'test.rtf', { type: 'application/rtf' });
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      
      Object.defineProperty(input, 'files', {
        value: [file],
        writable: false,
      });
      
      fireEvent.change(input);
      
      // Status should be announced
      await screen.findByText('test.rtf');
      
      // File count should be announced
      expect(screen.getByText('Selected Files (1)')).toBeInTheDocument();
    });

    it('should announce conversion completion', async () => {
      const mockFiles = [{
        id: 'file-1',
        file: {
          name: 'test.rtf',
          size: 1024,
          type: 'application/rtf',
          lastModified: Date.now(),
        } as File,
        status: 'completed' as const,
        progress: 100,
        conversionTime: 2.5,
      }];
      
      mockUseFileStore.mockReturnValue(mockFiles as any);
      
      render(<ConversionProgress />);
      
      // Completion status should be visible
      expect(screen.getByText('completed')).toBeInTheDocument();
      expect(screen.getByText('Converted in 2.5s')).toBeInTheDocument();
    });
  });

  describe('High Contrast Mode', () => {
    it('should work in high contrast mode', async () => {
      // Simulate high contrast mode
      const mediaQuery = window.matchMedia('(prefers-contrast: high)');
      Object.defineProperty(mediaQuery, 'matches', {
        writable: true,
        value: true,
      });
      
      const { container } = render(
        <div>
          <DragDropZone />
          <ConversionProgress />
          <MonitoringDashboard />
        </div>
      );
      
      // Components should still be accessible
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });
  });
});