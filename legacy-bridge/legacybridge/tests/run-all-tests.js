#!/usr/bin/env node

/**
 * Comprehensive test runner for LegacyBridge
 * Executes all test suites and generates unified report
 */

const { LegacyBridgeTestFramework } = require('./framework/test-automation-framework');
const chalk = require('chalk');

async function main() {
  console.log(chalk.bold.blue('🚀 LegacyBridge Comprehensive Test Suite'));
  console.log(chalk.gray('=====================================\n'));

  const framework = new LegacyBridgeTestFramework({
    parallel: true,
    maxParallelSuites: 4,
    timeout: 30 * 60 * 1000, // 30 minutes
    retryFailedTests: true,
    maxRetries: 2,
    generateReports: true,
    reportFormat: ['html', 'json', 'junit'],
    qualityGates: {
      codeCoverage: { minimum: 95, failBuild: true },
      securityScan: { 
        maxHighVulnerabilities: 0, 
        maxMediumVulnerabilities: 2, 
        failBuild: true 
      },
      performanceTests: { 
        maxResponseTimeP95: 500, 
        maxMemoryUsage: 100, 
        failBuild: true 
      },
      accessibility: { 
        wcagLevel: 'AA', 
        maxViolations: 0, 
        failBuild: true 
      },
      loadTests: { 
        minConcurrentUsers: 1000, 
        maxErrorRate: 0.01, 
        failBuild: true 
      }
    }
  });

  try {
    // Run full test suite
    const results = await framework.runFullTestSuite();

    // Display summary
    console.log(chalk.bold('\n📊 Test Summary'));
    console.log(chalk.gray('====================================='));
    console.log(`Total Tests: ${results.metrics.totalTests}`);
    console.log(`Passed: ${chalk.green(results.metrics.passedTests)}`);
    console.log(`Failed: ${chalk.red(results.metrics.failedTests)}`);
    console.log(`Skipped: ${chalk.yellow(results.metrics.skippedTests)}`);
    console.log(`Pass Rate: ${results.metrics.passRate >= 99 ? chalk.green(results.metrics.passRate + '%') : chalk.yellow(results.metrics.passRate + '%')}`);
    console.log(`Duration: ${formatDuration(results.duration)}`);

    // Display code coverage
    if (results.metrics.codeCoverage) {
      console.log(chalk.bold('\n📈 Code Coverage'));
      console.log(chalk.gray('====================================='));
      console.log(`Overall: ${formatCoverage(results.metrics.codeCoverage.overall)}`);
      console.log(`Lines: ${formatCoverage(results.metrics.codeCoverage.line)}`);
      console.log(`Functions: ${formatCoverage(results.metrics.codeCoverage.function)}`);
      console.log(`Branches: ${formatCoverage(results.metrics.codeCoverage.branch)}`);
    }

    // Display performance metrics
    if (results.metrics.performanceMetrics) {
      console.log(chalk.bold('\n⚡ Performance Metrics'));
      console.log(chalk.gray('====================================='));
      console.log(`Avg Response Time: ${results.metrics.performanceMetrics.avgResponseTime}ms`);
      console.log(`P95 Response Time: ${results.metrics.performanceMetrics.p95ResponseTime}ms`);
      console.log(`P99 Response Time: ${results.metrics.performanceMetrics.p99ResponseTime}ms`);
      console.log(`Throughput: ${results.metrics.performanceMetrics.throughput} req/s`);
      console.log(`Memory Usage: ${results.metrics.performanceMetrics.memoryUsage}MB`);
    }

    // Display security summary
    if (results.metrics.securityVulnerabilities) {
      console.log(chalk.bold('\n🔒 Security Summary'));
      console.log(chalk.gray('====================================='));
      const sec = results.metrics.securityVulnerabilities;
      console.log(`High: ${sec.high > 0 ? chalk.red(sec.high) : chalk.green(sec.high)}`);
      console.log(`Medium: ${sec.medium > 2 ? chalk.yellow(sec.medium) : chalk.green(sec.medium)}`);
      console.log(`Low: ${chalk.gray(sec.low)}`);
      console.log(`Info: ${chalk.gray(sec.info)}`);
    }

    // Display quality gates
    console.log(chalk.bold('\n🚦 Quality Gates'));
    console.log(chalk.gray('====================================='));
    const gates = framework.config.qualityGates;
    
    const coveragePass = results.metrics.codeCoverage?.overall >= gates.codeCoverage.minimum;
    console.log(`Code Coverage (>=${gates.codeCoverage.minimum}%): ${coveragePass ? chalk.green('✅ PASS') : chalk.red('❌ FAIL')}`);
    
    const securityPass = results.metrics.securityVulnerabilities?.high <= gates.securityScan.maxHighVulnerabilities;
    console.log(`Security Scan: ${securityPass ? chalk.green('✅ PASS') : chalk.red('❌ FAIL')}`);
    
    const perfPass = results.metrics.performanceMetrics?.p95ResponseTime <= gates.performanceTests.maxResponseTimeP95;
    console.log(`Performance Tests: ${perfPass ? chalk.green('✅ PASS') : chalk.red('❌ FAIL')}`);

    // Load test results
    if (results.metrics.maxConcurrentUsers) {
      const loadPass = results.metrics.maxConcurrentUsers >= gates.loadTests.minConcurrentUsers;
      console.log(`Load Tests (${gates.loadTests.minConcurrentUsers}+ users): ${loadPass ? chalk.green('✅ PASS') : chalk.red('❌ FAIL')}`);
    }

    // Report locations
    console.log(chalk.bold('\n📄 Reports Generated'));
    console.log(chalk.gray('====================================='));
    console.log('HTML Report: tests/reports/test-report.html');
    console.log('JSON Report: tests/reports/test-report.json');
    console.log('JUnit Report: tests/reports/test-report.xml');
    console.log('Summary: tests/reports/test-summary.md');

    // Exit with appropriate code
    if (results.hasFailures()) {
      console.log(chalk.bold.red('\n❌ Test suite failed'));
      process.exit(1);
    } else {
      console.log(chalk.bold.green('\n✅ All tests passed!'));
      process.exit(0);
    }

  } catch (error) {
    console.error(chalk.bold.red('\n💥 Fatal error:'), error);
    process.exit(1);
  }
}

function formatDuration(ms) {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  const minutes = Math.floor(ms / 60000);
  const seconds = ((ms % 60000) / 1000).toFixed(0);
  return `${minutes}m ${seconds}s`;
}

function formatCoverage(percent) {
  if (percent >= 95) return chalk.green(`${percent.toFixed(1)}%`);
  if (percent >= 80) return chalk.yellow(`${percent.toFixed(1)}%`);
  return chalk.red(`${percent.toFixed(1)}%`);
}

// Run if called directly
if (require.main === module) {
  main().catch(console.error);
}

module.exports = { main };