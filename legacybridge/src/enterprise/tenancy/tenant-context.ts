// Multi-tenant Context System for LegacyBridge Enterprise
// Provides tenant isolation and context management throughout the application

export interface SubscriptionTier {
  name: 'basic' | 'professional' | 'enterprise';
  limits: {
    maxUsers: number;
    maxFileSizeMB: number;
    maxStorageGB: number;
    maxConcurrentJobs: number;
    maxApiCallsPerMinute: number;
    maxApiCallsPerDay: number;
  };
  features: string[];
}

export interface Organization {
  id: string;
  name: string;
  slug: string;
  subscriptionTier: SubscriptionTier['name'];
  status: 'active' | 'suspended' | 'canceled';
  subscriptionStartedAt: Date;
  subscriptionExpiresAt?: Date;
  trialEndsAt?: Date;
  branding: {
    logoUrl?: string;
    primaryColor: string;
    customDomain?: string;
  };
  compliance: {
    dataRetentionDays: number;
    auditLogLevel: 'basic' | 'detailed' | 'forensic';
    encryptionRequired: boolean;
  };
  createdAt: Date;
  updatedAt: Date;
}

export interface FeatureFlag {
  name: string;
  enabled: boolean;
  config?: Record<string, any>;
}

export interface TenantContext {
  organizationId: string;
  organization: Organization;
  subscriptionTier: SubscriptionTier;
  features: FeatureFlag[];
  limits: SubscriptionTier['limits'];
  branding: Organization['branding'];
  compliance: Organization['compliance'];
  
  // Helper methods
  hasFeature(featureName: string): boolean;
  checkLimit(limitName: keyof SubscriptionTier['limits'], value: number): boolean;
  isWithinLimit(limitName: keyof SubscriptionTier['limits'], currentValue: number): boolean;
}

// Subscription tier definitions
export const SUBSCRIPTION_TIERS: Record<SubscriptionTier['name'], SubscriptionTier> = {
  basic: {
    name: 'basic',
    limits: {
      maxUsers: 5,
      maxFileSizeMB: 50,
      maxStorageGB: 100,
      maxConcurrentJobs: 10,
      maxApiCallsPerMinute: 60,
      maxApiCallsPerDay: 10000,
    },
    features: [
      'basic_conversion',
      'email_support',
      'api_access',
      'basic_analytics',
    ],
  },
  professional: {
    name: 'professional',
    limits: {
      maxUsers: 50,
      maxFileSizeMB: 200,
      maxStorageGB: 1000,
      maxConcurrentJobs: 50,
      maxApiCallsPerMinute: 300,
      maxApiCallsPerDay: 100000,
    },
    features: [
      'basic_conversion',
      'advanced_conversion',
      'priority_support',
      'api_access',
      'advanced_analytics',
      'custom_branding',
      'sso_integration',
      'webhook_integration',
    ],
  },
  enterprise: {
    name: 'enterprise',
    limits: {
      maxUsers: -1, // Unlimited
      maxFileSizeMB: 1000,
      maxStorageGB: -1, // Unlimited
      maxConcurrentJobs: 200,
      maxApiCallsPerMinute: 1000,
      maxApiCallsPerDay: -1, // Unlimited
    },
    features: [
      'basic_conversion',
      'advanced_conversion',
      'dedicated_support',
      'api_access',
      'advanced_analytics',
      'custom_branding',
      'sso_integration',
      'webhook_integration',
      'audit_logs',
      'compliance_reports',
      'custom_domain',
      'sla_guarantee',
      'on_premise_option',
    ],
  },
};

// Tenant context implementation
export class TenantContextImpl implements TenantContext {
  organizationId: string;
  organization: Organization;
  subscriptionTier: SubscriptionTier;
  features: FeatureFlag[];
  limits: SubscriptionTier['limits'];
  branding: Organization['branding'];
  compliance: Organization['compliance'];

  constructor(organization: Organization, features: FeatureFlag[]) {
    this.organizationId = organization.id;
    this.organization = organization;
    this.subscriptionTier = SUBSCRIPTION_TIERS[organization.subscriptionTier];
    this.features = features;
    this.limits = this.subscriptionTier.limits;
    this.branding = organization.branding;
    this.compliance = organization.compliance;
  }

  hasFeature(featureName: string): boolean {
    // Check if feature is in subscription tier
    if (this.subscriptionTier.features.includes(featureName)) {
      // Check if it's not disabled by feature flag
      const flag = this.features.find(f => f.name === featureName);
      return !flag || flag.enabled;
    }
    
    // Check if enabled by feature flag
    const flag = this.features.find(f => f.name === featureName);
    return flag?.enabled || false;
  }

  checkLimit(limitName: keyof SubscriptionTier['limits'], value: number): boolean {
    const limit = this.limits[limitName];
    return limit === -1 || value <= limit;
  }

  isWithinLimit(limitName: keyof SubscriptionTier['limits'], currentValue: number): boolean {
    const limit = this.limits[limitName];
    return limit === -1 || currentValue < limit;
  }
}

// Tenant context provider
export class TenantContextProvider {
  private static contexts = new Map<string, TenantContext>();

  static async getContext(organizationId: string): Promise<TenantContext> {
    // Check cache
    if (this.contexts.has(organizationId)) {
      return this.contexts.get(organizationId)!;
    }

    // Load from database
    const context = await this.loadContext(organizationId);
    this.contexts.set(organizationId, context);
    
    // Set cache expiry (5 minutes)
    setTimeout(() => {
      this.contexts.delete(organizationId);
    }, 5 * 60 * 1000);

    return context;
  }

  private static async loadContext(organizationId: string): Promise<TenantContext> {
    // This would load from database in real implementation
    // For now, returning mock data
    const organization: Organization = {
      id: organizationId,
      name: 'Example Organization',
      slug: 'example-org',
      subscriptionTier: 'professional',
      status: 'active',
      subscriptionStartedAt: new Date(),
      branding: {
        primaryColor: '#007bff',
      },
      compliance: {
        dataRetentionDays: 365,
        auditLogLevel: 'detailed',
        encryptionRequired: true,
      },
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    const features: FeatureFlag[] = [
      { name: 'advanced_conversion', enabled: true },
      { name: 'webhook_integration', enabled: true },
    ];

    return new TenantContextImpl(organization, features);
  }

  static clearContext(organizationId: string): void {
    this.contexts.delete(organizationId);
  }

  static clearAllContexts(): void {
    this.contexts.clear();
  }
}

// Tenant-aware service base class
export abstract class TenantAwareService {
  protected tenantContext: TenantContext;

  constructor(tenantContext: TenantContext) {
    this.tenantContext = tenantContext;
  }

  protected validateLimit(limitName: keyof SubscriptionTier['limits'], value: number): void {
    if (!this.tenantContext.checkLimit(limitName, value)) {
      throw new Error(`Limit exceeded: ${limitName}. Maximum allowed: ${this.tenantContext.limits[limitName]}`);
    }
  }

  protected requireFeature(featureName: string): void {
    if (!this.tenantContext.hasFeature(featureName)) {
      throw new Error(`Feature not available: ${featureName}. Please upgrade your subscription.`);
    }
  }

  protected applyTenantFilter<T extends { organizationId: string }>(query: T[]): T[] {
    return query.filter(item => item.organizationId === this.tenantContext.organizationId);
  }

  protected addTenantContext<T>(data: T): T & { organizationId: string } {
    return {
      ...data,
      organizationId: this.tenantContext.organizationId,
    };
  }
}

// Middleware for Express/Next.js to inject tenant context
export async function tenantContextMiddleware(req: any, res: any, next: any) {
  try {
    // Extract organization ID from JWT token, subdomain, or header
    const organizationId = extractOrganizationId(req);
    
    if (!organizationId) {
      return res.status(401).json({ error: 'Organization context required' });
    }

    // Load tenant context
    const context = await TenantContextProvider.getContext(organizationId);
    
    // Check if organization is active
    if (context.organization.status !== 'active') {
      return res.status(403).json({ error: 'Organization is not active' });
    }

    // Attach to request
    req.tenantContext = context;
    
    // Set organization ID for database RLS
    if (req.db) {
      await req.db.query(`SET app.current_organization_id = '${organizationId}'`);
    }

    next();
  } catch (error) {
    console.error('Tenant context middleware error:', error);
    res.status(500).json({ error: 'Failed to load organization context' });
  }
}

function extractOrganizationId(req: any): string | null {
  // 1. Check subdomain (e.g., acme.legacybridge.com)
  const host = req.get('host');
  const subdomain = host?.split('.')[0];
  if (subdomain && subdomain !== 'www' && subdomain !== 'app') {
    return subdomain;
  }

  // 2. Check JWT token
  const token = req.headers.authorization?.replace('Bearer ', '');
  if (token) {
    // Decode JWT and extract organization ID
    // This is a placeholder - implement actual JWT decoding
    const decoded = decodeJWT(token);
    if (decoded?.organizationId) {
      return decoded.organizationId;
    }
  }

  // 3. Check custom header
  const orgHeader = req.headers['x-organization-id'];
  if (orgHeader) {
    return orgHeader;
  }

  // 4. Check session
  if (req.session?.organizationId) {
    return req.session.organizationId;
  }

  return null;
}

function decodeJWT(token: string): any {
  // Placeholder for JWT decoding
  // In real implementation, use jsonwebtoken library
  try {
    const payload = token.split('.')[1];
    return JSON.parse(Buffer.from(payload, 'base64').toString());
  } catch {
    return null;
  }
}

// Export types for global use
export type { TenantContext, Organization, FeatureFlag, SubscriptionTier };