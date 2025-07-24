'use client';

import React, { useState, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { format } from 'date-fns';
import { 
  AlertCircle, 
  AlertTriangle, 
  Info, 
  Bug, 
  XCircle, 
  Search,
  Download,
  RefreshCw,
  Filter,
  TrendingUp,
  Clock
} from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu';
import logger, { LogLevel, LogEntry, ErrorAnalytics } from '@/lib/error-logger';

interface ErrorLogViewerProps {
  embedded?: boolean;
  maxHeight?: string;
  autoRefresh?: boolean;
  refreshInterval?: number;
}

const ErrorLogViewer: React.FC<ErrorLogViewerProps> = ({
  embedded = false,
  maxHeight = '600px',
  autoRefresh = true,
  refreshInterval = 5000
}) => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [filteredLogs, setFilteredLogs] = useState<LogEntry[]>([]);
  const [analytics, setAnalytics] = useState<ErrorAnalytics | null>(null);
  const [selectedLevel, setSelectedLevel] = useState<LogLevel | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [expandedLogId, setExpandedLogId] = useState<string | null>(null);

  // Fetch logs and analytics
  const fetchLogs = useCallback(async () => {
    setIsRefreshing(true);
    try {
      const recentLogs = logger.getRecentLogs(500);
      setLogs(recentLogs);
      
      const analyticsData = logger.getAnalytics();
      setAnalytics(analyticsData);
    } finally {
      setIsRefreshing(false);
    }
  }, []);

  // Initial fetch and auto-refresh
  useEffect(() => {
    fetchLogs();

    if (autoRefresh) {
      const interval = setInterval(fetchLogs, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [fetchLogs, autoRefresh, refreshInterval]);

  // Filter logs based on criteria
  useEffect(() => {
    let filtered = [...logs];

    if (selectedLevel !== null) {
      filtered = filtered.filter(log => log.level === selectedLevel);
    }

    if (selectedCategory) {
      filtered = filtered.filter(log => log.category === selectedCategory);
    }

    if (searchQuery) {
      filtered = filtered.filter(log => 
        log.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
        JSON.stringify(log.context).toLowerCase().includes(searchQuery.toLowerCase())
      );
    }

    setFilteredLogs(filtered.reverse()); // Show newest first
  }, [logs, selectedLevel, selectedCategory, searchQuery]);

  // Get unique categories
  const categories = Array.from(new Set(logs.map(log => log.category)));

  // Get log level icon and color
  const getLogLevelIcon = (level: LogLevel) => {
    switch (level) {
      case LogLevel.DEBUG:
        return <Bug className="w-4 h-4" />;
      case LogLevel.INFO:
        return <Info className="w-4 h-4" />;
      case LogLevel.WARN:
        return <AlertTriangle className="w-4 h-4" />;
      case LogLevel.ERROR:
        return <AlertCircle className="w-4 h-4" />;
      case LogLevel.FATAL:
        return <XCircle className="w-4 h-4" />;
    }
  };

  const getLogLevelColor = (level: LogLevel) => {
    switch (level) {
      case LogLevel.DEBUG:
        return 'text-blue-400';
      case LogLevel.INFO:
        return 'text-green-400';
      case LogLevel.WARN:
        return 'text-amber-400';
      case LogLevel.ERROR:
        return 'text-red-400';
      case LogLevel.FATAL:
        return 'text-purple-400';
    }
  };

  // Export logs
  const exportLogs = () => {
    const data = JSON.stringify(filteredLogs, null, 2);
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `legacybridge-logs-${format(new Date(), 'yyyy-MM-dd-HHmmss')}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className={`error-log-viewer ${embedded ? '' : 'p-6'}`}>
      {/* Header */}
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-slate-900 dark:text-slate-100 mb-2">
          Error Logs & Analytics
        </h2>
        
        {/* Analytics Summary */}
        {analytics && (
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
            <Card className="p-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-slate-600 dark:text-slate-400">Total Errors</p>
                  <p className="text-2xl font-bold text-red-600">{analytics.errorCount}</p>
                </div>
                <AlertCircle className="w-8 h-8 text-red-400" />
              </div>
            </Card>
            
            <Card className="p-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-slate-600 dark:text-slate-400">Error Rate</p>
                  <p className="text-2xl font-bold text-amber-600">{analytics.errorRate}/min</p>
                </div>
                <TrendingUp className="w-8 h-8 text-amber-400" />
              </div>
            </Card>
            
            <Card className="p-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-slate-600 dark:text-slate-400">Categories</p>
                  <p className="text-2xl font-bold text-blue-600">
                    {Object.keys(analytics.errorsByCategory).length}
                  </p>
                </div>
                <Filter className="w-8 h-8 text-blue-400" />
              </div>
            </Card>
            
            <Card className="p-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-slate-600 dark:text-slate-400">Recent Logs</p>
                  <p className="text-2xl font-bold text-green-600">{logs.length}</p>
                </div>
                <Clock className="w-8 h-8 text-green-400" />
              </div>
            </Card>
          </div>
        )}

        {/* Controls */}
        <div className="flex flex-wrap gap-4 items-center">
          {/* Search */}
          <div className="relative flex-1 min-w-[200px]">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-slate-400" />
            <Input
              type="text"
              placeholder="Search logs..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-10"
            />
          </div>

          {/* Level Filter */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" className="gap-2">
                <Filter className="w-4 h-4" />
                {selectedLevel !== null ? LogLevel[selectedLevel] : 'All Levels'}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              <DropdownMenuItem onClick={() => setSelectedLevel(null)}>
                All Levels
              </DropdownMenuItem>
              {Object.values(LogLevel).filter(v => typeof v === 'number').map(level => (
                <DropdownMenuItem 
                  key={level} 
                  onClick={() => setSelectedLevel(level as LogLevel)}
                >
                  <span className={`flex items-center gap-2 ${getLogLevelColor(level as LogLevel)}`}>
                    {getLogLevelIcon(level as LogLevel)}
                    {LogLevel[level as LogLevel]}
                  </span>
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>

          {/* Category Filter */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" className="gap-2">
                <Filter className="w-4 h-4" />
                {selectedCategory || 'All Categories'}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              <DropdownMenuItem onClick={() => setSelectedCategory(null)}>
                All Categories
              </DropdownMenuItem>
              {categories.map(category => (
                <DropdownMenuItem 
                  key={category}
                  onClick={() => setSelectedCategory(category)}
                >
                  {category}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>

          {/* Actions */}
          <Button
            variant="outline"
            size="icon"
            onClick={fetchLogs}
            disabled={isRefreshing}
          >
            <RefreshCw className={`w-4 h-4 ${isRefreshing ? 'animate-spin' : ''}`} />
          </Button>
          
          <Button
            variant="outline"
            size="icon"
            onClick={exportLogs}
          >
            <Download className="w-4 h-4" />
          </Button>
        </div>
      </div>

      {/* Logs List */}
      <div 
        className="space-y-2 overflow-y-auto pr-2"
        style={{ maxHeight }}
      >
        <AnimatePresence>
          {filteredLogs.map((log) => (
            <motion.div
              key={log.id}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -20 }}
              transition={{ duration: 0.2 }}
            >
              <Card 
                className={`p-4 cursor-pointer transition-all hover:shadow-md ${
                  expandedLogId === log.id ? 'ring-2 ring-blue-500' : ''
                }`}
                onClick={() => setExpandedLogId(expandedLogId === log.id ? null : log.id)}
              >
                <div className="flex items-start gap-3">
                  <span className={getLogLevelColor(log.level)}>
                    {getLogLevelIcon(log.level)}
                  </span>
                  
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <Badge variant="outline" className="text-xs">
                        {log.category}
                      </Badge>
                      <span className="text-xs text-slate-500">
                        {format(new Date(log.timestamp), 'HH:mm:ss.SSS')}
                      </span>
                      {log.userId && (
                        <span className="text-xs text-slate-500">
                          User: {log.userId}
                        </span>
                      )}
                      {log.duration && (
                        <span className="text-xs text-slate-500">
                          {log.duration}ms
                        </span>
                      )}
                    </div>
                    
                    <p className="text-sm text-slate-900 dark:text-slate-100 break-words">
                      {log.message}
                    </p>
                    
                    {/* Expanded Details */}
                    <AnimatePresence>
                      {expandedLogId === log.id && (
                        <motion.div
                          initial={{ opacity: 0, height: 0 }}
                          animate={{ opacity: 1, height: 'auto' }}
                          exit={{ opacity: 0, height: 0 }}
                          transition={{ duration: 0.2 }}
                          className="mt-3"
                        >
                          {log.context && Object.keys(log.context).length > 0 && (
                            <div className="mb-3">
                              <p className="text-xs font-semibold text-slate-600 dark:text-slate-400 mb-1">
                                Context:
                              </p>
                              <pre className="text-xs bg-slate-100 dark:bg-slate-800 p-2 rounded overflow-x-auto">
                                {JSON.stringify(log.context, null, 2)}
                              </pre>
                            </div>
                          )}
                          
                          {log.stackTrace && (
                            <div>
                              <p className="text-xs font-semibold text-slate-600 dark:text-slate-400 mb-1">
                                Stack Trace:
                              </p>
                              <pre className="text-xs bg-red-50 dark:bg-red-950 text-red-700 dark:text-red-300 p-2 rounded overflow-x-auto">
                                {log.stackTrace}
                              </pre>
                            </div>
                          )}
                          
                          {log.metadata && (
                            <div className="mt-3">
                              <p className="text-xs font-semibold text-slate-600 dark:text-slate-400 mb-1">
                                Metadata:
                              </p>
                              <div className="text-xs text-slate-500">
                                <span>Environment: {log.metadata.environment}</span>
                                <span className="mx-2">•</span>
                                <span>Version: {log.metadata.version}</span>
                                <span className="mx-2">•</span>
                                <span>Platform: {log.metadata.platform}</span>
                              </div>
                            </div>
                          )}
                        </motion.div>
                      )}
                    </AnimatePresence>
                  </div>
                </div>
              </Card>
            </motion.div>
          ))}
        </AnimatePresence>
        
        {filteredLogs.length === 0 && (
          <div className="text-center py-12">
            <p className="text-slate-500">No logs found matching your criteria</p>
          </div>
        )}
      </div>

      {/* Top Errors */}
      {analytics && analytics.topErrors.length > 0 && (
        <div className="mt-6">
          <h3 className="text-lg font-semibold text-slate-900 dark:text-slate-100 mb-3">
            Top Errors
          </h3>
          <div className="space-y-2">
            {analytics.topErrors.slice(0, 5).map((error, index) => (
              <Card key={index} className="p-3">
                <div className="flex items-center justify-between">
                  <p className="text-sm text-slate-700 dark:text-slate-300 truncate flex-1">
                    {error.message}
                  </p>
                  <div className="flex items-center gap-2 ml-4">
                    <Badge variant="destructive">{error.count}</Badge>
                    <span className="text-xs text-slate-500">
                      {format(new Date(error.lastOccurred), 'HH:mm:ss')}
                    </span>
                  </div>
                </div>
              </Card>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default ErrorLogViewer;