/** @type {import('next').NextConfig} */
const nextConfig = {
  // Next.js 16 configuration

  // Configure webpack
  webpack: (config, { isServer }) => {
    if (!isServer) {
      // Optimize bundle splitting for better caching
      config.optimization.splitChunks = {
        chunks: 'all',
        cacheGroups: {
          // Vendor libraries
          vendor: {
            test: /[\\/]node_modules[\\/]/,
            name: 'vendors',
            chunks: 'all',
            priority: 10,
          },
          // GSAP specific chunk
          gsap: {
            test: /[\\/]node_modules[\\/]gsap[\\/]/,
            name: 'gsap',
            chunks: 'all',
            priority: 20,
          },
          // React libraries
          react: {
            test: /[\\/]node_modules[\\/](react|react-dom)[\\/]/,
            name: 'react',
            chunks: 'all',
            priority: 15,
          },
          // UI libraries
          ui: {
            test: /[\\/]node_modules[\\/](lucide-react|class-variance-authority|clsx|tailwind-merge)[\\/]/,
            name: 'ui',
            chunks: 'all',
            priority: 12,
          },
          // Common components
          common: {
            name: 'common',
            minChunks: 2,
            chunks: 'all',
            priority: 5,
          },
        },
      };
    }
    return config;
  },

  // Configure images
  images: {
    domains: [],
    formats: ['image/avif', 'image/webp'],
    deviceSizes: [640, 750, 828, 1080, 1200, 1920],
    imageSizes: [16, 32, 48, 64, 96, 128, 256, 384],
    minimumCacheTTL: 60,
    dangerouslyAllowSVG: true,
    contentSecurityPolicy: "default-src 'self'; script-src 'none'; sandbox;",
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

  // Configure Turbopack (Next.js 16 default)
  turbopack: {},

  // Configure output
  output: 'standalone',
  
  // Performance optimizations
  experimental: {
    optimizeCss: true,
    optimizePackageImports: ['lucide-react', 'gsap'],
  },

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

export default nextConfig;