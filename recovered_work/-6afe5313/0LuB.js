/** @type {import('next').NextConfig} */
const nextConfig = {
  // Enable experimental features
  experimental: {
    // Enable server components
    serverComponents: true,
    // Enable app directory
    appDir: true,
  },

  // Configure webpack
  webpack: (config, { isServer }) => {
    // Add custom webpack configuration if needed
    return config;
  },

  // Configure images
  images: {
    domains: [],
    formats: ['image/webp', 'image/avif'],
  },

  // Configure redirects
  async redirects() {
    return [
      // Add any redirects here
    ];
  },

  // Configure rewrites
  async rewrites() {
    return [
      // Add any rewrites here
    ];
  },

  // Configure headers
  async headers() {
    return [
      {
        source: '/(.*)',
        headers: [
          {
            key: 'X-Frame-Options',
            value: 'DENY',
          },
          {
            key: 'X-Content-Type-Options',
            value: 'nosniff',
          },
          {
            key: 'Referrer-Policy',
            value: 'origin-when-cross-origin',
          },
        ],
      },
    ];
  },

  // Configure environment variables
  env: {
    // Add any environment variables here
  },

  // Configure TypeScript
  typescript: {
    // Ignore TypeScript errors during build
    ignoreBuildErrors: false,
  },

  // Configure ESLint
  eslint: {
    // Ignore ESLint errors during build
    ignoreDuringBuilds: false,
  },

  // Configure output
  output: 'standalone',

  // Configure trailing slash
  trailingSlash: false,

  // Configure powered by header
  poweredByHeader: false,

  // Configure compress
  compress: true,

  // Configure dev indicator
  devIndicators: {
    buildActivity: true,
    buildActivityPosition: 'bottom-right',
  },
};

module.exports = nextConfig;