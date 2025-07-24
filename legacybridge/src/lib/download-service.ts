import { saveAs } from 'file-saver';
import JSZip from 'jszip';
import { ProcessedFile } from '@/lib/stores/files';
import { ConversionResult } from '@/lib/tauri-api';

export interface DownloadProgress {
  fileId: string;
  fileName: string;
  progress: number;
  status: 'pending' | 'downloading' | 'completed' | 'error';
  error?: string;
}

export interface DownloadHistory {
  id: string;
  fileName: string;
  fileSize: number;
  downloadDate: Date;
  format: 'rtf' | 'md';
  status: 'success' | 'failed';
}

export class DownloadService {
  private downloadHistory: DownloadHistory[] = [];
  private activeDownloads: Map<string, DownloadProgress> = new Map();
  private progressCallbacks: Map<string, (progress: DownloadProgress) => void> = new Map();

  constructor() {
    // Load download history from localStorage
    const savedHistory = localStorage.getItem('legacybridge_download_history');
    if (savedHistory) {
      this.downloadHistory = JSON.parse(savedHistory);
    }
  }

  // Download a single file
  async downloadFile(
    file: ProcessedFile,
    onProgress?: (progress: DownloadProgress) => void
  ): Promise<void> {
    const downloadId = `download_${file.id}_${Date.now()}`;
    const progress: DownloadProgress = {
      fileId: file.id,
      fileName: file.file.name,
      progress: 0,
      status: 'pending'
    };

    this.activeDownloads.set(downloadId, progress);
    if (onProgress) {
      this.progressCallbacks.set(downloadId, onProgress);
    }

    try {
      // Update progress to downloading
      this.updateProgress(downloadId, { status: 'downloading', progress: 20 });

      // Get the converted content
      const content = file.result?.content;
      if (!content) {
        throw new Error('No content available for download');
      }

      // Determine file extension based on conversion
      const originalExt = file.file.name.split('.').pop()?.toLowerCase();
      const targetExt = originalExt === 'rtf' ? 'md' : 'rtf';
      const fileName = file.file.name.replace(/\.(rtf|md)$/i, `.${targetExt}`);

      // Create blob based on file type
      let blob: Blob;
      if (targetExt === 'rtf') {
        // RTF files need proper encoding
        blob = new Blob([content], { type: 'application/rtf;charset=utf-8' });
      } else {
        // Markdown files are plain text
        blob = new Blob([content], { type: 'text/markdown;charset=utf-8' });
      }

      // Simulate download progress
      for (let i = 30; i <= 90; i += 10) {
        await new Promise(resolve => setTimeout(resolve, 100));
        this.updateProgress(downloadId, { progress: i });
      }

      // Trigger download
      saveAs(blob, fileName);

      // Update progress to completed
      this.updateProgress(downloadId, { status: 'completed', progress: 100 });

      // Add to history
      this.addToHistory({
        id: downloadId,
        fileName,
        fileSize: blob.size,
        downloadDate: new Date(),
        format: targetExt,
        status: 'success'
      });

    } catch (error) {
      this.updateProgress(downloadId, {
        status: 'error',
        error: error instanceof Error ? error.message : 'Download failed'
      });

      // Add failed download to history
      this.addToHistory({
        id: downloadId,
        fileName: file.file.name,
        fileSize: 0,
        downloadDate: new Date(),
        format: 'md',
        status: 'failed'
      });

      throw error;
    } finally {
      // Clean up
      setTimeout(() => {
        this.activeDownloads.delete(downloadId);
        this.progressCallbacks.delete(downloadId);
      }, 5000);
    }
  }

  // Download multiple files as a zip
  async downloadBatch(
    files: ProcessedFile[],
    zipFileName: string = 'legacybridge_converted_files.zip',
    onProgress?: (progress: DownloadProgress) => void
  ): Promise<void> {
    const downloadId = `batch_${Date.now()}`;
    const progress: DownloadProgress = {
      fileId: 'batch',
      fileName: zipFileName,
      progress: 0,
      status: 'pending'
    };

    this.activeDownloads.set(downloadId, progress);
    if (onProgress) {
      this.progressCallbacks.set(downloadId, onProgress);
    }

    try {
      this.updateProgress(downloadId, { status: 'downloading', progress: 10 });

      const zip = new JSZip();
      const successfulFiles: string[] = [];
      const failedFiles: string[] = [];

      // Process each file
      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        const progressPerFile = 80 / files.length;
        const currentProgress = 10 + (i * progressPerFile);

        try {
          const content = file.result?.content;
          if (!content) {
            failedFiles.push(file.file.name);
            continue;
          }

          // Determine file extension
          const originalExt = file.file.name.split('.').pop()?.toLowerCase();
          const targetExt = originalExt === 'rtf' ? 'md' : 'rtf';
          const fileName = file.file.name.replace(/\.(rtf|md)$/i, `.${targetExt}`);

          // Add file to zip
          zip.file(fileName, content);
          successfulFiles.push(fileName);

          this.updateProgress(downloadId, { progress: currentProgress });
        } catch (error) {
          failedFiles.push(file.file.name);
        }
      }

      // Add summary file
      const summary = this.generateBatchSummary(successfulFiles, failedFiles);
      zip.file('conversion_summary.txt', summary);

      this.updateProgress(downloadId, { progress: 90 });

      // Generate and download zip
      const blob = await zip.generateAsync({ type: 'blob' });
      saveAs(blob, zipFileName);

      this.updateProgress(downloadId, { status: 'completed', progress: 100 });

      // Add to history
      this.addToHistory({
        id: downloadId,
        fileName: zipFileName,
        fileSize: blob.size,
        downloadDate: new Date(),
        format: 'md', // zip contains mixed formats
        status: 'success'
      });

    } catch (error) {
      this.updateProgress(downloadId, {
        status: 'error',
        error: error instanceof Error ? error.message : 'Batch download failed'
      });
      throw error;
    } finally {
      // Clean up
      setTimeout(() => {
        this.activeDownloads.delete(downloadId);
        this.progressCallbacks.delete(downloadId);
      }, 5000);
    }
  }

  // Get download history
  getHistory(): DownloadHistory[] {
    return [...this.downloadHistory].sort((a, b) => 
      b.downloadDate.getTime() - a.downloadDate.getTime()
    );
  }

  // Clear download history
  clearHistory(): void {
    this.downloadHistory = [];
    localStorage.removeItem('legacybridge_download_history');
  }

  // Get active downloads
  getActiveDownloads(): DownloadProgress[] {
    return Array.from(this.activeDownloads.values());
  }

  // Private methods
  private updateProgress(downloadId: string, updates: Partial<DownloadProgress>): void {
    const progress = this.activeDownloads.get(downloadId);
    if (progress) {
      Object.assign(progress, updates);
      const callback = this.progressCallbacks.get(downloadId);
      if (callback) {
        callback(progress);
      }
    }
  }

  private addToHistory(entry: DownloadHistory): void {
    this.downloadHistory.push(entry);
    // Keep only last 100 entries
    if (this.downloadHistory.length > 100) {
      this.downloadHistory = this.downloadHistory.slice(-100);
    }
    // Save to localStorage
    localStorage.setItem('legacybridge_download_history', JSON.stringify(this.downloadHistory));
  }

  private generateBatchSummary(successful: string[], failed: string[]): string {
    const timestamp = new Date().toLocaleString();
    let summary = `LegacyBridge Batch Conversion Summary\n`;
    summary += `Generated: ${timestamp}\n`;
    summary += `=======================================\n\n`;
    
    summary += `Total Files: ${successful.length + failed.length}\n`;
    summary += `Successful: ${successful.length}\n`;
    summary += `Failed: ${failed.length}\n\n`;
    
    if (successful.length > 0) {
      summary += `Successfully Converted Files:\n`;
      summary += `----------------------------\n`;
      successful.forEach(file => {
        summary += `✓ ${file}\n`;
      });
      summary += `\n`;
    }
    
    if (failed.length > 0) {
      summary += `Failed Files:\n`;
      summary += `-------------\n`;
      failed.forEach(file => {
        summary += `✗ ${file}\n`;
      });
    }
    
    return summary;
  }
}

// Export singleton instance
export const downloadService = new DownloadService();