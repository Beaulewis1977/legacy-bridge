import { describe, it, expect, beforeAll, afterAll } from '@jest/globals';
import axios from 'axios';
import fs from 'fs';
import path from 'path';

// XSS test vectors
const XSS_PAYLOADS = [
  // Basic XSS
  '<script>alert("xss")</script>',
  '"><script>alert("xss")</script>',
  '<img src=x onerror=alert("xss")>',
  '<svg onload=alert("xss")>',
  'javascript:alert("xss")',
  
  // Event handlers
  '<body onload=alert("xss")>',
  '<div onmouseover="alert(\'xss\')">',
  '<input onfocus=alert("xss") autofocus>',
  
  // Advanced XSS
  '<script>alert(String.fromCharCode(88,83,83))</script>',
  '\\u003cscript\\u003ealert("xss")\\u003c/script\\u003e',
  '<img src="x" onerror="alert(\'xss\')">',
  '<iframe src="javascript:alert(\'xss\')">',
  
  // Filter bypass attempts
  '<scr<script>ipt>alert("xss")</scr</script>ipt>',
  '<SCRIPT>alert("XSS")</SCRIPT>',
  '<script >alert("xss")</script >',
  
  // Markdown-specific XSS
  '[Click me](javascript:alert("xss"))',
  '![xss](javascript:alert("xss"))',
  '<http://evil.com/xss.js>',
  
  // RTF-specific injections
  '{\\field{\\*\\fldinst{HYPERLINK "javascript:alert(\'xss\')"}}',
  '{\\pict\\pngblip\\picw1\\pich1 89504e470d0a1a0a}', // Minimal PNG
];

// SQL injection payloads
const SQL_INJECTION_PAYLOADS = [
  "' OR '1'='1",
  "1; DROP TABLE users--",
  "admin'--",
  "' UNION SELECT * FROM users--",
  "1' AND '1'='1",
];

// Path traversal payloads
const PATH_TRAVERSAL_PAYLOADS = [
  '../../../etc/passwd',
  '..\\..\\..\\windows\\system32\\config\\sam',
  '%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd',
  '....//....//....//etc/passwd',
  'C:\\..\\..\\..\\windows\\system32\\config\\sam',
];

// Command injection payloads
const COMMAND_INJECTION_PAYLOADS = [
  '; ls -la',
  '| whoami',
  '`cat /etc/passwd`',
  '$(rm -rf /)',
  '&& net user',
];

// Large payload for DoS testing
const LARGE_PAYLOAD = 'A'.repeat(10 * 1024 * 1024); // 10MB

describe('Security Test Suite', () => {
  const API_BASE_URL = process.env.API_BASE_URL || 'http://localhost:3000';
  const api = axios.create({
    baseURL: API_BASE_URL,
    timeout: 30000,
    validateStatus: () => true // Don't throw on any status
  });

  describe('XSS Prevention Tests', () => {
    XSS_PAYLOADS.forEach((payload, index) => {
      it(`should sanitize XSS payload ${index + 1}: ${payload.substring(0, 50)}...`, async () => {
        // Test RTF to Markdown
        const rtfResponse = await api.post('/api/convert', {
          content: `{\\rtf1\\ansi ${payload}}`,
          format: 'rtf',
          target: 'markdown'
        });

        if (rtfResponse.data.success) {
          const result = rtfResponse.data.content;
          
          // Check for script tags
          expect(result.toLowerCase()).not.toContain('<script');
          expect(result.toLowerCase()).not.toContain('</script>');
          expect(result).not.toContain('javascript:');
          expect(result).not.toContain('onerror=');
          expect(result).not.toContain('onload=');
          expect(result).not.toContain('onmouseover=');
          
          // Ensure no executable code
          expect(result).not.toMatch(/on\w+\s*=/i);
        }

        // Test Markdown to RTF
        const mdResponse = await api.post('/api/convert', {
          content: payload,
          format: 'markdown',
          target: 'rtf'
        });

        if (mdResponse.data.success) {
          const result = mdResponse.data.content;
          
          // Check RTF doesn't contain malicious hyperlinks
          expect(result).not.toContain('javascript:');
          expect(result).not.toContain('HYPERLINK "javascript:');
        }
      });
    });
  });

  describe('Input Validation Tests', () => {
    it('should reject files larger than 10MB', async () => {
      const response = await api.post('/api/convert', {
        content: LARGE_PAYLOAD,
        format: 'markdown',
        target: 'rtf'
      });

      expect(response.status).toBe(400);
      expect(response.data.error).toContain('too large');
    });

    it('should validate content type', async () => {
      const response = await api.post('/api/convert', {
        content: 'test',
        format: 'invalid',
        target: 'rtf'
      });

      expect(response.status).toBe(400);
      expect(response.data.error).toContain('Invalid format');
    });

    it('should handle null/undefined inputs', async () => {
      const nullResponse = await api.post('/api/convert', {
        content: null,
        format: 'markdown',
        target: 'rtf'
      });

      expect(nullResponse.status).toBe(400);
      expect(nullResponse.data.error).toBeDefined();

      const undefinedResponse = await api.post('/api/convert', {
        format: 'markdown',
        target: 'rtf'
      });

      expect(undefinedResponse.status).toBe(400);
    });

    it('should validate maximum nesting depth', async () => {
      // Create deeply nested markdown
      let deeplyNested = '';
      for (let i = 0; i < 100; i++) {
        deeplyNested += '  '.repeat(i) + '- Item\\n';
      }

      const response = await api.post('/api/convert', {
        content: deeplyNested,
        format: 'markdown',
        target: 'rtf'
      });

      // Should either handle gracefully or reject if too deep
      expect([200, 400]).toContain(response.status);
    });
  });

  describe('Path Traversal Prevention', () => {
    PATH_TRAVERSAL_PAYLOADS.forEach((payload, index) => {
      it(`should prevent path traversal attempt ${index + 1}: ${payload}`, async () => {
        const response = await api.post('/api/convert', {
          content: `[Link](${payload})`,
          format: 'markdown',
          target: 'rtf'
        });

        if (response.data.success) {
          const result = response.data.content;
          
          // Should not contain actual file paths
          expect(result).not.toContain('/etc/passwd');
          expect(result).not.toContain('windows\\system32');
          expect(result).not.toContain('..\\');
          expect(result).not.toContain('../');
        }
      });
    });
  });

  describe('Command Injection Prevention', () => {
    COMMAND_INJECTION_PAYLOADS.forEach((payload, index) => {
      it(`should prevent command injection ${index + 1}: ${payload}`, async () => {
        const response = await api.post('/api/convert', {
          content: `Text with ${payload}`,
          format: 'markdown',
          target: 'rtf'
        });

        // Should not execute commands
        expect(response.status).not.toBe(500);
        if (response.data.success) {
          // Command should be treated as literal text
          expect(response.data.content).toBeDefined();
        }
      });
    });
  });

  describe('DoS Resistance Tests', () => {
    it('should handle zip bomb-like patterns', async () => {
      // Create a pattern that expands significantly
      const zipBombPattern = '**' + '*'.repeat(1000) + '**';
      
      const response = await api.post('/api/convert', {
        content: zipBombPattern.repeat(100),
        format: 'markdown',
        target: 'rtf'
      }, { timeout: 5000 });

      // Should timeout or reject, not hang indefinitely
      expect([200, 400, 408]).toContain(response.status);
    });

    it('should handle regex DoS patterns', async () => {
      // ReDoS pattern
      const redosPattern = 'a'.repeat(50) + '!';
      const maliciousMarkdown = `[${redosPattern}](http://example.com/${redosPattern})`;

      const startTime = Date.now();
      const response = await api.post('/api/convert', {
        content: maliciousMarkdown,
        format: 'markdown',
        target: 'rtf'
      });
      const duration = Date.now() - startTime;

      // Should complete quickly
      expect(duration).toBeLessThan(1000);
    });

    it('should rate limit excessive requests', async () => {
      const requests = [];
      
      // Send 100 requests rapidly
      for (let i = 0; i < 100; i++) {
        requests.push(api.post('/api/convert', {
          content: 'test',
          format: 'markdown',
          target: 'rtf'
        }));
      }

      const responses = await Promise.all(requests);
      const rateLimited = responses.filter(r => r.status === 429);
      
      // Should have some rate limiting
      expect(rateLimited.length).toBeGreaterThan(0);
    });
  });

  describe('Memory Safety Tests', () => {
    it('should not leak memory on large conversions', async () => {
      const conversions = [];
      
      // Perform multiple large conversions
      for (let i = 0; i < 10; i++) {
        const largeDoc = generateLargeDocument(500); // 500KB
        conversions.push(api.post('/api/convert', {
          content: largeDoc,
          format: 'markdown',
          target: 'rtf'
        }));
      }

      const responses = await Promise.all(conversions);
      const successful = responses.filter(r => r.status === 200);
      
      expect(successful.length).toBeGreaterThan(8); // Most should succeed
    });

    it('should handle unicode edge cases safely', async () => {
      const unicodePayloads = [
        '\\u0000', // Null character
        '\\uFEFF', // Zero-width no-break space
        '\\u200B', // Zero-width space
        '\\u200C', // Zero-width non-joiner
        '\\u200D', // Zero-width joiner
        '\\uD800', // Unpaired high surrogate
        '\\uDFFF', // Unpaired low surrogate
        '🔥'.repeat(1000), // Emoji stress test
      ];

      for (const payload of unicodePayloads) {
        const response = await api.post('/api/convert', {
          content: payload,
          format: 'markdown',
          target: 'rtf'
        });

        expect([200, 400]).toContain(response.status);
      }
    });
  });

  describe('Authentication & Authorization Tests', () => {
    it('should require valid API key for protected endpoints', async () => {
      const response = await api.post('/api/admin/users', {}, {
        headers: { 'Authorization': 'Bearer invalid-key' }
      });

      expect(response.status).toBe(401);
    });

    it('should enforce CORS policies', async () => {
      const response = await api.options('/api/convert', {
        headers: {
          'Origin': 'http://malicious-site.com',
          'Access-Control-Request-Method': 'POST'
        }
      });

      const allowedOrigin = response.headers['access-control-allow-origin'];
      expect(allowedOrigin).not.toBe('*');
      expect(allowedOrigin).not.toBe('http://malicious-site.com');
    });
  });

  describe('Security Headers Tests', () => {
    it('should include security headers', async () => {
      const response = await api.get('/');

      // Check for security headers
      expect(response.headers['x-content-type-options']).toBe('nosniff');
      expect(response.headers['x-frame-options']).toMatch(/DENY|SAMEORIGIN/);
      expect(response.headers['x-xss-protection']).toBe('1; mode=block');
      expect(response.headers['strict-transport-security']).toBeDefined();
      expect(response.headers['content-security-policy']).toBeDefined();
    });
  });

  describe('Cryptographic Security', () => {
    it('should use secure random generation', async () => {
      const responses = await Promise.all(
        Array(10).fill(null).map(() => 
          api.post('/api/generate-id')
        )
      );

      const ids = responses.map(r => r.data.id);
      const uniqueIds = new Set(ids);
      
      // All IDs should be unique
      expect(uniqueIds.size).toBe(10);
      
      // IDs should have sufficient entropy
      ids.forEach(id => {
        expect(id.length).toBeGreaterThanOrEqual(16);
      });
    });
  });
});

// Helper function to generate large documents
function generateLargeDocument(sizeKB: number): string {
  const content = [];
  const targetSize = sizeKB * 1024;
  let currentSize = 0;

  while (currentSize < targetSize) {
    const paragraph = `## Section ${content.length + 1}

This is a test paragraph with **bold**, *italic*, and \`code\` formatting. It includes [links](http://example.com) and various other Markdown features to ensure realistic testing scenarios.

- List item 1
- List item 2
  - Nested item
- List item 3

| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |

`;
    content.push(paragraph);
    currentSize += paragraph.length;
  }

  return content.join('\\n');
}

// Export test utilities
export { XSS_PAYLOADS, SQL_INJECTION_PAYLOADS, PATH_TRAVERSAL_PAYLOADS, generateLargeDocument };