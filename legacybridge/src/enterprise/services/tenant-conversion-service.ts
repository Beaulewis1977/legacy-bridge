// Tenant-Aware Conversion Service for LegacyBridge Enterprise
// Handles document conversions with multi-tenant isolation and rate limiting

import { TenantContext, TenantAwareService } from '../tenancy/tenant-context';
import { AuditLogger, AuditAction, ResourceType } from '../audit/audit-logger';
import { Queue, Worker, Job } from 'bullmq';
import * as Redis from 'ioredis';
import { RateLimiter } from 'limiter';
import { EventEmitter } from 'events';

export interface ConversionJob {
  id: string;
  organizationId: string;
  userId: string;
  inputFileName: string;
  inputFileSize: number;
  inputFileHash?: string;
  conversionType: 'rtf_to_md' | 'md_to_rtf';
  priority: 'low' | 'normal' | 'high';
  options?: ConversionOptions;
  createdAt: Date;
}

export interface ConversionOptions {
  preserveFormatting?: boolean;
  includeMetadata?: boolean;
  customStyles?: Record<string, any>;
  outputFormat?: string;
}

export interface ConversionResult {
  jobId: string;
  success: boolean;
  outputFileName?: string;
  outputFileSize?: number;
  outputFileHash?: string;
  outputPath?: string;
  processingTimeMs?: number;
  error?: string;
  metadata?: Record<string, any>;
}

export interface ConversionProgress {
  jobId: string;
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'canceled';
  progress: number;
  currentStep?: string;
  estimatedTimeRemaining?: number;
}

export class TenantAwareConversionService extends TenantAwareService {
  private queue: Queue;
  private worker: Worker;
  private rateLimiters: Map<string, RateLimiter>;
  private progressEmitter: EventEmitter;
  private redis: Redis.Redis;

  constructor(
    tenantContext: TenantContext,
    private db: any,
    private auditLogger: AuditLogger,
    redisConfig: any
  ) {
    super(tenantContext);
    
    this.redis = new Redis.Redis(redisConfig);
    this.progressEmitter = new EventEmitter();
    this.rateLimiters = new Map();
    
    // Initialize queue
    this.queue = new Queue(`conversions-${tenantContext.organizationId}`, {
      connection: this.redis,
      defaultJobOptions: {
        attempts: 3,
        backoff: {
          type: 'exponential',
          delay: 2000,
        },
        removeOnComplete: {
          age: 24 * 3600, // 24 hours
          count: 1000,
        },
        removeOnFail: {
          age: 7 * 24 * 3600, // 7 days
        },
      },
    });

    // Initialize worker
    this.worker = new Worker(
      `conversions-${tenantContext.organizationId}`,
      async (job: Job) => this.processConversion(job),
      {
        connection: this.redis,
        concurrency: this.tenantContext.limits.maxConcurrentJobs,
        limiter: {
          max: this.tenantContext.limits.maxConcurrentJobs,
          duration: 60000, // per minute
        },
      }
    );

    this.setupWorkerHandlers();
  }

  // Submit conversion job
  async submitConversion(
    userId: string,
    filePath: string,
    conversionType: 'rtf_to_md' | 'md_to_rtf',
    options?: ConversionOptions
  ): Promise<string> {
    // Validate file size
    const fileInfo = await this.getFileInfo(filePath);
    this.validateLimit('maxFileSizeMB', fileInfo.size / (1024 * 1024));

    // Check rate limit
    await this.checkRateLimit(userId);

    // Check concurrent jobs limit
    const activeJobs = await this.getActiveJobsCount();
    if (!this.tenantContext.isWithinLimit('maxConcurrentJobs', activeJobs)) {
      throw new Error('Concurrent job limit reached. Please try again later.');
    }

    // Create job record in database
    const jobResult = await this.db.query(`
      INSERT INTO conversion_jobs (
        organization_id, user_id, input_file_name, input_file_size,
        input_file_hash, conversion_type, status, input_storage_path
      ) VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7)
      RETURNING id
    `, [
      this.tenantContext.organizationId,
      userId,
      fileInfo.name,
      fileInfo.size,
      fileInfo.hash,
      conversionType,
      filePath,
    ]);

    const jobId = jobResult.rows[0].id;

    // Add to queue
    const priority = this.calculatePriority(fileInfo.size);
    await this.queue.add('convert', {
      jobId,
      organizationId: this.tenantContext.organizationId,
      userId,
      filePath,
      conversionType,
      options,
      fileInfo,
    }, {
      priority: this.getPriorityValue(priority),
      delay: this.calculateDelay(),
    });

    // Audit log
    await this.auditLogger.log({
      action: AuditAction.DOCUMENT_UPLOADED,
      resourceType: ResourceType.DOCUMENT,
      resourceId: jobId,
      userId,
      details: {
        fileName: fileInfo.name,
        fileSize: fileInfo.size,
        conversionType,
      },
    });

    return jobId;
  }

  // Get conversion status
  async getConversionStatus(jobId: string): Promise<ConversionProgress> {
    // Check database
    const result = await this.db.query(`
      SELECT status, progress FROM conversion_jobs
      WHERE id = $1 AND organization_id = $2
    `, [jobId, this.tenantContext.organizationId]);

    if (result.rows.length === 0) {
      throw new Error('Job not found');
    }

    const job = result.rows[0];
    
    // Get queue job for additional details
    const queueJob = await this.queue.getJob(jobId);
    
    return {
      jobId,
      status: job.status,
      progress: job.progress || 0,
      currentStep: queueJob?.data?.currentStep,
      estimatedTimeRemaining: await this.estimateTimeRemaining(jobId),
    };
  }

  // Cancel conversion
  async cancelConversion(jobId: string, userId: string): Promise<void> {
    // Verify ownership
    const result = await this.db.query(`
      SELECT user_id, status FROM conversion_jobs
      WHERE id = $1 AND organization_id = $2
    `, [jobId, this.tenantContext.organizationId]);

    if (result.rows.length === 0) {
      throw new Error('Job not found');
    }

    const job = result.rows[0];
    
    if (job.status !== 'pending' && job.status !== 'processing') {
      throw new Error('Job cannot be canceled');
    }

    // Remove from queue
    const queueJob = await this.queue.getJob(jobId);
    if (queueJob) {
      await queueJob.remove();
    }

    // Update database
    await this.db.query(`
      UPDATE conversion_jobs
      SET status = 'canceled', completed_at = NOW()
      WHERE id = $1
    `, [jobId]);

    // Audit log
    await this.auditLogger.log({
      action: 'document.conversion.canceled',
      resourceType: ResourceType.DOCUMENT,
      resourceId: jobId,
      userId,
    });
  }

  // Get conversion history
  async getConversionHistory(
    userId?: string,
    filters?: {
      startDate?: Date;
      endDate?: Date;
      status?: string;
      conversionType?: string;
    },
    pagination?: {
      page: number;
      limit: number;
    }
  ): Promise<{ jobs: any[]; total: number }> {
    let query = `
      SELECT 
        j.*,
        u.email as user_email,
        u.first_name || ' ' || u.last_name as user_name
      FROM conversion_jobs j
      LEFT JOIN users u ON u.id = j.user_id
      WHERE j.organization_id = $1
    `;
    const params: any[] = [this.tenantContext.organizationId];
    let paramIndex = 2;

    if (userId) {
      query += ` AND j.user_id = $${paramIndex}`;
      params.push(userId);
      paramIndex++;
    }

    if (filters?.startDate) {
      query += ` AND j.created_at >= $${paramIndex}`;
      params.push(filters.startDate);
      paramIndex++;
    }

    if (filters?.endDate) {
      query += ` AND j.created_at <= $${paramIndex}`;
      params.push(filters.endDate);
      paramIndex++;
    }

    if (filters?.status) {
      query += ` AND j.status = $${paramIndex}`;
      params.push(filters.status);
      paramIndex++;
    }

    if (filters?.conversionType) {
      query += ` AND j.conversion_type = $${paramIndex}`;
      params.push(filters.conversionType);
      paramIndex++;
    }

    // Count total
    const countResult = await this.db.query(
      `SELECT COUNT(*) FROM (${query}) as counted`,
      params
    );
    const total = parseInt(countResult.rows[0].count);

    // Add pagination
    query += ` ORDER BY j.created_at DESC`;
    
    if (pagination) {
      const offset = (pagination.page - 1) * pagination.limit;
      query += ` LIMIT $${paramIndex} OFFSET $${paramIndex + 1}`;
      params.push(pagination.limit, offset);
    }

    const result = await this.db.query(query, params);

    return {
      jobs: result.rows,
      total,
    };
  }

  // Process conversion (worker)
  private async processConversion(job: Job): Promise<ConversionResult> {
    const { jobId, userId, filePath, conversionType, options, fileInfo } = job.data;

    try {
      // Update job status
      await this.updateJobStatus(jobId, 'processing');
      const startTime = Date.now();

      // Update progress
      await this.updateProgress(job, 10, 'Loading file');

      // Load file
      const fileContent = await this.loadFile(filePath);
      await this.updateProgress(job, 20, 'Validating content');

      // Validate content
      await this.validateContent(fileContent, conversionType);
      await this.updateProgress(job, 30, 'Converting');

      // Perform conversion
      let result: any;
      if (conversionType === 'rtf_to_md') {
        result = await this.convertRtfToMarkdown(fileContent, options);
      } else {
        result = await this.convertMarkdownToRtf(fileContent, options);
      }

      await this.updateProgress(job, 80, 'Saving output');

      // Save output
      const outputPath = await this.saveOutput(
        jobId,
        result.content,
        result.metadata
      );

      await this.updateProgress(job, 90, 'Finalizing');

      // Calculate metrics
      const processingTime = Date.now() - startTime;

      // Update database
      await this.db.query(`
        UPDATE conversion_jobs
        SET 
          status = 'completed',
          output_file_name = $2,
          output_file_size = $3,
          output_file_hash = $4,
          output_storage_path = $5,
          processing_time_ms = $6,
          completed_at = NOW(),
          progress = 100
        WHERE id = $1
      `, [
        jobId,
        result.fileName,
        result.fileSize,
        result.fileHash,
        outputPath,
        processingTime,
      ]);

      // Audit log
      await this.auditLogger.logDocumentConversion(
        userId,
        jobId,
        conversionType,
        true,
        {
          processingTimeMs: processingTime,
          outputSize: result.fileSize,
        }
      );

      await this.updateProgress(job, 100, 'Completed');

      return {
        jobId,
        success: true,
        outputFileName: result.fileName,
        outputFileSize: result.fileSize,
        outputFileHash: result.fileHash,
        outputPath,
        processingTimeMs: processingTime,
        metadata: result.metadata,
      };

    } catch (error: any) {
      // Update database
      await this.db.query(`
        UPDATE conversion_jobs
        SET 
          status = 'failed',
          error_message = $2,
          error_details = $3,
          completed_at = NOW()
        WHERE id = $1
      `, [
        jobId,
        error.message,
        JSON.stringify({ stack: error.stack }),
      ]);

      // Audit log
      await this.auditLogger.logDocumentConversion(
        userId,
        jobId,
        conversionType,
        false,
        { error: error.message }
      );

      throw error;
    }
  }

  // Helper methods
  private async checkRateLimit(userId: string): Promise<void> {
    const key = `${this.tenantContext.organizationId}:${userId}`;
    
    if (!this.rateLimiters.has(key)) {
      this.rateLimiters.set(key, new RateLimiter({
        tokensPerInterval: this.tenantContext.limits.maxApiCallsPerMinute,
        interval: 'minute',
      }));
    }

    const limiter = this.rateLimiters.get(key)!;
    const hasToken = await limiter.tryRemoveTokens(1);
    
    if (!hasToken) {
      throw new Error('Rate limit exceeded. Please try again later.');
    }
  }

  private async getActiveJobsCount(): Promise<number> {
    const result = await this.db.query(`
      SELECT COUNT(*) FROM conversion_jobs
      WHERE organization_id = $1 
        AND status IN ('pending', 'processing')
    `, [this.tenantContext.organizationId]);
    
    return parseInt(result.rows[0].count);
  }

  private calculatePriority(fileSize: number): 'low' | 'normal' | 'high' {
    // Enterprise tier gets high priority
    if (this.tenantContext.subscriptionTier.name === 'enterprise') {
      return 'high';
    }
    
    // Professional tier gets normal/high based on file size
    if (this.tenantContext.subscriptionTier.name === 'professional') {
      return fileSize < 10 * 1024 * 1024 ? 'high' : 'normal';
    }
    
    // Basic tier gets normal/low
    return fileSize < 5 * 1024 * 1024 ? 'normal' : 'low';
  }

  private getPriorityValue(priority: 'low' | 'normal' | 'high'): number {
    switch (priority) {
      case 'high': return 1;
      case 'normal': return 5;
      case 'low': return 10;
    }
  }

  private calculateDelay(): number {
    // No delay for enterprise
    if (this.tenantContext.subscriptionTier.name === 'enterprise') {
      return 0;
    }
    
    // Small delay for professional during peak hours
    if (this.tenantContext.subscriptionTier.name === 'professional') {
      const hour = new Date().getHours();
      return (hour >= 9 && hour <= 17) ? 1000 : 0; // 1 second during business hours
    }
    
    // Basic tier gets longer delays
    return 5000; // 5 seconds
  }

  private async getFileInfo(filePath: string): Promise<any> {
    // Implementation would get actual file info
    // This is a placeholder
    return {
      name: filePath.split('/').pop(),
      size: 1024 * 1024, // 1MB
      hash: 'hash123',
    };
  }

  private async updateJobStatus(jobId: string, status: string): Promise<void> {
    await this.db.query(`
      UPDATE conversion_jobs
      SET status = $2, started_at = COALESCE(started_at, NOW())
      WHERE id = $1
    `, [jobId, status]);
  }

  private async updateProgress(job: Job, progress: number, step: string): Promise<void> {
    await job.updateProgress(progress);
    
    await this.db.query(`
      UPDATE conversion_jobs SET progress = $2 WHERE id = $1
    `, [job.data.jobId, progress]);
    
    this.progressEmitter.emit('progress', {
      jobId: job.data.jobId,
      progress,
      step,
    });
  }

  private async loadFile(filePath: string): Promise<string> {
    // Implementation would load actual file
    return 'file content';
  }

  private async validateContent(content: string, type: string): Promise<void> {
    // Implementation would validate content
    if (!content) {
      throw new Error('Empty content');
    }
  }

  private async convertRtfToMarkdown(content: string, options?: any): Promise<any> {
    // Implementation would perform actual conversion
    return {
      content: '# Converted Markdown\n\nContent here...',
      fileName: 'output.md',
      fileSize: 1024,
      fileHash: 'hash456',
      metadata: { convertedAt: new Date() },
    };
  }

  private async convertMarkdownToRtf(content: string, options?: any): Promise<any> {
    // Implementation would perform actual conversion
    return {
      content: '{\\rtf1 Converted RTF content}',
      fileName: 'output.rtf',
      fileSize: 2048,
      fileHash: 'hash789',
      metadata: { convertedAt: new Date() },
    };
  }

  private async saveOutput(jobId: string, content: string, metadata: any): Promise<string> {
    // Implementation would save to actual storage
    const path = `/storage/${this.tenantContext.organizationId}/${jobId}/output`;
    return path;
  }

  private async estimateTimeRemaining(jobId: string): Promise<number> {
    // Implementation would calculate based on historical data
    return 30; // seconds
  }

  private setupWorkerHandlers(): void {
    this.worker.on('completed', (job) => {
      console.log(`Job ${job.id} completed`);
    });

    this.worker.on('failed', (job, err) => {
      console.error(`Job ${job?.id} failed:`, err);
    });

    this.worker.on('progress', (job, progress) => {
      console.log(`Job ${job.id} progress: ${progress}%`);
    });
  }

  // Subscribe to progress updates
  onProgress(callback: (progress: ConversionProgress) => void): void {
    this.progressEmitter.on('progress', callback);
  }

  // Cleanup
  async destroy(): Promise<void> {
    await this.queue.close();
    await this.worker.close();
    await this.redis.quit();
  }
}

export default TenantAwareConversionService;