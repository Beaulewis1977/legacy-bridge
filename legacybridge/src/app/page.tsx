'use client';

import { useState, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ArrowRight, FileText, Download, Sparkles, CheckCircle2, Archive } from 'lucide-react';
import { DragDropZone } from '@/components/DragDropZone';
import { ConversionProgress } from '@/components/ConversionProgress';
import { PreviewPanel } from '@/components/PreviewPanel';
import { DownloadManager } from '@/components/DownloadManager';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { useFileStore } from '@/lib/stores/files';
import { tauriApi } from '@/lib/tauri-api';
import { downloadService } from '@/lib/download-service';
import { useEffect } from 'react';
import type { FileWithStatus } from '@/lib/stores/files';
import type { ConversionResult } from '@/lib/tauri-api';
import { ErrorBoundary, ConversionErrorBoundary } from '@/components/ErrorBoundary';

export default function Home() {
  const { files, updateFileStatus, updateFileProgress, clearFiles } = useFileStore();
  const [isConverting, setIsConverting] = useState(false);
  const [conversionResults, setConversionResults] = useState<Array<{ file: FileWithStatus; result: ConversionResult }>>([]);
  const [showPreview, setShowPreview] = useState(true);
  const [selectedFileContent, setSelectedFileContent] = useState<string>('');
  const [selectedFileType, setSelectedFileType] = useState<'rtf' | 'md' | 'markdown'>('rtf');
  const [selectedFileName, setSelectedFileName] = useState<string>('');
  const [showDownloadManager, setShowDownloadManager] = useState(false);

  const handleConvertToMarkdown = useCallback(async () => {
    setIsConverting(true);
    const rtfFiles = files.filter(file => file.type === 'rtf');
    const results = [];

    for (const file of rtfFiles) {
      updateFileStatus(file.id, 'converting');
      updateFileProgress(file.id, 0);

      // Simulate progress with cleanup
      let progressInterval: NodeJS.Timeout | null = null;
      progressInterval = setInterval(() => {
        updateFileProgress(file.id, Math.min(90, Math.random() * 100));
      }, 200);

      try {
        const result = await tauriApi.convertRtfToMarkdown(file.path);
        if (progressInterval) clearInterval(progressInterval);
        updateFileProgress(file.id, 100);
        updateFileStatus(file.id, result.success ? 'completed' : 'error', result);
        results.push({ file, result });
      } catch (error) {
        if (progressInterval) clearInterval(progressInterval);
        updateFileStatus(file.id, 'error');
      }
    }

    setConversionResults(results);
    setIsConverting(false);
  }, [files, updateFileStatus, updateFileProgress]);

  const handleConvertToRtf = useCallback(async () => {
    setIsConverting(true);
    const mdFiles = files.filter(file => file.type === 'md');
    const results = [];

    for (const file of mdFiles) {
      updateFileStatus(file.id, 'converting');
      updateFileProgress(file.id, 0);

      // Simulate progress with cleanup
      let progressInterval: NodeJS.Timeout | null = null;
      progressInterval = setInterval(() => {
        updateFileProgress(file.id, Math.min(90, Math.random() * 100));
      }, 200);

      try {
        const result = await tauriApi.convertMarkdownToRtf(file.path);
        if (progressInterval) clearInterval(progressInterval);
        updateFileProgress(file.id, 100);
        updateFileStatus(file.id, result.success ? 'completed' : 'error', result);
        results.push({ file, result });
      } catch (error) {
        if (progressInterval) clearInterval(progressInterval);
        updateFileStatus(file.id, 'error');
      }
    }

    setConversionResults(results);
    setIsConverting(false);
  }, [files, updateFileStatus, updateFileProgress]);

  const rtfCount = files.filter(f => f.type === 'rtf').length;
  const mdCount = files.filter(f => f.type === 'md').length;
  const completedCount = files.filter(f => f.status === 'completed').length;

  // Load file content for preview
  const loadFileContent = useCallback(async (file: FileWithStatus) => {
    try {
      // Read file content using Tauri API
      const response = await tauriApi.readFileContent(file.path);
      if (response.success && response.content) {
        setSelectedFileContent(response.content);
        setSelectedFileType(file.type);
        setSelectedFileName(file.name);
      }
    } catch (error) {
      console.error('Failed to load file content:', error);
    }
  }, []);

  // Auto-load first file when files are added
  useEffect(() => {
    if (files.length > 0 && showPreview && !selectedFileContent) {
      const firstFile = files[0];
      if (firstFile) {
        loadFileContent(firstFile);
      }
    }
  }, [files, showPreview, selectedFileContent, loadFileContent]);

  return (
    <ConversionErrorBoundary>
    <div className="min-h-screen bg-gradient-to-b from-background to-secondary/20">
      <div className="container mx-auto px-4 py-12 max-w-5xl">
        {/* Header */}
        <motion.div
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          className="text-center mb-12"
        >
          <div className="inline-flex items-center gap-2 mb-4">
            <motion.div
              animate={{ rotate: [0, 360] }}
              transition={{ duration: 20, repeat: Infinity, ease: "linear" }}
            >
              <Sparkles className="w-6 h-6 text-primary" />
            </motion.div>
            <h1 className="text-4xl font-bold bg-gradient-to-r from-primary to-primary/60 bg-clip-text text-transparent">
              LegacyBridge
            </h1>
          </div>
          <p className="text-muted-foreground text-lg">
            Convert between RTF and Markdown with ease
          </p>
        </motion.div>

        {/* Main Content */}
        <div className="space-y-8">
          {/* Drop Zone */}
          <DragDropZone className="mb-8" />

          {/* Conversion Actions */}
          <AnimatePresence>
            {files.length > 0 && (
              <motion.div
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.95 }}
                transition={{ duration: 0.3 }}
              >
                <Card className="p-6">
                  <div className="flex flex-col sm:flex-row items-center justify-between gap-4">
                    <div className="flex items-center gap-4">
                      <FileText className="w-5 h-5 text-muted-foreground" />
                      <div className="text-sm">
                        <span className="font-medium">{files.length} files selected</span>
                        <div className="flex gap-2 mt-1">
                          {rtfCount > 0 && (
                            <Badge variant="outline">{rtfCount} RTF</Badge>
                          )}
                          {mdCount > 0 && (
                            <Badge variant="outline">{mdCount} Markdown</Badge>
                          )}
                          {completedCount > 0 && (
                            <Badge variant="default">
                              <CheckCircle2 className="w-3 h-3 mr-1" />
                              {completedCount} completed
                            </Badge>
                          )}
                        </div>
                      </div>
                    </div>

                    {/* File selector for preview */}
                    {showPreview && files.length > 1 && (
                      <div className="flex items-center gap-2 mt-2">
                        <span className="text-xs text-muted-foreground">Preview:</span>
                        <select
                          className="text-xs bg-background border rounded px-2 py-1"
                          value={selectedFileName}
                          onChange={(e) => {
                            const file = files.find(f => f.name === e.target.value);
                            if (file) loadFileContent(file);
                          }}
                        >
                          {files.map(file => (
                            <option key={file.id} value={file.name}>
                              {file.name}
                            </option>
                          ))}
                        </select>
                      </div>
                    )}

                    <div className="flex gap-3">
                      <Button
                        onClick={handleConvertToMarkdown}
                        disabled={rtfCount === 0 || isConverting}
                        className="group"
                      >
                        Convert to Markdown
                        <ArrowRight className="ml-2 w-4 h-4 group-hover:translate-x-1 transition-transform" />
                      </Button>
                      <Button
                        onClick={handleConvertToRtf}
                        disabled={mdCount === 0 || isConverting}
                        variant="secondary"
                        className="group"
                      >
                        Convert to RTF
                        <ArrowRight className="ml-2 w-4 h-4 group-hover:translate-x-1 transition-transform" />
                      </Button>
                      <Button
                        onClick={clearFiles}
                        variant="outline"
                        disabled={isConverting}
                      >
                        Clear All
                      </Button>
                    </div>
                  </div>
                </Card>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Conversion Progress */}
          <AnimatePresence>
            {files.length > 0 && files.some(f => f.status !== 'idle') && (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: 20 }}
              >
                <ConversionProgress
                  onDownload={async (file) => {
                    try {
                      await downloadService.downloadFile(file);
                    } catch (error) {
                      console.error('Download failed:', error);
                    }
                  }}
                  onPreview={(file) => {
                    if (file.result?.content) {
                      setSelectedFileContent(file.result.content);
                      setSelectedFileType(file.result.metadata?.convertedFormat as 'rtf' | 'md' | 'markdown' || 'md');
                      setSelectedFileName(file.file.name.replace(/\.(rtf|md)$/i, `.${file.result.metadata?.convertedFormat || 'md'}`));
                      setShowPreview(true);
                    }
                  }}
                  onRetry={async (file) => {
                    // Re-run conversion for failed file
                    updateFileStatus(file.id, 'converting');
                    updateFileProgress(file.id, 0);
                    
                    try {
                      const result = file.file.type === 'rtf' 
                        ? await tauriApi.convertRtfToMarkdown(file.file.path)
                        : await tauriApi.convertMarkdownToRtf(file.file.path);
                        
                      updateFileStatus(file.id, result.success ? 'completed' : 'error', result);
                    } catch (error) {
                      updateFileStatus(file.id, 'error');
                    }
                  }}
                />
              </motion.div>
            )}
          </AnimatePresence>

          {/* Preview Toggle */}
          <AnimatePresence>
            {files.length > 0 && (
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: 10 }}
                className="flex items-center justify-between p-4 bg-secondary/30 rounded-lg"
              >
                <div className="flex items-center gap-2">
                  <Label htmlFor="preview-toggle" className="text-sm font-medium">
                    Real-time Preview
                  </Label>
                  <Switch
                    id="preview-toggle"
                    checked={showPreview}
                    onCheckedChange={setShowPreview}
                  />
                </div>
                <div className="flex items-center gap-3">
                  <p className="text-xs text-muted-foreground">
                    {showPreview ? 'Preview enabled' : 'Preview disabled'}
                  </p>
                  {completedCount > 0 && (
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => setShowDownloadManager(true)}
                      className="gap-2"
                    >
                      <Archive className="w-4 h-4" />
                      Download Manager ({completedCount})
                    </Button>
                  )}
                </div>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Live Preview Panel */}
          <AnimatePresence>
            {showPreview && selectedFileContent && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
                transition={{ duration: 0.3 }}
                className="overflow-hidden"
              >
                <PreviewPanel
                  sourceContent={selectedFileContent}
                  sourceType={selectedFileType}
                  fileName={selectedFileName}
                  className="h-[600px]"
                />
              </motion.div>
            )}
          </AnimatePresence>

          {/* Results Section */}
          <AnimatePresence>
            {conversionResults.length > 0 && !showPreview && (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: 20 }}
              >
                <Card>
                  <div className="p-6">
                    <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
                      <CheckCircle2 className="w-5 h-5 text-green-500" />
                      Conversion Results
                    </h3>
                    
                    <Tabs defaultValue="preview" className="w-full">
                      <TabsList className="grid w-full grid-cols-2">
                        <TabsTrigger value="preview">Preview</TabsTrigger>
                        <TabsTrigger value="download">Download</TabsTrigger>
                      </TabsList>
                      
                      <TabsContent value="preview" className="mt-4">
                        <div className="space-y-4 max-h-96 overflow-y-auto">
                          {conversionResults.map((result, index) => (
                            <motion.div
                              key={index}
                              initial={{ opacity: 0, x: -20 }}
                              animate={{ opacity: 1, x: 0 }}
                              transition={{ delay: index * 0.1 }}
                              className="p-4 bg-secondary/50 rounded-lg"
                            >
                              <div className="flex items-center justify-between mb-2">
                                <span className="font-medium">{result.file.name}</span>
                                <Badge variant={result.result.success ? 'default' : 'destructive'}>
                                  {result.result.success ? 'Success' : 'Failed'}
                                </Badge>
                              </div>
                              {result.result.content && (
                                <pre className="text-xs bg-background p-3 rounded overflow-x-auto">
                                  {result.result.content.slice(0, 200)}...
                                </pre>
                              )}
                            </motion.div>
                          ))}
                        </div>
                      </TabsContent>
                      
                      <TabsContent value="download" className="mt-4">
                        <div className="space-y-3">
                          {conversionResults
                            .filter(r => r.result.success)
                            .map((result, index) => (
                              <motion.div
                                key={index}
                                initial={{ opacity: 0, y: 10 }}
                                animate={{ opacity: 1, y: 0 }}
                                transition={{ delay: index * 0.05 }}
                                className="flex items-center justify-between p-3 bg-secondary/30 rounded-lg"
                              >
                                <span className="text-sm font-medium">
                                  {result.file.name.replace(/\.(rtf|md)$/, '')}
                                  {result.result.metadata?.convertedFormat === 'md' ? '.md' : '.rtf'}
                                </span>
                                <Button 
                                  size="sm" 
                                  variant="ghost" 
                                  className="gap-2"
                                  onClick={async () => {
                                    const processedFile = files.find(f => f.id === result.file.id);
                                    if (processedFile) {
                                      try {
                                        await downloadService.downloadFile(processedFile);
                                      } catch (error) {
                                        console.error('Download failed:', error);
                                      }
                                    }
                                  }}
                                >
                                  <Download className="w-4 h-4" />
                                  Download
                                </Button>
                              </motion.div>
                            ))}
                        </div>
                      </TabsContent>
                    </Tabs>
                  </div>
                </Card>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Download Manager Modal */}
          <AnimatePresence>
            {showDownloadManager && (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4"
                onClick={() => setShowDownloadManager(false)}
              >
                <motion.div
                  initial={{ scale: 0.9, opacity: 0 }}
                  animate={{ scale: 1, opacity: 1 }}
                  exit={{ scale: 0.9, opacity: 0 }}
                  onClick={(e) => e.stopPropagation()}
                >
                  <DownloadManager 
                    files={files}
                    onClose={() => setShowDownloadManager(false)}
                  />
                  <Button
                    className="mt-4 w-full"
                    variant="outline"
                    onClick={() => setShowDownloadManager(false)}
                  >
                    Close
                  </Button>
                </motion.div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
    </ConversionErrorBoundary>
  );
}
