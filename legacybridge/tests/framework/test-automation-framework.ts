import { TestResults, TestSuite, TestMetrics } from './types';
import { execSync, spawn } from 'child_process';
import fs from 'fs/promises';
import path from 'path';
import os from 'os';

/**
 * Comprehensive Test Automation Framework for LegacyBridge
 * Orchestrates all test types and generates unified reports
 */
export class LegacyBridgeTestFramework {
  private testSuites: Map<string, TestSuite> = new Map();
  private metrics: TestMetrics = new TestMetrics();
  private config: TestConfig;

  constructor(config?: Partial<TestConfig>) {
    this.config = {
      parallel: true,
      maxParallelSuites: os.cpus().length,
      timeout: 20 * 60 * 1000, // 20 minutes
      retryFailedTests: true,
      maxRetries: 2,
      generateReports: true,
      reportFormat: ['html', 'json', 'junit'],
      qualityGates: {
        codeCoverage: { minimum: 95, failBuild: true },
        securityScan: { maxHighVulnerabilities: 0, maxMediumVulnerabilities: 2, failBuild: true },
        performanceTests: { maxResponseTimeP95: 500, maxMemoryUsage: 100, failBuild: true },
        accessibility: { wcagLevel: 'AA', maxViolations: 0, failBuild: true },
        loadTests: { minConcurrentUsers: 1000, maxErrorRate: 0.01, failBuild: true }
      },
      ...config
    };

    this.initializeTestSuites();
  }

  private initializeTestSuites() {
    // Unit Tests
    this.testSuites.set('unit', {
      name: 'Unit Tests',
      type: 'unit',
      commands: {
        frontend: 'npm test -- --coverage --watchAll=false',
        backend: 'cd src-tauri && cargo test --all-features',
        ffi: 'cd dll-build && cargo test'
      },
      enabled: true,
      weight: 0.3
    });

    // Integration Tests
    this.testSuites.set('integration', {
      name: 'Integration Tests',
      type: 'integration',
      commands: {
        api: 'npm run test:integration:api',
        e2e: 'npm run test:e2e',
        system: 'npm run test:system'
      },
      enabled: true,
      weight: 0.25
    });

    // Security Tests
    this.testSuites.set('security', {
      name: 'Security Tests',
      type: 'security',
      commands: {
        vulnerabilities: 'npm audit --production',
        sast: 'cargo audit',
        dast: 'npm run test:security',
        dependencies: 'npm run check:dependencies'
      },
      enabled: true,
      weight: 0.15
    });

    // Performance Tests
    this.testSuites.set('performance', {
      name: 'Performance Tests',
      type: 'performance',
      commands: {
        benchmarks: 'cd src-tauri && cargo bench',
        regression: 'npm run test:performance:regression',
        memory: 'npm run test:performance:memory'
      },
      enabled: true,
      weight: 0.1
    });

    // Load Tests
    this.testSuites.set('load', {
      name: 'Load Tests',
      type: 'load',
      commands: {
        k6: 'k6 run tests/load/k6-load-test.js',
        stress: 'k6 run --stage stress tests/load/k6-load-test.js',
        spike: 'k6 run --stage spike tests/load/k6-load-test.js'
      },
      enabled: true,
      weight: 0.1
    });

    // Accessibility Tests
    this.testSuites.set('accessibility', {
      name: 'Accessibility Tests',
      type: 'accessibility',
      commands: {
        wcag: 'npm run test:a11y',
        keyboard: 'npm run test:a11y:keyboard',
        screenReader: 'npm run test:a11y:screen-reader'
      },
      enabled: true,
      weight: 0.05
    });

    // Visual Regression Tests
    this.testSuites.set('visual', {
      name: 'Visual Regression Tests',
      type: 'visual',
      commands: {
        screenshots: 'npm run test:visual',
        components: 'npm run test:visual:components',
        responsive: 'npm run test:visual:responsive'
      },
      enabled: true,
      weight: 0.03
    });

    // Chaos Engineering Tests
    this.testSuites.set('chaos', {
      name: 'Chaos Engineering Tests',
      type: 'chaos',
      commands: {
        failures: 'npm run test:chaos',
        recovery: 'npm run test:chaos:recovery',
        resilience: 'npm run test:chaos:resilience'
      },
      enabled: true,
      weight: 0.02
    });
  }

  /**
   * Run the complete test suite
   */
  async runFullTestSuite(): Promise<TestResults> {
    console.log('🚀 Starting LegacyBridge Comprehensive Test Suite');
    const startTime = Date.now();
    const results = new TestResults();

    try {
      // Pre-test setup
      await this.setupTestEnvironment();

      // Run tests based on configuration
      if (this.config.parallel) {
        await this.runTestsInParallel(results);
      } else {
        await this.runTestsSequentially(results);
      }

      // Post-test analysis
      await this.analyzeResults(results);

      // Generate reports
      if (this.config.generateReports) {
        await this.generateTestReports(results);
      }

      // Apply quality gates
      await this.applyQualityGates(results);

    } catch (error) {
      console.error('❌ Test suite failed:', error);
      results.addError('framework', error as Error);
    } finally {
      // Cleanup
      await this.cleanupTestEnvironment();
      results.duration = Date.now() - startTime;
    }

    return results;
  }

  /**
   * Run specific test suite for targeted testing
   */
  async runTestSuite(suiteName: string): Promise<TestResults> {
    const suite = this.testSuites.get(suiteName);
    if (!suite) {
      throw new Error(`Test suite '${suiteName}' not found`);
    }

    const results = new TestResults();
    await this.executeSuite(suite, results);
    return results;
  }

  /**
   * Run load test with specific user count
   */
  async runLoadTest(targetUsers: number): Promise<LoadTestResult> {
    console.log(`🔄 Running load test for ${targetUsers} users`);

    const loadTestConfig = {
      executor: 'ramping-vus',
      stages: [
        { duration: '2m', target: Math.floor(targetUsers * 0.1) },
        { duration: '3m', target: Math.floor(targetUsers * 0.5) },
        { duration: '5m', target: targetUsers },
        { duration: '10m', target: targetUsers },
        { duration: '2m', target: 0 }
      ]
    };

    const result = await this.executeLoadTest(loadTestConfig);
    return result;
  }

  private async runTestsInParallel(results: TestResults): Promise<void> {
    const enabledSuites = Array.from(this.testSuites.values())
      .filter(suite => suite.enabled);

    // Group suites by priority
    const priorityGroups = this.groupSuitesByPriority(enabledSuites);

    for (const group of priorityGroups) {
      const promises = group.map(suite => 
        this.executeSuite(suite, results).catch(error => {
          results.addError(suite.name, error);
        })
      );

      // Run group in parallel with concurrency limit
      await this.runWithConcurrencyLimit(promises, this.config.maxParallelSuites);
    }
  }

  private async runTestsSequentially(results: TestResults): Promise<void> {
    for (const suite of this.testSuites.values()) {
      if (suite.enabled) {
        await this.executeSuite(suite, results);
      }
    }
  }

  private async executeSuite(suite: TestSuite, results: TestResults): Promise<void> {
    console.log(`\n📋 Running ${suite.name}...`);
    const suiteStart = Date.now();

    for (const [testName, command] of Object.entries(suite.commands)) {
      let attempts = 0;
      let success = false;

      while (attempts <= this.config.maxRetries && !success) {
        attempts++;
        
        try {
          console.log(`  ▶️  ${testName} (attempt ${attempts})`);
          const testResult = await this.executeCommand(command);
          
          results.addTestResult(suite.name, testName, {
            status: 'passed',
            duration: testResult.duration,
            output: testResult.output
          });
          
          success = true;
          console.log(`  ✅ ${testName} passed`);
        } catch (error) {
          console.log(`  ❌ ${testName} failed (attempt ${attempts})`);
          
          if (attempts > this.config.maxRetries || !this.config.retryFailedTests) {
            results.addTestResult(suite.name, testName, {
              status: 'failed',
              error: error as Error,
              duration: 0
            });
          }
        }
      }
    }

    const suiteDuration = Date.now() - suiteStart;
    console.log(`📊 ${suite.name} completed in ${(suiteDuration / 1000).toFixed(2)}s`);
  }

  private async executeCommand(command: string): Promise<CommandResult> {
    return new Promise((resolve, reject) => {
      const startTime = Date.now();
      const output: string[] = [];

      const child = spawn(command, {
        shell: true,
        env: { ...process.env, CI: 'true' }
      });

      child.stdout.on('data', (data) => {
        output.push(data.toString());
      });

      child.stderr.on('data', (data) => {
        output.push(data.toString());
      });

      child.on('close', (code) => {
        const duration = Date.now() - startTime;
        
        if (code === 0) {
          resolve({
            success: true,
            output: output.join(''),
            duration
          });
        } else {
          reject(new Error(`Command failed with code ${code}: ${output.join('')}`));
        }
      });

      // Timeout handling
      setTimeout(() => {
        child.kill('SIGTERM');
        reject(new Error(`Command timed out after ${this.config.timeout}ms`));
      }, this.config.timeout);
    });
  }

  private async executeLoadTest(config: any): Promise<LoadTestResult> {
    const tempFile = path.join(os.tmpdir(), `k6-config-${Date.now()}.json`);
    await fs.writeFile(tempFile, JSON.stringify(config));

    try {
      const result = await this.executeCommand(
        `k6 run --config ${tempFile} tests/load/k6-load-test.js`
      );

      // Parse K6 output
      return this.parseLoadTestResults(result.output);
    } finally {
      await fs.unlink(tempFile).catch(() => {});
    }
  }

  private parseLoadTestResults(output: string): LoadTestResult {
    // Parse K6 JSON output
    const jsonMatch = output.match(/\{[\s\S]*\}/);
    if (!jsonMatch) {
      throw new Error('Failed to parse load test results');
    }

    const data = JSON.parse(jsonMatch[0]);
    
    return {
      totalRequests: data.metrics.http_reqs.values.count,
      requestsPerSecond: data.metrics.http_reqs.values.rate,
      responseTime: {
        avg: data.metrics.http_req_duration.values.avg,
        p50: data.metrics.http_req_duration.values['p(50)'],
        p90: data.metrics.http_req_duration.values['p(90)'],
        p95: data.metrics.http_req_duration.values['p(95)'],
        p99: data.metrics.http_req_duration.values['p(99)']
      },
      errorRate: data.metrics.http_req_failed.values.rate,
      maxVirtualUsers: data.metrics.vus_max.values.value,
      success: data.thresholds.http_req_duration.ok && data.thresholds.http_req_failed.ok
    };
  }

  private groupSuitesByPriority(suites: TestSuite[]): TestSuite[][] {
    // Group by weight/priority
    const critical = suites.filter(s => s.weight >= 0.2);
    const important = suites.filter(s => s.weight >= 0.1 && s.weight < 0.2);
    const standard = suites.filter(s => s.weight < 0.1);

    return [critical, important, standard].filter(g => g.length > 0);
  }

  private async runWithConcurrencyLimit(promises: Promise<any>[], limit: number): Promise<void> {
    const executing: Promise<any>[] = [];
    
    for (const promise of promises) {
      const p = Promise.resolve().then(() => promise).then(
        result => executing.splice(executing.indexOf(p), 1)
      );
      
      executing.push(p);
      
      if (executing.length >= limit) {
        await Promise.race(executing);
      }
    }
    
    await Promise.all(executing);
  }

  private async setupTestEnvironment(): Promise<void> {
    console.log('🔧 Setting up test environment...');
    
    // Ensure test directories exist
    const testDirs = ['tests/reports', 'tests/screenshots', 'tests/coverage'];
    for (const dir of testDirs) {
      await fs.mkdir(dir, { recursive: true });
    }

    // Start test database if needed
    if (process.env.USE_TEST_DB) {
      await this.executeCommand('docker-compose -f docker-compose.test.yml up -d');
    }

    // Clear previous test artifacts
    await this.executeCommand('rm -rf tests/reports/*').catch(() => {});
  }

  private async cleanupTestEnvironment(): Promise<void> {
    console.log('🧹 Cleaning up test environment...');
    
    // Stop test database
    if (process.env.USE_TEST_DB) {
      await this.executeCommand('docker-compose -f docker-compose.test.yml down');
    }

    // Archive test artifacts
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    await this.executeCommand(`tar -czf tests/archives/test-run-${timestamp}.tar.gz tests/reports`).catch(() => {});
  }

  private async analyzeResults(results: TestResults): Promise<void> {
    console.log('\n📊 Analyzing test results...');
    
    // Calculate metrics
    this.metrics.totalTests = results.getTotalTests();
    this.metrics.passedTests = results.getPassedTests();
    this.metrics.failedTests = results.getFailedTests();
    this.metrics.skippedTests = results.getSkippedTests();
    this.metrics.passRate = (this.metrics.passedTests / this.metrics.totalTests) * 100;
    this.metrics.duration = results.duration;

    // Analyze code coverage
    if (await this.fileExists('coverage/lcov.info')) {
      this.metrics.codeCoverage = await this.analyzeCodeCoverage();
    }

    // Analyze performance metrics
    this.metrics.performanceMetrics = await this.analyzePerformanceMetrics();

    // Analyze security vulnerabilities
    this.metrics.securityVulnerabilities = await this.analyzeSecurityResults();

    results.metrics = this.metrics;
  }

  private async analyzeCodeCoverage(): Promise<CodeCoverageMetrics> {
    const lcovData = await fs.readFile('coverage/lcov.info', 'utf-8');
    
    // Parse LCOV data
    const lines = lcovData.split('\n');
    let totalLines = 0;
    let coveredLines = 0;
    let totalFunctions = 0;
    let coveredFunctions = 0;
    let totalBranches = 0;
    let coveredBranches = 0;

    for (const line of lines) {
      if (line.startsWith('LF:')) totalLines += parseInt(line.slice(3));
      if (line.startsWith('LH:')) coveredLines += parseInt(line.slice(3));
      if (line.startsWith('FNF:')) totalFunctions += parseInt(line.slice(4));
      if (line.startsWith('FNH:')) coveredFunctions += parseInt(line.slice(4));
      if (line.startsWith('BRF:')) totalBranches += parseInt(line.slice(4));
      if (line.startsWith('BRH:')) coveredBranches += parseInt(line.slice(4));
    }

    return {
      line: (coveredLines / totalLines) * 100,
      function: (coveredFunctions / totalFunctions) * 100,
      branch: (coveredBranches / totalBranches) * 100,
      overall: ((coveredLines + coveredFunctions + coveredBranches) / 
                (totalLines + totalFunctions + totalBranches)) * 100
    };
  }

  private async analyzePerformanceMetrics(): Promise<PerformanceMetrics> {
    // Read benchmark results
    const benchmarkResults = await this.readJsonFile('tests/reports/benchmarks.json').catch(() => ({}));
    
    return {
      avgResponseTime: benchmarkResults.avgResponseTime || 0,
      p95ResponseTime: benchmarkResults.p95ResponseTime || 0,
      p99ResponseTime: benchmarkResults.p99ResponseTime || 0,
      throughput: benchmarkResults.throughput || 0,
      memoryUsage: benchmarkResults.memoryUsage || 0,
      cpuUsage: benchmarkResults.cpuUsage || 0
    };
  }

  private async analyzeSecurityResults(): Promise<SecurityMetrics> {
    const auditResults = await this.readJsonFile('tests/reports/security-audit.json').catch(() => ({}));
    
    return {
      high: auditResults.high || 0,
      medium: auditResults.medium || 0,
      low: auditResults.low || 0,
      info: auditResults.info || 0
    };
  }

  private async generateTestReports(results: TestResults): Promise<void> {
    console.log('\n📝 Generating test reports...');
    
    for (const format of this.config.reportFormat) {
      switch (format) {
        case 'html':
          await this.generateHTMLReport(results);
          break;
        case 'json':
          await this.generateJSONReport(results);
          break;
        case 'junit':
          await this.generateJUnitReport(results);
          break;
      }
    }

    // Generate consolidated report
    await this.generateConsolidatedReport(results);
  }

  private async generateHTMLReport(results: TestResults): Promise<void> {
    const html = `
<!DOCTYPE html>
<html>
<head>
  <title>LegacyBridge Test Report</title>
  <meta charset="UTF-8">
  <style>
    body { 
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      margin: 0;
      padding: 20px;
      background: #f5f5f5;
    }
    .container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
    h1 { color: #333; margin-bottom: 30px; }
    h2 { color: #555; margin-top: 30px; }
    .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 30px; }
    .metric { background: #f8f9fa; padding: 20px; border-radius: 8px; text-align: center; }
    .metric h3 { margin: 0 0 10px 0; color: #666; font-size: 14px; text-transform: uppercase; }
    .metric .value { font-size: 36px; font-weight: bold; margin: 10px 0; }
    .metric.success .value { color: #28a745; }
    .metric.warning .value { color: #ffc107; }
    .metric.danger .value { color: #dc3545; }
    .progress { background: #e9ecef; border-radius: 4px; height: 8px; overflow: hidden; margin-top: 10px; }
    .progress-bar { height: 100%; transition: width 0.3s; }
    .progress-bar.success { background: #28a745; }
    .progress-bar.warning { background: #ffc107; }
    .progress-bar.danger { background: #dc3545; }
    table { width: 100%; border-collapse: collapse; margin-top: 20px; }
    th, td { padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; }
    th { background: #f8f9fa; font-weight: 600; color: #495057; }
    tr:hover { background: #f8f9fa; }
    .status { display: inline-block; padding: 4px 8px; border-radius: 4px; font-size: 12px; font-weight: 600; }
    .status.passed { background: #d4edda; color: #155724; }
    .status.failed { background: #f8d7da; color: #721c24; }
    .status.skipped { background: #fff3cd; color: #856404; }
    .quality-gate { margin-top: 30px; padding: 20px; border-radius: 8px; }
    .quality-gate.passed { background: #d4edda; border: 1px solid #c3e6cb; }
    .quality-gate.failed { background: #f8d7da; border: 1px solid #f5c6cb; }
    .charts { display: grid; grid-template-columns: repeat(auto-fit, minmax(400px, 1fr)); gap: 30px; margin-top: 30px; }
    .chart { background: #f8f9fa; padding: 20px; border-radius: 8px; }
    pre { background: #f8f9fa; padding: 15px; border-radius: 4px; overflow-x: auto; }
    .timestamp { color: #6c757d; font-size: 14px; margin-top: 30px; text-align: right; }
  </style>
</head>
<body>
  <div class="container">
    <h1>🔍 LegacyBridge Test Report</h1>
    
    <div class="summary">
      <div class="metric ${this.metrics.passRate >= 99 ? 'success' : this.metrics.passRate >= 95 ? 'warning' : 'danger'}">
        <h3>Pass Rate</h3>
        <div class="value">${this.metrics.passRate.toFixed(1)}%</div>
        <div class="progress">
          <div class="progress-bar ${this.metrics.passRate >= 99 ? 'success' : this.metrics.passRate >= 95 ? 'warning' : 'danger'}" 
               style="width: ${this.metrics.passRate}%"></div>
        </div>
      </div>
      
      <div class="metric">
        <h3>Total Tests</h3>
        <div class="value">${this.metrics.totalTests}</div>
        <div style="font-size: 14px; color: #6c757d;">
          ✅ ${this.metrics.passedTests} | ❌ ${this.metrics.failedTests} | ⏭️ ${this.metrics.skippedTests}
        </div>
      </div>
      
      <div class="metric ${this.metrics.codeCoverage?.overall >= 95 ? 'success' : this.metrics.codeCoverage?.overall >= 80 ? 'warning' : 'danger'}">
        <h3>Code Coverage</h3>
        <div class="value">${(this.metrics.codeCoverage?.overall || 0).toFixed(1)}%</div>
        <div style="font-size: 12px; color: #6c757d;">
          Line: ${(this.metrics.codeCoverage?.line || 0).toFixed(1)}% | 
          Branch: ${(this.metrics.codeCoverage?.branch || 0).toFixed(1)}%
        </div>
      </div>
      
      <div class="metric">
        <h3>Duration</h3>
        <div class="value">${this.formatDuration(this.metrics.duration)}</div>
        <div style="font-size: 14px; color: #6c757d;">
          Completed ${new Date().toLocaleString()}
        </div>
      </div>
    </div>

    <h2>📊 Test Suite Results</h2>
    <table>
      <thead>
        <tr>
          <th>Test Suite</th>
          <th>Tests</th>
          <th>Passed</th>
          <th>Failed</th>
          <th>Duration</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        ${this.generateSuiteRows(results)}
      </tbody>
    </table>

    <h2>🎯 Quality Gates</h2>
    ${this.generateQualityGatesHTML(results)}

    <h2>📈 Performance Metrics</h2>
    <div class="charts">
      <div class="chart">
        <h3>Response Times</h3>
        <pre>
Average: ${this.metrics.performanceMetrics?.avgResponseTime.toFixed(2)}ms
P95: ${this.metrics.performanceMetrics?.p95ResponseTime.toFixed(2)}ms
P99: ${this.metrics.performanceMetrics?.p99ResponseTime.toFixed(2)}ms
Throughput: ${this.metrics.performanceMetrics?.throughput.toFixed(2)} req/s
        </pre>
      </div>
      
      <div class="chart">
        <h3>Resource Usage</h3>
        <pre>
Memory: ${this.metrics.performanceMetrics?.memoryUsage.toFixed(2)}MB
CPU: ${this.metrics.performanceMetrics?.cpuUsage.toFixed(2)}%
        </pre>
      </div>
    </div>

    <h2>🔒 Security Summary</h2>
    <div class="chart">
      <pre>
High: ${this.metrics.securityVulnerabilities?.high || 0}
Medium: ${this.metrics.securityVulnerabilities?.medium || 0}
Low: ${this.metrics.securityVulnerabilities?.low || 0}
Info: ${this.metrics.securityVulnerabilities?.info || 0}
      </pre>
    </div>

    ${results.hasFailures() ? this.generateFailureDetails(results) : ''}

    <div class="timestamp">
      Generated on ${new Date().toLocaleString()} | 
      LegacyBridge Test Framework v1.0.0
    </div>
  </div>
</body>
</html>
    `;

    await fs.writeFile('tests/reports/test-report.html', html);
    console.log('  ✅ HTML report generated: tests/reports/test-report.html');
  }

  private generateSuiteRows(results: TestResults): string {
    return Array.from(this.testSuites.values())
      .map(suite => {
        const suiteResults = results.getSuiteResults(suite.name);
        const total = suiteResults.length;
        const passed = suiteResults.filter(r => r.status === 'passed').length;
        const failed = suiteResults.filter(r => r.status === 'failed').length;
        const duration = suiteResults.reduce((sum, r) => sum + (r.duration || 0), 0);
        const status = failed === 0 ? 'passed' : 'failed';

        return `
          <tr>
            <td>${suite.name}</td>
            <td>${total}</td>
            <td>${passed}</td>
            <td>${failed}</td>
            <td>${this.formatDuration(duration)}</td>
            <td><span class="status ${status}">${status.toUpperCase()}</span></td>
          </tr>
        `;
      })
      .join('');
  }

  private generateQualityGatesHTML(results: TestResults): string {
    const gates = this.config.qualityGates;
    const allPassed = this.checkQualityGates(results);

    return `
      <div class="quality-gate ${allPassed ? 'passed' : 'failed'}">
        <h3>${allPassed ? '✅ All Quality Gates Passed' : '❌ Quality Gates Failed'}</h3>
        <ul>
          <li>Code Coverage: ${this.metrics.codeCoverage?.overall.toFixed(1)}% (required: ${gates.codeCoverage.minimum}%) ${this.metrics.codeCoverage?.overall >= gates.codeCoverage.minimum ? '✅' : '❌'}</li>
          <li>Security High Vulnerabilities: ${this.metrics.securityVulnerabilities?.high} (max allowed: ${gates.securityScan.maxHighVulnerabilities}) ${this.metrics.securityVulnerabilities?.high <= gates.securityScan.maxHighVulnerabilities ? '✅' : '❌'}</li>
          <li>P95 Response Time: ${this.metrics.performanceMetrics?.p95ResponseTime}ms (max allowed: ${gates.performanceTests.maxResponseTimeP95}ms) ${this.metrics.performanceMetrics?.p95ResponseTime <= gates.performanceTests.maxResponseTimeP95 ? '✅' : '❌'}</li>
          <li>Memory Usage: ${this.metrics.performanceMetrics?.memoryUsage}MB (max allowed: ${gates.performanceTests.maxMemoryUsage}MB) ${this.metrics.performanceMetrics?.memoryUsage <= gates.performanceTests.maxMemoryUsage ? '✅' : '❌'}</li>
        </ul>
      </div>
    `;
  }

  private generateFailureDetails(results: TestResults): string {
    const failures = results.getFailures();
    
    return `
      <h2>❌ Failure Details</h2>
      ${failures.map(failure => `
        <div class="chart">
          <h3>${failure.suite} - ${failure.test}</h3>
          <pre>${failure.error?.message || 'Unknown error'}</pre>
          ${failure.error?.stack ? `<pre>${failure.error.stack}</pre>` : ''}
        </div>
      `).join('')}
    `;
  }

  private async generateJSONReport(results: TestResults): Promise<void> {
    const report = {
      timestamp: new Date().toISOString(),
      summary: {
        totalTests: this.metrics.totalTests,
        passed: this.metrics.passedTests,
        failed: this.metrics.failedTests,
        skipped: this.metrics.skippedTests,
        passRate: this.metrics.passRate,
        duration: this.metrics.duration
      },
      coverage: this.metrics.codeCoverage,
      performance: this.metrics.performanceMetrics,
      security: this.metrics.securityVulnerabilities,
      suites: Array.from(this.testSuites.values()).map(suite => ({
        name: suite.name,
        type: suite.type,
        results: results.getSuiteResults(suite.name)
      })),
      qualityGates: {
        passed: this.checkQualityGates(results),
        details: this.config.qualityGates
      }
    };

    await fs.writeFile('tests/reports/test-report.json', JSON.stringify(report, null, 2));
    console.log('  ✅ JSON report generated: tests/reports/test-report.json');
  }

  private async generateJUnitReport(results: TestResults): Promise<void> {
    const junit = `<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="LegacyBridge Test Suite" tests="${this.metrics.totalTests}" failures="${this.metrics.failedTests}" time="${this.metrics.duration / 1000}">
  ${Array.from(this.testSuites.values()).map(suite => {
    const suiteResults = results.getSuiteResults(suite.name);
    const failures = suiteResults.filter(r => r.status === 'failed').length;
    const time = suiteResults.reduce((sum, r) => sum + (r.duration || 0), 0) / 1000;
    
    return `
  <testsuite name="${suite.name}" tests="${suiteResults.length}" failures="${failures}" time="${time}">
    ${suiteResults.map(result => `
    <testcase name="${result.name}" classname="${suite.name}" time="${(result.duration || 0) / 1000}">
      ${result.status === 'failed' ? `
      <failure message="${result.error?.message || 'Test failed'}" type="AssertionError">
        ${result.error?.stack || ''}
      </failure>` : ''}
      ${result.status === 'skipped' ? '<skipped/>' : ''}
    </testcase>`).join('')}
  </testsuite>`;
  }).join('')}
</testsuites>`;

    await fs.writeFile('tests/reports/test-report.xml', junit);
    console.log('  ✅ JUnit report generated: tests/reports/test-report.xml');
  }

  private async generateConsolidatedReport(results: TestResults): Promise<void> {
    const report = `# LegacyBridge Test Report

## Executive Summary
- **Total Tests**: ${this.metrics.totalTests}
- **Pass Rate**: ${this.metrics.passRate.toFixed(1)}%
- **Duration**: ${this.formatDuration(this.metrics.duration)}
- **Code Coverage**: ${this.metrics.codeCoverage?.overall.toFixed(1)}%

## Quality Gates
${this.checkQualityGates(results) ? '✅ All quality gates passed' : '❌ Some quality gates failed'}

### Details
- Code Coverage: ${this.metrics.codeCoverage?.overall.toFixed(1)}% (required: ${this.config.qualityGates.codeCoverage.minimum}%)
- Security Vulnerabilities: ${this.metrics.securityVulnerabilities?.high} high, ${this.metrics.securityVulnerabilities?.medium} medium
- Performance: P95 ${this.metrics.performanceMetrics?.p95ResponseTime}ms (max: ${this.config.qualityGates.performanceTests.maxResponseTimeP95}ms)
- Load Test: Supports ${this.metrics.maxConcurrentUsers || 'N/A'} concurrent users

## Test Results by Suite
${Array.from(this.testSuites.values()).map(suite => {
  const suiteResults = results.getSuiteResults(suite.name);
  const passed = suiteResults.filter(r => r.status === 'passed').length;
  const total = suiteResults.length;
  return `### ${suite.name}: ${passed}/${total} passed`;
}).join('\n')}

## Recommendations
${this.generateRecommendations(results)}

---
Generated: ${new Date().toISOString()}
`;

    await fs.writeFile('tests/reports/test-summary.md', report);
    console.log('  ✅ Summary report generated: tests/reports/test-summary.md');
  }

  private async applyQualityGates(results: TestResults): Promise<void> {
    console.log('\n🚦 Applying quality gates...');
    
    const passed = this.checkQualityGates(results);
    
    if (!passed && this.config.qualityGates.codeCoverage.failBuild) {
      throw new Error('Quality gates failed - build should not proceed');
    }
    
    console.log(passed ? '  ✅ All quality gates passed' : '  ⚠️  Some quality gates failed');
  }

  private checkQualityGates(results: TestResults): boolean {
    const gates = this.config.qualityGates;
    
    const checks = [
      this.metrics.codeCoverage?.overall >= gates.codeCoverage.minimum,
      this.metrics.securityVulnerabilities?.high <= gates.securityScan.maxHighVulnerabilities,
      this.metrics.securityVulnerabilities?.medium <= gates.securityScan.maxMediumVulnerabilities,
      this.metrics.performanceMetrics?.p95ResponseTime <= gates.performanceTests.maxResponseTimeP95,
      this.metrics.performanceMetrics?.memoryUsage <= gates.performanceTests.maxMemoryUsage
    ];
    
    return checks.every(check => check);
  }

  private generateRecommendations(results: TestResults): string {
    const recommendations: string[] = [];
    
    if (this.metrics.codeCoverage?.overall < 95) {
      recommendations.push('- Increase code coverage to meet the 95% target');
    }
    
    if (this.metrics.securityVulnerabilities?.high > 0) {
      recommendations.push('- Address high-severity security vulnerabilities immediately');
    }
    
    if (this.metrics.performanceMetrics?.p95ResponseTime > 400) {
      recommendations.push('- Optimize performance to reduce P95 response time below 400ms');
    }
    
    if (results.getFailedTests() > 0) {
      recommendations.push('- Fix failing tests before proceeding with deployment');
    }
    
    return recommendations.length > 0 ? recommendations.join('\n') : '- All metrics are within acceptable ranges';
  }

  private formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    const minutes = Math.floor(ms / 60000);
    const seconds = ((ms % 60000) / 1000).toFixed(0);
    return `${minutes}m ${seconds}s`;
  }

  private async fileExists(path: string): Promise<boolean> {
    try {
      await fs.access(path);
      return true;
    } catch {
      return false;
    }
  }

  private async readJsonFile(path: string): Promise<any> {
    const content = await fs.readFile(path, 'utf-8');
    return JSON.parse(content);
  }
}

// Type definitions
interface TestConfig {
  parallel: boolean;
  maxParallelSuites: number;
  timeout: number;
  retryFailedTests: boolean;
  maxRetries: number;
  generateReports: boolean;
  reportFormat: ('html' | 'json' | 'junit')[];
  qualityGates: QualityGates;
}

interface QualityGates {
  codeCoverage: { minimum: number; failBuild: boolean };
  securityScan: { maxHighVulnerabilities: number; maxMediumVulnerabilities: number; failBuild: boolean };
  performanceTests: { maxResponseTimeP95: number; maxMemoryUsage: number; failBuild: boolean };
  accessibility: { wcagLevel: string; maxViolations: number; failBuild: boolean };
  loadTests: { minConcurrentUsers: number; maxErrorRate: number; failBuild: boolean };
}

interface TestSuite {
  name: string;
  type: string;
  commands: Record<string, string>;
  enabled: boolean;
  weight: number;
}

interface CommandResult {
  success: boolean;
  output: string;
  duration: number;
}

interface LoadTestResult {
  totalRequests: number;
  requestsPerSecond: number;
  responseTime: {
    avg: number;
    p50: number;
    p90: number;
    p95: number;
    p99: number;
  };
  errorRate: number;
  maxVirtualUsers: number;
  success: boolean;
}

interface CodeCoverageMetrics {
  line: number;
  function: number;
  branch: number;
  overall: number;
}

interface PerformanceMetrics {
  avgResponseTime: number;
  p95ResponseTime: number;
  p99ResponseTime: number;
  throughput: number;
  memoryUsage: number;
  cpuUsage: number;
}

interface SecurityMetrics {
  high: number;
  medium: number;
  low: number;
  info: number;
}

// Test results management
class TestResults {
  private results: Map<string, TestResult[]> = new Map();
  private errors: Map<string, Error> = new Map();
  public duration: number = 0;
  public metrics?: TestMetrics;

  addTestResult(suite: string, test: string, result: Partial<TestResult>): void {
    if (!this.results.has(suite)) {
      this.results.set(suite, []);
    }
    
    this.results.get(suite)!.push({
      name: test,
      status: result.status || 'pending',
      duration: result.duration || 0,
      error: result.error,
      output: result.output
    });
  }

  addError(context: string, error: Error): void {
    this.errors.set(context, error);
  }

  getSuiteResults(suite: string): TestResult[] {
    return this.results.get(suite) || [];
  }

  getTotalTests(): number {
    let total = 0;
    for (const results of this.results.values()) {
      total += results.length;
    }
    return total;
  }

  getPassedTests(): number {
    let passed = 0;
    for (const results of this.results.values()) {
      passed += results.filter(r => r.status === 'passed').length;
    }
    return passed;
  }

  getFailedTests(): number {
    let failed = 0;
    for (const results of this.results.values()) {
      failed += results.filter(r => r.status === 'failed').length;
    }
    return failed;
  }

  getSkippedTests(): number {
    let skipped = 0;
    for (const results of this.results.values()) {
      skipped += results.filter(r => r.status === 'skipped').length;
    }
    return skipped;
  }

  hasFailures(): boolean {
    return this.getFailedTests() > 0 || this.errors.size > 0;
  }

  getFailures(): Array<{suite: string, test: string, error?: Error}> {
    const failures: Array<{suite: string, test: string, error?: Error}> = [];
    
    for (const [suite, results] of this.results.entries()) {
      for (const result of results) {
        if (result.status === 'failed') {
          failures.push({ suite, test: result.name, error: result.error });
        }
      }
    }
    
    return failures;
  }
}

class TestMetrics {
  totalTests: number = 0;
  passedTests: number = 0;
  failedTests: number = 0;
  skippedTests: number = 0;
  passRate: number = 0;
  duration: number = 0;
  codeCoverage?: CodeCoverageMetrics;
  performanceMetrics?: PerformanceMetrics;
  securityVulnerabilities?: SecurityMetrics;
  maxConcurrentUsers?: number;
}

interface TestResult {
  name: string;
  status: 'passed' | 'failed' | 'skipped' | 'pending';
  duration: number;
  error?: Error;
  output?: string;
}

// Export the framework
export { TestResults, TestSuite, TestMetrics, TestConfig, QualityGates };