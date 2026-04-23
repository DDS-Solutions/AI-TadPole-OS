/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **@module ViteConfig**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[vite_config]` in observability traces.
 */

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
    // ─── Security Headers ────────────────────────────────────────────────────
    // Injects production-grade security headers on every dev-server response.
    // These are also applied via the configureServer plugin below.
    // TODO: tighten CSP for production (remove 'unsafe-eval', 'unsafe-inline')
    //       when HMR / Vite style injection is no longer needed.
    headers: {
      'Content-Security-Policy': [
        "default-src 'self'",
        // Vite HMR requires unsafe-eval; ws: for the WebSocket HMR channel
        "script-src 'self' 'unsafe-eval' 'unsafe-inline'",
        // Vite injects <style> tags at runtime in dev mode
        "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
        // Google Fonts + data URIs for icons
        "font-src 'self' https://fonts.gstatic.com data:",
        // API, WebSocket to same host; wss: for production TLS upgrade
        "connect-src 'self' http://localhost:8000 http://127.0.0.1:8000 ws: wss:",
        // Media capture (getUserMedia for voice)
        "media-src 'self' blob:",
        // Inline SVGs and canvas used by the visualizer
        "img-src 'self' data: blob:",
        // Disallow all framing
        "frame-src 'none'",
        "frame-ancestors 'none'",
        // No plugins (Flash etc.)
        "object-src 'none'",
        "base-uri 'self'",
        "form-action 'self'",
      ].join('; '),
      'X-Frame-Options': 'DENY',
      'X-Content-Type-Options': 'nosniff',
      'Referrer-Policy': 'strict-origin-when-cross-origin',
      'Permissions-Policy': 'camera=(), microphone=(self), geolocation=()',
    },
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

// Metadata: [vite_config]
