import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { jest } from '@jest/globals';
import userEvent from '@testing-library/user-event';
import { MonitoringDashboard, BuildStatus, PerformanceMetrics, LegacyFunctionStats, SystemHealth } from '@/components/monitoring/MonitoringDashboard';

// Mock child components
jest.mock('@/components/monitoring/BuildProgressRing', () => ({
  BuildProgressRing: ({ buildStatus }: any) => (
    <div data-testid="build-progress-ring">
      Build Progress: {buildStatus.progress}%
    </div>
  ),
}));

jest.mock('@/components/monitoring/PerformanceChart', () => ({
  PerformanceChart: ({ metrics, detailed }: any) => (
    <div data-testid="performance-chart">
      Performance Chart {detailed ? '(Detailed)' : ''}
      Conversions: {metrics.conversionsPerSecond}
    </div>
  ),
}));

jest.mock('@/components/monitoring/FunctionCallMatrix', () => ({
  FunctionCallMatrix: ({ functions }: any) => (
    <div data-testid="function-call-matrix">
      Function Matrix: {functions.length} functions
    </div>
  ),
}));

jest.mock('@/components/monitoring/SystemHealthCard', () => ({
  SystemHealthCard: ({ systemHealth }: any) => (
    <div data-testid="system-health-card">
      System Health: {systemHealth.status}
    </div>
  ),
}));

jest.mock('@/components/monitoring/LogStreamViewer', () => ({
  LogStreamViewer: () => <div data-testid="log-stream-viewer">Log Stream Viewer</div>,
}));

jest.mock('../ErrorLogViewer', () => ({
  default: () => <div data-testid="error-log-viewer">Error Log Viewer</div>,
}));

// Mock framer-motion
jest.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

describe('MonitoringDashboard Component', () => {
  const mockBuildStatus: BuildStatus = {
    status: 'building',
    progress: 45,
    currentFile: 'test.rs',
    totalFiles: 100,
    completedFiles: 45,
    startTime: new Date(),
    estimatedTime: 120,
    errors: ['Error 1'],
    warnings: ['Warning 1', 'Warning 2'],
  };

  const mockPerformanceMetrics: PerformanceMetrics = {
    conversionsPerSecond: 25.5,
    memoryUsage: 65,
    cpuUsage: 45,
    activeConnections: 12,
    averageResponseTime: 250,
    throughput: 1024,
    history: [
      {
        timestamp: new Date(),
        conversionsPerSecond: 20,
        memoryUsage: 60,
        cpuUsage: 40,
      },
    ],
  };

  const mockLegacyFunctions: LegacyFunctionStats[] = [
    {
      name: 'rtf_to_markdown',
      calls: 1500,
      averageTime: 45,
      lastCallTime: new Date(),
      errorRate: 0.02,
      successRate: 0.98,
      trend: 'up',
    },
    {
      name: 'markdown_to_rtf',
      calls: 1200,
      averageTime: 38,
      lastCallTime: new Date(),
      errorRate: 0.01,
      successRate: 0.99,
      trend: 'stable',
    },
  ];

  const mockSystemHealth: SystemHealth = {
    status: 'healthy',
    uptime: 86400,
    version: '1.2.3',
    environment: 'production',
    lastUpdate: new Date(),
    services: [
      { name: 'API', status: 'running', health: 100 },
      { name: 'Database', status: 'running', health: 95 },
    ],
  };

  beforeEach(() => {
    jest.clearAllMocks();
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('Basic Rendering', () => {
    it('should render dashboard with title', () => {
      render(<MonitoringDashboard />);
      
      expect(screen.getByText('LegacyBridge Monitor')).toBeInTheDocument();
      expect(screen.getByText('Real-time system monitoring and analytics')).toBeInTheDocument();
    });

    it('should render all status cards', () => {
      render(<MonitoringDashboard performanceMetrics={mockPerformanceMetrics} />);
      
      expect(screen.getByText('Conversions/sec')).toBeInTheDocument();
      expect(screen.getByText('25.5')).toBeInTheDocument();
      
      expect(screen.getByText('Memory Usage')).toBeInTheDocument();
      expect(screen.getByText('65%')).toBeInTheDocument();
      
      expect(screen.getByText('Active Builds')).toBeInTheDocument();
      expect(screen.getByText('System Health')).toBeInTheDocument();
    });

    it('should render with default props when none provided', () => {
      render(<MonitoringDashboard />);
      
      expect(screen.getByText('0.0')).toBeInTheDocument(); // Default conversions/sec
      expect(screen.getByText('0%')).toBeInTheDocument(); // Default memory usage
      expect(screen.getByText('healthy')).toBeInTheDocument(); // Default system health
    });
  });

  describe('Status Cards', () => {
    it('should display active builds correctly', () => {
      render(<MonitoringDashboard buildStatus={mockBuildStatus} />);
      
      expect(screen.getByText('Active Builds')).toBeInTheDocument();
      expect(screen.getByText('1')).toBeInTheDocument();
    });

    it('should show 0 active builds when idle', () => {
      const idleBuildStatus = { ...mockBuildStatus, status: 'idle' as const };
      render(<MonitoringDashboard buildStatus={idleBuildStatus} />);
      
      const activeBuildCard = screen.getByText('Active Builds').closest('div');
      expect(activeBuildCard?.parentElement?.textContent).toContain('0');
    });

    it('should apply correct styling based on system health', () => {
      const { rerender } = render(<MonitoringDashboard systemHealth={mockSystemHealth} />);
      
      let healthIcon = screen.getByText('healthy').parentElement?.querySelector('.gradient-success');
      expect(healthIcon).toBeInTheDocument();
      
      // Test warning status
      const warningHealth = { ...mockSystemHealth, status: 'warning' as const };
      rerender(<MonitoringDashboard systemHealth={warningHealth} />);
      
      healthIcon = screen.getByText('warning').parentElement?.querySelector('.gradient-warning');
      expect(healthIcon).toBeInTheDocument();
      
      // Test critical status
      const criticalHealth = { ...mockSystemHealth, status: 'critical' as const };
      rerender(<MonitoringDashboard systemHealth={criticalHealth} />);
      
      healthIcon = screen.getByText('critical').parentElement?.querySelector('.gradient-error');
      expect(healthIcon).toBeInTheDocument();
    });
  });

  describe('Fullscreen Mode', () => {
    it('should toggle fullscreen mode', async () => {
      const user = userEvent.setup({ advanceTimers: jest.advanceTimersByTime });
      render(<MonitoringDashboard />);
      
      const fullscreenButton = screen.getByRole('button', { name: /fullscreen/i });
      expect(fullscreenButton).toHaveTextContent('Fullscreen');
      
      await user.click(fullscreenButton);
      
      expect(fullscreenButton).toHaveTextContent('Exit Fullscreen');
      const dashboard = screen.getByText('LegacyBridge Monitor').closest('.monitoring-dashboard');
      expect(dashboard).toHaveClass('fixed', 'inset-0', 'z-50');
    });

    it('should exit fullscreen mode', async () => {
      const user = userEvent.setup({ advanceTimers: jest.advanceTimersByTime });
      render(<MonitoringDashboard />);
      
      const fullscreenButton = screen.getByRole('button', { name: /fullscreen/i });
      
      // Enter fullscreen
      await user.click(fullscreenButton);
      expect(fullscreenButton).toHaveTextContent('Exit Fullscreen');
      
      // Exit fullscreen
      await user.click(fullscreenButton);
      expect(fullscreenButton).toHaveTextContent('Fullscreen');
      
      const dashboard = screen.getByText('LegacyBridge Monitor').closest('.monitoring-dashboard');
      expect(dashboard).not.toHaveClass('fixed', 'inset-0', 'z-50');
    });
  });

  describe('Main Content Areas', () => {
    it('should render build progress component', () => {
      render(<MonitoringDashboard buildStatus={mockBuildStatus} />);
      
      expect(screen.getByText('DLL Compilation')).toBeInTheDocument();
      expect(screen.getByTestId('build-progress-ring')).toBeInTheDocument();
      expect(screen.getByText('Build Progress: 45%')).toBeInTheDocument();
    });

    it('should render performance chart', () => {
      render(<MonitoringDashboard performanceMetrics={mockPerformanceMetrics} />);
      
      expect(screen.getByText('Performance Metrics')).toBeInTheDocument();
      expect(screen.getByTestId('performance-chart')).toBeInTheDocument();
      expect(screen.getByText('Conversions: 25.5')).toBeInTheDocument();
    });

    it('should render system health card', () => {
      render(<MonitoringDashboard systemHealth={mockSystemHealth} />);
      
      expect(screen.getByTestId('system-health-card')).toBeInTheDocument();
      expect(screen.getByText('System Health: healthy')).toBeInTheDocument();
    });
  });

  describe('Tab Navigation', () => {
    it('should render all tabs', () => {
      render(<MonitoringDashboard />);
      
      expect(screen.getByRole('tab', { name: 'Performance' })).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: 'Functions' })).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: 'Logs' })).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: 'Error Logs' })).toBeInTheDocument();
    });

    it('should show performance tab by default', () => {
      render(<MonitoringDashboard performanceMetrics={mockPerformanceMetrics} />);
      
      expect(screen.getByRole('tab', { name: 'Performance' })).toHaveAttribute('data-state', 'active');
      expect(screen.getByText('Performance Chart (Detailed)')).toBeInTheDocument();
    });

    it('should switch to functions tab', async () => {
      const user = userEvent.setup({ advanceTimers: jest.advanceTimersByTime });
      render(<MonitoringDashboard legacyFunctions={mockLegacyFunctions} />);
      
      const functionsTab = screen.getByRole('tab', { name: 'Functions' });
      await user.click(functionsTab);
      
      expect(functionsTab).toHaveAttribute('data-state', 'active');
      expect(screen.getByTestId('function-call-matrix')).toBeInTheDocument();
      expect(screen.getByText('Function Matrix: 2 functions')).toBeInTheDocument();
    });

    it('should switch to logs tab', async () => {
      const user = userEvent.setup({ advanceTimers: jest.advanceTimersByTime });
      render(<MonitoringDashboard />);
      
      const logsTab = screen.getByRole('tab', { name: 'Logs' });
      await user.click(logsTab);
      
      expect(logsTab).toHaveAttribute('data-state', 'active');
      expect(screen.getByTestId('log-stream-viewer')).toBeInTheDocument();
    });

    it('should switch to error logs tab', async () => {
      const user = userEvent.setup({ advanceTimers: jest.advanceTimersByTime });
      render(<MonitoringDashboard />);
      
      const errorLogsTab = screen.getByRole('tab', { name: 'Error Logs' });
      await user.click(errorLogsTab);
      
      expect(errorLogsTab).toHaveAttribute('data-state', 'active');
      expect(screen.getByTestId('error-log-viewer')).toBeInTheDocument();
    });
  });

  describe('Auto-refresh', () => {
    it('should set up auto-refresh interval', () => {
      const setIntervalSpy = jest.spyOn(global, 'setInterval');
      
      render(<MonitoringDashboard />);
      
      expect(setIntervalSpy).toHaveBeenCalledWith(expect.any(Function), 5000);
    });

    it('should clear interval on unmount', () => {
      const clearIntervalSpy = jest.spyOn(global, 'clearInterval');
      
      const { unmount } = render(<MonitoringDashboard />);
      
      unmount();
      
      expect(clearIntervalSpy).toHaveBeenCalled();
    });
  });

  describe('Data Display', () => {
    it('should format conversions per second correctly', () => {
      render(<MonitoringDashboard performanceMetrics={mockPerformanceMetrics} />);
      
      expect(screen.getByText('25.5')).toBeInTheDocument();
    });

    it('should handle zero values', () => {
      const zeroMetrics: PerformanceMetrics = {
        conversionsPerSecond: 0,
        memoryUsage: 0,
        cpuUsage: 0,
        activeConnections: 0,
        averageResponseTime: 0,
        throughput: 0,
        history: [],
      };
      
      render(<MonitoringDashboard performanceMetrics={zeroMetrics} />);
      
      expect(screen.getByText('0.0')).toBeInTheDocument();
      expect(screen.getByText('0%')).toBeInTheDocument();
    });

    it('should display build errors and warnings count', () => {
      render(<MonitoringDashboard buildStatus={mockBuildStatus} />);
      
      // Build status is passed to BuildProgressRing component
      expect(screen.getByTestId('build-progress-ring')).toBeInTheDocument();
    });
  });

  describe('Hover Effects', () => {
    it('should apply hover effects to status cards', () => {
      const { container } = render(<MonitoringDashboard />);
      
      const hoverCards = container.querySelectorAll('.hover-lift');
      expect(hoverCards.length).toBeGreaterThan(0);
    });
  });

  describe('Responsive Layout', () => {
    it('should use responsive grid classes', () => {
      const { container } = render(<MonitoringDashboard />);
      
      // Check for responsive grid classes
      const grids = container.querySelectorAll('[class*="grid-cols"]');
      expect(grids.length).toBeGreaterThan(0);
      
      // Check for responsive breakpoints
      const responsiveElements = container.querySelectorAll('[class*="md:"], [class*="lg:"]');
      expect(responsiveElements.length).toBeGreaterThan(0);
    });
  });

  describe('Component Integration', () => {
    it('should pass correct props to child components', () => {
      render(
        <MonitoringDashboard
          buildStatus={mockBuildStatus}
          performanceMetrics={mockPerformanceMetrics}
          legacyFunctions={mockLegacyFunctions}
          systemHealth={mockSystemHealth}
        />
      );
      
      // Verify props are passed correctly
      expect(screen.getByText('Build Progress: 45%')).toBeInTheDocument();
      expect(screen.getByText('Conversions: 25.5')).toBeInTheDocument();
      expect(screen.getByText('System Health: healthy')).toBeInTheDocument();
    });

    it('should render detailed performance chart in tab', async () => {
      const user = userEvent.setup({ advanceTimers: jest.advanceTimersByTime });
      render(<MonitoringDashboard performanceMetrics={mockPerformanceMetrics} />);
      
      // Performance tab should be active by default and show detailed view
      expect(screen.getByText('Performance Chart (Detailed)')).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty legacy functions array', async () => {
      const user = userEvent.setup({ advanceTimers: jest.advanceTimersByTime });
      render(<MonitoringDashboard legacyFunctions={[]} />);
      
      await user.click(screen.getByRole('tab', { name: 'Functions' }));
      
      expect(screen.getByText('Function Matrix: 0 functions')).toBeInTheDocument();
    });

    it('should handle undefined optional props', () => {
      render(<MonitoringDashboard />);
      
      // Should render with defaults
      expect(screen.getByText('0.0')).toBeInTheDocument();
      expect(screen.getByText('idle')).toBeInTheDocument();
    });

    it('should handle long uptime values', () => {
      const longUptimeHealth = {
        ...mockSystemHealth,
        uptime: 2592000, // 30 days in seconds
      };
      
      render(<MonitoringDashboard systemHealth={longUptimeHealth} />);
      
      expect(screen.getByTestId('system-health-card')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have proper heading hierarchy', () => {
      render(<MonitoringDashboard />);
      
      const h2 = screen.getByRole('heading', { level: 2 });
      expect(h2).toHaveTextContent('LegacyBridge Monitor');
    });

    it('should have accessible tab navigation', async () => {
      const user = userEvent.setup({ advanceTimers: jest.advanceTimersByTime });
      render(<MonitoringDashboard />);
      
      // Tab through tabs
      await user.tab();
      const firstTab = screen.getByRole('tab', { name: 'Performance' });
      
      // Arrow key navigation
      await user.keyboard('{ArrowRight}');
      expect(screen.getByRole('tab', { name: 'Functions' })).toHaveFocus();
    });

    it('should have proper button labels', () => {
      render(<MonitoringDashboard />);
      
      const fullscreenButton = screen.getByRole('button', { name: /fullscreen/i });
      expect(fullscreenButton).toHaveAccessibleName();
    });
  });

  describe('Visual States', () => {
    it('should show correct icon colors based on card type', () => {
      const { container } = render(<MonitoringDashboard />);
      
      expect(container.querySelector('.gradient-primary')).toBeInTheDocument();
      expect(container.querySelector('.gradient-info')).toBeInTheDocument();
      expect(container.querySelector('.gradient-warning')).toBeInTheDocument();
    });

    it('should apply glass panel styling', () => {
      const { container } = render(<MonitoringDashboard />);
      
      const glassPanels = container.querySelectorAll('.glass-panel');
      expect(glassPanels.length).toBeGreaterThan(0);
    });
  });
});