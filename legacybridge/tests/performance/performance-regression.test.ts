import { renderHook, act } from '@testing-library/react';
import { render } from '@testing-library/react';
import React from 'react';
import { useFileStore } from '@/lib/stores/files';
import { ConversionProgress } from '@/components/ConversionProgress';
import { MarkdownPreview } from '@/components/MarkdownPreview';
import { MonitoringDashboard } from '@/components/monitoring/MonitoringDashboard';

// Mock heavy components
jest.mock('@/components/monitoring/BuildProgressRing', () => ({
  BuildProgressRing: () => <div>BuildProgressRing</div>,
}));

jest.mock('@/components/monitoring/PerformanceChart', () => ({
  PerformanceChart: () => <div>PerformanceChart</div>,
}));

jest.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

describe('Performance Regression Tests', () => {
  // Performance thresholds
  const THRESHOLDS = {
    storeUpdate: 50, // ms
    componentRender: 100, // ms
    largeListRender: 500, // ms
    markdownParse: 200, // ms
    batchOperation: 1000, // ms
  };

  describe('Store Performance', () => {
    it('should update file status within threshold', () => {
      const { result } = renderHook(() => useFileStore());
      
      // Add test files
      const files = Array.from({ length: 100 }, (_, i) => 
        new File([`content${i}`], `file${i}.rtf`, { type: 'application/rtf' })
      );
      
      act(() => {
        result.current.addFiles(files);
      });
      
      const fileId = result.current.files[50].id;
      
      // Measure status update time
      const startTime = performance.now();
      
      act(() => {
        result.current.updateFileStatus(fileId, 'converting');
      });
      
      const updateTime = performance.now() - startTime;
      
      expect(updateTime).toBeLessThan(THRESHOLDS.storeUpdate);
    });

    it('should handle batch updates efficiently', () => {
      const { result } = renderHook(() => useFileStore());
      
      // Add many files
      const files = Array.from({ length: 500 }, (_, i) => 
        new File([`content${i}`], `file${i}.rtf`, { type: 'application/rtf' })
      );
      
      const startTime = performance.now();
      
      act(() => {
        result.current.addFiles(files);
        
        // Update all files
        result.current.files.forEach((file, index) => {
          result.current.updateFileProgress(file.id, index % 100);
        });
      });
      
      const batchTime = performance.now() - startTime;
      
      expect(batchTime).toBeLessThan(THRESHOLDS.batchOperation);
    });

    it('should clear large file lists efficiently', () => {
      const { result } = renderHook(() => useFileStore());
      
      // Add many files
      const files = Array.from({ length: 1000 }, (_, i) => 
        new File([`content${i}`], `file${i}.rtf`, { type: 'application/rtf' })
      );
      
      act(() => {
        result.current.addFiles(files);
      });
      
      expect(result.current.files).toHaveLength(1000);
      
      const startTime = performance.now();
      
      act(() => {
        result.current.clearFiles();
      });
      
      const clearTime = performance.now() - startTime;
      
      expect(clearTime).toBeLessThan(THRESHOLDS.storeUpdate);
      expect(result.current.files).toHaveLength(0);
    });
  });

  describe('Component Render Performance', () => {
    it('should render ConversionProgress with many files efficiently', () => {
      // Mock large file list
      const mockFiles = Array.from({ length: 100 }, (_, i) => ({
        id: `file-${i}`,
        file: {
          name: `document${i}.rtf`,
          size: 1024 * (i + 1),
          type: 'application/rtf',
          lastModified: Date.now(),
        } as File,
        status: i % 3 === 0 ? 'completed' : i % 3 === 1 ? 'converting' : 'error',
        progress: i % 3 === 1 ? i : i % 3 === 0 ? 100 : 0,
        error: i % 3 === 2 ? 'Test error' : undefined,
      }));
      
      jest.spyOn(useFileStore, 'getState').mockReturnValue({
        files: mockFiles as any,
        addFiles: jest.fn(),
        removeFile: jest.fn(),
        updateFileStatus: jest.fn(),
        updateFileProgress: jest.fn(),
        clearFiles: jest.fn(),
        getFileById: jest.fn(),
      });
      
      const startTime = performance.now();
      
      render(<ConversionProgress />);
      
      const renderTime = performance.now() - startTime;
      
      expect(renderTime).toBeLessThan(THRESHOLDS.largeListRender);
    });

    it('should render MarkdownPreview with large content efficiently', () => {
      const largeContent = Array.from({ length: 1000 }, (_, i) => 
        `# Heading ${i}\n\nParagraph ${i} with **bold** and *italic* text.\n\n`
      ).join('');
      
      const startTime = performance.now();
      
      render(<MarkdownPreview content={largeContent} />);
      
      const renderTime = performance.now() - startTime;
      
      expect(renderTime).toBeLessThan(THRESHOLDS.markdownParse);
    });

    it('should render MonitoringDashboard efficiently', () => {
      const mockMetrics = {
        conversionsPerSecond: 25,
        memoryUsage: 65,
        cpuUsage: 45,
        activeConnections: 10,
        averageResponseTime: 250,
        throughput: 1024,
        history: Array.from({ length: 100 }, (_, i) => ({
          timestamp: new Date(Date.now() - i * 1000),
          conversionsPerSecond: 20 + Math.random() * 10,
          memoryUsage: 60 + Math.random() * 20,
          cpuUsage: 40 + Math.random() * 30,
        })),
      };
      
      const startTime = performance.now();
      
      render(<MonitoringDashboard performanceMetrics={mockMetrics} />);
      
      const renderTime = performance.now() - startTime;
      
      expect(renderTime).toBeLessThan(THRESHOLDS.componentRender);
    });
  });

  describe('Re-render Performance', () => {
    it('should handle rapid prop updates efficiently', () => {
      const { rerender } = render(<MarkdownPreview content="Initial content" />);
      
      const startTime = performance.now();
      
      // Simulate rapid updates
      for (let i = 0; i < 50; i++) {
        rerender(<MarkdownPreview content={`Updated content ${i}`} />);
      }
      
      const totalTime = performance.now() - startTime;
      const averageTime = totalTime / 50;
      
      expect(averageTime).toBeLessThan(THRESHOLDS.componentRender / 10);
    });

    it('should memoize expensive computations', () => {
      const content = '# Large Document\n\n' + 'Content '.repeat(1000);
      
      const { rerender } = render(<MarkdownPreview content={content} />);
      
      // First render (cold)
      const firstRenderStart = performance.now();
      rerender(<MarkdownPreview content={content} />);
      const firstRenderTime = performance.now() - firstRenderStart;
      
      // Second render with same content (should use memoization)
      const secondRenderStart = performance.now();
      rerender(<MarkdownPreview content={content} />);
      const secondRenderTime = performance.now() - secondRenderStart;
      
      // Second render should be significantly faster
      expect(secondRenderTime).toBeLessThan(firstRenderTime * 0.5);
    });
  });

  describe('Memory Performance', () => {
    it('should not leak memory when adding/removing files', () => {
      const { result } = renderHook(() => useFileStore());
      
      // Get initial memory (if available)
      const initialMemory = (performance as any).memory?.usedJSHeapSize || 0;
      
      // Add and remove files multiple times
      for (let cycle = 0; cycle < 10; cycle++) {
        const files = Array.from({ length: 100 }, (_, i) => 
          new File([`content${i}`], `file${i}-cycle${cycle}.rtf`, { type: 'application/rtf' })
        );
        
        act(() => {
          result.current.addFiles(files);
        });
        
        expect(result.current.files.length).toBeGreaterThan(0);
        
        act(() => {
          result.current.clearFiles();
        });
        
        expect(result.current.files).toHaveLength(0);
      }
      
      // Force garbage collection if available
      if (global.gc) {
        global.gc();
      }
      
      // Check memory hasn't grown significantly
      const finalMemory = (performance as any).memory?.usedJSHeapSize || 0;
      
      if (initialMemory > 0 && finalMemory > 0) {
        const memoryGrowth = finalMemory - initialMemory;
        const growthPercentage = (memoryGrowth / initialMemory) * 100;
        
        // Memory growth should be less than 10%
        expect(growthPercentage).toBeLessThan(10);
      }
    });
  });

  describe('Concurrent Operations Performance', () => {
    it('should handle concurrent store updates efficiently', async () => {
      const { result } = renderHook(() => useFileStore());
      
      // Add initial files
      const files = Array.from({ length: 50 }, (_, i) => 
        new File([`content${i}`], `file${i}.rtf`, { type: 'application/rtf' })
      );
      
      act(() => {
        result.current.addFiles(files);
      });
      
      const startTime = performance.now();
      
      // Simulate concurrent updates
      const updatePromises = result.current.files.map((file, index) => {
        return new Promise<void>(resolve => {
          setTimeout(() => {
            act(() => {
              result.current.updateFileProgress(file.id, Math.random() * 100);
              if (index % 2 === 0) {
                result.current.updateFileStatus(file.id, 'converting');
              }
            });
            resolve();
          }, Math.random() * 10);
        });
      });
      
      await Promise.all(updatePromises);
      
      const totalTime = performance.now() - startTime;
      
      expect(totalTime).toBeLessThan(THRESHOLDS.batchOperation);
    });
  });

  describe('Performance Regression Detection', () => {
    it('should maintain performance benchmarks', () => {
      const benchmarks = {
        storeAddFiles: 0,
        storeUpdateStatus: 0,
        componentRender: 0,
        markdownParse: 0,
      };
      
      // Benchmark: Store add files
      const { result } = renderHook(() => useFileStore());
      const files = Array.from({ length: 100 }, (_, i) => 
        new File([`content${i}`], `file${i}.rtf`, { type: 'application/rtf' })
      );
      
      let start = performance.now();
      act(() => {
        result.current.addFiles(files);
      });
      benchmarks.storeAddFiles = performance.now() - start;
      
      // Benchmark: Store update status
      start = performance.now();
      act(() => {
        result.current.updateFileStatus(result.current.files[0].id, 'completed');
      });
      benchmarks.storeUpdateStatus = performance.now() - start;
      
      // Benchmark: Component render
      start = performance.now();
      render(<ConversionProgress />);
      benchmarks.componentRender = performance.now() - start;
      
      // Benchmark: Markdown parse
      const content = '# Title\n\n' + 'Paragraph '.repeat(100);
      start = performance.now();
      render(<MarkdownPreview content={content} />);
      benchmarks.markdownParse = performance.now() - start;
      
      // Log benchmarks for tracking
      console.log('Performance Benchmarks:', benchmarks);
      
      // Assert all operations are within acceptable limits
      expect(benchmarks.storeAddFiles).toBeLessThan(100);
      expect(benchmarks.storeUpdateStatus).toBeLessThan(10);
      expect(benchmarks.componentRender).toBeLessThan(200);
      expect(benchmarks.markdownParse).toBeLessThan(100);
    });
  });
});