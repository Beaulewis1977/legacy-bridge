'use client';

import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { SystemHealth } from './MonitoringDashboard';
import { 
  Activity, 
  Server, 
  Clock, 
  GitBranch,
  CheckCircle2,
  AlertCircle,
  XCircle,
  TrendingUp,
  TrendingDown,
  Minus
} from 'lucide-react';
import { cn } from '@/lib/utils';

interface SystemHealthCardProps {
  systemHealth: SystemHealth;
}

export function SystemHealthCard({ systemHealth }: SystemHealthCardProps) {
  const getHealthIcon = (status: string) => {
    switch (status) {
      case 'healthy':
        return <CheckCircle2 className="w-5 h-5 text-green-500" />;
      case 'warning':
        return <AlertCircle className="w-5 h-5 text-amber-500" />;
      case 'critical':
        return <XCircle className="w-5 h-5 text-red-500" />;
      default:
        return <Activity className="w-5 h-5 text-gray-500" />;
    }
  };

  const getServiceStatusColor = (status: string) => {
    switch (status) {
      case 'running':
        return 'bg-green-500';
      case 'stopped':
        return 'bg-gray-500';
      case 'error':
        return 'bg-red-500';
      default:
        return 'bg-gray-400';
    }
  };

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  return (
    <>
      {/* Main Health Status Card */}
      <motion.div 
        className="col-span-full"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        <Card className="glass-panel overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-legacy-blue-500/10 via-transparent to-legacy-emerald-500/10 pointer-events-none" />
          
          <CardContent className="p-6">
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-3">
                {getHealthIcon(systemHealth.status)}
                <div>
                  <h3 className="text-xl font-semibold">System Health</h3>
                  <p className="text-sm text-muted-foreground">
                    Last updated: {new Date(systemHealth.lastUpdate).toLocaleTimeString()}
                  </p>
                </div>
              </div>
              
              <Badge 
                variant={systemHealth.status === 'healthy' ? 'default' : systemHealth.status === 'warning' ? 'secondary' : 'destructive'}
                className="px-3 py-1 text-sm font-medium"
              >
                {systemHealth.status.toUpperCase()}
              </Badge>
            </div>

            {/* System Info Grid */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
              <motion.div 
                className="flex items-center gap-3 p-3 rounded-lg bg-slate-100/50 dark:bg-slate-800/50"
                whileHover={{ scale: 1.02 }}
                transition={{ type: "spring", stiffness: 300 }}
              >
                <Clock className="w-5 h-5 text-legacy-blue-500" />
                <div>
                  <p className="text-sm text-muted-foreground">Uptime</p>
                  <p className="font-semibold">{formatUptime(systemHealth.uptime)}</p>
                </div>
              </motion.div>

              <motion.div 
                className="flex items-center gap-3 p-3 rounded-lg bg-slate-100/50 dark:bg-slate-800/50"
                whileHover={{ scale: 1.02 }}
                transition={{ type: "spring", stiffness: 300 }}
              >
                <GitBranch className="w-5 h-5 text-legacy-emerald-500" />
                <div>
                  <p className="text-sm text-muted-foreground">Version</p>
                  <p className="font-semibold">{systemHealth.version}</p>
                </div>
              </motion.div>

              <motion.div 
                className="flex items-center gap-3 p-3 rounded-lg bg-slate-100/50 dark:bg-slate-800/50"
                whileHover={{ scale: 1.02 }}
                transition={{ type: "spring", stiffness: 300 }}
              >
                <Server className="w-5 h-5 text-legacy-amber-500" />
                <div>
                  <p className="text-sm text-muted-foreground">Environment</p>
                  <p className="font-semibold capitalize">{systemHealth.environment}</p>
                </div>
              </motion.div>
            </div>

            {/* Services Status */}
            <div className="space-y-3">
              <h4 className="text-sm font-medium text-muted-foreground mb-2">Service Status</h4>
              <AnimatePresence>
                {systemHealth.services.map((service, index) => (
                  <motion.div
                    key={service.name}
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    exit={{ opacity: 0, x: 20 }}
                    transition={{ delay: index * 0.05 }}
                    className="flex items-center justify-between p-3 rounded-lg bg-slate-100/30 dark:bg-slate-800/30 backdrop-blur-sm"
                  >
                    <div className="flex items-center gap-3">
                      <div className={cn(
                        "w-2 h-2 rounded-full animate-pulse",
                        getServiceStatusColor(service.status)
                      )} />
                      <span className="font-medium">{service.name}</span>
                    </div>
                    
                    <div className="flex items-center gap-3">
                      <Progress 
                        value={service.health} 
                        className="w-24 h-2"
                      />
                      <span className="text-sm text-muted-foreground">{service.health}%</span>
                    </div>
                  </motion.div>
                ))}
              </AnimatePresence>
            </div>
          </CardContent>
        </Card>
      </motion.div>
    </>
  );
}