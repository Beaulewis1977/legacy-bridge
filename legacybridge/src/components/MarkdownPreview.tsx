'use client';

import { useMemo } from 'react';
import { cn } from '@/lib/utils';

interface MarkdownPreviewProps {
  content: string;
  className?: string;
  showLineNumbers?: boolean;
}

export function MarkdownPreview({ 
  content, 
  className,
  showLineNumbers = false 
}: MarkdownPreviewProps) {
  // Parse and render markdown content
  const renderedContent = useMemo(() => {
    if (!content) return '';

    // Simple markdown parser for preview
    // In production, you'd use a library like react-markdown
    let html = content;

    // Headers
    html = html.replace(/^### (.*$)/gim, '<h3 class="text-lg font-semibold mt-4 mb-2">$1</h3>');
    html = html.replace(/^## (.*$)/gim, '<h2 class="text-xl font-bold mt-6 mb-3">$1</h2>');
    html = html.replace(/^# (.*$)/gim, '<h1 class="text-2xl font-bold mt-8 mb-4">$1</h1>');

    // Bold
    html = html.replace(/\*\*(.+?)\*\*/g, '<strong class="font-semibold">$1</strong>');
    html = html.replace(/__(.+?)__/g, '<strong class="font-semibold">$1</strong>');

    // Italic
    html = html.replace(/\*(.+?)\*/g, '<em class="italic">$1</em>');
    html = html.replace(/_(.+?)_/g, '<em class="italic">$1</em>');

    // Code blocks
    html = html.replace(/```(\w+)?\n([\s\S]*?)```/g, (match, lang, code) => {
      return `<pre class="bg-muted p-4 rounded-lg overflow-x-auto my-4"><code class="text-sm font-mono">${escapeHtml(code.trim())}</code></pre>`;
    });

    // Inline code
    html = html.replace(/`(.+?)`/g, '<code class="bg-muted px-1.5 py-0.5 rounded text-sm font-mono">$1</code>');

    // Links
    html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" class="text-primary underline hover:no-underline">$1</a>');

    // Lists
    html = html.replace(/^\* (.+)$/gim, '<li class="ml-4 list-disc">$1</li>');
    html = html.replace(/^\d+\. (.+)$/gim, '<li class="ml-4 list-decimal">$1</li>');
    html = html.replace(/(<li.*<\/li>)\n(?!<li)/g, '$1</ul>\n');
    html = html.replace(/(?<!<\/ul>)\n(<li)/g, '\n<ul class="my-2">$1');

    // Blockquotes
    html = html.replace(/^> (.+)$/gim, '<blockquote class="border-l-4 border-muted-foreground/30 pl-4 py-1 my-2 text-muted-foreground">$1</blockquote>');

    // Horizontal rules
    html = html.replace(/^---$/gim, '<hr class="my-4 border-t border-border" />');

    // Tables
    html = html.replace(/\|(.+)\|/g, (match) => {
      const cells = match.split('|').filter(cell => cell.trim());
      const isHeader = cells.every(cell => cell.includes('---'));
      
      if (isHeader) {
        return '';
      }
      
      const cellsHtml = cells.map(cell => 
        `<td class="border border-border px-3 py-2">${cell.trim()}</td>`
      ).join('');
      
      return `<tr>${cellsHtml}</tr>`;
    });

    // Wrap table rows in table
    html = html.replace(/(<tr>[\s\S]*?<\/tr>)/g, (match) => {
      return `<table class="border-collapse border border-border my-4">${match}</table>`;
    });

    // Paragraphs
    html = html.replace(/\n\n/g, '</p><p class="mb-4">');
    html = `<p class="mb-4">${html}</p>`;

    // Clean up empty paragraphs
    html = html.replace(/<p class="mb-4"><\/p>/g, '');
    html = html.replace(/<p class="mb-4">(<h[1-6]|<ul|<ol|<blockquote|<pre|<hr|<table)/g, '$1');
    html = html.replace(/(<\/h[1-6]>|<\/ul>|<\/ol>|<\/blockquote>|<\/pre>|<\/table>)<\/p>/g, '$1');

    return html;
  }, [content]);

  // Escape HTML to prevent XSS
  function escapeHtml(text: string): string {
    const map: Record<string, string> = {
      '&': '&amp;',
      '<': '&lt;',
      '>': '&gt;',
      '"': '&quot;',
      "'": '&#039;'
    };
    return text.replace(/[&<>"']/g, m => map[m]);
  }

  // Add line numbers if requested
  const contentWithLineNumbers = useMemo(() => {
    if (!showLineNumbers || !content) return renderedContent;

    const lines = content.split('\n');
    return lines.map((line, index) => {
      const lineNumber = index + 1;
      return `
        <div class="flex">
          <span class="select-none text-muted-foreground text-xs font-mono pr-4 min-w-[3ch] text-right">
            ${lineNumber}
          </span>
          <div class="flex-1">${line || '&nbsp;'}</div>
        </div>
      `;
    }).join('');
  }, [content, renderedContent, showLineNumbers]);

  return (
    <div 
      className={cn(
        "prose prose-sm dark:prose-invert max-w-none",
        "prose-headings:scroll-mt-4",
        "prose-code:before:content-none prose-code:after:content-none",
        "prose-pre:bg-muted prose-pre:text-foreground",
        className
      )}
    >
      {showLineNumbers ? (
        <div className="font-mono text-sm">
          <div dangerouslySetInnerHTML={{ __html: contentWithLineNumbers }} />
        </div>
      ) : (
        <div dangerouslySetInnerHTML={{ __html: renderedContent }} />
      )}
    </div>
  );
}