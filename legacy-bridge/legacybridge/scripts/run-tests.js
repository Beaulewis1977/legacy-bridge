#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// ANSI color codes for console output
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m'
};

// Test categories and their configurations
const testCategories = {
  unit: {
    name: 'Unit Tests',
    command: 'npm test -- --testPathPattern=tests/unit',
    description: 'React component and utility function tests',
    icon: '🧪'
  },
  integration: {
    name: 'Integration Tests',
    command: 'npx playwright test tests/integration',
    description: 'Full workflow and system integration tests',
    icon: '🔗'
  },
  e2e: {
    name: 'End-to-End Tests',
    command: 'npx playwright test tests/e2e',
    description: 'Complete user journey testing',
    icon: '🎭'
  },
  accessibility: {
    name: 'Accessibility Tests',
    command: 'npx playwright test tests/accessibility',
    description: 'WCAG 2.1 AA compliance validation',
    icon: '♿'
  },
  performance: {
    name: 'Performance Tests',
    command: 'npx playwright test tests/performance',
    description: 'Speed and responsiveness benchmarks',
    icon: '⚡'
  },
  security: {
    name: 'Security Tests',
    command: 'npx playwright test tests/security',
    description: 'XSS prevention and input validation',
    icon: '🔒'
  },
  visual: {
    name: 'Visual Regression Tests',
    command: 'npx playwright test tests/visual-regression',
    description: 'UI consistency across browsers',
    icon: '👁️'
  },
  chaos: {
    name: 'Chaos Tests',
    command: 'npx playwright test tests/chaos',
    description: 'Error recovery and resilience testing',
    icon: '🌪️'
  }
};

// Parse command line arguments
const args = process.argv.slice(2);
const category = args[0];
const options = args.slice(1);

function printHeader() {
  console.log(`${colors.cyan}${colors.bright}`);
  console.log('╔══════════════════════════════════════════════════════════════╗');
  console.log('║                    LegacyBridge Test Suite                   ║');
  console.log('║                  Comprehensive Testing Tool                  ║');
  console.log('╚══════════════════════════════════════════════════════════════╝');
  console.log(`${colors.reset}\n`);
}

function printUsage() {
  console.log(`${colors.yellow}Usage:${colors.reset}`);
  console.log('  node scripts/run-tests.js <category> [options]\n');
  
  console.log(`${colors.yellow}Available test categories:${colors.reset}`);
  Object.entries(testCategories).forEach(([key, config]) => {
    console.log(`  ${config.icon} ${colors.green}${key.padEnd(12)}${colors.reset} - ${config.description}`);
  });
  
  console.log(`\n${colors.yellow}Special commands:${colors.reset}`);
  console.log(`  🚀 ${colors.green}all${colors.reset.padEnd(12)} - Run all test categories`);
  console.log(`  📊 ${colors.green}report${colors.reset.padEnd(10)} - Generate comprehensive test report`);
  console.log(`  🧹 ${colors.green}clean${colors.reset.padEnd(11)} - Clean test artifacts and reports`);
  
  console.log(`\n${colors.yellow}Options:${colors.reset}`);
  console.log('  --headed     - Run tests in headed mode (visible browser)');
  console.log('  --debug      - Run tests in debug mode');
  console.log('  --project    - Specify browser project (chromium, firefox, webkit)');
  console.log('  --workers    - Number of parallel workers');
  
  console.log(`\n${colors.yellow}Examples:${colors.reset}`);
  console.log('  node scripts/run-tests.js e2e');
  console.log('  node scripts/run-tests.js performance --headed');
  console.log('  node scripts/run-tests.js all --project chromium');
  console.log('  node scripts/run-tests.js security --debug');
}

function runCommand(command, description) {
  console.log(`${colors.blue}▶ Running: ${colors.bright}${description}${colors.reset}`);
  console.log(`${colors.cyan}Command: ${command}${colors.reset}\n`);
  
  try {
    const startTime = Date.now();
    execSync(command, { stdio: 'inherit', cwd: process.cwd() });
    const duration = ((Date.now() - startTime) / 1000).toFixed(2);
    
    console.log(`\n${colors.green}✅ ${description} completed successfully in ${duration}s${colors.reset}\n`);
    return true;
  } catch (error) {
    console.log(`\n${colors.red}❌ ${description} failed${colors.reset}\n`);
    return false;
  }
}

function runTestCategory(categoryKey) {
  const config = testCategories[categoryKey];
  if (!config) {
    console.log(`${colors.red}❌ Unknown test category: ${categoryKey}${colors.reset}`);
    return false;
  }
  
  let command = config.command;
  
  // Add options to command
  if (options.includes('--headed')) {
    command += ' --headed';
  }
  if (options.includes('--debug')) {
    command += ' --debug';
  }
  
  const projectIndex = options.indexOf('--project');
  if (projectIndex !== -1 && projectIndex + 1 < options.length) {
    command += ` --project ${options[projectIndex + 1]}`;
  }
  
  const workersIndex = options.indexOf('--workers');
  if (workersIndex !== -1 && workersIndex + 1 < options.length) {
    command += ` --workers ${options[workersIndex + 1]}`;
  }
  
  return runCommand(command, `${config.icon} ${config.name}`);
}

function runAllTests() {
  console.log(`${colors.magenta}🚀 Running all test categories...${colors.reset}\n`);
  
  const results = {};
  let totalPassed = 0;
  let totalFailed = 0;
  
  // Run each test category
  Object.keys(testCategories).forEach(categoryKey => {
    const success = runTestCategory(categoryKey);
    results[categoryKey] = success;
    
    if (success) {
      totalPassed++;
    } else {
      totalFailed++;
    }
  });
  
  // Print summary
  console.log(`${colors.bright}${colors.cyan}═══════════════════════════════════════════════════════════════${colors.reset}`);
  console.log(`${colors.bright}                        TEST SUMMARY                           ${colors.reset}`);
  console.log(`${colors.bright}${colors.cyan}═══════════════════════════════════════════════════════════════${colors.reset}`);
  
  Object.entries(results).forEach(([categoryKey, success]) => {
    const config = testCategories[categoryKey];
    const status = success ? `${colors.green}✅ PASSED` : `${colors.red}❌ FAILED`;
    console.log(`${config.icon} ${config.name.padEnd(25)} ${status}${colors.reset}`);
  });
  
  console.log(`\n${colors.bright}Total: ${colors.green}${totalPassed} passed${colors.reset}${colors.bright}, ${colors.red}${totalFailed} failed${colors.reset}`);
  
  if (totalFailed === 0) {
    console.log(`\n${colors.green}${colors.bright}🎉 All tests passed! LegacyBridge is ready for deployment.${colors.reset}`);
  } else {
    console.log(`\n${colors.red}${colors.bright}⚠️  Some tests failed. Please review the results above.${colors.reset}`);
  }
  
  return totalFailed === 0;
}

function generateReport() {
  console.log(`${colors.blue}📊 Generating comprehensive test report...${colors.reset}\n`);
  
  // Run Playwright report generation
  const success = runCommand('npx playwright show-report', 'Test Report Generation');
  
  if (success) {
    console.log(`${colors.green}📋 Test report available at: tests/reports/playwright-report/index.html${colors.reset}`);
  }
  
  return success;
}

function cleanArtifacts() {
  console.log(`${colors.yellow}🧹 Cleaning test artifacts...${colors.reset}\n`);
  
  const dirsToClean = [
    'tests/results',
    'test-results',
    'playwright-report',
    'tests/reports'
  ];
  
  dirsToClean.forEach(dir => {
    try {
      if (fs.existsSync(dir)) {
        fs.rmSync(dir, { recursive: true, force: true });
        console.log(`${colors.green}✅ Cleaned: ${dir}${colors.reset}`);
      }
    } catch (error) {
      console.log(`${colors.red}❌ Failed to clean: ${dir}${colors.reset}`);
    }
  });
  
  console.log(`\n${colors.green}🧹 Cleanup completed${colors.reset}`);
  return true;
}

// Main execution
function main() {
  printHeader();
  
  if (!category) {
    printUsage();
    process.exit(1);
  }
  
  let success = false;
  
  switch (category) {
    case 'all':
      success = runAllTests();
      break;
    case 'report':
      success = generateReport();
      break;
    case 'clean':
      success = cleanArtifacts();
      break;
    default:
      success = runTestCategory(category);
      break;
  }
  
  process.exit(success ? 0 : 1);
}

// Handle uncaught exceptions
process.on('uncaughtException', (error) => {
  console.error(`${colors.red}❌ Uncaught exception: ${error.message}${colors.reset}`);
  process.exit(1);
});

process.on('unhandledRejection', (reason, promise) => {
  console.error(`${colors.red}❌ Unhandled rejection at: ${promise}, reason: ${reason}${colors.reset}`);
  process.exit(1);
});

// Run main function
main();