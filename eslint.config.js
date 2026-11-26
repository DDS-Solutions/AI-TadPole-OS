/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Assist Note
 * - Keep typed production rules enabled when mapping separate test projects and tooling fixtures.
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / eslint.config
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import js from '@eslint/js';
import globals from 'globals';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import tseslint from 'typescript-eslint';
import { defineConfig, globalIgnores } from 'eslint/config';

export default defineConfig([
  globalIgnores(['**/dist/**', 'node_modules', '.tmp', 'scratch', 'coverage', 'src-tauri', 'external', '**/target/**', 'docs/.vitepress/cache']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommended,
      reactHooks.configs.flat.recommended,
      reactRefresh.configs.vite,
    ],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
      parserOptions: {
        projectService: {
          allowDefaultProject: [
            'playwright.config.ts',
            'tools/adg/fixtures/must-fail/*.ts',
          ],
        },
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      'no-console': ['warn', { allow: ['warn', 'error', 'debug'] }],
      'no-restricted-globals': ['error', {
        name: 'i18n',
        message: 'Always import { i18n } from the project root singleton to ensure localization stability.',
      }],
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-misused-promises': ['error', {
        checksVoidReturn: { attributes: false },
      }],
      '@typescript-eslint/await-thenable': 'error',
    },
  },
  // Relax strict typing rules and console output in test files — mocks legitimately use 'any' and tests log diagnostic info
  {
    files: ['**/*.test.{ts,tsx}', '**/*.spec.{ts,tsx}', '**/tests/**/*.{ts,tsx}'],
    languageOptions: {
      parserOptions: {
        // Tests have their own project and are deliberately excluded from the app build.
        projectService: false,
        project: './tsconfig.test.json',
      },
    },
    rules: {
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-floating-promises': 'off',
      '@typescript-eslint/no-misused-promises': 'off',
      'no-console': 'off',
    },
  },
  // CLI scripts run in Node environment where console output is expected
  {
    files: ['scripts/**/*.{ts,js,cjs,mjs}'],
    languageOptions: {
      globals: globals.node,
    },
    rules: {
      'no-console': 'off',
    },
  },
]);
