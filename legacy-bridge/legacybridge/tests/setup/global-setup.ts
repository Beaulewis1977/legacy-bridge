import { chromium, FullConfig } from '@playwright/test';

async function globalSetup(config: FullConfig) {
  console.log('🚀 Starting LegacyBridge test suite...');
  
  // Launch browser for setup
  const browser = await chromium.launch();
  const page = await browser.newPage();
  
  try {
    // Wait for the application to be ready
    console.log('⏳ Waiting for application to be ready...');
    await page.goto(config.projects[0].use.baseURL || 'http://localhost:3000');
    
    // Wait for the main elements to be loaded
    await page.waitForLoadState('networkidle');
    await page.waitForSelector('h1', { timeout: 10000 });
    await page.waitForSelector('text=Drag & drop files here', { timeout: 10000 });
    
    console.log('✅ Application is ready for testing');
    
    // Perform any global setup tasks
    await setupTestEnvironment(page);
    
  } catch (error) {
    console.error('❌ Global setup failed:', error);
    throw error;
  } finally {
    await browser.close();
  }
}

async function setupTestEnvironment(page: unknown) {
  // Clear any existing data
  await page.evaluate(() => {
    // Clear localStorage
    localStorage.clear();
    
    // Clear sessionStorage
    sessionStorage.clear();
    
    // Clear any IndexedDB data if used
    if ('indexedDB' in window) {
      // This would need to be customized based on your app's IndexedDB usage
    }
  });
  
  // Set up test-specific configurations
  await page.evaluate(() => {
    // Set test mode flag
    window.TEST_MODE = true;
    
    // Disable animations for faster testing
    const style = document.createElement('style');
    style.textContent = `
      *, *::before, *::after {
        animation-duration: 0.01ms !important;
        animation-delay: -0.01ms !important;
        animation-iteration-count: 1 !important;
        background-attachment: initial !important;
        scroll-behavior: auto !important;
        transition-duration: 0.01ms !important;
        transition-delay: 0ms !important;
      }
    `;
    document.head.appendChild(style);
  });
  
  console.log('🔧 Test environment configured');
}

export default globalSetup;