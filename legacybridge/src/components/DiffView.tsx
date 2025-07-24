'use client';

import { useMemo } from 'react';
import { cn } from '@/lib/utils';
import { Plus, Minus, Equal } from 'lucide-react';

interface DiffViewProps {
  original: string;
  modified: string;
  className?: string;
}

interface DiffLine {
  type: 'add' | 'remove' | 'unchanged';
  content: string;
  lineNumber?: number;
}

export function DiffView({ original, modified, className }: DiffViewProps) {
  const diffLines = useMemo(() => {
    const originalLines = original.split('\n');
    const modifiedLines = modified.split('\n');
    const result: DiffLine[] = [];
    
    // Simple line-by-line diff (in production, use a proper diff algorithm)
    const maxLength = Math.max(originalLines.length, modifiedLines.length);
    
    for (let i = 0; i < maxLength; i++) {
      const origLine = originalLines[i];
      const modLine = modifiedLines[i];
      
      if (origLine === modLine) {
        result.push({
          type: 'unchanged',
          content: origLine || '',
          lineNumber: i + 1
        });
      } else {
        if (origLine !== undefined && modLine === undefined) {
          result.push({
            type: 'remove',
            content: origLine,
            lineNumber: i + 1
          });
        } else if (origLine === undefined && modLine !== undefined) {
          result.push({
            type: 'add',
            content: modLine,
            lineNumber: i + 1
          });
        } else if (origLine !== modLine) {
          result.push({
            type: 'remove',
            content: origLine,
            lineNumber: i + 1
          });
          result.push({
            type: 'add',
            content: modLine,
            lineNumber: i + 1
          });
        }
      }
    }
    
    return result;
  }, [original, modified]);

  const stats = useMemo(() => {
    const added = diffLines.filter(line => line.type === 'add').length;
    const removed = diffLines.filter(line => line.type === 'remove').length;
    const unchanged = diffLines.filter(line => line.type === 'unchanged').length;
    
    return { added, removed, unchanged };
  }, [diffLines]);

  return (
    <div className={cn("flex flex-col h-full", className)}>
      {/* Stats Bar */}
      <div className="flex items-center gap-4 px-4 py-2 bg-muted/50 border-b text-sm">
        <div className="flex items-center gap-1">
          <Plus className="w-4 h-4 text-green-600" />
          <span className="text-green-600">{stats.added} added</span>
        </div>
        <div className="flex items-center gap-1">
          <Minus className="w-4 h-4 text-red-600" />
          <span className="text-red-600">{stats.removed} removed</span>
        </div>
        <div className="flex items-center gap-1">
          <Equal className="w-4 h-4 text-gray-600" />
          <span className="text-gray-600">{stats.unchanged} unchanged</span>
        </div>
      </div>

      {/* Diff Content */}
      <div className="flex-1 overflow-auto font-mono text-sm">
        {diffLines.map((line, index) => (
          <div
            key={index}
            className={cn(
              "flex items-start border-b border-border/50",
              {
                "bg-green-50 dark:bg-green-950/20": line.type === 'add',
                "bg-red-50 dark:bg-red-950/20": line.type === 'remove',
                "hover:bg-muted/50": line.type === 'unchanged'
              }
            )}
          >
            {/* Line Number */}
            <div className="w-12 px-2 py-1 text-right text-muted-foreground text-xs select-none">
              {line.lineNumber}
            </div>

            {/* Diff Indicator */}
            <div className="w-6 py-1 flex items-center justify-center">
              {line.type === 'add' && <Plus className="w-3 h-3 text-green-600" />}
              {line.type === 'remove' && <Minus className="w-3 h-3 text-red-600" />}
              {line.type === 'unchanged' && <span className="text-gray-400">|</span>}
            </div>

            {/* Content */}
            <div className="flex-1 px-2 py-1 whitespace-pre-wrap break-all">
              {line.content || ' '}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}