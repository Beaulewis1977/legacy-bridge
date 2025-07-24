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
  Legend
} from 'recharts';
import { motion } from 'framer-motion';
import { PerformanceDataPoint } from '@/types/monitoring';

interface PerformanceChartProps {
  data: PerformanceDataPoint[];
  type?: 'line' | 'area';
}

export const PerformanceChart: React.FC<PerformanceChartProps> = ({ 
  data, 
  type = 'area' 
}) => {
  const chartData = data.slice(-30); // Show last 30 data points

  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      return (
        <div className="bg-white dark:bg-slate-800 p-3 rounded-lg shadow-lg border border-slate-200 dark:border-slate-700">
          <p className="text-sm font-medium text-slate-900 dark:text-slate-100">
            {new Date(label).toLocaleTimeString()}
          </p>
          {payload.map((entry: any, index: number) => (
            <p key={index} className="text-sm" style={{ color: entry.color }}>
              {entry.name}: {entry.value.toFixed(2)}
              {entry.name === 'Memory' ? ' MB' : 
               entry.name === 'CPU' ? '%' : 
               ' req/s'}
            </p>
          ))}
        </div>
      );
    }
    return null;
  };

  const ChartComponent = type === 'area' ? AreaChart : LineChart;
  const DataComponent = type === 'area' ? Area : Line;

  return (
    <motion.div
      className="w-full h-full"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
    >
      <ResponsiveContainer width="100%" height="100%">
        <ChartComponent
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
              second: '2-digit'
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
          
          <Legend 
            wrapperStyle={{ paddingTop: '20px' }}
            iconType="line"
          />
          
          <DataComponent
            type="monotone"
            dataKey="conversionsPerSecond"
            stroke="#3b82f6"
            strokeWidth={2}
            fill={type === 'area' ? "url(#colorConversions)" : undefined}
            name="Conversions/sec"
            dot={false}
            animationDuration={500}
          />
          
          <DataComponent
            type="monotone"
            dataKey="memoryUsage"
            stroke="#10b981"
            strokeWidth={2}
            fill={type === 'area' ? "url(#colorMemory)" : undefined}
            name="Memory"
            dot={false}
            animationDuration={500}
          />
          
          <DataComponent
            type="monotone"
            dataKey="cpuUsage"
            stroke="#f59e0b"
            strokeWidth={2}
            fill={type === 'area' ? "url(#colorCPU)" : undefined}
            name="CPU"
            dot={false}
            animationDuration={500}
          />
        </ChartComponent>
      </ResponsiveContainer>
    </motion.div>
  );
};

interface MetricCardProps {
  title: string;
  value: number | string;
  unit?: string;
  trend?: 'up' | 'down' | 'stable';
  color?: string;
}

export const MetricCard: React.FC<MetricCardProps> = ({
  title,
  value,
  unit,
  trend,
  color = '#3b82f6'
}) => {
  const getTrendIcon = () => {
    switch (trend) {
      case 'up':
        return (
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 10l7-7m0 0l7 7m-7-7v18" />
          </svg>
        );
      case 'down':
        return (
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
          </svg>
        );
      default:
        return (
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14" />
          </svg>
        );
    }
  };

  const getTrendColor = () => {
    switch (trend) {
      case 'up': return 'text-green-500';
      case 'down': return 'text-red-500';
      default: return 'text-gray-500';
    }
  };

  return (
    <motion.div
      className="bg-white dark:bg-slate-800 rounded-xl p-6 shadow-sm border border-slate-200 dark:border-slate-700"
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.3 }}
      whileHover={{ scale: 1.02 }}
    >
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-medium text-slate-600 dark:text-slate-400">
          {title}
        </h3>
        {trend && (
          <div className={`${getTrendColor()}`}>
            {getTrendIcon()}
          </div>
        )}
      </div>
      <div className="flex items-baseline">
        <span 
          className="text-3xl font-bold"
          style={{ color }}
        >
          {typeof value === 'number' ? value.toFixed(1) : value}
        </span>
        {unit && (
          <span className="ml-2 text-sm text-slate-500 dark:text-slate-400">
            {unit}
          </span>
        )}
      </div>
    </motion.div>
  );
};