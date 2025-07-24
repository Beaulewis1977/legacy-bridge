import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { ConversionProgress } from '@/components/ConversionProgress'

describe('ConversionProgress Component', () => {
  const mockFiles = [
    { name: 'document1.rtf', size: 1024, status: 'pending' as const },
    { name: 'document2.rtf', size: 2048, status: 'converting' as const },
    { name: 'document3.rtf', size: 1536, status: 'completed' as const },
    { name: 'document4.rtf', size: 512, status: 'error' as const }
  ]

  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Progress Display', () => {
    it('should show overall progress correctly', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={75}
          isActive={true}
        />
      )
      
      expect(screen.getByText('75%')).toBeInTheDocument()
      expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '75')
    })

    it('should display total file count', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
        />
      )
      
      expect(screen.getByText(/4 files/i)).toBeInTheDocument()
    })

    it('should show completed vs total files', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={25}
          isActive={true}
        />
      )
      
      // 1 completed out of 4 total
      expect(screen.getByText(/1.*4.*files/i)).toBeInTheDocument()
    })
  })

  describe('File Status Indicators', () => {
    it('should show all file statuses correctly', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
        />
      )
      
      expect(screen.getByText('document1.rtf')).toBeInTheDocument()
      expect(screen.getByText('document2.rtf')).toBeInTheDocument()
      expect(screen.getByText('document3.rtf')).toBeInTheDocument()
      expect(screen.getByText('document4.rtf')).toBeInTheDocument()
    })

    it('should display correct status icons', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
        />
      )
      
      // Check for status indicators
      expect(screen.getByTestId('status-pending')).toBeInTheDocument()
      expect(screen.getByTestId('status-converting')).toBeInTheDocument()
      expect(screen.getByTestId('status-completed')).toBeInTheDocument()
      expect(screen.getByTestId('status-error')).toBeInTheDocument()
    })

    it('should show file sizes correctly', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
        />
      )
      
      expect(screen.getByText('1.0 KB')).toBeInTheDocument()
      expect(screen.getByText('2.0 KB')).toBeInTheDocument()
      expect(screen.getByText('1.5 KB')).toBeInTheDocument()
      expect(screen.getByText('512 B')).toBeInTheDocument()
    })
  })

  describe('Animation States', () => {
    it('should show converting animation for active files', async () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
        />
      )
      
      const convertingFile = screen.getByText('document2.rtf').closest('.file-item')
      expect(convertingFile).toHaveClass('converting')
      
      // Animation should be visible
      await waitFor(() => {
        const spinner = screen.getByTestId('conversion-spinner')
        expect(spinner).toBeInTheDocument()
      })
    })

    it('should show completion animation for completed files', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
        />
      )
      
      const completedIcon = screen.getByTestId('status-completed')
      expect(completedIcon).toHaveClass('text-green-500')
    })

    it('should show error state for failed files', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
        />
      )
      
      const errorIcon = screen.getByTestId('status-error')
      expect(errorIcon).toHaveClass('text-red-500')
    })
  })

  describe('Timing Information', () => {
    it('should show elapsed time when provided', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
          elapsedTime={65000} // 1 minute 5 seconds
        />
      )
      
      expect(screen.getByText(/1m 5s/i)).toBeInTheDocument()
    })

    it('should show estimated time remaining', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
          estimatedTimeRemaining={30000} // 30 seconds
        />
      )
      
      expect(screen.getByText(/30s remaining/i)).toBeInTheDocument()
    })

    it('should show conversion speed', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
          conversionSpeed={2.5} // files per second
        />
      )
      
      expect(screen.getByText(/2.5.*files\/s/i)).toBeInTheDocument()
    })
  })

  describe('Error Handling', () => {
    it('should display error messages for failed conversions', () => {
      const filesWithErrors = [
        ...mockFiles,
        { 
          name: 'error.rtf', 
          size: 1024, 
          status: 'error' as const,
          error: 'Invalid RTF format'
        }
      ]

      render(
        <ConversionProgress 
          files={filesWithErrors}
          overallProgress={50}
          isActive={true}
        />
      )
      
      expect(screen.getByText('Invalid RTF format')).toBeInTheDocument()
    })

    it('should show retry option for failed files', () => {
      const filesWithErrors = [
        { 
          name: 'error.rtf', 
          size: 1024, 
          status: 'error' as const,
          error: 'Network timeout'
        }
      ]

      const mockOnRetry = vi.fn()

      render(
        <ConversionProgress 
          files={filesWithErrors}
          overallProgress={0}
          isActive={false}
          onRetry={mockOnRetry}
        />
      )
      
      const retryButton = screen.getByRole('button', { name: /retry/i })
      expect(retryButton).toBeInTheDocument()
    })
  })

  describe('Batch Operations', () => {
    it('should show pause/resume controls when active', () => {
      const mockOnPause = vi.fn()
      const mockOnResume = vi.fn()

      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
          onPause={mockOnPause}
          onResume={mockOnResume}
        />
      )
      
      expect(screen.getByRole('button', { name: /pause/i })).toBeInTheDocument()
    })

    it('should show cancel option', () => {
      const mockOnCancel = vi.fn()

      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
          onCancel={mockOnCancel}
        />
      )
      
      expect(screen.getByRole('button', { name: /cancel/i })).toBeInTheDocument()
    })

    it('should show completion summary when all files are done', () => {
      const completedFiles = mockFiles.map(file => ({
        ...file,
        status: 'completed' as const
      }))

      render(
        <ConversionProgress 
          files={completedFiles}
          overallProgress={100}
          isActive={false}
        />
      )
      
      expect(screen.getByText(/conversion complete/i)).toBeInTheDocument()
      expect(screen.getByText(/4.*files.*converted/i)).toBeInTheDocument()
    })
  })

  describe('Accessibility', () => {
    it('should have proper ARIA labels', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
        />
      )
      
      const progressBar = screen.getByRole('progressbar')
      expect(progressBar).toHaveAttribute('aria-label', expect.stringContaining('Conversion progress'))
      expect(progressBar).toHaveAttribute('aria-valuenow', '50')
      expect(progressBar).toHaveAttribute('aria-valuemin', '0')
      expect(progressBar).toHaveAttribute('aria-valuemax', '100')
    })

    it('should announce status changes to screen readers', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
        />
      )
      
      const statusRegion = screen.getByRole('status')
      expect(statusRegion).toBeInTheDocument()
    })

    it('should have proper heading structure', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
        />
      )
      
      expect(screen.getByRole('heading', { level: 3 })).toBeInTheDocument()
    })
  })

  describe('Visual States', () => {
    it('should dim the interface when inactive', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={100}
          isActive={false}
        />
      )
      
      const container = screen.getByTestId('conversion-progress')
      expect(container).toHaveClass('opacity-75')
    })

    it('should highlight currently converting files', () => {
      render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={50}
          isActive={true}
        />
      )
      
      const convertingFile = screen.getByText('document2.rtf').closest('.file-item')
      expect(convertingFile).toHaveClass('highlight-converting')
    })

    it('should show smooth progress transitions', async () => {
      const { rerender } = render(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={25}
          isActive={true}
        />
      )
      
      rerender(
        <ConversionProgress 
          files={mockFiles}
          overallProgress={75}
          isActive={true}
        />
      )
      
      await waitFor(() => {
        const progressBar = screen.getByRole('progressbar')
        expect(progressBar).toHaveAttribute('aria-valuenow', '75')
      })
    })
  })

  describe('Empty States', () => {
    it('should handle empty file list', () => {
      render(
        <ConversionProgress 
          files={[]}
          overallProgress={0}
          isActive={false}
        />
      )
      
      expect(screen.getByText(/no files to convert/i)).toBeInTheDocument()
    })

    it('should show loading state when files are being prepared', () => {
      render(
        <ConversionProgress 
          files={[]}
          overallProgress={0}
          isActive={true}
          isLoading={true}
        />
      )
      
      expect(screen.getByText(/preparing files/i)).toBeInTheDocument()
    })
  })
})