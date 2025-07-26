'use client';

import React, { useCallback, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Upload, FileText, X, AlertCircle } from 'lucide-react';
import { useFileStore } from '@/lib/stores/files';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

interface DragDropZoneProps {
  onFilesAdded?: (files: File[]) => void;
  className?: string;
}

export const DragDropZone: React.FC<DragDropZoneProps> = ({ onFilesAdded, className }) => {
  const [isDragging, setIsDragging] = useState(false);
  const [dragCounter, setDragCounter] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const { files, addFiles, removeFile } = useFileStore();

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragCounter(prev => prev + 1);
    if (e.dataTransfer.items && e.dataTransfer.items.length > 0) {
      setIsDragging(true);
    }
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragCounter(prev => prev - 1);
    if (dragCounter - 1 === 0) {
      setIsDragging(false);
    }
  }, [dragCounter]);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const validateFiles = (fileList: FileList): File[] => {
    const validFiles: File[] = [];
    const errors: string[] = [];

    Array.from(fileList).forEach(file => {
      if (file.name.endsWith('.rtf') || file.name.endsWith('.md')) {
        validFiles.push(file);
      } else {
        errors.push(`${file.name} is not a valid file type`);
      }
    });

    if (errors.length > 0) {
      setError(errors.join(', '));
      setTimeout(() => setError(null), 5000);
    }

    return validFiles;
  };

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
    setDragCounter(0);

    const validFiles = validateFiles(e.dataTransfer.files);
    if (validFiles.length > 0) {
      addFiles(validFiles);
      onFilesAdded?.(validFiles);
    }
  }, [addFiles, onFilesAdded]);

  const handleFileInput = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      const validFiles = validateFiles(e.target.files);
      if (validFiles.length > 0) {
        addFiles(validFiles);
        onFilesAdded?.(validFiles);
      }
    }
  }, [addFiles, onFilesAdded]);

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  };

  return (
    <div className={cn('w-full', className)}>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        {/* Drop Zone */}
        <div
          onDragEnter={handleDragEnter}
          onDragLeave={handleDragLeave}
          onDragOver={handleDragOver}
          onDrop={handleDrop}
          className="relative"
        >
          <input
            type="file"
            id="file-input"
            className="sr-only"
            multiple
            accept=".rtf,.md"
            onChange={handleFileInput}
            aria-describedby="file-input-description"
          />
          <div id="file-input-description" className="sr-only">
            Select RTF or Markdown files to convert. Maximum file size: 10MB. Supported formats: RTF, Markdown.
          </div>
          
          <motion.label
            htmlFor="file-input"
            className={cn(
              'relative flex flex-col items-center justify-center',
              'w-full h-64 p-8',
              'border-2 border-dashed rounded-lg',
              'cursor-pointer transition-all duration-200',
              'hover:border-primary hover:bg-primary/5',
              'focus-within:border-primary focus-within:bg-primary/5',
              'focus-within:ring-2 focus-within:ring-primary focus-within:ring-offset-2',
              isDragging && 'border-primary bg-primary/10',
              'group'
            )}
            animate={{
              scale: isDragging ? 1.02 : 1,
              borderColor: isDragging ? 'rgb(99 102 241)' : 'rgb(229 231 235)'
            }}
            transition={{ duration: 0.2 }}
            role="button"
            aria-label="Upload RTF or Markdown files by clicking or dragging and dropping"
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                document.getElementById('file-input')?.click();
              }
            }}
          >
            <motion.div
              animate={{ 
                y: isDragging ? -10 : 0,
                scale: isDragging ? 1.1 : 1
              }}
              transition={{ type: 'spring', stiffness: 300, damping: 20 }}
            >
              <Upload className={cn(
                'w-12 h-12 mb-4 text-muted-foreground',
                'group-hover:text-primary transition-colors',
                isDragging && 'text-primary'
              )} />
            </motion.div>
            
            <motion.h3
              className="text-lg font-semibold mb-2 text-foreground"
              animate={{ scale: isDragging ? 1.05 : 1 }}
            >
              {isDragging ? 'Drop your files here' : 'Drag & drop files here'}
            </motion.h3>
            
            <p className="text-sm text-muted-foreground mb-4">
              or click to browse
            </p>
            
            <div className="flex gap-2">
              <Badge variant="secondary">RTF</Badge>
              <Badge variant="secondary">Markdown</Badge>
            </div>
          </motion.label>
        </div>

        {/* Error Message */}
        <AnimatePresence>
          {error && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="mt-4 p-3 bg-destructive/10 text-destructive rounded-md flex items-center gap-2"
            >
              <AlertCircle className="w-4 h-4" />
              <span className="text-sm">{error}</span>
            </motion.div>
          )}
        </AnimatePresence>

        {/* File Preview Cards */}
        <AnimatePresence>
          {files.length > 0 && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: 'auto' }}
              exit={{ opacity: 0, height: 0 }}
              className="mt-6 space-y-3"
            >
              <h4 className="text-sm font-medium text-muted-foreground">
                Selected Files ({files.length})
              </h4>
              
              {files.map((file, index) => (
                <motion.div
                  key={file.id}
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: 20 }}
                  transition={{ delay: index * 0.05 }}
                >
                  <Card className="p-4 hover:shadow-md transition-shadow">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <motion.div
                          whileHover={{ rotate: 5 }}
                          whileTap={{ scale: 0.95 }}
                        >
                          <FileText className="w-8 h-8 text-primary" />
                        </motion.div>
                        
                        <div>
                          <p className="font-medium truncate max-w-[300px]">
                            {file.name}
                          </p>
                          <div className="flex items-center gap-2 text-sm text-muted-foreground">
                            <span>{formatFileSize(file.size)}</span>
                            <span>•</span>
                            <Badge variant="outline" className="text-xs">
                              {file.type.toUpperCase()}
                            </Badge>
                            {file.status !== 'idle' && (
                              <>
                                <span>•</span>
                                <Badge 
                                  variant={
                                    file.status === 'completed' ? 'default' :
                                    file.status === 'error' ? 'destructive' :
                                    'secondary'
                                  }
                                  className="text-xs"
                                >
                                  {file.status}
                                </Badge>
                              </>
                            )}
                          </div>
                        </div>
                      </div>
                      
                      <motion.div
                        whileHover={{ scale: 1.1 }}
                        whileTap={{ scale: 0.9 }}
                      >
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => removeFile(file.id)}
                          className="h-8 w-8"
                        >
                          <X className="h-4 w-4" />
                        </Button>
                      </motion.div>
                    </div>
                    
                    {/* Progress Bar */}
                    {file.status === 'converting' && file.progress !== undefined && (
                      <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        className="mt-3"
                      >
                        <div className="w-full bg-secondary rounded-full h-2 overflow-hidden">
                          <motion.div
                            className="h-full bg-primary"
                            initial={{ width: 0 }}
                            animate={{ width: `${file.progress}%` }}
                            transition={{ duration: 0.3 }}
                          />
                        </div>
                      </motion.div>
                    )}
                  </Card>
                </motion.div>
              ))}
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>
    </div>
  );
};