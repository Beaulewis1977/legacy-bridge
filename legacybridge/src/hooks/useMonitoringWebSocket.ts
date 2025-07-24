'use client';

import { useState, useEffect, useCallback, useRef } from 'react';
import { MonitoringDashboard, LogEntry } from '@/types/monitoring';

interface WebSocketMessage {
  type: 'initial_state' | 'metrics_update' | 'log_entry' | 'build_update' | 'function_update';
  data: any;
  timestamp: string;
}

export function useMonitoringWebSocket(url: string = 'ws://localhost:8080') {
  const [data, setData] = useState<MonitoringDashboard | null>(null);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);

  const connect = useCallback(() => {
    try {
      // Clean up existing connection
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.close();
      }

      console.log('Attempting to connect to monitoring WebSocket...');
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        console.log('WebSocket connected');
        setConnected(true);
        setError(null);
        reconnectAttemptsRef.current = 0;
      };

      ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          
          switch (message.type) {
            case 'initial_state':
              setData(message.data);
              break;
              
            case 'metrics_update':
              setData(prev => prev ? {
                ...prev,
                performanceMetrics: message.data.performanceMetrics,
                systemHealth: message.data.systemHealth
              } : null);
              break;
              
            case 'build_update':
              setData(prev => prev ? {
                ...prev,
                buildStatus: message.data
              } : null);
              break;
              
            case 'function_update':
              setData(prev => prev ? {
                ...prev,
                legacyFunctions: message.data
              } : null);
              break;
              
            case 'log_entry':
              setData(prev => {
                if (!prev) return null;
                const newLogs = [...prev.realTimeLogs, message.data];
                // Keep only the last 100 logs
                return {
                  ...prev,
                  realTimeLogs: newLogs.slice(-100)
                };
              });
              break;
          }
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };

      ws.onerror = (event) => {
        console.error('WebSocket error:', event);
        setError('WebSocket connection error');
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected');
        setConnected(false);
        wsRef.current = null;

        // Implement exponential backoff for reconnection
        const backoffDelay = Math.min(1000 * Math.pow(2, reconnectAttemptsRef.current), 30000);
        reconnectAttemptsRef.current += 1;

        console.log(`Reconnecting in ${backoffDelay}ms (attempt ${reconnectAttemptsRef.current})`);
        
        reconnectTimeoutRef.current = setTimeout(() => {
          connect();
        }, backoffDelay);
      };
    } catch (error) {
      console.error('Failed to connect to WebSocket:', error);
      setError('Failed to connect to monitoring server');
      setConnected(false);
    }
  }, [url]);

  useEffect(() => {
    // Simulate connection for development
    if (process.env.NODE_ENV === 'development') {
      console.log('Running in development mode - using simulated data');
      setConnected(true);
      simulateData();
      return;
    }

    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [connect]);

  // Simulate data for development
  const simulateData = () => {
    const initialData: MonitoringDashboard = {
      buildStatus: {
        compilation: 'idle',
        progress: 0,
        timeElapsed: 0,
        estimatedTimeRemaining: 0,
        currentStep: 'Idle',
        errors: [],
        warnings: []
      },
      performanceMetrics: {
        conversionsPerSecond: 75,
        memoryUsage: {
          used: 512,
          total: 1024,
          percentage: 50
        },
        cpuUtilization: 35,
        activeConnections: 5,
        queuedJobs: 2
      },
      legacyFunctions: [
        {
          functionName: 'legacybridge_rtf_to_markdown',
          callCount: 1234,
          averageResponseTime: 25.5,
          errorRate: 0.5,
          lastCalled: new Date(),
          peakUsage: 85,
          status: 'active'
        },
        {
          functionName: 'legacybridge_markdown_to_rtf',
          callCount: 987,
          averageResponseTime: 32.1,
          errorRate: 1.2,
          lastCalled: new Date(),
          peakUsage: 62,
          status: 'active'
        },
        {
          functionName: 'legacybridge_validate_rtf',
          callCount: 456,
          averageResponseTime: 15.3,
          errorRate: 0,
          lastCalled: new Date(),
          peakUsage: 34,
          status: 'idle'
        },
        {
          functionName: 'legacybridge_batch_convert',
          callCount: 234,
          averageResponseTime: 125.7,
          errorRate: 2.5,
          lastCalled: new Date(),
          peakUsage: 45,
          status: 'active'
        }
      ],
      systemHealth: {
        status: 'healthy',
        uptime: 86400,
        version: '1.0.0',
        environment: 'production'
      },
      realTimeLogs: [
        {
          timestamp: new Date(),
          level: 'info',
          message: 'Monitoring system initialized',
          context: 'System'
        }
      ]
    };

    setData(initialData);

    // Simulate updates
    const interval = setInterval(() => {
      setData(prev => {
        if (!prev) return null;

        // Update performance metrics
        const newMetrics = {
          ...prev.performanceMetrics,
          conversionsPerSecond: Math.max(0, prev.performanceMetrics.conversionsPerSecond + (Math.random() - 0.5) * 10),
          cpuUtilization: Math.max(0, Math.min(100, prev.performanceMetrics.cpuUtilization + (Math.random() - 0.5) * 5)),
          memoryUsage: {
            ...prev.performanceMetrics.memoryUsage,
            used: Math.max(256, Math.min(900, prev.performanceMetrics.memoryUsage.used + (Math.random() - 0.5) * 50)),
            percentage: 0
          }
        };
        newMetrics.memoryUsage.percentage = (newMetrics.memoryUsage.used / newMetrics.memoryUsage.total) * 100;

        // Add random log entries
        const logTypes: LogEntry['level'][] = ['debug', 'info', 'warn', 'error'];
        const logMessages = [
          'Processing RTF conversion request',
          'Completed markdown transformation',
          'Cache hit for document template',
          'Starting batch conversion job',
          'Memory cleanup completed',
          'Connection established from legacy client',
          'Optimizing conversion pipeline'
        ];

        const newLog: LogEntry = {
          timestamp: new Date(),
          level: logTypes[Math.floor(Math.random() * logTypes.length)],
          message: logMessages[Math.floor(Math.random() * logMessages.length)],
          context: 'Conversion'
        };

        return {
          ...prev,
          performanceMetrics: newMetrics,
          realTimeLogs: [...prev.realTimeLogs.slice(-99), newLog]
        };
      });
    }, 2000);

    return () => clearInterval(interval);
  };

  const sendMessage = useCallback((message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    }
  }, []);

  return {
    data,
    connected,
    error,
    sendMessage,
    reconnect: connect
  };
}