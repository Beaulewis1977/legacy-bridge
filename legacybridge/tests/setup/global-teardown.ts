import { FullConfig } from '@playwright/test';
import fs from 'fs/promises';
import path from 'path';

async function globalTeardown(config: FullConfig) {
  console.log('🧹 Running global test teardown...');

  // Generate consolidated test report
  try {
    const reports = [];
    const reportFiles = [
      'tests/reports/playwright-results.json',
      'tests/reports/jest-results.json',
      'tests/reports/security-results.json',
      'tests/reports/performance-results.json'
    ];

    for (const file of reportFiles) {
      try {
        const content = await fs.readFile(file, 'utf-8');
        reports.push(JSON.parse(content));
      } catch (error) {
        // File might not exist
      }
    }

    // Create consolidated report
    const consolidatedReport = {
      timestamp: new Date().toISOString(),
      environment: process.env.NODE_ENV,
      reports: reports,
      summary: {
        totalTests: reports.reduce((sum, r) => sum + (r.totalTests || 0), 0),
        passedTests: reports.reduce((sum, r) => sum + (r.passedTests || 0), 0),
        failedTests: reports.reduce((sum, r) => sum + (r.failedTests || 0), 0)
      }
    };

    await fs.writeFile(
      'tests/reports/consolidated-report.json',
      JSON.stringify(consolidatedReport, null, 2)
    );
  } catch (error) {
    console.error('Failed to generate consolidated report:', error);
  }

  // Archive test artifacts
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const archiveDir = path.join('tests/archives', timestamp);
  
  try {
    await fs.mkdir(archiveDir, { recursive: true });
    
    // Copy important artifacts
    const artifacts = [
      'tests/reports',
      'tests/screenshots',
      'tests/visual-regression/diffs'
    ];
    
    for (const artifact of artifacts) {
      try {
        await fs.cp(artifact, path.join(archiveDir, path.basename(artifact)), {
          recursive: true
        });
      } catch (error) {
        // Artifact might not exist
      }
    }
    
    console.log(`📦 Test artifacts archived to: ${archiveDir}`);
  } catch (error) {
    console.error('Failed to archive test artifacts:', error);
  }

  // Stop test services if needed
  if (process.env.START_TEST_SERVICES) {
    console.log('🛑 Stopping test services...');
    // Add service shutdown logic here
  }

  console.log('✅ Global teardown complete');
}

export default globalTeardown;