module.exports = {
  root: true,
  extends: ["eslint:recommended"],
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: 2022,
    sourceType: "module",
    project: "./tsconfig.json",
  },
  plugins: ["@typescript-eslint"],
  env: {
    node: true,
    es2022: true,
  },
  rules: {
    // General code quality
    "no-console": "off", // Allow console in server-side code
    "prefer-const": "error",
    "no-var": "error",
    "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
    "@typescript-eslint/no-explicit-any": "warn", // Warn about any usage
    "@typescript-eslint/prefer-nullish-coalescing": "error",
    "@typescript-eslint/prefer-optional-chain": "error",
  },
  globals: {
    NodeJS: "readonly", // Define NodeJS global
  },
  ignorePatterns: [
    "node_modules/",
    "dist/",
    "coverage/",
    "**/*.d.ts",
    "**/*.js",
    "test-*.js", // Ignore our test scripts
    "iterations/", // Ignore iterations directory to avoid tsconfig conflicts
    "docs/archive/", // Ignore archived documentation files
    "playground/", // Ignore playground files
  ],
};
