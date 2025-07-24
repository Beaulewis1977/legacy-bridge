import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';
import { randomString, randomItem } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';

// Custom metrics
const errorRate = new Rate('errors');
const conversionDuration = new Trend('conversion_duration');
const batchProcessingDuration = new Trend('batch_processing_duration');

// Test configuration
export const options = {
  stages: [
    // Ramp-up phase
    { duration: '2m', target: 100 },   // Ramp up to 100 users
    { duration: '5m', target: 500 },   // Ramp up to 500 users
    { duration: '10m', target: 1000 }, // Ramp up to 1000 users
    { duration: '15m', target: 1000 }, // Stay at 1000 users
    { duration: '5m', target: 500 },   // Ramp down to 500 users
    { duration: '2m', target: 0 },     // Ramp down to 0 users
  ],
  thresholds: {
    // Response time thresholds
    http_req_duration: ['p(95)<2000', 'p(99)<5000'], // 95% of requests < 2s, 99% < 5s
    http_req_failed: ['rate<0.05'],                  // Error rate < 5%
    errors: ['rate<0.05'],                            // Custom error rate < 5%
    conversion_duration: ['p(95)<3000'],              // 95% of conversions < 3s
    batch_processing_duration: ['p(95)<10000'],       // 95% of batch ops < 10s
    
    // Throughput thresholds
    http_reqs: ['rate>1000'],                         // > 1000 requests per second
  },
  
  // Test configuration
  setupTimeout: '30s',
  teardownTimeout: '30s',
  
  // Output configuration
  summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)'],
  
  // Tags for better organization
  tags: {
    test_type: 'load',
    environment: 'production',
  },
};

// Test data
const testDocuments = [
  // Small RTF document
  `{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}}
   \\f0\\fs24 This is a small test document with \\b bold\\b0 and \\i italic\\i0 text.\\par}`,
  
  // Medium RTF document with table
  `{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}}
   \\trowd\\trgaph108\\trleft-108
   \\cellx2160\\cellx4320\\cellx6480
   \\pard\\intbl Cell 1\\cell Cell 2\\cell Cell 3\\cell\\row
   \\pard\\intbl Data 1\\cell Data 2\\cell Data 3\\cell\\row
   \\pard}`,
  
  // Large RTF document with formatting
  generateLargeRTF(),
];

const testMarkdown = [
  '# Simple Document\n\nThis is a **bold** and *italic* test.',
  '# Table Test\n\n| Col1 | Col2 | Col3 |\n|------|------|------|\n| Data1 | Data2 | Data3 |',
  generateLargeMarkdown(),
];

// Base URL from environment or default
const BASE_URL = __ENV.BASE_URL || 'https://api.legacybridge.io';

// Authentication token (if needed)
const AUTH_TOKEN = __ENV.AUTH_TOKEN || '';

// Helper functions
function generateLargeRTF() {
  let rtf = '{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}}\\f0\\fs24 ';
  for (let i = 0; i < 100; i++) {
    rtf += `\\par Paragraph ${i}: ${randomString(100)} \\b bold text\\b0 `;
  }
  rtf += '}';
  return rtf;
}

function generateLargeMarkdown() {
  let md = '# Large Document\n\n';
  for (let i = 0; i < 100; i++) {
    md += `## Section ${i}\n\n${randomString(100)} **bold text**\n\n`;
  }
  return md;
}

// Setup function - runs once before the test
export function setup() {
  console.log('Setting up load test...');
  
  // Health check
  const healthCheck = http.get(`${BASE_URL}/api/health`);
  check(healthCheck, {
    'Health check passed': (r) => r.status === 200,
  });
  
  if (healthCheck.status !== 200) {
    throw new Error('Health check failed');
  }
  
  return { startTime: new Date().toISOString() };
}

// Main test scenarios
export default function () {
  const headers = {
    'Content-Type': 'application/json',
  };
  
  if (AUTH_TOKEN) {
    headers['Authorization'] = `Bearer ${AUTH_TOKEN}`;
  }
  
  // Scenario weights
  const scenario = Math.random();
  
  if (scenario < 0.4) {
    // 40% - Single RTF to Markdown conversion
    singleRTFConversion(headers);
  } else if (scenario < 0.7) {
    // 30% - Single Markdown to RTF conversion
    singleMarkdownConversion(headers);
  } else if (scenario < 0.85) {
    // 15% - Batch processing
    batchProcessing(headers);
  } else if (scenario < 0.95) {
    // 10% - File operations
    fileOperations(headers);
  } else {
    // 5% - Heavy operations
    heavyOperations(headers);
  }
  
  // Random sleep between requests (0.5-2 seconds)
  sleep(0.5 + Math.random() * 1.5);
}

// Test scenarios
function singleRTFConversion(headers) {
  const rtfContent = randomItem(testDocuments);
  const payload = JSON.stringify({
    content: rtfContent,
    options: {
      preserveFormatting: true,
      includeMetadata: false,
    },
  });
  
  const start = new Date();
  const response = http.post(`${BASE_URL}/api/convert/rtf-to-markdown`, payload, { headers });
  const duration = new Date() - start;
  
  const success = check(response, {
    'RTF conversion successful': (r) => r.status === 200,
    'Response has markdown content': (r) => r.json('markdown') !== undefined,
    'Response time acceptable': (r) => r.timings.duration < 2000,
  });
  
  errorRate.add(!success);
  conversionDuration.add(duration);
}

function singleMarkdownConversion(headers) {
  const mdContent = randomItem(testMarkdown);
  const payload = JSON.stringify({
    content: mdContent,
    options: {
      template: 'default',
      includeStyles: true,
    },
  });
  
  const start = new Date();
  const response = http.post(`${BASE_URL}/api/convert/markdown-to-rtf`, payload, { headers });
  const duration = new Date() - start;
  
  const success = check(response, {
    'Markdown conversion successful': (r) => r.status === 200,
    'Response has RTF content': (r) => r.json('rtf') !== undefined,
    'Response time acceptable': (r) => r.timings.duration < 2000,
  });
  
  errorRate.add(!success);
  conversionDuration.add(duration);
}

function batchProcessing(headers) {
  // Create batch of 10-50 documents
  const batchSize = 10 + Math.floor(Math.random() * 40);
  const documents = [];
  
  for (let i = 0; i < batchSize; i++) {
    documents.push({
      id: `doc-${i}`,
      content: randomItem(testDocuments),
      options: {
        preserveFormatting: Math.random() > 0.5,
      },
    });
  }
  
  const payload = JSON.stringify({ documents });
  
  const start = new Date();
  const response = http.post(`${BASE_URL}/api/convert/batch/rtf-to-markdown`, payload, {
    headers,
    timeout: '30s',
  });
  const duration = new Date() - start;
  
  const success = check(response, {
    'Batch processing successful': (r) => r.status === 200,
    'All documents processed': (r) => {
      const result = r.json();
      return result && result.processed === batchSize;
    },
    'Batch time reasonable': (r) => r.timings.duration < 10000,
  });
  
  errorRate.add(!success);
  batchProcessingDuration.add(duration);
}

function fileOperations(headers) {
  // Simulate file upload
  const fileName = `test-${randomString(8)}.rtf`;
  const fileContent = randomItem(testDocuments);
  
  const formData = {
    file: http.file(fileContent, fileName, 'application/rtf'),
    options: JSON.stringify({
      preserveFormatting: true,
      outputFormat: 'markdown',
    }),
  };
  
  const response = http.post(`${BASE_URL}/api/convert/file`, formData, {
    headers: { ...headers, 'Content-Type': undefined }, // Let k6 set multipart headers
  });
  
  const success = check(response, {
    'File conversion successful': (r) => r.status === 200,
    'Response has download URL': (r) => r.json('downloadUrl') !== undefined,
  });
  
  errorRate.add(!success);
}

function heavyOperations(headers) {
  // Test with very large document
  const largeDoc = generateLargeRTF();
  const payload = JSON.stringify({
    content: largeDoc,
    options: {
      preserveFormatting: true,
      includeMetadata: true,
      generateTOC: true,
      extractImages: true,
    },
  });
  
  const response = http.post(`${BASE_URL}/api/convert/rtf-to-markdown`, payload, {
    headers,
    timeout: '60s',
  });
  
  const success = check(response, {
    'Large document processed': (r) => r.status === 200,
    'Response time under 30s': (r) => r.timings.duration < 30000,
  });
  
  errorRate.add(!success);
}

// Teardown function - runs once after the test
export function teardown(data) {
  console.log(`Load test completed. Started at: ${data.startTime}`);
  
  // Optional: Send test results to monitoring system
  const summary = {
    startTime: data.startTime,
    endTime: new Date().toISOString(),
    // Add more summary data as needed
  };
  
  // http.post(`${BASE_URL}/api/test-results`, JSON.stringify(summary));
}

// Custom summary handler
export function handleSummary(data) {
  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    'summary.json': JSON.stringify(data),
    'summary.html': htmlReport(data),
  };
}

// Generate HTML report
function htmlReport(data) {
  return `
<!DOCTYPE html>
<html>
<head>
    <title>LegacyBridge Load Test Results</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #333; }
        .metric { margin: 10px 0; padding: 10px; background: #f5f5f5; }
        .pass { color: green; }
        .fail { color: red; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #4CAF50; color: white; }
    </style>
</head>
<body>
    <h1>LegacyBridge Load Test Results</h1>
    <div class="metric">
        <h2>Test Summary</h2>
        <p>Duration: ${data.state.testRunDurationMs}ms</p>
        <p>Total Requests: ${data.metrics.http_reqs.values.count}</p>
        <p>RPS: ${data.metrics.http_reqs.values.rate}</p>
    </div>
    <div class="metric">
        <h2>Response Times</h2>
        <table>
            <tr>
                <th>Metric</th>
                <th>Value</th>
            </tr>
            <tr>
                <td>Average</td>
                <td>${data.metrics.http_req_duration.values.avg}ms</td>
            </tr>
            <tr>
                <td>95th Percentile</td>
                <td>${data.metrics.http_req_duration.values['p(95)']}ms</td>
            </tr>
            <tr>
                <td>99th Percentile</td>
                <td>${data.metrics.http_req_duration.values['p(99)']}ms</td>
            </tr>
        </table>
    </div>
</body>
</html>
  `;
}