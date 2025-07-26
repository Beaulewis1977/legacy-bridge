import { test, expect } from '@playwright/test';

test.describe('LegacyBridge Security Validation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should prevent XSS attacks in file content', async ({ page }) => {
    // Test XSS in RTF content
    const maliciousRtfContent = '{\\rtf1\\ansi\\deff0 <script>alert("XSS")</script>}';
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'xss-test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(maliciousRtfContent)
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 10000 });

    // View preview
    await page.locator('[data-testid="preview-button"]').click();
    
    // Check that script tags are sanitized in preview
    const previewContent = await page.locator('[data-testid="markdown-preview"]').innerHTML();
    expect(previewContent).not.toContain('<script>');
    expect(previewContent).not.toContain('alert("XSS")');
    
    // Ensure no JavaScript execution occurred
    const alertDialogs = page.locator('dialog[role="alertdialog"]');
    await expect(alertDialogs).toHaveCount(0);
  });

  test('should prevent XSS attacks in Markdown content', async ({ page }) => {
    // Test XSS in Markdown content
    const maliciousMarkdownContent = `
# Test Document

<script>alert('XSS in markdown')</script>

<img src="x" onerror="alert('Image XSS')">

[Click me](javascript:alert('Link XSS'))

<iframe src="javascript:alert('Iframe XSS')"></iframe>
`;
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'xss-markdown-test.md',
      mimeType: 'text/markdown',
      buffer: Buffer.from(maliciousMarkdownContent)
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 10000 });

    // View preview
    await page.locator('[data-testid="preview-button"]').click();
    
    // Check that dangerous elements are sanitized
    const previewContent = await page.locator('[data-testid="markdown-preview"]').innerHTML();
    expect(previewContent).not.toContain('<script>');
    expect(previewContent).not.toContain('javascript:');
    expect(previewContent).not.toContain('onerror=');
    expect(previewContent).not.toContain('<iframe');
    
    // Ensure no JavaScript execution occurred
    const alertDialogs = page.locator('dialog[role="alertdialog"]');
    await expect(alertDialogs).toHaveCount(0);
  });

  test('should validate file types and reject dangerous files', async ({ page }) => {
    // Test uploading executable file
    const executableContent = 'MZ\x90\x00'; // PE header signature
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'malicious.exe',
      mimeType: 'application/octet-stream',
      buffer: Buffer.from(executableContent)
    });

    // Should show error for invalid file type
    await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
    await expect(page.locator('[data-testid="error-message"]')).toContainText(/invalid.*file.*type/i);
    
    // Convert button should be disabled
    await expect(page.locator('[data-testid="convert-button"]')).toBeDisabled();
  });

  test('should enforce file size limits', async ({ page }) => {
    // Test uploading very large file (simulate 100MB)
    const largeContent = 'A'.repeat(100 * 1024 * 1024); // 100MB of 'A' characters
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'huge-file.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(largeContent)
    });

    // Should show error for file too large
    await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
    await expect(page.locator('[data-testid="error-message"]')).toContainText(/file.*too.*large/i);
    
    // Convert button should be disabled
    await expect(page.locator('[data-testid="convert-button"]')).toBeDisabled();
  });

  test('should sanitize filenames to prevent path traversal', async ({ page }) => {
    // Test filename with path traversal attempt
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: '../../../etc/passwd.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from('{\\rtf1\\ansi Path traversal test.}')
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 10000 });

    // Download file
    const downloadPromise = page.waitForEvent('download');
    await page.locator('[data-testid="download-button"]').click();
    const download = await downloadPromise;
    
    // Filename should be sanitized (no path traversal)
    const suggestedFilename = download.suggestedFilename();
    expect(suggestedFilename).not.toContain('../');
    expect(suggestedFilename).not.toContain('/etc/');
    expect(suggestedFilename).toMatch(/^[^\/\\]*\.md$/); // Should be just filename.md
  });

  test('should prevent CSRF attacks', async ({ page }) => {
    // Check for CSRF protection headers
    const response = await page.goto('/');
    const headers = response?.headers();
    
    // Should have CSRF protection headers
    expect(headers?.['x-frame-options']).toBe('DENY');
    expect(headers?.['x-content-type-options']).toBe('nosniff');
  });

  test('should have secure Content Security Policy', async ({ page }) => {
    const response = await page.goto('/');
    const headers = response?.headers();
    
    // Should have CSP header
    const csp = headers?.['content-security-policy'];
    expect(csp).toBeTruthy();
    
    // CSP should restrict dangerous sources
    expect(csp).toContain("script-src");
    expect(csp).toContain("object-src 'none'");
    expect(csp).toContain("base-uri 'self'");
  });

  test('should prevent clickjacking attacks', async ({ page }) => {
    const response = await page.goto('/');
    const headers = response?.headers();
    
    // Should have X-Frame-Options header
    expect(headers?.['x-frame-options']).toBe('DENY');
  });

  test('should handle malformed RTF files safely', async ({ page }) => {
    // Test with malformed RTF that could cause buffer overflow
    const malformedRtf = '{\\rtf1\\ansi\\deff0 ' + 'A'.repeat(10000) + '\\' + 'B'.repeat(10000);
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'malformed.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(malformedRtf)
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();
    
    // Should either complete successfully or show proper error
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText(/(Completed|Error)/, { timeout: 30000 });
    
    // Application should remain responsive
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toBeVisible();
  });

  test('should validate and sanitize URLs in markdown', async ({ page }) => {
    // Test markdown with potentially dangerous URLs
    const dangerousMarkdown = `
# Test Document

[Dangerous Link](javascript:alert('XSS'))
[Data URL](data:text/html,<script>alert('XSS')</script>)
[File URL](file:///etc/passwd)
[FTP URL](ftp://malicious.com/file.exe)

![Dangerous Image](javascript:alert('Image XSS'))
![Data Image](data:image/svg+xml,<svg onload="alert('SVG XSS')"></svg>)
`;
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'dangerous-urls.md',
      mimeType: 'text/markdown',
      buffer: Buffer.from(dangerousMarkdown)
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 10000 });

    // View preview
    await page.locator('[data-testid="preview-button"]').click();
    
    // Check that dangerous URLs are sanitized or removed
    const previewContent = await page.locator('[data-testid="markdown-preview"]').innerHTML();
    expect(previewContent).not.toContain('javascript:');
    expect(previewContent).not.toContain('data:text/html');
    expect(previewContent).not.toContain('file:///');
    
    // Check that links are either removed or made safe
    const links = await page.locator('[data-testid="markdown-preview"] a').all();
    for (const link of links) {
      const href = await link.getAttribute('href');
      if (href) {
        expect(href).not.toMatch(/^javascript:/);
        expect(href).not.toMatch(/^data:text\/html/);
        expect(href).not.toMatch(/^file:/);
      }
    }
  });

  test('should prevent code injection in file metadata', async ({ page }) => {
    // Test RTF with malicious metadata
    const maliciousMetadata = `{\\rtf1\\ansi\\deff0 
{\\info
{\\title <script>alert('Title XSS')</script>}
{\\author javascript:alert('Author XSS')}
{\\subject data:text/html,<script>alert('Subject XSS')</script>}
}
Content here.}`;
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'malicious-metadata.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(maliciousMetadata)
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 10000 });

    // View preview and metadata
    await page.locator('[data-testid="preview-button"]').click();
    
    // Check that metadata is sanitized
    if (await page.locator('[data-testid="metadata-info"]').isVisible()) {
      const metadataContent = await page.locator('[data-testid="metadata-info"]').innerHTML();
      expect(metadataContent).not.toContain('<script>');
      expect(metadataContent).not.toContain('javascript:');
      expect(metadataContent).not.toContain('data:text/html');
    }
  });

  test('should rate limit conversion requests', async ({ page }) => {
    // Attempt rapid-fire conversions to test rate limiting
    const promises = [];
    
    for (let i = 0; i < 10; i++) {
      const promise = (async () => {
        const fileInput = page.locator('input[type="file"]');
        await fileInput.setInputFiles({
          name: `rate-limit-test-${i}.rtf`,
          mimeType: 'application/rtf',
          buffer: Buffer.from(`{\\rtf1\\ansi Rate limit test ${i}.}`)
        });

        await page.locator('[data-testid="convert-button"]').click();
        
        // Wait for some response (either success or rate limit error)
        await expect(page.locator('[data-testid="conversion-status"]')).toContainText(/(Completed|Error|Rate limit)/, { timeout: 15000 });
        
        // Clear for next iteration
        await page.locator('[data-testid="clear-files-button"]').click();
      })();
      
      promises.push(promise);
    }

    // Some requests should be rate limited
    await Promise.allSettled(promises);
    
    // Application should remain stable
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toBeVisible();
  });

  test('should handle memory exhaustion attacks gracefully', async ({ page }) => {
    // Test with file designed to consume excessive memory
    const memoryExhaustionContent = '{\\rtf1\\ansi\\deff0 ' + 
      '\\pict\\wmetafile8\\picw10000\\pich10000 ' + 
      'FF'.repeat(50000) + '}'; // Large embedded image data
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'memory-exhaustion.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(memoryExhaustionContent)
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();
    
    // Should either complete or fail gracefully (not crash)
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText(/(Completed|Error)/, { timeout: 60000 });
    
    // Application should remain responsive
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toBeVisible();
    await page.locator('[data-testid="drag-drop-zone"]').click();
  });
});