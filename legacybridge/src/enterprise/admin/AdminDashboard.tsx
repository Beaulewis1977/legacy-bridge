'use client';

import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Users, Shield, Activity, Settings, FileText, TrendingUp,
  Building, Key, AlertCircle, CheckCircle, Clock, BarChart3,
  UserPlus, UserMinus, Lock, Unlock, Download, RefreshCw
} from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Progress } from '@/components/ui/progress';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

// Enterprise Admin Dashboard Component
export const AdminDashboard: React.FC = () => {
  const [activeTab, setActiveTab] = useState('overview');
  const [organizationData, setOrganizationData] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Load organization data
    loadOrganizationData();
  }, []);

  const loadOrganizationData = async () => {
    // Mock data for demonstration
    setOrganizationData({
      organization: {
        id: 'org-123',
        name: 'Acme Corporation',
        subscriptionTier: 'enterprise',
        status: 'active',
        usersCount: 342,
        maxUsers: -1,
        storageUsedGB: 245,
        maxStorageGB: -1,
      },
      metrics: {
        activeUsers: 298,
        conversionsToday: 1542,
        avgConversionTime: 3.2,
        errorRate: 0.02,
        apiCallsToday: 45623,
      },
      recentActivity: [
        { id: 1, user: 'john.doe@acme.com', action: 'Converted RTF to Markdown', time: '2 minutes ago', status: 'success' },
        { id: 2, user: 'jane.smith@acme.com', action: 'Created API key', time: '5 minutes ago', status: 'success' },
        { id: 3, user: 'bob.wilson@acme.com', action: 'Failed login attempt', time: '10 minutes ago', status: 'error' },
      ]
    });
    setLoading(false);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">Enterprise Admin Dashboard</h1>
            <p className="text-muted-foreground mt-1">
              Manage your organization and monitor system performance
            </p>
          </div>
          <div className="flex items-center gap-3">
            <Badge variant="outline" className="px-3 py-1">
              <Building className="w-3 h-3 mr-1" />
              {organizationData.organization.name}
            </Badge>
            <Badge className="px-3 py-1">
              {organizationData.organization.subscriptionTier.toUpperCase()}
            </Badge>
          </div>
        </div>

        {/* Quick Stats */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <StatCard
            title="Active Users"
            value={organizationData.metrics.activeUsers}
            total={organizationData.organization.usersCount}
            icon={Users}
            trend="+12%"
            color="text-blue-500"
          />
          <StatCard
            title="Conversions Today"
            value={organizationData.metrics.conversionsToday.toLocaleString()}
            icon={FileText}
            trend="+8%"
            color="text-green-500"
          />
          <StatCard
            title="API Calls"
            value={organizationData.metrics.apiCallsToday.toLocaleString()}
            icon={Activity}
            trend="+15%"
            color="text-purple-500"
          />
          <StatCard
            title="System Health"
            value={`${((1 - organizationData.metrics.errorRate) * 100).toFixed(1)}%`}
            icon={Shield}
            trend="Stable"
            color="text-emerald-500"
          />
        </div>

        {/* Main Tabs */}
        <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-4">
          <TabsList className="grid w-full grid-cols-5">
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="users">Users</TabsTrigger>
            <TabsTrigger value="roles">Roles & Permissions</TabsTrigger>
            <TabsTrigger value="settings">Settings</TabsTrigger>
            <TabsTrigger value="audit">Audit Logs</TabsTrigger>
          </TabsList>

          <TabsContent value="overview" className="space-y-4">
            <OverviewTab data={organizationData} />
          </TabsContent>

          <TabsContent value="users" className="space-y-4">
            <UsersTab />
          </TabsContent>

          <TabsContent value="roles" className="space-y-4">
            <RolesTab />
          </TabsContent>

          <TabsContent value="settings" className="space-y-4">
            <SettingsTab organization={organizationData.organization} />
          </TabsContent>

          <TabsContent value="audit" className="space-y-4">
            <AuditTab />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
};

// Stat Card Component
const StatCard: React.FC<{
  title: string;
  value: string | number;
  total?: number;
  icon: any;
  trend?: string;
  color: string;
}> = ({ title, value, total, icon: Icon, trend, color }) => (
  <Card className="p-6">
    <div className="flex items-center justify-between">
      <div className="space-y-2">
        <p className="text-sm font-medium text-muted-foreground">{title}</p>
        <div className="flex items-baseline gap-2">
          <p className="text-2xl font-bold">{value}</p>
          {total && (
            <span className="text-sm text-muted-foreground">/ {total}</span>
          )}
        </div>
        {trend && (
          <div className="flex items-center gap-1">
            <TrendingUp className="w-3 h-3 text-green-500" />
            <span className="text-xs text-green-500">{trend}</span>
          </div>
        )}
      </div>
      <div className={`p-3 rounded-full bg-secondary ${color}`}>
        <Icon className="w-5 h-5" />
      </div>
    </div>
  </Card>
);

// Overview Tab
const OverviewTab: React.FC<{ data: any }> = ({ data }) => (
  <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
    {/* Usage Chart */}
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Conversion Usage (7 days)</h3>
      <div className="h-64 flex items-center justify-center text-muted-foreground">
        <BarChart3 className="w-12 h-12" />
        <span className="ml-2">Chart visualization here</span>
      </div>
    </Card>

    {/* Recent Activity */}
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Recent Activity</h3>
      <div className="space-y-3">
        {data.recentActivity.map((activity: any) => (
          <div key={activity.id} className="flex items-center justify-between p-3 rounded-lg bg-secondary/50">
            <div className="flex items-center gap-3">
              <div className={`p-2 rounded-full ${
                activity.status === 'success' ? 'bg-green-500/10 text-green-500' : 'bg-red-500/10 text-red-500'
              }`}>
                {activity.status === 'success' ? (
                  <CheckCircle className="w-4 h-4" />
                ) : (
                  <AlertCircle className="w-4 h-4" />
                )}
              </div>
              <div>
                <p className="text-sm font-medium">{activity.user}</p>
                <p className="text-xs text-muted-foreground">{activity.action}</p>
              </div>
            </div>
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Clock className="w-3 h-3" />
              {activity.time}
            </div>
          </div>
        ))}
      </div>
    </Card>

    {/* Storage Usage */}
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Storage Usage</h3>
      <div className="space-y-4">
        <div>
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium">Used Storage</span>
            <span className="text-sm text-muted-foreground">
              {data.organization.storageUsedGB} GB / {
                data.organization.maxStorageGB === -1 ? 'Unlimited' : `${data.organization.maxStorageGB} GB`
              }
            </span>
          </div>
          <Progress value={data.organization.maxStorageGB === -1 ? 25 : (data.organization.storageUsedGB / data.organization.maxStorageGB) * 100} />
        </div>
      </div>
    </Card>

    {/* System Performance */}
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">System Performance</h3>
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <span className="text-sm">Avg Conversion Time</span>
          <Badge variant="outline">{data.metrics.avgConversionTime}s</Badge>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-sm">Error Rate</span>
          <Badge variant={data.metrics.errorRate > 0.05 ? 'destructive' : 'outline'}>
            {(data.metrics.errorRate * 100).toFixed(2)}%
          </Badge>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-sm">API Response Time</span>
          <Badge variant="outline">45ms</Badge>
        </div>
      </div>
    </Card>
  </div>
);

// Users Tab
const UsersTab: React.FC = () => {
  const [users, setUsers] = useState<any[]>([
    { id: 1, name: 'John Doe', email: 'john.doe@acme.com', role: 'Admin', status: 'active', lastLogin: '2 hours ago' },
    { id: 2, name: 'Jane Smith', email: 'jane.smith@acme.com', role: 'Manager', status: 'active', lastLogin: '1 day ago' },
    { id: 3, name: 'Bob Wilson', email: 'bob.wilson@acme.com', role: 'User', status: 'suspended', lastLogin: '1 week ago' },
  ]);

  return (
    <div className="space-y-4">
      {/* Actions */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Input placeholder="Search users..." className="w-64" />
          <Button variant="outline">Filter</Button>
        </div>
        <Button>
          <UserPlus className="w-4 h-4 mr-2" />
          Invite User
        </Button>
      </div>

      {/* Users List */}
      <Card>
        <div className="divide-y">
          {users.map(user => (
            <div key={user.id} className="p-4 hover:bg-secondary/50 transition-colors">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center">
                    <span className="text-sm font-semibold">{user.name.split(' ').map(n => n[0]).join('')}</span>
                  </div>
                  <div>
                    <p className="font-medium">{user.name}</p>
                    <p className="text-sm text-muted-foreground">{user.email}</p>
                  </div>
                </div>
                <div className="flex items-center gap-4">
                  <Badge variant="outline">{user.role}</Badge>
                  <Badge variant={user.status === 'active' ? 'default' : 'destructive'}>
                    {user.status}
                  </Badge>
                  <span className="text-sm text-muted-foreground">Last login: {user.lastLogin}</span>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="sm">Actions</Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      <DropdownMenuItem>Edit User</DropdownMenuItem>
                      <DropdownMenuItem>Change Role</DropdownMenuItem>
                      <DropdownMenuItem>Reset Password</DropdownMenuItem>
                      <DropdownMenuSeparator />
                      <DropdownMenuItem className="text-red-600">
                        {user.status === 'active' ? 'Suspend User' : 'Activate User'}
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </div>
            </div>
          ))}
        </div>
      </Card>
    </div>
  );
};

// Roles Tab
const RolesTab: React.FC = () => {
  const roles = [
    { id: 1, name: 'Organization Admin', users: 3, permissions: 28, isSystem: true },
    { id: 2, name: 'Manager', users: 12, permissions: 18, isSystem: true },
    { id: 3, name: 'User', users: 285, permissions: 8, isSystem: true },
    { id: 4, name: 'Custom Developer Role', users: 42, permissions: 15, isSystem: false },
  ];

  return (
    <div className="space-y-4">
      {/* Actions */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">Role Management</h3>
        <Button>
          <Shield className="w-4 h-4 mr-2" />
          Create Custom Role
        </Button>
      </div>

      {/* Roles Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {roles.map(role => (
          <Card key={role.id} className="p-6">
            <div className="flex items-start justify-between mb-4">
              <div>
                <h4 className="font-semibold flex items-center gap-2">
                  {role.name}
                  {role.isSystem && <Badge variant="secondary">System</Badge>}
                </h4>
                <p className="text-sm text-muted-foreground mt-1">
                  {role.users} users • {role.permissions} permissions
                </p>
              </div>
              <Button variant="ghost" size="sm">Edit</Button>
            </div>
            <div className="space-y-2">
              <div className="text-xs font-medium text-muted-foreground">Key Permissions:</div>
              <div className="flex flex-wrap gap-2">
                <Badge variant="outline" className="text-xs">Documents: Full Access</Badge>
                <Badge variant="outline" className="text-xs">Users: Read Only</Badge>
                <Badge variant="outline" className="text-xs">Reports: View</Badge>
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
};

// Settings Tab
const SettingsTab: React.FC<{ organization: any }> = ({ organization }) => (
  <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
    {/* General Settings */}
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">General Settings</h3>
      <div className="space-y-4">
        <div>
          <Label htmlFor="org-name">Organization Name</Label>
          <Input id="org-name" defaultValue={organization.name} className="mt-1" />
        </div>
        <div>
          <Label htmlFor="primary-color">Primary Brand Color</Label>
          <Input id="primary-color" type="color" defaultValue="#007bff" className="mt-1 h-10" />
        </div>
        <div className="flex items-center justify-between">
          <Label htmlFor="custom-domain">Custom Domain</Label>
          <Switch id="custom-domain" />
        </div>
      </div>
    </Card>

    {/* Security Settings */}
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Security Settings</h3>
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="font-medium">Require MFA</p>
            <p className="text-sm text-muted-foreground">All users must enable 2FA</p>
          </div>
          <Switch defaultChecked />
        </div>
        <div className="flex items-center justify-between">
          <div>
            <p className="font-medium">SSO Integration</p>
            <p className="text-sm text-muted-foreground">Enable SAML/OAuth2</p>
          </div>
          <Switch />
        </div>
        <div className="flex items-center justify-between">
          <div>
            <p className="font-medium">IP Whitelist</p>
            <p className="text-sm text-muted-foreground">Restrict access by IP</p>
          </div>
          <Switch />
        </div>
      </div>
    </Card>

    {/* Compliance Settings */}
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Compliance Settings</h3>
      <div className="space-y-4">
        <div>
          <Label htmlFor="retention">Data Retention (days)</Label>
          <Input id="retention" type="number" defaultValue="365" className="mt-1" />
        </div>
        <div>
          <Label htmlFor="audit-level">Audit Log Level</Label>
          <select id="audit-level" className="w-full mt-1 p-2 border rounded">
            <option value="basic">Basic</option>
            <option value="detailed" selected>Detailed</option>
            <option value="forensic">Forensic</option>
          </select>
        </div>
        <div className="flex items-center justify-between">
          <Label htmlFor="encryption">Enforce Encryption</Label>
          <Switch id="encryption" defaultChecked />
        </div>
      </div>
    </Card>

    {/* API Settings */}
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">API Settings</h3>
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <p className="text-sm">Active API Keys</p>
          <Badge>12</Badge>
        </div>
        <div className="flex items-center justify-between">
          <p className="text-sm">Rate Limit (per minute)</p>
          <Input type="number" defaultValue="1000" className="w-24" />
        </div>
        <Button variant="outline" className="w-full">
          <Key className="w-4 h-4 mr-2" />
          Manage API Keys
        </Button>
      </div>
    </Card>
  </div>
);

// Audit Logs Tab
const AuditTab: React.FC = () => {
  const logs = [
    { id: 1, user: 'john.doe@acme.com', action: 'user.login', resource: 'auth', timestamp: '2024-01-24 14:32:15', ip: '192.168.1.100' },
    { id: 2, user: 'jane.smith@acme.com', action: 'document.convert', resource: 'documents', timestamp: '2024-01-24 14:30:22', ip: '192.168.1.101' },
    { id: 3, user: 'admin@acme.com', action: 'user.role.update', resource: 'users', timestamp: '2024-01-24 14:28:10', ip: '192.168.1.102' },
  ];

  return (
    <div className="space-y-4">
      {/* Filters */}
      <div className="flex items-center gap-4">
        <Input placeholder="Search logs..." className="flex-1" />
        <Button variant="outline">
          <Download className="w-4 h-4 mr-2" />
          Export Logs
        </Button>
        <Button variant="outline">
          <RefreshCw className="w-4 h-4 mr-2" />
          Refresh
        </Button>
      </div>

      {/* Logs Table */}
      <Card>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="border-b">
              <tr>
                <th className="text-left p-4 font-medium">User</th>
                <th className="text-left p-4 font-medium">Action</th>
                <th className="text-left p-4 font-medium">Resource</th>
                <th className="text-left p-4 font-medium">Timestamp</th>
                <th className="text-left p-4 font-medium">IP Address</th>
              </tr>
            </thead>
            <tbody className="divide-y">
              {logs.map(log => (
                <tr key={log.id} className="hover:bg-secondary/50">
                  <td className="p-4 text-sm">{log.user}</td>
                  <td className="p-4 text-sm font-mono">{log.action}</td>
                  <td className="p-4 text-sm">
                    <Badge variant="outline">{log.resource}</Badge>
                  </td>
                  <td className="p-4 text-sm text-muted-foreground">{log.timestamp}</td>
                  <td className="p-4 text-sm font-mono">{log.ip}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>
    </div>
  );
};

export default AdminDashboard;