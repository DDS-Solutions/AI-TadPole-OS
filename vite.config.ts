/**
 * @module ViteConfig
 * Build, dev server, and test configuration for Tadpole OS frontend.
 * Includes manual vendor chunk splitting for cache efficiency
 * and Vitest integration with jsdom + v8 coverage.
 */
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
  ],
  server: {
    host: '0.0.0.0',
    port: 5173,
  },
  build: {
    chunkSizeWarningLimit: 800,
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes('node_modules')) {
              if (id.includes('react/') || id.includes('react-dom/')) return 'vendor-react';
              if (id.includes('react-router-dom/')) return 'vendor-router';
            if (id.includes('framer-motion')) return 'vendor-motion';
            if (id.includes('reactflow/') || id.includes('dagre/')) return 'vendor-flow';
            if (id.includes('lucide-react')) return 'vendor-ui';
            if (id.includes('zustand')) return 'vendor-state';
          }
        },
      },
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './tests/setup.ts',
    alias: {
      'react-dom/test-utils': 'react-dom/test-utils',
    },
    exclude: ['**/node_modules/**', '**/dist/**', '**/.tmp/**', '**/setup.ts', 'tests/e2e/**'],
    deps: {
      optimizer: {
        web: {
          include: ['react-dom', 'react-dom/client']
        }
      }
    },
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['src/**/*.ts', 'src/**/*.tsx'],
      exclude: ['src/main.tsx', 'src/vite-env.d.ts', 'tests/**']
    }
  },
})
