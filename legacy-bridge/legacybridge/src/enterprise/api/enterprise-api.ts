// Enterprise API Endpoints for LegacyBridge
// RESTful API with full RBAC, rate limiting, and tenant isolation

import { Router } from 'express';
import { TenantContextProvider } from '../tenancy/tenant-context';
import { RBACService, RequirePermission } from '../auth/rbac';
import { AuthenticationService, jwtMiddleware, apiKeyMiddleware } from '../auth/authentication';
import { AuditLogger, AuditAction, ResourceType } from '../audit/audit-logger';
import { TenantAwareConversionService } from '../services/tenant-conversion-service';
import { tenantContextMiddleware } from '../tenancy/tenant-context';
import { auditMiddleware } from '../audit/audit-logger';
import { rbacMiddleware } from '../auth/rbac';
import multer from 'multer';
import { RateLimiterRedis } from 'rate-limiter-flexible';
import Redis from 'ioredis';

// Initialize services
const initializeServices = (config: any) => {
  const redis = new Redis(config.redis);
  const authService = new AuthenticationService(config.db, config.auth);
  
  // Rate limiter
  const rateLimiter = new RateLimiterRedis({
    storeClient: redis,
    keyPrefix: 'rl',
    points: 100, // requests
    duration: 60, // per minute
    blockDuration: 60, // block for 1 minute
  });

  // File upload configuration
  const upload = multer({
    storage: multer.memoryStorage(),
    limits: {
      fileSize: 1000 * 1024 * 1024, // 1GB max
    },
    fileFilter: (req, file, cb) => {
      const allowedTypes = ['text/rtf', 'text/markdown', 'text/plain'];
      if (allowedTypes.includes(file.mimetype)) {
        cb(null, true);
      } else {
        cb(new Error('Invalid file type'));
      }
    },
  });

  return { authService, rateLimiter, upload };
};

// Rate limiting middleware
const rateLimitMiddleware = (rateLimiter: RateLimiterRedis) => {
  return async (req: any, res: any, next: any) => {
    try {
      const key = req.user?.userId || req.ip;
      await rateLimiter.consume(key);
      next();
    } catch (rejRes: any) {
      res.status(429).json({
        error: 'Too many requests',
        retryAfter: Math.round(rejRes.msBeforeNext / 1000) || 60,
      });
    }
  };
};

// Create API router
export function createEnterpriseAPI(config: any): Router {
  const router = Router();
  const { authService, rateLimiter, upload } = initializeServices(config);

  // Apply global middleware
  router.use(apiKeyMiddleware(authService));
  router.use(jwtMiddleware(config.auth));
  router.use(tenantContextMiddleware);
  router.use(rbacMiddleware(config.db));
  router.use(auditMiddleware(config.db));
  router.use(rateLimitMiddleware(rateLimiter));

  // Health check endpoints
  router.get('/health', (req, res) => {
    res.json({ status: 'healthy', timestamp: new Date() });
  });

  router.get('/ready', async (req, res) => {
    try {
      // Check database connection
      await req.db.query('SELECT 1');
      res.json({ status: 'ready', timestamp: new Date() });
    } catch (error) {
      res.status(503).json({ status: 'not ready', error: error.message });
    }
  });

  // Authentication endpoints
  router.post('/auth/login', async (req, res) => {
    try {
      const { email, password, mfaCode } = req.body;
      const tokens = await authService.login({
        email,
        password,
        mfaCode,
        organizationId: req.tenantContext.organizationId,
        ipAddress: req.ip,
        userAgent: req.get('user-agent'),
      });
      
      res.json(tokens);
    } catch (error: any) {
      res.status(401).json({ error: error.message });
    }
  });

  router.post('/auth/refresh', async (req, res) => {
    try {
      const { refreshToken } = req.body;
      const tokens = await authService.refreshToken(refreshToken);
      res.json(tokens);
    } catch (error: any) {
      res.status(401).json({ error: error.message });
    }
  });

  router.post('/auth/logout', async (req, res) => {
    try {
      const token = req.headers.authorization?.replace('Bearer ', '');
      if (token) {
        await authService.logout(token);
      }
      res.json({ success: true });
    } catch (error: any) {
      res.status(500).json({ error: error.message });
    }
  });

  // User management endpoints
  router.get('/users', RequirePermission('users', 'list'), async (req, res) => {
    try {
      const { page = 1, limit = 20, search, role, status } = req.query;
      
      let query = `
        SELECT 
          u.id, u.email, u.first_name, u.last_name, u.status,
          u.created_at, u.last_login_at,
          json_agg(DISTINCT r.name) as roles
        FROM users u
        LEFT JOIN user_roles ur ON ur.user_id = u.id
        LEFT JOIN roles r ON r.id = ur.role_id
        WHERE u.organization_id = $1 AND u.deleted_at IS NULL
      `;
      const params: any[] = [req.tenantContext.organizationId];
      let paramIndex = 2;

      if (search) {
        query += ` AND (u.email ILIKE $${paramIndex} OR u.first_name ILIKE $${paramIndex} OR u.last_name ILIKE $${paramIndex})`;
        params.push(`%${search}%`);
        paramIndex++;
      }

      if (status) {
        query += ` AND u.status = $${paramIndex}`;
        params.push(status);
        paramIndex++;
      }

      query += ` GROUP BY u.id ORDER BY u.created_at DESC`;
      query += ` LIMIT $${paramIndex} OFFSET $${paramIndex + 1}`;
      params.push(limit, (page - 1) * limit);

      const result = await req.db.query(query, params);
      
      res.json({
        users: result.rows,
        pagination: {
          page: parseInt(page),
          limit: parseInt(limit),
          total: result.rowCount,
        },
      });
    } catch (error: any) {
      res.status(500).json({ error: error.message });
    }
  });

  router.post('/users', RequirePermission('users', 'create'), async (req, res) => {
    try {
      const { email, firstName, lastName, roleIds } = req.body;
      
      // Create user
      const userResult = await req.db.query(`
        INSERT INTO users (
          organization_id, email, first_name, last_name, status
        ) VALUES ($1, $2, $3, $4, 'pending')
        RETURNING id
      `, [req.tenantContext.organizationId, email, firstName, lastName]);

      const userId = userResult.rows[0].id;

      // Assign roles
      if (roleIds && roleIds.length > 0) {
        for (const roleId of roleIds) {
          await req.rbac.assignRole(userId, roleId, req.user.userId);
        }
      }

      // Audit log
      await req.auditLogger.log({
        action: AuditAction.USER_CREATED,
        resourceType: ResourceType.USER,
        resourceId: userId,
        userId: req.user.userId,
        details: { email, roles: roleIds },
      });

      res.json({ id: userId, email });
    } catch (error: any) {
      res.status(500).json({ error: error.message });
    }
  });

  // Document conversion endpoints
  router.post('/conversions/upload', 
    RequirePermission('documents', 'create'),
    upload.single('file'),
    async (req, res) => {
      try {
        if (!req.file) {
          return res.status(400).json({ error: 'No file uploaded' });
        }

        const { conversionType, options } = req.body;
        
        // Initialize conversion service
        const conversionService = new TenantAwareConversionService(
          req.tenantContext,
          req.db,
          req.auditLogger,
          config.redis
        );

        // Save file temporarily
        const filePath = `/tmp/${req.file.originalname}`;
        // In production, save to S3 or similar
        
        // Submit conversion
        const jobId = await conversionService.submitConversion(
          req.user.userId,
          filePath,
          conversionType,
          options ? JSON.parse(options) : undefined
        );

        res.json({ jobId, status: 'pending' });
      } catch (error: any) {
        res.status(500).json({ error: error.message });
      }
    }
  );

  router.get('/conversions/:jobId/status', 
    RequirePermission('documents', 'read'),
    async (req, res) => {
      try {
        const conversionService = new TenantAwareConversionService(
          req.tenantContext,
          req.db,
          req.auditLogger,
          config.redis
        );

        const status = await conversionService.getConversionStatus(req.params.jobId);
        res.json(status);
      } catch (error: any) {
        res.status(404).json({ error: error.message });
      }
    }
  );

  router.get('/conversions/history',
    RequirePermission('documents', 'list'),
    async (req, res) => {
      try {
        const { page = 1, limit = 20, startDate, endDate, status } = req.query;
        
        const conversionService = new TenantAwareConversionService(
          req.tenantContext,
          req.db,
          req.auditLogger,
          config.redis
        );

        const history = await conversionService.getConversionHistory(
          req.user.isAdmin ? undefined : req.user.userId,
          {
            startDate: startDate ? new Date(startDate as string) : undefined,
            endDate: endDate ? new Date(endDate as string) : undefined,
            status: status as string,
          },
          {
            page: parseInt(page as string),
            limit: parseInt(limit as string),
          }
        );

        res.json(history);
      } catch (error: any) {
        res.status(500).json({ error: error.message });
      }
    }
  );

  // Organization management endpoints
  router.get('/organization',
    RequirePermission('settings', 'read'),
    async (req, res) => {
      try {
        const result = await req.db.query(`
          SELECT 
            o.*,
            COUNT(DISTINCT u.id) as user_count,
            COUNT(DISTINCT j.id) as total_conversions,
            SUM(j.input_file_size) as total_storage_bytes
          FROM organizations o
          LEFT JOIN users u ON u.organization_id = o.id AND u.deleted_at IS NULL
          LEFT JOIN conversion_jobs j ON j.organization_id = o.id
          WHERE o.id = $1
          GROUP BY o.id
        `, [req.tenantContext.organizationId]);

        res.json(result.rows[0]);
      } catch (error: any) {
        res.status(500).json({ error: error.message });
      }
    }
  );

  router.patch('/organization',
    RequirePermission('settings', 'update'),
    async (req, res) => {
      try {
        const updates = req.body;
        const allowedFields = ['name', 'logo_url', 'primary_color'];
        
        const setClause = Object.keys(updates)
          .filter(key => allowedFields.includes(key))
          .map((key, i) => `${key} = $${i + 2}`)
          .join(', ');

        if (!setClause) {
          return res.status(400).json({ error: 'No valid fields to update' });
        }

        const values = Object.keys(updates)
          .filter(key => allowedFields.includes(key))
          .map(key => updates[key]);

        await req.db.query(`
          UPDATE organizations 
          SET ${setClause}, updated_at = NOW()
          WHERE id = $1
        `, [req.tenantContext.organizationId, ...values]);

        // Audit log
        await req.auditLogger.log({
          action: AuditAction.ORGANIZATION_UPDATED,
          resourceType: ResourceType.ORGANIZATION,
          resourceId: req.tenantContext.organizationId,
          userId: req.user.userId,
          beforeData: req.tenantContext.organization,
          afterData: updates,
        });

        // Clear cache
        TenantContextProvider.clearContext(req.tenantContext.organizationId);

        res.json({ success: true });
      } catch (error: any) {
        res.status(500).json({ error: error.message });
      }
    }
  );

  // Analytics endpoints
  router.get('/analytics/usage',
    RequirePermission('reports', 'read'),
    async (req, res) => {
      try {
        const { startDate, endDate, granularity = 'day' } = req.query;
        
        const result = await req.db.query(`
          SELECT 
            DATE_TRUNC($2, created_at) as period,
            COUNT(*) as conversions,
            COUNT(DISTINCT user_id) as unique_users,
            AVG(processing_time_ms) as avg_processing_time,
            SUM(input_file_size) as total_input_size,
            SUM(output_file_size) as total_output_size
          FROM conversion_jobs
          WHERE organization_id = $1
            AND created_at BETWEEN $3 AND $4
            AND status = 'completed'
          GROUP BY period
          ORDER BY period DESC
        `, [
          req.tenantContext.organizationId,
          granularity,
          startDate || '1970-01-01',
          endDate || '2100-01-01',
        ]);

        res.json({ data: result.rows });
      } catch (error: any) {
        res.status(500).json({ error: error.message });
      }
    }
  );

  // Audit log endpoints
  router.get('/audit-logs',
    RequirePermission('admin', 'admin'),
    async (req, res) => {
      try {
        const { page = 1, limit = 50, ...filters } = req.query;
        
        const logs = await req.auditLogger.search({
          ...filters,
          limit: parseInt(limit as string),
          offset: (parseInt(page as string) - 1) * parseInt(limit as string),
        });

        res.json(logs);
      } catch (error: any) {
        res.status(500).json({ error: error.message });
      }
    }
  );

  router.post('/audit-logs/export',
    RequirePermission('reports', 'export'),
    async (req, res) => {
      try {
        const { startDate, endDate, format = 'csv' } = req.body;
        
        // Log the export action
        await req.auditLogger.logDataExport(
          req.user.userId,
          'audit_logs',
          { startDate, endDate, format }
        );

        // Generate export
        // In production, this would generate actual CSV/JSON file
        
        res.json({ 
          exportId: 'export_123',
          status: 'processing',
          estimatedTime: 30,
        });
      } catch (error: any) {
        res.status(500).json({ error: error.message });
      }
    }
  );

  // API key management
  router.post('/api-keys',
    RequirePermission('api', 'create'),
    async (req, res) => {
      try {
        const { name, scopes = ['read'] } = req.body;
        
        const apiKey = await authService.createApiKey(
          req.user.userId,
          req.tenantContext.organizationId,
          name,
          scopes
        );

        res.json({
          id: apiKey.id,
          name: apiKey.name,
          key: apiKey.key, // Only shown once
          prefix: apiKey.prefix,
          scopes: apiKey.scopes,
        });
      } catch (error: any) {
        res.status(500).json({ error: error.message });
      }
    }
  );

  router.get('/api-keys',
    RequirePermission('api', 'read'),
    async (req, res) => {
      try {
        const result = await req.db.query(`
          SELECT 
            id, name, prefix, scopes, created_at, 
            last_used_at, usage_count, expires_at
          FROM api_keys
          WHERE organization_id = $1 
            AND user_id = $2
            AND is_active = true
          ORDER BY created_at DESC
        `, [req.tenantContext.organizationId, req.user.userId]);

        res.json({ apiKeys: result.rows });
      } catch (error: any) {
        res.status(500).json({ error: error.message });
      }
    }
  );

  router.delete('/api-keys/:keyId',
    RequirePermission('api', 'delete'),
    async (req, res) => {
      try {
        await req.db.query(`
          UPDATE api_keys 
          SET is_active = false, revoked_at = NOW()
          WHERE id = $1 
            AND organization_id = $2 
            AND user_id = $3
        `, [
          req.params.keyId,
          req.tenantContext.organizationId,
          req.user.userId,
        ]);

        // Audit log
        await req.auditLogger.log({
          action: AuditAction.API_KEY_DELETED,
          resourceType: ResourceType.API_KEY,
          resourceId: req.params.keyId,
          userId: req.user.userId,
        });

        res.json({ success: true });
      } catch (error: any) {
        res.status(500).json({ error: error.message });
      }
    }
  );

  return router;
}

export default createEnterpriseAPI;