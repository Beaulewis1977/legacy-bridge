'use client';

import React, { useState, useCallback, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Play, 
  Pause, 
  Square, 
  SkipForward, 
  RefreshCw, 
  CheckCircle2, 
  AlertCircle, 
  Clock,
  FileText,
  Download,
  Settings
} from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Slider } from '@/components/ui/slider';
import { useFileStore, FileWithStatus } from '@/lib/stores/files';
import { tauriApi } from '@/lib/tauri-api';
import { downloadService } from '@/lib/download-service';
import { cn } from '@/lib/utils';

interface BatchProcessorProps {
  onComplete?: (results: Array<{ file: FileWithStatus; success: boolean }>) => void;
  className?: string;
}

interface BatchSettings {
  concurrency: number;
  pauseOnError: boolean;
  autoDownload: boolean;
  skipCompleted: boolean;
}

export const BatchProcessor: React.FC<BatchProcessorProps> = ({
  onComplete,
  className
}) => {
  const { files, updateFileStatus, updateFileProgress } = useFileStore();
  const [isProcessing, setIsProcessing] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [currentFileIndex, setCurrentFileIndex] = useState(0);
  const [processedCount, setProcessedCount] = useState(0);
  const [successCount, setSuccessCount] = useState(0);
  const [errorCount, setErrorCount] = useState(0);
  const [startTime, setStartTime] = useState<number | null>(null);
  const [estimatedTimeRemaining, setEstimatedTimeRemaining] = useState<number | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  
  const [settings, setSettings] = useState<BatchSettings>({
    concurrency: 2,
    pauseOnError: false,
    autoDownload: true,
    skipCompleted: true
  });

  const pendingFiles = files.filter(f => 
    f.status === 'idle' || (f.status === 'error' && !settings.skipCompleted)
  );
  const totalFiles = pendingFiles.length;
  const progress = totalFiles > 0 ? (processedCount / totalFiles) * 100 : 0;

  // Calculate estimated time remaining
  useEffect(() => {
    if (isProcessing && startTime && processedCount > 0) {
      const elapsed = Date.now() - startTime;
      const avgTimePerFile = elapsed / processedCount;
      const remaining = (totalFiles - processedCount) * avgTimePerFile;
      setEstimatedTimeRemaining(remaining);
    }
  }, [isProcessing, startTime, processedCount, totalFiles]);

  const processFile = useCallback(async (file: FileWithStatus): Promise<boolean> => {
    updateFileStatus(file.id, 'converting');
    updateFileProgress(file.id, 0);

    // Simulate progress updates
    const progressInterval = setInterval(() => {
      updateFileProgress(file.id, Math.min(90, Math.random() * 100));
    }, 200);

    try {
      const result = file.type === 'rtf' 
        ? await tauriApi.convertRtfToMarkdown(file.path)
        : await tauriApi.convertMarkdownToRtf(file.path);

      clearInterval(progressInterval);
      updateFileProgress(file.id, 100);
      updateFileStatus(file.id, result.success ? 'completed' : 'error', result);

      // Auto-download if enabled
      if (result.success && settings.autoDownload) {
        try {
          await downloadService.downloadFile(file);
        } catch (error) {
          console.error('Auto-download failed:', error);
        }
      }

      return result.success;
    } catch (error) {
      clearInterval(progressInterval);
      updateFileStatus(file.id, 'error');
      return false;
    }
  }, [updateFileStatus, updateFileProgress, settings.autoDownload]);

  const processBatch = useCallback(async () => {
    if (pendingFiles.length === 0) return;

    setIsProcessing(true);
    setIsPaused(false);
    setStartTime(Date.now());
    setProcessedCount(0);
    setSuccessCount(0);
    setErrorCount(0);

    const results: Array<{ file: FileWithStatus; success: boolean }> = [];
    
    // Process files with concurrency control
    const semaphore = Array(settings.concurrency).fill(null);
    let fileIndex = 0;

    const processNext = async (): Promise<void> => {
      if (fileIndex >= pendingFiles.length || isPaused) return;

      const file = pendingFiles[fileIndex++];
      setCurrentFileIndex(fileIndex);

      const success = await processFile(file);
      results.push({ file, success });

      setProcessedCount(prev => prev + 1);
      if (success) {
        setSuccessCount(prev => prev + 1);
      } else {
        setErrorCount(prev => prev + 1);
        
        // Pause on error if enabled
        if (settings.pauseOnError) {
          setIsPaused(true);
          return;
        }
      }

      // Continue processing if not paused
      if (!isPaused) {
        await processNext();
      }
    };

    // Start concurrent processing
    await Promise.all(semaphore.map(() => processNext()));

    setIsProcessing(false);
    onComplete?.(results);
  }, [pendingFiles, settings, processFile, isPaused, onComplete]);

  const handleStart = () => {
    processBatch();
  };

  const handlePause = () => {
    setIsPaused(true);
  };

  const handleResume = () => {
    setIsPaused(false);
    processBatch();
  };

  const handleStop = () => {
    setIsProcessing(false);
    setIsPaused(false);
    setCurrentFileIndex(0);
    setProcessedCount(0);
    setSuccessCount(0);
    setErrorCount(0);
    setStartTime(null);
    setEstimatedTimeRemaining(null);
  };

  const handleSkipCurrent = () => {
    if (currentFileIndex < pendingFiles.length) {
      const currentFile = pendingFiles[currentFileIndex - 1];
      if (currentFile) {
        updateFileStatus(currentFile.id, 'error');
        setErrorCount(prev => prev + 1);
        setProcessedCount(prev => prev + 1);
      }
    }
  };

  const formatTime = (ms: number): string => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    
    if (hours > 0) {
      return `${hours}h ${minutes % 60}m`;
    } else if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    } else {
      return `${seconds}s`;
    }
  };

  if (totalFiles === 0) {
    return null;
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className={cn('w-full', className)}
    >
      <Card className="p-6">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <motion.div
              animate={{ rotate: isProcessing && !isPaused ? 360 : 0 }}
              transition={{ duration: 2, repeat: isProcessing && !isPaused ? Infinity : 0, ease: "linear" }}
            >
              <RefreshCw className="w-5 h-5 text-primary" />
            </motion.div>
            <div>
              <h3 className="text-lg font-semibold">Batch Processor</h3>
              <p className="text-sm text-muted-foreground">
                {totalFiles} files ready for processing
              </p>
            </div>
          </div>
          
          <Button
            onClick={() => setShowSettings(!showSettings)}
            variant="outline"
            size="sm"
            className="gap-2"
          >
            <Settings className="w-4 h-4" />
            Settings
          </Button>
        </div>

        {/* Settings Panel */}
        <AnimatePresence>
          {showSettings && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: 'auto' }}
              exit={{ opacity: 0, height: 0 }}
              className="mb-6 p-4 bg-secondary/30 rounded-lg space-y-4"
            >
              <h4 className="font-medium text-sm">Processing Settings</h4>
              
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* Concurrency */}
                <div className="space-y-2">
                  <Label className="text-sm">
                    Concurrency: {settings.concurrency} files
                  </Label>
                  <Slider
                    value={[settings.concurrency]}
                    onValueChange={([value]) => 
                      setSettings(prev => ({ ...prev, concurrency: value }))
                    }
                    max={5}
                    min={1}
                    step={1}
                    className="w-full"
                  />
                </div>

                {/* Options */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label htmlFor="pause-on-error" className="text-sm">
                      Pause on error
                    </Label>
                    <Switch
                      id="pause-on-error"
                      checked={settings.pauseOnError}
                      onCheckedChange={(checked) =>
                        setSettings(prev => ({ ...prev, pauseOnError: checked }))
                      }
                    />
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <Label htmlFor="auto-download" className="text-sm">
                      Auto-download results
                    </Label>
                    <Switch
                      id="auto-download"
                      checked={settings.autoDownload}
                      onCheckedChange={(checked) =>
                        setSettings(prev => ({ ...prev, autoDownload: checked }))
                      }
                    />
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <Label htmlFor="skip-completed" className="text-sm">
                      Skip completed files
                    </Label>
                    <Switch
                      id="skip-completed"
                      checked={settings.skipCompleted}
                      onCheckedChange={(checked) =>
                        setSettings(prev => ({ ...prev, skipCompleted: checked }))
                      }
                    />
                  </div>
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Progress Section */}
        <div className="space-y-4 mb-6">
          {/* Overall Progress */}
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span>Overall Progress</span>
              <span className="font-mono">
                {processedCount}/{totalFiles} ({progress.toFixed(1)}%)
              </span>
            </div>
            <Progress value={progress} className="h-3" />
          </div>

          {/* Status Cards */}
          <div className="grid grid-cols-3 gap-4">
            <Card className="p-3">
              <div className="flex items-center gap-2">
                <CheckCircle2 className="w-4 h-4 text-green-500" />
                <div>
                  <p className="text-sm font-medium">Success</p>
                  <p className="text-lg font-bold text-green-600">{successCount}</p>
                </div>
              </div>
            </Card>
            
            <Card className="p-3">
              <div className="flex items-center gap-2">
                <AlertCircle className="w-4 h-4 text-red-500" />
                <div>
                  <p className="text-sm font-medium">Errors</p>
                  <p className="text-lg font-bold text-red-600">{errorCount}</p>
                </div>
              </div>
            </Card>
            
            <Card className="p-3">
              <div className="flex items-center gap-2">
                <Clock className="w-4 h-4 text-blue-500" />
                <div>
                  <p className="text-sm font-medium">ETA</p>
                  <p className="text-lg font-bold text-blue-600">
                    {estimatedTimeRemaining ? formatTime(estimatedTimeRemaining) : '--'}
                  </p>
                </div>
              </div>
            </Card>
          </div>
        </div>

        {/* Current File */}
        {isProcessing && currentFileIndex > 0 && currentFileIndex <= pendingFiles.length && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="mb-6 p-4 bg-primary/5 rounded-lg border border-primary/20"
          >
            <div className="flex items-center gap-3">
              <FileText className="w-5 h-5 text-primary" />
              <div className="flex-1">
                <p className="font-medium text-sm">
                  Currently processing: {pendingFiles[currentFileIndex - 1]?.name}
                </p>
                <p className="text-xs text-muted-foreground">
                  File {currentFileIndex} of {totalFiles}
                </p>
              </div>
              {isPaused && (
                <Badge variant="secondary" className="text-xs">
                  Paused
                </Badge>
              )}
            </div>
          </motion.div>
        )}

        {/* Controls */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {!isProcessing ? (
              <Button onClick={handleStart} className="gap-2">
                <Play className="w-4 h-4" />
                Start Processing
              </Button>
            ) : (
              <>
                {isPaused ? (
                  <Button onClick={handleResume} className="gap-2">
                    <Play className="w-4 h-4" />
                    Resume
                  </Button>
                ) : (
                  <Button onClick={handlePause} variant="secondary" className="gap-2">
                    <Pause className="w-4 h-4" />
                    Pause
                  </Button>
                )}
                
                <Button onClick={handleStop} variant="outline" className="gap-2">
                  <Square className="w-4 h-4" />
                  Stop
                </Button>
                
                <Button 
                  onClick={handleSkipCurrent} 
                  variant="ghost" 
                  size="sm"
                  className="gap-2"
                  disabled={!isProcessing || isPaused}
                >
                  <SkipForward className="w-4 h-4" />
                  Skip Current
                </Button>
              </>
            )}
          </div>

          {/* Status Badge */}
          <Badge 
            variant={
              isProcessing ? (isPaused ? "secondary" : "default") : "outline"
            }
            className="gap-1"
          >
            {isProcessing ? (isPaused ? "Paused" : "Processing") : "Ready"}
          </Badge>
        </div>
      </Card>
    </motion.div>
  );
};