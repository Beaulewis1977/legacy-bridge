import DOMPurify from 'dompurify';

/**
 * Security utility for HTML sanitization to prevent XSS attacks
 */

// Configuration for different sanitization contexts
const MARKDOWN_CONFIG = {
  ALLOWED_TAGS: [
    'p', 'br', 'span', 'div', 
    'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
    'strong', 'em', 'b', 'i', 'u',
    'a', 'img',
    'ul', 'ol', 'li',
    'blockquote', 'pre', 'code',
    'table', 'thead', 'tbody', 'tr', 'th', 'td',
    'hr'
  ],
  ALLOWED_ATTR: ['href', 'src', 'alt', 'title', 'class', 'id'],
  ALLOW_DATA_ATTR: false,
  ALLOW_UNKNOWN_PROTOCOLS: false,
  ALLOWED_URI_REGEXP: /^(?:(?:https?|mailto):\/\/|#)/i
};

const SYNTAX_HIGHLIGHT_CONFIG = {
  ALLOWED_TAGS: ['span', 'div', 'pre', 'code'],
  ALLOWED_ATTR: ['class'],
  ALLOW_DATA_ATTR: false,
  KEEP_CONTENT: true
};

/**
 * Sanitize HTML content for markdown preview
 * @param dirty - The potentially unsafe HTML string
 * @returns Sanitized HTML string safe for rendering
 */
export function sanitizeMarkdown(dirty: string): string {
  // Server-side rendering check
  if (typeof window === 'undefined') {
    // For SSR, return escaped HTML
    return escapeHtml(dirty);
  }
  
  return DOMPurify.sanitize(dirty, MARKDOWN_CONFIG);
}

/**
 * Sanitize HTML content for syntax highlighting
 * @param dirty - The potentially unsafe HTML string
 * @returns Sanitized HTML string safe for rendering
 */
export function sanitizeSyntaxHighlight(dirty: string): string {
  // Server-side rendering check
  if (typeof window === 'undefined') {
    // For SSR, return escaped HTML
    return escapeHtml(dirty);
  }
  
  return DOMPurify.sanitize(dirty, SYNTAX_HIGHLIGHT_CONFIG);
}

/**
 * Escape HTML characters to prevent XSS (fallback for SSR)
 * @param text - The text to escape
 * @returns Escaped text safe for HTML rendering
 */
export function escapeHtml(text: string): string {
  const map: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#039;',
    '/': '&#x2F;'
  };
  return text.replace(/[&<>"'/]/g, (m) => map[m]);
}

/**
 * Sanitize user input for display (no HTML allowed)
 * @param input - User input string
 * @returns Sanitized string with all HTML removed
 */
export function sanitizeUserInput(input: string): string {
  if (typeof window === 'undefined') {
    return escapeHtml(input);
  }
  
  return DOMPurify.sanitize(input, {
    ALLOWED_TAGS: [],
    ALLOWED_ATTR: [],
    KEEP_CONTENT: true
  });
}

/**
 * Check if a string contains potentially dangerous content
 * @param content - Content to check
 * @returns true if content appears safe, false if suspicious
 */
export function isContentSafe(content: string): boolean {
  // Check for common XSS patterns
  const dangerousPatterns = [
    /<script[\s>]/i,
    /javascript:/i,
    /on\w+\s*=/i, // Event handlers like onclick=
    /vbscript:/i,
    /data:text\/html/i,
    /<iframe/i,
    /<object/i,
    /<embed/i,
    /<link/i,
    /<meta/i,
    /<form/i
  ];
  
  return !dangerousPatterns.some(pattern => pattern.test(content));
}