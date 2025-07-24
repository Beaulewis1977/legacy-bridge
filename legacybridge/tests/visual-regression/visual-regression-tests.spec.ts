import { test, expect } from '@playwright/test';
import { argosScreenshot } from '@argos-ci/playwright';
import pixelmatch from 'pixelmatch';
import { PNG } from 'pngjs';
import fs from 'fs';
import path from 'path';

// Visual regression test configuration
const VISUAL_TEST_CONFIG = {
  threshold: 0.01, // 1% difference threshold
  screenshotOptions: {
    fullPage: true,
    animations: 'disabled',
    maskSelectors: ['.timestamp', '.dynamic-content'] // Mask dynamic elements
  },
  viewports: [
    { name: 'desktop', width: 1920, height: 1080 },
    { name: 'laptop', width: 1366, height: 768 },
    { name: 'tablet', width: 768, height: 1024 },
    { name: 'mobile', width: 375, height: 667 }
  ]
};

test.describe('Visual Regression Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Wait for fonts to load
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');
    await page.evaluate(() => document.fonts.ready);
    
    // Disable animations and transitions
    await page.addStyleTag({
      content: `
        *, *::before, *::after {
          animation-duration: 0s !important;
          animation-delay: 0s !important;
          transition-duration: 0s !important;
          transition-delay: 0s !important;
        }
      `
    });
  });

  test('main interface should match baseline', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Take screenshot with Argos
    await argosScreenshot(page, 'main-interface', {
      fullPage: true,
      viewports: VISUAL_TEST_CONFIG.viewports.map(v => v.name)
    });
    
    // Additional Playwright screenshot for local comparison
    await expect(page).toHaveScreenshot('main-interface.png', {
      fullPage: true,
      threshold: VISUAL_TEST_CONFIG.threshold,
      maxDiffPixels: 100
    });
  });

  test('editor view should remain consistent', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Add sample content to editor
    const editor = await page.locator('[data-testid="markdown-editor"]');
    await editor.fill(`# Sample Document

This is a test document for visual regression testing.

## Features
- **Bold text**
- *Italic text*
- \`Code blocks\`

### Table Example
| Column 1 | Column 2 |
|----------|----------|
| Data 1   | Data 2   |
`);

    await argosScreenshot(page, 'editor-with-content', {
      fullPage: false,
      selector: '[data-testid="editor-container"]'
    });
  });

  test('RTF preview should render consistently', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Load RTF content
    const rtfContent = `{\\rtf1\\ansi\\deff0 {\\fonttbl{\\f0 Times New Roman;}}
\\f0\\fs24 This is RTF content with \\b bold\\b0 and \\i italic\\i0 text.\\par}`;
    
    // Trigger RTF preview
    await page.locator('[data-testid="rtf-input"]').fill(rtfContent);
    await page.locator('[data-testid="preview-button"]').click();
    
    await page.waitForSelector('[data-testid="rtf-preview"]');
    
    await argosScreenshot(page, 'rtf-preview', {
      selector: '[data-testid="rtf-preview"]'
    });
  });

  test('dark mode should match baseline', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Toggle dark mode
    await page.locator('[data-testid="theme-toggle"]').click();
    await page.waitForTimeout(500); // Wait for theme transition
    
    await argosScreenshot(page, 'dark-mode-interface', {
      fullPage: true
    });
    
    await expect(page).toHaveScreenshot('dark-mode.png', {
      fullPage: true,
      threshold: VISUAL_TEST_CONFIG.threshold
    });
  });

  test('settings modal appearance', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Open settings
    await page.locator('[data-testid="settings-button"]').click();
    await page.waitForSelector('[role="dialog"]');
    
    await argosScreenshot(page, 'settings-modal', {
      selector: '[role="dialog"]'
    });
  });

  test('error states should be visually consistent', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Trigger various error states
    const errorStates = [
      {
        name: 'invalid-format-error',
        action: async () => {
          await page.locator('[data-testid="format-input"]').fill('invalid');
          await page.locator('[data-testid="convert-button"]').click();
        }
      },
      {
        name: 'file-too-large-error',
        action: async () => {
          // Simulate large file error
          await page.evaluate(() => {
            window.dispatchEvent(new CustomEvent('file-error', {
              detail: { type: 'size', message: 'File too large' }
            }));
          });
        }
      }
    ];
    
    for (const errorState of errorStates) {
      await errorState.action();
      await page.waitForSelector('[data-testid="error-message"]');
      
      await argosScreenshot(page, errorState.name, {
        selector: '[data-testid="error-message"]'
      });
    }
  });

  test('loading states visual consistency', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Trigger loading state
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('loading-start'));
    });
    
    await page.waitForSelector('[data-testid="loading-indicator"]');
    
    await argosScreenshot(page, 'loading-state', {
      fullPage: true
    });
  });

  test('responsive layout breakpoints', async ({ page }) => {
    for (const viewport of VISUAL_TEST_CONFIG.viewports) {
      await page.setViewportSize({
        width: viewport.width,
        height: viewport.height
      });
      
      await page.goto('http://localhost:3000');
      await page.waitForLoadState('networkidle');
      
      await argosScreenshot(page, `responsive-${viewport.name}`, {
        fullPage: true
      });
      
      // Test mobile menu if applicable
      if (viewport.name === 'mobile' || viewport.name === 'tablet') {
        const mobileMenuButton = await page.locator('[data-testid="mobile-menu-button"]');
        if (await mobileMenuButton.isVisible()) {
          await mobileMenuButton.click();
          await page.waitForSelector('[data-testid="mobile-menu"]');
          
          await argosScreenshot(page, `mobile-menu-${viewport.name}`, {
            fullPage: true
          });
        }
      }
    }
  });

  test('component states visual testing', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Test button states
    const buttons = await page.locator('button').all();
    for (let i = 0; i < Math.min(buttons.length, 5); i++) {
      const button = buttons[i];
      
      // Hover state
      await button.hover();
      await argosScreenshot(page, `button-hover-${i}`, {
        selector: button
      });
      
      // Focus state
      await button.focus();
      await argosScreenshot(page, `button-focus-${i}`, {
        selector: button
      });
    }
    
    // Test input states
    const inputs = await page.locator('input').all();
    for (let i = 0; i < Math.min(inputs.length, 3); i++) {
      const input = inputs[i];
      
      // Focus state
      await input.focus();
      await argosScreenshot(page, `input-focus-${i}`, {
        selector: input
      });
      
      // Filled state
      await input.fill('Test content');
      await argosScreenshot(page, `input-filled-${i}`, {
        selector: input
      });
    }
  });

  test('print layout visual testing', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Add content for print testing
    await page.locator('[data-testid="markdown-editor"]').fill(`
# Print Test Document

This document tests the print layout styling.

## Section 1
Lorem ipsum dolor sit amet, consectetur adipiscing elit.

## Section 2
Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
`);
    
    // Emulate print media
    await page.emulateMedia({ media: 'print' });
    
    await argosScreenshot(page, 'print-layout', {
      fullPage: true
    });
  });
});

test.describe('Cross-browser Visual Tests', () => {
  const browsers = ['chromium', 'firefox', 'webkit'];
  
  for (const browserName of browsers) {
    test(`should render consistently in ${browserName}`, async ({ page, browserName: currentBrowser }) => {
      if (currentBrowser === browserName) {
        await page.goto('http://localhost:3000');
        
        await argosScreenshot(page, `cross-browser-${browserName}`, {
          fullPage: true
        });
      }
    });
  }
});

test.describe('Pixel-perfect Comparison Tests', () => {
  const baselineDir = path.join(__dirname, 'baselines');
  const diffDir = path.join(__dirname, 'diffs');
  
  test.beforeAll(() => {
    // Create directories if they don't exist
    if (!fs.existsSync(baselineDir)) {
      fs.mkdirSync(baselineDir, { recursive: true });
    }
    if (!fs.existsSync(diffDir)) {
      fs.mkdirSync(diffDir, { recursive: true });
    }
  });
  
  test('custom pixel comparison', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    const screenshotName = 'custom-comparison';
    const screenshotPath = path.join(diffDir, `${screenshotName}-current.png`);
    const baselinePath = path.join(baselineDir, `${screenshotName}.png`);
    const diffPath = path.join(diffDir, `${screenshotName}-diff.png`);
    
    // Take current screenshot
    await page.screenshot({
      path: screenshotPath,
      fullPage: true
    });
    
    // Compare if baseline exists
    if (fs.existsSync(baselinePath)) {
      const baseline = PNG.sync.read(fs.readFileSync(baselinePath));
      const current = PNG.sync.read(fs.readFileSync(screenshotPath));
      const { width, height } = baseline;
      const diff = new PNG({ width, height });
      
      const numDiffPixels = pixelmatch(
        baseline.data,
        current.data,
        diff.data,
        width,
        height,
        { threshold: 0.1 }
      );
      
      fs.writeFileSync(diffPath, PNG.sync.write(diff));
      
      const diffPercentage = (numDiffPixels / (width * height)) * 100;
      
      // Generate HTML report
      if (diffPercentage > VISUAL_TEST_CONFIG.threshold * 100) {
        const reportHtml = `
<!DOCTYPE html>
<html>
<head>
  <title>Visual Regression Report</title>
  <style>
    body { font-family: Arial, sans-serif; margin: 20px; }
    .comparison { display: flex; gap: 20px; margin: 20px 0; }
    .image-container { flex: 1; }
    img { width: 100%; border: 1px solid #ccc; }
    .diff-info { background: #f5f5f5; padding: 10px; border-radius: 5px; }
    .fail { color: red; }
    .pass { color: green; }
  </style>
</head>
<body>
  <h1>Visual Regression Report: ${screenshotName}</h1>
  <div class="diff-info">
    <p>Difference: <span class="${diffPercentage > 1 ? 'fail' : 'pass'}">${diffPercentage.toFixed(2)}%</span></p>
    <p>Diff pixels: ${numDiffPixels} / ${width * height}</p>
    <p>Threshold: ${VISUAL_TEST_CONFIG.threshold * 100}%</p>
  </div>
  <div class="comparison">
    <div class="image-container">
      <h3>Baseline</h3>
      <img src="../baselines/${screenshotName}.png" alt="Baseline">
    </div>
    <div class="image-container">
      <h3>Current</h3>
      <img src="${screenshotName}-current.png" alt="Current">
    </div>
    <div class="image-container">
      <h3>Difference</h3>
      <img src="${screenshotName}-diff.png" alt="Difference">
    </div>
  </div>
</body>
</html>
        `;
        
        fs.writeFileSync(path.join(diffDir, `${screenshotName}-report.html`), reportHtml);
      }
      
      expect(diffPercentage).toBeLessThan(VISUAL_TEST_CONFIG.threshold * 100);
    } else {
      // Save as new baseline
      fs.copyFileSync(screenshotPath, baselinePath);
      console.log(`Created new baseline: ${baselinePath}`);
    }
  });
});

// Helper function to wait for stable rendering
async function waitForStableRender(page: any, selector?: string) {
  const target = selector ? await page.locator(selector) : page;
  
  // Wait for animations to complete
  await page.waitForTimeout(300);
  
  // Take two screenshots and compare
  const screenshot1 = await target.screenshot();
  await page.waitForTimeout(100);
  const screenshot2 = await target.screenshot();
  
  // If screenshots are identical, rendering is stable
  return screenshot1.equals(screenshot2);
}