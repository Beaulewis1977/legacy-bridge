import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { DragDropZone } from '@/components/DragDropZone'

// Mock file utilities
const createMockFile = (name: string, type: string, content: string) => {
  return new File([content], name, { type })
}

const createMockDataTransfer = (files: File[]) => {
  return {
    files: {
      length: files.length,
      item: (index: number) => files[index],
      [Symbol.iterator]: function* () {
        for (let i = 0; i < files.length; i++) {
          yield files[i]
        }
      }
    },
    types: ['Files']
  } as any
}

describe('DragDropZone Component', () => {
  const mockOnFilesSelected = vi.fn()
  const mockOnError = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  const defaultProps = {
    onFilesSelected: mockOnFilesSelected,
    onError: mockOnError,
    acceptedTypes: ['.rtf', '.md', '.txt'],
    maxFiles: 10,
    maxSize: 10 * 1024 * 1024 // 10MB
  }

  describe('Initial Render', () => {
    it('should render the drop zone with correct initial state', () => {
      render(<DragDropZone {...defaultProps} />)
      
      expect(screen.getByText(/drag.*drop.*files/i)).toBeInTheDocument()
      expect(screen.getByText(/click to select files/i)).toBeInTheDocument()
      expect(screen.getByText(/supported formats.*rtf.*md.*txt/i)).toBeInTheDocument()
    })

    it('should show the correct file size limit', () => {
      render(<DragDropZone {...defaultProps} />)
      
      expect(screen.getByText(/maximum.*10.*mb/i)).toBeInTheDocument()
    })

    it('should be accessible with proper ARIA attributes', () => {
      render(<DragDropZone {...defaultProps} />)
      
      const dropZone = screen.getByRole('button')
      expect(dropZone).toHaveAttribute('aria-label')
      expect(dropZone).toHaveAttribute('tabIndex', '0')
    })
  })

  describe('File Selection via Click', () => {
    it('should trigger file input when clicked', () => {
      render(<DragDropZone {...defaultProps} />)
      
      const fileInput = screen.getByLabelText(/file upload/i)
      const clickSpy = vi.spyOn(fileInput, 'click')
      
      fireEvent.click(screen.getByRole('button'))
      
      expect(clickSpy).toHaveBeenCalled()
    })

    it('should handle file selection from input', async () => {
      render(<DragDropZone {...defaultProps} />)
      
      const file = createMockFile('test.rtf', 'application/rtf', 'test content')
      const fileInput = screen.getByLabelText(/file upload/i) as HTMLInputElement
      
      // Mock the files property
      Object.defineProperty(fileInput, 'files', {
        value: [file],
        writable: false
      })

      fireEvent.change(fileInput)
      
      await waitFor(() => {
        expect(mockOnFilesSelected).toHaveBeenCalledWith([file])
      })
    })
  })

  describe('Drag and Drop Functionality', () => {
    it('should handle drag enter correctly', () => {
      render(<DragDropZone {...defaultProps} />)
      
      const dropZone = screen.getByRole('button')
      
      fireEvent.dragEnter(dropZone, {
        dataTransfer: createMockDataTransfer([])
      })
      
      expect(dropZone).toHaveClass('border-blue-500')
    })

    it('should handle drag over correctly', () => {
      render(<DragDropZone {...defaultProps} />)
      
      const dropZone = screen.getByRole('button')
      
      fireEvent.dragOver(dropZone, {
        dataTransfer: createMockDataTransfer([])
      })
      
      expect(dropZone).toHaveClass('bg-blue-50')
    })

    it('should handle drag leave correctly', () => {
      render(<DragDropZone {...defaultProps} />)
      
      const dropZone = screen.getByRole('button')
      
      // First enter drag state
      fireEvent.dragEnter(dropZone)
      expect(dropZone).toHaveClass('border-blue-500')
      
      // Then leave
      fireEvent.dragLeave(dropZone)
      expect(dropZone).not.toHaveClass('border-blue-500')
    })

    it('should handle file drop correctly', async () => {
      render(<DragDropZone {...defaultProps} />)
      
      const dropZone = screen.getByRole('button')
      const file = createMockFile('test.rtf', 'application/rtf', 'test content')
      
      fireEvent.drop(dropZone, {
        dataTransfer: createMockDataTransfer([file])
      })
      
      await waitFor(() => {
        expect(mockOnFilesSelected).toHaveBeenCalledWith([file])
      })
    })
  })

  describe('File Validation', () => {
    it('should accept valid file types', async () => {
      render(<DragDropZone {...defaultProps} />)
      
      const validFiles = [
        createMockFile('test.rtf', 'application/rtf', 'content'),
        createMockFile('test.md', 'text/markdown', 'content'),
        createMockFile('test.txt', 'text/plain', 'content')
      ]
      
      const dropZone = screen.getByRole('button')
      
      fireEvent.drop(dropZone, {
        dataTransfer: createMockDataTransfer(validFiles)
      })
      
      await waitFor(() => {
        expect(mockOnFilesSelected).toHaveBeenCalledWith(validFiles)
        expect(mockOnError).not.toHaveBeenCalled()
      })
    })

    it('should reject invalid file types', async () => {
      render(<DragDropZone {...defaultProps} />)
      
      const invalidFile = createMockFile('test.pdf', 'application/pdf', 'content')
      const dropZone = screen.getByRole('button')
      
      fireEvent.drop(dropZone, {
        dataTransfer: createMockDataTransfer([invalidFile])
      })
      
      await waitFor(() => {
        expect(mockOnError).toHaveBeenCalledWith(
          expect.stringContaining('Invalid file type')
        )
        expect(mockOnFilesSelected).not.toHaveBeenCalled()
      })
    })

    it('should reject files that are too large', async () => {
      render(<DragDropZone {...defaultProps} />)
      
      // Create a file that's larger than the 10MB limit
      const largeContent = 'x'.repeat(11 * 1024 * 1024) // 11MB
      const largeFile = createMockFile('large.rtf', 'application/rtf', largeContent)
      
      // Mock the file size
      Object.defineProperty(largeFile, 'size', {
        value: 11 * 1024 * 1024,
        writable: false
      })
      
      const dropZone = screen.getByRole('button')
      
      fireEvent.drop(dropZone, {
        dataTransfer: createMockDataTransfer([largeFile])
      })
      
      await waitFor(() => {
        expect(mockOnError).toHaveBeenCalledWith(
          expect.stringContaining('File size exceeds')
        )
      })
    })

    it('should reject too many files', async () => {
      const propsWithLowLimit = { ...defaultProps, maxFiles: 2 }
      render(<DragDropZone {...propsWithLowLimit} />)
      
      const files = [
        createMockFile('test1.rtf', 'application/rtf', 'content1'),
        createMockFile('test2.rtf', 'application/rtf', 'content2'),
        createMockFile('test3.rtf', 'application/rtf', 'content3')
      ]
      
      const dropZone = screen.getByRole('button')
      
      fireEvent.drop(dropZone, {
        dataTransfer: createMockDataTransfer(files)
      })
      
      await waitFor(() => {
        expect(mockOnError).toHaveBeenCalledWith(
          expect.stringContaining('Too many files')
        )
      })
    })
  })

  describe('Loading State', () => {
    it('should show loading state when processing files', () => {
      render(<DragDropZone {...defaultProps} isLoading={true} />)
      
      expect(screen.getByText(/processing files/i)).toBeInTheDocument()
      expect(screen.getByRole('button')).toBeDisabled()
    })

    it('should show progress when provided', () => {
      render(<DragDropZone {...defaultProps} isLoading={true} progress={45} />)
      
      expect(screen.getByText('45%')).toBeInTheDocument()
      expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '45')
    })
  })

  describe('Keyboard Accessibility', () => {
    it('should handle Enter key press', () => {
      render(<DragDropZone {...defaultProps} />)
      
      const fileInput = screen.getByLabelText(/file upload/i)
      const clickSpy = vi.spyOn(fileInput, 'click')
      
      fireEvent.keyDown(screen.getByRole('button'), { key: 'Enter' })
      
      expect(clickSpy).toHaveBeenCalled()
    })

    it('should handle Space key press', () => {
      render(<DragDropZone {...defaultProps} />)
      
      const fileInput = screen.getByLabelText(/file upload/i)
      const clickSpy = vi.spyOn(fileInput, 'click')
      
      fireEvent.keyDown(screen.getByRole('button'), { key: ' ' })
      
      expect(clickSpy).toHaveBeenCalled()
    })

    it('should ignore other key presses', () => {
      render(<DragDropZone {...defaultProps} />)
      
      const fileInput = screen.getByLabelText(/file upload/i)
      const clickSpy = vi.spyOn(fileInput, 'click')
      
      fireEvent.keyDown(screen.getByRole('button'), { key: 'Escape' })
      
      expect(clickSpy).not.toHaveBeenCalled()
    })
  })

  describe('Visual Feedback', () => {
    it('should show file preview after selection', async () => {
      render(<DragDropZone {...defaultProps} />)
      
      const file = createMockFile('test.rtf', 'application/rtf', 'test content')
      const dropZone = screen.getByRole('button')
      
      fireEvent.drop(dropZone, {
        dataTransfer: createMockDataTransfer([file])
      })
      
      await waitFor(() => {
        expect(screen.getByText('test.rtf')).toBeInTheDocument()
        expect(screen.getByText(/1.*kb/i)).toBeInTheDocument() // File size
      })
    })

    it('should show file type icons correctly', async () => {
      render(<DragDropZone {...defaultProps} />)
      
      const rtfFile = createMockFile('test.rtf', 'application/rtf', 'content')
      const dropZone = screen.getByRole('button')
      
      fireEvent.drop(dropZone, {
        dataTransfer: createMockDataTransfer([rtfFile])
      })
      
      await waitFor(() => {
        const fileIcon = screen.getByTestId('file-type-icon')
        expect(fileIcon).toBeInTheDocument()
      })
    })
  })

  describe('Error Display', () => {
    it('should show error messages to the user', () => {
      render(<DragDropZone {...defaultProps} error="Test error message" />)
      
      expect(screen.getByText('Test error message')).toBeInTheDocument()
      expect(screen.getByRole('alert')).toBeInTheDocument()
    })

    it('should clear error when new files are selected', async () => {
      const { rerender } = render(
        <DragDropZone {...defaultProps} error="Previous error" />
      )
      
      expect(screen.getByText('Previous error')).toBeInTheDocument()
      
      rerender(<DragDropZone {...defaultProps} error={null} />)
      
      expect(screen.queryByText('Previous error')).not.toBeInTheDocument()
    })
  })
})