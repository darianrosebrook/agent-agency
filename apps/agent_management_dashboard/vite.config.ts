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
  },
  // Ensure errors are logged to console and terminal
  logLevel: "info",
  clearScreen: false, // Keep error output visible in terminal
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
