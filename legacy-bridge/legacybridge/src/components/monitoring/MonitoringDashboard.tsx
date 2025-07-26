'use client';

import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { cn } from '@/lib/utils';
import { BuildProgressRing } from './BuildProgressRing';
import { PerformanceChart } from './PerformanceChart';
import { FunctionCallMatrix } from './FunctionCallMatrix';
import { SystemHealthCard } from './SystemHealthCard';
import { LogStreamViewer } from './LogStreamViewer';
import ErrorLogViewer from '../ErrorLogViewer';
import { 
  Activity, 
  BarChart3, 
  Database, 
  FileCode2, 
  Gauge, 
  Server,
  Maximize2,
  Minimize2
} from 'lucide-react';

export interface BuildStatus {
  status: 'idle' | 'building' | 'success' | 'error';
  progress: number;
  currentFile?: string;
  totalFiles: number;
  completedFiles: number;
  startTime?: Date;
  estimatedTime?: number;
  errors: string[];
  warnings: string[];
}

export interface PerformanceMetrics {
  conversionsPerSecond: number;
  memoryUsage: number;
  cpuUsage: number;
  activeConnections: number;
  averageResponseTime: number;
  throughput: number;
  history: {
    timestamp: Date;
    conversionsPerSecond: number;
    memoryUsage: number;
    cpuUsage: number;
  }[];
}

export interface LegacyFunctionStats {
  name: string;
  calls: number;
  averageTime: number;
  lastCallTime: Date;
  errorRate: number;
  successRate: number;
  trend: 'up' | 'down' | 'stable';
}

export interface SystemHealth {
  status: 'healthy' | 'warning' | 'critical';
  uptime: number;
  version: string;
  environment: string;
  lastUpdate: Date;
  services: {
    name: string;
    status: 'running' | 'stopped' | 'error';
    health: number;
  }[];
}

interface MonitoringDashboardProps {
  buildStatus?: BuildStatus;
  performanceMetrics?: PerformanceMetrics;
  legacyFunctions?: LegacyFunctionStats[];
  systemHealth?: SystemHealth;
}

export function MonitoringDashboard({
  buildStatus = {
    status: 'idle',
    progress: 0,
    totalFiles: 0,
    completedFiles: 0,
    errors: [],
    warnings: []
  },
  performanceMetrics = {
    conversionsPerSecond: 0,
    memoryUsage: 0,
    cpuUsage: 0,
    activeConnections: 0,
    averageResponseTime: 0,
    throughput: 0,
    history: []
  },
  legacyFunctions = [],
  systemHealth = {
    status: 'healthy',
    uptime: 0,
    version: '1.0.0',
    environment: 'production',
    lastUpdate: new Date(),
    services: []
  }
}: MonitoringDashboardProps) {
  const [selectedMetric, setSelectedMetric] = useState<'performance' | 'functions' | 'logs' | 'errors'>('performance');
  const [isFullscreen, setIsFullscreen] = useState(false);

  // Auto-refresh data
  useEffect(() => {
    const interval = setInterval(() => {
      // In real implementation, this would fetch new data
      // For now, we'll just trigger a re-render
    }, 5000);

    return () => clearInterval(interval);
  }, []);

  return (
    <motion.div
      className={cn(
        "monitoring-dashboard p-6 space-y-6",
        isFullscreen && "fixed inset-0 z-50 bg-background overflow-auto"
      )}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.5 }}
    >
      {/* Dashboard Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-3xl font-bold text-gradient">LegacyBridge Monitor</h2>
          <p className="text-muted-foreground">Real-time system monitoring and analytics</p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setIsFullscreen(!isFullscreen)}
            className="gap-2"
          >
            {isFullscreen ? <Minimize2 className="w-4 h-4" /> : <Maximize2 className="w-4 h-4" />}
            {isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}
          </Button>
        </div>
      </div>

      {/* Status Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <motion.div whileHover={{ scale: 1.02 }} className="hover-lift">
          <Card className="glass-panel p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Conversions/sec</p>
                <p className="text-2xl font-bold">{performanceMetrics.conversionsPerSecond.toFixed(1)}</p>
              </div>
              <div className="w-12 h-12 rounded-full gradient-primary flex items-center justify-center">
                <Gauge className="w-6 h-6 text-white" />
              </div>
            </div>
          </Card>
        </motion.div>

        <motion.div whileHover={{ scale: 1.02 }} className="hover-lift">
          <Card className="glass-panel p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Memory Usage</p>
                <p className="text-2xl font-bold">{performanceMetrics.memoryUsage}%</p>
              </div>
              <div className="w-12 h-12 rounded-full gradient-info flex items-center justify-center">
                <Database className="w-6 h-6 text-white" />
              </div>
            </div>
          </Card>
        </motion.div>

        <motion.div whileHover={{ scale: 1.02 }} className="hover-lift">
          <Card className="glass-panel p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Active Builds</p>
                <p className="text-2xl font-bold">{buildStatus.status === 'building' ? 1 : 0}</p>
              </div>
              <div className="w-12 h-12 rounded-full gradient-warning flex items-center justify-center">
                <FileCode2 className="w-6 h-6 text-white" />
              </div>
            </div>
          </Card>
        </motion.div>

        <motion.div whileHover={{ scale: 1.02 }} className="hover-lift">
          <Card className="glass-panel p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">System Health</p>
                <p className="text-2xl font-bold capitalize">{systemHealth.status}</p>
              </div>
              <div className={cn(
                "w-12 h-12 rounded-full flex items-center justify-center",
                systemHealth.status === 'healthy' && "gradient-success",
                systemHealth.status === 'warning' && "gradient-warning",
                systemHealth.status === 'critical' && "gradient-error"
              )}>
                <Activity className="w-6 h-6 text-white" />
              </div>
            </div>
          </Card>
        </motion.div>
      </div>

      {/* Main Content Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Build Progress */}
        <div className="lg:col-span-1">
          <Card className="glass-panel h-full">
            <CardHeader>
              <CardTitle className="text-lg flex items-center gap-2">
                <FileCode2 className="w-5 h-5" />
                DLL Compilation
              </CardTitle>
            </CardHeader>
            <CardContent>
              <BuildProgressRing buildStatus={buildStatus} />
            </CardContent>
          </Card>
        </div>

        {/* Performance Metrics */}
        <div className="lg:col-span-2">
          <Card className="glass-panel h-full">
            <CardHeader>
              <CardTitle className="text-lg flex items-center gap-2">
                <BarChart3 className="w-5 h-5" />
                Performance Metrics
              </CardTitle>
            </CardHeader>
            <CardContent>
              <PerformanceChart metrics={performanceMetrics} />
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Tabbed Content Area */}
      <Card className="glass-panel">
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg">Detailed Analytics</CardTitle>
            <Tabs value={selectedMetric} onValueChange={(v) => setSelectedMetric(v as any)}>
              <TabsList>
                <TabsTrigger value="performance">Performance</TabsTrigger>
                <TabsTrigger value="functions">Functions</TabsTrigger>
                <TabsTrigger value="logs">Logs</TabsTrigger>
                <TabsTrigger value="errors">Error Logs</TabsTrigger>
              </TabsList>
            </Tabs>
          </div>
        </CardHeader>
        <CardContent>
          <AnimatePresence mode="wait">
            {selectedMetric === 'performance' && (
              <motion.div
                key="performance"
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -20 }}
              >
                <PerformanceChart metrics={performanceMetrics} detailed />
              </motion.div>
            )}
            {selectedMetric === 'functions' && (
              <motion.div
                key="functions"
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -20 }}
              >
                <FunctionCallMatrix functions={legacyFunctions} />
              </motion.div>
            )}
            {selectedMetric === 'logs' && (
              <motion.div
                key="logs"
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -20 }}
              >
                <LogStreamViewer />
              </motion.div>
            )}
            {selectedMetric === 'errors' && (
              <motion.div
                key="errors"
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -20 }}
              >
                <ErrorLogViewer embedded maxHeight="600px" />
              </motion.div>
            )}
          </AnimatePresence>
        </CardContent>
      </Card>

      {/* System Health Overview */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <SystemHealthCard systemHealth={systemHealth} />
      </div>
    </motion.div>
  );
}