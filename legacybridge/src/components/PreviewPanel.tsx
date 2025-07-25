'use client';

import { useState, useEffect, useCallback, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Card } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import { 
  FileText, 
  Eye, 
  EyeOff, 
  Copy, 
  Download, 
  AlertCircle,
  CheckCircle2,
  Loader2,
  SplitSquareHorizontal,
  FileCode,
  FileWarning,
  GitCompare
} from 'lucide-react';
import { MarkdownPreview } from './MarkdownPreview';
import { SyntaxHighlighter } from './SyntaxHighlighter';
import { DiffView } from './DiffView';
import { cn } from '@/lib/utils';
import { tauriApi } from '@/lib/tauri-api';
import { useDebounce } from '@/hooks/useDebounce';

interface ValidationResult {
  level: 'error' | 'warning' | 'info';
  code: string;
  message: string;
  location?: string;
}

interface PreviewPanelProps {
  sourceContent: string;
  sourceType: 'rtf' | 'markdown';
  fileName?: string;
  onContentChange?: (content: string) => void;
  className?: string;
}

export function PreviewPanel({
  sourceContent,
  sourceType,
  fileName,
  onContentChange,
  className
}: PreviewPanelProps) {
  const [convertedContent, setConvertedContent] = useState<string>('');
  const [isConverting, setIsConverting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validationResults, setValidationResults] = useState<ValidationResult[]>([]);
  const [viewMode, setViewMode] = useState<'split' | 'preview' | 'source' | 'diff'>('split');
  const [copySuccess, setCopySuccess] = useState(false);
  
  const sourceRef = useRef<HTMLDivElement>(null);
  const previewRef = useRef<HTMLDivElement>(null);
  
  // Debounce the source content to avoid too many conversions
  const debouncedContent = useDebounce(sourceContent, 300);

  // Convert content whenever source changes
  useEffect(() => {
    const convertContent = async () => {
      if (!debouncedContent.trim()) {
        setConvertedContent('');
        setValidationResults([]);
        return;
      }

      setIsConverting(true);
      setError(null);

      try {
        // Use appropriate API based on source type
        if (sourceType === 'rtf') {
          // Use pipeline API for RTF conversion
          const response = await tauriApi.convertWithPipeline(
            debouncedContent,
            sourceType,
            {
              strict_validation: true,
              auto_recovery: true,
              preserve_formatting: true
            }
          );
          
          if (response.success && response.markdown) {
            setConvertedContent(response.markdown);
            setValidationResults(response.validation_results || []);
          } else {
            setError(response.error || 'Conversion failed');
            setConvertedContent('');
          }
        } else {
          // For markdown source, just display it as-is
          setConvertedContent(debouncedContent);
          setValidationResults([]);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error occurred');
        setConvertedContent('');
      } finally {
        setIsConverting(false);
      }
    };

    convertContent();
  }, [debouncedContent, sourceType]);

  // Sync scroll between source and preview
  const handleSourceScroll = useCallback(() => {
    if (sourceRef.current && previewRef.current && viewMode === 'split') {
      const scrollPercentage = 
        sourceRef.current.scrollTop / 
        (sourceRef.current.scrollHeight - sourceRef.current.clientHeight);
      
      previewRef.current.scrollTop = 
        scrollPercentage * 
        (previewRef.current.scrollHeight - previewRef.current.clientHeight);
    }
  }, [viewMode]);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(convertedContent);
      setCopySuccess(true);
      setTimeout(() => setCopySuccess(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }, [convertedContent]);

  const handleDownload = useCallback(() => {
    const blob = new Blob([convertedContent], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = fileName ? fileName.replace(/\.(rtf|md)$/, '.md') : 'converted.md';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }, [convertedContent, fileName]);

  const getValidationIcon = (level: string) => {
    switch (level) {
      case 'error':
        return <AlertCircle className="w-4 h-4 text-destructive" />;
      case 'warning':
        return <FileWarning className="w-4 h-4 text-yellow-500" />;
      default:
        return <AlertCircle className="w-4 h-4 text-blue-500" />;
    }
  };

  return (
    <Card className={cn("flex flex-col h-full", className)}>
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <div className="flex items-center gap-3">
          <FileText className="w-5 h-5 text-muted-foreground" />
          <h3 className="font-semibold">Live Preview</h3>
          {fileName && (
            <Badge variant="secondary" className="text-xs">
              {fileName}
            </Badge>
          )}
          {isConverting && (
            <Loader2 className="w-4 h-4 animate-spin text-muted-foreground" />
          )}
        </div>

        <div className="flex items-center gap-2">
          {/* View Mode Toggle */}
          <div className="flex items-center rounded-md border">
            <Button
              variant={viewMode === 'split' ? 'secondary' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('split')}
              className="rounded-r-none"
            >
              <SplitSquareHorizontal className="w-4 h-4" />
            </Button>
            <Button
              variant={viewMode === 'source' ? 'secondary' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('source')}
              className="rounded-none border-x"
            >
              <FileCode className="w-4 h-4" />
            </Button>
            <Button
              variant={viewMode === 'preview' ? 'secondary' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('preview')}
              className="rounded-none border-x"
            >
              <Eye className="w-4 h-4" />
            </Button>
            <Button
              variant={viewMode === 'diff' ? 'secondary' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('diff')}
              className="rounded-l-none"
            >
              <GitCompare className="w-4 h-4" />
            </Button>
          </div>

          {/* Actions */}
          <Button
            variant="ghost"
            size="sm"
            onClick={handleCopy}
            disabled={!convertedContent}
          >
            {copySuccess ? (
              <>
                <CheckCircle2 className="w-4 h-4 mr-1" />
                Copied
              </>
            ) : (
              <>
                <Copy className="w-4 h-4 mr-1" />
                Copy
              </>
            )}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleDownload}
            disabled={!convertedContent}
          >
            <Download className="w-4 h-4" />
          </Button>
        </div>
      </div>

      {/* Validation Results */}
      <AnimatePresence>
        {validationResults.length > 0 && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            className="border-b overflow-hidden"
          >
            <div className="p-3 space-y-1 max-h-24 overflow-y-auto">
              {validationResults.map((result, index) => (
                <div
                  key={index}
                  className="flex items-start gap-2 text-sm"
                >
                  {getValidationIcon(result.level)}
                  <div className="flex-1">
                    <span className="font-medium">{result.code}:</span>{' '}
                    <span className="text-muted-foreground">{result.message}</span>
                    {result.location && (
                      <span className="text-xs text-muted-foreground ml-2">
                        ({result.location})
                      </span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Content Area */}
      <div className="flex-1 overflow-hidden">
        {error ? (
          <div className="flex items-center justify-center h-full p-8">
            <div className="text-center space-y-2">
              <AlertCircle className="w-12 h-12 text-destructive mx-auto" />
              <p className="text-sm text-muted-foreground">Conversion Error</p>
              <p className="text-sm">{error}</p>
            </div>
          </div>
        ) : (
          <div className="h-full flex">
            {/* Diff View */}
            <AnimatePresence>
              {viewMode === 'diff' && (
                <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className="flex-1"
                >
                  <DiffView
                    original={sourceContent}
                    modified={convertedContent}
                    className="h-full"
                  />
                </motion.div>
              )}
            </AnimatePresence>

            {/* Source View */}
            <AnimatePresence>
              {viewMode !== 'diff' && (viewMode === 'split' || viewMode === 'source') && (
                <motion.div
                  initial={{ width: 0, opacity: 0 }}
                  animate={{ 
                    width: viewMode === 'split' ? '50%' : '100%', 
                    opacity: 1 
                  }}
                  exit={{ width: 0, opacity: 0 }}
                  transition={{ duration: 0.2 }}
                  className="border-r overflow-hidden"
                >
                  <div className="h-full flex flex-col">
                    <div className="px-4 py-2 bg-muted/50 border-b">
                      <p className="text-xs font-medium text-muted-foreground uppercase">
                        {sourceType === 'rtf' ? 'RTF Source' : 'Markdown Source'}
                      </p>
                    </div>
                    <div
                      ref={sourceRef}
                      onScroll={handleSourceScroll}
                      className="flex-1 overflow-auto p-4"
                    >
                      <SyntaxHighlighter
                        code={sourceContent}
                        language={sourceType === 'rtf' ? 'rtf' : 'markdown'}
                        showLineNumbers={true}
                      />
                    </div>
                  </div>
                </motion.div>
              )}
            </AnimatePresence>

            {/* Preview */}
            <AnimatePresence>
              {viewMode !== 'diff' && (viewMode === 'split' || viewMode === 'preview') && (
                <motion.div
                  initial={{ width: 0, opacity: 0 }}
                  animate={{ 
                    width: viewMode === 'split' ? '50%' : '100%', 
                    opacity: 1 
                  }}
                  exit={{ width: 0, opacity: 0 }}
                  transition={{ duration: 0.2 }}
                  className="overflow-hidden"
                >
                  <div className="h-full flex flex-col">
                    <div className="px-4 py-2 bg-muted/50 border-b">
                      <p className="text-xs font-medium text-muted-foreground uppercase">
                        Markdown Preview
                      </p>
                    </div>
                    <div
                      ref={previewRef}
                      className="flex-1 overflow-auto"
                    >
                      {isConverting ? (
                        <div className="flex items-center justify-center h-full">
                          <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
                        </div>
                      ) : (
                        <MarkdownPreview
                          content={convertedContent}
                          className="p-4"
                        />
                      )}
                    </div>
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        )}
      </div>
    </Card>
  );
}