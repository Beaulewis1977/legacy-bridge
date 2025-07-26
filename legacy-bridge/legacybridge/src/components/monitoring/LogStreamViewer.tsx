'use client';

import React, { useRef, useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { LogEntry } from '@/types/monitoring';

interface LogStreamViewerProps {
  logs: LogEntry[];
  maxHeight?: string;
  showFilters?: boolean;
}

export const LogStreamViewer: React.FC<LogStreamViewerProps> = ({ 
  logs, 
  maxHeight = '400px',
  showFilters = true 
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [filter, setFilter] = useState<LogEntry['level'] | 'all'>('all');
  const [autoScroll, setAutoScroll] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    if (autoScroll && containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [logs, autoScroll]);

  const getLogColor = (level: LogEntry['level']) => {
    switch (level) {
      case 'error': return 'text-red-400';
      case 'warn': return 'text-amber-400';
      case 'info': return 'text-blue-400';
      case 'debug': return 'text-slate-400';
      default: return 'text-slate-300';
    }
  };

  const getLogBgColor = (level: LogEntry['level']) => {
    switch (level) {
      case 'error': return 'bg-red-950/20';
      case 'warn': return 'bg-amber-950/20';
      case 'info': return 'bg-blue-950/20';
      case 'debug': return 'bg-slate-950/20';
      default: return '';
    }
  };

  const filteredLogs = logs.filter(log => {
    const matchesFilter = filter === 'all' || log.level === filter;
    const matchesSearch = searchTerm === '' || 
      log.message.toLowerCase().includes(searchTerm.toLowerCase()) ||
      (log.context && log.context.toLowerCase().includes(searchTerm.toLowerCase()));
    return matchesFilter && matchesSearch;
  });

  const highlightText = (text: string) => {
    if (!searchTerm) return text;
    
    const parts = text.split(new RegExp(`(${searchTerm})`, 'gi'));
    return parts.map((part, index) => 
      part.toLowerCase() === searchTerm.toLowerCase() 
        ? <mark key={index} className="bg-yellow-400/30 text-yellow-200">{part}</mark>
        : part
    );
  };

  const formatTimestamp = (date: Date) => {
    return new Date(date).toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      fractionalSecondDigits: 3
    });
  };

  return (
    <div className="flex flex-col h-full">
      {/* Controls */}
      {showFilters && (
        <div className="flex items-center justify-between mb-4 space-x-4">
          <div className="flex items-center space-x-2">
            {/* Filter buttons */}
            <button
              onClick={() => setFilter('all')}
              className={`px-3 py-1 text-xs font-medium rounded-lg transition-colors ${
                filter === 'all' 
                  ? 'bg-slate-700 text-white' 
                  : 'bg-slate-800 text-slate-400 hover:bg-slate-700'
              }`}
            >
              All
            </button>
            {(['debug', 'info', 'warn', 'error'] as const).map(level => (
              <button
                key={level}
                onClick={() => setFilter(level)}
                className={`px-3 py-1 text-xs font-medium rounded-lg transition-colors ${
                  filter === level 
                    ? 'bg-slate-700 text-white' 
                    : 'bg-slate-800 text-slate-400 hover:bg-slate-700'
                }`}
              >
                <span className={getLogColor(level)}>{level.toUpperCase()}</span>
              </button>
            ))}
          </div>

          <div className="flex items-center space-x-2">
            {/* Search input */}
            <input
              type="text"
              placeholder="Search logs..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="px-3 py-1 text-xs bg-slate-800 border border-slate-700 rounded-lg text-slate-300 placeholder-slate-500 focus:outline-none focus:border-slate-600"
            />

            {/* Auto-scroll toggle */}
            <button
              onClick={() => setAutoScroll(!autoScroll)}
              className={`px-3 py-1 text-xs font-medium rounded-lg transition-colors ${
                autoScroll 
                  ? 'bg-green-900/50 text-green-400' 
                  : 'bg-slate-800 text-slate-400 hover:bg-slate-700'
              }`}
            >
              Auto-scroll {autoScroll ? 'ON' : 'OFF'}
            </button>
          </div>
        </div>
      )}

      {/* Log container */}
      <div
        ref={containerRef}
        className="flex-1 overflow-y-auto bg-slate-900 rounded-lg p-4 font-mono text-sm scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-slate-800"
        style={{ maxHeight }}
        onScroll={() => {
          if (containerRef.current) {
            const { scrollTop, scrollHeight, clientHeight } = containerRef.current;
            const isAtBottom = scrollTop + clientHeight >= scrollHeight - 10;
            if (!isAtBottom && autoScroll) {
              setAutoScroll(false);
            }
          }
        }}
      >
        <AnimatePresence initial={false}>
          {filteredLogs.length === 0 ? (
            <div className="text-center text-slate-500 py-8">
              No logs to display
            </div>
          ) : (
            filteredLogs.map((log, index) => (
              <motion.div
                key={`${log.timestamp}-${index}`}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: 20 }}
                transition={{ duration: 0.2 }}
                className={`flex items-start space-x-3 py-1.5 px-2 rounded ${getLogBgColor(log.level)} hover:bg-slate-800/50 transition-colors`}
              >
                {/* Timestamp */}
                <span className="text-slate-500 text-xs whitespace-nowrap select-none">
                  {formatTimestamp(log.timestamp)}
                </span>

                {/* Level */}
                <span className={`font-semibold text-xs w-14 ${getLogColor(log.level)}`}>
                  [{log.level.toUpperCase()}]
                </span>

                {/* Context (if present) */}
                {log.context && (
                  <span className="text-purple-400 text-xs">
                    [{highlightText(log.context)}]
                  </span>
                )}

                {/* Message */}
                <span className="text-slate-300 flex-1 break-all">
                  {highlightText(log.message)}
                </span>
              </motion.div>
            ))
          )}
        </AnimatePresence>

        {/* Loading indicator for new logs */}
        {autoScroll && (
          <div className="flex items-center justify-center py-2">
            <motion.div
              className="flex space-x-1"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
            >
              {[0, 1, 2].map((i) => (
                <motion.div
                  key={i}
                  className="w-1.5 h-1.5 bg-blue-400 rounded-full"
                  animate={{
                    y: [0, -8, 0],
                    opacity: [0.5, 1, 0.5]
                  }}
                  transition={{
                    duration: 1.5,
                    repeat: Infinity,
                    delay: i * 0.2
                  }}
                />
              ))}
            </motion.div>
          </div>
        )}
      </div>

      {/* Log statistics */}
      <div className="mt-2 flex items-center justify-between text-xs text-slate-500">
        <span>
          Showing {filteredLogs.length} of {logs.length} logs
        </span>
        <span>
          {logs.filter(l => l.level === 'error').length} errors, 
          {' '}{logs.filter(l => l.level === 'warn').length} warnings
        </span>
      </div>
    </div>
  );
};