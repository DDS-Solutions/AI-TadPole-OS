/**
 * @module ViteConfig
 * Build, dev server, and test configuration for Tadpole OS frontend.
 * Includes manual vendor chunk splitting for cache efficiency
 * and Vitest integration with jsdom + v8 coverage.
 */
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

import { VitePWA } from 'vite-plugin-pwa'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    VitePWA({
      registerType: 'autoUpdate',
      injectRegister: 'auto',
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg}'],
        navigateFallbackDenylist: [/^\/v1/, /^\/api/],
        sourcemap: true
      }
    }),
  ],
  server: {
    host: '0.0.0.0',
    port: 5173,
  },
  build: {
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes('node_modules')) {
            if (id.includes('react/') || id.includes('react-dom/')) return 'vendor-react';
            if (id.includes('react-router-dom/')) return 'vendor-router';
            if (id.includes('framer-motion')) return 'vendor-motion';
            if (id.includes('reactflow/') || id.includes('dagre/')) return 'vendor-flow';
            if (
              id.includes('react-force-graph-2d') ||
              id.includes('force-graph') ||
              id.includes('/d3-') ||
              id.includes('/three/')
            ) {
              return 'vendor-graph';
            }
            if (
              id.includes('react-markdown') ||
              id.includes('/remark-') ||
              id.includes('/rehype-') ||
              id.includes('/unified/') ||
              id.includes('/micromark') ||
              id.includes('/mdast-') ||
              id.includes('/hast-')
            ) {
              return 'vendor-markdown';
            }
            if (id.includes('@google/genai') || id.includes('groq-sdk')) return 'vendor-ai';
            if (id.includes('@msgpack/msgpack') || id.includes('/lodash/')) return 'vendor-utils';
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
