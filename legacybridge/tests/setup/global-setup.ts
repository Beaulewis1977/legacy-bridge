import { chromium, FullConfig } from '@playwright/test';
import fs from 'fs/promises';
import path from 'path';

async function globalSetup(config: FullConfig) {
  console.log('🔧 Running global test setup...');

  // Create test directories
  const testDirs = [
    'tests/reports',
    'tests/screenshots',
    'tests/results',
    'tests/visual-regression/baselines',
    'tests/visual-regression/diffs'
  ];

  for (const dir of testDirs) {
    await fs.mkdir(dir, { recursive: true });
  }

  // Clear previous test results
  try {
    const resultsDir = path.join(process.cwd(), 'tests/results');
    const files = await fs.readdir(resultsDir);
    for (const file of files) {
      await fs.unlink(path.join(resultsDir, file));
    }
  } catch (error) {
    // Directory might not exist on first run
  }

  // Set up authentication state if needed
  if (process.env.REQUIRES_AUTH) {
    const browser = await chromium.launch();
    const page = await browser.newPage();
    
    // Perform authentication
    await page.goto(config.projects[0].use.baseURL!);
    // Add authentication logic here
    
    // Save storage state
    await page.context().storageState({ 
      path: 'tests/setup/auth.json' 
    });
    
    await browser.close();
  }

  // Start test services if needed
  if (process.env.START_TEST_SERVICES) {
    console.log('🚀 Starting test services...');
    // Add service startup logic here
  }

  console.log('✅ Global setup complete');
}

export default globalSetup;