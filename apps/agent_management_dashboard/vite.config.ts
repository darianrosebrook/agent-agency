import react from "@vitejs/plugin-react";
import path from "path";
import { defineConfig } from "vite";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react({
      // Enable error overlay in development
      jsxRuntime: "automatic",
    }),
  ],
  root: ".",
  build: {
    outDir: "out",
    emptyOutDir: true,
  },
  server: {
    port: 3000,
    host: true,
    proxy: {
      // Proxy API requests to the Rust backend
      '/api/proxy': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/proxy/, ''),
      },
    },
  },
  // Ensure errors are logged to console and terminal
  logLevel: "info",
  clearScreen: false, // Keep error output visible in terminal
  define: {
    // Provide process.env for Next.js compatibility
    'process.env': JSON.stringify({
      NODE_ENV: process.env.NODE_ENV || 'development',
    }),
    // Provide process object for Next.js code that expects it
    'process': JSON.stringify({
      env: {
        NODE_ENV: process.env.NODE_ENV || 'development',
      },
    }),
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  css: {
    modules: {
      localsConvention: "camelCase",
    },
  },
});
