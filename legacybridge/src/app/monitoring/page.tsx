'use client';

import { useState, useEffect } from 'react';
import { MonitoringDashboard } from '@/components/monitoring/MonitoringDashboard';
import { MainLayout } from '@/components/layout/MainLayout';
import { 
  BuildStatus, 
  PerformanceMetrics, 
  LegacyFunctionStats, 
  SystemHealth 
} from '@/components/monitoring/MonitoringDashboard';

// Generate mock data for demonstration
function generateMockData() {
  const buildStatus: BuildStatus = {
    status: 'building',
    progress: Math.floor(Math.random() * 100),
    currentFile: 'src/legacy/authentication.dll',
    totalFiles: 45,
    completedFiles: Math.floor(Math.random() * 45),
    startTime: new Date(Date.now() - Math.random() * 3600000),
    estimatedTime: Math.floor(Math.random() * 600),
    errors: [],
    warnings: Math.random() > 0.7 ? ['Warning: Deprecated API usage in auth module'] : []
  };

  const performanceMetrics: PerformanceMetrics = {
    conversionsPerSecond: 75 + Math.random() * 50,
    memoryUsage: 45 + Math.random() * 30,
    cpuUsage: 35 + Math.random() * 40,
    activeConnections: Math.floor(Math.random() * 100),
    averageResponseTime: 50 + Math.random() * 100,
    throughput: 1000 + Math.random() * 500,
    history: Array.from({ length: 30 }, (_, i) => ({
      timestamp: new Date(Date.now() - (30 - i) * 60000),
      conversionsPerSecond: 75 + Math.random() * 50,
      memoryUsage: 45 + Math.random() * 30,
      cpuUsage: 35 + Math.random() * 40
    }))
  };

  const legacyFunctions: LegacyFunctionStats[] = [
    {
      name: 'legacy_auth_validate',
      calls: 15420,
      averageTime: 45.2,
      lastCallTime: new Date(),
      errorRate: 0.5,
      successRate: 99.5,
      trend: 'up'
    },
    {
      name: 'legacy_user_fetch',
      calls: 8932,
      averageTime: 125.8,
      lastCallTime: new Date(Date.now() - 60000),
      errorRate: 2.1,
      successRate: 97.9,
      trend: 'stable'
    },
    {
      name: 'legacy_db_connect',
      calls: 3245,
      averageTime: 340.5,
      lastCallTime: new Date(Date.now() - 120000),
      errorRate: 5.3,
      successRate: 94.7,
      trend: 'down'
    },
    {
      name: 'legacy_file_process',
      calls: 6789,
      averageTime: 89.3,
      lastCallTime: new Date(Date.now() - 30000),
      errorRate: 1.2,
      successRate: 98.8,
      trend: 'up'
    },
    {
      name: 'legacy_cache_lookup',
      calls: 28456,
      averageTime: 12.4,
      lastCallTime: new Date(),
      errorRate: 0.1,
      successRate: 99.9,
      trend: 'stable'
    },
    {
      name: 'legacy_encrypt_data',
      calls: 4567,
      averageTime: 67.8,
      lastCallTime: new Date(Date.now() - 90000),
      errorRate: 0.8,
      successRate: 99.2,
      trend: 'up'
    },
    {
      name: 'legacy_log_write',
      calls: 12890,
      averageTime: 23.5,
      lastCallTime: new Date(Date.now() - 15000),
      errorRate: 0.3,
      successRate: 99.7,
      trend: 'stable'
    },
    {
      name: 'legacy_report_generate',
      calls: 892,
      averageTime: 2340.6,
      lastCallTime: new Date(Date.now() - 300000),
      errorRate: 8.5,
      successRate: 91.5,
      trend: 'down'
    }
  ];

  const systemHealth: SystemHealth = {
    status: 'healthy',
    uptime: 432000, // 5 days in seconds
    version: '2.4.1',
    environment: 'production',
    lastUpdate: new Date(),
    services: [
      { name: 'API Gateway', status: 'running', health: 98 },
      { name: 'Legacy Converter', status: 'running', health: 95 },
      { name: 'Cache Service', status: 'running', health: 100 },
      { name: 'Database Pool', status: 'running', health: 92 },
      { name: 'Message Queue', status: 'running', health: 88 }
    ]
  };

  return { buildStatus, performanceMetrics, legacyFunctions, systemHealth };
}

export default function MonitoringPage() {
  const [data, setData] = useState(generateMockData());

  // Update data every 5 seconds to simulate real-time monitoring
  useEffect(() => {
    const interval = setInterval(() => {
      setData(generateMockData());
    }, 5000);

    return () => clearInterval(interval);
  }, []);

  return (
    <MainLayout>
      <MonitoringDashboard
        buildStatus={data.buildStatus}
        performanceMetrics={data.performanceMetrics}
        legacyFunctions={data.legacyFunctions}
        systemHealth={data.systemHealth}
      />
    </MainLayout>
  );
}