'use client';

import React, { useState, useCallback, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Eye, 
  EyeOff, 
  ArrowLeftRight, 
  Copy, 
  Download, 
  Maximize2, 
  Minimize2,
  FileText,
  Code,
  Split,
  Layers
} from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { MarkdownPreview } from './MarkdownPreview';
import { SyntaxHighlighter } from './SyntaxHighlighter';
import { cn } from '@/lib/utils';

interface FileComparisonProps {
  originalContent: string;
  convertedContent: string;
  originalType: 'rtf' | 'md' | 'markdown';
  convertedType: 'rtf' | 'md' | 'markdown';
  fileName: string;
  onDownload?: (content: string, type: string) => void;
  className?: string;
}

type ViewMode = 'side-by-side' | 'overlay' | 'tabs';
type DisplayMode = 'rendered' | 'source' | 'both';

export const FileComparison: React.FC<FileComparisonProps> = ({
  originalContent,
  convertedContent,
  originalType,
  convertedType,
  fileName,
  onDownload,
  className
}) => {
  const [viewMode, setViewMode] = useState<ViewMode>('side-by-side');
  const [displayMode, setDisplayMode] = useState<DisplayMode>('rendered');
  const [showLineNumbers, setShowLineNumbers] = useState(true);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [activeTab, setActiveTab] = useState<'original' | 'converted'>('original');

  // Calculate content statistics
  const stats = useMemo(() => {
    const originalLines = originalContent.split('\n').length;
    const convertedLines = convertedContent.split('\n').length;
    const originalChars = originalContent.length;
    const convertedChars = convertedContent.length;
    
    return {
      original: {
        lines: originalLines,
        characters: originalChars,
        words: originalContent.split(/\s+/).filter(w => w.length > 0).length
      },
      converted: {
        lines: convertedLines,
        characters: convertedChars,
        words: convertedContent.split(/\s+/).filter(w => w.length > 0).length
      },
      difference: {
        lines: convertedLines - originalLines,
        characters: convertedChars - originalChars
      }
    };
  }, [originalContent, convertedContent]);

  const handleCopyContent = useCallback(async (content: string, type: string) => {
    try {
      await navigator.clipboard.writeText(content);
      // Could add toast notification here
    } catch (error) {
      console.error('Failed to copy content:', error);
    }
  }, []);

  const renderContent = useCallback((content: string, type: string, mode: DisplayMode) => {
    if (mode === 'source' || (mode === 'both' && type === 'rtf')) {
      return (
        <SyntaxHighlighter
          code={content}
          language={type === 'rtf' ? 'rtf' : 'markdown'}
          showLineNumbers={showLineNumbers}
          className="h-full"
        />
      );
    }
    
    if (type === 'md' || type === 'markdown') {
      return (
        <MarkdownPreview
          content={content}
          showLineNumbers={showLineNumbers}
          className="h-full"
        />
      );
    }
    
    // RTF rendered view (simplified)
    return (
      <div className="p-4 bg-background border rounded h-full overflow-auto">
        <pre className="whitespace-pre-wrap text-sm font-mono">
          {content}
        </pre>
      </div>
    );
  }, [showLineNumbers]);

  const ViewModeSelector = () => (
    <div className="flex items-center gap-2">
      <Button
        onClick={() => setViewMode('side-by-side')}
        variant={viewMode === 'side-by-side' ? 'default' : 'outline'}
        size="sm"
        className="gap-2"
      >
        <Split className="w-4 h-4" />
        Side by Side
      </Button>
      <Button
        onClick={() => setViewMode('overlay')}
        variant={viewMode === 'overlay' ? 'default' : 'outline'}
        size="sm"
        className="gap-2"
      >
        <Layers className="w-4 h-4" />
        Overlay
      </Button>
      <Button
        onClick={() => setViewMode('tabs')}
        variant={viewMode === 'tabs' ? 'default' : 'outline'}
        size="sm"
        className="gap-2"
      >
        <FileText className="w-4 h-4" />
        Tabs
      </Button>
    </div>
  );

  const DisplayModeSelector = () => (
    <div className="flex items-center gap-2">
      <Button
        onClick={() => setDisplayMode('rendered')}
        variant={displayMode === 'rendered' ? 'default' : 'outline'}
        size="sm"
        className="gap-2"
      >
        <Eye className="w-4 h-4" />
        Rendered
      </Button>
      <Button
        onClick={() => setDisplayMode('source')}
        variant={displayMode === 'source' ? 'default' : 'outline'}
        size="sm"
        className="gap-2"
      >
        <Code className="w-4 h-4" />
        Source
      </Button>
    </div>
  );

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className={cn(
        'w-full',
        isFullscreen && 'fixed inset-0 z-50 bg-background p-4',
        className
      )}
    >
      <Card className="h-full flex flex-col">
        {/* Header */}
        <div className="p-4 border-b space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-lg font-semibold">File Comparison</h3>
              <p className="text-sm text-muted-foreground">{fileName}</p>
            </div>
            
            <div className="flex items-center gap-2">
              <Button
                onClick={() => setIsFullscreen(!isFullscreen)}
                variant="outline"
                size="sm"
                className="gap-2"
              >
                {isFullscreen ? (
                  <>
                    <Minimize2 className="w-4 h-4" />
                    Exit Fullscreen
                  </>
                ) : (
                  <>
                    <Maximize2 className="w-4 h-4" />
                    Fullscreen
                  </>
                )}
              </Button>
            </div>
          </div>

          {/* Controls */}
          <div className="flex flex-wrap items-center justify-between gap-4">
            <div className="flex items-center gap-4">
              <ViewModeSelector />
              <DisplayModeSelector />
            </div>
            
            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
                <Label htmlFor="line-numbers" className="text-sm">
                  Line numbers
                </Label>
                <Switch
                  id="line-numbers"
                  checked={showLineNumbers}
                  onCheckedChange={setShowLineNumbers}
                />
              </div>
            </div>
          </div>

          {/* Statistics */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
            <div className="space-y-1">
              <p className="text-muted-foreground">Original</p>
              <div className="space-y-0.5">
                <p>{stats.original.lines} lines</p>
                <p>{stats.original.characters} chars</p>
                <p>{stats.original.words} words</p>
              </div>
            </div>
            
            <div className="space-y-1">
              <p className="text-muted-foreground">Converted</p>
              <div className="space-y-0.5">
                <p>{stats.converted.lines} lines</p>
                <p>{stats.converted.characters} chars</p>
                <p>{stats.converted.words} words</p>
              </div>
            </div>
            
            <div className="space-y-1">
              <p className="text-muted-foreground">Difference</p>
              <div className="space-y-0.5">
                <p className={cn(
                  stats.difference.lines > 0 ? 'text-green-600' : 
                  stats.difference.lines < 0 ? 'text-red-600' : 'text-muted-foreground'
                )}>
                  {stats.difference.lines > 0 ? '+' : ''}{stats.difference.lines} lines
                </p>
                <p className={cn(
                  stats.difference.characters > 0 ? 'text-green-600' : 
                  stats.difference.characters < 0 ? 'text-red-600' : 'text-muted-foreground'
                )}>
                  {stats.difference.characters > 0 ? '+' : ''}{stats.difference.characters} chars
                </p>
              </div>
            </div>
            
            <div className="space-y-1">
              <p className="text-muted-foreground">Types</p>
              <div className="flex gap-2">
                <Badge variant="outline">{originalType.toUpperCase()}</Badge>
                <ArrowLeftRight className="w-4 h-4 text-muted-foreground" />
                <Badge variant="outline">{convertedType.toUpperCase()}</Badge>
              </div>
            </div>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 p-4">
          {viewMode === 'side-by-side' && (
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 h-full">
              {/* Original */}
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <h4 className="font-medium text-sm flex items-center gap-2">
                    Original ({originalType.toUpperCase()})
                    <Badge variant="secondary" className="text-xs">
                      {stats.original.lines} lines
                    </Badge>
                  </h4>
                  <div className="flex items-center gap-1">
                    <Button
                      onClick={() => handleCopyContent(originalContent, originalType)}
                      variant="ghost"
                      size="sm"
                      className="gap-1"
                    >
                      <Copy className="w-3 h-3" />
                    </Button>
                    <Button
                      onClick={() => onDownload?.(originalContent, originalType)}
                      variant="ghost"
                      size="sm"
                      className="gap-1"
                    >
                      <Download className="w-3 h-3" />
                    </Button>
                  </div>
                </div>
                <div className="h-[500px] border rounded overflow-hidden">
                  {renderContent(originalContent, originalType, displayMode)}
                </div>
              </div>

              {/* Converted */}
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <h4 className="font-medium text-sm flex items-center gap-2">
                    Converted ({convertedType.toUpperCase()})
                    <Badge variant="secondary" className="text-xs">
                      {stats.converted.lines} lines
                    </Badge>
                  </h4>
                  <div className="flex items-center gap-1">
                    <Button
                      onClick={() => handleCopyContent(convertedContent, convertedType)}
                      variant="ghost"
                      size="sm"
                      className="gap-1"
                    >
                      <Copy className="w-3 h-3" />
                    </Button>
                    <Button
                      onClick={() => onDownload?.(convertedContent, convertedType)}
                      variant="ghost"
                      size="sm"
                      className="gap-1"
                    >
                      <Download className="w-3 h-3" />
                    </Button>
                  </div>
                </div>
                <div className="h-[500px] border rounded overflow-hidden">
                  {renderContent(convertedContent, convertedType, displayMode)}
                </div>
              </div>
            </div>
          )}

          {viewMode === 'overlay' && (
            <div className="space-y-4">
              <div className="flex items-center justify-center gap-4">
                <Button
                  onClick={() => setActiveTab('original')}
                  variant={activeTab === 'original' ? 'default' : 'outline'}
                  className="gap-2"
                >
                  <EyeOff className={cn('w-4 h-4', activeTab === 'converted' && 'opacity-50')} />
                  Show Original
                </Button>
                <Button
                  onClick={() => setActiveTab('converted')}
                  variant={activeTab === 'converted' ? 'default' : 'outline'}
                  className="gap-2"
                >
                  <Eye className={cn('w-4 h-4', activeTab === 'original' && 'opacity-50')} />
                  Show Converted
                </Button>
              </div>

              <AnimatePresence mode="wait">
                <motion.div
                  key={activeTab}
                  initial={{ opacity: 0, x: activeTab === 'converted' ? 20 : -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: activeTab === 'converted' ? -20 : 20 }}
                  transition={{ duration: 0.2 }}
                  className="h-[500px] border rounded overflow-hidden"
                >
                  {activeTab === 'original' 
                    ? renderContent(originalContent, originalType, displayMode)
                    : renderContent(convertedContent, convertedType, displayMode)
                  }
                </motion.div>
              </AnimatePresence>
            </div>
          )}

          {viewMode === 'tabs' && (
            <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as 'original' | 'converted')}>
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="original" className="gap-2">
                  Original ({originalType.toUpperCase()})
                  <Badge variant="secondary" className="text-xs">
                    {stats.original.lines}
                  </Badge>
                </TabsTrigger>
                <TabsTrigger value="converted" className="gap-2">
                  Converted ({convertedType.toUpperCase()})
                  <Badge variant="secondary" className="text-xs">
                    {stats.converted.lines}
                  </Badge>
                </TabsTrigger>
              </TabsList>
              
              <TabsContent value="original" className="mt-4">
                <div className="h-[500px] border rounded overflow-hidden">
                  {renderContent(originalContent, originalType, displayMode)}
                </div>
              </TabsContent>
              
              <TabsContent value="converted" className="mt-4">
                <div className="h-[500px] border rounded overflow-hidden">
                  {renderContent(convertedContent, convertedType, displayMode)}
                </div>
              </TabsContent>
            </Tabs>
          )}
        </div>
      </Card>
    </motion.div>
  );
};