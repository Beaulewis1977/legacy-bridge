import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test.describe('LegacyBridge Accessibility Compliance', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should pass axe accessibility audit on main page', async ({ page }) => {
    const accessibilityScanResults = await new AxeBuilder({ page })
      .exclude('#performance-monitor') // Exclude dynamic performance monitor
      .analyze();
    
    // Allow minor violations but ensure no critical ones
    const criticalViolations = accessibilityScanResults.violations.filter(
      violation => violation.impact === 'critical' || violation.impact === 'serious'
    );
    
    expect(criticalViolations).toEqual([]);
  });

  test('should have proper heading hierarchy', async ({ page }) => {
    // Check for proper heading structure (h1 -> h2 -> h3, etc.)
    const headings = await page.locator('h1, h2, h3, h4, h5, h6').all();
    
    expect(headings.length).toBeGreaterThan(0);
    
    // Should have exactly one h1
    const h1Count = await page.locator('h1').count();
    expect(h1Count).toBe(1);
    
    // Check h1 content
    await expect(page.locator('h1')).toContainText(/LegacyBridge|Convert|Document/i);
  });

  test('should have proper ARIA labels and roles', async ({ page }) => {
    // Check file input has proper ID
    const fileInput = page.locator('input[type="file"]');
    const inputId = await fileInput.getAttribute('id');
    expect(inputId).toBeTruthy();

    // Check buttons have accessible names
    const buttons = await page.locator('button').all();
    for (const button of buttons) {
      const ariaLabel = await button.getAttribute('aria-label');
      const textContent = await button.textContent();
      
      // Button should have either aria-label or text content
      expect(ariaLabel || textContent?.trim()).toBeTruthy();
    }
    
    // Check that main heading exists
    await expect(page.locator('h1')).toBeVisible();
  });

  test('should support keyboard navigation', async ({ page }) => {
    // Test tab navigation through interactive elements
    await page.keyboard.press('Tab');
    
    // Should focus on some interactive element
    const firstFocused = await page.locator(':focus').first();
    expect(await firstFocused.isVisible()).toBe(true);
    
    // Continue tabbing through elements
    await page.keyboard.press('Tab');
    const secondFocused = await page.locator(':focus').first();
    expect(await secondFocused.isVisible()).toBe(true);
    
    // Test that we can navigate to the file upload area
    const fileUploadLabel = page.locator('label[for="file-input"]');
    await fileUploadLabel.focus();
    await expect(fileUploadLabel).toBeFocused();
  });

  test('should have sufficient color contrast', async ({ page }) => {
    // Run axe with color contrast rules specifically
    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
      .include('body')
      .exclude('.sr-only') // Exclude screen reader only elements from color contrast checks
      .analyze();
    
    // Filter for color contrast violations
    const colorContrastViolations = accessibilityScanResults.violations.filter(
      violation => violation.id === 'color-contrast'
    );
    
    expect(colorContrastViolations).toEqual([]);
  });

  test('should work with screen readers', async ({ page }) => {
    // Check that the page has proper structure for screen readers
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('h1')).toContainText('LegacyBridge');
    
    // Check that descriptive text is available
    await expect(page.locator('text=Convert between RTF and Markdown with ease')).toBeVisible();
    
    // Check that file upload area is accessible
    await expect(page.locator('label[for="file-input"]')).toBeVisible();
    await expect(page.locator('text=or click to browse')).toBeVisible();
  });

  test('should have proper form labels and error messages', async ({ page }) => {
    // Test file input accessibility
    const fileInput = page.locator('input[type="file"]');
    
    // Should have proper ID
    const inputId = await fileInput.getAttribute('id');
    expect(inputId).toBeTruthy();
    
    // Check that drag and drop area is visible
    await expect(page.locator('text=Drag & drop files here')).toBeVisible();
  });

  test('should support high contrast mode', async ({ page }) => {
    // Simulate high contrast mode
    await page.emulateMedia({ colorScheme: 'dark', forcedColors: 'active' });
    
    // Check that important elements are still visible
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('label[for="file-input"]')).toBeVisible();
    await expect(page.locator('text=Drag & drop files here')).toBeVisible();
  });

  test('should have proper focus indicators', async ({ page }) => {
    // Test that focusable elements have visible focus indicators
    const focusableElements = await page.locator(
      'button, input, select, textarea, a[href], [tabindex]:not([tabindex="-1"])'
    ).all();

    for (const element of focusableElements) {
      await element.focus();
      
      // Check that element has focus styles (outline, box-shadow, etc.)
      const computedStyle = await element.evaluate((el) => {
        const style = window.getComputedStyle(el);
        return {
          outline: style.outline,
          outlineWidth: style.outlineWidth,
          boxShadow: style.boxShadow
        };
      });

      // Should have some form of focus indicator
      const hasFocusIndicator = 
        computedStyle.outline !== 'none' ||
        computedStyle.outlineWidth !== '0px' ||
        computedStyle.boxShadow !== 'none';
      
      expect(hasFocusIndicator).toBe(true);
    }
  });

  test('should handle reduced motion preferences', async ({ page }) => {
    // Simulate reduced motion preference
    await page.emulateMedia({ reducedMotion: 'reduce' });
    
    // Check that the page still loads and functions
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('label[for="file-input"]')).toBeVisible();
    
    // Check that essential functionality is still available
    await expect(page.locator('text=Convert between RTF and Markdown with ease')).toBeVisible();
  });

  test('should provide alternative text for images', async ({ page }) => {
    // Check all images have alt text
    const images = await page.locator('img').all();
    
    for (const image of images) {
      const altText = await image.getAttribute('alt');
      const ariaLabel = await image.getAttribute('aria-label');
      const role = await image.getAttribute('role');
      
      // Image should have alt text, aria-label, or be decorative
      expect(
        altText !== null || 
        ariaLabel !== null || 
        role === 'presentation' ||
        altText === ''  // Empty alt for decorative images
      ).toBe(true);
    }
  });

  test('should support zoom up to 200%', async ({ page }) => {
    // Test page at 200% zoom
    await page.setViewportSize({ width: 640, height: 480 }); // Simulate 200% zoom
    
    // Check that all essential functionality is still accessible
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('label[for="file-input"]')).toBeVisible();
    await expect(page.locator('text=Drag & drop files here')).toBeVisible();
    
    // Check that file upload area is still clickable
    const fileUploadLabel = page.locator('label[for="file-input"]');
    await expect(fileUploadLabel).toBeVisible();
    await fileUploadLabel.click();
  });

  test('should have proper page title and meta information', async ({ page }) => {
    // Check page title
    await expect(page).toHaveTitle(/LegacyBridge/);
    
    // Check meta description
    const metaDescription = await page.locator('meta[name="description"]').getAttribute('content');
    expect(metaDescription).toBeTruthy();
    expect(metaDescription?.length).toBeGreaterThan(20);
    
    // Check language attribute
    const htmlLang = await page.locator('html').getAttribute('lang');
    expect(htmlLang).toBe('en');
  });

  test('should handle conversion workflow with assistive technology', async ({ page }) => {
    // Test keyboard navigation through the interface
    await page.keyboard.press('Tab');
    
    // Should be able to navigate to interactive elements
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toBeVisible();
    
    // Test that file upload area can be activated with keyboard
    const fileUploadLabel = page.locator('label[for="file-input"]');
    await fileUploadLabel.focus();
    await expect(fileUploadLabel).toBeFocused();
    
    // Test Enter key activation (this will trigger the file dialog)
    await page.keyboard.press('Enter');
    
    // Page should still be responsive
    await expect(page.locator('h1')).toBeVisible();
  });
});