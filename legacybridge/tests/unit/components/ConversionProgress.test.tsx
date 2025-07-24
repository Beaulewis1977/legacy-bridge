import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { jest } from '@jest/globals';
import userEvent from '@testing-library/user-event';
import { ConversionProgress } from '@/components/ConversionProgress';
import { useFileStore } from '@/lib/stores/files';
import { ProcessedFile } from '@/lib/stores/files';

// Mock the file store
jest.mock('@/lib/stores/files');

// Mock framer-motion to avoid animation issues in tests
jest.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

describe('ConversionProgress Component', () => {
  const mockUseFileStore = useFileStore as unknown as jest.MockedFunction<typeof useFileStore>;
  
  const mockFiles: ProcessedFile[] = [
    {
      id: 'file-1',
      file: {
        name: 'document1.rtf',
        size: 1024 * 100, // 100KB
        type: 'application/rtf',
        lastModified: Date.now(),
      } as File,
      status: 'completed',
      progress: 100,
      conversionTime: 2.5,
      output: 'Converted markdown content',
    },
    {
      id: 'file-2',
      file: {
        name: 'document2.rtf',
        size: 1024 * 250, // 250KB
        type: 'application/rtf',
        lastModified: Date.now(),
      } as File,
      status: 'converting',
      progress: 45,
    },
    {
      id: 'file-3',
      file: {
        name: 'document3.rtf',
        size: 1024 * 500, // 500KB
        type: 'application/rtf',
        lastModified: Date.now(),
      } as File,
      status: 'error',
      progress: 0,
      error: 'Invalid RTF format',
    },
  ];

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Progress Display', () => {
    it('should display overall progress percentage correctly', () => {
      mockUseFileStore.mockReturnValue(mockFiles);
      
      render(<ConversionProgress />);
      
      // 1 completed out of 3 files = 33%
      expect(screen.getByText('33%')).toBeInTheDocument();
      expect(screen.getByText('Overall Progress')).toBeInTheDocument();
    });

    it('should show correct file count badge', () => {
      mockUseFileStore.mockReturnValue(mockFiles);
      
      render(<ConversionProgress />);
      
      expect(screen.getByText('1/3 files')).toBeInTheDocument();
    });

    it('should display progress bar with correct value', () => {
      mockUseFileStore.mockReturnValue(mockFiles);
      
      render(<ConversionProgress />);
      
      const progressBar = screen.getByRole('progressbar');
      expect(progressBar).toHaveAttribute('aria-valuenow', '33');
    });

    it('should handle empty file list', () => {
      mockUseFileStore.mockReturnValue([]);
      
      render(<ConversionProgress />);
      
      expect(screen.getByText('0%')).toBeInTheDocument();
      expect(screen.getByText('0/0 files')).toBeInTheDocument();
    });

    it('should show ETA when files are converting', () => {
      mockUseFileStore.mockReturnValue(mockFiles);
      
      render(<ConversionProgress />);
      
      expect(screen.getByText(/ETA:/)).toBeInTheDocument();
    });
  });

  describe('File Status Display', () => {
    it('should display all files with correct status icons', () => {
      mockUseFileStore.mockReturnValue(mockFiles);
      
      render(<ConversionProgress />);
      
      // Check file names
      expect(screen.getByText('document1.rtf')).toBeInTheDocument();
      expect(screen.getByText('document2.rtf')).toBeInTheDocument();
      expect(screen.getByText('document3.rtf')).toBeInTheDocument();
      
      // Check status badges
      expect(screen.getByText('completed')).toBeInTheDocument();
      expect(screen.getByText('converting')).toBeInTheDocument();
      expect(screen.getByText('error')).toBeInTheDocument();
    });

    it('should show progress bar for converting files', () => {
      mockUseFileStore.mockReturnValue([mockFiles[1]]); // Only converting file
      
      render(<ConversionProgress />);
      
      const progressBars = screen.getAllByRole('progressbar');
      expect(progressBars).toHaveLength(2); // Overall progress + file progress
      expect(progressBars[1]).toHaveAttribute('aria-valuenow', '45');
    });

    it('should display file sizes correctly', () => {
      mockUseFileStore.mockReturnValue(mockFiles);
      
      render(<ConversionProgress />);
      
      expect(screen.getByText('100.0 KB')).toBeInTheDocument();
      expect(screen.getByText('250.0 KB')).toBeInTheDocument();
      expect(screen.getByText('500.0 KB')).toBeInTheDocument();
    });

    it('should display conversion time for completed files', () => {
      mockUseFileStore.mockReturnValue([mockFiles[0]]);
      
      render(<ConversionProgress />);
      
      expect(screen.getByText('Converted in 2.5s')).toBeInTheDocument();
    });

    it('should display error message for failed files', () => {
      mockUseFileStore.mockReturnValue([mockFiles[2]]);
      
      render(<ConversionProgress />);
      
      expect(screen.getByText('Error: Invalid RTF format')).toBeInTheDocument();
    });
  });

  describe('File Actions', () => {
    it('should show preview and download buttons for completed files', () => {
      mockUseFileStore.mockReturnValue([mockFiles[0]]);
      const onPreview = jest.fn();
      const onDownload = jest.fn();
      
      render(
        <ConversionProgress 
          onPreview={onPreview} 
          onDownload={onDownload} 
        />
      );
      
      expect(screen.getByRole('button', { name: /preview/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /download/i })).toBeInTheDocument();
    });

    it('should call onPreview when preview button is clicked', async () => {
      mockUseFileStore.mockReturnValue([mockFiles[0]]);
      const onPreview = jest.fn();
      const user = userEvent.setup();
      
      render(<ConversionProgress onPreview={onPreview} />);
      
      await user.click(screen.getByRole('button', { name: /preview/i }));
      
      expect(onPreview).toHaveBeenCalledWith(mockFiles[0]);
      expect(onPreview).toHaveBeenCalledTimes(1);
    });

    it('should call onDownload when download button is clicked', async () => {
      mockUseFileStore.mockReturnValue([mockFiles[0]]);
      const onDownload = jest.fn();
      const user = userEvent.setup();
      
      render(<ConversionProgress onDownload={onDownload} />);
      
      await user.click(screen.getByRole('button', { name: /download/i }));
      
      expect(onDownload).toHaveBeenCalledWith(mockFiles[0]);
      expect(onDownload).toHaveBeenCalledTimes(1);
    });

    it('should show retry button for error files', () => {
      mockUseFileStore.mockReturnValue([mockFiles[2]]);
      const onRetry = jest.fn();
      
      render(<ConversionProgress onRetry={onRetry} />);
      
      expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
    });

    it('should call onRetry when retry button is clicked', async () => {
      mockUseFileStore.mockReturnValue([mockFiles[2]]);
      const onRetry = jest.fn();
      const user = userEvent.setup();
      
      render(<ConversionProgress onRetry={onRetry} />);
      
      await user.click(screen.getByRole('button', { name: /retry/i }));
      
      expect(onRetry).toHaveBeenCalledWith(mockFiles[2]);
      expect(onRetry).toHaveBeenCalledTimes(1);
    });

    it('should not show action buttons when callbacks are not provided', () => {
      mockUseFileStore.mockReturnValue(mockFiles);
      
      render(<ConversionProgress />);
      
      expect(screen.queryByRole('button', { name: /preview/i })).not.toBeInTheDocument();
      expect(screen.queryByRole('button', { name: /download/i })).not.toBeInTheDocument();
      expect(screen.queryByRole('button', { name: /retry/i })).not.toBeInTheDocument();
    });
  });

  describe('Summary Statistics', () => {
    it('should display correct summary statistics', () => {
      mockUseFileStore.mockReturnValue(mockFiles);
      
      render(<ConversionProgress />);
      
      // Check summary numbers
      expect(screen.getByText('1')).toBeInTheDocument(); // Completed
      expect(screen.getByText('1')).toBeInTheDocument(); // Processing
      expect(screen.getByText('1')).toBeInTheDocument(); // Failed
      
      // Check labels
      expect(screen.getByText('Completed')).toBeInTheDocument();
      expect(screen.getByText('Processing')).toBeInTheDocument();
      expect(screen.getByText('Failed')).toBeInTheDocument();
    });

    it('should not display summary when no files', () => {
      mockUseFileStore.mockReturnValue([]);
      
      render(<ConversionProgress />);
      
      expect(screen.queryByText('Completed')).not.toBeInTheDocument();
      expect(screen.queryByText('Processing')).not.toBeInTheDocument();
      expect(screen.queryByText('Failed')).not.toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have proper ARIA labels for progress bars', () => {
      mockUseFileStore.mockReturnValue(mockFiles);
      
      render(<ConversionProgress />);
      
      const progressBars = screen.getAllByRole('progressbar');
      progressBars.forEach(bar => {
        expect(bar).toHaveAttribute('aria-valuenow');
        expect(bar).toHaveAttribute('aria-valuemin', '0');
        expect(bar).toHaveAttribute('aria-valuemax', '100');
      });
    });

    it('should be keyboard navigable', async () => {
      mockUseFileStore.mockReturnValue([mockFiles[0]]);
      const onPreview = jest.fn();
      const user = userEvent.setup();
      
      render(<ConversionProgress onPreview={onPreview} />);
      
      // Tab to preview button
      await user.tab();
      expect(screen.getByRole('button', { name: /preview/i })).toHaveFocus();
      
      // Press Enter to activate
      await user.keyboard('{Enter}');
      expect(onPreview).toHaveBeenCalled();
    });

    it('should announce status changes to screen readers', () => {
      mockUseFileStore.mockReturnValue(mockFiles);
      
      render(<ConversionProgress />);
      
      // Check for status text that would be announced
      expect(screen.getByText('completed')).toHaveAttribute('class', expect.stringContaining('badge'));
      expect(screen.getByText('converting')).toHaveAttribute('class', expect.stringContaining('badge'));
      expect(screen.getByText('error')).toHaveAttribute('class', expect.stringContaining('badge'));
    });
  });

  describe('Edge Cases', () => {
    it('should handle very large file sizes', () => {
      const largeFile: ProcessedFile = {
        id: 'large-file',
        file: {
          name: 'large.rtf',
          size: 1024 * 1024 * 100, // 100MB
          type: 'application/rtf',
          lastModified: Date.now(),
        } as File,
        status: 'completed',
        progress: 100,
      };
      
      mockUseFileStore.mockReturnValue([largeFile]);
      
      render(<ConversionProgress />);
      
      expect(screen.getByText('100.0 MB')).toBeInTheDocument();
    });

    it('should handle very small file sizes', () => {
      const smallFile: ProcessedFile = {
        id: 'small-file',
        file: {
          name: 'small.rtf',
          size: 500, // 500 bytes
          type: 'application/rtf',
          lastModified: Date.now(),
        } as File,
        status: 'completed',
        progress: 100,
      };
      
      mockUseFileStore.mockReturnValue([smallFile]);
      
      render(<ConversionProgress />);
      
      expect(screen.getByText('500 B')).toBeInTheDocument();
    });

    it('should handle long file names gracefully', () => {
      const longNameFile: ProcessedFile = {
        id: 'long-name',
        file: {
          name: 'this-is-a-very-long-file-name-that-should-be-truncated-properly-in-the-ui.rtf',
          size: 1024,
          type: 'application/rtf',
          lastModified: Date.now(),
        } as File,
        status: 'completed',
        progress: 100,
      };
      
      mockUseFileStore.mockReturnValue([longNameFile]);
      
      render(<ConversionProgress />);
      
      const fileName = screen.getByText(/this-is-a-very-long-file-name/);
      expect(fileName).toHaveClass('truncate');
    });

    it('should handle negative progress values', () => {
      const invalidFile: ProcessedFile = {
        id: 'invalid',
        file: {
          name: 'invalid.rtf',
          size: 1024,
          type: 'application/rtf',
          lastModified: Date.now(),
        } as File,
        status: 'converting',
        progress: -10,
      };
      
      mockUseFileStore.mockReturnValue([invalidFile]);
      
      render(<ConversionProgress />);
      
      const progressBar = screen.getAllByRole('progressbar')[1];
      expect(progressBar).toHaveAttribute('aria-valuenow', '0');
    });

    it('should handle progress values over 100', () => {
      const overflowFile: ProcessedFile = {
        id: 'overflow',
        file: {
          name: 'overflow.rtf',
          size: 1024,
          type: 'application/rtf',
          lastModified: Date.now(),
        } as File,
        status: 'converting',
        progress: 150,
      };
      
      mockUseFileStore.mockReturnValue([overflowFile]);
      
      render(<ConversionProgress />);
      
      const progressBar = screen.getAllByRole('progressbar')[1];
      expect(progressBar).toHaveAttribute('aria-valuenow', '100');
    });
  });

  describe('Custom Styling', () => {
    it('should apply custom className', () => {
      mockUseFileStore.mockReturnValue([]);
      
      const { container } = render(<ConversionProgress className="custom-class" />);
      
      expect(container.firstChild).toHaveClass('custom-class');
    });
  });

  describe('Performance', () => {
    it('should handle large number of files efficiently', () => {
      const manyFiles: ProcessedFile[] = Array.from({ length: 100 }, (_, i) => ({
        id: `file-${i}`,
        file: {
          name: `document${i}.rtf`,
          size: 1024 * (i + 1),
          type: 'application/rtf',
          lastModified: Date.now(),
        } as File,
        status: i % 3 === 0 ? 'completed' : i % 3 === 1 ? 'converting' : 'error',
        progress: i % 3 === 0 ? 100 : i % 3 === 1 ? i : 0,
      }));
      
      mockUseFileStore.mockReturnValue(manyFiles);
      
      const startTime = performance.now();
      render(<ConversionProgress />);
      const renderTime = performance.now() - startTime;
      
      // Should render within reasonable time
      expect(renderTime).toBeLessThan(1000);
      
      // Check that files are rendered
      expect(screen.getByText('document0.rtf')).toBeInTheDocument();
      expect(screen.getByText('document99.rtf')).toBeInTheDocument();
    });
  });
});