'use client';

import React from 'react';
import { motion } from 'framer-motion';
import { BuildStatus } from './MonitoringDashboard';
import { 
  FileCode2, 
  Package, 
  Zap, 
  CheckCircle, 
  XCircle,
  AlertCircle,
  Loader2
} from 'lucide-react';

interface BuildProgressRingProps {
  buildStatus: BuildStatus;
}

export const BuildProgressRing: React.FC<BuildProgressRingProps> = ({
  buildStatus
}) => {
  const { status, progress, currentFile, totalFiles, completedFiles, errors, warnings } = buildStatus;

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'building': return 'rgb(139, 92, 246)'; // Purple-500
      case 'success': return 'rgb(16, 185, 129)'; // Emerald-500
      case 'error': return 'rgb(239, 68, 68)'; // Red-500
      default: return 'rgb(148, 163, 184)'; // Gray-400
    }
  };

  const getStatusIcon = () => {
    switch (status) {
      case 'building':
        return <Loader2 className="w-8 h-8 text-purple-500 animate-spin" />;
      case 'success':
        return <CheckCircle className="w-8 h-8 text-emerald-500" />;
      case 'error':
        return <XCircle className="w-8 h-8 text-red-500" />;
      default:
        return <FileCode2 className="w-8 h-8 text-gray-400" />;
    }
  };

  const radius = 56;
  const circumference = 2 * Math.PI * radius;
  const strokeDashoffset = circumference - (progress / 100) * circumference;

  return (
    <motion.div
      className="relative"
      initial={{ scale: 0, opacity: 0 }}
      animate={{ scale: 1, opacity: 1 }}
      transition={{ duration: 0.5, type: "spring" }}
    >
      <div className="relative w-48 h-48">
        <svg className="w-48 h-48 transform -rotate-90">
          {/* Background circle */}
          <circle
            cx="96"
            cy="96"
            r={radius}
            stroke="currentColor"
            strokeWidth="8"
            fill="transparent"
            className="text-slate-200 dark:text-slate-700"
          />
          {/* Progress circle */}
          <motion.circle
            cx="96"
            cy="96"
            r={radius}
            stroke={getStatusColor(status)}
            strokeWidth="8"
            fill="transparent"
            strokeLinecap="round"
            strokeDasharray={circumference}
            strokeDashoffset={strokeDashoffset}
            initial={{ strokeDashoffset: circumference }}
            animate={{ strokeDashoffset }}
            transition={{ duration: 0.5, ease: "easeOut" }}
            style={{
              filter: `drop-shadow(0 0 10px ${getStatusColor(status)}40)`
            }}
          />
        </svg>

        {/* Center content */}
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="text-center">
            {status === 'idle' || status === 'building' ? (
              <>
                <motion.div
                  className="text-4xl font-bold bg-gradient-to-r from-legacy-blue-600 to-legacy-blue-700 bg-clip-text text-transparent"
                  key={progress}
                  initial={{ scale: 1.2 }}
                  animate={{ scale: 1 }}
                  transition={{ duration: 0.2 }}
                >
                  {Math.round(progress)}%
                </motion.div>
                <div className="text-sm font-medium text-slate-600 dark:text-slate-400 capitalize mt-1">
                  {status}
                </div>
              </>
            ) : (
              <motion.div
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{ type: "spring", bounce: 0.5 }}
              >
                {getStatusIcon()}
              </motion.div>
            )}
          </div>
        </div>
      </div>

      {/* Build details */}
      <div className="mt-6 space-y-3">
        {currentFile && (
          <motion.div 
            className="p-3 rounded-lg bg-slate-100/50 dark:bg-slate-800/50 backdrop-blur-sm"
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
          >
            <div className="flex items-center gap-2 mb-1">
              <FileCode2 className="w-4 h-4 text-legacy-blue-500" />
              <span className="text-sm font-medium">Current File</span>
            </div>
            <p className="text-xs text-muted-foreground truncate">
              {currentFile}
            </p>
          </motion.div>
        )}
        
        <div className="grid grid-cols-2 gap-2">
          <div className="p-2 rounded-lg bg-slate-100/30 dark:bg-slate-800/30">
            <div className="flex items-center gap-1.5">
              <Package className="w-3.5 h-3.5 text-legacy-blue-500" />
              <span className="text-xs text-muted-foreground">Progress</span>
            </div>
            <p className="text-sm font-medium mt-1">
              {completedFiles} / {totalFiles}
            </p>
          </div>
          
          <div className="p-2 rounded-lg bg-slate-100/30 dark:bg-slate-800/30">
            <div className="flex items-center gap-1.5">
              <Zap className="w-3.5 h-3.5 text-legacy-amber-500" />
              <span className="text-xs text-muted-foreground">Status</span>
            </div>
            <p className="text-sm font-medium mt-1 capitalize">
              {status}
            </p>
          </div>
        </div>

        {/* Error and Warning indicators */}
        {(errors.length > 0 || warnings.length > 0) && (
          <div className="flex gap-2">
            {errors.length > 0 && (
              <motion.div 
                className="flex items-center gap-1.5 px-2 py-1 rounded-md bg-red-100 dark:bg-red-900/20"
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{ type: "spring" }}
              >
                <XCircle className="w-3.5 h-3.5 text-red-500" />
                <span className="text-xs font-medium text-red-700 dark:text-red-400">
                  {errors.length} error{errors.length > 1 ? 's' : ''}
                </span>
              </motion.div>
            )}
            {warnings.length > 0 && (
              <motion.div 
                className="flex items-center gap-1.5 px-2 py-1 rounded-md bg-amber-100 dark:bg-amber-900/20"
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{ type: "spring" }}
              >
                <AlertCircle className="w-3.5 h-3.5 text-amber-500" />
                <span className="text-xs font-medium text-amber-700 dark:text-amber-400">
                  {warnings.length} warning{warnings.length > 1 ? 's' : ''}
                </span>
              </motion.div>
            )}
          </div>
        )}
      </div>

      {/* Pulse animation when building */}
      {status === 'building' && (
        <motion.div
          className="absolute inset-0 rounded-full"
          style={{
            background: `radial-gradient(circle, ${getStatusColor(status)}20 0%, transparent 70%)`
          }}
          animate={{
            scale: [1, 1.2, 1],
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
};

function formatTime(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}s`;
  }
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}m ${remainingSeconds}s`;
}