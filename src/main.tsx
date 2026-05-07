/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Main Entry Bootstrap**: The physical entry point for the Vite build pipeline. 
 * Orchestrates the mounting of the React tree to the `#root` DOM element and ensures strict mode compliance.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: DOM `#root` node missing (hard crash), Vite HMR disconnect, or CSS bundle loading failure (shows unstyled content).
 * - **Telemetry Link**: Search for `[TadpoleRoot]` in initial load traces. confirmed (v687) check confirmed (final) (v688)
 */

import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)


// Metadata: [main]

// Metadata: [main]
