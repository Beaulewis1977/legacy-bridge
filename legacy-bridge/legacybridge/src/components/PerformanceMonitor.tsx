'use client';

import React, { useState, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Activity, Clock, Zap, TrendingUp, AlertTriangle, CheckCircle2 } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { cn } from '@/lib/utils';

interface PerformanceMetrics {
  conversionTime: number;
  memoryUsage: number;
  cpuUsage: number;
  throughput: number;
  errorRate: number;
  timestamp: number;
}

interface PerformanceMonitorProps {
  isVisible?: boolean;
  onToggle?: () => void;
  className?: string;
}

export const PerformanceMonitor: React.FC<PerformanceMonitorProps> = ({
  isVisible = false,
  onToggle,
  className
}) => {
  const [metrics, setMetrics] = useState<PerformanceMetrics[]>([]);
  const [currentMetrics, setCurrentMetrics] = useState<PerformanceMetrics | null>(null);
  const [isMonitoring, setIsMonitoring] = useState(false);

  // Simulate performance monitoring
  const collectMetrics = useCallback((): PerformanceMetrics => {
    // In a real implementation, these would come from actual performance APIs
    const performance = window.performance;
    const memory = (performance as any).memory;
    
    return {
      conversionTime: Math.random() * 2000 + 500, // 500-2500ms
      memoryUsage: memory ? (memory.usedJSHeapSize / memory.totalJSHeapSize) * 100 : Math.random() * 60 + 20,
      cpuUsage: Math.random() * 40 + 10, // 10-50%
      throughput: Math.random() * 100 + 50, // 50-150 files/min
      errorRate: Math.random() * 5, // 0-5%
      timestamp: Date.now()
    };
  }, []);

  // Start/stop monitoring
  useEffect(() => {
    let interval: NodeJS.Timeout;
    
    if (isMonitoring) {
      interval = setInterval(() => {
        const newMetrics = collectMetrics();
        setCurrentMetrics(newMetrics);
        setMetrics(prev => [...prev.slice(-19), newMetrics]); // Keep last 20 metrics
      }, 1000);
    }

    return () => {
      if (interval) clearInterval(interval);
    };
  }, [isMonitoring, collectMetrics]);

  // Auto-start monitoring when visible
  useEffect(() => {
    if (isVisible) {
      setIsMonitoring(true);
    } else {
      setIsMonitoring(false);
    }
  }, [isVisible]);

  const getStatusColor = (value: number, thresholds: { good: number; warning: number }) => {
    if (value <= thresholds.good) return 'text-green-500';
    if (value <= thresholds.warning) return 'text-yellow-500';
    return 'text-red-500';
  };

  const getStatusIcon = (value: number, thresholds: { good: number; warning: number }) => {
    if (value <= thresholds.good) return <CheckCircle2 className="w-4 h-4 text-green-500" />;
    if (value <= thresholds.warning) return <AlertTriangle className="w-4 h-4 text-yellow-500" />;
    return <AlertTriangle className="w-4 h-4 text-red-500" />;
  };

  const averageMetrics = metrics.length > 0 ? {
    conversionTime: metrics.reduce((sum, m) => sum + m.conversionTime, 0) / metrics.length,
    memoryUsage: metrics.reduce((sum, m) => sum + m.memoryUsage, 0) / metrics.length,
    cpuUsage: metrics.reduce((sum, m) => sum + m.cpuUsage, 0) / metrics.length,
    throughput: metrics.reduce((sum, m) => sum + m.throughput, 0) / metrics.length,
    errorRate: metrics.reduce((sum, m) => sum + m.errorRate, 0) / metrics.length
  } : null;

  if (!isVisible) {
    return (
      <motion.div
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        className="fixed bottom-4 right-4 z-50"
      >
        <Button
          onClick={onToggle}
          size="sm"
          variant="outline"
          className="gap-2 bg-background/80 backdrop-blur-sm"
        >
          <Activity className="w-4 h-4" />
          Performance
        </Button>
      </motion.div>
    );
  }

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: 20 }}
        className={cn('fixed bottom-4 right-4 z-50 w-80', className)}
      >
        <Card className="p-4 bg-background/95 backdrop-blur-sm border shadow-lg">
          {/* Header */}
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-2">
              <motion.div
                animate={{ rotate: isMonitoring ? 360 : 0 }}
                transition={{ duration: 2, repeat: isMonitoring ? Infinity : 0, ease: "linear" }}
              >
                <Activity className="w-5 h-5 text-primary" />
              </motion.div>
              <h3 className="font-semibold text-sm">Performance Monitor</h3>
            </div>
            <div className="flex items-center gap-2">
              <Badge variant={isMonitoring ? "default" : "secondary"} className="text-xs">
                {isMonitoring ? "Live" : "Paused"}
              </Badge>
              <Button
                onClick={onToggle}
                size="sm"
                variant="ghost"
                className="h-6 w-6 p-0"
              >
                ×
              </Button>
            </div>
          </div>

          {/* Current Metrics */}
          {currentMetrics && (
            <div className="space-y-3 mb-4">
              {/* Conversion Time */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Clock className="w-4 h-4 text-muted-foreground" />
                  <span className="text-sm">Conversion Time</span>
                </div>
                <div className="flex items-center gap-2">
                  {getStatusIcon(currentMetrics.conversionTime, { good: 1000, warning: 2000 })}
                  <span className={cn("text-sm font-mono", 
                    getStatusColor(currentMetrics.conversionTime, { good: 1000, warning: 2000 })
                  )}>
                    {currentMetrics.conversionTime.toFixed(0)}ms
                  </span>
                </div>
              </div>

              {/* Memory Usage */}
              <div className="space-y-1">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Zap className="w-4 h-4 text-muted-foreground" />
                    <span className="text-sm">Memory Usage</span>
                  </div>
                  <span className={cn("text-sm font-mono",
                    getStatusColor(currentMetrics.memoryUsage, { good: 50, warning: 80 })
                  )}>
                    {currentMetrics.memoryUsage.toFixed(1)}%
                  </span>
                </div>
                <Progress 
                  value={currentMetrics.memoryUsage} 
                  className="h-2"
                />
              </div>

              {/* CPU Usage */}
              <div className="space-y-1">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <TrendingUp className="w-4 h-4 text-muted-foreground" />
                    <span className="text-sm">CPU Usage</span>
                  </div>
                  <span className={cn("text-sm font-mono",
                    getStatusColor(currentMetrics.cpuUsage, { good: 30, warning: 60 })
                  )}>
                    {currentMetrics.cpuUsage.toFixed(1)}%
                  </span>
                </div>
                <Progress 
                  value={currentMetrics.cpuUsage} 
                  className="h-2"
                />
              </div>

              {/* Throughput */}
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Throughput</span>
                <span className="text-sm font-mono text-green-600">
                  {currentMetrics.throughput.toFixed(0)} files/min
                </span>
              </div>

              {/* Error Rate */}
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Error Rate</span>
                <span className={cn("text-sm font-mono",
                  getStatusColor(currentMetrics.errorRate, { good: 1, warning: 3 })
                )}>
                  {currentMetrics.errorRate.toFixed(1)}%
                </span>
              </div>
            </div>
          )}

          {/* Average Metrics */}
          {averageMetrics && metrics.length > 5 && (
            <div className="pt-3 border-t">
              <h4 className="text-xs font-medium text-muted-foreground mb-2">
                Average (last {metrics.length} samples)
              </h4>
              <div className="grid grid-cols-2 gap-2 text-xs">
                <div>
                  <span className="text-muted-foreground">Conversion:</span>
                  <span className="ml-1 font-mono">
                    {averageMetrics.conversionTime.toFixed(0)}ms
                  </span>
                </div>
                <div>
                  <span className="text-muted-foreground">Memory:</span>
                  <span className="ml-1 font-mono">
                    {averageMetrics.memoryUsage.toFixed(1)}%
                  </span>
                </div>
                <div>
                  <span className="text-muted-foreground">CPU:</span>
                  <span className="ml-1 font-mono">
                    {averageMetrics.cpuUsage.toFixed(1)}%
                  </span>
                </div>
                <div>
                  <span className="text-muted-foreground">Errors:</span>
                  <span className="ml-1 font-mono">
                    {averageMetrics.errorRate.toFixed(1)}%
                  </span>
                </div>
              </div>
            </div>
          )}

          {/* Mini Chart */}
          {metrics.length > 1 && (
            <div className="pt-3 border-t">
              <h4 className="text-xs font-medium text-muted-foreground mb-2">
                Conversion Time Trend
              </h4>
              <div className="h-8 flex items-end gap-1">
                {metrics.slice(-10).map((metric, index) => {
                  const height = Math.max(4, (metric.conversionTime / 3000) * 100);
                  return (
                    <motion.div
                      key={metric.timestamp}
                      initial={{ height: 0 }}
                      animate={{ height: `${Math.min(height, 100)}%` }}
                      className="flex-1 bg-primary/60 rounded-sm min-h-[4px]"
                      transition={{ delay: index * 0.05 }}
                    />
                  );
                })}
              </div>
            </div>
          )}

          {/* Controls */}
          <div className="pt-3 border-t flex justify-between items-center">
            <Button
              onClick={() => setIsMonitoring(!isMonitoring)}
              size="sm"
              variant="outline"
              className="text-xs"
            >
              {isMonitoring ? 'Pause' : 'Resume'}
            </Button>
            <Button
              onClick={() => {
                setMetrics([]);
                setCurrentMetrics(null);
              }}
              size="sm"
              variant="ghost"
              className="text-xs"
            >
              Clear
            </Button>
          </div>
        </Card>
      </motion.div>
    </AnimatePresence>
  );
};