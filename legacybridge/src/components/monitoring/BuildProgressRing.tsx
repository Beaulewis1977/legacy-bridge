'use client';

import React from 'react';
import { motion } from 'framer-motion';
import { BuildStatus } from '@/types/monitoring';

interface BuildProgressRingProps {
  progress: number;
  status: BuildStatus['compilation'];
  timeElapsed: number;
  estimatedTimeRemaining: number;
  currentStep: string;
}

export const BuildProgressRing: React.FC<BuildProgressRingProps> = ({
  progress,
  status,
  timeElapsed,
  estimatedTimeRemaining,
  currentStep
}) => {
  const getStatusColor = (status: BuildStatus['compilation']) => {
    switch (status) {
      case 'building': return 'rgb(168, 85, 247)'; // Purple
      case 'success': return 'rgb(34, 197, 94)'; // Green
      case 'failed': return 'rgb(239, 68, 68)'; // Red
      default: return 'rgb(148, 163, 184)'; // Gray
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
            <motion.div
              className="text-4xl font-bold text-slate-900 dark:text-slate-100"
              key={progress}
              initial={{ scale: 1.2 }}
              animate={{ scale: 1 }}
              transition={{ duration: 0.2 }}
            >
              {Math.round(progress)}%
            </motion.div>
            <div className="text-sm font-medium text-slate-600 dark:text-slate-400 capitalize">
              {status}
            </div>
          </div>
        </div>
      </div>

      {/* Build details */}
      <div className="mt-6 space-y-2">
        <div className="flex justify-between text-sm">
          <span className="text-slate-600 dark:text-slate-400">Current Step:</span>
          <span className="font-medium text-slate-900 dark:text-slate-100">
            {currentStep}
          </span>
        </div>
        <div className="flex justify-between text-sm">
          <span className="text-slate-600 dark:text-slate-400">Time Elapsed:</span>
          <span className="font-medium text-slate-900 dark:text-slate-100">
            {formatTime(timeElapsed)}
          </span>
        </div>
        {status === 'building' && (
          <div className="flex justify-between text-sm">
            <span className="text-slate-600 dark:text-slate-400">Est. Remaining:</span>
            <span className="font-medium text-slate-900 dark:text-slate-100">
              {formatTime(estimatedTimeRemaining)}
            </span>
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