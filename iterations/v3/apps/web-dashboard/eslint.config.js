/**
 * ESLint Configuration for Agent Agency V3 Dashboard
 * 
 * @author @darianrosebrook
 * 
 * Modern ESLint v9 configuration with Next.js, TypeScript, and React support
 */

import js from '@eslint/js';
import typescript from '@typescript-eslint/eslint-plugin';
import typescriptParser from '@typescript-eslint/parser';
import react from 'eslint-plugin-react';
import reactHooks from 'eslint-plugin-react-hooks';
import jsxA11y from 'eslint-plugin-jsx-a11y';

export default [
  js.configs.recommended,
  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      parser: typescriptParser,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
        ecmaFeatures: {
          jsx: true,
        },
      },
      globals: {
        HTMLElement: 'readonly',
        HTMLDivElement: 'readonly',
        process: 'readonly',
        console: 'readonly',
        window: 'readonly',
        document: 'readonly',
        fetch: 'readonly',
        AbortSignal: 'readonly',
        Response: 'readonly',
        setTimeout: 'readonly',
        clearTimeout: 'readonly',
        navigator: 'readonly',
        KeyboardEvent: 'readonly',
        FocusOptions: 'readonly',
        URL: 'readonly',
        URLSearchParams: 'readonly',
        ReadableStreamDefaultController: 'readonly',
        setInterval: 'readonly',
        TextEncoder: 'readonly',
        jest: 'readonly',
        describe: 'readonly',
        it: 'readonly',
        test: 'readonly',
        expect: 'readonly',
        beforeEach: 'readonly',
        afterEach: 'readonly',
        beforeAll: 'readonly',
        afterAll: 'readonly',
      },
    },
    settings: {
      react: {
        version: 'detect',
      },
    },
    plugins: {
      '@typescript-eslint': typescript,
      'react': react,
      'react-hooks': reactHooks,
      'jsx-a11y': jsxA11y,
    },
    rules: {
      // Code Quality
      'prefer-const': 'error',
      'no-var': 'error',
      'no-console': 'off', // Allow console in development
      'no-debugger': 'error',
      'no-duplicate-imports': 'error',
      'no-unused-expressions': 'error',
      'no-unused-vars': 'off', // Handled by TypeScript
      
      // React Hooks
      'react-hooks/exhaustive-deps': 'warn',
      'react-hooks/rules-of-hooks': 'error',
      
      // React
      'react/no-unescaped-entities': 'off',
      'react/prop-types': 'off',
      'react/react-in-jsx-scope': 'off',
      'react/jsx-key': 'error',
      'react/jsx-no-duplicate-props': 'error',
      'react/jsx-no-undef': 'error',
      
      // TypeScript
      '@typescript-eslint/no-unused-vars': ['error', { 
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_' 
      }],
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-non-null-assertion': 'warn',
      
      // Performance
      'react/jsx-no-bind': ['warn', {
        allowArrowFunctions: true,
        allowFunctions: false,
        allowBind: false,
        ignoreRefs: true
      }],
      'react/jsx-no-constructed-context-values': 'warn',
      'react/jsx-no-leaked-render': 'error',
      'react/jsx-no-useless-fragment': 'error',
      'react/no-array-index-key': 'warn',
      'react/no-danger': 'warn',
      'react/self-closing-comp': 'error',
      
      // Best Practices
      'no-alert': 'warn',
      'no-eval': 'error',
      'no-implied-eval': 'error',
      'no-new-func': 'error',
      'no-script-url': 'error',
      'prefer-arrow-callback': 'error',
      'prefer-template': 'error',
    },
  },
  {
    ignores: [
      'node_modules/',
      '.next/',
      'out/',
      'dist/',
      'build/',
      '*.config.js',
      '*.config.ts',
      'cypress/',
      'public/',
      'coverage/',
      '*.d.ts'
    ],
  },
];
