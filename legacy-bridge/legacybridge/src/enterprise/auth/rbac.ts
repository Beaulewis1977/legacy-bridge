// Role-Based Access Control (RBAC) System for LegacyBridge Enterprise
// Provides comprehensive permission management for 1000+ users

export interface Permission {
  id: string;
  resource: 'documents' | 'users' | 'settings' | 'reports' | 'admin' | 'api';
  action: 'create' | 'read' | 'update' | 'delete' | 'list' | 'export' | 'admin';
  description?: string;
}

export interface Role {
  id: string;
  organizationId: string;
  name: string;
  description: string;
  permissions: Permission[];
  isSystem: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface User {
  id: string;
  organizationId: string;
  email: string;
  firstName?: string;
  lastName?: string;
  roles: Role[];
  status: 'active' | 'inactive' | 'suspended' | 'pending';
  mfaEnabled: boolean;
  lastLoginAt?: Date;
  createdAt: Date;
  updatedAt: Date;
}

// System-defined roles
export const SYSTEM_ROLES = {
  SUPER_ADMIN: 'super_admin',
  ORGANIZATION_ADMIN: 'organization_admin',
  MANAGER: 'manager',
  USER: 'user',
  VIEWER: 'viewer',
  API_USER: 'api_user',
} as const;

// Permission definitions
export const PERMISSIONS: Record<string, Permission> = {
  // Document permissions
  DOCUMENTS_CREATE: { id: 'doc_create', resource: 'documents', action: 'create', description: 'Create new document conversions' },
  DOCUMENTS_READ: { id: 'doc_read', resource: 'documents', action: 'read', description: 'View document conversions' },
  DOCUMENTS_UPDATE: { id: 'doc_update', resource: 'documents', action: 'update', description: 'Update document conversion settings' },
  DOCUMENTS_DELETE: { id: 'doc_delete', resource: 'documents', action: 'delete', description: 'Delete document conversions' },
  DOCUMENTS_LIST: { id: 'doc_list', resource: 'documents', action: 'list', description: 'List all document conversions' },
  DOCUMENTS_EXPORT: { id: 'doc_export', resource: 'documents', action: 'export', description: 'Export conversion history' },
  
  // User permissions
  USERS_CREATE: { id: 'user_create', resource: 'users', action: 'create', description: 'Create new users' },
  USERS_READ: { id: 'user_read', resource: 'users', action: 'read', description: 'View user profiles' },
  USERS_UPDATE: { id: 'user_update', resource: 'users', action: 'update', description: 'Update user information' },
  USERS_DELETE: { id: 'user_delete', resource: 'users', action: 'delete', description: 'Delete users' },
  USERS_LIST: { id: 'user_list', resource: 'users', action: 'list', description: 'List all users' },
  
  // Settings permissions
  SETTINGS_READ: { id: 'settings_read', resource: 'settings', action: 'read', description: 'View organization settings' },
  SETTINGS_UPDATE: { id: 'settings_update', resource: 'settings', action: 'update', description: 'Update organization settings' },
  
  // Reports permissions
  REPORTS_READ: { id: 'reports_read', resource: 'reports', action: 'read', description: 'View reports and analytics' },
  REPORTS_EXPORT: { id: 'reports_export', resource: 'reports', action: 'export', description: 'Export reports' },
  
  // Admin permissions
  ADMIN_FULL: { id: 'admin_full', resource: 'admin', action: 'admin', description: 'Full administrative access' },
  
  // API permissions
  API_CREATE: { id: 'api_create', resource: 'api', action: 'create', description: 'Create API keys' },
  API_READ: { id: 'api_read', resource: 'api', action: 'read', description: 'View API keys' },
  API_DELETE: { id: 'api_delete', resource: 'api', action: 'delete', description: 'Delete API keys' },
};

// Default role permissions mapping
export const DEFAULT_ROLE_PERMISSIONS: Record<string, string[]> = {
  [SYSTEM_ROLES.SUPER_ADMIN]: Object.keys(PERMISSIONS), // All permissions
  
  [SYSTEM_ROLES.ORGANIZATION_ADMIN]: [
    'doc_create', 'doc_read', 'doc_update', 'doc_delete', 'doc_list', 'doc_export',
    'user_create', 'user_read', 'user_update', 'user_delete', 'user_list',
    'settings_read', 'settings_update',
    'reports_read', 'reports_export',
    'api_create', 'api_read', 'api_delete',
  ],
  
  [SYSTEM_ROLES.MANAGER]: [
    'doc_create', 'doc_read', 'doc_update', 'doc_list', 'doc_export',
    'user_read', 'user_list',
    'settings_read',
    'reports_read', 'reports_export',
  ],
  
  [SYSTEM_ROLES.USER]: [
    'doc_create', 'doc_read', 'doc_update', 'doc_delete', 'doc_list',
    'user_read', // Own profile only
  ],
  
  [SYSTEM_ROLES.VIEWER]: [
    'doc_read', 'doc_list',
    'user_read', // Own profile only
    'reports_read',
  ],
  
  [SYSTEM_ROLES.API_USER]: [
    'doc_create', 'doc_read', 'doc_list',
    'api_read', // Own API keys only
  ],
};

// RBAC Service
export class RBACService {
  private userCache = new Map<string, User>();
  private roleCache = new Map<string, Role>();
  private permissionCache = new Map<string, Set<string>>();

  constructor(private db: any) {}

  // Check if user has permission
  async hasPermission(
    userId: string,
    resource: Permission['resource'],
    action: Permission['action']
  ): Promise<boolean> {
    const user = await this.getUser(userId);
    if (!user || user.status !== 'active') {
      return false;
    }

    // Check cache
    const cacheKey = `${userId}:${resource}:${action}`;
    if (this.permissionCache.has(cacheKey)) {
      return this.permissionCache.get(cacheKey)!.has(`${resource}:${action}`);
    }

    // Check user's roles
    for (const role of user.roles) {
      for (const permission of role.permissions) {
        if (permission.resource === resource && permission.action === action) {
          this.cachePermission(cacheKey, `${resource}:${action}`);
          return true;
        }
        // Check for admin permission
        if (permission.resource === 'admin' && permission.action === 'admin') {
          this.cachePermission(cacheKey, `${resource}:${action}`);
          return true;
        }
      }
    }

    return false;
  }

  // Check multiple permissions (AND logic)
  async hasAllPermissions(
    userId: string,
    permissions: Array<{ resource: Permission['resource']; action: Permission['action'] }>
  ): Promise<boolean> {
    for (const perm of permissions) {
      if (!(await this.hasPermission(userId, perm.resource, perm.action))) {
        return false;
      }
    }
    return true;
  }

  // Check multiple permissions (OR logic)
  async hasAnyPermission(
    userId: string,
    permissions: Array<{ resource: Permission['resource']; action: Permission['action'] }>
  ): Promise<boolean> {
    for (const perm of permissions) {
      if (await this.hasPermission(userId, perm.resource, perm.action)) {
        return true;
      }
    }
    return false;
  }

  // Get user with roles and permissions
  async getUser(userId: string): Promise<User | null> {
    // Check cache
    if (this.userCache.has(userId)) {
      return this.userCache.get(userId)!;
    }

    // Load from database
    const userResult = await this.db.query(`
      SELECT 
        u.*,
        json_agg(DISTINCT jsonb_build_object(
          'id', r.id,
          'name', r.name,
          'description', r.description,
          'isSystem', r.is_system,
          'permissions', (
            SELECT json_agg(DISTINCT jsonb_build_object(
              'id', p.id,
              'resource', p.resource,
              'action', p.action,
              'description', p.description
            ))
            FROM role_permissions rp
            JOIN permissions p ON p.id = rp.permission_id
            WHERE rp.role_id = r.id
          )
        )) as roles
      FROM users u
      LEFT JOIN user_roles ur ON ur.user_id = u.id
      LEFT JOIN roles r ON r.id = ur.role_id
      WHERE u.id = $1 AND u.deleted_at IS NULL
      GROUP BY u.id
    `, [userId]);

    if (userResult.rows.length === 0) {
      return null;
    }

    const user = this.mapUser(userResult.rows[0]);
    
    // Cache for 5 minutes
    this.userCache.set(userId, user);
    setTimeout(() => this.userCache.delete(userId), 5 * 60 * 1000);

    return user;
  }

  // Assign role to user
  async assignRole(userId: string, roleId: string, grantedBy: string): Promise<void> {
    await this.db.query(`
      INSERT INTO user_roles (user_id, role_id, granted_by, granted_at)
      VALUES ($1, $2, $3, NOW())
      ON CONFLICT (user_id, role_id) DO NOTHING
    `, [userId, roleId, grantedBy]);

    // Clear cache
    this.clearUserCache(userId);
  }

  // Remove role from user
  async removeRole(userId: string, roleId: string): Promise<void> {
    await this.db.query(`
      DELETE FROM user_roles
      WHERE user_id = $1 AND role_id = $2
    `, [userId, roleId]);

    // Clear cache
    this.clearUserCache(userId);
  }

  // Create custom role
  async createRole(
    organizationId: string,
    name: string,
    description: string,
    permissionIds: string[]
  ): Promise<Role> {
    const roleResult = await this.db.query(`
      INSERT INTO roles (organization_id, name, description, is_system)
      VALUES ($1, $2, $3, false)
      RETURNING *
    `, [organizationId, name, description]);

    const roleId = roleResult.rows[0].id;

    // Assign permissions
    if (permissionIds.length > 0) {
      const values = permissionIds.map((pid, i) => `($${i*2+1}, $${i*2+2})`).join(',');
      const params = permissionIds.flatMap(pid => [roleId, pid]);
      
      await this.db.query(`
        INSERT INTO role_permissions (role_id, permission_id)
        VALUES ${values}
      `, params);
    }

    return this.getRole(roleId);
  }

  // Get role details
  async getRole(roleId: string): Promise<Role> {
    if (this.roleCache.has(roleId)) {
      return this.roleCache.get(roleId)!;
    }

    const result = await this.db.query(`
      SELECT 
        r.*,
        json_agg(DISTINCT jsonb_build_object(
          'id', p.id,
          'resource', p.resource,
          'action', p.action,
          'description', p.description
        )) as permissions
      FROM roles r
      LEFT JOIN role_permissions rp ON rp.role_id = r.id
      LEFT JOIN permissions p ON p.id = rp.permission_id
      WHERE r.id = $1
      GROUP BY r.id
    `, [roleId]);

    const role = this.mapRole(result.rows[0]);
    
    // Cache
    this.roleCache.set(roleId, role);
    setTimeout(() => this.roleCache.delete(roleId), 5 * 60 * 1000);

    return role;
  }

  // List roles for organization
  async listRoles(organizationId: string): Promise<Role[]> {
    const result = await this.db.query(`
      SELECT 
        r.*,
        json_agg(DISTINCT jsonb_build_object(
          'id', p.id,
          'resource', p.resource,
          'action', p.action,
          'description', p.description
        )) as permissions
      FROM roles r
      LEFT JOIN role_permissions rp ON rp.role_id = r.id
      LEFT JOIN permissions p ON p.id = rp.permission_id
      WHERE r.organization_id = $1 OR r.is_system = true
      GROUP BY r.id
      ORDER BY r.is_system DESC, r.name
    `, [organizationId]);

    return result.rows.map(row => this.mapRole(row));
  }

  // Helper methods
  private mapUser(row: any): User {
    return {
      id: row.id,
      organizationId: row.organization_id,
      email: row.email,
      firstName: row.first_name,
      lastName: row.last_name,
      roles: row.roles || [],
      status: row.status,
      mfaEnabled: row.mfa_enabled,
      lastLoginAt: row.last_login_at,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    };
  }

  private mapRole(row: any): Role {
    return {
      id: row.id,
      organizationId: row.organization_id,
      name: row.name,
      description: row.description,
      permissions: row.permissions || [],
      isSystem: row.is_system,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    };
  }

  private cachePermission(key: string, permission: string): void {
    if (!this.permissionCache.has(key)) {
      this.permissionCache.set(key, new Set());
    }
    this.permissionCache.get(key)!.add(permission);
    
    // Expire after 5 minutes
    setTimeout(() => this.permissionCache.delete(key), 5 * 60 * 1000);
  }

  private clearUserCache(userId: string): void {
    this.userCache.delete(userId);
    // Clear permission cache for user
    for (const key of this.permissionCache.keys()) {
      if (key.startsWith(`${userId}:`)) {
        this.permissionCache.delete(key);
      }
    }
  }
}

// Permission decorator for route handlers
export function RequirePermission(resource: Permission['resource'], action: Permission['action']) {
  return function (target: any, propertyKey: string, descriptor: PropertyDescriptor) {
    const originalMethod = descriptor.value;

    descriptor.value = async function (...args: any[]) {
      const req = args[0];
      const res = args[1];

      if (!req.user || !req.rbac) {
        return res.status(401).json({ error: 'Unauthorized' });
      }

      const hasPermission = await req.rbac.hasPermission(req.user.id, resource, action);
      if (!hasPermission) {
        return res.status(403).json({ 
          error: 'Forbidden',
          message: `Missing permission: ${resource}:${action}`
        });
      }

      return originalMethod.apply(this, args);
    };

    return descriptor;
  };
}

// Middleware for Express/Next.js
export function rbacMiddleware(db: any) {
  return async (req: any, res: any, next: any) => {
    if (req.user) {
      req.rbac = new RBACService(db);
    }
    next();
  };
}

// Export types and constants
export type { User, Role, Permission };
export { SYSTEM_ROLES, PERMISSIONS, DEFAULT_ROLE_PERMISSIONS };