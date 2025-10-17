/**
 * ESLint Configuration for Agent Agency V2
 *
 * @author @darianrosebrook
 */

module.exports = {
  root: true,
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: 2022,
    sourceType: "module",
  },
  plugins: ["@typescript-eslint"],
  env: {
    node: true,
    es2022: true,
    jest: true, // Enable Jest globals
  },
  globals: {
    console: "readonly",
    process: "readonly",
    Buffer: "readonly",
    __dirname: "readonly",
    __filename: "readonly",
  },
  rules: {
    // Disable base rule for TypeScript files (use @typescript-eslint/no-unused-vars instead)
    "no-unused-vars": "off",
    "@typescript-eslint/no-unused-vars": [
      "warn", // Downgrade to warning for type definitions
      {
        argsIgnorePattern: "^_",
        varsIgnorePattern: "^_",
        ignoreRestSiblings: true,
        caughtErrors: "none",
      },
    ],
    "no-undef": "error",
    "no-console": "off", // Allow console for logging
    "prefer-const": "error",
    "no-var": "error",
  },
  ignorePatterns: ["dist/", "node_modules/", "coverage/", "*.js", "*.d.ts"],
};
