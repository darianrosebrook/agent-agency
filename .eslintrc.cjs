module.exports = {
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: 2022,
    sourceType: "module",
  },
  plugins: ["@typescript-eslint"],
  extends: ["eslint:recommended"],
  rules: {
    // General rules
    "no-console": "warn",
    "prefer-const": "error",
    "no-var": "error",
    "object-shorthand": "error",
    "prefer-template": "error",

    // CAWS specific rules
    "no-duplicate-imports": "error",
    "no-unused-expressions": "error",
    "no-constant-condition": "error",
  },
  env: {
    node: true,
    es2022: true,
    jest: true,
  },
  ignorePatterns: ["dist/", "node_modules/", "coverage/", "*.js"],
};
