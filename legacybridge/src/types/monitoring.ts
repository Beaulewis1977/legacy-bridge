// Monitoring dashboard types and interfaces

export interface MonitoringDashboard {
  buildStatus: BuildStatus;
  performanceMetrics: PerformanceMetrics;
  legacyFunctions: LegacyFunctionStats[];
  systemHealth: SystemHealth;
  realTimeLogs: LogEntry[];
}

export interface BuildStatus {
  compilation: 'idle' | 'building' | 'success' | 'failed';
  progress: number; // 0-100
  timeElapsed: number; // seconds
  estimatedTimeRemaining: number; // seconds
  currentStep: string; // "Compiling Rust", "Linking DLL"
  errors: CompilationError[];
  warnings: CompilationWarning[];
}

export interface CompilationError {
  message: string;
  file?: string;
  line?: number;
}

export interface CompilationWarning {
  message: string;
  file?: string;
  line?: number;
}

export interface PerformanceMetrics {
  conversionsPerSecond: number;
  memoryUsage: {
    used: number; // MB
    total: number; // MB
    percentage: number; // 0-100
  };
  cpuUtilization: number; // 0-100
  activeConnections: number;
  queuedJobs: number;
}

export interface LegacyFunctionStats {
  functionName: string;
  callCount: number;
  averageResponseTime: number; // milliseconds
  errorRate: number; // 0-100 percentage
  lastCalled: Date;
  peakUsage: number; // calls per minute
  status: 'active' | 'idle' | 'error';
}

export interface SystemHealth {
  status: 'healthy' | 'warning' | 'critical';
  uptime: number; // seconds
  version: string;
  environment: 'development' | 'staging' | 'production';
  lastError?: SystemError;
}

export interface SystemError {
  message: string;
  timestamp: Date;
  severity: 'low' | 'medium' | 'high' | 'critical';
}

export interface LogEntry {
  timestamp: Date;
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  context?: string;
}

export interface PerformanceDataPoint {
  timestamp: string;
  conversionsPerSecond: number;
  memoryUsage: number;
  cpuUsage: number;
}

export interface AlertRule {
  id: string;
  name: string;
  condition: string;
  threshold: number;
  severity: 'low' | 'medium' | 'high' | 'critical';
  actions: AlertAction[];
}

export interface AlertAction {
  type: 'email' | 'slack' | 'webhook';
  config: Record<string, any>;
}