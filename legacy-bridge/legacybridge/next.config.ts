import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Temporarily disable static export to allow API routes
  // output: 'export',
  // distDir: 'out',
  eslint: {
    ignoreDuringBuilds: true,
  },
  typescript: {
    ignoreBuildErrors: true,
  },
};

export default nextConfig;
