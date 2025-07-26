/**
 * Type definitions for LegacyBridge Test Framework
 */

export interface TestConfig {
  parallel: boolean;
  maxParallelSuites: number;
  timeout: number;
  retryFailedTests: boolean;
  maxRetries: number;
  generateReports: boolean;
  reportFormat: ('html' | 'json' | 'junit')[];
  qualityGates: QualityGates;
}

export interface QualityGates {
  codeCoverage: {
    minimum: number;
    failBuild: boolean;
  };
  securityScan: {
    maxHighVulnerabilities: number;
    maxMediumVulnerabilities: number;
    failBuild: boolean;
  };
  performanceTests: {
    maxResponseTimeP95: number;
    maxMemoryUsage: number;
    failBuild: boolean;
  };
  accessibility: {
    wcagLevel: string;
    maxViolations: number;
    failBuild: boolean;
  };
  loadTests: {
    minConcurrentUsers: number;
    maxErrorRate: number;
    failBuild: boolean;
  };
}

export interface TestSuite {
  name: string;
  type: 'unit' | 'integration' | 'security' | 'performance' | 'load' | 'accessibility' | 'visual' | 'chaos';
  commands: Record<string, string>;
  enabled: boolean;
  weight: number;
}

export interface TestResult {
  name: string;
  status: 'passed' | 'failed' | 'skipped' | 'pending';
  duration: number;
  error?: Error;
  output?: string;
}

export interface CommandResult {
  success: boolean;
  output: string;
  duration: number;
}

export interface LoadTestResult {
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

export interface CodeCoverageMetrics {
  line: number;
  function: number;
  branch: number;
  overall: number;
}

export interface PerformanceMetrics {
  avgResponseTime: number;
  p95ResponseTime: number;
  p99ResponseTime: number;
  throughput: number;
  memoryUsage: number;
  cpuUsage: number;
}

export interface SecurityMetrics {
  high: number;
  medium: number;
  low: number;
  info: number;
}

export class TestMetrics {
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

export class TestResults {
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

  toJSON(): object {
    return {
      duration: this.duration,
      metrics: this.metrics,
      results: Object.fromEntries(this.results),
      errors: Object.fromEntries(
        Array.from(this.errors.entries()).map(([k, v]) => [k, v.message])
      )
    };
  }
}