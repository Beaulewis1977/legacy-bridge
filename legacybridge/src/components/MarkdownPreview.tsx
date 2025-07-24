'use client';

import { useMemo } from 'react';
import { cn } from '@/lib/utils';
import { sanitizeMarkdown, escapeHtml } from '@/lib/sanitizer';

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

    // Headers - escape content to prevent XSS
    html = html.replace(/^### (.*$)/gim, (match, header) => {
      return `<h3 class="text-lg font-semibold mt-4 mb-2">${escapeHtml(header)}</h3>`;
    });
    html = html.replace(/^## (.*$)/gim, (match, header) => {
      return `<h2 class="text-xl font-bold mt-6 mb-3">${escapeHtml(header)}</h2>`;
    });
    html = html.replace(/^# (.*$)/gim, (match, header) => {
      return `<h1 class="text-2xl font-bold mt-8 mb-4">${escapeHtml(header)}</h1>`;
    });

    // Bold - escape content
    html = html.replace(/\*\*(.+?)\*\*/g, (match, text) => {
      return `<strong class="font-semibold">${escapeHtml(text)}</strong>`;
    });
    html = html.replace(/__(.+?)__/g, (match, text) => {
      return `<strong class="font-semibold">${escapeHtml(text)}</strong>`;
    });

    // Italic - escape content
    html = html.replace(/\*(.+?)\*/g, (match, text) => {
      return `<em class="italic">${escapeHtml(text)}</em>`;
    });
    html = html.replace(/_(.+?)_/g, (match, text) => {
      return `<em class="italic">${escapeHtml(text)}</em>`;
    });

    // Code blocks
    html = html.replace(/```(\w+)?\n([\s\S]*?)```/g, (match, lang, code) => {
      return `<pre class="bg-muted p-4 rounded-lg overflow-x-auto my-4"><code class="text-sm font-mono">${escapeHtml(code.trim())}</code></pre>`;
    });

    // Inline code - escape content to prevent XSS
    html = html.replace(/`(.+?)`/g, (match, code) => {
      return `<code class="bg-muted px-1.5 py-0.5 rounded text-sm font-mono">${escapeHtml(code)}</code>`;
    });

    // Links - escape content and validate URLs
    html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (match, text, url) => {
      // Only allow safe URLs
      if (!/^(https?:\/\/|mailto:|#)/.test(url)) {
        return escapeHtml(match);
      }
      return `<a href="${escapeHtml(url)}" class="text-primary underline hover:no-underline">${escapeHtml(text)}</a>`;
    });

    // Lists - escape content
    html = html.replace(/^\* (.+)$/gim, (match, item) => {
      return `<li class="ml-4 list-disc">${escapeHtml(item)}</li>`;
    });
    html = html.replace(/^\d+\. (.+)$/gim, (match, item) => {
      return `<li class="ml-4 list-decimal">${escapeHtml(item)}</li>`;
    });
    html = html.replace(/(<li.*<\/li>)\n(?!<li)/g, '$1</ul>\n');
    html = html.replace(/(?<!<\/ul>)\n(<li)/g, '\n<ul class="my-2">$1');

    // Blockquotes - escape content
    html = html.replace(/^> (.+)$/gim, (match, quote) => {
      return `<blockquote class="border-l-4 border-muted-foreground/30 pl-4 py-1 my-2 text-muted-foreground">${escapeHtml(quote)}</blockquote>`;
    });

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
        `<td class="border border-border px-3 py-2">${escapeHtml(cell.trim())}</td>`
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
          <div class="flex-1">${escapeHtml(line) || '&nbsp;'}</div>
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
          <div dangerouslySetInnerHTML={{ __html: sanitizeMarkdown(contentWithLineNumbers) }} />
        </div>
      ) : (
        <div dangerouslySetInnerHTML={{ __html: sanitizeMarkdown(renderedContent) }} />
      )}
    </div>
  );
}