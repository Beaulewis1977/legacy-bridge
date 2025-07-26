// Enterprise Audit Logging System for LegacyBridge
// Provides comprehensive activity logging for compliance and security

import { EventEmitter } from 'events';
import { TenantContext } from '../tenancy/tenant-context';

export interface AuditLogEntry {
  id: string;
  organizationId: string;
  userId?: string;
  sessionId?: string;
  action: string;
  resourceType: string;
  resourceId?: string;
  details: Record<string, any>;
  beforeData?: Record<string, any>;
  afterData?: Record<string, any>;
  ipAddress?: string;
  userAgent?: string;
  requestId?: string;
  complianceTags?: string[];
  retentionUntil?: Date;
  timestamp: Date;
}

export enum AuditAction {
  // Authentication
  USER_LOGIN = 'user.login',
  USER_LOGOUT = 'user.logout',
  USER_LOGIN_FAILED = 'user.login.failed',
  USER_LOCKED = 'user.locked',
  USER_UNLOCKED = 'user.unlocked',
  PASSWORD_CHANGED = 'user.password.changed',
  PASSWORD_RESET = 'user.password.reset',
  MFA_ENABLED = 'user.mfa.enabled',
  MFA_DISABLED = 'user.mfa.disabled',
  API_KEY_CREATED = 'api.key.created',
  API_KEY_DELETED = 'api.key.deleted',
  
  // User Management
  USER_CREATED = 'user.created',
  USER_UPDATED = 'user.updated',
  USER_DELETED = 'user.deleted',
  USER_SUSPENDED = 'user.suspended',
  USER_ACTIVATED = 'user.activated',
  USER_ROLE_ASSIGNED = 'user.role.assigned',
  USER_ROLE_REMOVED = 'user.role.removed',
  
  // Document Operations
  DOCUMENT_CONVERTED = 'document.converted',
  DOCUMENT_UPLOADED = 'document.uploaded',
  DOCUMENT_DOWNLOADED = 'document.downloaded',
  DOCUMENT_DELETED = 'document.deleted',
  DOCUMENT_SHARED = 'document.shared',
  CONVERSION_FAILED = 'document.conversion.failed',
  
  // Organization Management
  ORGANIZATION_UPDATED = 'organization.updated',
  SUBSCRIPTION_CHANGED = 'organization.subscription.changed',
  SETTINGS_UPDATED = 'organization.settings.updated',
  BRANDING_UPDATED = 'organization.branding.updated',
  
  // Security Events
  PERMISSION_DENIED = 'security.permission.denied',
  RATE_LIMIT_EXCEEDED = 'security.rate.limit.exceeded',
  SUSPICIOUS_ACTIVITY = 'security.suspicious.activity',
  DATA_EXPORT = 'security.data.export',
  
  // Compliance
  COMPLIANCE_REPORT_GENERATED = 'compliance.report.generated',
  DATA_RETENTION_APPLIED = 'compliance.retention.applied',
  GDPR_REQUEST_PROCESSED = 'compliance.gdpr.processed',
}

export enum ResourceType {
  USER = 'user',
  DOCUMENT = 'document',
  ORGANIZATION = 'organization',
  API_KEY = 'api_key',
  ROLE = 'role',
  SETTINGS = 'settings',
  REPORT = 'report',
}

export interface AuditLoggerConfig {
  db: any;
  tenantContext: TenantContext;
  enableRealtime?: boolean;
  batchSize?: number;
  flushInterval?: number;
}

export class AuditLogger extends EventEmitter {
  private db: any;
  private tenantContext: TenantContext;
  private queue: AuditLogEntry[] = [];
  private batchSize: number;
  private flushInterval: number;
  private flushTimer?: NodeJS.Timeout;

  constructor(config: AuditLoggerConfig) {
    super();
    this.db = config.db;
    this.tenantContext = config.tenantContext;
    this.batchSize = config.batchSize || 100;
    this.flushInterval = config.flushInterval || 5000; // 5 seconds
    
    if (config.enableRealtime) {
      this.startFlushTimer();
    }
  }

  // Log an action
  async log(params: {
    action: AuditAction | string;
    resourceType: ResourceType | string;
    resourceId?: string;
    userId?: string;
    details?: Record<string, any>;
    beforeData?: Record<string, any>;
    afterData?: Record<string, any>;
    ipAddress?: string;
    userAgent?: string;
    sessionId?: string;
    requestId?: string;
  }): Promise<void> {
    const entry: AuditLogEntry = {
      id: this.generateId(),
      organizationId: this.tenantContext.organizationId,
      userId: params.userId,
      sessionId: params.sessionId,
      action: params.action,
      resourceType: params.resourceType,
      resourceId: params.resourceId,
      details: params.details || {},
      beforeData: params.beforeData,
      afterData: params.afterData,
      ipAddress: params.ipAddress,
      userAgent: params.userAgent,
      requestId: params.requestId,
      complianceTags: this.generateComplianceTags(params.action, params.resourceType),
      retentionUntil: this.calculateRetentionDate(),
      timestamp: new Date(),
    };

    // Apply audit level filtering
    if (!this.shouldLog(entry)) {
      return;
    }

    // Add to queue
    this.queue.push(entry);

    // Emit real-time event
    this.emit('audit_event', entry);

    // Flush if batch size reached
    if (this.queue.length >= this.batchSize) {
      await this.flush();
    }
  }

  // Convenience methods for common actions
  async logLogin(userId: string, success: boolean, context: any): Promise<void> {
    await this.log({
      action: success ? AuditAction.USER_LOGIN : AuditAction.USER_LOGIN_FAILED,
      resourceType: ResourceType.USER,
      resourceId: userId,
      userId: success ? userId : undefined,
      details: {
        success,
        method: context.method || 'password',
        mfaUsed: context.mfaUsed || false,
      },
      ipAddress: context.ipAddress,
      userAgent: context.userAgent,
      sessionId: context.sessionId,
    });
  }

  async logDocumentConversion(
    userId: string,
    documentId: string,
    conversionType: string,
    success: boolean,
    details?: any
  ): Promise<void> {
    await this.log({
      action: success ? AuditAction.DOCUMENT_CONVERTED : AuditAction.CONVERSION_FAILED,
      resourceType: ResourceType.DOCUMENT,
      resourceId: documentId,
      userId,
      details: {
        conversionType,
        success,
        ...details,
      },
    });
  }

  async logPermissionDenied(
    userId: string,
    resource: string,
    action: string,
    context: any
  ): Promise<void> {
    await this.log({
      action: AuditAction.PERMISSION_DENIED,
      resourceType: resource,
      userId,
      details: {
        requestedAction: action,
        reason: 'Insufficient permissions',
      },
      ipAddress: context.ipAddress,
      userAgent: context.userAgent,
    });
  }

  async logDataExport(userId: string, exportType: string, filters: any): Promise<void> {
    await this.log({
      action: AuditAction.DATA_EXPORT,
      resourceType: ResourceType.REPORT,
      userId,
      details: {
        exportType,
        filters,
        recordCount: filters.count || 0,
      },
    });
  }

  // Search audit logs
  async search(params: {
    startDate?: Date;
    endDate?: Date;
    userId?: string;
    action?: string;
    resourceType?: string;
    resourceId?: string;
    limit?: number;
    offset?: number;
  }): Promise<{ logs: AuditLogEntry[]; total: number }> {
    let query = `
      SELECT *, COUNT(*) OVER() as total_count
      FROM audit_logs
      WHERE organization_id = $1
    `;
    const queryParams: any[] = [this.tenantContext.organizationId];
    let paramIndex = 2;

    if (params.startDate) {
      query += ` AND timestamp >= $${paramIndex}`;
      queryParams.push(params.startDate);
      paramIndex++;
    }

    if (params.endDate) {
      query += ` AND timestamp <= $${paramIndex}`;
      queryParams.push(params.endDate);
      paramIndex++;
    }

    if (params.userId) {
      query += ` AND user_id = $${paramIndex}`;
      queryParams.push(params.userId);
      paramIndex++;
    }

    if (params.action) {
      query += ` AND action = $${paramIndex}`;
      queryParams.push(params.action);
      paramIndex++;
    }

    if (params.resourceType) {
      query += ` AND resource_type = $${paramIndex}`;
      queryParams.push(params.resourceType);
      paramIndex++;
    }

    if (params.resourceId) {
      query += ` AND resource_id = $${paramIndex}`;
      queryParams.push(params.resourceId);
      paramIndex++;
    }

    query += ` ORDER BY timestamp DESC`;

    if (params.limit) {
      query += ` LIMIT $${paramIndex}`;
      queryParams.push(params.limit);
      paramIndex++;
    }

    if (params.offset) {
      query += ` OFFSET $${paramIndex}`;
      queryParams.push(params.offset);
    }

    const result = await this.db.query(query, queryParams);
    
    return {
      logs: result.rows.map((row: any) => this.mapAuditLog(row)),
      total: result.rows.length > 0 ? parseInt(result.rows[0].total_count) : 0,
    };
  }

  // Generate compliance report
  async generateComplianceReport(params: {
    startDate: Date;
    endDate: Date;
    reportType: 'SOX' | 'GDPR' | 'HIPAA' | 'GENERAL';
  }): Promise<any> {
    const relevantActions = this.getComplianceActions(params.reportType);
    
    const result = await this.db.query(`
      SELECT 
        action,
        resource_type,
        COUNT(*) as count,
        COUNT(DISTINCT user_id) as unique_users,
        json_agg(DISTINCT compliance_tags) as tags
      FROM audit_logs
      WHERE organization_id = $1
        AND timestamp BETWEEN $2 AND $3
        AND action = ANY($4)
      GROUP BY action, resource_type
      ORDER BY count DESC
    `, [
      this.tenantContext.organizationId,
      params.startDate,
      params.endDate,
      relevantActions,
    ]);

    // Log the report generation
    await this.log({
      action: AuditAction.COMPLIANCE_REPORT_GENERATED,
      resourceType: ResourceType.REPORT,
      details: {
        reportType: params.reportType,
        startDate: params.startDate,
        endDate: params.endDate,
        actionCount: result.rows.reduce((sum: number, row: any) => sum + parseInt(row.count), 0),
      },
    });

    return {
      reportType: params.reportType,
      period: {
        start: params.startDate,
        end: params.endDate,
      },
      summary: result.rows,
      generatedAt: new Date(),
    };
  }

  // Private methods
  private shouldLog(entry: AuditLogEntry): boolean {
    const level = this.tenantContext.compliance.auditLogLevel;
    
    // Basic level - only critical actions
    if (level === 'basic') {
      return [
        AuditAction.USER_LOGIN,
        AuditAction.USER_DELETED,
        AuditAction.PERMISSION_DENIED,
        AuditAction.DATA_EXPORT,
        AuditAction.SUSPICIOUS_ACTIVITY,
      ].includes(entry.action as AuditAction);
    }
    
    // Detailed level - most actions except reads
    if (level === 'detailed') {
      return !entry.action.includes('.read') && !entry.action.includes('.list');
    }
    
    // Forensic level - everything
    return true;
  }

  private generateComplianceTags(action: string, resourceType: string): string[] {
    const tags: string[] = [];
    
    // GDPR tags
    if (resourceType === ResourceType.USER || action.includes('data.export')) {
      tags.push('GDPR');
    }
    
    // SOX tags
    if (action.includes('role') || action.includes('permission') || action.includes('audit')) {
      tags.push('SOX');
    }
    
    // HIPAA tags (if applicable)
    if (resourceType === ResourceType.DOCUMENT && this.tenantContext.hasFeature('hipaa_compliance')) {
      tags.push('HIPAA');
    }
    
    return tags;
  }

  private calculateRetentionDate(): Date {
    const retentionDays = this.tenantContext.compliance.dataRetentionDays;
    const date = new Date();
    date.setDate(date.getDate() + retentionDays);
    return date;
  }

  private getComplianceActions(reportType: string): string[] {
    switch (reportType) {
      case 'SOX':
        return [
          AuditAction.USER_ROLE_ASSIGNED,
          AuditAction.USER_ROLE_REMOVED,
          AuditAction.PERMISSION_DENIED,
          AuditAction.SETTINGS_UPDATED,
        ];
      case 'GDPR':
        return [
          AuditAction.USER_CREATED,
          AuditAction.USER_DELETED,
          AuditAction.DATA_EXPORT,
          AuditAction.GDPR_REQUEST_PROCESSED,
        ];
      case 'HIPAA':
        return [
          AuditAction.DOCUMENT_CONVERTED,
          AuditAction.DOCUMENT_DOWNLOADED,
          AuditAction.DOCUMENT_SHARED,
          AuditAction.PERMISSION_DENIED,
        ];
      default:
        return Object.values(AuditAction);
    }
  }

  private async flush(): Promise<void> {
    if (this.queue.length === 0) return;

    const entries = [...this.queue];
    this.queue = [];

    try {
      // Batch insert
      const values = entries.map((entry, i) => {
        const base = i * 11;
        return `($${base + 1}, $${base + 2}, $${base + 3}, $${base + 4}, $${base + 5}, $${base + 6}, $${base + 7}, $${base + 8}, $${base + 9}, $${base + 10}, $${base + 11})`;
      }).join(',');

      const params = entries.flatMap(entry => [
        entry.organizationId,
        entry.userId,
        entry.action,
        entry.resourceType,
        entry.resourceId,
        JSON.stringify(entry.details),
        entry.ipAddress,
        entry.userAgent,
        entry.requestId,
        entry.complianceTags,
        entry.timestamp,
      ]);

      await this.db.query(`
        INSERT INTO audit_logs (
          organization_id, user_id, action, resource_type, resource_id,
          details, ip_address, user_agent, request_id, compliance_tags, timestamp
        ) VALUES ${values}
      `, params);
    } catch (error) {
      console.error('Failed to flush audit logs:', error);
      // Re-queue failed entries
      this.queue.unshift(...entries);
    }
  }

  private startFlushTimer(): void {
    this.flushTimer = setInterval(() => {
      this.flush();
    }, this.flushInterval);
  }

  private generateId(): string {
    return `audit_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private mapAuditLog(row: any): AuditLogEntry {
    return {
      id: row.id,
      organizationId: row.organization_id,
      userId: row.user_id,
      sessionId: row.session_id,
      action: row.action,
      resourceType: row.resource_type,
      resourceId: row.resource_id,
      details: row.details,
      beforeData: row.before_data,
      afterData: row.after_data,
      ipAddress: row.ip_address,
      userAgent: row.user_agent,
      requestId: row.request_id,
      complianceTags: row.compliance_tags,
      retentionUntil: row.retention_until,
      timestamp: row.timestamp,
    };
  }

  // Cleanup
  destroy(): void {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
    }
    this.flush();
  }
}

// Audit middleware for Express/Next.js
export function auditMiddleware(db: any) {
  return async (req: any, res: any, next: any) => {
    if (req.tenantContext) {
      req.auditLogger = new AuditLogger({
        db,
        tenantContext: req.tenantContext,
        enableRealtime: true,
      });

      // Generate request ID
      req.requestId = `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

      // Log response after processing
      const originalSend = res.send;
      res.send = function(data: any) {
        res.send = originalSend;
        
        // Log based on response status
        if (res.statusCode === 403) {
          req.auditLogger?.logPermissionDenied(
            req.user?.userId,
            req.path.split('/')[1] || 'unknown',
            req.method,
            {
              ipAddress: req.ip,
              userAgent: req.get('user-agent'),
            }
          );
        }
        
        return res.send(data);
      };
    }
    next();
  };
}

export default AuditLogger;