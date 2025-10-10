module.exports = {
  root: false,
  extends: ["eslint:recommended"],
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: 2022,
    sourceType: "module",
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
    "@typescript-eslint/no-explicit-any": "off", // Allow any for rapid prototyping
  },
  ignorePatterns: ["node_modules/", "apps/", "scripts/", "codemod/"],
};
