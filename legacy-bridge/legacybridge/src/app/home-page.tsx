'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { MainLayout } from '@/components/layout/MainLayout';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { 
  Sparkles, 
  ArrowRight, 
  Zap, 
  Shield, 
  Globe, 
  Users,
  BarChart3,
  Code2,
  Database,
  Layers,
  CheckCircle,
  TrendingUp
} from 'lucide-react';
import Link from 'next/link';

const features = [
  {
    icon: Zap,
    title: 'Lightning Fast',
    description: 'Convert legacy systems in seconds with our high-performance engine',
    gradient: 'from-amber-500 to-orange-500'
  },
  {
    icon: Shield,
    title: 'Enterprise Security',
    description: 'Bank-grade security with SOC2 compliance and end-to-end encryption',
    gradient: 'from-emerald-500 to-green-500'
  },
  {
    icon: Globe,
    title: 'Global Scale',
    description: 'Deploy anywhere with multi-region support and 99.99% uptime',
    gradient: 'from-blue-500 to-cyan-500'
  },
  {
    icon: Users,
    title: 'Team Collaboration',
    description: 'Work together seamlessly with real-time updates and version control',
    gradient: 'from-purple-500 to-pink-500'
  }
];

const stats = [
  { label: 'Active Users', value: '10K+', icon: Users },
  { label: 'Conversions/Day', value: '1M+', icon: TrendingUp },
  { label: 'Uptime', value: '99.99%', icon: BarChart3 },
  { label: 'Response Time', value: '<50ms', icon: Zap }
];

export default function HomePage() {
  const [hoveredFeature, setHoveredFeature] = useState<number | null>(null);

  return (
    <MainLayout>
      {/* Hero Section */}
      <section className="relative overflow-hidden py-20 lg:py-32">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
          className="text-center max-w-4xl mx-auto px-4"
        >
          {/* Floating Badge */}
          <motion.div
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            transition={{ delay: 0.2, type: "spring" }}
            className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-legacy-blue-100 dark:bg-legacy-blue-900/20 mb-8"
          >
            <Sparkles className="w-4 h-4 text-legacy-blue-600 dark:text-legacy-blue-400" />
            <span className="text-sm font-medium text-legacy-blue-700 dark:text-legacy-blue-300">
              Version 2.4.1 - Now with AI-Powered Conversions
            </span>
          </motion.div>

          {/* Main Heading */}
          <h1 className="text-5xl lg:text-7xl font-bold mb-6">
            <span className="bg-gradient-to-r from-legacy-blue-600 via-legacy-blue-700 to-legacy-emerald-600 bg-clip-text text-transparent">
              Modernize Legacy Systems
            </span>
            <br />
            <span className="text-foreground">In Minutes, Not Months</span>
          </h1>

          <p className="text-xl text-muted-foreground mb-10 max-w-2xl mx-auto">
            Transform your legacy codebase into modern, maintainable applications with our 
            AI-powered platform. Reduce migration time by 90% and eliminate technical debt.
          </p>

          {/* CTA Buttons */}
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link href="/demo">
              <Button size="lg" className="group px-8 py-6 text-lg">
                Start Demo
                <ArrowRight className="ml-2 w-5 h-5 group-hover:translate-x-1 transition-transform" />
              </Button>
            </Link>
            <Link href="/monitoring">
              <Button size="lg" variant="outline" className="px-8 py-6 text-lg">
                View Dashboard
              </Button>
            </Link>
          </div>
        </motion.div>

        {/* Animated Background Elements */}
        <div className="absolute inset-0 -z-10 overflow-hidden">
          <motion.div
            className="absolute -top-40 -right-40 w-80 h-80 bg-legacy-blue-400 rounded-full mix-blend-multiply filter blur-3xl opacity-20"
            animate={{
              x: [0, 100, 0],
              y: [0, -100, 0],
            }}
            transition={{
              duration: 20,
              repeat: Infinity,
              ease: "easeInOut"
            }}
          />
          <motion.div
            className="absolute -bottom-40 -left-40 w-80 h-80 bg-legacy-emerald-400 rounded-full mix-blend-multiply filter blur-3xl opacity-20"
            animate={{
              x: [0, -100, 0],
              y: [0, 100, 0],
            }}
            transition={{
              duration: 15,
              repeat: Infinity,
              ease: "easeInOut"
            }}
          />
        </div>
      </section>

      {/* Stats Section */}
      <section className="py-16 border-y border-border/50">
        <div className="max-w-6xl mx-auto px-4">
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-8">
            {stats.map((stat, index) => {
              const Icon = stat.icon;
              return (
                <motion.div
                  key={stat.label}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: index * 0.1 }}
                  className="text-center"
                >
                  <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-legacy-blue-100 dark:bg-legacy-blue-900/20 mb-3">
                    <Icon className="w-6 h-6 text-legacy-blue-600 dark:text-legacy-blue-400" />
                  </div>
                  <div className="text-3xl font-bold text-foreground">{stat.value}</div>
                  <div className="text-sm text-muted-foreground">{stat.label}</div>
                </motion.div>
              );
            })}
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20">
        <div className="max-w-6xl mx-auto px-4">
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.2 }}
            className="text-center mb-16"
          >
            <h2 className="text-3xl lg:text-4xl font-bold mb-4">
              Enterprise-Grade Features
            </h2>
            <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
              Everything you need to modernize your legacy systems with confidence
            </p>
          </motion.div>

          <div className="grid md:grid-cols-2 gap-6">
            {features.map((feature, index) => {
              const Icon = feature.icon;
              return (
                <motion.div
                  key={feature.title}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: index * 0.1 }}
                  onMouseEnter={() => setHoveredFeature(index)}
                  onMouseLeave={() => setHoveredFeature(null)}
                >
                  <Card className="relative h-full p-6 glass-panel hover-lift hover-glow transition-all duration-300 overflow-hidden group">
                    {/* Gradient Background */}
                    <div className={`absolute inset-0 bg-gradient-to-br ${feature.gradient} opacity-0 group-hover:opacity-10 transition-opacity duration-300`} />
                    
                    <div className="relative z-10">
                      <div className={`inline-flex items-center justify-center w-14 h-14 rounded-xl bg-gradient-to-br ${feature.gradient} mb-4`}>
                        <Icon className="w-7 h-7 text-white" />
                      </div>
                      
                      <h3 className="text-xl font-semibold mb-2">{feature.title}</h3>
                      <p className="text-muted-foreground">{feature.description}</p>
                      
                      <motion.div
                        initial={{ width: 0 }}
                        animate={{ width: hoveredFeature === index ? '100%' : 0 }}
                        className="absolute bottom-0 left-0 h-1 bg-gradient-to-r from-legacy-blue-500 to-legacy-emerald-500"
                      />
                    </div>
                  </Card>
                </motion.div>
              );
            })}
          </div>
        </div>
      </section>

      {/* Technical Highlights */}
      <section className="py-20 bg-muted/30">
        <div className="max-w-6xl mx-auto px-4">
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="text-center mb-16"
          >
            <h2 className="text-3xl lg:text-4xl font-bold mb-4">
              Built for Modern Development
            </h2>
            <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
              Cutting-edge technology stack for maximum performance and reliability
            </p>
          </motion.div>

          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {[
              { icon: Code2, label: 'TypeScript', desc: 'Type-safe code' },
              { icon: Layers, label: 'React 18', desc: 'Modern UI' },
              { icon: Database, label: 'PostgreSQL', desc: 'Reliable data' },
              { icon: Shield, label: 'SOC2', desc: 'Compliant' }
            ].map((tech, index) => {
              const Icon = tech.icon;
              return (
                <motion.div
                  key={tech.label}
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ delay: index * 0.1 }}
                  whileHover={{ scale: 1.05 }}
                  className="p-6 rounded-xl glass-panel text-center"
                >
                  <Icon className="w-8 h-8 mx-auto mb-3 text-legacy-blue-600 dark:text-legacy-blue-400" />
                  <h4 className="font-semibold mb-1">{tech.label}</h4>
                  <p className="text-sm text-muted-foreground">{tech.desc}</p>
                </motion.div>
              );
            })}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="max-w-4xl mx-auto px-4 text-center"
        >
          <Card className="p-12 glass-morphism border-legacy-blue-500/20">
            <CheckCircle className="w-16 h-16 mx-auto mb-6 text-legacy-emerald-500" />
            <h2 className="text-3xl font-bold mb-4">
              Ready to Transform Your Legacy Systems?
            </h2>
            <p className="text-lg text-muted-foreground mb-8 max-w-2xl mx-auto">
              Join thousands of developers who are already modernizing their applications 
              with LegacyBridge. Start your free trial today.
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Link href="/demo">
                <Button size="lg" className="px-8">
                  Try It Free
                  <ArrowRight className="ml-2 w-5 h-5" />
                </Button>
              </Link>
              <Button size="lg" variant="outline" className="px-8">
                Contact Sales
              </Button>
            </div>
          </Card>
        </motion.div>
      </section>
    </MainLayout>
  );
}