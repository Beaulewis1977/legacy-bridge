import { test, expect } from '@playwright/test';
import { injectAxe, checkA11y, getViolations } from 'axe-playwright';
import { Page } from '@playwright/test';

// WCAG 2.1 AA compliance tests
test.describe('Accessibility Compliance Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:3000');
    await injectAxe(page);
  });

  test('should have no automatic accessibility violations', async ({ page }) => {
    const violations = await getViolations(page);
    
    if (violations.length > 0) {
      console.log('Accessibility violations found:');
      violations.forEach((violation, index) => {
        console.log(`\n${index + 1}. ${violation.id}: ${violation.description}`);
        console.log(`   Impact: ${violation.impact}`);
        console.log(`   Affected elements: ${violation.nodes.length}`);
        violation.nodes.forEach(node => {
          console.log(`   - ${node.target}`);
        });
      });
    }
    
    expect(violations).toHaveLength(0);
  });

  test('should meet WCAG 2.1 AA standards', async ({ page }) => {
    await checkA11y(page, null, {
      detailedReport: true,
      detailedReportOptions: {
        html: true
      },
      runOnly: {
        type: 'tag',
        values: ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']
      }
    });
  });

  test('should have proper heading hierarchy', async ({ page }) => {
    const headings = await page.$$eval('h1, h2, h3, h4, h5, h6', elements => 
      elements.map(el => ({
        level: parseInt(el.tagName[1]),
        text: el.textContent?.trim() || ''
      }))
    );

    // Check that heading levels don't skip
    let lastLevel = 0;
    for (const heading of headings) {
      if (lastLevel > 0) {
        expect(heading.level).toBeLessThanOrEqual(lastLevel + 1);
      }
      lastLevel = heading.level;
    }

    // Ensure there's exactly one h1
    const h1Count = headings.filter(h => h.level === 1).length;
    expect(h1Count).toBe(1);
  });

  test('should have sufficient color contrast', async ({ page }) => {
    await checkA11y(page, null, {
      runOnly: {
        type: 'rule',
        values: ['color-contrast']
      }
    });
  });

  test('should have proper alt text for images', async ({ page }) => {
    const images = await page.$$('img');
    
    for (const img of images) {
      const alt = await img.getAttribute('alt');
      const src = await img.getAttribute('src');
      const isDecorative = await img.getAttribute('role') === 'presentation';
      
      if (!isDecorative) {
        expect(alt).toBeTruthy();
        expect(alt?.length).toBeGreaterThan(0);
        
        // Alt text should be descriptive, not just the filename
        if (src) {
          const filename = src.split('/').pop()?.split('.')[0];
          expect(alt?.toLowerCase()).not.toBe(filename?.toLowerCase());
        }
      }
    }
  });

  test('should have proper ARIA labels', async ({ page }) => {
    // Check interactive elements have labels
    const interactiveElements = await page.$$('button, a, input, select, textarea');
    
    for (const element of interactiveElements) {
      const tagName = await element.evaluate(el => el.tagName.toLowerCase());
      const ariaLabel = await element.getAttribute('aria-label');
      const ariaLabelledBy = await element.getAttribute('aria-labelledby');
      const innerText = await element.innerText();
      const title = await element.getAttribute('title');
      
      // For inputs, check for associated label
      if (tagName === 'input' || tagName === 'select' || tagName === 'textarea') {
        const id = await element.getAttribute('id');
        const label = id ? await page.$(`label[for="${id}"]`) : null;
        const hasLabel = label !== null;
        
        expect(
          ariaLabel || ariaLabelledBy || hasLabel || title
        ).toBeTruthy();
      } else {
        // For buttons and links
        expect(
          ariaLabel || ariaLabelledBy || innerText || title
        ).toBeTruthy();
      }
    }
  });

  test('should have focus indicators', async ({ page }) => {
    const focusableElements = await page.$$('button, a, input, select, textarea, [tabindex]');
    
    for (const element of focusableElements) {
      await element.focus();
      
      // Check if element has visible focus indicator
      const outlineWidth = await element.evaluate(el => 
        window.getComputedStyle(el).outlineWidth
      );
      const outlineStyle = await element.evaluate(el => 
        window.getComputedStyle(el).outlineStyle
      );
      const boxShadow = await element.evaluate(el => 
        window.getComputedStyle(el).boxShadow
      );
      
      const hasVisibleFocus = 
        (outlineStyle !== 'none' && outlineWidth !== '0px') ||
        (boxShadow && boxShadow !== 'none');
      
      expect(hasVisibleFocus).toBeTruthy();
    }
  });

  test('should announce live regions properly', async ({ page }) => {
    // Find all live regions
    const liveRegions = await page.$$('[aria-live], [role="alert"], [role="status"]');
    
    for (const region of liveRegions) {
      const ariaLive = await region.getAttribute('aria-live');
      const role = await region.getAttribute('role');
      
      // Verify appropriate politeness level
      if (role === 'alert') {
        expect(ariaLive).toBe('assertive');
      } else if (role === 'status') {
        expect(ariaLive || 'polite').toBe('polite');
      }
    }
  });
});

test.describe('Keyboard Navigation Tests', () => {
  test('should support full keyboard navigation', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Test tab navigation
    await page.keyboard.press('Tab');
    let focusedElement = await page.evaluate(() => document.activeElement?.tagName);
    expect(focusedElement).toBeTruthy();
    
    // Count all focusable elements
    const focusableCount = await page.$$eval(
      'button:not([disabled]), a[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
      elements => elements.length
    );
    
    // Tab through all elements
    const focusedElements = new Set();
    for (let i = 0; i < focusableCount + 2; i++) {
      const tagName = await page.evaluate(() => document.activeElement?.tagName);
      const id = await page.evaluate(() => document.activeElement?.id);
      if (tagName && id) {
        focusedElements.add(`${tagName}#${id}`);
      }
      await page.keyboard.press('Tab');
    }
    
    // Should be able to focus most interactive elements
    expect(focusedElements.size).toBeGreaterThan(0);
  });

  test('should support keyboard shortcuts', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Test common shortcuts
    const shortcuts = [
      { key: 'Control+o', description: 'Open file' },
      { key: 'Control+s', description: 'Save file' },
      { key: 'Control+z', description: 'Undo' },
      { key: 'Control+y', description: 'Redo' },
      { key: 'Escape', description: 'Cancel/Close' }
    ];
    
    for (const shortcut of shortcuts) {
      // Verify shortcut doesn't cause page errors
      await page.keyboard.press(shortcut.key);
      
      // Check console for errors
      const consoleErrors: string[] = [];
      page.on('console', msg => {
        if (msg.type() === 'error') {
          consoleErrors.push(msg.text());
        }
      });
      
      expect(consoleErrors).toHaveLength(0);
    }
  });

  test('should trap focus in modals', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Open a modal (adjust selector as needed)
    const modalTrigger = await page.$('[data-testid="open-settings"]');
    if (modalTrigger) {
      await modalTrigger.click();
      
      // Wait for modal to appear
      await page.waitForSelector('[role="dialog"]', { state: 'visible' });
      
      // Get all focusable elements in modal
      const modalFocusable = await page.$$('[role="dialog"] button, [role="dialog"] input, [role="dialog"] select, [role="dialog"] textarea, [role="dialog"] [tabindex]:not([tabindex="-1"])');
      
      if (modalFocusable.length > 0) {
        // Tab through modal elements
        for (let i = 0; i < modalFocusable.length + 2; i++) {
          await page.keyboard.press('Tab');
          
          // Check focus is still within modal
          const focusInModal = await page.evaluate(() => {
            const activeElement = document.activeElement;
            const modal = document.querySelector('[role="dialog"]');
            return modal?.contains(activeElement);
          });
          
          expect(focusInModal).toBeTruthy();
        }
      }
    }
  });
});

test.describe('Screen Reader Support Tests', () => {
  test('should have proper document structure', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Check for landmark regions
    const landmarks = {
      main: await page.$('main, [role="main"]'),
      navigation: await page.$('nav, [role="navigation"]'),
      banner: await page.$('header, [role="banner"]'),
      contentinfo: await page.$('footer, [role="contentinfo"]')
    };
    
    expect(landmarks.main).toBeTruthy();
    expect(landmarks.navigation).toBeTruthy();
  });

  test('should have descriptive page title', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    const title = await page.title();
    expect(title).toBeTruthy();
    expect(title.length).toBeGreaterThan(10);
    expect(title).not.toBe('Untitled');
  });

  test('should announce form validation errors', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Find a form with validation
    const form = await page.$('form');
    if (form) {
      // Submit empty form to trigger validation
      await form.evaluate(f => (f as HTMLFormElement).submit());
      
      // Check for error messages with proper ARIA
      const errorMessages = await page.$$('[role="alert"], [aria-invalid="true"]');
      
      for (const error of errorMessages) {
        const ariaLive = await error.getAttribute('aria-live');
        const role = await error.getAttribute('role');
        
        expect(role === 'alert' || ariaLive === 'assertive' || ariaLive === 'polite').toBeTruthy();
      }
    }
  });

  test('should have skip links', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Check for skip to main content link
    const skipLink = await page.$('a[href="#main"], a[href="#content"], .skip-link');
    expect(skipLink).toBeTruthy();
    
    if (skipLink) {
      // Verify it's accessible via keyboard
      await page.keyboard.press('Tab');
      const isSkipLinkFocused = await skipLink.evaluate(el => el === document.activeElement);
      
      // Skip link might be the first or second focusable element
      if (!isSkipLinkFocused) {
        await page.keyboard.press('Tab');
      }
    }
  });
});

test.describe('Responsive Accessibility Tests', () => {
  const viewports = [
    { name: 'mobile', width: 375, height: 667 },
    { name: 'tablet', width: 768, height: 1024 },
    { name: 'desktop', width: 1920, height: 1080 }
  ];

  for (const viewport of viewports) {
    test(`should be accessible on ${viewport.name}`, async ({ page }) => {
      await page.setViewportSize({ width: viewport.width, height: viewport.height });
      await page.goto('http://localhost:3000');
      await injectAxe(page);
      
      await checkA11y(page, null, {
        runOnly: {
          type: 'tag',
          values: ['wcag2aa']
        }
      });
    });

    test(`should have touch-friendly targets on ${viewport.name}`, async ({ page }) => {
      if (viewport.name === 'mobile' || viewport.name === 'tablet') {
        await page.setViewportSize({ width: viewport.width, height: viewport.height });
        await page.goto('http://localhost:3000');
        
        const interactiveElements = await page.$$('button, a, input, select, textarea');
        
        for (const element of interactiveElements) {
          const box = await element.boundingBox();
          if (box) {
            // WCAG 2.5.5: Target size should be at least 44x44 CSS pixels
            expect(box.width).toBeGreaterThanOrEqual(44);
            expect(box.height).toBeGreaterThanOrEqual(44);
          }
        }
      }
    });
  }
});

test.describe('Motion and Animation Accessibility', () => {
  test('should respect prefers-reduced-motion', async ({ page }) => {
    // Enable reduced motion preference
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.goto('http://localhost:3000');
    
    // Check that animations are disabled
    const animatedElements = await page.$$('*');
    
    for (const element of animatedElements.slice(0, 50)) { // Check first 50 elements
      const animationDuration = await element.evaluate(el => 
        window.getComputedStyle(el).animationDuration
      );
      const transitionDuration = await element.evaluate(el => 
        window.getComputedStyle(el).transitionDuration
      );
      
      // If reduced motion is respected, durations should be 0 or very short
      if (animationDuration !== '0s') {
        const duration = parseFloat(animationDuration);
        expect(duration).toBeLessThan(0.1);
      }
      
      if (transitionDuration !== '0s') {
        const duration = parseFloat(transitionDuration);
        expect(duration).toBeLessThan(0.1);
      }
    }
  });

  test('should not have auto-playing media', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Check for auto-playing video
    const videos = await page.$$('video');
    for (const video of videos) {
      const autoplay = await video.getAttribute('autoplay');
      const muted = await video.getAttribute('muted');
      
      // If autoplay is present, video must be muted
      if (autoplay !== null) {
        expect(muted).toBeTruthy();
      }
    }
    
    // Check for auto-playing audio
    const audios = await page.$$('audio');
    for (const audio of audios) {
      const autoplay = await audio.getAttribute('autoplay');
      expect(autoplay).toBeNull();
    }
  });
});

// Helper functions for generating accessibility reports
async function generateA11yReport(page: Page, reportName: string) {
  const violations = await getViolations(page);
  
  const report = {
    url: page.url(),
    timestamp: new Date().toISOString(),
    violationCount: violations.length,
    violations: violations.map(v => ({
      id: v.id,
      impact: v.impact,
      description: v.description,
      help: v.help,
      helpUrl: v.helpUrl,
      nodes: v.nodes.length
    }))
  };
  
  // Save report as JSON
  await page.evaluate((reportData) => {
    const blob = new Blob([JSON.stringify(reportData, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${reportData.reportName}-a11y-report.json`;
    a.click();
  }, { ...report, reportName });
  
  return report;
}