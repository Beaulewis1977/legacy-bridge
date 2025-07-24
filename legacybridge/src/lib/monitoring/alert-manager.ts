// Alert system integration for monitoring dashboard
import { 
  MonitoringDashboard, 
  AlertRule, 
  AlertAction,
  SystemError,
  PerformanceMetrics,
  BuildStatus,
  LegacyFunctionStats
} from '@/types/monitoring';

export interface Alert {
  id: string;
  ruleId: string;
  title: string;
  message: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  timestamp: Date;
  acknowledged: boolean;
  resolvedAt?: Date;
  metadata?: Record<string, any>;
}

export class AlertManager {
  private rules: AlertRule[] = [];
  private activeAlerts: Map<string, Alert> = new Map();
  private alertHistory: Alert[] = [];
  private webhooks: Map<string, string> = new Map();
  private evaluationInterval: NodeJS.Timeout | null = null;

  constructor() {
    this.initializeDefaultRules();
  }

  private initializeDefaultRules() {
    this.rules = [
      {
        id: 'high_error_rate',
        name: 'High Error Rate',
        condition: 'error_rate',
        threshold: 5, // 5% error rate
        severity: 'high',
        actions: [
          { type: 'webhook', config: { url: '/api/alerts/slack' } },
          { type: 'email', config: { recipients: ['admin@company.com'] } }
        ]
      },
      {
        id: 'build_failure',
        name: 'Build Failure',
        condition: 'build_status',
        threshold: 1,
        severity: 'critical',
        actions: [
          { type: 'webhook', config: { url: '/api/alerts/slack', channel: '#dev-team' } }
        ]
      },
      {
        id: 'high_memory_usage',
        name: 'High Memory Usage',
        condition: 'memory_usage',
        threshold: 90, // 90% memory usage
        severity: 'high',
        actions: [
          { type: 'webhook', config: { url: '/api/alerts/ops' } }
        ]
      },
      {
        id: 'high_cpu_usage',
        name: 'High CPU Usage',
        condition: 'cpu_usage',
        threshold: 85, // 85% CPU usage
        severity: 'medium',
        actions: [
          { type: 'webhook', config: { url: '/api/alerts/ops' } }
        ]
      },
      {
        id: 'low_conversion_rate',
        name: 'Low Conversion Rate',
        condition: 'conversion_rate',
        threshold: 10, // Less than 10 conversions/sec
        severity: 'low',
        actions: [
          { type: 'webhook', config: { url: '/api/alerts/monitoring' } }
        ]
      },
      {
        id: 'function_error_spike',
        name: 'Function Error Spike',
        condition: 'function_error_rate',
        threshold: 20, // 20% error rate for any function
        severity: 'high',
        actions: [
          { type: 'webhook', config: { url: '/api/alerts/dev' } }
        ]
      }
    ];
  }

  public startEvaluation(intervalMs: number = 30000) {
    this.stopEvaluation();
    this.evaluationInterval = setInterval(() => {
      // In production, this would receive metrics from the monitoring system
      console.log('Evaluating alert rules...');
    }, intervalMs);
  }

  public stopEvaluation() {
    if (this.evaluationInterval) {
      clearInterval(this.evaluationInterval);
      this.evaluationInterval = null;
    }
  }

  public evaluateAlerts(metrics: MonitoringDashboard): Alert[] {
    const newAlerts: Alert[] = [];

    for (const rule of this.rules) {
      const shouldTrigger = this.evaluateRule(rule, metrics);
      const alertKey = `${rule.id}_${rule.condition}`;
      const existingAlert = this.activeAlerts.get(alertKey);

      if (shouldTrigger && !existingAlert) {
        // Create new alert
        const alert = this.createAlert(rule, metrics);
        this.activeAlerts.set(alertKey, alert);
        this.alertHistory.push(alert);
        newAlerts.push(alert);
        
        // Trigger actions
        this.triggerActions(alert, rule.actions);
      } else if (!shouldTrigger && existingAlert && !existingAlert.resolvedAt) {
        // Resolve existing alert
        existingAlert.resolvedAt = new Date();
        this.triggerResolvedActions(existingAlert, rule.actions);
      }
    }

    return newAlerts;
  }

  private evaluateRule(rule: AlertRule, metrics: MonitoringDashboard): boolean {
    switch (rule.condition) {
      case 'error_rate':
        return this.calculateOverallErrorRate(metrics) > rule.threshold;
      
      case 'build_status':
        return metrics.buildStatus.compilation === 'failed';
      
      case 'memory_usage':
        return metrics.performanceMetrics.memoryUsage.percentage > rule.threshold;
      
      case 'cpu_usage':
        return metrics.performanceMetrics.cpuUtilization > rule.threshold;
      
      case 'conversion_rate':
        return metrics.performanceMetrics.conversionsPerSecond < rule.threshold;
      
      case 'function_error_rate':
        return metrics.legacyFunctions.some(func => func.errorRate > rule.threshold);
      
      default:
        return false;
    }
  }

  private calculateOverallErrorRate(metrics: MonitoringDashboard): number {
    const functions = metrics.legacyFunctions;
    if (functions.length === 0) return 0;
    
    const totalCalls = functions.reduce((sum, func) => sum + func.callCount, 0);
    const totalErrors = functions.reduce((sum, func) => 
      sum + (func.callCount * func.errorRate / 100), 0
    );
    
    return totalCalls > 0 ? (totalErrors / totalCalls) * 100 : 0;
  }

  private createAlert(rule: AlertRule, metrics: MonitoringDashboard): Alert {
    const alert: Alert = {
      id: `alert_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      ruleId: rule.id,
      title: rule.name,
      message: this.generateAlertMessage(rule, metrics),
      severity: rule.severity,
      timestamp: new Date(),
      acknowledged: false,
      metadata: this.collectAlertMetadata(rule, metrics)
    };

    return alert;
  }

  private generateAlertMessage(rule: AlertRule, metrics: MonitoringDashboard): string {
    switch (rule.condition) {
      case 'error_rate':
        const errorRate = this.calculateOverallErrorRate(metrics);
        return `System error rate has exceeded ${rule.threshold}% threshold. Current rate: ${errorRate.toFixed(2)}%`;
      
      case 'build_status':
        return `Build has failed. Errors: ${metrics.buildStatus.errors.length}, Warnings: ${metrics.buildStatus.warnings.length}`;
      
      case 'memory_usage':
        return `Memory usage is critically high at ${metrics.performanceMetrics.memoryUsage.percentage.toFixed(1)}%`;
      
      case 'cpu_usage':
        return `CPU usage is high at ${metrics.performanceMetrics.cpuUtilization.toFixed(1)}%`;
      
      case 'conversion_rate':
        return `Conversion rate has dropped to ${metrics.performanceMetrics.conversionsPerSecond} req/s`;
      
      case 'function_error_rate':
        const problematicFuncs = metrics.legacyFunctions
          .filter(func => func.errorRate > rule.threshold)
          .map(func => `${func.functionName} (${func.errorRate.toFixed(1)}%)`)
          .join(', ');
        return `High error rate detected in functions: ${problematicFuncs}`;
      
      default:
        return `Alert triggered for ${rule.name}`;
    }
  }

  private collectAlertMetadata(rule: AlertRule, metrics: MonitoringDashboard): Record<string, any> {
    const metadata: Record<string, any> = {
      condition: rule.condition,
      threshold: rule.threshold,
      timestamp: new Date().toISOString()
    };

    switch (rule.condition) {
      case 'error_rate':
        metadata.errorRate = this.calculateOverallErrorRate(metrics);
        metadata.functionErrors = metrics.legacyFunctions.map(f => ({
          name: f.functionName,
          errorRate: f.errorRate
        }));
        break;
      
      case 'memory_usage':
        metadata.memoryUsage = metrics.performanceMetrics.memoryUsage;
        break;
      
      case 'cpu_usage':
        metadata.cpuUsage = metrics.performanceMetrics.cpuUtilization;
        break;
      
      case 'build_status':
        metadata.buildStatus = metrics.buildStatus;
        break;
    }

    return metadata;
  }

  private async triggerActions(alert: Alert, actions: AlertAction[]) {
    for (const action of actions) {
      try {
        await this.executeAction(action, alert, 'triggered');
      } catch (error) {
        console.error(`Failed to execute alert action ${action.type}:`, error);
      }
    }
  }

  private async triggerResolvedActions(alert: Alert, actions: AlertAction[]) {
    for (const action of actions) {
      try {
        await this.executeAction(action, alert, 'resolved');
      } catch (error) {
        console.error(`Failed to execute resolved action ${action.type}:`, error);
      }
    }
  }

  private async executeAction(action: AlertAction, alert: Alert, status: 'triggered' | 'resolved') {
    switch (action.type) {
      case 'webhook':
        await this.sendWebhook(action.config.url, {
          alert,
          status,
          timestamp: new Date().toISOString()
        });
        break;
      
      case 'email':
        await this.sendEmail(action.config.recipients, alert, status);
        break;
      
      case 'slack':
        await this.sendSlackNotification(action.config, alert, status);
        break;
    }
  }

  private async sendWebhook(url: string, payload: any) {
    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload)
      });

      if (!response.ok) {
        throw new Error(`Webhook failed with status ${response.status}`);
      }
    } catch (error) {
      console.error('Failed to send webhook:', error);
      throw error;
    }
  }

  private async sendEmail(recipients: string[], alert: Alert, status: 'triggered' | 'resolved') {
    // In production, this would integrate with an email service
    console.log(`Sending email to ${recipients.join(', ')} for alert ${alert.id} (${status})`);
  }

  private async sendSlackNotification(config: any, alert: Alert, status: 'triggered' | 'resolved') {
    const color = status === 'triggered' ? 
      (alert.severity === 'critical' ? '#FF0000' : '#FFA500') : 
      '#00FF00';

    const slackPayload = {
      channel: config.channel || '#alerts',
      attachments: [{
        color,
        title: `${status === 'triggered' ? '🚨' : '✅'} ${alert.title}`,
        text: alert.message,
        fields: [
          {
            title: 'Severity',
            value: alert.severity,
            short: true
          },
          {
            title: 'Status',
            value: status,
            short: true
          }
        ],
        timestamp: Math.floor(alert.timestamp.getTime() / 1000)
      }]
    };

    await this.sendWebhook(config.url || '/api/alerts/slack', slackPayload);
  }

  // Public methods for managing alerts
  public acknowledgeAlert(alertId: string): boolean {
    const alert = Array.from(this.activeAlerts.values()).find(a => a.id === alertId);
    if (alert) {
      alert.acknowledged = true;
      return true;
    }
    return false;
  }

  public getActiveAlerts(): Alert[] {
    return Array.from(this.activeAlerts.values())
      .filter(alert => !alert.resolvedAt)
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());
  }

  public getAlertHistory(limit: number = 100): Alert[] {
    return this.alertHistory
      .slice(-limit)
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());
  }

  public addRule(rule: AlertRule) {
    this.rules.push(rule);
  }

  public removeRule(ruleId: string) {
    this.rules = this.rules.filter(rule => rule.id !== ruleId);
  }

  public getRules(): AlertRule[] {
    return [...this.rules];
  }
}