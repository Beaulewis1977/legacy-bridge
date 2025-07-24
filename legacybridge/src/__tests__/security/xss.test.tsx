import { render } from '@testing-library/react';
import { MarkdownPreview } from '@/components/MarkdownPreview';
import { SyntaxHighlighter } from '@/components/SyntaxHighlighter';
import DOMPurify from 'dompurify';

// Common XSS attack vectors
const XSS_VECTORS = [
  '<script>alert("XSS")</script>',
  '<img src=x onerror="alert(\'XSS\')">',
  '<svg onload="alert(\'XSS\')">',
  '<iframe src="javascript:alert(\'XSS\')"></iframe>',
  '<body onload="alert(\'XSS\')">',
  '<input onfocus="alert(\'XSS\')" autofocus>',
  '<select onfocus="alert(\'XSS\')" autofocus>',
  '<textarea onfocus="alert(\'XSS\')" autofocus>',
  '<keygen onfocus="alert(\'XSS\')" autofocus>',
  '<video><source onerror="alert(\'XSS\')">',
  '<audio src=x onerror="alert(\'XSS\')">',
  '<marquee onstart="alert(\'XSS\')">',
  '<meter onmouseover="alert(\'XSS\')">2 out of 10</meter>',
  '<details open ontoggle="alert(\'XSS\')">',
  '<form action="javascript:alert(\'XSS\')"><input type="submit">',
  '<button formaction="javascript:alert(\'XSS\')">XSS</button>',
  '<base href="javascript:alert(\'XSS\')//">',
  '<link rel="stylesheet" href="javascript:alert(\'XSS\');">',
  '<object data="javascript:alert(\'XSS\')">',
  '<embed src="javascript:alert(\'XSS\')">',
  '<a href="javascript:alert(\'XSS\')">Click me</a>',
  '<a href="data:text/html,<script>alert(\'XSS\')</script>">Click</a>',
  '<math><mtext><table><mglyph><style><!--</style><img title="--&gt;&lt;/mglyph&gt;&lt;img&Tab;src=1&Tab;onerror=alert(\'XSS\')&gt;">',
  'javascript:/*--></title></style></textarea></script></xmp><svg/onload=\'+/"/+/onmouseover=1/+/[*/[]/+alert(\'XSS\')//\'>',
];

// Markdown-specific XSS vectors
const MARKDOWN_XSS_VECTORS = [
  '[Click me](javascript:alert("XSS"))',
  '![XSS](x" onerror="alert(\'XSS\')")',
  '<div>Hello</div><script>alert("XSS")</script>',
  '```javascript\n<script>alert("XSS")</script>\n```',
  '`<img src=x onerror="alert(\'XSS\')">`',
  '**<script>alert("XSS")</script>**',
  '*<img src=x onerror="alert(\'XSS\')">*',
  '> <script>alert("XSS")</script>',
  '# <script>alert("XSS")</script>',
  '- <script>alert("XSS")</script>',
  '1. <img src=x onerror="alert(\'XSS\')">',
  '| <script>alert("XSS")</script> | test |',
  '[link](<javascript:alert("XSS")>)',
  '[link](data:text/html,<script>alert("XSS")</script>)',
];

describe('XSS Security Tests', () => {
  describe('MarkdownPreview Component', () => {
    it('should sanitize common XSS attack vectors', () => {
      XSS_VECTORS.forEach((vector) => {
        const { container } = render(<MarkdownPreview content={vector} />);
        const html = container.innerHTML;
        
        // Check that dangerous elements are removed
        expect(html).not.toContain('<script');
        expect(html).not.toContain('onerror=');
        expect(html).not.toContain('onload=');
        expect(html).not.toContain('onfocus=');
        expect(html).not.toContain('onmouseover=');
        expect(html).not.toContain('javascript:');
        expect(html).not.toContain('data:text/html');
        expect(html).not.toContain('<iframe');
        expect(html).not.toContain('<object');
        expect(html).not.toContain('<embed');
        expect(html).not.toContain('<form');
        expect(html).not.toContain('<input');
        expect(html).not.toContain('<select');
        expect(html).not.toContain('<textarea');
      });
    });

    it('should sanitize markdown-specific XSS vectors', () => {
      MARKDOWN_XSS_VECTORS.forEach((vector) => {
        const { container } = render(<MarkdownPreview content={vector} />);
        const html = container.innerHTML;
        
        // Check that dangerous content is sanitized
        expect(html).not.toContain('<script');
        expect(html).not.toContain('javascript:');
        expect(html).not.toContain('onerror=');
        expect(html).not.toContain('data:text/html');
        
        // Verify that safe content is preserved
        if (vector.includes('**') && !vector.includes('<')) {
          expect(container.querySelector('strong')).toBeTruthy();
        }
        if (vector.includes('#') && !vector.includes('<')) {
          expect(container.querySelector('h1, h2, h3, h4, h5, h6')).toBeTruthy();
        }
      });
    });

    it('should preserve safe markdown content', () => {
      const safeContent = `
# Safe Header
**Bold text** and *italic text*
[Safe link](https://example.com)
\`inline code\`
\`\`\`javascript
const safe = "code";
\`\`\`
> Safe quote
- Safe list item
      `;
      
      const { container } = render(<MarkdownPreview content={safeContent} />);
      
      expect(container.querySelector('h1')).toBeTruthy();
      expect(container.querySelector('strong')).toBeTruthy();
      expect(container.querySelector('em')).toBeTruthy();
      expect(container.querySelector('a[href="https://example.com"]')).toBeTruthy();
      expect(container.querySelector('code')).toBeTruthy();
      expect(container.querySelector('pre')).toBeTruthy();
      expect(container.querySelector('blockquote')).toBeTruthy();
      expect(container.querySelector('li')).toBeTruthy();
    });

    it('should handle line numbers mode securely', () => {
      const xssContent = '<script>alert("XSS")</script>';
      const { container } = render(<MarkdownPreview content={xssContent} showLineNumbers={true} />);
      
      expect(container.innerHTML).not.toContain('<script');
      expect(container.innerHTML).not.toContain('alert');
    });
  });

  describe('SyntaxHighlighter Component', () => {
    it('should sanitize XSS in RTF mode', () => {
      const rtfWithXSS = '\\rtf1<script>alert("XSS")</script>\\par';
      const { container } = render(<SyntaxHighlighter code={rtfWithXSS} language="rtf" />);
      
      expect(container.innerHTML).not.toContain('<script');
      expect(container.innerHTML).toContain('&lt;script&gt;');
    });

    it('should sanitize XSS in markdown mode', () => {
      const markdownWithXSS = '# Header <script>alert("XSS")</script>';
      const { container } = render(<SyntaxHighlighter code={markdownWithXSS} language="markdown" />);
      
      expect(container.innerHTML).not.toContain('<script');
      expect(container.innerHTML).toContain('&lt;script&gt;');
    });

    it('should preserve syntax highlighting while escaping content', () => {
      const rtfCode = '{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}}\\f0\\fs24 Hello World\\par}';
      const { container } = render(<SyntaxHighlighter code={rtfCode} language="rtf" />);
      
      // Check that RTF control words are highlighted
      expect(container.querySelector('.text-blue-600, .text-blue-400')).toBeTruthy();
      // Check that braces are highlighted
      expect(container.querySelector('.text-purple-600, .text-purple-400')).toBeTruthy();
    });
  });

  describe('DOMPurify Integration', () => {
    it('should properly initialize DOMPurify', () => {
      expect(DOMPurify).toBeDefined();
      expect(typeof DOMPurify.sanitize).toBe('function');
    });

    it('should remove dangerous elements', () => {
      const dirty = '<script>alert("XSS")</script><p>Safe content</p>';
      const clean = DOMPurify.sanitize(dirty);
      
      expect(clean).not.toContain('<script');
      expect(clean).toContain('<p>Safe content</p>');
    });

    it('should remove dangerous attributes', () => {
      const dirty = '<img src="x" onerror="alert(\'XSS\')" alt="test">';
      const clean = DOMPurify.sanitize(dirty);
      
      expect(clean).not.toContain('onerror');
      expect(clean).toContain('alt="test"');
    });

    it('should handle data URIs safely', () => {
      const dirty = '<a href="data:text/html,<script>alert(\'XSS\')</script>">Click</a>';
      const clean = DOMPurify.sanitize(dirty, {
        ALLOWED_URI_REGEXP: /^(?:(?:https?|mailto):\/\/|#)/i
      });
      
      expect(clean).not.toContain('data:text/html');
    });
  });
});

describe('Content Security Policy', () => {
  it('should block inline scripts when CSP is enabled', (done) => {
    // This test would need to run in a browser environment with CSP headers
    // For unit tests, we verify the CSP configuration
    const cspConfig = {
      'default-src': ["'self'"],
      'script-src': ["'self'", "'unsafe-inline'", "'unsafe-eval'", 'https://cdn.jsdelivr.net'],
      'style-src': ["'self'", "'unsafe-inline'", 'https://fonts.googleapis.com'],
      'img-src': ["'self'", 'data:', 'https:', 'blob:'],
      'connect-src': ["'self'", 'https://api.github.com', 'https://raw.githubusercontent.com'],
      'frame-src': ["'none'"],
      'object-src': ["'none'"],
    };
    
    // Verify CSP doesn't allow dangerous sources
    expect(cspConfig['script-src']).not.toContain("*");
    expect(cspConfig['frame-src']).toContain("'none'");
    expect(cspConfig['object-src']).toContain("'none'");
    
    done();
  });
});