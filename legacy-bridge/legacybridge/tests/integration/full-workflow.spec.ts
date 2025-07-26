import { test, expect } from '@playwright/test';
import { join } from 'path';

test.describe('LegacyBridge Full Integration Workflow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should complete end-to-end RTF to Markdown workflow', async ({ page }) => {
    // Step 1: Upload RTF file
    const rtfContent = `{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0\\froman\\fcharset0 Times New Roman;}}
{\\colortbl ;\\red255\\green0\\blue0;\\red0\\green255\\blue0;\\red0\\green0\\blue255;}
\\f0\\fs24 
{\\b Bold Title}\\par
\\par
This is a test document with \\i italic text \\i0 and \\cf1 colored text\\cf0.\\par
\\par
{\\ul Underlined section:}\\par
\\bullet Item 1\\par
\\bullet Item 2\\par
\\bullet Item 3\\par
\\par
{\\field{\\*\\fldinst{HYPERLINK "https://example.com"}}{\\fldrslt{Click here}}}\\par
}`;

    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'integration-test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(rtfContent)
    });

    // Step 2: Verify file is loaded
    await expect(page.locator('[data-testid="file-item"]')).toBeVisible();
    await expect(page.locator('[data-testid="file-name"]')).toContainText('integration-test.rtf');
    await expect(page.locator('[data-testid="file-size"]')).toBeVisible();
    await expect(page.locator('[data-testid="file-type"]')).toContainText('RTF');

    // Step 3: Start conversion
    await expect(page.locator('[data-testid="convert-button"]')).toBeEnabled();
    await page.locator('[data-testid="convert-button"]').click();

    // Step 4: Monitor conversion progress
    await expect(page.locator('[data-testid="conversion-progress"]')).toBeVisible();
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Converting');
    
    // Progress should update
    const initialProgress = await page.locator('[data-testid="conversion-progress"]').getAttribute('aria-valuenow');
    expect(parseInt(initialProgress || '0')).toBeGreaterThanOrEqual(0);

    // Step 5: Wait for completion
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 30000 });
    
    const finalProgress = await page.locator('[data-testid="conversion-progress"]').getAttribute('aria-valuenow');
    expect(parseInt(finalProgress || '0')).toBe(100);

    // Step 6: Verify preview functionality
    await expect(page.locator('[data-testid="preview-button"]')).toBeEnabled();
    await page.locator('[data-testid="preview-button"]').click();

    await expect(page.locator('[data-testid="markdown-preview"]')).toBeVisible();
    
    // Check that RTF formatting was converted to Markdown
    const previewContent = await page.locator('[data-testid="markdown-preview"]').textContent();
    expect(previewContent).toContain('Bold Title');
    expect(previewContent).toContain('italic text');
    expect(previewContent).toContain('Item 1');
    expect(previewContent).toContain('Click here');

    // Step 7: Test download functionality
    const downloadPromise = page.waitForEvent('download');
    await page.locator('[data-testid="download-button"]').click();
    const download = await downloadPromise;
    
    expect(download.suggestedFilename()).toBe('integration-test.md');
    
    // Verify download content
    const downloadPath = await download.path();
    expect(downloadPath).toBeTruthy();

    // Step 8: Test file clearing
    await page.locator('[data-testid="clear-files-button"]').click();
    await expect(page.locator('[data-testid="file-item"]')).toHaveCount(0);
    await expect(page.locator('[data-testid="convert-button"]')).toBeDisabled();
  });

  test('should handle complex RTF document with tables and images', async ({ page }) => {
    // Complex RTF with table structure
    const complexRtfContent = `{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}}
{\\colortbl ;\\red255\\green0\\blue0;}
\\f0\\fs24 
{\\b Complex Document Test}\\par
\\par
This document contains various elements:\\par
\\par
{\\trowd\\cellx2000\\cellx4000\\cellx6000
\\intbl Cell 1\\cell Cell 2\\cell Cell 3\\cell\\row
\\intbl Data 1\\cell Data 2\\cell Data 3\\cell\\row}
\\par
{\\pict\\wmetafile8\\picw1000\\pich1000 
010009000003000000000000000000000000000000000000}\\par
\\par
End of document.\\par
}`;

    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'complex-document.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(complexRtfContent)
    });

    // Convert and verify
    await page.locator('[data-testid="convert-button"]').click();
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 30000 });

    // Check preview handles complex content
    await page.locator('[data-testid="preview-button"]').click();
    await expect(page.locator('[data-testid="markdown-preview"]')).toBeVisible();
    
    const previewContent = await page.locator('[data-testid="markdown-preview"]').textContent();
    expect(previewContent).toContain('Complex Document Test');
    expect(previewContent).toContain('Cell 1');
    expect(previewContent).toContain('Data 1');
  });

  test('should handle batch conversion workflow', async ({ page }) => {
    // Upload multiple files
    const files = [
      {
        name: 'batch-doc-1.rtf',
        mimeType: 'application/rtf',
        buffer: Buffer.from('{\\rtf1\\ansi Document 1 content with \\b bold text\\b0.}')
      },
      {
        name: 'batch-doc-2.rtf',
        mimeType: 'application/rtf',
        buffer: Buffer.from('{\\rtf1\\ansi Document 2 content with \\i italic text\\i0.}')
      },
      {
        name: 'batch-doc-3.md',
        mimeType: 'text/markdown',
        buffer: Buffer.from('# Markdown Document\n\nThis is **markdown** content.')
      }
    ];

    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(files);

    // Verify all files are loaded
    await expect(page.locator('[data-testid="file-item"]')).toHaveCount(3);
    
    // Check file types are detected correctly
    const fileTypes = await page.locator('[data-testid="file-type"]').allTextContents();
    expect(fileTypes).toContain('RTF');
    expect(fileTypes).toContain('Markdown');

    // Start batch conversion
    await expect(page.locator('[data-testid="convert-all-button"]')).toBeEnabled();
    await page.locator('[data-testid="convert-all-button"]').click();

    // Monitor batch progress
    await expect(page.locator('[data-testid="batch-progress"]')).toBeVisible();
    await expect(page.locator('[data-testid="batch-status"]')).toContainText('Converting');

    // Wait for all conversions to complete
    await expect(page.locator('[data-testid="batch-status"]')).toContainText('3 of 3 completed', { timeout: 60000 });

    // Verify individual file statuses
    const statusElements = await page.locator('[data-testid="conversion-status"]').all();
    for (const status of statusElements) {
      await expect(status).toContainText('Completed');
    }

    // Test batch download
    const downloadPromise = page.waitForEvent('download');
    await page.locator('[data-testid="download-all-button"]').click();
    const download = await downloadPromise;
    
    expect(download.suggestedFilename()).toMatch(/converted-files.*\.zip/);
  });

  test('should handle error recovery and retry workflow', async ({ page }) => {
    // Upload a file that might cause conversion issues
    const problematicContent = '{\\rtf1\\ansi\\deff0 Problematic content with \\unknown_command}';
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'problematic.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(problematicContent)
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();

    // Wait for result (could be success or error)
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText(/(Completed|Error)/, { timeout: 30000 });

    const status = await page.locator('[data-testid="conversion-status"]').textContent();
    
    if (status?.includes('Error')) {
      // Test error handling
      await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
      await expect(page.locator('[data-testid="retry-button"]')).toBeVisible();

      // Test retry functionality
      await page.locator('[data-testid="retry-button"]').click();
      await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Converting');
      
      // Wait for retry result
      await expect(page.locator('[data-testid="conversion-status"]')).toContainText(/(Completed|Error)/, { timeout: 30000 });
    }

    // Application should remain stable regardless of outcome
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toBeVisible();
  });

  test('should maintain state across page refresh', async ({ page }) => {
    // Upload file
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'state-test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from('{\\rtf1\\ansi State persistence test.}')
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 30000 });

    // Refresh page
    await page.reload();

    // Check if state is preserved (depending on implementation)
    // This test verifies the application handles refresh gracefully
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toBeVisible();
    await expect(page.locator('h1')).toBeVisible();
  });

  test('should handle concurrent operations correctly', async ({ page }) => {
    // Upload first file
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'concurrent-1.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from('{\\rtf1\\ansi Concurrent test 1.}')
    });

    // Start first conversion
    await page.locator('[data-testid="convert-button"]').click();

    // While first is converting, try to upload another file
    // (This tests the UI's handling of concurrent operations)
    await page.locator('[data-testid="add-more-files-button"]').click();
    
    const secondFileInput = page.locator('input[type="file"]').last();
    await secondFileInput.setInputFiles({
      name: 'concurrent-2.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from('{\\rtf1\\ansi Concurrent test 2.}')
    });

    // Wait for first conversion to complete
    await expect(page.locator('[data-testid="conversion-status"]').first()).toContainText('Completed', { timeout: 30000 });

    // Start second conversion
    await page.locator('[data-testid="convert-button"]').last().click();
    await expect(page.locator('[data-testid="conversion-status"]').last()).toContainText('Completed', { timeout: 30000 });

    // Both files should be successfully converted
    const completedStatuses = await page.locator('[data-testid="conversion-status"]').allTextContents();
    expect(completedStatuses.filter(status => status.includes('Completed'))).toHaveLength(2);
  });

  test('should handle different file encodings correctly', async ({ page }) => {
    // Test with UTF-8 content
    const utf8Content = '{\\rtf1\\ansi\\deff0 UTF-8 test: Héllo Wörld! 你好世界 🌍}';
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'utf8-test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(utf8Content, 'utf8')
    });

    // Convert and verify
    await page.locator('[data-testid="convert-button"]').click();
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 30000 });

    // Check preview preserves encoding
    await page.locator('[data-testid="preview-button"]').click();
    const previewContent = await page.locator('[data-testid="markdown-preview"]').textContent();
    expect(previewContent).toContain('Héllo Wörld!');
    expect(previewContent).toContain('你好世界');
    expect(previewContent).toContain('🌍');
  });

  test('should provide accurate file information and metadata', async ({ page }) => {
    // RTF with metadata
    const rtfWithMetadata = `{\\rtf1\\ansi\\deff0 
{\\info
{\\title Test Document Title}
{\\author John Doe}
{\\subject Test Subject}
{\\keywords test, rtf, conversion}
{\\creatim\\yr2024\\mo1\\dy15\\hr10\\min30}
}
{\\fonttbl {\\f0 Times New Roman;}}
\\f0\\fs24 Document content here.}`;

    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'metadata-test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(rtfWithMetadata)
    });

    // Verify file information is displayed
    await expect(page.locator('[data-testid="file-name"]')).toContainText('metadata-test.rtf');
    await expect(page.locator('[data-testid="file-size"]')).toBeVisible();
    await expect(page.locator('[data-testid="file-type"]')).toContainText('RTF');

    // Convert and check metadata preservation
    await page.locator('[data-testid="convert-button"]').click();
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 30000 });

    // View metadata if available
    if (await page.locator('[data-testid="metadata-button"]').isVisible()) {
      await page.locator('[data-testid="metadata-button"]').click();
      await expect(page.locator('[data-testid="metadata-info"]')).toBeVisible();
      
      const metadataText = await page.locator('[data-testid="metadata-info"]').textContent();
      expect(metadataText).toContain('Test Document Title');
      expect(metadataText).toContain('John Doe');
    }
  });

  test('should handle network connectivity issues gracefully', async ({ page }) => {
    // Upload file
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'network-test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from('{\\rtf1\\ansi Network connectivity test.}')
    });

    // Simulate network failure during conversion
    await page.route('**/api/**', route => route.abort());

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();

    // Should show network error
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Error', { timeout: 15000 });
    await expect(page.locator('[data-testid="error-message"]')).toContainText(/network|connection/i);

    // Restore network
    await page.unroute('**/api/**');

    // Retry should work
    await page.locator('[data-testid="retry-button"]').click();
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 30000 });
  });
});