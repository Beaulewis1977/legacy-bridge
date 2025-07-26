'use client';

import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Download, 
  Clock, 
  CheckCircle2, 
  XCircle, 
  FileDown, 
  FolderDown,
  Trash2,
  Archive,
  History
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { 
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { ProcessedFile } from '@/lib/stores/files';
import { downloadService, DownloadProgress, DownloadHistory } from '@/lib/download-service';
import { cn } from '@/lib/utils';

interface DownloadManagerProps {
  files?: ProcessedFile[];
  className?: string;
  onClose?: () => void;
}

export function DownloadManager({ files = [], className, onClose }: DownloadManagerProps) {
  const [activeDownloads, setActiveDownloads] = useState<DownloadProgress[]>([]);
  const [downloadHistory, setDownloadHistory] = useState<DownloadHistory[]>([]);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [isDownloading, setIsDownloading] = useState(false);

  // Load history on mount
  useEffect(() => {
    setDownloadHistory(downloadService.getHistory());
  }, []);

  // Update active downloads periodically
  useEffect(() => {
    const interval = setInterval(() => {
      setActiveDownloads(downloadService.getActiveDownloads());
    }, 100);
    return () => clearInterval(interval);
  }, []);

  const handleSingleDownload = async (file: ProcessedFile) => {
    setIsDownloading(true);
    try {
      await downloadService.downloadFile(file, (progress) => {
        setActiveDownloads(prev => {
          const filtered = prev.filter(p => p.fileId !== file.id);
          return [...filtered, progress];
        });
      });
      // Refresh history
      setDownloadHistory(downloadService.getHistory());
    } catch (error) {
      console.error('Download failed:', error);
    } finally {
      setIsDownloading(false);
    }
  };

  const handleBatchDownload = async () => {
    const filesToDownload = files.filter(f => selectedFiles.has(f.id));
    if (filesToDownload.length === 0) return;

    setIsDownloading(true);
    try {
      await downloadService.downloadBatch(
        filesToDownload, 
        `legacybridge_batch_${Date.now()}.zip`,
        (progress) => {
          setActiveDownloads(prev => {
            const filtered = prev.filter(p => p.fileId !== 'batch');
            return [...filtered, progress];
          });
        }
      );
      // Clear selection after successful download
      setSelectedFiles(new Set());
      // Refresh history
      setDownloadHistory(downloadService.getHistory());
    } catch (error) {
      console.error('Batch download failed:', error);
    } finally {
      setIsDownloading(false);
    }
  };

  const handleSelectAll = () => {
    if (selectedFiles.size === files.length) {
      setSelectedFiles(new Set());
    } else {
      setSelectedFiles(new Set(files.map(f => f.id)));
    }
  };

  const clearHistory = () => {
    downloadService.clearHistory();
    setDownloadHistory([]);
    setShowClearConfirm(false);
  };

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const formatDate = (date: Date | string) => {
    const d = typeof date === 'string' ? new Date(date) : date;
    return d.toLocaleDateString() + ' ' + d.toLocaleTimeString();
  };

  const completedFiles = files.filter(f => f.status === 'completed');

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.95 }}
      className={cn("w-full max-w-4xl", className)}
    >
      <Card className="overflow-hidden">
        <CardHeader className="bg-gradient-to-r from-primary/10 to-primary/5">
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <Download className="w-5 h-5" />
              Download Manager
            </CardTitle>
            <Badge variant="outline">
              {completedFiles.length} files ready
            </Badge>
          </div>
        </CardHeader>
        
        <CardContent className="p-0">
          <Tabs defaultValue="files" className="w-full">
            <TabsList className="w-full rounded-none border-b">
              <TabsTrigger value="files" className="flex-1 gap-2">
                <FileDown className="w-4 h-4" />
                Files ({completedFiles.length})
              </TabsTrigger>
              <TabsTrigger value="active" className="flex-1 gap-2">
                <Clock className="w-4 h-4" />
                Active ({activeDownloads.length})
              </TabsTrigger>
              <TabsTrigger value="history" className="flex-1 gap-2">
                <History className="w-4 h-4" />
                History ({downloadHistory.length})
              </TabsTrigger>
            </TabsList>

            {/* Files Tab */}
            <TabsContent value="files" className="p-4 space-y-4">
              {completedFiles.length > 0 ? (
                <>
                  {/* Batch Actions */}
                  <div className="flex items-center justify-between p-3 bg-secondary/30 rounded-lg">
                    <div className="flex items-center gap-3">
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={handleSelectAll}
                      >
                        {selectedFiles.size === completedFiles.length ? 'Deselect All' : 'Select All'}
                      </Button>
                      <span className="text-sm text-muted-foreground">
                        {selectedFiles.size} selected
                      </span>
                    </div>
                    <Button
                      size="sm"
                      onClick={handleBatchDownload}
                      disabled={selectedFiles.size === 0 || isDownloading}
                      className="gap-2"
                    >
                      <Archive className="w-4 h-4" />
                      Download as ZIP
                    </Button>
                  </div>

                  {/* File List */}
                  <div className="space-y-2 max-h-96 overflow-y-auto">
                    {completedFiles.map(file => {
                      const originalExt = file.file.name.split('.').pop()?.toLowerCase();
                      const targetExt = originalExt === 'rtf' ? 'md' : 'rtf';
                      const convertedName = file.file.name.replace(/\.(rtf|md)$/i, `.${targetExt}`);
                      const isSelected = selectedFiles.has(file.id);

                      return (
                        <motion.div
                          key={file.id}
                          initial={{ opacity: 0, x: -20 }}
                          animate={{ opacity: 1, x: 0 }}
                          className={cn(
                            "flex items-center justify-between p-3 rounded-lg border transition-colors",
                            isSelected ? "bg-primary/5 border-primary/20" : "hover:bg-secondary/50"
                          )}
                        >
                          <div className="flex items-center gap-3">
                            <input
                              type="checkbox"
                              checked={isSelected}
                              onChange={(e) => {
                                const newSelection = new Set(selectedFiles);
                                if (e.target.checked) {
                                  newSelection.add(file.id);
                                } else {
                                  newSelection.delete(file.id);
                                }
                                setSelectedFiles(newSelection);
                              }}
                              className="rounded border-gray-300"
                            />
                            <div>
                              <p className="font-medium text-sm">{convertedName}</p>
                              <p className="text-xs text-muted-foreground">
                                {formatFileSize(file.file.size)} • {targetExt.toUpperCase()}
                              </p>
                            </div>
                          </div>
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => handleSingleDownload(file)}
                            disabled={isDownloading}
                            className="gap-2"
                          >
                            <Download className="w-4 h-4" />
                            Download
                          </Button>
                        </motion.div>
                      );
                    })}
                  </div>
                </>
              ) : (
                <div className="text-center py-12 text-muted-foreground">
                  <FolderDown className="w-12 h-12 mx-auto mb-3 opacity-30" />
                  <p>No files ready for download</p>
                  <p className="text-sm mt-1">Complete some conversions first</p>
                </div>
              )}
            </TabsContent>

            {/* Active Downloads Tab */}
            <TabsContent value="active" className="p-4 space-y-3">
              {activeDownloads.length > 0 ? (
                <AnimatePresence>
                  {activeDownloads.map(download => (
                    <motion.div
                      key={`${download.fileId}_${download.fileName}`}
                      initial={{ opacity: 0, y: 10 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0, y: -10 }}
                      className="p-4 border rounded-lg"
                    >
                      <div className="flex items-center justify-between mb-2">
                        <span className="font-medium text-sm">{download.fileName}</span>
                        <Badge variant={
                          download.status === 'completed' ? 'default' : 
                          download.status === 'error' ? 'destructive' : 
                          'secondary'
                        }>
                          {download.status}
                        </Badge>
                      </div>
                      <Progress value={download.progress} className="h-2" />
                      {download.error && (
                        <p className="text-xs text-red-500 mt-2">{download.error}</p>
                      )}
                    </motion.div>
                  ))}
                </AnimatePresence>
              ) : (
                <div className="text-center py-12 text-muted-foreground">
                  <Clock className="w-12 h-12 mx-auto mb-3 opacity-30" />
                  <p>No active downloads</p>
                </div>
              )}
            </TabsContent>

            {/* History Tab */}
            <TabsContent value="history" className="p-4">
              {downloadHistory.length > 0 ? (
                <>
                  <div className="flex justify-end mb-3">
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => setShowClearConfirm(true)}
                      className="gap-2"
                    >
                      <Trash2 className="w-4 h-4" />
                      Clear History
                    </Button>
                  </div>
                  <div className="space-y-2 max-h-96 overflow-y-auto">
                    {downloadHistory.map(entry => (
                      <div
                        key={entry.id}
                        className="flex items-center justify-between p-3 border rounded-lg hover:bg-secondary/50"
                      >
                        <div className="flex items-center gap-3">
                          {entry.status === 'success' ? (
                            <CheckCircle2 className="w-4 h-4 text-green-500" />
                          ) : (
                            <XCircle className="w-4 h-4 text-red-500" />
                          )}
                          <div>
                            <p className="font-medium text-sm">{entry.fileName}</p>
                            <p className="text-xs text-muted-foreground">
                              {formatFileSize(entry.fileSize)} • {formatDate(entry.downloadDate)}
                            </p>
                          </div>
                        </div>
                        <Badge variant="outline" className="text-xs">
                          {entry.format.toUpperCase()}
                        </Badge>
                      </div>
                    ))}
                  </div>
                </>
              ) : (
                <div className="text-center py-12 text-muted-foreground">
                  <History className="w-12 h-12 mx-auto mb-3 opacity-30" />
                  <p>No download history</p>
                </div>
              )}
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>

      {/* Clear History Confirmation */}
      <AlertDialog open={showClearConfirm} onOpenChange={setShowClearConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Clear Download History?</AlertDialogTitle>
            <AlertDialogDescription>
              This will permanently remove all download history. This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={clearHistory}>
              Clear History
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </motion.div>
  );
}