module.exports = {
  root: true,
  extends: ["eslint:recommended"],
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: 2022,
    sourceType: "module",
    project: "./tsconfig.eslint.json",
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
    "no-unused-vars": "off", // Turn off base rule as it can report incorrect errors
    "@typescript-eslint/no-unused-vars": [
      "error",
      {
        argsIgnorePattern: "^_",
        varsIgnorePattern: "^_",
        caughtErrorsIgnorePattern: "^_",
      },
    ],
    "@typescript-eslint/no-explicit-any": "off", // Allow any for rapid prototyping
    "no-undef": "off", // TypeScript handles this
  },
  globals: {
    NodeJS: "readonly", // Define NodeJS global
  },
  overrides: [
    {
      files: ["tests/**/*.ts"],
      env: {
        jest: true,
      },
    },
  ],
  ignorePatterns: [
    "dist/",
    "node_modules/",
    "coverage/",
    "**/*.js",
    "**/*.d.ts",
    "test-*.js", // Ignore our test scripts
  ],
};
