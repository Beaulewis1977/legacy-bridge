import { renderHook, act } from '@testing-library/react';
import { useFileStore, FileWithStatus } from '@/lib/stores/files';
import { ConversionResult } from '@/lib/tauri-api';

describe('Files Store', () => {
  // Reset store before each test
  beforeEach(() => {
    const { result } = renderHook(() => useFileStore());
    act(() => {
      result.current.clearFiles();
    });
  });

  describe('addFiles', () => {
    it('should add valid RTF files correctly', () => {
      const { result } = renderHook(() => useFileStore());
      
      const rtfFile = new File(['rtf content'], 'test.rtf', { type: 'application/rtf' });
      
      act(() => {
        result.current.addFiles([rtfFile]);
      });
      
      expect(result.current.files).toHaveLength(1);
      expect(result.current.files[0].name).toBe('test.rtf');
      expect(result.current.files[0].type).toBe('rtf');
      expect(result.current.files[0].status).toBe('idle');
      expect(result.current.files[0].id).toBeDefined();
    });

    it('should add valid Markdown files correctly', () => {
      const { result } = renderHook(() => useFileStore());
      
      const mdFile = new File(['markdown content'], 'test.md', { type: 'text/markdown' });
      
      act(() => {
        result.current.addFiles([mdFile]);
      });
      
      expect(result.current.files).toHaveLength(1);
      expect(result.current.files[0].name).toBe('test.md');
      expect(result.current.files[0].type).toBe('md');
      expect(result.current.files[0].status).toBe('idle');
    });

    it('should filter out non-RTF/MD files', () => {
      const { result } = renderHook(() => useFileStore());
      
      const files = [
        new File(['rtf'], 'valid.rtf', { type: 'application/rtf' }),
        new File(['txt'], 'invalid.txt', { type: 'text/plain' }),
        new File(['md'], 'valid.md', { type: 'text/markdown' }),
        new File(['pdf'], 'invalid.pdf', { type: 'application/pdf' }),
      ];
      
      act(() => {
        result.current.addFiles(files);
      });
      
      expect(result.current.files).toHaveLength(2);
      expect(result.current.files[0].name).toBe('valid.rtf');
      expect(result.current.files[1].name).toBe('valid.md');
    });

    it('should handle multiple files with same name', () => {
      const { result } = renderHook(() => useFileStore());
      
      const file1 = new File(['content1'], 'test.rtf', { type: 'application/rtf' });
      const file2 = new File(['content2'], 'test.rtf', { type: 'application/rtf' });
      
      act(() => {
        result.current.addFiles([file1, file2]);
      });
      
      expect(result.current.files).toHaveLength(2);
      expect(result.current.files[0].id).not.toBe(result.current.files[1].id);
    });

    it('should preserve existing files when adding new ones', () => {
      const { result } = renderHook(() => useFileStore());
      
      const file1 = new File(['content1'], 'first.rtf', { type: 'application/rtf' });
      const file2 = new File(['content2'], 'second.rtf', { type: 'application/rtf' });
      
      act(() => {
        result.current.addFiles([file1]);
      });
      
      const firstFileId = result.current.files[0].id;
      
      act(() => {
        result.current.addFiles([file2]);
      });
      
      expect(result.current.files).toHaveLength(2);
      expect(result.current.files[0].id).toBe(firstFileId);
      expect(result.current.files[1].name).toBe('second.rtf');
    });

    it('should handle empty file array', () => {
      const { result } = renderHook(() => useFileStore());
      
      act(() => {
        result.current.addFiles([]);
      });
      
      expect(result.current.files).toHaveLength(0);
    });

    it('should set correct file properties', () => {
      const { result } = renderHook(() => useFileStore());
      
      const file = new File(['content'], 'test.rtf', { type: 'application/rtf' });
      Object.defineProperty(file, 'size', { value: 1024 });
      Object.defineProperty(file, 'webkitRelativePath', { value: 'folder/test.rtf' });
      
      act(() => {
        result.current.addFiles([file]);
      });
      
      expect(result.current.files[0]).toMatchObject({
        name: 'test.rtf',
        path: 'folder/test.rtf',
        size: 1024,
        type: 'rtf',
        status: 'idle',
      });
    });
  });

  describe('removeFile', () => {
    it('should remove file by id', () => {
      const { result } = renderHook(() => useFileStore());
      
      const files = [
        new File(['1'], 'file1.rtf', { type: 'application/rtf' }),
        new File(['2'], 'file2.rtf', { type: 'application/rtf' }),
        new File(['3'], 'file3.rtf', { type: 'application/rtf' }),
      ];
      
      act(() => {
        result.current.addFiles(files);
      });
      
      const idToRemove = result.current.files[1].id;
      
      act(() => {
        result.current.removeFile(idToRemove);
      });
      
      expect(result.current.files).toHaveLength(2);
      expect(result.current.files.find(f => f.id === idToRemove)).toBeUndefined();
      expect(result.current.files[0].name).toBe('file1.rtf');
      expect(result.current.files[1].name).toBe('file3.rtf');
    });

    it('should handle removing non-existent file', () => {
      const { result } = renderHook(() => useFileStore());
      
      const file = new File(['1'], 'file1.rtf', { type: 'application/rtf' });
      
      act(() => {
        result.current.addFiles([file]);
      });
      
      act(() => {
        result.current.removeFile('non-existent-id');
      });
      
      expect(result.current.files).toHaveLength(1);
    });

    it('should handle removing from empty store', () => {
      const { result } = renderHook(() => useFileStore());
      
      act(() => {
        result.current.removeFile('any-id');
      });
      
      expect(result.current.files).toHaveLength(0);
    });
  });

  describe('updateFileStatus', () => {
    it('should update file status correctly', () => {
      const { result } = renderHook(() => useFileStore());
      
      const file = new File(['content'], 'test.rtf', { type: 'application/rtf' });
      
      act(() => {
        result.current.addFiles([file]);
      });
      
      const fileId = result.current.files[0].id;
      
      act(() => {
        result.current.updateFileStatus(fileId, 'converting');
      });
      
      expect(result.current.files[0].status).toBe('converting');
    });

    it('should update status with result for completed files', () => {
      const { result } = renderHook(() => useFileStore());
      
      const file = new File(['content'], 'test.rtf', { type: 'application/rtf' });
      
      act(() => {
        result.current.addFiles([file]);
      });
      
      const fileId = result.current.files[0].id;
      const conversionResult: ConversionResult = {
        content: 'Converted markdown',
        metadata: {
          sourceFormat: 'rtf',
          targetFormat: 'md',
          conversionTime: 1.5,
        },
      };
      
      act(() => {
        result.current.updateFileStatus(fileId, 'completed', conversionResult);
      });
      
      expect(result.current.files[0].status).toBe('completed');
      expect(result.current.files[0].result).toEqual(conversionResult);
      expect(result.current.files[0].progress).toBe(100);
    });

    it('should not affect other files when updating status', () => {
      const { result } = renderHook(() => useFileStore());
      
      const files = [
        new File(['1'], 'file1.rtf', { type: 'application/rtf' }),
        new File(['2'], 'file2.rtf', { type: 'application/rtf' }),
      ];
      
      act(() => {
        result.current.addFiles(files);
      });
      
      const fileId = result.current.files[0].id;
      
      act(() => {
        result.current.updateFileStatus(fileId, 'error');
      });
      
      expect(result.current.files[0].status).toBe('error');
      expect(result.current.files[1].status).toBe('idle');
    });

    it('should handle updating non-existent file', () => {
      const { result } = renderHook(() => useFileStore());
      
      const file = new File(['content'], 'test.rtf', { type: 'application/rtf' });
      
      act(() => {
        result.current.addFiles([file]);
      });
      
      act(() => {
        result.current.updateFileStatus('non-existent-id', 'completed');
      });
      
      expect(result.current.files[0].status).toBe('idle');
    });
  });

  describe('updateFileProgress', () => {
    it('should update file progress correctly', () => {
      const { result } = renderHook(() => useFileStore());
      
      const file = new File(['content'], 'test.rtf', { type: 'application/rtf' });
      
      act(() => {
        result.current.addFiles([file]);
      });
      
      const fileId = result.current.files[0].id;
      
      act(() => {
        result.current.updateFileProgress(fileId, 50);
      });
      
      expect(result.current.files[0].progress).toBe(50);
    });

    it('should handle progress updates for multiple files', () => {
      const { result } = renderHook(() => useFileStore());
      
      const files = [
        new File(['1'], 'file1.rtf', { type: 'application/rtf' }),
        new File(['2'], 'file2.rtf', { type: 'application/rtf' }),
      ];
      
      act(() => {
        result.current.addFiles(files);
      });
      
      act(() => {
        result.current.updateFileProgress(result.current.files[0].id, 25);
        result.current.updateFileProgress(result.current.files[1].id, 75);
      });
      
      expect(result.current.files[0].progress).toBe(25);
      expect(result.current.files[1].progress).toBe(75);
    });

    it('should handle invalid progress values', () => {
      const { result } = renderHook(() => useFileStore());
      
      const file = new File(['content'], 'test.rtf', { type: 'application/rtf' });
      
      act(() => {
        result.current.addFiles([file]);
      });
      
      const fileId = result.current.files[0].id;
      
      // Test negative progress
      act(() => {
        result.current.updateFileProgress(fileId, -10);
      });
      
      expect(result.current.files[0].progress).toBe(-10);
      
      // Test progress over 100
      act(() => {
        result.current.updateFileProgress(fileId, 150);
      });
      
      expect(result.current.files[0].progress).toBe(150);
    });
  });

  describe('clearFiles', () => {
    it('should remove all files from store', () => {
      const { result } = renderHook(() => useFileStore());
      
      const files = [
        new File(['1'], 'file1.rtf', { type: 'application/rtf' }),
        new File(['2'], 'file2.rtf', { type: 'application/rtf' }),
        new File(['3'], 'file3.rtf', { type: 'application/rtf' }),
      ];
      
      act(() => {
        result.current.addFiles(files);
      });
      
      expect(result.current.files).toHaveLength(3);
      
      act(() => {
        result.current.clearFiles();
      });
      
      expect(result.current.files).toHaveLength(0);
    });

    it('should handle clearing empty store', () => {
      const { result } = renderHook(() => useFileStore());
      
      act(() => {
        result.current.clearFiles();
      });
      
      expect(result.current.files).toHaveLength(0);
    });
  });

  describe('getFileById', () => {
    it('should return file by id', () => {
      const { result } = renderHook(() => useFileStore());
      
      const files = [
        new File(['1'], 'file1.rtf', { type: 'application/rtf' }),
        new File(['2'], 'file2.rtf', { type: 'application/rtf' }),
      ];
      
      act(() => {
        result.current.addFiles(files);
      });
      
      const targetId = result.current.files[1].id;
      
      const file = result.current.getFileById(targetId);
      
      expect(file).toBeDefined();
      expect(file?.name).toBe('file2.rtf');
      expect(file?.id).toBe(targetId);
    });

    it('should return undefined for non-existent id', () => {
      const { result } = renderHook(() => useFileStore());
      
      const file = new File(['1'], 'file1.rtf', { type: 'application/rtf' });
      
      act(() => {
        result.current.addFiles([file]);
      });
      
      const file2 = result.current.getFileById('non-existent-id');
      
      expect(file2).toBeUndefined();
    });

    it('should return undefined when store is empty', () => {
      const { result } = renderHook(() => useFileStore());
      
      const file = result.current.getFileById('any-id');
      
      expect(file).toBeUndefined();
    });
  });

  describe('State Persistence', () => {
    it('should maintain state across multiple operations', () => {
      const { result } = renderHook(() => useFileStore());
      
      // Add files
      const files = [
        new File(['1'], 'file1.rtf', { type: 'application/rtf' }),
        new File(['2'], 'file2.rtf', { type: 'application/rtf' }),
      ];
      
      act(() => {
        result.current.addFiles(files);
      });
      
      const file1Id = result.current.files[0].id;
      const file2Id = result.current.files[1].id;
      
      // Update first file
      act(() => {
        result.current.updateFileStatus(file1Id, 'converting');
        result.current.updateFileProgress(file1Id, 50);
      });
      
      // Update second file
      act(() => {
        result.current.updateFileStatus(file2Id, 'completed');
      });
      
      // Add another file
      act(() => {
        result.current.addFiles([new File(['3'], 'file3.rtf', { type: 'application/rtf' })]);
      });
      
      // Verify state
      expect(result.current.files).toHaveLength(3);
      expect(result.current.files[0].status).toBe('converting');
      expect(result.current.files[0].progress).toBe(50);
      expect(result.current.files[1].status).toBe('completed');
      expect(result.current.files[1].progress).toBe(100);
      expect(result.current.files[2].status).toBe('idle');
    });
  });

  describe('Concurrent Updates', () => {
    it('should handle rapid status updates', () => {
      const { result } = renderHook(() => useFileStore());
      
      const file = new File(['content'], 'test.rtf', { type: 'application/rtf' });
      
      act(() => {
        result.current.addFiles([file]);
      });
      
      const fileId = result.current.files[0].id;
      
      // Simulate rapid status changes
      act(() => {
        result.current.updateFileStatus(fileId, 'converting');
        result.current.updateFileProgress(fileId, 10);
        result.current.updateFileProgress(fileId, 30);
        result.current.updateFileProgress(fileId, 50);
        result.current.updateFileProgress(fileId, 80);
        result.current.updateFileStatus(fileId, 'completed');
      });
      
      expect(result.current.files[0].status).toBe('completed');
      expect(result.current.files[0].progress).toBe(100);
    });

    it('should handle concurrent operations on multiple files', () => {
      const { result } = renderHook(() => useFileStore());
      
      const files = Array.from({ length: 10 }, (_, i) => 
        new File([`content${i}`], `file${i}.rtf`, { type: 'application/rtf' })
      );
      
      act(() => {
        result.current.addFiles(files);
      });
      
      // Update all files concurrently
      act(() => {
        result.current.files.forEach((file, index) => {
          result.current.updateFileStatus(file.id, 'converting');
          result.current.updateFileProgress(file.id, index * 10);
        });
      });
      
      // Verify all updates applied correctly
      result.current.files.forEach((file, index) => {
        expect(file.status).toBe('converting');
        expect(file.progress).toBe(index * 10);
      });
    });
  });
});