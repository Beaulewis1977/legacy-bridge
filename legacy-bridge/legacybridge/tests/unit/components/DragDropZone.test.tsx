import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { jest } from '@jest/globals';
import userEvent from '@testing-library/user-event';
import { DragDropZone } from '@/components/DragDropZone';
import { useFileStore } from '@/lib/stores/files';

// Mock the file store
jest.mock('@/lib/stores/files');

// Mock framer-motion
jest.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
    label: ({ children, ...props }: any) => <label {...props}>{children}</label>,
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

describe('DragDropZone Component', () => {
  const mockUseFileStore = useFileStore as unknown as jest.MockedFunction<typeof useFileStore>;
  const mockAddFiles = jest.fn();
  const mockRemoveFile = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
    mockUseFileStore.mockReturnValue({
      files: [],
      addFiles: mockAddFiles,
      removeFile: mockRemoveFile,
      updateFileStatus: jest.fn(),
      updateFileProgress: jest.fn(),
      clearFiles: jest.fn(),
      getFileById: jest.fn(),
    });
  });

  describe('File Input', () => {
    it('should render drop zone with correct text', () => {
      render(<DragDropZone />);
      
      expect(screen.getByText('Drag & drop files here')).toBeInTheDocument();
      expect(screen.getByText('or click to browse')).toBeInTheDocument();
      expect(screen.getByText('RTF')).toBeInTheDocument();
      expect(screen.getByText('Markdown')).toBeInTheDocument();
    });

    it('should accept RTF and Markdown files via file input', async () => {
      const onFilesAdded = jest.fn();
      render(<DragDropZone onFilesAdded={onFilesAdded} />);
      
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      expect(input).toHaveAttribute('accept', '.rtf,.md');
      expect(input).toHaveAttribute('multiple');
      
      const rtfFile = new File(['rtf content'], 'test.rtf', { type: 'application/rtf' });
      const mdFile = new File(['md content'], 'test.md', { type: 'text/markdown' });
      
      Object.defineProperty(input, 'files', {
        value: [rtfFile, mdFile],
        writable: false,
      });
      
      fireEvent.change(input);
      
      expect(mockAddFiles).toHaveBeenCalledWith([rtfFile, mdFile]);
      expect(onFilesAdded).toHaveBeenCalledWith([rtfFile, mdFile]);
    });

    it('should reject invalid file types', async () => {
      render(<DragDropZone />);
      
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      
      const invalidFile = new File(['content'], 'test.txt', { type: 'text/plain' });
      const validFile = new File(['content'], 'test.rtf', { type: 'application/rtf' });
      
      Object.defineProperty(input, 'files', {
        value: [invalidFile, validFile],
        writable: false,
      });
      
      fireEvent.change(input);
      
      // Should only add valid files
      expect(mockAddFiles).toHaveBeenCalledWith([validFile]);
      
      // Should show error for invalid file
      await waitFor(() => {
        expect(screen.getByText('test.txt is not a valid file type')).toBeInTheDocument();
      });
    });

    it('should clear error message after 5 seconds', async () => {
      jest.useFakeTimers();
      render(<DragDropZone />);
      
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      const invalidFile = new File(['content'], 'test.txt', { type: 'text/plain' });
      
      Object.defineProperty(input, 'files', {
        value: [invalidFile],
        writable: false,
      });
      
      fireEvent.change(input);
      
      expect(screen.getByText('test.txt is not a valid file type')).toBeInTheDocument();
      
      jest.advanceTimersByTime(5000);
      
      await waitFor(() => {
        expect(screen.queryByText('test.txt is not a valid file type')).not.toBeInTheDocument();
      });
      
      jest.useRealTimers();
    });
  });

  describe('Drag and Drop', () => {
    const createDragEvent = (type: string, files: File[] = []) => {
      const event = new Event(type, { bubbles: true }) as any;
      event.preventDefault = jest.fn();
      event.stopPropagation = jest.fn();
      event.dataTransfer = {
        files,
        items: files.length > 0 ? [{}] : [],
      };
      return event;
    };

    it('should show drag state when dragging files over', () => {
      render(<DragDropZone />);
      
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      
      fireEvent(dropZone, createDragEvent('dragenter', [new File([''], 'test.rtf')]));
      
      expect(screen.getByText('Drop your files here')).toBeInTheDocument();
    });

    it('should handle drag leave correctly', () => {
      render(<DragDropZone />);
      
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      
      fireEvent(dropZone, createDragEvent('dragenter', [new File([''], 'test.rtf')]));
      expect(screen.getByText('Drop your files here')).toBeInTheDocument();
      
      fireEvent(dropZone, createDragEvent('dragleave'));
      expect(screen.getByText('Drag & drop files here')).toBeInTheDocument();
    });

    it('should handle file drop with valid files', () => {
      const onFilesAdded = jest.fn();
      render(<DragDropZone onFilesAdded={onFilesAdded} />);
      
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      const validFiles = [
        new File(['rtf'], 'doc1.rtf', { type: 'application/rtf' }),
        new File(['md'], 'doc2.md', { type: 'text/markdown' }),
      ];
      
      fireEvent(dropZone, createDragEvent('drop', validFiles));
      
      expect(mockAddFiles).toHaveBeenCalledWith(validFiles);
      expect(onFilesAdded).toHaveBeenCalledWith(validFiles);
    });

    it('should filter out invalid files on drop', async () => {
      render(<DragDropZone />);
      
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      const mixedFiles = [
        new File(['rtf'], 'valid.rtf', { type: 'application/rtf' }),
        new File(['txt'], 'invalid.txt', { type: 'text/plain' }),
        new File(['pdf'], 'invalid.pdf', { type: 'application/pdf' }),
      ];
      
      fireEvent(dropZone, createDragEvent('drop', mixedFiles));
      
      expect(mockAddFiles).toHaveBeenCalledWith([mixedFiles[0]]);
      
      await waitFor(() => {
        expect(screen.getByText(/invalid.txt is not a valid file type.*invalid.pdf is not a valid file type/)).toBeInTheDocument();
      });
    });

    it('should prevent default drag behavior', () => {
      render(<DragDropZone />);
      
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      const dragOverEvent = createDragEvent('dragover');
      
      fireEvent(dropZone, dragOverEvent);
      
      expect(dragOverEvent.preventDefault).toHaveBeenCalled();
      expect(dragOverEvent.stopPropagation).toHaveBeenCalled();
    });
  });

  describe('File List Display', () => {
    it('should display selected files', () => {
      const mockFiles = [
        {
          id: 'file-1',
          name: 'document.rtf',
          size: 1024 * 50, // 50KB
          type: 'rtf',
          status: 'idle' as const,
          path: 'document.rtf',
        },
        {
          id: 'file-2',
          name: 'readme.md',
          size: 1024 * 150, // 150KB
          type: 'md',
          status: 'converting' as const,
          progress: 45,
          path: 'readme.md',
        },
      ];
      
      mockUseFileStore.mockReturnValue({
        files: mockFiles,
        addFiles: mockAddFiles,
        removeFile: mockRemoveFile,
        updateFileStatus: jest.fn(),
        updateFileProgress: jest.fn(),
        clearFiles: jest.fn(),
        getFileById: jest.fn(),
      });
      
      render(<DragDropZone />);
      
      expect(screen.getByText('Selected Files (2)')).toBeInTheDocument();
      expect(screen.getByText('document.rtf')).toBeInTheDocument();
      expect(screen.getByText('50.0 KB')).toBeInTheDocument();
      expect(screen.getByText('RTF')).toBeInTheDocument();
      
      expect(screen.getByText('readme.md')).toBeInTheDocument();
      expect(screen.getByText('150.0 KB')).toBeInTheDocument();
      expect(screen.getByText('MD')).toBeInTheDocument();
      expect(screen.getByText('converting')).toBeInTheDocument();
    });

    it('should show progress bar for converting files', () => {
      const mockFiles = [
        {
          id: 'file-1',
          name: 'converting.rtf',
          size: 1024,
          type: 'rtf',
          status: 'converting' as const,
          progress: 75,
          path: 'converting.rtf',
        },
      ];
      
      mockUseFileStore.mockReturnValue({
        files: mockFiles,
        addFiles: mockAddFiles,
        removeFile: mockRemoveFile,
        updateFileStatus: jest.fn(),
        updateFileProgress: jest.fn(),
        clearFiles: jest.fn(),
        getFileById: jest.fn(),
      });
      
      const { container } = render(<DragDropZone />);
      
      const progressBar = container.querySelector('.bg-primary');
      expect(progressBar).toBeInTheDocument();
      expect(progressBar).toHaveStyle({ width: '75%' });
    });

    it('should handle file removal', async () => {
      const mockFiles = [
        {
          id: 'file-1',
          name: 'document.rtf',
          size: 1024,
          type: 'rtf',
          status: 'idle' as const,
          path: 'document.rtf',
        },
      ];
      
      mockUseFileStore.mockReturnValue({
        files: mockFiles,
        addFiles: mockAddFiles,
        removeFile: mockRemoveFile,
        updateFileStatus: jest.fn(),
        updateFileProgress: jest.fn(),
        clearFiles: jest.fn(),
        getFileById: jest.fn(),
      });
      
      const user = userEvent.setup();
      render(<DragDropZone />);
      
      const removeButton = screen.getByRole('button', { name: '' }); // X button
      await user.click(removeButton);
      
      expect(mockRemoveFile).toHaveBeenCalledWith('file-1');
    });

    it('should display correct status badges', () => {
      const mockFiles = [
        {
          id: 'file-1',
          name: 'completed.rtf',
          size: 1024,
          type: 'rtf',
          status: 'completed' as const,
          path: 'completed.rtf',
        },
        {
          id: 'file-2',
          name: 'error.rtf',
          size: 1024,
          type: 'rtf',
          status: 'error' as const,
          path: 'error.rtf',
        },
      ];
      
      mockUseFileStore.mockReturnValue({
        files: mockFiles,
        addFiles: mockAddFiles,
        removeFile: mockRemoveFile,
        updateFileStatus: jest.fn(),
        updateFileProgress: jest.fn(),
        clearFiles: jest.fn(),
        getFileById: jest.fn(),
      });
      
      render(<DragDropZone />);
      
      const badges = screen.getAllByText(/completed|error/);
      expect(badges).toHaveLength(2);
    });
  });

  describe('File Size Formatting', () => {
    it('should format file sizes correctly', () => {
      const mockFiles = [
        {
          id: 'file-1',
          name: 'tiny.rtf',
          size: 500, // 500 B
          type: 'rtf',
          status: 'idle' as const,
          path: 'tiny.rtf',
        },
        {
          id: 'file-2',
          name: 'medium.rtf',
          size: 1024 * 100, // 100 KB
          type: 'rtf',
          status: 'idle' as const,
          path: 'medium.rtf',
        },
        {
          id: 'file-3',
          name: 'large.rtf',
          size: 1024 * 1024 * 5, // 5 MB
          type: 'rtf',
          status: 'idle' as const,
          path: 'large.rtf',
        },
      ];
      
      mockUseFileStore.mockReturnValue({
        files: mockFiles,
        addFiles: mockAddFiles,
        removeFile: mockRemoveFile,
        updateFileStatus: jest.fn(),
        updateFileProgress: jest.fn(),
        clearFiles: jest.fn(),
        getFileById: jest.fn(),
      });
      
      render(<DragDropZone />);
      
      expect(screen.getByText('500 B')).toBeInTheDocument();
      expect(screen.getByText('100.0 KB')).toBeInTheDocument();
      expect(screen.getByText('5.0 MB')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have proper ARIA labels', () => {
      render(<DragDropZone />);
      
      const fileInput = screen.getByLabelText(/drag & drop files here/i);
      expect(fileInput).toBeInTheDocument();
    });

    it('should be keyboard accessible', async () => {
      const user = userEvent.setup();
      render(<DragDropZone />);
      
      // Tab to the drop zone
      await user.tab();
      
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      expect(dropZone).toHaveFocus();
    });

    it('should handle keyboard file selection', async () => {
      const onFilesAdded = jest.fn();
      render(<DragDropZone onFilesAdded={onFilesAdded} />);
      
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      
      // Simulate keyboard activation
      fireEvent.click(input);
      
      // File dialog would open here in real browser
      expect(input).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty file drop', () => {
      render(<DragDropZone />);
      
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      const emptyDrop = createDragEvent('drop', []);
      
      fireEvent(dropZone, emptyDrop);
      
      expect(mockAddFiles).not.toHaveBeenCalled();
    });

    it('should handle multiple drag enter/leave events', () => {
      render(<DragDropZone />);
      
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      
      // Multiple drag enters
      fireEvent(dropZone, createDragEvent('dragenter', [new File([''], 'test.rtf')]));
      fireEvent(dropZone, createDragEvent('dragenter', [new File([''], 'test.rtf')]));
      fireEvent(dropZone, createDragEvent('dragenter', [new File([''], 'test.rtf')]));
      
      expect(screen.getByText('Drop your files here')).toBeInTheDocument();
      
      // One drag leave shouldn't hide the drag state
      fireEvent(dropZone, createDragEvent('dragleave'));
      expect(screen.getByText('Drop your files here')).toBeInTheDocument();
      
      // Multiple drag leaves to match enters
      fireEvent(dropZone, createDragEvent('dragleave'));
      fireEvent(dropZone, createDragEvent('dragleave'));
      
      expect(screen.getByText('Drag & drop files here')).toBeInTheDocument();
    });

    it('should handle files with very long names', () => {
      const longFileName = 'this-is-a-very-long-file-name-that-should-be-truncated-in-the-ui-to-prevent-layout-issues.rtf';
      const mockFiles = [
        {
          id: 'file-1',
          name: longFileName,
          size: 1024,
          type: 'rtf',
          status: 'idle' as const,
          path: longFileName,
        },
      ];
      
      mockUseFileStore.mockReturnValue({
        files: mockFiles,
        addFiles: mockAddFiles,
        removeFile: mockRemoveFile,
        updateFileStatus: jest.fn(),
        updateFileProgress: jest.fn(),
        clearFiles: jest.fn(),
        getFileById: jest.fn(),
      });
      
      render(<DragDropZone />);
      
      const fileName = screen.getByText(longFileName);
      expect(fileName).toHaveClass('truncate');
      expect(fileName).toHaveClass('max-w-[300px]');
    });
  });

  describe('Custom className', () => {
    it('should apply custom className', () => {
      const { container } = render(<DragDropZone className="custom-class" />);
      
      expect(container.firstChild).toHaveClass('custom-class');
    });
  });

  describe('Callback Integration', () => {
    it('should not crash when onFilesAdded is not provided', () => {
      render(<DragDropZone />);
      
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      const validFile = new File(['content'], 'test.rtf', { type: 'application/rtf' });
      
      Object.defineProperty(input, 'files', {
        value: [validFile],
        writable: false,
      });
      
      fireEvent.change(input);
      
      expect(mockAddFiles).toHaveBeenCalledWith([validFile]);
    });

    it('should handle both file input and drag-drop with same callback', () => {
      const onFilesAdded = jest.fn();
      render(<DragDropZone onFilesAdded={onFilesAdded} />);
      
      const file1 = new File(['content1'], 'input.rtf', { type: 'application/rtf' });
      const file2 = new File(['content2'], 'dropped.rtf', { type: 'application/rtf' });
      
      // File input
      const input = screen.getByLabelText(/drag & drop files here/i).parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
      Object.defineProperty(input, 'files', {
        value: [file1],
        writable: false,
      });
      fireEvent.change(input);
      
      expect(onFilesAdded).toHaveBeenCalledWith([file1]);
      
      // Drag and drop
      const dropZone = screen.getByLabelText(/drag & drop files here/i);
      fireEvent(dropZone, createDragEvent('drop', [file2]));
      
      expect(onFilesAdded).toHaveBeenCalledWith([file2]);
      expect(onFilesAdded).toHaveBeenCalledTimes(2);
    });
  });
});