import { test, expect } from '@playwright/test';
import { join } from 'path';

test.describe('LegacyBridge Conversion Workflow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/LegacyBridge/);
  });

  test('should complete RTF to Markdown conversion workflow', async ({ page }) => {
    // Test file upload via drag and drop zone
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toBeVisible();
    
    // Create a test RTF file content
    const testRtfContent = '{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}} Hello World from RTF!}';
    
    // Upload file using file input
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'test-document.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(testRtfContent)
    });

    // Wait for file to be processed
    await expect(page.locator('[data-testid="file-item"]')).toBeVisible();
    await expect(page.locator('[data-testid="file-name"]')).toContainText('test-document.rtf');

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();

    // Wait for conversion to complete
    await expect(page.locator('[data-testid="conversion-progress"]')).toBeVisible();
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Converting', { timeout: 10000 });
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 30000 });

    // Verify preview is available
    await expect(page.locator('[data-testid="preview-button"]')).toBeEnabled();
    await page.locator('[data-testid="preview-button"]').click();

    // Check preview content
    await expect(page.locator('[data-testid="markdown-preview"]')).toBeVisible();
    await expect(page.locator('[data-testid="markdown-preview"]')).toContainText('Hello World from RTF!');

    // Test download functionality
    const downloadPromise = page.waitForEvent('download');
    await page.locator('[data-testid="download-button"]').click();
    const download = await downloadPromise;
    
    expect(download.suggestedFilename()).toBe('test-document.md');
  });

  test('should handle multiple file conversion', async ({ page }) => {
    // Upload multiple files
    const files = [
      {
        name: 'doc1.rtf',
        mimeType: 'application/rtf',
        buffer: Buffer.from('{\\rtf1\\ansi Document 1 content}')
      },
      {
        name: 'doc2.rtf', 
        mimeType: 'application/rtf',
        buffer: Buffer.from('{\\rtf1\\ansi Document 2 content}')
      }
    ];

    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(files);

    // Verify both files are listed
    await expect(page.locator('[data-testid="file-item"]')).toHaveCount(2);

    // Start batch conversion
    await page.locator('[data-testid="convert-all-button"]').click();

    // Wait for all conversions to complete
    await expect(page.locator('[data-testid="batch-progress"]')).toBeVisible();
    await expect(page.locator('[data-testid="batch-status"]')).toContainText('2 of 2 completed', { timeout: 60000 });

    // Verify download all functionality
    const downloadPromise = page.waitForEvent('download');
    await page.locator('[data-testid="download-all-button"]').click();
    const download = await downloadPromise;
    
    expect(download.suggestedFilename()).toMatch(/converted-files.*\.zip/);
  });

  test('should handle conversion errors gracefully', async ({ page }) => {
    // Upload invalid file
    const invalidFile = {
      name: 'invalid.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from('This is not valid RTF content')
    };

    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles([invalidFile]);

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();

    // Wait for error state
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Error', { timeout: 30000 });
    await expect(page.locator('[data-testid="error-message"]')).toBeVisible();

    // Verify retry functionality
    await expect(page.locator('[data-testid="retry-button"]')).toBeVisible();
    await page.locator('[data-testid="retry-button"]').click();

    // Should attempt conversion again
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Converting');
  });

  test('should support Markdown to RTF conversion', async ({ page }) => {
    // Upload Markdown file
    const markdownContent = '# Hello World\n\nThis is a **bold** text with *italic* formatting.\n\n- List item 1\n- List item 2';
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'test-document.md',
      mimeType: 'text/markdown',
      buffer: Buffer.from(markdownContent)
    });

    // Verify file is recognized as Markdown
    await expect(page.locator('[data-testid="file-type"]')).toContainText('Markdown');

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();

    // Wait for conversion to complete
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 30000 });

    // Download RTF file
    const downloadPromise = page.waitForEvent('download');
    await page.locator('[data-testid="download-button"]').click();
    const download = await downloadPromise;
    
    expect(download.suggestedFilename()).toBe('test-document.rtf');
  });

  test('should handle large file conversion', async ({ page }) => {
    // Create a large RTF content (simulate large document)
    const largeContent = '{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}} ' + 
      'Large document content. '.repeat(10000) + '}';
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'large-document.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(largeContent)
    });

    // Verify file size is displayed
    await expect(page.locator('[data-testid="file-size"]')).toBeVisible();

    // Start conversion with extended timeout
    await page.locator('[data-testid="convert-button"]').click();

    // Wait for conversion with longer timeout for large files
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 120000 });

    // Verify progress was shown during conversion
    await expect(page.locator('[data-testid="conversion-progress"]')).toHaveAttribute('aria-valuenow', '100');
  });

  test('should maintain accessibility standards', async ({ page }) => {
    // Check for proper ARIA labels
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toHaveAttribute('role', 'button');
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toHaveAttribute('aria-label');

    // Test keyboard navigation
    await page.keyboard.press('Tab');
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toBeFocused();

    // Test keyboard file selection
    await page.keyboard.press('Enter');
    // File dialog should open (can't test file dialog directly in Playwright)

    // Upload file for further accessibility testing
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from('{\\rtf1\\ansi Test content}')
    });

    // Test keyboard navigation through conversion workflow
    await page.keyboard.press('Tab');
    await expect(page.locator('[data-testid="convert-button"]')).toBeFocused();

    await page.keyboard.press('Enter');
    
    // Wait for conversion
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 30000 });

    // Test keyboard access to results
    await page.keyboard.press('Tab');
    await expect(page.locator('[data-testid="preview-button"]')).toBeFocused();

    await page.keyboard.press('Tab');
    await expect(page.locator('[data-testid="download-button"]')).toBeFocused();
  });

  test('should handle network interruption gracefully', async ({ page }) => {
    // Upload file
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from('{\\rtf1\\ansi Test content}')
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();

    // Simulate network interruption
    await page.route('**/api/convert', route => route.abort());

    // Should show error state
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Error', { timeout: 30000 });
    await expect(page.locator('[data-testid="error-message"]')).toContainText('network');

    // Restore network and retry
    await page.unroute('**/api/convert');
    await page.locator('[data-testid="retry-button"]').click();

    // Should complete successfully
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 30000 });
  });

  test('should preserve file metadata during conversion', async ({ page }) => {
    // Upload file with specific metadata
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'document-with-metadata.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from('{\\rtf1\\ansi\\deff0 {\\info{\\title Document Title}{\\author John Doe}} Content}')
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 30000 });

    // Check if metadata is preserved in preview
    await page.locator('[data-testid="preview-button"]').click();
    await expect(page.locator('[data-testid="metadata-info"]')).toBeVisible();
    await expect(page.locator('[data-testid="metadata-info"]')).toContainText('Document Title');
    await expect(page.locator('[data-testid="metadata-info"]')).toContainText('John Doe');
  });
}); 