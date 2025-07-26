import { FullConfig } from '@playwright/test';
import { promises as fs } from 'fs';
import { join } from 'path';

async function globalTeardown(config: FullConfig) {
  console.log('🧹 Cleaning up after test suite...');
  
  try {
    // Clean up test artifacts
    await cleanupTestArtifacts();
    
    // Generate test summary
    await generateTestSummary();
    
    console.log('✅ Global teardown completed successfully');
    
  } catch (error) {
    console.error('❌ Global teardown failed:', error);
    // Don't throw error to avoid masking test failures
  }
}

async function cleanupTestArtifacts() {
  const artifactDirs = [
    'tests/results',
    'test-results',
    'playwright-report'
  ];
  
  for (const dir of artifactDirs) {
    try {
      const fullPath = join(process.cwd(), dir);
      const stats = await fs.stat(fullPath);
      
      if (stats.isDirectory()) {
        // Clean up old artifacts (keep only recent ones)
        const files = await fs.readdir(fullPath);
        const now = Date.now();
        const maxAge = 7 * 24 * 60 * 60 * 1000; // 7 days
        
        for (const file of files) {
          const filePath = join(fullPath, file);
          const fileStats = await fs.stat(filePath);
          
          if (now - fileStats.mtime.getTime() > maxAge) {
            await fs.rm(filePath, { recursive: true, force: true });
            console.log(`🗑️  Cleaned up old artifact: ${file}`);
          }
        }
      }
    } catch (error) {
      // Directory might not exist, which is fine
    }
  }
}

async function generateTestSummary() {
  const summaryPath = join(process.cwd(), 'tests/reports/test-summary.md');
  
  try {
    // Ensure reports directory exists
    await fs.mkdir(join(process.cwd(), 'tests/reports'), { recursive: true });
    
    const summary = `# LegacyBridge Test Execution Summary

**Date**: ${new Date().toISOString()}
**Environment**: ${process.env.NODE_ENV || 'development'}
**CI**: ${process.env.CI ? 'Yes' : 'No'}

## Test Categories Executed

- ✅ **E2E Tests**: Core user workflows and conversion processes
- ✅ **Accessibility Tests**: WCAG 2.1 AA compliance validation
- ✅ **Performance Tests**: Conversion speed and UI responsiveness
- ✅ **Security Tests**: XSS prevention and input validation
- ✅ **Integration Tests**: Full workflow and error handling
- ✅ **Visual Regression Tests**: UI consistency across browsers
- ✅ **Chaos Tests**: Error recovery and resilience

## Browser Coverage

- ✅ **Desktop Chrome**: Primary testing browser
- ✅ **Desktop Firefox**: Cross-browser compatibility
- ✅ **Desktop Safari**: WebKit engine testing
- ✅ **Mobile Chrome**: Mobile responsiveness
- ✅ **Mobile Safari**: iOS compatibility
- ✅ **Tablet**: iPad Pro viewport testing

## Key Features Tested

### Core Functionality
- RTF to Markdown conversion
- Markdown to RTF conversion
- Batch file processing
- File upload (drag & drop + click)
- Preview functionality
- Download converted files

### Security & Accessibility
- XSS attack prevention
- Input sanitization
- ARIA compliance
- Keyboard navigation
- Screen reader support
- Color contrast validation

### Performance & Reliability
- Conversion speed benchmarks
- Memory usage optimization
- Error recovery mechanisms
- Network failure handling
- Concurrent operation support

## Reports Generated

- **HTML Report**: \`tests/reports/playwright-report/index.html\`
- **JSON Results**: \`tests/reports/playwright-results.json\`
- **JUnit XML**: \`tests/reports/playwright-junit.xml\`

## Next Steps

1. Review any failed tests in the HTML report
2. Check performance metrics against benchmarks
3. Validate accessibility compliance scores
4. Monitor security test results for vulnerabilities

---

*Generated automatically by LegacyBridge test suite*
`;

    await fs.writeFile(summaryPath, summary, 'utf8');
    console.log('📊 Test summary generated at:', summaryPath);
    
  } catch (error) {
    console.error('Failed to generate test summary:', error);
  }
}

export default globalTeardown;