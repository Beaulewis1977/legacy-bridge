'use client';

import React, { useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { LegacyFunctionStats } from './MonitoringDashboard';
import { 
  Activity, 
  Zap, 
  AlertCircle, 
  TrendingUp, 
  TrendingDown,
  Clock,
  BarChart3
} from 'lucide-react';
import { cn } from '@/lib/utils';

interface FunctionCallMatrixProps {
  functions: LegacyFunctionStats[];
}

export const FunctionCallMatrix: React.FC<FunctionCallMatrixProps> = ({ functions }) => {
  const maxCalls = useMemo(() => {
    return Math.max(...functions.map(f => f.calls), 1);
  }, [functions]);

  const getHeatmapColor = (intensity: number, errorRate: number) => {
    if (errorRate > 50) {
      return {
        bg: 'from-red-500/20 to-red-600/30',
        border: 'border-red-500/50',
        glow: 'hover:shadow-red-500/20'
      };
    }
    
    if (intensity > 0.7) {
      return {
        bg: 'from-emerald-500/20 to-emerald-600/30',
        border: 'border-emerald-500/50',
        glow: 'hover:shadow-emerald-500/20'
      };
    } else if (intensity > 0.3) {
      return {
        bg: 'from-blue-500/20 to-blue-600/30',
        border: 'border-blue-500/50',
        glow: 'hover:shadow-blue-500/20'
      };
    } else {
      return {
        bg: 'from-slate-500/10 to-slate-600/20',
        border: 'border-slate-500/30',
        glow: 'hover:shadow-slate-500/10'
      };
    }
  };

  const formatFunctionName = (name: string) => {
    return name
      .replace(/^legacy_/, '')
      .split('_')
      .map(word => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');
  };

  const getTrendIcon = (trend: 'up' | 'down' | 'stable') => {
    switch (trend) {
      case 'up':
        return <TrendingUp className="w-3.5 h-3.5 text-emerald-500" />;
      case 'down':
        return <TrendingDown className="w-3.5 h-3.5 text-red-500" />;
      default:
        return <Activity className="w-3.5 h-3.5 text-gray-500" />;
    }
  };

  return (
    <div className="w-full">
      {/* Summary Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        <motion.div 
          className="glass-panel p-4 rounded-xl"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
        >
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-muted-foreground">Total Functions</p>
              <p className="text-2xl font-bold">{functions.length}</p>
            </div>
            <BarChart3 className="w-8 h-8 text-legacy-blue-500 opacity-50" />
          </div>
        </motion.div>

        <motion.div 
          className="glass-panel p-4 rounded-xl"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
        >
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-muted-foreground">Total Calls</p>
              <p className="text-2xl font-bold">
                {functions.reduce((sum, f) => sum + f.calls, 0).toLocaleString()}
              </p>
            </div>
            <Zap className="w-8 h-8 text-legacy-amber-500 opacity-50" />
          </div>
        </motion.div>

        <motion.div 
          className="glass-panel p-4 rounded-xl"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
        >
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-muted-foreground">Avg Response</p>
              <p className="text-2xl font-bold">
                {(functions.reduce((sum, f) => sum + f.averageTime, 0) / functions.length).toFixed(1)}ms
              </p>
            </div>
            <Clock className="w-8 h-8 text-legacy-emerald-500 opacity-50" />
          </div>
        </motion.div>

        <motion.div 
          className="glass-panel p-4 rounded-xl"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
        >
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-muted-foreground">Error Rate</p>
              <p className="text-2xl font-bold">
                {(functions.reduce((sum, f) => sum + f.errorRate, 0) / functions.length).toFixed(1)}%
              </p>
            </div>
            <AlertCircle className="w-8 h-8 text-red-500 opacity-50" />
          </div>
        </motion.div>
      </div>

      {/* Function Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        <AnimatePresence mode="popLayout">
          {functions.map((func, index) => {
            const intensity = func.calls / maxCalls;
            const colors = getHeatmapColor(intensity, func.errorRate);
            
            return (
              <motion.div
                key={func.name}
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
                  scale: 1.03,
                  transition: { duration: 0.2 }
                }}
                className="group"
              >
                <div className={cn(
                  "relative p-5 rounded-xl glass-morphism border transition-all duration-300",
                  "bg-gradient-to-br",
                  colors.bg,
                  colors.border,
                  colors.glow,
                  "hover:shadow-xl cursor-pointer overflow-hidden"
                )}>
                  {/* Animated background effect */}
                  <div className="absolute inset-0 opacity-30">
                    <div className="absolute inset-0 bg-gradient-to-br from-white/10 to-transparent" />
                  </div>

                  {/* Content */}
                  <div className="relative z-10">
                    {/* Header */}
                    <div className="flex items-start justify-between mb-3">
                      <div className="flex-1">
                        <h4 className="font-semibold text-sm line-clamp-1">
                          {formatFunctionName(func.name)}
                        </h4>
                        <p className="text-xs text-muted-foreground mt-0.5">
                          Last: {new Date(func.lastCallTime).toLocaleTimeString()}
                        </p>
                      </div>
                      <div className="ml-2">
                        {getTrendIcon(func.trend)}
                      </div>
                    </div>

                    {/* Metrics */}
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <span className="text-xs text-muted-foreground">Calls</span>
                        <span className="text-sm font-bold">
                          {func.calls.toLocaleString()}
                        </span>
                      </div>
                      
                      <div className="flex items-center justify-between">
                        <span className="text-xs text-muted-foreground">Avg Time</span>
                        <span className="text-sm font-medium">
                          {func.averageTime.toFixed(1)}ms
                        </span>
                      </div>

                      {/* Progress bars */}
                      <div className="space-y-1.5 mt-3">
                        <div>
                          <div className="flex justify-between text-xs mb-1">
                            <span className="text-muted-foreground">Success Rate</span>
                            <span className="font-medium text-emerald-600 dark:text-emerald-400">
                              {func.successRate.toFixed(1)}%
                            </span>
                          </div>
                          <div className="w-full h-1.5 bg-slate-200 dark:bg-slate-700 rounded-full overflow-hidden">
                            <motion.div 
                              className="h-full bg-gradient-to-r from-emerald-400 to-emerald-500"
                              initial={{ width: 0 }}
                              animate={{ width: `${func.successRate}%` }}
                              transition={{ duration: 1, delay: index * 0.05 }}
                            />
                          </div>
                        </div>

                        {func.errorRate > 0 && (
                          <div>
                            <div className="flex justify-between text-xs mb-1">
                              <span className="text-muted-foreground">Error Rate</span>
                              <span className="font-medium text-red-600 dark:text-red-400">
                                {func.errorRate.toFixed(1)}%
                              </span>
                            </div>
                            <div className="w-full h-1.5 bg-slate-200 dark:bg-slate-700 rounded-full overflow-hidden">
                              <motion.div 
                                className="h-full bg-gradient-to-r from-red-400 to-red-500"
                                initial={{ width: 0 }}
                                animate={{ width: `${func.errorRate}%` }}
                                transition={{ duration: 1, delay: index * 0.05 + 0.1 }}
                              />
                            </div>
                          </div>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* Activity indicator */}
                  <motion.div
                    className={cn(
                      "absolute top-3 right-3 w-2 h-2 rounded-full",
                      func.calls > 0 ? "bg-emerald-500" : "bg-gray-400"
                    )}
                    animate={func.calls > 0 ? {
                      scale: [1, 1.5, 1],
                      opacity: [1, 0.5, 1]
                    } : {}}
                    transition={{
                      duration: 2,
                      repeat: Infinity,
                      ease: "easeInOut"
                    }}
                  />
                </div>
              </motion.div>
            );
          })}
        </AnimatePresence>
      </div>
    </div>
  );
};