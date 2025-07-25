'use client';

import React, { useRef, useEffect } from 'react';
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
  ReferenceLine
} from 'recharts';
import { motion } from 'framer-motion';
import { PerformanceMetrics } from './MonitoringDashboard';
import { 
  TrendingUp, 
  TrendingDown, 
  Minus,
  Activity,
  Cpu,
  Database,
  Zap
} from 'lucide-react';

interface PerformanceChartProps {
  metrics: PerformanceMetrics;
  detailed?: boolean;
}

export const PerformanceChart: React.FC<PerformanceChartProps> = ({ 
  metrics, 
  detailed = false 
}) => {
  const chartData = metrics.history.slice(-30).map(point => ({
    ...point,
    timestamp: new Date(point.timestamp).getTime()
  }));

  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      return (
        <motion.div 
          className="glass-panel p-4 rounded-lg shadow-xl"
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
        >
          <p className="text-sm font-medium text-slate-900 dark:text-slate-100 mb-2">
            {new Date(label).toLocaleTimeString()}
          </p>
          {payload.map((entry: any, index: number) => (
            <div key={index} className="flex items-center gap-2 text-sm">
              <div 
                className="w-3 h-3 rounded-full" 
                style={{ backgroundColor: entry.color }}
              />
              <span className="text-slate-600 dark:text-slate-400">
                {entry.name}:
              </span>
              <span className="font-medium" style={{ color: entry.color }}>
                {entry.value.toFixed(2)}
                {entry.name === 'Memory' ? ' MB' : 
                 entry.name === 'CPU' ? '%' : 
                 ' req/s'}
              </span>
            </div>
          ))}
        </motion.div>
      );
    }
    return null;
  };

  const chartHeight = detailed ? 400 : 300;

  return (
    <motion.div
      className="w-full"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
    >
      {detailed && (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
          <MetricCard
            title="Conversions/sec"
            value={metrics.conversionsPerSecond}
            unit="req/s"
            icon={<Zap className="w-5 h-5" />}
            color="text-legacy-blue-500"
            trend={metrics.conversionsPerSecond > 50 ? 'up' : 'stable'}
          />
          <MetricCard
            title="Memory Usage"
            value={metrics.memoryUsage}
            unit="%"
            icon={<Database className="w-5 h-5" />}
            color="text-legacy-emerald-500"
            trend={metrics.memoryUsage > 80 ? 'up' : 'stable'}
          />
          <MetricCard
            title="CPU Usage"
            value={metrics.cpuUsage}
            unit="%"
            icon={<Cpu className="w-5 h-5" />}
            color="text-legacy-amber-500"
            trend={metrics.cpuUsage > 70 ? 'up' : 'stable'}
          />
          <MetricCard
            title="Response Time"
            value={metrics.averageResponseTime}
            unit="ms"
            icon={<Activity className="w-5 h-5" />}
            color="text-purple-500"
            trend={metrics.averageResponseTime < 100 ? 'down' : 'up'}
          />
        </div>
      )}

      <div style={{ height: `${chartHeight}px` }}>
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart
            data={chartData}
            margin={{ top: 5, right: 30, left: 20, bottom: 5 }}
          >
            <defs>
              <linearGradient id="colorConversions" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.8}/>
                <stop offset="95%" stopColor="#3b82f6" stopOpacity={0.1}/>
              </linearGradient>
              <linearGradient id="colorMemory" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#10b981" stopOpacity={0.8}/>
                <stop offset="95%" stopColor="#10b981" stopOpacity={0.1}/>
              </linearGradient>
              <linearGradient id="colorCPU" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#f59e0b" stopOpacity={0.8}/>
                <stop offset="95%" stopColor="#f59e0b" stopOpacity={0.1}/>
              </linearGradient>
            </defs>
            
            <CartesianGrid 
              strokeDasharray="3 3" 
              stroke="rgba(148, 163, 184, 0.1)"
              vertical={false}
            />
            
            <XAxis
              dataKey="timestamp"
              tickFormatter={(value) => new Date(value).toLocaleTimeString('en-US', {
                hour: '2-digit',
                minute: '2-digit',
              })}
              stroke="#94a3b8"
              tick={{ fontSize: 12 }}
            />
            
            <YAxis
              stroke="#94a3b8"
              tick={{ fontSize: 12 }}
              domain={[0, 'auto']}
            />
            
            <Tooltip content={<CustomTooltip />} />
            
            {detailed && (
              <>
                <ReferenceLine 
                  y={80} 
                  stroke="#ef4444" 
                  strokeDasharray="5 5" 
                  label="Warning Threshold"
                />
                <ReferenceLine 
                  y={90} 
                  stroke="#dc2626" 
                  strokeDasharray="5 5" 
                  label="Critical Threshold"
                />
              </>
            )}
            
            <Legend 
              wrapperStyle={{ paddingTop: '20px' }}
              iconType="rect"
              formatter={(value) => (
                <span className="text-sm font-medium">{value}</span>
              )}
            />
            
            <Area
              type="monotone"
              dataKey="conversionsPerSecond"
              stroke="#3b82f6"
              strokeWidth={2}
              fill="url(#colorConversions)"
              name="Conversions/sec"
              dot={false}
              animationDuration={500}
            />
            
            <Area
              type="monotone"
              dataKey="memoryUsage"
              stroke="#10b981"
              strokeWidth={2}
              fill="url(#colorMemory)"
              name="Memory"
              dot={false}
              animationDuration={500}
            />
            
            <Area
              type="monotone"
              dataKey="cpuUsage"
              stroke="#f59e0b"
              strokeWidth={2}
              fill="url(#colorCPU)"
              name="CPU"
              dot={false}
              animationDuration={500}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </motion.div>
  );
};

interface MetricCardProps {
  title: string;
  value: number | string;
  unit?: string;
  trend?: 'up' | 'down' | 'stable';
  color?: string;
  icon?: React.ReactNode;
}

export const MetricCard: React.FC<MetricCardProps> = ({
  title,
  value,
  unit,
  trend,
  color = 'text-legacy-blue-500',
  icon
}) => {
  const getTrendIcon = () => {
    switch (trend) {
      case 'up':
        return <TrendingUp className="w-4 h-4 text-emerald-500" />;
      case 'down':
        return <TrendingDown className="w-4 h-4 text-red-500" />;
      default:
        return <Minus className="w-4 h-4 text-gray-500" />;
    }
  };

  return (
    <motion.div
      className="glass-panel p-6 rounded-xl hover-lift hover-glow"
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.3 }}
      whileHover={{ scale: 1.02 }}
    >
      <div className="flex items-center justify-between mb-3">
        <div className={`${color} opacity-80`}>
          {icon}
        </div>
        {trend && getTrendIcon()}
      </div>
      
      <h3 className="text-sm font-medium text-slate-600 dark:text-slate-400 mb-1">
        {title}
      </h3>
      
      <div className="flex items-baseline gap-1">
        <span className={`text-3xl font-bold ${color}`}>
          {typeof value === 'number' ? value.toFixed(1) : value}
        </span>
        {unit && (
          <span className="text-sm text-slate-500 dark:text-slate-400">
            {unit}
          </span>
        )}
      </div>
    </motion.div>
  );
};