// WebSocket server for real-time monitoring updates
import { WebSocketServer, WebSocket } from 'ws';
import { 
  MonitoringDashboard, 
  BuildStatus, 
  PerformanceMetrics, 
  LegacyFunctionStats,
  SystemHealth,
  LogEntry 
} from '@/types/monitoring';

interface WebSocketMessage {
  type: 'initial_state' | 'metrics_update' | 'log_entry' | 'build_update' | 'function_update';
  data: any;
  timestamp: string;
}

export class MonitoringWebSocketServer {
  private wss: WebSocketServer;
  private clients: Set<WebSocket> = new Set();
  private currentMetrics: MonitoringDashboard;
  private metricsInterval: NodeJS.Timeout | null = null;
  private logBuffer: LogEntry[] = [];
  private maxLogEntries = 1000;

  constructor(port: number) {
    this.wss = new WebSocketServer({ port });
    this.currentMetrics = this.getInitialMetrics();
    this.setupServer();
    this.startMetricsCollection();
  }

  private getInitialMetrics(): MonitoringDashboard {
    return {
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
        conversionsPerSecond: 0,
        memoryUsage: {
          used: 0,
          total: 0,
          percentage: 0
        },
        cpuUtilization: 0,
        activeConnections: 0,
        queuedJobs: 0
      },
      legacyFunctions: [],
      systemHealth: {
        status: 'healthy',
        uptime: 0,
        version: '1.0.0',
        environment: 'production'
      },
      realTimeLogs: []
    };
  }

  private setupServer() {
    this.wss.on('connection', (ws: WebSocket) => {
      this.clients.add(ws);
      console.log(`New monitoring client connected. Total clients: ${this.clients.size}`);

      // Send initial state
      this.sendToClient(ws, {
        type: 'initial_state',
        data: this.currentMetrics,
        timestamp: new Date().toISOString()
      });

      ws.on('close', () => {
        this.clients.delete(ws);
        console.log(`Monitoring client disconnected. Total clients: ${this.clients.size}`);
      });

      ws.on('error', (error) => {
        console.error('WebSocket error:', error);
        this.clients.delete(ws);
      });
    });
  }

  private startMetricsCollection() {
    // Collect metrics every second
    this.metricsInterval = setInterval(() => {
      this.updateMetrics();
    }, 1000);

    // Simulate build status updates
    this.simulateBuildStatus();

    // Simulate function call monitoring
    this.simulateFunctionCalls();
  }

  private async updateMetrics() {
    try {
      // In production, these would call the actual FFI functions
      const metrics = await this.getCurrentMetrics();
      
      this.currentMetrics.performanceMetrics = metrics.performanceMetrics;
      this.currentMetrics.systemHealth = metrics.systemHealth;

      this.broadcast({
        type: 'metrics_update',
        data: {
          performanceMetrics: metrics.performanceMetrics,
          systemHealth: metrics.systemHealth
        },
        timestamp: new Date().toISOString()
      });
    } catch (error) {
      console.error('Failed to update metrics:', error);
    }
  }

  private async getCurrentMetrics(): Promise<MonitoringDashboard> {
    // In production, this would call the Rust FFI functions
    // For now, we'll simulate with realistic data
    const baseMemory = 1024 * 1024 * 1024; // 1GB
    const usedMemory = Math.floor(baseMemory * (0.3 + Math.random() * 0.4));
    
    return {
      ...this.currentMetrics,
      performanceMetrics: {
        conversionsPerSecond: Math.floor(Math.random() * 100) + 50,
        memoryUsage: {
          used: Math.floor(usedMemory / (1024 * 1024)), // Convert to MB
          total: Math.floor(baseMemory / (1024 * 1024)),
          percentage: (usedMemory / baseMemory) * 100
        },
        cpuUtilization: Math.floor(Math.random() * 60) + 20,
        activeConnections: Math.floor(Math.random() * 10) + 1,
        queuedJobs: Math.floor(Math.random() * 5)
      },
      systemHealth: {
        status: 'healthy',
        uptime: Math.floor(process.uptime()),
        version: '1.0.0',
        environment: 'production'
      }
    };
  }

  private simulateBuildStatus() {
    // Simulate a build every 30 seconds
    setInterval(() => {
      if (this.currentMetrics.buildStatus.compilation === 'idle') {
        this.startBuild();
      }
    }, 30000);
  }

  private startBuild() {
    const buildSteps = [
      'Initializing build environment',
      'Parsing Rust source files',
      'Compiling Rust to native code',
      'Optimizing binary',
      'Linking DLL',
      'Running post-build scripts',
      'Verifying DLL exports',
      'Build complete'
    ];

    let currentStepIndex = 0;
    const buildStartTime = Date.now();
    const totalBuildTime = 15000; // 15 seconds
    
    this.currentMetrics.buildStatus = {
      compilation: 'building',
      progress: 0,
      timeElapsed: 0,
      estimatedTimeRemaining: totalBuildTime / 1000,
      currentStep: buildSteps[0],
      errors: [],
      warnings: []
    };

    const buildInterval = setInterval(() => {
      const elapsed = Date.now() - buildStartTime;
      const progress = Math.min((elapsed / totalBuildTime) * 100, 100);
      
      currentStepIndex = Math.floor((progress / 100) * buildSteps.length);
      
      this.currentMetrics.buildStatus = {
        ...this.currentMetrics.buildStatus,
        progress,
        timeElapsed: Math.floor(elapsed / 1000),
        estimatedTimeRemaining: Math.max(0, Math.floor((totalBuildTime - elapsed) / 1000)),
        currentStep: buildSteps[Math.min(currentStepIndex, buildSteps.length - 1)]
      };

      // Add some random warnings
      if (progress > 30 && progress < 35 && this.currentMetrics.buildStatus.warnings.length === 0) {
        this.currentMetrics.buildStatus.warnings.push({
          message: 'Unused variable in rtf_parser.rs',
          file: 'src/conversion/rtf_parser.rs',
          line: 234
        });
      }

      this.broadcast({
        type: 'build_update',
        data: this.currentMetrics.buildStatus,
        timestamp: new Date().toISOString()
      });

      if (progress >= 100) {
        clearInterval(buildInterval);
        this.currentMetrics.buildStatus.compilation = 'success';
        this.addLog('info', 'Build completed successfully');
        
        setTimeout(() => {
          this.currentMetrics.buildStatus.compilation = 'idle';
        }, 5000);
      }
    }, 250);
  }

  private simulateFunctionCalls() {
    const functions = [
      'legacybridge_rtf_to_markdown',
      'legacybridge_markdown_to_rtf',
      'legacybridge_validate_rtf',
      'legacybridge_clean_formatting',
      'legacybridge_batch_convert',
      'legacybridge_extract_text',
      'legacybridge_apply_template',
      'legacybridge_get_version'
    ];

    // Initialize function stats
    this.currentMetrics.legacyFunctions = functions.map(name => ({
      functionName: name,
      callCount: Math.floor(Math.random() * 1000),
      averageResponseTime: Math.floor(Math.random() * 50) + 10,
      errorRate: Math.random() * 5,
      lastCalled: new Date(),
      peakUsage: Math.floor(Math.random() * 100) + 20,
      status: 'active' as const
    }));

    // Update function stats periodically
    setInterval(() => {
      this.currentMetrics.legacyFunctions = this.currentMetrics.legacyFunctions.map(func => {
        const isActive = Math.random() > 0.2;
        
        return {
          ...func,
          callCount: func.callCount + (isActive ? Math.floor(Math.random() * 10) : 0),
          averageResponseTime: func.averageResponseTime + (Math.random() - 0.5) * 5,
          errorRate: Math.max(0, Math.min(100, func.errorRate + (Math.random() - 0.5) * 2)),
          lastCalled: isActive ? new Date() : func.lastCalled,
          status: func.errorRate > 50 ? 'error' : (isActive ? 'active' : 'idle')
        };
      });

      this.broadcast({
        type: 'function_update',
        data: this.currentMetrics.legacyFunctions,
        timestamp: new Date().toISOString()
      });
    }, 2000);
  }

  public addLog(level: LogEntry['level'], message: string, context?: string) {
    const logEntry: LogEntry = {
      timestamp: new Date(),
      level,
      message,
      context
    };

    this.logBuffer.push(logEntry);
    
    // Keep only the latest entries
    if (this.logBuffer.length > this.maxLogEntries) {
      this.logBuffer = this.logBuffer.slice(-this.maxLogEntries);
    }

    this.currentMetrics.realTimeLogs = this.logBuffer.slice(-100);

    this.broadcast({
      type: 'log_entry',
      data: logEntry,
      timestamp: new Date().toISOString()
    });
  }

  private sendToClient(client: WebSocket, message: WebSocketMessage) {
    if (client.readyState === WebSocket.OPEN) {
      client.send(JSON.stringify(message));
    }
  }

  private broadcast(message: WebSocketMessage) {
    const messageStr = JSON.stringify(message);
    this.clients.forEach(client => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(messageStr);
      }
    });
  }

  public updateBuildStatus(status: Partial<BuildStatus>) {
    this.currentMetrics.buildStatus = {
      ...this.currentMetrics.buildStatus,
      ...status
    };

    this.broadcast({
      type: 'build_update',
      data: this.currentMetrics.buildStatus,
      timestamp: new Date().toISOString()
    });
  }

  public stop() {
    if (this.metricsInterval) {
      clearInterval(this.metricsInterval);
    }
    
    this.clients.forEach(client => {
      client.close();
    });
    
    this.wss.close();
  }
}