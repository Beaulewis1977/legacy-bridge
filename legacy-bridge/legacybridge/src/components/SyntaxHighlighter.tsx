'use client';

import { useMemo } from 'react';
import { cn } from '@/lib/utils';
import { sanitizeSyntaxHighlight, escapeHtml } from '@/lib/sanitizer';

interface SyntaxHighlighterProps {
  code: string;
  language: 'rtf' | 'markdown';
  className?: string;
  showLineNumbers?: boolean;
}

export function SyntaxHighlighter({
  code,
  language,
  className,
  showLineNumbers = true
}: SyntaxHighlighterProps) {
  const highlightedCode = useMemo(() => {
    if (!code) return '';

    // First escape the code to prevent XSS
    let highlighted = escapeHtml(code);

    if (language === 'rtf') {
      // RTF syntax highlighting
      // Control words
      highlighted = highlighted.replace(
        /\\([a-z]+)(-?\d*)/g,
        '<span class="text-blue-600 dark:text-blue-400">\\$1$2</span>'
      );
      
      // Groups
      highlighted = highlighted.replace(
        /\{/g,
        '<span class="text-purple-600 dark:text-purple-400">{</span>'
      );
      highlighted = highlighted.replace(
        /\}/g,
        '<span class="text-purple-600 dark:text-purple-400">}</span>'
      );
      
      // Special characters
      highlighted = highlighted.replace(
        /\\&#039;/g,
        '<span class="text-green-600 dark:text-green-400">\\&#039;</span>'
      );
      
      // Comments (if any)
      highlighted = highlighted.replace(
        /\\\\.*$/gm,
        (match) => `<span class="text-gray-500 dark:text-gray-400">${match}</span>`
      );
    } else if (language === 'markdown') {
      // Markdown syntax highlighting
      // Headers
      highlighted = highlighted.replace(
        /^(#{1,6})\s+(.*)$/gm,
        '<span class="text-blue-600 dark:text-blue-400">$1</span> <span class="font-semibold">$2</span>'
      );
      
      // Bold
      highlighted = highlighted.replace(
        /\*\*([^*]+)\*\*/g,
        '<span class="text-orange-600 dark:text-orange-400">**</span><span class="font-semibold">$1</span><span class="text-orange-600 dark:text-orange-400">**</span>'
      );
      
      // Italic
      highlighted = highlighted.replace(
        /\*([^*]+)\*/g,
        '<span class="text-orange-600 dark:text-orange-400">*</span><span class="italic">$1</span><span class="text-orange-600 dark:text-orange-400">*</span>'
      );
      
      // Code blocks - ensure escaped content stays escaped
      highlighted = highlighted.replace(
        /```([\\w]*)\n([\\s\\S]*?)```/g,
        '<span class="text-green-600 dark:text-green-400">```$1</span>\n<span class="bg-gray-100 dark:bg-gray-800">$2</span><span class="text-green-600 dark:text-green-400">```</span>'
      );
      
      // Inline code - ensure escaped content stays escaped
      highlighted = highlighted.replace(
        /`([^`]+)`/g,
        '<span class="text-green-600 dark:text-green-400">`</span><span class="bg-gray-100 dark:bg-gray-800 px-1">$1</span><span class="text-green-600 dark:text-green-400">`</span>'
      );
      
      // Links
      highlighted = highlighted.replace(
        /\[([^\]]+)\]\(([^)]+)\)/g,
        '<span class="text-purple-600 dark:text-purple-400">[</span>$1<span class="text-purple-600 dark:text-purple-400">](</span><span class="text-blue-600 dark:text-blue-400 underline">$2</span><span class="text-purple-600 dark:text-purple-400">)</span>'
      );
      
      // Lists
      highlighted = highlighted.replace(
        /^(\s*)([-*+]|\d+\.)\s+/gm,
        '$1<span class="text-gray-600 dark:text-gray-400">$2</span> '
      );
      
      // Blockquotes
      highlighted = highlighted.replace(
        /^&gt;\s+(.*)$/gm,
        '<span class="text-gray-600 dark:text-gray-400">&gt;</span> <span class="text-gray-700 dark:text-gray-300">$1</span>'
      );
    }

    return highlighted;
  }, [code, language]);

  const lines = useMemo(() => {
    return code.split('\n');
  }, [code]);

  return (
    <div className={cn("font-mono text-sm overflow-auto", className)}>
      {showLineNumbers ? (
        <div className="flex">
          <div className="select-none text-gray-500 dark:text-gray-600 pr-4 text-right">
            {lines.map((_, index) => (
              <div key={index} className="leading-6">
                {index + 1}
              </div>
            ))}
          </div>
          <div className="flex-1 overflow-x-auto">
            {lines.map((line, index) => (
              <div
                key={index}
                className="leading-6 whitespace-pre"
                dangerouslySetInnerHTML={{
                  __html: sanitizeSyntaxHighlight(highlightLine(line, index, language, highlightedCode))
                }}
              />
            ))}
          </div>
        </div>
      ) : (
        <div
          className="whitespace-pre-wrap"
          dangerouslySetInnerHTML={{ __html: sanitizeSyntaxHighlight(highlightedCode) }}
        />
      )}
    </div>
  );
}

function highlightLine(
  line: string,
  lineIndex: number,
  language: string,
  fullHighlighted: string
): string {
  // Extract the highlighted version of this specific line
  const highlightedLines = fullHighlighted.split('\n');
  return highlightedLines[lineIndex] || escapeHtml(line);
}