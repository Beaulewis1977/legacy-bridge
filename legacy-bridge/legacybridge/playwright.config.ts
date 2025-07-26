import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  // Test directory
  testDir: './tests',
  
  // Test match patterns
  testMatch: [
    '**/e2e/**/*.spec.ts',
    '**/accessibility/**/*.spec.ts',
    '**/performance/**/*.spec.ts',
    '**/security/**/*.spec.ts',
    '**/integration/**/*.spec.ts',
    '**/visual-regression/**/*.spec.ts',
    '**/chaos/**/*.spec.ts'
  ],
  
  // Timeout for each test
  timeout: 60 * 1000,
  
  // Expect timeout
  expect: {
    timeout: 10000
  },
  
  // Fail on console errors
  use: {
    // Base URL
    baseURL: process.env.BASE_URL || 'http://localhost:3000',
    
    // Collect trace on failure
    trace: 'on-first-retry',
    
    // Video on failure
    video: 'retain-on-failure',
    
    // Screenshot on failure
    screenshot: 'only-on-failure',
    
    // Viewport
    viewport: { width: 1280, height: 720 },
    
    // Ignore HTTPS errors
    ignoreHTTPSErrors: true,
    
    // Locale
    locale: 'en-US',
    
    // Timezone
    timezoneId: 'America/New_York',
    
    // Permissions - removed clipboard permissions due to browser compatibility
    // permissions: ['clipboard-read', 'clipboard-write'],
    
    // Color scheme
    colorScheme: 'light'
  },
  
  // Configure projects for different browsers
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] }
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] }
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] }
    },
    {
      name: 'mobile-chrome',
      use: { ...devices['Pixel 5'] }
    },
    {
      name: 'mobile-safari',
      use: { ...devices['iPhone 13'] }
    },
    {
      name: 'tablet',
      use: { ...devices['iPad Pro'] }
    }
  ],
  
  // Reporter configuration
  reporter: [
    ['html', { outputFolder: 'tests/reports/playwright-report' }],
    ['json', { outputFile: 'tests/reports/playwright-results.json' }],
    ['junit', { outputFile: 'tests/reports/playwright-junit.xml' }],
    ['list']
  ],
  
  // Retry configuration
  retries: process.env.CI ? 2 : 1,
  
  // Parallel execution
  workers: process.env.CI ? 2 : undefined,
  fullyParallel: true,
  
  // Forbid only mode
  forbidOnly: !!process.env.CI,
  
  // Global setup
  globalSetup: './tests/setup/global-setup.ts',
  
  // Global teardown
  globalTeardown: './tests/setup/global-teardown.ts',
  
  // Output directory
  outputDir: 'tests/results',
  
  // Web server configuration - disabled since server is already running
  // webServer: {
  //   command: 'npm run dev:frontend',
  //   port: 3000,
  //   timeout: 120 * 1000,
  //   reuseExistingServer: !process.env.CI
  // }
});