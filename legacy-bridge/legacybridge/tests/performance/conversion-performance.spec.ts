import { test, expect } from '@playwright/test';

test.describe('LegacyBridge Performance Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should convert small files within performance threshold', async ({ page }) => {
    const startTime = Date.now();
    
    // Upload small RTF file (< 1KB)
    const smallRtfContent = '{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}} Small test document.}';
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'small-test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(smallRtfContent)
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();
    
    // Wait for completion
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 5000 });
    
    const endTime = Date.now();
    const conversionTime = endTime - startTime;
    
    // Small files should convert in under 2 seconds
    expect(conversionTime).toBeLessThan(2000);
    
    console.log(`Small file conversion time: ${conversionTime}ms`);
  });

  test('should convert medium files within performance threshold', async ({ page }) => {
    const startTime = Date.now();
    
    // Upload medium RTF file (~50KB)
    const mediumContent = '{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}} ' + 
      'Medium document content with formatting. '.repeat(2000) + '}';
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'medium-test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(mediumContent)
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();
    
    // Wait for completion
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 10000 });
    
    const endTime = Date.now();
    const conversionTime = endTime - startTime;
    
    // Medium files should convert in under 5 seconds
    expect(conversionTime).toBeLessThan(5000);
    
    console.log(`Medium file conversion time: ${conversionTime}ms`);
  });

  test('should convert large files within performance threshold', async ({ page }) => {
    const startTime = Date.now();
    
    // Upload large RTF file (~500KB)
    const largeContent = '{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}} ' + 
      'Large document content with extensive formatting and text. '.repeat(20000) + '}';
    
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'large-test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(largeContent)
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();
    
    // Wait for completion with extended timeout
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 30000 });
    
    const endTime = Date.now();
    const conversionTime = endTime - startTime;
    
    // Large files should convert in under 15 seconds
    expect(conversionTime).toBeLessThan(15000);
    
    console.log(`Large file conversion time: ${conversionTime}ms`);
  });

  test('should handle batch conversion efficiently', async ({ page }) => {
    const startTime = Date.now();
    
    // Upload 5 files for batch conversion
    const files = Array.from({ length: 5 }, (_, i) => ({
      name: `batch-test-${i + 1}.rtf`,
      mimeType: 'application/rtf',
      buffer: Buffer.from(`{\\rtf1\\ansi Batch document ${i + 1} content.}`)
    }));

    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(files);

    // Verify all files are loaded
    await expect(page.locator('[data-testid="file-item"]')).toHaveCount(5);

    // Start batch conversion
    await page.locator('[data-testid="convert-all-button"]').click();
    
    // Wait for all conversions to complete
    await expect(page.locator('[data-testid="batch-status"]')).toContainText('5 of 5 completed', { timeout: 30000 });
    
    const endTime = Date.now();
    const totalTime = endTime - startTime;
    const averageTimePerFile = totalTime / 5;
    
    // Batch conversion should be efficient (average < 3 seconds per file)
    expect(averageTimePerFile).toBeLessThan(3000);
    
    console.log(`Batch conversion total time: ${totalTime}ms`);
    console.log(`Average time per file: ${averageTimePerFile}ms`);
  });

  test('should maintain UI responsiveness during conversion', async ({ page }) => {
    // Upload file
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'responsiveness-test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from('{\\rtf1\\ansi\\deff0 Test content for responsiveness.}')
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();

    // Test UI responsiveness during conversion
    const uiTestStart = Date.now();
    
    // Click on different UI elements to test responsiveness
    await page.locator('[data-testid="settings-button"]').click({ timeout: 1000 });
    await page.locator('[data-testid="help-button"]').click({ timeout: 1000 });
    await page.locator('[data-testid="theme-toggle"]').click({ timeout: 1000 });
    
    const uiTestEnd = Date.now();
    const uiResponseTime = uiTestEnd - uiTestStart;
    
    // UI should remain responsive (< 500ms for interactions)
    expect(uiResponseTime).toBeLessThan(500);
    
    // Wait for conversion to complete
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 10000 });
  });

  test('should handle memory efficiently with multiple conversions', async ({ page }) => {
    // Perform multiple sequential conversions to test memory usage
    for (let i = 0; i < 3; i++) {
      const fileInput = page.locator('input[type="file"]');
      await fileInput.setInputFiles({
        name: `memory-test-${i + 1}.rtf`,
        mimeType: 'application/rtf',
        buffer: Buffer.from(`{\\rtf1\\ansi Memory test document ${i + 1}.}`)
      });

      await page.locator('[data-testid="convert-button"]').click();
      await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 10000 });
      
      // Clear the file to prepare for next iteration
      await page.locator('[data-testid="clear-files-button"]').click();
      await expect(page.locator('[data-testid="file-item"]')).toHaveCount(0);
    }

    // Check that the page is still responsive after multiple conversions
    const finalResponseTest = Date.now();
    await page.locator('[data-testid="drag-drop-zone"]').click();
    const finalResponseTime = Date.now() - finalResponseTest;
    
    expect(finalResponseTime).toBeLessThan(200);
  });

  test('should show accurate progress indicators', async ({ page }) => {
    // Upload file
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'progress-test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from('{\\rtf1\\ansi Progress indicator test content.}')
    });

    // Start conversion
    await page.locator('[data-testid="convert-button"]').click();

    // Check that progress indicator appears
    await expect(page.locator('[data-testid="conversion-progress"]')).toBeVisible();
    
    // Progress should start at 0 or low value
    const initialProgress = await page.locator('[data-testid="conversion-progress"]').getAttribute('aria-valuenow');
    expect(parseInt(initialProgress || '0')).toBeLessThanOrEqual(10);

    // Wait for completion and check final progress
    await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 10000 });
    
    const finalProgress = await page.locator('[data-testid="conversion-progress"]').getAttribute('aria-valuenow');
    expect(parseInt(finalProgress || '0')).toBe(100);
  });

  test('should handle concurrent conversions efficiently', async ({ page }) => {
    // Open multiple tabs/contexts to simulate concurrent users
    const context = page.context();
    const page2 = await context.newPage();
    await page2.goto('/');

    const startTime = Date.now();

    // Start conversion on both pages simultaneously
    const conversion1Promise = (async () => {
      const fileInput = page.locator('input[type="file"]');
      await fileInput.setInputFiles({
        name: 'concurrent-test-1.rtf',
        mimeType: 'application/rtf',
        buffer: Buffer.from('{\\rtf1\\ansi Concurrent test 1.}')
      });
      await page.locator('[data-testid="convert-button"]').click();
      await expect(page.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 15000 });
    })();

    const conversion2Promise = (async () => {
      const fileInput = page2.locator('input[type="file"]');
      await fileInput.setInputFiles({
        name: 'concurrent-test-2.rtf',
        mimeType: 'application/rtf',
        buffer: Buffer.from('{\\rtf1\\ansi Concurrent test 2.}')
      });
      await page2.locator('[data-testid="convert-button"]').click();
      await expect(page2.locator('[data-testid="conversion-status"]')).toContainText('Completed', { timeout: 15000 });
    })();

    // Wait for both conversions to complete
    await Promise.all([conversion1Promise, conversion2Promise]);

    const endTime = Date.now();
    const totalTime = endTime - startTime;

    // Concurrent conversions should not significantly impact performance
    expect(totalTime).toBeLessThan(20000);

    await page2.close();
  });
});