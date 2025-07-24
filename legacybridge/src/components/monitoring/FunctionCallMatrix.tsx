'use client';

import React, { useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { LegacyFunctionStats } from '@/types/monitoring';
import { Tooltip } from '@/components/ui/tooltip';

interface FunctionCallMatrixProps {
  functions: LegacyFunctionStats[];
}

export const FunctionCallMatrix: React.FC<FunctionCallMatrixProps> = ({ functions }) => {
  const maxCalls = useMemo(() => {
    return Math.max(...functions.map(f => f.callCount), 1);
  }, [functions]);

  const getHeatmapColor = (intensity: number, status: LegacyFunctionStats['status']) => {
    if (status === 'error') {
      return `rgba(239, 68, 68, ${0.3 + intensity * 0.7})`; // Red for errors
    }
    
    // Green gradient for normal operation
    const r = Math.floor(16 + (1 - intensity) * 100);
    const g = Math.floor(185 + intensity * 42);
    const b = Math.floor(129 - intensity * 40);
    return `rgba(${r}, ${g}, ${b}, ${0.8 + intensity * 0.2})`;
  };

  const formatFunctionName = (name: string) => {
    return name
      .replace('legacybridge_', '')
      .split('_')
      .map(word => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');
  };

  const getStatusIcon = (status: LegacyFunctionStats['status']) => {
    switch (status) {
      case 'active':
        return (
          <motion.div
            className="w-2 h-2 bg-green-400 rounded-full"
            animate={{ scale: [1, 1.2, 1] }}
            transition={{ duration: 2, repeat: Infinity }}
          />
        );
      case 'error':
        return (
          <motion.div
            className="w-2 h-2 bg-red-400 rounded-full"
            animate={{ opacity: [1, 0.5, 1] }}
            transition={{ duration: 1, repeat: Infinity }}
          />
        );
      default:
        return <div className="w-2 h-2 bg-gray-400 rounded-full" />;
    }
  };

  return (
    <div className="w-full">
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-3">
        <AnimatePresence mode="popLayout">
          {functions.map((func, index) => {
            const intensity = func.callCount / maxCalls;
            const heatColor = getHeatmapColor(intensity, func.status);
            
            return (
              <motion.div
                key={func.functionName}
                layout
                initial={{ opacity: 0, scale: 0.8, y: 20 }}
                animate={{ opacity: 1, scale: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.8 }}
                transition={{ 
                  delay: index * 0.05,
                  type: "spring",
                  stiffness: 300,
                  damping: 25
                }}
                whileHover={{ 
                  scale: 1.05,
                  zIndex: 10,
                  transition: { duration: 0.2 }
                }}
                className="relative group"
              >
                <div
                  className="relative p-4 rounded-lg cursor-pointer overflow-hidden backdrop-blur-sm"
                  style={{ 
                    backgroundColor: heatColor,
                    boxShadow: `0 4px 20px ${heatColor}40`
                  }}
                >
                  {/* Animated background pattern */}
                  <div className="absolute inset-0 opacity-10">
                    <div className="absolute inset-0 bg-gradient-to-br from-white to-transparent" />
                  </div>

                  {/* Status indicator */}
                  <div className="absolute top-2 right-2">
                    {getStatusIcon(func.status)}
                  </div>

                  {/* Content */}
                  <div className="relative z-10">
                    <div className="text-xs font-medium text-slate-900 dark:text-slate-100 mb-1 line-clamp-2">
                      {formatFunctionName(func.functionName)}
                    </div>
                    <div className="text-2xl font-bold text-slate-900 dark:text-slate-100">
                      {func.callCount.toLocaleString()}
                    </div>
                    <div className="text-xs text-slate-700 dark:text-slate-300 mt-1">
                      {func.averageResponseTime.toFixed(1)}ms avg
                    </div>
                    {func.errorRate > 0 && (
                      <div className="text-xs text-red-600 dark:text-red-400 font-medium mt-1">
                        {func.errorRate.toFixed(1)}% errors
                      </div>
                    )}
                  </div>

                  {/* Hover tooltip */}
                  <div className="absolute inset-x-0 -bottom-20 left-1/2 transform -translate-x-1/2 mb-2 px-3 py-2 bg-slate-900 dark:bg-slate-100 text-white dark:text-slate-900 text-xs rounded-lg opacity-0 group-hover:opacity-100 transition-opacity z-20 pointer-events-none whitespace-nowrap">
                    <div className="font-semibold">{func.functionName}</div>
                    <div>Total Calls: {func.callCount.toLocaleString()}</div>
                    <div>Avg Response: {func.averageResponseTime.toFixed(2)}ms</div>
                    <div>Error Rate: {func.errorRate.toFixed(2)}%</div>
                    <div>Peak Usage: {func.peakUsage} calls/min</div>
                    <div>Last Called: {new Date(func.lastCalled).toLocaleTimeString()}</div>
                    
                    {/* Tooltip arrow */}
                    <div className="absolute -top-2 left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-b-4 border-transparent border-b-slate-900 dark:border-b-slate-100" />
                  </div>
                </div>

                {/* Pulse effect for active functions */}
                {func.status === 'active' && (
                  <motion.div
                    className="absolute inset-0 rounded-lg"
                    style={{ backgroundColor: heatColor }}
                    animate={{
                      scale: [1, 1.1, 1],
                      opacity: [0.5, 0, 0.5]
                    }}
                    transition={{
                      duration: 2,
                      repeat: Infinity,
                      ease: "easeInOut"
                    }}
                  />
                )}
              </motion.div>
            );
          })}
        </AnimatePresence>
      </div>

      {/* Legend */}
      <div className="mt-6 flex items-center justify-center space-x-6 text-xs">
        <div className="flex items-center space-x-2">
          <div className="w-4 h-4 rounded" style={{ backgroundColor: 'rgba(16, 185, 129, 0.3)' }} />
          <span className="text-slate-600 dark:text-slate-400">Low Activity</span>
        </div>
        <div className="flex items-center space-x-2">
          <div className="w-4 h-4 rounded" style={{ backgroundColor: 'rgba(16, 185, 129, 0.6)' }} />
          <span className="text-slate-600 dark:text-slate-400">Medium Activity</span>
        </div>
        <div className="flex items-center space-x-2">
          <div className="w-4 h-4 rounded" style={{ backgroundColor: 'rgba(16, 185, 129, 1)' }} />
          <span className="text-slate-600 dark:text-slate-400">High Activity</span>
        </div>
        <div className="flex items-center space-x-2">
          <div className="w-4 h-4 rounded" style={{ backgroundColor: 'rgba(239, 68, 68, 0.8)' }} />
          <span className="text-slate-600 dark:text-slate-400">Error State</span>
        </div>
      </div>
    </div>
  );
};