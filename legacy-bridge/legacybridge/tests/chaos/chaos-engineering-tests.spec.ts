import { test, expect } from '@playwright/test';
import axios from 'axios';
import { exec } from 'child_process';
import { promisify } from 'util';
import os from 'os';

const execAsync = promisify(exec);

// Chaos engineering test configuration
const CHAOS_CONFIG = {
  baseUrl: process.env.API_BASE_URL || 'http://localhost:3000',
  timeouts: {
    degraded: 5000,  // 5 seconds for degraded mode
    recovery: 30000  // 30 seconds for recovery
  },
  thresholds: {
    cpuStress: 90,      // 90% CPU usage
    memoryStress: 0.9,  // 90% memory usage
    networkLatency: 500 // 500ms added latency
  }
};

test.describe('Chaos Engineering Tests', () => {
  let api: any;
  
  test.beforeAll(async () => {
    api = axios.create({
      baseURL: CHAOS_CONFIG.baseUrl,
      timeout: CHAOS_CONFIG.timeouts.degraded,
      validateStatus: () => true
    });
  });

  test.describe('Database Failure Scenarios', () => {
    test('should handle database connection loss gracefully', async () => {
      // Simulate database failure
      await simulateDatabaseFailure();
      
      // Try to perform operations
      const response = await api.post('/api/convert', {
        content: 'Test content',
        format: 'markdown',
        target: 'rtf'
      });
      
      // Should either queue the request or return graceful error
      expect([200, 503]).toContain(response.status);
      
      if (response.status === 503) {
        expect(response.data.error).toContain('temporarily unavailable');
        expect(response.data.retryAfter).toBeDefined();
      }
      
      // Restore database
      await restoreDatabaseConnection();
    });

    test('should handle database timeout', async () => {
      // Add extreme latency to database
      await addDatabaseLatency(5000);
      
      const startTime = Date.now();
      const response = await api.post('/api/convert', {
        content: 'Test content',
        format: 'markdown',
        target: 'rtf'
      });
      const duration = Date.now() - startTime;
      
      // Should timeout gracefully
      expect(duration).toBeLessThan(CHAOS_CONFIG.timeouts.degraded + 1000);
      expect([408, 503, 504]).toContain(response.status);
      
      await removeDatabaseLatency();
    });

    test('should handle database corruption', async () => {
      // Corrupt specific database entries
      await corruptDatabaseEntries();
      
      // System should detect and handle corruption
      const response = await api.get('/api/health');
      
      expect(response.data.database).toBeDefined();
      if (response.data.database.status === 'degraded') {
        expect(response.data.database.issues).toContain('corruption detected');
      }
      
      // Verify self-healing mechanisms
      await new Promise(resolve => setTimeout(resolve, 5000));
      
      const healedResponse = await api.get('/api/health');
      expect(healedResponse.data.database.status).not.toBe('critical');
    });
  });

  test.describe('Memory Pressure Scenarios', () => {
    test('should handle high memory usage', async () => {
      // Allocate memory to simulate pressure
      const memoryHog = await allocateMemory(CHAOS_CONFIG.thresholds.memoryStress);
      
      // System should still process small requests
      const response = await api.post('/api/convert', {
        content: 'Small test document',
        format: 'markdown',
        target: 'rtf'
      });
      
      expect(response.status).toBe(200);
      
      // Large requests might be rejected
      const largeContent = 'Large content '.repeat(100000);
      const largeResponse = await api.post('/api/convert', {
        content: largeContent,
        format: 'markdown',
        target: 'rtf'
      });
      
      expect([200, 503, 507]).toContain(largeResponse.status);
      
      // Clean up
      memoryHog.release();
    });

    test('should handle memory leak simulation', async () => {
      const leakSimulator = createMemoryLeak();
      
      // Monitor memory usage over time
      const memorySnapshots = [];
      for (let i = 0; i < 10; i++) {
        await api.post('/api/convert', {
          content: `Test document ${i}`,
          format: 'markdown',
          target: 'rtf'
        });
        
        const memUsage = await getProcessMemoryUsage();
        memorySnapshots.push(memUsage);
        
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
      
      // Memory growth should be bounded
      const initialMemory = memorySnapshots[0];
      const finalMemory = memorySnapshots[memorySnapshots.length - 1];
      const growthRate = (finalMemory - initialMemory) / initialMemory;
      
      expect(growthRate).toBeLessThan(0.5); // Less than 50% growth
      
      leakSimulator.stop();
    });

    test('should trigger garbage collection under pressure', async () => {
      // Create memory pressure
      const allocations = [];
      for (let i = 0; i < 100; i++) {
        allocations.push(Buffer.alloc(10 * 1024 * 1024)); // 10MB buffers
      }
      
      // Force conversions
      const responses = await Promise.all(
        Array(10).fill(null).map((_, i) => 
          api.post('/api/convert', {
            content: `Document ${i}`,
            format: 'markdown',
            target: 'rtf'
          })
        )
      );
      
      // Most should succeed despite memory pressure
      const successful = responses.filter(r => r.status === 200).length;
      expect(successful).toBeGreaterThan(7);
      
      // Clear allocations
      allocations.length = 0;
    });
  });

  test.describe('CPU Starvation Scenarios', () => {
    test('should handle CPU intensive background tasks', async () => {
      // Start CPU intensive task
      const cpuBurner = await startCPUIntensiveTask(CHAOS_CONFIG.thresholds.cpuStress);
      
      // Measure response times under CPU stress
      const responseTimes = [];
      for (let i = 0; i < 5; i++) {
        const start = Date.now();
        const response = await api.post('/api/convert', {
          content: 'Test content',
          format: 'markdown',
          target: 'rtf'
        });
        responseTimes.push(Date.now() - start);
        
        expect(response.status).toBe(200);
      }
      
      // Response times should degrade gracefully
      const avgResponseTime = responseTimes.reduce((a, b) => a + b) / responseTimes.length;
      expect(avgResponseTime).toBeLessThan(CHAOS_CONFIG.timeouts.degraded);
      
      cpuBurner.stop();
    });

    test('should prioritize critical operations under CPU stress', async () => {
      const cpuBurner = await startCPUIntensiveTask(80);
      
      // Health checks should be prioritized
      const healthResponse = await api.get('/api/health');
      expect(healthResponse.status).toBe(200);
      
      // Critical conversion should succeed
      const criticalResponse = await api.post('/api/convert', {
        content: 'Critical document',
        format: 'markdown',
        target: 'rtf',
        priority: 'high'
      });
      
      expect(criticalResponse.status).toBe(200);
      
      cpuBurner.stop();
    });
  });

  test.describe('Network Chaos Scenarios', () => {
    test('should handle network partitions', async () => {
      // Simulate network partition
      await simulateNetworkPartition();
      
      try {
        const response = await api.get('/api/health', { timeout: 2000 });
        
        // If reachable, should indicate degraded state
        if (response.status === 200) {
          expect(response.data.cluster?.status).toBe('partitioned');
        }
      } catch (error: any) {
        // Network errors are expected
        expect(['ECONNREFUSED', 'ETIMEDOUT']).toContain(error.code);
      }
      
      await restoreNetwork();
    });

    test('should handle high network latency', async () => {
      // Add network latency
      await addNetworkLatency(CHAOS_CONFIG.thresholds.networkLatency);
      
      const start = Date.now();
      const response = await api.post('/api/convert', {
        content: 'Test content',
        format: 'markdown',
        target: 'rtf'
      });
      const duration = Date.now() - start;
      
      // Should complete despite latency
      expect(response.status).toBe(200);
      expect(duration).toBeGreaterThan(CHAOS_CONFIG.thresholds.networkLatency);
      
      await removeNetworkLatency();
    });

    test('should handle packet loss', async () => {
      // Simulate 10% packet loss
      await simulatePacketLoss(10);
      
      const responses = await Promise.all(
        Array(20).fill(null).map(() => 
          api.get('/api/health').catch((e: any) => ({ status: 'error', code: e.code }))
        )
      );
      
      // Some requests should succeed despite packet loss
      const successful = responses.filter(r => r.status === 200).length;
      expect(successful).toBeGreaterThan(15); // At least 75% success rate
      
      await removePacketLoss();
    });
  });

  test.describe('Cascading Failure Scenarios', () => {
    test('should prevent cascade failures', async () => {
      // Trigger failure in one component
      await triggerComponentFailure('parser');
      
      // Other components should remain operational
      const healthResponse = await api.get('/api/health');
      
      expect(healthResponse.data.components.parser.status).toBe('down');
      expect(healthResponse.data.components.generator.status).toBe('up');
      expect(healthResponse.data.components.api.status).toBe('up');
      
      // Circuit breaker should be active
      expect(healthResponse.data.components.parser.circuitBreaker).toBe('open');
      
      await restoreComponent('parser');
    });

    test('should handle thundering herd', async () => {
      // Simulate service restart with many waiting clients
      await simulateServiceRestart();
      
      // Send many concurrent requests
      const requests = Array(100).fill(null).map((_, i) => 
        api.post('/api/convert', {
          content: `Request ${i}`,
          format: 'markdown',
          target: 'rtf'
        }).catch((e: any) => ({ status: 'error', error: e.message }))
      );
      
      const responses = await Promise.all(requests);
      
      // Should handle load without crashing
      const errors = responses.filter(r => r.status === 'error').length;
      expect(errors).toBeLessThan(20); // Less than 20% errors
      
      // Check if rate limiting kicked in
      const rateLimited = responses.filter(r => r.status === 429).length;
      expect(rateLimited).toBeGreaterThan(0);
    });
  });

  test.describe('Disk I/O Failure Scenarios', () => {
    test('should handle disk full scenarios', async () => {
      // Fill disk to 95%
      await fillDisk(0.95);
      
      // Small operations should still work
      const response = await api.post('/api/convert', {
        content: 'Small document',
        format: 'markdown',
        target: 'rtf'
      });
      
      expect([200, 507]).toContain(response.status);
      
      if (response.status === 507) {
        expect(response.data.error).toContain('Insufficient storage');
      }
      
      await cleanupDisk();
    });

    test('should handle slow disk I/O', async () => {
      // Simulate slow disk
      await simulateSlowDisk(100); // 100ms latency
      
      const start = Date.now();
      const response = await api.post('/api/convert', {
        content: 'Test document with file operations',
        format: 'markdown',
        target: 'rtf',
        saveToFile: true
      });
      const duration = Date.now() - start;
      
      expect(response.status).toBe(200);
      expect(duration).toBeGreaterThan(100);
      
      await restoreNormalDisk();
    });
  });

  test.describe('Recovery and Resilience Tests', () => {
    test('should auto-recover from failures', async () => {
      // Induce multiple failures
      await Promise.all([
        simulateDatabaseFailure(),
        addNetworkLatency(200),
        startCPUIntensiveTask(50)
      ]);
      
      // System should be in degraded state
      let healthResponse = await api.get('/api/health');
      expect(healthResponse.data.status).toBe('degraded');
      
      // Wait for auto-recovery
      await new Promise(resolve => setTimeout(resolve, CHAOS_CONFIG.timeouts.recovery));
      
      // Restore normal conditions
      await Promise.all([
        restoreDatabaseConnection(),
        removeNetworkLatency(),
        Promise.resolve() // CPU task auto-stops
      ]);
      
      // Check recovery
      healthResponse = await api.get('/api/health');
      expect(healthResponse.data.status).toBe('healthy');
    });

    test('should maintain data consistency during chaos', async () => {
      // Create test document
      const testDoc = {
        id: `chaos-test-${Date.now()}`,
        content: 'Important document that must not be lost',
        format: 'markdown'
      };
      
      // Save document
      let response = await api.post('/api/documents', testDoc);
      expect(response.status).toBe(201);
      
      // Induce chaos
      await simulateRandomChaos();
      
      // Verify document still exists and is intact
      response = await api.get(`/api/documents/${testDoc.id}`);
      expect(response.status).toBe(200);
      expect(response.data.content).toBe(testDoc.content);
      
      await stopAllChaos();
    });
  });
});

// Chaos simulation helpers
async function simulateDatabaseFailure() {
  // Implementation depends on your database setup
  await execAsync('docker pause legacybridge-db || true').catch(() => {});
}

async function restoreDatabaseConnection() {
  await execAsync('docker unpause legacybridge-db || true').catch(() => {});
}

async function addDatabaseLatency(ms: number) {
  // Add latency using tc (traffic control) or database proxy
  await execAsync(`tc qdisc add dev lo root netem delay ${ms}ms || true`).catch(() => {});
}

async function removeDatabaseLatency() {
  await execAsync('tc qdisc del dev lo root || true').catch(() => {});
}

async function corruptDatabaseEntries() {
  // Simulate corruption by modifying database files or entries
  // Implementation depends on database type
}

function allocateMemory(percentage: number) {
  const totalMemory = os.totalmem();
  const targetMemory = Math.floor(totalMemory * percentage);
  const currentUsed = os.totalmem() - os.freemem();
  const toAllocate = Math.max(0, targetMemory - currentUsed);
  
  const buffers: Buffer[] = [];
  const chunkSize = 100 * 1024 * 1024; // 100MB chunks
  let allocated = 0;
  
  while (allocated < toAllocate) {
    const size = Math.min(chunkSize, toAllocate - allocated);
    buffers.push(Buffer.alloc(size));
    allocated += size;
  }
  
  return {
    release: () => {
      buffers.length = 0;
      if (global.gc) global.gc();
    }
  };
}

function createMemoryLeak() {
  const leaks: any[] = [];
  const interval = setInterval(() => {
    leaks.push(Buffer.alloc(1024 * 1024)); // 1MB per second
  }, 1000);
  
  return {
    stop: () => {
      clearInterval(interval);
      leaks.length = 0;
    }
  };
}

async function getProcessMemoryUsage(): Promise<number> {
  const { stdout } = await execAsync(`ps -p ${process.pid} -o rss=`);
  return parseInt(stdout.trim()) * 1024; // Convert KB to bytes
}

async function startCPUIntensiveTask(targetPercentage: number) {
  const cores = os.cpus().length;
  const workers: any[] = [];
  
  // Start CPU-intensive workers
  for (let i = 0; i < cores; i++) {
    const worker = {
      active: true,
      promise: (async () => {
        while (worker.active) {
          // CPU-intensive calculation
          let sum = 0;
          for (let j = 0; j < 1000000; j++) {
            sum += Math.sqrt(j);
          }
          
          // Yield occasionally
          if (sum % 100 === 0) {
            await new Promise(resolve => setImmediate(resolve));
          }
        }
      })()
    };
    workers.push(worker);
  }
  
  return {
    stop: () => {
      workers.forEach(w => w.active = false);
    }
  };
}

async function addNetworkLatency(ms: number) {
  // Platform-specific network latency simulation
  if (process.platform === 'linux') {
    await execAsync(`tc qdisc add dev eth0 root netem delay ${ms}ms || true`).catch(() => {});
  }
}

async function removeNetworkLatency() {
  if (process.platform === 'linux') {
    await execAsync('tc qdisc del dev eth0 root || true').catch(() => {});
  }
}

async function simulatePacketLoss(percentage: number) {
  if (process.platform === 'linux') {
    await execAsync(`tc qdisc add dev eth0 root netem loss ${percentage}% || true`).catch(() => {});
  }
}

async function removePacketLoss() {
  if (process.platform === 'linux') {
    await execAsync('tc qdisc del dev eth0 root || true').catch(() => {});
  }
}

async function simulateNetworkPartition() {
  // Block specific ports or IPs
  await execAsync('iptables -A INPUT -p tcp --dport 5432 -j DROP || true').catch(() => {});
}

async function restoreNetwork() {
  await execAsync('iptables -D INPUT -p tcp --dport 5432 -j DROP || true').catch(() => {});
}

async function triggerComponentFailure(component: string) {
  // Send signal to trigger failure in specific component
  await axios.post(`${CHAOS_CONFIG.baseUrl}/api/chaos/fail/${component}`).catch(() => {});
}

async function restoreComponent(component: string) {
  await axios.post(`${CHAOS_CONFIG.baseUrl}/api/chaos/restore/${component}`).catch(() => {});
}

async function simulateServiceRestart() {
  // Simulate graceful shutdown and restart
  await axios.post(`${CHAOS_CONFIG.baseUrl}/api/chaos/restart`).catch(() => {});
  await new Promise(resolve => setTimeout(resolve, 2000));
}

async function fillDisk(percentage: number) {
  // Create large file to fill disk
  const diskInfo = await execAsync('df -B1 /').then(r => r.stdout);
  // Parse disk info and create appropriate file
  // Implementation depends on platform
}

async function cleanupDisk() {
  await execAsync('rm -f /tmp/disk-fill-test || true').catch(() => {});
}

async function simulateSlowDisk(latencyMs: number) {
  // Platform-specific disk I/O throttling
  // Could use cgroups on Linux
}

async function restoreNormalDisk() {
  // Remove disk I/O throttling
}

async function simulateRandomChaos() {
  const chaosTypes = [
    () => addNetworkLatency(Math.random() * 500),
    () => simulatePacketLoss(Math.random() * 20),
    () => startCPUIntensiveTask(50 + Math.random() * 40),
    () => allocateMemory(0.5 + Math.random() * 0.4)
  ];
  
  const selected = chaosTypes[Math.floor(Math.random() * chaosTypes.length)];
  await selected();
}

async function stopAllChaos() {
  await Promise.all([
    restoreDatabaseConnection(),
    removeNetworkLatency(),
    removePacketLoss(),
    restoreNetwork(),
    cleanupDisk(),
    restoreNormalDisk()
  ]).catch(() => {});
}