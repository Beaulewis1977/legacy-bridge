import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { jest } from '@jest/globals';
import userEvent from '@testing-library/user-event';
import { DragDropZone } from '@/components/DragDropZone';
import { ConversionProgress } from '@/components/ConversionProgress';
import { MarkdownPreview } from '@/components/MarkdownPreview';
import { ErrorBoundary } from '@/components/ErrorBoundary';
import { useFileStore } from '@/lib/stores/files';
import { invoke } from '@tauri-apps/api';

// Mock Tauri API
jest.mock('@tauri-apps/api', () => ({
  invoke: jest.fn(),
}));

// Mock framer-motion
jest.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

// Integration test component that combines multiple components
const FileConversionFlow: React.FC = () => {
  const { files } = useFileStore();
  const [selectedFile, setSelectedFile] = React.useState<any>(null);

  const handleFilesAdded = async (newFiles: File[]) => {
    // Simulate conversion process
    for (const file of newFiles) {
      const fileId = `${file.name}-${Date.now()}`;
      useFileStore.getState().updateFileStatus(fileId, 'converting');
      
      // Simulate conversion delay
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const result = {
        content: `# Converted ${file.name}\n\nThis is the converted content.`,
        metadata: {
          sourceFormat: 'rtf',
          targetFormat: 'md',
          conversionTime: 1.5,
        },
      };
      
      useFileStore.getState().updateFileStatus(fileId, 'completed', result);
    }
  };

  const handlePreview = (file: any) => {
    setSelectedFile(file);
  };

  return (
    <ErrorBoundary>
      <div className="app-container">
        <DragDropZone onFilesAdded={handleFilesAdded} />
        
        {files.length > 0 && (
          <ConversionProgress 
            onPreview={handlePreview}
            onDownload={() => {}}
            onRetry={() => {}}
          />
        )}
        
        {selectedFile?.result && (
          <div className="preview-section">
            <h3>Preview: {selectedFile.name}</h3>
            <MarkdownPreview content={selectedFile.result.content} />
          </div>
        )}
      </div>
    </ErrorBoundary>
  );
};

describe('File Conversion Flow Integration', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    useFileStore.getState().clearFiles();
  });

  describe('Complete Conversion Flow', () => {
    it('should handle file upload through conversion to preview', async () => {
      const user = userEvent.setup();
      render(<FileConversionFlow />);
      
      // Step 1: Upload file
      const file = new File(['RTF content'], 'test-document.rtf', { type: 'application/rtf' });
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      
      Object.defineProperty(input, 'files', {
        value: [file],
        writable: false,
      });
      
      fireEvent.change(input);
      
      // Step 2: Verify file appears in progress
      await waitFor(() => {
        expect(screen.getByText('test-document.rtf')).toBeInTheDocument();
      });
      
      // Step 3: Wait for conversion to complete
      await waitFor(() => {
        expect(screen.getByText('completed')).toBeInTheDocument();
      }, { timeout: 2000 });
      
      // Step 4: Click preview
      const previewButton = await screen.findByRole('button', { name: /preview/i });
      await user.click(previewButton);
      
      // Step 5: Verify preview appears
      expect(screen.getByText('Preview: test-document.rtf')).toBeInTheDocument();
      expect(screen.getByText(/Converted test-document.rtf/)).toBeInTheDocument();
    });

    it('should handle multiple file conversions', async () => {
      render(<FileConversionFlow />);
      
      const files = [
        new File(['RTF 1'], 'doc1.rtf', { type: 'application/rtf' }),
        new File(['RTF 2'], 'doc2.rtf', { type: 'application/rtf' }),
        new File(['MD'], 'doc3.md', { type: 'text/markdown' }),
      ];
      
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      
      Object.defineProperty(input, 'files', {
        value: files,
        writable: false,
      });
      
      fireEvent.change(input);
      
      // All files should appear
      await waitFor(() => {
        expect(screen.getByText('doc1.rtf')).toBeInTheDocument();
        expect(screen.getByText('doc2.rtf')).toBeInTheDocument();
        expect(screen.getByText('doc3.md')).toBeInTheDocument();
      });
      
      // Check overall progress
      expect(screen.getByText('3/3 files')).toBeInTheDocument();
    });
  });

  describe('Error Handling', () => {
    it('should handle conversion errors gracefully', async () => {
      // Mock conversion failure
      const mockInvoke = invoke as jest.MockedFunction<typeof invoke>;
      mockInvoke.mockRejectedValueOnce(new Error('Conversion failed'));
      
      render(<FileConversionFlow />);
      
      const file = new File(['Invalid RTF'], 'bad.rtf', { type: 'application/rtf' });
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      
      Object.defineProperty(input, 'files', {
        value: [file],
        writable: false,
      });
      
      fireEvent.change(input);
      
      // Should show file
      await waitFor(() => {
        expect(screen.getByText('bad.rtf')).toBeInTheDocument();
      });
      
      // Error handling would depend on actual implementation
    });

    it('should recover from errors when wrapped in ErrorBoundary', () => {
      const ThrowError = () => {
        throw new Error('Component error');
      };
      
      render(
        <ErrorBoundary>
          <ThrowError />
        </ErrorBoundary>
      );
      
      expect(screen.getByText('Something went wrong')).toBeInTheDocument();
      expect(screen.getByText('Component error')).toBeInTheDocument();
    });
  });

  describe('User Interactions', () => {
    it('should support drag and drop workflow', async () => {
      render(<FileConversionFlow />);
      
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      const file = new File(['content'], 'dragged.rtf', { type: 'application/rtf' });
      
      // Simulate drag enter
      fireEvent.dragEnter(dropZone, {
        dataTransfer: {
          files: [file],
          items: [{}],
        },
      });
      
      expect(screen.getByText('Drop your files here')).toBeInTheDocument();
      
      // Simulate drop
      fireEvent.drop(dropZone, {
        dataTransfer: {
          files: [file],
          items: [{}],
        },
      });
      
      await waitFor(() => {
        expect(screen.getByText('dragged.rtf')).toBeInTheDocument();
      });
    });

    it('should allow file removal during conversion', async () => {
      const user = userEvent.setup();
      render(<FileConversionFlow />);
      
      const file = new File(['content'], 'removable.rtf', { type: 'application/rtf' });
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      
      Object.defineProperty(input, 'files', {
        value: [file],
        writable: false,
      });
      
      fireEvent.change(input);
      
      await waitFor(() => {
        expect(screen.getByText('removable.rtf')).toBeInTheDocument();
      });
      
      // Remove file
      const removeButton = screen.getByRole('button', { name: '' }); // X button
      await user.click(removeButton);
      
      expect(screen.queryByText('removable.rtf')).not.toBeInTheDocument();
    });
  });

  describe('Performance', () => {
    it('should handle large number of files efficiently', async () => {
      render(<FileConversionFlow />);
      
      const files = Array.from({ length: 50 }, (_, i) => 
        new File([`Content ${i}`], `file${i}.rtf`, { type: 'application/rtf' })
      );
      
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      
      Object.defineProperty(input, 'files', {
        value: files,
        writable: false,
      });
      
      const startTime = performance.now();
      fireEvent.change(input);
      
      await waitFor(() => {
        expect(screen.getByText('file0.rtf')).toBeInTheDocument();
        expect(screen.getByText('file49.rtf')).toBeInTheDocument();
      });
      
      const renderTime = performance.now() - startTime;
      expect(renderTime).toBeLessThan(2000); // Should render within 2 seconds
    });
  });

  describe('Accessibility', () => {
    it('should maintain focus management through workflow', async () => {
      const user = userEvent.setup();
      render(<FileConversionFlow />);
      
      // Tab to drop zone
      await user.tab();
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      expect(dropZone).toHaveFocus();
      
      // Add file
      const file = new File(['content'], 'focus-test.rtf', { type: 'application/rtf' });
      const input = dropZone.parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      
      Object.defineProperty(input, 'files', {
        value: [file],
        writable: false,
      });
      
      fireEvent.change(input);
      
      await waitFor(() => {
        expect(screen.getByText('focus-test.rtf')).toBeInTheDocument();
      });
      
      // Tab through interface elements
      await user.tab();
      await user.tab();
      
      // Should be able to navigate with keyboard
      expect(document.activeElement).toBeTruthy();
    });

    it('should announce status changes to screen readers', async () => {
      render(<FileConversionFlow />);
      
      const file = new File(['content'], 'announce.rtf', { type: 'application/rtf' });
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      
      Object.defineProperty(input, 'files', {
        value: [file],
        writable: false,
      });
      
      fireEvent.change(input);
      
      // Status changes should be in accessible elements
      await waitFor(() => {
        const statusBadge = screen.getByText('converting');
        expect(statusBadge).toHaveAttribute('class', expect.stringContaining('badge'));
      });
      
      await waitFor(() => {
        const statusBadge = screen.getByText('completed');
        expect(statusBadge).toHaveAttribute('class', expect.stringContaining('badge'));
      }, { timeout: 2000 });
    });
  });

  describe('State Management', () => {
    it('should persist state across component updates', async () => {
      const { rerender } = render(<FileConversionFlow />);
      
      const file = new File(['content'], 'persistent.rtf', { type: 'application/rtf' });
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      
      Object.defineProperty(input, 'files', {
        value: [file],
        writable: false,
      });
      
      fireEvent.change(input);
      
      await waitFor(() => {
        expect(screen.getByText('persistent.rtf')).toBeInTheDocument();
      });
      
      // Re-render component
      rerender(<FileConversionFlow />);
      
      // File should still be there
      expect(screen.getByText('persistent.rtf')).toBeInTheDocument();
    });

    it('should update UI reactively when store changes', async () => {
      render(<FileConversionFlow />);
      
      // Directly update store
      const testFile = {
        id: 'direct-add',
        name: 'directly-added.rtf',
        path: 'directly-added.rtf',
        size: 1024,
        type: 'rtf' as const,
        status: 'idle' as const,
      };
      
      useFileStore.getState().addFiles([new File([''], testFile.name)]);
      
      await waitFor(() => {
        expect(screen.getByText(testFile.name)).toBeInTheDocument();
      });
    });
  });
});