// Enterprise Authentication System for LegacyBridge
// Supports OAuth2, SAML, JWT, and API key authentication

import { createHash, randomBytes } from 'crypto';
import * as bcrypt from 'bcryptjs';
import * as jwt from 'jsonwebtoken';
import * as speakeasy from 'speakeasy';
import * as qrcode from 'qrcode';

export interface AuthConfig {
  jwtSecret: string;
  jwtExpiresIn: string;
  refreshTokenExpiresIn: string;
  sessionTimeout: number;
  maxLoginAttempts: number;
  lockoutDuration: number;
  passwordMinLength: number;
  passwordRequireUppercase: boolean;
  passwordRequireNumbers: boolean;
  passwordRequireSpecial: boolean;
  mfaRequired: boolean;
  apiKeyPrefix: string;
}

export interface LoginCredentials {
  email: string;
  password: string;
  mfaCode?: string;
  organizationId?: string;
}

export interface AuthToken {
  accessToken: string;
  refreshToken: string;
  expiresIn: number;
  tokenType: 'Bearer';
}

export interface Session {
  id: string;
  userId: string;
  organizationId: string;
  tokenHash: string;
  ipAddress: string;
  userAgent: string;
  deviceInfo?: any;
  expiresAt: Date;
  lastActivityAt: Date;
}

export interface ApiKey {
  id: string;
  organizationId: string;
  userId: string;
  name: string;
  key: string; // Only returned on creation
  keyHash: string;
  prefix: string;
  scopes: string[];
  rateLimitPerMinute: number;
  rateLimitPerDay: number;
  lastUsedAt?: Date;
  expiresAt?: Date;
}

export class AuthenticationService {
  constructor(
    private db: any,
    private config: AuthConfig
  ) {}

  // User login with email/password
  async login(credentials: LoginCredentials): Promise<AuthToken> {
    const { email, password, mfaCode, organizationId } = credentials;

    // Find user
    const userResult = await this.db.query(`
      SELECT u.*, o.status as org_status
      FROM users u
      JOIN organizations o ON o.id = u.organization_id
      WHERE u.email = $1 
        AND u.deleted_at IS NULL
        ${organizationId ? 'AND u.organization_id = $2' : ''}
    `, organizationId ? [email, organizationId] : [email]);

    if (userResult.rows.length === 0) {
      throw new Error('Invalid credentials');
    }

    const user = userResult.rows[0];

    // Check organization status
    if (user.org_status !== 'active') {
      throw new Error('Organization is not active');
    }

    // Check user status
    if (user.status !== 'active') {
      if (user.status === 'suspended') {
        throw new Error('Account suspended');
      }
      if (user.status === 'pending') {
        throw new Error('Account not verified');
      }
      throw new Error('Account inactive');
    }

    // Check if account is locked
    if (user.locked_until && new Date(user.locked_until) > new Date()) {
      throw new Error('Account temporarily locked');
    }

    // Verify password
    const passwordValid = await bcrypt.compare(password, user.password_hash);
    if (!passwordValid) {
      await this.handleFailedLogin(user.id);
      throw new Error('Invalid credentials');
    }

    // Check MFA if enabled
    if (user.mfa_enabled || this.config.mfaRequired) {
      if (!mfaCode) {
        throw new Error('MFA code required');
      }
      
      const mfaValid = speakeasy.totp.verify({
        secret: user.mfa_secret,
        encoding: 'base32',
        token: mfaCode,
        window: 2
      });

      if (!mfaValid) {
        throw new Error('Invalid MFA code');
      }
    }

    // Update login info
    await this.db.query(`
      UPDATE users
      SET last_login_at = NOW(),
          last_login_ip = $2,
          login_count = login_count + 1,
          failed_login_count = 0
      WHERE id = $1
    `, [user.id, credentials.ipAddress || null]);

    // Generate tokens
    const tokens = await this.generateTokens(user);

    // Create session
    await this.createSession(user.id, tokens.refreshToken, {
      ipAddress: credentials.ipAddress || '',
      userAgent: credentials.userAgent || '',
      deviceInfo: credentials.deviceInfo
    });

    return tokens;
  }

  // OAuth2 login
  async oauthLogin(provider: string, profile: any, organizationId: string): Promise<AuthToken> {
    // Find or create user
    let user = await this.db.query(`
      SELECT * FROM users
      WHERE email = $1 AND organization_id = $2
    `, [profile.email, organizationId]);

    if (user.rows.length === 0) {
      // Create new user
      const result = await this.db.query(`
        INSERT INTO users (
          organization_id, email, first_name, last_name,
          email_verified, status, avatar_url
        ) VALUES ($1, $2, $3, $4, true, 'active', $5)
        RETURNING *
      `, [
        organizationId,
        profile.email,
        profile.firstName,
        profile.lastName,
        profile.picture
      ]);
      user = result.rows[0];
    } else {
      user = user.rows[0];
    }

    // Generate tokens
    const tokens = await this.generateTokens(user);

    // Create session
    await this.createSession(user.id, tokens.refreshToken, {
      ipAddress: '',
      userAgent: `OAuth2/${provider}`,
      deviceInfo: { provider }
    });

    return tokens;
  }

  // Generate JWT tokens
  private async generateTokens(user: any): Promise<AuthToken> {
    const payload = {
      userId: user.id,
      organizationId: user.organization_id,
      email: user.email,
      roles: await this.getUserRoles(user.id)
    };

    const accessToken = jwt.sign(payload, this.config.jwtSecret, {
      expiresIn: this.config.jwtExpiresIn
    });

    const refreshToken = jwt.sign(
      { userId: user.id, type: 'refresh' },
      this.config.jwtSecret,
      { expiresIn: this.config.refreshTokenExpiresIn }
    );

    return {
      accessToken,
      refreshToken,
      expiresIn: 3600, // 1 hour
      tokenType: 'Bearer'
    };
  }

  // Refresh access token
  async refreshToken(refreshToken: string): Promise<AuthToken> {
    try {
      const decoded = jwt.verify(refreshToken, this.config.jwtSecret) as any;
      
      if (decoded.type !== 'refresh') {
        throw new Error('Invalid token type');
      }

      // Check if session exists
      const session = await this.db.query(`
        SELECT * FROM sessions
        WHERE token_hash = $1 AND is_active = true AND expires_at > NOW()
      `, [this.hashToken(refreshToken)]);

      if (session.rows.length === 0) {
        throw new Error('Session not found');
      }

      // Get user
      const user = await this.db.query(`
        SELECT * FROM users WHERE id = $1 AND status = 'active'
      `, [decoded.userId]);

      if (user.rows.length === 0) {
        throw new Error('User not found');
      }

      // Update session activity
      await this.db.query(`
        UPDATE sessions SET last_activity_at = NOW() WHERE id = $1
      `, [session.rows[0].id]);

      return this.generateTokens(user.rows[0]);
    } catch (error) {
      throw new Error('Invalid refresh token');
    }
  }

  // Logout
  async logout(token: string): Promise<void> {
    await this.db.query(`
      UPDATE sessions 
      SET is_active = false, revoked_at = NOW(), revoked_reason = 'User logout'
      WHERE token_hash = $1
    `, [this.hashToken(token)]);
  }

  // Create API key
  async createApiKey(
    userId: string,
    organizationId: string,
    name: string,
    scopes: string[] = ['read']
  ): Promise<ApiKey> {
    const key = this.generateApiKey();
    const keyHash = this.hashToken(key);
    const prefix = key.substring(0, 8);

    const result = await this.db.query(`
      INSERT INTO api_keys (
        organization_id, user_id, name, key_hash, prefix, scopes
      ) VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING *
    `, [organizationId, userId, name, keyHash, prefix, scopes]);

    return {
      ...result.rows[0],
      key // Only return the actual key on creation
    };
  }

  // Validate API key
  async validateApiKey(apiKey: string): Promise<any> {
    const keyHash = this.hashToken(apiKey);
    
    const result = await this.db.query(`
      SELECT ak.*, u.email, o.status as org_status
      FROM api_keys ak
      JOIN users u ON u.id = ak.user_id
      JOIN organizations o ON o.id = ak.organization_id
      WHERE ak.key_hash = $1 AND ak.is_active = true
    `, [keyHash]);

    if (result.rows.length === 0) {
      throw new Error('Invalid API key');
    }

    const key = result.rows[0];

    // Check expiration
    if (key.expires_at && new Date(key.expires_at) < new Date()) {
      throw new Error('API key expired');
    }

    // Check organization status
    if (key.org_status !== 'active') {
      throw new Error('Organization is not active');
    }

    // Update usage
    await this.db.query(`
      UPDATE api_keys 
      SET last_used_at = NOW(), usage_count = usage_count + 1
      WHERE id = $1
    `, [key.id]);

    return {
      organizationId: key.organization_id,
      userId: key.user_id,
      scopes: key.scopes,
      rateLimits: {
        perMinute: key.rate_limit_per_minute,
        perDay: key.rate_limit_per_day
      }
    };
  }

  // Setup MFA
  async setupMFA(userId: string): Promise<{ secret: string; qrCode: string }> {
    const user = await this.db.query(`
      SELECT email, organization_id FROM users WHERE id = $1
    `, [userId]);

    if (user.rows.length === 0) {
      throw new Error('User not found');
    }

    const secret = speakeasy.generateSecret({
      name: `LegacyBridge (${user.rows[0].email})`,
      issuer: 'LegacyBridge'
    });

    // Save secret
    await this.db.query(`
      UPDATE users SET mfa_secret = $2 WHERE id = $1
    `, [userId, secret.base32]);

    // Generate QR code
    const qrCode = await qrcode.toDataURL(secret.otpauth_url!);

    return {
      secret: secret.base32,
      qrCode
    };
  }

  // Enable MFA
  async enableMFA(userId: string, code: string): Promise<void> {
    const user = await this.db.query(`
      SELECT mfa_secret FROM users WHERE id = $1
    `, [userId]);

    if (user.rows.length === 0) {
      throw new Error('User not found');
    }

    const valid = speakeasy.totp.verify({
      secret: user.rows[0].mfa_secret,
      encoding: 'base32',
      token: code,
      window: 2
    });

    if (!valid) {
      throw new Error('Invalid MFA code');
    }

    await this.db.query(`
      UPDATE users SET mfa_enabled = true WHERE id = $1
    `, [userId]);
  }

  // Password validation
  validatePassword(password: string): string[] {
    const errors: string[] = [];

    if (password.length < this.config.passwordMinLength) {
      errors.push(`Password must be at least ${this.config.passwordMinLength} characters`);
    }

    if (this.config.passwordRequireUppercase && !/[A-Z]/.test(password)) {
      errors.push('Password must contain at least one uppercase letter');
    }

    if (this.config.passwordRequireNumbers && !/\d/.test(password)) {
      errors.push('Password must contain at least one number');
    }

    if (this.config.passwordRequireSpecial && !/[!@#$%^&*]/.test(password)) {
      errors.push('Password must contain at least one special character');
    }

    return errors;
  }

  // Change password
  async changePassword(userId: string, oldPassword: string, newPassword: string): Promise<void> {
    // Validate new password
    const errors = this.validatePassword(newPassword);
    if (errors.length > 0) {
      throw new Error(errors.join(', '));
    }

    // Verify old password
    const user = await this.db.query(`
      SELECT password_hash FROM users WHERE id = $1
    `, [userId]);

    if (user.rows.length === 0) {
      throw new Error('User not found');
    }

    const valid = await bcrypt.compare(oldPassword, user.rows[0].password_hash);
    if (!valid) {
      throw new Error('Invalid current password');
    }

    // Update password
    const passwordHash = await bcrypt.hash(newPassword, 10);
    await this.db.query(`
      UPDATE users SET password_hash = $2, updated_at = NOW() WHERE id = $1
    `, [userId, passwordHash]);

    // Revoke all sessions
    await this.db.query(`
      UPDATE sessions 
      SET is_active = false, revoked_at = NOW(), revoked_reason = 'Password changed'
      WHERE user_id = $1
    `, [userId]);
  }

  // Helper methods
  private async createSession(userId: string, token: string, context: any): Promise<void> {
    const expiresAt = new Date();
    expiresAt.setHours(expiresAt.getHours() + 24); // 24 hours

    await this.db.query(`
      INSERT INTO sessions (
        user_id, token_hash, ip_address, user_agent, device_info, expires_at
      ) VALUES ($1, $2, $3, $4, $5, $6)
    `, [
      userId,
      this.hashToken(token),
      context.ipAddress,
      context.userAgent,
      JSON.stringify(context.deviceInfo),
      expiresAt
    ]);
  }

  private async handleFailedLogin(userId: string): Promise<void> {
    await this.db.query(`
      UPDATE users 
      SET failed_login_count = failed_login_count + 1
      WHERE id = $1
    `, [userId]);

    // Check if should lock account
    const user = await this.db.query(`
      SELECT failed_login_count FROM users WHERE id = $1
    `, [userId]);

    if (user.rows[0].failed_login_count >= this.config.maxLoginAttempts) {
      const lockUntil = new Date();
      lockUntil.setMinutes(lockUntil.getMinutes() + this.config.lockoutDuration);

      await this.db.query(`
        UPDATE users SET locked_until = $2 WHERE id = $1
      `, [userId, lockUntil]);
    }
  }

  private async getUserRoles(userId: string): Promise<string[]> {
    const result = await this.db.query(`
      SELECT r.name
      FROM user_roles ur
      JOIN roles r ON r.id = ur.role_id
      WHERE ur.user_id = $1
    `, [userId]);

    return result.rows.map((row: any) => row.name);
  }

  private generateApiKey(): string {
    return `lbk_${randomBytes(32).toString('hex')}`;
  }

  private hashToken(token: string): string {
    return createHash('sha256').update(token).digest('hex');
  }
}

// JWT Middleware
export function jwtMiddleware(config: AuthConfig) {
  return async (req: any, res: any, next: any) => {
    try {
      const token = req.headers.authorization?.replace('Bearer ', '');
      
      if (!token) {
        return res.status(401).json({ error: 'No token provided' });
      }

      const decoded = jwt.verify(token, config.jwtSecret) as any;
      
      // Check if session is active
      const session = await req.db.query(`
        SELECT * FROM sessions
        WHERE user_id = $1 AND is_active = true AND expires_at > NOW()
        ORDER BY created_at DESC
        LIMIT 1
      `, [decoded.userId]);

      if (session.rows.length === 0) {
        return res.status(401).json({ error: 'Session expired' });
      }

      // Update session activity
      await req.db.query(`
        UPDATE sessions SET last_activity_at = NOW() WHERE id = $1
      `, [session.rows[0].id]);

      req.user = decoded;
      req.sessionId = session.rows[0].id;
      
      next();
    } catch (error) {
      return res.status(401).json({ error: 'Invalid token' });
    }
  };
}

// API Key Middleware
export function apiKeyMiddleware(authService: AuthenticationService) {
  return async (req: any, res: any, next: any) => {
    try {
      const apiKey = req.headers['x-api-key'];
      
      if (!apiKey) {
        return next(); // Continue to JWT middleware
      }

      const keyData = await authService.validateApiKey(apiKey);
      
      req.user = {
        userId: keyData.userId,
        organizationId: keyData.organizationId,
        scopes: keyData.scopes,
        isApiKey: true
      };
      
      req.rateLimits = keyData.rateLimits;
      
      next();
    } catch (error) {
      return res.status(401).json({ error: 'Invalid API key' });
    }
  };
}

export default AuthenticationService;