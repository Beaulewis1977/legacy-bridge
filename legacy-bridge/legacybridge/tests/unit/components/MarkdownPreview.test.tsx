import React from 'react';
import { render, screen } from '@testing-library/react';
import { jest } from '@jest/globals';
import { MarkdownPreview } from '@/components/MarkdownPreview';
import { sanitizeMarkdown, escapeHtml } from '@/lib/sanitizer';

// Mock the sanitizer functions
jest.mock('@/lib/sanitizer', () => ({
  sanitizeMarkdown: jest.fn((html: string) => html),
  escapeHtml: jest.fn((text: string) => {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }),
}));

describe('MarkdownPreview Component', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Basic Rendering', () => {
    it('should render empty content correctly', () => {
      const { container } = render(<MarkdownPreview content="" />);
      
      expect(container.querySelector('.prose')).toBeInTheDocument();
      expect(container.textContent).toBe('');
    });

    it('should render plain text', () => {
      render(<MarkdownPreview content="This is plain text" />);
      
      expect(screen.getByText('This is plain text')).toBeInTheDocument();
    });

    it('should apply custom className', () => {
      const { container } = render(
        <MarkdownPreview content="Test" className="custom-class" />
      );
      
      expect(container.firstChild).toHaveClass('custom-class');
      expect(container.firstChild).toHaveClass('prose');
    });
  });

  describe('Headers', () => {
    it('should render h1 headers', () => {
      const { container } = render(<MarkdownPreview content="# Header 1" />);
      
      const h1 = container.querySelector('h1');
      expect(h1).toBeInTheDocument();
      expect(h1).toHaveTextContent('Header 1');
      expect(h1).toHaveClass('text-2xl', 'font-bold', 'mt-8', 'mb-4');
    });

    it('should render h2 headers', () => {
      const { container } = render(<MarkdownPreview content="## Header 2" />);
      
      const h2 = container.querySelector('h2');
      expect(h2).toBeInTheDocument();
      expect(h2).toHaveTextContent('Header 2');
      expect(h2).toHaveClass('text-xl', 'font-bold', 'mt-6', 'mb-3');
    });

    it('should render h3 headers', () => {
      const { container } = render(<MarkdownPreview content="### Header 3" />);
      
      const h3 = container.querySelector('h3');
      expect(h3).toBeInTheDocument();
      expect(h3).toHaveTextContent('Header 3');
      expect(h3).toHaveClass('text-lg', 'font-semibold', 'mt-4', 'mb-2');
    });

    it('should handle multiple headers', () => {
      const content = `# H1\n## H2\n### H3`;
      const { container } = render(<MarkdownPreview content={content} />);
      
      expect(container.querySelector('h1')).toHaveTextContent('H1');
      expect(container.querySelector('h2')).toHaveTextContent('H2');
      expect(container.querySelector('h3')).toHaveTextContent('H3');
    });

    it('should escape HTML in headers', () => {
      render(<MarkdownPreview content="# <script>alert('xss')</script>" />);
      
      expect(escapeHtml).toHaveBeenCalledWith("<script>alert('xss')</script>");
      expect(screen.queryByText('alert')).not.toBeInTheDocument();
    });
  });

  describe('Text Formatting', () => {
    it('should render bold text with **', () => {
      const { container } = render(<MarkdownPreview content="This is **bold** text" />);
      
      const strong = container.querySelector('strong');
      expect(strong).toBeInTheDocument();
      expect(strong).toHaveTextContent('bold');
      expect(strong).toHaveClass('font-semibold');
    });

    it('should render bold text with __', () => {
      const { container } = render(<MarkdownPreview content="This is __bold__ text" />);
      
      const strong = container.querySelector('strong');
      expect(strong).toBeInTheDocument();
      expect(strong).toHaveTextContent('bold');
    });

    it('should render italic text with *', () => {
      const { container } = render(<MarkdownPreview content="This is *italic* text" />);
      
      const em = container.querySelector('em');
      expect(em).toBeInTheDocument();
      expect(em).toHaveTextContent('italic');
      expect(em).toHaveClass('italic');
    });

    it('should render italic text with _', () => {
      const { container } = render(<MarkdownPreview content="This is _italic_ text" />);
      
      const em = container.querySelector('em');
      expect(em).toBeInTheDocument();
      expect(em).toHaveTextContent('italic');
    });

    it('should handle nested formatting', () => {
      const { container } = render(<MarkdownPreview content="This is **bold and *italic* text**" />);
      
      const strong = container.querySelector('strong');
      const em = container.querySelector('em');
      expect(strong).toBeInTheDocument();
      expect(em).toBeInTheDocument();
    });

    it('should escape HTML in formatted text', () => {
      render(<MarkdownPreview content="**<img src=x onerror=alert('xss')>**" />);
      
      expect(escapeHtml).toHaveBeenCalledWith("<img src=x onerror=alert('xss')>");
    });
  });

  describe('Code Formatting', () => {
    it('should render inline code', () => {
      const { container } = render(<MarkdownPreview content="Use `code` inline" />);
      
      const code = container.querySelector('code');
      expect(code).toBeInTheDocument();
      expect(code).toHaveTextContent('code');
      expect(code).toHaveClass('bg-muted', 'px-1.5', 'py-0.5', 'rounded', 'text-sm', 'font-mono');
    });

    it('should render code blocks', () => {
      const content = '```\nfunction test() {\n  return true;\n}\n```';
      const { container } = render(<MarkdownPreview content={content} />);
      
      const pre = container.querySelector('pre');
      const code = pre?.querySelector('code');
      expect(pre).toBeInTheDocument();
      expect(pre).toHaveClass('bg-muted', 'p-4', 'rounded-lg', 'overflow-x-auto', 'my-4');
      expect(code).toHaveTextContent('function test() {\n  return true;\n}');
    });

    it('should handle code blocks with language', () => {
      const content = '```javascript\nconsole.log("test");\n```';
      const { container } = render(<MarkdownPreview content={content} />);
      
      const code = container.querySelector('code');
      expect(code).toHaveTextContent('console.log("test");');
    });

    it('should escape HTML in code', () => {
      render(<MarkdownPreview content="`<script>alert('xss')</script>`" />);
      
      expect(escapeHtml).toHaveBeenCalledWith("<script>alert('xss')</script>");
    });
  });

  describe('Links', () => {
    it('should render links with proper styling', () => {
      const { container } = render(<MarkdownPreview content="[Link text](https://example.com)" />);
      
      const link = container.querySelector('a');
      expect(link).toBeInTheDocument();
      expect(link).toHaveAttribute('href', 'https://example.com');
      expect(link).toHaveTextContent('Link text');
      expect(link).toHaveClass('text-primary', 'underline', 'hover:no-underline');
    });

    it('should allow mailto links', () => {
      const { container } = render(<MarkdownPreview content="[Email](mailto:test@example.com)" />);
      
      const link = container.querySelector('a');
      expect(link).toHaveAttribute('href', 'mailto:test@example.com');
    });

    it('should allow anchor links', () => {
      const { container } = render(<MarkdownPreview content="[Section](#section)" />);
      
      const link = container.querySelector('a');
      expect(link).toHaveAttribute('href', '#section');
    });

    it('should reject unsafe URLs', () => {
      const { container } = render(<MarkdownPreview content="[Unsafe](javascript:alert('xss'))" />);
      
      const link = container.querySelector('a');
      expect(link).not.toBeInTheDocument();
      expect(escapeHtml).toHaveBeenCalledWith("[Unsafe](javascript:alert('xss'))");
    });

    it('should escape HTML in link text', () => {
      render(<MarkdownPreview content="[<script>alert('xss')</script>](https://example.com)" />);
      
      expect(escapeHtml).toHaveBeenCalledWith("<script>alert('xss')</script>");
    });
  });

  describe('Lists', () => {
    it('should render unordered lists', () => {
      const content = '* Item 1\n* Item 2\n* Item 3';
      const { container } = render(<MarkdownPreview content={content} />);
      
      const ul = container.querySelector('ul');
      const items = container.querySelectorAll('li');
      
      expect(ul).toBeInTheDocument();
      expect(ul).toHaveClass('my-2');
      expect(items).toHaveLength(3);
      expect(items[0]).toHaveTextContent('Item 1');
      expect(items[0]).toHaveClass('ml-4', 'list-disc');
    });

    it('should render ordered lists', () => {
      const content = '1. First\n2. Second\n3. Third';
      const { container } = render(<MarkdownPreview content={content} />);
      
      const items = container.querySelectorAll('li');
      
      expect(items).toHaveLength(3);
      expect(items[0]).toHaveTextContent('First');
      expect(items[0]).toHaveClass('ml-4', 'list-decimal');
    });

    it('should handle mixed content with lists', () => {
      const content = 'Text before\n\n* List item\n\nText after';
      const { container } = render(<MarkdownPreview content={content} />);
      
      expect(container.textContent).toContain('Text before');
      expect(container.querySelector('li')).toHaveTextContent('List item');
      expect(container.textContent).toContain('Text after');
    });

    it('should escape HTML in list items', () => {
      render(<MarkdownPreview content="* <img src=x onerror=alert('xss')>" />);
      
      expect(escapeHtml).toHaveBeenCalledWith("<img src=x onerror=alert('xss')>");
    });
  });

  describe('Blockquotes', () => {
    it('should render blockquotes', () => {
      const { container } = render(<MarkdownPreview content="> This is a quote" />);
      
      const blockquote = container.querySelector('blockquote');
      expect(blockquote).toBeInTheDocument();
      expect(blockquote).toHaveTextContent('This is a quote');
      expect(blockquote).toHaveClass('border-l-4', 'border-muted-foreground/30', 'pl-4', 'py-1', 'my-2', 'text-muted-foreground');
    });

    it('should handle multiple blockquote lines', () => {
      const content = '> Line 1\n> Line 2';
      const { container } = render(<MarkdownPreview content={content} />);
      
      const blockquotes = container.querySelectorAll('blockquote');
      expect(blockquotes).toHaveLength(2);
    });

    it('should escape HTML in blockquotes', () => {
      render(<MarkdownPreview content="> <script>alert('xss')</script>" />);
      
      expect(escapeHtml).toHaveBeenCalledWith("<script>alert('xss')</script>");
    });
  });

  describe('Horizontal Rules', () => {
    it('should render horizontal rules', () => {
      const { container } = render(<MarkdownPreview content="Text\n---\nMore text" />);
      
      const hr = container.querySelector('hr');
      expect(hr).toBeInTheDocument();
      expect(hr).toHaveClass('my-4', 'border-t', 'border-border');
    });
  });

  describe('Tables', () => {
    it('should render basic tables', () => {
      const content = '| Header 1 | Header 2 |\n| --- | --- |\n| Cell 1 | Cell 2 |';
      const { container } = render(<MarkdownPreview content={content} />);
      
      const table = container.querySelector('table');
      const cells = container.querySelectorAll('td');
      
      expect(table).toBeInTheDocument();
      expect(table).toHaveClass('border-collapse', 'border', 'border-border', 'my-4');
      expect(cells).toHaveLength(2);
      expect(cells[0]).toHaveTextContent('Cell 1');
      expect(cells[0]).toHaveClass('border', 'border-border', 'px-3', 'py-2');
    });

    it('should escape HTML in table cells', () => {
      const content = '| <script>alert("xss")</script> |';
      render(<MarkdownPreview content={content} />);
      
      expect(escapeHtml).toHaveBeenCalledWith('<script>alert("xss")</script>');
    });
  });

  describe('Paragraphs', () => {
    it('should wrap text in paragraphs', () => {
      const content = 'Paragraph 1\n\nParagraph 2';
      const { container } = render(<MarkdownPreview content={content} />);
      
      const paragraphs = container.querySelectorAll('p');
      expect(paragraphs.length).toBeGreaterThanOrEqual(2);
      expect(container.textContent).toContain('Paragraph 1');
      expect(container.textContent).toContain('Paragraph 2');
    });

    it('should handle empty paragraphs', () => {
      const content = 'Text\n\n\n\nMore text';
      const { container } = render(<MarkdownPreview content={content} />);
      
      // Should not have empty paragraph tags
      const emptyParagraphs = Array.from(container.querySelectorAll('p')).filter(
        p => p.textContent?.trim() === ''
      );
      expect(emptyParagraphs).toHaveLength(0);
    });
  });

  describe('Line Numbers', () => {
    it('should show line numbers when enabled', () => {
      const content = 'Line 1\nLine 2\nLine 3';
      const { container } = render(<MarkdownPreview content={content} showLineNumbers />);
      
      expect(container.querySelector('.font-mono')).toBeInTheDocument();
      expect(screen.getByText('1')).toBeInTheDocument();
      expect(screen.getByText('2')).toBeInTheDocument();
      expect(screen.getByText('3')).toBeInTheDocument();
    });

    it('should not show line numbers by default', () => {
      const content = 'Line 1\nLine 2';
      render(<MarkdownPreview content={content} />);
      
      // Line numbers should not be present
      expect(screen.queryByText('1')).not.toBeInTheDocument();
    });

    it('should handle empty lines with line numbers', () => {
      const content = 'Line 1\n\nLine 3';
      const { container } = render(<MarkdownPreview content={content} showLineNumbers />);
      
      expect(screen.getByText('1')).toBeInTheDocument();
      expect(screen.getByText('2')).toBeInTheDocument();
      expect(screen.getByText('3')).toBeInTheDocument();
      
      // Empty line should have non-breaking space
      const lines = container.querySelectorAll('.flex');
      expect(lines[1].innerHTML).toContain('&nbsp;');
    });

    it('should style line numbers correctly', () => {
      const { container } = render(<MarkdownPreview content="Test" showLineNumbers />);
      
      const lineNumber = container.querySelector('.select-none');
      expect(lineNumber).toHaveClass('text-muted-foreground', 'text-xs', 'font-mono', 'pr-4', 'min-w-[3ch]', 'text-right');
    });
  });

  describe('Security', () => {
    it('should sanitize all content', () => {
      const content = '# Header\n**Bold**\n[Link](https://example.com)';
      render(<MarkdownPreview content={content} />);
      
      expect(sanitizeMarkdown).toHaveBeenCalled();
    });

    it('should escape all user content', () => {
      const maliciousContent = `
        # <script>alert('h1')</script>
        **<img src=x onerror=alert('bold')>**
        *<svg onload=alert('italic')>*
        \`<iframe src="javascript:alert('code')">\`
        [<script>alert('link')</script>](https://example.com)
        * <script>alert('list')</script>
        > <script>alert('quote')</script>
      `;
      
      render(<MarkdownPreview content={maliciousContent} />);
      
      // escapeHtml should be called for each piece of content
      const calls = (escapeHtml as jest.Mock).mock.calls;
      expect(calls.some(call => call[0].includes('<script>alert(\'h1\')</script>'))).toBe(true);
      expect(calls.some(call => call[0].includes('<img src=x onerror=alert(\'bold\')>'))).toBe(true);
    });

    it('should not execute scripts in content', () => {
      const alertSpy = jest.spyOn(window, 'alert').mockImplementation(() => {});
      
      render(<MarkdownPreview content="<script>alert('xss')</script>" />);
      
      expect(alertSpy).not.toHaveBeenCalled();
      alertSpy.mockRestore();
    });
  });

  describe('Complex Content', () => {
    it('should handle mixed markdown elements', () => {
      const content = `
# Main Title

This is a paragraph with **bold** and *italic* text.

## Subsection

Here's a list:
* Item 1
* Item 2 with \`inline code\`

\`\`\`javascript
function example() {
  return true;
}
\`\`\`

> A blockquote with [a link](https://example.com)

---

| Column 1 | Column 2 |
| --- | --- |
| Data 1 | Data 2 |
      `;
      
      const { container } = render(<MarkdownPreview content={content} />);
      
      expect(container.querySelector('h1')).toHaveTextContent('Main Title');
      expect(container.querySelector('h2')).toHaveTextContent('Subsection');
      expect(container.querySelector('strong')).toHaveTextContent('bold');
      expect(container.querySelector('em')).toHaveTextContent('italic');
      expect(container.querySelector('li')).toHaveTextContent('Item 1');
      expect(container.querySelector('pre code')).toHaveTextContent('function example()');
      expect(container.querySelector('blockquote')).toBeInTheDocument();
      expect(container.querySelector('hr')).toBeInTheDocument();
      expect(container.querySelector('table')).toBeInTheDocument();
    });

    it('should preserve whitespace in code blocks', () => {
      const content = '```\n  indented\n    more indented\n```';
      const { container } = render(<MarkdownPreview content={content} />);
      
      const code = container.querySelector('code');
      expect(code?.textContent).toContain('  indented');
      expect(code?.textContent).toContain('    more indented');
    });
  });

  describe('Performance', () => {
    it('should memoize rendered content', () => {
      const content = '# Test';
      const { rerender } = render(<MarkdownPreview content={content} />);
      
      const initialCalls = (escapeHtml as jest.Mock).mock.calls.length;
      
      // Re-render with same content
      rerender(<MarkdownPreview content={content} />);
      
      // escapeHtml should not be called again due to memoization
      expect((escapeHtml as jest.Mock).mock.calls.length).toBe(initialCalls);
    });

    it('should re-render when content changes', () => {
      const { rerender } = render(<MarkdownPreview content="# First" />);
      
      const initialCalls = (escapeHtml as jest.Mock).mock.calls.length;
      
      // Re-render with different content
      rerender(<MarkdownPreview content="# Second" />);
      
      // escapeHtml should be called again for new content
      expect((escapeHtml as jest.Mock).mock.calls.length).toBeGreaterThan(initialCalls);
    });
  });

  describe('Edge Cases', () => {
    it('should handle null/undefined content gracefully', () => {
      const { container } = render(<MarkdownPreview content={null as any} />);
      
      expect(container.textContent).toBe('');
    });

    it('should handle very long content', () => {
      const longContent = 'Lorem ipsum '.repeat(1000);
      const { container } = render(<MarkdownPreview content={longContent} />);
      
      expect(container.textContent).toContain('Lorem ipsum');
    });

    it('should handle special characters', () => {
      const content = 'Text with special chars: < > & " \' © ™ €';
      render(<MarkdownPreview content={content} />);
      
      expect(escapeHtml).toHaveBeenCalled();
    });

    it('should handle Unicode content', () => {
      const content = '# 你好世界 🌍\n\nEmoji: 😀 💻 🚀';
      const { container } = render(<MarkdownPreview content={content} />);
      
      expect(container.textContent).toContain('你好世界');
      expect(container.textContent).toContain('😀');
    });
  });
});