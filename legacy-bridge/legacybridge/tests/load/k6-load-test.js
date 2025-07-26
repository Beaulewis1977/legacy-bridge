import http from 'k6/http';
import { check, sleep } from 'k6';
import { SharedArray } from 'k6/data';
import { Rate } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const conversionSuccess = new Rate('conversion_success');

// Load test documents from fixtures
const testDocuments = new SharedArray('testDocuments', function() {
  return [
    // Small document (10KB)
    {
      name: 'small_rtf',
      content: generateRTFContent(10 * 1024),
      format: 'rtf',
      target: 'markdown'
    },
    // Medium document (100KB)
    {
      name: 'medium_rtf',
      content: generateRTFContent(100 * 1024),
      format: 'rtf',
      target: 'markdown'
    },
    // Large document (1MB)
    {
      name: 'large_rtf',
      content: generateRTFContent(1024 * 1024),
      format: 'rtf',
      target: 'markdown'
    },
    // Markdown to RTF conversions
    {
      name: 'markdown_doc',
      content: generateMarkdownContent(50 * 1024),
      format: 'markdown',
      target: 'rtf'
    }
  ];
});

// Load test configuration for 1000+ users
export let options = {
  scenarios: {
    // Smoke test scenario
    smoke_test: {
      executor: 'constant-vus',
      vus: 1,
      duration: '30s',
      startTime: '0s',
      tags: { scenario: 'smoke' }
    },
    
    // Ramp up to 1000 users
    load_test: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: 100 },   // Ramp up to 100 users
        { duration: '3m', target: 500 },   // Ramp up to 500 users  
        { duration: '5m', target: 1000 },  // Ramp up to 1000 users
        { duration: '15m', target: 1000 }, // Stay at 1000 users
        { duration: '5m', target: 0 },     // Ramp down
      ],
      startTime: '30s',
      tags: { scenario: 'load' }
    },
    
    // Spike test scenario
    spike_test: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '10s', target: 100 },
        { duration: '30s', target: 100 },
        { duration: '10s', target: 2000 }, // Spike to 2000 users
        { duration: '3m', target: 2000 },
        { duration: '10s', target: 100 },
        { duration: '1m', target: 0 },
      ],
      startTime: '35m',
      tags: { scenario: 'spike' }
    },
    
    // Stress test scenario
    stress_test: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '5m', target: 1000 },
        { duration: '5m', target: 1500 },
        { duration: '5m', target: 2000 },
        { duration: '5m', target: 2500 },
        { duration: '5m', target: 3000 },
        { duration: '10m', target: 0 },
      ],
      startTime: '45m',
      tags: { scenario: 'stress' }
    }
  },
  
  thresholds: {
    // Response time thresholds
    http_req_duration: [
      'p(50)<200',   // 50% of requests under 200ms
      'p(90)<400',   // 90% of requests under 400ms
      'p(95)<500',   // 95% of requests under 500ms
      'p(99)<1000',  // 99% of requests under 1s
    ],
    
    // Error rate thresholds
    http_req_failed: ['rate<0.01'],      // Error rate under 1%
    errors: ['rate<0.01'],               // Custom error rate under 1%
    conversion_success: ['rate>0.99'],   // 99% conversion success rate
    
    // Throughput thresholds
    http_reqs: ['rate>100'],             // At least 100 requests per second
  },
  
  // External data output for analysis
  ext: {
    loadimpact: {
      projectID: 'legacybridge',
      name: 'LegacyBridge 1000+ User Load Test'
    }
  }
};

// API endpoint configuration
const BASE_URL = __ENV.BASE_URL || 'http://api.legacybridge.com';
const API_KEY = __ENV.API_KEY || 'test-api-key';

// Helper function to generate RTF content
function generateRTFContent(size) {
  const rtfHeader = '{\\rtf1\\ansi\\deff0 {\\fonttbl{\\f0 Times New Roman;}}';
  const rtfFooter = '}';
  let content = rtfHeader;
  
  // Generate content to reach target size
  const paragraph = '\\f0\\fs24 This is a test paragraph with some \\b bold\\b0 and \\i italic\\i0 text. ';
  const paragraphSize = paragraph.length;
  const paragraphsNeeded = Math.floor((size - rtfHeader.length - rtfFooter.length) / paragraphSize);
  
  for (let i = 0; i < paragraphsNeeded; i++) {
    content += paragraph + '\\par ';
  }
  
  return content + rtfFooter;
}

// Helper function to generate Markdown content
function generateMarkdownContent(size) {
  let content = '# Test Document\n\n';
  
  const paragraph = 'This is a test paragraph with **bold** and *italic* text. It contains [links](http://example.com) and `inline code`.\n\n';
  const paragraphSize = paragraph.length;
  const paragraphsNeeded = Math.floor((size - content.length) / paragraphSize);
  
  for (let i = 0; i < paragraphsNeeded; i++) {
    if (i % 10 === 0) {
      content += `## Section ${Math.floor(i / 10) + 1}\n\n`;
    }
    content += paragraph;
  }
  
  return content;
}

// Main test function
export default function() {
  // Select random test document
  const doc = testDocuments[Math.floor(Math.random() * testDocuments.length)];
  
  // Prepare request payload
  const payload = JSON.stringify({
    content: doc.content,
    format: doc.format,
    target: doc.target,
    options: {
      preserveFormatting: true,
      enableCache: true,
      timeout: 30000
    }
  });
  
  // Set up request headers
  const params = {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${API_KEY}`,
      'X-Request-ID': `${__VU}-${__ITER}-${Date.now()}`,
      'X-User-ID': `user-${__VU}`
    },
    timeout: '30s',
    tags: {
      document_type: doc.name,
      conversion_type: `${doc.format}_to_${doc.target}`
    }
  };
  
  // Make conversion request
  const startTime = new Date();
  const response = http.post(`${BASE_URL}/api/v1/convert`, payload, params);
  const endTime = new Date();
  const responseTime = endTime - startTime;
  
  // Check response
  const checkResult = check(response, {
    'status is 200': (r) => r.status === 200,
    'conversion successful': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.success === true;
      } catch (e) {
        return false;
      }
    },
    'response time OK': (r) => r.timings.duration < 500,
    'has converted content': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.content && body.content.length > 0;
      } catch (e) {
        return false;
      }
    }
  });
  
  // Record custom metrics
  errorRate.add(!checkResult);
  conversionSuccess.add(response.status === 200);
  
  // Log errors for debugging
  if (response.status !== 200) {
    console.error(`Conversion failed: ${response.status} - ${response.body}`);
  }
  
  // Simulate realistic user behavior with think time
  sleep(Math.random() * 2 + 1); // 1-3 seconds between requests
}

// Setup function for test initialization
export function setup() {
  // Verify API is accessible
  const response = http.get(`${BASE_URL}/health`);
  check(response, {
    'API is healthy': (r) => r.status === 200
  });
  
  return {
    startTime: new Date().toISOString()
  };
}

// Teardown function for test cleanup
export function teardown(data) {
  console.log(`Load test completed. Started at: ${data.startTime}`);
}

// Handle test lifecycle events
export function handleSummary(data) {
  const summary = {
    testName: 'LegacyBridge 1000+ User Load Test',
    timestamp: new Date().toISOString(),
    duration: data.state.testRunDurationMs,
    scenarios: {},
    metrics: {
      requests: {
        total: data.metrics.http_reqs?.values?.count || 0,
        rps: data.metrics.http_reqs?.values?.rate || 0
      },
      responseTime: {
        avg: data.metrics.http_req_duration?.values?.avg || 0,
        p50: data.metrics.http_req_duration?.values?.['p(50)'] || 0,
        p90: data.metrics.http_req_duration?.values?.['p(90)'] || 0,
        p95: data.metrics.http_req_duration?.values?.['p(95)'] || 0,
        p99: data.metrics.http_req_duration?.values?.['p(99)'] || 0,
        max: data.metrics.http_req_duration?.values?.max || 0
      },
      errors: {
        rate: data.metrics.http_req_failed?.values?.rate || 0,
        count: data.metrics.http_req_failed?.values?.passes || 0
      },
      conversionSuccess: {
        rate: data.metrics.conversion_success?.values?.rate || 0
      }
    },
    thresholds: data.thresholds
  };
  
  return {
    'stdout': JSON.stringify(summary, null, 2),
    'load-test-report.json': JSON.stringify(summary, null, 2),
    'load-test-report.html': generateHTMLReport(summary)
  };
}

// Generate HTML report
function generateHTMLReport(summary) {
  return `
<!DOCTYPE html>
<html>
<head>
  <title>LegacyBridge Load Test Report</title>
  <style>
    body { font-family: Arial, sans-serif; margin: 20px; }
    h1, h2 { color: #333; }
    .metric { background: #f5f5f5; padding: 10px; margin: 10px 0; border-radius: 5px; }
    .success { color: green; }
    .failure { color: red; }
    table { border-collapse: collapse; width: 100%; }
    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
    th { background: #f2f2f2; }
  </style>
</head>
<body>
  <h1>LegacyBridge Load Test Report</h1>
  <p>Test completed at: ${summary.timestamp}</p>
  <p>Duration: ${(summary.duration / 1000 / 60).toFixed(2)} minutes</p>
  
  <h2>Performance Metrics</h2>
  <div class="metric">
    <h3>Requests</h3>
    <p>Total: ${summary.metrics.requests.total.toLocaleString()}</p>
    <p>Rate: ${summary.metrics.requests.rps.toFixed(2)} req/s</p>
  </div>
  
  <div class="metric">
    <h3>Response Times</h3>
    <table>
      <tr><th>Metric</th><th>Value (ms)</th></tr>
      <tr><td>Average</td><td>${summary.metrics.responseTime.avg.toFixed(2)}</td></tr>
      <tr><td>P50</td><td>${summary.metrics.responseTime.p50.toFixed(2)}</td></tr>
      <tr><td>P90</td><td>${summary.metrics.responseTime.p90.toFixed(2)}</td></tr>
      <tr><td>P95</td><td>${summary.metrics.responseTime.p95.toFixed(2)}</td></tr>
      <tr><td>P99</td><td>${summary.metrics.responseTime.p99.toFixed(2)}</td></tr>
      <tr><td>Max</td><td>${summary.metrics.responseTime.max.toFixed(2)}</td></tr>
    </table>
  </div>
  
  <div class="metric">
    <h3>Success Rate</h3>
    <p>Conversion Success: <span class="${summary.metrics.conversionSuccess.rate > 0.99 ? 'success' : 'failure'}">${(summary.metrics.conversionSuccess.rate * 100).toFixed(2)}%</span></p>
    <p>Error Rate: <span class="${summary.metrics.errors.rate < 0.01 ? 'success' : 'failure'}">${(summary.metrics.errors.rate * 100).toFixed(2)}%</span></p>
  </div>
  
  <h2>Threshold Results</h2>
  <table>
    <tr><th>Threshold</th><th>Status</th></tr>
    ${Object.entries(summary.thresholds || {}).map(([name, result]) => `
      <tr>
        <td>${name}</td>
        <td class="${result.ok ? 'success' : 'failure'}">${result.ok ? 'PASS' : 'FAIL'}</td>
      </tr>
    `).join('')}
  </table>
</body>
</html>
  `;
}