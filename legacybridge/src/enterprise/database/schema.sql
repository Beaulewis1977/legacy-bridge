-- Enterprise Multi-tenant Database Schema for LegacyBridge
-- Supports 1000+ concurrent users with data isolation

-- Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Organizations table
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    subscription_tier VARCHAR(50) NOT NULL CHECK (subscription_tier IN ('basic', 'professional', 'enterprise')),
    status VARCHAR(50) DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'canceled')),
    
    -- Subscription details
    subscription_started_at TIMESTAMP NOT NULL DEFAULT NOW(),
    subscription_expires_at TIMESTAMP,
    trial_ends_at TIMESTAMP,
    
    -- Limits based on tier
    max_users INTEGER NOT NULL DEFAULT 5,
    max_file_size_mb INTEGER NOT NULL DEFAULT 50,
    max_storage_gb INTEGER NOT NULL DEFAULT 100,
    max_concurrent_jobs INTEGER NOT NULL DEFAULT 10,
    
    -- Branding
    logo_url VARCHAR(500),
    primary_color VARCHAR(7) DEFAULT '#007bff',
    custom_domain VARCHAR(255),
    
    -- Compliance settings
    data_retention_days INTEGER DEFAULT 365,
    audit_log_level VARCHAR(20) DEFAULT 'basic' CHECK (audit_log_level IN ('basic', 'detailed', 'forensic')),
    encryption_required BOOLEAN DEFAULT true,
    
    -- Metadata
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    deleted_at TIMESTAMP,
    
    -- Indexes
    INDEX idx_organizations_slug (slug),
    INDEX idx_organizations_status (status),
    INDEX idx_organizations_subscription_tier (subscription_tier)
);

-- Roles table
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_system BOOLEAN DEFAULT false,
    
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    
    UNIQUE(organization_id, name),
    INDEX idx_roles_organization (organization_id)
);

-- Permissions table
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resource VARCHAR(50) NOT NULL CHECK (resource IN ('documents', 'users', 'settings', 'reports', 'admin', 'api')),
    action VARCHAR(50) NOT NULL CHECK (action IN ('create', 'read', 'update', 'delete', 'list', 'export', 'admin')),
    description TEXT,
    
    UNIQUE(resource, action),
    INDEX idx_permissions_resource (resource)
);

-- Role permissions junction table
CREATE TABLE role_permissions (
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID REFERENCES permissions(id) ON DELETE CASCADE,
    
    PRIMARY KEY (role_id, permission_id),
    INDEX idx_role_permissions_role (role_id),
    INDEX idx_role_permissions_permission (permission_id)
);

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    
    -- Profile
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    avatar_url VARCHAR(500),
    timezone VARCHAR(50) DEFAULT 'UTC',
    locale VARCHAR(10) DEFAULT 'en',
    
    -- Status
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'suspended', 'pending')),
    email_verified BOOLEAN DEFAULT false,
    email_verified_at TIMESTAMP,
    
    -- Authentication
    last_login_at TIMESTAMP,
    last_login_ip INET,
    login_count INTEGER DEFAULT 0,
    failed_login_count INTEGER DEFAULT 0,
    locked_until TIMESTAMP,
    
    -- MFA
    mfa_enabled BOOLEAN DEFAULT false,
    mfa_secret VARCHAR(255),
    
    -- API access
    api_key_hash VARCHAR(255),
    api_key_created_at TIMESTAMP,
    
    -- Metadata
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    deleted_at TIMESTAMP,
    
    INDEX idx_users_organization (organization_id),
    INDEX idx_users_email (email),
    INDEX idx_users_status (status)
);

-- User roles junction table
CREATE TABLE user_roles (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id),
    granted_at TIMESTAMP DEFAULT NOW(),
    
    PRIMARY KEY (user_id, role_id),
    INDEX idx_user_roles_user (user_id),
    INDEX idx_user_roles_role (role_id)
);

-- Sessions table for managing user sessions
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    
    -- Session data
    ip_address INET,
    user_agent TEXT,
    device_info JSONB,
    
    -- Expiration
    expires_at TIMESTAMP NOT NULL,
    last_activity_at TIMESTAMP DEFAULT NOW(),
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    revoked_at TIMESTAMP,
    revoked_reason VARCHAR(255),
    
    created_at TIMESTAMP DEFAULT NOW(),
    
    INDEX idx_sessions_user (user_id),
    INDEX idx_sessions_token (token_hash),
    INDEX idx_sessions_expires (expires_at)
);

-- Conversion jobs table with multi-tenant support
CREATE TABLE conversion_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    
    -- Job details
    input_file_name VARCHAR(255) NOT NULL,
    input_file_size BIGINT NOT NULL,
    input_file_hash VARCHAR(64),
    output_file_name VARCHAR(255),
    output_file_size BIGINT,
    output_file_hash VARCHAR(64),
    
    -- Conversion details
    conversion_type VARCHAR(20) NOT NULL CHECK (conversion_type IN ('rtf_to_md', 'md_to_rtf')),
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'canceled')),
    progress INTEGER DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
    
    -- Storage
    input_storage_path TEXT,
    output_storage_path TEXT,
    
    -- Performance metrics
    processing_time_ms INTEGER,
    queue_time_ms INTEGER,
    
    -- Error handling
    error_message TEXT,
    error_details JSONB,
    retry_count INTEGER DEFAULT 0,
    
    -- Metadata
    created_at TIMESTAMP DEFAULT NOW(),
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    expires_at TIMESTAMP,
    
    INDEX idx_jobs_organization (organization_id),
    INDEX idx_jobs_user (user_id),
    INDEX idx_jobs_status (status),
    INDEX idx_jobs_created (created_at DESC)
);

-- Audit logs table for compliance
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    
    -- Action details
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    
    -- Details
    details JSONB,
    before_data JSONB,
    after_data JSONB,
    
    -- Request context
    ip_address INET,
    user_agent TEXT,
    request_id UUID,
    session_id UUID,
    
    -- Compliance
    compliance_tags TEXT[],
    retention_until TIMESTAMP,
    
    timestamp TIMESTAMP DEFAULT NOW(),
    
    INDEX idx_audit_organization (organization_id),
    INDEX idx_audit_user (user_id),
    INDEX idx_audit_timestamp (timestamp DESC),
    INDEX idx_audit_action (action),
    INDEX idx_audit_resource (resource_type, resource_id)
);

-- API keys table for programmatic access
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) UNIQUE NOT NULL,
    prefix VARCHAR(10) NOT NULL,
    
    -- Permissions
    scopes TEXT[] DEFAULT ARRAY['read'],
    
    -- Rate limiting
    rate_limit_per_minute INTEGER DEFAULT 60,
    rate_limit_per_day INTEGER DEFAULT 10000,
    
    -- Usage tracking
    last_used_at TIMESTAMP,
    usage_count INTEGER DEFAULT 0,
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    expires_at TIMESTAMP,
    
    created_at TIMESTAMP DEFAULT NOW(),
    revoked_at TIMESTAMP,
    
    INDEX idx_api_keys_organization (organization_id),
    INDEX idx_api_keys_prefix (prefix),
    INDEX idx_api_keys_hash (key_hash)
);

-- Feature flags table for gradual rollouts
CREATE TABLE feature_flags (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    
    -- Targeting
    enabled_globally BOOLEAN DEFAULT false,
    enabled_for_tiers TEXT[] DEFAULT ARRAY[]::TEXT[],
    enabled_for_organizations UUID[] DEFAULT ARRAY[]::UUID[],
    enabled_percentage INTEGER DEFAULT 0 CHECK (enabled_percentage >= 0 AND enabled_percentage <= 100),
    
    -- Configuration
    config JSONB DEFAULT '{}',
    
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    
    INDEX idx_feature_flags_name (name)
);

-- Usage metrics table for analytics
CREATE TABLE usage_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    
    -- Metrics
    metric_date DATE NOT NULL,
    active_users INTEGER DEFAULT 0,
    conversion_count INTEGER DEFAULT 0,
    total_file_size_mb BIGINT DEFAULT 0,
    api_calls INTEGER DEFAULT 0,
    
    -- Performance
    avg_conversion_time_ms INTEGER,
    p95_conversion_time_ms INTEGER,
    p99_conversion_time_ms INTEGER,
    
    -- Errors
    error_count INTEGER DEFAULT 0,
    
    created_at TIMESTAMP DEFAULT NOW(),
    
    UNIQUE(organization_id, metric_date),
    INDEX idx_metrics_organization_date (organization_id, metric_date DESC)
);

-- Create default system roles
INSERT INTO permissions (resource, action, description) VALUES
    ('documents', 'create', 'Create new document conversions'),
    ('documents', 'read', 'View document conversions'),
    ('documents', 'update', 'Update document conversion settings'),
    ('documents', 'delete', 'Delete document conversions'),
    ('documents', 'list', 'List all document conversions'),
    ('documents', 'export', 'Export conversion history'),
    ('users', 'create', 'Create new users'),
    ('users', 'read', 'View user profiles'),
    ('users', 'update', 'Update user information'),
    ('users', 'delete', 'Delete users'),
    ('users', 'list', 'List all users'),
    ('settings', 'read', 'View organization settings'),
    ('settings', 'update', 'Update organization settings'),
    ('reports', 'read', 'View reports and analytics'),
    ('reports', 'export', 'Export reports'),
    ('admin', 'admin', 'Full administrative access'),
    ('api', 'create', 'Create API keys'),
    ('api', 'read', 'View API keys'),
    ('api', 'delete', 'Delete API keys');

-- Create update timestamp trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply update trigger to all tables with updated_at
CREATE TRIGGER update_organizations_updated_at BEFORE UPDATE ON organizations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    
CREATE TRIGGER update_roles_updated_at BEFORE UPDATE ON roles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    
CREATE TRIGGER update_feature_flags_updated_at BEFORE UPDATE ON feature_flags
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create partitioned tables for high-volume data
CREATE TABLE audit_logs_partitioned (
    LIKE audit_logs INCLUDING ALL
) PARTITION BY RANGE (timestamp);

-- Create monthly partitions for audit logs
CREATE TABLE audit_logs_y2024m01 PARTITION OF audit_logs_partitioned
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
    
CREATE TABLE audit_logs_y2024m02 PARTITION OF audit_logs_partitioned
    FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');

-- Add more partitions as needed...

-- Create indexes for performance
CREATE INDEX idx_jobs_organization_status_created 
    ON conversion_jobs(organization_id, status, created_at DESC);
    
CREATE INDEX idx_audit_compliance 
    ON audit_logs USING GIN(compliance_tags);

-- Row Level Security policies
ALTER TABLE organizations ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE conversion_jobs ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;

-- Create policies for tenant isolation
CREATE POLICY tenant_isolation_organizations ON organizations
    FOR ALL
    USING (id = current_setting('app.current_organization_id')::uuid);
    
CREATE POLICY tenant_isolation_users ON users
    FOR ALL
    USING (organization_id = current_setting('app.current_organization_id')::uuid);
    
CREATE POLICY tenant_isolation_jobs ON conversion_jobs
    FOR ALL
    USING (organization_id = current_setting('app.current_organization_id')::uuid);
    
CREATE POLICY tenant_isolation_audit ON audit_logs
    FOR ALL
    USING (organization_id = current_setting('app.current_organization_id')::uuid);